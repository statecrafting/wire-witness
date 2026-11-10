---
id: "020-readme-describes-functionality"
title: "The README describes what wire-witness does"
status: approved
implementation: complete
created: "2026-11-10"
summary: >
  Rewrites README.md as a user guide to wire-witness: what it records, its
  guarantees, install, a quick start taken from a real run, usage, trust
  variables, exit codes, concepts and limits. Toolchain and governance
  chronology leaves the README for the specs and the dated audit.
refines:
  - aspect: "user-guide"
    unit: { kind: file, path: "README.md" }
depends_on:
  - "017-profile-15-and-required-boundary-checks"
  - "018-spec-spine-029-adoption-prerequisites"
  - "019-statecraft-profile-28"
obligations:
  - id: "R-1"
    kind: requirement
    text: "Every command, option value, environment variable, exit code, schema identifier and output path the README names exists in the source, and a surface the live runtime does not implement is stated as not implemented."
    anchor: "3-behavior"
  - id: "R-2"
    kind: requirement
    text: "Example output in the README is taken from a real run of the documented command, abridged but not invented."
    anchor: "3-behavior"
  - id: "V-1"
    kind: verification
    text: "The README names every option and value the built executable prints in its usage text, carries no engine pin chronology, and the gate passes."
    anchor: "verification"
    inputs:
      - "specs/020-readme-describes-functionality/spec.md"
      - "README.md"
      - "crates/wire-witness-cli/src/main.rs"
---

# 020: The README describes what wire-witness does

## 1. Purpose

The README written under specs 017 and 018 described the crates accurately
but spent most of its length on engine pins, setup-profile revisions and
commit identities. It had no install path, usage, exit codes or output
description. A reader learned how the repository was governed before learning
what the tool does.

## 2. Territory

This spec refines `README.md`, which specs 017 and 018 already refine, for
its structure as a user guide. The facts specs 017 and 018 required remain
true in the new text or in the documents it links: implemented surfaces,
actual local commands, and the exact engine (`spec-spine.toml`).

Ratification follows spec 011 section 3.2. A change that moves a
specification to approved is a ratification event, and coupling refuses a
changed path a draft specification owns, so a spec that owns the paths a pull
request changes is ratified in that pull request. The ratification event
requires the protected `statecraft-review-exception` environment, which
ci-gate enforces for the run; `status: approved` is the content of that event,
not evidence of it.

## 3. Behavior

1. Order: purpose and authority statement, guarantees, install, quick start,
   usage with trust variables and exit codes, concepts, limits, repository
   layout, development, license.
2. Claims are checked against source and the built executable: the usage
   text the executable prints when run without a command; options, trust
   variables and exit codes against `standalone_host.rs`; attempt layout,
   retention and custody against `live_runtime.rs`; and routing against the
   proxy's `live.rs`.
3. The quick start's output comes from a real run on 2026-11-10 of the
   documented `curl` command against `api.anthropic.com` without a key. The
   absent-capture example comes from a real run of Node's built-in `fetch`
   without `NODE_USE_ENV_PROXY`.
4. Surfaces parsed but not relayed live (HTTP/2, WebSocket) and requests the
   runtime downgrades (content retention) are stated as such.

## 4. Out of scope

Generated reference documentation, release and distribution, and changes to
approved specs.

## Verification

```verify:cli
.bin/spec-spine check --fail-on-warn
python3 -c 'import re, subprocess, sys; p = subprocess.run(["cargo", "run", "-q", "--locked", "-p", "wire-witness-cli"], capture_output=True, text=True); u = p.stderr; (p.returncode == 3 and "usage: wire-witness run" in u) or sys.exit("the executable did not print its usage: exit %d, stderr %r" % (p.returncode, u[:300])); r = open("README.md").read(); missing = [w for w in sorted(set(re.findall(r"--[a-z-]+|[a-z]+(?:-[a-z]+)+", u.split("usage:", 1)[1]))) if w not in r]; missing and sys.exit("README lacks " + ", ".join(missing))'
python3 -c 'r = open("README.md").read(); assert "0.29.0" not in r and "revision 21" not in r'
scripts/check-authored-content.sh
```
