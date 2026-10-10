# Constitution (tier 2)

Durable principles that govern this corpus. This document is **tier 2**: it is
subordinate to the bootstrap spec, whose `unamendable` anchors it may not
contradict, and it governs all ordinary specs.

**Normative hierarchy (highest wins):**

1. the bootstrap spec (`000`): non-overridable.
2. this constitution.
3. the contract: a normative summary of the bootstrap spec.
4. ordinary specs: feature-level claims within this envelope.

When two specs conflict, resolve in this order, then by the typed authority
graph.

---

## I. Markdown-only authored truth

Authored truth lives only in markdown with YAML frontmatter. If a fact governs
the system, it is written in a `spec.md` (or a standards document), never in a
derived artifact.

## II. Compiler-owned JSON machine truth

Machine-consumable truth is emitted by the compiler into the derived tree and is
read only through `spec-spine` subcommands. Hand-editing a derived artifact is a
workflow violation; ad-hoc parsing of one (`jq`/`awk`/`sed`) is equally
forbidden, because a typed read fails at the deserializer instead of silently
encoding a stale assumption.

## III. Spec-first development

A change to behavior begins with a change to a spec: the spec declares the units
it owns and the typed edges to its neighbours before the code is written. The
coupling gate enforces this at PR time. The escape valve is a named, scoped
waiver in the PR body, never a silent edit to an owner spec.

## IV. Determinism and validation

Every artifact-producing function is a pure function of (config, file contents):
the same inputs produce byte-identical output. Validation is mechanical, so
staleness is detectable by content-hash comparison alone.

## V. Legacy as evidence

Code that predates a governing spec is evidence, not a violation: a spec
claiming it declares `origin.retroactive: true` rather than masquerading as a
fresh `establishes` claim. Code adopted from outside the corpus is specced **as
found**, and the behavior the adopting spec would not have chosen is recorded
under a `## Known defects` heading. A defect recorded there is not thereby
blessed: it is what a later spec is written against.

---

## VI. Testimony never grants authority

wire-witness records attributed observations and custody facts. A witness
record, digest, finding, attestation, confidence, provider statement, or
successful check never authorizes an effect, admits evidence, changes policy,
establishes correctness, or decides acceptance. Each consumer supplies and
records its own authority.

## VII. Secrets are reduced before durable custody

Captured material is secret-grade until redaction proves otherwise. Credentials
and configured secret-detector findings are removed before any durable sink,
diagnostic, protocol message, log, panic, or recovery artifact can receive
content. Content that cannot be safely decoded, inspected, and redacted within
configured bounds is reduced to metadata and a finding rather than persisted.

## VIII. Capture never mutates ambient trust or routing

Capture authority is confined to the explicitly started attempt and its child
process tree. wire-witness never changes a global proxy, trust store, shell,
browser, Git configuration, package-manager configuration, or operating-system
setting. A runtime without a safe child-only additional-CA mechanism is
unsupported and refuses before child spawn.

## IX. Supervised binding is supplied exactly

A supervised binding is an input from the supervisor, not a correlation
result. Every run id, attempt number, and effect id is validated and repeated
byte for byte. Time, process ancestry, working directory, connection order,
provider identifiers, repository state, or content never substitute for a
missing or invalid binding.

## X. Lifecycle and evidence claims remain precise

Specified, drafted, approved, implemented, locally tested, CI-tested, merged,
released, published, adopted, observed, admitted, and qualified are separate
claims. One never implies another. A green local command proves only the exact
command, inputs, checkout, and environment it judged.

## XI. Absence, incompleteness, and residual risk remain explicit

Known, unknown, absent, incomplete, and deferred are distinct states and are
never collapsed into success or zero. Missing observations, capture gaps,
redaction downgrade, cleanup residue, confinement limits, unsupported
transports, and unperformed checks remain visible to consumers. A summary may
not erase a lower-level limitation or residual risk.

---

## Amendment

This constitution is changed by an ordinary spec that is `approved`, **claims
the affected text as an authority unit**, and contradicts no `unamendable`
anchor of the bootstrap spec.

The claim uses the ordinary ownership vocabulary over a section unit of this
file: `establishes` for a principle the spec adds, `refines` (with a named
`aspect`) for one it tightens, `co_authority` for one genuinely shared.

```yaml
refines:
  - aspect: "legacy-as-evidence"
    unit: { kind: section, file: "standards/spec/constitution.md", anchor: "v-legacy-as-evidence" }
```

The anchor is the heading slug, so `## V. Legacy as evidence` is
`v-legacy-as-evidence`. `amends` is **not** the instrument: its targets are spec
ids, and this file is not a spec.

Unlike an amended `spec.md`, which is a record of what the corpus held when it
was ratified and is therefore never edited to mention its successors, this
document is a standing statement of what is true now. It is edited in place, and
its history lives in the specs that claimed each section, and in git.
