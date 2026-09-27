//! Pure redaction, retention, and custody decisions.
//!
//! Governed by spec 003, redaction, custody, and retention.

// region: redaction-custody-and-retention

use action_gate_core::secrets::{self, SecretRules};

use crate::exchange::{JsonValue, canonical_json_bytes, parse_json, sha256_hex};

const REDACTED: &str = "[REDACTED]";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RetentionMode {
    #[default]
    MetadataOnly,
    Content,
    Disabled,
}

impl RetentionMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MetadataOnly => "metadata-only",
            Self::Content => "content",
            Self::Disabled => "disabled",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BodyKind {
    Json,
    FormUrlEncoded,
    Text,
    Compressed,
    Encrypted,
    Binary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeaderField {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RedactionCategory {
    SensitiveHeader,
    SensitiveField,
    DetectedCredential,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RedactionRecord {
    pub category: RedactionCategory,
    pub detector_id: Option<String>,
    pub offset: usize,
    pub removed_length: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustodyFinding {
    pub code: String,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ByteMetadata {
    pub original_length: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionResult {
    pub requested: RetentionMode,
    pub effective: RetentionMode,
    pub byte_metadata: Option<ByteMetadata>,
    pub retained_content: Option<Vec<u8>>,
    pub redactions: Vec<RedactionRecord>,
    pub findings: Vec<CustodyFinding>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryLockPosture {
    Locked,
    Unavailable,
    Failed,
    NotAttempted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustodyMetadata {
    pub requested: RetentionMode,
    pub effective: RetentionMode,
    pub retention_deadline: Option<String>,
    pub erasure_policy_id: Option<String>,
    pub memory_lock: MemoryLockPosture,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RedactedText {
    value: String,
    records: Vec<RedactionRecord>,
}

/// Produces a redacted capture copy of headers without changing forwarding data.
#[must_use]
pub fn redact_headers(headers: &[HeaderField]) -> (Vec<HeaderField>, Vec<RedactionRecord>) {
    let mut output = Vec::with_capacity(headers.len());
    let mut records = Vec::new();
    for header in headers {
        let redacted_name = redact_detected(&header.name);
        records.extend(redacted_name.records);
        let value = if sensitive_header(&header.name) || bearer_value(&header.value) {
            records.push(RedactionRecord {
                category: RedactionCategory::SensitiveHeader,
                detector_id: None,
                offset: 0,
                removed_length: header.value.len(),
            });
            REDACTED.to_owned()
        } else {
            let redacted_value = redact_detected(&header.value);
            records.extend(redacted_value.records);
            redacted_value.value
        };
        output.push(HeaderField {
            name: redacted_name.value,
            value,
        });
    }
    (output, records)
}

/// Applies the requested retention to one body after mandatory redaction.
#[must_use]
pub fn retain_body(requested: RetentionMode, kind: BodyKind, body: &[u8]) -> RetentionResult {
    if requested == RetentionMode::Disabled {
        return RetentionResult {
            requested,
            effective: RetentionMode::Disabled,
            byte_metadata: None,
            retained_content: None,
            redactions: Vec::new(),
            findings: Vec::new(),
        };
    }

    let byte_metadata = Some(ByteMetadata {
        original_length: body.len() as u64,
        sha256: sha256_hex(body),
    });
    let mut findings = Vec::new();
    let redacted = match redact_body(&kind, body) {
        Ok(value) => value,
        Err(detail) => {
            findings.push(CustodyFinding {
                code: "content-not-retained".into(),
                detail: detail.into(),
            });
            return RetentionResult {
                requested,
                effective: RetentionMode::MetadataOnly,
                byte_metadata,
                retained_content: None,
                redactions: Vec::new(),
                findings,
            };
        }
    };
    let retained_content =
        (requested == RetentionMode::Content).then_some(redacted.value.into_bytes());
    RetentionResult {
        requested,
        effective: if requested == RetentionMode::Content {
            RetentionMode::Content
        } else {
            RetentionMode::MetadataOnly
        },
        byte_metadata,
        retained_content,
        redactions: redacted.records,
        findings,
    }
}

fn redact_body(kind: &BodyKind, body: &[u8]) -> Result<RedactedText, &'static str> {
    match kind {
        BodyKind::Json => redact_json(body),
        BodyKind::FormUrlEncoded => redact_form(body),
        BodyKind::Text => {
            let text = std::str::from_utf8(body).map_err(|_| "text-invalid-utf8")?;
            Ok(redact_detected(text))
        }
        BodyKind::Compressed => Err("compressed-body-unscannable"),
        BodyKind::Encrypted => Err("encrypted-body-unscannable"),
        BodyKind::Binary => Err("binary-body-unscannable"),
    }
}

fn redact_json(body: &[u8]) -> Result<RedactedText, &'static str> {
    let mut value = parse_json(body).map_err(|_| "malformed-json")?;
    let mut records = Vec::new();
    redact_json_value(&mut value, false, &mut records);
    let value = String::from_utf8(canonical_json_bytes(&value)).expect("canonical JSON is UTF-8");
    Ok(RedactedText { value, records })
}

fn redact_json_value(value: &mut JsonValue, sensitive: bool, records: &mut Vec<RedactionRecord>) {
    if sensitive {
        let removed_length = canonical_json_bytes(value).len();
        *value = JsonValue::String(REDACTED.into());
        records.push(RedactionRecord {
            category: RedactionCategory::SensitiveField,
            detector_id: None,
            offset: 0,
            removed_length,
        });
        return;
    }
    match value {
        JsonValue::String(text) => {
            let redacted = redact_detected(text);
            *text = redacted.value;
            records.extend(redacted.records);
        }
        JsonValue::Array(values) => {
            for child in values {
                redact_json_value(child, false, records);
            }
        }
        JsonValue::Object(values) => {
            let original = std::mem::take(values);
            for (index, (key, mut child)) in original.into_iter().enumerate() {
                let redacted_key = redact_detected(&key);
                let output_key = if redacted_key.records.is_empty() {
                    key.clone()
                } else {
                    format!("[REDACTED-KEY-{index}]")
                };
                records.extend(redacted_key.records);
                redact_json_value(&mut child, sensitive_field(&key), records);
                values.insert(output_key, child);
            }
        }
        JsonValue::Null | JsonValue::Bool(_) | JsonValue::Number(_) => {}
    }
}

fn redact_form(body: &[u8]) -> Result<RedactedText, &'static str> {
    let text = std::str::from_utf8(body).map_err(|_| "form-invalid-utf8")?;
    let mut output = String::with_capacity(text.len());
    let mut records = Vec::new();
    for (index, field) in text.split('&').enumerate() {
        if index != 0 {
            output.push('&');
        }
        let (name, value) = field.split_once('=').unwrap_or((field, ""));
        let decoded_name = percent_decode_ascii(name).ok_or("malformed-form-encoding")?;
        let redacted_name = redact_detected(&decoded_name);
        if redacted_name.records.is_empty() {
            output.push_str(name);
        } else {
            records.extend(redacted_name.records);
            output.push_str(REDACTED);
        }
        if field.contains('=') {
            output.push('=');
        }
        let decoded_value = percent_decode_ascii(value).ok_or("malformed-form-encoding")?;
        if sensitive_field(&decoded_name) {
            records.push(RedactionRecord {
                category: RedactionCategory::SensitiveField,
                detector_id: None,
                offset: 0,
                removed_length: value.len(),
            });
            output.push_str(REDACTED);
        } else {
            let redacted = redact_detected(&decoded_value);
            if redacted.records.is_empty() {
                output.push_str(value);
            } else {
                records.extend(redacted.records.into_iter().map(|record| RedactionRecord {
                    offset: 0,
                    removed_length: value.len(),
                    ..record
                }));
                output.push_str(REDACTED);
            }
        }
    }
    Ok(RedactedText {
        value: output,
        records,
    })
}

fn redact_detected(text: &str) -> RedactedText {
    let rules = SecretRules::default();
    let mut output = String::with_capacity(text.len());
    let mut records = Vec::new();
    let mut cursor = 0;
    while cursor < text.len() {
        let Some(finding) = secrets::scan(&text[cursor..], &rules) else {
            output.push_str(&text[cursor..]);
            break;
        };
        let start = cursor + finding.offset;
        output.push_str(&text[cursor..start]);
        let end = detected_end(text, start, finding.detector.as_str());
        output.push_str(REDACTED);
        records.push(RedactionRecord {
            category: RedactionCategory::DetectedCredential,
            detector_id: Some(finding.detector.as_str().to_owned()),
            offset: start,
            removed_length: end.saturating_sub(start),
        });
        cursor = end;
    }
    if cursor == text.len() && text.is_empty() {
        output.clear();
    }
    RedactedText {
        value: output,
        records,
    }
}

fn detected_end(text: &str, start: usize, detector: &str) -> usize {
    if detector.contains("private-key") {
        return text.len();
    }
    let rest = &text[start..];
    let length = rest
        .char_indices()
        .find(|(_, character)| {
            character.is_whitespace()
                || matches!(
                    character,
                    '"' | '\''
                        | '`'
                        | ','
                        | ';'
                        | '('
                        | ')'
                        | '['
                        | ']'
                        | '{'
                        | '}'
                        | '<'
                        | '>'
                        | '|'
                        | '\\'
                )
        })
        .map_or(rest.len(), |(offset, _)| offset);
    start + length.max(rest.chars().next().map_or(0, char::len_utf8))
}

fn sensitive_header(name: &str) -> bool {
    let normalized = name.trim().to_ascii_lowercase().replace('_', "-");
    matches!(
        normalized.as_str(),
        "authorization"
            | "proxy-authorization"
            | "cookie"
            | "set-cookie"
            | "x-api-key"
            | "api-key"
            | "anthropic-api-key"
            | "openai-api-key"
            | "x-goog-api-key"
    ) || normalized.contains("api-key")
        || normalized.ends_with("-token")
}

fn bearer_value(value: &str) -> bool {
    value
        .trim_start()
        .get(..7)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("bearer "))
}

fn sensitive_field(name: &str) -> bool {
    let normalized = name.trim().to_ascii_lowercase().replace('-', "_");
    matches!(
        normalized.as_str(),
        "client_secret"
            | "authorization_code"
            | "code"
            | "refresh_token"
            | "access_token"
            | "device_code"
            | "assertion"
            | "client_assertion"
    )
}

fn percent_decode_ascii(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => output.push(b' '),
            b'%' => {
                let high = hex(*bytes.get(index + 1)?)?;
                let low = hex(*bytes.get(index + 2)?)?;
                output.push((high << 4) | low);
                index += 2;
            }
            byte => output.push(byte),
        }
        index += 1;
    }
    String::from_utf8(output).ok()
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_only_is_default_and_retains_only_metadata() {
        assert_eq!(RetentionMode::default(), RetentionMode::MetadataOnly);
        let result = retain_body(RetentionMode::default(), BodyKind::Text, b"ordinary body");
        assert_eq!(result.effective, RetentionMode::MetadataOnly);
        assert!(result.retained_content.is_none());
        assert_eq!(result.byte_metadata.unwrap().original_length, 13);
    }

    #[test]
    fn disabled_retains_neither_content_nor_digest() {
        let result = retain_body(RetentionMode::Disabled, BodyKind::Text, b"body");
        assert_eq!(result.effective, RetentionMode::Disabled);
        assert!(result.retained_content.is_none());
        assert!(result.byte_metadata.is_none());
    }

    #[test]
    fn sensitive_headers_are_removed_case_insensitively() {
        let headers = vec![
            HeaderField {
                name: "AUTHORIZATION".into(),
                value: "Bearer should-not-remain".into(),
            },
            HeaderField {
                name: "content-type".into(),
                value: "application/json".into(),
            },
        ];
        let (redacted, records) = redact_headers(&headers);
        assert_eq!(redacted[0].value, REDACTED);
        assert_eq!(redacted[1], headers[1]);
        assert_eq!(records[0].category, RedactionCategory::SensitiveHeader);
        assert_eq!(records[0].removed_length, 24);
    }

    #[test]
    fn bearer_values_are_removed_from_custom_headers() {
        let headers = vec![HeaderField {
            name: "x-provider-auth".into(),
            value: "bEaReR short".into(),
        }];
        let (redacted, records) = redact_headers(&headers);
        assert_eq!(redacted[0].value, REDACTED);
        assert_eq!(records[0].category, RedactionCategory::SensitiveHeader);
    }

    #[test]
    fn json_oauth_fields_are_removed_before_detector_scan() {
        let body = br#"{"access_token":"not-a-pattern","nested":{"client-secret":"also-sensitive"},"message":"safe"}"#;
        let result = retain_body(RetentionMode::Content, BodyKind::Json, body);
        let content = String::from_utf8(result.retained_content.unwrap()).unwrap();
        assert_eq!(
            content,
            r#"{"access_token":"[REDACTED]","message":"safe","nested":{"client-secret":"[REDACTED]"}}"#
        );
        assert_eq!(result.redactions.len(), 2);
        assert!(
            result
                .redactions
                .iter()
                .all(|record| record.category == RedactionCategory::SensitiveField)
        );
    }

    #[test]
    fn credentials_in_json_keys_are_not_retained() {
        let body = br#"{"AKIA4QD7XZLM2VNBKTRW":"value"}"#;
        let result = retain_body(RetentionMode::Content, BodyKind::Json, body);
        let content = String::from_utf8(result.retained_content.unwrap()).unwrap();
        assert_eq!(content, r#"{"[REDACTED-KEY-0]":"value"}"#);
        assert_eq!(
            result.redactions[0].detector_id.as_deref(),
            Some("aws-access-key-id")
        );
    }

    #[test]
    fn released_detector_removes_credentials_without_reporting_values() {
        let body = b"prefix AKIA4QD7XZLM2VNBKTRW suffix";
        let result = retain_body(RetentionMode::Content, BodyKind::Text, body);
        let content = String::from_utf8(result.retained_content.unwrap()).unwrap();
        assert_eq!(content, "prefix [REDACTED] suffix");
        assert_eq!(
            result.redactions[0].detector_id.as_deref(),
            Some("aws-access-key-id")
        );
        assert_eq!(result.redactions[0].offset, 7);
        assert_eq!(result.redactions[0].removed_length, 20);
    }

    #[test]
    fn form_oauth_fields_are_removed() {
        let body = b"grant_type=refresh_token&refresh_token=plain-secret&scope=read";
        let result = retain_body(RetentionMode::Content, BodyKind::FormUrlEncoded, body);
        assert_eq!(
            result.retained_content.unwrap(),
            b"grant_type=refresh_token&refresh_token=[REDACTED]&scope=read"
        );
    }

    #[test]
    fn unscannable_content_falls_back_to_metadata_only() {
        let result = retain_body(
            RetentionMode::Content,
            BodyKind::Compressed,
            b"compressed bytes",
        );
        assert_eq!(result.effective, RetentionMode::MetadataOnly);
        assert!(result.retained_content.is_none());
        assert_eq!(result.findings[0].detail, "compressed-body-unscannable");
    }

    #[test]
    fn malformed_structured_content_is_not_retained() {
        let result = retain_body(RetentionMode::Content, BodyKind::Json, b"{broken");
        assert_eq!(result.effective, RetentionMode::MetadataOnly);
        assert_eq!(result.findings[0].detail, "malformed-json");
    }

    #[test]
    fn malformed_form_encoding_is_not_retained() {
        let result = retain_body(
            RetentionMode::Content,
            BodyKind::FormUrlEncoded,
            b"access_token=%zz",
        );
        assert_eq!(result.effective, RetentionMode::MetadataOnly);
        assert_eq!(result.findings[0].detail, "malformed-form-encoding");
    }
}

// endregion
