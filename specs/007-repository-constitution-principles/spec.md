---
id: "007-repository-constitution-principles"
title: "Repository constitution principles"
status: approved
implementation: complete
created: "2026-09-26"
summary: >
  Establishes six wire-witness-specific constitution principles covering
  testimony, secret custody, ambient-system mutation, exact supervised
  binding, lifecycle claims, and explicit incomplete or residual-risk states.
  The owner-approved principles are activated in the standing constitution.
establishes:
  - { kind: section, file: "standards/spec/constitution.md", anchor: "vi-testimony-never-grants-authority" }
  - { kind: section, file: "standards/spec/constitution.md", anchor: "vii-secrets-are-reduced-before-durable-custody" }
  - { kind: section, file: "standards/spec/constitution.md", anchor: "viii-capture-never-mutates-ambient-trust-or-routing" }
  - { kind: section, file: "standards/spec/constitution.md", anchor: "ix-supervised-binding-is-supplied-exactly" }
  - { kind: section, file: "standards/spec/constitution.md", anchor: "x-lifecycle-and-evidence-claims-remain-precise" }
  - { kind: section, file: "standards/spec/constitution.md", anchor: "xi-absence-incompleteness-and-residual-risk-remain-explicit" }
depends_on:
  - "001-boundaries-and-authority"
  - "003-redaction-custody-and-retention"
  - "005-binding-and-sidecar-protocol"
  - "006-standalone-host"
obligations:
  - id: "I-1"
    kind: invariant
    text: "Approval of this spec, not draft authorship, is the authority required before its proposed text can become standing constitution text."
    anchor: "3-1-activation-boundary"
  - id: "R-1"
    kind: requirement
    text: "The activated constitution states that witness testimony grants no authority and that supervised identity is supplied exactly rather than inferred."
    anchor: "3-2-approved-normative-text"
  - id: "R-2"
    kind: requirement
    text: "The activated constitution requires redaction before durable custody, forbids ambient-system mutation, and reports lifecycle, absence, incompleteness, and residual risk precisely."
    anchor: "3-2-approved-normative-text"
  - id: "V-1"
    kind: verification
    text: "The approved spec compiles with six resolved section claims, and the standing constitution contains exactly the approved principles from section 3.2."
    anchor: "verification"
    inputs:
      - "specs/007-repository-constitution-principles/spec.md"
      - "standards/spec/constitution.md"
---

# 007: Repository constitution principles

## 1. Purpose

Establish the small set of product principles that govern every
wire-witness specification. Approved specs 001 through 006 already state the
underlying feature rules. This spec does not duplicate their feature contracts.
It establishes the durable constitutional constraints that future features must
also obey.

## 2. Territory

This spec establishes six section units in
`standards/spec/constitution.md`. It owns no product code, generated artifact,
Statecraft behavior, evidence-admission policy, provider qualification, or
remote configuration.

The constitution is managed standing content. Under its Amendment section,
this approved ordinary spec claims the affected section units and authorizes
the in-place standing-text change. Draft authorship alone authorizes neither
approval nor activation.

## 3. Behavior

### 3.1 Activation boundary

The owner approved activation on 2026-10-10 after refining principle VII to
allow content that can be safely decoded, inspected, and redacted within
configured bounds. Approval activates all six principles together: this spec
is `approved` with `implementation: complete`, its section claims are no
longer planned, and the standing constitution replaces its generic `VI onward`
placeholder with the exact approved principles below. Derived artifacts are
regenerated from those authored changes.

The principles govern ordinary specs under the constitution's existing
normative hierarchy. Activation does not itself establish implementation
conformance, release readiness, or qualification.

### 3.2 Approved normative text

The following text is approved verbatim for the standing constitution, with
heading levels adjusted to that document's structure.

#### VI. Testimony never grants authority

wire-witness records attributed observations and custody facts. A witness
record, digest, finding, attestation, confidence, provider statement, or
successful check never authorizes an effect, admits evidence, changes policy,
establishes correctness, or decides acceptance. Each consumer supplies and
records its own authority.

#### VII. Secrets are reduced before durable custody

Captured material is secret-grade until redaction proves otherwise. Credentials
and configured secret-detector findings are removed before any durable sink,
diagnostic, protocol message, log, panic, or recovery artifact can receive
content. Content that cannot be safely decoded, inspected, and redacted within
configured bounds is reduced to metadata and a finding rather than persisted.

#### VIII. Capture never mutates ambient trust or routing

Capture authority is confined to the explicitly started attempt and its child
process tree. wire-witness never changes a global proxy, trust store, shell,
browser, Git configuration, package-manager configuration, or operating-system
setting. A runtime without a safe child-only additional-CA mechanism is
unsupported and refuses before child spawn.

#### IX. Supervised binding is supplied exactly

A supervised binding is an input from the supervisor, not a correlation
result. Every run id, attempt number, and effect id is validated and repeated
byte for byte. Time, process ancestry, working directory, connection order,
provider identifiers, repository state, or content never substitute for a
missing or invalid binding.

#### X. Lifecycle and evidence claims remain precise

Specified, drafted, approved, implemented, locally tested, CI-tested, merged,
released, published, adopted, observed, admitted, and qualified are separate
claims. One never implies another. A green local command proves only the exact
command, inputs, checkout, and environment it judged.

#### XI. Absence, incompleteness, and residual risk remain explicit

Known, unknown, absent, incomplete, and deferred are distinct states and are
never collapsed into success or zero. Missing observations, capture gaps,
redaction downgrade, cleanup residue, confinement limits, unsupported
transports, and unperformed checks remain visible to consumers. A summary may
not erase a lower-level limitation or residual risk.

## 4. Authority boundaries

The principles govern wire-witness contracts within the repository. They do
not:

- grant wire-witness authority over Statecraft admission, execution, or policy;
- grant Statecraft authority to alter the witness record or its binding;
- change action-gate secret detection, Rustev replay, or another corpus;
- approve any current draft, implementation, release, or qualification; or
- authorize managed-content edits beyond the standing-constitution change
  claimed here, a remote mutation, provider activity, credential access,
  spending, publication, or deployment.

## 5. Observable negative cases

| Case | Required result |
|---|---|
| This spec is approved but the standing constitution still has its generic placeholder | Activation is incomplete; verification fails. |
| A complete capture is presented as permission to execute | Refused by the authority boundary; testimony grants no authority. |
| Content cannot be scanned safely | Content is not persisted; metadata and the limitation remain. |
| A child runtime needs a global trust-store edit | The attempt refuses before child spawn. |
| A supervised record lacks an exact supplied effect id | No supervised record is produced and correlation is not substituted. |
| A local gate passes | Only that local gate is reported; CI, merge, release, publication, adoption, observation, admission, and qualification remain unclaimed. |
| A capture gap or cleanup residue exists | It remains explicit in detailed and summary results. |

## 6. Out of scope

Product implementation, feature-specific schemas, release, publication,
provider measurement, evidence admission, and sibling-corpus changes are out
of scope. Owner approval of this spec does not substitute for the separate
owner environment approval required before its activation PR merges.

## Verification

```verify:cli
.bin/spec-spine check --fail-on-warn
.bin/spec-spine lint --fail-on-warn
.bin/spec-spine index check --fail-on-unresolved
grep -qxF 'status: approved' specs/007-repository-constitution-principles/spec.md
grep -qxF 'implementation: complete' specs/007-repository-constitution-principles/spec.md
python3 -c 'from pathlib import Path; s = Path("specs/007-repository-constitution-principles/spec.md").read_text(); c = Path("standards/spec/constitution.md").read_text(); expected = s.split("#### VI. Testimony never grants authority\n", 1)[1].split("\n## 4. Authority boundaries", 1)[0].strip().replace("\n#### ", "\n## "); actual = c.split("## VI. Testimony never grants authority\n", 1)[1].split("\n---\n", 1)[0].strip(); assert actual == expected, "standing principles differ from approved section 3.2"; assert "Replace this section with your first principle." not in c, "constitution placeholder remains"; assert "planned: true" not in s.split("---", 2)[1], "activation claims remain planned"'
```
