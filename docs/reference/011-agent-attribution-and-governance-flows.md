<!-- spec-spine:begin generated id="header" -->
# `011-agent-attribution-and-governance-flows` Agent attribution is refused and ratification is owner-gated

- Path: `specs/011-agent-attribution-and-governance-flows/spec.md`
- Summary: Refuses agent attribution and known agent commit identities, suppresses provider attribution at its source, and adopts the two-flow Statecraft profile in which implementation and owner ratification remain separate.
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
| created | `2026-09-30` |
<!-- spec-spine:end generated id="lifecycle" -->

<!-- spec-spine:begin generated id="requirements" -->
## Requirements

| Obligation | Kind | Text | Anchor | Withdrawn |
| --- | --- | --- | --- | --- |
| `011-agent-attribution-and-governance-flows#R-1` | `requirement` | Repository-authored text and commit identities refuse known agent attribution while human identities remain allowed. | `3-1-attribution-and-identity-refusal` | no |
| `011-agent-attribution-and-governance-flows#R-2` | `requirement` | A specification becoming approved requires the owner exception independently of implementation review. | `3-2-two-flow-ratification` | no |
| `011-agent-attribution-and-governance-flows#V-1` | `verification` | The checker self-test and Statecraft profile policy prove attribution refusal, identity refusal, and owner-gated ratification. | `verification` | no |
<!-- spec-spine:end generated id="requirements" -->

<!-- spec-spine:begin generated id="ownership" -->
## Ownership

| Edge | Target spec | Unit | Planned | Resolution |
| --- | --- | --- | --- | --- |
| `establishes` | None declared. | `{"kind":"file","path":".claude/settings.json"}` | no | `resolved` |
| `extends` | `001-boundaries-and-authority` | `{"kind":"file","path":"scripts/check-authored-content.sh"}` | no | `resolved` |

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
| `011-agent-attribution-and-governance-flows#V-1` | `scripts/check-authored-content.sh`, `.claude/settings.json`, `.statecraft/setup/github-actions-rust.json` |

### Acceptance commands (declared, not executed)

```console
scripts/check-authored-content.sh --self-test
scripts/check-authored-content.sh --identity HEAD HEAD
python3 -c 'import json; a = json.load(open(".claude/settings.json"))["attribution"]; assert a == {"commit": "", "pr": "", "sessionUrl": False}, a'
jq -e '.revision == 12 and .parameters.require_ratified == true' .statecraft/setup/github-actions-rust.json
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
| amended by | `012-profile-13-engine-location` |
| superseded by | None declared. |
| extended by | None declared. |
<!-- spec-spine:end generated id="changes" -->

<!-- spec-spine:begin generated id="inputs" -->
## Inputs

| Identity | Digest |
| --- | --- |
| `index:011-agent-attribution-and-governance-flows` | `sha256:b09c5d9288b5120392cbcdb9ac61495eb324eb7d05e8a143aee4cdbacb9fa56c` |
| `registry:011-agent-attribution-and-governance-flows` | `sha256:b09c5d9288b5120392cbcdb9ac61495eb324eb7d05e8a143aee4cdbacb9fa56c` |
| `traceability:011-agent-attribution-and-governance-flows` | `sha256:37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
<!-- spec-spine:end generated id="inputs" -->

<!-- spec-spine:begin authored id="notes" -->
<!-- spec-spine:end authored id="notes" -->
