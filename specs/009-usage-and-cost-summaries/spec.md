---
id: "009-usage-and-cost-summaries"
title: "Usage and cost summaries"
status: approved
implementation: complete
created: "2026-09-27"
summary: >
  Defines a deterministic binding-level summary of observed request counts,
  provider-attributed usage, reported cost, labeled estimates, identity
  variation, completeness, and gaps without converting testimony into a
  charge, quota decision, budget, or provider qualification.
extends:
  - { spec: "001-boundaries-and-authority", unit: { kind: section, file: "crates/wire-witness-core/src/lib.rs", anchor: "measurement-summary-module-export" }, nature: additive }
establishes:
  - { kind: section, file: "crates/wire-witness-core/src/measurement_summary.rs", anchor: "measurement-summary" }
depends_on:
  - "002-exchange-record-and-normalization"
  - "005-binding-and-sidecar-protocol"
obligations:
  - id: "I-1"
    kind: invariant
    text: "A summary preserves source attribution and never relabels provider-reported usage, reported cost, or a local estimate as an observed charge, quota, or spending decision."
    anchor: "3-1-summary-authority"
  - id: "I-2"
    kind: invariant
    text: "Only records with one exact binding and a contiguous exchange sequence may form a complete summary; missing, duplicate, foreign, or reordered inputs remain explicit findings."
    anchor: "3-2-input-set-and-order"
  - id: "R-1"
    kind: requirement
    text: "Usage and cost are aggregated only within identical provider, meaning, unit, numeric construction, and attribution keys, while incompatible entries remain separate deterministically ordered groups."
    anchor: "3-3-aggregation-rules"
  - id: "R-2"
    kind: requirement
    text: "The summary binds its ordered exchange digests and reports capture completeness, requested and served identity variation, unknowns, and cost provenance without requiring retained content."
    anchor: "3-4-summary-schema"
  - id: "V-1"
    kind: verification
    text: "The deterministic summary engine passes its exact arithmetic, ordering, validation, gap, attribution, and canonicalization tests."
    anchor: "verification"
    inputs:
      - "specs/009-usage-and-cost-summaries/spec.md"
---

# 009: Usage and cost summaries

## 1. Purpose

Summarize the measurements already present in exchange testimony so a consumer
does not need to invent aggregation rules. The summary remains a producer
record: it does not decide a cost ceiling, park work, establish a provider
bill, or qualify an adapter.

## 2. Territory

This spec owns the planned pure `wire_witness_core::measurement_summary`
module and schema `wire-witness.measurement-summary/1`. It consumes validated
`wire-witness.exchange/1` records and performs deterministic grouping,
addition, identity inventory, completeness folding, canonicalization, and
digest construction.

Spec 002 owns the meaning and attribution of each exchange field. Spec 005
owns supervised binding and terminal capture facts. Statecraft owns admission,
findings F-06 and F-07, spending authority, and scheduling decisions. A billing
system or provider statement owns any charge claim.

## 3. Behavior

### 3.1 Summary authority

A summary describes records supplied to it. It preserves whether each value
was provider-reported, locally estimated, unknown, or absent. Addition does not
upgrade attribution. In particular:

1. provider-reported usage is not provider-reported cost;
2. reported cost is not an independently observed charge;
3. estimated cost remains estimated and retains its rate-table identity;
4. a request count is an observed exchange count, not a provider quota debit;
   and
5. no total authorizes spending, selects a provider, or changes run policy.

### 3.2 Input set and order

One summary covers exactly one immutable binding. Its input manifest lists the
exchange digest for every sequence entry in ascending order. Supervised and
unsupervised bindings never share a summary.

A complete summary requires a sequence beginning at one with no gap,
duplicate, conflicting digest, or record after the terminal capture boundary.
A foreign binding, invalid digest, or duplicate sequence refuses summary
construction. A known missing sequence or incomplete terminal boundary may
produce an incomplete summary only when the gap and its source are recorded.

Input order supplied by a caller is not trusted. Records are validated by
binding and sequence, then ordered deterministically. The result records the
first and last sequence, count, ordered digest list, capture terminal state,
and any gap.

### 3.3 Aggregation rules

Request counts group by observed provider family, API surface (the exchange's
transport operation and request path), transport protocol, and authority. Unknown values form explicit groups and are never filled from
another record.

Usage values may be added only when all of these match exactly:

- provider family;
- original provider field path and field name;
- unit, including an explicit unknown unit;
- numeric representation and scale rules; and
- source attribution.

Decimal addition uses the fixed construction in section 3.5, which counts
non-finite, malformed, or out-of-range input as an unknown rather than a
value. Values with different meanings or units
remain separate groups even if their display names look similar. Missing usage
does not contribute zero; the group records observed, absent, and unknown
exchange counts separately.

Reported costs group by provider family, currency or unit, numeric
construction, and source attribution. Estimated costs additionally group by
provider family, rate-table identity, currency, and usage-input set, which is
the estimate's recorded `usage_inputs` list with duplicates removed, sorted by
the lexicographic order of each entry's UTF-8 bytes. Every entry is a string in
`wire-witness.exchange/1`, so no serialization rule is needed. Estimates from different rate tables are never summed into one
number. Unknown cost remains a counted unknown with reasons. The arithmetic
construction is an identity in every numeric group, so a later implementation change cannot silently alter a total.

Requested identities and served identities are separate ordered inventories.
Each entry carries its observed count and disclosure class. A requested model
never fills a missing served identity, and variation is reported rather than
collapsed into a preferred identity.

Every request-count group, numeric group, identity entry, and unknown-reason
entry is ordered by the lexicographic bytes of its canonical group key.
Source findings remain ordered by exchange sequence and then their order in
the source record. Summary-construction findings follow a closed production
order. Caller input order, hash-map iteration, and locale never affect summary
bytes.

### 3.4 Summary schema

`wire-witness.measurement-summary/1` contains:

1. the exact binding and terminal capture identity;
2. the ordered exchange-digest manifest and its own digest;
3. exchange and request counts grouped by observed transport facts;
4. provider-attributed usage groups with presence counts;
5. reported-cost and estimated-cost groups kept separate;
6. requested and served identity inventories;
7. complete, incomplete, and absent capture counts plus ordered findings; and
8. the summary producer identity, the arithmetic construction, and the
   rate-table identities used.

The summary can be formed from metadata-only records. It contains no prompt,
message, tool schema, response content, credential, raw header, or retained
body reference. It does not make a content-retention request broader.

Canonical bytes use canonical-keysort-json. The input-manifest digest uses
`wire-witness.measurement-inputs/1+keysort-json+sha256`; the summary digest uses
`wire-witness.measurement-summary/1+keysort-json+sha256`. Each digest is
carried beside the bytes it identifies.

### 3.5 Arithmetic construction and source identities

Numeric groups use `decimal-exact-v1`: each value is the provider's number
text, parsed without floating point as an optional sign, decimal digits, an
optional fraction, and an optional decimal exponent (`1.5e6` is exactly
1500000). After the exponent is applied the value has at most 38 significant
digits and a scale (the number of digits after the decimal point) of at most
18. Each group is summed in ascending order of
the exchange records' `sequence` numbers. A summary covers exactly one
binding (section 3.2), whose sequence numbers are unique within it, and section
3.2 refuses a duplicate before any summing, so this order is total and has no
ties. Independently of that, the bounds are
always checked on every addend and on the running sum after each addition. A value outside those bounds, `NaN`, an
infinity, or text that is not a JSON number is excluded from the total and
counted as an unknown with a finding. A running sum that leaves the bounds
makes that group's total `unknown` with an overflow finding, and no partial
total is reported. A group in which no value was included has total `unknown`,
never zero, so an all-unknown group stays distinct from one that sums to zero. The bounds
are fixed by this construction name rather than configured, so changing them
means a new construction name.

The exchange record carries a rate-table identity for an estimate but no
separate estimator or normalizer identity, and this spec does not invent one.
A summary therefore cannot name the normalizer behind its inputs; its ordered
exchange-digest manifest is the link back to the records, and normalizer
attribution needs a later change to `wire-witness.exchange/1`, not to this
summary.
The rate-table identity is the estimate's source identity: two estimates
under the same rate-table identity are grouped together, so an estimator that
computes differently from the same prices must publish a distinct rate-table
identity. The summary cannot detect an estimator that breaks this rule, because
the exchange record gives it nothing to compare. It therefore labels every
estimated-cost total as sound only to the extent its rate-table identity is
unique, carries that identity beside the total, and never presents an
estimated total as verified. Spec 002 section
3.3 describes reported, estimated, and unknown cost as separate slots, and the
current exchange record carries one of them. The summary does not rely on
that: an exchange contributes each reported or estimated value it carries to
that value's own group, and counts as an unknown cost only when it carries
neither.

## 4. Observable negative cases

| Case | Expected |
|---|---|
| A group's running sum leaves the decimal-exact-v1 bounds | The group's total is `unknown` with an overflow finding; no partial total is reported, and its presence counts remain. |
| A provider value has a scale above 18 | Excluded from its group's total and counted as an unknown with a finding; the total is visibly partial, never silently low. |
| One exchange sequence is missing | Summary completeness is `incomplete` with the exact gap; no zero-valued exchange is invented. |
| Two providers use the field name `input_tokens` with different units | Separate usage groups; the values are not summed together. |
| Two estimates use different rate-table identities | Separate estimated-cost groups. |
| A requested model is present and served identity is unknown | Requested inventory records the model; served inventory records unknown. |
| A provider reports usage but no monetary value | Usage remains reported; cost is estimated or unknown, never reported. |
| A consumer treats an estimated total as a spending grant | Refused by the authority boundary; the summary grants nothing. |

## 5. Out of scope

Billing reconciliation, provider quota interpretation, cost ceilings, work
parking, price-table publication, currency conversion, provider selection,
adapter qualification, evidence admission, and policy decisions are out of
scope.

## Verification

```verify:cli
./.tooling/bin/spec-spine check --fail-on-warn
./.tooling/bin/spec-spine index check --fail-on-unresolved
cargo test -p wire-witness-core measurement_summary --locked
```
