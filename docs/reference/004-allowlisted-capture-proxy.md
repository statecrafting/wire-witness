<!-- spec-spine:begin generated id="header" -->
# `004-allowlisted-capture-proxy` Allowlisted capture proxy and transparent streams

- Path: `specs/004-allowlisted-capture-proxy/spec.md`
- Summary: Defines the per-attempt proxy that intercepts only explicitly allowlisted provider hosts, tunnels authentication endpoints and every other host without inspection, and transparently handles CONNECT, TLS, HTTP/1.1, HTTP/2, SSE, and WebSocket traffic with bounded parsing and backpressure.
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
| created | `2026-09-26` |
<!-- spec-spine:end generated id="lifecycle" -->

<!-- spec-spine:begin generated id="requirements" -->
## Requirements

| Obligation | Kind | Text | Anchor | Withdrawn |
| --- | --- | --- | --- | --- |
| `004-allowlisted-capture-proxy#I-1` | `invariant` | Only an exact capture-allowlist authority is TLS-intercepted; authentication endpoints and every other authority are tunneled without inspection. | `3-1-routing-and-interception` | no |
| `004-allowlisted-capture-proxy#I-2` | `invariant` | Transport forwarding preserves ordering, framing, half-close behavior, and backpressure; capture work never creates an unbounded buffer. | `3-3-transparent-streams` | no |
| `004-allowlisted-capture-proxy#R-1` | `requirement` | CONNECT, TLS, HTTP/1.1, HTTP/2, SSE, and WebSocket parsing are owned by the sidecar proxy process, not by the trusted supervisor. | `3-2-protocol-ownership` | no |
| `004-allowlisted-capture-proxy#R-2` | `requirement` | Unknown events remain unknown and malformed input is a finding while opaque forwarding continues when transport safety permits. | `3-4-bounded-observation` | no |
| `004-allowlisted-capture-proxy#V-1` | `verification` | The proxy implementation tests pass, its source is indexed, and the Statecraft confinement pin verifies. | `verification` | no |
<!-- spec-spine:end generated id="requirements" -->

<!-- spec-spine:begin generated id="ownership" -->
## Ownership

| Edge | Target spec | Unit | Planned | Resolution |
| --- | --- | --- | --- | --- |
| `establishes` | None declared. | `{"anchor":"allowlisted-capture-proxy","file":"crates/wire-witness-proxy/src/lib.rs","kind":"section"}` | no | `resolved` |
| `establishes` | None declared. | `{"id":"wire-witness-proxy","kind":"crate"}` | no | `resolved` |

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
| `004-allowlisted-capture-proxy#V-1` | `specs/004-allowlisted-capture-proxy/spec.md` |

### Acceptance commands (declared, not executed)

```console
./.tooling/bin/spec-spine check --fail-on-warn
./.tooling/bin/spec-spine interface verify --spec 004 --export statecraft-cli=/Users/bart/DevWork/statecraft-cli
cargo test -p wire-witness-proxy --locked
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
| amended by | None declared. |
| superseded by | None declared. |
| extended by | `013-live-loopback-capture-runtime` |
<!-- spec-spine:end generated id="changes" -->

<!-- spec-spine:begin generated id="inputs" -->
## Inputs

| Identity | Digest |
| --- | --- |
| `index:004-allowlisted-capture-proxy` | `sha256:d3bed1d92f454122488b8065efd70a23fcf0b7c5cb3ced9bc66fea575409af7f` |
| `registry:004-allowlisted-capture-proxy` | `sha256:39bfd4d7607665200729a6bb8ef9dfd4a8be7eed97bd1def05555cd75216521f` |
| `traceability:004-allowlisted-capture-proxy` | `sha256:37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
<!-- spec-spine:end generated id="inputs" -->

<!-- spec-spine:begin authored id="notes" -->
<!-- spec-spine:end authored id="notes" -->
