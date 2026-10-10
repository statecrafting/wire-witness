//! The `wire-witness` executable.
//!
//! Governed by spec 013, live loopback capture runtime. Parsing, the host
//! sequence, rendering, and the exit vocabulary are spec 006's; this file
//! only connects them to the live runtime.

// region: wire-witness-executable

use std::io::{self, Write};
use std::process::ExitCode;

use wire_witness_cli::live_runtime::LiveRuntime;
use wire_witness_cli::standalone_host::{
    CliCommand, HostError, OutputFormat, parse_arguments, run,
};

const USAGE: &str = "usage: wire-witness run --capture HOST:PORT,PROVIDER,PROTOCOLS [--auth-tunnel HOST:PORT,PROVIDER,PROTOCOLS]... --output DIR --trust node-extra-ca-certs|aws-ca-bundle|existing-roots-bundle [--retention metadata-only|content|disabled] [--json] -- PROGRAM [ARGS...]\n       wire-witness sidecar";

fn main() -> ExitCode {
    let mut arguments = Vec::new();
    for argument in std::env::args_os().skip(1) {
        match argument.into_string() {
            Ok(argument) => arguments.push(argument),
            Err(_) => return refuse(&HostError::Usage("argument-not-utf8")),
        }
    }
    let command = match parse_arguments(&arguments) {
        Ok(command) => command,
        Err(error) => return refuse(&error),
    };
    match command {
        CliCommand::Sidecar => refuse(&HostError::Precondition("sidecar-host-not-implemented")),
        CliCommand::Run { request, output } => match run(&request, LiveRuntime::system()) {
            Ok(result) => {
                let rendered = match output {
                    OutputFormat::Json => {
                        let mut bytes = result.json_bytes();
                        bytes.push(b'\n');
                        bytes
                    }
                    OutputFormat::Human => result.human().into_bytes(),
                };
                let mut stdout = io::stdout().lock();
                if stdout
                    .write_all(&rendered)
                    .and_then(|()| stdout.flush())
                    .is_err()
                {
                    return ExitCode::from(4);
                }
                ExitCode::from(result.exit().code())
            }
            Err(error) => refuse(&error),
        },
    }
}

fn refuse(error: &HostError) -> ExitCode {
    let description = match error {
        HostError::Usage(code) | HostError::Precondition(code) => (*code).to_owned(),
        HostError::InstructionProbe(_) => "instruction-probe-refused".to_owned(),
        HostError::RandomnessUnavailable => "randomness-unavailable".to_owned(),
        HostError::Custody(code) => format!("custody-failed: {code}"),
        HostError::Spawn(kind) => format!("child-spawn-failed: {kind}"),
    };
    let mut stderr = io::stderr().lock();
    let _ = writeln!(stderr, "wire-witness: {description}");
    if matches!(error, HostError::Usage(_)) {
        let _ = writeln!(stderr, "{USAGE}");
    }
    ExitCode::from(error.exit().code())
}

// endregion
