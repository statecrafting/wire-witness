<!-- spec-spine:begin generated id="header" -->
# `019-statecraft-profile-28` Adopt Statecraft profile 28 and spec-spine 0.32 with a supply-chain policy

- Path: `specs/019-statecraft-profile-28/spec.md`
- Summary: Moves the exact engine pin from 0.31.0 to 0.32.0 and the setup profile from github-actions-rust revision 21 to 28, in two pull requests: first the pin, a committed cargo-deny policy the lockfile passes, and the other prerequisites revisions 22 and 24 add; then the plan-bound profile render, which changes no policy path.
- Reference page schema: `1.0.0`
<!-- spec-spine:end generated id="header" -->

<!-- spec-spine:begin generated id="lifecycle" -->
## Lifecycle

| Field | Value |
| --- | --- |
| status | `approved` |
| implementation | `in-progress` |
| superseded_by | None declared. |
| retirement_rationale | None declared. |
| created | `2026-11-10` |
<!-- spec-spine:end generated id="lifecycle" -->

<!-- spec-spine:begin generated id="requirements" -->
## Requirements

| Obligation | Kind | Text | Anchor | Withdrawn |
| --- | --- | --- | --- | --- |
| `019-statecraft-profile-28#R-1` | `requirement` | The setup profile is github-actions-rust revision 28, applied only through Statecraft's plan-bound apply; no managed file is edited by hand. | `3-behavior` | no |
| `019-statecraft-profile-28#R-2` | `requirement` | The engine pin, the installed engine and the recorded producer all agree at spec-spine 0.32.0. | `3-behavior` | no |
| `019-statecraft-profile-28#R-3` | `requirement` | cargo deny check passes against the committed deny.toml with no advisory, ban, license or source error. | `3-behavior` | no |
| `019-statecraft-profile-28#R-4` | `requirement` | Every check spec 018 keeps required stays required, and the wire-witness-boundaries job remains a declared extra required job. | `3-behavior` | no |
| `019-statecraft-profile-28#V-1` | `verification` | Profile, pin, policy and boundary checks below pass, with the gate. | `verification` | no |
<!-- spec-spine:end generated id="requirements" -->

<!-- spec-spine:begin generated id="ownership" -->
## Ownership

| Edge | Target spec | Unit | Planned | Resolution |
| --- | --- | --- | --- | --- |
| `establishes` | None declared. | `{"kind":"file","path":"deny.toml"}` | no | `resolved` |
| `extends` | `002-exchange-record-and-normalization` | `{"anchor":"exchange-record-and-normalization","file":"crates/wire-witness-core/src/exchange.rs","kind":"section"}` | no | `resolved` |
| `extends` | `001-boundaries-and-authority` | `{"anchor":"workspace.dependencies","file":"Cargo.toml","kind":"section"}` | no | `resolved` |
| `extends` | `001-boundaries-and-authority` | `{"anchor":"workspace.package","file":"Cargo.toml","kind":"section"}` | no | `resolved` |
| `extends` | `017-profile-15-and-required-boundary-checks` | `{"kind":"file","path":".github/workflows/wire-witness-boundaries.yml"}` | no | `resolved` |
| `extends` | `001-boundaries-and-authority` | `{"kind":"file","path":"Cargo.lock"}` | no | `resolved` |
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

| Obligation | Inputs |
| --- | --- |
| `019-statecraft-profile-28#V-1` | `specs/019-statecraft-profile-28/spec.md`, `deny.toml`, `Cargo.lock` |

### Acceptance commands (declared, not executed)

```console
scripts/check-authored-content.sh --self-test
python3 -c 'import json; p = json.load(open(".statecraft/setup/github-actions-rust.json")); assert p["revision"] == 28; assert p["parameters"]["require_ratified"] is True; assert {"job": "wire-witness-boundaries", "workflow": ".github/workflows/wire-witness-boundaries.yml"} in p["parameters"]["extra_required_jobs"]'
.bin/spec-spine --version | grep -qx 'spec-spine 0.32.0'
python3 -c 'import json; p = json.load(open(".statecraft/environment.json"))["pins"]; assert p["spec_spine"] == p["producer"]["version"] == "0.32.0"'
cargo deny check
cargo test -p wire-witness-core --locked
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
| amends | `018-spec-spine-029-adoption-prerequisites` |
| amends_sections | None declared. |
| amends_verification | `018-spec-spine-029-adoption-prerequisites` |
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
| `index:019-statecraft-profile-28` | `sha256:8efc943d9269dbc58afe51342a5edd6291c792ecd2900d54c7b7c1aeaf6227db` |
| `registry:019-statecraft-profile-28` | `sha256:0c29eace608f119d88417988e3c35dca87fc562441bbcec470ecf0e6ede316c7` |
| `traceability:019-statecraft-profile-28` | `sha256:37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
<!-- spec-spine:end generated id="inputs" -->

<!-- spec-spine:begin authored id="notes" -->
<!-- spec-spine:end authored id="notes" -->
