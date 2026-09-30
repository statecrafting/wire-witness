//! Standalone process host, custody, rendering, and exit vocabulary.
//!
//! Governed by spec 006, standalone host and per-process environment.

// region: standalone-host

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

use wire_witness_core::custody::RetentionMode;
use wire_witness_core::exchange::{JsonValue, ProviderFamily, canonical_json_bytes};
use wire_witness_core::instruction_observation::ComparisonPlan;
use wire_witness_proxy::{ApplicationProtocol, Authority, CaptureAuthority, ProxyConfig};

use crate::instruction_observation::{InstructionProbeSource, ProbeLoadError, load_plans};
use crate::sidecar_protocol::StartAuthority;

pub const RESULT_SCHEMA: &str = "wire-witness.result/1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputFormat {
    Human,
    Json,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliCommand {
    Run {
        request: RunRequest,
        output: OutputFormat,
    },
    Sidecar,
}

pub fn parse_arguments(arguments: &[String]) -> Result<CliCommand, HostError> {
    let Some(command) = arguments.first().map(String::as_str) else {
        return Err(HostError::Usage("command-required"));
    };
    if command == "sidecar" {
        return if arguments.len() == 1 {
            Ok(CliCommand::Sidecar)
        } else {
            Err(HostError::Usage("sidecar-takes-no-options"))
        };
    }
    if command != "run" {
        return Err(HostError::Usage("unknown-command"));
    }

    let mut capture_authorities = Vec::new();
    let mut authentication_tunnels = Vec::new();
    let mut retention = RetentionMode::MetadataOnly;
    let mut content_opt_in = false;
    let mut output_directory = None;
    let mut trust = None;
    let mut output = OutputFormat::Human;
    let mut position = 1;
    while position < arguments.len() && arguments[position] != "--" {
        match arguments[position].as_str() {
            "--capture" | "--auth-tunnel" => {
                let option = arguments[position].as_str();
                let value = arguments
                    .get(position + 1)
                    .ok_or(HostError::Usage("option-value-required"))?;
                let parsed = parse_authority_option(value)?;
                if option == "--capture" {
                    capture_authorities.push(parsed);
                } else {
                    authentication_tunnels.push(parsed);
                }
                position += 2;
            }
            "--output" => {
                output_directory = Some(PathBuf::from(
                    arguments
                        .get(position + 1)
                        .ok_or(HostError::Usage("option-value-required"))?,
                ));
                position += 2;
            }
            "--retention" => {
                retention = match arguments.get(position + 1).map(String::as_str) {
                    Some("metadata-only") => RetentionMode::MetadataOnly,
                    Some("disabled") => RetentionMode::Disabled,
                    Some("content") => {
                        content_opt_in = true;
                        RetentionMode::Content
                    }
                    _ => return Err(HostError::Usage("invalid-retention")),
                };
                position += 2;
            }
            "--trust" => {
                trust = Some(match arguments.get(position + 1).map(String::as_str) {
                    Some("node-extra-ca-certs") => TrustMechanism::NodeExtraCaCerts,
                    Some("aws-ca-bundle") => TrustMechanism::AwsCaBundle,
                    Some("existing-roots-bundle") => TrustMechanism::ExistingRootsBundle,
                    _ => return Err(HostError::Usage("invalid-trust-mechanism")),
                });
                position += 2;
            }
            "--json" => {
                output = OutputFormat::Json;
                position += 1;
            }
            _ => return Err(HostError::Usage("unknown-option")),
        }
    }
    if arguments.get(position).map(String::as_str) != Some("--") {
        return Err(HostError::Usage("program-separator-required"));
    }
    let program = arguments
        .get(position + 1)
        .filter(|value| !value.is_empty())
        .ok_or(HostError::Usage("program-required"))?
        .clone();
    Ok(CliCommand::Run {
        request: RunRequest {
            capture_authorities,
            authentication_tunnels,
            requested_retention: retention,
            content_opt_in,
            output_directory: output_directory
                .ok_or(HostError::Usage("output-directory-required"))?,
            trust: trust.ok_or(HostError::Usage("trust-mechanism-required"))?,
            instruction_probes: Vec::new(),
            program,
            arguments: arguments[position + 2..].to_vec(),
        },
        output,
    })
}

fn parse_authority_option(value: &str) -> Result<StartAuthority, HostError> {
    let mut parts = value.split(',');
    let authority = Authority::parse(parts.next().unwrap_or_default())
        .map_err(|_| HostError::Usage("invalid-authority"))?;
    let provider = match parts.next() {
        Some("anthropic") => ProviderFamily::Anthropic,
        Some("openai") => ProviderFamily::OpenAi,
        Some("unknown") => ProviderFamily::Unknown,
        _ => return Err(HostError::Usage("invalid-provider")),
    };
    let protocols = parts
        .next()
        .ok_or(HostError::Usage("protocol-required"))?
        .split('+')
        .map(|protocol| match protocol {
            "http/1.1" => Ok(ApplicationProtocol::Http1),
            "h2" => Ok(ApplicationProtocol::Http2),
            "sse" => Ok(ApplicationProtocol::Sse),
            "websocket" => Ok(ApplicationProtocol::WebSocket),
            _ => Err(HostError::Usage("invalid-protocol")),
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    if protocols.is_empty() || parts.next().is_some() {
        return Err(HostError::Usage("invalid-authority-option"));
    }
    Ok(StartAuthority {
        authority,
        provider,
        protocols,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum HostExit {
    Success = 0,
    Finding = 1,
    Refused = 2,
    Usage = 3,
    Unexpected = 4,
}

impl HostExit {
    #[must_use]
    pub const fn code(self) -> u8 {
        self as u8
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsupervisedBinding {
    pub session_id: String,
}

impl UnsupervisedBinding {
    pub fn generate() -> Result<Self, HostError> {
        let mut source =
            File::open("/dev/urandom").map_err(|_| HostError::RandomnessUnavailable)?;
        Self::from_reader(&mut source)
    }

    pub fn from_reader(reader: &mut impl Read) -> Result<Self, HostError> {
        let mut bytes = [0_u8; 16];
        reader
            .read_exact(&mut bytes)
            .map_err(|_| HostError::RandomnessUnavailable)?;
        let mut session_id = String::with_capacity(32);
        for byte in bytes {
            use std::fmt::Write as _;
            write!(&mut session_id, "{byte:02x}").expect("writing to a string cannot fail");
        }
        Ok(Self { session_id })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustMechanism {
    NodeExtraCaCerts,
    AwsCaBundle,
    ExistingRootsBundle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunRequest {
    pub capture_authorities: Vec<StartAuthority>,
    pub authentication_tunnels: Vec<StartAuthority>,
    pub requested_retention: RetentionMode,
    pub content_opt_in: bool,
    pub output_directory: PathBuf,
    pub trust: TrustMechanism,
    pub instruction_probes: Vec<InstructionProbeSource>,
    pub program: String,
    pub arguments: Vec<String>,
}

impl RunRequest {
    pub fn validate(&self) -> Result<ProxyConfig, HostError> {
        if self.capture_authorities.is_empty() {
            return Err(HostError::Precondition("capture-authority-required"));
        }
        if self.program.is_empty() || self.program.contains(['\0', '\r', '\n']) {
            return Err(HostError::Usage("program-required"));
        }
        if self.requested_retention == RetentionMode::Content && !self.content_opt_in {
            return Err(HostError::Precondition("content-retention-requires-opt-in"));
        }
        if self
            .capture_authorities
            .iter()
            .chain(&self.authentication_tunnels)
            .any(|entry| entry.protocols.is_empty())
        {
            return Err(HostError::Precondition("authority-protocol-required"));
        }
        validate_output_directory(&self.output_directory)?;
        ProxyConfig::new(
            self.capture_authorities
                .iter()
                .map(|entry| CaptureAuthority {
                    authority: entry.authority.clone(),
                    provider: entry.provider.clone(),
                    protocols: entry.protocols.clone(),
                })
                .collect(),
            self.authentication_tunnels
                .iter()
                .map(|entry| entry.authority.clone())
                .collect(),
        )
        .map_err(|_| HostError::Precondition("invalid-authority-configuration"))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExchangeSequence {
    last: u64,
}

impl ExchangeSequence {
    pub fn next_sequence(&mut self) -> Result<u64, HostError> {
        self.last = self
            .last
            .checked_add(1)
            .ok_or(HostError::Custody("exchange-sequence-overflow".into()))?;
        Ok(self.last)
    }

    #[must_use]
    pub const fn observed(&self) -> u64 {
        self.last
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChildEnvironment {
    values: BTreeMap<String, String>,
}

impl ChildEnvironment {
    pub fn new(
        proxy: &Authority,
        certificate_or_bundle: &Path,
        trust: TrustMechanism,
    ) -> Result<Self, HostError> {
        if !proxy.host().eq_ignore_ascii_case("127.0.0.1") && proxy.host() != "[::1]" {
            return Err(HostError::Precondition("proxy-must-be-loopback"));
        }
        let trust_path = certificate_or_bundle
            .to_str()
            .ok_or(HostError::Precondition("trust-path-not-utf8"))?;
        if trust_path.contains(['\0', '\r', '\n']) {
            return Err(HostError::Precondition("invalid-trust-path"));
        }
        let endpoint = format!("http://{}", proxy.canonical());
        let mut values = BTreeMap::from([
            ("HTTP_PROXY".into(), endpoint.clone()),
            ("HTTPS_PROXY".into(), endpoint.clone()),
            ("http_proxy".into(), endpoint.clone()),
            ("https_proxy".into(), endpoint),
        ]);
        let trust_name = match trust {
            TrustMechanism::NodeExtraCaCerts => "NODE_EXTRA_CA_CERTS",
            TrustMechanism::AwsCaBundle => "AWS_CA_BUNDLE",
            TrustMechanism::ExistingRootsBundle => "SSL_CERT_FILE",
        };
        values.insert(trust_name.into(), trust_path.into());
        Ok(Self { values })
    }

    pub fn apply(&self, command: &mut Command) {
        command.envs(&self.values);
    }

    #[must_use]
    pub fn values(&self) -> &BTreeMap<String, String> {
        &self.values
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChildExit {
    Code(i32),
    Signal,
}

impl From<ExitStatus> for ChildExit {
    fn from(status: ExitStatus) -> Self {
        status.code().map_or(Self::Signal, Self::Code)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CaptureState {
    Complete,
    Incomplete,
    Absent,
}

impl CaptureState {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Incomplete => "incomplete",
            Self::Absent => "absent",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostFinding {
    pub code: String,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostResult {
    pub binding: UnsupervisedBinding,
    pub child_exit: ChildExit,
    pub capture: CaptureState,
    pub exchange_count: u64,
    pub manifest_path: Option<String>,
    pub findings: Vec<HostFinding>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedAttempt {
    pub proxy: Authority,
    pub certificate_or_bundle: PathBuf,
    pub cleanup_paths: Vec<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosedCapture {
    pub capture: CaptureState,
    pub exchange_count: u64,
    pub manifest_path: Option<String>,
    pub findings: Vec<HostFinding>,
}

pub trait AttemptRuntime {
    fn prepare(
        &mut self,
        request: &RunRequest,
        binding: &UnsupervisedBinding,
        instruction_plans: Vec<ComparisonPlan>,
    ) -> Result<PreparedAttempt, HostError>;

    fn close(self) -> Result<ClosedCapture, HostError>;
}

impl HostResult {
    #[must_use]
    pub fn exit(&self) -> HostExit {
        if !self.findings.is_empty()
            || self.capture != CaptureState::Complete
            || self.child_exit != ChildExit::Code(0)
        {
            HostExit::Finding
        } else {
            HostExit::Success
        }
    }

    #[must_use]
    pub fn json_bytes(&self) -> Vec<u8> {
        let findings = self
            .findings
            .iter()
            .map(|finding| {
                object([
                    ("code", string(&finding.code)),
                    ("detail", string(&finding.detail)),
                ])
            })
            .collect();
        let child_exit = match self.child_exit {
            ChildExit::Code(code) => object([
                ("kind", string("code")),
                ("value", JsonValue::Number(code.to_string())),
            ]),
            ChildExit::Signal => object([("kind", string("signal")), ("value", JsonValue::Null)]),
        };
        canonical_json_bytes(&object([
            (
                "binding",
                object([
                    ("kind", string("unsupervised")),
                    ("session_id", string(&self.binding.session_id)),
                ]),
            ),
            ("capture", string(self.capture.as_str())),
            ("child_exit", child_exit),
            (
                "exchange_count",
                JsonValue::Number(self.exchange_count.to_string()),
            ),
            ("findings", JsonValue::Array(findings)),
            (
                "manifest_path",
                self.manifest_path
                    .as_deref()
                    .map_or(JsonValue::Null, string),
            ),
            ("schema", string(RESULT_SCHEMA)),
        ]))
    }

    #[must_use]
    pub fn human(&self) -> String {
        let child = match self.child_exit {
            ChildExit::Code(code) => code.to_string(),
            ChildExit::Signal => "signal".into(),
        };
        format!(
            "session {}\ncapture {}\nchild {}\nexchanges {}\nfindings {}\n",
            self.binding.session_id,
            self.capture.as_str(),
            child,
            self.exchange_count,
            self.findings.len()
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HostError {
    Usage(&'static str),
    Precondition(&'static str),
    InstructionProbe(ProbeLoadError),
    RandomnessUnavailable,
    Custody(String),
    Spawn(String),
}

impl HostError {
    #[must_use]
    pub const fn exit(&self) -> HostExit {
        match self {
            Self::Usage(_) => HostExit::Usage,
            Self::Precondition(_) | Self::InstructionProbe(_) => HostExit::Refused,
            Self::RandomnessUnavailable | Self::Custody(_) | Self::Spawn(_) => HostExit::Unexpected,
        }
    }
}

pub fn run<R: AttemptRuntime>(
    request: &RunRequest,
    mut runtime: R,
) -> Result<HostResult, HostError> {
    request.validate()?;
    let instruction_plans =
        load_plans(&request.instruction_probes).map_err(HostError::InstructionProbe)?;
    let binding = UnsupervisedBinding::generate()?;
    let prepared = runtime.prepare(request, &binding, instruction_plans)?;
    let child = ChildEnvironment::new(
        &prepared.proxy,
        &prepared.certificate_or_bundle,
        request.trust,
    )
    .and_then(|environment| spawn_child(request, &environment));
    let closed = runtime.close();
    let mut cleanup_findings = Vec::new();
    for path in prepared.cleanup_paths {
        if let Some(finding) = remove_ephemeral_file(&path) {
            cleanup_findings.push(finding);
        }
    }
    let child_exit = child?;
    let mut closed = closed?;
    closed.findings.extend(cleanup_findings);
    Ok(HostResult {
        binding,
        child_exit,
        capture: closed.capture,
        exchange_count: closed.exchange_count,
        manifest_path: closed.manifest_path,
        findings: closed.findings,
    })
}

pub fn spawn_child(
    request: &RunRequest,
    environment: &ChildEnvironment,
) -> Result<ChildExit, HostError> {
    let mut command = Command::new(&request.program);
    command.args(&request.arguments);
    environment.apply(&mut command);
    command
        .status()
        .map(ChildExit::from)
        .map_err(|error| HostError::Spawn(error.kind().to_string()))
}

pub fn write_private_file_new(path: &Path, bytes: &[u8]) -> Result<(), HostError> {
    if path.file_name().is_none() {
        return Err(HostError::Custody("invalid-output-path".into()));
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut output = options
        .open(path)
        .map_err(|error| HostError::Custody(error.kind().to_string()))?;
    output
        .write_all(bytes)
        .and_then(|()| output.sync_all())
        .map_err(|error| HostError::Custody(error.kind().to_string()))
}

#[must_use]
pub fn remove_ephemeral_file(path: &Path) -> Option<HostFinding> {
    match fs::remove_file(path) {
        Ok(()) => None,
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => Some(HostFinding {
            code: "cleanup-residue".into(),
            detail: format!("{}: {}", display_redacted_path(path), error.kind()),
        }),
    }
}

fn validate_output_directory(path: &Path) -> Result<(), HostError> {
    if !path.is_absolute() || path.components().any(|part| part.as_os_str() == "..") {
        return Err(HostError::Precondition("output-directory-must-be-absolute"));
    }
    let metadata =
        fs::metadata(path).map_err(|_| HostError::Precondition("output-directory-unavailable"))?;
    if !metadata.is_dir() {
        return Err(HostError::Precondition("output-directory-not-directory"));
    }
    Ok(())
}

fn display_redacted_path(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .map_or_else(|| "<output>".into(), |value| format!("<output>/{value}"))
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
    use std::io::Cursor;

    use wire_witness_core::exchange::ProviderFamily;
    use wire_witness_proxy::ApplicationProtocol;

    use super::*;

    fn authority(value: &str) -> StartAuthority {
        StartAuthority {
            authority: Authority::parse(value).unwrap(),
            provider: ProviderFamily::OpenAi,
            protocols: BTreeSet::from([ApplicationProtocol::Http2]),
        }
    }

    fn request(output_directory: PathBuf) -> RunRequest {
        RunRequest {
            capture_authorities: vec![authority("api.example.com:443")],
            authentication_tunnels: vec![authority("auth.example.com:443")],
            requested_retention: RetentionMode::MetadataOnly,
            content_opt_in: false,
            output_directory,
            trust: TrustMechanism::NodeExtraCaCerts,
            instruction_probes: Vec::new(),
            program: "/usr/bin/true".into(),
            arguments: Vec::new(),
        }
    }

    #[test]
    fn session_identity_is_exactly_random_128_bits_and_unsupervised() {
        let mut bytes = Cursor::new([0_u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 255]);
        let binding = UnsupervisedBinding::from_reader(&mut bytes).unwrap();
        assert_eq!(binding.session_id, "000102030405060708090a0b0c0d0eff");
        assert_eq!(binding.session_id.len(), 32);
    }

    #[test]
    fn validation_precedes_host_resources() {
        let directory = std::env::temp_dir();
        let mut request = request(directory);
        request.capture_authorities.clear();
        assert_eq!(
            request.validate(),
            Err(HostError::Precondition("capture-authority-required"))
        );
    }

    #[test]
    fn content_requires_invocation_opt_in() {
        let mut request = request(std::env::temp_dir());
        request.requested_retention = RetentionMode::Content;
        assert_eq!(
            request.validate(),
            Err(HostError::Precondition("content-retention-requires-opt-in"))
        );
    }

    #[test]
    fn environment_is_child_only_and_loopback_only() {
        let parent_value = std::env::var_os("NODE_EXTRA_CA_CERTS");
        let environment = ChildEnvironment::new(
            &Authority::parse("127.0.0.1:38191").unwrap(),
            Path::new("/attempt/ca.pem"),
            TrustMechanism::NodeExtraCaCerts,
        )
        .unwrap();
        assert_eq!(
            environment.values().get("HTTPS_PROXY").unwrap(),
            "http://127.0.0.1:38191"
        );
        assert_eq!(
            environment.values().get("NODE_EXTRA_CA_CERTS").unwrap(),
            "/attempt/ca.pem"
        );
        assert_eq!(std::env::var_os("NODE_EXTRA_CA_CERTS"), parent_value);
        assert_eq!(
            ChildEnvironment::new(
                &Authority::parse("proxy.example.com:443").unwrap(),
                Path::new("/attempt/ca.pem"),
                TrustMechanism::NodeExtraCaCerts,
            ),
            Err(HostError::Precondition("proxy-must-be-loopback"))
        );
    }

    #[test]
    fn result_renderings_share_one_value_and_exit_vocabulary() {
        let result = HostResult {
            binding: UnsupervisedBinding {
                session_id: "00112233445566778899aabbccddeeff".into(),
            },
            child_exit: ChildExit::Code(7),
            capture: CaptureState::Complete,
            exchange_count: 2,
            manifest_path: Some("/attempt/manifest.json".into()),
            findings: Vec::new(),
        };
        assert_eq!(result.exit(), HostExit::Finding);
        let json = String::from_utf8(result.json_bytes()).unwrap();
        let human = result.human();
        for fact in ["00112233445566778899aabbccddeeff", "complete", "2"] {
            assert!(json.contains(fact));
            assert!(human.contains(fact));
        }
        assert_eq!(HostError::Usage("bad").exit(), HostExit::Usage);
        assert_eq!(HostError::Precondition("bad").exit(), HostExit::Refused);
    }

    #[test]
    fn private_files_are_mode_0600_and_never_overwritten() {
        let directory =
            std::env::temp_dir().join(format!("wire-witness-test-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("custody.bin");
        let _ = fs::remove_file(&path);
        write_private_file_new(&path, b"first").unwrap();
        assert!(write_private_file_new(&path, b"second").is_err());
        assert_eq!(fs::read(&path).unwrap(), b"first");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        assert_eq!(remove_ephemeral_file(&path), None);
        fs::remove_dir(&directory).unwrap();
    }

    struct FakeRuntime {
        directory: PathBuf,
        prepared: bool,
        expected_plans: usize,
    }

    impl AttemptRuntime for FakeRuntime {
        fn prepare(
            &mut self,
            _request: &RunRequest,
            binding: &UnsupervisedBinding,
            instruction_plans: Vec<ComparisonPlan>,
        ) -> Result<PreparedAttempt, HostError> {
            assert_eq!(binding.session_id.len(), 32);
            assert_eq!(instruction_plans.len(), self.expected_plans);
            self.prepared = true;
            let certificate = self.directory.join("ca.pem");
            write_private_file_new(&certificate, b"certificate")?;
            Ok(PreparedAttempt {
                proxy: Authority::parse("127.0.0.1:38191").unwrap(),
                certificate_or_bundle: certificate.clone(),
                cleanup_paths: vec![certificate],
            })
        }

        fn close(self) -> Result<ClosedCapture, HostError> {
            assert!(self.prepared);
            Ok(ClosedCapture {
                capture: CaptureState::Complete,
                exchange_count: 0,
                manifest_path: Some("manifest.json".into()),
                findings: Vec::new(),
            })
        }
    }

    #[test]
    fn host_orders_validation_prepare_spawn_close_and_cleanup() {
        let directory =
            std::env::temp_dir().join(format!("wire-witness-run-test-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let request = request(directory.clone());
        let result = run(
            &request,
            FakeRuntime {
                directory: directory.clone(),
                prepared: false,
                expected_plans: 0,
            },
        )
        .unwrap();
        assert_eq!(result.child_exit, ChildExit::Code(0));
        assert_eq!(result.capture, CaptureState::Complete);
        assert!(!directory.join("ca.pem").exists());
        fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn instruction_plans_are_loaded_before_runtime_prepare() {
        use wire_witness_core::exchange::sha256_hex;
        use wire_witness_core::instruction_observation::{ComponentKind, ComponentSelector};

        let directory = std::env::temp_dir().join(format!(
            "wire-witness-instruction-host-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).unwrap();
        let target_path = directory.join("instruction.txt");
        fs::write(&target_path, b"exact instruction").unwrap();
        let mut request = request(directory.clone());
        request.instruction_probes.push(InstructionProbeSource {
            probe_name: "primary".into(),
            target_path: target_path.clone(),
            target_digest: sha256_hex(b"exact instruction"),
            selector: ComponentSelector {
                provider: ProviderFamily::OpenAi,
                operation: "POST /v1/responses".into(),
                kind: ComponentKind::Instructions,
                component_index: 0,
                text_part_index: 0,
            },
        });
        let result = run(
            &request,
            FakeRuntime {
                directory: directory.clone(),
                prepared: false,
                expected_plans: 1,
            },
        )
        .unwrap();
        assert_eq!(result.child_exit, ChildExit::Code(0));
        fs::remove_file(target_path).unwrap();
        fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn exchange_sequences_begin_at_one_and_are_contiguous() {
        let mut sequence = ExchangeSequence::default();
        assert_eq!(sequence.next_sequence().unwrap(), 1);
        assert_eq!(sequence.next_sequence().unwrap(), 2);
        assert_eq!(sequence.observed(), 2);
    }

    #[test]
    fn command_surface_has_run_and_sidecar_with_safe_defaults() {
        assert_eq!(
            parse_arguments(&["sidecar".into()]).unwrap(),
            CliCommand::Sidecar
        );
        let directory = std::env::temp_dir();
        let arguments = vec![
            "run".into(),
            "--capture".into(),
            "api.example.com:443,openai,h2+sse".into(),
            "--auth-tunnel".into(),
            "auth.example.com:443,openai,http/1.1".into(),
            "--output".into(),
            directory.to_string_lossy().into_owned(),
            "--trust".into(),
            "node-extra-ca-certs".into(),
            "--json".into(),
            "--".into(),
            "/usr/bin/true".into(),
        ];
        let CliCommand::Run { request, output } = parse_arguments(&arguments).unwrap() else {
            panic!("expected run command");
        };
        assert_eq!(output, OutputFormat::Json);
        assert_eq!(request.requested_retention, RetentionMode::MetadataOnly);
        assert!(!request.content_opt_in);
        request.validate().unwrap();
    }

    #[test]
    fn command_surface_refuses_implicit_content_and_missing_program() {
        assert_eq!(
            parse_arguments(&["run".into()]),
            Err(HostError::Usage("program-separator-required"))
        );
        assert_eq!(
            parse_arguments(&["sidecar".into(), "extra".into()]),
            Err(HostError::Usage("sidecar-takes-no-options"))
        );
    }
}

// endregion
