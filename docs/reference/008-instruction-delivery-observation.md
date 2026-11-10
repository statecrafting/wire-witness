<!-- spec-spine:begin generated id="header" -->
# `008-instruction-delivery-observation` Instruction delivery observation

- Path: `specs/008-instruction-delivery-observation/spec.md`
- Summary: Defines a content-gated, offline comparison that reports whether exact operator-supplied instruction bytes were observed in a complete outbound provider request, without turning one observation into adapter qualification, policy, or authority.
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
| `008-instruction-delivery-observation#I-1` | `invariant` | An instruction observation is testimony about one exact captured request and never establishes an adapter-wide delivery guarantee, qualification, policy, or authority. | `3-1-observation-not-qualification` | no |
| `008-instruction-delivery-observation#I-2` | `invariant` | The comparison target, normalized component selector, and matching mode are supplied before observation and are never inferred from repository state, filenames, prompt meaning, or nearby text. | `3-2-predeclared-comparison-plan` | no |
| `008-instruction-delivery-observation#R-1` | `requirement` | Present and absent results require a complete decoded request component and exact-byte comparison; every incomplete, disabled, redacted, or unsupported case is unknown with a reason. | `3-3-three-state-result` | no |
| `008-instruction-delivery-observation#R-2` | `requirement` | The durable result binds the target digest and length, exchange digest, selected component identity and digest, match count, completeness, and findings without retaining target or observed instruction bytes. | `3-4-durable-result` | no |
| `008-instruction-delivery-observation#R-3` | `requirement` | Request components are decoded by this spec's own closed decoder over the complete outbound request body, which never changes wire-witness.exchange/1 and never reports a match, absence, digest, length, or offset for a component that mandatory redaction touched. | `3-5-request-component-decoding` | no |
| `008-instruction-delivery-observation#V-1` | `verification` | The observation engine and host-side target acquisition tests pass, both owned source sections are indexed, and the repository gate remains clean. | `verification` | no |
<!-- spec-spine:end generated id="requirements" -->

<!-- spec-spine:begin generated id="ownership" -->
## Ownership

| Edge | Target spec | Unit | Planned | Resolution |
| --- | --- | --- | --- | --- |
| `establishes` | None declared. | `{"anchor":"instruction-delivery-observation","file":"crates/wire-witness-cli/src/instruction_observation.rs","kind":"section"}` | no | `resolved` |
| `establishes` | None declared. | `{"anchor":"instruction-delivery-observation","file":"crates/wire-witness-core/src/instruction_observation.rs","kind":"section"}` | no | `resolved` |
| `extends` | `001-boundaries-and-authority` | `{"anchor":"dependencies","file":"crates/wire-witness-core/Cargo.toml","kind":"section"}` | no | `resolved` |
| `extends` | `006-standalone-host` | `{"anchor":"instruction-observation-module-export","file":"crates/wire-witness-cli/src/lib.rs","kind":"section"}` | no | `resolved` |
| `extends` | `001-boundaries-and-authority` | `{"anchor":"instruction-observation-module-export","file":"crates/wire-witness-core/src/lib.rs","kind":"section"}` | no | `resolved` |
| `extends` | `001-boundaries-and-authority` | `{"anchor":"workspace.dependencies","file":"Cargo.toml","kind":"section"}` | no | `resolved` |
| `extends` | `001-boundaries-and-authority` | `{"kind":"file","path":"Cargo.lock"}` | no | `resolved` |
| `extends` | `006-standalone-host` | `{"kind":"file","path":"crates/wire-witness-cli/src/standalone_host.rs"}` | no | `resolved` |
| `extends` | `001-boundaries-and-authority` | `{"kind":"file","path":"scripts/check-boundaries.sh"}` | no | `resolved` |

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
| `008-instruction-delivery-observation#V-1` | `specs/008-instruction-delivery-observation/spec.md` |

### Acceptance commands (declared, not executed)

```console
./.tooling/bin/spec-spine check --fail-on-warn
./.tooling/bin/spec-spine index check --fail-on-unresolved
cargo test -p wire-witness-core instruction_observation --locked
cargo test -p wire-witness-cli instruction_observation --locked
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
| `index:008-instruction-delivery-observation` | `sha256:fbad55d6a3b22c44fc554e256046a1f9fbfa66445c21204076523933cc9e43dd` |
| `registry:008-instruction-delivery-observation` | `sha256:af1b2bde7c3d4805b159bc72f843bcc89d21f955d742f58e2b2f18eded998df3` |
| `traceability:008-instruction-delivery-observation` | `sha256:37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
<!-- spec-spine:end generated id="inputs" -->

<!-- spec-spine:begin authored id="notes" -->
<!-- spec-spine:end authored id="notes" -->
