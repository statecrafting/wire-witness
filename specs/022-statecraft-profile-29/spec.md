---
id: "022-statecraft-profile-29"
title: "Adopt Statecraft profile 29"
status: approved
implementation: complete
created: "2026-11-26"
summary: >
  Moves the setup profile from github-actions-rust revision 28 to 29 through
  the plan-bound render. Revision 29 adds the policy-isolation guard's
  new-crate class, so a new workspace crate that adds no dependency lands in
  one pull request. No policy path, pin or recorded parameter changes.
amends:
  - "019-statecraft-profile-28"
amends_verification:
  - "019-statecraft-profile-28"
depends_on:
  - "019-statecraft-profile-28"
obligations:
  - id: "R-1"
    kind: requirement
    text: "The setup profile is github-actions-rust revision 29, applied only through Statecraft's plan-bound apply; no managed file is edited by hand."
    anchor: "3-behavior"
  - id: "R-2"
    kind: requirement
    text: "Every check spec 019 keeps required stays required, and the engine pin stays spec-spine 0.32.0."
    anchor: "3-behavior"
  - id: "V-1"
    kind: verification
    text: "Profile, pin and boundary checks below pass, with the gate."
    anchor: "verification"
    inputs:
      - "specs/022-statecraft-profile-29/spec.md"
---

# 022: Adopt Statecraft profile 29

## 1. Purpose

Statecraft CLI main at `68832a2a97b9b53151a1f2e365a13ef65a5a6d39` registers
`github-actions-rust` revision 29 (Statecraft spec 073). The owner directed
its adoption in every Statecraft-governed repository on 2026-11-26.

Under revision 28 a workspace whose members are a glob cannot add a crate
without an admin merge: the manifest alone breaks the build, and with its
sources the guard reads policy beside other paths. Revision 29 classes such
a manifest and its sourceless lockfile entry as new-crate, ignored like
generated paths, while any registry or git package change stays policy.

## 2. Territory

This spec establishes no file. It amends spec 019 only by replacing its
revision-specific verification. Statecraft keeps ownership of every managed
file the plan renders. Its compiled shards,
`.statecraft/derived/spec-registry/by-spec/022-statecraft-profile-29.json` and
`.statecraft/derived/codebase-index/by-spec/022-statecraft-profile-29.json`,
are committed with it, as for every spec.

## 3. Behavior

1. Revision 29 replaces revision 28 through `init plan` and the matching
   plan-bound apply, plan
   `9084c7097d4e202b133d793a8ca11a5abaa588cfad5401e4a6053334ad56fd66`. It
   changes no policy path.
2. `scripts/statecraft/guard-policy-files.sh` gains the new-crate class;
   every other managed file changes only its revision header and profile
   identity.
3. No recorded setup parameter changes, and the pin stays `=0.32.0`.

## 4. Out of scope

Remote settings, release and publication.

## Verification

```verify:cli
scripts/check-authored-content.sh --self-test
python3 -c 'import json; p = json.load(open(".statecraft/setup/github-actions-rust.json")); assert p["revision"] == 29; assert p["parameters"]["require_ratified"] is True; assert {"job": "wire-witness-boundaries", "workflow": ".github/workflows/wire-witness-boundaries.yml"} in p["parameters"]["extra_required_jobs"]'
.bin/spec-spine --version | grep -qx 'spec-spine 0.32.0'
python3 -c 'import json; p = json.load(open(".statecraft/environment.json"))["pins"]; assert p["spec_spine"] == p["producer"]["version"] == "0.32.0"'
grep -q 'new crate (ignored)' scripts/statecraft/guard-policy-files.sh
make gate
```
