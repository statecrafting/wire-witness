# Backlog disposition

This document accounts for all 19 entries in `docs/backlog.md` as observed on
2026-09-26. The source was preserved byte for byte with SHA-256
`3f296372f70df0e285e0abbe9b904d77e8e80980660e4df9365a384b76553055`
and size 17,283 bytes.

## Evidence baseline

- Current merged base: `origin/main` at
  `360ce30126121b75b8241b0b70096cc51e5ed7d4`, tree
  `624fa1b7c64418d52f5da6fc8576471c35c5d974`. The same tree was verified on
  signed implementation tip `90e0673452d257b4124eae6aad037924157f0981`.
- Approved specs 001 through 006 have `implementation: complete` on that base.
  Their three-crate implementation, declared acceptance, repository gates,
  and merged PR are present. This evidence does not establish a release,
  publication, deployment, adoption, provider observation, admission, or
  consumer qualification.
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
| Implemented under an existing spec | 5 |
| New local draft required | 4 |
| External-corpus work | 6 |
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
- **Current lifecycle:** approved, implemented, locally tested, CI-tested, and
  merged on the current base. Not released, published, adopted, observed,
  admitted, or consumer-qualified.
- **Primary disposition:** implemented under an existing spec.
- **Evidence and rationale:** spec 002 already owns the planned
  `wire_witness_core::exchange` module and defines the schema, identity,
  usage, cost, normalization, completeness, canonicalization, and negative
  cases named by the backlog. A second product spec would duplicate approved
  authority.
- **Dependencies or conditions:** later release or consumer work must preserve
  the exact Rustev and canonical-keysort-json interface pins.
- **Spec created this session:** no.

### 2. Feature 003: implement redaction, custody, and retention.

- **Owning corpus or repository:** wire-witness.
- **Existing governing spec:** approved spec
  `003-redaction-custody-and-retention`.
- **Current lifecycle:** approved, implemented, locally tested, CI-tested, and
  merged on the current base. No release or later lifecycle evidence is
  claimed.
- **Primary disposition:** implemented under an existing spec.
- **Evidence and rationale:** spec 003 already owns the planned custody module
  and governs default metadata-only retention, pre-persistence redaction,
  mode-0600 atomic custody, fresh per-attempt CA handling, and the backlog's
  negative cases. The action-gate interface pin verified current.
- **Dependencies or conditions:** later release or consumer work must preserve
  exact action-gate interface verification and custody guarantees.
- **Spec created this session:** no.

### 3. Feature 004: implement the allowlisted capture proxy.

- **Owning corpus or repository:** wire-witness.
- **Existing governing spec:** approved spec
  `004-allowlisted-capture-proxy`.
- **Current lifecycle:** approved, implemented, locally tested, CI-tested, and
  merged on the current base. No release or later lifecycle evidence is
  claimed.
- **Primary disposition:** implemented under an existing spec.
- **Evidence and rationale:** spec 004 already owns the proxy crate and exact
  authority routing, hostile-protocol isolation, bounded streaming,
  backpressure, completeness, and loopback-only behavior. Its Statecraft
  interface reference verified current.
- **Dependencies or conditions:** later qualification must preserve the
  allowlist boundary and begin with local fake-transport acceptance.
- **Spec created this session:** no.

### 4. Feature 005: implement exact binding and the sidecar protocol.

- **Owning corpus or repository:** wire-witness.
- **Existing governing spec:** approved spec
  `005-binding-and-sidecar-protocol`.
- **Current lifecycle:** approved, implemented, locally tested, CI-tested, and
  merged on the current base. No release, Statecraft adoption, or later
  lifecycle evidence is claimed.
- **Primary disposition:** implemented under an existing spec.
- **Evidence and rationale:** spec 005 already owns the planned protocol module
  and governs exact `AttemptBinding`, ordered newline-delimited JSON lifecycle,
  digest-only notices, terminal closure, incomplete states, and the testimony
  versus admission boundary. Its Statecraft interface references verified.
- **Dependencies or conditions:** a later Statecraft adoption remains a
  separate consumer act.
- **Spec created this session:** no.

### 5. Feature 006: implement the standalone and sidecar hosts.

- **Owning corpus or repository:** wire-witness.
- **Existing governing spec:** approved spec `006-standalone-host`.
- **Current lifecycle:** approved, implemented, locally tested, CI-tested, and
  merged on the current base. No release or runtime qualification is claimed.
- **Primary disposition:** implemented under an existing spec.
- **Evidence and rationale:** spec 006 already owns the CLI crate and both host
  modes, unsupervised identity, child-only proxy and additional-CA settings,
  cleanup findings, separate child and witness results, and the refusal to
  mutate global configuration.
- **Dependencies or conditions:** later qualification begins with offline
  runtime-specific fixtures and requires separate live authority.
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
- **Current lifecycle:** complete, locally tested, CI-tested, and merged on the
  current base. No release, publication, deployment, or adoption is claimed.
- **Primary disposition:** repository or release operation.
- **Evidence and rationale:** this is orchestration of work already divided by
  the approved dependency graph, not an independently reviewable product
  contract. New feature specs would duplicate authority.
- **Dependencies or conditions:** any follow-on release or adoption must bind
  the exact merged tree and preserve the existing validation evidence.
- **Spec created this session:** no.

### 13. Run final aggregate verification on the completed campaign.

- **Owning corpus or repository:** wire-witness.
- **Existing governing spec:** the combined acceptance obligations of approved
  specs 001 through 006 and repository gates.
- **Current lifecycle:** completed locally and in CI for the merged candidate.
  The merged tree matches the verified implementation tip tree.
- **Primary disposition:** repository or release operation.
- **Evidence and rationale:** an aggregate run verifies a completed candidate;
  it does not create new product behavior. Its output is evidence, not a spec.
- **Dependencies or conditions:** rerun against any changed release candidate;
  the existing result applies only to the exact merged tree.
- **Spec created this session:** no.

### 14. Publish and integrate wire-witness under separate owner authority.

- **Owning corpus or repository:** wire-witness repository and its release
  channels.
- **Existing governing spec:** no new product spec is needed; approved feature
  specs and repository governance constrain the artifact.
- **Current lifecycle:** branch push, PR, CI, and merge are complete. Release,
  publication, deployment, adoption, and consumer qualification remain
  unclaimed.
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
  unique normalized-component resolution, component completeness, and
  content-free durable results are reusable producer behavior. One result
  remains testimony and cannot become adapter qualification or policy
  authority by itself.
- **Dependencies or conditions:** implemented capture and custody contracts,
  exact adapter and producer identities, a predeclared comparison plan, and
  separate provider and spending authority for a live measurement.
- **Spec created this session:** yes,
  `008-instruction-delivery-observation`.

### 17. Supply wire measurements to Statecraft cost and provider findings.

- **Owning corpus or repository:** wire-witness owns deterministic producer
  summaries; statecraft-cli owns evidence admission, findings, interpretation,
  scheduling, and spending authority.
- **Existing governing spec:** approved wire-witness spec 002 defines each
  exchange measurement. Draft spec `009-usage-and-cost-summaries` adds the
  missing binding-level aggregation contract without changing Statecraft's
  F-06 or F-07 authority.
- **Current lifecycle:** draft producer contract; implementation, observation,
  admission, and Statecraft findings remain pending.
- **Primary disposition:** new local draft required.
- **Evidence and rationale:** deterministic grouping, source-preserving decimal
  addition, canonical group ordering, identity inventories, gap handling, and
  an ordered input manifest are reusable producer behavior. Statecraft
  interpretation remains consumer work and cannot treat an estimate as a
  charge or a summary as spending authority.
- **Dependencies or conditions:** implemented exchange and binding contracts,
  then Statecraft evidence admission and policy-permitted consumer use.
- **Spec created this session:** yes, `009-usage-and-cost-summaries`.

### 18. Use captured exchanges as Rustev replay input.

- **Owning corpus or repository:** wire-witness for an immutable producer
  export manifest; Rustev for transformation, replay, and evaluation.
- **Existing governing spec:** approved wire-witness specs 002, 003, and 005
  govern the source testimony. Draft specs 009 and
  `010-policy-bounded-corpus-export` define optional summaries and the missing
  policy-bounded export contract. Rustev approved specs
  `004-evaluation-and-replay`, `006-cli-surface`, and
  `009-remote-adapter-protocol` retain consumer authority.
- **Current lifecycle:** draft producer contract; implementation, an admitted
  and retention-permitted corpus, transformation, replay, and evaluation all
  remain pending.
- **Primary disposition:** new local draft required.
- **Evidence and rationale:** deterministic selection, source identity,
  attributed policy receipts, materialization, expiry, gaps, bounds, and
  retention preservation are reusable producer behavior. Replay schema,
  equivalence, execution, comparison, and evaluation remain explicitly
  Rustev-owned.
- **Dependencies or conditions:** implementation of the producer draft, an
  explicit external policy decision, eligible unexpired testimony, and a
  Rustev-owned transformer operating under its own authority.
- **Spec created this session:** yes, `010-policy-bounded-corpus-export`.

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
2. Spec 007 contains reviewable proposed constitutional text, but the standing
   constitution remains generic and unchanged until owner approval.
3. Codex instruction-delivery observation has a local draft contract in spec
   008. Live measurement and any adapter-wide qualification remain separate
   conditional acts.
4. Binding-level usage and cost summarization has a local draft contract in
   spec 009. Admission, cost findings, scheduling, and spending authority
   remain Statecraft-owned.
5. Policy-bounded corpus export has a local producer draft in spec 010.
   Rustev retains transformation, replay, comparison, and evaluation
   authority, while Statecraft retains evidence admission and run policy.
6. Statecraft supervision and admission, Statecraft setup repair, and remote
   desired state remain in their owning corpora.
7. Passing local validation does not change any lifecycle beyond drafted and
   locally validated for the files created here.
