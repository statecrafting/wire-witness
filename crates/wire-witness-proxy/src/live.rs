//! Live loopback capture runtime: listener, CONNECT routing, TLS interception,
//! and the HTTP/1.1 relay that observes intercepted exchanges.
//!
//! Governed by spec 013, live loopback capture runtime.

// region: live-capture-runtime

use std::collections::BTreeMap;
use std::io::{self, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{SyncSender, TrySendError};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime};

use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, ExtendedKeyUsagePurpose, IsCa,
    KeyPair, KeyUsagePurpose,
};
use rustls::crypto::CryptoProvider;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName};
use rustls::{
    ClientConfig, ClientConnection, RootCertStore, ServerConfig, ServerConnection, StreamOwned,
};
use sha2::{Digest, Sha256};
use wire_witness_core::exchange::{ByteReference, ProviderFamily};

use crate::{
    ApplicationProtocol, Authority, ProxyConfig, ProxyLimits, RouteDecision, parse_connect,
};

/// The only application protocol this runtime offers on either TLS side.
const HTTP1_ALPN: &[u8] = b"http/1.1";
const COPY_CHUNK: usize = 16 * 1024;
const CHUNK_LINE_LIMIT: usize = 4096;
const ACCEPT_POLL: Duration = Duration::from_millis(10);

/// Bounds for one live attempt. Every bound is finite.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LiveLimits {
    pub header_bytes: usize,
    pub header_count: usize,
    pub captured_body_bytes: usize,
    pub decoded_body_bytes: usize,
    pub connections: usize,
    pub sink_capacity: usize,
    pub idle: Duration,
    pub connect: Duration,
}

impl Default for LiveLimits {
    fn default() -> Self {
        Self {
            header_bytes: 64 * 1024,
            header_count: 256,
            captured_body_bytes: 8 * 1024 * 1024,
            decoded_body_bytes: 32 * 1024 * 1024,
            connections: 64,
            sink_capacity: 1024,
            idle: Duration::from_secs(600),
            connect: Duration::from_secs(30),
        }
    }
}

impl LiveLimits {
    fn validate(self) -> Result<Self, LiveError> {
        if self.header_bytes == 0
            || self.header_count == 0
            || self.captured_body_bytes == 0
            || self.decoded_body_bytes == 0
            || self.connections == 0
            || self.sink_capacity == 0
            || self.idle.is_zero()
            || self.connect.is_zero()
        {
            return Err(LiveError::InvalidLimits);
        }
        Ok(self)
    }

    fn connect_limits(self) -> ProxyLimits {
        ProxyLimits {
            header_bytes: self.header_bytes,
            header_count: self.header_count,
            frame_bytes: self.captured_body_bytes,
            event_bytes: self.captured_body_bytes,
            queue_bytes: self.captured_body_bytes,
        }
    }
}

/// Trust the provider-facing TLS client uses. The command line only ever
/// selects `System`; `Roots` exists for supervisors and tests that supply an
/// exact root set through the library.
#[derive(Clone, Debug)]
pub enum ProviderTrust {
    System,
    Roots(Vec<CertificateDer<'static>>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LiveError {
    InvalidLimits,
    CertificateAuthority(&'static str),
    ProviderTrustUnavailable,
    Tls(&'static str),
    Listener(io::ErrorKind),
}

impl LiveError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidLimits => "invalid-live-limits",
            Self::CertificateAuthority(code) | Self::Tls(code) => code,
            Self::ProviderTrustUnavailable => "provider-trust-unavailable",
            Self::Listener(_) => "loopback-listener-unavailable",
        }
    }
}

/// The per-attempt certificate authority. Its private key is generated from
/// operating-system randomness, lives only inside this value, and is never
/// serialized. Only the certificate leaves it.
pub struct EphemeralCa {
    key: KeyPair,
    certificate: rcgen::Certificate,
    not_before: time::OffsetDateTime,
    not_after: time::OffsetDateTime,
}

impl std::fmt::Debug for EphemeralCa {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EphemeralCa")
            .finish_non_exhaustive()
    }
}

impl EphemeralCa {
    /// `issued_at` is the host's clock reading; this crate reads no clock.
    pub fn generate(issued_at: SystemTime) -> Result<Self, LiveError> {
        let issued_at = time::OffsetDateTime::from(issued_at);
        let not_before = issued_at - time::Duration::hours(1);
        let not_after = issued_at + time::Duration::days(7);
        let key = KeyPair::generate()
            .map_err(|_| LiveError::CertificateAuthority("ca-key-generation-failed"))?;
        let mut params = CertificateParams::default();
        params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
        params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
            KeyUsagePurpose::DigitalSignature,
        ];
        let mut name = DistinguishedName::new();
        name.push(DnType::CommonName, "wire-witness ephemeral attempt CA");
        params.distinguished_name = name;
        params.not_before = not_before;
        params.not_after = not_after;
        let certificate = params
            .self_signed(&key)
            .map_err(|_| LiveError::CertificateAuthority("ca-certificate-failed"))?;
        Ok(Self {
            key,
            certificate,
            not_before,
            not_after,
        })
    }

    #[must_use]
    pub fn certificate_pem(&self) -> String {
        self.certificate.pem()
    }

    #[must_use]
    pub fn certificate_der(&self) -> CertificateDer<'static> {
        self.certificate.der().clone()
    }

    fn issue_leaf(
        &self,
        host: &str,
    ) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), LiveError> {
        let key = KeyPair::generate()
            .map_err(|_| LiveError::CertificateAuthority("leaf-key-generation-failed"))?;
        let mut params = CertificateParams::new(vec![host.to_owned()])
            .map_err(|_| LiveError::CertificateAuthority("leaf-name-invalid"))?;
        params.is_ca = IsCa::ExplicitNoCa;
        params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
        params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
        params.use_authority_key_identifier_extension = true;
        let mut name = DistinguishedName::new();
        name.push(DnType::CommonName, host);
        params.distinguished_name = name;
        params.not_before = self.not_before;
        params.not_after = self.not_after;
        let leaf = params
            .signed_by(&key, &self.certificate, &self.key)
            .map_err(|_| LiveError::CertificateAuthority("leaf-certificate-failed"))?;
        let private = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key.serialize_der()));
        Ok((vec![leaf.der().clone(), self.certificate_der()], private))
    }
}

/// PEM text for the operating system's ordinary roots, for a runtime whose
/// trust variable replaces rather than extends them (spec 006, section 3.3).
pub fn system_roots_pem() -> Result<String, LiveError> {
    let loaded = rustls_native_certs::load_native_certs();
    if loaded.certs.is_empty() {
        return Err(LiveError::ProviderTrustUnavailable);
    }
    let mut output = String::new();
    for certificate in &loaded.certs {
        output.push_str(&pem_certificate(certificate.as_ref()));
    }
    Ok(output)
}

fn pem_certificate(der: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = Vec::with_capacity(der.len().div_ceil(3) * 4);
    for chunk in der.chunks(3) {
        let bytes = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let triple = (u32::from(bytes[0]) << 16) | (u32::from(bytes[1]) << 8) | u32::from(bytes[2]);
        for index in 0..4 {
            if index <= chunk.len() {
                encoded.push(ALPHABET[((triple >> (18 - 6 * index)) & 0x3f) as usize]);
            } else {
                encoded.push(b'=');
            }
        }
    }
    let mut output = String::from("-----BEGIN CERTIFICATE-----\n");
    for line in encoded.chunks(64) {
        output.push_str(std::str::from_utf8(line).expect("base64 alphabet is ASCII"));
        output.push('\n');
    }
    output.push_str("-----END CERTIFICATE-----\n");
    output
}

/// A body as the capture tap saw it, after transfer decoding and any content
/// decoding, or the closed reason it is not available.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CapturedBody {
    Observed(Vec<u8>),
    Unavailable(&'static str),
}

/// How the exchange's transport ended.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExchangeEnd {
    Complete,
    RequestTruncated,
    ResponseMissing,
    ResponseTruncated,
}

impl ExchangeEnd {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::RequestTruncated => "request-truncated",
            Self::ResponseMissing => "response-missing",
            Self::ResponseTruncated => "response-truncated",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveFinding {
    pub code: &'static str,
    pub direction: &'static str,
    pub authority: Option<String>,
    pub detail: String,
}

/// One intercepted request and response. Headers are the observed values and
/// may carry credentials: the receiving host redacts before any durable write.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservedExchange {
    pub authority: Authority,
    pub provider: ProviderFamily,
    pub protocol: &'static str,
    pub method: String,
    pub target: String,
    pub status: Option<u16>,
    pub request_headers: Vec<(String, String)>,
    pub response_headers: Vec<(String, String)>,
    pub request_wire: ByteReference,
    pub response_wire: ByteReference,
    pub request_body: CapturedBody,
    pub response_body: CapturedBody,
    pub end: ExchangeEnd,
    pub findings: Vec<LiveFinding>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LiveCounters {
    pub intercepted_connections: u64,
    pub authentication_tunnels: u64,
    pub unintercepted_tunnels: u64,
    pub unintercepted_plain_requests: u64,
    pub refused_connections: u64,
    pub dropped_exchanges: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LiveSummary {
    pub counters: LiveCounters,
    pub findings: Vec<LiveFinding>,
    pub open_connections: usize,
}

#[derive(Default)]
struct Counters {
    intercepted_connections: AtomicU64,
    authentication_tunnels: AtomicU64,
    unintercepted_tunnels: AtomicU64,
    unintercepted_plain_requests: AtomicU64,
    refused_connections: AtomicU64,
    dropped_exchanges: AtomicU64,
}

impl Counters {
    fn snapshot(&self) -> LiveCounters {
        LiveCounters {
            intercepted_connections: self.intercepted_connections.load(Ordering::SeqCst),
            authentication_tunnels: self.authentication_tunnels.load(Ordering::SeqCst),
            unintercepted_tunnels: self.unintercepted_tunnels.load(Ordering::SeqCst),
            unintercepted_plain_requests: self.unintercepted_plain_requests.load(Ordering::SeqCst),
            refused_connections: self.refused_connections.load(Ordering::SeqCst),
            dropped_exchanges: self.dropped_exchanges.load(Ordering::SeqCst),
        }
    }
}

struct Shared {
    config: ProxyConfig,
    limits: LiveLimits,
    ca: EphemeralCa,
    crypto: Arc<CryptoProvider>,
    client: Arc<ClientConfig>,
    servers: Mutex<BTreeMap<String, Arc<ServerConfig>>>,
    sink: SyncSender<ObservedExchange>,
    stop: AtomicBool,
    active: AtomicUsize,
    counters: Counters,
    findings: Mutex<Vec<LiveFinding>>,
}

impl Shared {
    fn finding(
        &self,
        code: &'static str,
        direction: &'static str,
        authority: Option<&Authority>,
        detail: impl Into<String>,
    ) {
        self.findings
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(LiveFinding {
                code,
                direction,
                authority: authority.map(Authority::canonical),
                detail: detail.into(),
            });
    }

    fn server_config(&self, host: &str) -> Result<Arc<ServerConfig>, LiveError> {
        let mut servers = self.servers.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(config) = servers.get(host) {
            return Ok(Arc::clone(config));
        }
        let (chain, key) = self.ca.issue_leaf(host)?;
        let mut config = ServerConfig::builder_with_provider(Arc::clone(&self.crypto))
            .with_safe_default_protocol_versions()
            .map_err(|_| LiveError::Tls("server-protocol-versions"))?
            .with_no_client_auth()
            .with_single_cert(chain, key)
            .map_err(|_| LiveError::Tls("server-certificate-rejected"))?;
        config.alpn_protocols = vec![HTTP1_ALPN.to_vec()];
        let config = Arc::new(config);
        servers.insert(host.to_owned(), Arc::clone(&config));
        Ok(config)
    }

    fn emit(&self, exchange: ObservedExchange) {
        match self.sink.try_send(exchange) {
            Ok(()) => {}
            Err(TrySendError::Full(_) | TrySendError::Disconnected(_)) => {
                self.counters
                    .dropped_exchanges
                    .fetch_add(1, Ordering::SeqCst);
            }
        }
    }
}

/// A running per-attempt proxy bound to an ephemeral loopback port.
pub struct LiveProxy {
    local: SocketAddr,
    shared: Arc<Shared>,
    accept: Option<JoinHandle<()>>,
}

impl LiveProxy {
    pub fn start(
        config: ProxyConfig,
        ca: EphemeralCa,
        trust: ProviderTrust,
        limits: LiveLimits,
        sink: SyncSender<ObservedExchange>,
    ) -> Result<Self, LiveError> {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .map_err(|error| LiveError::Listener(error.kind()))?;
        Self::start_on(listener, config, ca, trust, limits, sink)
    }

    /// Starts the proxy on a listener the caller already bound, so a
    /// supervisor can fix the endpoint before the attempt begins. Only a
    /// loopback listener is accepted.
    pub fn start_on(
        listener: TcpListener,
        config: ProxyConfig,
        ca: EphemeralCa,
        trust: ProviderTrust,
        limits: LiveLimits,
        sink: SyncSender<ObservedExchange>,
    ) -> Result<Self, LiveError> {
        let limits = limits.validate()?;
        let bound = listener
            .local_addr()
            .map_err(|error| LiveError::Listener(error.kind()))?;
        if !bound.ip().is_loopback() {
            return Err(LiveError::Listener(io::ErrorKind::AddrNotAvailable));
        }
        let crypto = Arc::new(rustls::crypto::ring::default_provider());
        let client = client_config(&crypto, trust)?;
        listener
            .set_nonblocking(true)
            .map_err(|error| LiveError::Listener(error.kind()))?;
        let local = listener
            .local_addr()
            .map_err(|error| LiveError::Listener(error.kind()))?;
        let shared = Arc::new(Shared {
            config,
            limits,
            ca,
            crypto,
            client,
            servers: Mutex::new(BTreeMap::new()),
            sink,
            stop: AtomicBool::new(false),
            active: AtomicUsize::new(0),
            counters: Counters::default(),
            findings: Mutex::new(Vec::new()),
        });
        let accept_shared = Arc::clone(&shared);
        let accept = thread::Builder::new()
            .name("wire-witness-accept".into())
            .spawn(move || accept_loop(&listener, &accept_shared))
            .map_err(|error| LiveError::Listener(error.kind()))?;
        Ok(Self {
            local,
            shared,
            accept: Some(accept),
        })
    }

    /// The loopback endpoint the child is told to use.
    #[must_use]
    pub fn authority(&self) -> Authority {
        Authority::parse(&self.local.to_string())
            .expect("a bound IPv4 loopback address is an authority")
    }

    /// The bound loopback socket address.
    #[must_use]
    pub const fn local_address(&self) -> SocketAddr {
        self.local
    }

    /// Findings recorded so far, from position `start` on, in recorded
    /// order. A caller that remembers how many it has read sees each once.
    #[must_use]
    pub fn findings_since(&self, start: usize) -> Vec<LiveFinding> {
        self.shared
            .findings
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(start..)
            .map_or_else(Vec::new, <[LiveFinding]>::to_vec)
    }

    /// Stops accepting, waits up to `grace` for open connections to end, and
    /// reports what was observed. Connections still open are counted, never
    /// presented as closed.
    #[must_use]
    pub fn stop(mut self, grace: Duration) -> LiveSummary {
        self.shared.stop.store(true, Ordering::SeqCst);
        if let Some(accept) = self.accept.take() {
            let _ = accept.join();
        }
        let deadline = Instant::now() + grace;
        while self.shared.active.load(Ordering::SeqCst) > 0 && Instant::now() < deadline {
            thread::sleep(ACCEPT_POLL);
        }
        LiveSummary {
            counters: self.shared.counters.snapshot(),
            findings: self
                .shared
                .findings
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .clone(),
            open_connections: self.shared.active.load(Ordering::SeqCst),
        }
    }
}

impl Drop for LiveProxy {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        if let Some(accept) = self.accept.take() {
            let _ = accept.join();
        }
    }
}

fn client_config(
    crypto: &Arc<CryptoProvider>,
    trust: ProviderTrust,
) -> Result<Arc<ClientConfig>, LiveError> {
    let mut roots = RootCertStore::empty();
    match trust {
        ProviderTrust::System => {
            let loaded = rustls_native_certs::load_native_certs();
            roots.add_parsable_certificates(loaded.certs);
        }
        ProviderTrust::Roots(certificates) => {
            roots.add_parsable_certificates(certificates);
        }
    }
    if roots.is_empty() {
        return Err(LiveError::ProviderTrustUnavailable);
    }
    let mut config = ClientConfig::builder_with_provider(Arc::clone(crypto))
        .with_safe_default_protocol_versions()
        .map_err(|_| LiveError::Tls("client-protocol-versions"))?
        .with_root_certificates(roots)
        .with_no_client_auth();
    config.alpn_protocols = vec![HTTP1_ALPN.to_vec()];
    Ok(Arc::new(config))
}

struct ActiveGuard(Arc<Shared>);

impl Drop for ActiveGuard {
    fn drop(&mut self) {
        self.0.active.fetch_sub(1, Ordering::SeqCst);
    }
}

fn accept_loop(listener: &TcpListener, shared: &Arc<Shared>) {
    while !shared.stop.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, peer)) => {
                if !peer.ip().is_loopback()
                    || shared.active.load(Ordering::SeqCst) >= shared.limits.connections
                    || stream.set_nonblocking(false).is_err()
                {
                    shared
                        .counters
                        .refused_connections
                        .fetch_add(1, Ordering::SeqCst);
                    let _ = stream.shutdown(Shutdown::Both);
                    continue;
                }
                shared.active.fetch_add(1, Ordering::SeqCst);
                let guard = ActiveGuard(Arc::clone(shared));
                let spawned = thread::Builder::new()
                    .name("wire-witness-connection".into())
                    .spawn(move || {
                        let guard = guard;
                        handle_connection(stream, &guard.0);
                    });
                if spawned.is_err() {
                    shared
                        .counters
                        .refused_connections
                        .fetch_add(1, Ordering::SeqCst);
                }
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => thread::sleep(ACCEPT_POLL),
            Err(_) => thread::sleep(ACCEPT_POLL),
        }
    }
}

fn handle_connection(stream: TcpStream, shared: &Arc<Shared>) {
    let idle = Some(shared.limits.idle);
    if stream.set_read_timeout(idle).is_err() || stream.set_write_timeout(idle).is_err() {
        return;
    }
    let mut child = Wire::new(stream);
    let head = match child.read_head(shared.limits.header_bytes) {
        Ok(Some(head)) => head,
        Ok(None) => return,
        Err(error) => {
            shared.finding(error.code(), "request", None, "client head before routing");
            return;
        }
    };
    if head.starts_with(b"CONNECT ") {
        handle_connect(child, &head, shared);
    } else {
        handle_plain(child, head, shared);
    }
}

fn handle_connect(mut child: Wire<TcpStream>, head: &[u8], shared: &Arc<Shared>) {
    let request = match parse_connect(head, shared.limits.connect_limits()) {
        Ok(request) => request,
        Err(_) => {
            shared.finding("malformed-connect", "request", None, "CONNECT head refused");
            let _ = child.io.write_all(
                b"HTTP/1.1 400 Bad Request\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",
            );
            return;
        }
    };
    let authority = request.authority;
    match shared.config.route(&authority) {
        RouteDecision::AuthenticationTunnel => {
            shared
                .counters
                .authentication_tunnels
                .fetch_add(1, Ordering::SeqCst);
            tunnel(child, &authority, shared);
        }
        RouteDecision::OpaqueTunnel => {
            shared
                .counters
                .unintercepted_tunnels
                .fetch_add(1, Ordering::SeqCst);
            tunnel(child, &authority, shared);
        }
        RouteDecision::Intercept {
            provider,
            protocols,
        } => {
            shared
                .counters
                .intercepted_connections
                .fetch_add(1, Ordering::SeqCst);
            if !protocols.contains(&ApplicationProtocol::Http1)
                && !protocols.contains(&ApplicationProtocol::Sse)
            {
                shared.finding(
                    "protocol-unsupported-by-runtime",
                    "request",
                    Some(&authority),
                    "the authority allows no protocol this runtime relays",
                );
                let _ = child.io.write_all(
                    b"HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",
                );
                return;
            }
            intercept(child, &authority, provider, shared);
        }
    }
}

fn connect_upstream(authority: &Authority, limits: LiveLimits) -> io::Result<TcpStream> {
    let host = authority.host().trim_matches(['[', ']']);
    let mut last = io::Error::from(io::ErrorKind::AddrNotAvailable);
    for address in (host, authority.port()).to_socket_addrs()? {
        match TcpStream::connect_timeout(&address, limits.connect) {
            Ok(stream) => return Ok(stream),
            Err(error) => last = error,
        }
    }
    Err(last)
}

fn tunnel(mut child: Wire<TcpStream>, authority: &Authority, shared: &Arc<Shared>) {
    let upstream = match connect_upstream(authority, shared.limits) {
        Ok(stream) => stream,
        Err(error) => {
            shared.finding(
                "upstream-unreachable",
                "request",
                Some(authority),
                error.kind().to_string(),
            );
            let _ = child.io.write_all(
                b"HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",
            );
            return;
        }
    };
    if child
        .io
        .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
        .is_err()
    {
        return;
    }
    // A tunnel may idle in one direction while the other streams, so it has
    // no per-direction read timeout; it ends when either side closes.
    let _ = child.io.set_read_timeout(None);
    let _ = upstream.set_read_timeout(None);
    let leftover = child.take_buffered();
    let (Ok(child_reader), Ok(mut upstream_writer)) = (child.io.try_clone(), upstream.try_clone())
    else {
        return;
    };
    if !leftover.is_empty() && upstream_writer.write_all(&leftover).is_err() {
        return;
    }
    let forward = thread::spawn(move || pipe(child_reader, upstream_writer));
    pipe(upstream, child.io);
    let _ = forward.join();
}

fn pipe(mut from: TcpStream, mut to: TcpStream) {
    let mut buffer = vec![0_u8; COPY_CHUNK];
    loop {
        match from.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(read) => {
                if to.write_all(&buffer[..read]).is_err() {
                    break;
                }
            }
        }
    }
    let _ = to.shutdown(Shutdown::Write);
}

fn intercept(
    mut child: Wire<TcpStream>,
    authority: &Authority,
    provider: ProviderFamily,
    shared: &Arc<Shared>,
) {
    let host = authority.host().trim_matches(['[', ']']).to_owned();
    let server_config = match shared.server_config(&host) {
        Ok(config) => config,
        Err(error) => {
            shared.finding(
                error.code(),
                "response",
                Some(authority),
                "leaf certificate",
            );
            let _ = child.io.write_all(
                b"HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",
            );
            return;
        }
    };
    let Ok(server_name) = ServerName::try_from(host.clone()) else {
        shared.finding(
            "provider-name-invalid",
            "request",
            Some(authority),
            "not a TLS server name",
        );
        let _ = child.io.write_all(
            b"HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",
        );
        return;
    };
    if child
        .io
        .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
        .is_err()
    {
        return;
    }
    let Ok(mut server) = ServerConnection::new(server_config) else {
        return;
    };
    let mut child_socket = Prefixed::new(child.take_buffered(), child.io);
    if let Err(error) = handshake(&mut *server, &mut child_socket) {
        shared.finding(
            "child-tls-handshake-failed",
            "request",
            Some(authority),
            error.kind().to_string(),
        );
        return;
    }
    let sni = server.server_name().unwrap_or_default().to_owned();
    if shared
        .config
        .validate_interception_identity(authority, &sni, &host)
        .is_err()
    {
        shared.finding(
            "tls-identity-mismatch",
            "request",
            Some(authority),
            "CONNECT authority and SNI disagree",
        );
        server.send_close_notify();
        let _ = server.complete_io(&mut child_socket);
        return;
    }
    let upstream = match connect_upstream(authority, shared.limits) {
        Ok(stream) => stream,
        Err(error) => {
            shared.finding(
                "upstream-unreachable",
                "request",
                Some(authority),
                error.kind().to_string(),
            );
            return;
        }
    };
    let idle = Some(shared.limits.idle);
    if upstream.set_read_timeout(idle).is_err() || upstream.set_write_timeout(idle).is_err() {
        return;
    }
    let Ok(mut client) = ClientConnection::new(Arc::clone(&shared.client), server_name) else {
        return;
    };
    let mut upstream = upstream;
    if let Err(error) = handshake(&mut *client, &mut upstream) {
        shared.finding(
            "provider-tls-failed",
            "response",
            Some(authority),
            error.kind().to_string(),
        );
        server.send_close_notify();
        let _ = server.complete_io(&mut child_socket);
        return;
    }
    let mut child = Wire::new(StreamOwned::new(server, child_socket));
    let mut provider_side = Wire::new(StreamOwned::new(client, upstream));
    let context = CaptureContext {
        authority: authority.clone(),
        provider,
    };
    relay(&mut child, &mut provider_side, Some(&context), shared);
    child.io.conn.send_close_notify();
    let _ = child.io.conn.complete_io(&mut child.io.sock);
    provider_side.io.conn.send_close_notify();
    let _ = provider_side
        .io
        .conn
        .complete_io(&mut provider_side.io.sock);
}

fn handshake<D, S>(connection: &mut rustls::ConnectionCommon<D>, socket: &mut S) -> io::Result<()>
where
    S: Read + Write,
{
    while connection.is_handshaking() {
        connection.complete_io(socket)?;
    }
    Ok(())
}

/// Plain-HTTP proxy requests are forwarded without capture: the head and body
/// are framed only to find message boundaries and the next origin.
fn handle_plain(mut child: Wire<TcpStream>, first: Vec<u8>, shared: &Arc<Shared>) {
    let mut pending = Some(first);
    let mut upstream: Option<(Authority, Wire<TcpStream>)> = None;
    loop {
        let raw = match pending.take() {
            Some(raw) => raw,
            None => match child.read_head(shared.limits.header_bytes) {
                Ok(Some(raw)) => raw,
                Ok(None) => return,
                Err(error) => {
                    shared.finding(error.code(), "request", None, "plain request head");
                    return;
                }
            },
        };
        let head = match Head::parse(raw, shared.limits.header_count) {
            Ok(head) => head,
            Err(error) => {
                shared.finding(error.code(), "request", None, "plain request head");
                let _ = child.io.write_all(
                    b"HTTP/1.1 400 Bad Request\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",
                );
                return;
            }
        };
        let Some(authority) = absolute_form_authority(head.target()) else {
            shared.finding(
                "plain-target-not-absolute",
                "request",
                None,
                "plain proxy request target",
            );
            let _ = child.io.write_all(
                b"HTTP/1.1 400 Bad Request\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",
            );
            return;
        };
        shared
            .counters
            .unintercepted_plain_requests
            .fetch_add(1, Ordering::SeqCst);
        if upstream
            .as_ref()
            .is_none_or(|(current, _)| *current != authority)
        {
            match connect_upstream(&authority, shared.limits) {
                Ok(stream) => {
                    let idle = Some(shared.limits.idle);
                    let _ = stream.set_read_timeout(idle);
                    let _ = stream.set_write_timeout(idle);
                    upstream = Some((authority.clone(), Wire::new(stream)));
                }
                Err(error) => {
                    shared.finding(
                        "upstream-unreachable",
                        "request",
                        Some(&authority),
                        error.kind().to_string(),
                    );
                    let _ = child.io.write_all(b"HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\nContent-Length: 0\r\n\r\n");
                    return;
                }
            }
        }
        let (_, origin) = upstream.as_mut().expect("an upstream was just ensured");
        match exchange(&mut child, origin, head, None, shared.limits) {
            Ok(outcome) if outcome.keep_alive => {}
            Ok(_) => return,
            Err(error) => {
                shared.finding(error.code(), "response", Some(&authority), "plain exchange");
                return;
            }
        }
    }
}

fn absolute_form_authority(target: &str) -> Option<Authority> {
    let rest = target.strip_prefix("http://")?;
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..end];
    if authority.contains(':') && !authority.ends_with(']') {
        Authority::parse(authority).ok()
    } else {
        Authority::parse(&format!("{authority}:80")).ok()
    }
}

struct CaptureContext {
    authority: Authority,
    provider: ProviderFamily,
}

fn relay<C, U>(
    child: &mut Wire<C>,
    provider: &mut Wire<U>,
    context: Option<&CaptureContext>,
    shared: &Arc<Shared>,
) where
    C: Read + Write,
    U: Read + Write,
{
    loop {
        let raw = match child.read_head(shared.limits.header_bytes) {
            Ok(Some(raw)) => raw,
            Ok(None) => return,
            Err(error) => {
                shared.finding(
                    error.code(),
                    "request",
                    context.map(|c| &c.authority),
                    "intercepted request head",
                );
                return;
            }
        };
        let head = match Head::parse(raw, shared.limits.header_count) {
            Ok(head) => head,
            Err(error) => {
                shared.finding(
                    error.code(),
                    "request",
                    context.map(|c| &c.authority),
                    "intercepted request head",
                );
                return;
            }
        };
        if head.header("upgrade").is_some() {
            shared.finding(
                "upgrade-capture-unsupported",
                "request",
                context.map(|c| &c.authority),
                "a protocol upgrade on a capture authority is refused before forwarding",
            );
            let _ = child.io.write_all(
                b"HTTP/1.1 501 Not Implemented\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",
            );
            let _ = child.io.flush();
            return;
        }
        match exchange(child, provider, head, context, shared.limits) {
            Ok(outcome) => {
                if let Some(observed) = outcome.observed {
                    shared.emit(observed);
                }
                if !outcome.keep_alive {
                    return;
                }
            }
            Err(error) => {
                shared.finding(
                    error.code(),
                    "response",
                    context.map(|c| &c.authority),
                    "intercepted exchange",
                );
                return;
            }
        }
    }
}

struct ExchangeOutcome {
    keep_alive: bool,
    observed: Option<ObservedExchange>,
}

/// Forwards one request and its response verbatim. With a capture context,
/// both directions are tapped: wire bytes are hashed in full and bodies are
/// captured up to the limit. Forwarding never waits on capture.
fn exchange<C, U>(
    child: &mut Wire<C>,
    provider: &mut Wire<U>,
    request: Head,
    context: Option<&CaptureContext>,
    limits: LiveLimits,
) -> Result<ExchangeOutcome, RelayError>
where
    C: Read + Write,
    U: Read + Write,
{
    let capture = context.is_some();
    let mut request_tap = Tap::new(capture, limits.captured_body_bytes);
    let mut response_tap = Tap::new(capture, limits.captured_body_bytes);
    let request_framing = request.request_framing()?;
    request_tap.wire(&request.raw);
    provider
        .io
        .write_all(&request.raw)
        .map_err(RelayError::io)?;
    let request_end = forward_body(child, &mut provider.io, request_framing, &mut request_tap)?;
    provider.io.flush().map_err(RelayError::io)?;
    let mut findings = Vec::new();
    let method = request.method().to_owned();
    if request_end == BodyEnd::Truncated {
        return Ok(ExchangeOutcome {
            keep_alive: false,
            observed: context.map(|context| {
                observed(
                    context,
                    &request,
                    None,
                    request_tap,
                    response_tap,
                    ExchangeEnd::RequestTruncated,
                    findings,
                    limits,
                )
            }),
        });
    }
    let response = loop {
        let raw = match provider.read_head(limits.header_bytes) {
            Ok(Some(raw)) => raw,
            Ok(None) => {
                return Ok(ExchangeOutcome {
                    keep_alive: false,
                    observed: context.map(|context| {
                        observed(
                            context,
                            &request,
                            None,
                            request_tap,
                            response_tap,
                            ExchangeEnd::ResponseMissing,
                            findings,
                            limits,
                        )
                    }),
                });
            }
            Err(error) => return Err(error),
        };
        let head = Head::parse(raw, limits.header_count)?;
        let status = head.status()?;
        response_tap.wire(&head.raw);
        child.io.write_all(&head.raw).map_err(RelayError::io)?;
        child.io.flush().map_err(RelayError::io)?;
        if status == 101 {
            return Err(RelayError::Ambiguous("unrequested-switching-protocols"));
        }
        if (100..200).contains(&status) {
            continue;
        }
        break head;
    };
    let status = response.status()?;
    let framing = response.response_framing(&method, status)?;
    let response_end = forward_body(provider, &mut child.io, framing, &mut response_tap)?;
    child.io.flush().map_err(RelayError::io)?;
    if response_tap.overflow {
        findings.push(("capture-limit-exceeded", "response"));
    }
    if request_tap.overflow {
        findings.push(("capture-limit-exceeded", "request"));
    }
    let end = if response_end == BodyEnd::Truncated {
        ExchangeEnd::ResponseTruncated
    } else {
        ExchangeEnd::Complete
    };
    let keep_alive = end == ExchangeEnd::Complete
        && framing != Framing::UntilClose
        && request.keeps_alive()
        && response.keeps_alive();
    Ok(ExchangeOutcome {
        keep_alive,
        observed: context.map(|context| {
            observed(
                context,
                &request,
                Some(&response),
                request_tap,
                response_tap,
                end,
                findings,
                limits,
            )
        }),
    })
}

#[allow(clippy::too_many_arguments)]
fn observed(
    context: &CaptureContext,
    request: &Head,
    response: Option<&Head>,
    request_tap: Tap,
    response_tap: Tap,
    end: ExchangeEnd,
    findings: Vec<(&'static str, &'static str)>,
    limits: LiveLimits,
) -> ObservedExchange {
    let request_body = request_tap.body(&request.headers, limits.decoded_body_bytes);
    let response_body = response
        .map_or(CapturedBody::Unavailable("response-not-observed"), |head| {
            response_tap.body(&head.headers, limits.decoded_body_bytes)
        });
    ObservedExchange {
        authority: context.authority.clone(),
        provider: context.provider.clone(),
        protocol: "http/1.1",
        method: request.method().to_owned(),
        target: request.target().to_owned(),
        status: response.and_then(|head| head.status().ok()),
        request_headers: request.headers.clone(),
        response_headers: response
            .map(|head| head.headers.clone())
            .unwrap_or_default(),
        request_wire: request_tap.reference(),
        response_wire: response_tap.reference(),
        request_body,
        response_body,
        end,
        findings: findings
            .into_iter()
            .map(|(code, direction)| LiveFinding {
                code,
                direction,
                authority: Some(context.authority.canonical()),
                detail: "structured capture stopped; forwarding continued".into(),
            })
            .collect(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Framing {
    None,
    Length(u64),
    Chunked,
    UntilClose,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BodyEnd {
    Complete,
    Truncated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RelayError {
    Io(io::ErrorKind),
    Limit(&'static str),
    Ambiguous(&'static str),
}

impl RelayError {
    fn io(error: io::Error) -> Self {
        Self::Io(error.kind())
    }

    const fn code(self) -> &'static str {
        match self {
            Self::Io(io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut) => "idle-timeout",
            Self::Io(_) => "connection-failed",
            Self::Limit(code) | Self::Ambiguous(code) => code,
        }
    }
}

struct Head {
    raw: Vec<u8>,
    start: Vec<String>,
    headers: Vec<(String, String)>,
}

impl Head {
    fn parse(raw: Vec<u8>, count_limit: usize) -> Result<Self, RelayError> {
        let text = std::str::from_utf8(&raw).map_err(|_| RelayError::Ambiguous("non-utf8-head"))?;
        let text = text
            .strip_suffix("\r\n\r\n")
            .ok_or(RelayError::Ambiguous("unterminated-head"))?;
        let mut lines = text.split("\r\n");
        let start: Vec<String> = lines
            .next()
            .unwrap_or_default()
            .splitn(3, ' ')
            .map(str::to_owned)
            .collect();
        if start.len() < 2 {
            return Err(RelayError::Ambiguous("malformed-start-line"));
        }
        let mut headers = Vec::new();
        for line in lines {
            if headers.len() >= count_limit {
                return Err(RelayError::Limit("header-count-limit"));
            }
            if line.starts_with([' ', '\t']) {
                return Err(RelayError::Ambiguous("obsolete-header-folding"));
            }
            let (name, value) = line
                .split_once(':')
                .ok_or(RelayError::Ambiguous("malformed-header"))?;
            if name.is_empty() || name.bytes().any(|byte| byte.is_ascii_whitespace()) {
                return Err(RelayError::Ambiguous("malformed-header"));
            }
            headers.push((name.to_ascii_lowercase(), value.trim().to_owned()));
        }
        Ok(Self {
            raw,
            start,
            headers,
        })
    }

    fn method(&self) -> &str {
        &self.start[0]
    }

    fn target(&self) -> &str {
        &self.start[1]
    }

    fn version(&self, request: bool) -> &str {
        if request {
            self.start.get(2).map_or("", String::as_str)
        } else {
            &self.start[0]
        }
    }

    fn status(&self) -> Result<u16, RelayError> {
        if !self.start[0].starts_with("HTTP/1.") {
            return Err(RelayError::Ambiguous("malformed-status-line"));
        }
        self.start[1]
            .parse::<u16>()
            .ok()
            .filter(|status| (100..1000).contains(status))
            .ok_or(RelayError::Ambiguous("malformed-status-line"))
    }

    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(candidate, _)| candidate == name)
            .map(|(_, value)| value.as_str())
    }

    fn tokens(&self, name: &str) -> Vec<String> {
        self.headers
            .iter()
            .filter(|(candidate, _)| candidate == name)
            .flat_map(|(_, value)| value.split(','))
            .map(|token| token.trim().to_ascii_lowercase())
            .filter(|token| !token.is_empty())
            .collect()
    }

    fn content_length(&self) -> Result<Option<u64>, RelayError> {
        let mut length = None;
        for (name, value) in &self.headers {
            if name != "content-length" {
                continue;
            }
            for part in value.split(',') {
                let parsed = part
                    .trim()
                    .parse::<u64>()
                    .map_err(|_| RelayError::Ambiguous("invalid-content-length"))?;
                if length.is_some_and(|existing| existing != parsed) {
                    return Err(RelayError::Ambiguous("conflicting-content-length"));
                }
                length = Some(parsed);
            }
        }
        Ok(length)
    }

    fn request_framing(&self) -> Result<Framing, RelayError> {
        if !self.version(true).starts_with("HTTP/1.") {
            return Err(RelayError::Ambiguous("unsupported-http-version"));
        }
        let encodings = self.tokens("transfer-encoding");
        let length = self.content_length()?;
        match (encodings.is_empty(), length) {
            (false, Some(_)) => Err(RelayError::Ambiguous("length-and-transfer-encoding")),
            (false, None) if encodings == ["chunked"] => Ok(Framing::Chunked),
            (false, None) => Err(RelayError::Ambiguous("unsupported-transfer-encoding")),
            (true, Some(0) | None) => Ok(Framing::None),
            (true, Some(length)) => Ok(Framing::Length(length)),
        }
    }

    fn response_framing(&self, method: &str, status: u16) -> Result<Framing, RelayError> {
        if method == "HEAD" || (100..200).contains(&status) || status == 204 || status == 304 {
            return Ok(Framing::None);
        }
        let encodings = self.tokens("transfer-encoding");
        let length = self.content_length()?;
        match (encodings.is_empty(), length) {
            (false, Some(_)) => Err(RelayError::Ambiguous("length-and-transfer-encoding")),
            (false, None) if encodings.last().map(String::as_str) == Some("chunked") => {
                if encodings.len() == 1 {
                    Ok(Framing::Chunked)
                } else {
                    Err(RelayError::Ambiguous("unsupported-transfer-encoding"))
                }
            }
            (false, None) => Ok(Framing::UntilClose),
            (true, Some(0)) => Ok(Framing::None),
            (true, Some(length)) => Ok(Framing::Length(length)),
            (true, None) => Ok(Framing::UntilClose),
        }
    }

    fn keeps_alive(&self) -> bool {
        let connection = self.tokens("connection");
        if connection.iter().any(|token| token == "close") {
            return false;
        }
        let version = if self.start[0].starts_with("HTTP/") {
            self.version(false)
        } else {
            self.version(true)
        };
        version != "HTTP/1.0" || connection.iter().any(|token| token == "keep-alive")
    }
}

/// Observes one direction: the SHA-256 and length of every wire byte, and the
/// transfer-decoded body up to a fixed limit.
struct Tap {
    capture: bool,
    hasher: Sha256,
    length: u64,
    body: Vec<u8>,
    limit: usize,
    overflow: bool,
}

impl Tap {
    fn new(capture: bool, limit: usize) -> Self {
        Self {
            capture,
            hasher: Sha256::new(),
            length: 0,
            body: Vec::new(),
            limit,
            overflow: false,
        }
    }

    fn wire(&mut self, bytes: &[u8]) {
        if self.capture {
            self.hasher.update(bytes);
            self.length += bytes.len() as u64;
        }
    }

    fn data(&mut self, bytes: &[u8]) {
        if !self.capture || self.overflow {
            return;
        }
        if self.body.len() + bytes.len() > self.limit {
            self.overflow = true;
            self.body = Vec::new();
        } else {
            self.body.extend_from_slice(bytes);
        }
    }

    fn reference(&self) -> ByteReference {
        let digest = self.hasher.clone().finalize();
        let mut sha256 = String::with_capacity(64);
        for byte in digest {
            use std::fmt::Write as _;
            write!(sha256, "{byte:02x}").expect("writing to a string cannot fail");
        }
        ByteReference {
            original_length: self.length,
            sha256,
        }
    }

    fn body(&self, headers: &[(String, String)], decoded_limit: usize) -> CapturedBody {
        if self.overflow {
            return CapturedBody::Unavailable("capture-limit-exceeded");
        }
        let encodings: Vec<String> = headers
            .iter()
            .filter(|(name, _)| name == "content-encoding")
            .flat_map(|(_, value)| value.split(','))
            .map(|token| token.trim().to_ascii_lowercase())
            .filter(|token| !token.is_empty() && token != "identity")
            .collect();
        let mut body = self.body.clone();
        for encoding in encodings.iter().rev() {
            body = match decode(encoding, &body, decoded_limit) {
                Ok(decoded) => decoded,
                Err(reason) => return CapturedBody::Unavailable(reason),
            };
        }
        CapturedBody::Observed(body)
    }
}

fn decode(encoding: &str, body: &[u8], limit: usize) -> Result<Vec<u8>, &'static str> {
    let bounded = |reader: &mut dyn Read| -> Result<Vec<u8>, &'static str> {
        let mut output = Vec::new();
        reader
            .take(limit as u64 + 1)
            .read_to_end(&mut output)
            .map_err(|_| "content-decoding-failed")?;
        if output.len() > limit {
            return Err("decoded-limit-exceeded");
        }
        Ok(output)
    };
    match encoding {
        "gzip" | "x-gzip" => bounded(&mut flate2::read::MultiGzDecoder::new(body)),
        "deflate" => bounded(&mut flate2::read::ZlibDecoder::new(body))
            .or_else(|_| bounded(&mut flate2::read::DeflateDecoder::new(body))),
        "br" => bounded(&mut brotli_decompressor::Decompressor::new(body, 4096)),
        _ => Err("unsupported-content-encoding"),
    }
}

fn forward_body<R, W>(
    source: &mut Wire<R>,
    sink: &mut W,
    framing: Framing,
    tap: &mut Tap,
) -> Result<BodyEnd, RelayError>
where
    R: Read + Write,
    W: Write,
{
    match framing {
        Framing::None => Ok(BodyEnd::Complete),
        Framing::Length(mut remaining) => {
            while remaining > 0 {
                let want = usize::try_from(remaining)
                    .unwrap_or(usize::MAX)
                    .min(COPY_CHUNK);
                let bytes = source.next_bytes(want).map_err(RelayError::io)?;
                if bytes.is_empty() {
                    return Ok(BodyEnd::Truncated);
                }
                tap.wire(&bytes);
                tap.data(&bytes);
                sink.write_all(&bytes).map_err(RelayError::io)?;
                remaining -= bytes.len() as u64;
            }
            Ok(BodyEnd::Complete)
        }
        Framing::UntilClose => loop {
            let bytes = match source.next_bytes(COPY_CHUNK) {
                Ok(bytes) => bytes,
                Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => Vec::new(),
                Err(error) => return Err(RelayError::io(error)),
            };
            if bytes.is_empty() {
                return Ok(BodyEnd::Complete);
            }
            tap.wire(&bytes);
            tap.data(&bytes);
            sink.write_all(&bytes).map_err(RelayError::io)?;
            sink.flush().map_err(RelayError::io)?;
        },
        Framing::Chunked => loop {
            let Some(line) = source.read_line(CHUNK_LINE_LIMIT)? else {
                return Ok(BodyEnd::Truncated);
            };
            tap.wire(&line);
            sink.write_all(&line).map_err(RelayError::io)?;
            let size_text = std::str::from_utf8(&line[..line.len() - 2])
                .map_err(|_| RelayError::Ambiguous("malformed-chunk-size"))?;
            let size_text = size_text.split(';').next().unwrap_or_default().trim();
            let mut remaining = u64::from_str_radix(size_text, 16)
                .map_err(|_| RelayError::Ambiguous("malformed-chunk-size"))?;
            if remaining == 0 {
                loop {
                    let Some(trailer) = source.read_line(CHUNK_LINE_LIMIT)? else {
                        return Ok(BodyEnd::Truncated);
                    };
                    tap.wire(&trailer);
                    sink.write_all(&trailer).map_err(RelayError::io)?;
                    if trailer == b"\r\n" {
                        sink.flush().map_err(RelayError::io)?;
                        return Ok(BodyEnd::Complete);
                    }
                }
            }
            while remaining > 0 {
                let want = usize::try_from(remaining)
                    .unwrap_or(usize::MAX)
                    .min(COPY_CHUNK);
                let bytes = source.next_bytes(want).map_err(RelayError::io)?;
                if bytes.is_empty() {
                    return Ok(BodyEnd::Truncated);
                }
                tap.wire(&bytes);
                tap.data(&bytes);
                sink.write_all(&bytes).map_err(RelayError::io)?;
                remaining -= bytes.len() as u64;
            }
            let Some(terminator) = source.read_line(2)? else {
                return Ok(BodyEnd::Truncated);
            };
            if terminator != b"\r\n" {
                return Err(RelayError::Ambiguous("malformed-chunk-terminator"));
            }
            tap.wire(&terminator);
            sink.write_all(&terminator).map_err(RelayError::io)?;
            sink.flush().map_err(RelayError::io)?;
        },
    }
}

/// A byte source with a small read-ahead buffer, so heads can be read
/// without consuming the bytes that follow them.
struct Wire<S> {
    io: S,
    buffer: Vec<u8>,
    position: usize,
}

impl<S: Read + Write> Wire<S> {
    const fn new(io: S) -> Self {
        Self {
            io,
            buffer: Vec::new(),
            position: 0,
        }
    }

    fn buffered(&self) -> &[u8] {
        &self.buffer[self.position..]
    }

    fn take_buffered(&mut self) -> Vec<u8> {
        let leftover = self.buffered().to_vec();
        self.buffer.clear();
        self.position = 0;
        leftover
    }

    fn fill(&mut self) -> io::Result<usize> {
        if self.position > 0 {
            self.buffer.drain(..self.position);
            self.position = 0;
        }
        let mut chunk = [0_u8; COPY_CHUNK];
        let read = loop {
            match self.io.read(&mut chunk) {
                Ok(read) => break read,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => return Err(error),
            }
        };
        self.buffer.extend_from_slice(&chunk[..read]);
        Ok(read)
    }

    /// Reads one head through its blank line. `None` is a clean close before
    /// any byte of a new message.
    fn read_head(&mut self, limit: usize) -> Result<Option<Vec<u8>>, RelayError> {
        loop {
            if let Some(end) = find(self.buffered(), b"\r\n\r\n") {
                let end = end + 4;
                if end > limit {
                    return Err(RelayError::Limit("header-bytes-limit"));
                }
                let head = self.buffered()[..end].to_vec();
                self.position += end;
                return Ok(Some(head));
            }
            if self.buffered().len() > limit {
                return Err(RelayError::Limit("header-bytes-limit"));
            }
            let read = match self.fill() {
                Ok(read) => read,
                Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => 0,
                Err(error) => return Err(RelayError::io(error)),
            };
            if read == 0 {
                return if self.buffered().is_empty() {
                    Ok(None)
                } else {
                    Err(RelayError::Ambiguous("truncated-head"))
                };
            }
        }
    }

    /// Reads one CRLF-terminated line, including the CRLF. `None` is EOF.
    fn read_line(&mut self, limit: usize) -> Result<Option<Vec<u8>>, RelayError> {
        loop {
            if let Some(end) = find(self.buffered(), b"\r\n") {
                let end = end + 2;
                if end > limit {
                    return Err(RelayError::Limit("chunk-line-limit"));
                }
                let line = self.buffered()[..end].to_vec();
                self.position += end;
                return Ok(Some(line));
            }
            if self.buffered().len() > limit {
                return Err(RelayError::Ambiguous("malformed-chunk-line"));
            }
            match self.fill() {
                Ok(0) => return Ok(None),
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
                Err(error) => return Err(RelayError::io(error)),
            }
        }
    }

    /// Up to `want` bytes; empty only at EOF.
    fn next_bytes(&mut self, want: usize) -> io::Result<Vec<u8>> {
        if self.buffered().is_empty() {
            match self.fill() {
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(Vec::new()),
                Err(error) => return Err(error),
            }
        }
        let take = self.buffered().len().min(want);
        let bytes = self.buffered()[..take].to_vec();
        self.position += take;
        Ok(bytes)
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// A socket whose first reads return bytes already read past a CONNECT head.
struct Prefixed {
    prefix: Vec<u8>,
    position: usize,
    inner: TcpStream,
}

impl Prefixed {
    const fn new(prefix: Vec<u8>, inner: TcpStream) -> Self {
        Self {
            prefix,
            position: 0,
            inner,
        }
    }
}

impl Read for Prefixed {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.position < self.prefix.len() {
            let take = (self.prefix.len() - self.position).min(buffer.len());
            buffer[..take].copy_from_slice(&self.prefix[self.position..self.position + take]);
            self.position += take;
            return Ok(take);
        }
        self.inner.read(buffer)
    }
}

impl Write for Prefixed {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.inner.write(buffer)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn head(text: &str) -> Head {
        Head::parse(text.as_bytes().to_vec(), 32).unwrap()
    }

    #[test]
    fn request_framing_refuses_ambiguity() {
        assert_eq!(
            head("POST / HTTP/1.1\r\nContent-Length: 4\r\n\r\n").request_framing(),
            Ok(Framing::Length(4))
        );
        assert_eq!(
            head("POST / HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n").request_framing(),
            Ok(Framing::Chunked)
        );
        assert_eq!(
            head("POST / HTTP/1.1\r\nContent-Length: 4\r\nTransfer-Encoding: chunked\r\n\r\n")
                .request_framing(),
            Err(RelayError::Ambiguous("length-and-transfer-encoding"))
        );
        assert_eq!(
            head("POST / HTTP/1.1\r\nContent-Length: 4\r\nContent-Length: 5\r\n\r\n")
                .request_framing(),
            Err(RelayError::Ambiguous("conflicting-content-length"))
        );
        assert!(Head::parse(b"GET / HTTP/1.1\r\nX: a\r\n b\r\n\r\n".to_vec(), 32).is_err());
    }

    #[test]
    fn response_framing_follows_method_and_status() {
        let response = head("HTTP/1.1 200 OK\r\nContent-Length: 9\r\n\r\n");
        assert_eq!(response.response_framing("HEAD", 200), Ok(Framing::None));
        assert_eq!(
            response.response_framing("GET", 200),
            Ok(Framing::Length(9))
        );
        assert_eq!(
            head("HTTP/1.1 204 No Content\r\n\r\n").response_framing("GET", 204),
            Ok(Framing::None)
        );
        assert_eq!(
            head("HTTP/1.1 200 OK\r\n\r\n").response_framing("GET", 200),
            Ok(Framing::UntilClose)
        );
    }

    #[test]
    fn chunked_bodies_forward_verbatim_and_capture_decoded() {
        let wire = b"4\r\nwire\r\n8;ext=1\r\n-witness\r\n0\r\nTrailer: x\r\n\r\n";
        let mut source = Wire::new(io::Cursor::new(wire.to_vec()));
        let mut sink = Vec::new();
        let mut tap = Tap::new(true, 64);
        assert_eq!(
            forward_body(&mut source, &mut sink, Framing::Chunked, &mut tap),
            Ok(BodyEnd::Complete)
        );
        assert_eq!(sink, wire);
        assert_eq!(tap.body, b"wire-witness");
        assert_eq!(tap.reference(), ByteReference::observed(wire));
    }

    #[test]
    fn capture_limit_stops_capture_but_not_forwarding() {
        let mut source = Wire::new(io::Cursor::new(b"0123456789".to_vec()));
        let mut sink = Vec::new();
        let mut tap = Tap::new(true, 4);
        assert_eq!(
            forward_body(&mut source, &mut sink, Framing::Length(10), &mut tap),
            Ok(BodyEnd::Complete)
        );
        assert_eq!(sink, b"0123456789");
        assert!(tap.overflow);
        assert_eq!(
            tap.body(&[], 64),
            CapturedBody::Unavailable("capture-limit-exceeded")
        );
    }

    #[test]
    fn truncated_length_body_is_reported() {
        let mut source = Wire::new(io::Cursor::new(b"abc".to_vec()));
        let mut sink = Vec::new();
        let mut tap = Tap::new(true, 64);
        assert_eq!(
            forward_body(&mut source, &mut sink, Framing::Length(10), &mut tap),
            Ok(BodyEnd::Truncated)
        );
    }

    #[test]
    fn content_decoding_is_bounded_and_closed() {
        use std::io::Write as _;
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(b"{\"model\":\"m\"}").unwrap();
        let gzip = encoder.finish().unwrap();
        let headers = vec![("content-encoding".to_owned(), "gzip".to_owned())];
        let mut tap = Tap::new(true, 1024);
        tap.data(&gzip);
        assert_eq!(
            tap.body(&headers, 1024),
            CapturedBody::Observed(b"{\"model\":\"m\"}".to_vec())
        );
        assert_eq!(
            tap.body(&headers, 4),
            CapturedBody::Unavailable("decoded-limit-exceeded")
        );
        let zstd = vec![("content-encoding".to_owned(), "zstd".to_owned())];
        assert_eq!(
            tap.body(&zstd, 1024),
            CapturedBody::Unavailable("unsupported-content-encoding")
        );
    }

    #[test]
    fn plain_targets_name_their_origin() {
        assert_eq!(
            absolute_form_authority("http://example.com/x?y"),
            Some(Authority::parse("example.com:80").unwrap())
        );
        assert_eq!(
            absolute_form_authority("http://example.com:8080"),
            Some(Authority::parse("example.com:8080").unwrap())
        );
        assert_eq!(absolute_form_authority("/relative"), None);
    }

    #[test]
    fn heads_keep_following_bytes() {
        let mut wire = Wire::new(io::Cursor::new(b"GET / HTTP/1.1\r\n\r\nNEXT".to_vec()));
        assert_eq!(
            wire.read_head(1024).unwrap().unwrap(),
            b"GET / HTTP/1.1\r\n\r\n"
        );
        assert_eq!(wire.take_buffered(), b"NEXT");
    }

    #[test]
    fn pem_encoding_matches_base64() {
        let pem = pem_certificate(b"wire-witness");
        assert_eq!(
            pem,
            "-----BEGIN CERTIFICATE-----\nd2lyZS13aXRuZXNz\n-----END CERTIFICATE-----\n"
        );
        assert!(pem_certificate(b"ab").contains("YWI=\n"));
    }

    #[test]
    fn ephemeral_ca_exposes_only_its_certificate() {
        let ca = EphemeralCa::generate(SystemTime::UNIX_EPOCH + Duration::from_secs(1_790_000_000))
            .unwrap();
        let pem = ca.certificate_pem();
        assert!(pem.starts_with("-----BEGIN CERTIFICATE-----"));
        assert!(!pem.contains("PRIVATE KEY"));
        assert!(!format!("{ca:?}").contains("key"));
        let (chain, _) = ca.issue_leaf("api.example.com").unwrap();
        assert_eq!(chain.len(), 2);
    }
}

// endregion
