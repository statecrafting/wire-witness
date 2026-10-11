# Wire Witness codebase audit, 2026-10-10

## Scope and baseline

Reviewed merged main `c777d25bb64880d01b5d93690e5a79193ada9f68`, refreshed from
GitHub on 2026-10-10. Local main and origin/main matched; no open Wire Witness
pull requests were observed. The main push workflow at that head completed
successfully (run 38086314151). Historical PR failures do not overturn that
exact-head result. Candidate hosted CI has not run.

The primary checkout contains an untracked `.tooling/` directory. Other
worktrees include an unfinished older profile-14 draft. Those bytes were
preserved. Work is isolated on `chore/toolchain-refresh-2026-10-10` under the
required Statecraft worktree root, using its shared Cargo target directory.

## Code and governance evaluation

The three-crate workspace has executable product paths rather than only
schemas: standalone child spawning, supervised sidecar stdio handling,
loopback transport, per-attempt TLS, exact identity binding, private custody,
redaction, terminal closure, and cleanup. Offline integration tests exercise
fake TLS providers and both host modes. The live transport offers HTTP/1.1;
pure framing support alone does not establish live HTTP/2 or WebSocket capture.

At the merged baseline, governance passed with zero lint errors or warnings,
24 of 24 source files specifically claimed, and all three additional governed
files claimed. Workspace build, tests, clippy with warnings denied, formatting,
and the exact dependency graph check passed. Test executables reported 44 CLI
unit tests, 3 executable tests, 3 live-capture tests, 10 sidecar-host tests,
44 core unit tests, and 21 proxy unit tests. The live-capture test suite also
runs one selected helper test in a child process; that is not a separate suite.

Specs 001 through 014 present in the baseline are approved and marked complete;
there is no spec 015. Spec 000 is approved with implementation n-a. Spec 016 is
a merged draft verification amendment, not an approved amendment. These metadata
and tests do not establish comprehensive security conformance or qualification.

No product-code defect was demonstrated during this bounded tooling review.
The improvements address evidenced setup, CI, documentation, and acceptance
issues rather than speculative runtime refactoring.

## Findings and changes

| Finding | Disposition |
|---|---|
| Profile revision 13 trails current Statecraft revision 15. | Rendered revision 15 using the exact current CLI source and supported plan-bound apply. Draft spec 017 records the adoption. |
| Local `make code` includes dependency boundaries; hosted code invokes only the managed Rust checks. | Added a required reusable boundary workflow through `ci.extra_required_jobs`, preserving the mandatory Rust job. |
| `make boundaries` runs a Bash checker through `sh`, which does not support `pipefail` on Ubuntu. | Changed the repository-owned Makefile to invoke Bash explicitly. Local and hosted checks use the same target. |
| README says no product code exists and describes recovered bootstrap gaps as current. | Replaced it with current architecture, check commands, exact tooling status, and explicit limitations. |
| Spec-spine 0.29.0 exists, but Statecraft's linked producer remains 0.28.0. | Deferred supported engine migration under draft spec 018; no pin or producer identity was falsified. |
| Older acceptance commands name `.tooling/bin/spec-spine`. | Spec 016 covers spec 005; spec 017 replaces spec 012's profile-specific acceptance. Remaining legacy acceptance needs verification amendments under spec 018. |
| Two of eight cross-corpus interface references are stale. | Recorded below for semantic review; approved source and pinned digests were preserved. |
| Current Statecraft `doctor` refuses the machine boundary. | It reports a multiply linked protected Cargo incremental object under an unrelated attest-ledger shared target. No boundary bypass or unrelated file deletion was attempted; machine qualification remains blocked. |

## Tool identities and probe

- Latest published spec-spine observed: v0.29.0, published 2026-10-05,
  [release](https://github.com/statecrafting/spec-spine/releases/tag/v0.29.0).
- Statecraft has no published GitHub release. Current source main and the
  clean local build used here:
  `f9fcc888da8b0ba4a0581124420ffc4457700436`.
- That Statecraft build supplies profile revision 15, identity
  `97fb592673c429309bbfabd93c4c8e48fdccc84b7e74fbf5df35b93a37a4086d`,
  and links spec-spine-core 0.28.0. Its supported `--spine =0.29.0` plan
  refuses before mutation because the requested version differs from the
  linked producer.
- An isolated 0.29.0 probe at the merged baseline passed compile, index,
  freshness, lint, and coverage. It found the same two stale references as
  0.28.0. This is bounded corpus compatibility evidence, not supported
  Statecraft migration or full 0.29 acceptance.

## External interface drift

All eight declared references were checked against explicitly named local
Statecraft, Rustev, action-gate, and canonical-keysort-json checkouts. Result:
one current, five sections-current, two stale, zero missing or unverified.

1. Spec 002, canonical-keysort-json spec 000: pinned
   `77a7ada646fc7d36463746ee9449ff0973a991afe3926a1581e3a796862b6216`,
   observed `101b65b48660cacb22eb9878f3050b26f706e4aa164c8ec0a438d2aa5cc83457`.
2. Spec 004, Statecraft spec 004, section
   `3-18-the-protected-evidence-boundary`: pinned
   `0901cb23659e66ff19b8f9745025c1ccf99b069a7f364f076a25fbc6ae68d101`,
   observed `4e22e015069e6c6bab94cc883d9cb9c57c45809c4c4fb43b838fee32aaa196a9`.

The second pin must be compared with upstream confinement and record-publication
changes. The canonicalization change also needs semantic review before any
replacement digest is proposed. A stale pin is an integration finding, not
permission to silently adopt new authority. Other cited Statecraft sections
remain current even where their full spec digest changed.

## Evidence and remaining work

Command logs, setup inputs, and plan/apply reports live in the ignored
`.statecraft/state/toolchain-audit/` directory of the task worktree. Baseline
workspace checks passed. Candidate governance, spec 017 acceptance (seven declared commands),
standard coupling, and actionlint on all three workflows passed. The generated
CI calls the reusable boundary workflow and includes it in aggregate `needs`.
Dash reproduced the former `pipefail` failure, while the corrected Bash
invocation passed. Spec 005 candidate acceptance resolves through draft spec
016 and passed all three declared commands, including locked CLI tests.
This validates candidate acceptance without ratifying spec 016.

A repeated Statecraft initialization plan proposed no further file writes.
The full managed coupling gate passed ordinary coupling and then returned a
finding for spec 017-owned changes because spec 017 remains draft. This is
the expected owner-ratification boundary, not a cleared merge gate.

Draft specs 017 and 018 are proposals. Owner ratification and the protected
`statecraft-review-exception` approval remain distinct from implementation.
No remote settings, release, provider activity, publication, deployment,
adoption by a supervisor, or consumer qualification is claimed. Statecraft's
Codex import delivery assessment remains unverified.


## Current upgrade candidate, 2026-10-10

This section supersedes the earlier candidate's tooling status, not its
historical baseline evidence. The owner requested the newest dependencies.
The latest published spec-spine remains v0.29.0. Refreshed Statecraft main
`4c12c5236b8e507f0c7569274862a4f5579d101d` supplies profile 16 but
still links spec-spine-core 0.28.0. A separate isolated producer adoption
candidate `1e8d51a924422af958e5c8c3cba3b9aa7729663a` moves Statecraft's
exact CLI and linked-core pins to 0.29.0. Its Git signature verifies.

Wire Witness was migrated with that build through `init plan --spine =0.29.0`
and matching plan-bound apply. Apply returned `ok` (exit 0), compiled and
indexed the corpus, and reported registration qualified with arming false.
This initialization verdict is distinct from machine qualification, release,
owner ratification and upstream merge.

The resulting declaration pins engine and producer 0.29.0 with registry core
checksum `7567085df32090df82182bc1449012ae12229c138f307ed425a7672d22a1e5a5`.
It carries profile 16 identity
`84502467bf0415db39b0bf77b9bc072cb09c7ab49505371306b80ab198f45e9f`.
Profile 16 adds the remote desired-state document for read-only comparison;
no GitHub settings were applied. The extra required dependency-boundary job,
workspace checks and all recorded governance parameters are preserved.

Local validation passed: workspace build and tests, clippy with warnings
denied, formatting, exact dependency boundaries, corpus freshness, lint,
coverage, unresolved claims, authored content, workflow lint, ordinary
coupling, and all eight commands of spec 018 acceptance. Spec 005 acceptance
also passed through its existing draft verification amendment. The spec 012
verification plan resolves transitively through 017 to 018. All managed-file
digests match, previously approved specs remain byte-identical to merged main, and a
second initialization plan reports no writes.

The existing two stale interface pins and remaining historical acceptance
paths are separate semantic and verification-amendment work. They were not
silently repinned or rewritten. The owner subsequently ratified specs 017
and 018 as recorded below. The protected owner Environment approval remains
binding.
The latest machine doctor again refused the pre-existing multiply linked
protected Cargo object under the unrelated attest-ledger shared target.

Exact plan, apply, idempotence, checks and binary hashes are retained in the
ignored `.statecraft/state/toolchain-audit/upgrade029-*` evidence files.
The validated Statecraft build has SHA-256
`192a2fe0faf8150cb3c8749f28223ad2b605fd77bdf8b00dbe8b9cc8ffb1bf4b`;
the repository-local spec-spine has SHA-256
`2dd46867de2d687c321b431f49fd006639cb703ee57b697b7e9f8897124afafc`.


The first hosted review questioned whether spec 017's superseded profile-15
verification could run before its amendment. Direct execution on the upgrade
commit, `.bin/spec-spine verify 017`, passed all eight commands from spec 018,
including profile 16 and version 0.29.0 assertions. Verification resolves the
amendment before execution; the older block is historical, just like the
approved spec 012 block it replaced. The spec 012 plan independently confirms
that transitive resolution. Logs are `upgrade029-profile-verify.log` and
`upgrade029-profile-plan.log`. Replacing approved history is unnecessary to
make effective acceptance current. The spec 017 and 018 source bytes are
unchanged by this clarification.

### Owner ratification of specs 017 and 018

On 2026-10-10 the owner explicitly replied "Ratify specs 017 and 018 as written"
to the ratification request naming PR #14 and signed commit
`8b3a95411a12118c164606d553130e7e8799e2a0`. The two spec files still
matched that commit byte for byte before recording `status: approved`. No
requirements or verification commands changed in the ratification commit.
Protected GitHub Environment approval and upstream producer merge remain
separate delivery requirements.

The latest hosted review cleared the superseded-verification finding. Its
remaining operational concern was addressed by explicitly installing the
`rust-toolchain.toml` channel with the minimal profile and caching Rust
toolchains by runner platform and pin-file digest in the boundary workflow.
The existing approved pinned-toolchain requirement is unchanged.
