---
id: "021-node-children-use-the-proxy"
title: "Node children trusting the attempt CA also use the proxy"
status: approved
implementation: complete
created: "2026-11-10"
summary: >
  With --trust node-extra-ca-certs, the child environment overlay also sets
  NODE_USE_ENV_PROXY=1, so Node's built-in fetch and http agents (Node 24 and
  later) route through the witness instead of bypassing it.
refines:
  - aspect: "node-proxy-opt-in"
    unit: { kind: section, file: "crates/wire-witness-cli/src/standalone_host.rs", anchor: "standalone-host" }
  - aspect: "node-trust-variables"
    unit: { kind: file, path: "README.md" }
depends_on:
  - "006-standalone-host"
  - "013-live-loopback-capture-runtime"
  - "020-readme-describes-functionality"
obligations:
  - id: "R-1"
    kind: requirement
    text: "Under the node-extra-ca-certs trust mechanism the child overlay sets NODE_USE_ENV_PROXY to 1; under every other mechanism it does not set the variable."
    anchor: "3-behavior"
  - id: "V-1"
    kind: verification
    text: "The overlay unit test asserts the variable for the Node mechanism and its absence for the others, and the CLI tests pass."
    anchor: "verification"
    inputs:
      - "specs/021-node-children-use-the-proxy/spec.md"
      - "crates/wire-witness-cli/src/standalone_host.rs"
---

# 021: Node children trusting the attempt CA also use the proxy

## 1. Purpose

Node's built-in `fetch` and its default http agents ignore `HTTP_PROXY` and
`HTTPS_PROXY` unless `NODE_USE_ENV_PROXY=1` is set (Node 24 and later). A Node
child run with `--trust node-extra-ca-certs` therefore trusted the attempt CA
but sent its requests straight to the provider. The capture was correctly
reported absent with exit 1, yet the most common Node clients could not be
witnessed without a variable the user had to know about.

## 2. Territory

This spec refines spec 006's standalone host section, which owns the child
overlay, and `README.md`, which documents the trust variables.

Ratification follows spec 011 section 3.2. A change that moves a
specification to approved is a ratification event, and coupling refuses a
changed path a draft specification owns, so a spec that owns the paths a pull
request changes is ratified in that pull request. The ratification event
requires the protected `statecraft-review-exception` environment, which
ci-gate enforces for the run; `status: approved` is the content of that event,
not evidence of it.

## 3. Behavior

1. Spec 006 section 3.3's overlay gains one variable: with
   `TrustMechanism::NodeExtraCaCerts`, `NODE_USE_ENV_PROXY=1`.
2. Only that mechanism sets it. It is the only one under which Node trusts
   the attempt CA; under the others, routing Node through the proxy would
   turn an intercepted authority into a TLS failure in the child instead of
   an absent capture.
3. Like the rest of the overlay, the variable reaches only the child and its
   descendants and overrides an inherited value.
4. The sidecar host does not spawn the child; a supervisor composes the
   child's environment and is out of scope.

Verified by hand on 2026-11-10 with Node 24.6.0 and a built-in `fetch`
client: under `node-extra-ca-certs` the capture is `complete` without the
variable in the parent environment; under `existing-roots-bundle` it remains
`absent` with exit 1.

## 4. Out of scope

Proxy variables of other runtimes, and the sidecar protocol.

## Verification

```verify:cli
.bin/spec-spine check --fail-on-warn
cargo test -p wire-witness-cli --locked --lib environment_is_child_only_and_loopback_only
cargo test -p wire-witness-cli --locked
```
