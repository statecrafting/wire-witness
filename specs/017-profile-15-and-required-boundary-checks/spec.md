---
id: "017-profile-15-and-required-boundary-checks"
title: "Profile 15 preserves required workspace and dependency checks"
status: approved
implementation: complete
created: "2026-10-10"
summary: >
  Adopts the current Statecraft setup profile with the supported exact engine
  pin, makes dependency-boundary verification a required CI job, and replaces
  historical setup prose with current operator instructions.
amends:
  - "012-profile-13-engine-location"
amends_verification:
  - "012-profile-13-engine-location"
establishes:
  - ".github/workflows/wire-witness-boundaries.yml"
  - "docs/codebase-audit-2026-10-10.md"
refines:
  - aspect: "portable-boundary-check-invocation"
    unit: "Makefile"
  - aspect: "current-operator-documentation"
    unit: "README.md"
depends_on:
  - "001-boundaries-and-authority"
  - "011-agent-attribution-and-governance-flows"
  - "012-profile-13-engine-location"
---

# 017: Profile 15 preserves required workspace and dependency checks

## 1. Purpose

Refresh setup using Statecraft CLI main commit
`f9fcc888da8b0ba4a0581124420ffc4457700436`, whose registered
`github-actions-rust` revision is 15. Statecraft has no published release;
this names source provenance rather than inventing a release version.

The existing local `make code` runs `make boundaries`, but the managed CI code
job invokes only `scripts/statecraft/gate.sh code`. Dependency boundaries
therefore have local evidence without an equivalent required hosted check.

## 2. Territory

This spec establishes a repository-owned reusable boundary workflow and a dated
audit. It refines README operator documentation. Statecraft retains ownership
of generated policy, workflows, scripts, manifest records, and ignore fragments.
Approved specs, including spec 012, remain byte for byte unchanged.

## 3. Behavior

1. Plan and apply the profile through the exact Statecraft build using a
   plan identity, preserving recorded governance and review parameters.
2. Keep `required_version = "=0.28.0"` and the regular repository-local
   `.bin/spec-spine`. Current Statecraft links spec-spine-core 0.28.0 and
   refuses a supported `--spine =0.29.0` migration. Spec 018 records that
   prerequisite without bypassing it.
3. Declare `wire-witness-boundaries` through `ci.extra_required_jobs`, naming
   `.github/workflows/wire-witness-boundaries.yml`. The reusable workflow
   checks out the candidate with read-only permissions and runs the existing
   locked `scripts/check-boundaries.sh` using the pinned Rust toolchain.
   Failure, cancellation, or skipping must fail the generated aggregate gate.
   Both local and hosted checks use `make boundaries`, which invokes Bash
   explicitly because the existing checker uses `set -o pipefail`. Ubuntu
   `/bin/sh` does not support that option.
4. Retain the profile's required Rust build, tests, clippy, and formatting,
   together with coverage, unresolved-claim refusal, authored-content checks,
   signed-commit checks, per-commit checks, and owner-gated ratification.
5. README describes implemented runtime surfaces and actual local commands.
   The dated audit distinguishes local evidence, merged-base CI, candidate CI,
   interface drift, release, and consumer qualification.

The verification below replaces spec 012's profile-13-specific acceptance
through `amends_verification`. Running draft acceptance validates the proposal;
it does not approve it. Profile authority changes still require the owner's
protected Environment approval. No remote setting is changed by this spec.

## 4. Out of scope

Product behavior, provider calls, dependency upgrades, interface repinning,
owner ratification, merge, publication, release, and consumer qualification.

## Verification

```verify:cli
scripts/check-authored-content.sh --self-test
scripts/check-authored-content.sh --identity HEAD HEAD
python3 -c 'import json; a = json.load(open(".claude/settings.json"))["attribution"]; assert a == {"commit": "", "pr": "", "sessionUrl": False}, a'
python3 -c 'import json; p = json.load(open(".statecraft/setup/github-actions-rust.json")); assert p["revision"] == 15; assert p["parameters"]["require_ratified"] is True; assert {"job": "wire-witness-boundaries", "workflow": ".github/workflows/wire-witness-boundaries.yml"} in p["parameters"]["extra_required_jobs"]'
.bin/spec-spine --version | grep -qx 'spec-spine 0.28.0'
make boundaries
make gate
```
