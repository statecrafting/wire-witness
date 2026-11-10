<!-- spec-spine:begin generated id="header" -->
# `003-redaction-custody-and-retention` Redaction, custody, and retention

- Path: `specs/003-redaction-custody-and-retention/spec.md`
- Summary: Defines metadata-only default retention, explicit content capture, mandatory credential removal before durable writes, per-attempt custody, 0600 outputs, and a fresh memory-only CA private key for every capture attempt.
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
| `003-redaction-custody-and-retention#I-1` | `invariant` | Authorization headers, API keys, cookies, OAuth exchange bodies, and detected credentials are removed before any durable write. | `3-2-redaction-before-custody` | no |
| `003-redaction-custody-and-retention#I-2` | `invariant` | Each attempt has a fresh CA private key held only in witness memory; only a certificate path is exposed to the child. | `3-4-ephemeral-ca-custody` | no |
| `003-redaction-custody-and-retention#R-1` | `requirement` | Metadata-only is the default and every output records requested and effective retention. | `3-1-retention-modes` | no |
| `003-redaction-custody-and-retention#R-2` | `requirement` | Every durable output is created with mode 0600 and is never made visible at its final path before redaction and complete write. | `3-3-durable-custody` | no |
| `003-redaction-custody-and-retention#V-1` | `verification` | The custody implementation tests pass, its source is indexed, and the action-gate detector pin verifies. | `verification` | no |
<!-- spec-spine:end generated id="requirements" -->

<!-- spec-spine:begin generated id="ownership" -->
## Ownership

| Edge | Target spec | Unit | Planned | Resolution |
| --- | --- | --- | --- | --- |
| `establishes` | None declared. | `{"anchor":"redaction-custody-and-retention","file":"crates/wire-witness-core/src/custody.rs","kind":"section"}` | no | `resolved` |

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
| `003-redaction-custody-and-retention#V-1` | `specs/003-redaction-custody-and-retention/spec.md` |

### Acceptance commands (declared, not executed)

```console
./.tooling/bin/spec-spine check --fail-on-warn
./.tooling/bin/spec-spine interface verify --spec 003 --export action-gate=/Users/bart/DevWork/action-gate
cargo test -p wire-witness-core --locked
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
| extended by | None declared. |
<!-- spec-spine:end generated id="changes" -->

<!-- spec-spine:begin generated id="inputs" -->
## Inputs

| Identity | Digest |
| --- | --- |
| `index:003-redaction-custody-and-retention` | `sha256:2c1ea1f9625313202740a94a87f3cecb18dfa790ec94afb975c8c0a35f86c3c2` |
| `registry:003-redaction-custody-and-retention` | `sha256:86ba7b2ee74b02726fbc8b48e7f44f8f7ceb7492365196c7622c854afff54642` |
| `traceability:003-redaction-custody-and-retention` | `sha256:37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
<!-- spec-spine:end generated id="inputs" -->

<!-- spec-spine:begin authored id="notes" -->
<!-- spec-spine:end authored id="notes" -->
