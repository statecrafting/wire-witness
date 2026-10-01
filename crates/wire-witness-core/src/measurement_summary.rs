//! Deterministic binding-level summaries of exchange testimony.
//!
//! Governed by spec 009, usage and cost summaries.

// region: measurement-summary

use std::collections::{BTreeMap, BTreeSet};

use crate::exchange::{
    Binding, Completeness, Cost, ExchangeRecord, JsonValue, Presence, ProviderFamily,
    ServedDisclosure, UsageEntry, UsageSource, canonical_json_bytes, sha256_hex,
};

pub const SCHEMA: &str = "wire-witness.measurement-summary/1";
pub const DIGEST_CONSTRUCTION: &str = "wire-witness.measurement-summary/1+keysort-json+sha256";
pub const INPUT_DIGEST_CONSTRUCTION: &str = "wire-witness.measurement-inputs/1+keysort-json+sha256";
pub const ARITHMETIC_CONSTRUCTION: &str = "decimal-exact-v1";

#[derive(Clone, Debug)]
pub struct MeasurementInput<'a> {
    pub record: &'a ExchangeRecord,
    pub claimed_digest: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalCapture {
    pub identity: String,
    pub last_sequence: Option<u64>,
    pub complete: bool,
}

#[derive(Clone, Debug)]
pub struct SummaryRequest<'a> {
    pub binding: &'a Binding,
    pub inputs: Vec<MeasurementInput<'a>>,
    pub terminal: TerminalCapture,
    pub producer_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SummaryError {
    EmptyInput,
    ForeignBinding { sequence: u64 },
    InvalidDigest { sequence: u64 },
    DuplicateSequence { sequence: u64 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestCountGroup {
    pub provider: String,
    pub operation: String,
    pub request_path: String,
    pub protocol: String,
    pub authority: String,
    pub count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NumericGroup {
    pub provider: String,
    pub field_path: Option<String>,
    pub field_name: Option<String>,
    pub unit_or_currency: String,
    pub attribution: String,
    pub rate_table_identity: Option<String>,
    pub usage_inputs: Vec<String>,
    pub total: Option<String>,
    pub observed_exchange_count: u64,
    pub absent_exchange_count: u64,
    pub unknown_value_count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityCount {
    pub identity: Option<String>,
    pub disclosure: Option<String>,
    pub posture: String,
    pub reason: Option<String>,
    pub count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReasonCount {
    pub reason: String,
    pub count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SummaryFinding {
    pub sequence: Option<u64>,
    pub code: String,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeasurementSummary {
    pub binding: Binding,
    pub terminal: TerminalCapture,
    pub first_sequence: u64,
    pub last_sequence: u64,
    pub exchange_count: u64,
    pub ordered_exchange_digests: Vec<String>,
    pub input_manifest_digest: String,
    pub request_counts: Vec<RequestCountGroup>,
    pub usage: Vec<NumericGroup>,
    pub reported_cost: Vec<NumericGroup>,
    pub estimated_cost: Vec<NumericGroup>,
    pub unknown_cost: Vec<ReasonCount>,
    pub requested_identities: Vec<IdentityCount>,
    pub served_identities: Vec<IdentityCount>,
    pub complete_capture_count: u64,
    pub incomplete_capture_count: u64,
    pub absent_capture_count: u64,
    pub complete: bool,
    pub findings: Vec<SummaryFinding>,
    pub producer_identity: String,
    pub rate_table_identities: Vec<String>,
}

impl MeasurementSummary {
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
            ("arithmetic_construction", string(ARITHMETIC_CONSTRUCTION)),
            ("binding", binding_json(&self.binding)),
            ("complete", JsonValue::Bool(self.complete)),
            (
                "completeness_counts",
                object([
                    ("absent", number(self.absent_capture_count)),
                    ("complete", number(self.complete_capture_count)),
                    ("incomplete", number(self.incomplete_capture_count)),
                ]),
            ),
            ("digest_construction", string(DIGEST_CONSTRUCTION)),
            (
                "estimated_cost",
                JsonValue::Array(self.estimated_cost.iter().map(numeric_json).collect()),
            ),
            ("exchange_count", number(self.exchange_count)),
            (
                "findings",
                JsonValue::Array(self.findings.iter().map(finding_json).collect()),
            ),
            ("first_sequence", number(self.first_sequence)),
            ("input_manifest_digest", string(&self.input_manifest_digest)),
            (
                "input_manifest_digest_construction",
                string(INPUT_DIGEST_CONSTRUCTION),
            ),
            ("last_sequence", number(self.last_sequence)),
            (
                "ordered_exchange_digests",
                JsonValue::Array(self.ordered_exchange_digests.iter().map(string).collect()),
            ),
            ("producer_identity", string(&self.producer_identity)),
            (
                "rate_table_identities",
                JsonValue::Array(self.rate_table_identities.iter().map(string).collect()),
            ),
            (
                "reported_cost",
                JsonValue::Array(self.reported_cost.iter().map(numeric_json).collect()),
            ),
            (
                "request_counts",
                JsonValue::Array(self.request_counts.iter().map(request_count_json).collect()),
            ),
            (
                "requested_identities",
                JsonValue::Array(
                    self.requested_identities
                        .iter()
                        .map(identity_json)
                        .collect(),
                ),
            ),
            ("schema", string(SCHEMA)),
            (
                "served_identities",
                JsonValue::Array(self.served_identities.iter().map(identity_json).collect()),
            ),
            (
                "terminal_capture",
                object([
                    ("complete", JsonValue::Bool(self.terminal.complete)),
                    ("identity", string(&self.terminal.identity)),
                    (
                        "last_sequence",
                        self.terminal.last_sequence.map_or(JsonValue::Null, number),
                    ),
                ]),
            ),
            (
                "usage",
                JsonValue::Array(self.usage.iter().map(numeric_json).collect()),
            ),
            (
                "unknown_cost",
                JsonValue::Array(self.unknown_cost.iter().map(reason_count_json).collect()),
            ),
        ])
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct NumericKey {
    provider: String,
    field_path: Option<String>,
    field_name: Option<String>,
    unit_or_currency: String,
    attribution: String,
    rate_table_identity: Option<String>,
    usage_inputs: Vec<String>,
}

#[derive(Default)]
struct NumericAccumulator {
    total: Option<Decimal>,
    total_unknown: bool,
    seen_sequences: BTreeSet<u64>,
    unknown_value_count: u64,
}

type IdentityKey = (Option<String>, Option<String>, String, Option<String>);
type IdentityGroups = BTreeMap<IdentityKey, u64>;

pub fn summarize(request: SummaryRequest<'_>) -> Result<MeasurementSummary, SummaryError> {
    if request.inputs.is_empty() {
        return Err(SummaryError::EmptyInput);
    }

    let mut ordered = request.inputs;
    ordered.sort_by_key(|input| input.record.sequence);
    let mut sequences = BTreeSet::new();
    for input in &ordered {
        let sequence = input.record.sequence;
        if &input.record.binding != request.binding {
            return Err(SummaryError::ForeignBinding { sequence });
        }
        if input.record.digest() != input.claimed_digest {
            return Err(SummaryError::InvalidDigest { sequence });
        }
        if !sequences.insert(sequence) {
            return Err(SummaryError::DuplicateSequence { sequence });
        }
    }

    let exchange_count = ordered.len() as u64;
    let first_sequence = ordered[0].record.sequence;
    let last_sequence = ordered[ordered.len() - 1].record.sequence;
    let ordered_exchange_digests: Vec<_> = ordered
        .iter()
        .map(|input| input.claimed_digest.to_owned())
        .collect();
    let input_manifest = object([
        ("binding", binding_json(request.binding)),
        (
            "exchange_digests",
            JsonValue::Array(ordered_exchange_digests.iter().map(string).collect()),
        ),
        ("schema", string("wire-witness.measurement-inputs/1")),
    ]);
    let input_manifest_digest = sha256_hex(&canonical_json_bytes(&input_manifest));

    let mut construction_findings = Vec::new();
    let mut source_findings = Vec::new();
    let mut complete = request.terminal.complete && first_sequence == 1;
    if first_sequence != 1 {
        complete = false;
        construction_findings.push(SummaryFinding {
            sequence: None,
            code: "missing-sequence".into(),
            detail: format!("missing sequence range 1..{}", first_sequence - 1),
        });
    }
    for window in ordered.windows(2) {
        let left = window[0].record.sequence;
        let right = window[1].record.sequence;
        if right != left + 1 {
            complete = false;
            construction_findings.push(SummaryFinding {
                sequence: None,
                code: "missing-sequence".into(),
                detail: format!("missing sequence range {}..{}", left + 1, right - 1),
            });
        }
    }
    match request.terminal.last_sequence {
        Some(terminal) if last_sequence > terminal => {
            complete = false;
            construction_findings.push(SummaryFinding {
                sequence: None,
                code: "record-after-terminal".into(),
                detail: format!("sequence {last_sequence} follows terminal sequence {terminal}"),
            });
        }
        Some(terminal) if last_sequence < terminal => {
            complete = false;
            construction_findings.push(SummaryFinding {
                sequence: None,
                code: "missing-terminal-sequence".into(),
                detail: format!("terminal sequence is {terminal}, last input is {last_sequence}"),
            });
        }
        None => {
            complete = false;
            construction_findings.push(SummaryFinding {
                sequence: None,
                code: "terminal-boundary-unknown".into(),
                detail: "terminal capture sequence was not supplied".into(),
            });
        }
        Some(_) => {}
    }

    let mut request_counts = BTreeMap::<(String, String, String, String, String), u64>::new();
    let mut usage = BTreeMap::<NumericKey, NumericAccumulator>::new();
    let mut reported_cost = BTreeMap::<NumericKey, NumericAccumulator>::new();
    let mut estimated_cost = BTreeMap::<NumericKey, NumericAccumulator>::new();
    let mut unknown_cost = BTreeMap::<String, u64>::new();
    let mut requested_identities = IdentityGroups::new();
    let mut served_identities = IdentityGroups::new();
    let mut complete_capture_count = 0;
    let mut incomplete_capture_count = 0;
    let mut absent_capture_count = 0;
    let mut rate_table_identities = BTreeSet::new();

    for input in &ordered {
        let record = input.record;
        let provider = provider_name(&record.provider).to_owned();
        *request_counts
            .entry((
                provider.clone(),
                record.transport.operation.clone(),
                record.transport.request_path.clone(),
                record.transport.protocol.clone(),
                record.transport.authority.clone(),
            ))
            .or_default() += 1;

        for entry in &record.usage {
            let key = usage_key(&provider, entry);
            add_numeric(
                &mut usage,
                key,
                record.sequence,
                &entry.value_text,
                &mut construction_findings,
            );
        }
        match &record.cost {
            Cost::Reported {
                value_text,
                currency_or_unit,
            } => add_numeric(
                &mut reported_cost,
                NumericKey {
                    provider: provider.clone(),
                    field_path: None,
                    field_name: None,
                    unit_or_currency: currency_or_unit.clone(),
                    attribution: "provider-reported-cost".into(),
                    rate_table_identity: None,
                    usage_inputs: Vec::new(),
                },
                record.sequence,
                value_text,
                &mut construction_findings,
            ),
            Cost::Estimated {
                value_text,
                currency,
                rate_table_identity,
                usage_inputs,
            } => {
                let mut inputs = usage_inputs.clone();
                inputs.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
                inputs.dedup();
                rate_table_identities.insert(rate_table_identity.clone());
                add_numeric(
                    &mut estimated_cost,
                    NumericKey {
                        provider: provider.clone(),
                        field_path: None,
                        field_name: None,
                        unit_or_currency: currency.clone(),
                        attribution: "local-estimate".into(),
                        rate_table_identity: Some(rate_table_identity.clone()),
                        usage_inputs: inputs,
                    },
                    record.sequence,
                    value_text,
                    &mut construction_findings,
                );
            }
            Cost::Unknown { reason } => {
                *unknown_cost.entry(reason.clone()).or_default() += 1;
            }
        }

        add_requested_identity(&mut requested_identities, &record.requested_identity);
        add_served_identity(&mut served_identities, &record.served_identity);
        match record.completeness {
            Completeness::Complete => complete_capture_count += 1,
            Completeness::Incomplete(_) => {
                incomplete_capture_count += 1;
                complete = false;
            }
            Completeness::Absent(_) => {
                absent_capture_count += 1;
                complete = false;
            }
        }
        for source in &record.findings {
            source_findings.push(SummaryFinding {
                sequence: Some(record.sequence),
                code: source.code.clone(),
                detail: format!("{}: {}", source.direction, source.detail),
            });
        }
    }

    source_findings.extend(construction_findings);

    Ok(MeasurementSummary {
        binding: request.binding.clone(),
        terminal: request.terminal,
        first_sequence,
        last_sequence,
        exchange_count,
        ordered_exchange_digests,
        input_manifest_digest,
        request_counts: request_counts
            .into_iter()
            .map(
                |((provider, operation, request_path, protocol, authority), count)| {
                    RequestCountGroup {
                        provider,
                        operation,
                        request_path,
                        protocol,
                        authority,
                        count,
                    }
                },
            )
            .collect(),
        usage: finish_numeric(usage, exchange_count),
        reported_cost: finish_numeric(reported_cost, exchange_count),
        estimated_cost: finish_numeric(estimated_cost, exchange_count),
        unknown_cost: unknown_cost
            .into_iter()
            .map(|(reason, count)| ReasonCount { reason, count })
            .collect(),
        requested_identities: finish_identities(requested_identities),
        served_identities: finish_identities(served_identities),
        complete_capture_count,
        incomplete_capture_count,
        absent_capture_count,
        complete,
        findings: source_findings,
        producer_identity: request.producer_identity,
        rate_table_identities: rate_table_identities.into_iter().collect(),
    })
}

fn add_numeric(
    groups: &mut BTreeMap<NumericKey, NumericAccumulator>,
    key: NumericKey,
    sequence: u64,
    value_text: &str,
    findings: &mut Vec<SummaryFinding>,
) {
    let group = groups.entry(key).or_default();
    group.seen_sequences.insert(sequence);
    match Decimal::parse(value_text) {
        Ok(value) if !group.total_unknown => {
            let result = group
                .total
                .map_or(Ok(value), |total| total.checked_add(value));
            match result {
                Ok(total) => group.total = Some(total),
                Err(()) => {
                    group.total = None;
                    group.total_unknown = true;
                    findings.push(SummaryFinding {
                        sequence: Some(sequence),
                        code: "decimal-overflow".into(),
                        detail: "numeric group running sum exceeds decimal-exact-v1 bounds".into(),
                    });
                }
            }
        }
        Ok(_) => {}
        Err(()) => {
            group.unknown_value_count += 1;
            findings.push(SummaryFinding {
                sequence: Some(sequence),
                code: "invalid-decimal".into(),
                detail: format!("value is outside {ARITHMETIC_CONSTRUCTION} bounds"),
            });
        }
    }
}

fn finish_numeric(
    groups: BTreeMap<NumericKey, NumericAccumulator>,
    exchange_count: u64,
) -> Vec<NumericGroup> {
    groups
        .into_iter()
        .map(|(key, value)| NumericGroup {
            provider: key.provider,
            field_path: key.field_path,
            field_name: key.field_name,
            unit_or_currency: key.unit_or_currency,
            attribution: key.attribution,
            rate_table_identity: key.rate_table_identity,
            usage_inputs: key.usage_inputs,
            total: if value.total_unknown {
                None
            } else {
                value.total.map(Decimal::canonical)
            },
            observed_exchange_count: value.seen_sequences.len() as u64,
            absent_exchange_count: exchange_count - value.seen_sequences.len() as u64,
            unknown_value_count: value.unknown_value_count,
        })
        .collect()
}

fn usage_key(provider: &str, entry: &UsageEntry) -> NumericKey {
    NumericKey {
        provider: provider.into(),
        field_path: Some(entry.message_location.clone()),
        field_name: Some(entry.field_name.clone()),
        unit_or_currency: presence_key(&entry.unit),
        attribution: match entry.source {
            UsageSource::ProviderReported => "provider-reported-usage".into(),
        },
        rate_table_identity: None,
        usage_inputs: Vec::new(),
    }
}

fn presence_key(value: &Presence<String>) -> String {
    match value {
        Presence::Known(value) => format!("known:{value}"),
        Presence::Unknown { reason } => format!("unknown:{reason}"),
        Presence::Absent => "absent".into(),
    }
}

fn add_requested_identity(groups: &mut IdentityGroups, identity: &Presence<String>) {
    let key = match identity {
        Presence::Known(value) => (Some(value.clone()), None, "known".into(), None),
        Presence::Unknown { reason } => (None, None, "unknown".into(), Some(reason.clone())),
        Presence::Absent => (None, None, "absent".into(), None),
    };
    *groups.entry(key).or_default() += 1;
}

fn add_served_identity(
    groups: &mut IdentityGroups,
    identity: &Presence<crate::exchange::ServedIdentity>,
) {
    let key = match identity {
        Presence::Known(value) => (
            Some(value.identity.clone()),
            Some(disclosure_name(&value.disclosure).into()),
            "known".into(),
            None,
        ),
        Presence::Unknown { reason } => (None, None, "unknown".into(), Some(reason.clone())),
        Presence::Absent => (None, None, "absent".into(), None),
    };
    *groups.entry(key).or_default() += 1;
}

fn finish_identities(groups: IdentityGroups) -> Vec<IdentityCount> {
    groups
        .into_iter()
        .map(
            |((identity, disclosure, posture, reason), count)| IdentityCount {
                identity,
                disclosure,
                posture,
                reason,
                count,
            },
        )
        .collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Decimal {
    coefficient: i128,
    scale: u32,
}

impl Decimal {
    fn parse(text: &str) -> Result<Self, ()> {
        let bytes = text.as_bytes();
        if bytes.is_empty() || text.trim() != text {
            return Err(());
        }
        let mut index = 0;
        let negative = match bytes.first() {
            Some(b'-') => {
                index = 1;
                true
            }
            Some(b'+') => return Err(()),
            _ => false,
        };
        let integer_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if index == integer_start || (index - integer_start > 1 && bytes[integer_start] == b'0') {
            return Err(());
        }
        let mut digits = text[integer_start..index].to_owned();
        let mut fractional = 0i32;
        if bytes.get(index) == Some(&b'.') {
            index += 1;
            let start = index;
            while bytes.get(index).is_some_and(u8::is_ascii_digit) {
                index += 1;
            }
            if index == start {
                return Err(());
            }
            fractional = (index - start) as i32;
            digits.push_str(&text[start..index]);
        }
        let mut exponent = 0i32;
        if matches!(bytes.get(index), Some(b'e' | b'E')) {
            index += 1;
            let exponent_negative = match bytes.get(index) {
                Some(b'-') => {
                    index += 1;
                    true
                }
                Some(b'+') => {
                    index += 1;
                    false
                }
                _ => false,
            };
            let start = index;
            while bytes.get(index).is_some_and(u8::is_ascii_digit) {
                index += 1;
            }
            if index == start || index - start > 4 {
                return Err(());
            }
            exponent = text[start..index].parse::<i32>().map_err(|_| ())?;
            if exponent_negative {
                exponent = -exponent;
            }
        }
        if index != bytes.len() {
            return Err(());
        }
        let significant = digits.trim_start_matches('0');
        if significant.len() > 38 {
            return Err(());
        }
        if significant.is_empty() {
            return Ok(Self {
                coefficient: 0,
                scale: 0,
            });
        }
        let target_scale = fractional.checked_sub(exponent).ok_or(())?;
        if target_scale > 18 {
            return Err(());
        }
        if target_scale < 0 {
            let zeros = usize::try_from(-target_scale).map_err(|_| ())?;
            if significant.len() + zeros > 38 {
                return Err(());
            }
            digits.extend(std::iter::repeat_n('0', zeros));
        }
        let coefficient = digits.parse::<i128>().map_err(|_| ())?;
        let coefficient = if negative { -coefficient } else { coefficient };
        let value = Self {
            coefficient,
            scale: target_scale.max(0) as u32,
        };
        value.within_bounds().then_some(value).ok_or(())
    }

    fn checked_add(self, other: Self) -> Result<Self, ()> {
        let scale = self.scale.max(other.scale);
        let left = self
            .coefficient
            .checked_mul(pow10(scale - self.scale).ok_or(())?)
            .ok_or(())?;
        let right = other
            .coefficient
            .checked_mul(pow10(scale - other.scale).ok_or(())?)
            .ok_or(())?;
        let sum = Self {
            coefficient: left.checked_add(right).ok_or(())?,
            scale,
        };
        sum.within_bounds().then_some(sum).ok_or(())
    }

    fn within_bounds(self) -> bool {
        self.scale <= 18 && decimal_digits(self.coefficient) <= 38
    }

    fn canonical(self) -> String {
        if self.coefficient == 0 {
            return "0".into();
        }
        let negative = self.coefficient < 0;
        let mut digits = self.coefficient.unsigned_abs().to_string();
        if self.scale > 0 {
            let scale = self.scale as usize;
            if digits.len() <= scale {
                digits.insert_str(0, &"0".repeat(scale + 1 - digits.len()));
            }
            digits.insert(digits.len() - scale, '.');
            while digits.ends_with('0') {
                digits.pop();
            }
            if digits.ends_with('.') {
                digits.pop();
            }
        }
        if negative {
            format!("-{digits}")
        } else {
            digits
        }
    }
}

fn pow10(power: u32) -> Option<i128> {
    (0..power).try_fold(1i128, |value, _| value.checked_mul(10))
}

fn decimal_digits(value: i128) -> usize {
    value
        .unsigned_abs()
        .to_string()
        .trim_start_matches('0')
        .len()
        .max(1)
}

fn provider_name(provider: &ProviderFamily) -> &'static str {
    match provider {
        ProviderFamily::Anthropic => "anthropic",
        ProviderFamily::OpenAi => "openai",
        ProviderFamily::Unknown => "unknown",
    }
}

fn disclosure_name(disclosure: &ServedDisclosure) -> &'static str {
    match disclosure {
        ServedDisclosure::Pinned => "pinned",
        ServedDisclosure::Reported => "reported",
        ServedDisclosure::Unknown => "unknown",
    }
}

fn object<const N: usize>(entries: [(&str, JsonValue); N]) -> JsonValue {
    JsonValue::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.into(), value))
            .collect(),
    )
}

fn string(value: impl AsRef<str>) -> JsonValue {
    JsonValue::String(value.as_ref().to_owned())
}

fn number(value: impl ToString) -> JsonValue {
    JsonValue::Number(value.to_string())
}

fn binding_json(binding: &Binding) -> JsonValue {
    match binding {
        Binding::Supervised {
            run_id,
            attempt,
            effect_id,
        } => object([
            ("attempt", number(*attempt)),
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

fn request_count_json(group: &RequestCountGroup) -> JsonValue {
    object([
        ("authority", string(&group.authority)),
        ("count", number(group.count)),
        ("operation", string(&group.operation)),
        ("protocol", string(&group.protocol)),
        ("provider", string(&group.provider)),
        ("request_path", string(&group.request_path)),
    ])
}

fn numeric_json(group: &NumericGroup) -> JsonValue {
    object([
        ("absent_exchange_count", number(group.absent_exchange_count)),
        ("attribution", string(&group.attribution)),
        (
            "field_name",
            group.field_name.as_deref().map_or(JsonValue::Null, string),
        ),
        (
            "field_path",
            group.field_path.as_deref().map_or(JsonValue::Null, string),
        ),
        ("numeric_construction", string(ARITHMETIC_CONSTRUCTION)),
        (
            "observed_exchange_count",
            number(group.observed_exchange_count),
        ),
        ("provider", string(&group.provider)),
        (
            "rate_table_identity",
            group
                .rate_table_identity
                .as_deref()
                .map_or(JsonValue::Null, string),
        ),
        (
            "total",
            group.total.as_deref().map_or(JsonValue::Null, string),
        ),
        ("unit_or_currency", string(&group.unit_or_currency)),
        ("unknown_value_count", number(group.unknown_value_count)),
        (
            "usage_inputs",
            JsonValue::Array(group.usage_inputs.iter().map(string).collect()),
        ),
    ])
}

fn identity_json(identity: &IdentityCount) -> JsonValue {
    object([
        ("count", number(identity.count)),
        (
            "disclosure",
            identity
                .disclosure
                .as_deref()
                .map_or(JsonValue::Null, string),
        ),
        (
            "identity",
            identity.identity.as_deref().map_or(JsonValue::Null, string),
        ),
        ("posture", string(&identity.posture)),
        (
            "reason",
            identity.reason.as_deref().map_or(JsonValue::Null, string),
        ),
    ])
}

fn finding_json(finding: &SummaryFinding) -> JsonValue {
    object([
        ("code", string(&finding.code)),
        ("detail", string(&finding.detail)),
        ("sequence", finding.sequence.map_or(JsonValue::Null, number)),
    ])
}

fn reason_count_json(reason: &ReasonCount) -> JsonValue {
    object([
        ("count", number(reason.count)),
        ("reason", string(&reason.reason)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exchange::{
        ByteReference, Finding, IncompleteReason, RetentionState, ServedIdentity, TransportMetadata,
    };

    fn record(sequence: u64, usage: &str, cost: Cost) -> ExchangeRecord {
        ExchangeRecord {
            binding: Binding::Unsupervised {
                session_id: "session-1".into(),
            },
            sequence,
            transport: TransportMetadata {
                protocol: "https".into(),
                authority: "api.example".into(),
                operation: "POST /v1/messages".into(),
                request_path: "/v1/messages".into(),
                response_status: Presence::Known(200),
                stream_state: "complete".into(),
            },
            provider: ProviderFamily::Anthropic,
            requested_identity: Presence::Known("requested".into()),
            served_identity: Presence::Known(ServedIdentity {
                identity: "served".into(),
                disclosure: ServedDisclosure::Reported,
            }),
            usage: vec![UsageEntry {
                field_name: "input_tokens".into(),
                value_text: usage.into(),
                unit: Presence::Known("tokens".into()),
                message_location: "usage.input_tokens".into(),
                source: UsageSource::ProviderReported,
            }],
            cost,
            retention: RetentionState {
                requested: "metadata-only".into(),
                effective: "metadata-only".into(),
            },
            redaction_count: 0,
            completeness: Completeness::Complete,
            findings: Vec::new(),
            request_bytes: ByteReference::observed(b"request"),
            response_bytes: ByteReference::observed(b"response"),
            events: Vec::new(),
        }
    }

    fn summarize_records(records: &[ExchangeRecord]) -> Result<MeasurementSummary, SummaryError> {
        let digests: Vec<_> = records.iter().map(ExchangeRecord::digest).collect();
        summarize(SummaryRequest {
            binding: &records[0].binding,
            inputs: records
                .iter()
                .zip(&digests)
                .map(|(record, digest)| MeasurementInput {
                    record,
                    claimed_digest: digest,
                })
                .collect(),
            terminal: TerminalCapture {
                identity: "capture:1".into(),
                last_sequence: Some(records.len() as u64),
                complete: true,
            },
            producer_identity: "wire-witness:test".into(),
        })
    }

    #[test]
    fn exact_decimal_supports_exponents_without_floats() {
        assert_eq!(Decimal::parse("1.5e6").unwrap().canonical(), "1500000");
        assert_eq!(
            Decimal::parse("1e-18").unwrap().canonical(),
            "0.000000000000000001"
        );
        assert!(Decimal::parse("1e-19").is_err());
        assert!(Decimal::parse("NaN").is_err());
        assert!(Decimal::parse("01").is_err());
    }

    #[test]
    fn canonical_summary_is_independent_of_caller_order() {
        let first = record(
            1,
            "1.5e1",
            Cost::Reported {
                value_text: "0.10".into(),
                currency_or_unit: "USD".into(),
            },
        );
        let second = record(
            2,
            "2",
            Cost::Reported {
                value_text: "0.20".into(),
                currency_or_unit: "USD".into(),
            },
        );
        let forward = summarize_records(&[first.clone(), second.clone()]).unwrap();
        let reverse = summarize_records(&[second, first]).unwrap();
        assert_eq!(forward.canonical_bytes(), reverse.canonical_bytes());
        assert_eq!(forward.usage[0].total.as_deref(), Some("17"));
        assert_eq!(forward.reported_cost[0].total.as_deref(), Some("0.3"));
        assert_eq!(forward.requested_identities[0].count, 2);
    }

    #[test]
    fn gaps_and_invalid_values_remain_explicit() {
        let mut first = record(
            1,
            "1.0000000000000000000",
            Cost::Unknown {
                reason: "not-reported".into(),
            },
        );
        first.completeness = Completeness::Incomplete(IncompleteReason::StreamEnded);
        first.findings.push(Finding {
            code: "stream-ended".into(),
            direction: "response".into(),
            detail: "terminal event absent".into(),
        });
        let third = record(
            3,
            "2",
            Cost::Unknown {
                reason: "not-reported".into(),
            },
        );
        let summary = summarize_records(&[first, third]).unwrap();
        assert!(!summary.complete);
        assert_eq!(summary.usage[0].total.as_deref(), Some("2"));
        assert_eq!(summary.usage[0].unknown_value_count, 1);
        assert!(
            summary
                .findings
                .iter()
                .any(|finding| finding.code == "missing-sequence")
        );
        assert!(
            summary
                .findings
                .iter()
                .any(|finding| finding.code == "invalid-decimal")
        );
    }

    #[test]
    fn foreign_binding_duplicate_sequence_and_digest_mismatch_refuse() {
        let one = record(
            1,
            "1",
            Cost::Unknown {
                reason: "none".into(),
            },
        );
        let mut foreign = record(
            2,
            "1",
            Cost::Unknown {
                reason: "none".into(),
            },
        );
        foreign.binding = Binding::Unsupervised {
            session_id: "foreign".into(),
        };
        assert!(matches!(
            summarize_records(&[one.clone(), foreign]),
            Err(SummaryError::ForeignBinding { .. })
        ));
        let duplicate = record(
            1,
            "2",
            Cost::Unknown {
                reason: "none".into(),
            },
        );
        assert!(matches!(
            summarize_records(&[one.clone(), duplicate]),
            Err(SummaryError::DuplicateSequence { .. })
        ));
        assert!(matches!(
            summarize(SummaryRequest {
                binding: &one.binding,
                inputs: vec![MeasurementInput {
                    record: &one,
                    claimed_digest: "bad"
                }],
                terminal: TerminalCapture {
                    identity: "capture:1".into(),
                    last_sequence: Some(1),
                    complete: true
                },
                producer_identity: "test".into(),
            }),
            Err(SummaryError::InvalidDigest { .. })
        ));
    }
}

// endregion
