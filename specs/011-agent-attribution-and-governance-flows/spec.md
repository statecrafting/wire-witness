---
id: "011-agent-attribution-and-governance-flows"
title: "Agent attribution is refused and ratification is owner-gated"
status: draft
implementation: complete
created: "2026-09-30"
summary: >
  Refuses agent attribution and known agent commit identities, suppresses
  provider attribution at its source, and adopts the two-flow Statecraft
  profile in which implementation and owner ratification remain separate.
extends:
  - { spec: "001-boundaries-and-authority", unit: "scripts/check-authored-content.sh", nature: additive }
establishes:
  - ".claude/settings.json"
depends_on:
  - "001-boundaries-and-authority"
obligations:
  - id: "R-1"
    kind: requirement
    text: "Repository-authored text and commit identities refuse known agent attribution while human identities remain allowed."
    anchor: "3-1-attribution-and-identity-refusal"
  - id: "R-2"
    kind: requirement
    text: "A specification becoming approved requires the owner exception independently of implementation review."
    anchor: "3-2-two-flow-ratification"
  - id: "V-1"
    kind: verification
    text: "The checker self-test and Statecraft profile policy prove attribution refusal, identity refusal, and owner-gated ratification."
    anchor: "verification"
    inputs:
      - "scripts/check-authored-content.sh"
      - ".claude/settings.json"
      - ".statecraft/setup/github-actions-rust.json"
---

# 011: Agent attribution is refused and ratification is owner-gated

## 1. Purpose

Repository history is published under human authority. Agent session links,
agent attribution, and known agent author or committer addresses must not be
presented as that authority. Implementation review and owner ratification are
also distinct flows: completing work does not approve its governing spec.

## 2. Territory

This spec establishes `.claude/settings.json` and extends
`scripts/check-authored-content.sh`. The Statecraft setup profile owns its
rendered workflows, policy, and scripts under `scripts/statecraft/`.

## 3. Behavior

### 3.1 Attribution and identity refusal

1. Claude Code attribution for commits, pull requests, and session URLs is
   disabled in repository settings.
2. Authored text refuses agent session links, agent co-author trailers,
   generated-by footers, attribution markers, and agent session trailers.
3. The checker refuses commits whose author or committer address is a known
   agent identity. It judges addresses case-insensitively and does not judge
   display names.
4. Human co-author trailers and human commit addresses remain allowed.
5. The no-argument checker judges `BASE_SHA..HEAD` identities when `BASE_SHA`
   names a commit. Explicit `--identity BASE HEAD` provides the same check.

### 3.2 Two-flow ratification

1. Implementation may complete while its specification remains draft.
2. A change that moves a specification to approved is a ratification event.
3. Ratification requires the protected `statecraft-review-exception`
   environment, independently of the implementation review verdict.
4. Coupling refuses implementation changes owned by a draft specification
   when `governance.require_ratified` is enabled.

The profile migration that introduces this rule is judged by the revision 9
gate read from its trusted base. After that migration lands, revision 12 judges
later candidates. Spec 011 deliberately remains draft until the owner performs
the separate ratification flow; until then, later changes to paths it owns are
refused.

## 4. Out of scope

- Rewriting existing branch history.
- Restricting human contributors to an allowlist.
- Granting owner approval or ratifying this specification.

## Verification

```verify:cli
scripts/check-authored-content.sh --self-test
scripts/check-authored-content.sh --identity HEAD HEAD
python3 -c 'import json; a = json.load(open(".claude/settings.json"))["attribution"]; assert a == {"commit": "", "pr": "", "sessionUrl": False}, a'
jq -e '.revision == 12 and .parameters.require_ratified == true' .statecraft/setup/github-actions-rust.json
```
