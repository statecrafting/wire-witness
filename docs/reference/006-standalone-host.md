<!-- spec-spine:begin generated id="header" -->
# `006-standalone-host` Standalone host and per-process environment

- Path: `specs/006-standalone-host/spec.md`
- Summary: Defines the standalone CLI, unsupervised session identity, child-only proxy and trust environment, additive JSON output, and the prohibition on global proxy, certificate, shell, browser, or operating-system trust mutation.
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
| `006-standalone-host#I-1` | `invariant` | A standalone session has an unsupervised identity and never claims a Statecraft run, attempt, or effect id. | `3-2-unsupervised-identity` | no |
| `006-standalone-host#I-2` | `invariant` | Proxy and CA trust configuration is applied only to the spawned child process and never mutates global or persistent user configuration. | `3-3-child-only-environment` | no |
| `006-standalone-host#R-1` | `requirement` | Standalone and sidecar commands host the same core and proxy contracts without reimplementing normalization or policy. | `3-1-command-surface` | no |
| `006-standalone-host#R-2` | `requirement` | Human and JSON output render the same result; JSON may gain fields but existing fields are not removed or retyped within version 1. | `3-4-results-and-exits` | no |
| `006-standalone-host#V-1` | `verification` | The standalone host tests pass, its source is indexed, and both Statecraft interface pins verify. | `verification` | no |
<!-- spec-spine:end generated id="requirements" -->

<!-- spec-spine:begin generated id="ownership" -->
## Ownership

| Edge | Target spec | Unit | Planned | Resolution |
| --- | --- | --- | --- | --- |
| `establishes` | None declared. | `{"anchor":"standalone-host","file":"crates/wire-witness-cli/src/standalone_host.rs","kind":"section"}` | no | `resolved` |
| `establishes` | None declared. | `{"id":"wire-witness-cli","kind":"crate"}` | no | `resolved` |

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
| `006-standalone-host#V-1` | `specs/006-standalone-host/spec.md` |

### Acceptance commands (declared, not executed)

```console
./.tooling/bin/spec-spine check --fail-on-warn
./.tooling/bin/spec-spine interface verify --spec 006 --export statecraft-cli=/Users/bart/DevWork/statecraft-cli
cargo test -p wire-witness-cli --locked
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
| extended by | `008-instruction-delivery-observation`, `010-policy-bounded-corpus-export`, `013-live-loopback-capture-runtime` |
<!-- spec-spine:end generated id="changes" -->

<!-- spec-spine:begin generated id="inputs" -->
## Inputs

| Identity | Digest |
| --- | --- |
| `index:006-standalone-host` | `sha256:be796dd08dec2eb9a5f2a388d5fff68e4e6aafafa049ca7bc121416b2320473e` |
| `registry:006-standalone-host` | `sha256:c9dcf12b2411f13ee69901bfb19a2b7b91a989b7080f9874f228a9820a4af19b` |
| `traceability:006-standalone-host` | `sha256:37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
<!-- spec-spine:end generated id="inputs" -->

<!-- spec-spine:begin authored id="notes" -->
<!-- spec-spine:end authored id="notes" -->
