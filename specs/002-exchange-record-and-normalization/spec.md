---
id: "002-exchange-record-and-normalization"
title: "Exchange record and provider normalization"
status: approved
implementation: pending
created: "2026-09-26"
summary: >
  Defines wire-witness.exchange/1, the canonical testimony produced from one
  observed provider exchange, and lossless normalization rules for Anthropic
  and OpenAI request, response, streaming, identity, usage, and cost fields.
establishes:
  - { kind: module, id: "wire_witness_core::exchange", planned: true }
depends_on:
  - "001-boundaries-and-authority"
interface_references:
  - corpus: "rustev"
    spec: "009-remote-adapter-protocol"
    digest: "sha256:d7c70bac7cf82e627e7cb3c19eb6aee842e95b78ba0c61555314e2d2f62c6e35"
    sections:
      - anchor: "3-1-authority-and-provenance"
        digest: "sha256:ed94499ca9b4687dcf509ff71a84e04e2613cb6c6b804a9d96cf22c70743022c"
      - anchor: "3-7-model-and-served-identity"
        digest: "sha256:8bd763356721271561156a685ba43b2e32965328eae516d91446eb54696c85ef"
      - anchor: "3-8-cost-and-privacy"
        digest: "sha256:28f1ade8823b560b6d053bd7f68cfc90558102088826743dc5bf55e1bea27f57"
      - anchor: "3-9-exchange-records-and-replay"
        digest: "sha256:bd689328c829ce1bbfa5751c7e453d0f2bd8f3996428f87ea3d28487bdea642e"
    obtained: "2026-09-26"
    rationale: "Align the witness record with Rustev while keeping ownership here."
  - corpus: "canonical-keysort-json"
    spec: "000-canonical-keysort-json-bootstrap"
    digest: "sha256:77a7ada646fc7d36463746ee9449ff0973a991afe3926a1581e3a796862b6216"
    obtained: "2026-09-26"
    rationale: "Pin the canonical byte construction used before hashing records."
obligations:
  - id: "I-1"
    kind: invariant
    text: "An exchange is testimony only; provider and harness statements remain attributed claims and never become authority."
    anchor: "3-1-testimony-and-schema"
  - id: "I-2"
    kind: invariant
    text: "Requested identity and served identity are separate fields, and unknown is an explicit value that is never filled from another identity."
    anchor: "3-3-identity-usage-and-cost"
  - id: "R-1"
    kind: requirement
    text: "Unknown provider event types are preserved as unknown and malformed provider input produces a finding rather than a normalized success."
    anchor: "3-4-provider-normalization"
  - id: "R-2"
    kind: requirement
    text: "Canonical exchange bytes use canonical-keysort-json followed by SHA-256 under a named construction."
    anchor: "3-5-canonical-bytes-and-digests"
  - id: "V-1"
    kind: verification
    text: "The draft compiles, its planned module is recorded, and both producer interface pins verify from the named local exports."
    anchor: "verification"
    inputs:
      - "specs/002-exchange-record-and-normalization/spec.md"
---

# 002: Exchange record and provider normalization

## 1. Purpose

Define the record that wire-witness produces from bytes it observed on a
provider connection. The record supports audit and later evidence admission.
It does not evaluate an answer, establish provider truth, or authorize an
effect.

## 2. Territory

This spec owns the planned `wire_witness_core::exchange` module. The module
contains the `wire-witness.exchange/1` types and pure normalization and
canonical-byte functions. It performs no transport, storage, clock, or policy
work. Specs 003 and 005 separately own redaction, custody, and host binding.

## 3. Behavior

### 3.1 Testimony and schema

Every record has schema `wire-witness.exchange/1` and contains:

1. an immutable binding from spec 005;
2. a monotonically increasing exchange sequence within that binding;
3. transport metadata: observed protocol, authority, method or WebSocket
   operation, request path, response status when present, and stream state;
4. provider family: `anthropic`, `openai`, or `unknown`;
5. requested and served identity from section 3.3;
6. provider-reported usage and cost, and any separate estimate;
7. retention, redaction, completeness, and findings from spec 003;
8. request and response byte references, each with the original byte length
   and SHA-256 digest whether or not content was retained; and
9. normalized events in observed order.

The record describes what the witness observed. A value supplied by the child,
provider, response body, or header remains attributed to that source. No field
means approved, accepted, safe, correct, or authorized.

### 3.2 Presence, absence, and completeness

Optional observations use explicit tagged values:

- `{status: known, value: ...}` means the witness observed the value;
- `{status: unknown, reason: ...}` means the value could not be established;
- `{status: absent}` means the relevant complete message was observed and did
  not contain the value.

`unknown`, `absent`, and an omitted field are not interchangeable. Fields in
this schema are not omitted merely because their value is unknown.

Completeness is one of `complete`, `incomplete`, or `absent`. An incomplete or
absent capture carries a closed reason such as `disabled`, `connection-failed`,
`stream-ended`, `limit-exceeded`, `malformed`, `sidecar-stopped`, or
`sink-failed`. A new reason is a schema change. Whether an incomplete capture
fails the surrounding run is host policy and is not decided by this record.

### 3.3 Identity, usage, and cost

`requested_identity` records only the identity placed on the outbound wire.
`served_identity` records only an identity stated by the provider, together
with `disclosure: pinned | reported | unknown`. When no served identity was
observed, its status is `unknown`; it is never copied from the request, route,
endpoint, or adapter configuration.

Usage entries retain the provider's field name, integer or decimal text, unit
when stated, message location, and `source: provider-reported`. Normalization
may group entries but may not rename a token count into a cross-provider unit,
sum fields with different meanings, or invent zero for a missing field.

Cost has separate slots:

- `reported`: the exact decimal text and currency or unit reported for this
  exchange;
- `estimated`: the computed decimal text, currency, rate-table identity, and
  usage inputs used; and
- `unknown`: the reason neither value is available.

An estimate is always labelled `estimated`. Provider-reported usage is not
provider-reported cost, and neither becomes an observed charge merely because
a local price table exists.

### 3.4 Provider normalization

Normalization is lossless with respect to meaning and preserves event order.

For Anthropic messages:

- the outbound `model` is the requested identity;
- a response or `message_start` model is a reported served identity;
- usage keys including cache-related keys retain their provider names;
- SSE event names and content block indices are preserved; and
- an unrecognized SSE event becomes an `unknown` event carrying its event name,
  position, and redacted payload reference.

For OpenAI responses:

- the outbound `model` is the requested identity;
- a response model is a reported served identity;
- usage keys, including nested details, retain their provider names and path;
- response, chat-completion, and streaming event names remain distinguishable;
  and
- an unrecognized event becomes an `unknown` event with the same bounded,
  redacted evidence treatment.

For either family, invalid JSON, a malformed SSE frame, invalid WebSocket
framing, contradictory terminal messages, or a provider value outside the
schema produces a finding and makes the affected direction incomplete. The
witness never repairs the bytes into a success. The proxy may still forward
opaque bytes under spec 004.

### 3.5 Canonical bytes and digests

The canonical record bytes are compact JSON produced by recursively sorting
object keys by Unicode code point, preserving array order and scalar values,
under canonical-keysort-json's pinned contract. The named construction is
`wire-witness.exchange/1+keysort-json+sha256`.

The exchange digest is SHA-256 over those canonical bytes and is carried beside
the record, not inside the bytes it hashes. Request and response byte digests
use `file-bytes-sha256` over the exact observed direction bytes before
normalization. Redaction and retention never change what those byte digests
identify, but spec 003 controls whether the bytes themselves may be retained.

## 4. Observable negative cases

| Case | Expected |
|---|---|
| Requested model exists but served model is missing | Served identity is explicitly `unknown`; requested identity is not copied. |
| Provider reports token usage but no charge | Usage stays provider-reported; cost is estimated or unknown, never reported. |
| An unknown SSE event arrives | It is preserved as `unknown` in sequence with a redacted payload reference. |
| A malformed terminal event arrives after valid deltas | The finding is retained and the affected direction is `incomplete`. |
| Two maps with different insertion order describe the same record | Canonical bytes and exchange digest are identical. |
| A consumer treats the record as permission | The consumer is wrong; this schema carries no grant. |

## 5. Out of scope

Transport interception, durable storage, provider calls, evidence admission,
answer evaluation, pricing authority, publication, and implementation are out
of scope for this draft.

## Verification

```verify:cli
./.tooling/bin/spec-spine check --fail-on-warn
./.tooling/bin/spec-spine interface verify --spec 002 --export rustev=/Users/bart/DevWork/rustev --export canonical-keysort-json=/Users/bart/DevWork/canonical-keysort-json
test ! -d crates
```
