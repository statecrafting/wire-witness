<!-- spec-spine:begin generated id="header" -->
# `017-profile-15-and-required-boundary-checks` Profile 15 preserves required workspace and dependency checks

- Path: `specs/017-profile-15-and-required-boundary-checks/spec.md`
- Summary: Adopts the current Statecraft setup profile with the supported exact engine pin, makes dependency-boundary verification a required CI job, and replaces historical setup prose with current operator instructions.
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
| `establishes` | None declared. | `{"kind":"file","path":".github/workflows/wire-witness-boundaries.yml"}` | no | `resolved` |
| `establishes` | None declared. | `{"kind":"file","path":"docs/codebase-audit-2026-10-10.md"}` | no | `resolved` |
| `refines` | None declared. | `{"kind":"file","path":"Makefile"}` | no | `resolved` |
| `refines` | None declared. | `{"kind":"file","path":"README.md"}` | no | `resolved` |

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
python3 -c 'import json; p = json.load(open(".statecraft/setup/github-actions-rust.json")); assert p["revision"] == 15; assert p["parameters"]["require_ratified"] is True; assert {"job": "wire-witness-boundaries", "workflow": ".github/workflows/wire-witness-boundaries.yml"} in p["parameters"]["extra_required_jobs"]'
.bin/spec-spine --version | grep -qx 'spec-spine 0.28.0'
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
| amends | `012-profile-13-engine-location` |
| amends_sections | None declared. |
| amends_verification | `012-profile-13-engine-location` |
| supersedes | None declared. |
| amendment_record | None declared. |
| amended by | `018-spec-spine-029-adoption-prerequisites` |
| superseded by | None declared. |
| extended by | `019-statecraft-profile-28` |
<!-- spec-spine:end generated id="changes" -->

<!-- spec-spine:begin generated id="inputs" -->
## Inputs

| Identity | Digest |
| --- | --- |
| `index:017-profile-15-and-required-boundary-checks` | `sha256:637406e804e9ac812a2dfed0d68263cb3310f367ef935c09441752c093f156b5` |
| `registry:017-profile-15-and-required-boundary-checks` | `sha256:637406e804e9ac812a2dfed0d68263cb3310f367ef935c09441752c093f156b5` |
| `traceability:017-profile-15-and-required-boundary-checks` | `sha256:37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
<!-- spec-spine:end generated id="inputs" -->

<!-- spec-spine:begin authored id="notes" -->
<!-- spec-spine:end authored id="notes" -->
