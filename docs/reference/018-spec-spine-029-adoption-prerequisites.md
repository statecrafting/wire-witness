<!-- spec-spine:begin generated id="header" -->
# `018-spec-spine-029-adoption-prerequisites` Adopt spec-spine 0.29 through the updated Statecraft producer

- Path: `specs/018-spec-spine-029-adoption-prerequisites/spec.md`
- Summary: Migrates the exact governance engine through a Statecraft build that links the same released producer, adopting current profile 16 and all required checks.
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
| created | `2026-10-10` |
<!-- spec-spine:end generated id="lifecycle" -->

<!-- spec-spine:begin generated id="requirements" -->
## Requirements

None declared.
<!-- spec-spine:end generated id="requirements" -->

<!-- spec-spine:begin generated id="ownership" -->
## Ownership

| Edge | Target spec | Unit | Planned | Resolution |
| --- | --- | --- | --- | --- |
| `refines` | None declared. | `{"kind":"file","path":"README.md"}` | no | `resolved` |
| `refines` | None declared. | `{"kind":"file","path":"docs/codebase-audit-2026-10-10.md"}` | no | `resolved` |
| `refines` | None declared. | `{"kind":"file","path":"spec-spine.toml"}` | no | `resolved` |

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
python3 -c 'import json; p = json.load(open(".statecraft/setup/github-actions-rust.json")); assert p["revision"] == 21; assert p["parameters"]["require_ratified"] is True; assert {"job": "wire-witness-boundaries", "workflow": ".github/workflows/wire-witness-boundaries.yml"} in p["parameters"]["extra_required_jobs"]; assert p["jobs"]["preflight"]["required"] is True'
.bin/spec-spine --version | grep -qx 'spec-spine 0.31.0'
python3 -c 'import json; p = json.load(open(".statecraft/environment.json"))["pins"]; assert p["spec_spine"] == p["producer"]["version"] == "0.31.0"; assert p["producer"]["checksum"] == "de002551d95919c9f136900a402f1ed5b462f6f8f7a5338cf54a2e70e89215e2"'
make boundaries
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
| amends | `017-profile-15-and-required-boundary-checks` |
| amends_sections | None declared. |
| amends_verification | `017-profile-15-and-required-boundary-checks` |
| supersedes | None declared. |
| amendment_record | None declared. |
| amended by | `019-statecraft-profile-28` |
| superseded by | None declared. |
| extended by | None declared. |
<!-- spec-spine:end generated id="changes" -->

<!-- spec-spine:begin generated id="inputs" -->
## Inputs

| Identity | Digest |
| --- | --- |
| `index:018-spec-spine-029-adoption-prerequisites` | `sha256:1152ee3466a9fcb2c8b848692f667e3111804a1d946add096a36838a438ab471` |
| `registry:018-spec-spine-029-adoption-prerequisites` | `sha256:1152ee3466a9fcb2c8b848692f667e3111804a1d946add096a36838a438ab471` |
| `traceability:018-spec-spine-029-adoption-prerequisites` | `sha256:37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
<!-- spec-spine:end generated id="inputs" -->

<!-- spec-spine:begin authored id="notes" -->
<!-- spec-spine:end authored id="notes" -->
