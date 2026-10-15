# Qualification: wire-witness 0.1.0 on aarch64-unknown-linux-gnu

Spec 014 section 3.8. This record qualifies the exact executable below on this
platform only. It does not qualify any other build, platform, or Statecraft
confinement profile.

## Identity

| Fact | Value |
|---|---|
| Version | `0.1.0` |
| Interface | `wire-witness.sidecar/1` |
| AttemptBinding schema | `1` |
| Terminal schema | `1` |
| Evidence | `statecraft/wire-exchange/v1`, schema `1` |
| Release channel | `release` |
| Executable digest | `sha256:468a8bac6b6536b64aa86f78251ff35e10842d3f98ea29bd085ca9b7b70d2c08` |
| Executable size | 4,434,912 bytes, ELF 64-bit aarch64 PIE, dynamically linked against glibc (highest symbol version `GLIBC_2.34`) |
| Archive | `wire-witness-0.1.0-aarch64-unknown-linux-gnu.tar.gz` |
| Archive digest | `sha256:a6b493089e2ab5d74ac641dd2ce8fd6b03ad19d5fd7ccade2c84ad50787100f7` |
| Source commit | `4a7c603b8c26eed66955b5b814af3cd576a5a3ef` (`crates/` tree `639aa03fda162ab55668ab79da6f76df7e440377`, `Cargo.lock` blob `4306658611b058a173288fe80bce00d15e005925`) |
| Qualified on | 2026-10-15 |

`wire-witness identity --json` printed:

```json
{"applicationProtocols":["http/1.1"],"attemptBindingSchema":1,"evidence":[{"schemaVersion":1,"type":"statecraft/wire-exchange/v1"}],"executableDigest":"sha256:468a8bac6b6536b64aa86f78251ff35e10842d3f98ea29bd085ca9b7b70d2c08","interface":"wire-witness.sidecar/1","name":"wire-witness","releaseChannel":"release","terminalSchema":1,"version":"0.1.0"}
```

## Platform

| Fact | Value |
|---|---|
| Operating system | Debian GNU/Linux 12 (bookworm), glibc 2.36-9+deb12u14, kernel 7.0.14-linuxkit, `linux/arm64/v8` |
| Container | `rust:1.98.1-bookworm` platform image `sha256:5b993f23fb69746405496e76f91e511d96c82b5b23304fd5d20b4a80a8b223ea` (index `sha256:93ce27a88655056a51dbdd8f5f2d7ddc071c7b0070fb288a37b5a285fc83971e`), run as uid 1000 with the source mounted read-only |
| Host | Docker Desktop 29.8.2 on macOS 26.5.1, Apple silicon arm64; the container ran natively on the host CPU |
| Toolchain | rustc 1.98.1 (48a229cea 2026-09-01), LLVM 22.1.8; cargo 1.98.1 (797e8a9bc 2026-08-05); host `aarch64-unknown-linux-gnu` |

## Build

Inside the container, from a copy of the source at `/build/wire-witness`,
with `CARGO_HOME=/build/cargo` and `CARGO_TARGET_DIR=/build/target`:

```sh
WIRE_WITNESS_RELEASE_CHANNEL=release \
RUSTFLAGS="--remap-path-prefix=/build/wire-witness=/build/wire-witness --remap-path-prefix=$CARGO_HOME=/build/cargo" \
cargo build --release --locked -p wire-witness-cli --bin wire-witness
```

The executable was copied out of the release target directory unchanged; the
digest above is of those bytes, which are the bytes in the archive.

## Commands and results

All three ran in the same container as the build.

| # | Command | Result |
|---|---|---|
| 1 | `cargo test --workspace --locked` | Pass: exit 0, 126 passed, 0 failed, 0 ignored. |
| 2 | `WIRE_WITNESS_EXECUTABLE=<the executable above> cargo test -p wire-witness-cli --test sidecar_host --locked -- --test-threads=1` | Pass: exit 0, 10 passed, 0 failed. |
| 3 | `<the executable above> identity --json` | Pass: exit 0, output above, digest equals `sha256sum` of the file. |

Command 2 runs the `sidecar-1` compatibility fixtures against the release
executable: `identity_names_the_exact_executable_and_versions`,
`sidecar_arguments_are_a_usage_error_with_no_stdout`,
`fixture_refusals_write_nothing_and_create_nothing`,
`oversized_binding_and_existing_directory_are_refused`,
`fixture_starts_reach_ready_and_one_finished`,
`a_second_supervisor_line_closes_with_a_gap`,
`a_closed_stdout_exits_4_without_finished_and_keeps_custody`, and
`the_deadline_closes_custody` each spawn that executable. The same command
runs the supervised end-to-end capture against a local fake provider,
`supervised_capture_produces_exact_evidence` and
`a_deadline_with_the_child_connected_is_incomplete`. As spec 014 section 3.8
states, those two run the same host through the library with an exact root
set, because the executable trusts only system roots for provider TLS and a
local fake provider cannot use them.

## Residuals

- The host applies no confinement of its own (spec 014 sections 3.9 and 5).
  This record does not qualify a Statecraft confinement profile; that
  qualification is Statecraft's.
- The supervised capture against the fake provider runs through the library,
  not the executable, for the TLS-root reason above.
- `memory_lock` is `not-attempted`; the attempt CA key may be paged to swap.
- No live provider was contacted. Live-provider observation is not claimed.
- The run used Docker Desktop's Linux VM kernel, not a distribution kernel on
  bare metal or a cloud host. A host with glibc older than 2.34 cannot load
  this executable.
- The executable digest is self-reported; a supervisor compares it with its
  own adoption ledger.
