# wire-witness

`wire-witness` runs a program behind a per-attempt loopback proxy, captures the
exchanges it makes with allowlisted model providers, and writes attributed,
redacted observations with custody facts.

Its output is **testimony**. It never grants authority to execute, admit
evidence, change policy, or decide acceptance; a consumer decides what the
testimony is worth.

## What it does

- **Captures one attempt.** A fresh in-memory CA and a loopback listener exist
  only for the life of one child process. Only exact allowlisted authorities
  are TLS-intercepted; every other connection is an opaque tunnel.
- **Keeps metadata only.** Records hold digests, counts and normalized
  identities, never bodies, header values or query strings. The live runtime
  answers `--retention content` with metadata-only retention and a
  `content-retention-unavailable` finding.
- **Leaves ambient trust alone.** Proxy and additional-CA settings reach the
  child process only; system trust stores and routing are unchanged.
- **Reports what it did not see.** A missing observation or an unsupported
  transport is recorded as such, never filled in.

## Status

| Surface | State |
|---|---|
| Live capture over HTTP/1.1 (including SSE) | Implemented |
| Content (body) retention | Not in the live runtime; metadata-only |
| HTTP/2 and WebSocket framing | Parsers only; live capture is not claimed |
| Standalone host (`wire-witness run`) | Implemented |
| Supervised host (`wire-witness sidecar`, protocol `wire-witness.sidecar/1`) | Implemented |
| Published releases (crates.io, npm, PyPI) | Not yet; see [distribution](#distribution) |
| Live-provider observation and consumer qualification | Not yet established |

Local tests and merged source do not establish release, live-provider
observation, or Statecraft consumer qualification; each needs its own evidence.

## Install

No release is published yet. Build from source with the pinned toolchain in
`rust-toolchain.toml`:

```sh
cargo install --locked --path crates/wire-witness-cli
wire-witness identity
```

## Usage

```text
wire-witness run --capture HOST:PORT,PROVIDER,PROTOCOLS
                 [--auth-tunnel HOST:PORT,PROVIDER,PROTOCOLS]...
                 --output DIR
                 --trust node-extra-ca-certs|aws-ca-bundle|existing-roots-bundle
                 [--retention metadata-only|content|disabled] [--json]
                 -- PROGRAM [ARGS...]
wire-witness sidecar
wire-witness identity [--json]
```

- `PROVIDER` is `anthropic`, `openai` or `unknown`.
- `PROTOCOLS` joins `http/1.1`, `h2`, `sse` and `websocket` with `+`.
- `--trust` names how the child learns the attempt CA: `NODE_EXTRA_CA_CERTS`,
  `AWS_CA_BUNDLE`, or a bundle that extends the existing roots.
- `--retention` defaults to `metadata-only`.

Example: capture a Node client's calls to the Anthropic API.

```sh
wire-witness run \
  --capture api.anthropic.com:443,anthropic,http/1.1+sse \
  --trust node-extra-ca-certs \
  --output ./attempt \
  --json -- node client.js
```

Each attempt gets its own directory, `DIR/<session-id>/`, holding one
`exchanges/exchange-<sequence>.json` record per captured exchange and
`manifest.json` (`wire-witness.capture-manifest/1`), installed last. `--json` prints a
`wire-witness.result/1` document.

### Exit codes

| Code | Meaning |
|---|---|
| 0 | Child exited 0, capture complete, no findings |
| 1 | Finding: a finding, an incomplete capture, or a non-zero child exit |
| 2 | Refused: a precondition did not hold |
| 3 | Usage error |
| 4 | Unexpected failure |

### Output schemas

| Schema | Produced by |
|---|---|
| `wire-witness.exchange/1` | One normalized exchange record |
| `wire-witness.capture-manifest/1` | The attempt's manifest |
| `wire-witness.result/1` | `run --json` |
| `wire-witness.instruction-observation/1` | Instruction delivery observation |
| `wire-witness.measurement-summary/1` | Usage and cost summaries |
| `wire-witness.corpus/1` | Policy-bounded corpus export |

## Workspace

| Crate | Responsibilities |
|---|---|
| `wire-witness-core` | Exchange records, provider normalization, canonical bytes and digests, redaction, retention, instruction observations, measurement summaries, policy-bounded corpus export. |
| `wire-witness-proxy` | Exact allowlist routing, bounded protocol parsers, loopback listener, CONNECT tunnels, per-attempt TLS interception, the HTTP/1.1 relay. |
| `wire-witness-cli` | The `wire-witness` executable: child execution, the sidecar host, attempt binding, producer identity, private filesystem custody, cleanup and result rendering. |

`unsafe` code is forbidden workspace-wide. Dependency direction is checked by
`make boundaries`.

## Documentation

Behavior is specified in [`specs/`](specs/); each spec names the files it owns,
and the gate refuses source no spec claims.

| Topic | Spec |
|---|---|
| Boundaries and authority | [001](specs/001-boundaries-and-authority/spec.md) |
| Exchange record and normalization | [002](specs/002-exchange-record-and-normalization/spec.md) |
| Redaction, custody and retention | [003](specs/003-redaction-custody-and-retention/spec.md) |
| Allowlisted capture proxy | [004](specs/004-allowlisted-capture-proxy/spec.md) |
| Binding and sidecar protocol | [005](specs/005-binding-and-sidecar-protocol/spec.md), [014](specs/014-sidecar-host/spec.md) |
| Standalone host | [006](specs/006-standalone-host/spec.md) |
| Instruction delivery observation | [008](specs/008-instruction-delivery-observation/spec.md) |
| Usage and cost summaries | [009](specs/009-usage-and-cost-summaries/spec.md) |
| Corpus export | [010](specs/010-policy-bounded-corpus-export/spec.md) |
| Live loopback capture runtime | [013](specs/013-live-loopback-capture-runtime/spec.md) |
| Distribution (draft) | [021](specs/021-release-and-distribution/spec.md) |

[`docs/reference/`](docs/reference/) holds one generated page per spec:
lifecycle, obligations, owned files and relations. Regenerate it with
`scripts/reference.sh generate`; `scripts/reference.sh check` fails when a page
is stale.

Toolchain and governance history (engine pins, setup-profile revisions,
adoption evidence) lives in the specs and in
[the dated audit](docs/codebase-audit-2026-10-10.md), not here.

## Distribution

[Spec 021](specs/021-release-and-distribution/spec.md) (draft) proposes the
channels: the `wire-witness` binary as GitHub release archives, the crates on
crates.io, an npm launcher with per-platform binary packages, and Python
wheels. Every registry package is published from CI through trusted publishing
with provenance.

## Development

Use the repository-local engine. A globally installed `spec-spine` may be
another version and must not judge this repository.

```sh
make tools        # install the pinned spec-spine into .bin
make gate         # registry and index freshness, lint, ownership, authored content
make code         # build, test, clippy, fmt, locked dependency graph, boundaries
scripts/reference.sh generate   # regenerate docs/reference
```

CI runs the same scripts through the Statecraft `github-actions-rust` setup
profile; `.statecraft/AGENTS.md` describes the managed files.

## License

[Apache-2.0](LICENSE).
