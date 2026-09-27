# wire-witness backlog

Evaluated 2026-09-26 against the clean primary checkout at
`ebfca9bb2cfd4f0436c710644fca888b8e3d166b`, the clean retained implementation
worktree at `28bdf77e743ba1e12107ba4917be350537a9eec2`, and the current coordination
records under `/Users/bart/DevWork/spec-spine+statecraft-cli` and
`/Users/bart/DevWork/travel-memory`.

Specs 000 through 006 are approved. Spec 001 is implemented and locally
tested. Specs 002 through 006 are pending. Nothing is CI-tested, merged,
released, published, deployed, adopted, observed, admitted, or
consumer-qualified.

## Product features

- [ ] **Feature 002: implement exchange records and provider normalization.**
  - State: next ready unit; not started.
  - Resume from clean signed HEAD
    `28bdf77e743ba1e12107ba4917be350537a9eec2` in the retained
    `implementation/wire-witness-batch` worktree.
  - Implement `wire-witness.exchange/1` in the pure
    `wire_witness_core::exchange` module.
  - Preserve immutable binding, exchange sequence, transport metadata,
    provider family, requested and served identity, provider-attributed usage,
    reported versus estimated cost, retention and completeness, exact observed
    byte lengths and digests, and normalized events in observed order.
  - Represent known, unknown, and absent values distinctly. Never copy the
    requested model into served identity or invent zero usage or cost.
  - Normalize Anthropic Messages and OpenAI Responses, including JSON, SSE,
    and WebSocket event shapes. Preserve unknown events and extension fields;
    record malformed input as a finding and incomplete capture.
  - Produce canonical compact JSON and SHA-256 exchange digests through the
    pinned canonical-keysort-json contract.
  - Test deterministic canonicalization, unknown and malformed events,
    identity separation, usage attribution, cost labels, byte digests, and all
    declared negative cases with offline fixtures only.

- [ ] **Feature 003: implement redaction, custody, and retention.**
  - State: pending after feature 002.
  - Implement pure retention classification, redaction decisions, findings,
    and custody metadata in `wire_witness_core::custody`.
  - Default every attempt to `metadata-only`. Require an explicit per-attempt
    opt-in for `content`; record `disabled` capture as an explicit absence.
  - Remove authorization headers, API keys, cookies, OAuth credentials, and
    every value detected by the pinned action-gate secret scanner before any
    sink can persist content.
  - Downgrade an unsafe or unscannable field to metadata-only and record the
    reason. Never echo candidate secrets through logs, diagnostics, panics,
    stdio, or errors.
  - Create attempt-specific mode-0600 artifacts through complete temporary
    files and atomic new-path installation. Never overwrite an artifact or
    place raw bytes in temporary or recovery files.
  - Generate one fresh CA per attempt. Keep its private key in protected
    process memory only, zeroize it at end of life, write only the certificate,
    and record memory-lock posture honestly.
  - Test default retention, explicit content opt-in, mandatory redaction,
    unscannable input, sink failure, atomic refusal, permissions, capture
    manifests, and proof that no CA private key bytes reach disk.

- [ ] **Feature 004: implement the allowlisted capture proxy.**
  - State: pending after features 002 and 003; the proxy crate is only an empty
    shell today.
  - Implement exact, disjoint capture and authentication-tunnel authority
    sets. Intercept only exact capture authorities; tunnel authentication and
    every unlisted authority without inspection.
  - Require CONNECT authority, SNI, and certificate-name agreement. Re-evaluate
    redirects as new authorities and never infer access from wildcards, DNS,
    provider branding, or suffixes.
  - Keep all hostile CONNECT, TLS, HTTP/1.1, HTTP/2, SSE, and WebSocket parsing
    in the sidecar proxy, outside the pure core and trusted supervisor.
  - Preserve stream ordering, fragmentation, control frames, half-close,
    cancellation, and provider close codes. Use bounded queues and propagate
    backpressure without buffering whole streams.
  - Bound headers, frames, messages, decompression, nesting, events, streams,
    connections, idle time, and total time. Record parse or sink gaps instead
    of silently truncating a complete capture.
  - Bind only to a per-attempt loopback endpoint or supplied descriptor. Do not
    expose a LAN listener or share a CA between attempts.
  - Test all routing, mismatch, malformed framing, unknown-event,
    backpressure, bound, and transparent-forwarding cases with local sockets
    and fake transports only.

- [ ] **Feature 005: implement exact binding and the sidecar protocol.**
  - State: pending after features 002 through 004.
  - Implement `wire-witness.sidecar/1` as newline-delimited UTF-8 JSON in
    `wire_witness_cli::sidecar_protocol`.
  - Validate the supervisor-supplied
    `AttemptBinding { run_id, attempt, effect_id }` before creating a CA or
    listener, then repeat its values byte for byte on every protocol record.
  - Accept exactly one `start`, emit exactly one ordered `ready`, zero or more
    digest-only finding and exchange notices, and exactly one `finished`.
  - Preserve unknown fields when relaying. Treat unknown message types,
    malformed or oversized lines, invalid UTF-8, duplicate lifecycle messages,
    messages after terminal, and premature EOF as findings, never success.
  - Return manifest identity, capture digest or explicit absence, completeness,
    counts, and the `statecraft/wire-exchange/v1` evidence reference without
    admitting evidence or deciding run policy.
  - Preserve the Statecraft effect order: durable intent, sidecar spawn,
    readiness, child spawn, custody close, terminal result, then Statecraft
    outcome. Never mint, infer, amend, or silently recover a supervised binding.
  - Test crashes, supervisor death, deadlines, double start, duplicate terminal,
    broken stdout, incomplete capture, binding mismatch, and digest failure.

- [ ] **Feature 006: implement the standalone and sidecar hosts.**
  - State: pending after features 002 through 005; the CLI crate is only an
    empty shell today.
  - Implement `wire-witness run [options] -- <program> [args...]` and
    `wire-witness sidecar` over the shared core and proxy operations.
  - Require explicit capture authorities, disjoint authentication tunnels, and
    metadata-only retention unless `content` is selected for that invocation.
  - Give standalone runs a random 128-bit `unsupervised` identity that can
    never be upgraded to a Statecraft attempt from environment, ancestry,
    repository, or working-directory clues.
  - Apply proxy and additional-CA settings only to the child and descendants.
    Support the relevant process environment for Claude Code and Codex CLI
    without changing the parent shell, global proxy settings, trust stores,
    browser settings, Git configuration, or package-manager configuration.
  - Preserve ordinary trust for unintercepted tunnels. Refuse before child
    spawn if a runtime has no safe additional-CA mechanism.
  - Remove temporary certificates and trust bundles after custody closes;
    report residue as a finding. Keep child exit, witness exit, capture
    completeness, and cleanup posture separate in human and JSON results.
  - Test runtime-specific CA behavior and known Claude Code and Codex transport
    hazards against fakes. Do not contact a provider or inspect real credentials.

## Fixes and integration enhancements

- [ ] **Fix: integrate and qualify Statecraft's transfer-aware init repair.**
  - State: implemented and locally tested at Statecraft commit
    `dfbe034507bfdcde94e500abad5e2c0cbc9b0a46`; not CI-tested, merged,
    released, or consumer-qualified.
  - Review and integrate the repair under its owning approved Statecraft spec
    without hand-editing managed state.
  - Prove repeated `init plan` convergence, transfer-journal and manifest
    agreement, byte preservation, and a supported consumer recovery path.
  - Keep the completed wire-witness recovery intact. Do not repeat it or repin
    wire-witness to an unreleased Statecraft identity.

- [ ] **Fix: make first-bootstrap governance tool resolution exact.**
  - State: setup gap retained from the wire-witness bootstrap; the completed
    recovery used an explicit repository-local pinned-binary override.
  - Define and implement a supported Statecraft path that resolves the exact
    governance pin before planning, without accidentally selecting a stale
    bare `PATH` binary.
  - Preserve explicit unknown and partial findings when the exact tool is not
    available. Do not silently substitute another version or repeat the
    already completed wire-witness recovery.

- [ ] **Enhancement: author wire-witness-specific constitution principles.**
  - State: deferred consumer-owned setup. The generic managed constitution is
    still the only constitution text.
  - Add repository-owned principles only through the governed ownership and
    specification workflow, covering testimony without authority, secret-grade
    custody, no global trust mutation, exact binding, and explicit lifecycle
    claims.
  - Do not edit managed scaffold content in place or imply that generic seeded
    text already expresses these product-specific rules.

- [ ] **Enhancement: review and ratify Statecraft spec 013 for supervised witness adoption.**
  - State: clean signed draft at
    `92097682a6578f0d0309dbf831266bd76d705337`; implementation pending.
  - Review the exact immutable wire-witness spec 005 interface pin and the
    amendments to Statecraft specs 003 and 004. Ratification remains an owner
    act.
  - After ratification and an exact released wire-witness artifact exist,
    implement one supervisor-owned sidecar and endpoint per attempt, durable
    effect bracketing, exact binding propagation, child-only environment, and
    distinct witnessed and unwitnessed posture states.
  - Implement honest macOS Seatbelt and Linux Landlock confinement with every
    residual stated. Do not claim destination-address or UDP controls that
    Landlock does not provide.
  - Adopt wire-witness by exact released version and interface identity with an
    explicit rollback identity. Do not add runtime plugin loading or mutable
    `PATH` selection.

- [ ] **Feature: draft, review, ratify, and implement Statecraft spec 014 for wire evidence admission.**
  - State: unstarted. It follows the supervised-adoption contract and must use
    its own branch and governed lifecycle.
  - Admit `statecraft/wire-exchange/v1` only through Statecraft's existing
    evidence envelope and bind it to the exact run, attempt, and effect.
  - Require producer and interface identities, capture digest, retention and
    redaction results, completeness and gaps, requested and served identities,
    provider-attributed usage, labeled estimated cost, and attestation identity.
  - Allow `require_artifacts` to require the evidence type, name absence
    explicitly, and expose it through an additive `run show` field.
  - Keep testimony separate from correctness, acceptance, provider
    qualification, policy, and authority.

- [ ] **Enhancement: complete declared remote setup for wire-witness.**
  - State: Statecraft spec 012 is a clean signed draft at
    `b04e659230b64807ad3ade2dd32f1ab9c1f86312`; no applier was selected or run.
  - Review and ratify the desired remote-state contract, then implement its
    renderer and read-only `doctor --remote` comparisons without adding remote
    writes to Statecraft.
  - Select and separately authorize an external Terraform or bounded `gh api`
    applier. Apply required checks, code-owner review, merge queue, review
    environment, workflow-token defaults, secret names and scopes, and custom
    property expectations only after reviewing an exact plan.
  - Report unsupported GitHub-plan capabilities explicitly. Never weaken the
    desired state silently or read secret values.
  - Use wire-witness as the acceptance repository for fully declared setup,
    then verify every remote field independently with `doctor --remote`.

## Campaign, release, and qualification tasks

- [ ] **Complete the approved local implementation campaign.**
  - Implement one dependency-ready spec at a time in order 002 through 006,
    with one signed commit per complete unit and no signed-history rewrite.
  - For each unit, verify current producer interfaces, update only authorized
    ownership and lifecycle metadata, regenerate derived artifacts with the
    pinned tool, run declared acceptance separately from repository gates, run
    workspace tests, clippy with warnings denied, formatting, authored-content
    checks, coupling, signature verification, and require a clean worktree.
  - Use deterministic offline fixtures, synthetic provider records, local
    sockets, and fake transports. Do not use providers, real credentials, or
    paid calls during implementation.
  - If a later aggregate gate exposes an earlier defect, add a new signed fix
    commit naming the owning spec.

- [ ] **Run final aggregate verification on the completed campaign.**
  - Run the complete gate and code suite, every implemented spec's declared
    acceptance, registry and index freshness, lint, diagnostics, ownership
    coverage, all pinned cross-corpus interface checks, affected acceptance
    from the ratification base, exact-base coupling, commit signature checks,
    authored-content checks, and clean-worktree checks.
  - Produce a final immutable report with exact commits, trees, dependencies,
    commands, results, evidence identities, and honest lifecycle labels.

- [ ] **Publish and integrate wire-witness under separate owner authority.**
  - After the local campaign and final clean report, push the exact branch,
    open and review a pull request, obtain required CI evidence, merge through
    the governed path, and remove the merged task worktree and local branch.
  - Release and publish exact signed artifacts only after merge and release
    authorization. Do not infer publication from a local build or green CI.
  - Qualify the published artifact from a fresh consumer with no path or Git
    override before Statecraft adopts it.

- [ ] **Qualify the Statecraft consumer integration.**
  - Integrate ratified and implemented Statecraft specs 013 and 014 only after
    an exact qualified wire-witness producer identity exists.
  - Exercise the sidecar lifecycle, attempt binding, confinement, evidence
    envelope, required-artifact policy, absence and incomplete postures, and
    `run show` projection with offline fixtures first.
  - Keep Statecraft implementation, CI, merge, release, adoption, and live
    qualification as separate claims.

## Conditional enhancements

- [ ] **Measure Codex instruction delivery with a witnessed session.**
  - State: deferred until explicit provider authorization and an accepted
    witness path exist.
  - Determine whether Codex expands the configured instruction path into the
    provider-visible request. Record a measured result without promoting it to
    adapter qualification or policy authority.

- [ ] **Supply wire measurements to Statecraft cost and provider findings.**
  - State: deferred until wire evidence is admitted.
  - Feed labeled request counts, provider-reported usage, cache fields,
    estimated cost inputs, and requested versus served identity into the
    existing F-06 and F-07 decision work.
  - Do not treat estimates as charges, testimony as provider qualification, or
    measurement as spending authority.

- [ ] **Use captured exchanges as Rustev replay input.**
  - State: deferred until Rustev's semantic backend contract is implemented
    and an explicitly authorized capture corpus exists.
  - Transform only admitted, policy-permitted wire testimony into replay input
    for plan comparison. Keep Rustev evaluation and decision authority outside
    wire-witness.

- [ ] **Conditional fix: repair cross-corpus interface pinning only on proof.**
  - State: no current producer defect. Existing wire-witness interface pins
    verify.
  - Reopen only if a minimal reproducer proves the pinned spec-spine release
    cannot express or verify the exact wire-witness and Statecraft interface.
  - File the smallest producer-owned repair; do not add remote discovery,
    fetching, trust policy, or a shared Git transaction.

## Execution boundaries

- Use only worktrees under `~/.statecraft/worktrees/<repo>/<task>` and preserve
  dirty or retained checkouts.
- Never hand-edit generated artifacts. Regenerate and gate them with the pinned
  in-tree tool.
- Do not ratify, push, open or merge a pull request, release, publish, deploy,
  arm, enroll, inspect credentials, use a provider, spend, change protection,
  or apply remote settings without the corresponding explicit authority.
- Keep training or fine-tuning on captured output, a hosted service, global
  proxy or trust mutation, and packet-wide transparent interception outside
  this backlog unless a new governed contract explicitly admits them.
