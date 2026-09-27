//! Versioned stdio sidecar protocol and evidence references.
//!
//! Governed by spec 005, attempt binding and stdio sidecar protocol.

// region: binding-and-sidecar-protocol

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use wire_witness_core::custody::RetentionMode;
use wire_witness_core::exchange::{
    JsonValue, ProviderFamily, canonical_json_bytes, parse_json, sha256_hex,
};
use wire_witness_proxy::{
    ApplicationProtocol, Authority, CaptureAuthority, ProxyConfig, ProxyError,
};

pub const PROTOCOL: &str = "wire-witness.sidecar/1";
pub const EVIDENCE_TYPE: &str = "statecraft/wire-exchange/v1";
pub const EVIDENCE_CONSTRUCTION: &str = "file-bytes-sha256";
pub const MAX_BINDING_TEXT: usize = 4096;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttemptBinding {
    pub run_id: String,
    pub attempt: u32,
    pub effect_id: String,
}

impl AttemptBinding {
    pub fn validate(&self) -> Result<(), ProtocolError> {
        for value in [&self.run_id, &self.effect_id] {
            if value.is_empty() || value.len() > MAX_BINDING_TEXT || value.contains(['\r', '\n']) {
                return Err(ProtocolError::InvalidBinding);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageKind {
    Start,
    Ready,
    Finding,
    Exchange,
    Finished,
}

impl MessageKind {
    fn parse(value: &str) -> Result<Self, ProtocolError> {
        match value {
            "start" => Ok(Self::Start),
            "ready" => Ok(Self::Ready),
            "finding" => Ok(Self::Finding),
            "exchange" => Ok(Self::Exchange),
            "finished" => Ok(Self::Finished),
            _ => Err(ProtocolError::UnknownMessageType),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Ready => "ready",
            Self::Finding => "finding",
            Self::Exchange => "exchange",
            Self::Finished => "finished",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedMessage {
    pub kind: MessageKind,
    pub binding: AttemptBinding,
    raw: BTreeMap<String, JsonValue>,
}

impl ParsedMessage {
    #[must_use]
    pub fn canonical_line(&self) -> Vec<u8> {
        let mut bytes = canonical_json_bytes(&JsonValue::Object(self.raw.clone()));
        bytes.push(b'\n');
        bytes
    }

    #[must_use]
    pub fn field(&self, name: &str) -> Option<&JsonValue> {
        self.raw.get(name)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtocolError {
    OversizedLine,
    MalformedLine,
    InvalidProtocol,
    UnknownMessageType,
    InvalidBinding,
    ForbiddenField,
    MissingField,
    InvalidField,
    UnexpectedMessage,
    BindingMismatch,
    MessageAfterTerminal,
}

pub fn parse_line(line: &[u8], max_line_bytes: usize) -> Result<ParsedMessage, ProtocolError> {
    if max_line_bytes == 0 || line.len() > max_line_bytes {
        return Err(ProtocolError::OversizedLine);
    }
    if !line.ends_with(b"\n") || line[..line.len() - 1].contains(&b'\n') {
        return Err(ProtocolError::MalformedLine);
    }
    let value = parse_json(&line[..line.len() - 1]).map_err(|_| ProtocolError::MalformedLine)?;
    reject_forbidden_fields(&value)?;
    let JsonValue::Object(raw) = value else {
        return Err(ProtocolError::MalformedLine);
    };
    if string_field(&raw, "protocol")? != PROTOCOL {
        return Err(ProtocolError::InvalidProtocol);
    }
    let kind = MessageKind::parse(string_field(&raw, "type")?)?;
    let binding = parse_binding(raw.get("binding").ok_or(ProtocolError::MissingField)?)?;
    Ok(ParsedMessage { kind, binding, raw })
}

fn reject_forbidden_fields(value: &JsonValue) -> Result<(), ProtocolError> {
    match value {
        JsonValue::Array(values) => {
            for child in values {
                reject_forbidden_fields(child)?;
            }
        }
        JsonValue::Object(values) => {
            for (name, child) in values {
                let normalized = name.to_ascii_lowercase().replace('-', "_");
                if matches!(
                    normalized.as_str(),
                    "request_body"
                        | "response_body"
                        | "captured_content"
                        | "content"
                        | "ca_key"
                        | "ca_key_path"
                        | "private_key"
                        | "private_key_path"
                        | "authorization"
                        | "proxy_authorization"
                        | "cookie"
                        | "set_cookie"
                        | "api_key"
                        | "access_token"
                        | "refresh_token"
                        | "client_secret"
                ) {
                    return Err(ProtocolError::ForbiddenField);
                }
                reject_forbidden_fields(child)?;
            }
        }
        JsonValue::Null | JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::String(_) => {}
    }
    Ok(())
}

fn parse_binding(value: &JsonValue) -> Result<AttemptBinding, ProtocolError> {
    let JsonValue::Object(fields) = value else {
        return Err(ProtocolError::InvalidBinding);
    };
    let attempt = number_field(fields, "attempt")?;
    let binding = AttemptBinding {
        run_id: string_field(fields, "run_id")?.to_owned(),
        attempt: u32::try_from(attempt).map_err(|_| ProtocolError::InvalidBinding)?,
        effect_id: string_field(fields, "effect_id")?.to_owned(),
    };
    binding.validate()?;
    Ok(binding)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartLimits {
    pub header_bytes: u64,
    pub header_count: u64,
    pub frame_bytes: u64,
    pub message_bytes: u64,
    pub decompressed_bytes: u64,
    pub event_bytes: u64,
    pub concurrent_streams: u64,
    pub connection_count: u64,
    pub queue_bytes: u64,
    pub idle_seconds: u64,
    pub total_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartRequest {
    pub binding: AttemptBinding,
    pub capture_authorities: Vec<StartAuthority>,
    pub authentication_tunnels: Vec<StartAuthority>,
    pub requested_retention: RetentionMode,
    pub limits: StartLimits,
    pub output_directory: String,
    pub certificate_path: String,
    pub host_policy_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartAuthority {
    pub authority: Authority,
    pub provider: ProviderFamily,
    pub protocols: BTreeSet<ApplicationProtocol>,
}

impl StartRequest {
    pub fn parse(message: &ParsedMessage) -> Result<Self, ProtocolError> {
        if message.kind != MessageKind::Start {
            return Err(ProtocolError::UnexpectedMessage);
        }
        let capture_authorities = authority_array_field(&message.raw, "capture_authorities")?;
        if capture_authorities.is_empty() {
            return Err(ProtocolError::InvalidField);
        }
        let authentication_tunnels = authority_array_field(&message.raw, "authentication_tunnels")?;
        ProxyConfig::new(
            capture_authorities
                .iter()
                .map(|entry| CaptureAuthority {
                    authority: entry.authority.clone(),
                    provider: entry.provider.clone(),
                    protocols: entry.protocols.clone(),
                })
                .collect(),
            authentication_tunnels
                .iter()
                .map(|entry| entry.authority.clone())
                .collect(),
        )
        .map_err(|error| match error {
            ProxyError::InvalidAuthority | ProxyError::InvalidConfiguration => {
                ProtocolError::InvalidField
            }
            _ => ProtocolError::InvalidField,
        })?;
        let retention = match string_field(&message.raw, "requested_retention")? {
            "metadata-only" => RetentionMode::MetadataOnly,
            "content" => RetentionMode::Content,
            "disabled" => RetentionMode::Disabled,
            _ => return Err(ProtocolError::InvalidField),
        };
        let limits_value = message
            .raw
            .get("limits")
            .ok_or(ProtocolError::MissingField)?;
        let JsonValue::Object(limits) = limits_value else {
            return Err(ProtocolError::InvalidField);
        };
        let positive = |name| {
            let value = number_field(limits, name)?;
            if value == 0 {
                Err(ProtocolError::InvalidField)
            } else {
                Ok(value)
            }
        };
        let request = Self {
            binding: message.binding.clone(),
            capture_authorities,
            authentication_tunnels,
            requested_retention: retention,
            limits: StartLimits {
                header_bytes: positive("header_bytes")?,
                header_count: positive("header_count")?,
                frame_bytes: positive("frame_bytes")?,
                message_bytes: positive("message_bytes")?,
                decompressed_bytes: positive("decompressed_bytes")?,
                event_bytes: positive("event_bytes")?,
                concurrent_streams: positive("concurrent_streams")?,
                connection_count: positive("connection_count")?,
                queue_bytes: positive("queue_bytes")?,
                idle_seconds: positive("idle_seconds")?,
                total_seconds: positive("total_seconds")?,
            },
            output_directory: required_bounded_string(&message.raw, "output_directory")?,
            certificate_path: required_bounded_string(&message.raw, "certificate_path")?,
            host_policy_id: required_bounded_string(&message.raw, "host_policy_id")?,
        };
        Ok(request)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptureCompleteness {
    Complete,
    Incomplete,
    Absent,
}

impl CaptureCompleteness {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Incomplete => "incomplete",
            Self::Absent => "absent",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinishedMessage {
    pub binding: AttemptBinding,
    pub completeness: CaptureCompleteness,
    pub exchange_count: u64,
    pub finding_count: u64,
    pub manifest_path: Option<String>,
    pub capture_digest: Option<String>,
    pub reason: Option<String>,
}

impl FinishedMessage {
    pub fn parse(message: &ParsedMessage) -> Result<Self, ProtocolError> {
        if message.kind != MessageKind::Finished {
            return Err(ProtocolError::UnexpectedMessage);
        }
        let completeness = match string_field(&message.raw, "completeness")? {
            "complete" => CaptureCompleteness::Complete,
            "incomplete" => CaptureCompleteness::Incomplete,
            "absent" => CaptureCompleteness::Absent,
            _ => return Err(ProtocolError::InvalidField),
        };
        let manifest_path = optional_string_field(&message.raw, "manifest_path")?;
        let capture_digest = optional_string_field(&message.raw, "capture_digest")?;
        let reason = optional_string_field(&message.raw, "reason")?;
        if completeness == CaptureCompleteness::Complete
            && (manifest_path.is_none() || capture_digest.is_none() || reason.is_some())
        {
            return Err(ProtocolError::InvalidField);
        }
        if completeness != CaptureCompleteness::Complete && reason.is_none() {
            return Err(ProtocolError::InvalidField);
        }
        Ok(Self {
            binding: message.binding.clone(),
            completeness,
            exchange_count: number_field(&message.raw, "exchange_count")?,
            finding_count: number_field(&message.raw, "finding_count")?,
            manifest_path,
            capture_digest,
            reason,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SessionPhase {
    AwaitReady,
    Running,
    Finished,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SidecarSession {
    binding: AttemptBinding,
    phase: SessionPhase,
}

impl SidecarSession {
    pub fn from_start(message: &ParsedMessage) -> Result<(Self, StartRequest), ProtocolError> {
        let start = StartRequest::parse(message)?;
        Ok((
            Self {
                binding: start.binding.clone(),
                phase: SessionPhase::AwaitReady,
            },
            start,
        ))
    }

    pub fn observe_sidecar(&mut self, message: &ParsedMessage) -> Result<(), ProtocolError> {
        if self.phase == SessionPhase::Finished {
            return Err(ProtocolError::MessageAfterTerminal);
        }
        if message.binding != self.binding {
            return Err(ProtocolError::BindingMismatch);
        }
        match (self.phase, message.kind) {
            (SessionPhase::AwaitReady, MessageKind::Ready) => {
                self.phase = SessionPhase::Running;
                Ok(())
            }
            (SessionPhase::Running, MessageKind::Finding | MessageKind::Exchange) => Ok(()),
            (SessionPhase::Running, MessageKind::Finished) => {
                FinishedMessage::parse(message)?;
                self.phase = SessionPhase::Finished;
                Ok(())
            }
            _ => Err(ProtocolError::UnexpectedMessage),
        }
    }

    #[must_use]
    pub const fn is_finished(&self) -> bool {
        matches!(self.phase, SessionPhase::Finished)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceReference {
    pub evidence_type: &'static str,
    pub schema_version: u32,
    pub byte_length: u64,
    pub sha256: String,
    pub construction: &'static str,
    pub producer_identity: Option<String>,
    pub producer_digest: Option<String>,
    pub subject: AttemptBinding,
    pub completeness: CaptureCompleteness,
}

impl EvidenceReference {
    #[must_use]
    pub fn for_manifest(
        bytes: &[u8],
        subject: AttemptBinding,
        completeness: CaptureCompleteness,
        producer_identity: Option<String>,
        producer_digest: Option<String>,
    ) -> Self {
        Self {
            evidence_type: EVIDENCE_TYPE,
            schema_version: 1,
            byte_length: bytes.len() as u64,
            sha256: sha256_hex(bytes),
            construction: EVIDENCE_CONSTRUCTION,
            producer_identity,
            producer_digest,
            subject,
            completeness,
        }
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        canonical_json_bytes(&object([
            ("byte_length", number(self.byte_length)),
            ("completeness", string(self.completeness.as_str())),
            ("construction", string(self.construction)),
            ("evidence_type", string(self.evidence_type)),
            (
                "producer_digest",
                optional_string(self.producer_digest.as_deref()),
            ),
            (
                "producer_identity",
                optional_string(self.producer_identity.as_deref()),
            ),
            ("schema_version", number(u64::from(self.schema_version))),
            ("sha256", string(&self.sha256)),
            ("subject", binding_json(&self.subject)),
        ]))
    }
}

fn string_field<'a>(
    fields: &'a BTreeMap<String, JsonValue>,
    name: &str,
) -> Result<&'a str, ProtocolError> {
    match fields.get(name) {
        Some(JsonValue::String(value)) => Ok(value),
        Some(_) => Err(ProtocolError::InvalidField),
        None => Err(ProtocolError::MissingField),
    }
}

fn required_bounded_string(
    fields: &BTreeMap<String, JsonValue>,
    name: &str,
) -> Result<String, ProtocolError> {
    let value = string_field(fields, name)?;
    if value.is_empty() || value.len() > MAX_BINDING_TEXT || value.contains(['\r', '\n']) {
        return Err(ProtocolError::InvalidField);
    }
    Ok(value.to_owned())
}

fn optional_string_field(
    fields: &BTreeMap<String, JsonValue>,
    name: &str,
) -> Result<Option<String>, ProtocolError> {
    match fields.get(name) {
        Some(JsonValue::String(value)) if !value.is_empty() && value.len() <= MAX_BINDING_TEXT => {
            Ok(Some(value.clone()))
        }
        Some(JsonValue::Null) | None => Ok(None),
        Some(_) => Err(ProtocolError::InvalidField),
    }
}

fn authority_array_field(
    fields: &BTreeMap<String, JsonValue>,
    name: &str,
) -> Result<Vec<StartAuthority>, ProtocolError> {
    let Some(JsonValue::Array(values)) = fields.get(name) else {
        return Err(if fields.contains_key(name) {
            ProtocolError::InvalidField
        } else {
            ProtocolError::MissingField
        });
    };
    values
        .iter()
        .map(|value| {
            let JsonValue::Object(entry) = value else {
                return Err(ProtocolError::InvalidField);
            };
            let authority = Authority::parse(string_field(entry, "authority")?)
                .map_err(|_| ProtocolError::InvalidField)?;
            let provider = match string_field(entry, "provider")? {
                "anthropic" => ProviderFamily::Anthropic,
                "openai" => ProviderFamily::OpenAi,
                "unknown" => ProviderFamily::Unknown,
                _ => return Err(ProtocolError::InvalidField),
            };
            let Some(JsonValue::Array(values)) = entry.get("protocols") else {
                return Err(ProtocolError::InvalidField);
            };
            let protocols = values
                .iter()
                .map(|value| match value {
                    JsonValue::String(value) if value == "http/1.1" => {
                        Ok(ApplicationProtocol::Http1)
                    }
                    JsonValue::String(value) if value == "h2" => Ok(ApplicationProtocol::Http2),
                    JsonValue::String(value) if value == "sse" => Ok(ApplicationProtocol::Sse),
                    JsonValue::String(value) if value == "websocket" => {
                        Ok(ApplicationProtocol::WebSocket)
                    }
                    _ => Err(ProtocolError::InvalidField),
                })
                .collect::<Result<BTreeSet<_>, _>>()?;
            if protocols.is_empty() {
                return Err(ProtocolError::InvalidField);
            }
            Ok(StartAuthority {
                authority,
                provider,
                protocols,
            })
        })
        .collect()
}

fn number_field(fields: &BTreeMap<String, JsonValue>, name: &str) -> Result<u64, ProtocolError> {
    match fields.get(name) {
        Some(JsonValue::Number(value)) => value.parse().map_err(|_| ProtocolError::InvalidField),
        Some(_) => Err(ProtocolError::InvalidField),
        None => Err(ProtocolError::MissingField),
    }
}

fn object<const N: usize>(entries: [(&str, JsonValue); N]) -> JsonValue {
    JsonValue::Object(
        entries
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value))
            .collect(),
    )
}

fn string(value: &str) -> JsonValue {
    JsonValue::String(value.to_owned())
}

fn number(value: u64) -> JsonValue {
    JsonValue::Number(value.to_string())
}

fn optional_string(value: Option<&str>) -> JsonValue {
    value.map_or(JsonValue::Null, string)
}

fn binding_json(binding: &AttemptBinding) -> JsonValue {
    object([
        ("attempt", number(u64::from(binding.attempt))),
        ("effect_id", string(&binding.effect_id)),
        ("run_id", string(&binding.run_id)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    const START: &str = concat!(
        "{\"protocol\":\"wire-witness.sidecar/1\",\"type\":\"start\",",
        "\"binding\":{\"run_id\":\"run-A\",\"attempt\":2,\"effect_id\":\"effect-B\"},",
        "\"capture_authorities\":[{\"authority\":\"api.example.com:443\",",
        "\"provider\":\"openai\",\"protocols\":[\"h2\",\"sse\"]}],",
        "\"authentication_tunnels\":[{\"authority\":\"auth.example.com:443\",",
        "\"provider\":\"openai\",\"protocols\":[\"http/1.1\"]}],",
        "\"requested_retention\":\"metadata-only\",",
        "\"limits\":{\"header_bytes\":1,\"header_count\":1,\"frame_bytes\":1,",
        "\"message_bytes\":1,\"decompressed_bytes\":1,\"event_bytes\":1,",
        "\"concurrent_streams\":1,\"connection_count\":1,\"queue_bytes\":1,",
        "\"idle_seconds\":1,\"total_seconds\":1},",
        "\"output_directory\":\"/attempt\",\"certificate_path\":\"/attempt/ca.pem\",",
        "\"host_policy_id\":\"policy-1\",\"future\":{\"kept\":true}}\n"
    );

    fn message(kind: &str, extra: &str) -> Vec<u8> {
        format!(
            "{{\"protocol\":\"{PROTOCOL}\",\"type\":\"{kind}\",\"binding\":{{\"run_id\":\"run-A\",\"attempt\":2,\"effect_id\":\"effect-B\"}}{extra}}}\n"
        )
        .into_bytes()
    }

    #[test]
    fn start_is_typed_and_unknown_fields_are_preserved() {
        let parsed = parse_line(START.as_bytes(), 16_384).unwrap();
        let request = StartRequest::parse(&parsed).unwrap();
        assert_eq!(request.binding.run_id, "run-A");
        assert_eq!(request.requested_retention, RetentionMode::MetadataOnly);
        assert!(parsed.field("future").is_some());
        let reparsed = parse_line(&parsed.canonical_line(), 16_384).unwrap();
        assert_eq!(reparsed.field("future"), parsed.field("future"));
    }

    #[test]
    fn duplicate_keys_and_unknown_types_are_not_success() {
        let duplicate = b"{\"protocol\":\"wire-witness.sidecar/1\",\"protocol\":\"wire-witness.sidecar/1\",\"type\":\"ready\",\"binding\":{\"run_id\":\"r\",\"attempt\":1,\"effect_id\":\"e\"}}\n";
        assert_eq!(
            parse_line(duplicate, 4096),
            Err(ProtocolError::MalformedLine)
        );
        assert_eq!(
            parse_line(&message("future", ""), 4096),
            Err(ProtocolError::UnknownMessageType)
        );
    }

    #[test]
    fn invalid_binding_refuses_start() {
        let invalid = START.replace("effect-B", "");
        assert_eq!(
            parse_line(invalid.as_bytes(), 16_384),
            Err(ProtocolError::InvalidBinding)
        );
    }

    #[test]
    fn invalid_or_overlapping_authorities_refuse_start() {
        let invalid = START.replace("api.example.com:443", "*.example.com:443");
        let parsed = parse_line(invalid.as_bytes(), 16_384).unwrap();
        assert_eq!(
            StartRequest::parse(&parsed),
            Err(ProtocolError::InvalidField)
        );

        let overlap = START.replace("auth.example.com:443", "api.example.com:443");
        let parsed = parse_line(overlap.as_bytes(), 16_384).unwrap();
        assert_eq!(
            StartRequest::parse(&parsed),
            Err(ProtocolError::InvalidField)
        );
    }

    #[test]
    fn forbidden_content_and_private_key_fields_are_rejected() {
        assert_eq!(
            parse_line(&message("ready", ",\"private_key_path\":\"/bad\""), 4096),
            Err(ProtocolError::ForbiddenField)
        );
        assert_eq!(
            parse_line(&message("exchange", ",\"response_body\":\"bad\""), 4096),
            Err(ProtocolError::ForbiddenField)
        );
    }

    #[test]
    fn sidecar_order_and_exact_binding_are_enforced() {
        let start = parse_line(START.as_bytes(), 16_384).unwrap();
        let (mut session, _) = SidecarSession::from_start(&start).unwrap();
        let finding = parse_line(&message("finding", ""), 4096).unwrap();
        assert_eq!(
            session.observe_sidecar(&finding),
            Err(ProtocolError::UnexpectedMessage)
        );
        let ready = parse_line(&message("ready", ""), 4096).unwrap();
        session.observe_sidecar(&ready).unwrap();
        session.observe_sidecar(&finding).unwrap();
        let finished = parse_line(
            &message(
                "finished",
                ",\"completeness\":\"complete\",\"exchange_count\":1,\"finding_count\":0,\"manifest_path\":\"/attempt/manifest.json\",\"capture_digest\":\"abc\"",
            ),
            4096,
        )
        .unwrap();
        session.observe_sidecar(&finished).unwrap();
        assert!(session.is_finished());
        assert_eq!(
            session.observe_sidecar(&finding),
            Err(ProtocolError::MessageAfterTerminal)
        );
    }

    #[test]
    fn binding_mismatch_never_correlates() {
        let start = parse_line(START.as_bytes(), 16_384).unwrap();
        let (mut session, _) = SidecarSession::from_start(&start).unwrap();
        let other = String::from_utf8(message("ready", ""))
            .unwrap()
            .replace("run-A", "run-other");
        let other = parse_line(other.as_bytes(), 4096).unwrap();
        assert_eq!(
            session.observe_sidecar(&other),
            Err(ProtocolError::BindingMismatch)
        );
    }

    #[test]
    fn complete_finish_requires_manifest_and_digest() {
        let finished = parse_line(
            &message(
                "finished",
                ",\"completeness\":\"complete\",\"exchange_count\":0,\"finding_count\":0",
            ),
            4096,
        )
        .unwrap();
        assert_eq!(
            FinishedMessage::parse(&finished),
            Err(ProtocolError::InvalidField)
        );
    }

    #[test]
    fn evidence_reference_binds_exact_subject_and_manifest_bytes() {
        let subject = AttemptBinding {
            run_id: "run-A".into(),
            attempt: 2,
            effect_id: "effect-B".into(),
        };
        let reference = EvidenceReference::for_manifest(
            b"manifest",
            subject.clone(),
            CaptureCompleteness::Complete,
            Some("wire-witness".into()),
            Some("producer-digest".into()),
        );
        assert_eq!(reference.subject, subject);
        assert_eq!(reference.byte_length, 8);
        assert_eq!(
            reference.sha256,
            "05b3abf2579a5eb66403cd78be557fd860633a1fe2103c7642030defe32c657f"
        );
        let text = String::from_utf8(reference.canonical_bytes()).unwrap();
        assert!(text.contains("statecraft/wire-exchange/v1"));
        assert!(text.contains("effect-B"));
    }

    #[test]
    fn eof_shape_and_line_bounds_are_strict() {
        assert_eq!(
            parse_line(START.trim_end().as_bytes(), 16_384),
            Err(ProtocolError::MalformedLine)
        );
        assert_eq!(
            parse_line(START.as_bytes(), 8),
            Err(ProtocolError::OversizedLine)
        );
    }

    #[test]
    fn message_kind_round_trip_names_are_stable() {
        for (kind, name) in [
            (MessageKind::Start, "start"),
            (MessageKind::Ready, "ready"),
            (MessageKind::Finding, "finding"),
            (MessageKind::Exchange, "exchange"),
            (MessageKind::Finished, "finished"),
        ] {
            assert_eq!(kind.as_str(), name);
        }
    }
}

// endregion
