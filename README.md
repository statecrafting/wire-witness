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

The candidate setup uses Statecraft `github-actions-rust` revision 15 and adds
a required reusable dependency-boundary job to its hosted aggregate gate.
Managed scripts and workflows are rendered through Statecraft plan and apply;
repository-owned checks are declared as profile parameters.

## Tooling and integration status

The supported exact engine pin remains spec-spine 0.28.0. The newest published
release checked on 2026-10-10 is 0.29.0, but current Statecraft main still links
spec-spine-core 0.28.0 and refuses the supported 0.29.0 pin migration. Draft
[spec 018](specs/018-spec-spine-029-adoption-prerequisites/spec.md) records the
prerequisite and adoption checks. A successful standalone probe is not adoption.

Draft [spec 017](specs/017-profile-15-and-required-boundary-checks/spec.md)
records the profile upgrade and required boundary checks. Draft spec 016
modernizes binding-protocol verification without changing its approved source.
These drafts do not supply owner ratification or protected Environment approval.

[The dated codebase audit](docs/codebase-audit-2026-10-10.md) records local check
results, two stale external interface references, and remaining acceptance
work. The historical [backlog disposition](docs/backlog-disposition.md) is a
2026-09-26 snapshot rather than a current implementation inventory.

Release, publication, live-provider observation, and Statecraft consumer
qualification require separate evidence. Local tests and merged source alone
do not establish them.
