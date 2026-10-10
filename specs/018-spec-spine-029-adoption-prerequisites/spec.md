---
id: "018-spec-spine-029-adoption-prerequisites"
title: "Adopt spec-spine 0.29 after the Statecraft producer prerequisite"
status: draft
implementation: deferred
created: "2026-10-10"
summary: >
  Defines the evidence and producer prerequisites for adopting the newest
  published spec-spine without bypassing Statecraft pin-migration refusal or
  treating interface drift as compatible by assumption.
depends_on:
  - "017-profile-15-and-required-boundary-checks"
  - "016-binding-verification-engine"
---

# 018: Adopt spec-spine 0.29 after the Statecraft producer prerequisite

## 1. Purpose

The latest published spec-spine release observed on 2026-10-10 is v0.29.0,
published 2026-10-05. Wire Witness currently pins 0.28.0. Statecraft CLI main
`f9fcc888da8b0ba4a0581124420ffc4457700436` links spec-spine-core 0.28.0.
Its `init plan --spine =0.29.0` refuses before mutation with:

```text
requested =0.29.0 differs from linked producer 0.28.0
```

## 2. Territory

This is a deferred consumer adoption contract. It claims no Statecraft source,
managed file, approved upstream interface, or replacement engine pin. A later
implementation must declare any repository-owned units it changes before code
is written. Producer adoption belongs to Statecraft's governed corpus.

## 3. Behavior

1. Obtain an exact reviewed Statecraft build that links the adopted
   spec-spine-core release and supports that same exact migration target.
   A different binary on PATH, a successful standalone probe, or a manual
   manifest edit does not clear the producer prerequisite.
2. Recheck the newest published release at implementation time. A later
   release requires explicit evaluation rather than a floating latest pin.
3. Use Statecraft's supported plan-bound migration and the exact local engine;
   record source, version, checksums, plan identity, and apply disposition.
4. Regenerate registry and index with the new engine. Require freshness, lint,
   ownership coverage, unresolved-claim checks, coupling, per-commit validation,
   workspace checks, and current verification amendments.
5. Reconcile the two stale interface references found at the 2026-10-10 base:
   spec 002's canonical-keysort-json bootstrap digest and spec 004's Statecraft
   protected-evidence section digest. Inspect each cited change and test the
   affected behavior before proposing a new pin under separate spec authority.
   Never replace a digest just to make verification green.
6. Modernize remaining historical `.tooling/bin/spec-spine` acceptance paths
   through verification amendments while preserving approved source bytes and
   unrelated interface pins. Spec 016 already proposes this for spec 005 and
   remains draft; spec 017 proposes the current profile acceptance for 012.
7. Add a bounded typed consumer-contract declaration for the commands actually
   used by the profile when its supported workflow adopts the 0.29 capabilities
   and consumer-contract surface. Do not emulate its output with JSON parsing.
8. Record local, hosted, ratified, merged, released, and qualified states
   separately. A successful probe alone establishes no adoption or authority.

## 4. Probe evidence

A disposable worktree at merged base
`c777d25bb64880d01b5d93690e5a79193ada9f68` was evaluated with the regular
spec-spine 0.29.0 executable and an experimental exact pin. Compile, index,
freshness, lint, and ownership coverage passed. Interface verification returned
two stale references, also present under 0.28.0. These are existing integration
drift rather than a demonstrated 0.29 regression. The probe does not change the
repository's supported 0.28.0 pin or Statecraft producer identity.

## 5. Out of scope

Changing Statecraft's producer dependency from this repository, bypassing a
pin refusal, ratifying spec 016 or 017, rewriting approved specs, provider
activity, remote configuration, release, and consumer qualification.

## Verification

Acceptance is deferred until the producer prerequisite is satisfied. The
implementation must add executable acceptance for the exact adopted version
and demonstrated interface compatibility before claiming completion.
