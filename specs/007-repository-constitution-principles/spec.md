---
id: "007-repository-constitution-principles"
title: "Repository constitution principles"
status: draft
implementation: deferred
created: "2026-09-26"
summary: >
  Proposes six wire-witness-specific constitution principles covering
  testimony, secret custody, ambient-system mutation, exact supervised
  binding, lifecycle claims, and explicit incomplete or residual-risk states.
  The principles remain inactive until owner approval and the governed
  standing-constitution edit.
establishes:
  - { kind: section, file: "standards/spec/constitution.md", anchor: "vi-testimony-never-grants-authority", planned: true }
  - { kind: section, file: "standards/spec/constitution.md", anchor: "vii-secrets-are-reduced-before-durable-custody", planned: true }
  - { kind: section, file: "standards/spec/constitution.md", anchor: "viii-capture-never-mutates-ambient-trust-or-routing", planned: true }
  - { kind: section, file: "standards/spec/constitution.md", anchor: "ix-supervised-binding-is-supplied-exactly", planned: true }
  - { kind: section, file: "standards/spec/constitution.md", anchor: "x-lifecycle-and-evidence-claims-remain-precise", planned: true }
  - { kind: section, file: "standards/spec/constitution.md", anchor: "xi-absence-incompleteness-and-residual-risk-remain-explicit", planned: true }
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
    anchor: "3-2-proposed-normative-text"
  - id: "R-2"
    kind: requirement
    text: "The activated constitution requires redaction before durable custody, forbids ambient-system mutation, and reports lifecycle, absence, incompleteness, and residual risk precisely."
    anchor: "3-2-proposed-normative-text"
  - id: "V-1"
    kind: verification
    text: "The draft compiles with six planned section claims while the standing constitution remains unchanged and therefore inactive."
    anchor: "verification"
    inputs:
      - "specs/007-repository-constitution-principles/spec.md"
      - "standards/spec/constitution.md"
---

# 007: Repository constitution principles

## 1. Purpose

Propose the small set of product principles that should govern every
wire-witness specification. Approved specs 001 through 006 already state the
underlying feature rules. This spec does not duplicate their feature contracts.
It proposes the durable constitutional constraints that future features must
also obey.

## 2. Territory

This draft proposes six planned section units in
`standards/spec/constitution.md`. It owns no product code, generated artifact,
Statecraft behavior, evidence-admission policy, provider qualification, or
remote configuration.

The generic constitution is managed standing content. It is not edited by this
draft. Under its Amendment section, only an approved ordinary spec claiming
the affected section units can authorize the in-place standing-text change.
Draft authorship is neither approval nor activation.

## 3. Behavior

### 3.1 Activation boundary

The complete proposed normative text is retained in section 3.2 so it can be
reviewed without pretending that the constitution already contains it. The
planned section claims describe the intended ownership graph only.

If the owner approves this spec, the approval change must also replace the
generic `VI onward` placeholder in the standing constitution with the exact
approved principles and regenerate derived artifacts. Until that owner act:

1. this spec remains `draft` with `implementation: deferred`;
2. the standing constitution remains unchanged;
3. no principle below is described as ratified, active, or constitutionally
   governing; and
4. approved specs 001 through 006 remain the current product authority.

### 3.2 Proposed normative text

The following text is proposed verbatim for the future standing constitution.

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
content. Unsafe, malformed, encrypted, compressed, unscannable, or otherwise
uncertain content is reduced to metadata and a finding rather than persisted
on a hopeful interpretation.

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

The proposed principles constrain future wire-witness contracts only. They do
not:

- grant wire-witness authority over Statecraft admission, execution, or policy;
- grant Statecraft authority to alter the witness record or its binding;
- change action-gate secret detection, Rustev replay, or another corpus;
- approve any current draft, implementation, release, or qualification; or
- authorize an edit to managed content, a remote mutation, provider activity,
  credential access, spending, publication, or deployment.

## 5. Observable negative cases

| Case | Required result |
|---|---|
| This spec exists as a draft but the standing constitution has its generic placeholder | The principles are proposed and inactive; no constitutional claim is made. |
| A complete capture is presented as permission to execute | Refused by the authority boundary; testimony grants no authority. |
| Content cannot be scanned safely | Content is not persisted; metadata and the limitation remain. |
| A child runtime needs a global trust-store edit | The attempt refuses before child spawn. |
| A supervised record lacks an exact supplied effect id | No supervised record is produced and correlation is not substituted. |
| A local gate passes | Only that local gate is reported; CI, merge, release, publication, adoption, observation, admission, and qualification remain unclaimed. |
| A capture gap or cleanup residue exists | It remains explicit in detailed and summary results. |

## 6. Out of scope

Product implementation, feature-specific schemas, standing-constitution
activation, owner approval, amendment ratification, release, publication,
provider measurement, evidence admission, and sibling-corpus changes are out
of scope.

## Verification

```verify:cli
./.tooling/bin/spec-spine check --fail-on-warn
./.tooling/bin/spec-spine lint --fail-on-warn
grep -qF 'status: draft' specs/007-repository-constitution-principles/spec.md
grep -qF 'implementation: deferred' specs/007-repository-constitution-principles/spec.md
grep -qF 'Replace this section with your first principle.' standards/spec/constitution.md
git diff --quiet -- standards/spec/constitution.md
```
