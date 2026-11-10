<!-- spec-spine:begin generated id="header" -->
# `020-readme-and-reference-documentation` The README is a user guide and reference pages are generated from the specs

- Path: `specs/020-readme-and-reference-documentation/spec.md`
- Summary: Rewrites README.md as a user-facing guide (what wire-witness does, status, install, usage, exit codes, output schemas, workspace, documentation map, development) and moves toolchain and governance chronology out of it. Adds one generated spec-spine reference page per spec under docs/reference, with a repository-owned script that regenerates or checks them.
- Reference page schema: `1.0.0`
<!-- spec-spine:end generated id="header" -->

<!-- spec-spine:begin generated id="lifecycle" -->
## Lifecycle

| Field | Value |
| --- | --- |
| status | `draft` |
| implementation | `in-progress` |
| superseded_by | None declared. |
| retirement_rationale | None declared. |
| created | `2026-11-10` |
<!-- spec-spine:end generated id="lifecycle" -->

<!-- spec-spine:begin generated id="requirements" -->
## Requirements

| Obligation | Kind | Text | Anchor | Withdrawn |
| --- | --- | --- | --- | --- |
| `020-readme-and-reference-documentation#R-1` | `requirement` | Every command, option value, exit code, schema identifier and output path the README names exists in the source; a surface the live runtime does not implement is stated as not implemented. | `3-1-readme` | no |
| `020-readme-and-reference-documentation#R-2` | `requirement` | The README carries no engine pin chronology, setup-profile revision history or commit identities; it links to the specs and the dated audit that hold them. | `3-1-readme` | no |
| `020-readme-and-reference-documentation#R-3` | `requirement` | docs/reference holds exactly the pages spec-spine reference generate writes for the committed corpus, and scripts/reference.sh check passes. | `3-2-reference-pages` | no |
| `020-readme-and-reference-documentation#V-1` | `verification` | The README usage block matches the executable's usage text, the reference check passes, and the gate passes. | `verification` | no |
<!-- spec-spine:end generated id="requirements" -->

<!-- spec-spine:begin generated id="ownership" -->
## Ownership

| Edge | Target spec | Unit | Planned | Resolution |
| --- | --- | --- | --- | --- |
| `establishes` | None declared. | `{"kind":"file","path":"docs/reference/request.json"}` | no | `resolved` |
| `establishes` | None declared. | `{"kind":"file","path":"scripts/reference.sh"}` | no | `resolved` |
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

| Obligation | Inputs |
| --- | --- |
| `020-readme-and-reference-documentation#V-1` | `specs/020-readme-and-reference-documentation/spec.md`, `README.md`, `crates/wire-witness-cli/src/main.rs`, `docs/reference/request.json` |

### Acceptance commands (declared, not executed)

```console
.bin/spec-spine check --fail-on-warn
scripts/reference.sh check
python3 -c 'import re; u = re.search(r"const USAGE: &str = \"(.*?)\";", open("crates/wire-witness-cli/src/main.rs").read()).group(1); r = open("README.md").read(); [None for w in re.findall(r"--[a-z-]+|metadata-only|node-extra-ca-certs|aws-ca-bundle|existing-roots-bundle", u) if w in r or exit("README lacks " + w)]'
python3 -c 'r = open("README.md").read(); assert "0.29.0" not in r and "revision 21" not in r'
scripts/check-authored-content.sh
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
| `index:020-readme-and-reference-documentation` | `sha256:33616d64ede8516ac6eb17d5b40864eb6e7269a83613e4c187fa0f0c733888ab` |
| `registry:020-readme-and-reference-documentation` | `sha256:33616d64ede8516ac6eb17d5b40864eb6e7269a83613e4c187fa0f0c733888ab` |
| `traceability:020-readme-and-reference-documentation` | `sha256:37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
<!-- spec-spine:end generated id="inputs" -->

<!-- spec-spine:begin authored id="notes" -->
<!-- spec-spine:end authored id="notes" -->
