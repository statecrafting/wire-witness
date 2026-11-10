---
id: "019-statecraft-profile-28"
title: "Adopt Statecraft profile 28 and spec-spine 0.32 with a supply-chain policy"
status: approved
implementation: complete
created: "2026-11-10"
summary: >
  Moves the exact engine pin from 0.31.0 to 0.32.0 and the setup profile from
  github-actions-rust revision 21 to 28, in two pull requests: first the pin,
  a committed cargo-deny policy the lockfile passes, and the other
  prerequisites revisions 22 and 24 add; then the plan-bound profile render,
  which changes no policy path.
amends:
  - "018-spec-spine-029-adoption-prerequisites"
amends_verification:
  - "018-spec-spine-029-adoption-prerequisites"
establishes:
  - "deny.toml"
extends:
  - { spec: "001-boundaries-and-authority", unit: { kind: section, file: "Cargo.toml", anchor: "workspace.package" }, nature: additive }
  - { spec: "001-boundaries-and-authority", unit: { kind: section, file: "Cargo.toml", anchor: "workspace.dependencies" }, nature: additive }
  - { spec: "001-boundaries-and-authority", unit: "Cargo.lock", nature: additive }
  - { spec: "017-profile-15-and-required-boundary-checks", unit: ".github/workflows/wire-witness-boundaries.yml", nature: additive }
  - { spec: "002-exchange-record-and-normalization", unit: { kind: section, file: "crates/wire-witness-core/src/exchange.rs", anchor: "exchange-record-and-normalization" }, nature: "behavior-preserving lint conformance at MSRV 1.88" }
refines:
  - aspect: "exact-governance-engine-pin"
    unit: "spec-spine.toml"
  - aspect: "supply-chain-policy-hashed-input"
    unit: "spec-spine.toml"
depends_on:
  - "001-boundaries-and-authority"
  - "011-agent-attribution-and-governance-flows"
  - "017-profile-15-and-required-boundary-checks"
  - "018-spec-spine-029-adoption-prerequisites"
obligations:
  - id: "R-1"
    kind: requirement
    text: "The setup profile is github-actions-rust revision 28, applied only through Statecraft's plan-bound apply; no managed file is edited by hand."
    anchor: "3-behavior"
  - id: "R-2"
    kind: requirement
    text: "The engine pin, the installed engine and the recorded producer all agree at spec-spine 0.32.0."
    anchor: "3-behavior"
  - id: "R-3"
    kind: requirement
    text: "cargo deny check passes against the committed deny.toml with no advisory, ban, license or source error."
    anchor: "3-behavior"
  - id: "R-4"
    kind: requirement
    text: "Every check spec 018 keeps required stays required, and the wire-witness-boundaries job remains a declared extra required job."
    anchor: "3-behavior"
  - id: "V-1"
    kind: verification
    text: "Profile, pin, policy and boundary checks below pass, with the gate."
    anchor: "verification"
    inputs:
      - "specs/019-statecraft-profile-28/spec.md"
      - "deny.toml"
      - "Cargo.lock"
---

# 019: Adopt Statecraft profile 28 and spec-spine 0.32 with a supply-chain policy

## 1. Purpose

Statecraft CLI main at `23c290bc5c7053b30be7bd171962d104571746bc` registers
`github-actions-rust` revision 28 and links spec-spine-core 0.32.0. The
repository is on revision 21 with pin `=0.31.0`. The owner directed adoption
of the current profile on 2026-11-10.

Its `init plan --spine =0.32.0` withheld the whole profile for one unmet
prerequisite: revision 22 requires a committed `deny.toml`, and Statecraft
never writes a supply-chain policy.

Revision 22 also renders a policy-isolation guard. Where the base carries
none, the candidate's copy runs, so the pull request that renders the
profile is refused if it also changes `deny.toml`, a Cargo manifest or
lockfile, or `spec-spine.toml`. Adoption therefore takes two pull requests,
as rustev's did.

## 2. Territory

This spec establishes `deny.toml`. It extends spec 001's workspace manifest
and lockfile, and spec 017's boundary workflow, for the prerequisites below.
It refines the engine pin and adds `deny.toml` to the index's hashed inputs.
It amends spec 018 only by replacing its version-specific verification.
Statecraft keeps ownership of every managed file the plan renders; approved
specs remain byte for byte unchanged.

## 3. Behavior

1. `deny.toml` denies yanked crates, unknown registries and git sources, and
   wildcard dependencies, and allows the licenses the dependency graph uses
   (Apache-2.0, BSD-3-Clause, ISC, MIT, Unicode-3.0).
2. Against that policy the lockfile carried six advisories, each fixed by a
   semver-compatible update: rustls 0.23.46 (RUSTSEC-2026-0346, 0347, 0348,
   0349), rustls-webpki 0.103.16 (RUSTSEC-2026-0382) and time 0.3.47
   (RUSTSEC-2026-0009). time 0.3.47 requires Rust 1.88, so the workspace
   `rust-version` moves from 1.85 to 1.88; the pinned toolchain is 1.98.1.
3. Internal path dependencies declare the exact workspace version
   (`=0.1.0`). cargo-deny treats a path dependency without a version as a
   wildcard in a publishable crate, and a registry release needs it.
4. At MSRV 1.88 clippy requires let-chains for two nested `if let` blocks
   and `as_chunks` for the fixed-size chunking in the SHA-256 helper of
   `exchange.rs`. Both rewrites preserve behavior; spec 002's tests are the
   evidence.
5. The boundary workflow declares the boolean `workflow_call` input
   `post_merge` that revision 24 requires of an extra required job, and
   skips `make boundaries` when it is true.
6. Policy step: the pin moves to `required_version = "=0.32.0"` and
   `deny.toml` joins the index's hashed inputs, together with items 1 to 5.
   No managed file changes; the managed installer already reads the pin.
7. Profile step: revision 28 replaces revision 21 through `init plan --spine
   =0.32.0 --verify-local` and the matching plan-bound apply, which records
   the 0.32.0 producer in `.statecraft/environment.json`. It changes no
   policy path. Applied as plan
   `spine-init:5383b6346b1c7f451496cccae17d732dfb34fa00cef833a28178f4d1a675453f`. Revisions 22 to 28 arrive together: required
   supply-chain, workflow-lint, sg-scan and secret-scan jobs; the
   policy-isolation guard; the managed review guide; post-merge push
   handling; the optional free-disk step (off); AI review finding severity;
   and version-only classes for manifests and the pin.
8. No recorded setup parameter changes.

From this revision on, a change to `deny.toml`, any `Cargo.toml` or
`Cargo.lock`, or `spec-spine.toml` beside other paths is refused by the
policy-isolation guard and must be its own pull request.

## 4. Out of scope

Remote settings, owner ratification, release and publication.

## Verification

```verify:cli
scripts/check-authored-content.sh --self-test
python3 -c 'import json; p = json.load(open(".statecraft/setup/github-actions-rust.json")); assert p["revision"] == 28; assert p["parameters"]["require_ratified"] is True; assert {"job": "wire-witness-boundaries", "workflow": ".github/workflows/wire-witness-boundaries.yml"} in p["parameters"]["extra_required_jobs"]'
.bin/spec-spine --version | grep -qx 'spec-spine 0.32.0'
python3 -c 'import json; p = json.load(open(".statecraft/environment.json"))["pins"]; assert p["spec_spine"] == p["producer"]["version"] == "0.32.0"'
cargo deny check
cargo test -p wire-witness-core --locked
make boundaries
make gate
```
