---
id: "001-boundaries-and-authority"
title: "Boundaries, authority, and dependency layout"
status: draft
implementation: pending
created: "2026-09-26"
summary: >
  Defines wire-witness as a library plus isolated hosts that capture provider
  wire exchanges and produce testimony without granting authority. Assigns
  ecosystem responsibilities, fixes the three-crate dependency layout, and
  reserves captured-output training, hosting, and publication to the owner.
establishes:
  - "Cargo.toml"
  - "Cargo.lock"
  - "Makefile"
  - "scripts/check-authored-content.sh"
  - { kind: crate, id: "wire-witness-core", planned: true }
depends_on:
  - "000-bootstrap"
obligations:
  - id: "I-1"
    kind: invariant
    text: "Every output is testimony about an observed exchange and never authorizes, admits, evaluates, or changes a plan or policy."
    anchor: "3-1-testimony-never-authority"
  - id: "I-2"
    kind: invariant
    text: "Supervised capture repeats its supplied attempt binding verbatim and never infers, mints, or alters that binding."
    anchor: "3-2-binding-not-correlation"
  - id: "R-1"
    kind: requirement
    text: "The pure core performs no I/O, reads no clock, forbids unsafe, and depends on no async runtime, HTTP stack, or TLS stack."
    anchor: "3-4-dependency-rules"
  - id: "V-1"
    kind: verification
    text: "The corpus gate and authored-content check hold while all product crates remain planned and absent."
    anchor: "verification"
    inputs:
      - "specs/001-boundaries-and-authority/spec.md"
      - "Cargo.toml"
      - "scripts/check-authored-content.sh"
---

# 001: Boundaries, authority, and dependency layout

## 1. Purpose

Fix the product boundary before implementation. A coding harness reports what
it believes happened. wire-witness records what crossed the provider boundary
and turns that observation into evidence. The record is a claim with custody,
not authority over any consumer.

## 2. Territory

This spec owns the virtual workspace, governance Makefile, authored-content
check, and the planned `wire-witness-core` crate. That crate will contain pure
schema, normalization, redaction, and digest logic. Later specs refine its
behavior without taking ownership of the crate.

The product has three planned crates:

| Crate | Responsibility |
|---|---|
| `wire-witness-core` | Pure record types, normalization, redaction, canonical bytes, and digests. |
| `wire-witness-proxy` | Child-facing TLS, HTTP, SSE, and WebSocket interception. |
| `wire-witness-cli` | Standalone host and supervised sidecar process. |

No crate directory or source file is created in this session.

## 3. Behavior

### 3.1 Testimony, never authority

1. Capture, normalization, redaction, custody, and attestation production are
   this product's responsibilities.
2. A record states what the witness observed and which limitations applied.
3. Nothing emitted by this product authorizes an effect, admits evidence,
   evaluates an answer, changes a plan, or changes policy.
4. Provider and harness claims remain attributed claims. Observation does not
   make them trusted facts.

### 3.2 Binding, not correlation

1. In supervised mode, the supervisor supplies `AttemptBinding { run_id,
   attempt, effect_id }` before spawning the child.
2. The witness repeats all three values verbatim on every record. It never
   infers a run from time, process ancestry, content, or harness identifiers.
3. Standalone mode mints an identity whose kind is `unsupervised`. It never
   presents that identity as a statecraft run or attempt.

### 3.3 Library plus isolated hosts

1. The capture engine is a library with two hosts: a standalone CLI and a
   sidecar spawned once per supervised attempt.
2. The sidecar is out of process. A potentially hostile child therefore cannot
   drive HTTP, TLS, or WebSocket parsing inside the trusted supervisor.
3. statecraft-cli may link and pin the interface in spec 005 the same way its
   D-06 pins `spec-spine-core`; linkage does not transfer authority.

### 3.4 Dependency rules

1. `wire-witness-core` performs no I/O, reads no clock, and declares
   `unsafe_code = "forbid"`.
2. The core has no direct or transitive async runtime, HTTP stack, TLS stack,
   process host, or provider SDK.
3. `wire-witness-proxy` may depend on the core and transport stacks.
4. `wire-witness-cli` may depend on the core and proxy. Neither the core nor
   proxy depends on the CLI.
5. Crate ownership is one-way: each crate has exactly one owning spec, while
   later specs may refine behavior through typed edges.

### 3.5 Ecosystem responsibilities

| Component | Owns | Does not own |
|---|---|---|
| wire-witness | Capture, normalization, redaction, custody, attestation production | Supervision, admission, policy, decisions, governance |
| statecraft-cli | Supervision, attempt binding, admission, policy, confinement | Provider-wire interpretation |
| rustev | Decisions and evaluation | Capture custody or action authority |
| attest-ledger | Hash-linked records | Canonicalization or admission |
| canonical-keysort-json | Canonical bytes | Record meaning or custody |
| spec-spine | Governance and contract identity | Product runtime behavior |

## 4. Out of scope

- Training or fine-tuning on captured outputs. This is reserved to the owner;
  provider terms may restrict using outputs to build competing models.
- Any hosted service.
- Publication, release, deployment, or remote repository configuration.
- Product implementation in this bootstrap session.

## 5. Observable negative cases

| Case | Expected |
|---|---|
| A consumer treats an attestation as permission to execute | Refused by the consumer boundary; the attestation carries no grant. |
| The witness guesses a run id from a child process or timestamp | No supervised record is produced; a binding must be supplied. |
| Standalone capture is labeled as a statecraft attempt | Invalid record; its binding kind must be `unsupervised`. |
| The core gains Tokio, Hyper, rustls, process, filesystem, or clock use | The dependency and boundary checks fail. |
| A child-facing parser is linked into the supervisor | Refused in review; the parser belongs in the sidecar process. |

## Verification

Acceptance for this draft is declarative: no product implementation exists.
The governance files are owned, the three-crate layout is explicit, and the
planned core claim is accepted only if the pinned index represents it without
requiring an empty crate.

```verify:cli
make gate
scripts/check-authored-content.sh
test ! -d crates
```
