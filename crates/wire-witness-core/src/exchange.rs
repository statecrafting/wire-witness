//! `wire-witness.exchange/1` testimony and provider normalization.
//!
//! Governed by spec 002, exchange record and normalization.

// region: exchange-record-and-normalization

use std::collections::BTreeMap;

pub const SCHEMA: &str = "wire-witness.exchange/1";
pub const DIGEST_CONSTRUCTION: &str = "wire-witness.exchange/1+keysort-json+sha256";
pub const BYTE_DIGEST_CONSTRUCTION: &str = "file-bytes-sha256";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Presence<T> {
    Known(T),
    Unknown { reason: String },
    Absent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Completeness {
    Complete,
    Incomplete(IncompleteReason),
    Absent(IncompleteReason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IncompleteReason {
    Disabled,
    ConnectionFailed,
    StreamEnded,
    LimitExceeded,
    Malformed,
    SidecarStopped,
    SinkFailed,
    SinkBackpressure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderFamily {
    Anthropic,
    OpenAi,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ServedDisclosure {
    Pinned,
    Reported,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServedIdentity {
    pub identity: String,
    pub disclosure: ServedDisclosure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsageEntry {
    pub field_name: String,
    pub value_text: String,
    pub unit: Presence<String>,
    pub message_location: String,
    pub source: UsageSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReportedCost {
    pub value_text: String,
    pub currency_or_unit: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UsageSource {
    ProviderReported,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Cost {
    Reported {
        value_text: String,
        currency_or_unit: String,
    },
    Estimated {
        value_text: String,
        currency: String,
        rate_table_identity: String,
        usage_inputs: Vec<String>,
    },
    Unknown {
        reason: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ByteReference {
    pub original_length: u64,
    pub sha256: String,
}

impl ByteReference {
    #[must_use]
    pub fn observed(bytes: &[u8]) -> Self {
        Self {
            original_length: bytes.len() as u64,
            sha256: sha256_hex(bytes),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Binding {
    Supervised {
        run_id: String,
        attempt: u32,
        effect_id: String,
    },
    Unsupervised {
        session_id: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransportMetadata {
    pub protocol: String,
    pub authority: String,
    pub operation: String,
    pub request_path: String,
    pub response_status: Presence<u16>,
    pub stream_state: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionState {
    pub requested: String,
    pub effective: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    pub code: String,
    pub direction: String,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NormalizedEvent {
    Known {
        name: String,
        position: u64,
        content_block_index: Presence<u64>,
    },
    Unknown {
        name: String,
        position: u64,
        redacted_payload: ByteReference,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExchangeRecord {
    pub binding: Binding,
    pub sequence: u64,
    pub transport: TransportMetadata,
    pub provider: ProviderFamily,
    pub requested_identity: Presence<String>,
    pub served_identity: Presence<ServedIdentity>,
    pub usage: Vec<UsageEntry>,
    pub cost: Cost,
    pub retention: RetentionState,
    pub redaction_count: u64,
    pub completeness: Completeness,
    pub findings: Vec<Finding>,
    pub request_bytes: ByteReference,
    pub response_bytes: ByteReference,
    pub events: Vec<NormalizedEvent>,
}

impl ExchangeRecord {
    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        canonical_json_bytes(&self.as_json())
    }

    #[must_use]
    pub fn digest(&self) -> String {
        sha256_hex(&self.canonical_bytes())
    }

    fn as_json(&self) -> JsonValue {
        object([
            ("binding", binding_json(&self.binding)),
            ("completeness", completeness_json(&self.completeness)),
            ("cost", cost_json(&self.cost)),
            (
                "events",
                JsonValue::Array(self.events.iter().map(event_json).collect()),
            ),
            (
                "findings",
                JsonValue::Array(self.findings.iter().map(finding_json).collect()),
            ),
            ("provider", string(provider_name(&self.provider))),
            ("redaction_count", number(self.redaction_count.to_string())),
            ("request_bytes", byte_ref_json(&self.request_bytes)),
            (
                "requested_identity",
                presence_json(&self.requested_identity, |v| string(v)),
            ),
            ("response_bytes", byte_ref_json(&self.response_bytes)),
            (
                "retention",
                object([
                    ("effective", string(&self.retention.effective)),
                    ("requested", string(&self.retention.requested)),
                ]),
            ),
            ("schema", string(SCHEMA)),
            ("sequence", number(self.sequence.to_string())),
            (
                "served_identity",
                presence_json(&self.served_identity, |v| {
                    object([
                        ("disclosure", string(disclosure_name(&v.disclosure))),
                        ("identity", string(&v.identity)),
                    ])
                }),
            ),
            (
                "transport",
                object([
                    ("authority", string(&self.transport.authority)),
                    ("operation", string(&self.transport.operation)),
                    ("protocol", string(&self.transport.protocol)),
                    ("request_path", string(&self.transport.request_path)),
                    (
                        "response_status",
                        presence_json(&self.transport.response_status, |v| number(v.to_string())),
                    ),
                    ("stream_state", string(&self.transport.stream_state)),
                ]),
            ),
            (
                "usage",
                JsonValue::Array(self.usage.iter().map(usage_json).collect()),
            ),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Normalization {
    pub requested_identity: Presence<String>,
    pub served_identity: Presence<ServedIdentity>,
    pub usage: Vec<UsageEntry>,
    pub reported_cost: Presence<ReportedCost>,
    pub events: Vec<NormalizedEvent>,
    pub completeness: Completeness,
    pub findings: Vec<Finding>,
}

/// Normalizes one complete JSON message without repairing malformed input.
#[must_use]
pub fn normalize_provider_json(
    family: ProviderFamily,
    direction: &str,
    bytes: &[u8],
) -> Normalization {
    let malformed = |detail: String| Normalization {
        requested_identity: Presence::Unknown {
            reason: "malformed".into(),
        },
        served_identity: Presence::Unknown {
            reason: "malformed".into(),
        },
        usage: Vec::new(),
        reported_cost: Presence::Unknown {
            reason: "malformed".into(),
        },
        events: Vec::new(),
        completeness: Completeness::Incomplete(IncompleteReason::Malformed),
        findings: vec![Finding {
            code: "malformed-json".into(),
            direction: direction.into(),
            detail,
        }],
    };
    let root = match parse_json(bytes) {
        Ok(value) => value,
        Err(error) => return malformed(error),
    };
    let JsonValue::Object(map) = &root else {
        return malformed("provider message must be a JSON object".into());
    };
    let model_value = provider_field(&family, map, "model");
    let model = match model_value {
        Some(JsonValue::String(value)) => Presence::Known(value.clone()),
        Some(_) => return malformed("model must be a string".into()),
        None => Presence::Absent,
    };
    let (requested_identity, served_identity) = match direction {
        "request" => (
            model,
            Presence::Unknown {
                reason: "not-provider-response".into(),
            },
        ),
        "response" => (
            Presence::Unknown {
                reason: "not-observed-on-response".into(),
            },
            match model {
                Presence::Known(identity) => Presence::Known(ServedIdentity {
                    identity,
                    disclosure: ServedDisclosure::Reported,
                }),
                Presence::Absent => Presence::Unknown {
                    reason: "provider-did-not-report".into(),
                },
                Presence::Unknown { reason } => Presence::Unknown { reason },
            },
        ),
        _ => return malformed("direction must be request or response".into()),
    };
    let mut usage = Vec::new();
    if let Some(value) = provider_field(&family, map, "usage") {
        if !collect_usage(value, "usage", &mut usage) {
            return malformed("usage values must be integer or decimal numbers".into());
        }
    }
    let reported_cost = match provider_field(&family, map, "cost") {
        Some(JsonValue::Number(value) | JsonValue::String(value)) if is_decimal_text(value) => {
            let unit = provider_field(&family, map, "currency")
                .or_else(|| provider_field(&family, map, "unit"));
            match unit {
                Some(JsonValue::String(unit)) => Presence::Known(ReportedCost {
                    value_text: value.clone(),
                    currency_or_unit: unit.clone(),
                }),
                Some(_) => return malformed("cost currency or unit must be a string".into()),
                None => Presence::Unknown {
                    reason: "reported-cost-unit-missing".into(),
                },
            }
        }
        Some(_) => return malformed("cost must be integer or decimal text".into()),
        None => Presence::Absent,
    };
    let mut events = Vec::new();
    if let Some(JsonValue::String(name)) = map.get("type") {
        let known = is_known_event(&family, name);
        events.push(if known {
            NormalizedEvent::Known {
                name: name.clone(),
                position: 0,
                content_block_index: map
                    .get("index")
                    .and_then(JsonValue::as_u64)
                    .map_or(Presence::Absent, Presence::Known),
            }
        } else {
            NormalizedEvent::Unknown {
                name: name.clone(),
                position: 0,
                redacted_payload: ByteReference::observed(bytes),
            }
        });
    }
    Normalization {
        requested_identity,
        served_identity,
        usage,
        reported_cost,
        events,
        completeness: Completeness::Complete,
        findings: Vec::new(),
    }
}

/// Normalizes SSE frames in observed order. Each frame must have a valid
/// `event` field and exactly one JSON `data` value.
#[must_use]
pub fn normalize_sse(family: ProviderFamily, bytes: &[u8]) -> Normalization {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return incomplete_stream("invalid-utf8");
    };
    let mut result = Normalization {
        requested_identity: Presence::Unknown {
            reason: "stream-request-not-provided".into(),
        },
        served_identity: Presence::Unknown {
            reason: "provider-did-not-report".into(),
        },
        usage: Vec::new(),
        reported_cost: Presence::Absent,
        events: Vec::new(),
        completeness: Completeness::Complete,
        findings: Vec::new(),
    };
    let mut terminal_seen = false;
    for (position, frame) in text.split("\n\n").filter(|f| !f.is_empty()).enumerate() {
        if terminal_seen {
            return incomplete_stream_after(result, "event-after-terminal");
        }
        let mut event_name = None;
        let mut data = Vec::new();
        for line in frame.lines() {
            if let Some(value) = line.strip_prefix("event:") {
                event_name = Some(value.trim().to_owned());
            } else if let Some(value) = line.strip_prefix("data:") {
                data.push(value.trim());
            } else if !line.starts_with(':') {
                return incomplete_stream_after(result, "malformed-sse-field");
            }
        }
        let Some(name) = event_name else {
            return incomplete_stream_after(result, "missing-sse-field");
        };
        if data.is_empty() {
            return incomplete_stream_after(result, "missing-sse-field");
        }
        let payload = data.join("\n");
        let parsed = match parse_json(payload.as_bytes()) {
            Ok(value) => value,
            Err(_) => return incomplete_stream_after(result, "malformed-sse-json"),
        };
        if let JsonValue::Object(map) = &parsed {
            if let Some(JsonValue::String(model)) = provider_field(&family, map, "model") {
                result.served_identity = Presence::Known(ServedIdentity {
                    identity: model.clone(),
                    disclosure: ServedDisclosure::Reported,
                });
            }
            if let Some(value) = provider_field(&family, map, "usage") {
                if !collect_usage(value, "usage", &mut result.usage) {
                    return incomplete_stream_after(result, "invalid-usage");
                }
            }
            if let Some(cost) = provider_field(&family, map, "cost") {
                let unit = provider_field(&family, map, "currency")
                    .or_else(|| provider_field(&family, map, "unit"));
                if let (
                    JsonValue::Number(value) | JsonValue::String(value),
                    Some(JsonValue::String(unit)),
                ) = (cost, unit)
                {
                    if is_decimal_text(value) {
                        result.reported_cost = Presence::Known(ReportedCost {
                            value_text: value.clone(),
                            currency_or_unit: unit.clone(),
                        });
                    } else {
                        return incomplete_stream_after(result, "invalid-cost");
                    }
                } else {
                    return incomplete_stream_after(result, "invalid-cost");
                }
            }
        }
        let is_terminal = matches!(
            name.as_str(),
            "message_stop"
                | "response.completed"
                | "response.failed"
                | "response.incomplete"
                | "done"
        );
        result.events.push(if is_known_event(&family, &name) {
            let index = parsed
                .get("index")
                .and_then(JsonValue::as_u64)
                .map_or(Presence::Absent, Presence::Known);
            NormalizedEvent::Known {
                name,
                position: position as u64,
                content_block_index: index,
            }
        } else {
            NormalizedEvent::Unknown {
                name,
                position: position as u64,
                redacted_payload: ByteReference::observed(payload.as_bytes()),
            }
        });
        terminal_seen = is_terminal;
    }
    result
}

fn incomplete_stream(detail: &str) -> Normalization {
    incomplete_stream_after(
        Normalization {
            requested_identity: Presence::Unknown {
                reason: "stream-request-not-provided".into(),
            },
            served_identity: Presence::Unknown {
                reason: "stream-incomplete".into(),
            },
            usage: Vec::new(),
            reported_cost: Presence::Absent,
            events: Vec::new(),
            completeness: Completeness::Incomplete(IncompleteReason::Malformed),
            findings: Vec::new(),
        },
        detail,
    )
}

fn incomplete_stream_after(mut result: Normalization, detail: &str) -> Normalization {
    result.completeness = Completeness::Incomplete(IncompleteReason::Malformed);
    result.findings.push(Finding {
        code: "malformed-sse".into(),
        direction: "response".into(),
        detail: detail.into(),
    });
    result
}

fn collect_usage(value: &JsonValue, path: &str, out: &mut Vec<UsageEntry>) -> bool {
    match value {
        JsonValue::Object(map) => map.iter().all(|(key, value)| {
            let nested = format!("{path}.{key}");
            match value {
                JsonValue::Number(number) => {
                    out.push(UsageEntry {
                        field_name: key.clone(),
                        value_text: number.clone(),
                        unit: Presence::Absent,
                        message_location: nested,
                        source: UsageSource::ProviderReported,
                    });
                    true
                }
                JsonValue::String(number) if is_decimal_text(number) => {
                    out.push(UsageEntry {
                        field_name: key.clone(),
                        value_text: number.clone(),
                        unit: Presence::Absent,
                        message_location: nested,
                        source: UsageSource::ProviderReported,
                    });
                    true
                }
                JsonValue::Object(_) => collect_usage(value, &nested, out),
                _ => false,
            }
        }),
        _ => false,
    }
}

fn provider_field<'a>(
    family: &ProviderFamily,
    map: &'a BTreeMap<String, JsonValue>,
    field: &str,
) -> Option<&'a JsonValue> {
    map.get(field).or_else(|| {
        let container = match family {
            ProviderFamily::Anthropic => "message",
            ProviderFamily::OpenAi => "response",
            ProviderFamily::Unknown => return None,
        };
        match map.get(container) {
            Some(JsonValue::Object(nested)) => nested.get(field),
            _ => None,
        }
    })
}

fn is_decimal_text(value: &str) -> bool {
    if value.is_empty() || value.trim() != value {
        return false;
    }
    let bytes = value.as_bytes();
    let mut index = usize::from(bytes.first() == Some(&b'-'));
    let integer_start = index;
    while bytes.get(index).is_some_and(u8::is_ascii_digit) {
        index += 1;
    }
    if index == integer_start {
        return false;
    }
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let fraction_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if index == fraction_start {
            return false;
        }
    }
    index == bytes.len()
}

fn is_known_event(family: &ProviderFamily, name: &str) -> bool {
    match family {
        ProviderFamily::Anthropic => matches!(
            name,
            "message_start"
                | "content_block_start"
                | "content_block_delta"
                | "content_block_stop"
                | "message_delta"
                | "message_stop"
                | "ping"
                | "error"
        ),
        ProviderFamily::OpenAi => matches!(
            name,
            "response.created"
                | "response.in_progress"
                | "response.output_item.added"
                | "response.output_item.done"
                | "response.content_part.added"
                | "response.content_part.done"
                | "response.output_text.delta"
                | "response.output_text.done"
                | "response.completed"
                | "response.failed"
                | "response.incomplete"
                | "response.error"
                | "chat.completion.chunk"
                | "done"
        ),
        ProviderFamily::Unknown => false,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

impl JsonValue {
    fn get(&self, key: &str) -> Option<&Self> {
        match self {
            Self::Object(map) => map.get(key),
            _ => None,
        }
    }

    fn as_u64(&self) -> Option<u64> {
        match self {
            Self::Number(value) => value.parse().ok(),
            _ => None,
        }
    }
}

/// Parses strict UTF-8 JSON and retains number spelling and object meaning.
pub fn parse_json(bytes: &[u8]) -> Result<JsonValue, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "invalid UTF-8".to_owned())?;
    let mut parser = Parser {
        bytes: text.as_bytes(),
        offset: 0,
    };
    let value = parser.value()?;
    parser.space();
    if parser.offset != parser.bytes.len() {
        return Err("trailing JSON content".into());
    }
    Ok(value)
}

#[must_use]
pub fn canonical_json_bytes(value: &JsonValue) -> Vec<u8> {
    let mut output = String::new();
    write_json(value, &mut output);
    output.into_bytes()
}

fn write_json(value: &JsonValue, out: &mut String) {
    match value {
        JsonValue::Null => out.push_str("null"),
        JsonValue::Bool(value) => out.push_str(if *value { "true" } else { "false" }),
        JsonValue::Number(value) => out.push_str(value),
        JsonValue::String(value) => {
            out.push('"');
            for character in value.chars() {
                match character {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\u{08}' => out.push_str("\\b"),
                    '\u{0c}' => out.push_str("\\f"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c if c < '\u{20}' => {
                        use std::fmt::Write;
                        write!(out, "\\u{:04x}", c as u32).expect("string write");
                    }
                    c => out.push(c),
                }
            }
            out.push('"');
        }
        JsonValue::Array(values) => {
            out.push('[');
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    out.push(',');
                }
                write_json(value, out);
            }
            out.push(']');
        }
        JsonValue::Object(map) => {
            out.push('{');
            for (index, (key, value)) in map.iter().enumerate() {
                if index != 0 {
                    out.push(',');
                }
                write_json(&JsonValue::String(key.clone()), out);
                out.push(':');
                write_json(value, out);
            }
            out.push('}');
        }
    }
}

struct Parser<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl Parser<'_> {
    fn space(&mut self) {
        while self
            .bytes
            .get(self.offset)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.offset += 1;
        }
    }
    fn value(&mut self) -> Result<JsonValue, String> {
        self.space();
        match self.bytes.get(self.offset) {
            Some(b'n') => {
                self.literal(b"null")?;
                Ok(JsonValue::Null)
            }
            Some(b't') => {
                self.literal(b"true")?;
                Ok(JsonValue::Bool(true))
            }
            Some(b'f') => {
                self.literal(b"false")?;
                Ok(JsonValue::Bool(false))
            }
            Some(b'"') => self.string().map(JsonValue::String),
            Some(b'[') => self.array(),
            Some(b'{') => self.object(),
            Some(b'-' | b'0'..=b'9') => self.number().map(JsonValue::Number),
            _ => Err(format!("invalid JSON value at byte {}", self.offset)),
        }
    }
    fn literal(&mut self, literal: &[u8]) -> Result<(), String> {
        if self.bytes.get(self.offset..self.offset + literal.len()) == Some(literal) {
            self.offset += literal.len();
            Ok(())
        } else {
            Err("invalid literal".into())
        }
    }
    fn string(&mut self) -> Result<String, String> {
        self.offset += 1;
        let mut output = String::new();
        while let Some(&byte) = self.bytes.get(self.offset) {
            self.offset += 1;
            match byte {
                b'"' => return Ok(output),
                b'\\' => {
                    let escape = *self.bytes.get(self.offset).ok_or("incomplete escape")?;
                    self.offset += 1;
                    match escape {
                        b'"' => output.push('"'),
                        b'\\' => output.push('\\'),
                        b'/' => output.push('/'),
                        b'b' => output.push('\u{08}'),
                        b'f' => output.push('\u{0c}'),
                        b'n' => output.push('\n'),
                        b'r' => output.push('\r'),
                        b't' => output.push('\t'),
                        b'u' => {
                            let code = self.hex4()?;
                            if (0xd800..=0xdbff).contains(&code) {
                                if self.bytes.get(self.offset..self.offset + 2) != Some(b"\\u") {
                                    return Err("missing low surrogate".into());
                                }
                                self.offset += 2;
                                let low = self.hex4()?;
                                if !(0xdc00..=0xdfff).contains(&low) {
                                    return Err("invalid low surrogate".into());
                                }
                                let scalar = 0x10000
                                    + (((code - 0xd800) as u32) << 10)
                                    + (low - 0xdc00) as u32;
                                output
                                    .push(char::from_u32(scalar).ok_or("invalid unicode scalar")?);
                            } else if (0xdc00..=0xdfff).contains(&code) {
                                return Err("unpaired low surrogate".into());
                            } else {
                                output.push(
                                    char::from_u32(code as u32).ok_or("invalid unicode scalar")?,
                                );
                            }
                        }
                        _ => return Err("invalid escape".into()),
                    }
                }
                0..=31 => return Err("control character in string".into()),
                32..=127 => output.push(byte as char),
                _ => {
                    self.offset -= 1;
                    let rest = std::str::from_utf8(&self.bytes[self.offset..])
                        .map_err(|_| "invalid UTF-8")?;
                    let character = rest.chars().next().ok_or("incomplete UTF-8")?;
                    self.offset += character.len_utf8();
                    output.push(character);
                }
            }
        }
        Err("unterminated string".into())
    }
    fn hex4(&mut self) -> Result<u16, String> {
        let end = self.offset + 4;
        let digits = self
            .bytes
            .get(self.offset..end)
            .ok_or("incomplete unicode escape")?;
        let text = std::str::from_utf8(digits).map_err(|_| "invalid unicode escape")?;
        let value = u16::from_str_radix(text, 16).map_err(|_| "invalid unicode escape")?;
        self.offset = end;
        Ok(value)
    }
    fn number(&mut self) -> Result<String, String> {
        let start = self.offset;
        if self.bytes.get(self.offset) == Some(&b'-') {
            self.offset += 1;
        }
        match self.bytes.get(self.offset) {
            Some(b'0') => self.offset += 1,
            Some(b'1'..=b'9') => {
                self.offset += 1;
                while self.bytes.get(self.offset).is_some_and(u8::is_ascii_digit) {
                    self.offset += 1;
                }
            }
            _ => return Err("invalid number".into()),
        }
        if self.bytes.get(self.offset) == Some(&b'.') {
            self.offset += 1;
            let before = self.offset;
            while self.bytes.get(self.offset).is_some_and(u8::is_ascii_digit) {
                self.offset += 1;
            }
            if before == self.offset {
                return Err("invalid fraction".into());
            }
        }
        if matches!(self.bytes.get(self.offset), Some(b'e' | b'E')) {
            self.offset += 1;
            if matches!(self.bytes.get(self.offset), Some(b'+' | b'-')) {
                self.offset += 1;
            }
            let before = self.offset;
            while self.bytes.get(self.offset).is_some_and(u8::is_ascii_digit) {
                self.offset += 1;
            }
            if before == self.offset {
                return Err("invalid exponent".into());
            }
        }
        Ok(std::str::from_utf8(&self.bytes[start..self.offset])
            .expect("number is ASCII")
            .to_owned())
    }
    fn array(&mut self) -> Result<JsonValue, String> {
        self.offset += 1;
        let mut values = Vec::new();
        self.space();
        if self.bytes.get(self.offset) == Some(&b']') {
            self.offset += 1;
            return Ok(JsonValue::Array(values));
        }
        loop {
            values.push(self.value()?);
            self.space();
            match self.bytes.get(self.offset) {
                Some(b',') => self.offset += 1,
                Some(b']') => {
                    self.offset += 1;
                    break;
                }
                _ => return Err("invalid array separator".into()),
            }
        }
        Ok(JsonValue::Array(values))
    }
    fn object(&mut self) -> Result<JsonValue, String> {
        self.offset += 1;
        let mut values = BTreeMap::new();
        self.space();
        if self.bytes.get(self.offset) == Some(&b'}') {
            self.offset += 1;
            return Ok(JsonValue::Object(values));
        }
        loop {
            self.space();
            if self.bytes.get(self.offset) != Some(&b'"') {
                return Err("object key must be a string".into());
            }
            let key = self.string()?;
            self.space();
            if self.bytes.get(self.offset) != Some(&b':') {
                return Err("missing object colon".into());
            }
            self.offset += 1;
            let value = self.value()?;
            if values.insert(key, value).is_some() {
                return Err("duplicate object key".into());
            }
            self.space();
            match self.bytes.get(self.offset) {
                Some(b',') => self.offset += 1,
                Some(b'}') => {
                    self.offset += 1;
                    break;
                }
                _ => return Err("invalid object separator".into()),
            }
        }
        Ok(JsonValue::Object(values))
    }
}

#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let words = sha256(bytes);
    let mut output = String::with_capacity(64);
    use std::fmt::Write;
    for word in words {
        write!(output, "{word:08x}").expect("string write");
    }
    output
}

fn sha256(input: &[u8]) -> [u32; 8] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let bit_len = (input.len() as u64).wrapping_mul(8);
    let mut data = input.to_vec();
    data.push(0x80);
    while data.len() % 64 != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_be_bytes());
    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    for chunk in data.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, bytes) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes(bytes.try_into().expect("four bytes"));
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (slot, value) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = (*slot).wrapping_add(value);
        }
    }
    state
}

fn object<const N: usize>(entries: [(&str, JsonValue); N]) -> JsonValue {
    JsonValue::Object(
        entries
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v))
            .collect(),
    )
}
fn string(value: &str) -> JsonValue {
    JsonValue::String(value.to_owned())
}
fn number(value: String) -> JsonValue {
    JsonValue::Number(value)
}
fn presence_json<T>(value: &Presence<T>, convert: impl Fn(&T) -> JsonValue) -> JsonValue {
    match value {
        Presence::Known(value) => object([("status", string("known")), ("value", convert(value))]),
        Presence::Unknown { reason } => {
            object([("reason", string(reason)), ("status", string("unknown"))])
        }
        Presence::Absent => object([("status", string("absent"))]),
    }
}
fn provider_name(value: &ProviderFamily) -> &'static str {
    match value {
        ProviderFamily::Anthropic => "anthropic",
        ProviderFamily::OpenAi => "openai",
        ProviderFamily::Unknown => "unknown",
    }
}
fn disclosure_name(value: &ServedDisclosure) -> &'static str {
    match value {
        ServedDisclosure::Pinned => "pinned",
        ServedDisclosure::Reported => "reported",
        ServedDisclosure::Unknown => "unknown",
    }
}
fn reason_name(value: &IncompleteReason) -> &'static str {
    match value {
        IncompleteReason::Disabled => "disabled",
        IncompleteReason::ConnectionFailed => "connection-failed",
        IncompleteReason::StreamEnded => "stream-ended",
        IncompleteReason::LimitExceeded => "limit-exceeded",
        IncompleteReason::Malformed => "malformed",
        IncompleteReason::SidecarStopped => "sidecar-stopped",
        IncompleteReason::SinkFailed => "sink-failed",
        IncompleteReason::SinkBackpressure => "sink-backpressure",
    }
}
fn binding_json(value: &Binding) -> JsonValue {
    match value {
        Binding::Supervised {
            run_id,
            attempt,
            effect_id,
        } => object([
            ("attempt", number(attempt.to_string())),
            ("effect_id", string(effect_id)),
            ("kind", string("supervised")),
            ("run_id", string(run_id)),
        ]),
        Binding::Unsupervised { session_id } => object([
            ("kind", string("unsupervised")),
            ("session_id", string(session_id)),
        ]),
    }
}
fn completeness_json(value: &Completeness) -> JsonValue {
    match value {
        Completeness::Complete => object([("status", string("complete"))]),
        Completeness::Incomplete(reason) => object([
            ("reason", string(reason_name(reason))),
            ("status", string("incomplete")),
        ]),
        Completeness::Absent(reason) => object([
            ("reason", string(reason_name(reason))),
            ("status", string("absent")),
        ]),
    }
}
fn byte_ref_json(value: &ByteReference) -> JsonValue {
    object([
        ("construction", string(BYTE_DIGEST_CONSTRUCTION)),
        ("original_length", number(value.original_length.to_string())),
        ("sha256", string(&value.sha256)),
    ])
}
fn cost_json(value: &Cost) -> JsonValue {
    match value {
        Cost::Reported {
            value_text,
            currency_or_unit,
        } => object([
            ("currency_or_unit", string(currency_or_unit)),
            ("status", string("reported")),
            ("value", string(value_text)),
        ]),
        Cost::Estimated {
            value_text,
            currency,
            rate_table_identity,
            usage_inputs,
        } => object([
            ("currency", string(currency)),
            ("rate_table_identity", string(rate_table_identity)),
            ("status", string("estimated")),
            (
                "usage_inputs",
                JsonValue::Array(usage_inputs.iter().map(|v| string(v)).collect()),
            ),
            ("value", string(value_text)),
        ]),
        Cost::Unknown { reason } => {
            object([("reason", string(reason)), ("status", string("unknown"))])
        }
    }
}
fn usage_json(value: &UsageEntry) -> JsonValue {
    object([
        ("field_name", string(&value.field_name)),
        ("message_location", string(&value.message_location)),
        ("source", string("provider-reported")),
        ("unit", presence_json(&value.unit, |v| string(v))),
        ("value", string(&value.value_text)),
    ])
}
fn event_json(value: &NormalizedEvent) -> JsonValue {
    match value {
        NormalizedEvent::Known {
            name,
            position,
            content_block_index,
        } => object([
            (
                "content_block_index",
                presence_json(content_block_index, |v| number(v.to_string())),
            ),
            ("kind", string("known")),
            ("name", string(name)),
            ("position", number(position.to_string())),
        ]),
        NormalizedEvent::Unknown {
            name,
            position,
            redacted_payload,
        } => object([
            ("kind", string("unknown")),
            ("name", string(name)),
            ("position", number(position.to_string())),
            ("redacted_payload", byte_ref_json(redacted_payload)),
        ]),
    }
}
fn finding_json(value: &Finding) -> JsonValue {
    object([
        ("code", string(&value.code)),
        ("detail", string(&value.detail)),
        ("direction", string(&value.direction)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_published_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn canonical_json_sorts_keys_and_preserves_arrays() {
        let left = parse_json(br#"{"z":1,"a":{"b":2,"a":3},"items":[2,1]}"#).unwrap();
        let right = parse_json(br#"{"items":[2,1],"a":{"a":3,"b":2},"z":1}"#).unwrap();
        assert_eq!(canonical_json_bytes(&left), canonical_json_bytes(&right));
        assert_eq!(
            canonical_json_bytes(&left),
            br#"{"a":{"a":3,"b":2},"items":[2,1],"z":1}"#
        );
    }

    #[test]
    fn requested_identity_never_fills_missing_served_identity() {
        let request = normalize_provider_json(
            ProviderFamily::Anthropic,
            "request",
            br#"{"model":"claude-x"}"#,
        );
        let response = normalize_provider_json(
            ProviderFamily::Anthropic,
            "response",
            br#"{"type":"message_stop"}"#,
        );
        assert_eq!(
            request.requested_identity,
            Presence::Known("claude-x".into())
        );
        assert!(matches!(response.served_identity, Presence::Unknown { .. }));
    }

    #[test]
    fn nested_usage_names_and_decimal_spelling_are_preserved() {
        let result = normalize_provider_json(
            ProviderFamily::OpenAi,
            "response",
            br#"{"usage":{"input_tokens":7,"input_tokens_details":{"cached_tokens":2.50}}}"#,
        );
        assert_eq!(
            result
                .usage
                .iter()
                .map(|u| (&u.field_name, &u.value_text, &u.message_location))
                .collect::<Vec<_>>(),
            vec![
                (
                    &"input_tokens".into(),
                    &"7".into(),
                    &"usage.input_tokens".into()
                ),
                (
                    &"cached_tokens".into(),
                    &"2.50".into(),
                    &"usage.input_tokens_details.cached_tokens".into()
                )
            ]
        );
    }

    #[test]
    fn nested_provider_fields_and_reported_cost_remain_attributed() {
        let anthropic = normalize_sse(
            ProviderFamily::Anthropic,
            b"event: message_start\ndata: {\"message\":{\"model\":\"claude-x\",\"usage\":{\"input_tokens\":\"7\"}}}\n\n",
        );
        let openai = normalize_provider_json(
            ProviderFamily::OpenAi,
            "response",
            br#"{"response":{"model":"gpt-x","cost":"0.0120","currency":"USD"}}"#,
        );
        assert!(
            matches!(anthropic.served_identity, Presence::Known(ServedIdentity { identity, .. }) if identity == "claude-x")
        );
        assert_eq!(anthropic.usage[0].value_text, "7");
        assert!(
            matches!(openai.reported_cost, Presence::Known(ReportedCost { value_text, currency_or_unit }) if value_text == "0.0120" && currency_or_unit == "USD")
        );
    }

    #[test]
    fn sse_multiple_data_lines_follow_event_stream_framing() {
        let result = normalize_sse(
            ProviderFamily::OpenAi,
            b"event: response.completed\ndata: {\"response\":\ndata: {\"model\":\"gpt-x\"}}\n\n",
        );
        assert!(matches!(result.completeness, Completeness::Complete));
        assert!(
            matches!(result.served_identity, Presence::Known(ServedIdentity { identity, .. }) if identity == "gpt-x")
        );
    }

    #[test]
    fn unknown_event_is_ordered_and_payload_is_not_retained() {
        let result = normalize_sse(
            ProviderFamily::Anthropic,
            b"event: future_event\ndata: {\"secret\":\"not-retained\"}\n\n",
        );
        assert!(
            matches!(result.events.as_slice(), [NormalizedEvent::Unknown { name, position: 0, .. }] if name == "future_event")
        );
    }

    #[test]
    fn malformed_input_is_never_normalized_as_success() {
        let json = normalize_provider_json(ProviderFamily::OpenAi, "response", b"{broken");
        let sse = normalize_sse(
            ProviderFamily::OpenAi,
            b"event: response.done\ndata: nope\n\n",
        );
        assert!(matches!(
            json.completeness,
            Completeness::Incomplete(IncompleteReason::Malformed)
        ));
        assert!(matches!(
            sse.completeness,
            Completeness::Incomplete(IncompleteReason::Malformed)
        ));
        assert!(!json.findings.is_empty() && !sse.findings.is_empty());
    }

    #[test]
    fn malformed_terminal_retains_preceding_events() {
        let result = normalize_sse(
            ProviderFamily::Anthropic,
            b"event: content_block_delta\ndata: {\"index\":0}\n\nevent: message_stop\ndata: nope\n\n",
        );
        assert_eq!(result.events.len(), 1);
        assert!(matches!(
            result.completeness,
            Completeness::Incomplete(IncompleteReason::Malformed)
        ));
    }

    #[test]
    fn event_after_terminal_is_contradictory() {
        let result = normalize_sse(
            ProviderFamily::OpenAi,
            b"event: response.completed\ndata: {}\n\nevent: response.output_text.delta\ndata: {}\n\n",
        );
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.findings[0].detail, "event-after-terminal");
    }

    #[test]
    fn duplicate_keys_are_malformed_not_last_value_wins() {
        assert!(parse_json(br#"{"model":"a","model":"b"}"#).is_err());
    }
}

// endregion
