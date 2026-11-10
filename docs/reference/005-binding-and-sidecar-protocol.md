<!-- spec-spine:begin generated id="header" -->
# `005-binding-and-sidecar-protocol` Attempt binding and stdio sidecar protocol

- Path: `specs/005-binding-and-sidecar-protocol/spec.md`
- Summary: Defines the immutable AttemptBinding supplied by Statecraft, the versioned stdio protocol for one sidecar per attempt, capture-digest closure of the bracketed effect, and statecraft/wire-exchange/v1 testimony for admission by the host rather than by the witness.
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
| `005-binding-and-sidecar-protocol#I-1` | `invariant` | A supervised sidecar repeats the supplied run id, attempt, and effect id unchanged and never infers or mints any member. | `3-1-attemptbinding` | no |
| `005-binding-and-sidecar-protocol#I-2` | `invariant` | The witness emits testimony and an evidence reference but never admits it, changes policy, or writes the Statecraft intent or outcome. | `3-4-evidence-envelope-boundary` | no |
| `005-binding-and-sidecar-protocol#R-1` | `requirement` | The terminal sidecar message carries capture completeness and digest, allowing the supervisor to close the exact bracketed effect. | `3-3-bracketed-lifetime` | no |
| `005-binding-and-sidecar-protocol#R-2` | `requirement` | Unknown JSON fields are ignored and preserved where relayed, but an unknown message type or malformed line is a protocol finding and never success. | `3-2-versioned-stdio` | no |
| `005-binding-and-sidecar-protocol#V-1` | `verification` | The sidecar protocol tests pass, its source is indexed, and both Statecraft interface pins verify. | `verification` | no |
<!-- spec-spine:end generated id="requirements" -->

<!-- spec-spine:begin generated id="ownership" -->
## Ownership

| Edge | Target spec | Unit | Planned | Resolution |
| --- | --- | --- | --- | --- |
| `establishes` | None declared. | `{"anchor":"binding-and-sidecar-protocol","file":"crates/wire-witness-cli/src/sidecar_protocol.rs","kind":"section"}` | no | `resolved` |
| `establishes` | None declared. | `{"id":"wire_witness_cli::sidecar_protocol","kind":"module"}` | no | `resolved` |

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
| `005-binding-and-sidecar-protocol#V-1` | `specs/005-binding-and-sidecar-protocol/spec.md` |

### Acceptance commands (declared, not executed)

```console
./.tooling/bin/spec-spine check --fail-on-warn
./.tooling/bin/spec-spine interface verify --spec 005 --export statecraft-cli=/Users/bart/DevWork/statecraft-cli
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
| amended by | `016-binding-verification-engine` |
| superseded by | None declared. |
| extended by | None declared. |
<!-- spec-spine:end generated id="changes" -->

<!-- spec-spine:begin generated id="inputs" -->
## Inputs

| Identity | Digest |
| --- | --- |
| `index:005-binding-and-sidecar-protocol` | `sha256:f0fcc79767ad3d2342378a21b41f72f094efdcb015dc876489df2b726ffdb0fb` |
| `registry:005-binding-and-sidecar-protocol` | `sha256:ae12eabd4da259e539456c7a6c3f047a9311fa25afb3de46159a2934a987cfea` |
| `traceability:005-binding-and-sidecar-protocol` | `sha256:37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
<!-- spec-spine:end generated id="inputs" -->

<!-- spec-spine:begin authored id="notes" -->
<!-- spec-spine:end authored id="notes" -->
