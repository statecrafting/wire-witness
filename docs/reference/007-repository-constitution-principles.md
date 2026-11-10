<!-- spec-spine:begin generated id="header" -->
# `007-repository-constitution-principles` Repository constitution principles

- Path: `specs/007-repository-constitution-principles/spec.md`
- Summary: Establishes six wire-witness-specific constitution principles covering testimony, secret custody, ambient-system mutation, exact supervised binding, lifecycle claims, and explicit incomplete or residual-risk states. The owner-approved principles are activated in the standing constitution.
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
| `007-repository-constitution-principles#I-1` | `invariant` | Approval of this spec, not draft authorship, is the authority required before its proposed text can become standing constitution text. | `3-1-activation-boundary` | no |
| `007-repository-constitution-principles#R-1` | `requirement` | The activated constitution states that witness testimony grants no authority and that supervised identity is supplied exactly rather than inferred. | `3-2-approved-normative-text` | no |
| `007-repository-constitution-principles#R-2` | `requirement` | The activated constitution requires redaction before durable custody, forbids ambient-system mutation, and reports lifecycle, absence, incompleteness, and residual risk precisely. | `3-2-approved-normative-text` | no |
| `007-repository-constitution-principles#V-1` | `verification` | The approved spec compiles with six resolved section claims, and the standing constitution contains exactly the approved principles from section 3.2. | `verification` | no |
<!-- spec-spine:end generated id="requirements" -->

<!-- spec-spine:begin generated id="ownership" -->
## Ownership

| Edge | Target spec | Unit | Planned | Resolution |
| --- | --- | --- | --- | --- |
| `establishes` | None declared. | `{"anchor":"ix-supervised-binding-is-supplied-exactly","file":"standards/spec/constitution.md","kind":"section"}` | no | `resolved` |
| `establishes` | None declared. | `{"anchor":"vi-testimony-never-grants-authority","file":"standards/spec/constitution.md","kind":"section"}` | no | `resolved` |
| `establishes` | None declared. | `{"anchor":"vii-secrets-are-reduced-before-durable-custody","file":"standards/spec/constitution.md","kind":"section"}` | no | `resolved` |
| `establishes` | None declared. | `{"anchor":"viii-capture-never-mutates-ambient-trust-or-routing","file":"standards/spec/constitution.md","kind":"section"}` | no | `resolved` |
| `establishes` | None declared. | `{"anchor":"x-lifecycle-and-evidence-claims-remain-precise","file":"standards/spec/constitution.md","kind":"section"}` | no | `resolved` |
| `establishes` | None declared. | `{"anchor":"xi-absence-incompleteness-and-residual-risk-remain-explicit","file":"standards/spec/constitution.md","kind":"section"}` | no | `resolved` |

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
| `007-repository-constitution-principles#V-1` | `specs/007-repository-constitution-principles/spec.md`, `standards/spec/constitution.md` |

### Acceptance commands (declared, not executed)

```console
.bin/spec-spine check --fail-on-warn
.bin/spec-spine lint --fail-on-warn
.bin/spec-spine index check --fail-on-unresolved
grep -qxF 'status: approved' specs/007-repository-constitution-principles/spec.md
grep -qxF 'implementation: complete' specs/007-repository-constitution-principles/spec.md
python3 -c 'from pathlib import Path; s = Path("specs/007-repository-constitution-principles/spec.md").read_text(); c = Path("standards/spec/constitution.md").read_text(); expected = s.split("#### VI. Testimony never grants authority\n", 1)[1].split("\n## 4. Authority boundaries", 1)[0].strip().replace("\n#### ", "\n## "); actual = c.split("## VI. Testimony never grants authority\n", 1)[1].split("\n---\n", 1)[0].strip(); assert actual == expected, "standing principles differ from approved section 3.2"; assert "Replace this section with your first principle." not in c, "constitution placeholder remains"; assert "planned: true" not in s.split("---", 2)[1], "activation claims remain planned"'
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
| `index:007-repository-constitution-principles` | `sha256:b5406f11aae008974dd4d6c83ea610f4fc26097db7cd3663cde023a5e979d319` |
| `registry:007-repository-constitution-principles` | `sha256:8933e01ef9bd7dbd82b034c0df0711f28c64893fb582e9413bc3804cdbe02760` |
| `traceability:007-repository-constitution-principles` | `sha256:37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
<!-- spec-spine:end generated id="inputs" -->

<!-- spec-spine:begin authored id="notes" -->
<!-- spec-spine:end authored id="notes" -->
