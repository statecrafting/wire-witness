---
id: "012-profile-13-engine-location"
title: "Profile 13 uses the repository-local engine in .bin"
status: draft
implementation: complete
created: "2026-10-02"
summary: >
  Adopts Statecraft profile revision 13 with the existing exact engine pin,
  and replaces revision 12 acceptance with the recorded revision 13 identity.
amends:
  - "011-agent-attribution-and-governance-flows"
amends_verification:
  - "011-agent-attribution-and-governance-flows"
depends_on:
  - "011-agent-attribution-and-governance-flows"
---

# 012: Profile 13 uses the repository-local engine in .bin

## 1. Purpose

The owner requested a uniform fleet upgrade on 2026-10-02. Profile 13 is
rendered by statecraft-cli `8f718e2`, using released spec-spine 0.28.0.

## 2. Territory

Statecraft retains ownership of its managed scripts, workflows and policy.
This amendment changes the recorded profile identity and the acceptance
revision in spec 011, leaving its approved source unchanged.

## 3. Behavior

The profile revision is 13. Its engine is the regular executable
`.bin/spec-spine`, installed from the exact `spec-spine.toml` pin.
Attribution refusal and the separate owner ratification flow remain required.
Managed files come from `init plan` and plan-bound `init apply`, preserving
all recorded project parameters. Owner Environment review remains required.

## Verification

```verify:cli
scripts/check-authored-content.sh --self-test
scripts/check-authored-content.sh --identity HEAD HEAD
python3 -c 'import json; a = json.load(open(".claude/settings.json"))["attribution"]; assert a == {"commit": "", "pr": "", "sessionUrl": False}, a'
jq -e '.revision == 13 and .parameters.require_ratified == true' .statecraft/setup/github-actions-rust.json
.bin/spec-spine --version | grep -qx 'spec-spine 0.28.0'
make gate
```
