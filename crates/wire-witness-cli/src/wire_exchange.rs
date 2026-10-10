//! The `statecraft/wire-exchange/v1` manifest: the one file a supervisor
//! reads through the evidence reference.
//!
//! Governed by spec 014, the wire-witness.sidecar/1 stdio host.

// region: wire-exchange-manifest

use std::collections::BTreeSet;

use wire_witness_core::custody::RetentionMode;
use wire_witness_core::exchange::{JsonValue, Presence, UsageEntry, canonical_json_bytes};

use crate::live_runtime::InstalledExchange;
use crate::producer_identity::ProducerIdentity;
use crate::sidecar_protocol::{AttemptBinding, EVIDENCE_CONSTRUCTION, EVIDENCE_TYPE};

pub const WIRE_EXCHANGE_FILE: &str = "wire-exchange.json";
pub const WIRE_EXCHANGE_SCHEMA_VERSION: u64 = 1;

/// The facts one manifest states. Every member is supplied; none is
/// inferred here.
pub struct WireExchangeFacts<'a> {
    pub binding: &'a AttemptBinding,
    pub producer: &'a ProducerIdentity,
    pub host_policy_id: &'a str,
    pub end_disposition: &'a str,
    pub gaps: &'a BTreeSet<String>,
    pub requested_retention: RetentionMode,
    /// Records that were built but could not be installed.
    pub missing_records: u64,
    pub exchanges: &'a [InstalledExchange],
    pub observed_exchanges: u64,
    pub dropped_exchanges: u64,
    pub findings: u64,
}

/// The canonical manifest bytes.
#[must_use]
pub fn manifest_bytes(facts: &WireExchangeFacts<'_>) -> Vec<u8> {
    canonical_json_bytes(&manifest(facts))
}

fn manifest(facts: &WireExchangeFacts<'_>) -> JsonValue {
    let mut exchanges: Vec<&InstalledExchange> = facts.exchanges.iter().collect();
    exchanges.sort_by_key(|exchange| exchange.sequence);
    let classes: BTreeSet<&str> = exchanges
        .iter()
        .flat_map(|exchange| exchange.redaction_classes.iter().copied())
        .collect();
    let (retention_result, retention_reason) = if facts.missing_records > 0 {
        ("failed", Some("record-not-installed"))
    } else if facts.requested_retention == RetentionMode::Content {
        ("metadata-only", Some("content-retention-unavailable"))
    } else {
        ("metadata-only", None)
    };
    object([
        ("artifactType", string(EVIDENCE_TYPE)),
        ("attestation", object([("state", string("unknown"))])),
        (
            "capture",
            object([
                ("endDisposition", string(facts.end_disposition)),
                ("startDisposition", string("listening")),
            ]),
        ),
        (
            "completeness",
            object([
                (
                    "gaps",
                    JsonValue::Array(facts.gaps.iter().map(|gap| string(gap)).collect()),
                ),
                (
                    "state",
                    string(if facts.gaps.is_empty() {
                        "complete"
                    } else {
                        "incomplete"
                    }),
                ),
            ]),
        ),
        (
            "counts",
            object([
                ("droppedExchanges", number(facts.dropped_exchanges)),
                ("findings", number(facts.findings)),
                ("observedExchanges", number(facts.observed_exchanges)),
                ("recordedExchanges", number(exchanges.len() as u64)),
            ]),
        ),
        (
            "exchanges",
            JsonValue::Array(exchanges.into_iter().map(exchange).collect()),
        ),
        ("hostPolicyId", string(facts.host_policy_id)),
        ("producer", facts.producer.manifest_producer()),
        ("recordDigestConstruction", string(EVIDENCE_CONSTRUCTION)),
        (
            "redaction",
            object([
                (
                    "fieldClasses",
                    JsonValue::Array(classes.iter().map(|class| string(class)).collect()),
                ),
                ("reason", JsonValue::Null),
                (
                    "result",
                    string(if classes.is_empty() {
                        "not-needed"
                    } else {
                        "applied"
                    }),
                ),
            ]),
        ),
        (
            "retention",
            object([
                ("reason", retention_reason.map_or(JsonValue::Null, string)),
                ("requested", string(facts.requested_retention.as_str())),
                ("result", string(retention_result)),
            ]),
        ),
        ("schemaVersion", number(WIRE_EXCHANGE_SCHEMA_VERSION)),
        (
            "subject",
            object([
                ("attempt", number(u64::from(facts.binding.attempt))),
                ("runId", string(&facts.binding.run_id)),
                ("sidecarEffectId", string(&facts.binding.effect_id)),
            ]),
        ),
    ])
}

fn exchange(exchange: &InstalledExchange) -> JsonValue {
    // The host never redirects, so the served destination is the requested
    // one; both are still stated.
    let side = |model: &Presence<String>| {
        object([
            ("destination", string(&exchange.authority)),
            ("model", presence(model)),
            ("protocol", string(exchange.protocol)),
        ])
    };
    object([
        ("completeness", string(exchange.completeness)),
        (
            "estimatedCost",
            object([("state", string("not-estimated"))]),
        ),
        ("recordByteLength", number(exchange.byte_length)),
        (
            "recordDigest",
            string(&format!("sha256:{}", exchange.digest)),
        ),
        ("requested", side(&exchange.requested_model)),
        ("sequence", number(exchange.sequence)),
        ("served", side(&exchange.served_model)),
        ("usage", usage(&exchange.usage)),
    ])
}

fn presence(model: &Presence<String>) -> JsonValue {
    match model {
        Presence::Known(value) => object([("state", string("known")), ("value", string(value))]),
        Presence::Absent => object([("state", string("absent"))]),
        Presence::Unknown { .. } => object([("state", string("unknown"))]),
    }
}

fn usage(entries: &[UsageEntry]) -> JsonValue {
    if entries.is_empty() {
        return object([("label", string("absent"))]);
    }
    let mut values: Vec<&UsageEntry> = entries.iter().collect();
    values.sort_by(|left, right| {
        (&left.field_name, &left.message_location)
            .cmp(&(&right.field_name, &right.message_location))
    });
    object([
        ("label", string("provider-testimony")),
        (
            "values",
            JsonValue::Array(
                values
                    .into_iter()
                    .map(|entry| {
                        object([
                            ("location", string(&entry.message_location)),
                            ("providerField", string(&entry.field_name)),
                            ("unit", unit(entry)),
                            ("value", JsonValue::Number(entry.value_text.clone())),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

/// The provider's stated unit; else `tokens` when the provider's own field
/// name says so; else unknown.
fn unit(entry: &UsageEntry) -> JsonValue {
    match &entry.unit {
        Presence::Known(unit) => string(unit),
        _ if entry.field_name.ends_with("_tokens") => string("tokens"),
        _ => object([("state", string("unknown"))]),
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

#[cfg(test)]
mod tests {
    use wire_witness_core::exchange::{UsageSource, parse_json};

    use super::*;

    fn installed(sequence: u64, usage: Vec<UsageEntry>) -> InstalledExchange {
        InstalledExchange {
            sequence,
            path: format!("exchanges/exchange-{sequence:020}.json"),
            digest: "ab".repeat(32),
            byte_length: 10,
            completeness: "complete",
            authority: "api.anthropic.com:443".into(),
            protocol: "http/1.1",
            requested_model: Presence::Known("claude-requested".into()),
            served_model: Presence::Known("claude-served".into()),
            usage,
            redaction_classes: ["sensitive-header"].into_iter().collect(),
        }
    }

    fn binding() -> AttemptBinding {
        AttemptBinding {
            run_id: "run-A".into(),
            attempt: 2,
            effect_id: "effect-B".into(),
        }
    }

    fn render(exchanges: &[InstalledExchange], gaps: &BTreeSet<String>) -> JsonValue {
        let producer = ProducerIdentity::for_executable(None);
        let binding = binding();
        parse_json(&manifest_bytes(&WireExchangeFacts {
            binding: &binding,
            producer: &producer,
            host_policy_id: "policy-1",
            end_disposition: "supervisor-closed",
            gaps,
            requested_retention: RetentionMode::MetadataOnly,
            missing_records: 0,
            exchanges,
            observed_exchanges: exchanges.len() as u64,
            dropped_exchanges: 0,
            findings: 0,
        }))
        .unwrap()
    }

    fn keys(value: &JsonValue) -> Vec<&str> {
        let JsonValue::Object(fields) = value else {
            panic!("object expected");
        };
        fields.keys().map(String::as_str).collect()
    }

    fn get<'a>(value: &'a JsonValue, name: &str) -> &'a JsonValue {
        let JsonValue::Object(fields) = value else {
            panic!("object expected");
        };
        &fields[name]
    }

    #[test]
    fn member_sets_are_exact() {
        let value = render(&[installed(1, Vec::new())], &BTreeSet::new());
        assert_eq!(
            keys(&value),
            [
                "artifactType",
                "attestation",
                "capture",
                "completeness",
                "counts",
                "exchanges",
                "hostPolicyId",
                "producer",
                "recordDigestConstruction",
                "redaction",
                "retention",
                "schemaVersion",
                "subject",
            ]
        );
        let JsonValue::Array(exchanges) = get(&value, "exchanges") else {
            panic!("array expected");
        };
        assert_eq!(
            keys(&exchanges[0]),
            [
                "completeness",
                "estimatedCost",
                "recordByteLength",
                "recordDigest",
                "requested",
                "sequence",
                "served",
                "usage",
            ]
        );
    }

    #[test]
    fn absent_usage_is_never_zero_and_models_stay_separate() {
        let text = String::from_utf8(canonical_json_bytes(&render(
            &[installed(1, Vec::new())],
            &BTreeSet::new(),
        )))
        .unwrap();
        assert!(text.contains(r#""usage":{"label":"absent"}"#));
        assert!(text.contains(r#""estimatedCost":{"state":"not-estimated"}"#));
        assert!(text.contains(r#""model":{"state":"known","value":"claude-requested"}"#));
        assert!(text.contains(r#""model":{"state":"known","value":"claude-served"}"#));
        assert!(
            text.contains(
                r#""subject":{"attempt":2,"runId":"run-A","sidecarEffectId":"effect-B"}"#
            )
        );
    }

    #[test]
    fn usage_is_provider_testimony_sorted_by_field() {
        let entry = |field: &str, value: &str| UsageEntry {
            field_name: field.into(),
            value_text: value.into(),
            unit: Presence::Absent,
            message_location: "$.usage".into(),
            source: UsageSource::ProviderReported,
        };
        let value = render(
            &[installed(
                1,
                vec![entry("output_tokens", "5"), entry("input_tokens", "3")],
            )],
            &BTreeSet::new(),
        );
        let text = String::from_utf8(canonical_json_bytes(&value)).unwrap();
        let input = text.find("input_tokens").unwrap();
        let output = text.find("output_tokens").unwrap();
        assert!(input < output);
        assert!(text.contains(r#""label":"provider-testimony""#));
        assert!(text.contains(r#""unit":"tokens","value":3"#));
    }

    #[test]
    fn gaps_make_the_state_incomplete() {
        let gaps: BTreeSet<String> = ["deadline-reached".to_owned()].into_iter().collect();
        let value = render(&[installed(1, Vec::new())], &gaps);
        let text = String::from_utf8(canonical_json_bytes(get(&value, "completeness"))).unwrap();
        assert_eq!(
            text,
            r#"{"gaps":["deadline-reached"],"state":"incomplete"}"#
        );
    }
}

// endregion
