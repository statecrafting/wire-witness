---
id: "018-spec-spine-029-adoption-prerequisites"
title: "Adopt spec-spine 0.29 through the updated Statecraft producer"
status: approved
implementation: complete
created: "2026-10-10"
summary: >
  Migrates the exact governance engine through a Statecraft build that links
  the same released producer, adopting current profile 16 and all required checks.
amends:
  - "017-profile-15-and-required-boundary-checks"
amends_verification:
  - "017-profile-15-and-required-boundary-checks"
refines:
  - aspect: "exact-governance-engine-pin"
    unit: "spec-spine.toml"
  - aspect: "current-toolchain-status"
    unit: "README.md"
  - aspect: "migration-evidence"
    unit: "docs/codebase-audit-2026-10-10.md"
depends_on:
  - "017-profile-15-and-required-boundary-checks"
  - "016-binding-verification-engine"
---

# 018: Adopt spec-spine 0.29 through the updated Statecraft producer

## 1. Purpose

The owner requested the newest Statecraft CLI and spec-spine dependencies on
2026-10-10. The newest published spec-spine release is v0.29.0, published
2026-10-05. Statecraft main at
`4c12c5236b8e507f0c7569274862a4f5579d101d` still links 0.28.0 and
refuses a supported 0.29.0 migration. A separate Statecraft adoption candidate
moves its exact CLI and core dependencies together without changing product
code. Its exact signed source identity is recorded in the dated audit.

## 2. Territory

This amendment supersedes spec 017 section 3 item 2 and its version-specific
verification. It refines repository-owned configuration and documentation.
Statecraft retains ownership of generated policy, workflows, scripts and
environment records. Approved source, including specs 002 and 004 and their
interface pins, remains unchanged. No Statecraft source is claimed here.

## 3. Behavior

1. Use the separately validated Statecraft producer candidate for the supported
   `init plan --spine =0.29.0` and matching plan-bound apply. Record source,
   binary checksums, plan identity and disposition. Never hand-edit producer
   identity or managed-file digests.
2. Pin `required_version = "=0.29.0"` and install the regular repository-local
   `.bin/spec-spine` from that exact published release. Regenerate the corpus
   with that engine. Environment producer and engine identities must agree.
3. Adopt current profile 16 and its rendered remote desired-state document.
   Preserve the required workspace job, the extra required dependency
   boundary job, authored-content, coverage, unresolved-claim, signature,
   per-commit and owner-ratification checks.
4. Require local freshness, lint, coverage, ordinary coupling and locked
   workspace build, tests, clippy and formatting. Record hosted and local
   results separately. Draft acceptance is evidence, not owner ratification.
5. Keep the two pre-existing stale external interface pins visible as separate
   semantic integration work. They occur under both 0.28 and 0.29, and are not
   evidence that the engine migration failed. Their replacement requires an
   interface amendment and affected-behavior review rather than a digest-only
   update. Historical acceptance paths beyond specs 005 and 012 likewise need
   separate verification amendments; this migration does not rewrite approved
   specs to conceal that debt.
6. Use the existing public command surfaces. Optional 0.29 selected-content,
   affected-acceptance and consumer-contract capabilities are not introduced
   into the managed workflow by this version update.

## 4. Delivery boundary

The exact source producer can support a local migration before its upstream
adoption merges. That is a local candidate, not released Statecraft delivery.
Merge of the producer prerequisite, owner ratification and the protected
Environment approval remain separate. The known machine-level doctor refusal
also remains separate from code and corpus checks.

## 5. Revision 21 and spec-spine 0.31.0 refresh (2026-10-28)

The owner directed the Statecraft profile revision 21 and spec-spine 0.31.0
fleet rollout. Statecraft CLI main at
`167a02f3ce69472c029bdd64d0bcd04421e3df75` registers `github-actions-rust`
revision 21 (renderer identity
`29183723d833fc7b8c332dbac0e83edfb15eafe2ffc50f01c817f89a70785217`) and
links spec-spine-core 0.31.0. Its plan withholds the whole profile while the
pin is `=0.29.0`, so the pin move and the profile upgrade are one change.

1. The exact pin moves to `required_version = "=0.31.0"`, and the regular
   repository-local `.bin/spec-spine` is installed from that release. This
   supersedes item 2's version; environment producer and engine identities
   must agree at 0.31.0.
2. Profile revision 21 replaces profile 16 (item 3) through the supported
   `init plan` and plan-bound apply, plan
   `a2ffd828db53fee823c97f467f884f928eb90ecb59a7f68058af7f12131bdc2b`.
   Revisions 17 to 21 arrive together: governance documents read as base
   blobs, a required preflight job with job-level concurrency, failure
   summaries with the cache artifact drop, the AI review stall watchdog, and
   the AI review verdict published as a `statecraft-ai-review` check run.
   Managed files are never hand-edited.
3. Every check item 3 preserves stays required, with the preflight job in
   addition. No recorded setup parameter changes.

The version-specific verification below is updated to this refresh; sections
1 to 4 remain the historical record of the 0.29.0 and profile 16 migration.

## Verification

```verify:cli
scripts/check-authored-content.sh --self-test
scripts/check-authored-content.sh --identity HEAD HEAD
python3 -c 'import json; a = json.load(open(".claude/settings.json"))["attribution"]; assert a == {"commit": "", "pr": "", "sessionUrl": False}, a'
python3 -c 'import json; p = json.load(open(".statecraft/setup/github-actions-rust.json")); assert p["revision"] == 21; assert p["parameters"]["require_ratified"] is True; assert {"job": "wire-witness-boundaries", "workflow": ".github/workflows/wire-witness-boundaries.yml"} in p["parameters"]["extra_required_jobs"]; assert p["jobs"]["preflight"]["required"] is True'
.bin/spec-spine --version | grep -qx 'spec-spine 0.31.0'
python3 -c 'import json; p = json.load(open(".statecraft/environment.json"))["pins"]; assert p["spec_spine"] == p["producer"]["version"] == "0.31.0"; assert p["producer"]["checksum"] == "de002551d95919c9f136900a402f1ed5b462f6f8f7a5338cf54a2e70e89215e2"'
make boundaries
make gate
```
