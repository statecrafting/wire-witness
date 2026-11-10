---
id: "020-readme-and-reference-documentation"
title: "The README is a user guide and reference pages are generated from the specs"
status: approved
implementation: complete
created: "2026-11-10"
summary: >
  Rewrites README.md as a user-facing guide (what wire-witness does, status,
  install, usage, exit codes, output schemas, workspace, documentation map,
  development) and moves toolchain and governance chronology out of it. Adds
  one generated spec-spine reference page per spec under docs/reference, with
  a repository-owned script that regenerates or checks them.
refines:
  - aspect: "user-guide-structure"
    unit: "README.md"
establishes:
  - "scripts/reference.sh"
  - "docs/reference/request.json"
depends_on:
  - "011-agent-attribution-and-governance-flows"
  - "017-profile-15-and-required-boundary-checks"
  - "018-spec-spine-029-adoption-prerequisites"
  - "019-statecraft-profile-28"
obligations:
  - id: "R-1"
    kind: requirement
    text: "Every command, option value, exit code, schema identifier and output path the README names exists in the source; a surface the live runtime does not implement is stated as not implemented."
    anchor: "3-1-readme"
  - id: "R-2"
    kind: requirement
    text: "The README carries no engine pin chronology, setup-profile revision history or commit identities; it links to the specs and the dated audit that hold them."
    anchor: "3-1-readme"
  - id: "R-3"
    kind: requirement
    text: "docs/reference holds exactly the pages spec-spine reference generate writes for the committed corpus, and scripts/reference.sh check passes."
    anchor: "3-2-reference-pages"
  - id: "V-1"
    kind: verification
    text: "The README usage block matches the executable's usage text, the reference check passes, and the gate passes."
    anchor: "verification"
    inputs:
      - "specs/020-readme-and-reference-documentation/spec.md"
      - "README.md"
      - "crates/wire-witness-cli/src/main.rs"
      - "docs/reference/request.json"
---

# 020: The README is a user guide and reference pages are generated from the specs

## 1. Purpose

The README written under specs 017 and 018 described the implemented crates
accurately, but spent most of its length on engine pins, setup-profile
revisions and commit identities. It had no install path, no usage, no exit
codes and no output description, and its pin narrative went stale whenever a
local engine lagged the pin. A reader learned how the repository was governed
before learning what the tool does.

## 2. Territory

This spec refines README.md for its structure as a user guide. Specs 017 and
018 keep their refinements; the facts they required (implemented surfaces,
actual local commands, the exact engine) remain true in the new text or in
the documents it links. It establishes `scripts/reference.sh` and the
reference request. The generated pages under `docs/reference/` are output of
`spec-spine reference generate`, never edited by hand except inside each
page's authored `notes` region.

The Makefile is a Statecraft-tracked file, so this spec adds no make target.

## 3. Behavior

### 3.1 README

1. Order: purpose and authority statement, capabilities, status table,
   install, usage with exit codes and output schemas, workspace, a
   documentation map to the specs, distribution, development, license.
2. Claims are checked against source: the usage text against
   `crates/wire-witness-cli/src/main.rs`, exit codes against `HostExit`,
   schema identifiers against their `SCHEMA` constants, and the attempt layout
   against `live_runtime.rs`.
3. A surface parsed but not relayed live (HTTP/2, WebSocket) or requested but
   downgraded (content retention) is stated as such.
4. Until a release exists, install is from source. The distribution section
   points at spec 021 and claims no published package.

### 3.2 Reference pages

1. `docs/reference/request.json` selects every spec with traceability and no
   capability catalog.
2. `scripts/reference.sh generate` compiles and indexes (generation needs a
   fresh ledger), writes the pages, and indexes again (the pages are hashed
   inputs); `scripts/reference.sh
   check` never writes and fails on any drift or a stale ledger.
3. A change that edits a spec regenerates the pages in the same change.
   Making `check` a required hosted job is a Statecraft profile parameter
   (`extra_required_jobs`) and needs the owner's profile approval; until then
   the check is local and part of this spec's verification.

## 4. Out of scope

Release and publication (spec 021), a rendered documentation site, changes to
approved specs, and changes to Statecraft-managed files.

## Verification

```verify:cli
.bin/spec-spine check --fail-on-warn
scripts/reference.sh check
python3 -c 'import re; u = re.search(r"const USAGE: &str = \"(.*?)\";", open("crates/wire-witness-cli/src/main.rs").read()).group(1); r = open("README.md").read(); [None for w in re.findall(r"--[a-z-]+|metadata-only|node-extra-ca-certs|aws-ca-bundle|existing-roots-bundle", u) if w in r or exit("README lacks " + w)]'
python3 -c 'r = open("README.md").read(); assert "0.29.0" not in r and "revision 21" not in r'
scripts/check-authored-content.sh
```
