<!-- spec-spine:begin generated id="header" -->
# `010-policy-bounded-corpus-export` Policy-bounded corpus export

- Path: `specs/010-policy-bounded-corpus-export/spec.md`
- Summary: Defines an immutable, bounded export manifest for policy-permitted wire testimony, preserving binding, retention, completeness, redaction, and artifact identity without granting permission to retain, disclose, replay, train on, admit, or evaluate the captured material.
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
| created | `2026-09-27` |
<!-- spec-spine:end generated id="lifecycle" -->

<!-- spec-spine:begin generated id="requirements" -->
## Requirements

| Obligation | Kind | Text | Anchor | Withdrawn |
| --- | --- | --- | --- | --- |
| `010-policy-bounded-corpus-export#I-1` | `invariant` | An export records an external policy decision and exact source identities but never grants retention, disclosure, replay, training, evidence admission, or evaluation authority. | `3-1-export-authority` | no |
| `010-policy-bounded-corpus-export#I-2` | `invariant` | Export never resurrects erased or expired content, broadens effective retention, removes redaction findings, or represents an incomplete source set as complete. | `3-2-source-eligibility` | no |
| `010-policy-bounded-corpus-export#R-1` | `requirement` | The canonical manifest binds every selected capture, exchange, observation, summary, and included artifact by exact identity, digest, length, retention posture, completeness, and ordered findings. | `3-3-export-manifest` | no |
| `010-policy-bounded-corpus-export#R-2` | `requirement` | Digest-only, embedded, and external-reference entries are explicit and bounded, and any missing, expired, corrupt, foreign, or unauthorized input refuses export or remains an explicit gap. | `3-4-materialization-and-bounds` | no |
| `010-policy-bounded-corpus-export#R-3` | `requirement` | The only accepted policy input is a wire-witness.export-policy-receipt/1 envelope whose closed fields are validated structurally, and instruction-observation or measurement-summary entries are accepted only when the producing build implements their exact schema version. | `3-5-policy-receipt-envelope` | no |
| `010-policy-bounded-corpus-export#V-1` | `verification` | The receipt validator, bounded manifest constructor, and exclusive private atomic installer pass their closed-envelope, eligibility, integrity, gap, limit, expiry, and custody tests. | `verification` | no |
<!-- spec-spine:end generated id="requirements" -->

<!-- spec-spine:begin generated id="ownership" -->
## Ownership

| Edge | Target spec | Unit | Planned | Resolution |
| --- | --- | --- | --- | --- |
| `establishes` | None declared. | `{"anchor":"policy-bounded-corpus-export","file":"crates/wire-witness-core/src/corpus_export.rs","kind":"section"}` | no | `resolved` |
| `establishes` | None declared. | `{"anchor":"policy-bounded-corpus-export-host","file":"crates/wire-witness-cli/src/corpus_export.rs","kind":"section"}` | no | `resolved` |
| `extends` | `006-standalone-host` | `{"anchor":"corpus-export-module-export","file":"crates/wire-witness-cli/src/lib.rs","kind":"section"}` | no | `resolved` |
| `extends` | `001-boundaries-and-authority` | `{"anchor":"corpus-export-module-export","file":"crates/wire-witness-core/src/lib.rs","kind":"section"}` | no | `resolved` |

### References (non-owning)

- `{"kind":"file","path":"specs/008-instruction-delivery-observation/spec.md"}`
- `{"kind":"file","path":"specs/009-usage-and-cost-summaries/spec.md"}`
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
| `010-policy-bounded-corpus-export#V-1` | `specs/010-policy-bounded-corpus-export/spec.md` |

### Acceptance commands (declared, not executed)

```console
./.tooling/bin/spec-spine check --fail-on-warn
./.tooling/bin/spec-spine index check --fail-on-unresolved
cargo test -p wire-witness-core corpus_export --locked
cargo test -p wire-witness-cli corpus_export --locked
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
| `index:010-policy-bounded-corpus-export` | `sha256:5c58c54dfb917b4a3f2620ecff17480839ba4fdc545281cf1894549595d106c0` |
| `registry:010-policy-bounded-corpus-export` | `sha256:d49bf7206c1bbe5c98bfa30a59fee957ff7577e999b86e21cadc33937f8a7f6b` |
| `traceability:010-policy-bounded-corpus-export` | `sha256:37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
<!-- spec-spine:end generated id="inputs" -->

<!-- spec-spine:begin authored id="notes" -->
<!-- spec-spine:end authored id="notes" -->
