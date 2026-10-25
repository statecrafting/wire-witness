# wire-witness

`wire-witness` captures provider wire exchanges and produces attributed
observations with custody facts. Its output is testimony and never grants
authority to execute, admit evidence, change policy, or decide acceptance.

## Codebase

| Crate | Implemented responsibilities |
|---|---|
| `wire-witness-core` | Exchange records, provider normalization, canonical bytes and digests, redaction, retention, instruction observations, measurement summaries, and policy-bounded corpus exports. |
| `wire-witness-proxy` | Exact allowlist routing, bounded protocol parsers, loopback listener, CONNECT tunnels, per-attempt TLS interception, and a live HTTP/1.1 relay. |
| `wire-witness-cli` | Standalone child execution, supervised `wire-witness.sidecar/1` stdio host, exact attempt binding, producer identity, private filesystem custody, cleanup, and result rendering. |

The live transport offers HTTP/1.1. Pure HTTP/2 and WebSocket framing support
does not establish live capture support for those transports. Metadata-only
retention is the default. Child-only proxy and additional-CA settings never
change ambient trust or routing. Missing observations and capture limitations
remain explicit.

## Local checks

Use the exact repository-local engine. A globally installed `spec-spine` may
have another version and must not judge this repository.

```sh
make tools
.bin/spec-spine --version
make gate
make code
make boundaries
```

`make gate` checks registry and index freshness, lint, ownership coverage,
unresolved claims, and authored content. `make code` builds and tests the
workspace, runs clippy and formatting, then checks the locked dependency graph.
`make boundaries` invokes the Bash checker explicitly on macOS and Linux.

The candidate setup uses Statecraft `github-actions-rust` revision 21 and adds
a required reusable dependency-boundary job to its hosted aggregate gate.
Managed scripts and workflows are rendered through Statecraft plan and apply;
repository-owned checks are declared as profile parameters.

## Tooling and integration status

The candidate pins spec-spine 0.31.0. The dated 2026-10-28 section of
[spec 018](specs/018-spec-spine-029-adoption-prerequisites/spec.md) records
that pin move and the profile 21 adoption together, through Statecraft CLI
main `167a02f3ce69472c029bdd64d0bcd04421e3df75`, because profile 21 refuses a
pin that does not admit its linked producer.

The earlier pin, spec-spine 0.29.0, was the newest published release checked on
2026-10-10. It was migrated through Statecraft's supported plan-bound apply
using a separate producer adoption candidate based on current main
`4c12c5236b8e507f0c7569274862a4f5579d101d`, with the CLI and linked core
both pinned to 0.29.0. The exact producer candidate and checksums are recorded
in [the audit](docs/codebase-audit-2026-10-10.md).

Approved [spec 018](specs/018-spec-spine-029-adoption-prerequisites/spec.md)
records the engine migration and profile 16 adoption. The producer prerequisite
must merge upstream before this represents delivery from Statecraft main.
Profile 16 renders the expected remote settings for read-only comparison;
local apply does not change GitHub settings.

Approved [spec 017](specs/017-profile-15-and-required-boundary-checks/spec.md)
records the profile upgrade and required boundary checks. Draft spec 016
modernizes binding-protocol verification without changing its approved source.
The owner ratified specs 017 and 018 on 2026-10-10. Protected GitHub
Environment approval remains separate; spec 016 remains draft.

[The dated codebase audit](docs/codebase-audit-2026-10-10.md) records local check
results, two stale external interface references, and remaining acceptance
work. The historical [backlog disposition](docs/backlog-disposition.md) is a
2026-09-26 snapshot rather than a current implementation inventory.

Release, publication, live-provider observation, and Statecraft consumer
qualification require separate evidence. Local tests and merged source alone
do not establish them.
