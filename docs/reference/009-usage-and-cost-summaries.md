<!-- spec-spine:begin generated id="header" -->
# `009-usage-and-cost-summaries` Usage and cost summaries

- Path: `specs/009-usage-and-cost-summaries/spec.md`
- Summary: Defines a deterministic binding-level summary of observed request counts, provider-attributed usage, reported cost, labeled estimates, identity variation, completeness, and gaps without converting testimony into a charge, quota decision, budget, or provider qualification.
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
| `009-usage-and-cost-summaries#I-1` | `invariant` | A summary preserves source attribution and never relabels provider-reported usage, reported cost, or a local estimate as an observed charge, quota, or spending decision. | `3-1-summary-authority` | no |
| `009-usage-and-cost-summaries#I-2` | `invariant` | Only records with one exact binding and a contiguous exchange sequence may form a complete summary; missing, duplicate, foreign, or reordered inputs remain explicit findings. | `3-2-input-set-and-order` | no |
| `009-usage-and-cost-summaries#R-1` | `requirement` | Usage and cost are aggregated only within identical provider, meaning, unit, numeric construction, and attribution keys, while incompatible entries remain separate deterministically ordered groups. | `3-3-aggregation-rules` | no |
| `009-usage-and-cost-summaries#R-2` | `requirement` | The summary binds its ordered exchange digests and reports capture completeness, requested and served identity variation, unknowns, and cost provenance without requiring retained content. | `3-4-summary-schema` | no |
| `009-usage-and-cost-summaries#V-1` | `verification` | The deterministic summary engine passes its exact arithmetic, ordering, validation, gap, attribution, and canonicalization tests. | `verification` | no |
<!-- spec-spine:end generated id="requirements" -->

<!-- spec-spine:begin generated id="ownership" -->
## Ownership

| Edge | Target spec | Unit | Planned | Resolution |
| --- | --- | --- | --- | --- |
| `establishes` | None declared. | `{"anchor":"measurement-summary","file":"crates/wire-witness-core/src/measurement_summary.rs","kind":"section"}` | no | `resolved` |
| `extends` | `001-boundaries-and-authority` | `{"anchor":"measurement-summary-module-export","file":"crates/wire-witness-core/src/lib.rs","kind":"section"}` | no | `resolved` |

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
| `009-usage-and-cost-summaries#V-1` | `specs/009-usage-and-cost-summaries/spec.md` |

### Acceptance commands (declared, not executed)

```console
./.tooling/bin/spec-spine check --fail-on-warn
./.tooling/bin/spec-spine index check --fail-on-unresolved
cargo test -p wire-witness-core measurement_summary --locked
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
| `index:009-usage-and-cost-summaries` | `sha256:167612a66eac2667bacc0fedf68c74518a40361e62d41815613654176b8d88de` |
| `registry:009-usage-and-cost-summaries` | `sha256:b78739389d5bbf9b633e2b916d721f1b2755add0d5607b0ef356b676e59cce9a` |
| `traceability:009-usage-and-cost-summaries` | `sha256:37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
<!-- spec-spine:end generated id="inputs" -->

<!-- spec-spine:begin authored id="notes" -->
<!-- spec-spine:end authored id="notes" -->
