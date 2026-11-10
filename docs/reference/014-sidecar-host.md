<!-- spec-spine:begin generated id="header" -->
# `014-sidecar-host` The wire-witness.sidecar/1 stdio host

- Path: `specs/014-sidecar-host/spec.md`
- Summary: Implements `wire-witness sidecar` over the spec 005 protocol and the spec 013 live runtime: one supervised attempt per process, exact AttemptBinding and terminal schema versions, a statecraft/wire-exchange/v1 manifest with a fixed member set, a producer identity Statecraft can pin, and the compatibility, rollback, and qualification records a release must carry.
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
| created | `2026-10-10` |
<!-- spec-spine:end generated id="lifecycle" -->

<!-- spec-spine:begin generated id="requirements" -->
## Requirements

| Obligation | Kind | Text | Anchor | Withdrawn |
| --- | --- | --- | --- | --- |
| `014-sidecar-host#I-1` | `invariant` | The sidecar validates the supplied AttemptBinding before it creates a CA, a listener, or any file, and repeats its three values byte for byte on every stdout message and in the wire-exchange manifest. | `3-2-start-and-binding-validation` | no |
| `014-sidecar-host#I-2` | `invariant` | Stdout carries only wire-witness.sidecar/1 messages with identities, counts, codes, and digests; no header value, body, query string, credential, or CA key reaches stdout, stderr, or any file. | `3-4-notices-and-stdout-discipline` | no |
| `014-sidecar-host#I-3` | `invariant` | The sidecar never admits evidence, decides a run outcome, writes a Statecraft record, or reports success it did not observe; every unknown, absent, and incomplete state stays explicit. | `3-5-closing-custody-and-the-terminal-message` | no |
| `014-sidecar-host#R-1` | `requirement` | Exactly one ready precedes exchange and finding notices, and exactly one finished message ends a session that reached ready; a session refused before ready writes nothing to stdout and exits 2. | `3-3-ready` | no |
| `014-sidecar-host#R-2` | `requirement` | The statecraft/wire-exchange/v1 manifest contains exactly the members of section 3.6, keeps requested and served identities, provider-testimony usage, estimate state, attestation, retention, and redaction as separate facts, and is installed last. | `3-6-the-statecraft-wire-exchange-v1-manifest` | no |
| `014-sidecar-host#R-3` | `requirement` | The producer identity names the exact version, executable digest, interface wire-witness.sidecar/1, AttemptBinding schema 1, terminal schema 1, evidence schema 1, and release channel, and the same values appear in ready, finished, and the manifest. | `3-7-producer-identity-and-schema-versions` | no |
| `014-sidecar-host#R-4` | `requirement` | Every release carries the sidecar-1 compatibility fixtures, its rollback identity, and per-platform qualification records; a change that breaks a fixture requires a new interface major. | `3-8-compatibility-rollback-and-qualification` | no |
| `014-sidecar-host#V-1` | `verification` | The sidecar host, manifest, identity, and fixture tests pass, including the end-to-end supervised capture against a local fake provider and every section 4 negative case. | `verification` | no |
<!-- spec-spine:end generated id="requirements" -->

<!-- spec-spine:begin generated id="ownership" -->
## Ownership

| Edge | Target spec | Unit | Planned | Resolution |
| --- | --- | --- | --- | --- |
| `establishes` | None declared. | `{"anchor":"producer-identity","file":"crates/wire-witness-cli/src/producer_identity.rs","kind":"section"}` | no | `resolved` |
| `establishes` | None declared. | `{"anchor":"sidecar-host","file":"crates/wire-witness-cli/src/sidecar_host.rs","kind":"section"}` | no | `resolved` |
| `establishes` | None declared. | `{"anchor":"sidecar-host-acceptance","file":"crates/wire-witness-cli/tests/sidecar_host.rs","kind":"section"}` | no | `resolved` |
| `establishes` | None declared. | `{"anchor":"sidecar-host-module-export","file":"crates/wire-witness-cli/src/lib.rs","kind":"section"}` | no | `resolved` |
| `establishes` | None declared. | `{"anchor":"wire-exchange-manifest","file":"crates/wire-witness-cli/src/wire_exchange.rs","kind":"section"}` | no | `resolved` |
| `establishes` | None declared. | `{"kind":"directory","path":"crates/wire-witness-cli/tests/fixtures/sidecar-1/"}` | no | `resolved` |
| `establishes` | None declared. | `{"kind":"directory","path":"qualification/"}` | no | `resolved` |
| `extends` | `013-live-loopback-capture-runtime` | `{"anchor":"executable-acceptance","file":"crates/wire-witness-cli/tests/executable.rs","kind":"section"}` | no | `resolved` |
| `extends` | `013-live-loopback-capture-runtime` | `{"anchor":"live-attempt-runtime","file":"crates/wire-witness-cli/src/live_runtime.rs","kind":"section"}` | no | `resolved` |
| `extends` | `013-live-loopback-capture-runtime` | `{"anchor":"live-capture-runtime","file":"crates/wire-witness-proxy/src/live.rs","kind":"section"}` | no | `resolved` |
| `extends` | `013-live-loopback-capture-runtime` | `{"anchor":"wire-witness-executable","file":"crates/wire-witness-cli/src/main.rs","kind":"section"}` | no | `resolved` |

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
| `014-sidecar-host#V-1` | `specs/014-sidecar-host/spec.md` |

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
| amends | `013-live-loopback-capture-runtime` |
| amends_sections | None declared. |
| amends_verification | None declared. |
| supersedes | None declared. |
| amendment_record | None declared. |
| amended by | None declared. |
| superseded by | None declared. |
| extended by | None declared. |
<!-- spec-spine:end generated id="changes" -->

<!-- spec-spine:begin generated id="inputs" -->
## Inputs

| Identity | Digest |
| --- | --- |
| `index:014-sidecar-host` | `sha256:346eed56cbdb2f77110d80721687141e8fe47785611c35c060501afa8f9275d9` |
| `registry:014-sidecar-host` | `sha256:e92d3c73d160d992629432c6e5bcc687d76fcad5935e7490f92dcd1cef9af287` |
| `traceability:014-sidecar-host` | `sha256:37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
<!-- spec-spine:end generated id="inputs" -->

<!-- spec-spine:begin authored id="notes" -->
<!-- spec-spine:end authored id="notes" -->
