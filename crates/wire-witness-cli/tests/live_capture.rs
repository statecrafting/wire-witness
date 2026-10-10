//! End-to-end acceptance for the live runtime against a local fake provider.
//!
//! Governed by spec 013, live loopback capture runtime. The test re-runs its
//! own binary as the witnessed child: `live_child_helper` acts only when its
//! marker filter is present, and is otherwise an empty pass.

// region: live-capture-acceptance

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName};
use wire_witness_cli::live_runtime::{EXCHANGES_DIRECTORY, LiveRuntime, MANIFEST_FILE};
use wire_witness_cli::sidecar_protocol::StartAuthority;
use wire_witness_cli::standalone_host::{
    CaptureState, ChildExit, HostExit, RunRequest, TrustMechanism, run,
};
use wire_witness_core::custody::RetentionMode;
use wire_witness_core::exchange::ProviderFamily;
use wire_witness_proxy::live::{LiveLimits, ProviderTrust};
use wire_witness_proxy::{ApplicationProtocol, Authority};

const MARKER: &str = "wire-witness-live-child:";
const SECRET: &str = "sk-ant-api03-WITNESS-NEVER-PERSISTS";
const SERVED: &str = "claude-served-by-fake-provider";

fn crypto() -> Arc<rustls::crypto::CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

struct FakeProvider {
    port: u16,
    root: CertificateDer<'static>,
}

/// A TLS origin for `localhost` that answers one JSON message request with a
/// gzip body and one streaming request with chunked SSE, on one connection.
fn fake_provider() -> FakeProvider {
    let ca_key = rcgen::KeyPair::generate().unwrap();
    let mut ca_params = rcgen::CertificateParams::default();
    ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    ca_params.key_usages = vec![
        rcgen::KeyUsagePurpose::KeyCertSign,
        rcgen::KeyUsagePurpose::DigitalSignature,
    ];
    let ca = ca_params.self_signed(&ca_key).unwrap();
    let leaf_key = rcgen::KeyPair::generate().unwrap();
    let leaf = rcgen::CertificateParams::new(vec!["localhost".to_owned()])
        .unwrap()
        .signed_by(&leaf_key, &ca, &ca_key)
        .unwrap();
    let config = rustls::ServerConfig::builder_with_provider(crypto())
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(
            vec![leaf.der().clone()],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(leaf_key.serialize_der())),
        )
        .unwrap();
    let config = Arc::new(config);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { return };
            let config = Arc::clone(&config);
            thread::spawn(move || serve_provider(stream, config));
        }
    });
    FakeProvider {
        port,
        root: ca.der().clone(),
    }
}

fn serve_provider(stream: TcpStream, config: Arc<rustls::ServerConfig>) {
    let connection = rustls::ServerConnection::new(config).unwrap();
    let mut tls = BufReader::new(rustls::StreamOwned::new(connection, stream));
    loop {
        let Some((head, body)) = read_message(&mut tls) else {
            return;
        };
        let streaming =
            !head.is_empty() && String::from_utf8_lossy(&body).contains("\"stream\":true");
        let stream = tls.get_mut();
        if streaming {
            let events = [
                format!(
                    "event: message_start\ndata: {{\"type\":\"message_start\",\"message\":{{\"model\":\"{SERVED}\",\"usage\":{{\"input_tokens\":7}}}}}}\n\n"
                ),
                "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0}\n\n".to_owned(),
                "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n".to_owned(),
            ];
            stream
                .write_all(b"HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\ntransfer-encoding: chunked\r\n\r\n")
                .unwrap();
            for event in events {
                write!(stream, "{:x}\r\n{event}\r\n", event.len()).unwrap();
                stream.flush().unwrap();
            }
            stream.write_all(b"0\r\n\r\n").unwrap();
        } else {
            let body = format!(
                "{{\"type\":\"message\",\"model\":\"{SERVED}\",\"usage\":{{\"input_tokens\":3,\"output_tokens\":5}}}}"
            );
            let mut encoder =
                flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(body.as_bytes()).unwrap();
            let gzip = encoder.finish().unwrap();
            write!(
                stream,
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-encoding: gzip\r\nset-cookie: session={SECRET}\r\ncontent-length: {}\r\n\r\n",
                gzip.len()
            )
            .unwrap();
            stream.write_all(&gzip).unwrap();
        }
        stream.flush().unwrap();
    }
}

/// Reads one HTTP/1.1 message with a content-length or chunked body.
fn read_message<R: BufRead>(reader: &mut R) -> Option<(String, Vec<u8>)> {
    let mut head = String::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            return None;
        }
        head.push_str(&line);
        if line == "\r\n" {
            break;
        }
    }
    let lower = head.to_ascii_lowercase();
    let mut body = Vec::new();
    if lower.contains("transfer-encoding: chunked") {
        loop {
            let mut size = String::new();
            reader.read_line(&mut size).ok()?;
            let size = usize::from_str_radix(size.trim(), 16).ok()?;
            let mut chunk = vec![0; size + 2];
            reader.read_exact(&mut chunk).ok()?;
            if size == 0 {
                break;
            }
            body.extend_from_slice(&chunk[..size]);
        }
    } else if let Some(line) = lower
        .lines()
        .find(|line| line.starts_with("content-length:"))
    {
        let length: usize = line["content-length:".len()..].trim().parse().ok()?;
        body.resize(length, 0);
        reader.read_exact(&mut body).ok()?;
    }
    Some((head, body))
}

/// A plain TCP echo server standing in for a host outside the capture list.
fn echo_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { return };
            thread::spawn(move || {
                let mut buffer = [0_u8; 64];
                while let Ok(read) = stream.read(&mut buffer) {
                    if read == 0 || stream.write_all(&buffer[..read]).is_err() {
                        return;
                    }
                }
            });
        }
    });
    port
}

fn connect_through(proxy: &str, authority: &str) -> TcpStream {
    let mut stream = TcpStream::connect(proxy).unwrap();
    write!(
        stream,
        "CONNECT {authority} HTTP/1.1\r\nHost: {authority}\r\n\r\n"
    )
    .unwrap();
    let mut response = Vec::new();
    let mut byte = [0_u8; 1];
    while !response.ends_with(b"\r\n\r\n") {
        stream.read_exact(&mut byte).unwrap();
        response.push(byte[0]);
    }
    assert!(
        response.starts_with(b"HTTP/1.1 200"),
        "CONNECT refused: {}",
        String::from_utf8_lossy(&response)
    );
    stream
}

/// The witnessed child. It reads only what the host's per-process overlay
/// gave it: the proxy endpoint and the ephemeral CA certificate.
#[test]
fn live_child_helper() {
    let Some(ports) =
        std::env::args().find_map(|argument| argument.strip_prefix(MARKER).map(str::to_owned))
    else {
        return;
    };
    let (provider_port, echo_port) = ports.split_once(',').unwrap();
    let proxy = std::env::var("HTTPS_PROXY").unwrap();
    let proxy = proxy.strip_prefix("http://").unwrap().to_owned();
    let ca_path = std::env::var("NODE_EXTRA_CA_CERTS").unwrap();
    let ca = CertificateDer::from_pem_file(&ca_path).unwrap();
    let mut roots = rustls::RootCertStore::empty();
    roots.add(ca).unwrap();
    let config = rustls::ClientConfig::builder_with_provider(crypto())
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(roots)
        .with_no_client_auth();

    let authority = format!("localhost:{provider_port}");
    let socket = connect_through(&proxy, &authority);
    let connection =
        rustls::ClientConnection::new(Arc::new(config), ServerName::try_from("localhost").unwrap())
            .unwrap();
    let mut tls = BufReader::new(rustls::StreamOwned::new(connection, socket));
    for body in [
        "{\"model\":\"claude-requested\",\"max_tokens\":8}",
        "{\"model\":\"claude-requested\",\"max_tokens\":8,\"stream\":true}",
    ] {
        write!(
            tls.get_mut(),
            "POST /v1/messages HTTP/1.1\r\nHost: {authority}\r\nx-api-key: {SECRET}\r\nauthorization: Bearer {SECRET}\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
        tls.get_mut().flush().unwrap();
        let (head, _) = read_message(&mut tls).expect("a response through the witness");
        assert!(head.starts_with("HTTP/1.1 200"));
    }
    drop(tls);

    let mut tunnel = connect_through(&proxy, &format!("127.0.0.1:{echo_port}"));
    tunnel.write_all(b"opaque").unwrap();
    let mut echoed = [0_u8; 6];
    tunnel.read_exact(&mut echoed).unwrap();
    assert_eq!(&echoed, b"opaque");
}

fn scratch(name: &str) -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("wire-witness-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).unwrap();
    directory
}

fn files_under(directory: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(files_under(&path));
        } else {
            files.push(path);
        }
    }
    files.sort();
    files
}

#[test]
fn live_capture_records_redacted_exchanges_and_tunnels_the_rest() {
    let provider = fake_provider();
    let echo = echo_server();
    let output = scratch("live-capture");
    let request = RunRequest {
        capture_authorities: vec![StartAuthority {
            authority: Authority::parse(&format!("localhost:{}", provider.port)).unwrap(),
            provider: ProviderFamily::Anthropic,
            protocols: [ApplicationProtocol::Http1, ApplicationProtocol::Sse]
                .into_iter()
                .collect(),
        }],
        authentication_tunnels: Vec::new(),
        requested_retention: RetentionMode::MetadataOnly,
        content_opt_in: false,
        output_directory: output.clone(),
        trust: TrustMechanism::NodeExtraCaCerts,
        instruction_probes: Vec::new(),
        program: std::env::current_exe()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
        arguments: vec![
            "--exact".into(),
            "live_child_helper".into(),
            format!("{MARKER}{},{echo}", provider.port),
            "--test-threads=1".into(),
            "--quiet".into(),
        ],
    };
    let runtime = LiveRuntime::with_provider_trust(
        ProviderTrust::Roots(vec![provider.root.clone()]),
        LiveLimits::default(),
    );
    let result = run(&request, runtime).unwrap();

    assert_eq!(result.child_exit, ChildExit::Code(0));
    assert_eq!(result.findings, Vec::new());
    assert_eq!(result.capture, CaptureState::Complete);
    assert_eq!(result.exchange_count, 2);
    assert_eq!(result.exit(), HostExit::Success);

    let attempt = output.join(&result.binding.session_id);
    let manifest_path = attempt.join(MANIFEST_FILE);
    assert_eq!(result.manifest_path.as_deref(), manifest_path.to_str());
    let manifest = fs::read_to_string(&manifest_path).unwrap();
    assert!(manifest.contains("\"schema\":\"wire-witness.capture-manifest/1\""));
    assert!(manifest.contains("\"status\":\"complete\""));
    assert!(manifest.contains("\"unintercepted_tunnels\":1"));
    assert!(manifest.contains("\"intercepted_connections\":1"));

    let exchanges: Vec<String> = files_under(&attempt.join(EXCHANGES_DIRECTORY))
        .iter()
        .map(|path| fs::read_to_string(path).unwrap())
        .collect();
    assert_eq!(exchanges.len(), 2);
    for exchange in &exchanges {
        assert!(exchange.contains(
            "\"requested_identity\":{\"status\":\"known\",\"value\":\"claude-requested\"}"
        ));
        assert!(exchange.contains(&format!("\"identity\":\"{SERVED}\"")));
        assert!(exchange.contains("\"completeness\":{\"status\":\"complete\"}"));
        assert!(exchange.contains("\"effective\":\"metadata-only\""));
    }
    assert!(exchanges[0].contains("\"field_name\":\"output_tokens\""));
    assert!(exchanges[1].contains("\"name\":\"message_stop\""));

    // Nothing durable carries a credential, and the trust file is gone.
    let files = files_under(&output);
    assert_eq!(files.len(), 3, "{files:?}");
    for path in &files {
        let bytes = fs::read(path).unwrap();
        assert!(!String::from_utf8_lossy(&bytes).contains("WITNESS-NEVER-PERSISTS"));
        assert!(!String::from_utf8_lossy(&bytes).contains("PRIVATE KEY"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
    fs::remove_dir_all(output).unwrap();
}

#[test]
fn a_child_that_never_reaches_the_provider_is_an_absent_capture() {
    let provider = fake_provider();
    let output = scratch("live-absent");
    let request = RunRequest {
        capture_authorities: vec![StartAuthority {
            authority: Authority::parse(&format!("localhost:{}", provider.port)).unwrap(),
            provider: ProviderFamily::OpenAi,
            protocols: [ApplicationProtocol::Http1].into_iter().collect(),
        }],
        authentication_tunnels: Vec::new(),
        requested_retention: RetentionMode::MetadataOnly,
        content_opt_in: false,
        output_directory: output.clone(),
        trust: TrustMechanism::NodeExtraCaCerts,
        instruction_probes: Vec::new(),
        program: "true".into(),
        arguments: Vec::new(),
    };
    let runtime = LiveRuntime::with_provider_trust(
        ProviderTrust::Roots(vec![provider.root]),
        LiveLimits::default(),
    );
    let result = run(&request, runtime).unwrap();
    assert_eq!(result.child_exit, ChildExit::Code(0));
    assert_eq!(result.capture, CaptureState::Absent);
    assert_eq!(result.exit(), HostExit::Finding);
    let manifest = fs::read_to_string(result.manifest_path.unwrap()).unwrap();
    assert!(manifest.contains("\"reason\":\"no-exchange-observed\""));
    fs::remove_dir_all(output).unwrap();
}

// endregion
