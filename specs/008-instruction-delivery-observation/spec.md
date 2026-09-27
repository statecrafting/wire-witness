---
id: "008-instruction-delivery-observation"
title: "Instruction delivery observation"
status: draft
implementation: pending
created: "2026-09-27"
summary: >
  Defines a content-gated, offline comparison that reports whether exact
  operator-supplied instruction bytes were observed in a complete outbound
  provider request, without turning one observation into adapter
  qualification, policy, or authority.
establishes:
  - { kind: module, id: "wire_witness_core::instruction_observation", planned: true }
depends_on:
  - "002-exchange-record-and-normalization"
  - "003-redaction-custody-and-retention"
  - "006-standalone-host"
obligations:
  - id: "I-1"
    kind: invariant
    text: "An instruction observation is testimony about one exact captured request and never establishes an adapter-wide delivery guarantee, qualification, policy, or authority."
    anchor: "3-1-observation-not-qualification"
  - id: "I-2"
    kind: invariant
    text: "The comparison target and matching mode are supplied before observation and are never inferred from repository state, filenames, prompt meaning, or nearby text."
    anchor: "3-2-predeclared-comparison-plan"
  - id: "R-1"
    kind: requirement
    text: "Present and absent results require a complete decoded request component and exact-byte comparison; every incomplete, disabled, redacted, or unsupported case is unknown with a reason."
    anchor: "3-3-three-state-result"
  - id: "R-2"
    kind: requirement
    text: "The durable result binds the target digest and length, exchange digest, component identity, match count, completeness, and findings without retaining target or observed instruction bytes."
    anchor: "3-4-durable-result"
  - id: "V-1"
    kind: verification
    text: "The draft compiles, its planned pure-core module resolves, and no implementation file is introduced."
    anchor: "verification"
    inputs:
      - "specs/008-instruction-delivery-observation/spec.md"
---

# 008: Instruction delivery observation

## 1. Purpose

Allow an operator to answer a bounded question such as whether the exact bytes
of a selected instruction document appeared in one provider-bound request.
The answer is tied to that exchange and its capture posture. It is not a claim
about every request, an undocumented harness contract, or provider behavior.

## 2. Territory

This spec owns the planned pure
`wire_witness_core::instruction_observation` module and schema
`wire-witness.instruction-observation/1`. The host may supply transient bytes
to the module, but the module performs no file access, provider call, transport
capture, durable write, policy decision, or adapter qualification.

Spec 002 continues to own provider request normalization and exchange digests.
Spec 003 owns retention and redaction. Spec 006 owns host-side acquisition of
an explicitly named comparison target. A consumer owns any conclusion drawn
from the resulting testimony.

## 3. Behavior

### 3.1 Observation, not qualification

An observation answers only whether one predeclared target was found in one
identified outbound request component under one exact comparison mode. The
result carries the exchange digest and binding. It never claims that a CLI
always expands an instruction file, that a provider followed the instruction,
or that a harness, version, account, or configuration is qualified.

The witness does not invoke a provider to obtain an observation. A live
measurement occurs only when a separately authorized child produces traffic.
Local acceptance uses synthetic requests.

### 3.2 Predeclared comparison plan

Before the child starts, the operator supplies a comparison plan with:

1. a stable probe name;
2. the exact UTF-8 target bytes held transiently in protected process memory;
3. `file-bytes-sha256` digest and byte length derived from those bytes;
4. an exact request component selector, such as one decoded system-instruction
   text field; and
5. matching mode `exact-contiguous-utf8-v1`.

The target is never inferred from the working directory, repository, an
`AGENTS.md` filename, a configured import path, similar language, or a digest
found after capture. The plan is immutable for the attempt. Duplicate probe
names, a digest mismatch, invalid UTF-8, an empty target, or an unsupported
selector refuses before child spawn.

Comparison operates on the decoded component produced by the exact provider
normalizer. It does not search raw JSON serialization, HTTP headers, response
content, unrelated messages, tool output, or every retained byte by default.

### 3.3 Three-state result

The result status is one of:

- `observed-present`: the exact target byte sequence occurred at least once in
  the complete selected request component;
- `observed-absent`: the complete selected component was observed and decoded,
  and the exact target sequence did not occur; or
- `unknown`: the comparison could not justify either claim.

`observed-absent` requires a complete request and a complete selected
component. Disabled content observation, metadata-only capture, redaction that
touches the selected component, truncation, malformed input, an unsupported
provider shape, selector absence, or a capture gap produces `unknown` with a
closed reason. None is converted to absence.

The result records the number and byte offsets of non-overlapping exact
matches. Those offsets identify the decoded selected component only. They do
not claim positions in the original HTTP body unless the normalizer provides a
separately verified source map.

### 3.4 Durable result

The durable `wire-witness.instruction-observation/1` record contains:

1. the exchange binding, sequence, and exchange digest;
2. probe name, target digest, target byte length, selector, and matching mode;
3. result status, match count, and decoded-component byte offsets when known;
4. request and selected-component completeness;
5. the exact normalizer and witness producer identities; and
6. ordered findings and unknown reasons.

The durable record never contains the target bytes, matched bytes, surrounding
text, a reconstructed instruction document, credentials, or provider response
content. The transient target and decoded comparison buffer are cleared after
the result is formed. Content retention under spec 003 does not silently add
content to this result schema.

Canonical bytes use canonical-keysort-json and the named construction
`wire-witness.instruction-observation/1+keysort-json+sha256`. The record digest
is carried beside the bytes it identifies.

## 4. Observable negative cases

| Case | Expected |
|---|---|
| The target appears in a tool result but not the selected system component | `observed-absent` for a complete system component; unrelated components are not searched. |
| Capture stops before the request completes | `unknown` with the capture-gap reason, never absent. |
| Redaction changes bytes inside the selected component | `unknown`; the witness does not search a lossy representation and claim absence. |
| A similar paraphrase appears | Absent under exact-byte matching; semantic similarity is not inferred. |
| The same target appears twice | Present with count two and both decoded-component offsets. |
| One successful session is presented as a CLI guarantee | Refused by the authority boundary; the record describes one exchange only. |

## 5. Out of scope

Provider invocation, instruction effectiveness, semantic similarity, prompt
reconstruction, adapter qualification, policy changes, evidence admission,
publication of captured instructions, and provider-wide claims are out of
scope.

## Verification

```verify:cli
./.tooling/bin/spec-spine check --fail-on-warn
./.tooling/bin/spec-spine index check --fail-on-unresolved
test ! -e crates/wire-witness-core/src/instruction_observation.rs
```
