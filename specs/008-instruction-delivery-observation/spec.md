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
  - { kind: module, id: "wire_witness_cli::instruction_observation", planned: true }
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
    text: "The comparison target, normalized component selector, and matching mode are supplied before observation and are never inferred from repository state, filenames, prompt meaning, or nearby text."
    anchor: "3-2-predeclared-comparison-plan"
  - id: "R-1"
    kind: requirement
    text: "Present and absent results require a complete decoded request component and exact-byte comparison; every incomplete, disabled, redacted, or unsupported case is unknown with a reason."
    anchor: "3-3-three-state-result"
  - id: "R-2"
    kind: requirement
    text: "The durable result binds the target digest and length, exchange digest, selected component identity and digest, match count, completeness, and findings without retaining target or observed instruction bytes."
    anchor: "3-4-durable-result"
  - id: "R-3"
    kind: requirement
    text: "Request components are decoded by this spec's own closed decoder over the complete outbound request body, which neither changes wire-witness.exchange/1 nor searches a component that mandatory redaction touched."
    anchor: "3-5-request-component-decoding"
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
`wire_witness_core::instruction_observation` module, the planned
`wire_witness_cli::instruction_observation` host integration, and schema
`wire-witness.instruction-observation/1`. The CLI acquires the explicitly named
target before child spawn and supplies transient bytes to the pure module. The
module performs no file access, provider call, transport capture, durable
write, policy decision, or adapter qualification.

The module also owns the closed request-component decoder in section 3.5. That
decoder is new surface: spec 002's `wire-witness.exchange/1` records identity,
usage, cost, transport, and response events, and has no request-component
model. The decoder does not add fields to that schema, alter its canonical
bytes, or change its digest construction; it reads the exchange's provider
family and transport operation only to choose a decoding shape.

Spec 002 continues to own the exchange record, provider response
normalization, and exchange digests.
Spec 003 owns retention and redaction. Spec 006 owns the host lifecycle,
child-spawn boundary, and result rendering that this spec extends with a
predeclared comparison plan. A consumer owns any conclusion drawn from the
resulting testimony.

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
4. an exact normalized request component selector containing provider family,
   API surface, component kind, component index, and text-part index; and
5. matching mode `exact-contiguous-utf8-v1`.

The target is never inferred from the working directory, repository, an
`AGENTS.md` filename, a configured import path, similar language, or a digest
found after capture. The plan is immutable for the attempt. Duplicate probe
names, a digest mismatch, invalid UTF-8, an empty target, or an unsupported
selector refuses before child spawn. A selector is unsupported when its
provider family, operation, and component kind are not a row of the table in
section 3.5; that check needs no request body. Whether the named component and
text part exist is known only after decoding, so a supported selector whose
component is not present produces `unknown` with reason `component-absent`,
and one whose component exists without the named text part produces `unknown`
with reason `text-part-absent`; neither is a refusal.

Comparison operates on the decoded component produced by the request-component
decoder in section 3.5. It does not search raw JSON serialization, HTTP headers, response
content, unrelated messages, tool output, or every retained byte by default.
The selector must resolve to exactly one string component in the identified
exchange. No component, more than one component, a non-string component, or an
unsupported selector produces `unknown`; the witness never broadens the search
to obtain a present result.

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
3. selected-component decoded-byte digest and length when the component is
   complete, uniquely resolved, and untouched by redaction (section 3.5);
4. result status, match count, and decoded-component byte offsets when known;
5. request and selected-component completeness;
6. the request-component decoder and witness producer identities; and
7. ordered findings and unknown reasons.

The durable record never contains the target bytes, matched bytes, surrounding
text, a reconstructed instruction document, credentials, or provider response
content. The transient target and decoded comparison buffer are cleared after
the result is formed. Content retention under spec 003 does not silently add
content to this result schema.

Canonical bytes use canonical-keysort-json and the named construction
`wire-witness.instruction-observation/1+keysort-json+sha256`. The record digest
is carried beside the bytes it identifies.

### 3.5 Request-component decoding

The decoder accepts only a complete, identity-encoded outbound JSON request
body. A compressed, truncated, or malformed body, or one whose provider family
or operation is not listed below, produces `unknown` with a closed reason.

| Provider family and operation | Component kind | Component index | Text-part index |
|---|---|---|---|
| Anthropic `POST /v1/messages` | `system` | always 0 | 0 for a string `system`; the position among `text` blocks for an array |
| Anthropic `POST /v1/messages` | `message` | position in `messages` | 0 for string `content`; the position among `text` blocks for an array |
| OpenAI `POST /v1/responses` | `instructions` | always 0 | always 0; only a string value is supported |
| OpenAI `POST /v1/responses` | `input` | 0 for a string `input`; the position in the `input` array otherwise | 0 for string content; the position among text content parts for an array |
| OpenAI `POST /v1/chat/completions` | `message` | position in `messages` | 0 for string `content`; the position among `text` parts for an array |

Every index is zero-based. A text-part index is an ordinal among the text
parts only: in `[image, text, text]` the second text block is text-part 1, not
array position 2. A component index is the position in the named array,
counting every element whatever its content.

A selected component is the JSON string at that location, decoded from its
JSON escape form to UTF-8 bytes. Image, file, tool-use, and tool-result blocks,
tool definitions, and any key outside the table are never selectable, and they
are not counted by a text-part index. Adding a shape is a later spec's change, not a
decoder inference.

Order matters here. First, spec 003's mandatory redaction scan of the request
body completes. Then the host compares against the component decoded from the
original, unredacted bytes, held only transiently.
When any redaction replacement's recorded input offset and length intersect
the selected component's source span, or a scan could not complete, the result
is `unknown` with reason `redaction-intersects-component` and no component
digest, length, or offset is recorded. Otherwise the component's bytes are the
same before and after redaction, so the durable result carries nothing
mandatory redaction would have removed.

## 4. Observable negative cases

| Case | Expected |
|---|---|
| The target appears in a tool result but not the selected system component | `observed-absent` for a complete system component; unrelated components are not searched. |
| Capture stops before the request completes | `unknown` with the capture-gap reason, never absent. |
| Redaction changes bytes inside the selected component | `unknown` with `redaction-intersects-component`; the witness does not search a lossy representation and claim absence. |
| A similar paraphrase appears | Absent under exact-byte matching; semantic similarity is not inferred. |
| The same target appears twice | Present with count two and both decoded-component offsets. |
| A secret detector fires inside the selected system prompt | `unknown` with `redaction-intersects-component`; no component digest, length, or offset is recorded. |
| The selector names a component kind outside section 3.5's table, such as tool definitions | Refused before child spawn as an unsupported selector. |
| A supported selector's component index exceeds the components present | `unknown` with reason `component-absent`; no other component is searched. |
| A supported selector's text-part index exceeds the text parts present, for example because the rest are images | `unknown` with reason `text-part-absent`; no other part is searched. |
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
test ! -e crates/wire-witness-cli/src/instruction_observation.rs
```
