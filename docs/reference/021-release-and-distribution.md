<!-- spec-spine:begin generated id="header" -->
# `021-release-and-distribution` One signed tag releases wire-witness to GitHub, crates.io, npm and PyPI

- Path: `specs/021-release-and-distribution/spec.md`
- Summary: Proposes how the wire-witness executable and crates reach users: a release workflow started by the owner's signed annotated tag on main builds static binaries for four Unix targets once, attests them, and publishes the same bytes as GitHub release archives, the three crates on crates.io, an npm launcher with per-platform binary packages, and Python wheels. Every registry package is published from CI through trusted publishing with provenance. Modelled on spec-spine's distribution (its specs 006, 007, 019, 209) for the binary channels and on rustev's spec 058 for the tag gate.
- Reference page schema: `1.0.0`
<!-- spec-spine:end generated id="header" -->

<!-- spec-spine:begin generated id="lifecycle" -->
## Lifecycle

| Field | Value |
| --- | --- |
| status | `draft` |
| implementation | `pending` |
| superseded_by | None declared. |
| retirement_rationale | None declared. |
| created | `2026-11-10` |
<!-- spec-spine:end generated id="lifecycle" -->

<!-- spec-spine:begin generated id="requirements" -->
## Requirements

None declared.
<!-- spec-spine:end generated id="requirements" -->

<!-- spec-spine:begin generated id="ownership" -->
## Ownership

| Edge | Target spec | Unit | Planned | Resolution |
| --- | --- | --- | --- | --- |
| `establishes` | None declared. | `{"kind":"file","path":".github/workflows/release.yml"}` | yes | `unresolved` |
| `establishes` | None declared. | `{"kind":"file","path":"docs/releasing.md"}` | yes | `unresolved` |
| `establishes` | None declared. | `{"kind":"file","path":"npm/bin/wire-witness.js"}` | yes | `unresolved` |
| `establishes` | None declared. | `{"kind":"file","path":"npm/lib/platform.js"}` | yes | `unresolved` |
| `establishes` | None declared. | `{"kind":"file","path":"npm/package.json"}` | yes | `unresolved` |
| `establishes` | None declared. | `{"kind":"file","path":"npm/scripts/generate-platform-packages.js"}` | yes | `unresolved` |
| `establishes` | None declared. | `{"kind":"file","path":"py/pyproject.toml"}` | yes | `unresolved` |
| `establishes` | None declared. | `{"kind":"file","path":"py/scripts/generate_wheels.py"}` | yes | `unresolved` |
| `establishes` | None declared. | `{"kind":"file","path":"scripts/release/bump-version.py"}` | yes | `unresolved` |
| `establishes` | None declared. | `{"kind":"file","path":"scripts/release/check-tag.sh"}` | yes | `unresolved` |

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
.bin/spec-spine check --fail-on-warn
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
| `index:021-release-and-distribution` | `sha256:ca728a07f19eb285a23da2c901f4732772e6bf4e27265197ba915913b4587d80` |
| `registry:021-release-and-distribution` | `sha256:ca728a07f19eb285a23da2c901f4732772e6bf4e27265197ba915913b4587d80` |
| `traceability:021-release-and-distribution` | `sha256:37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
<!-- spec-spine:end generated id="inputs" -->

<!-- spec-spine:begin authored id="notes" -->
<!-- spec-spine:end authored id="notes" -->
