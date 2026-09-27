---
id: "006-standalone-host"
title: "Standalone host and per-process environment"
status: approved
implementation: pending
created: "2026-09-26"
summary: >
  Defines the standalone CLI, unsupervised session identity, child-only proxy
  and trust environment, additive JSON output, and the prohibition on global
  proxy, certificate, shell, browser, or operating-system trust mutation.
establishes:
  - { kind: crate, id: "wire-witness-cli", planned: true }
depends_on:
  - "001-boundaries-and-authority"
  - "002-exchange-record-and-normalization"
  - "003-redaction-custody-and-retention"
  - "004-allowlisted-capture-proxy"
  - "005-binding-and-sidecar-protocol"
interface_references:
  - corpus: "statecraft-cli"
    spec: "002-environment-lifecycle"
    digest: "sha256:e4673bee00f957d08b5365cdc18d8e191f23dd6ee36abc4bba52743d03dafafa"
    sections:
      - anchor: "3-2-the-three-ownership-classes"
        digest: "sha256:02617148a08a07e69a421a88a68e2de0c50139d2d21a01ef22051e639debb044"
      - anchor: "3-16-configuration-authority"
        digest: "sha256:1f6d251188ea6e6456fe86b5762a0b0a480be4ae68f358986452162b5f9c40d7"
    obtained: "2026-09-26"
    rationale: "Keep standalone configuration explicit and separate from managed Statecraft ownership."
  - corpus: "statecraft-cli"
    spec: "006-command-surface"
    digest: "sha256:e22eeaa09d82c8ee2a8e0775f2d503b065472755fa4aeac34525d37c55ae532e"
    sections:
      - anchor: "3-3-the-exit-code-vocabulary"
        digest: "sha256:f35a79008a237f5d879e0f9c6404a115623f82d503e462d4563e0a91936a5813"
      - anchor: "3-4-two-renderings-of-one-value"
        digest: "sha256:d120bb76f4fd03b77c7671afa49fbeb0ad2bd1a0016a396b180d649358bf8045"
    obtained: "2026-09-26"
    rationale: "Use the family exit vocabulary and additive JSON rendering rule."
obligations:
  - id: "I-1"
    kind: invariant
    text: "A standalone session has an unsupervised identity and never claims a Statecraft run, attempt, or effect id."
    anchor: "3-2-unsupervised-identity"
  - id: "I-2"
    kind: invariant
    text: "Proxy and CA trust configuration is applied only to the spawned child process and never mutates global or persistent user configuration."
    anchor: "3-3-child-only-environment"
  - id: "R-1"
    kind: requirement
    text: "Standalone and sidecar commands host the same core and proxy contracts without reimplementing normalization or policy."
    anchor: "3-1-command-surface"
  - id: "R-2"
    kind: requirement
    text: "Human and JSON output render the same result; JSON may gain fields but existing fields are not removed or retyped within version 1."
    anchor: "3-4-results-and-exits"
  - id: "V-1"
    kind: verification
    text: "The draft compiles, the planned CLI crate is recorded, and both Statecraft interface pins verify."
    anchor: "verification"
    inputs:
      - "specs/006-standalone-host/spec.md"
---

# 006: Standalone host and per-process environment

## 1. Purpose

Provide a usable standalone witness without pretending it was supervised by
Statecraft and without changing the operator's machine-wide proxy or trust
configuration.

## 2. Territory

This spec owns the planned `wire-witness-cli` crate and its two hosts. It may
depend on core and proxy. It contains argument parsing, process spawning,
filesystem custody, clocks, randomness, and rendering, but no second copy of
normalization, redaction, transport, or admission rules.

## 3. Behavior

### 3.1 Command surface

The executable is `wire-witness` with two commands:

- `wire-witness run [options] -- <program> [args...]` creates one standalone
  session, one fresh CA, one proxy, and one child process; and
- `wire-witness sidecar` speaks spec 005 on stdio for one supervised attempt.

`run` requires explicit capture authorities. Authentication tunnel authorities
are explicit and disjoint. Retention defaults to `metadata-only`; `content`
requires a command-line opt-in for that invocation. The command never invokes a
provider by itself. Provider traffic occurs only if the user-named child makes
it.

Both commands call the same core and proxy library operations. The CLI parses,
calls, renders, and maps results to exit codes. It does not infer policy from a
provider, a prior session, or a Statecraft checkout.

### 3.2 Unsupervised identity

Before standalone child spawn, `run` generates a cryptographically random
128-bit session id and records `binding.kind: unsupervised`. Each exchange also
has a contiguous sequence number beginning at one. The identity contains no
`run_id`, `attempt`, or `effect_id` field.

The CLI never reads Statecraft environment variables, parent records, process
ancestry, repository state, or working-directory names to upgrade an
unsupervised identity. If a caller needs a Statecraft binding, it must use the
sidecar protocol and supply `AttemptBinding` before spawn.

### 3.3 Child-only environment

The CLI starts the child directly with a per-process environment overlay that
names:

- the loopback proxy endpoint for the child's supported HTTP and HTTPS proxy
  variables; and
- the ephemeral CA certificate as an additional trust input through an
  explicitly supported runtime-specific variable or argument.

Ordinary system trust remains available for unintercepted tunnels. A runtime
whose trust control replaces rather than extends ordinary roots receives a
per-attempt mode-0600 bundle composed from the selected system roots and the
ephemeral certificate. The original trust store is never edited.

Only the child and its descendants receive the overlay. The CLI never runs
`export` in the parent shell, edits shell startup files, writes Git or package
manager configuration, changes browser settings, calls operating-system proxy
tools, imports the CA into a user or system trust store, or changes another
process's environment.

On exit, the CLI removes the certificate and temporary trust bundle after
custody closes. Failure to remove is a named finding with the path, never a
claim of cleanup. The private key has no path and follows spec 003.

### 3.4 Results and exits

Human output and `--json` render one result value. JSON objects accept additive
fields within version 1. Removing or retyping an existing field requires a new
major version. Captured content and credentials never appear in terminal
output.

The exit vocabulary is:

| Code | Meaning |
|---|---|
| 0 | The requested host operation completed and found no witness problem. |
| 1 | The operation completed with a finding, including incomplete capture or cleanup residue. |
| 2 | A precondition was refused and the child was not spawned. |
| 3 | Usage error. |
| 4 | The host operation failed unexpectedly. |

The child's exit status is a field in the result, not silently collapsed into
the witness exit. By default a nonzero child status makes the overall operation
a finding. A witness failure is distinguishable from a child failure. Capture
completeness remains a fact; callers may impose a stricter policy outside the
witness.

## 4. Observable negative cases

| Case | Expected |
|---|---|
| `run` is invoked without a capture authority | Exit 2; no CA, proxy, or child is created. |
| A standalone process runs inside a Statecraft workspace | Identity remains `unsupervised`. |
| The child runtime has no safe additional-CA mechanism | Exit 2 before spawn; no global trust mutation is attempted. |
| Content retention is not named on this invocation | Effective retention is `metadata-only`. |
| Certificate cleanup fails | Exit 1 with a redacted path finding; cleanup is not claimed. |
| The child exits nonzero after complete capture | Child status and complete capture are both reported; neither rewrites the other. |
| A JSON consumer ignores a newly added field | Existing version-1 meanings remain unchanged. |

## 5. Out of scope

Shell activation, daemon-wide proxying, browser installation, global CA trust,
transparent packet capture, a hosted service, publication, deployment, and
product implementation are out of scope.

## Verification

```verify:cli
./.tooling/bin/spec-spine check --fail-on-warn
./.tooling/bin/spec-spine interface verify --spec 006 --export statecraft-cli=/Users/bart/DevWork/statecraft-cli
test ! -d crates
```
