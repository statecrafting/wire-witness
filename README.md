# wire-witness

`wire-witness` captures provider wire exchanges and produces evidence claims.
It produces testimony, never authority. The approved specification corpus and
three-crate workspace are present. The pure core implements the
`wire-witness.exchange/1` record, provider normalization, canonical JSON, and
SHA-256 digest construction. Capture, custody, proxy, sidecar, and standalone
host behavior remains pending under specs 003 through 006.

## Setup gaps

The bootstrap recorded these gaps instead of silently working around them:

| Gap in statecraft-cli initialization | Repository action |
|---|---|
| `project register` reports a new repository as ungoverned before the corpus exists and exits with a finding. | Continued with `init plan`; registration became qualified during `init apply`. |
| The first profile plan evaluates prerequisites before the same initialization writes `spec-spine.toml`. | Applied the governance scaffold first, then ran a second profile plan. |
| The generated `spec-spine.toml` leaves `required_version` commented. | Transferred the file from `managed` to `user`, then set the exact release pinned by statecraft-cli. A later initialization re-adopted the manifest entry while the transfer journal still records `user`; the exact current statecraft-cli reports `journal-disagrees`, and supported apply or revert refuses pending a producer repair. No manual manifest repair was made. |
| The profile requires but does not create `Cargo.toml`, `Cargo.lock`, or `rust-toolchain.toml`. | Created the virtual workspace and toolchain files, then generated the lockfile without adding a crate. |
| The profile accepts an authored-content script path but does not create the repository-specific script. | Created `scripts/check-authored-content.sh` before selecting it in the setup parameters. |
| Profile parameters cannot be supplied on the `init plan` command line. | Added the requested parameter block to the project declaration, then re-planned and applied it. |
| The scaffold does not create product boundary specs, repository-specific constitution text, or planned crate directories. | Authored the draft specs and left crate directories absent because this session writes no product code. |
| The scaffold does not create a product README or record bootstrap gaps. | Expanded this README with the product boundary and this gap record. |
| Initialization does not discover the profile-installed `.tooling/bin/spec-spine` when the matching version is absent from `PATH`. | Ran the exact statecraft-cli checkout with `STATECRAFT_SPEC_SPINE` naming the repository-local pinned binary. |
| Codex delivery through an `@.statecraft/AGENTS.md` root bridge is not established by the harness documentation statecraft-cli can evaluate. | Recorded the delivery as `unverified`; no alternate instruction injection was added. |
