<!-- spec-spine:begin generated id="header" -->
# `013-live-loopback-capture-runtime` Live loopback capture runtime and the wire-witness executable

- Path: `specs/013-live-loopback-capture-runtime/spec.md`
- Summary: Supplies the I/O shell that specs 003, 004, and 006 describe but the workspace did not contain: the wire-witness executable, a per-attempt loopback listener, a fresh in-memory CA, exact-allowlist TLS interception, an HTTP/1.1 relay that forwards verbatim and observes within bounds, redacted exchange records, and a capture manifest. It names every protocol and retention path it does not yet implement and refuses or reports them.
- Reference page schema: `1.0.0`
<!-- spec-spine:end generated id="header" -->

<!-- spec-spine:begin generated id="lifecycle" -->
## Lifecycle

| Field | Value |
| --- | --- |
| status | `approved` |
| implementation | `complete` |
| superseded_by | None declared. |
| retirement_rationale | None declared. |
| created | `2026-10-04` |
<!-- spec-spine:end generated id="lifecycle" -->

<!-- spec-spine:begin generated id="requirements" -->
## Requirements

| Obligation | Kind | Text | Anchor | Withdrawn |
| --- | --- | --- | --- | --- |
| `013-live-loopback-capture-runtime#I-1` | `invariant` | The attempt CA private key is generated after the attempt begins, held only by the running proxy, and never serialized, written, logged, or sent; only its certificate reaches a file. | `3-2-attempt-preparation-and-ephemeral-trust` | no |
| `013-live-loopback-capture-runtime#I-2` | `invariant` | Only an exact capture-allowlist authority is TLS-intercepted; every other CONNECT is an opaque tunnel and every plain request is forwarded without capture. | `3-3-listener-and-routing` | no |
| `013-live-loopback-capture-runtime#I-3` | `invariant` | No durable output carries a request or response body, a header value, a query string, or the CA key; records hold metadata, digests, counts, and normalized identities only. | `3-7-exchange-records` | no |
| `013-live-loopback-capture-runtime#R-1` | `requirement` | The HTTP/1.1 relay forwards every byte verbatim and in order, refuses ambiguous framing by closing the connection, and stops structured capture at a fixed bound while forwarding continues. | `3-5-http-1-1-relay-and-bounded-observation` | no |
| `013-live-loopback-capture-runtime#R-2` | `requirement` | The child overlay names an empty proxy bypass list, so an inherited NO_PROXY cannot route a capture authority around the witness. | `3-6-child-environment-bypass` | no |
| `013-live-loopback-capture-runtime#R-3` | `requirement` | The capture manifest is installed last, lists every recorded exchange by digest, and reports capture as absent or incomplete whenever the evidence does not support complete. | `3-8-capture-manifest-and-capture-state` | no |
| `013-live-loopback-capture-runtime#R-4` | `requirement` | The wire-witness executable maps spec 006's parse, host sequence, rendering, and exit vocabulary onto this runtime, and refuses the sidecar command until its host exists. | `3-1-the-executable` | no |
| `013-live-loopback-capture-runtime#V-1` | `verification` | The relay, decoding, record, install, and executable tests pass, the fake-provider end-to-end capture passes, and the dependency boundary holds. | `verification` | no |
<!-- spec-spine:end generated id="requirements" -->

<!-- spec-spine:begin generated id="ownership" -->
## Ownership

| Edge | Target spec | Unit | Planned | Resolution |
| --- | --- | --- | --- | --- |
| `establishes` | None declared. | `{"anchor":"executable-acceptance","file":"crates/wire-witness-cli/tests/executable.rs","kind":"section"}` | no | `resolved` |
| `establishes` | None declared. | `{"anchor":"live-attempt-runtime","file":"crates/wire-witness-cli/src/live_runtime.rs","kind":"section"}` | no | `resolved` |
| `establishes` | None declared. | `{"anchor":"live-capture-acceptance","file":"crates/wire-witness-cli/tests/live_capture.rs","kind":"section"}` | no | `resolved` |
| `establishes` | None declared. | `{"anchor":"live-capture-runtime","file":"crates/wire-witness-proxy/src/live.rs","kind":"section"}` | no | `resolved` |
| `establishes` | None declared. | `{"anchor":"wire-witness-executable","file":"crates/wire-witness-cli/src/main.rs","kind":"section"}` | no | `resolved` |
| `extends` | `004-allowlisted-capture-proxy` | `{"anchor":"dependencies","file":"crates/wire-witness-proxy/Cargo.toml","kind":"section"}` | no | `resolved` |
| `extends` | `004-allowlisted-capture-proxy` | `{"anchor":"live-capture-runtime-module-export","file":"crates/wire-witness-proxy/src/lib.rs","kind":"section"}` | no | `resolved` |
| `extends` | `006-standalone-host` | `{"anchor":"live-runtime-module-export","file":"crates/wire-witness-cli/src/lib.rs","kind":"section"}` | no | `resolved` |
| `extends` | `001-boundaries-and-authority` | `{"anchor":"workspace.dependencies","file":"Cargo.toml","kind":"section"}` | no | `resolved` |
| `extends` | `001-boundaries-and-authority` | `{"kind":"file","path":"Cargo.lock"}` | no | `resolved` |
| `extends` | `006-standalone-host` | `{"kind":"file","path":"crates/wire-witness-cli/Cargo.toml"}` | no | `resolved` |
| `extends` | `006-standalone-host` | `{"kind":"file","path":"crates/wire-witness-cli/src/standalone_host.rs"}` | no | `resolved` |
| `extends` | `001-boundaries-and-authority` | `{"kind":"file","path":"scripts/check-boundaries.sh"}` | no | `resolved` |

### References (non-owning)

None declared.
<!-- spec-spine:end generated id="ownership" -->

<!-- spec-spine:begin generated id="signatures" -->
## Signatures

None declared.
<!-- spec-spine:end generated id="signatures" -->

<!-- spec-spine:begin generated id="relations" -->
## Relations

None declared.
<!-- spec-spine:end generated id="relations" -->

<!-- spec-spine:begin generated id="operations" -->
## Operations

Not supplied.
<!-- spec-spine:end generated id="operations" -->

<!-- spec-spine:begin generated id="evidence" -->
## Evidence

### Declared inputs

| Obligation | Inputs |
| --- | --- |
| `013-live-loopback-capture-runtime#V-1` | `specs/013-live-loopback-capture-runtime/spec.md` |

### Acceptance commands (declared, not executed)

```console
.bin/spec-spine check --fail-on-warn
cargo test -p wire-witness-proxy --locked
cargo test -p wire-witness-cli --locked
scripts/check-boundaries.sh
```
<!-- spec-spine:end generated id="evidence" -->

<!-- spec-spine:begin generated id="limitations" -->
## Limitations

None declared.

This page restates declared and structurally resolved records. It does not state that code is correct, that a test ran or passed, or that anything was accepted, released, adopted, or observed.
<!-- spec-spine:end generated id="limitations" -->

<!-- spec-spine:begin generated id="changes" -->
## Changes

| Edge | Specs |
| --- | --- |
| amends | None declared. |
| amends_sections | None declared. |
| amends_verification | None declared. |
| supersedes | None declared. |
| amendment_record | None declared. |
| amended by | `014-sidecar-host` |
| superseded by | None declared. |
| extended by | `014-sidecar-host` |
<!-- spec-spine:end generated id="changes" -->

<!-- spec-spine:begin generated id="inputs" -->
## Inputs

| Identity | Digest |
| --- | --- |
| `index:013-live-loopback-capture-runtime` | `sha256:be52693ae370e5f04afbbc8e31c86a8f59d549237cd3afff215445a481965246` |
| `registry:013-live-loopback-capture-runtime` | `sha256:b0ff4cc5de6420f4a8f426c5078b3b0b113c7a9ffd7895de0e9b257b6f1e4e48` |
| `traceability:013-live-loopback-capture-runtime` | `sha256:37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
<!-- spec-spine:end generated id="inputs" -->

<!-- spec-spine:begin authored id="notes" -->
<!-- spec-spine:end authored id="notes" -->
