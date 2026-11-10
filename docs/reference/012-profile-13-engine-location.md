<!-- spec-spine:begin generated id="header" -->
# `012-profile-13-engine-location` Profile 13 uses the repository-local engine in .bin

- Path: `specs/012-profile-13-engine-location/spec.md`
- Summary: Adopts Statecraft profile revision 13 with the existing exact engine pin, and replaces revision 12 acceptance with the recorded revision 13 identity.
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
| created | `2026-10-02` |
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
scripts/check-authored-content.sh --self-test
scripts/check-authored-content.sh --identity HEAD HEAD
python3 -c 'import json; a = json.load(open(".claude/settings.json"))["attribution"]; assert a == {"commit": "", "pr": "", "sessionUrl": False}, a'
jq -e '.revision == 13 and .parameters.require_ratified == true' .statecraft/setup/github-actions-rust.json
.bin/spec-spine --version | grep -qx 'spec-spine 0.28.0'
make gate
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
| amends | `011-agent-attribution-and-governance-flows` |
| amends_sections | None declared. |
| amends_verification | `011-agent-attribution-and-governance-flows` |
| supersedes | None declared. |
| amendment_record | None declared. |
| amended by | `017-profile-15-and-required-boundary-checks` |
| superseded by | None declared. |
| extended by | None declared. |
<!-- spec-spine:end generated id="changes" -->

<!-- spec-spine:begin generated id="inputs" -->
## Inputs

| Identity | Digest |
| --- | --- |
| `index:012-profile-13-engine-location` | `sha256:f24c0192ddab01027d47c03b81d667c10dddb467b8fa2fa116d43169a07f0541` |
| `registry:012-profile-13-engine-location` | `sha256:f24c0192ddab01027d47c03b81d667c10dddb467b8fa2fa116d43169a07f0541` |
| `traceability:012-profile-13-engine-location` | `sha256:37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
<!-- spec-spine:end generated id="inputs" -->

<!-- spec-spine:begin authored id="notes" -->
<!-- spec-spine:end authored id="notes" -->
