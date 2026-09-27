# Backlog disposition

This document accounts for all 19 entries in `docs/backlog.md` as observed on
2026-09-26. The source was preserved byte for byte with SHA-256
`3f296372f70df0e285e0abbe9b904d77e8e80980660e4df9365a384b76553055`
and size 17,283 bytes.

## Evidence baseline

- Approved wire-witness corpus base: signed commit
  `ebfca9bb2cfd4f0436c710644fca888b8e3d166b`, tree
  `b51ae0631f098cceb8e40973c13b677b444c491f`.
- Retained implementation worktree: clean signed commit
  `28bdf77e743ba1e12107ba4917be350537a9eec2`, one descendant of the corpus
  base. It marks spec 001 implementation complete and contains the three-crate
  boundary implementation. Specs 002 through 006 remain pending there.
- Pinned governance tool: repository-local `spec-spine 0.27.0`, matching
  `required_version = "=0.27.0"`.
- The approved registry at the base contains specs 000 through 006. All seven
  are approved. The product specs 001 through 006 have implementation work
  scheduled by the authority graph.
- All eight existing cross-corpus interface references verified as current
  against the exact local Rustev, canonical-keysort-json, action-gate, and
  statecraft-cli checkouts. No interface-pin defect was reproduced.
- Relevant Statecraft evidence: main at
  `d77e011e39dbc4689e49492c4e528d860be7eb86`; transfer repair implemented and
  locally tested on signed commit
  `dfbe034507bfdcde94e500abad5e2c0cbc9b0a46`; spec 012 is a signed draft at
  `b04e659230b64807ad3ade2dd32f1ab9c1f86312`; spec 013 is a signed draft at
  `92097682a6578f0d0309dbf831266bd76d705337`; and the retained spec 014 branch
  has no spec 014 authored yet.
- This is local evidence only. It does not establish CI, merge, release,
  publication, deployment, adoption, provider observation, evidence admission,
  or consumer qualification.

## Disposition summary

| Primary disposition | Count |
|---|---:|
| Implementation under an existing spec | 5 |
| New local draft required | 2 |
| External-corpus work | 8 |
| Repository or release operation | 3 |
| Conditional experiment or qualification | 1 |
| **Total** | **19** |

No entry is classified merely from its checkbox. Each disposition below uses
the approved authority graph and current local evidence.

## Item-by-item dispositions

### 1. Feature 002: implement exchange records and provider normalization.

- **Owning corpus or repository:** wire-witness.
- **Existing governing spec:** approved spec
  `002-exchange-record-and-normalization`.
- **Current lifecycle:** approved contract; implementation pending at the
  approved-corpus base and retained implementation HEAD. Not implemented,
  locally tested, CI-tested, merged, released, published, adopted, observed,
  admitted, or qualified.
- **Primary disposition:** implementation under an existing spec.
- **Evidence and rationale:** spec 002 already owns the planned
  `wire_witness_core::exchange` module and defines the schema, identity,
  usage, cost, normalization, completeness, canonicalization, and negative
  cases named by the backlog. A second product spec would duplicate approved
  authority.
- **Dependencies or conditions:** begin from the retained descendant where
  spec 001 is complete, then implement and validate spec 002 with its current
  Rustev and canonical-keysort-json pins.
- **Spec created this session:** no.

### 2. Feature 003: implement redaction, custody, and retention.

- **Owning corpus or repository:** wire-witness.
- **Existing governing spec:** approved spec
  `003-redaction-custody-and-retention`.
- **Current lifecycle:** approved contract; implementation pending. No local
  implementation or later lifecycle evidence was found.
- **Primary disposition:** implementation under an existing spec.
- **Evidence and rationale:** spec 003 already owns the planned custody module
  and governs default metadata-only retention, pre-persistence redaction,
  mode-0600 atomic custody, fresh per-attempt CA handling, and the backlog's
  negative cases. The action-gate interface pin verified current.
- **Dependencies or conditions:** implemented specs 001 and 002, exact
  action-gate interface verification, and offline deterministic acceptance.
- **Spec created this session:** no.

### 3. Feature 004: implement the allowlisted capture proxy.

- **Owning corpus or repository:** wire-witness.
- **Existing governing spec:** approved spec
  `004-allowlisted-capture-proxy`.
- **Current lifecycle:** approved contract; implementation pending. The proxy
  crate exists only as the boundary shell on the retained implementation
  branch.
- **Primary disposition:** implementation under an existing spec.
- **Evidence and rationale:** spec 004 already owns the proxy crate and exact
  authority routing, hostile-protocol isolation, bounded streaming,
  backpressure, completeness, and loopback-only behavior. Its Statecraft
  interface reference verified current.
- **Dependencies or conditions:** implemented specs 001 through 003 and local
  fake-transport acceptance only.
- **Spec created this session:** no.

### 4. Feature 005: implement exact binding and the sidecar protocol.

- **Owning corpus or repository:** wire-witness.
- **Existing governing spec:** approved spec
  `005-binding-and-sidecar-protocol`.
- **Current lifecycle:** approved contract; implementation pending. No local
  sidecar-protocol implementation or later lifecycle evidence was found.
- **Primary disposition:** implementation under an existing spec.
- **Evidence and rationale:** spec 005 already owns the planned protocol module
  and governs exact `AttemptBinding`, ordered newline-delimited JSON lifecycle,
  digest-only notices, terminal closure, incomplete states, and the testimony
  versus admission boundary. Its Statecraft interface references verified.
- **Dependencies or conditions:** implemented specs 001 through 004. A later
  Statecraft adoption does not block local protocol implementation.
- **Spec created this session:** no.

### 5. Feature 006: implement the standalone and sidecar hosts.

- **Owning corpus or repository:** wire-witness.
- **Existing governing spec:** approved spec `006-standalone-host`.
- **Current lifecycle:** approved contract; implementation pending. The CLI
  crate is only the boundary shell on the retained implementation branch.
- **Primary disposition:** implementation under an existing spec.
- **Evidence and rationale:** spec 006 already owns the CLI crate and both host
  modes, unsupervised identity, child-only proxy and additional-CA settings,
  cleanup findings, separate child and witness results, and the refusal to
  mutate global configuration.
- **Dependencies or conditions:** implemented specs 001 through 005 and
  offline runtime-specific fixtures.
- **Spec created this session:** no.

### 6. Fix: integrate and qualify Statecraft's transfer-aware init repair.

- **Owning corpus or repository:** statecraft-cli.
- **Existing governing spec:** approved Statecraft spec
  `002-environment-lifecycle`.
- **Current lifecycle:** implemented and locally tested on signed Statecraft
  commit `dfbe034507bfdcde94e500abad5e2c0cbc9b0a46`; retained branch only. No
  current evidence of CI, merge to Statecraft main, release, or consumer
  qualification.
- **Primary disposition:** external-corpus work.
- **Evidence and rationale:** the changed transfer, home-flow, tests, and spec
  002 history are all Statecraft-owned. wire-witness already recovered its
  checkout and must not duplicate or repin that repair.
- **Dependencies or conditions:** Statecraft review and governed integration,
  then a supported consumer recovery qualification under separate authority.
- **Spec created this session:** no.

### 7. Fix: make first-bootstrap governance tool resolution exact.

- **Owning corpus or repository:** statecraft-cli.
- **Existing governing spec:** approved Statecraft spec
  `002-environment-lifecycle`, whose setup profile already requires the exact
  project pin and repository-local governance binary.
- **Current lifecycle:** contract exists; the first-bootstrap resolution gap
  remains an unverified Statecraft implementation concern. wire-witness used an
  explicit local override during completed recovery.
- **Primary disposition:** external-corpus work.
- **Evidence and rationale:** resolving a tool before Statecraft plans and
  writes a consumer scaffold is initialization behavior, not a wire-witness
  product contract. Repeating the completed recovery would add no evidence.
- **Dependencies or conditions:** Statecraft-owned reproducer, implementation,
  and acceptance for unavailable, exact, and stale-tool cases.
- **Spec created this session:** no.

### 8. Enhancement: author wire-witness-specific constitution principles.

- **Owning corpus or repository:** wire-witness.
- **Existing governing spec:** approved specs 001, 003, 005, and 006 contain
  the feature rules, but no ordinary spec owns future product-specific
  constitution sections.
- **Current lifecycle:** new spec 007 drafted this session with
  `implementation: deferred`. The standing constitution is intentionally
  unchanged; the proposed principles are not approved or active.
- **Primary disposition:** new local draft required.
- **Evidence and rationale:** the generic constitution explicitly reserves
  section VI onward for product principles and requires an approved ordinary
  spec claiming each section before the standing document changes. Spec 007
  proposes exact text for testimony, secret custody, ambient mutation, exact
  binding, lifecycle precision, and incomplete or residual-risk states.
- **Dependencies or conditions:** owner review and approval. If approved, the
  same governed unit must replace the generic placeholder with the exact
  approved text and regenerate artifacts.
- **Spec created this session:** yes,
  `007-repository-constitution-principles`.

### 9. Enhancement: review and ratify Statecraft spec 013 for supervised witness adoption.

- **Owning corpus or repository:** statecraft-cli.
- **Existing governing spec:** Statecraft draft
  `013-supervised-wire-witness`, amending approved Statecraft specs 003 and
  004 and pinning wire-witness spec 005.
- **Current lifecycle:** clean signed draft at
  `92097682a6578f0d0309dbf831266bd76d705337`; implementation pending. The
  exact wire-witness interface reference verified locally. Not approved,
  implemented, released, adopted, or qualified.
- **Primary disposition:** external-corpus work.
- **Evidence and rationale:** supervision, effect bracketing, confinement,
  adoption identity, rollback, and run posture are Statecraft authority. The
  existing draft means wire-witness should not duplicate the consumer
  contract.
- **Dependencies or conditions:** owner ratification, a released and qualified
  wire-witness artifact, then Statecraft implementation and qualification.
- **Spec created this session:** no.

### 10. Feature: draft, review, ratify, and implement Statecraft spec 014 for wire evidence admission.

- **Owning corpus or repository:** statecraft-cli.
- **Existing governing spec:** none yet. The retained Statecraft branch
  `014-wire-evidence-admission` is at main and contains no spec 014.
- **Current lifecycle:** unstarted draft; no implementation or later lifecycle
  evidence.
- **Primary disposition:** external-corpus work.
- **Evidence and rationale:** evidence envelopes, admission policy,
  `require_artifacts`, and `run show` are Statecraft consumer behavior. Spec
  005 deliberately stops at producing testimony and a reference.
- **Dependencies or conditions:** Statecraft spec 013 authority and exact
  producer identity. Drafting, approval, and implementation remain separate.
- **Spec created this session:** no.

### 11. Enhancement: complete declared remote setup for wire-witness.

- **Owning corpus or repository:** statecraft-cli for the desired-state and
  read-only doctor contract; a separately authorized external applier for
  remote writes; wire-witness only as a later acceptance consumer.
- **Existing governing spec:** signed Statecraft draft
  `012-remote-desired-state` at
  `b04e659230b64807ad3ade2dd32f1ab9c1f86312`.
- **Current lifecycle:** Statecraft draft, implementation pending. No applier
  selected or authorized, and no remote setup or comparison was performed.
- **Primary disposition:** external-corpus work.
- **Evidence and rationale:** the draft owns rendering and read-only
  comparison while expressly denying Statecraft remote-write authority. Remote
  application is an operational act, not a wire-witness product feature.
- **Dependencies or conditions:** owner ratification, Statecraft
  implementation, exact external plan review and authority, then independent
  remote verification without reading secret values.
- **Spec created this session:** no.

### 12. Complete the approved local implementation campaign.

- **Owning corpus or repository:** wire-witness.
- **Existing governing spec:** approved specs 001 through 006.
- **Current lifecycle:** spec 001 is implemented and locally tested on the
  retained signed implementation branch; specs 002 through 006 are approved
  with implementation pending. No aggregate completion is claimed.
- **Primary disposition:** repository or release operation.
- **Evidence and rationale:** this is orchestration of work already divided by
  the approved dependency graph, not an independently reviewable product
  contract. New feature specs would duplicate authority.
- **Dependencies or conditions:** implement one approved spec at a time on the
  retained branch, preserve signed additive history, and run each declared
  acceptance plus repository gates.
- **Spec created this session:** no.

### 13. Run final aggregate verification on the completed campaign.

- **Owning corpus or repository:** wire-witness.
- **Existing governing spec:** the combined acceptance obligations of approved
  specs 001 through 006 and repository gates.
- **Current lifecycle:** not started because specs 002 through 006 remain
  pending. No aggregate evidence record exists.
- **Primary disposition:** repository or release operation.
- **Evidence and rationale:** an aggregate run verifies a completed candidate;
  it does not create new product behavior. Its output is evidence, not a spec.
- **Dependencies or conditions:** complete all local implementation units,
  then run exact-base coupling, all declared acceptance, full offline gates,
  signatures, and clean-tree checks.
- **Spec created this session:** no.

### 14. Publish and integrate wire-witness under separate owner authority.

- **Owning corpus or repository:** wire-witness repository and its release
  channels.
- **Existing governing spec:** no new product spec is needed; approved feature
  specs and repository governance constrain the artifact.
- **Current lifecycle:** not started. Local approved specifications and one
  implementation commit do not establish push, PR, CI, merge, release,
  publication, or consumer qualification.
- **Primary disposition:** repository or release operation.
- **Evidence and rationale:** push, review, merge, release, publication, and a
  registry-only consumer test are lifecycle acts over an exact candidate.
- **Dependencies or conditions:** complete local campaign, immutable report,
  and separate owner authority for each remote and release act.
- **Spec created this session:** no.

### 15. Qualify the Statecraft consumer integration.

- **Owning corpus or repository:** statecraft-cli as consumer, using an exact
  wire-witness producer.
- **Existing governing spec:** Statecraft draft 013 and future Statecraft spec
  014; wire-witness approved spec 005 is the producer interface.
- **Current lifecycle:** deferred. Spec 013 is drafted, spec 014 is absent, and
  no exact released and qualified wire-witness producer exists.
- **Primary disposition:** external-corpus work.
- **Evidence and rationale:** consumer lifecycle, policy, absence posture,
  admission, and projection are Statecraft-owned. wire-witness qualification
  as a producer does not qualify the consumer automatically.
- **Dependencies or conditions:** ratified and implemented Statecraft specs
  013 and 014, exact published producer qualification, then offline consumer
  fixtures before any separately authorized live qualification.
- **Spec created this session:** no.

### 16. Measure Codex instruction delivery with a witnessed session.

- **Owning corpus or repository:** wire-witness owns the bounded observation
  record; the measured adapter or harness owns any broader compatibility or
  qualification claim.
- **Existing governing spec:** approved specs 002, 003, and 006 govern the
  captured request, content posture, and host. Draft spec
  `008-instruction-delivery-observation` adds the missing exact comparison and
  result contract without changing those authorities.
- **Current lifecycle:** draft contract; implementation and any live
  observation remain pending. A provider-bound measurement still requires an
  accepted witness path and explicit provider authority.
- **Primary disposition:** new local draft required.
- **Evidence and rationale:** exact target selection, three-state presence,
  component completeness, and content-free durable results are reusable
  producer behavior. One result remains testimony and cannot become adapter
  qualification or policy authority by itself.
- **Dependencies or conditions:** implemented capture and custody contracts,
  exact adapter and producer identities, a predeclared comparison plan, and
  separate provider and spending authority for a live measurement.
- **Spec created this session:** yes,
  `008-instruction-delivery-observation`.

### 17. Supply wire measurements to Statecraft cost and provider findings.

- **Owning corpus or repository:** statecraft-cli findings and decision work;
  wire-witness supplies testimony under approved spec 002.
- **Existing governing spec:** wire-witness spec 002 for labeled usage, cost,
  and requested versus served identity; Statecraft owns F-06 and F-07 and must
  own admission and interpretation.
- **Current lifecycle:** deferred pending implemented capture and Statecraft
  evidence admission. No measurement is observed or admitted.
- **Primary disposition:** external-corpus work.
- **Evidence and rationale:** the producer schema already carries the required
  facts. Turning them into Statecraft findings is consumer work and cannot
  treat an estimate as a charge or measurement as spending authority.
- **Dependencies or conditions:** wire-witness implementation, Statecraft spec
  014, policy-permitted admitted evidence, and exact source attribution.
- **Spec created this session:** no.

### 18. Use captured exchanges as Rustev replay input.

- **Owning corpus or repository:** Rustev for replay transformation and
  evaluation; wire-witness for already-governed testimony production.
- **Existing governing spec:** Rustev approved specs
  `004-evaluation-and-replay`, `006-cli-surface`, and
  `009-remote-adapter-protocol`; wire-witness spec 002 defines its producer
  record.
- **Current lifecycle:** Rustev has approved replay authority and local
  implementation history, but this cross-product transformation is deferred
  until a policy-permitted admitted capture corpus exists. No such corpus was
  inspected or created.
- **Primary disposition:** external-corpus work.
- **Evidence and rationale:** replay semantics and decision authority are
  explicitly Rustev-owned. wire-witness must not acquire evaluator authority
  merely because it produced source testimony.
- **Dependencies or conditions:** exact compatible interfaces, admitted and
  retention-permitted testimony, and Rustev-owned transformation acceptance.
- **Spec created this session:** no.

### 19. Conditional fix: repair cross-corpus interface pinning only on proof.

- **Owning corpus or repository:** spec-spine only if a minimal producer defect
  is reproduced; otherwise no work item exists.
- **Existing governing spec:** current spec-spine interface-reference contract
  as exercised by wire-witness specs 002 through 006.
- **Current lifecycle:** no defect reproduced. Eight of eight current local
  references verified. The fix remains deferred and hypothetical.
- **Primary disposition:** conditional experiment or qualification.
- **Evidence and rationale:** a speculative wire-witness or spec-spine draft
  would claim a defect contradicted by current evidence. The existing tool can
  express and verify the present pins.
- **Dependencies or conditions:** reopen only with a minimal exact-version
  reproducer. Any resulting repair belongs to the smallest producer-owned
  contract and must not add fetching, remote trust, or shared Git mutation.
- **Spec created this session:** no.

## Decision record

1. Specs 002 through 006 are not duplicated. Their approved territory matches
   the five backlog feature descriptions.
2. Spec 007 is the only new wire-witness-owned draft. Its complete proposed
   constitutional text is reviewable, but the standing constitution remains
   generic and unchanged until owner approval.
3. Codex instruction-delivery observation has a local draft contract in spec
   008. Live measurement and any adapter-wide qualification remain separate
   conditional acts.
4. Rustev replay, Statecraft supervision and admission, Statecraft setup
   repair, and remote desired state remain in their owning corpora.
5. Passing local validation does not change any lifecycle beyond drafted and
   locally validated for the files created here.
