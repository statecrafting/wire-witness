# wire-witness

Records what a program actually sent to and received from model providers.

You run a program (a coding agent, an SDK client, a script) under
`wire-witness`. For the life of that one run it gets a private loopback proxy
and a fresh certificate authority. Only the provider hosts you name are
decrypted and observed. Every exchange becomes a redacted, digest-addressed
record, and a manifest states what was captured, what was not, and why.

The output is **testimony**: attributed observations with custody facts. It
never authorizes anything, admits evidence or decides acceptance. Whoever
consumes it decides what it is worth.

## What wire-witness guarantees

- **One attempt, one trust root.** The CA key is generated after the attempt
  starts, held only in memory by the running proxy, and never written, logged
  or sent. Only its certificate reaches disk, and it is removed at cleanup.
- **Exact allowlist.** Only the `HOST:PORT` authorities you pass to
  `--capture` are intercepted. Every other connection is an opaque tunnel,
  forwarded byte for byte and never decrypted.
- **Metadata, not content.** Records hold digests, lengths, status codes,
  normalized provider events, usage and model identities. Bodies, header
  values and query strings are never retained.
- **Your machine's trust is untouched.** Proxy and CA settings reach the child
  process only, through its environment. System trust stores and other
  processes are unaffected.
- **Absence is reported, not guessed.** A client that bypasses the proxy, a
  protocol the runtime does not relay, a dropped record: each shows up as an
  `absent` or `incomplete` capture with a reason, and a non-zero exit.
- **Private custody.** Attempt directories are created mode 0700 and files
  0600, installed atomically and never overwritten.

## Install

There is no published release yet. Build from source with the toolchain
pinned in `rust-toolchain.toml`:

```sh
cargo install --locked --path crates/wire-witness-cli
wire-witness identity
```

`identity` prints the version, the executable's digest and the protocols the
runtime relays.

## Quick start

Capture one request to the Anthropic API with `curl`. No API key is needed;
the 401 is still an exchange worth recording.

```sh
mkdir -m 700 /tmp/ww
wire-witness run \
  --capture api.anthropic.com:443,anthropic,http/1.1 \
  --trust existing-roots-bundle \
  --output /tmp/ww \
  --json -- \
  curl -sS -o /dev/null -w '%{http_code}\n' \
    https://api.anthropic.com/v1/messages \
    -H 'content-type: application/json' -d '{}'
```

The child's own output comes first (`401`), then the result:

```json
{"binding":{"kind":"unsupervised","session_id":"4d15d6c4cc2d6f430e2d0967170eb91c"},
 "capture":"complete","child_exit":{"kind":"code","value":0},"exchange_count":1,
 "findings":[],"manifest_path":"/tmp/ww/4d15d6c4.../manifest.json",
 "schema":"wire-witness.result/1"}
```

The attempt directory holds the evidence:

```text
/tmp/ww/4d15d6c4cc2d6f430e2d0967170eb91c/
├── manifest.json                              # installed last
└── exchanges/
    └── exchange-00000000000000000001.json
```

An abridged exchange record. The request and response appear only as lengths
and SHA-256 digests:

```json
{
  "schema": "wire-witness.exchange/1",
  "provider": "anthropic",
  "transport": {
    "authority": "api.anthropic.com:443", "operation": "POST",
    "request_path": "/v1/messages", "protocol": "http/1.1",
    "response_status": {"status": "known", "value": 401},
    "stream_state": "complete"
  },
  "request_bytes":  {"original_length": 145, "sha256": "123d4218..."},
  "response_bytes": {"original_length": 740, "sha256": "46fb94be..."},
  "events": [{"kind": "known", "name": "error", "position": 0}],
  "usage": [],
  "served_identity": {"status": "unknown", "reason": "provider-did-not-report"},
  "cost": {"status": "unknown", "reason": "provider-did-not-report"},
  "retention": {"requested": "metadata-only", "effective": "metadata-only"},
  "completeness": {"status": "complete"}
}
```

The manifest lists each record by digest and adds routing counters
(intercepted, tunneled, refused) and custody facts such as
`"ca_private_key": "memory-only"`.

### When nothing is captured

Node's built-in `fetch` ignores proxy variables unless `NODE_USE_ENV_PROXY=1`
is set (Node 24 and later). Without it the request goes straight to the
provider, and wire-witness says so:

```text
$ wire-witness run --capture api.anthropic.com:443,anthropic,http/1.1 \
    --trust node-extra-ca-certs --output /tmp/ww -- node client.mjs
401
session 096064d48113cb2b819af5fc2cfed5d6
capture absent
child 0
exchanges 0
findings 0
$ echo $?
1
```

With `NODE_USE_ENV_PROXY=1` the same run reports `capture complete`. A client
must honor `HTTPS_PROXY` and the trust variable you choose; anything else is
reported as absent, never as clean.

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

| Option | Meaning |
|---|---|
| `--capture` | An authority to intercept and record. `PROVIDER` is `anthropic`, `openai` or `unknown`; `PROTOCOLS` joins `http/1.1`, `sse`, `h2` and `websocket` with `+`. Repeatable. |
| `--auth-tunnel` | An authority the client also needs (for example an OAuth endpoint): tunneled and counted, never decrypted. Overlapping a capture authority is refused (`invalid-authority-configuration`). Repeatable. |
| `--output` | An existing, absolute directory. Each attempt writes `DIR/<session-id>/`. |
| `--trust` | How the child learns the attempt CA (see below). |
| `--retention` | `metadata-only` (default); `disabled` (no records, capture reported absent); or `content`, which the live runtime downgrades to metadata-only with a `content-retention-unavailable` finding. |
| `--json` | Print the `wire-witness.result/1` document instead of the summary. |

The child gets `HTTP_PROXY` and `HTTPS_PROXY` (both cases) pointing at the
loopback proxy, an empty `NO_PROXY`, and one trust variable:

| `--trust` | Variable | Typical clients |
|---|---|---|
| `node-extra-ca-certs` | `NODE_EXTRA_CA_CERTS`: the attempt CA | Node.js (`fetch` needs `NODE_USE_ENV_PROXY=1`) |
| `aws-ca-bundle` | `AWS_CA_BUNDLE`: the attempt CA | AWS SDKs and CLI |
| `existing-roots-bundle` | `SSL_CERT_FILE`: system roots plus the attempt CA | curl and OpenSSL-based clients |

`wire-witness sidecar` runs one supervised attempt over the
`wire-witness.sidecar/1` stdio protocol, for a supervisor that binds the
capture to its own attempt identity. `identity --json` gives that supervisor
the executable digest to pin.

### Exit codes

| Code | Meaning |
|---|---|
| 0 | The child exited 0, capture is complete, and there are no findings. |
| 1 | A finding was recorded, capture is absent or incomplete, or the child exited non-zero. |
| 2 | Refused: a precondition failed (for example `output-directory-must-be-absolute`). |
| 3 | Usage error. |
| 4 | Unexpected failure. |

## Concepts

| Term | Meaning |
|---|---|
| Attempt | One run of one child under one fresh CA and listener, named by its session id. |
| Capture authority | A `HOST:PORT` you allow wire-witness to decrypt and record. |
| Exchange record | One request and response: transport facts, digests, normalized events, usage, identities. |
| Manifest | The attempt's index: every record by digest, routing counters, custody facts, capture status. |
| Finding | A named fact a consumer must see, such as a dropped record or an unsupported protocol. |
| Binding | Whose attempt it is: `unsupervised` for `run`, the supervisor's identity for `sidecar`. |

## Limits

- **HTTP/1.1 only, live.** Server-sent events over HTTP/1.1 are relayed and
  parsed. HTTP/2 and WebSocket have parsers but no live relay; an authority
  that allows only those is refused with a `protocol-unsupported-by-runtime`
  finding.
- **No body retention** in the live runtime (see `--retention`).
- **Unix custody.** Private directory and file modes are applied on Unix
  only.
- **Two provider families.** Anthropic and OpenAI responses are normalized;
  `unknown` records transport facts only.
- **Not released, and not yet observed against live providers at scale.**
  The test suite and the quick start above are the current evidence.

## Repository layout

| Path | Contents |
|---|---|
| `crates/wire-witness-core` | Exchange records, provider normalization, canonical bytes and digests, redaction, retention, instruction observations, usage and cost summaries, policy-bounded corpus export. |
| `crates/wire-witness-proxy` | Allowlist routing, bounded protocol parsers, the loopback listener, CONNECT tunnels, per-attempt TLS interception, the HTTP/1.1 relay. |
| `crates/wire-witness-cli` | The `wire-witness` executable: child execution, the sidecar host, attempt binding, producer identity, private custody and result rendering. |
| `specs/` | The governed specifications. |
| `docs/` | Dated audits and dispositions. |

`unsafe` code is forbidden workspace-wide, and `make boundaries` enforces the
dependency direction between the three crates.

## Development

The repository is governed by
[spec-spine](https://github.com/statecrafting/spec-spine), pinned in
`spec-spine.toml`, and its CI is rendered by Statecraft. A behavior change
starts with a spec change.

```sh
make tools        # install the pinned spec-spine into .bin
make gate         # governance: registry, index, lint, ownership, authored content
make code         # build, test, clippy, fmt, locked dependencies, boundaries
cargo deny check  # supply-chain policy (deny.toml)
```

Commit regenerated `.statecraft/derived/` shards with the change that staled
them. Contributor and agent rules are in [AGENTS.md](AGENTS.md).

## License

Apache-2.0. See [LICENSE](LICENSE).
