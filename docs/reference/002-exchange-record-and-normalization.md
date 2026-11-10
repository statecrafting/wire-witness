<!-- spec-spine:begin generated id="header" -->
# `002-exchange-record-and-normalization` Exchange record and provider normalization

- Path: `specs/002-exchange-record-and-normalization/spec.md`
- Summary: Defines wire-witness.exchange/1, the canonical testimony produced from one observed provider exchange, and lossless normalization rules for Anthropic and OpenAI request, response, streaming, identity, usage, and cost fields.
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
| `002-exchange-record-and-normalization#I-1` | `invariant` | An exchange is testimony only; provider and harness statements remain attributed claims and never become authority. | `3-1-testimony-and-schema` | no |
| `002-exchange-record-and-normalization#I-2` | `invariant` | Requested identity and served identity are separate fields, and unknown is an explicit value that is never filled from another identity. | `3-3-identity-usage-and-cost` | no |
| `002-exchange-record-and-normalization#R-1` | `requirement` | Unknown provider event types are preserved as unknown and malformed provider input produces a finding rather than a normalized success. | `3-4-provider-normalization` | no |
| `002-exchange-record-and-normalization#R-2` | `requirement` | Canonical exchange bytes use canonical-keysort-json followed by SHA-256 under a named construction. | `3-5-canonical-bytes-and-digests` | no |
| `002-exchange-record-and-normalization#V-1` | `verification` | The implementation tests pass, its module is indexed, and both producer interface pins verify from the named local exports. | `verification` | no |
<!-- spec-spine:end generated id="requirements" -->

<!-- spec-spine:begin generated id="ownership" -->
## Ownership

| Edge | Target spec | Unit | Planned | Resolution |
| --- | --- | --- | --- | --- |
| `establishes` | None declared. | `{"anchor":"exchange-record-and-normalization","file":"crates/wire-witness-core/src/exchange.rs","kind":"section"}` | no | `resolved` |

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
| `002-exchange-record-and-normalization#V-1` | `specs/002-exchange-record-and-normalization/spec.md` |

### Acceptance commands (declared, not executed)

```console
./.tooling/bin/spec-spine check --fail-on-warn
./.tooling/bin/spec-spine interface verify --spec 002 --export rustev=/Users/bart/DevWork/rustev --export canonical-keysort-json=/Users/bart/DevWork/canonical-keysort-json
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
| extended by | `019-statecraft-profile-28` |
<!-- spec-spine:end generated id="changes" -->

<!-- spec-spine:begin generated id="inputs" -->
## Inputs

| Identity | Digest |
| --- | --- |
| `index:002-exchange-record-and-normalization` | `sha256:14e4dacc672ee0fb067b3757fa2e1630912aeb770449dd70ad026e7fddc880f9` |
| `registry:002-exchange-record-and-normalization` | `sha256:210604581f8aab409b5a262d39a56a01cf561eaaeb29af2ddf99a7bc4c3031a7` |
| `traceability:002-exchange-record-and-normalization` | `sha256:37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
<!-- spec-spine:end generated id="inputs" -->

<!-- spec-spine:begin authored id="notes" -->
<!-- spec-spine:end authored id="notes" -->
