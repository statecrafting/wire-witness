<!-- spec-spine:begin generated id="header" -->
# `016-binding-verification-engine` Binding protocol verification uses the repository-local engine

- Path: `specs/016-binding-verification-engine/spec.md`
- Summary: Replaces spec 005 acceptance commands with their profile 13 engine paths, preserving its approved source and pinned interface section digests.
- Reference page schema: `1.0.0`
<!-- spec-spine:end generated id="header" -->

<!-- spec-spine:begin generated id="lifecycle" -->
## Lifecycle

| Field | Value |
| --- | --- |
| status | `draft` |
| implementation | `n-a` |
| superseded_by | None declared. |
| retirement_rationale | None declared. |
| created | `2026-10-10` |
<!-- spec-spine:end generated id="lifecycle" -->

<!-- spec-spine:begin generated id="requirements" -->
## Requirements

None declared.
<!-- spec-spine:end generated id="requirements" -->

<!-- spec-spine:begin generated id="ownership" -->
## Ownership

None declared.

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

None declared.

### Acceptance commands (declared, not executed)

```console
.bin/spec-spine check --fail-on-warn
.bin/spec-spine interface verify --spec 005 --export statecraft-cli=/Users/bart/DevWork/statecraft-cli
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
| amends | `005-binding-and-sidecar-protocol` |
| amends_sections | None declared. |
| amends_verification | `005-binding-and-sidecar-protocol` |
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
| `index:016-binding-verification-engine` | `sha256:69d25ae77132fd29a9014fe326b5a68074d9897ee5a2b2dcec5a54bc80af8aa2` |
| `registry:016-binding-verification-engine` | `sha256:69d25ae77132fd29a9014fe326b5a68074d9897ee5a2b2dcec5a54bc80af8aa2` |
| `traceability:016-binding-verification-engine` | `sha256:37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
<!-- spec-spine:end generated id="inputs" -->

<!-- spec-spine:begin authored id="notes" -->
<!-- spec-spine:end authored id="notes" -->
