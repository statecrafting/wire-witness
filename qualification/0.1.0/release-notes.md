# wire-witness 0.1.0

The first release of `wire-witness`, carrying the `wire-witness.sidecar/1`
stdio host of spec 014. These notes are the release record spec 014 section
3.8 requires. They state what was built and qualified; they do not adopt the
release for any consumer. Statecraft's adoption ledger decides that.

## Identity

| Fact | Value |
|---|---|
| Version | `0.1.0` |
| Release channel | `release` (built with `WIRE_WITNESS_RELEASE_CHANNEL=release`) |
| Interface | `wire-witness.sidecar/1` |
| AttemptBinding schema | `1` |
| Terminal schema | `1` |
| Evidence | `statecraft/wire-exchange/v1`, schema version `1` |
| Application protocols | `http/1.1` |
| Rollback identity | `none` |

The rollback identity is `none` because no prior release claims interface
major 1. Rolling back 0.1.0 means the consumer removes the adoption and
records witness-unavailable; it never selects another executable.

## Interface digest

The protocol authority is spec 005,
`specs/005-binding-and-sidecar-protocol/spec.md`, unchanged by this release.
At the release source its section digests, as spec-spine 0.29.0 computes them,
are:

| Section | Digest |
|---|---|
| `3-1-attemptbinding` | `sha256:68aafeb66bf494eda066bff13eb506fce5f9a8b3777505a74d98ec09fa8e11ca` |
| `3-2-versioned-stdio` | `sha256:d64434f79125e34a81f99f23670275a9d7946a02df05d34b8deee56e10fca8ec` |
| `3-3-bracketed-lifetime` | `sha256:b40040b08f584f275ea4f52b4281378c23fd365c783e888fb957acf81cd9c5f1` |
| `3-4-evidence-envelope-boundary` | `sha256:f9fc1f6171b7fd157397a32b9ff1e711da66c7e6edce56401738e85c9794ab20` |

The whole-document digest spec-spine 0.29.0 observes for spec 005 is
`sha256:ae12eabd4da259e539456c7a6c3f047a9311fa25afb3de46159a2934a987cfea`.
A consumer pin made against an earlier draft of spec 005 resolves as
`sections-current` when these four section digests match.

## Platforms

| Target | Executable digest | Archive | Archive digest | Qualification |
|---|---|---|---|---|
| `aarch64-apple-darwin` | `sha256:649a2c099c6df14d9fe2a2906241032f382661389f3a30d3ffa71daed1eeb58c` | `wire-witness-0.1.0-aarch64-apple-darwin.tar.gz` | `sha256:64ccc41968420b581d58f9f638027d9a79886760aaa83b8a1f0ec02dab6dfd0e` | [`qualification/0.1.0/aarch64-apple-darwin.md`](aarch64-apple-darwin.md) |
| `aarch64-unknown-linux-gnu` | `sha256:468a8bac6b6536b64aa86f78251ff35e10842d3f98ea29bd085ca9b7b70d2c08` | `wire-witness-0.1.0-aarch64-unknown-linux-gnu.tar.gz` | `sha256:a6b493089e2ab5d74ac641dd2ce8fd6b03ad19d5fd7ccade2c84ad50787100f7` | [`qualification/0.1.0/aarch64-unknown-linux-gnu.md`](aarch64-unknown-linux-gnu.md) |
| `x86_64-unknown-linux-gnu` | `sha256:27a629a69afd0980acce778a7bbaaee7d10ab541e5adeaeb790b5d2e4126cc70` | `wire-witness-0.1.0-x86_64-unknown-linux-gnu.tar.gz` | `sha256:f0c225ae7109c4cce84742567842bdec11d9c20b6fba1de399a9424581820f4a` | [`qualification/0.1.0/x86_64-unknown-linux-gnu.md`](x86_64-unknown-linux-gnu.md) |

Each archive holds one directory, `wire-witness-0.1.0-<target>/`, with the
qualified `wire-witness` executable and `LICENSE`. Every executable was built
from commit `4a7c603b8c26eed66955b5b814af3cd576a5a3ef` with rustc 1.98.1. The
release commit adds only these records and the spec 014 claim on
`qualification/`; it changes no build input.

To check a download:

```sh
shasum -a 256 wire-witness-0.1.0-<target>.tar.gz
tar -xzf wire-witness-0.1.0-<target>.tar.gz
shasum -a 256 wire-witness-0.1.0-<target>/wire-witness
wire-witness-0.1.0-<target>/wire-witness identity --json
```

The digests must equal the table, and `identity --json` must report the same
`executableDigest` and `releaseChannel` `release`.

## Scope and residuals

- HTTP/1.1 only. HTTP/2 and WebSocket are reported, never captured.
- Retention is metadata-only; requested `content` retention is reported as
  unavailable.
- The host applies no confinement of its own. These records qualify the
  executables, not a Statecraft confinement profile.
- The supervised capture against a local fake provider runs through the
  library with an exact root set, because the executable trusts only system
  roots for provider TLS.
- The Linux qualifications ran in Debian 12 containers on Docker Desktop on an
  arm64 Mac. The x86-64 qualification ran under emulation, not on native
  x86-64 hardware.
- The macOS executable is ad-hoc signed and not notarized.
- No live provider was contacted, and nothing here is signed or attested.
