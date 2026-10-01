//! Policy-bounded construction of immutable corpus export manifests.
//!
//! Governed by spec 010, policy-bounded corpus export.

// region: policy-bounded-corpus-export

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::exchange::{Binding, JsonValue, canonical_json_bytes, parse_json, sha256_hex};

pub const SCHEMA: &str = "wire-witness.corpus/1";
pub const DIGEST_CONSTRUCTION: &str = "wire-witness.corpus/1+keysort-json+sha256";
pub const POLICY_SCHEMA: &str = "wire-witness.export-policy-receipt/1";
pub const POLICY_DIGEST_CONSTRUCTION: &str =
    "wire-witness.export-policy-receipt/1+keysort-json+sha256";
const MAX_RECEIPT_BYTES: usize = 65_536;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SourceKind {
    Exchange,
    InstructionObservation,
    MeasurementSummary,
}

impl SourceKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Exchange => "exchange",
            Self::InstructionObservation => "instruction-observation",
            Self::MeasurementSummary => "measurement-summary",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "exchange" => Some(Self::Exchange),
            "instruction-observation" => Some(Self::InstructionObservation),
            "measurement-summary" => Some(Self::MeasurementSummary),
            _ => None,
        }
    }

    fn accepted_schema(self) -> &'static str {
        match self {
            Self::Exchange => crate::exchange::SCHEMA,
            Self::InstructionObservation => crate::instruction_observation::SCHEMA,
            Self::MeasurementSummary => crate::measurement_summary::SCHEMA,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MaterializationMode {
    DigestOnly,
    Embedded,
    ExternalReference,
}

impl MaterializationMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::DigestOnly => "digest-only",
            Self::Embedded => "embedded",
            Self::ExternalReference => "external-reference",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "digest-only" => Some(Self::DigestOnly),
            "embedded" => Some(Self::Embedded),
            "external-reference" => Some(Self::ExternalReference),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionMode {
    Disabled,
    MetadataOnly,
    Content,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceCompleteness {
    Complete,
    Incomplete,
    Absent,
}

impl SourceCompleteness {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Incomplete => "incomplete",
            Self::Absent => "absent",
        }
    }
}

impl RetentionMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::MetadataOnly => "metadata-only",
            Self::Content => "content",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestedBinding {
    pub binding: Binding,
    pub capture_digest: String,
}

#[derive(Clone, Debug)]
pub struct SourceInput<'a> {
    pub binding: &'a Binding,
    pub capture_digest: &'a str,
    pub kind: SourceKind,
    pub schema: &'a str,
    pub sequence: Option<u64>,
    pub claimed_digest: String,
    pub claimed_length: u64,
    pub canonical_bytes: &'a [u8],
    pub retention: RetentionMode,
    pub expires_at: Option<&'a str>,
    pub completeness: SourceCompleteness,
    pub findings: Vec<String>,
    pub source_exchange_manifest: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ArtifactInput<'a> {
    pub artifact_id: &'a str,
    pub binding: &'a Binding,
    pub capture_digest: &'a str,
    pub media_type: &'a str,
    pub claimed_digest: String,
    pub claimed_length: u64,
    pub redacted_bytes: Option<&'a [u8]>,
    pub retention: RetentionMode,
    pub expires_at: Option<&'a str>,
    pub erased: bool,
    pub redaction_complete: bool,
    pub mode: MaterializationMode,
    pub custody_reference: Option<&'a str>,
    pub custodian_identity: Option<&'a str>,
    pub findings: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportLimits {
    pub max_bindings: u64,
    pub max_entries: u64,
    pub max_artifacts: u64,
    pub max_artifact_bytes: u64,
    pub max_total_bytes: u64,
}

#[derive(Clone, Debug)]
pub struct ExportRequest<'a> {
    pub export_id: &'a str,
    pub producer_identity: &'a str,
    pub created_at: &'a str,
    pub maximum_expires_at: &'a str,
    pub policy_receipt: &'a [u8],
    pub bindings: Vec<RequestedBinding>,
    pub sources: Vec<SourceInput<'a>>,
    pub artifacts: Vec<ArtifactInput<'a>>,
    pub allow_incomplete: bool,
    pub limits: ExportLimits,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExportError {
    InvalidRequest,
    LimitExceeded(&'static str),
    MalformedReceipt,
    PolicyDenied,
    PolicyUnverified,
    PolicyVerificationFailed,
    ReceiptExpired,
    ScopeMismatch,
    ForeignBinding,
    InvalidSourceDigest,
    InvalidArtifactDigest,
    ConflictingSequence,
    IneligibleArtifact(&'static str),
    UnsupportedSourceVersion,
    IncompleteNotPermitted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyReceipt {
    pub decision_schema: String,
    pub decision_id: String,
    pub decision_digest: String,
    pub decided_at: String,
    pub expires_at: String,
    pub outcome: String,
    pub asserted_principal: String,
    pub verifier: String,
    pub verification_outcome: String,
    pub scope_bindings: Vec<RequestedBinding>,
    pub source_kinds: Vec<SourceKind>,
    pub materialization_modes: Vec<MaterializationMode>,
    pub digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceEntry {
    pub binding: Binding,
    pub capture_digest: String,
    pub kind: SourceKind,
    pub schema: String,
    pub sequence: Option<u64>,
    pub digest: String,
    pub byte_length: u64,
    pub retention: RetentionMode,
    pub expires_at: Option<String>,
    pub completeness: SourceCompleteness,
    pub findings: Vec<String>,
    pub source_exchange_manifest: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactEntry {
    pub artifact_id: String,
    pub binding: Binding,
    pub capture_digest: String,
    pub media_type: String,
    pub digest: String,
    pub byte_length: u64,
    pub expiry: Option<String>,
    pub mode: MaterializationMode,
    pub redaction_complete: bool,
    pub embedded_base64: Option<String>,
    pub custody_reference: Option<String>,
    pub custodian_identity: Option<String>,
    pub findings: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportGap {
    pub binding: Binding,
    pub capture_digest: String,
    pub source_kind: Option<SourceKind>,
    pub artifact_id: Option<String>,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorpusManifest {
    pub export_id: String,
    pub created_at: String,
    pub expires_at: String,
    pub producer_identity: String,
    pub policy_receipt: PolicyReceipt,
    pub bindings: Vec<RequestedBinding>,
    pub sources: Vec<SourceEntry>,
    pub artifacts: Vec<ArtifactEntry>,
    pub gaps: Vec<ExportGap>,
    pub complete: bool,
    pub limits: ExportLimits,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorpusExport {
    pub manifest: CorpusManifest,
    pub manifest_bytes: Vec<u8>,
    pub manifest_digest: String,
}

impl CorpusManifest {
    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        canonical_json_bytes(&self.as_json())
    }

    fn as_json(&self) -> JsonValue {
        object([
            (
                "artifacts",
                JsonValue::Array(self.artifacts.iter().map(artifact_json).collect()),
            ),
            (
                "bindings",
                JsonValue::Array(self.bindings.iter().map(requested_binding_json).collect()),
            ),
            ("complete", JsonValue::Bool(self.complete)),
            (
                "counts",
                object([
                    ("artifacts", number(self.artifacts.len())),
                    ("bindings", number(self.bindings.len())),
                    ("gaps", number(self.gaps.len())),
                    ("sources", number(self.sources.len())),
                ]),
            ),
            ("created_at", string(&self.created_at)),
            ("digest_construction", string(DIGEST_CONSTRUCTION)),
            ("expires_at", string(&self.expires_at)),
            ("export_id", string(&self.export_id)),
            (
                "gaps",
                JsonValue::Array(self.gaps.iter().map(gap_json).collect()),
            ),
            ("limits", limits_json(self.limits)),
            ("policy_receipt", receipt_json(&self.policy_receipt)),
            ("producer_identity", string(&self.producer_identity)),
            ("schema", string(SCHEMA)),
            (
                "sources",
                JsonValue::Array(self.sources.iter().map(source_json).collect()),
            ),
        ])
    }
}

pub fn construct_export(request: ExportRequest<'_>) -> Result<CorpusExport, ExportError> {
    validate_request(&request)?;
    let created = Timestamp::parse(request.created_at).ok_or(ExportError::InvalidRequest)?;
    let maximum_expiry =
        Timestamp::parse(request.maximum_expires_at).ok_or(ExportError::InvalidRequest)?;
    if maximum_expiry <= created {
        return Err(ExportError::InvalidRequest);
    }
    let receipt = parse_receipt(request.policy_receipt, &created)?;
    validate_limits(&request)?;

    let requested_keys: BTreeSet<_> = request.bindings.iter().map(binding_key).collect();
    let scope_binding_keys: BTreeSet<_> = receipt.scope_bindings.iter().map(binding_key).collect();
    if !requested_keys.is_subset(&scope_binding_keys) {
        return Err(ExportError::ScopeMismatch);
    }
    let scope_kinds: BTreeSet<_> = receipt.source_kinds.iter().copied().collect();
    let scope_modes: BTreeSet<_> = receipt.materialization_modes.iter().copied().collect();

    let mut sources = Vec::new();
    let mut gaps = Vec::new();
    let mut sequence_keys = BTreeSet::new();
    let mut source_keys = BTreeSet::new();
    for source in request.sources {
        let requested = RequestedBinding {
            binding: source.binding.clone(),
            capture_digest: source.capture_digest.to_owned(),
        };
        if !requested_keys.contains(&binding_key(&requested)) {
            return Err(ExportError::ForeignBinding);
        }
        if !scope_kinds.contains(&source.kind) {
            return Err(ExportError::ScopeMismatch);
        }
        if source.claimed_length != source.canonical_bytes.len() as u64
            || source.claimed_digest != sha256_hex(source.canonical_bytes)
        {
            return Err(ExportError::InvalidSourceDigest);
        }
        verify_source_identity(&source)?;
        if !source_keys.insert((
            binding_bytes(source.binding),
            source.kind,
            source.sequence,
            source.claimed_digest.clone(),
        )) {
            return Err(ExportError::InvalidRequest);
        }
        if let Some(expires_at) = source.expires_at {
            Timestamp::parse(expires_at).ok_or(ExportError::InvalidRequest)?;
        }
        for digest in &source.source_exchange_manifest {
            validate_digest(digest)?;
        }
        if matches!(
            source.kind,
            SourceKind::Exchange | SourceKind::InstructionObservation
        ) {
            let Some(sequence) = source.sequence else {
                return Err(ExportError::InvalidRequest);
            };
            let key = (binding_bytes(source.binding), source.kind, sequence);
            if !sequence_keys.insert(key) {
                return Err(ExportError::ConflictingSequence);
            }
        } else if source.sequence.is_some() {
            return Err(ExportError::InvalidRequest);
        }
        if source.schema != source.kind.accepted_schema() {
            let gap = ExportGap {
                binding: source.binding.clone(),
                capture_digest: source.capture_digest.into(),
                source_kind: Some(source.kind),
                artifact_id: None,
                reason: "unsupported-source-version".into(),
            };
            if !request.allow_incomplete {
                return Err(ExportError::UnsupportedSourceVersion);
            }
            gaps.push(gap);
            continue;
        }
        sources.push(SourceEntry {
            binding: source.binding.clone(),
            capture_digest: source.capture_digest.into(),
            kind: source.kind,
            schema: source.schema.into(),
            sequence: source.sequence,
            digest: source.claimed_digest,
            byte_length: source.claimed_length,
            retention: source.retention,
            expires_at: source.expires_at.map(str::to_owned),
            completeness: source.completeness,
            findings: source.findings,
            source_exchange_manifest: source.source_exchange_manifest,
        });
    }

    let mut expiry = minimum_timestamp(request.maximum_expires_at, &receipt.expires_at)?;
    let mut artifacts = Vec::new();
    let mut materialized_bytes = 0u64;
    let mut artifact_keys = BTreeSet::new();
    for artifact in request.artifacts {
        let requested = RequestedBinding {
            binding: artifact.binding.clone(),
            capture_digest: artifact.capture_digest.to_owned(),
        };
        if !requested_keys.contains(&binding_key(&requested)) {
            return Err(ExportError::ForeignBinding);
        }
        if !scope_modes.contains(&artifact.mode) {
            return Err(ExportError::ScopeMismatch);
        }
        validate_digest(&artifact.claimed_digest)
            .map_err(|_| ExportError::InvalidArtifactDigest)?;
        if !valid_identity(artifact.artifact_id)
            || !valid_identity(artifact.media_type)
            || !artifact_keys.insert((
                binding_bytes(artifact.binding),
                artifact.artifact_id.to_owned(),
            ))
        {
            return Err(ExportError::InvalidRequest);
        }
        if artifact.claimed_length > request.limits.max_artifact_bytes {
            return Err(ExportError::LimitExceeded("artifact-bytes"));
        }

        let content_mode = matches!(
            artifact.mode,
            MaterializationMode::Embedded | MaterializationMode::ExternalReference
        );
        let artifact_expiry = artifact
            .expires_at
            .map(|value| Timestamp::parse(value).ok_or(ExportError::InvalidRequest))
            .transpose()?;
        let ineligible_reason = if content_mode && artifact.retention != RetentionMode::Content {
            Some("retention-not-content")
        } else if content_mode && artifact.erased {
            Some("erased")
        } else if content_mode && !artifact.redaction_complete {
            Some("redaction-incomplete")
        } else if content_mode && artifact_expiry.is_none() {
            Some("expiry-absent")
        } else if content_mode
            && artifact_expiry
                .as_ref()
                .is_some_and(|value| value <= &created)
        {
            Some("expired")
        } else {
            None
        };
        if let Some(reason) = ineligible_reason {
            if !request.allow_incomplete {
                return Err(ExportError::IneligibleArtifact(reason));
            }
            gaps.push(ExportGap {
                binding: artifact.binding.clone(),
                capture_digest: artifact.capture_digest.into(),
                source_kind: None,
                artifact_id: Some(artifact.artifact_id.into()),
                reason: reason.into(),
            });
            continue;
        }

        let (embedded_base64, custody_reference, custodian_identity) = match artifact.mode {
            MaterializationMode::DigestOnly => (None, None, None),
            MaterializationMode::Embedded => {
                let bytes = artifact
                    .redacted_bytes
                    .ok_or(ExportError::IneligibleArtifact("bytes-absent"))?;
                if bytes.len() as u64 != artifact.claimed_length
                    || sha256_hex(bytes) != artifact.claimed_digest
                {
                    return Err(ExportError::InvalidArtifactDigest);
                }
                materialized_bytes = materialized_bytes
                    .checked_add(artifact.claimed_length)
                    .ok_or(ExportError::LimitExceeded("total-bytes"))?;
                (Some(base64(bytes)), None, None)
            }
            MaterializationMode::ExternalReference => {
                let reference = artifact
                    .custody_reference
                    .filter(|value| !value.is_empty())
                    .ok_or(ExportError::IneligibleArtifact("reference-absent"))?;
                let custodian = artifact
                    .custodian_identity
                    .filter(|value| !value.is_empty())
                    .ok_or(ExportError::IneligibleArtifact("custodian-absent"))?;
                (None, Some(reference.into()), Some(custodian.into()))
            }
        };
        if materialized_bytes > request.limits.max_total_bytes {
            return Err(ExportError::LimitExceeded("total-bytes"));
        }
        if content_mode {
            let artifact_expiry_text = artifact.expires_at.expect("eligible content has expiry");
            expiry = minimum_timestamp(&expiry, artifact_expiry_text)?;
        }
        artifacts.push(ArtifactEntry {
            artifact_id: artifact.artifact_id.into(),
            binding: artifact.binding.clone(),
            capture_digest: artifact.capture_digest.into(),
            media_type: artifact.media_type.into(),
            digest: artifact.claimed_digest,
            byte_length: artifact.claimed_length,
            expiry: artifact.expires_at.map(str::to_owned),
            mode: artifact.mode,
            redaction_complete: artifact.redaction_complete,
            embedded_base64,
            custody_reference,
            custodian_identity,
            findings: artifact.findings,
        });
    }

    sources.sort_by(source_order);
    artifacts.sort_by(artifact_order);
    gaps.sort_by(gap_order);
    let complete = gaps.is_empty();
    if !complete && !request.allow_incomplete {
        return Err(ExportError::IncompleteNotPermitted);
    }
    let mut bindings = request.bindings;
    bindings.sort_by_key(binding_key);
    let manifest = CorpusManifest {
        export_id: request.export_id.into(),
        created_at: request.created_at.into(),
        expires_at: expiry,
        producer_identity: request.producer_identity.into(),
        policy_receipt: receipt,
        bindings,
        sources,
        artifacts,
        gaps,
        complete,
        limits: request.limits,
    };
    let manifest_bytes = manifest.canonical_bytes();
    if manifest_bytes.len() as u64 > manifest.limits.max_total_bytes {
        return Err(ExportError::LimitExceeded("total-bytes"));
    }
    let manifest_digest = sha256_hex(&manifest_bytes);
    Ok(CorpusExport {
        manifest,
        manifest_bytes,
        manifest_digest,
    })
}

fn validate_request(request: &ExportRequest<'_>) -> Result<(), ExportError> {
    if !valid_identity(request.export_id)
        || !valid_identity(request.producer_identity)
        || request.bindings.is_empty()
    {
        return Err(ExportError::InvalidRequest);
    }
    let mut keys = BTreeSet::new();
    for binding in &request.bindings {
        validate_digest(&binding.capture_digest)?;
        if !keys.insert(binding_key(binding)) {
            return Err(ExportError::InvalidRequest);
        }
    }
    Ok(())
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty() && !value.contains(['\0', '\r', '\n'])
}

fn validate_limits(request: &ExportRequest<'_>) -> Result<(), ExportError> {
    let limits = request.limits;
    if limits.max_bindings == 0
        || limits.max_entries == 0
        || limits.max_artifacts == 0
        || limits.max_artifact_bytes == 0
        || limits.max_total_bytes == 0
    {
        return Err(ExportError::InvalidRequest);
    }
    if request.bindings.len() as u64 > limits.max_bindings {
        return Err(ExportError::LimitExceeded("bindings"));
    }
    if request.sources.len() as u64 > limits.max_entries {
        return Err(ExportError::LimitExceeded("entries"));
    }
    if request.artifacts.len() as u64 > limits.max_artifacts {
        return Err(ExportError::LimitExceeded("artifacts"));
    }
    let mut embedded_bytes = 0u64;
    for artifact in &request.artifacts {
        if artifact.claimed_length > limits.max_artifact_bytes {
            return Err(ExportError::LimitExceeded("artifact-bytes"));
        }
        if artifact.mode == MaterializationMode::Embedded {
            embedded_bytes = embedded_bytes
                .checked_add(artifact.claimed_length)
                .ok_or(ExportError::LimitExceeded("total-bytes"))?;
        }
    }
    if embedded_bytes > limits.max_total_bytes {
        return Err(ExportError::LimitExceeded("total-bytes"));
    }
    Ok(())
}

fn parse_receipt(bytes: &[u8], created: &Timestamp) -> Result<PolicyReceipt, ExportError> {
    if bytes.len() > MAX_RECEIPT_BYTES {
        return Err(ExportError::MalformedReceipt);
    }
    let value = parse_json(bytes).map_err(|_| ExportError::MalformedReceipt)?;
    if canonical_json_bytes(&value) != bytes {
        return Err(ExportError::MalformedReceipt);
    }
    let map = exact_object(
        &value,
        &[
            "schema",
            "decision_schema",
            "decision_id",
            "decision_digest",
            "decided_at",
            "expires_at",
            "outcome",
            "asserted_principal",
            "verifier",
            "verification_outcome",
            "scope",
        ],
    )?;
    if required_string(map, "schema")? != POLICY_SCHEMA {
        return Err(ExportError::MalformedReceipt);
    }
    let decision_schema = nonempty_string(map, "decision_schema")?;
    let decision_id = nonempty_string(map, "decision_id")?;
    let decision_digest = nonempty_string(map, "decision_digest")?;
    validate_prefixed_digest(&decision_digest)?;
    let decided_at = nonempty_string(map, "decided_at")?;
    let expires_at = nonempty_string(map, "expires_at")?;
    let decided = Timestamp::parse(&decided_at).ok_or(ExportError::MalformedReceipt)?;
    let expires = Timestamp::parse(&expires_at).ok_or(ExportError::MalformedReceipt)?;
    if decided > expires || decided > *created {
        return Err(ExportError::MalformedReceipt);
    }
    if expires <= *created {
        return Err(ExportError::ReceiptExpired);
    }
    let outcome = required_string(map, "outcome")?.to_owned();
    match outcome.as_str() {
        "allow" => {}
        "deny" => return Err(ExportError::PolicyDenied),
        _ => return Err(ExportError::MalformedReceipt),
    }
    let verification_outcome = required_string(map, "verification_outcome")?.to_owned();
    match verification_outcome.as_str() {
        "verified" => {}
        "unverified" => return Err(ExportError::PolicyUnverified),
        "failed" => return Err(ExportError::PolicyVerificationFailed),
        _ => return Err(ExportError::MalformedReceipt),
    }
    let asserted_principal = nonempty_string(map, "asserted_principal")?;
    let verifier = nonempty_string(map, "verifier")?;
    let scope = exact_object(
        required(map, "scope")?,
        &["bindings", "source_kinds", "materialization_modes"],
    )?;
    let scope_bindings = parse_scope_bindings(required_array(scope, "bindings")?)?;
    let source_kinds =
        parse_closed_strings(required_array(scope, "source_kinds")?, SourceKind::parse)?;
    let materialization_modes = parse_closed_strings(
        required_array(scope, "materialization_modes")?,
        MaterializationMode::parse,
    )?;
    let digest = sha256_hex(&canonical_json_bytes(&value));
    Ok(PolicyReceipt {
        decision_schema,
        decision_id,
        decision_digest,
        decided_at,
        expires_at,
        outcome,
        asserted_principal,
        verifier,
        verification_outcome,
        scope_bindings,
        source_kinds,
        materialization_modes,
        digest,
    })
}

fn verify_source_identity(source: &SourceInput<'_>) -> Result<(), ExportError> {
    let value = parse_json(source.canonical_bytes).map_err(|_| ExportError::InvalidSourceDigest)?;
    if canonical_json_bytes(&value) != source.canonical_bytes {
        return Err(ExportError::InvalidSourceDigest);
    }
    let JsonValue::Object(map) = &value else {
        return Err(ExportError::InvalidSourceDigest);
    };
    if required_string(map, "schema").map_err(|_| ExportError::InvalidSourceDigest)?
        != source.schema
        || required(map, "binding").map_err(|_| ExportError::InvalidSourceDigest)?
            != &binding_json(source.binding)
    {
        return Err(ExportError::InvalidSourceDigest);
    }
    if let Some(sequence) = source.sequence {
        match map.get("sequence") {
            Some(JsonValue::Number(value)) if value == &sequence.to_string() => {}
            _ => return Err(ExportError::InvalidSourceDigest),
        }
    }
    let expected_manifest = match source.kind {
        SourceKind::Exchange => Vec::new(),
        SourceKind::InstructionObservation => vec![match map.get("exchange_digest") {
            Some(JsonValue::String(value)) => value.clone(),
            _ => return Err(ExportError::InvalidSourceDigest),
        }],
        SourceKind::MeasurementSummary => match map.get("ordered_exchange_digests") {
            Some(JsonValue::Array(values)) => values
                .iter()
                .map(|value| match value {
                    JsonValue::String(value) => Ok(value.clone()),
                    _ => Err(ExportError::InvalidSourceDigest),
                })
                .collect::<Result<Vec<_>, _>>()?,
            _ => return Err(ExportError::InvalidSourceDigest),
        },
    };
    if expected_manifest != source.source_exchange_manifest {
        return Err(ExportError::InvalidSourceDigest);
    }
    Ok(())
}

fn parse_scope_bindings(values: &[JsonValue]) -> Result<Vec<RequestedBinding>, ExportError> {
    if values.is_empty() {
        return Err(ExportError::MalformedReceipt);
    }
    reject_duplicate_values(values)?;
    values
        .iter()
        .map(|value| {
            let map = exact_object(value, &["binding", "capture_digest"])?;
            let capture_digest = nonempty_string(map, "capture_digest")?;
            validate_digest(&capture_digest).map_err(|_| ExportError::MalformedReceipt)?;
            Ok(RequestedBinding {
                binding: parse_binding(required(map, "binding")?)?,
                capture_digest,
            })
        })
        .collect()
}

fn parse_binding(value: &JsonValue) -> Result<Binding, ExportError> {
    let JsonValue::Object(map) = value else {
        return Err(ExportError::MalformedReceipt);
    };
    match required_string(map, "kind")? {
        "supervised" => {
            require_exact_keys(map, &["kind", "run_id", "attempt", "effect_id"])?;
            let attempt = match required(map, "attempt")? {
                JsonValue::Number(value) => value
                    .parse::<u32>()
                    .map_err(|_| ExportError::MalformedReceipt)?,
                _ => return Err(ExportError::MalformedReceipt),
            };
            Ok(Binding::Supervised {
                run_id: nonempty_string(map, "run_id")?,
                attempt,
                effect_id: nonempty_string(map, "effect_id")?,
            })
        }
        "unsupervised" => {
            require_exact_keys(map, &["kind", "session_id"])?;
            Ok(Binding::Unsupervised {
                session_id: nonempty_string(map, "session_id")?,
            })
        }
        _ => Err(ExportError::MalformedReceipt),
    }
}

fn parse_closed_strings<T: Copy + Ord>(
    values: &[JsonValue],
    parse: fn(&str) -> Option<T>,
) -> Result<Vec<T>, ExportError> {
    if values.is_empty() {
        return Err(ExportError::MalformedReceipt);
    }
    reject_duplicate_values(values)?;
    values
        .iter()
        .map(|value| match value {
            JsonValue::String(value) => parse(value).ok_or(ExportError::MalformedReceipt),
            _ => Err(ExportError::MalformedReceipt),
        })
        .collect()
}

fn reject_duplicate_values(values: &[JsonValue]) -> Result<(), ExportError> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(canonical_json_bytes(value)) {
            return Err(ExportError::MalformedReceipt);
        }
    }
    Ok(())
}

fn exact_object<'a>(
    value: &'a JsonValue,
    keys: &[&str],
) -> Result<&'a BTreeMap<String, JsonValue>, ExportError> {
    let JsonValue::Object(map) = value else {
        return Err(ExportError::MalformedReceipt);
    };
    require_exact_keys(map, keys)?;
    Ok(map)
}

fn require_exact_keys(map: &BTreeMap<String, JsonValue>, keys: &[&str]) -> Result<(), ExportError> {
    if map.len() != keys.len() || !keys.iter().all(|key| map.contains_key(*key)) {
        return Err(ExportError::MalformedReceipt);
    }
    Ok(())
}

fn required<'a>(
    map: &'a BTreeMap<String, JsonValue>,
    key: &str,
) -> Result<&'a JsonValue, ExportError> {
    map.get(key).ok_or(ExportError::MalformedReceipt)
}

fn required_string<'a>(
    map: &'a BTreeMap<String, JsonValue>,
    key: &str,
) -> Result<&'a str, ExportError> {
    match required(map, key)? {
        JsonValue::String(value) => Ok(value),
        _ => Err(ExportError::MalformedReceipt),
    }
}

fn nonempty_string(map: &BTreeMap<String, JsonValue>, key: &str) -> Result<String, ExportError> {
    let value = required_string(map, key)?;
    if value.is_empty() || value.contains(['\0', '\r', '\n']) {
        Err(ExportError::MalformedReceipt)
    } else {
        Ok(value.into())
    }
}

fn required_array<'a>(
    map: &'a BTreeMap<String, JsonValue>,
    key: &str,
) -> Result<&'a [JsonValue], ExportError> {
    match required(map, key)? {
        JsonValue::Array(values) => Ok(values),
        _ => Err(ExportError::MalformedReceipt),
    }
}

fn validate_digest(value: &str) -> Result<(), ExportError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(ExportError::InvalidRequest)
    }
}

fn validate_prefixed_digest(value: &str) -> Result<(), ExportError> {
    value
        .strip_prefix("sha256:")
        .ok_or(ExportError::MalformedReceipt)
        .and_then(|digest| validate_digest(digest).map_err(|_| ExportError::MalformedReceipt))
}

fn minimum_timestamp(left: &str, right: &str) -> Result<String, ExportError> {
    let left_value = Timestamp::parse(left).ok_or(ExportError::InvalidRequest)?;
    let right_value = Timestamp::parse(right).ok_or(ExportError::InvalidRequest)?;
    Ok(if left_value <= right_value {
        left.into()
    } else {
        right.into()
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Timestamp {
    components: (u32, u32, u32, u32, u32, u32),
    fraction: String,
}

impl Timestamp {
    fn parse(value: &str) -> Option<Self> {
        let bytes = value.as_bytes();
        if bytes.len() < 20
            || bytes.get(4) != Some(&b'-')
            || bytes.get(7) != Some(&b'-')
            || bytes.get(10) != Some(&b'T')
            || bytes.get(13) != Some(&b':')
            || bytes.get(16) != Some(&b':')
            || bytes.last() != Some(&b'Z')
        {
            return None;
        }
        let year = digits(bytes, 0, 4)?;
        let month = digits(bytes, 5, 2)?;
        let day = digits(bytes, 8, 2)?;
        let hour = digits(bytes, 11, 2)?;
        let minute = digits(bytes, 14, 2)?;
        let second = digits(bytes, 17, 2)?;
        let days = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if year % 400 == 0 || (year % 4 == 0 && year % 100 != 0) => 29,
            2 => 28,
            _ => return None,
        };
        if day == 0 || day > days || hour > 23 || minute > 59 || second > 60 {
            return None;
        }
        let fraction = match &value[19..value.len() - 1] {
            "" => String::new(),
            suffix
                if suffix.starts_with('.')
                    && suffix.len() > 1
                    && suffix[1..].bytes().all(|byte| byte.is_ascii_digit()) =>
            {
                suffix[1..].trim_end_matches('0').to_owned()
            }
            _ => return None,
        };
        Some(Self {
            components: (year, month, day, hour, minute, second),
            fraction,
        })
    }
}

impl Ord for Timestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        self.components.cmp(&other.components).then_with(|| {
            let length = self.fraction.len().max(other.fraction.len());
            self.fraction
                .bytes()
                .chain(std::iter::repeat(b'0'))
                .take(length)
                .cmp(
                    other
                        .fraction
                        .bytes()
                        .chain(std::iter::repeat(b'0'))
                        .take(length),
                )
        })
    }
}

impl PartialOrd for Timestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn digits(bytes: &[u8], offset: usize, length: usize) -> Option<u32> {
    let slice = bytes.get(offset..offset + length)?;
    slice.iter().all(u8::is_ascii_digit).then(|| {
        slice
            .iter()
            .fold(0u32, |value, digit| value * 10 + u32::from(*digit - b'0'))
    })
}

fn source_order(left: &SourceEntry, right: &SourceEntry) -> Ordering {
    binding_bytes(&left.binding)
        .cmp(&binding_bytes(&right.binding))
        .then(left.kind.cmp(&right.kind))
        .then(left.sequence.cmp(&right.sequence))
        .then(left.digest.as_bytes().cmp(right.digest.as_bytes()))
}

fn artifact_order(left: &ArtifactEntry, right: &ArtifactEntry) -> Ordering {
    binding_bytes(&left.binding)
        .cmp(&binding_bytes(&right.binding))
        .then(
            left.artifact_id
                .as_bytes()
                .cmp(right.artifact_id.as_bytes()),
        )
        .then(left.digest.as_bytes().cmp(right.digest.as_bytes()))
}

fn gap_order(left: &ExportGap, right: &ExportGap) -> Ordering {
    binding_bytes(&left.binding)
        .cmp(&binding_bytes(&right.binding))
        .then(left.source_kind.cmp(&right.source_kind))
        .then(left.artifact_id.cmp(&right.artifact_id))
        .then(left.reason.as_bytes().cmp(right.reason.as_bytes()))
}

fn binding_key(binding: &RequestedBinding) -> Vec<u8> {
    let mut key = binding_bytes(&binding.binding);
    key.extend_from_slice(binding.capture_digest.as_bytes());
    key
}

fn binding_bytes(binding: &Binding) -> Vec<u8> {
    canonical_json_bytes(&binding_json(binding))
}

fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        output.push(TABLE[(first >> 2) as usize] as char);
        output.push(TABLE[(((first & 3) << 4) | (second >> 4)) as usize] as char);
        output.push(if chunk.len() > 1 {
            TABLE[(((second & 15) << 2) | (third >> 6)) as usize] as char
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            TABLE[(third & 63) as usize] as char
        } else {
            '='
        });
    }
    output
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
            ("attempt", number(attempt)),
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

fn requested_binding_json(binding: &RequestedBinding) -> JsonValue {
    object([
        ("binding", binding_json(&binding.binding)),
        ("capture_digest", string(&binding.capture_digest)),
    ])
}

fn source_json(source: &SourceEntry) -> JsonValue {
    object([
        ("binding", binding_json(&source.binding)),
        ("byte_length", number(source.byte_length)),
        ("capture_digest", string(&source.capture_digest)),
        ("completeness", string(source.completeness.as_str())),
        ("digest", string(&source.digest)),
        (
            "expires_at",
            source.expires_at.as_deref().map_or(JsonValue::Null, string),
        ),
        (
            "findings",
            JsonValue::Array(source.findings.iter().map(string).collect()),
        ),
        ("kind", string(source.kind.as_str())),
        ("retention", string(source.retention.as_str())),
        ("schema", string(&source.schema)),
        ("sequence", source.sequence.map_or(JsonValue::Null, number)),
        (
            "source_exchange_manifest",
            JsonValue::Array(source.source_exchange_manifest.iter().map(string).collect()),
        ),
    ])
}

fn artifact_json(artifact: &ArtifactEntry) -> JsonValue {
    object([
        ("artifact_id", string(&artifact.artifact_id)),
        ("binding", binding_json(&artifact.binding)),
        ("byte_length", number(artifact.byte_length)),
        ("capture_digest", string(&artifact.capture_digest)),
        (
            "custodian_identity",
            artifact
                .custodian_identity
                .as_deref()
                .map_or(JsonValue::Null, string),
        ),
        (
            "custody_reference",
            artifact
                .custody_reference
                .as_deref()
                .map_or(JsonValue::Null, string),
        ),
        ("digest", string(&artifact.digest)),
        (
            "embedded_base64",
            artifact
                .embedded_base64
                .as_deref()
                .map_or(JsonValue::Null, string),
        ),
        (
            "expiry",
            artifact.expiry.as_deref().map_or(JsonValue::Null, string),
        ),
        (
            "findings",
            JsonValue::Array(artifact.findings.iter().map(string).collect()),
        ),
        ("materialization_mode", string(artifact.mode.as_str())),
        ("media_type", string(&artifact.media_type)),
        (
            "redaction_complete",
            JsonValue::Bool(artifact.redaction_complete),
        ),
    ])
}

fn gap_json(gap: &ExportGap) -> JsonValue {
    object([
        (
            "artifact_id",
            gap.artifact_id.as_deref().map_or(JsonValue::Null, string),
        ),
        ("binding", binding_json(&gap.binding)),
        ("capture_digest", string(&gap.capture_digest)),
        ("reason", string(&gap.reason)),
        (
            "source_kind",
            gap.source_kind
                .map(SourceKind::as_str)
                .map_or(JsonValue::Null, string),
        ),
    ])
}

fn receipt_json(receipt: &PolicyReceipt) -> JsonValue {
    let envelope = object([
        ("asserted_principal", string(&receipt.asserted_principal)),
        ("decided_at", string(&receipt.decided_at)),
        ("decision_digest", string(&receipt.decision_digest)),
        ("decision_id", string(&receipt.decision_id)),
        ("decision_schema", string(&receipt.decision_schema)),
        ("expires_at", string(&receipt.expires_at)),
        ("outcome", string(&receipt.outcome)),
        ("schema", string(POLICY_SCHEMA)),
        (
            "scope",
            object([
                (
                    "bindings",
                    JsonValue::Array(
                        receipt
                            .scope_bindings
                            .iter()
                            .map(requested_binding_json)
                            .collect(),
                    ),
                ),
                (
                    "materialization_modes",
                    JsonValue::Array(
                        receipt
                            .materialization_modes
                            .iter()
                            .map(|mode| string(mode.as_str()))
                            .collect(),
                    ),
                ),
                (
                    "source_kinds",
                    JsonValue::Array(
                        receipt
                            .source_kinds
                            .iter()
                            .map(|kind| string(kind.as_str()))
                            .collect(),
                    ),
                ),
            ]),
        ),
        (
            "verification_outcome",
            string(&receipt.verification_outcome),
        ),
        ("verifier", string(&receipt.verifier)),
    ]);
    object([
        ("digest", string(&receipt.digest)),
        ("digest_construction", string(POLICY_DIGEST_CONSTRUCTION)),
        ("envelope", envelope),
    ])
}

fn limits_json(limits: ExportLimits) -> JsonValue {
    object([
        ("max_artifact_bytes", number(limits.max_artifact_bytes)),
        ("max_artifacts", number(limits.max_artifacts)),
        ("max_bindings", number(limits.max_bindings)),
        ("max_entries", number(limits.max_entries)),
        ("max_total_bytes", number(limits.max_total_bytes)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding() -> Binding {
        Binding::Supervised {
            run_id: "run-1".into(),
            attempt: 1,
            effect_id: "effect-1".into(),
        }
    }

    fn digest(bytes: &[u8]) -> String {
        sha256_hex(bytes)
    }

    fn source_bytes(binding: &Binding, schema: &str) -> Vec<u8> {
        canonical_json_bytes(&object([
            ("binding", binding_json(binding)),
            ("schema", string(schema)),
            ("sequence", number(1)),
        ]))
    }

    fn receipt(binding: &Binding, capture: &str, outcome: &str, verification: &str) -> Vec<u8> {
        canonical_json_bytes(&object([
            ("asserted_principal", string("owner:test")),
            ("decided_at", string("2026-10-01T11:00:00Z")),
            (
                "decision_digest",
                string(format!("sha256:{}", "a".repeat(64))),
            ),
            ("decision_id", string("decision-1")),
            ("decision_schema", string("policy.example/1")),
            ("expires_at", string("2026-10-02T12:00:00Z")),
            ("outcome", string(outcome)),
            ("schema", string(POLICY_SCHEMA)),
            (
                "scope",
                object([
                    (
                        "bindings",
                        JsonValue::Array(vec![requested_binding_json(&RequestedBinding {
                            binding: binding.clone(),
                            capture_digest: capture.into(),
                        })]),
                    ),
                    (
                        "materialization_modes",
                        JsonValue::Array(vec![
                            string("digest-only"),
                            string("embedded"),
                            string("external-reference"),
                        ]),
                    ),
                    (
                        "source_kinds",
                        JsonValue::Array(vec![
                            string("exchange"),
                            string("instruction-observation"),
                            string("measurement-summary"),
                        ]),
                    ),
                ]),
            ),
            ("verification_outcome", string(verification)),
            ("verifier", string("policy-verifier:test")),
        ]))
    }

    fn request<'a>(
        binding: &'a Binding,
        capture: &'a str,
        policy: &'a [u8],
        source_bytes: &'a [u8],
        artifact_bytes: &'a [u8],
    ) -> ExportRequest<'a> {
        ExportRequest {
            export_id: "export-1",
            producer_identity: "wire-witness:test",
            created_at: "2026-10-01T12:00:00Z",
            maximum_expires_at: "2026-10-03T12:00:00Z",
            policy_receipt: policy,
            bindings: vec![RequestedBinding {
                binding: binding.clone(),
                capture_digest: capture.into(),
            }],
            sources: vec![SourceInput {
                binding,
                capture_digest: capture,
                kind: SourceKind::Exchange,
                schema: crate::exchange::SCHEMA,
                sequence: Some(1),
                claimed_digest: digest(source_bytes),
                claimed_length: source_bytes.len() as u64,
                canonical_bytes: source_bytes,
                retention: RetentionMode::Content,
                expires_at: Some("2026-10-02T00:00:00Z"),
                completeness: SourceCompleteness::Complete,
                findings: Vec::new(),
                source_exchange_manifest: Vec::new(),
            }],
            artifacts: vec![ArtifactInput {
                artifact_id: "response-body",
                binding,
                capture_digest: capture,
                media_type: "application/json",
                claimed_digest: digest(artifact_bytes),
                claimed_length: artifact_bytes.len() as u64,
                redacted_bytes: Some(artifact_bytes),
                retention: RetentionMode::Content,
                expires_at: Some("2026-10-01T18:00:00Z"),
                erased: false,
                redaction_complete: true,
                mode: MaterializationMode::Embedded,
                custody_reference: None,
                custodian_identity: None,
                findings: Vec::new(),
            }],
            allow_incomplete: false,
            limits: ExportLimits {
                max_bindings: 2,
                max_entries: 10,
                max_artifacts: 10,
                max_artifact_bytes: 10_000,
                max_total_bytes: 100_000,
            },
        }
    }

    #[test]
    fn constructs_deterministic_bounded_manifest() {
        let binding = binding();
        let capture = "b".repeat(64);
        let policy = receipt(&binding, &capture, "allow", "verified");
        let source = source_bytes(&binding, crate::exchange::SCHEMA);
        let result =
            construct_export(request(&binding, &capture, &policy, &source, b"redacted")).unwrap();
        assert!(result.manifest.complete);
        assert_eq!(result.manifest.expires_at, "2026-10-01T18:00:00Z");
        assert_eq!(
            result.manifest.artifacts[0].embedded_base64.as_deref(),
            Some("cmVkYWN0ZWQ=")
        );
        assert_eq!(result.manifest_digest, sha256_hex(&result.manifest_bytes));
        assert!(
            !String::from_utf8(result.manifest_bytes)
                .unwrap()
                .contains("permission")
        );
    }

    #[test]
    fn receipt_is_closed_and_policy_refusals_are_distinct() {
        let binding = binding();
        let capture = "b".repeat(64);
        for (outcome, verification, expected) in [
            ("deny", "verified", ExportError::PolicyDenied),
            ("allow", "unverified", ExportError::PolicyUnverified),
            ("allow", "failed", ExportError::PolicyVerificationFailed),
        ] {
            let policy = receipt(&binding, &capture, outcome, verification);
            let source = source_bytes(&binding, crate::exchange::SCHEMA);
            assert_eq!(
                construct_export(request(&binding, &capture, &policy, &source, b"artifact"))
                    .unwrap_err(),
                expected
            );
        }
        let mut parsed = parse_json(&receipt(&binding, &capture, "allow", "verified")).unwrap();
        let JsonValue::Object(map) = &mut parsed else {
            unreachable!()
        };
        map.insert("unknown".into(), JsonValue::Null);
        let malformed = canonical_json_bytes(&parsed);
        let source = source_bytes(&binding, crate::exchange::SCHEMA);
        assert_eq!(
            construct_export(request(
                &binding,
                &capture,
                &malformed,
                &source,
                b"artifact"
            ))
            .unwrap_err(),
            ExportError::MalformedReceipt
        );
    }

    #[test]
    fn unsupported_versions_and_expired_content_are_explicit_gaps() {
        let binding = binding();
        let capture = "b".repeat(64);
        let policy = receipt(&binding, &capture, "allow", "verified");
        let source = source_bytes(&binding, "wire-witness.exchange/2");
        let mut request = request(&binding, &capture, &policy, &source, b"artifact");
        request.allow_incomplete = true;
        request.sources[0].schema = "wire-witness.exchange/2";
        request.artifacts[0].expires_at = Some("2026-10-01T11:59:59Z");
        let result = construct_export(request).unwrap();
        assert!(!result.manifest.complete);
        assert_eq!(result.manifest.gaps.len(), 2);
        assert!(result.manifest.sources.is_empty());
        assert!(result.manifest.artifacts.is_empty());
    }

    #[test]
    fn corrupt_sources_and_artifacts_refuse_without_substitution() {
        let binding = binding();
        let capture = "b".repeat(64);
        let policy = receipt(&binding, &capture, "allow", "verified");
        let source_bytes = source_bytes(&binding, crate::exchange::SCHEMA);
        let mut source = request(&binding, &capture, &policy, &source_bytes, b"artifact");
        source.sources[0].claimed_length += 1;
        assert_eq!(
            construct_export(source).unwrap_err(),
            ExportError::InvalidSourceDigest
        );
        let mut artifact = request(&binding, &capture, &policy, &source_bytes, b"artifact");
        artifact.artifacts[0].claimed_digest = "c".repeat(64);
        assert_eq!(
            construct_export(artifact).unwrap_err(),
            ExportError::InvalidArtifactDigest
        );
    }

    #[test]
    fn conflicting_instruction_observations_refuse_per_binding_and_sequence() {
        let binding = binding();
        let capture = "b".repeat(64);
        let policy = receipt(&binding, &capture, "allow", "verified");
        let exchange_digest_one = "c".repeat(64);
        let exchange_digest_two = "d".repeat(64);
        let observation_one = canonical_json_bytes(&object([
            ("binding", binding_json(&binding)),
            ("exchange_digest", string(&exchange_digest_one)),
            ("schema", string(crate::instruction_observation::SCHEMA)),
            ("sequence", number(1)),
        ]));
        let observation_two = canonical_json_bytes(&object([
            ("binding", binding_json(&binding)),
            ("exchange_digest", string(&exchange_digest_two)),
            ("schema", string(crate::instruction_observation::SCHEMA)),
            ("sequence", number(1)),
        ]));
        let source = source_bytes(&binding, crate::exchange::SCHEMA);
        let mut request = request(&binding, &capture, &policy, &source, b"artifact");
        request.sources = vec![
            SourceInput {
                binding: &binding,
                capture_digest: &capture,
                kind: SourceKind::InstructionObservation,
                schema: crate::instruction_observation::SCHEMA,
                sequence: Some(1),
                claimed_digest: digest(&observation_one),
                claimed_length: observation_one.len() as u64,
                canonical_bytes: &observation_one,
                retention: RetentionMode::MetadataOnly,
                expires_at: None,
                completeness: SourceCompleteness::Complete,
                findings: Vec::new(),
                source_exchange_manifest: vec![exchange_digest_one],
            },
            SourceInput {
                binding: &binding,
                capture_digest: &capture,
                kind: SourceKind::InstructionObservation,
                schema: crate::instruction_observation::SCHEMA,
                sequence: Some(1),
                claimed_digest: digest(&observation_two),
                claimed_length: observation_two.len() as u64,
                canonical_bytes: &observation_two,
                retention: RetentionMode::MetadataOnly,
                expires_at: None,
                completeness: SourceCompleteness::Complete,
                findings: Vec::new(),
                source_exchange_manifest: vec![exchange_digest_two],
            },
        ];
        assert_eq!(
            construct_export(request).unwrap_err(),
            ExportError::ConflictingSequence
        );
    }

    #[test]
    fn timestamp_validation_and_order_are_exact() {
        assert!(Timestamp::parse("2024-02-29T23:59:60Z").is_some());
        assert!(Timestamp::parse("2025-02-29T00:00:00Z").is_none());
        assert!(Timestamp::parse("2026-10-01T12:00:00+00:00").is_none());
        assert!(
            Timestamp::parse("2026-10-01T12:00:00.01Z").unwrap()
                > Timestamp::parse("2026-10-01T12:00:00.001Z").unwrap()
        );
    }
}

// endregion
