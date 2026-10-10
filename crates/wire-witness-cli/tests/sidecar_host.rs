//! Acceptance for the `wire-witness sidecar` host: the sidecar-1
//! compatibility fixtures against the built executable, the section 4
//! negative cases, and a supervised capture against a local fake provider.
//!
//! Governed by spec 014, the wire-witness.sidecar/1 stdio host. Set
//! `WIRE_WITNESS_EXECUTABLE` to run the executable cases against another
//! build of the same version, as a qualification record does.

// region: sidecar-host-acceptance

use std::collections::BTreeSet;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName};
use wire_witness_cli::producer_identity::ProducerIdentity;
use wire_witness_cli::sidecar_host::{HostOptions, SidecarExit, run_session};
use wire_witness_cli::sidecar_protocol::{SidecarSession, parse_line};
use wire_witness_core::exchange::{JsonValue, parse_json, sha256_hex};
use wire_witness_proxy::live::ProviderTrust;

const SECRET: &str = "sk-ant-api03-SIDECAR-NEVER-PERSISTS";
const SERVED: &str = "claude-served-by-fake-provider";
const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/sidecar-1");

fn executable() -> PathBuf {
    std::env::var_os("WIRE_WITNESS_EXECUTABLE").map_or_else(
        || PathBuf::from(env!("CARGO_BIN_EXE_wire-witness")),
        PathBuf::from,
    )
}

/// A fresh private parent directory; the attempt directory is a new name
/// under it.
fn private_parent(name: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let parent = std::env::temp_dir().join(format!(
        "wire-witness-sidecar-{name}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&parent);
    fs::create_dir_all(&parent).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
    }
    parent
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn fill(template: &str, output: &Path, port: u16) -> String {
    template
        .replace("@OUTPUT@", output.to_str().unwrap())
        .replace("@BUSY_PORT@", &port.to_string())
        .replace("@PORT@", &port.to_string())
}

fn members(name: &str) -> BTreeSet<String> {
    fs::read_to_string(
        Path::new(FIXTURES)
            .join("members")
            .join(format!("{name}.txt")),
    )
    .unwrap()
    .lines()
    .map(str::to_owned)
    .collect()
}

fn object(value: &JsonValue) -> &std::collections::BTreeMap<String, JsonValue> {
    let JsonValue::Object(fields) = value else {
        panic!("object expected, found {value:?}");
    };
    fields
}

fn field<'a>(value: &'a JsonValue, name: &str) -> &'a JsonValue {
    object(value)
        .get(name)
        .unwrap_or_else(|| panic!("member {name} missing"))
}

fn text<'a>(value: &'a JsonValue, name: &str) -> &'a str {
    match field(value, name) {
        JsonValue::String(text) => text,
        other => panic!("{name} is not a string: {other:?}"),
    }
}

fn assert_members(value: &JsonValue, name: &str) {
    let present: BTreeSet<String> = object(value).keys().cloned().collect();
    let required = members(name);
    assert!(
        required.is_subset(&present),
        "{name} lacks {:?}",
        required.difference(&present).collect::<Vec<_>>()
    );
}

/// Checks a whole stdout transcript the way a supervisor would: spec 005's
/// parser and session order, the start binding on every line, the fixture
/// member sets, and one terminal message.
fn check_transcript(start: &str, stdout: &[u8]) -> Vec<JsonValue> {
    let start = parse_line(start.as_bytes(), 65_536).unwrap();
    let (mut session, _) = SidecarSession::from_start(&start).unwrap();
    let mut messages = Vec::new();
    for line in stdout.split_inclusive(|byte| *byte == b'\n') {
        let parsed = parse_line(line, 65_536).expect("every stdout line is a protocol message");
        session
            .observe_sidecar(&parsed)
            .expect("protocol order and binding");
        let value = parse_json(&line[..line.len() - 1]).unwrap();
        let kind = text(&value, "type").to_owned();
        assert_members(&value, &kind);
        if matches!(kind.as_str(), "ready" | "finished") {
            assert_members(field(&value, "producer"), "producer");
        }
        messages.push(value);
    }
    assert!(
        session.is_finished(),
        "exactly one finished ends the session"
    );
    messages
}

fn files_under(directory: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = fs::read_dir(directory) else {
        return files;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(files_under(&path));
        } else {
            files.push(path);
        }
    }
    files.sort();
    files
}

struct Run {
    code: i32,
    stdout: Vec<u8>,
    stderr: String,
}

/// Runs the executable with `input` on stdin, then closes stdin.
fn sidecar(arguments: &[&str], input: &str) -> Run {
    let mut child = Command::new(executable())
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(input.as_bytes()).unwrap();
    drop(stdin);
    let output = child.wait_with_output().unwrap();
    Run {
        code: output.status.code().unwrap(),
        stdout: output.stdout,
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// Spawns the executable and returns it with its `ready` message, stdin
/// still open.
fn spawn_ready(start: &str) -> (Child, BufReader<ChildStdout>, JsonValue) {
    let mut child = Command::new(executable())
        .arg("sidecar")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(start.as_bytes())
        .unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    stdout.read_line(&mut line).unwrap();
    let ready = parse_json(line.trim_end().as_bytes()).unwrap();
    assert_eq!(text(&ready, "type"), "ready");
    (child, stdout, ready)
}

fn fixture(path: &str) -> String {
    fs::read_to_string(Path::new(FIXTURES).join(path)).unwrap()
}

#[test]
fn identity_names_the_exact_executable_and_versions() {
    let run = sidecar(&["identity", "--json"], "");
    assert_eq!(run.code, 0);
    let identity = parse_json(run.stdout.trim_ascii_end()).unwrap();
    assert_eq!(text(&identity, "interface"), "wire-witness.sidecar/1");
    assert_eq!(text(&identity, "name"), "wire-witness");
    assert_eq!(
        field(&identity, "attemptBindingSchema"),
        &JsonValue::Number("1".into())
    );
    assert_eq!(
        field(&identity, "terminalSchema"),
        &JsonValue::Number("1".into())
    );
    assert_eq!(
        text(&identity, "executableDigest"),
        format!("sha256:{}", sha256_hex(&fs::read(executable()).unwrap()))
    );
    assert!(["release", "development"].contains(&text(&identity, "releaseChannel")));
    assert_eq!(
        String::from_utf8(run.stdout).unwrap().trim_end(),
        r#"{"applicationProtocols":["http/1.1"],"attemptBindingSchema":1,"evidence":[{"schemaVersion":1,"type":"statecraft/wire-exchange/v1"}],"executableDigest":"DIGEST","interface":"wire-witness.sidecar/1","name":"wire-witness","releaseChannel":"CHANNEL","terminalSchema":1,"version":"VERSION"}"#
            .replace("DIGEST", text(&identity, "executableDigest"))
            .replace("CHANNEL", text(&identity, "releaseChannel"))
            .replace("VERSION", text(&identity, "version"))
    );
    assert_eq!(sidecar(&["identity", "--human"], "").code, 3);
}

#[test]
fn sidecar_arguments_are_a_usage_error_with_no_stdout() {
    let run = sidecar(&["sidecar", "extra"], "");
    assert_eq!(run.code, 3);
    assert!(run.stdout.is_empty());
}

#[test]
fn fixture_refusals_write_nothing_and_create_nothing() {
    let mut seen = 0;
    for entry in fs::read_dir(Path::new(FIXTURES).join("refused")).unwrap() {
        let path = entry.unwrap().path();
        let code = path.file_stem().unwrap().to_str().unwrap().to_owned();
        let parent = private_parent("refused");
        let output = parent.join("attempt");
        let busy = TcpListener::bind("127.0.0.1:0").unwrap();
        let input = fill(
            &fs::read_to_string(&path).unwrap(),
            &output,
            busy.local_addr().unwrap().port(),
        );
        let run = sidecar(&["sidecar"], &input);
        drop(busy);
        assert_eq!(run.code, 2, "{code}: {}", run.stderr);
        assert!(run.stdout.is_empty(), "{code} wrote stdout");
        assert_eq!(run.stderr.trim_end(), format!("wire-witness: {code}"));
        assert_eq!(
            fs::read_dir(&parent).unwrap().count(),
            0,
            "{code} left files behind"
        );
        fs::remove_dir_all(parent).unwrap();
        seen += 1;
    }
    assert_eq!(seen, 9);
}

#[test]
fn oversized_binding_and_existing_directory_are_refused() {
    let parent = private_parent("bounds");
    let output = parent.join("attempt");
    let oversized = fill(&fixture("start/valid.jsonl"), &output, 0)
        .replace("run-01JFIXTURE", &"r".repeat(5000));
    let run = sidecar(&["sidecar"], &oversized);
    assert_eq!(
        (run.code, run.stderr.trim_end()),
        (2, "wire-witness: binding-invalid")
    );

    fs::create_dir(&output).unwrap();
    let run = sidecar(
        &["sidecar"],
        &fill(&fixture("start/valid.jsonl"), &output, 0),
    );
    assert_eq!(
        (run.code, run.stderr.trim_end()),
        (2, "wire-witness: output-directory-invalid")
    );
    assert_eq!(files_under(&output).len(), 0);
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn fixture_starts_reach_ready_and_one_finished() {
    for name in [
        "valid",
        "valid-listen-endpoint",
        "valid-unknown-members",
        "valid-content-retention",
        "valid-disabled-retention",
    ] {
        let parent = private_parent(name);
        let output = parent.join("attempt");
        let port = free_port();
        let start = fill(&fixture(&format!("start/{name}.jsonl")), &output, port);
        let run = sidecar(&["sidecar"], &start);
        assert_eq!(run.code, 0, "{name}: {}", run.stderr);
        let messages = check_transcript(&start, &run.stdout);
        let ready = &messages[0];
        let finished = messages.last().unwrap();
        assert_eq!(text(ready, "memory_lock"), "not-attempted");
        assert_eq!(
            text(ready, "certificate_path"),
            format!("{}/attempt-ca.pem", output.display())
        );
        assert_eq!(text(finished, "end_disposition"), "supervisor-closed");
        assert_eq!(text(finished, "completeness"), "absent");
        assert_eq!(field(finished, "capture_digest"), &JsonValue::Null);
        assert_eq!(field(finished, "manifest_path"), &JsonValue::Null);
        assert_eq!(field(finished, "evidence_reference"), &JsonValue::Null);
        assert!(!output.join("wire-exchange.json").exists());
        assert!(output.join("manifest.json").exists());
        match name {
            "valid-listen-endpoint" => {
                assert_eq!(text(ready, "endpoint"), format!("127.0.0.1:{port}"));
            }
            "valid-content-retention" => {
                assert_eq!(text(ready, "effective_retention"), "metadata-only");
                assert!(messages.iter().any(|message| {
                    text(message, "type") == "finding"
                        && text(message, "code") == "content-retention-unavailable"
                }));
            }
            "valid-disabled-retention" => {
                assert_eq!(text(finished, "reason"), "disabled");
            }
            _ => assert_eq!(text(finished, "reason"), "no-exchange-observed"),
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&output).unwrap().permissions().mode() & 0o777,
                0o700
            );
            assert_eq!(
                fs::metadata(output.join("attempt-ca.pem"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
        fs::remove_dir_all(parent).unwrap();
    }
}

#[test]
fn a_second_supervisor_line_closes_with_a_gap() {
    let parent = private_parent("violation");
    let output = parent.join("attempt");
    let start = fill(&fixture("start/valid.jsonl"), &output, 0);
    let run = sidecar(&["sidecar"], &format!("{start}{start}"));
    assert_eq!(run.code, 0);
    let messages = check_transcript(&start, &run.stdout);
    let finished = messages.last().unwrap();
    assert_eq!(
        text(finished, "end_disposition"),
        "supervisor-protocol-violation"
    );
    assert_eq!(
        field(finished, "gaps"),
        &JsonValue::Array(vec![JsonValue::String(
            "unexpected-supervisor-message".into()
        )])
    );
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn a_closed_stdout_exits_4_without_finished_and_keeps_custody() {
    let parent = private_parent("stdout");
    let output = parent.join("attempt");
    let start = fill(&fixture("start/valid.jsonl"), &output, 0);
    let (mut child, stdout, _) = spawn_ready(&start);
    drop(stdout);
    drop(child.stdin.take());
    let status = child.wait().unwrap();
    assert_eq!(status.code(), Some(4));
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    assert!(stderr.contains("stdout-broken"));
    assert!(output.join("manifest.json").exists());
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn the_deadline_closes_custody() {
    let parent = private_parent("deadline");
    let output = parent.join("attempt");
    let start = fill(&fixture("start/valid.jsonl"), &output, 0)
        .replace("\"total_seconds\":600", "\"total_seconds\":1");
    let (mut child, mut stdout, _) = spawn_ready(&start);
    let mut rest = String::new();
    stdout.read_to_string(&mut rest).unwrap();
    assert_eq!(child.wait().unwrap().code(), Some(0));
    let finished = parse_json(rest.lines().last().unwrap().as_bytes()).unwrap();
    assert_eq!(text(&finished, "end_disposition"), "deadline");
    assert_eq!(
        field(&finished, "gaps"),
        &JsonValue::Array(vec![JsonValue::String("deadline-reached".into())])
    );
    drop(child.stdin.take());
    fs::remove_dir_all(parent).unwrap();
}

// The supervised capture runs the library host with an exact provider root
// set; the executable trusts only system roots, which a local fake provider
// cannot use.

struct FakeProvider {
    port: u16,
    root: CertificateDer<'static>,
}

fn crypto() -> Arc<rustls::crypto::CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

/// A TLS origin for `localhost`. A request whose body names `no-usage` is
/// answered without usage.
fn fake_provider() -> FakeProvider {
    let ca_key = rcgen::KeyPair::generate().unwrap();
    let mut ca_params = rcgen::CertificateParams::default();
    ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    ca_params.key_usages = vec![
        rcgen::KeyUsagePurpose::KeyCertSign,
        rcgen::KeyUsagePurpose::DigitalSignature,
    ];
    let ca = ca_params.self_signed(&ca_key).unwrap();
    let leaf_key = rcgen::KeyPair::generate().unwrap();
    let leaf = rcgen::CertificateParams::new(vec!["localhost".to_owned()])
        .unwrap()
        .signed_by(&leaf_key, &ca, &ca_key)
        .unwrap();
    let config = Arc::new(
        rustls::ServerConfig::builder_with_provider(crypto())
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(
                vec![leaf.der().clone()],
                PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(leaf_key.serialize_der())),
            )
            .unwrap(),
    );
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { return };
            let config = Arc::clone(&config);
            thread::spawn(move || {
                let connection = rustls::ServerConnection::new(config).unwrap();
                let mut tls = BufReader::new(rustls::StreamOwned::new(connection, stream));
                while let Some(body) = read_request(&mut tls) {
                    let usage = if body.contains("no-usage") {
                        String::new()
                    } else {
                        ",\"usage\":{\"input_tokens\":3,\"output_tokens\":5}".to_owned()
                    };
                    let body = format!("{{\"type\":\"message\",\"model\":\"{SERVED}\"{usage}}}");
                    let stream = tls.get_mut();
                    write!(
                        stream,
                        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\nset-cookie: session={SECRET}\r\ncontent-length: {}\r\n\r\n{body}",
                        body.len()
                    )
                    .unwrap();
                    stream.flush().unwrap();
                }
            });
        }
    });
    FakeProvider {
        port,
        root: ca.der().clone(),
    }
}

fn read_request<R: BufRead>(reader: &mut R) -> Option<String> {
    let mut length = 0;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            return None;
        }
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            length = value.trim().parse().ok()?;
        }
        if line == "\r\n" {
            break;
        }
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body).ok()?;
    Some(String::from_utf8_lossy(&body).into_owned())
}

struct ChannelReader {
    receiver: Receiver<Vec<u8>>,
    pending: Vec<u8>,
}

impl Read for ChannelReader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if self.pending.is_empty() {
            match self.receiver.recv() {
                Ok(bytes) => self.pending = bytes,
                Err(_) => return Ok(0),
            }
        }
        let count = buffer.len().min(self.pending.len());
        buffer[..count].copy_from_slice(&self.pending[..count]);
        self.pending.drain(..count);
        Ok(count)
    }
}

struct ChannelWriter(Sender<Vec<u8>>);

impl Write for ChannelWriter {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.0
            .send(buffer.to_vec())
            .map_err(|_| std::io::Error::from(std::io::ErrorKind::BrokenPipe))?;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// A supervised session on its own thread: stdin and stdout are channels
/// the test drives.
struct LibrarySession {
    stdin: Option<Sender<Vec<u8>>>,
    stdout: Receiver<Vec<u8>>,
    transcript: Vec<u8>,
    host: thread::JoinHandle<(SidecarExit, String)>,
}

impl LibrarySession {
    fn start(start: &str, trust: ProviderTrust) -> Self {
        let (stdin, input) = mpsc::channel();
        let (output, stdout) = mpsc::channel();
        stdin.send(start.as_bytes().to_vec()).unwrap();
        let host = thread::spawn(move || {
            let mut diagnostics = Vec::new();
            let exit = run_session(
                ChannelReader {
                    receiver: input,
                    pending: Vec::new(),
                },
                &mut ChannelWriter(output),
                &mut diagnostics,
                &HostOptions {
                    trust,
                    grace: Duration::from_millis(500),
                    poll: Duration::from_millis(10),
                },
                &ProducerIdentity::current(),
            );
            (exit, String::from_utf8(diagnostics).unwrap())
        });
        Self {
            stdin: Some(stdin),
            stdout,
            transcript: Vec::new(),
            host,
        }
    }

    fn next_line(&mut self) -> JsonValue {
        while !self.transcript.contains(&b'\n') {
            let chunk = self.stdout.recv_timeout(Duration::from_secs(10)).unwrap();
            self.transcript.extend(chunk);
        }
        let end = self
            .transcript
            .iter()
            .position(|byte| *byte == b'\n')
            .unwrap();
        parse_json(&self.transcript[..end]).unwrap()
    }

    fn finish(mut self) -> (SidecarExit, Vec<u8>) {
        drop(self.stdin.take());
        let (exit, diagnostics) = self.host.join().unwrap();
        assert!(!diagnostics.contains(SECRET));
        while let Ok(chunk) = self.stdout.try_recv() {
            self.transcript.extend(chunk);
        }
        (exit, self.transcript)
    }
}

fn client(
    proxy: &str,
    provider_port: u16,
    ca_path: &str,
) -> BufReader<rustls::StreamOwned<rustls::ClientConnection, TcpStream>> {
    let mut socket = TcpStream::connect(proxy).unwrap();
    let authority = format!("localhost:{provider_port}");
    write!(
        socket,
        "CONNECT {authority} HTTP/1.1\r\nHost: {authority}\r\n\r\n"
    )
    .unwrap();
    let mut response = Vec::new();
    let mut byte = [0_u8; 1];
    while !response.ends_with(b"\r\n\r\n") {
        socket.read_exact(&mut byte).unwrap();
        response.push(byte[0]);
    }
    assert!(response.starts_with(b"HTTP/1.1 200"));
    let mut roots = rustls::RootCertStore::empty();
    roots
        .add(CertificateDer::from_pem_file(ca_path).unwrap())
        .unwrap();
    let config = rustls::ClientConfig::builder_with_provider(crypto())
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let connection =
        rustls::ClientConnection::new(Arc::new(config), ServerName::try_from("localhost").unwrap())
            .unwrap();
    BufReader::new(rustls::StreamOwned::new(connection, socket))
}

fn post(
    tls: &mut BufReader<rustls::StreamOwned<rustls::ClientConnection, TcpStream>>,
    provider_port: u16,
    body: &str,
) {
    write!(
        tls.get_mut(),
        "POST /v1/messages?key={SECRET} HTTP/1.1\r\nHost: localhost:{provider_port}\r\nx-api-key: {SECRET}\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{body}",
        body.len()
    )
    .unwrap();
    tls.get_mut().flush().unwrap();
    let mut length = 0;
    loop {
        let mut line = String::new();
        tls.read_line(&mut line).unwrap();
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            length = value.trim().parse().unwrap();
        }
        if line == "\r\n" {
            break;
        }
    }
    let mut response = vec![0; length];
    tls.read_exact(&mut response).unwrap();
}

fn supervised_start(output: &Path, provider_port: u16, total_seconds: u64) -> String {
    fill(&fixture("start/valid.jsonl"), output, 0)
        .replace(
            "api.anthropic.com:443",
            &format!("localhost:{provider_port}"),
        )
        .replace(
            "\"total_seconds\":600",
            &format!("\"total_seconds\":{total_seconds}"),
        )
}

#[test]
fn supervised_capture_produces_exact_evidence() {
    let provider = fake_provider();
    let parent = private_parent("capture");
    let output = parent.join("attempt");
    let start = supervised_start(&output, provider.port, 600);
    let mut session = LibrarySession::start(&start, ProviderTrust::Roots(vec![provider.root]));
    let ready = session.next_line();
    let mut tls = client(
        text(&ready, "endpoint"),
        provider.port,
        text(&ready, "certificate_path"),
    );
    post(
        &mut tls,
        provider.port,
        "{\"model\":\"claude-requested\",\"max_tokens\":8}",
    );
    post(
        &mut tls,
        provider.port,
        "{\"model\":\"claude-requested\",\"max_tokens\":8,\"metadata\":\"no-usage\"}",
    );
    drop(tls);
    thread::sleep(Duration::from_millis(200));
    let (exit, transcript) = session.finish();
    assert_eq!(exit, SidecarExit::Finished);

    let messages = check_transcript(&start, &transcript);
    let exchanges: Vec<&JsonValue> = messages
        .iter()
        .filter(|message| text(message, "type") == "exchange")
        .collect();
    assert_eq!(exchanges.len(), 2);
    let finished = messages.last().unwrap();
    assert_eq!(text(finished, "completeness"), "complete", "{finished:?}");
    assert_eq!(field(finished, "reason"), &JsonValue::Null);
    assert_eq!(field(finished, "gaps"), &JsonValue::Array(Vec::new()));
    assert_eq!(
        field(finished, "exchange_count"),
        &JsonValue::Number("2".into())
    );

    let manifest_path = output.join("wire-exchange.json");
    assert_eq!(
        text(finished, "manifest_path"),
        manifest_path.to_str().unwrap()
    );
    let bytes = fs::read(&manifest_path).unwrap();
    assert_eq!(
        text(finished, "capture_digest"),
        format!("sha256:{}", sha256_hex(&bytes))
    );
    let reference = field(finished, "evidence_reference");
    assert_eq!(text(reference, "sha256"), sha256_hex(&bytes));
    assert_eq!(
        text(reference, "evidence_type"),
        "statecraft/wire-exchange/v1"
    );
    assert_eq!(field(reference, "subject"), field(finished, "binding"));

    let manifest = parse_json(&bytes).unwrap();
    assert_eq!(
        text(&manifest, "artifactType"),
        "statecraft/wire-exchange/v1"
    );
    assert_eq!(
        String::from_utf8(wire_witness_core::exchange::canonical_json_bytes(field(
            &manifest, "subject"
        )))
        .unwrap(),
        r#"{"attempt":3,"runId":"run-01JFIXTURE","sidecarEffectId":"effect-01JFIXTURE"}"#
    );
    let JsonValue::Array(recorded) = field(&manifest, "exchanges") else {
        panic!("exchanges is an array");
    };
    for (notice, entry) in exchanges.iter().zip(recorded) {
        assert_eq!(field(notice, "record_digest"), field(entry, "recordDigest"));
        assert_eq!(
            text(field(entry, "requested"), "destination"),
            format!("localhost:{}", provider.port)
        );
        assert_eq!(
            text(field(field(entry, "requested"), "model"), "value"),
            "claude-requested"
        );
        assert_eq!(
            text(field(field(entry, "served"), "model"), "value"),
            SERVED
        );
        let record = fs::read(output.join("exchanges").join(format!(
            "exchange-{:020}.json",
            text_number(entry, "sequence")
        )))
        .unwrap();
        assert_eq!(
            text(entry, "recordDigest"),
            format!("sha256:{}", sha256_hex(&record))
        );
    }
    assert_eq!(
        text(field(&recorded[0], "usage"), "label"),
        "provider-testimony"
    );
    assert_eq!(
        field(&recorded[1], "usage"),
        &parse_json(br#"{"label":"absent"}"#).unwrap()
    );
    let classes = field(field(&manifest, "redaction"), "fieldClasses");
    let JsonValue::Array(classes) = classes else {
        panic!("fieldClasses is an array");
    };
    assert!(classes.contains(&JsonValue::String("sensitive-header".into())));
    assert!(classes.contains(&JsonValue::String("query-string".into())));
    assert_eq!(text(field(&manifest, "redaction"), "result"), "applied");

    assert!(!String::from_utf8_lossy(&transcript).contains(SECRET));
    for file in files_under(&output) {
        let contents = fs::read(&file).unwrap();
        assert!(
            !String::from_utf8_lossy(&contents).contains(SECRET),
            "{} holds a credential",
            file.display()
        );
        assert!(!String::from_utf8_lossy(&contents).contains("PRIVATE KEY"));
    }
    fs::remove_dir_all(parent).unwrap();
}

fn text_number(value: &JsonValue, name: &str) -> u64 {
    match field(value, name) {
        JsonValue::Number(number) => number.parse().unwrap(),
        other => panic!("{name} is not a number: {other:?}"),
    }
}

#[test]
fn a_deadline_with_the_child_connected_is_incomplete() {
    let provider = fake_provider();
    let parent = private_parent("deadline-connected");
    let output = parent.join("attempt");
    let start = supervised_start(&output, provider.port, 2);
    let mut session = LibrarySession::start(&start, ProviderTrust::Roots(vec![provider.root]));
    let ready = session.next_line();
    let mut tls = client(
        text(&ready, "endpoint"),
        provider.port,
        text(&ready, "certificate_path"),
    );
    post(&mut tls, provider.port, "{\"model\":\"claude-requested\"}");
    // The connection stays open through the deadline.
    let (exit, transcript) = {
        let stdin = session.stdin.take();
        let result = session.host.join().unwrap();
        drop(stdin);
        let mut transcript = session.transcript;
        while let Ok(chunk) = session.stdout.try_recv() {
            transcript.extend(chunk);
        }
        (result.0, transcript)
    };
    drop(tls);
    assert_eq!(exit, SidecarExit::Finished);
    let messages = check_transcript(&start, &transcript);
    let finished = messages.last().unwrap();
    assert_eq!(text(finished, "end_disposition"), "deadline");
    assert_eq!(text(finished, "completeness"), "incomplete");
    let JsonValue::Array(gaps) = field(finished, "gaps") else {
        panic!("gaps is an array");
    };
    assert!(gaps.contains(&JsonValue::String("deadline-reached".into())));
    assert!(gaps.contains(&JsonValue::String("connections-open-at-close".into())));
    let manifest = parse_json(&fs::read(output.join("wire-exchange.json")).unwrap()).unwrap();
    assert_eq!(
        text(field(&manifest, "completeness"), "state"),
        "incomplete"
    );
    assert_eq!(
        text(field(&manifest, "capture"), "endDisposition"),
        "deadline"
    );
    fs::remove_dir_all(parent).unwrap();
}

// endregion
