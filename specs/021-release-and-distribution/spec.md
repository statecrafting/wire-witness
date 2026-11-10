---
id: "021-release-and-distribution"
title: "One signed tag releases wire-witness to GitHub, crates.io, npm and PyPI"
status: draft
implementation: pending
created: "2026-11-10"
summary: >
  Proposes how the wire-witness executable and crates reach users: a release
  workflow started by the owner's signed annotated tag on main builds static
  binaries for four Unix targets once, attests them, and publishes the same
  bytes as GitHub release archives, the three crates on crates.io, an npm
  launcher with per-platform binary packages, and Python wheels. Every
  registry package is published from CI through trusted publishing with
  provenance. Modelled on spec-spine's distribution (its specs 006, 007, 019,
  209) for the binary channels and on rustev's spec 058 for the tag gate.
establishes:
  - { kind: file, path: ".github/workflows/release.yml", planned: true }
  - { kind: file, path: "scripts/release/check-tag.sh", planned: true }
  - { kind: file, path: "scripts/release/bump-version.py", planned: true }
  - { kind: file, path: "npm/package.json", planned: true }
  - { kind: file, path: "npm/bin/wire-witness.js", planned: true }
  - { kind: file, path: "npm/lib/platform.js", planned: true }
  - { kind: file, path: "npm/scripts/generate-platform-packages.js", planned: true }
  - { kind: file, path: "py/pyproject.toml", planned: true }
  - { kind: file, path: "py/scripts/generate_wheels.py", planned: true }
  - { kind: file, path: "docs/releasing.md", planned: true }
depends_on:
  - "001-boundaries-and-authority"
  - "006-standalone-host"
  - "013-live-loopback-capture-runtime"
  - "019-statecraft-profile-28"
---

# 021: One signed tag releases wire-witness to GitHub, crates.io, npm and PyPI

## 1. Purpose

wire-witness has no release. Consumers who supervise a coding agent run it
from Node and Python environments as often as from Rust, and should not need
a Rust toolchain to obtain the executable. Sibling Statecraft tools already
answer this: spec-spine ships one prebuilt CLI through GitHub, crates.io, npm
and PyPI, and rustev gates its release on the owner's signed tag. This spec
adopts the smallest combination of the two that fits one executable.

## 2. Territory

Planned units above. `Cargo.toml` changes (exact internal versions, package
metadata) are extensions of spec 001's manifest and are declared when the
implementing change is made. Statecraft-managed workflows and scripts are not
modified; `release.yml` is repository-owned and is not a required job of the
aggregate gate.

## 3. Behavior

### 3.1 Versioning

1. One lockstep version: `[workspace.package] version` is the version of all
   three crates, every npm package and the Python distribution.
2. Internal dependencies declare `{ path, version = "=X.Y.Z" }` so a packaged
   manifest resolves from the registry by version alone. Spec 019 already
   declares them, with MSRV 1.88 and a cargo-deny policy.
3. `scripts/release/bump-version.py X.Y.Z` moves Cargo, npm and Python
   versions together; `--check` refuses any disagreement. A version bump is
   its own pull request when a release cycle opens; profile revision 27
   classes such a change as version-only.

### 3.2 Trigger and tag gate

1. `push` of a tag `vX.Y.Z` or `vX.Y.Z-rc.N`, and `workflow_dispatch` as a dry
   run that builds and checks everything and publishes nothing.
2. `scripts/release/check-tag.sh` refuses unless: the tag is annotated; `git
   verify-tag` passes against the committed SSH `allowed_signers`; the tag
   version equals the workspace version; the commit is an ancestor of
   `origin/main`; and `ci-gate` succeeded on that commit.
3. An `-rc.N` tag ends at a GitHub pre-release and touches no registry.
4. A tag ruleset lets only the owner create, move or delete `v*` tags. A tag
   is never moved once anything is public.

### 3.3 Targets and build

| Target | Runner | npm package | Wheel tag |
|---|---|---|---|
| `aarch64-apple-darwin` | macOS arm64 | `@wire-witness/cli-darwin-arm64` | `macosx_11_0_arm64` |
| `x86_64-apple-darwin` | macOS (cross) | `@wire-witness/cli-darwin-x64` | `macosx_10_12_x86_64` |
| `x86_64-unknown-linux-musl` | Linux x64 | `@wire-witness/cli-linux-x64` | `manylinux_2_17_x86_64.musllinux_1_2_x86_64` |
| `aarch64-unknown-linux-musl` | Linux arm64 | `@wire-witness/cli-linux-arm64` | `manylinux_2_17_aarch64.musllinux_1_2_aarch64` |

1. Linux binaries are statically linked musl builds, so one artifact serves
   glibc and musl hosts, including Alpine containers.
2. Windows is not a target. Private custody relies on Unix modes 0700 and
   0600, which the code applies only under `cfg(unix)`; a Windows target
   requires a spec that defines equivalent custody first.
3. Each target builds once with `cargo build --release --locked -p
   wire-witness-cli`, is smoke-tested with `wire-witness identity`, and is
   archived deterministically (`SOURCE_DATE_EPOCH` at the commit time, sorted
   entries, `gzip -n`) as `wire-witness-X.Y.Z-<target>.tar.gz` with the
   binary, LICENSE and README.
4. Every channel repackages these bytes; no channel rebuilds Rust.

### 3.4 Supply-chain artifacts

1. A CycloneDX SBOM per archive; an empty SBOM fails the run.
2. `SHA256SUMS` over every release file.
3. `actions/attest-build-provenance` over every release file, with the bundle
   attached as `<file>.intoto.jsonl`. A run without an OIDC token fails before
   any attestation.

### 3.5 Publication

Each publish job runs in its own GitHub Environment with `id-token: write`
and no long-lived registry secret. Jobs are idempotent: a version already
public with the same checksum and provenance is skipped, any other is a
failure.

1. **GitHub release** (`contents: write`, the only job with it): archives,
   SBOMs, bundles and `SHA256SUMS`.
2. **crates.io** (Environment `crates-publish`):
   `rust-lang/crates-io-auth-action`, then `cargo publish --locked` in order
   `wire-witness-core`, `wire-witness-proxy`, `wire-witness-cli`.
3. **npm** (Environment `npm-publish`, npm at least 11.5.1): the main package
   `wire-witness` has `bin: { "wire-witness": "bin/wire-witness.js" }` and
   the four platform packages as `optionalDependencies` pinned to the exact
   version. Platform packages carry `os` and `cpu`, are generated from the
   archives at publish time, and are published first. The launcher resolves
   the platform package, executes the binary with inherited stdio and
   forwards its exit code, so exit codes 0 to 4 survive. No `postinstall`,
   no download at install time. Every package uses `npm publish --provenance
   --access public`.
4. **PyPI** (Environment `pypi`): one `py3-none-<tag>` wheel per target with
   the binary under the wheel's scripts, plus an sdist whose entry point
   refuses on an unsupported host. `pypa/gh-action-pypi-publish` with
   `attestations: true`.

### 3.6 Verification after publication

`verify-release` downloads the GitHub release, checks `SHA256SUMS`, and runs
`gh attestation verify --signer-workflow` for each file. A fresh-consumer job
then runs `cargo install --locked wire-witness-cli`, `npx wire-witness
identity` and `uvx wire-witness identity` against the registries and
requires the reported version.

### 3.7 Owner setup outside the code

1. GitHub Environments `crates-publish`, `npm-publish`, `pypi`; the tag
   ruleset; the owner's SSH key in `allowed_signers`.
2. Trusted publishers for each crate, each npm package and the PyPI project,
   bound to `release.yml` and the matching Environment.
3. crates.io and npm attach a trusted publisher only to an existing name. How
   the first version of each name becomes public with provenance is an open
   owner decision (§5); no package is published by hand.

## 4. Out of scope

Windows, a container image, a `curl | sh` installer, reproducible
double builds, and a Node-API addon. Each needs its own spec.

## 5. Open decisions

1. Registry names: `wire-witness` on crates.io (the CLI crate may need to be
   renamed from `wire-witness-cli` to give `cargo install wire-witness`), the
   npm scope `@wire-witness`, and the PyPI project `wire-witness`. Availability
   is unverified.
2. First publication of new crate and npm names, given that trusted
   publishing attaches only to an existing name.
3. Whether the library crates are published at all, or only the CLI.

## Verification

```verify:cli
.bin/spec-spine check --fail-on-warn
```
