---
id: "004-allowlisted-capture-proxy"
title: "Allowlisted capture proxy and transparent streams"
status: draft
implementation: pending
created: "2026-09-26"
summary: >
  Defines the per-attempt proxy that intercepts only explicitly allowlisted
  provider hosts, tunnels authentication endpoints and every other host
  without inspection, and transparently handles CONNECT, TLS, HTTP/1.1,
  HTTP/2, SSE, and WebSocket traffic with bounded parsing and backpressure.
establishes:
  - { kind: crate, id: "wire-witness-proxy", planned: true }
depends_on:
  - "001-boundaries-and-authority"
  - "002-exchange-record-and-normalization"
  - "003-redaction-custody-and-retention"
interface_references:
  - corpus: "statecraft-cli"
    spec: "004-execution-adapter"
    digest: "sha256:093f082dca2183c398957c9ee306937f14a202db9634bf2d3a23529720652c13"
    sections:
      - anchor: "3-18-the-protected-evidence-boundary"
        digest: "sha256:0901cb23659e66ff19b8f9745025c1ccf99b069a7f364f076a25fbc6ae68d101"
    obtained: "2026-09-26"
    rationale: "Keep child-facing parsers outside the trusted supervisor and within its confinement contract."
obligations:
  - id: "I-1"
    kind: invariant
    text: "Only an exact capture-allowlist authority is TLS-intercepted; authentication endpoints and every other authority are tunneled without inspection."
    anchor: "3-1-routing-and-interception"
  - id: "I-2"
    kind: invariant
    text: "Transport forwarding preserves ordering, framing, half-close behavior, and backpressure; capture work never creates an unbounded buffer."
    anchor: "3-3-transparent-streams"
  - id: "R-1"
    kind: requirement
    text: "CONNECT, TLS, HTTP/1.1, HTTP/2, SSE, and WebSocket parsing are owned by the sidecar proxy process, not by the trusted supervisor."
    anchor: "3-2-protocol-ownership"
  - id: "R-2"
    kind: requirement
    text: "Unknown events remain unknown and malformed input is a finding while opaque forwarding continues when transport safety permits."
    anchor: "3-4-bounded-observation"
  - id: "V-1"
    kind: verification
    text: "The draft compiles, the planned proxy crate is recorded, and the Statecraft confinement pin verifies."
    anchor: "verification"
    inputs:
      - "specs/004-allowlisted-capture-proxy/spec.md"
---

# 004: Allowlisted capture proxy and transparent streams

## 1. Purpose

Place risky network parsers in the per-attempt sidecar and limit interception
to the provider authorities the host named. Traffic outside that capture set is
not evidence merely because it crossed the same proxy.

## 2. Territory

This spec owns the planned `wire-witness-proxy` crate. It may depend on TLS,
HTTP, async, streaming, and WebSocket libraries and on `wire-witness-core`.
Neither the core nor the trusted Statecraft supervisor may depend on this crate.

## 3. Behavior

### 3.1 Routing and interception

The attempt configuration contains two disjoint exact authority sets, each
entry naming host, port, provider family, and allowed application protocols:

- the capture allowlist is eligible for TLS interception and observation;
- the authentication tunnel list is always passed as an opaque byte tunnel.

An authority in both lists makes configuration invalid. Matching uses the
normalized CONNECT authority and, after TLS starts, requires the SNI and
certificate name to agree with that authority. Wildcards, suffix matching,
redirect-based expansion, DNS inference, and provider-brand inference are not
allowed.

Every authority not on the capture allowlist is unintercepted. CONNECT traffic
is tunneled as opaque bytes. Plain proxy requests are forwarded without body or
header inspection and yield only a routing counter stating `unintercepted`.
Redirects are evaluated as new authorities and do not inherit interception.

The proxy never changes the destination, adds credentials, follows a redirect
on the child's behalf, retries a request, or falls back to another provider.

### 3.2 Protocol ownership

The sidecar proxy, outside the supervisor process, owns:

- CONNECT parsing and tunnel establishment;
- the child-facing TLS server and provider-facing TLS client;
- HTTP/1.1 and HTTP/2 framing;
- SSE field and event assembly; and
- WebSocket upgrade, frame, fragmentation, control-frame, and close handling.

TLS interception uses the fresh attempt CA from spec 003. The provider-facing
connection uses ordinary system trust and verifies the provider hostname. The
sidecar does not weaken provider TLS verification because the child trusts its
ephemeral CA.

ALPN negotiation is independent on the child and provider sides. Protocol
translation, if implemented, must preserve semantics and be recorded. A
protocol combination that cannot be translated transparently is refused before
forwarding request content.

### 3.3 Transparent streams

Forwarding is primary and capture is an observer. Each direction has bounded
queues with explicit high-water marks. Reads pause when the next writer is
backpressured; the proxy does not read an entire response, SSE stream, or
WebSocket message merely to normalize it.

Byte order, HTTP message order, SSE event order, WebSocket fragmentation and
control frames, EOF, half-close, cancellation, and provider close codes are
preserved. Capture output may lag only within the configured bounded queue. If
the sink cannot keep up, content capture becomes incomplete with reason
`sink-backpressure`; the proxy does not allocate without bound.

Limits cover header bytes, header count, frame bytes, message bytes, nesting,
decompressed bytes, event bytes, concurrent streams, connection count, and
idle and total time. Hitting a capture parse limit records a finding and stops
structured capture for that direction. It does not silently truncate a record
and call it complete.

### 3.4 Bounded observation

Known provider events are passed to spec 002 only after framing and bounds
checks. Unknown event names and extension fields are retained as unknown with
their order and redacted payload reference. They are never dropped or mapped to
the nearest known event.

Malformed application input is a finding. If the underlying transport remains
safe to forward opaquely, forwarding continues and the capture becomes
incomplete. Invalid TLS, HTTP, or WebSocket framing that makes forwarding
ambiguous closes the affected connection and records the exact parser class,
without echoing sensitive bytes.

The proxy binds only to a per-attempt loopback endpoint or a pre-opened
supervisor-supplied descriptor. It does not expose a LAN listener, accept an
unrelated process as a capture client, or share a CA across attempts.

## 4. Observable negative cases

| Case | Expected |
|---|---|
| A child connects to an OAuth token host on the tunnel list | Opaque tunnel, no TLS interception and no body capture. |
| A child connects to an unlisted analytics host | Opaque forwarding and an `unintercepted` counter only. |
| A capture host redirects to another hostname | The new authority is evaluated independently; it is not automatically intercepted. |
| CONNECT authority and TLS SNI disagree | Interception refuses and the connection closes with a finding. |
| An SSE stream is faster than the durable sink | Backpressure propagates or capture becomes incomplete within bounds; memory does not grow without limit. |
| An unknown WebSocket event arrives | It remains an ordered unknown event after redaction. |
| HTTP framing is ambiguous | The connection closes; no normalized success is emitted. |

## 5. Out of scope

Transparent kernel interception, packet capture, QUIC and HTTP/3, provider
traffic in this drafting session, global proxy configuration, DNS policy,
supervisor implementation, and product implementation are out of scope.

## Verification

```verify:cli
./.tooling/bin/spec-spine check --fail-on-warn
./.tooling/bin/spec-spine interface verify --spec 004 --export statecraft-cli=/Users/bart/DevWork/statecraft-cli
test ! -d crates
```
