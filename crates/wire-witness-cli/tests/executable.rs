//! The `wire-witness` executable's refusal paths and exit vocabulary.
//!
//! Governed by spec 013, live loopback capture runtime; spec 014 replaced the
//! sidecar refusal with its host.

// region: executable-acceptance

use std::process::Command;

fn witness(arguments: &[&str]) -> (i32, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_wire-witness"))
        .args(arguments)
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    (
        output.status.code().unwrap(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn no_command_is_a_usage_error() {
    let (code, stderr) = witness(&[]);
    assert_eq!(code, 3);
    assert!(stderr.contains("command-required"));
    assert!(stderr.contains("usage: wire-witness run"));
}

#[test]
fn run_without_a_capture_authority_refuses_before_spawn() {
    let directory = std::env::temp_dir();
    let marker = directory.join(format!("wire-witness-not-spawned-{}", std::process::id()));
    let (code, stderr) = witness(&[
        "run",
        "--output",
        directory.to_str().unwrap(),
        "--trust",
        "node-extra-ca-certs",
        "--",
        "touch",
        marker.to_str().unwrap(),
    ]);
    assert_eq!(code, 2);
    assert!(stderr.contains("capture-authority-required"));
    assert!(!marker.exists());
}

#[test]
fn sidecar_without_a_start_message_is_refused_before_ready() {
    let (code, stderr) = witness(&["sidecar"]);
    assert_eq!(code, 2);
    assert!(stderr.contains("start-missing"));
}

// endregion
