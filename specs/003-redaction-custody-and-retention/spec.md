---
id: "003-redaction-custody-and-retention"
title: "Redaction, custody, and retention"
status: draft
implementation: pending
created: "2026-09-26"
summary: >
  Defines metadata-only default retention, explicit content capture, mandatory
  credential removal before durable writes, per-attempt custody, 0600 outputs,
  and a fresh memory-only CA private key for every capture attempt.
establishes:
  - { kind: module, id: "wire_witness_core::custody", planned: true }
depends_on:
  - "001-boundaries-and-authority"
  - "002-exchange-record-and-normalization"
interface_references:
  - corpus: "action-gate"
    spec: "001-secret-detector-parity"
    digest: "sha256:dec12d328ecaa0f5cc0bac62001f4c370f63ead8b588da241bed8ad818c6de98"
    sections:
      - anchor: "3-1-the-registry"
        digest: "sha256:430b25b103f7bc3defe0727e40d6be577aa6d069d2325fe2a592a0fab61cc23d"
      - anchor: "3-2-findings-and-offsets"
        digest: "sha256:4f60c7fb4d44fba1e33689bb437aac2fc03f444ee6d887980d6e51128cce2abe"
      - anchor: "3-3-redaction-safe-reporting"
        digest: "sha256:f159c80ef6256e4f239080a349120dba859722d279a5c76147d7fbadd0a885f0"
    obtained: "2026-09-26"
    rationale: "Reuse the released family credential detector without copying its rules."
obligations:
  - id: "I-1"
    kind: invariant
    text: "Authorization headers, API keys, cookies, OAuth exchange bodies, and detected credentials are removed before any durable write."
    anchor: "3-2-redaction-before-custody"
  - id: "I-2"
    kind: invariant
    text: "Each attempt has a fresh CA private key held only in witness memory; only a certificate path is exposed to the child."
    anchor: "3-4-ephemeral-ca-custody"
  - id: "R-1"
    kind: requirement
    text: "Metadata-only is the default and every output records requested and effective retention."
    anchor: "3-1-retention-modes"
  - id: "R-2"
    kind: requirement
    text: "Every durable output is created with mode 0600 and is never made visible at its final path before redaction and complete write."
    anchor: "3-3-durable-custody"
  - id: "V-1"
    kind: verification
    text: "The draft compiles, the planned custody module is recorded, and the action-gate detector pin verifies."
    anchor: "verification"
    inputs:
      - "specs/003-redaction-custody-and-retention/spec.md"
---

# 003: Redaction, custody, and retention

## 1. Purpose

Make durable testimony useful without turning a provider exchange into a secret
archive. Redaction happens before custody, the default retains metadata only,
and the private key used for interception never becomes an artifact.

## 2. Territory

This spec owns the planned `wire_witness_core::custody` module: pure retention
classification, redaction decisions, findings, and custody metadata. The proxy
owns live byte transport and the CLI owns filesystem operations. Both consume
the decisions defined here without redefining them.

## 3. Behavior

### 3.1 Retention modes

Each attempt selects exactly one mode and records both `requested` and
`effective` in every exchange and capture manifest:

| Mode | Durable content |
|---|---|
| `metadata-only` | Structured metadata, lengths, digests, redaction counts, findings, identity, usage, and completeness. No request or response body content. This is the default. |
| `content` | The metadata-only record plus content that remains after mandatory redaction. Requires explicit opt-in for that attempt. |
| `disabled` | No exchange content or byte digest is retained. A manifest records capture absence with reason `disabled`. |

No configuration file, previous run, or provider setting silently promotes
`metadata-only` to `content`. If requested content cannot be made safe, the
effective mode becomes `metadata-only` for the affected field and a finding
names the reason.

Retention duration and erasure are host policy inputs recorded in the manifest.
The witness enforces the supplied deadline when it owns the output directory,
but it never claims erasure after custody has been transferred elsewhere.

### 3.2 Redaction before custody

The live processor identifies structure before any sink can persist bytes.
These values are always removed, in every retention mode:

- `Authorization` and `Proxy-Authorization` header values;
- API-key and bearer-token header values, including provider-specific names;
- `Cookie` and `Set-Cookie` values;
- client secrets, authorization codes, refresh tokens, access tokens, device
  codes, and assertions in OAuth request or response bodies; and
- every value detected by `action_gate_core::secrets::scan` using
  `SecretRules::default()` from the pinned released contract.

Header matching is ASCII case-insensitive. Structured JSON and form bodies are
redacted by field before free-text scanning. A redaction replacement records
only category, detector id when applicable, byte offset in the ephemeral input,
and removed byte length. It never records the removed value or a reversible
transform of it.

If a body is compressed, encrypted, malformed, or otherwise cannot be scanned
safely, content is not retained. Its metadata-only record carries a finding.
Logs, diagnostics, panic text, stdio protocol messages, and error values follow
the same rule and never echo the candidate secret.

### 3.3 Durable custody

The CLI creates every output file exclusively with mode `0600` inside an
attempt-specific directory not shared with another attempt. It writes a
redacted complete temporary file, flushes it, and atomically installs it at a
new final path. It never overwrites an existing artifact and never places raw
wire bytes in a temporary file, spill file, crash report, or retry journal.

The capture manifest lists every exchange digest, missing sequence, finding,
retention mode, and output path. A manifest is complete only after all exchange
files are durable. Its digest is computed over canonical bytes and is the
capture digest returned by spec 005.

A sink failure is recorded without retrying raw content through an unredacted
path. Capture absence and partial custody are explicit and never presented as a
complete empty capture.

### 3.4 Ephemeral CA custody

Every attempt generates a fresh CA key using operating-system cryptographic
randomness after the attempt begins. The private key:

1. exists only in the witness process's protected memory;
2. is never serialized, logged, included in a core value, swapped deliberately,
   exported, or sent over stdio;
3. is zeroized when its last live owner exits; and
4. is not reused after restart, even for a retry of the same run.

The corresponding certificate is written mode `0600` in the attempt directory.
Only its path, never key bytes or a key path, is exposed to the child. A child
that requires a private key for trust configuration is unsupported and the
attempt refuses before child spawn.

Memory locking is best effort and its result is recorded. Failure to lock memory
does not authorize writing the key to disk. Host policy decides whether that
posture is sufficient to start.

## 4. Observable negative cases

| Case | Expected |
|---|---|
| Content capture is not explicitly selected | Effective mode is `metadata-only`. |
| An OAuth token endpoint returns JSON credentials | The entire sensitive fields are removed before any sink sees them. |
| A detector fires on retained free text | The value is removed; only detector id, offset, and length remain. |
| A compressed body cannot be decoded within bounds | No content is retained; a finding and byte metadata remain. |
| The final output path already exists | The write refuses; the prior artifact is unchanged. |
| The process crashes after CA generation | No CA private key exists on disk; the incomplete manifest state is discoverable. |
| A caller requests world-readable output | Refused; output mode remains `0600`. |

## 5. Out of scope

Long-term archive administration, backup deletion, external key management,
global certificate installation, secrets inspection, publication, and product
implementation are out of scope.

## Verification

```verify:cli
./.tooling/bin/spec-spine check --fail-on-warn
./.tooling/bin/spec-spine interface verify --spec 003 --export action-gate=/Users/bart/DevWork/action-gate
test ! -d crates
```
