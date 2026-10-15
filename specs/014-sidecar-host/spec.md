---
id: "014-sidecar-host"
title: "The wire-witness.sidecar/1 stdio host"
status: approved
implementation: complete
created: "2026-10-10"
summary: >
  Implements `wire-witness sidecar` over the spec 005 protocol and the spec 013
  live runtime: one supervised attempt per process, exact AttemptBinding and
  terminal schema versions, a statecraft/wire-exchange/v1 manifest with a
  fixed member set, a producer identity Statecraft can pin, and the
  compatibility, rollback, and qualification records a release must carry.
amends:
  - "013-live-loopback-capture-runtime"
extends:
  - { spec: "013-live-loopback-capture-runtime", unit: { kind: section, file: "crates/wire-witness-cli/src/main.rs", anchor: "wire-witness-executable" }, nature: additive }
  - { spec: "013-live-loopback-capture-runtime", unit: { kind: section, file: "crates/wire-witness-cli/src/live_runtime.rs", anchor: "live-attempt-runtime" }, nature: additive }
  - { spec: "013-live-loopback-capture-runtime", unit: { kind: section, file: "crates/wire-witness-proxy/src/live.rs", anchor: "live-capture-runtime" }, nature: additive }
  - { spec: "013-live-loopback-capture-runtime", unit: { kind: section, file: "crates/wire-witness-cli/tests/executable.rs", anchor: "executable-acceptance" }, nature: additive }
establishes:
  - { kind: section, file: "crates/wire-witness-cli/src/lib.rs", anchor: "sidecar-host-module-export" }
  - { kind: section, file: "crates/wire-witness-cli/src/sidecar_host.rs", anchor: "sidecar-host" }
  - { kind: section, file: "crates/wire-witness-cli/src/wire_exchange.rs", anchor: "wire-exchange-manifest" }
  - { kind: section, file: "crates/wire-witness-cli/src/producer_identity.rs", anchor: "producer-identity" }
  - { kind: section, file: "crates/wire-witness-cli/tests/sidecar_host.rs", anchor: "sidecar-host-acceptance" }
  - { kind: directory, path: "crates/wire-witness-cli/tests/fixtures/sidecar-1/" }
  - { kind: directory, path: "qualification/" }
depends_on:
  - "002-exchange-record-and-normalization"
  - "003-redaction-custody-and-retention"
  - "005-binding-and-sidecar-protocol"
  - "006-standalone-host"
  - "013-live-loopback-capture-runtime"
obligations:
  - id: "I-1"
    kind: invariant
    text: "The sidecar validates the supplied AttemptBinding before it creates a CA, a listener, or any file, and repeats its three values byte for byte on every stdout message and in the wire-exchange manifest."
    anchor: "3-2-start-and-binding-validation"
  - id: "I-2"
    kind: invariant
    text: "Stdout carries only wire-witness.sidecar/1 messages with identities, counts, codes, and digests; no header value, body, query string, credential, or CA key reaches stdout, stderr, or any file."
    anchor: "3-4-notices-and-stdout-discipline"
  - id: "I-3"
    kind: invariant
    text: "The sidecar never admits evidence, decides a run outcome, writes a Statecraft record, or reports success it did not observe; every unknown, absent, and incomplete state stays explicit."
    anchor: "3-5-closing-custody-and-the-terminal-message"
  - id: "R-1"
    kind: requirement
    text: "Exactly one ready precedes exchange and finding notices, and exactly one finished message ends a session that reached ready; a session refused before ready writes nothing to stdout and exits 2."
    anchor: "3-3-ready"
  - id: "R-2"
    kind: requirement
    text: "The statecraft/wire-exchange/v1 manifest contains exactly the members of section 3.6, keeps requested and served identities, provider-testimony usage, estimate state, attestation, retention, and redaction as separate facts, and is installed last."
    anchor: "3-6-the-statecraft-wire-exchange-v1-manifest"
  - id: "R-3"
    kind: requirement
    text: "The producer identity names the exact version, executable digest, interface wire-witness.sidecar/1, AttemptBinding schema 1, terminal schema 1, evidence schema 1, and release channel, and the same values appear in ready, finished, and the manifest."
    anchor: "3-7-producer-identity-and-schema-versions"
  - id: "R-4"
    kind: requirement
    text: "Every release carries the sidecar-1 compatibility fixtures, its rollback identity, and per-platform qualification records; a change that breaks a fixture requires a new interface major."
    anchor: "3-8-compatibility-rollback-and-qualification"
  - id: "V-1"
    kind: verification
    text: "The sidecar host, manifest, identity, and fixture tests pass, including the end-to-end supervised capture against a local fake provider and every section 4 negative case."
    anchor: "verification"
    inputs:
      - "specs/014-sidecar-host/spec.md"
---

# 014: The wire-witness.sidecar/1 stdio host

## 1. Purpose

Spec 005 defines the supervised protocol and spec 013 supplies the live HTTP/1.1
runtime, but `wire-witness sidecar` still exits 2. Statecraft spec 013
(supervised adoption) requires an exact producer that speaks the protocol,
reports its interface and schema versions, and can be pinned and rolled back.
Statecraft spec 014 (wire evidence admission) requires a
`statecraft/wire-exchange/v1` manifest whose members it can screen against an
exact schema.

This spec supplies that host. It is one process per supervised attempt over
the spec 013 runtime. Its protocol scope is HTTP/1.1 only, matching spec 013;
HTTP/2 and WebSocket remain uncaptured and are reported, not translated.

## 2. Territory

This spec owns `wire_witness_cli::sidecar_host` (the stdio host),
`wire_witness_cli::wire_exchange` (the manifest), and
`wire_witness_cli::producer_identity` (the identity document), with their
acceptance tests and the `sidecar-1` compatibility fixtures.

It amends spec 013 in one respect: spec 013 R-4 and section 3.1 refuse
`wire-witness sidecar` until its host exists. This spec is that host, so the
command now runs it, and the spec 013 executable test that asserted the
refusal now asserts the host's `start-missing` refusal. Spec 013's text is not
edited. It extends, additively, the spec 013 executable, live runtime, and
proxy so the runtime can carry a supervised binding beside the existing
unsupervised one, start on a listener the host bound, and report installed
records and findings while it runs.

Spec 005 remains the protocol authority; its approved text and the section
digests Statecraft pins are unchanged. Normalization stays spec 002's,
redaction spec 003's, routing spec 004's, and every transport behavior spec
013's.

## 3. Behavior

### 3.1 The commands

`wire-witness sidecar` takes no further arguments. It reads spec 005 messages
on standard input, writes spec 005 messages on standard output, and writes
diagnostics only to standard error, as fixed codes with no captured values.
Any argument is usage error 3 with no stdout output.

`wire-witness identity [--json]` prints the section 3.7 producer identity and
exits 0. It creates no CA, listener, or file.

### 3.2 Start and binding validation

The first stdin line must be one complete `start` message within the stdin line
bound of 65,536 bytes. The host parses it with spec 005's parser, then:

1. validates the binding (spec 005 section 3.1);
2. validates every other `start` field (spec 005 section 3.2);
3. refuses a capture authority whose `protocols` contain no `http/1.1`;
4. requires `output_directory` to be absolute, not to exist, and to have an
   existing parent directory owned by the effective user and not group- or
   world-writable;
5. requires `certificate_path` to name a file directly inside
   `output_directory`; and
6. accepts an optional `listen_endpoint`. When present it must be
   `127.0.0.1:<port>` or `[::1]:<port>` with a non-zero port, and the host binds
   exactly that endpoint. When absent the host binds `127.0.0.1` on an
   ephemeral port. Statecraft supplies it when its confinement must name the
   port before the sidecar starts.

Any failure, including EOF or a non-`start` first message, is a refusal before
ready. The host writes nothing to stdout, writes one refusal code to stderr,
and exits 2. It leaves no CA, listener, directory, or file. To compare owners
without platform calls, the host creates `output_directory` (mode 0700) and
refuses it, removing it again, when its owner differs from the parent's. The refusal
codes are `start-missing`, `start-malformed`, `binding-invalid`,
`start-invalid`, `protocol-unsupported`, `output-directory-invalid`,
`certificate-path-invalid`, `endpoint-invalid`, and `endpoint-unavailable`.
The last is reported after the bind fails and before any CA or file exists.

After validation the host creates `output_directory` with mode 0700, generates
the attempt CA, writes the certificate to `certificate_path` with mode 0600,
and starts the spec 013 proxy on the bound endpoint. A failure in this step is
also a refusal before ready (`attempt-preparation-failed`, exit 2); the host
removes what it created.

Requested `content` retention is effective `metadata-only` with finding
`content-retention-unavailable`, as in spec 013. Requested `disabled` starts
the listener and routes traffic but records no exchange. Each `start` limit is
applied where the runtime has an equivalent bound; a limit with no equivalent
in the HTTP/1.1 runtime is listed in ready as unenforced, never silently
treated as applied. `host_policy_id` is recorded in the manifest and never
interpreted.

### 3.3 Ready

The host then writes exactly one `ready` message with:

- `protocol`, `type`, and the binding, as spec 005 requires;
- `endpoint`, the bound loopback address and port;
- `certificate_path`, exactly as supplied;
- `effective_retention`;
- `memory_lock`, which is `not-attempted` in this version;
- `application_protocols`, which is `["http/1.1"]`;
- `unenforced_limits`, the sorted names of `start` limits the runtime does
  not apply; and
- `producer`, the section 3.7 identity subset.

No other process may be told the endpoint by this host. A second `ready` is
never written.

### 3.4 Notices and stdout discipline

While running, the host writes, in observed order:

- one `exchange` notice per installed exchange record, with `sequence`,
  `record_digest`, `record_byte_length`, and `completeness`; and
- one `finding` notice per runtime finding, with `code` and, when the finding
  names one, the `authority`.

Notices carry no detail text. Every stdout line is canonical compact JSON
ending in one newline. Before writing, the host checks each message against
spec 005's forbidden-field rule and refuses to write a message that fails it,
recording finding `stdout-message-suppressed` instead. Diagnostics on stderr
are fixed codes.

If stdout is closed or a write fails, the host stops the proxy, closes custody
as in section 3.5 with gap `stdout-broken`, does not attempt further stdout
writes, and exits 4. The supervisor then holds an open effect, as spec 005
section 3.2 expects.

### 3.5 Closing custody and the terminal message

Custody closes at the first of:

| Trigger | End disposition | Gap |
|---|---|---|
| stdin EOF after `start` | `supervisor-closed` | none |
| any further stdin line | `supervisor-protocol-violation` | `unexpected-supervisor-message` |
| `limits.total_seconds` elapses | `deadline` | `deadline-reached` |
| stdout write fails | `stdout-broken` | `stdout-broken` |

The supervisor requests closure by closing the sidecar's stdin after the child
exits. The host installs no signal handler; a signal that ends the process
leaves no terminal message, and the supervisor records the termination.

To close, the host stops the proxy with spec 013's grace period, joins the
record writer, installs the spec 013 capture manifest with a supervised
binding, then installs `wire-exchange.json` (section 3.6) last. Completeness
follows spec 013's rules, plus every gap in the table above. Spec 013's rules
are named as gaps: `exchange-incomplete` (a record is not complete),
`record-not-installed`, `sink-backpressure` (an observed exchange was
dropped), `connections-open-at-close`, `capture-finding` (a connection
finding that may concern a capture authority), and `manifest-not-installed`.

It then writes exactly one `finished` message carrying spec 005's members
(`completeness`, `exchange_count`, `finding_count`, `manifest_path`,
`capture_digest`, `reason`) and:

- `end_disposition` from the table;
- `gaps`, the sorted gap names;
- `evidence_reference`, the spec 005 section 3.4 reference over the exact
  `wire-exchange.json` bytes, or `null` when no manifest was installed; and
- `producer`, as in ready.

`exchange_count` counts installed exchange records, one per `exchange`
notice. `manifest_path` names `wire-exchange.json` and `capture_digest` is
`sha256:<hex>` of its bytes. `reason` is `null` for `complete`, names the
absence for `absent`, and for `incomplete` is the sorted gap names joined by
commas, because spec 005 requires a reason for every non-complete finish. `absent` completeness (disabled retention, or no
exchange observed) installs no `wire-exchange.json`; `manifest_path`,
`capture_digest`, and `evidence_reference` are `null` and `reason` names why. A
manifest that cannot be installed downgrades `complete` to `incomplete` with
gap `manifest-not-installed` and null digest members.

The host exits 0 after writing `finished`, whatever the completeness:
completeness is testimony, not a process result. It does not read stdin after
closure and does not write after `finished`.

### 3.6 The `statecraft/wire-exchange/v1` manifest

`wire-exchange.json` is canonical compact JSON. Its member set is exact: a
member not listed here is never written, and an absent fact is one of the
listed explicit states rather than an omitted member. Arrays are sorted as
stated.

| Member | Value |
|---|---|
| `artifactType` | `statecraft/wire-exchange/v1` |
| `schemaVersion` | `1` |
| `recordDigestConstruction` | `file-bytes-sha256` |
| `producer` | `{ name, revision, contentDigest }` from section 3.7 |
| `subject` | `{ runId, attempt, sidecarEffectId }`, the binding's three values unchanged |
| `hostPolicyId` | the `start` value, uninterpreted |
| `capture` | `{ startDisposition: "listening", endDisposition }` |
| `completeness` | `{ state, gaps }`; `state` is `complete` or `incomplete`; `gaps` is sorted, empty exactly when `complete` |
| `retention` | `{ requested, result, reason }`; `result` is `metadata-only` or `failed`; `reason` is `content-retention-unavailable`, `record-not-installed` (with `failed`), or `null` |
| `redaction` | `{ result, reason, fieldClasses }`; `result` is `applied`, `not-needed`, or `failed`; `fieldClasses` is sorted |
| `attestation` | `{ state: "unknown" }`; this version signs nothing |
| `exchanges` | per recorded exchange, sorted by `sequence` (below) |
| `counts` | `{ observedExchanges, recordedExchanges, droppedExchanges, findings }` |

Each `exchanges` element has exactly:

| Member | Value |
|---|---|
| `sequence` | the spec 013 sequence |
| `recordDigest` | `sha256:<hex>` of the installed exchange record |
| `recordByteLength` | its byte length |
| `completeness` | the record's `complete`, `incomplete`, or `absent` |
| `requested` | `{ destination, protocol, model }` |
| `served` | `{ destination, protocol, model }` |
| `usage` | `{ label: "provider-testimony", values }` or `{ label: "absent" }` |
| `estimatedCost` | `{ state: "not-estimated" }` |

`destination` is the exact authority the child requested (`requested`) or the
authority the host connected to (`served`); they are equal in this version
because the host never redirects, but both are written. `protocol` is
`http/1.1`. `model` is `{ state: "known", value }`, `{ state: "absent" }`, or
`{ state: "unknown" }`, taken from spec 002 normalization; the requested model
is never copied into the served one. `values` is sorted by `providerField`,
then `location`, and each has exactly `{ providerField, location, unit, value }`:
spec 002's provider field name, its message location (a stream may report
one field twice), the provider's stated unit, else `tokens` when the
provider's own field name ends in `_tokens`, else `{ state: "unknown" }`, and
the provider's number unchanged. No price table is supplied to the sidecar, so
`estimatedCost` is always `not-estimated`; an estimate is never fabricated.

The manifest never contains a header map or value, a body or body excerpt, a
query string, a cookie, an authorization value, a token, a credential, a path
outside `output_directory`, or the CA key. `redaction.fieldClasses` names the
classes of value the exchange records omit: spec 003's `sensitive-header`,
`sensitive-field`, and `detected-credential`, and `query-string` for a
removed query. It names classes, never values. `redaction.result` is
`applied` when any class is named and `not-needed` otherwise; `reason` is
`null` in this version.

The exchange records and spec 013 capture manifest remain in
`output_directory`. Statecraft reads only `wire-exchange.json` through the
evidence reference; the other files are wire-witness custody.

### 3.7 Producer identity and schema versions

The producer identity is canonical compact JSON with exactly:

| Member | Value |
|---|---|
| `name` | `wire-witness` |
| `version` | the crate version, for example `0.1.0` |
| `releaseChannel` | `release` only when the build sets `WIRE_WITNESS_RELEASE_CHANNEL=release`; otherwise `development` |
| `executableDigest` | `sha256:<hex>` of the running executable's bytes, or `{ state: "unknown" }` when they cannot be read |
| `interface` | `wire-witness.sidecar/1` |
| `attemptBindingSchema` | `1` |
| `terminalSchema` | `1` |
| `evidence` | `[{ type: "statecraft/wire-exchange/v1", schemaVersion: 1 }]` |
| `applicationProtocols` | `["http/1.1"]` |

`ready.producer` and `finished.producer` repeat `name`, `version`,
`releaseChannel`, `executableDigest`, `interface`, `attemptBindingSchema`, and
`terminalSchema`. The manifest's `producer.revision` is `version` and
`producer.contentDigest` is `executableDigest`.

AttemptBinding schema 1 is `{ run_id: string, attempt: u32, effect_id: string }`
as spec 005 section 3.1 bounds it. Terminal schema 1 is the `finished` member
set in section 3.5. Adding an optional member keeps both versions. Removing,
renaming, or retyping a member, or changing a member's meaning, increments the
schema version and requires a new interface major `wire-witness.sidecar/2`.

The executable digest identifies bytes; it does not establish that those bytes
are a release. A `development` build is never presented as released, and
Statecraft's adoption ledger, not this document, decides what it accepts.

### 3.8 Compatibility, rollback, and qualification

`crates/wire-witness-cli/tests/fixtures/sidecar-1/` holds the interface 1
compatibility fixtures: recorded `start` lines (valid, each refusal code, and
unknown-member variants) and the expected stdout member sets and exit codes
for ready, notices, and finished. `attempt-preparation-failed` has no fixture
because no `start` line induces it. The fixture tests run the built executable,
or the one `WIRE_WITNESS_EXECUTABLE` names. A later release that claims interface 1 must
pass every fixture unchanged. A fixture may be added; changing or removing one
is an interface major change.

Each release's notes record its version, interface, schema versions,
per-platform executable and archive digests, the qualification record for each
supported platform, and its rollback identity: the prior release that claims
the same interface major, or `none` when no prior release exists. For 0.1.0 the
rollback identity is `none`; rolling back means Statecraft removes the adoption
and records witness-unavailable, never selecting another executable.

A qualification record is per release and platform, at
`qualification/<version>/<target-triple>.md`. It names the exact executable
digest, operating-system version, toolchain, the commands run, and each result:
the workspace tests, the `sidecar-1` fixtures run against the built
executable, and a supervised end-to-end capture against a local fake provider.
The executable trusts only system roots for provider TLS, which a local fake
provider cannot use, so the capture runs the same host through the library
with an exact root set; the executable's own run covers every path that does
not need provider TLS. It records residuals, including that the host
does not apply its own confinement. It qualifies those bytes on that platform
only.

### 3.9 Confinement inputs

The host applies no confinement itself; Statecraft confines it. To let a
supervisor build that profile, the host's needs are fixed: write
`output_directory`; read system trust roots and resolver configuration; bind
the one loopback endpoint; resolve names; and connect outbound to every
authority the child asks for, because spec 013 tunnels unlisted authorities
opaquely. A supervisor that narrows outbound connections makes the refused
tunnels appear as findings; the host does not treat that as a capture gap
unless the refused authority is a capture authority.

## 4. Observable negative cases

| Case | Expected |
|---|---|
| No stdin line, or first line is not `start` | Exit 2, `start-missing` or `start-malformed`; no stdout; no CA, listener, or file. |
| Empty `effect_id` or oversized `run_id` | Exit 2, `binding-invalid`; nothing created. |
| Capture authority lists only `h2` or `websocket` | Exit 2, `protocol-unsupported`. |
| `output_directory` exists or is relative | Exit 2, `output-directory-invalid`. |
| `certificate_path` outside `output_directory` | Exit 2, `certificate-path-invalid`. |
| `listen_endpoint` is `0.0.0.0:…` or a LAN address | Exit 2, `endpoint-invalid`. |
| `listen_endpoint` port already bound | Exit 2, `endpoint-unavailable`; no CA or file. |
| A second stdin line after `start` | Closes with gap `unexpected-supervisor-message`; one `finished`. |
| Deadline elapses with the child still connected | Closes with `deadline-reached`; `incomplete`. |
| Stdout closed by the supervisor | Exit 4, no `finished`; custody files still installed. |
| `content` retention requested | Effective `metadata-only` with finding; manifest retention `metadata-only`. |
| `disabled` retention | `finished` with `absent`, reason `disabled`, null digests. |
| An exchange carries an API key header | Value absent from stdout, stderr, records, both manifests; `redaction.fieldClasses` names the class. |
| Requested model differs from served model | Both recorded separately; neither copied. |
| No usage in the response | `usage` is `{ label: "absent" }`, never zero. |
| Every message written | Repeats the start binding byte for byte. |

## 5. Residual risks

- The host has no confinement of its own; it relies on the supervisor.
- `memory_lock` is `not-attempted`; the CA key may be paged to swap.
- A signal or crash leaves no terminal message; custody files may be partial
  and are named incomplete only by their absence from a manifest.
- The executable digest is self-reported by the process being identified; the
  supervisor must compare it with its own adoption ledger.

## 6. Out of scope

HTTP/2, WebSocket, content retention, signing or attestation, price tables and
cost estimation, own confinement, more than one attempt per process, Windows,
release publication, and any Statecraft-side change are out of scope.

## Verification

```verify:cli
.bin/spec-spine check --fail-on-warn
cargo test -p wire-witness-proxy --locked
cargo test -p wire-witness-cli --locked
scripts/check-boundaries.sh
```
