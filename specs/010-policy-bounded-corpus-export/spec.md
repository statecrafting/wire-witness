---
id: "010-policy-bounded-corpus-export"
title: "Policy-bounded corpus export"
status: draft
implementation: pending
created: "2026-09-27"
summary: >
  Defines an immutable, bounded export manifest for policy-permitted wire
  testimony, preserving binding, retention, completeness, redaction, and
  artifact identity without granting permission to retain, disclose, replay,
  train on, admit, or evaluate the captured material.
establishes:
  - { kind: module, id: "wire_witness_core::corpus_export", planned: true }
  - { kind: module, id: "wire_witness_cli::corpus_export", planned: true }
depends_on:
  - "002-exchange-record-and-normalization"
  - "003-redaction-custody-and-retention"
  - "005-binding-and-sidecar-protocol"
  - "006-standalone-host"
obligations:
  - id: "I-1"
    kind: invariant
    text: "An export records an external policy decision and exact source identities but never grants retention, disclosure, replay, training, evidence admission, or evaluation authority."
    anchor: "3-1-export-authority"
  - id: "I-2"
    kind: invariant
    text: "Export never resurrects erased or expired content, broadens effective retention, removes redaction findings, or represents an incomplete source set as complete."
    anchor: "3-2-source-eligibility"
  - id: "R-1"
    kind: requirement
    text: "The canonical manifest binds every selected capture, exchange, observation, summary, and included artifact by exact identity, digest, length, retention posture, completeness, and ordered findings."
    anchor: "3-3-export-manifest"
  - id: "R-2"
    kind: requirement
    text: "Digest-only, embedded, and external-reference entries are explicit and bounded, and any missing, expired, corrupt, foreign, or unauthorized input refuses export or remains an explicit gap."
    anchor: "3-4-materialization-and-bounds"
  - id: "R-3"
    kind: requirement
    text: "The only accepted policy input is a wire-witness.export-policy-receipt/1 envelope whose closed fields are validated structurally, and instruction-observation or measurement-summary entries are accepted only when the producing build implements their exact schema version."
    anchor: "3-5-policy-receipt-envelope"
  - id: "V-1"
    kind: verification
    text: "The draft compiles, its planned pure-core module resolves, and no implementation file is introduced."
    anchor: "verification"
    inputs:
      - "specs/010-policy-bounded-corpus-export/spec.md"
---

# 010: Policy-bounded corpus export

## 1. Purpose

Package selected wire testimony into a deterministic producer manifest that a
separately authorized consumer can inspect or transform. Export makes source
identity, gaps, retention, and materialization explicit. It does not decide
whether the testimony may be used or what a replay means.

## 2. Territory

This spec owns the planned pure `wire_witness_core::corpus_export` module, the
planned `wire_witness_cli::corpus_export` custody integration, and schema
`wire-witness.corpus/1`. The core module validates the structure and binding of
an externally supplied policy receipt, selects already governed testimony, and
constructs canonical manifest bytes. The CLI integration materializes eligible
retained artifacts without redefining policy or custody.

Spec 002 owns exchange records. Spec 003 owns retention, redaction, custody,
expiry, and erasure. Spec 005 owns attempt binding and capture closure. Spec
006 owns host lifecycle and filesystem operations. Spec 008 owns optional
instruction observations, and spec 009 owns optional measurement summaries.
Neither is a dependency: this spec can be built and used before either exists,
and section 3.5 says how their entries are admitted once they do.
Rustev owns conversion to any Rustev replay schema, replay scope and
equivalence, execution, comparison, and evaluation. Statecraft owns evidence
admission and run policy.

## 3. Behavior

### 3.1 Export authority

Export begins only from an explicit request naming:

1. a stable export id and exact producer identity;
2. one or more exact attempt bindings and capture digests;
3. an external policy receipt in the envelope defined in section 3.5;
4. permitted materialization modes and a maximum export lifetime; and
5. configured limits on bindings, records, artifacts, and total bytes.

The witness validates and records the receipt's structure, digest, binding,
scope, and expiry but does not authenticate its asserted principal or interpret
a policy name as permission. An absent, malformed, expired, scope-mismatched,
or non-allow receipt refuses export. A structurally valid export records only
that the attributed external receipt was supplied and satisfied the declared
constraints. It does not establish that the external decision was correct or
grant retention, disclosure, replay, training, evidence admission, provider
access, or evaluation authority.

Export performs no provider call and starts no replay. A later consumer must
make its own policy and authority decisions against the preserved manifest.

### 3.2 Source eligibility

Every source is verified against its declared schema, digest, byte length,
binding, capture digest, and custody state before selection. Sources from more
than one binding remain separately identified. They are never collapsed into
an inferred session or experiment.

The default materialization is `digest-only`. Content is eligible only when it
was already retained under effective `content` mode, remains unexpired and
unerased, passed mandatory redaction, and the supplied export decision
explicitly permits that artifact and materialization mode. Export cannot:

- recover erased, expired, disabled, or metadata-only content;
- extend a source retention deadline;
- remove or weaken a redaction, custody, or completeness finding;
- replace a missing artifact with similar bytes; or
- treat an unadmitted source as admitted evidence.

A source with an invalid digest, foreign binding, conflicting sequence, or
corrupt custody record refuses the export. A known absent or expired source may
appear only as an explicit gap when the export request permits incomplete
output. No gap is silently omitted.

The export expiry is no later than the request's maximum lifetime, the policy
receipt expiry, and the earliest expiry of any embedded or externally
referenced artifact. Export never extends a source deadline. A source expiring
during construction becomes a gap or refusal under the immutable request; it
is not emitted with a newly extended deadline.

### 3.3 Export manifest

`wire-witness.corpus/1` contains:

1. export id, creation time, expiry, producer identity, and construction;
2. the attributed external policy-receipt fields and digest, without embedding
   secret policy inputs;
3. each exact attempt binding and capture-manifest digest;
4. an ordered entry for every selected exchange with sequence, exchange
   digest, schema, byte length, retention mode, expiry, completeness, and
   ordered findings;
5. optional instruction-observation and measurement-summary entries with
   their exact schema, digest, length, and source exchange manifest;
6. each included artifact's materialization mode, media type, digest, byte
   length, expiry, redaction posture, and binding;
7. explicit gaps, refusals, and unsupported source versions; and
8. aggregate counts and limits used, without replacing entry-level facts.

Entries sort by binding bytes, then source kind, exchange sequence when
present, and digest. The manifest preserves each original digest and never
recomputes a source under a different schema. Canonical manifest bytes use
canonical-keysort-json. The manifest digest uses
`wire-witness.corpus/1+keysort-json+sha256` and is carried beside the exact
bytes it identifies.

A complete export means only that every source selected by the immutable
request is represented and verified. It does not mean the underlying capture
was complete. Capture completeness and export completeness remain distinct.

### 3.4 Materialization and bounds

Each artifact uses exactly one mode:

- `digest-only`: identity, digest, length, posture, and findings, with no
  content bytes or location;
- `embedded`: already-redacted eligible bytes carried in the bounded export,
  with exact digest and expiry; or
- `external-reference`: an opaque custody reference plus digest, length,
  expiry, and custodian identity.

An external reference is not a URL fetch instruction and does not grant access
to the referenced bytes. The pure core performs no filesystem or network I/O.
The host writes an export with the same mode-0600, exclusive-create, complete
temporary-write, flush, and atomic-install rules as spec 003. It never writes
raw or pre-redaction bytes as scratch data.

Configured limits are checked before materialization and during bounded byte
copying. Exceeding a binding, entry, artifact, per-artifact, or total-byte
limit refuses before final installation and leaves no partial export at the
final path. Embedded content is never downgraded to an external reference or
silently dropped to make an export fit.

### 3.5 Policy-receipt envelope

The witness defines the shape of the receipt it accepts, not the policy that
produced it. The only accepted input is `wire-witness.export-policy-receipt/1`,
a canonical-keysort-json object with exactly these fields:

| Field | Content |
|---|---|
| `schema` | The literal `wire-witness.export-policy-receipt/1`. |
| `decision_schema` | The external policy system's own decision schema name, recorded and never interpreted. |
| `decision_id` | The external decision's stable id. |
| `decision_digest` | `sha256:` hex over the external decision bytes, which the witness does not read. |
| `decided_at` and `expires_at` | RFC 3339 UTC times; both are required. |
| `outcome` | `allow` or `deny`. |
| `asserted_principal` | The authorizing principal as the receipt asserts it, not authenticated here. |
| `verifier` | The identity that verified the external decision, and its `verification_outcome`. |
| `scope` | The exact permitted binding and capture digests, source kinds, and materialization modes. |

An unknown field, a missing field, a second schema version, a non-`allow`
outcome, an expired receipt, or a request outside `scope` refuses export. The
receipt's own digest under `wire-witness.export-policy-receipt/1+keysort-json+sha256`
is recorded in the manifest.

An `instruction-observation` or `measurement-summary` entry is accepted only
when the producing build implements that exact schema version. Otherwise a selected entry of that
kind is an explicit `unsupported-source-version` gap, and refuses export when
incomplete output is not permitted.

## 4. Observable negative cases

| Case | Expected |
|---|---|
| Content was captured in metadata-only mode | Digest-only metadata may be exported; content cannot be reconstructed or embedded. |
| A retained body expired before export | The body is an explicit expired gap if incomplete output is permitted; otherwise export refuses. |
| One selected exchange digest does not verify | Export refuses; similar or reserialized bytes are not substituted. |
| Two captures use different bindings | Both bindings remain explicit and separately ordered in one manifest. |
| The policy decision permits disclosure but not training | Export records the decision; it does not infer or grant training permission. |
| An external reference points to unavailable bytes | The reference remains testimony about identity, not proof of current availability. |
| Rustev accepts a transformed replay bundle | That is a Rustev lifecycle fact and does not upgrade the wire-witness export. |

## 5. Out of scope

Policy adjudication, evidence admission, replay-schema construction, replay
execution, equivalence, evaluation, model training, dataset publication,
provider invocation, archive administration, and cross-custodian deletion are
out of scope.

## Verification

```verify:cli
./.tooling/bin/spec-spine check --fail-on-warn
./.tooling/bin/spec-spine index check --fail-on-unresolved
test ! -e crates/wire-witness-core/src/corpus_export.rs
test ! -e crates/wire-witness-cli/src/corpus_export.rs
```
