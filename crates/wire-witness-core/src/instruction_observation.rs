//! Exact instruction-delivery observations over one captured request.
//!
//! Governed by spec 008, instruction delivery observation.

// region: instruction-delivery-observation

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use zeroize::Zeroize;

use crate::exchange::{
    Binding, JsonValue, ProviderFamily, canonical_json_bytes, parse_json, sha256_hex,
};

pub const SCHEMA: &str = "wire-witness.instruction-observation/1";
pub const DIGEST_CONSTRUCTION: &str = "wire-witness.instruction-observation/1+keysort-json+sha256";
pub const TARGET_DIGEST_CONSTRUCTION: &str = "file-bytes-sha256";
pub const MATCHING_MODE: &str = "exact-contiguous-utf8-v1";
pub const DECODER_IDENTITY: &str = "wire-witness.request-component-decoder/1";
const MAX_JSON_NESTING: usize = 128;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ComponentKind {
    System,
    Message,
    Instructions,
    Input,
}

impl ComponentKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Message => "message",
            Self::Instructions => "instructions",
            Self::Input => "input",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentSelector {
    pub provider: ProviderFamily,
    pub operation: String,
    pub kind: ComponentKind,
    pub component_index: usize,
    pub text_part_index: usize,
}

impl ComponentSelector {
    pub fn validate(&self) -> Result<(), PlanError> {
        let supported = match (&self.provider, self.operation.as_str(), self.kind) {
            (ProviderFamily::Anthropic, "POST /v1/messages", ComponentKind::System) => {
                self.component_index == 0
            }
            (ProviderFamily::Anthropic, "POST /v1/messages", ComponentKind::Message) => true,
            (ProviderFamily::OpenAi, "POST /v1/responses", ComponentKind::Instructions) => {
                self.component_index == 0 && self.text_part_index == 0
            }
            (ProviderFamily::OpenAi, "POST /v1/responses", ComponentKind::Input) => true,
            (ProviderFamily::OpenAi, "POST /v1/chat/completions", ComponentKind::Message) => true,
            _ => false,
        };
        if supported {
            Ok(())
        } else {
            Err(PlanError::UnsupportedSelector)
        }
    }
}

#[derive(Eq, PartialEq)]
pub struct ComparisonPlan {
    probe_name: String,
    target: Vec<u8>,
    target_digest: String,
    selector: ComponentSelector,
}

impl ComparisonPlan {
    pub fn new(
        probe_name: String,
        mut target: Vec<u8>,
        claimed_target_digest: &str,
        selector: ComponentSelector,
    ) -> Result<Self, PlanError> {
        let validation = if probe_name.is_empty() || probe_name.contains(['\0', '\r', '\n']) {
            Err(PlanError::InvalidProbeName)
        } else if target.is_empty() {
            Err(PlanError::EmptyTarget)
        } else if std::str::from_utf8(&target).is_err() {
            Err(PlanError::InvalidUtf8)
        } else {
            selector.validate()
        };
        if let Err(error) = validation {
            target.zeroize();
            return Err(error);
        }
        let target_digest = sha256_hex(&target);
        if claimed_target_digest != target_digest {
            target.zeroize();
            return Err(PlanError::DigestMismatch);
        }
        Ok(Self {
            probe_name,
            target,
            target_digest,
            selector,
        })
    }

    #[must_use]
    pub fn target_digest(&self) -> &str {
        &self.target_digest
    }

    #[must_use]
    pub fn probe_name(&self) -> &str {
        &self.probe_name
    }

    #[must_use]
    pub fn selector(&self) -> &ComponentSelector {
        &self.selector
    }
}

impl fmt::Debug for ComparisonPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ComparisonPlan")
            .field("probe_name", &self.probe_name)
            .field("target", &"<transient-redacted>")
            .field("target_digest", &self.target_digest)
            .field("selector", &self.selector)
            .finish()
    }
}

impl Drop for ComparisonPlan {
    fn drop(&mut self) {
        self.target.zeroize();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanError {
    InvalidProbeName,
    EmptyTarget,
    InvalidUtf8,
    DigestMismatch,
    UnsupportedSelector,
    DuplicateProbeName,
}

pub fn validate_plan_set(plans: &[ComparisonPlan]) -> Result<(), PlanError> {
    let mut names = BTreeSet::new();
    for plan in plans {
        if !names.insert(plan.probe_name()) {
            return Err(PlanError::DuplicateProbeName);
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceSpan {
    pub offset: usize,
    pub length: usize,
}

impl SourceSpan {
    fn intersects(self, other: Self) -> bool {
        self.offset < other.offset.saturating_add(other.length)
            && other.offset < self.offset.saturating_add(self.length)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RedactionScan {
    Complete { replacements: Vec<SourceSpan> },
    Incomplete,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservationInput<'a> {
    pub binding: &'a Binding,
    pub sequence: u64,
    pub exchange_digest: &'a str,
    pub provider: ProviderFamily,
    pub operation: &'a str,
    pub request_body: &'a [u8],
    pub request_complete: bool,
    pub identity_encoded: bool,
    pub content_observation_enabled: bool,
    pub redaction_scan: RedactionScan,
    pub producer_identity: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservationStatus {
    ObservedPresent,
    ObservedAbsent,
    Unknown,
}

impl ObservationStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ObservedPresent => "observed-present",
            Self::ObservedAbsent => "observed-absent",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnknownReason {
    ContentObservationDisabled,
    RequestIncomplete,
    CompressedRequest,
    MalformedRequest,
    UnsupportedProviderShape,
    ComponentAbsent,
    TextPartAbsent,
    ComponentNotText,
    RedactionScanIncomplete,
    RedactionIntersectsComponent,
}

impl UnknownReason {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ContentObservationDisabled => "content-observation-disabled",
            Self::RequestIncomplete => "request-incomplete",
            Self::CompressedRequest => "compressed-request",
            Self::MalformedRequest => "malformed-request",
            Self::UnsupportedProviderShape => "unsupported-provider-shape",
            Self::ComponentAbsent => "component-absent",
            Self::TextPartAbsent => "text-part-absent",
            Self::ComponentNotText => "component-not-text",
            Self::RedactionScanIncomplete => "redaction-scan-incomplete",
            Self::RedactionIntersectsComponent => "redaction-intersects-component",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservationFinding {
    pub code: String,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstructionObservation {
    pub binding: Binding,
    pub sequence: u64,
    pub exchange_digest: String,
    pub probe_name: String,
    pub target_digest: String,
    pub target_length: u64,
    pub selector: ComponentSelector,
    pub component_digest: Option<String>,
    pub component_length: Option<u64>,
    pub status: ObservationStatus,
    pub match_count: Option<u64>,
    pub match_offsets: Vec<u64>,
    pub request_complete: bool,
    pub component_complete: bool,
    pub decoder_identity: String,
    pub producer_identity: String,
    pub findings: Vec<ObservationFinding>,
    pub unknown_reason: Option<UnknownReason>,
}

impl InstructionObservation {
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
            (
                "component_digest",
                optional_string(self.component_digest.as_deref()),
            ),
            ("component_length", optional_u64(self.component_length)),
            (
                "component_complete",
                JsonValue::Bool(self.component_complete),
            ),
            ("decoder_identity", string(&self.decoder_identity)),
            ("exchange_digest", string(&self.exchange_digest)),
            (
                "findings",
                JsonValue::Array(
                    self.findings
                        .iter()
                        .map(|finding| {
                            object([
                                ("code", string(&finding.code)),
                                ("detail", string(&finding.detail)),
                            ])
                        })
                        .collect(),
                ),
            ),
            ("match_count", optional_u64(self.match_count)),
            (
                "match_offsets",
                JsonValue::Array(
                    self.match_offsets
                        .iter()
                        .map(|offset| JsonValue::Number(offset.to_string()))
                        .collect(),
                ),
            ),
            ("matching_mode", string(MATCHING_MODE)),
            ("probe_name", string(&self.probe_name)),
            ("producer_identity", string(&self.producer_identity)),
            ("request_complete", JsonValue::Bool(self.request_complete)),
            ("schema", string(SCHEMA)),
            ("selector", selector_json(&self.selector)),
            ("sequence", JsonValue::Number(self.sequence.to_string())),
            ("status", string(self.status.as_str())),
            ("target_digest", string(&self.target_digest)),
            (
                "target_digest_construction",
                string(TARGET_DIGEST_CONSTRUCTION),
            ),
            (
                "target_length",
                JsonValue::Number(self.target_length.to_string()),
            ),
            (
                "unknown_reason",
                self.unknown_reason
                    .map(UnknownReason::as_str)
                    .map_or(JsonValue::Null, string),
            ),
        ])
    }
}

#[must_use]
pub fn observe(plan: &ComparisonPlan, input: ObservationInput<'_>) -> InstructionObservation {
    let unknown = |reason, detail: &str| unknown_observation(plan, &input, reason, detail);
    if !input.content_observation_enabled {
        return unknown(
            UnknownReason::ContentObservationDisabled,
            "content observation was disabled",
        );
    }
    if !input.request_complete {
        return unknown(
            UnknownReason::RequestIncomplete,
            "request capture was incomplete",
        );
    }
    if !input.identity_encoded {
        return unknown(
            UnknownReason::CompressedRequest,
            "request body was not identity encoded",
        );
    }
    if input.provider != plan.selector.provider || input.operation != plan.selector.operation {
        return unknown(
            UnknownReason::UnsupportedProviderShape,
            "exchange identity does not match the predeclared selector",
        );
    }
    let replacements = match &input.redaction_scan {
        RedactionScan::Complete { replacements } => replacements,
        RedactionScan::Incomplete => {
            return unknown(
                UnknownReason::RedactionScanIncomplete,
                "mandatory redaction scan did not complete",
            );
        }
    };
    let spans = match collect_string_spans(input.request_body) {
        Ok(spans) => spans,
        Err(()) => {
            return unknown(
                UnknownReason::MalformedRequest,
                "request string spans could not be decoded",
            );
        }
    };
    let root = match parse_json(input.request_body) {
        Ok(root) => root,
        Err(_) => {
            return unknown(
                UnknownReason::MalformedRequest,
                "request body was not strict JSON",
            );
        }
    };
    let selected = match select_component(&root, &plan.selector) {
        Ok(value) => value,
        Err(reason) => return unknown(reason, reason.as_str()),
    };
    let Some(source_span) = spans.get(&selected.path).copied() else {
        return unknown(
            UnknownReason::MalformedRequest,
            "selected component had no source span",
        );
    };
    if replacements
        .iter()
        .any(|replacement| source_span.intersects(*replacement))
    {
        return unknown(
            UnknownReason::RedactionIntersectsComponent,
            "mandatory redaction touched the selected component",
        );
    }
    let component = selected.value.as_bytes();
    let match_offsets = non_overlapping_matches(component, &plan.target);
    let status = if match_offsets.is_empty() {
        ObservationStatus::ObservedAbsent
    } else {
        ObservationStatus::ObservedPresent
    };
    InstructionObservation {
        binding: input.binding.clone(),
        sequence: input.sequence,
        exchange_digest: input.exchange_digest.to_owned(),
        probe_name: plan.probe_name.clone(),
        target_digest: plan.target_digest.clone(),
        target_length: plan.target.len() as u64,
        selector: plan.selector.clone(),
        component_digest: Some(sha256_hex(component)),
        component_length: Some(component.len() as u64),
        status,
        match_count: Some(match_offsets.len() as u64),
        match_offsets: match_offsets
            .into_iter()
            .map(|offset| offset as u64)
            .collect(),
        request_complete: true,
        component_complete: true,
        decoder_identity: DECODER_IDENTITY.into(),
        producer_identity: input.producer_identity.to_owned(),
        findings: Vec::new(),
        unknown_reason: None,
    }
}

fn unknown_observation(
    plan: &ComparisonPlan,
    input: &ObservationInput<'_>,
    reason: UnknownReason,
    detail: &str,
) -> InstructionObservation {
    InstructionObservation {
        binding: input.binding.clone(),
        sequence: input.sequence,
        exchange_digest: input.exchange_digest.to_owned(),
        probe_name: plan.probe_name.clone(),
        target_digest: plan.target_digest.clone(),
        target_length: plan.target.len() as u64,
        selector: plan.selector.clone(),
        component_digest: None,
        component_length: None,
        status: ObservationStatus::Unknown,
        match_count: None,
        match_offsets: Vec::new(),
        request_complete: input.request_complete,
        component_complete: false,
        decoder_identity: DECODER_IDENTITY.into(),
        producer_identity: input.producer_identity.to_owned(),
        findings: vec![ObservationFinding {
            code: reason.as_str().into(),
            detail: detail.into(),
        }],
        unknown_reason: Some(reason),
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum PathPart {
    Key(String),
    Index(usize),
}

struct Selected<'a> {
    value: &'a str,
    path: Vec<PathPart>,
}

fn select_component<'a>(
    root: &'a JsonValue,
    selector: &ComponentSelector,
) -> Result<Selected<'a>, UnknownReason> {
    match (
        &selector.provider,
        selector.operation.as_str(),
        selector.kind,
    ) {
        (ProviderFamily::Anthropic, "POST /v1/messages", ComponentKind::System) => {
            select_text(root, "system", 0, selector.text_part_index)
        }
        (ProviderFamily::Anthropic, "POST /v1/messages", ComponentKind::Message)
        | (ProviderFamily::OpenAi, "POST /v1/chat/completions", ComponentKind::Message) => {
            select_array_content(
                root,
                "messages",
                selector.component_index,
                selector.text_part_index,
            )
        }
        (ProviderFamily::OpenAi, "POST /v1/responses", ComponentKind::Instructions) => {
            select_text(root, "instructions", 0, 0)
        }
        (ProviderFamily::OpenAi, "POST /v1/responses", ComponentKind::Input) => {
            select_openai_input(root, selector.component_index, selector.text_part_index)
        }
        _ => Err(UnknownReason::UnsupportedProviderShape),
    }
}

fn select_text<'a>(
    root: &'a JsonValue,
    key: &str,
    component_index: usize,
    text_part_index: usize,
) -> Result<Selected<'a>, UnknownReason> {
    let object = as_root_object(root)?;
    let value = object.get(key).ok_or(UnknownReason::ComponentAbsent)?;
    let base = vec![PathPart::Key(key.into())];
    match value {
        JsonValue::String(text) if component_index == 0 && text_part_index == 0 => Ok(Selected {
            value: text,
            path: base,
        }),
        JsonValue::String(_) => Err(UnknownReason::TextPartAbsent),
        JsonValue::Array(parts) if component_index == 0 => {
            select_text_part(parts, base, text_part_index, false)
        }
        _ => Err(UnknownReason::ComponentNotText),
    }
}

fn select_array_content<'a>(
    root: &'a JsonValue,
    key: &str,
    component_index: usize,
    text_part_index: usize,
) -> Result<Selected<'a>, UnknownReason> {
    let object = as_root_object(root)?;
    let JsonValue::Array(components) = object.get(key).ok_or(UnknownReason::ComponentAbsent)?
    else {
        return Err(UnknownReason::UnsupportedProviderShape);
    };
    let component = components
        .get(component_index)
        .ok_or(UnknownReason::ComponentAbsent)?;
    let component = as_object(component)?;
    let content = component
        .get("content")
        .ok_or(UnknownReason::ComponentAbsent)?;
    let base = vec![
        PathPart::Key(key.into()),
        PathPart::Index(component_index),
        PathPart::Key("content".into()),
    ];
    match content {
        JsonValue::String(text) if text_part_index == 0 => Ok(Selected {
            value: text,
            path: base,
        }),
        JsonValue::String(_) => Err(UnknownReason::TextPartAbsent),
        JsonValue::Array(parts) => select_text_part(parts, base, text_part_index, false),
        _ => Err(UnknownReason::ComponentNotText),
    }
}

fn select_openai_input(
    root: &JsonValue,
    component_index: usize,
    text_part_index: usize,
) -> Result<Selected<'_>, UnknownReason> {
    let object = as_root_object(root)?;
    let input = object.get("input").ok_or(UnknownReason::ComponentAbsent)?;
    if let JsonValue::String(text) = input {
        return if component_index != 0 {
            Err(UnknownReason::ComponentAbsent)
        } else if text_part_index != 0 {
            Err(UnknownReason::TextPartAbsent)
        } else {
            Ok(Selected {
                value: text,
                path: vec![PathPart::Key("input".into())],
            })
        };
    }
    let JsonValue::Array(components) = input else {
        return Err(UnknownReason::ComponentNotText);
    };
    let component = components
        .get(component_index)
        .ok_or(UnknownReason::ComponentAbsent)?;
    let base = vec![
        PathPart::Key("input".into()),
        PathPart::Index(component_index),
    ];
    match component {
        JsonValue::String(text) if text_part_index == 0 => Ok(Selected {
            value: text,
            path: base,
        }),
        JsonValue::String(_) => Err(UnknownReason::TextPartAbsent),
        JsonValue::Object(object) => {
            let content = object
                .get("content")
                .ok_or(UnknownReason::ComponentAbsent)?;
            let mut content_path = base;
            content_path.push(PathPart::Key("content".into()));
            match content {
                JsonValue::String(text) if text_part_index == 0 => Ok(Selected {
                    value: text,
                    path: content_path,
                }),
                JsonValue::String(_) => Err(UnknownReason::TextPartAbsent),
                JsonValue::Array(parts) => {
                    select_text_part(parts, content_path, text_part_index, true)
                }
                _ => Err(UnknownReason::ComponentNotText),
            }
        }
        _ => Err(UnknownReason::ComponentNotText),
    }
}

fn select_text_part<'a>(
    parts: &'a [JsonValue],
    base: Vec<PathPart>,
    text_part_index: usize,
    allow_input_text: bool,
) -> Result<Selected<'a>, UnknownReason> {
    let mut text_position = 0;
    for (array_index, part) in parts.iter().enumerate() {
        let JsonValue::Object(part) = part else {
            continue;
        };
        let is_text = matches!(
            part.get("type"),
            Some(JsonValue::String(kind))
                if kind == "text" || (allow_input_text && kind == "input_text")
        );
        if !is_text {
            continue;
        }
        if text_position == text_part_index {
            let JsonValue::String(text) =
                part.get("text").ok_or(UnknownReason::ComponentNotText)?
            else {
                return Err(UnknownReason::ComponentNotText);
            };
            let mut path = base;
            path.extend([PathPart::Index(array_index), PathPart::Key("text".into())]);
            return Ok(Selected { value: text, path });
        }
        text_position += 1;
    }
    Err(UnknownReason::TextPartAbsent)
}

fn as_root_object(value: &JsonValue) -> Result<&BTreeMap<String, JsonValue>, UnknownReason> {
    match value {
        JsonValue::Object(value) => Ok(value),
        _ => Err(UnknownReason::MalformedRequest),
    }
}

fn as_object(value: &JsonValue) -> Result<&BTreeMap<String, JsonValue>, UnknownReason> {
    match value {
        JsonValue::Object(value) => Ok(value),
        _ => Err(UnknownReason::UnsupportedProviderShape),
    }
}

fn non_overlapping_matches(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
    if needle.is_empty() {
        return Vec::new();
    }
    let mut offsets = Vec::new();
    let mut cursor = 0;
    while cursor + needle.len() <= haystack.len() {
        if haystack[cursor..].starts_with(needle) {
            offsets.push(cursor);
            cursor += needle.len();
        } else {
            cursor += 1;
        }
    }
    offsets
}

fn collect_string_spans(bytes: &[u8]) -> Result<BTreeMap<Vec<PathPart>, SourceSpan>, ()> {
    let mut scanner = SpanScanner {
        bytes,
        offset: 0,
        spans: BTreeMap::new(),
    };
    scanner.value(&mut Vec::new(), 0)?;
    scanner.space();
    if scanner.offset == bytes.len() {
        Ok(scanner.spans)
    } else {
        Err(())
    }
}

struct SpanScanner<'a> {
    bytes: &'a [u8],
    offset: usize,
    spans: BTreeMap<Vec<PathPart>, SourceSpan>,
}

impl SpanScanner<'_> {
    fn space(&mut self) {
        while self
            .bytes
            .get(self.offset)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.offset += 1;
        }
    }

    fn value(&mut self, path: &mut Vec<PathPart>, depth: usize) -> Result<(), ()> {
        if depth > MAX_JSON_NESTING {
            return Err(());
        }
        self.space();
        match self.bytes.get(self.offset) {
            Some(b'"') => {
                let start = self.offset;
                self.string()?;
                self.spans.insert(
                    path.clone(),
                    SourceSpan {
                        offset: start,
                        length: self.offset - start,
                    },
                );
                Ok(())
            }
            Some(b'{') => self.object(path, depth),
            Some(b'[') => self.array(path, depth),
            Some(b'n') => self.literal(b"null"),
            Some(b't') => self.literal(b"true"),
            Some(b'f') => self.literal(b"false"),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(()),
        }
    }

    fn object(&mut self, path: &mut Vec<PathPart>, depth: usize) -> Result<(), ()> {
        self.offset += 1;
        self.space();
        if self.take(b'}') {
            return Ok(());
        }
        loop {
            self.space();
            let key = self.string()?;
            self.space();
            if !self.take(b':') {
                return Err(());
            }
            path.push(PathPart::Key(key));
            self.value(path, depth + 1)?;
            path.pop();
            self.space();
            if self.take(b'}') {
                return Ok(());
            }
            if !self.take(b',') {
                return Err(());
            }
        }
    }

    fn array(&mut self, path: &mut Vec<PathPart>, depth: usize) -> Result<(), ()> {
        self.offset += 1;
        self.space();
        if self.take(b']') {
            return Ok(());
        }
        let mut index = 0;
        loop {
            path.push(PathPart::Index(index));
            self.value(path, depth + 1)?;
            path.pop();
            index += 1;
            self.space();
            if self.take(b']') {
                return Ok(());
            }
            if !self.take(b',') {
                return Err(());
            }
        }
    }

    fn string(&mut self) -> Result<String, ()> {
        let start = self.offset;
        if !self.take(b'"') {
            return Err(());
        }
        let mut escaped = false;
        while let Some(byte) = self.bytes.get(self.offset).copied() {
            self.offset += 1;
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                let token = self.bytes.get(start..self.offset).ok_or(())?;
                let JsonValue::String(value) = parse_json(token).map_err(|_| ())? else {
                    return Err(());
                };
                return Ok(value);
            }
        }
        Err(())
    }

    fn literal(&mut self, literal: &[u8]) -> Result<(), ()> {
        if self.bytes.get(self.offset..self.offset + literal.len()) == Some(literal) {
            self.offset += literal.len();
            Ok(())
        } else {
            Err(())
        }
    }

    fn number(&mut self) -> Result<(), ()> {
        let start = self.offset;
        while self.bytes.get(self.offset).is_some_and(|byte| {
            byte.is_ascii_digit() || matches!(byte, b'-' | b'+' | b'.' | b'e' | b'E')
        }) {
            self.offset += 1;
        }
        (self.offset > start).then_some(()).ok_or(())
    }

    fn take(&mut self, byte: u8) -> bool {
        if self.bytes.get(self.offset) == Some(&byte) {
            self.offset += 1;
            true
        } else {
            false
        }
    }
}

fn selector_json(selector: &ComponentSelector) -> JsonValue {
    object([
        (
            "component_index",
            JsonValue::Number(selector.component_index.to_string()),
        ),
        ("component_kind", string(selector.kind.as_str())),
        ("operation", string(&selector.operation)),
        ("provider", string(provider_name(&selector.provider))),
        (
            "text_part_index",
            JsonValue::Number(selector.text_part_index.to_string()),
        ),
    ])
}

fn provider_name(provider: &ProviderFamily) -> &'static str {
    match provider {
        ProviderFamily::Anthropic => "anthropic",
        ProviderFamily::OpenAi => "openai",
        ProviderFamily::Unknown => "unknown",
    }
}

fn binding_json(binding: &Binding) -> JsonValue {
    match binding {
        Binding::Supervised {
            run_id,
            attempt,
            effect_id,
        } => object([
            ("attempt", JsonValue::Number(attempt.to_string())),
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

fn optional_string(value: Option<&str>) -> JsonValue {
    value.map_or(JsonValue::Null, string)
}

fn optional_u64(value: Option<u64>) -> JsonValue {
    value.map_or(JsonValue::Null, |value| {
        JsonValue::Number(value.to_string())
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(selector: ComponentSelector, target: &[u8]) -> ComparisonPlan {
        ComparisonPlan::new(
            "probe".into(),
            target.to_vec(),
            &sha256_hex(target),
            selector,
        )
        .unwrap()
    }

    fn anthropic(
        kind: ComponentKind,
        component_index: usize,
        text_part_index: usize,
    ) -> ComponentSelector {
        ComponentSelector {
            provider: ProviderFamily::Anthropic,
            operation: "POST /v1/messages".into(),
            kind,
            component_index,
            text_part_index,
        }
    }

    fn input<'a>(body: &'a [u8], binding: &'a Binding) -> ObservationInput<'a> {
        ObservationInput {
            binding,
            sequence: 3,
            exchange_digest: "exchange-digest",
            provider: ProviderFamily::Anthropic,
            operation: "POST /v1/messages",
            request_body: body,
            request_complete: true,
            identity_encoded: true,
            content_observation_enabled: true,
            redaction_scan: RedactionScan::Complete {
                replacements: Vec::new(),
            },
            producer_identity: "wire-witness-test",
        }
    }

    #[test]
    fn observes_non_overlapping_exact_matches_in_decoded_text() {
        let binding = Binding::Unsupervised {
            session_id: "session".into(),
        };
        let body = br#"{"system":[{"type":"text","text":"say \"go go\" now"}],"messages":[]}"#;
        let result = observe(
            &plan(anthropic(ComponentKind::System, 0, 0), b"go"),
            input(body, &binding),
        );
        assert_eq!(result.status, ObservationStatus::ObservedPresent);
        assert_eq!(result.match_count, Some(2));
        assert_eq!(result.match_offsets, vec![5, 8]);
        assert!(result.component_digest.is_some());
        let canonical = String::from_utf8(result.canonical_bytes()).unwrap();
        assert!(canonical.contains(SCHEMA));
        assert!(!canonical.contains("go go"));
    }

    #[test]
    fn unicode_escaped_quote_does_not_end_source_string_early() {
        let binding = Binding::Unsupervised {
            session_id: "session".into(),
        };
        let body = br#"{"system":"say \u0022go\u0022 now","messages":[]}"#;
        let result = observe(
            &plan(anthropic(ComponentKind::System, 0, 0), b"\"go\""),
            input(body, &binding),
        );
        assert_eq!(result.status, ObservationStatus::ObservedPresent);
        assert_eq!(result.match_count, Some(1));
        assert_eq!(result.match_offsets, vec![4]);
    }

    #[test]
    fn closed_unknown_reasons_match_request_structure_failures() {
        let binding = Binding::Unsupervised {
            session_id: "session".into(),
        };
        let cases = [
            (
                br#"[]"#.as_slice(),
                anthropic(ComponentKind::System, 0, 0),
                UnknownReason::MalformedRequest,
            ),
            (
                br#"{"messages":[]}"#.as_slice(),
                anthropic(ComponentKind::System, 0, 0),
                UnknownReason::ComponentAbsent,
            ),
            (
                br#"{"system":[{"type":"text","text":"first"}]}"#.as_slice(),
                anthropic(ComponentKind::System, 0, 1),
                UnknownReason::TextPartAbsent,
            ),
            (
                br#"{"system":{"text":"not a supported component"}}"#.as_slice(),
                anthropic(ComponentKind::System, 0, 0),
                UnknownReason::ComponentNotText,
            ),
        ];
        for (body, selector, expected) in cases {
            let result = observe(&plan(selector, b"needle"), input(body, &binding));
            assert_eq!(result.status, ObservationStatus::Unknown);
            assert_eq!(result.unknown_reason, Some(expected));
        }
    }

    #[test]
    fn empty_match_needle_is_defensively_bounded() {
        assert!(non_overlapping_matches(b"content", b"").is_empty());
    }

    #[test]
    fn does_not_search_unselected_components() {
        let binding = Binding::Unsupervised {
            session_id: "session".into(),
        };
        let body = br#"{"system":"safe","messages":[{"content":"needle"}]}"#;
        let result = observe(
            &plan(anthropic(ComponentKind::System, 0, 0), b"needle"),
            input(body, &binding),
        );
        assert_eq!(result.status, ObservationStatus::ObservedAbsent);
        assert_eq!(result.match_count, Some(0));
    }

    #[test]
    fn redaction_overlap_is_unknown_and_omits_component_identity() {
        let binding = Binding::Unsupervised {
            session_id: "session".into(),
        };
        let body = br#"{"system":"secret","messages":[]}"#;
        let mut observation = input(body, &binding);
        observation.redaction_scan = RedactionScan::Complete {
            replacements: vec![SourceSpan {
                offset: 10,
                length: 6,
            }],
        };
        let result = observe(
            &plan(anthropic(ComponentKind::System, 0, 0), b"secret"),
            observation,
        );
        assert_eq!(result.status, ObservationStatus::Unknown);
        assert_eq!(
            result.unknown_reason,
            Some(UnknownReason::RedactionIntersectsComponent)
        );
        assert_eq!(result.component_digest, None);
        assert_eq!(result.component_length, None);
    }

    #[test]
    fn incomplete_or_disabled_capture_can_never_report_absence() {
        let binding = Binding::Unsupervised {
            session_id: "session".into(),
        };
        let body = br#"{"system":"other","messages":[]}"#;
        let target = plan(anthropic(ComponentKind::System, 0, 0), b"needle");
        let mut incomplete = input(body, &binding);
        incomplete.request_complete = false;
        assert_eq!(
            observe(&target, incomplete).status,
            ObservationStatus::Unknown
        );
        let mut disabled = input(body, &binding);
        disabled.content_observation_enabled = false;
        assert_eq!(
            observe(&target, disabled).status,
            ObservationStatus::Unknown
        );
    }

    #[test]
    fn plan_validation_refuses_bad_inputs_and_duplicate_names() {
        let selector = anthropic(ComponentKind::System, 0, 0);
        assert_eq!(
            ComparisonPlan::new(
                "probe".into(),
                Vec::new(),
                &sha256_hex(b""),
                selector.clone()
            ),
            Err(PlanError::EmptyTarget)
        );
        assert_eq!(
            ComparisonPlan::new("probe".into(), b"x".to_vec(), "wrong", selector.clone()),
            Err(PlanError::DigestMismatch)
        );
        let plans = vec![plan(selector.clone(), b"a"), plan(selector, b"b")];
        assert_eq!(
            validate_plan_set(&plans),
            Err(PlanError::DuplicateProbeName)
        );
        let secret = plan(anthropic(ComponentKind::System, 0, 0), b"never-log-this");
        assert!(!format!("{secret:?}").contains("never-log-this"));
    }

    #[test]
    fn openai_closed_shapes_select_only_text_parts() {
        let binding = Binding::Unsupervised {
            session_id: "session".into(),
        };
        let body = br#"{"input":[{"content":[{"type":"input_image","image_url":"needle"},{"type":"input_text","text":"needle"}]}]}"#;
        let selector = ComponentSelector {
            provider: ProviderFamily::OpenAi,
            operation: "POST /v1/responses".into(),
            kind: ComponentKind::Input,
            component_index: 0,
            text_part_index: 0,
        };
        let mut observation = input(body, &binding);
        observation.provider = ProviderFamily::OpenAi;
        observation.operation = "POST /v1/responses";
        let result = observe(&plan(selector, b"needle"), observation);
        assert_eq!(result.status, ObservationStatus::ObservedPresent);
        assert_eq!(result.match_offsets, vec![0]);
    }

    #[test]
    fn excessive_json_nesting_is_unknown_without_recursive_parse() {
        let binding = Binding::Unsupervised {
            session_id: "session".into(),
        };
        let body = format!(
            "{}null{}",
            "[".repeat(MAX_JSON_NESTING + 1),
            "]".repeat(MAX_JSON_NESTING + 1)
        );
        let result = observe(
            &plan(anthropic(ComponentKind::System, 0, 0), b"needle"),
            input(body.as_bytes(), &binding),
        );
        assert_eq!(result.status, ObservationStatus::Unknown);
        assert_eq!(result.unknown_reason, Some(UnknownReason::MalformedRequest));
    }
}

// endregion
