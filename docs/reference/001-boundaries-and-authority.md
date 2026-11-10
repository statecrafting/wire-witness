<!-- spec-spine:begin generated id="header" -->
# `001-boundaries-and-authority` Boundaries, authority, and dependency layout

- Path: `specs/001-boundaries-and-authority/spec.md`
- Summary: Defines wire-witness as a library plus isolated hosts that capture provider wire exchanges and produce testimony without granting authority. Assigns ecosystem responsibilities, fixes the three-crate dependency layout, and reserves captured-output training, hosting, and publication to the owner.
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
| `001-boundaries-and-authority#I-1` | `invariant` | Every output is testimony about an observed exchange and never authorizes, admits, evaluates, or changes a plan or policy. | `3-1-testimony-never-authority` | no |
| `001-boundaries-and-authority#I-2` | `invariant` | Supervised capture repeats its supplied attempt binding verbatim and never infers, mints, or alters that binding. | `3-2-binding-not-correlation` | no |
| `001-boundaries-and-authority#R-1` | `requirement` | The pure core performs no I/O, reads no clock, forbids unsafe, and depends on no async runtime, HTTP stack, or TLS stack. | `3-4-dependency-rules` | no |
| `001-boundaries-and-authority#V-1` | `verification` | The corpus gate, authored-content check, and exact three-crate dependency boundary all pass. | `verification` | no |
<!-- spec-spine:end generated id="requirements" -->

<!-- spec-spine:begin generated id="ownership" -->
## Ownership

| Edge | Target spec | Unit | Planned | Resolution |
| --- | --- | --- | --- | --- |
| `establishes` | None declared. | `{"id":"wire-witness-core","kind":"crate"}` | no | `resolved` |
| `establishes` | None declared. | `{"kind":"file","path":"Cargo.lock"}` | no | `resolved` |
| `establishes` | None declared. | `{"kind":"file","path":"Cargo.toml"}` | no | `resolved` |
| `establishes` | None declared. | `{"kind":"file","path":"Makefile"}` | no | `resolved` |
| `establishes` | None declared. | `{"kind":"file","path":"scripts/check-authored-content.sh"}` | no | `resolved` |
| `establishes` | None declared. | `{"kind":"file","path":"scripts/check-boundaries.sh"}` | no | `resolved` |

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
| `001-boundaries-and-authority#V-1` | `specs/001-boundaries-and-authority/spec.md`, `Cargo.toml`, `scripts/check-authored-content.sh`, `scripts/check-boundaries.sh` |

### Acceptance commands (declared, not executed)

```console
make gate
scripts/check-authored-content.sh
scripts/check-boundaries.sh
cargo test --workspace --locked
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
| extended by | `008-instruction-delivery-observation`, `009-usage-and-cost-summaries`, `010-policy-bounded-corpus-export`, `011-agent-attribution-and-governance-flows`, `013-live-loopback-capture-runtime`, `019-statecraft-profile-28` |
<!-- spec-spine:end generated id="changes" -->

<!-- spec-spine:begin generated id="inputs" -->
## Inputs

| Identity | Digest |
| --- | --- |
| `index:001-boundaries-and-authority` | `sha256:12ae12bd0c2924d44575d17d9be05f5a803f7d56ffbb3db82a7babda33605c89` |
| `registry:001-boundaries-and-authority` | `sha256:12ae12bd0c2924d44575d17d9be05f5a803f7d56ffbb3db82a7babda33605c89` |
| `traceability:001-boundaries-and-authority` | `sha256:37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
<!-- spec-spine:end generated id="inputs" -->

<!-- spec-spine:begin authored id="notes" -->
<!-- spec-spine:end authored id="notes" -->
