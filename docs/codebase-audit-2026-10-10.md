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
