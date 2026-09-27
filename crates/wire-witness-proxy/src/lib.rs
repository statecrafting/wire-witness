//! Per-attempt allowlisted capture proxy and bounded protocol framing.
//!
//! Governed by spec 004, allowlisted capture proxy and transparent streams.

#![forbid(unsafe_code)]

// region: allowlisted-capture-proxy

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use wire_witness_core::exchange::{IncompleteReason, ProviderFamily};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Authority {
    host: String,
    port: u16,
}

impl Authority {
    pub fn parse(value: &str) -> Result<Self, ProxyError> {
        if value.is_empty()
            || value.contains("//")
            || value.contains('/')
            || value.contains('@')
            || value.contains('*')
        {
            return Err(ProxyError::InvalidAuthority);
        }
        let (host, port) = if let Some(rest) = value.strip_prefix('[') {
            let (host, port) = rest.split_once("]:").ok_or(ProxyError::InvalidAuthority)?;
            (format!("[{host}]"), port)
        } else {
            let (host, port) = value.rsplit_once(':').ok_or(ProxyError::InvalidAuthority)?;
            if host.contains(':') {
                return Err(ProxyError::InvalidAuthority);
            }
            (host.to_ascii_lowercase(), port)
        };
        if host.is_empty() || host.bytes().any(|byte| byte.is_ascii_whitespace()) {
            return Err(ProxyError::InvalidAuthority);
        }
        let port = port
            .parse::<u16>()
            .ok()
            .filter(|port| *port != 0)
            .ok_or(ProxyError::InvalidAuthority)?;
        Ok(Self { host, port })
    }

    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    #[must_use]
    pub fn canonical(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationProtocol {
    Http1,
    Http2,
    Sse,
    WebSocket,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureAuthority {
    pub authority: Authority,
    pub provider: ProviderFamily,
    pub protocols: BTreeSet<ApplicationProtocol>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProxyConfig {
    capture: BTreeMap<Authority, CaptureAuthority>,
    authentication_tunnels: BTreeSet<Authority>,
}

impl ProxyConfig {
    pub fn new(
        capture: Vec<CaptureAuthority>,
        authentication_tunnels: Vec<Authority>,
    ) -> Result<Self, ProxyError> {
        let authentication_tunnels: BTreeSet<_> = authentication_tunnels.into_iter().collect();
        let mut capture_map = BTreeMap::new();
        for entry in capture {
            if entry.protocols.is_empty()
                || authentication_tunnels.contains(&entry.authority)
                || capture_map.contains_key(&entry.authority)
            {
                return Err(ProxyError::InvalidConfiguration);
            }
            capture_map.insert(entry.authority.clone(), entry);
        }
        Ok(Self {
            capture: capture_map,
            authentication_tunnels,
        })
    }

    #[must_use]
    pub fn route(&self, authority: &Authority) -> RouteDecision {
        if self.authentication_tunnels.contains(authority) {
            return RouteDecision::AuthenticationTunnel;
        }
        self.capture
            .get(authority)
            .map_or(RouteDecision::OpaqueTunnel, |entry| {
                RouteDecision::Intercept {
                    provider: entry.provider.clone(),
                    protocols: entry.protocols.clone(),
                }
            })
    }

    pub fn validate_interception_identity(
        &self,
        connect: &Authority,
        sni: &str,
        certificate_name: &str,
    ) -> Result<(), ProxyError> {
        if !matches!(self.route(connect), RouteDecision::Intercept { .. }) {
            return Err(ProxyError::InterceptionNotAllowed);
        }
        let expected = connect.host.trim_matches(['[', ']']);
        if !expected.eq_ignore_ascii_case(sni) || !expected.eq_ignore_ascii_case(certificate_name) {
            return Err(ProxyError::TlsIdentityMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RouteDecision {
    Intercept {
        provider: ProviderFamily,
        protocols: BTreeSet<ApplicationProtocol>,
    },
    AuthenticationTunnel,
    OpaqueTunnel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProxyLimits {
    pub header_bytes: usize,
    pub header_count: usize,
    pub frame_bytes: usize,
    pub event_bytes: usize,
    pub queue_bytes: usize,
}

impl ProxyLimits {
    pub fn validate(self) -> Result<Self, ProxyError> {
        if self.header_bytes == 0
            || self.header_count == 0
            || self.frame_bytes == 0
            || self.event_bytes == 0
            || self.queue_bytes == 0
        {
            return Err(ProxyError::InvalidConfiguration);
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProxyError {
    InvalidAuthority,
    InvalidConfiguration,
    InterceptionNotAllowed,
    TlsIdentityMismatch,
    HeaderLimit,
    MalformedConnect,
    AmbiguousHttp,
    FrameLimit,
    MalformedFrame,
    SinkBackpressure,
}

impl ProxyError {
    #[must_use]
    pub const fn incomplete_reason(&self) -> Option<IncompleteReason> {
        match self {
            Self::HeaderLimit | Self::FrameLimit => Some(IncompleteReason::LimitExceeded),
            Self::MalformedConnect | Self::AmbiguousHttp | Self::MalformedFrame => {
                Some(IncompleteReason::Malformed)
            }
            Self::SinkBackpressure => Some(IncompleteReason::SinkBackpressure),
            Self::InvalidAuthority
            | Self::InvalidConfiguration
            | Self::InterceptionNotAllowed
            | Self::TlsIdentityMismatch => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConnectRequest {
    pub authority: Authority,
    pub headers: Vec<(String, String)>,
}

pub fn parse_connect(bytes: &[u8], limits: ProxyLimits) -> Result<ConnectRequest, ProxyError> {
    let limits = limits.validate()?;
    if bytes.len() > limits.header_bytes {
        return Err(ProxyError::HeaderLimit);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| ProxyError::MalformedConnect)?;
    if !text.ends_with("\r\n\r\n") || text[..text.len() - 4].contains("\r\n\r\n") {
        return Err(ProxyError::MalformedConnect);
    }
    let mut lines = text[..text.len() - 4].split("\r\n");
    let request = lines.next().ok_or(ProxyError::MalformedConnect)?;
    let mut words = request.split(' ');
    if words.next() != Some("CONNECT") {
        return Err(ProxyError::MalformedConnect);
    }
    let authority = Authority::parse(words.next().ok_or(ProxyError::MalformedConnect)?)?;
    if words.next() != Some("HTTP/1.1") || words.next().is_some() {
        return Err(ProxyError::MalformedConnect);
    }
    let mut headers = Vec::new();
    for line in lines {
        if headers.len() >= limits.header_count || line.starts_with([' ', '\t']) {
            return Err(ProxyError::HeaderLimit);
        }
        let (name, value) = line.split_once(':').ok_or(ProxyError::MalformedConnect)?;
        if name.is_empty()
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            || value.contains(['\r', '\n'])
        {
            return Err(ProxyError::MalformedConnect);
        }
        if name.eq_ignore_ascii_case("content-length") && value.trim() != "0"
            || name.eq_ignore_ascii_case("transfer-encoding")
        {
            return Err(ProxyError::AmbiguousHttp);
        }
        headers.push((name.to_ascii_lowercase(), value.trim().to_owned()));
    }
    Ok(ConnectRequest { authority, headers })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedQueue {
    high_water: usize,
    bytes: usize,
    chunks: VecDeque<Vec<u8>>,
}

impl BoundedQueue {
    pub fn new(high_water: usize) -> Result<Self, ProxyError> {
        if high_water == 0 {
            return Err(ProxyError::InvalidConfiguration);
        }
        Ok(Self {
            high_water,
            bytes: 0,
            chunks: VecDeque::new(),
        })
    }

    pub fn push(&mut self, chunk: &[u8]) -> Result<(), ProxyError> {
        let next = self
            .bytes
            .checked_add(chunk.len())
            .ok_or(ProxyError::SinkBackpressure)?;
        if next > self.high_water {
            return Err(ProxyError::SinkBackpressure);
        }
        self.chunks.push_back(chunk.to_vec());
        self.bytes = next;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<Vec<u8>> {
        let chunk = self.chunks.pop_front()?;
        self.bytes -= chunk.len();
        Some(chunk)
    }

    #[must_use]
    pub const fn buffered_bytes(&self) -> usize {
        self.bytes
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransparentQueues {
    pub child_to_provider: BoundedQueue,
    pub provider_to_child: BoundedQueue,
}

impl TransparentQueues {
    pub fn new(high_water: usize) -> Result<Self, ProxyError> {
        Ok(Self {
            child_to_provider: BoundedQueue::new(high_water)?,
            provider_to_child: BoundedQueue::new(high_water)?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SseAssembler {
    limit: usize,
    buffer: Vec<u8>,
}

impl SseAssembler {
    pub fn new(limit: usize) -> Result<Self, ProxyError> {
        if limit == 0 {
            return Err(ProxyError::InvalidConfiguration);
        }
        Ok(Self {
            limit,
            buffer: Vec::new(),
        })
    }

    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<Vec<u8>>, ProxyError> {
        if self.buffer.len().saturating_add(bytes.len()) > self.limit {
            self.buffer.clear();
            return Err(ProxyError::FrameLimit);
        }
        self.buffer.extend_from_slice(bytes);
        let mut frames = Vec::new();
        while let Some(end) = sse_boundary(&self.buffer) {
            frames.push(self.buffer.drain(..end).collect());
        }
        Ok(frames)
    }

    pub fn finish(self) -> Result<(), ProxyError> {
        if self.buffer.is_empty() {
            Ok(())
        } else {
            Err(ProxyError::MalformedFrame)
        }
    }
}

fn sse_boundary(bytes: &[u8]) -> Option<usize> {
    bytes
        .windows(2)
        .position(|window| window == b"\n\n")
        .map(|offset| offset + 2)
        .or_else(|| {
            bytes
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
                .map(|offset| offset + 4)
        })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Http2Frame {
    pub stream_id: u32,
    pub frame_type: u8,
    pub flags: u8,
    pub payload: Vec<u8>,
}

impl Http2Frame {
    pub fn parse(bytes: &[u8], max_payload: usize) -> Result<(Self, usize), ProxyError> {
        if bytes.len() < 9 {
            return Err(ProxyError::MalformedFrame);
        }
        let length =
            (usize::from(bytes[0]) << 16) | (usize::from(bytes[1]) << 8) | usize::from(bytes[2]);
        if length > max_payload {
            return Err(ProxyError::FrameLimit);
        }
        let end = 9usize.checked_add(length).ok_or(ProxyError::FrameLimit)?;
        if bytes.len() < end || bytes[5] & 0x80 != 0 {
            return Err(ProxyError::MalformedFrame);
        }
        let stream_id = u32::from_be_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]);
        Ok((
            Self {
                stream_id,
                frame_type: bytes[3],
                flags: bytes[4],
                payload: bytes[9..end].to_vec(),
            },
            end,
        ))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WebSocketFrame {
    pub fin: bool,
    pub opcode: u8,
    pub masked: bool,
    pub payload: Vec<u8>,
}

impl WebSocketFrame {
    pub fn parse(
        bytes: &[u8],
        max_payload: usize,
        expect_masked: bool,
    ) -> Result<(Self, usize), ProxyError> {
        if bytes.len() < 2 || bytes[0] & 0x70 != 0 {
            return Err(ProxyError::MalformedFrame);
        }
        let fin = bytes[0] & 0x80 != 0;
        let opcode = bytes[0] & 0x0f;
        let masked = bytes[1] & 0x80 != 0;
        if masked != expect_masked {
            return Err(ProxyError::MalformedFrame);
        }
        let mut offset = 2;
        let mut length = usize::from(bytes[1] & 0x7f);
        match length {
            126 => {
                let raw = bytes
                    .get(offset..offset + 2)
                    .ok_or(ProxyError::MalformedFrame)?;
                length = usize::from(u16::from_be_bytes([raw[0], raw[1]]));
                if length < 126 {
                    return Err(ProxyError::MalformedFrame);
                }
                offset += 2;
            }
            127 => {
                let raw = bytes
                    .get(offset..offset + 8)
                    .ok_or(ProxyError::MalformedFrame)?;
                if raw[0] & 0x80 != 0 {
                    return Err(ProxyError::MalformedFrame);
                }
                length = usize::try_from(u64::from_be_bytes(raw.try_into().expect("eight bytes")))
                    .map_err(|_| ProxyError::FrameLimit)?;
                if length <= usize::from(u16::MAX) {
                    return Err(ProxyError::MalformedFrame);
                }
                offset += 8;
            }
            _ => {}
        }
        if matches!(opcode, 3..=7 | 11..=15) {
            return Err(ProxyError::MalformedFrame);
        }
        if length > max_payload || opcode >= 8 && (!fin || length > 125) {
            return Err(ProxyError::FrameLimit);
        }
        let mask = if masked {
            let raw = bytes
                .get(offset..offset + 4)
                .ok_or(ProxyError::MalformedFrame)?;
            offset += 4;
            Some([raw[0], raw[1], raw[2], raw[3]])
        } else {
            None
        };
        let end = offset.checked_add(length).ok_or(ProxyError::FrameLimit)?;
        let raw_payload = bytes.get(offset..end).ok_or(ProxyError::MalformedFrame)?;
        let payload = mask.map_or_else(
            || raw_payload.to_vec(),
            |key| {
                raw_payload
                    .iter()
                    .enumerate()
                    .map(|(index, byte)| byte ^ key[index % 4])
                    .collect()
            },
        );
        Ok((
            Self {
                fin,
                opcode,
                masked,
                payload,
            },
            end,
        ))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WebSocketSequence {
    fragmented: Option<u8>,
}

impl WebSocketSequence {
    pub fn observe(&mut self, frame: &WebSocketFrame) -> Result<(), ProxyError> {
        match frame.opcode {
            0 => {
                if self.fragmented.is_none() {
                    return Err(ProxyError::MalformedFrame);
                }
                if frame.fin {
                    self.fragmented = None;
                }
            }
            1 | 2 => {
                if self.fragmented.is_some() {
                    return Err(ProxyError::MalformedFrame);
                }
                if !frame.fin {
                    self.fragmented = Some(frame.opcode);
                }
            }
            8..=10 => {}
            _ => return Err(ProxyError::MalformedFrame),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> ProxyLimits {
        ProxyLimits {
            header_bytes: 1024,
            header_count: 16,
            frame_bytes: 1024,
            event_bytes: 1024,
            queue_bytes: 8,
        }
    }

    fn capture(authority: &str) -> CaptureAuthority {
        CaptureAuthority {
            authority: Authority::parse(authority).unwrap(),
            provider: ProviderFamily::Anthropic,
            protocols: BTreeSet::from([ApplicationProtocol::Http1, ApplicationProtocol::Sse]),
        }
    }

    #[test]
    fn only_exact_allowlisted_authorities_are_intercepted() {
        let config = ProxyConfig::new(
            vec![capture("api.example.com:443")],
            vec![Authority::parse("auth.example.com:443").unwrap()],
        )
        .unwrap();
        assert!(matches!(
            config.route(&Authority::parse("api.example.com:443").unwrap()),
            RouteDecision::Intercept { .. }
        ));
        assert_eq!(
            config.route(&Authority::parse("auth.example.com:443").unwrap()),
            RouteDecision::AuthenticationTunnel
        );
        assert_eq!(
            config.route(&Authority::parse("logs.example.com:443").unwrap()),
            RouteDecision::OpaqueTunnel
        );
    }

    #[test]
    fn overlap_wildcards_and_duplicates_are_refused() {
        let authority = Authority::parse("api.example.com:443").unwrap();
        assert!(Authority::parse("*.example.com:443").is_err());
        assert!(ProxyConfig::new(vec![capture("api.example.com:443")], vec![authority]).is_err());
        assert!(
            ProxyConfig::new(
                vec![
                    capture("api.example.com:443"),
                    capture("api.example.com:443")
                ],
                Vec::new()
            )
            .is_err()
        );
    }

    #[test]
    fn connect_parser_is_strict_and_bounded() {
        let request = parse_connect(
            b"CONNECT api.example.com:443 HTTP/1.1\r\nHost: api.example.com:443\r\n\r\n",
            limits(),
        )
        .unwrap();
        assert_eq!(request.authority.canonical(), "api.example.com:443");
        assert_eq!(
            parse_connect(
                b"CONNECT api.example.com:443 HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n",
                limits()
            ),
            Err(ProxyError::AmbiguousHttp)
        );
    }

    #[test]
    fn tls_sni_and_certificate_must_match_connect() {
        let config = ProxyConfig::new(vec![capture("api.example.com:443")], Vec::new()).unwrap();
        let authority = Authority::parse("api.example.com:443").unwrap();
        assert!(
            config
                .validate_interception_identity(&authority, "api.example.com", "api.example.com")
                .is_ok()
        );
        assert_eq!(
            config.validate_interception_identity(
                &authority,
                "other.example.com",
                "api.example.com"
            ),
            Err(ProxyError::TlsIdentityMismatch)
        );
    }

    #[test]
    fn queue_refuses_overflow_without_changing_buffer() {
        let mut queue = BoundedQueue::new(8).unwrap();
        queue.push(b"123456").unwrap();
        assert_eq!(queue.push(b"789"), Err(ProxyError::SinkBackpressure));
        assert_eq!(queue.buffered_bytes(), 6);
        assert_eq!(queue.pop().unwrap(), b"123456");
    }

    #[test]
    fn sse_assembly_preserves_frame_bytes_and_order() {
        let mut assembler = SseAssembler::new(128).unwrap();
        assert!(assembler.push(b"event: one\nda").unwrap().is_empty());
        let frames = assembler
            .push(b"ta: {}\n\nevent: two\ndata: {}\n\n")
            .unwrap();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0], b"event: one\ndata: {}\n\n");
        assert_eq!(frames[1], b"event: two\ndata: {}\n\n");
        assert!(assembler.finish().is_ok());
    }

    #[test]
    fn sse_limit_clears_capture_buffer() {
        let mut assembler = SseAssembler::new(4).unwrap();
        assert_eq!(assembler.push(b"12345"), Err(ProxyError::FrameLimit));
        assert!(assembler.finish().is_ok());
    }

    #[test]
    fn http2_frame_parser_preserves_stream_and_payload() {
        let bytes = [0, 0, 3, 0, 1, 0, 0, 0, 7, b'a', b'b', b'c'];
        let (frame, consumed) = Http2Frame::parse(&bytes, 10).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(frame.stream_id, 7);
        assert_eq!(frame.payload, b"abc");
    }

    #[test]
    fn websocket_parser_unmasks_for_observation() {
        let bytes = [0x81, 0x82, 1, 2, 3, 4, b'h' ^ 1, b'i' ^ 2];
        let (frame, consumed) = WebSocketFrame::parse(&bytes, 16, true).unwrap();
        assert_eq!(consumed, bytes.len());
        assert!(frame.fin && frame.masked);
        assert_eq!(frame.payload, b"hi");
    }

    #[test]
    fn fragmented_control_frame_is_refused() {
        assert_eq!(
            WebSocketFrame::parse(&[0x09, 0x80, 1, 2, 3, 4], 16, true),
            Err(ProxyError::FrameLimit)
        );
    }

    #[test]
    fn websocket_fragmentation_sequence_is_checked() {
        let mut sequence = WebSocketSequence::default();
        let start = WebSocketFrame {
            fin: false,
            opcode: 1,
            masked: true,
            payload: b"a".to_vec(),
        };
        let ping = WebSocketFrame {
            fin: true,
            opcode: 9,
            masked: true,
            payload: Vec::new(),
        };
        let end = WebSocketFrame {
            fin: true,
            opcode: 0,
            masked: true,
            payload: b"b".to_vec(),
        };
        assert!(sequence.observe(&start).is_ok());
        assert!(sequence.observe(&ping).is_ok());
        assert!(sequence.observe(&end).is_ok());
        assert_eq!(sequence.observe(&end), Err(ProxyError::MalformedFrame));
    }
}

// endregion
