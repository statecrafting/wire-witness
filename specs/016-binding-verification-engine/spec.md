---
id: "016-binding-verification-engine"
title: "Binding protocol verification uses the repository-local engine"
status: draft
implementation: n-a
created: "2026-10-10"
summary: >
  Replaces spec 005 acceptance commands with their profile 13 engine paths,
  preserving its approved source and pinned interface section digests.
amends:
  - "005-binding-and-sidecar-protocol"
amends_verification:
  - "005-binding-and-sidecar-protocol"
depends_on:
  - "005-binding-and-sidecar-protocol"
  - "012-profile-13-engine-location"
---

# 016: Binding protocol verification uses the repository-local engine

## 1. Purpose

Spec 005's original acceptance invokes `./.tooling/bin/spec-spine`. Profile 13
installs the pinned engine at `.bin/spec-spine`. This amendment supplies
acceptance using the current engine without editing the approved spec that
Statecraft pins.

## 2. Territory

This amendment replaces only spec 005's verification commands. Its approved
source remains byte for byte unchanged, including the interface references,
protocol behavior, and section digests. It claims no product code or managed
profile files.

## 3. Behavior

Upon owner ratification, the verification block below replaces spec 005's
original verification block through `amends_verification`. Both spec-spine
commands use the regular repository-local `.bin/spec-spine` executable and
the existing exact `spec-spine.toml` pin. The corpus freshness check, both
Statecraft interface-pin checks, and the locked CLI tests remain required.

Spec-spine resolves this verification amendment in the candidate checkout
even while it is draft. Running that candidate acceptance validates the
proposal; it does not ratify the amendment.

The interface check reads the explicitly named local Statecraft checkout.
An unavailable checkout or a mismatched interface pin remains a verification
failure. This amendment changes neither interface pins nor their meaning.

## 4. Out of scope

Constitution activation, profile 14 migration, protocol changes, release,
publication, and qualification are out of scope. This draft becomes authority
only through owner ratification.

## Verification

```verify:cli
.bin/spec-spine check --fail-on-warn
.bin/spec-spine interface verify --spec 005 --export statecraft-cli=/Users/bart/DevWork/statecraft-cli
cargo test -p wire-witness-cli --locked
```
