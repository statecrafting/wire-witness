---
id: "005-binding-and-sidecar-protocol"
title: "Attempt binding and stdio sidecar protocol"
status: draft
implementation: pending
created: "2026-09-26"
summary: >
  Defines the immutable AttemptBinding supplied by Statecraft, the versioned
  stdio protocol for one sidecar per attempt, capture-digest closure of the
  bracketed effect, and statecraft/wire-exchange/v1 testimony for admission by
  the host rather than by the witness.
establishes:
  - { kind: module, id: "wire_witness_cli::sidecar_protocol", planned: true }
depends_on:
  - "001-boundaries-and-authority"
  - "002-exchange-record-and-normalization"
  - "003-redaction-custody-and-retention"
  - "004-allowlisted-capture-proxy"
interface_references:
  - corpus: "statecraft-cli"
    spec: "003-work-and-run-semantics"
    digest: "sha256:3ad363a4354d0277f2e3b4d862c144a597f14f0a471d150cb1c295bfbaab56c6"
    sections:
      - anchor: "3-3-the-run-record"
        digest: "sha256:4216112648ad63a3214ec6017c03ab08756cec1e76433325058c425d181e9c25"
      - anchor: "3-3-1-effect-identity-and-one-to-one-closing"
        digest: "sha256:fe9215ff1d25660f0740dd4be1546a3ee2b3c18baad2cd8bbfd582d14fbca8e4"
      - anchor: "3-6-recovery-and-reconciliation"
        digest: "sha256:50b88facefc2e4c69037b3f006fd9a464949c47498ed59d2b30ea066291d9bae"
    obtained: "2026-09-26"
    rationale: "Bind the sidecar lifetime to Statecraft's durable effect identity and fold."
  - corpus: "statecraft-cli"
    spec: "005-acceptance-and-evidence"
    digest: "sha256:74c24c29a40445928316dc32dacb13b1d27f5a79798fabbb47954eb2b488cdae"
    sections:
      - anchor: "3-5-the-four-evidence-dimensions-and-admission"
        digest: "sha256:1bac6db45b909e5e000096d6d0144a3687d06b92d2b3cbc60ee3e9a442c76e92"
      - anchor: "3-7-bytes-digests-and-constructions"
        digest: "sha256:036391834465af64e01765a99107b3d82a5c86da67dca644259b37dd685a119e"
      - anchor: "3-13-the-reference"
        digest: "sha256:e30e651e7d26a7558370958a20ea58c9a2475461bbe1573921c721c0e14c64f3"
    obtained: "2026-09-26"
    rationale: "Produce a typed evidence reference while leaving admission to Statecraft."
obligations:
  - id: "I-1"
    kind: invariant
    text: "A supervised sidecar repeats the supplied run id, attempt, and effect id unchanged and never infers or mints any member."
    anchor: "3-1-attemptbinding"
  - id: "I-2"
    kind: invariant
    text: "The witness emits testimony and an evidence reference but never admits it, changes policy, or writes the Statecraft intent or outcome."
    anchor: "3-4-evidence-envelope-boundary"
  - id: "R-1"
    kind: requirement
    text: "The terminal sidecar message carries capture completeness and digest, allowing the supervisor to close the exact bracketed effect."
    anchor: "3-3-bracketed-lifetime"
  - id: "R-2"
    kind: requirement
    text: "Unknown JSON fields are ignored and preserved where relayed, but an unknown message type or malformed line is a protocol finding and never success."
    anchor: "3-2-versioned-stdio"
  - id: "V-1"
    kind: verification
    text: "The draft compiles, the planned sidecar protocol module is recorded, and both Statecraft interface pins verify."
    anchor: "verification"
    inputs:
      - "specs/005-binding-and-sidecar-protocol/spec.md"
---

# 005: Attempt binding and stdio sidecar protocol

## 1. Purpose

Give Statecraft a narrow out-of-process capture seam. Statecraft owns the run,
effect journal, admission, and policy. The sidecar owns one attempt's proxy and
returns testimony tied to the identity it received.

## 2. Territory

This spec owns the planned `wire_witness_cli::sidecar_protocol` module and its
JSON message types. It does not own Statecraft record types, spawn policy,
evidence admission, or run outcome policy.

## 3. Behavior

### 3.1 `AttemptBinding`

The supervised binding is:

```text
AttemptBinding { run_id: String, attempt: u32, effect_id: String }
```

All strings are non-empty and bounded. `attempt` is the number Statecraft
supplies. Before spawning the sidecar, statecraft-cli mints the `effect_id` and
writes a durable intent carrying it. The sidecar validates the shape, then
repeats all three values byte for byte on every exchange, finding, readiness,
and terminal message.

The sidecar never derives a binding from time, PID, parent process, workspace,
provider ids, content, or prior attempts. It never changes case, Unicode form,
or numeric representation. Missing or invalid supervised binding refuses
startup before a proxy listener or CA is created.

### 3.2 Versioned stdio

The sidecar speaks newline-delimited UTF-8 JSON on dedicated stdin and stdout.
Each message contains `protocol: "wire-witness.sidecar/1"`, `type`, and the
binding. No request, response, secret, captured content, or CA key appears on
stdio. Diagnostics use stderr only after spec 003 redaction and are not protocol
evidence.

The supervisor sends exactly one `start` message containing:

- the binding;
- capture and authentication-tunnel allowlists;
- requested retention and bounded resource limits;
- an attempt output directory and certificate output path; and
- a host policy identifier, recorded but never interpreted as permission.

The sidecar emits, in order:

1. `ready`, with the local proxy endpoint, certificate path, effective
   retention, and CA memory-lock posture;
2. zero or more `finding` and `exchange` notices containing identities and
   digests only; and
3. exactly one `finished` message containing completeness, counts, manifest
   path, capture digest when one exists, and absence or incomplete reason.

Objects accept unknown fields so a producer may add compatible facts. A relay
preserves unknown fields it does not interpret. Removing or retyping a field is
a protocol-major change. An unknown `type`, duplicate `ready`, duplicate
terminal, message after terminal, invalid binding, oversized line, invalid
UTF-8, or malformed JSON is a protocol finding and never a successful finish.

EOF before `finished` is `sidecar-stopped`. A broken stdout pipe stops the
sidecar and leaves the supervisor to recover the open Statecraft effect.

### 3.3 Bracketed lifetime

The ordering is fixed:

1. Statecraft writes and durably acknowledges the effect intent.
2. Statecraft spawns the sidecar with the same `AttemptBinding`.
3. The sidecar creates its CA, proxy, and outputs, then reports `ready`.
4. Statecraft spawns the supervised child with the sidecar's per-process
   environment.
5. The sidecar closes custody and reports `finished` with the capture digest.
6. Statecraft writes the outcome repeating the same `effectId` and records the
   capture digest or explicit absence.

The witness does not write either Statecraft record. If it or the supervisor
crashes after step 1 and before step 6, the existing Statecraft fold finds an
open intent. Restart does not reuse the old CA or silently close the effect.

Capture completeness is testimony. The sidecar never maps `incomplete` to a
Statecraft run outcome. The host's recorded policy decides whether the child
may start after a degraded `ready` posture and whether an incomplete capture
fails, refuses, or merely annotates the run.

### 3.4 Evidence envelope boundary

The capture manifest is referenced as evidence type
`statecraft/wire-exchange/v1`. The reference carries schema version, byte
length, SHA-256 digest of the preserved manifest bytes, construction, producer
identity and digest when known, subject binding to `run_id`, `attempt`, and
`effect_id`, and capture completeness.

wire-witness produces that reference and the underlying testimony. The
statecraft-envelope verifier checks bytes and dimensions, and statecraft-cli
applies admission policy. The witness never sets `admit`, claims issuer trust,
signs for another principal, changes a policy, or authorizes a later effect.
Unknown or unperformed checks stay unknown in their own dimensions.

## 4. Observable negative cases

| Case | Expected |
|---|---|
| Binding is absent or has an empty effect id | Startup refusal before CA or listener creation. |
| A sidecar tries to substitute its PID for run id | Invalid implementation; every emitted binding must equal the supplied bytes. |
| Stdout ends after `ready` | No terminal success; Statecraft retains an open intent for recovery. |
| A second terminal message arrives | Protocol finding; it does not replace or amend the first. |
| Capture is incomplete but the child succeeded | Both facts remain; the witness does not decide the run outcome. |
| Manifest bytes fail their digest | Integrity may fail and admission may refuse; the witness does not repair bytes. |
| A caller treats the evidence reference as authorization | Refused at the consumer boundary; it is testimony only. |

## 5. Out of scope

Statecraft implementation, policy configuration, signing roots, product
implementation, automatic reconciliation, publication, and deployment are out
of scope.

## Verification

```verify:cli
./.tooling/bin/spec-spine check --fail-on-warn
./.tooling/bin/spec-spine interface verify --spec 005 --export statecraft-cli=/Users/bart/DevWork/statecraft-cli
test ! -d crates
```
