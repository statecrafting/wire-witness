//! The `wire-witness sidecar` stdio host: one supervised attempt per process
//! over the spec 005 protocol and the spec 013 live runtime.
//!
//! Governed by spec 014, the wire-witness.sidecar/1 stdio host.

// region: sidecar-host

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::path::{Component, Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, TryRecvError};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use wire_witness_core::exchange::{Binding, JsonValue, canonical_json_bytes, parse_json};
use wire_witness_proxy::live::{EphemeralCa, LiveLimits, ProviderTrust};
use wire_witness_proxy::{ApplicationProtocol, CaptureAuthority, ProxyConfig};

use crate::live_runtime::{
    EXCHANGES_DIRECTORY, MANIFEST_FILE, RuntimeEvent, SupervisedClosure, SupervisedRuntime,
    SupervisedSetup, create_private_directory, install_private,
};
use crate::producer_identity::{APPLICATION_PROTOCOL, ProducerIdentity};
use crate::sidecar_protocol::{
    AttemptBinding, CaptureCompleteness, EvidenceReference, MessageKind, PROTOCOL, ProtocolError,
    StartLimits, StartRequest, parse_line,
};
use crate::standalone_host::{CaptureState, write_private_file_new};
use crate::wire_exchange::{WIRE_EXCHANGE_FILE, WireExchangeFacts, manifest_bytes};

/// The bound on the `start` line, newline included.
pub const START_LINE_BYTES: usize = 65_536;

/// Exit codes of one sidecar process.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SidecarExit {
    /// `finished` was written; completeness is in the message, not here.
    Finished,
    /// Refused before `ready`; nothing was written to stdout.
    Refused,
    /// `finished` was not written: stdout failed, or custody could not be
    /// closed.
    Unfinished,
}

impl SidecarExit {
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::Finished => 0,
            Self::Refused => 2,
            Self::Unfinished => 4,
        }
    }
}

/// Host settings a supervisor or test may fix; the command line uses
/// [`HostOptions::system`].
#[derive(Clone, Debug)]
pub struct HostOptions {
    pub trust: ProviderTrust,
    pub grace: Duration,
    pub poll: Duration,
}

impl HostOptions {
    #[must_use]
    pub fn system() -> Self {
        Self {
            trust: ProviderTrust::System,
            grace: Duration::from_secs(5),
            poll: Duration::from_millis(25),
        }
    }
}

/// How custody closed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EndDisposition {
    SupervisorClosed,
    ProtocolViolation,
    Deadline,
    StdoutBroken,
}

impl EndDisposition {
    const fn as_str(self) -> &'static str {
        match self {
            Self::SupervisorClosed => "supervisor-closed",
            Self::ProtocolViolation => "supervisor-protocol-violation",
            Self::Deadline => "deadline",
            Self::StdoutBroken => "stdout-broken",
        }
    }

    const fn gap(self) -> Option<&'static str> {
        match self {
            Self::SupervisorClosed => None,
            Self::ProtocolViolation => Some("unexpected-supervisor-message"),
            Self::Deadline => Some("deadline-reached"),
            Self::StdoutBroken => Some("stdout-broken"),
        }
    }
}

enum Supervisor {
    Eof,
    Message,
}

/// Runs one supervised session. Diagnostics are fixed codes only.
pub fn run_session<I, O, E>(
    input: I,
    output: &mut O,
    diagnostics: &mut E,
    options: &HostOptions,
    producer: &ProducerIdentity,
) -> SidecarExit
where
    I: Read + Send + 'static,
    O: Write,
    E: Write,
{
    let mut input = BufReader::new(input);
    let prepared = match prepare(&mut input, options) {
        Ok(prepared) => prepared,
        Err(code) => {
            let _ = writeln!(diagnostics, "wire-witness: {code}");
            return SidecarExit::Refused;
        }
    };
    Session {
        output,
        diagnostics,
        binding: prepared.start.binding.clone(),
        producer,
        written_findings: 0,
        suppressed: 0,
        stdout_broken: false,
    }
    .run(input, prepared, options)
}

struct Prepared {
    start: StartRequest,
    directory: PathBuf,
    runtime: SupervisedRuntime,
    events: Receiver<RuntimeEvent>,
    unenforced: Vec<&'static str>,
}

fn prepare<R: BufRead>(input: &mut R, options: &HostOptions) -> Result<Prepared, &'static str> {
    let mut line = Vec::new();
    input
        .take(START_LINE_BYTES as u64)
        .read_until(b'\n', &mut line)
        .map_err(|_| "start-missing")?;
    if line.is_empty() {
        return Err("start-missing");
    }
    let message = parse_line(&line, START_LINE_BYTES).map_err(|error| match error {
        ProtocolError::InvalidBinding => "binding-invalid",
        _ => "start-malformed",
    })?;
    if message.kind != MessageKind::Start {
        return Err("start-malformed");
    }
    let start = StartRequest::parse(&message).map_err(|_| "start-invalid")?;
    if start
        .capture_authorities
        .iter()
        .any(|authority| !authority.protocols.contains(&ApplicationProtocol::Http1))
    {
        return Err("protocol-unsupported");
    }
    let directory = PathBuf::from(&start.output_directory);
    let parent = output_parent(&directory)?;
    certificate_in(&directory, &start.certificate_path)?;
    let endpoint = listen_endpoint(message.field("listen_endpoint"))?;
    let config = ProxyConfig::new(
        start
            .capture_authorities
            .iter()
            .map(|entry| CaptureAuthority {
                authority: entry.authority.clone(),
                provider: entry.provider.clone(),
                protocols: entry.protocols.clone(),
            })
            .collect(),
        start
            .authentication_tunnels
            .iter()
            .map(|entry| entry.authority.clone())
            .collect(),
    )
    .map_err(|_| "start-invalid")?;

    create_owned_directory(&directory, &parent)?;
    let Ok(listener) = TcpListener::bind(endpoint) else {
        let _ = fs::remove_dir(&directory);
        return Err("endpoint-unavailable");
    };
    let (limits, unenforced) = live_limits(&start.limits);
    let started = EphemeralCa::generate(SystemTime::now())
        .map_err(|_| ())
        .and_then(|ca| {
            write_private_file_new(
                Path::new(&start.certificate_path),
                ca.certificate_pem().as_bytes(),
            )
            .map_err(|_| ())?;
            SupervisedRuntime::start(SupervisedSetup {
                binding: Binding::Supervised {
                    run_id: start.binding.run_id.clone(),
                    attempt: start.binding.attempt,
                    effect_id: start.binding.effect_id.clone(),
                },
                config,
                directory: directory.clone(),
                listener,
                ca,
                requested: start.requested_retention,
                trust: options.trust.clone(),
                limits,
                grace: options.grace,
            })
            .map_err(|_| ())
        });
    match started {
        Ok((runtime, events)) => Ok(Prepared {
            start,
            directory,
            runtime,
            events,
            unenforced,
        }),
        Err(()) => {
            // The directory was created by this process moments ago and
            // holds only what this attempt wrote.
            let _ = fs::remove_dir_all(&directory);
            Err("attempt-preparation-failed")
        }
    }
}

/// `output_directory` is absolute, normal, absent, and under an existing
/// directory that is not group- or world-writable.
fn output_parent(directory: &Path) -> Result<PathBuf, &'static str> {
    const INVALID: &str = "output-directory-invalid";
    if !directory.is_absolute()
        || directory
            .components()
            .any(|part| matches!(part, Component::CurDir | Component::ParentDir))
        || fs::symlink_metadata(directory).is_ok()
    {
        return Err(INVALID);
    }
    let parent = directory.parent().ok_or(INVALID)?.to_path_buf();
    let metadata = fs::metadata(&parent).map_err(|_| INVALID)?;
    if !metadata.is_dir() {
        return Err(INVALID);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o022 != 0 {
            return Err(INVALID);
        }
    }
    Ok(parent)
}

/// Creates the attempt directory with mode 0700. Its owner is the effective
/// user, so a parent with any other owner is refused and the directory
/// removed again.
fn create_owned_directory(directory: &Path, parent: &Path) -> Result<(), &'static str> {
    const INVALID: &str = "output-directory-invalid";
    create_private_directory(directory).map_err(|_| INVALID)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let owned = match (fs::metadata(directory), fs::metadata(parent)) {
            (Ok(created), Ok(parent)) => created.uid() == parent.uid(),
            _ => false,
        };
        if !owned {
            let _ = fs::remove_dir(directory);
            return Err(INVALID);
        }
    }
    #[cfg(not(unix))]
    let _ = parent;
    Ok(())
}

fn certificate_in(directory: &Path, certificate: &str) -> Result<(), &'static str> {
    const INVALID: &str = "certificate-path-invalid";
    let path = Path::new(certificate);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(INVALID)?;
    if path.parent() != Some(directory)
        || name.starts_with('.')
        || [MANIFEST_FILE, WIRE_EXCHANGE_FILE, EXCHANGES_DIRECTORY].contains(&name)
    {
        return Err(INVALID);
    }
    Ok(())
}

fn listen_endpoint(value: Option<&JsonValue>) -> Result<SocketAddr, &'static str> {
    const INVALID: &str = "endpoint-invalid";
    let text = match value {
        None | Some(JsonValue::Null) => return Ok(SocketAddr::from(([127, 0, 0, 1], 0))),
        Some(JsonValue::String(text)) => text,
        Some(_) => return Err(INVALID),
    };
    let port = text
        .strip_prefix("127.0.0.1:")
        .or_else(|| text.strip_prefix("[::1]:"))
        .ok_or(INVALID)?;
    if port.is_empty() || !port.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(INVALID);
    }
    let port: u16 = port.parse().map_err(|_| INVALID)?;
    if port == 0 {
        return Err(INVALID);
    }
    text.parse().map_err(|_| INVALID)
}

/// Applies every `start` limit the HTTP/1.1 runtime has an equivalent for and
/// names the rest. `total_seconds` is the host's own deadline.
fn live_limits(limits: &StartLimits) -> (LiveLimits, Vec<&'static str>) {
    let size = |value: u64| usize::try_from(value).unwrap_or(usize::MAX);
    let defaults = LiveLimits::default();
    let applied = LiveLimits {
        header_bytes: size(limits.header_bytes),
        header_count: size(limits.header_count),
        captured_body_bytes: size(limits.message_bytes),
        decoded_body_bytes: size(limits.decompressed_bytes),
        connections: size(limits.connection_count),
        sink_capacity: defaults.sink_capacity,
        idle: Duration::from_secs(limits.idle_seconds),
        connect: defaults.connect,
    };
    let unenforced = vec![
        "concurrent_streams",
        "event_bytes",
        "frame_bytes",
        "queue_bytes",
    ];
    (applied, unenforced)
}

struct Session<'a, O, E> {
    output: &'a mut O,
    diagnostics: &'a mut E,
    binding: AttemptBinding,
    producer: &'a ProducerIdentity,
    written_findings: u64,
    suppressed: u64,
    stdout_broken: bool,
}

impl<O: Write, E: Write> Session<'_, O, E> {
    fn run<I: Read + Send + 'static>(
        mut self,
        input: BufReader<I>,
        prepared: Prepared,
        options: &HostOptions,
    ) -> SidecarExit {
        let Prepared {
            start,
            directory,
            mut runtime,
            events,
            unenforced,
        } = prepared;
        let endpoint = runtime.endpoint();
        let effective = runtime.effective_retention();
        self.emit(
            "ready",
            [
                (
                    "application_protocols",
                    JsonValue::Array(vec![string(APPLICATION_PROTOCOL)]),
                ),
                ("certificate_path", string(&start.certificate_path)),
                ("effective_retention", string(effective.as_str())),
                ("endpoint", string(&endpoint.to_string())),
                ("memory_lock", string("not-attempted")),
                ("producer", self.producer.message_subset()),
                (
                    "unenforced_limits",
                    JsonValue::Array(unenforced.iter().map(|name| string(name)).collect()),
                ),
            ],
        );
        for event in runtime.initial_findings() {
            self.notice(&event);
        }

        let supervisor = watch_supervisor(input);
        let deadline = Instant::now().checked_add(Duration::from_secs(start.limits.total_seconds));
        let disposition = loop {
            if self.stdout_broken {
                break EndDisposition::StdoutBroken;
            }
            for event in drain(&events) {
                self.notice(&event);
            }
            for event in runtime.new_proxy_findings() {
                self.notice(&event);
            }
            if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                break EndDisposition::Deadline;
            }
            match supervisor.recv_timeout(options.poll) {
                Ok(Supervisor::Eof) | Err(RecvTimeoutError::Disconnected) => {
                    break EndDisposition::SupervisorClosed;
                }
                Ok(Supervisor::Message) => break EndDisposition::ProtocolViolation,
                Err(RecvTimeoutError::Timeout) => {}
            }
        };

        let gaps: Vec<&str> = disposition.gap().into_iter().collect();
        let closure = match runtime.close(&gaps) {
            Ok(closure) => closure,
            Err(_) => {
                let _ = writeln!(self.diagnostics, "wire-witness: custody-close-failed");
                return SidecarExit::Unfinished;
            }
        };
        for event in drain(&events) {
            self.notice(&event);
        }
        for event in &closure.closing_findings {
            self.notice(event);
        }
        self.finish(&start, &directory, closure, disposition)
    }

    fn finish(
        mut self,
        start: &StartRequest,
        directory: &Path,
        mut closure: SupervisedClosure,
        disposition: EndDisposition,
    ) -> SidecarExit {
        let mut evidence: Option<(PathBuf, Vec<u8>)> = None;
        if closure.capture != CaptureState::Absent {
            let findings = closure.findings.len() as u64 + self.suppressed;
            let bytes = manifest_bytes(&WireExchangeFacts {
                binding: &self.binding,
                producer: self.producer,
                host_policy_id: &start.host_policy_id,
                end_disposition: disposition.as_str(),
                gaps: &closure.gaps,
                requested_retention: start.requested_retention,
                missing_records: closure.missing,
                exchanges: &closure.exchanges,
                observed_exchanges: closure.observed,
                dropped_exchanges: closure.dropped,
                findings,
            });
            match install_private(directory, WIRE_EXCHANGE_FILE, &bytes) {
                Ok(path) => evidence = Some((path, bytes)),
                Err(_) => {
                    closure.gaps.insert("manifest-not-installed".into());
                    closure.capture = CaptureState::Incomplete;
                    closure.findings.push(crate::standalone_host::HostFinding {
                        code: "manifest-not-installed".into(),
                        detail: WIRE_EXCHANGE_FILE.into(),
                    });
                    self.notice(&RuntimeEvent::Finding {
                        code: "manifest-not-installed".into(),
                        authority: None,
                    });
                }
            }
        }
        if self.stdout_broken || disposition == EndDisposition::StdoutBroken {
            let _ = writeln!(self.diagnostics, "wire-witness: stdout-broken");
            return SidecarExit::Unfinished;
        }

        let completeness = match closure.capture {
            CaptureState::Complete => CaptureCompleteness::Complete,
            CaptureState::Incomplete => CaptureCompleteness::Incomplete,
            CaptureState::Absent => CaptureCompleteness::Absent,
        };
        let reason = match closure.capture {
            CaptureState::Complete => JsonValue::Null,
            CaptureState::Absent => string(closure.reason.unwrap_or("no-exchange-observed")),
            CaptureState::Incomplete => string(
                &closure
                    .gaps
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join(","),
            ),
        };
        let (manifest_path, capture_digest, reference) = evidence.map_or(
            (JsonValue::Null, JsonValue::Null, JsonValue::Null),
            |(path, bytes)| {
                let reference = EvidenceReference::for_manifest(
                    &bytes,
                    self.binding.clone(),
                    completeness,
                    Some(format!("wire-witness@{}", self.producer.version)),
                    self.producer.executable_digest.clone(),
                );
                (
                    string(&path.to_string_lossy()),
                    string(&format!("sha256:{}", reference.sha256)),
                    parse_json(&reference.canonical_bytes()).unwrap_or(JsonValue::Null),
                )
            },
        );
        let finding_count = closure.findings.len() as u64 + self.suppressed;
        self.emit(
            "finished",
            [
                ("capture_digest", capture_digest),
                ("completeness", string(completeness_name(completeness))),
                ("end_disposition", string(disposition.as_str())),
                ("evidence_reference", reference),
                ("exchange_count", number(closure.exchanges.len() as u64)),
                ("finding_count", number(finding_count)),
                (
                    "gaps",
                    JsonValue::Array(closure.gaps.iter().map(|gap| string(gap)).collect()),
                ),
                ("manifest_path", manifest_path),
                ("producer", self.producer.message_subset()),
                ("reason", reason),
            ],
        );
        if self.stdout_broken {
            let _ = writeln!(self.diagnostics, "wire-witness: stdout-broken");
            return SidecarExit::Unfinished;
        }
        SidecarExit::Finished
    }

    fn notice(&mut self, event: &RuntimeEvent) {
        match event {
            RuntimeEvent::Exchange(exchange) => self.emit(
                "exchange",
                [
                    ("completeness", string(exchange.completeness)),
                    ("record_byte_length", number(exchange.byte_length)),
                    (
                        "record_digest",
                        string(&format!("sha256:{}", exchange.digest)),
                    ),
                    ("sequence", number(exchange.sequence)),
                ],
            ),
            RuntimeEvent::Finding { code, authority } => {
                self.written_findings += 1;
                match authority {
                    Some(authority) => self.emit(
                        "finding",
                        [("authority", string(authority)), ("code", string(code))],
                    ),
                    None => self.emit("finding", [("code", string(code))]),
                }
            }
        }
    }

    /// Writes one message, after checking it against spec 005's parser and
    /// forbidden-field rule. A message that fails is suppressed and counted.
    fn emit<const N: usize>(&mut self, kind: &str, members: [(&str, JsonValue); N]) {
        if self.stdout_broken {
            return;
        }
        let mut fields: std::collections::BTreeMap<String, JsonValue> = members
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value))
            .collect();
        fields.insert("protocol".into(), string(PROTOCOL));
        fields.insert("type".into(), string(kind));
        fields.insert("binding".into(), binding_json(&self.binding));
        let mut line = canonical_json_bytes(&JsonValue::Object(fields));
        line.push(b'\n');
        match parse_line(&line, usize::MAX) {
            Ok(parsed) if parsed.binding == self.binding => {}
            _ => {
                self.suppressed += 1;
                let _ = writeln!(self.diagnostics, "wire-witness: stdout-message-suppressed");
                return;
            }
        }
        if self
            .output
            .write_all(&line)
            .and_then(|()| self.output.flush())
            .is_err()
        {
            self.stdout_broken = true;
        }
    }
}

/// Reads the supervisor's side after `start`: any further byte is a protocol
/// violation and EOF is the request to close.
fn watch_supervisor<I: Read + Send + 'static>(mut input: BufReader<I>) -> Receiver<Supervisor> {
    let (sender, receiver) = mpsc::channel();
    let _ = thread::Builder::new()
        .name("wire-witness-supervisor".into())
        .spawn(move || {
            let mut byte = [0_u8; 1];
            let event = match input.read(&mut byte) {
                Ok(0) | Err(_) => Supervisor::Eof,
                Ok(_) => Supervisor::Message,
            };
            let _ = sender.send(event);
        });
    receiver
}

fn drain(events: &Receiver<RuntimeEvent>) -> Vec<RuntimeEvent> {
    let mut drained = Vec::new();
    loop {
        match events.try_recv() {
            Ok(event) => drained.push(event),
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => return drained,
        }
    }
}

const fn completeness_name(completeness: CaptureCompleteness) -> &'static str {
    match completeness {
        CaptureCompleteness::Complete => "complete",
        CaptureCompleteness::Incomplete => "incomplete",
        CaptureCompleteness::Absent => "absent",
    }
}

fn binding_json(binding: &AttemptBinding) -> JsonValue {
    JsonValue::Object(
        [
            ("attempt", number(u64::from(binding.attempt))),
            ("effect_id", string(&binding.effect_id)),
            ("run_id", string(&binding.run_id)),
        ]
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
    use super::*;

    #[test]
    fn listen_endpoint_is_loopback_with_a_port() {
        assert_eq!(
            listen_endpoint(None),
            Ok(SocketAddr::from(([127, 0, 0, 1], 0)))
        );
        assert_eq!(
            listen_endpoint(Some(&string("127.0.0.1:4100"))),
            Ok(SocketAddr::from(([127, 0, 0, 1], 4100)))
        );
        assert!(listen_endpoint(Some(&string("[::1]:4100"))).is_ok());
        for refused in [
            "0.0.0.0:4100",
            "192.168.1.2:4100",
            "127.0.0.1:0",
            "127.0.0.1:",
            "127.0.0.1:+80",
            "localhost:4100",
            "[::]:4100",
        ] {
            assert_eq!(
                listen_endpoint(Some(&string(refused))),
                Err("endpoint-invalid"),
                "{refused}"
            );
        }
        assert_eq!(
            listen_endpoint(Some(&number(4100))),
            Err("endpoint-invalid")
        );
    }

    #[test]
    fn certificate_must_sit_directly_inside_the_attempt_directory() {
        let directory = Path::new("/attempt");
        assert!(certificate_in(directory, "/attempt/ca.pem").is_ok());
        for refused in [
            "/elsewhere/ca.pem",
            "/attempt/sub/ca.pem",
            "/attempt/.ca.pem",
            "/attempt/manifest.json",
            "/attempt/wire-exchange.json",
            "/attempt/exchanges",
            "ca.pem",
        ] {
            assert_eq!(
                certificate_in(directory, refused),
                Err("certificate-path-invalid"),
                "{refused}"
            );
        }
    }

    #[test]
    fn output_directory_must_be_absolute_normal_and_new() {
        for refused in ["relative/attempt", "/tmp/../attempt", "/"] {
            assert_eq!(
                output_parent(Path::new(refused)),
                Err("output-directory-invalid"),
                "{refused}"
            );
        }
        let existing = std::env::temp_dir();
        assert_eq!(output_parent(&existing), Err("output-directory-invalid"));
    }

    #[test]
    fn unenforced_limits_are_named_and_sorted() {
        let (applied, unenforced) = live_limits(&StartLimits {
            header_bytes: 10,
            header_count: 11,
            frame_bytes: 12,
            message_bytes: 13,
            decompressed_bytes: 14,
            event_bytes: 15,
            concurrent_streams: 16,
            connection_count: 17,
            queue_bytes: 18,
            idle_seconds: 19,
            total_seconds: 20,
        });
        assert_eq!(applied.header_bytes, 10);
        assert_eq!(applied.captured_body_bytes, 13);
        assert_eq!(applied.decoded_body_bytes, 14);
        assert_eq!(applied.connections, 17);
        assert_eq!(applied.idle, Duration::from_secs(19));
        let mut sorted = unenforced.clone();
        sorted.sort_unstable();
        assert_eq!(unenforced, sorted);
    }
}

// endregion
