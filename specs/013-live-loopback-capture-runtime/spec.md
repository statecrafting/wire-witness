---
id: "013-live-loopback-capture-runtime"
title: "Live loopback capture runtime and the wire-witness executable"
status: approved
implementation: complete
created: "2026-10-04"
summary: >
  Supplies the I/O shell that specs 003, 004, and 006 describe but the
  workspace did not contain: the wire-witness executable, a per-attempt
  loopback listener, a fresh in-memory CA, exact-allowlist TLS interception,
  an HTTP/1.1 relay that forwards verbatim and observes within bounds,
  redacted exchange records, and a capture manifest. It names every protocol
  and retention path it does not yet implement and refuses or reports them.
extends:
  - { spec: "001-boundaries-and-authority", unit: { kind: section, file: "Cargo.toml", anchor: "workspace.dependencies" }, nature: additive }
  - { spec: "001-boundaries-and-authority", unit: "Cargo.lock", nature: additive }
  - { spec: "001-boundaries-and-authority", unit: "scripts/check-boundaries.sh", nature: additive }
  - { spec: "004-allowlisted-capture-proxy", unit: { kind: section, file: "crates/wire-witness-proxy/Cargo.toml", anchor: "dependencies" }, nature: additive }
  - { spec: "004-allowlisted-capture-proxy", unit: { kind: section, file: "crates/wire-witness-proxy/src/lib.rs", anchor: "live-capture-runtime-module-export" }, nature: additive }
  - { spec: "006-standalone-host", unit: "crates/wire-witness-cli/Cargo.toml", nature: additive }
  - { spec: "006-standalone-host", unit: { kind: section, file: "crates/wire-witness-cli/src/lib.rs", anchor: "live-runtime-module-export" }, nature: additive }
  - { spec: "006-standalone-host", unit: "crates/wire-witness-cli/src/standalone_host.rs", nature: additive }
establishes:
  - { kind: section, file: "crates/wire-witness-proxy/src/live.rs", anchor: "live-capture-runtime" }
  - { kind: section, file: "crates/wire-witness-cli/src/live_runtime.rs", anchor: "live-attempt-runtime" }
  - { kind: section, file: "crates/wire-witness-cli/src/main.rs", anchor: "wire-witness-executable" }
  - { kind: section, file: "crates/wire-witness-cli/tests/live_capture.rs", anchor: "live-capture-acceptance" }
  - { kind: section, file: "crates/wire-witness-cli/tests/executable.rs", anchor: "executable-acceptance" }
depends_on:
  - "001-boundaries-and-authority"
  - "002-exchange-record-and-normalization"
  - "003-redaction-custody-and-retention"
  - "004-allowlisted-capture-proxy"
  - "006-standalone-host"
obligations:
  - id: "I-1"
    kind: invariant
    text: "The attempt CA private key is generated after the attempt begins, held only by the running proxy, and never serialized, written, logged, or sent; only its certificate reaches a file."
    anchor: "3-2-attempt-preparation-and-ephemeral-trust"
  - id: "I-2"
    kind: invariant
    text: "Only an exact capture-allowlist authority is TLS-intercepted; every other CONNECT is an opaque tunnel and every plain request is forwarded without capture."
    anchor: "3-3-listener-and-routing"
  - id: "I-3"
    kind: invariant
    text: "No durable output carries a request or response body, a header value, a query string, or the CA key; records hold metadata, digests, counts, and normalized identities only."
    anchor: "3-7-exchange-records"
  - id: "R-1"
    kind: requirement
    text: "The HTTP/1.1 relay forwards every byte verbatim and in order, refuses ambiguous framing by closing the connection, and stops structured capture at a fixed bound while forwarding continues."
    anchor: "3-5-http-1-1-relay-and-bounded-observation"
  - id: "R-2"
    kind: requirement
    text: "The child overlay names an empty proxy bypass list, so an inherited NO_PROXY cannot route a capture authority around the witness."
    anchor: "3-6-child-environment-bypass"
  - id: "R-3"
    kind: requirement
    text: "The capture manifest is installed last, lists every recorded exchange by digest, and reports capture as absent or incomplete whenever the evidence does not support complete."
    anchor: "3-8-capture-manifest-and-capture-state"
  - id: "R-4"
    kind: requirement
    text: "The wire-witness executable maps spec 006's parse, host sequence, rendering, and exit vocabulary onto this runtime, and refuses the sidecar command until its host exists."
    anchor: "3-1-the-executable"
  - id: "V-1"
    kind: verification
    text: "The relay, decoding, record, install, and executable tests pass, the fake-provider end-to-end capture passes, and the dependency boundary holds."
    anchor: "verification"
    inputs:
      - "specs/013-live-loopback-capture-runtime/spec.md"
---

# 013: Live loopback capture runtime and the wire-witness executable

## 1. Purpose

Specs 003, 004, and 006 are approved with `implementation: complete`, and their
pure decisions are implemented and tested. The workspace nevertheless had no
executable target, no socket, no TLS stack, and no certificate generation:
`standalone_host::run` took an `AttemptRuntime` that only a test fake
implemented. Nothing could capture a live exchange.

This spec supplies that runtime as one bounded slice. It realizes the parts of
specs 003, 004, and 006 that need I/O, and it names each part it does not yet
realize so that no reader mistakes this slice for the whole of spec 004.

Its protocol scope is HTTP/1.1 only. HTTP/2 and WebSocket are not captured,
and `wire-witness sidecar` stays refused (R-4) until a separate spec supplies
the stdio host.

## 2. Territory

This spec owns three new sections and two acceptance files:

- `wire_witness_proxy::live`: the loopback listener, CONNECT routing, the
  ephemeral CA, both TLS sides, the HTTP/1.1 relay, and the capture tap;
- `wire_witness_cli::live_runtime`: the `AttemptRuntime` that prepares trust
  files, runs the proxy, writes redacted exchange records, and installs the
  manifest;
- `crates/wire-witness-cli/src/main.rs`: the `wire-witness` executable; and
- the end-to-end and executable tests.

It extends, additively, spec 001's workspace dependency table, lockfile, and
boundary check; spec 004's proxy manifest and module list; and spec 006's CLI
manifest, module list, and child environment overlay. It changes no approved
spec's text. Normalization stays spec 002's, redaction decisions stay spec
003's, and routing decisions stay spec 004's `ProxyConfig::route`.

## 3. Behavior

### 3.1 The executable

The CLI crate builds a binary named `wire-witness`. It reads arguments as
UTF-8 (a non-UTF-8 argument is usage error `argument-not-utf8`), calls spec
006's `parse_arguments` and `run` with `LiveRuntime::system()`, and renders
the result as spec 006's human or `--json` value on standard output. A refusal
prints one code on standard error and exits with spec 006's vocabulary: 3 for
usage, 2 for a precondition, 4 for a failed host operation.

`wire-witness sidecar` exits 2 with `sidecar-host-not-implemented`. The spec
005 protocol types exist; a stdio host that drives this runtime from them is
not part of this slice.

### 3.2 Attempt preparation and ephemeral trust

`prepare` refuses a second preparation, and refuses a non-empty instruction
plan with `instruction-observation-unsupported-by-live-runtime` (exit 2,
before any file or socket exists), because this runtime does not yet feed
spec 008's comparison.

It then creates `<output>/<session_id>/` and its `exchanges/` directory, each
exclusively and with mode `0700`, and generates the attempt CA from
operating-system randomness with the host's clock reading as its validity
origin (one hour before to seven days after). The core and proxy read no
clock. The CA is constrained to path length zero and signs one leaf per
intercepted host, cached for the attempt.

The CA private key exists only inside the running proxy. It is never
serialized; leaf keys are serialized only into the in-memory TLS server
configuration. The certificate is written mode `0600` as `attempt-ca.pem`. For
`--trust existing-roots-bundle`, a mode-`0600` `attempt-trust-bundle.pem` holds
the system roots followed by the attempt certificate, and an empty system root
set refuses with `system-trust-unavailable`. Both files are spec 006's cleanup
paths. A failure part way through preparation removes whatever it created.

### 3.3 Listener and routing

The proxy binds `127.0.0.1` on an ephemeral port and accepts only loopback
peers, up to a fixed connection limit; further connections are refused and
counted. Each connection's first head is read within the header bound.

- A `CONNECT` head is parsed by spec 004's `parse_connect` and routed by
  `ProxyConfig::route`. An authentication tunnel or an unlisted authority is
  connected and piped as opaque bytes in both directions, with half-close
  preserved and no read timeout, and counted as `authentication_tunnels` or
  `unintercepted_tunnels`. A capture authority is intercepted under 3.4.
- Any other head is a plain proxy request. Its absolute-form target names the
  origin; the request and response are framed under 3.5 only to find message
  boundaries and forwarded without capture. Each request is counted as
  `unintercepted_plain_requests`, including one whose origin is a capture
  authority, because only TLS interception produces testimony.

The proxy never changes a destination, adds a credential, follows a redirect,
retries, or falls back. An unreachable upstream answers `502` and records
`upstream-unreachable` with the authority and error kind only.

### 3.4 TLS interception

A capture authority whose protocol set contains neither `http/1.1` nor `sse`
is refused with `502` and `protocol-unsupported-by-runtime`. Otherwise the
proxy answers `200`, completes the child-side handshake with the attempt leaf,
and requires the client's SNI to equal the CONNECT host through spec 004's
`validate_interception_identity`; a mismatch closes the connection with
`tls-identity-mismatch`. Only then does it connect upstream and complete a
provider-side handshake that verifies the provider's certificate and hostname
against the provider trust in 3.9.

Both sides offer only the ALPN protocol `http/1.1`. A client that requires
HTTP/2 fails its handshake, which is recorded; no protocol is translated.

### 3.5 HTTP/1.1 relay and bounded observation

The relay reads one request head, forwards it and its body to the provider,
then forwards each response head and body to the child, repeating while both
sides keep the connection alive. Every byte is forwarded unmodified and in
order; chunked bodies are forwarded with their chunk framing and trailers.
Response bodies are flushed per chunk, so a streamed response reaches the
child as it arrives.

Framing is strict. Conflicting `Content-Length` values, `Content-Length` with
`Transfer-Encoding`, an unsupported transfer coding, obsolete header folding,
a malformed status line, a header count or byte count over its bound, or an
unsolicited `101` closes the connection and records the parser class. A
request carrying `Upgrade` on a capture authority is answered `501` and closed
with `upgrade-capture-unsupported` before any of it is forwarded.

Capture is an observer of each direction:

- every wire byte, heads included, feeds a SHA-256 digest and length, which
  become spec 002's request and response byte references;
- transfer-decoded body bytes are kept up to `captured_body_bytes`; past it,
  capture of that direction stops with `capture-limit-exceeded` and forwarding
  continues; and
- `Content-Encoding` of `gzip`, `x-gzip`, `deflate`, and `br` is decoded within
  `decoded_body_bytes`. Any other coding, or output past the bound, leaves the
  body unavailable with a named reason.

Idle reads and writes on intercepted and plain connections time out at the
`idle` bound.

### 3.6 Child environment bypass

Spec 006's overlay also sets `NO_PROXY` and `no_proxy` to the empty string. An
inherited bypass list would otherwise send a capture authority around the
witness while the run reports nothing wrong; with it emptied, every host goes
through the proxy, and hosts outside the capture list are tunneled unchanged.

### 3.7 Exchange records

Each observed exchange becomes one `wire-witness.exchange/1` record under spec
006's unsupervised binding, with sequences assigned contiguously from one in
the order exchanges complete:

- transport metadata is `http/1.1`, the canonical authority, the method, the
  request path with any query removed, the status or `unknown`, and how the
  exchange ended;
- `redaction_count` counts spec 003's header redactions in both directions
  plus one for a removed query; no header value or query is recorded;
- a JSON request body yields the requested identity through spec 002; an empty
  body yields `absent`; anything else yields `unknown` with a reason;
- a JSON response yields served identity, usage, reported cost, and events
  through spec 002, and a `text/event-stream` response goes through spec 002's
  SSE normalization; and
- an unavailable body, a truncated transfer, or a missing response makes the
  record incomplete. A capture limit, a decode bound, or an unsupported coding
  is `limit-exceeded` with a finding naming which; a failed decode is
  `malformed`; a truncated transfer is `stream-ended`; a missing response is
  `connection-failed`.

Effective retention is `metadata-only` for both `metadata-only` and `content`.
Content retention is not implemented in this slice, so a `content` request
records the attempt-level finding `content-retention-unavailable`, as spec 003
section 3.1 requires when content cannot be made safe. Under `disabled`, no
record or byte digest is written and only a count remains.

Each record is written as `exchanges/exchange-<sequence>.json`: a complete
mode-`0600` temporary file, installed at its final name by hard link so an
existing path is never replaced, then the temporary name removed. A failed
install records `sink-failed` and the sequence as missing.

### 3.8 Capture manifest and capture state

At close the proxy stops accepting and waits up to five seconds for open
connections. The manifest `wire-witness.capture-manifest/1` is then installed
as `manifest.json` in the attempt directory, the same way, after every record.
It carries the binding, requested and effective retention, capture status and
reason, each record's sequence, relative path, digest, construction, length,
and completeness, the missing sequences, the observed and dropped counts, the
connections still open at close, the routing counters, the findings, the
custody posture (`ca_private_key: memory-only`, `memory_lock: not-attempted`),
and the offered application protocols.

Capture state is:

- `absent` with reason `disabled` when retention is disabled;
- `absent` with reason `no-exchange-observed` when no exchange reached a
  capture authority, never a complete empty capture;
- `incomplete` when any record is incomplete, a record is missing or dropped
  for backpressure, a connection was still open at close, or a connection-level
  finding names a capture authority or names no authority; and
- `complete` otherwise.

A manifest that cannot be installed downgrades `complete` to `incomplete` and
records `manifest-not-installed`; the result then has no manifest path.

### 3.9 Provider trust

`LiveRuntime::system()`, the only runtime the executable builds, verifies
providers against the operating system's ordinary roots; an empty set refuses
with `provider-trust-unavailable`. `LiveRuntime::with_provider_trust` accepts an
exact root set for callers of the library, such as a supervisor or the
end-to-end test's local provider. No command-line option selects it, and the
child's trust in the attempt CA never weakens provider verification.

## 4. Observable negative cases

| Case | Expected |
|---|---|
| The inherited environment lists a capture host in `NO_PROXY` | The overlay empties it; the exchange is captured. |
| A child connects to an unlisted host | An opaque tunnel and an `unintercepted_tunnels` count; no record. |
| The child's SNI differs from the CONNECT host | The connection closes with `tls-identity-mismatch`; capture is `incomplete`. |
| A response body exceeds the capture bound | Forwarding completes; the record is `incomplete` with `capture-limit-exceeded`. |
| A response uses `zstd` | Forwarding completes; the record is `incomplete` with `unsupported-content-encoding`. |
| A request has both `Content-Length` and `Transfer-Encoding` | The connection closes; the parser class is recorded; nothing is normalized. |
| A request asks to upgrade to WebSocket on a capture authority | `501` before forwarding, `upgrade-capture-unsupported`, capture `incomplete`. |
| The child exits without reaching a capture authority | Capture `absent`, reason `no-exchange-observed`, exit 1. |
| `--retention content` | Effective `metadata-only` and `content-retention-unavailable`. |
| A credential is sent in a header or a query | Counted in `redaction_count`; the value appears in no output. |
| `wire-witness sidecar` | Exit 2, `sidecar-host-not-implemented`. |
| `run` without `--capture` | Exit 2 before any CA, listener, or child exists. |

## 5. Residual risks

- Any process of the same user can connect to the ephemeral loopback port
  while the attempt runs. Spec 004 also allows a pre-opened supervisor
  descriptor, which this slice does not implement.
- The CA and leaf private keys live in TLS library types that are dropped at
  close but not guaranteed to be zeroized, and memory locking is not
  attempted. The manifest records `memory_lock: not-attempted`.
- Exchange bodies, including credential-bearing headers, are held in process
  memory between capture and redaction. They never reach a file.
- A runtime-specific trust variable inherited from the parent, such as
  `CURL_CA_BUNDLE`, can take precedence over the overlay's variable. The child
  then refuses interception and the failure is recorded; capture is never
  silently bypassed by it.
- The proxy connects to providers directly. An inherited upstream proxy is
  replaced for the child and not chained.

## 6. Out of scope

HTTP/2 framing and ALPN translation, WebSocket capture, content retention and
its redacted body files, spec 008 comparison in the live runtime, the spec 005
stdio sidecar host, pre-opened descriptors, upstream proxy chaining, memory
locking, retention deadlines, and release or publication are out of scope.

## Verification

```verify:cli
.bin/spec-spine check --fail-on-warn
cargo test -p wire-witness-proxy --locked
cargo test -p wire-witness-cli --locked
scripts/check-boundaries.sh
```
