//! The live attempt runtime: the standalone host's loopback proxy, ephemeral
//! trust files, redacted exchange records, and capture manifest.
//!
//! Governed by spec 013, live loopback capture runtime.

// region: live-attempt-runtime

use std::collections::BTreeSet;
use std::fs::{self, DirBuilder};
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime};

#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt;

use wire_witness_core::custody::{HeaderField, RedactionCategory, RetentionMode, redact_headers};
use wire_witness_core::exchange::{
    Binding, Completeness, Cost, DIGEST_CONSTRUCTION, ExchangeRecord, Finding, IncompleteReason,
    JsonValue, Normalization, Presence, RetentionState, TransportMetadata, UsageEntry,
    canonical_json_bytes, normalize_provider_json, normalize_sse,
};
use wire_witness_core::instruction_observation::ComparisonPlan;
use wire_witness_proxy::live::{
    CapturedBody, EphemeralCa, ExchangeEnd, LiveError, LiveFinding, LiveLimits, LiveProxy,
    LiveSummary, ObservedExchange, ProviderTrust, system_roots_pem,
};
use wire_witness_proxy::{Authority, ProxyConfig, RouteDecision};

use crate::standalone_host::{
    AttemptRuntime, CaptureState, ClosedCapture, ExchangeSequence, HostError, HostFinding,
    PreparedAttempt, RunRequest, TrustMechanism, UnsupervisedBinding, write_private_file_new,
};

pub const MANIFEST_SCHEMA: &str = "wire-witness.capture-manifest/1";
pub const MANIFEST_FILE: &str = "manifest.json";
pub const EXCHANGES_DIRECTORY: &str = "exchanges";
const CA_FILE: &str = "attempt-ca.pem";
const BUNDLE_FILE: &str = "attempt-trust-bundle.pem";
const WRITER_POLL: Duration = Duration::from_millis(25);

/// The host's real `AttemptRuntime`. One value serves exactly one attempt.
pub struct LiveRuntime {
    trust: ProviderTrust,
    limits: LiveLimits,
    grace: Duration,
    running: Option<Running>,
}

struct Running {
    proxy: LiveProxy,
    config: ProxyConfig,
    writer: JoinHandle<WriterOutcome>,
    finish: Arc<AtomicBool>,
    attempt: PathBuf,
    binding: Binding,
    requested: RetentionMode,
    effective: RetentionMode,
    findings: Vec<HostFinding>,
}

#[derive(Default)]
struct WriterOutcome {
    entries: Vec<InstalledExchange>,
    missing: Vec<u64>,
    observed: u64,
    incomplete: u64,
    findings: Vec<HostFinding>,
}

/// One installed exchange record, with the normalized facts a supervisor's
/// evidence manifest repeats. It carries no header value, body, or query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledExchange {
    pub sequence: u64,
    /// Path relative to the attempt directory.
    pub path: String,
    /// Lowercase hex SHA-256 of the installed record bytes.
    pub digest: String,
    pub byte_length: u64,
    pub completeness: &'static str,
    pub authority: String,
    pub protocol: &'static str,
    pub requested_model: Presence<String>,
    pub served_model: Presence<String>,
    pub usage: Vec<UsageEntry>,
    /// Spec 003 class names of the values removed from this exchange.
    pub redaction_classes: BTreeSet<&'static str>,
}

/// What the runtime reports to a supervising host while an attempt runs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeEvent {
    Exchange(InstalledExchange),
    Finding {
        code: String,
        authority: Option<String>,
    },
}

impl LiveRuntime {
    /// The command-line runtime: provider TLS uses ordinary system trust.
    #[must_use]
    pub fn system() -> Self {
        Self::with_provider_trust(ProviderTrust::System, LiveLimits::default())
    }

    /// A runtime with an exact provider root set, for supervisors and tests
    /// that call the library. The command line never selects this.
    #[must_use]
    pub fn with_provider_trust(trust: ProviderTrust, limits: LiveLimits) -> Self {
        Self {
            trust,
            limits,
            grace: Duration::from_secs(5),
            running: None,
        }
    }

    fn start(
        &self,
        request: &RunRequest,
        binding: &UnsupervisedBinding,
        config: ProxyConfig,
        attempt: &Path,
        cleanup: &mut Vec<PathBuf>,
    ) -> Result<(PreparedAttempt, Running), HostError> {
        let ca = EphemeralCa::generate(SystemTime::now()).map_err(live_error)?;
        let certificate = ca.certificate_pem();
        let ca_path = attempt.join(CA_FILE);
        write_private_file_new(&ca_path, certificate.as_bytes())?;
        cleanup.push(ca_path.clone());
        let trust_path = if request.trust == TrustMechanism::ExistingRootsBundle {
            let mut bundle = system_roots_pem()
                .map_err(|_| HostError::Precondition("system-trust-unavailable"))?;
            bundle.push_str(&certificate);
            let bundle_path = attempt.join(BUNDLE_FILE);
            write_private_file_new(&bundle_path, bundle.as_bytes())?;
            cleanup.push(bundle_path.clone());
            bundle_path
        } else {
            ca_path
        };

        let requested = request.requested_retention;
        let mut findings = Vec::new();
        let effective = if requested == RetentionMode::Content {
            findings.push(HostFinding {
                code: "content-retention-unavailable".into(),
                detail:
                    "the live runtime retains metadata only; effective retention is metadata-only"
                        .into(),
            });
            RetentionMode::MetadataOnly
        } else {
            requested
        };

        let (sender, receiver) = mpsc::sync_channel(self.limits.sink_capacity);
        let finish = Arc::new(AtomicBool::new(false));
        let writer_finish = Arc::clone(&finish);
        let writer_directory = attempt.join(EXCHANGES_DIRECTORY);
        let writer_binding = Binding::Unsupervised {
            session_id: binding.session_id.clone(),
        };
        let writer = thread::Builder::new()
            .name("wire-witness-writer".into())
            .spawn(move || {
                write_exchanges(
                    &receiver,
                    &writer_directory,
                    &writer_binding,
                    requested,
                    effective,
                    &writer_finish,
                    None,
                )
            })
            .map_err(|error| HostError::Custody(error.kind().to_string()))?;
        let proxy = LiveProxy::start(config.clone(), ca, self.trust.clone(), self.limits, sender)
            .map_err(live_error)?;
        let prepared = PreparedAttempt {
            proxy: proxy.authority(),
            certificate_or_bundle: trust_path,
            cleanup_paths: cleanup.clone(),
        };
        Ok((
            prepared,
            Running {
                proxy,
                config,
                writer,
                finish,
                attempt: attempt.to_path_buf(),
                binding: Binding::Unsupervised {
                    session_id: binding.session_id.clone(),
                },
                requested,
                effective,
                findings,
            },
        ))
    }
}

impl AttemptRuntime for LiveRuntime {
    fn prepare(
        &mut self,
        request: &RunRequest,
        binding: &UnsupervisedBinding,
        instruction_plans: Vec<ComparisonPlan>,
    ) -> Result<PreparedAttempt, HostError> {
        if self.running.is_some() {
            return Err(HostError::Custody("runtime-already-prepared".into()));
        }
        if !instruction_plans.is_empty() {
            return Err(HostError::Precondition(
                "instruction-observation-unsupported-by-live-runtime",
            ));
        }
        let config = request.validate()?;
        let attempt = request.output_directory.join(&binding.session_id);
        create_private_directory(&attempt)?;
        create_private_directory(&attempt.join(EXCHANGES_DIRECTORY))?;
        let mut cleanup = Vec::new();
        match self.start(request, binding, config, &attempt, &mut cleanup) {
            Ok((prepared, running)) => {
                self.running = Some(running);
                Ok(prepared)
            }
            Err(error) => {
                for path in &cleanup {
                    let _ = fs::remove_file(path);
                }
                let _ = fs::remove_dir(attempt.join(EXCHANGES_DIRECTORY));
                let _ = fs::remove_dir(&attempt);
                Err(error)
            }
        }
    }

    fn close(self) -> Result<ClosedCapture, HostError> {
        let Some(running) = self.running else {
            return Err(HostError::Custody("runtime-not-prepared".into()));
        };
        let summary = running.proxy.stop(self.grace);
        running.finish.store(true, Ordering::SeqCst);
        let outcome = running
            .writer
            .join()
            .map_err(|_| HostError::Custody("exchange-writer-failed".into()))?;
        let closed = close_capture(
            &running.config,
            &running.attempt,
            &running.binding,
            running.requested,
            running.effective,
            running.findings,
            &summary,
            outcome,
            &[],
        );
        Ok(ClosedCapture {
            capture: closed.capture,
            exchange_count: closed.observed,
            manifest_path: closed
                .manifest_path
                .map(|path| path.to_string_lossy().into_owned()),
            findings: closed.findings,
        })
    }
}

/// The result of closing an attempt's custody.
struct Closed {
    capture: CaptureState,
    reason: Option<&'static str>,
    gaps: BTreeSet<String>,
    manifest_path: Option<PathBuf>,
    observed: u64,
    dropped: u64,
    missing: u64,
    exchanges: Vec<InstalledExchange>,
    findings: Vec<HostFinding>,
    /// Findings that arose at or after close, which no event reported.
    closing_findings: Vec<RuntimeEvent>,
}

#[allow(clippy::too_many_arguments)]
fn close_capture(
    config: &ProxyConfig,
    attempt: &Path,
    binding: &Binding,
    requested: RetentionMode,
    effective: RetentionMode,
    mut findings: Vec<HostFinding>,
    summary: &LiveSummary,
    mut outcome: WriterOutcome,
    supervisor_gaps: &[&str],
) -> Closed {
    let capture_affected = summary
        .findings
        .iter()
        .any(|finding| affects_capture(config, finding));
    let mut closing_findings = Vec::new();
    findings.extend(summary.findings.iter().map(|finding| HostFinding {
        code: finding.code.into(),
        detail: finding.authority.as_ref().map_or_else(
            || format!("{}: {}", finding.direction, finding.detail),
            |authority| format!("{authority} {}: {}", finding.direction, finding.detail),
        ),
    }));
    if summary.open_connections > 0 {
        closing_findings.push(closing_finding("connections-open-at-close"));
        findings.push(HostFinding {
            code: "connections-open-at-close".into(),
            detail: format!(
                "{} connection(s) were still open when the attempt closed",
                summary.open_connections
            ),
        });
    }
    if summary.counters.dropped_exchanges > 0 {
        closing_findings.push(closing_finding("sink-backpressure"));
        findings.push(HostFinding {
            code: "sink-backpressure".into(),
            detail: format!(
                "{} observed exchange(s) were not recorded",
                summary.counters.dropped_exchanges
            ),
        });
    }
    findings.extend(std::mem::take(&mut outcome.findings));

    let mut gaps: BTreeSet<String> = supervisor_gaps
        .iter()
        .map(|gap| (*gap).to_owned())
        .collect();
    for (present, gap) in [
        (outcome.incomplete > 0, "exchange-incomplete"),
        (!outcome.missing.is_empty(), "record-not-installed"),
        (summary.counters.dropped_exchanges > 0, "sink-backpressure"),
        (summary.open_connections > 0, "connections-open-at-close"),
        (capture_affected, "capture-finding"),
    ] {
        if present {
            gaps.insert(gap.to_owned());
        }
    }
    let (capture, reason) = if requested == RetentionMode::Disabled {
        (CaptureState::Absent, Some("disabled"))
    } else if outcome.observed == 0 && summary.counters.dropped_exchanges == 0 {
        (CaptureState::Absent, Some("no-exchange-observed"))
    } else if gaps.is_empty() {
        (CaptureState::Complete, None)
    } else {
        (CaptureState::Incomplete, None)
    };

    let manifest = manifest_json(
        binding, requested, effective, &capture, reason, summary, &outcome, &findings,
    );
    let bytes = canonical_json_bytes(&manifest);
    let (capture, manifest_path) = match install_private(attempt, MANIFEST_FILE, &bytes) {
        Ok(path) => (capture, Some(path)),
        Err(error) => {
            gaps.insert("manifest-not-installed".into());
            closing_findings.push(closing_finding("manifest-not-installed"));
            findings.push(HostFinding {
                code: "manifest-not-installed".into(),
                detail: error,
            });
            (
                if capture == CaptureState::Complete {
                    CaptureState::Incomplete
                } else {
                    capture
                },
                None,
            )
        }
    };
    Closed {
        capture,
        reason,
        gaps,
        manifest_path,
        observed: outcome.observed,
        dropped: summary.counters.dropped_exchanges,
        missing: outcome.missing.len() as u64,
        exchanges: outcome.entries,
        findings,
        closing_findings,
    }
}

fn closing_finding(code: &str) -> RuntimeEvent {
    RuntimeEvent::Finding {
        code: code.to_owned(),
        authority: None,
    }
}

/// A connection-level finding affects capture unless it names an authority
/// outside the capture allowlist. An unattributed finding may have been a
/// capture attempt, so it counts.
fn affects_capture(config: &ProxyConfig, finding: &LiveFinding) -> bool {
    finding
        .authority
        .as_deref()
        .and_then(|authority| Authority::parse(authority).ok())
        .is_none_or(|authority| matches!(config.route(&authority), RouteDecision::Intercept { .. }))
}

#[allow(clippy::too_many_arguments)]
fn manifest_json(
    binding: &Binding,
    requested: RetentionMode,
    effective: RetentionMode,
    capture: &CaptureState,
    reason: Option<&str>,
    summary: &LiveSummary,
    outcome: &WriterOutcome,
    findings: &[HostFinding],
) -> JsonValue {
    let status = match capture {
        CaptureState::Complete => "complete",
        CaptureState::Incomplete => "incomplete",
        CaptureState::Absent => "absent",
    };
    let counters = summary.counters;
    object([
        ("binding", manifest_binding(binding)),
        (
            "capture",
            object([
                ("reason", reason.map_or(JsonValue::Null, string)),
                ("status", string(status)),
            ]),
        ),
        (
            "custody",
            object([
                ("ca_private_key", string("memory-only")),
                ("memory_lock", string("not-attempted")),
            ]),
        ),
        ("dropped_exchange_count", number(counters.dropped_exchanges)),
        (
            "exchanges",
            JsonValue::Array(
                outcome
                    .entries
                    .iter()
                    .map(|entry| {
                        object([
                            ("byte_length", number(entry.byte_length)),
                            ("completeness", string(entry.completeness)),
                            ("digest", string(&entry.digest)),
                            ("digest_construction", string(DIGEST_CONSTRUCTION)),
                            ("path", string(&entry.path)),
                            ("sequence", number(entry.sequence)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "findings",
            JsonValue::Array(
                findings
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
        (
            "missing_sequences",
            JsonValue::Array(outcome.missing.iter().map(|value| number(*value)).collect()),
        ),
        ("observed_exchange_count", number(outcome.observed)),
        (
            "open_connections_at_close",
            number(summary.open_connections as u64),
        ),
        (
            "retention",
            object([
                ("effective", string(effective.as_str())),
                ("requested", string(requested.as_str())),
            ]),
        ),
        (
            "routing",
            object([
                (
                    "authentication_tunnels",
                    number(counters.authentication_tunnels),
                ),
                (
                    "intercepted_connections",
                    number(counters.intercepted_connections),
                ),
                ("refused_connections", number(counters.refused_connections)),
                (
                    "unintercepted_plain_requests",
                    number(counters.unintercepted_plain_requests),
                ),
                (
                    "unintercepted_tunnels",
                    number(counters.unintercepted_tunnels),
                ),
            ]),
        ),
        (
            "runtime",
            object([(
                "application_protocols",
                JsonValue::Array(vec![string("http/1.1")]),
            )]),
        ),
        ("schema", string(MANIFEST_SCHEMA)),
    ])
}

fn manifest_binding(binding: &Binding) -> JsonValue {
    match binding {
        Binding::Unsupervised { session_id } => object([
            ("kind", string("unsupervised")),
            ("session_id", string(session_id)),
        ]),
        Binding::Supervised {
            run_id,
            attempt,
            effect_id,
        } => object([
            ("attempt", number(u64::from(*attempt))),
            ("effect_id", string(effect_id)),
            ("kind", string("supervised")),
            ("run_id", string(run_id)),
        ]),
    }
}

fn write_exchanges(
    receiver: &Receiver<ObservedExchange>,
    directory: &Path,
    binding: &Binding,
    requested: RetentionMode,
    effective: RetentionMode,
    finish: &AtomicBool,
    events: Option<&Sender<RuntimeEvent>>,
) -> WriterOutcome {
    let report = |event: RuntimeEvent| {
        if let Some(events) = events {
            let _ = events.send(event);
        }
    };
    let mut outcome = WriterOutcome::default();
    let mut sequence = ExchangeSequence::default();
    loop {
        let observed = match receiver.recv_timeout(WRITER_POLL) {
            Ok(observed) => observed,
            Err(RecvTimeoutError::Timeout) if finish.load(Ordering::SeqCst) => break,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        };
        outcome.observed += 1;
        if requested == RetentionMode::Disabled {
            continue;
        }
        let Ok(number) = sequence.next_sequence() else {
            report(closing_finding("exchange-sequence-overflow"));
            outcome.findings.push(HostFinding {
                code: "exchange-sequence-overflow".into(),
                detail: "no further sequence numbers".into(),
            });
            break;
        };
        let record = exchange_record_for(binding, number, &observed, requested, effective);
        if record.completeness != Completeness::Complete {
            outcome.incomplete += 1;
        }
        let bytes = record.canonical_bytes();
        let name = format!("exchange-{number:020}.json");
        match install_private(directory, &name, &bytes) {
            Ok(_) => {
                let installed = InstalledExchange {
                    sequence: number,
                    path: format!("{EXCHANGES_DIRECTORY}/{name}"),
                    digest: record.digest(),
                    byte_length: bytes.len() as u64,
                    completeness: completeness_name(&record.completeness),
                    authority: record.transport.authority.clone(),
                    protocol: observed.protocol,
                    requested_model: record.requested_identity.clone(),
                    served_model: match &record.served_identity {
                        Presence::Known(served) => Presence::Known(served.identity.clone()),
                        Presence::Absent => Presence::Absent,
                        Presence::Unknown { reason } => Presence::Unknown {
                            reason: reason.clone(),
                        },
                    },
                    usage: record.usage.clone(),
                    redaction_classes: redaction_classes(&observed),
                };
                report(RuntimeEvent::Exchange(installed.clone()));
                outcome.entries.push(installed);
            }
            Err(error) => {
                report(closing_finding("sink-failed"));
                outcome.missing.push(number);
                outcome.findings.push(HostFinding {
                    code: "sink-failed".into(),
                    detail: format!("exchange {number}: {error}"),
                });
            }
        }
    }
    outcome
}

const fn completeness_name(completeness: &Completeness) -> &'static str {
    match completeness {
        Completeness::Complete => "complete",
        Completeness::Incomplete(_) => "incomplete",
        Completeness::Absent(_) => "absent",
    }
}

/// The spec 003 classes of the values a record omits for this exchange.
fn redaction_classes(observed: &ObservedExchange) -> BTreeSet<&'static str> {
    let (_, request) = redact_headers(&header_fields(&observed.request_headers));
    let (_, response) = redact_headers(&header_fields(&observed.response_headers));
    let mut classes: BTreeSet<&'static str> = request
        .iter()
        .chain(&response)
        .map(|record| match record.category {
            RedactionCategory::SensitiveHeader => "sensitive-header",
            RedactionCategory::SensitiveField => "sensitive-field",
            RedactionCategory::DetectedCredential => "detected-credential",
        })
        .collect();
    if observed.target.contains('?') {
        classes.insert("query-string");
    }
    classes
}

/// Builds the `wire-witness.exchange/1` record for one observed exchange.
/// Normalization is spec 002's; this function only places its results and
/// records what the transport and capture tap could not establish.
#[must_use]
pub fn exchange_record(
    binding: &UnsupervisedBinding,
    sequence: u64,
    observed: &ObservedExchange,
    requested: RetentionMode,
    effective: RetentionMode,
) -> ExchangeRecord {
    exchange_record_for(
        &Binding::Unsupervised {
            session_id: binding.session_id.clone(),
        },
        sequence,
        observed,
        requested,
        effective,
    )
}

/// [`exchange_record`] for either binding kind.
#[must_use]
pub fn exchange_record_for(
    binding: &Binding,
    sequence: u64,
    observed: &ObservedExchange,
    requested: RetentionMode,
    effective: RetentionMode,
) -> ExchangeRecord {
    let family = observed.provider.clone();
    let mut findings: Vec<Finding> = observed
        .findings
        .iter()
        .map(|finding| Finding {
            code: finding.code.into(),
            direction: finding.direction.into(),
            detail: finding.detail.clone(),
        })
        .collect();
    let mut completeness = match observed.end {
        ExchangeEnd::Complete => Completeness::Complete,
        ExchangeEnd::RequestTruncated | ExchangeEnd::ResponseTruncated => {
            Completeness::Incomplete(IncompleteReason::StreamEnded)
        }
        ExchangeEnd::ResponseMissing => {
            Completeness::Incomplete(IncompleteReason::ConnectionFailed)
        }
    };

    let (target_path, query_present) = observed
        .target
        .split_once('?')
        .map_or((observed.target.as_str(), false), |(path, _)| (path, true));
    let (_, request_redactions) = redact_headers(&header_fields(&observed.request_headers));
    let (_, response_redactions) = redact_headers(&header_fields(&observed.response_headers));
    let redaction_count = request_redactions.len() as u64
        + response_redactions.len() as u64
        + u64::from(query_present);

    let request = body_normalization(
        family.clone(),
        "request",
        &observed.request_body,
        &observed.request_headers,
    );
    let requested_identity = match &request {
        BodyNormalization::Normalized(normalization) => normalization.requested_identity.clone(),
        BodyNormalization::Empty => Presence::Absent,
        BodyNormalization::Skipped(reason) => Presence::Unknown {
            reason: (*reason).into(),
        },
        BodyNormalization::Unavailable(reason) => Presence::Unknown {
            reason: (*reason).into(),
        },
    };
    merge(&request, "request", &mut completeness, &mut findings);

    let response = if observed.status.is_none() {
        BodyNormalization::Unavailable("response-not-observed")
    } else {
        body_normalization(
            family,
            "response",
            &observed.response_body,
            &observed.response_headers,
        )
    };
    let (served_identity, usage, cost, events) = match &response {
        BodyNormalization::Normalized(normalization) => (
            normalization.served_identity.clone(),
            normalization.usage.clone(),
            match &normalization.reported_cost {
                Presence::Known(cost) => Cost::Reported {
                    value_text: cost.value_text.clone(),
                    currency_or_unit: cost.currency_or_unit.clone(),
                },
                Presence::Absent => Cost::Unknown {
                    reason: "provider-did-not-report".into(),
                },
                Presence::Unknown { reason } => Cost::Unknown {
                    reason: reason.clone(),
                },
            },
            normalization.events.clone(),
        ),
        BodyNormalization::Empty => unknown_response("provider-did-not-report"),
        BodyNormalization::Skipped(reason) | BodyNormalization::Unavailable(reason) => {
            unknown_response(reason)
        }
    };
    merge(&response, "response", &mut completeness, &mut findings);

    ExchangeRecord {
        binding: binding.clone(),
        sequence,
        transport: TransportMetadata {
            protocol: observed.protocol.into(),
            authority: observed.authority.canonical(),
            operation: observed.method.clone(),
            request_path: target_path.into(),
            response_status: observed.status.map_or_else(
                || Presence::Unknown {
                    reason: "response-not-observed".into(),
                },
                Presence::Known,
            ),
            stream_state: observed.end.as_str().into(),
        },
        provider: observed.provider.clone(),
        requested_identity,
        served_identity,
        usage,
        cost,
        retention: RetentionState {
            requested: requested.as_str().into(),
            effective: effective.as_str().into(),
        },
        redaction_count,
        completeness,
        findings,
        request_bytes: observed.request_wire.clone(),
        response_bytes: observed.response_wire.clone(),
        events,
    }
}

fn unknown_response(
    reason: &str,
) -> (
    Presence<wire_witness_core::exchange::ServedIdentity>,
    Vec<wire_witness_core::exchange::UsageEntry>,
    Cost,
    Vec<wire_witness_core::exchange::NormalizedEvent>,
) {
    (
        Presence::Unknown {
            reason: reason.into(),
        },
        Vec::new(),
        Cost::Unknown {
            reason: reason.into(),
        },
        Vec::new(),
    )
}

enum BodyNormalization {
    Normalized(Normalization),
    Empty,
    /// The body was observed but is not a provider message this runtime
    /// normalizes, such as an HTML error page.
    Skipped(&'static str),
    Unavailable(&'static str),
}

fn body_normalization(
    family: wire_witness_core::exchange::ProviderFamily,
    direction: &str,
    body: &CapturedBody,
    headers: &[(String, String)],
) -> BodyNormalization {
    let bytes = match body {
        CapturedBody::Observed(bytes) if bytes.is_empty() => return BodyNormalization::Empty,
        CapturedBody::Observed(bytes) => bytes,
        CapturedBody::Unavailable(reason) => return BodyNormalization::Unavailable(reason),
    };
    let media = headers
        .iter()
        .find(|(name, _)| name == "content-type")
        .map(|(_, value)| {
            value
                .split(';')
                .next()
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase()
        })
        .unwrap_or_default();
    if media == "application/json" || media.ends_with("+json") {
        BodyNormalization::Normalized(normalize_provider_json(family, direction, bytes))
    } else if direction == "response" && media == "text/event-stream" {
        BodyNormalization::Normalized(normalize_sse(family, bytes))
    } else if direction == "request" {
        BodyNormalization::Skipped("request-not-json")
    } else {
        BodyNormalization::Skipped("response-not-provider-message")
    }
}

fn merge(
    normalization: &BodyNormalization,
    direction: &str,
    completeness: &mut Completeness,
    findings: &mut Vec<Finding>,
) {
    match normalization {
        BodyNormalization::Normalized(normalized) => {
            findings.extend(normalized.findings.iter().cloned());
            if *completeness == Completeness::Complete {
                *completeness = normalized.completeness.clone();
            }
        }
        BodyNormalization::Unavailable(reason) => {
            findings.push(Finding {
                code: (*reason).into(),
                direction: direction.into(),
                detail: "body not normalized".into(),
            });
            if *completeness == Completeness::Complete {
                *completeness = Completeness::Incomplete(if *reason == "content-decoding-failed" {
                    IncompleteReason::Malformed
                } else {
                    IncompleteReason::LimitExceeded
                });
            }
        }
        BodyNormalization::Empty | BodyNormalization::Skipped(_) => {}
    }
}

fn header_fields(headers: &[(String, String)]) -> Vec<HeaderField> {
    headers
        .iter()
        .map(|(name, value)| HeaderField {
            name: name.clone(),
            value: value.clone(),
        })
        .collect()
}

/// Everything one supervised attempt needs, validated by its host before the
/// runtime starts. The directory exists, is private, and is empty; the
/// listener is bound to loopback; the CA was generated for this attempt.
pub struct SupervisedSetup {
    pub binding: Binding,
    pub config: ProxyConfig,
    pub directory: PathBuf,
    pub listener: TcpListener,
    pub ca: EphemeralCa,
    pub requested: RetentionMode,
    pub trust: ProviderTrust,
    pub limits: LiveLimits,
    pub grace: Duration,
}

/// A running supervised attempt: the spec 013 proxy and record writer with
/// a supervised binding, reporting installed records and findings as they
/// happen.
pub struct SupervisedRuntime {
    running: Running,
    grace: Duration,
    proxy_findings_read: usize,
}

/// A closed supervised attempt.
#[derive(Debug)]
pub struct SupervisedClosure {
    pub capture: CaptureState,
    /// Why capture is absent, when it is.
    pub reason: Option<&'static str>,
    /// Every reason capture is not complete, sorted.
    pub gaps: BTreeSet<String>,
    /// The installed spec 013 capture manifest.
    pub manifest_path: Option<PathBuf>,
    pub observed: u64,
    pub dropped: u64,
    /// Records that were built but could not be installed.
    pub missing: u64,
    pub exchanges: Vec<InstalledExchange>,
    pub findings: Vec<HostFinding>,
    /// Findings that arose during close and were reported by no event:
    /// late proxy findings and the closing assessments.
    pub closing_findings: Vec<RuntimeEvent>,
}

impl SupervisedRuntime {
    /// Starts the proxy on the supplied listener and the record writer under
    /// the supplied directory. Installed records and writer findings are
    /// sent on the returned receiver.
    pub fn start(setup: SupervisedSetup) -> Result<(Self, Receiver<RuntimeEvent>), HostError> {
        let requested = setup.requested;
        let mut findings = Vec::new();
        let effective = if requested == RetentionMode::Content {
            findings.push(HostFinding {
                code: "content-retention-unavailable".into(),
                detail:
                    "the live runtime retains metadata only; effective retention is metadata-only"
                        .into(),
            });
            RetentionMode::MetadataOnly
        } else {
            requested
        };
        let exchanges = setup.directory.join(EXCHANGES_DIRECTORY);
        create_private_directory(&exchanges)?;
        let (sender, receiver) = mpsc::sync_channel(setup.limits.sink_capacity);
        let (events, reported) = mpsc::channel();
        let finish = Arc::new(AtomicBool::new(false));
        let writer_finish = Arc::clone(&finish);
        let writer_binding = setup.binding.clone();
        let writer = thread::Builder::new()
            .name("wire-witness-writer".into())
            .spawn(move || {
                write_exchanges(
                    &receiver,
                    &exchanges,
                    &writer_binding,
                    requested,
                    effective,
                    &writer_finish,
                    Some(&events),
                )
            })
            .map_err(|error| HostError::Custody(error.kind().to_string()))?;
        let proxy = match LiveProxy::start_on(
            setup.listener,
            setup.config.clone(),
            setup.ca,
            setup.trust,
            setup.limits,
            sender,
        ) {
            Ok(proxy) => proxy,
            Err(error) => {
                finish.store(true, Ordering::SeqCst);
                let _ = writer.join();
                return Err(live_error(error));
            }
        };
        Ok((
            Self {
                running: Running {
                    proxy,
                    config: setup.config,
                    writer,
                    finish,
                    attempt: setup.directory,
                    binding: setup.binding,
                    requested,
                    effective,
                    findings,
                },
                grace: setup.grace,
                proxy_findings_read: 0,
            },
            reported,
        ))
    }

    #[must_use]
    pub const fn endpoint(&self) -> SocketAddr {
        self.running.proxy.local_address()
    }

    #[must_use]
    pub const fn effective_retention(&self) -> RetentionMode {
        self.running.effective
    }

    /// Findings the runtime recorded before any traffic, such as an
    /// unavailable retention mode.
    #[must_use]
    pub fn initial_findings(&self) -> Vec<RuntimeEvent> {
        self.running
            .findings
            .iter()
            .map(|finding| closing_finding(&finding.code))
            .collect()
    }

    /// Proxy findings recorded since the previous call, in recorded order.
    pub fn new_proxy_findings(&mut self) -> Vec<RuntimeEvent> {
        let fresh = self.running.proxy.findings_since(self.proxy_findings_read);
        self.proxy_findings_read += fresh.len();
        fresh.into_iter().map(proxy_event).collect()
    }

    /// Stops the proxy within the grace period, joins the writer, and
    /// installs the spec 013 capture manifest. `supervisor_gaps` are the
    /// host's own reasons the capture cannot be complete.
    pub fn close(self, supervisor_gaps: &[&str]) -> Result<SupervisedClosure, HostError> {
        let running = self.running;
        let summary = running.proxy.stop(self.grace);
        running.finish.store(true, Ordering::SeqCst);
        let outcome = running
            .writer
            .join()
            .map_err(|_| HostError::Custody("exchange-writer-failed".into()))?;
        let late: Vec<RuntimeEvent> = summary
            .findings
            .iter()
            .skip(self.proxy_findings_read)
            .cloned()
            .map(proxy_event)
            .collect();
        let closed = close_capture(
            &running.config,
            &running.attempt,
            &running.binding,
            running.requested,
            running.effective,
            running.findings,
            &summary,
            outcome,
            supervisor_gaps,
        );
        let mut closing_findings = late;
        closing_findings.extend(closed.closing_findings);
        Ok(SupervisedClosure {
            capture: closed.capture,
            reason: closed.reason,
            gaps: closed.gaps,
            manifest_path: closed.manifest_path,
            observed: closed.observed,
            dropped: closed.dropped,
            missing: closed.missing,
            exchanges: closed.exchanges,
            findings: closed.findings,
            closing_findings,
        })
    }
}

fn proxy_event(finding: LiveFinding) -> RuntimeEvent {
    RuntimeEvent::Finding {
        code: finding.code.to_owned(),
        authority: finding.authority,
    }
}

pub(crate) fn create_private_directory(path: &Path) -> Result<(), HostError> {
    let mut builder = DirBuilder::new();
    #[cfg(unix)]
    builder.mode(0o700);
    builder
        .create(path)
        .map_err(|error| HostError::Custody(error.kind().to_string()))
}

/// Writes a complete mode-0600 file under a temporary name and installs it at
/// a new final path. An existing final path is never replaced.
pub(crate) fn install_private(
    directory: &Path,
    name: &str,
    bytes: &[u8],
) -> Result<PathBuf, String> {
    let temporary = directory.join(format!(".{name}.partial"));
    let final_path = directory.join(name);
    write_private_file_new(&temporary, bytes).map_err(|error| format!("{error:?}"))?;
    let installed = fs::hard_link(&temporary, &final_path);
    let _ = fs::remove_file(&temporary);
    installed
        .map(|()| final_path)
        .map_err(|error| error.kind().to_string())
}

fn live_error(error: LiveError) -> HostError {
    match error {
        LiveError::InvalidLimits | LiveError::ProviderTrustUnavailable => {
            HostError::Precondition(error.code())
        }
        LiveError::CertificateAuthority(_) | LiveError::Tls(_) | LiveError::Listener(_) => {
            HostError::Custody(error.code().into())
        }
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
    use wire_witness_core::exchange::{ByteReference, ProviderFamily, ServedDisclosure};

    use super::*;

    fn observed(response_body: CapturedBody, content_type: &str) -> ObservedExchange {
        ObservedExchange {
            authority: Authority::parse("api.anthropic.com:443").unwrap(),
            provider: ProviderFamily::Anthropic,
            protocol: "http/1.1",
            method: "POST".into(),
            target: "/v1/messages?beta=true".into(),
            status: Some(200),
            request_headers: vec![
                ("content-type".into(), "application/json".into()),
                ("x-api-key".into(), "sk-ant-api03-not-a-real-key".into()),
            ],
            response_headers: vec![("content-type".into(), content_type.into())],
            request_wire: ByteReference::observed(b"request"),
            response_wire: ByteReference::observed(b"response"),
            request_body: CapturedBody::Observed(br#"{"model":"claude-requested"}"#.to_vec()),
            response_body,
            end: ExchangeEnd::Complete,
            findings: Vec::new(),
        }
    }

    fn binding() -> UnsupervisedBinding {
        UnsupervisedBinding {
            session_id: "0123456789abcdef0123456789abcdef".into(),
        }
    }

    #[test]
    fn records_place_normalized_identities_and_redact_without_values() {
        let record = exchange_record(
            &binding(),
            1,
            &observed(
                CapturedBody::Observed(
                    br#"{"model":"claude-served","usage":{"input_tokens":3}}"#.to_vec(),
                ),
                "application/json",
            ),
            RetentionMode::MetadataOnly,
            RetentionMode::MetadataOnly,
        );
        assert_eq!(
            record.requested_identity,
            Presence::Known("claude-requested".into())
        );
        let Presence::Known(served) = &record.served_identity else {
            panic!("served identity must be known");
        };
        assert_eq!(served.identity, "claude-served");
        assert_eq!(served.disclosure, ServedDisclosure::Reported);
        assert_eq!(record.transport.request_path, "/v1/messages");
        assert_eq!(record.redaction_count, 2);
        assert_eq!(record.completeness, Completeness::Complete);
        let bytes = String::from_utf8(record.canonical_bytes()).unwrap();
        assert!(!bytes.contains("sk-ant-api03"));
        assert!(!bytes.contains("beta=true"));
    }

    #[test]
    fn unavailable_bodies_are_unknown_and_incomplete_never_guessed() {
        let record = exchange_record(
            &binding(),
            1,
            &observed(
                CapturedBody::Unavailable("unsupported-content-encoding"),
                "application/json",
            ),
            RetentionMode::MetadataOnly,
            RetentionMode::MetadataOnly,
        );
        assert_eq!(
            record.served_identity,
            Presence::Unknown {
                reason: "unsupported-content-encoding".into()
            }
        );
        assert_eq!(
            record.completeness,
            Completeness::Incomplete(IncompleteReason::LimitExceeded)
        );
        assert!(
            record
                .findings
                .iter()
                .any(|finding| finding.code == "unsupported-content-encoding")
        );
    }

    #[test]
    fn missing_response_is_connection_failed() {
        let mut exchange = observed(CapturedBody::Unavailable("response-not-observed"), "");
        exchange.status = None;
        exchange.end = ExchangeEnd::ResponseMissing;
        let record = exchange_record(
            &binding(),
            1,
            &exchange,
            RetentionMode::MetadataOnly,
            RetentionMode::MetadataOnly,
        );
        assert_eq!(
            record.completeness,
            Completeness::Incomplete(IncompleteReason::ConnectionFailed)
        );
        assert_eq!(
            record.transport.response_status,
            Presence::Unknown {
                reason: "response-not-observed".into()
            }
        );
    }

    #[test]
    fn installs_are_private_and_never_overwrite() {
        let directory =
            std::env::temp_dir().join(format!("wire-witness-live-install-{}", std::process::id()));
        create_private_directory(&directory).unwrap();
        let path = install_private(&directory, "record.json", b"one").unwrap();
        assert!(install_private(&directory, "record.json", b"two").is_err());
        assert_eq!(fs::read(&path).unwrap(), b"one");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
            assert_eq!(
                fs::metadata(&directory).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        fs::remove_file(path).unwrap();
        fs::remove_dir(directory).unwrap();
    }
}

// endregion
