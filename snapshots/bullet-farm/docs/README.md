# Bullet Farm documentation map

Status: **pre-release; release authority remains blocked**  
Last reviewed: 2026-09-11

This file is the hub documentation index. Do not add `docs/INDEX.md`.
A document can explain a decision, but it cannot make a command, receipt,
Candidate, Evidence result, effect, or release true.

## Current authority and status

| Question | Authoritative source |
| --- | --- |
| What can cross a trust boundary? | Generated schemas and Rust DTOs under `contracts/`, `policy/`, and `crates/bullet-wire/` |
| What is durably authorized or complete? | Kernel ledger state plus exact signed receipts; never a portal projection or prose claim |
| Which repository subjects form a family release? | A verified signed `family.lock` and its signed tags |
| Which gates are still blocked? | [`release.md`](release.md) |
| What is still missing as a product? | [`assurance/product-gaps.md`](assurance/product-gaps.md) (G1–G18 index; explicit profiled `check release` wins) |
| What is the current dependency-ordered finish plan? | [`assurance/closure-roadmap.md`](assurance/closure-roadmap.md) (Waves 0–11; first GA is `self-hosted-v1`) |
| Which historical spec §38 completion boxes exist in code, and where? | [`assurance/spec-crosswalk.md`](assurance/spec-crosswalk.md) (one row per box; tracked crosswalk, never evidence) |
| What should owners execute next, and when can self-dogfood begin? | [`assurance/execution-plan.md`](assurance/execution-plan.md) (D0–D6 operational queue; planning only) |
| What exact packets bridge coordination, offline transaction, live self-hosting, and the complete target? | [`assurance/full-product-dogfood-plan.md`](assurance/full-product-dogfood-plan.md) (implementation/proof bridge; planning only) |
| How will Nightshift's operator model be fused without importing its authority model? | [`assurance/nightshift-fusion-plan.md`](assurance/nightshift-fusion-plan.md) (proposed, source-pinned, dependency-ordered) |
| Which of the 26 `check release` gates is which product gap? | [`assurance/product-gaps.md`](assurance/product-gaps.md#historical-26-gate-catalog-and-release-profiles) |
| Which controls have executable enforcement? | [`assurance/invariant-registry.md`](assurance/invariant-registry.md) and the generated [`assurance/invariant-crosswalk.generated.md`](assurance/invariant-crosswalk.generated.md) |
| What is the rendered release decision? | [`assurance/release-truth.generated.md`](assurance/release-truth.generated.md) (generated projection, exit 3; `check release` wins) |
| Which architecture decisions are current? | [`decisions/`](decisions/) |
| What is the shortest normative architecture map? | [`architecture.md`](architecture.md) |
| Which repository owns a proposed change? | [`code-map.md`](code-map.md) |
| Where are trust, repository, credential, and evidence boundaries? | [`boundaries.md`](boundaries.md) |
| What does a stable Hub error mean and how is it repaired? | [`errors.md`](errors.md) |
| What runs in the current dependency order? | [`assurance/phase-1-dependency-map.md`](assurance/phase-1-dependency-map.md) and the repository test maps |
| Which competitor subjects are pinned? | [`assurance/competitor-snapshot.md`](assurance/competitor-snapshot.md) |
| How do agents coordinate? | [`runbooks/fleet.md`](runbooks/fleet.md) |
| Which operator procedures exist? | [`runbooks/README.md`](runbooks/README.md) |
| Which operator decisions are still open? | [`decisions/0013-operator-decision-register.md`](decisions/0013-operator-decision-register.md) |
| What does a term mean here? | [`glossary.md`](glossary.md) |
| What follows the first-GA profile? | `evolution-v1` (Wave 9), independent provider/forge/platform profiles and later `universal-v1` (Wave 10), then `team-v1`/`saga-v1` (Wave 11) in the [closure roadmap](assurance/closure-roadmap.md) |
| Which paper-driven opportunities remain? | [`workplan.md`](workplan.md) (non-authoritative; the active closure roadmap wins) |
| What is the canonical byte pipeline? | [`assurance/canonicalization.md`](assurance/canonicalization.md) |

The release index is deliberately fail-closed. `BLOCKED`, `UNKNOWN`, a missing
tool, a skipped test, a zero-test run, or a simulator receipt does not become
green through documentation.

## Operator surface

The commands an operator actually runs. This section exists because the index
carried none of them for its first three weeks while they were already the
primary way anyone touches this product.

| Command | What it does | Authority |
| --- | --- | --- |
| `bullet auth login --stdin < TOKEN` | Exchanges a one-time, ten-minute bootstrap token for a private durable session. Never takes a credential as an argument. | [runbooks/dogfood-admission-kit.md](runbooks/dogfood-admission-kit.md) |
| `bullet auth status` | Reports the operator and session identity and its expiry, or refuses `AUTH_REQUIRED`. | [boundaries.md](boundaries.md) |
| `bullet tui` | The operator console. Browses authenticated durable work; `Ctrl+C` detaches the client and leaves the daemon running. | [testing.md](testing.md) |
| `bullet coding submit` | Submits a durable `run_coding` command against loopback farmd. | [runbooks/dogfood.md](runbooks/dogfood.md) |
| `bullet coding board` / `watch` | Reads the atomic operator board: hold, fleet, sessions, outbox, harness. | [runbooks/dogfood.md](runbooks/dogfood.md) |
| `bullet coding list` / `status` / `task` / `retry` | Discovers this operator's durable commands, including after local journal loss. | [runbooks/dogfood.md](runbooks/dogfood.md) |
| `bullet coding harness-check` | Inspects the local harness environment. Daemon runtime admission is a separate, unknown question. | [assurance/product-gaps.md](assurance/product-gaps.md) |
| `bullet run show` | Reads a run receipt back, verifies its digest and chain, then renders it. | [architecture.md](architecture.md) |
| `bullet mission` | Browses authenticated missions. | [architecture.md](architecture.md) |
| `bullet provider live-conformance` | Policy-gated provider conformance, fail-closed at runtime observation. | [decisions/0012-policy-v1alpha2-live-admission.md](decisions/0012-policy-v1alpha2-live-admission.md) |
| `bullet authority keygen` / `mint-launch-grant` | Operator-held launch-grant authority and offline minting. | [decisions/0011-signed-launch-grant-and-egress-isolation.md](decisions/0011-signed-launch-grant-and-egress-isolation.md) |
| `bullet dogfood` | Internal dogfood compose. Not a release profile and not live-conformance. | [decisions/0015-dogfood-track.md](decisions/0015-dogfood-track.md) |
| `bullet transaction` | The five-plane transaction receipt. Currently ABSENT and ineligible. | [release.md](release.md) |
| `bullet-family doctor --json` | This checkout and its family, as JSON. Exits 3 when BLOCKED, by design. | [README.md](README.md) |
| `bullet-family check <fast\|required\|release\|scorecard\|dogfood>` | The gate surfaces. `check release` is BLOCKED on every profile. | [release.md](release.md) |
| `bullet-family coord …` | Coordinator verbs. Forbidden while the operating hold stands. | [runbooks/coordinator-recovery.md](runbooks/coordinator-recovery.md) |

Colour on every one of these follows a single rule: `NO_COLOR` when present and
non-empty disables it, `CLICOLOR_FORCE` paints a pipe, a `dumb` or absent `TERM`
is never painted, and `--no-color`, `--plain` and `--color=<auto|always|never>`
override the environment.

## Explanatory documents

- [`architecture.md`](architecture.md) is the canonical short architecture
  entrypoint and current proof boundary.
- [`code-map.md`](code-map.md) routes family, policy, provider, Git, API, UI,
  release, CI, and evidence changes to their owning repository and proof lane.
- [`boundaries.md`](boundaries.md) maps repository, authority, credential,
  evidence, effect, and projection crossings to their fail-closed behavior.
- [`errors.md`](errors.md) maps stable Hub error classes to bounded repair and
  escalation steps; `UNKNOWN` remains an outcome state, never a green result.
- [`paper/`](paper/) is the IEEEtran arXiv preprint source. A compile is not
  a release, installer, or benchmark receipt.
- [`workplan.md`](workplan.md) is an opportunity backlog for paper evidence,
  forge publication, and later-profile hardening. It cannot change profile
  scope or gate status; [`assurance/closure-roadmap.md`](assurance/closure-roadmap.md)
  is the active dependency order.
- [`brand/mascots/`](brand/mascots/) is sticker-first image-generator briefs.
  Generated art is not a receipt.
- [`architecture/overview.md`](architecture/overview.md) is the concise system
  orientation.
- [`architecture/evolutionary-control.md`](architecture/evolutionary-control.md)
  defines roles, Variants, evidence-bound fitness, selection, fusion, budgets,
  and the V1 adaptation boundary.
- [`testing.md`](testing.md) maps test profiles, evidence classes, ownership,
  negative cases, and live-lane admission.
- [`readme-media/`](readme-media/) contains reproducible, accessible,
  credential-free component observations. Its manifests are not Bullet Evidence
  or release receipts.
- [`../media/operator-console/`](../media/operator-console/) holds the real
  xbabe2 authenticated loopback-farmd TUI and Portal GIFs (not replayable, not
  an installer, HOLD remains, not VERIFIED). Stage-one VHS tapes stay above.
- [`runbooks/`](runbooks/) describes operator procedures; a runbook does not
  bypass an API or policy gate.
- [`runbooks/source-setup.md`](runbooks/source-setup.md) separates contributor
  bootstrap from the blocked signed release installer.
- [`runbooks/backup-restore.md`](runbooks/backup-restore.md) covers receipt-bound
  SQLite snapshots and the mandatory restore quarantine.
- [`runbooks/README.md`](runbooks/README.md) is the runbook index.
- [`runbooks/loopback-console.md`](runbooks/loopback-console.md) is the
  unsigned contributor console: `just console`, no-args `bullet` after login,
  and a command-worker loop when sibling subjects exist. HOLD remains. It is
  not `just setup` and not first-GA. `path-to-100.md` is a frozen 2026-08-25
  snapshot and is not this loop’s finish line.
- [`runbooks/fleet.md`](runbooks/fleet.md) is the agent coordination runbook:
  claims, heartbeats, handoffs, receipts, receipt corrections, and the
  orchestrator-only commit rule.
- [`runbooks/live-conformance.md`](runbooks/live-conformance.md) is the operator
  act that ratifies a generation-2 policy outside the repositories and runs the
  Kernel live-conformance lane. No `LIVE_PROOF` receipt exists yet.
- [`assurance/`](assurance/) maps claims to code, schemas, and tests.
  Start at [`assurance/product-gaps.md`](assurance/product-gaps.md) for the
  remaining-gap index.
- [`assurance/closure-roadmap.md`](assurance/closure-roadmap.md) is the current
  dependency-ordered implementation and proof map.
- [`assurance/nightshift-fusion-plan.md`](assurance/nightshift-fusion-plan.md)
  translates the source-pinned Nightshift UX and scheduling study into Bullet's
  existing authority boundaries, Waves 0–11, and executable proof gates. It is
  proposed planning, not runtime or release authority.
- [`assurance/v1-closure-plan.md`](assurance/v1-closure-plan.md) is a frozen
  point-in-time component inventory; its old universal-first ordering is not
  current release authority.
- [`assurance/product-gaps.md`](assurance/product-gaps.md) is the remaining-gap
  index (G1–G18, historical V1-S leftovers, C1–C12 product status). Documentation closed
  the visibility gap; implementation remains. Explicit profiled
  `bullet-family check release` with an admitted absolute registry wins if they
  disagree.
- [`assurance/release-truth.generated.md`](assurance/release-truth.generated.md)
  is the generated, drift-checked diagnostic rendering of explicit
  `--profile universal-v1 --receipts <absolute-registry> --report --portable`
  (`just release-truth`). `--report --portable` is an internal projection mode,
  not the unprofiled release interface; the page exits 3 and is never a receipt.
- [`assurance/invariant-crosswalk.generated.md`](assurance/invariant-crosswalk.generated.md)
  is generated from `policy/v1alpha1/invariant-registry.json` by
  `just contract-generate`; hand edits are drift.
- [`decisions/`](decisions/) records reviewed design choices and their status.
- [`decisions/0013-operator-decision-register.md`](decisions/0013-operator-decision-register.md)
  lists the decisions only an operator can make (ratification, key custody,
  forge topology); an agent cannot close one.
- [`glossary.md`](glossary.md) defines the terms these documents use.
- [`spec/README.md`](spec/README.md) indexes the historical Centerrail/Bullet
  Farm design provenance under [`spec/`](spec/). It is useful context and never
  runtime or release authority.

## Executable local evidence

Run commands from the public hub checkout:

```bash
just ci-doctor required   # fail before the lane if any pinned required tool is absent
just fast
just lint
just docs
just contract-check       # generated policy/schema/client byte drift
just model-check          # exactly two pinned TLC models
just contract
just check-family
just family-contract
just security
just audit
just release-truth        # regenerate docs/assurance/release-truth.generated.md (decision exit 3 preserved)
registry="$(mktemp -d)"
cargo run --locked --quiet --bin bullet-family -- check release \
  --profile self-hosted-v1 --receipts "$registry" --json
cargo run --locked --quiet --bin bullet-family -- check release \
  --profile universal-v1 --receipts "$registry" --json
cargo run --locked --quiet --bin bullet-family -- check release \
  --profile legacy-v1-26 --receipts "$registry" --report --portable
rmdir "$registry"
```

The meaning and limitations of those lanes are defined in
[`testing.md`](testing.md) and [`release.md`](release.md). `self-hosted-v1` is
the first-GA product profile; `universal-v1` is the later maximum-scope
composition. `legacy-v1-26 --report` separately renders only the historical
26-gate operator brief and keeps exit 3 while every gate is `BLOCKED`;
`just release-truth` writes the 43-selected-row `universal-v1` projection bound
to all 46 global crosswalk rows; the `docs` lane
refuses a stale copy. Live-provider, live-forge,
package, signing, and release evidence must use their separately admitted lanes
and exact subjects; the commands above do not substitute for them.

## Complete document inventory

Every tracked Markdown document under `docs/`, so that a document cannot exist
here without appearing in the index. `tests/assurance_controls.rs` asserts this
list stays complete; it had drifted to 54 unlisted documents out of 91 before
that control existed, including every decision record and every runbook. Being
listed is not an endorsement: a document can explain a decision, and it still
cannot make a command, receipt, Candidate, Evidence result, effect, or release
true.


### Top level

- [architecture.md](architecture.md) — Architecture entrypoint
- [boundaries.md](boundaries.md) — Trust and repository boundaries
- [code-map.md](code-map.md) — Bullet Farm code map
- [errors.md](errors.md) — Error and repair contract
- [glossary.md](glossary.md) — Glossary
- [phase-9-10.md](phase-9-10.md) — Historical Phase 9–10 sketch
- [release.md](release.md) — Bullet Farm release contract
- [testing.md](testing.md) — Test and evidence strategy
- [workplan.md](workplan.md) — Bullet Farm opportunity workplan


### Architecture

- [architecture/evolutionary-control.md](architecture/evolutionary-control.md) — Evolutionary multi-agent control
- [architecture/overview.md](architecture/overview.md) — Architecture


### Assurance, gap registers and campaign plans

- [assurance/canonicalization.md](assurance/canonicalization.md) — Canonical document pipeline
- [assurance/closure-roadmap.md](assurance/closure-roadmap.md) — Bullet Farm closure roadmap
- [assurance/competitor-snapshot.md](assurance/competitor-snapshot.md) — Competitor comparison snapshot
- [assurance/corpus-coverage.generated.md](assurance/corpus-coverage.generated.md) — Corpus coverage (generated)
- [assurance/deep-audit-20260909.md](assurance/deep-audit-20260909.md) — Bullet Farm: production and delivery audit, 9 September 2026
- [assurance/deep-audit-20260911.md](assurance/deep-audit-20260911.md) — Bullet Farm: full repair audit, 11 September 2026
- [assurance/dogfood-execution-plan.md](assurance/dogfood-execution-plan.md) — Dogfood execution plan (v0 read-only, then v1 writing)
- [assurance/execution-plan.md](assurance/execution-plan.md) — Bullet Farm finish execution plan
- [assurance/full-product-dogfood-plan.md](assurance/full-product-dogfood-plan.md) — Full-product dogfood bridge
- [assurance/health-checkpoint-20260909.md](assurance/health-checkpoint-20260909.md) — Health checkpoint after the production audit
- [assurance/health-observations-20260908.md](assurance/health-observations-20260908.md) — health observations 20260908
- [assurance/invariant-crosswalk.generated.md](assurance/invariant-crosswalk.generated.md) — Invariant crosswalk
- [assurance/invariant-registry.md](assurance/invariant-registry.md) — Invariant registry contract
- [assurance/launch-plan.md](assurance/launch-plan.md) — Historical launch-plan checkpoint
- [assurance/mvp-tui-delivery-plan.md](assurance/mvp-tui-delivery-plan.md) — Lean MVP and no-argument TUI delivery plan
- [assurance/nightshift-fusion-plan.md](assurance/nightshift-fusion-plan.md) — Nightshift fusion plan
- [assurance/orphan-inventory.generated.md](assurance/orphan-inventory.generated.md) — Typed assurance inventory (Wave 0)
- [assurance/path-to-100.md](assurance/path-to-100.md) — Path to 100 — closing every gap, fairly
- [assurance/phase-1-dependency-map.md](assurance/phase-1-dependency-map.md) — Gate 0 dependency map
- [assurance/prerequisite-observations-20260908.md](assurance/prerequisite-observations-20260908.md) — Local production prerequisite checkpoints
- [assurance/product-gaps.md](assurance/product-gaps.md) — Product gap register
- [assurance/production-prerequisite-checkpoints.md](assurance/production-prerequisite-checkpoints.md) — Local production prerequisite checkpoints
- [assurance/release-truth.generated.md](assurance/release-truth.generated.md) — Release truth
- [assurance/scorecard.generated.md](assurance/scorecard.generated.md) — Scorecard (generated)
- [assurance/spec-crosswalk.md](assurance/spec-crosswalk.md) — Spec §38 implementation crosswalk
- [assurance/v1-closure-plan.md](assurance/v1-closure-plan.md) — Historical Safety-Complete V1 checkpoint
- [assurance/xbabe2-development-closeout.md](assurance/xbabe2-development-closeout.md) — xbabe2 development closeout


### Brand — mascots

- [brand/mascots/01-fence-the-goat.md](brand/mascots/01-fence-the-goat.md) — 1. Fence the Goat
- [brand/mascots/02-the-combine.md](brand/mascots/02-the-combine.md) — 2. The Combine
- [brand/mascots/03-hashfire-the-moth.md](brand/mascots/03-hashfire-the-moth.md) — 3. Hashfire the Moth
- [brand/mascots/04-one-rail-tractor.md](brand/mascots/04-one-rail-tractor.md) — 4. The One-Rail Tractor
- [brand/mascots/05-barn-owl-attestor.md](brand/mascots/05-barn-owl-attestor.md) — 5. The Barn-Owl Attestor
- [brand/mascots/README.md](brand/mascots/README.md) — Bullet Farm mascot concepts


### Decision records

- [decisions/0001-provider-execution-mode.md](decisions/0001-provider-execution-mode.md) — 0001 — Provider execution mode: providers propose, BulletGit writes
- [decisions/0002-jeryu-forge-requirements.md](decisions/0002-jeryu-forge-requirements.md) — 0002 — Jeryu as the Bullet Farm effect target: requirements and do-not-disturb rules
- [decisions/0003-five-trust-planes.md](decisions/0003-five-trust-planes.md) — ADR 0003: Five trust planes and principal separation
- [decisions/0004-scope-amendment-tracks.md](decisions/0004-scope-amendment-tracks.md) — ADR 0004: Scope amendment tracks
- [decisions/0005-signed-authority-key-lifecycle.md](decisions/0005-signed-authority-key-lifecycle.md) — ADR 0005: Signed authority and key lifecycle
- [decisions/0006-trusted-time-restore-replay.md](decisions/0006-trusted-time-restore-replay.md) — ADR 0006: Trusted time, restore epoch, and replay
- [decisions/0007-sandbox-secret-taint.md](decisions/0007-sandbox-secret-taint.md) — ADR 0007: Sandbox, secrets, and tainted tool data
- [decisions/0008-forge-gates.md](decisions/0008-forge-gates.md) — ADR 0008: Local Jeryu and GitHub are separate gates
- [decisions/0009-data-retention-audit-anchor.md](decisions/0009-data-retention-audit-anchor.md) — ADR 0009: Data classification, retention, and audit anchoring
- [decisions/0010-supply-chain-policy.md](decisions/0010-supply-chain-policy.md) — ADR 0010: Supply-chain and release policy
- [decisions/0011-signed-launch-grant-and-egress-isolation.md](decisions/0011-signed-launch-grant-and-egress-isolation.md) — 0011 — Signed launch grants and provider egress isolation
- [decisions/0012-policy-v1alpha2-live-admission.md](decisions/0012-policy-v1alpha2-live-admission.md) — 0012 — Policy v1alpha2: operator-ratified live provider admission
- [decisions/0013-operator-decision-register.md](decisions/0013-operator-decision-register.md) — ADR 0013: Operator decision register
- [decisions/0014-corpus-dispositions.md](decisions/0014-corpus-dispositions.md) — ADR 0014 — Corpus dispositions: what "addressed" means for the historical vision
- [decisions/0015-dogfood-track.md](decisions/0015-dogfood-track.md) — 0015 — The dogfood track: `DOGFOOD_RUN` operational observations and `dogfood-local-v0`
- [decisions/0016-legacy-contract-semantic-closure.md](decisions/0016-legacy-contract-semantic-closure.md) — ADR 0016: Legacy contract semantic closure
- [decisions/0017-catalog-type-expression-proof-annex.md](decisions/0017-catalog-type-expression-proof-annex.md) — ADR 0017 normative proof and admission annex
- [decisions/0017-catalog-type-expression-vocabulary.md](decisions/0017-catalog-type-expression-vocabulary.md) — ADR 0017: Catalog type-expression vocabulary
- [decisions/0018-evidence-authenticity-publication.md](decisions/0018-evidence-authenticity-publication.md) — ADR 0018: Evidence authenticity and publication
- [decisions/0019-w11-proof-support-correction.md](decisions/0019-w11-proof-support-correction.md) — ADR 0019: W11 proof support correction
- [decisions/0020-w11-test-inventory-baseline-correction.md](decisions/0020-w11-test-inventory-baseline-correction.md) — ADR 0020: W11 test-inventory baseline correction


### Demo media

- [demo-gif/README.md](demo-gif/README.md) — Private native capture and rendering


### Exceptions

- [exceptions/README.md](exceptions/README.md) — Dated exceptions


### Paper

- [paper/README.md](paper/README.md) — Bullet Farm paper and executive brief


### README live media

- [readme-live-media/README.md](readme-live-media/README.md) — Historical and operator-supplied media


### README media

- [readme-media/README.md](readme-media/README.md) — Reproducible README media


### Runbooks

- [runbooks/README.md](runbooks/README.md) — Runbooks
- [runbooks/backup-restore.md](runbooks/backup-restore.md) — SQLite backup and quarantined restore
- [runbooks/coordinator-recovery.md](runbooks/coordinator-recovery.md) — Coordinator recovery production
- [runbooks/dogfood-admission-kit.md](runbooks/dogfood-admission-kit.md) — Dogfood admission kit (operator)
- [runbooks/dogfood.md](runbooks/dogfood.md) — Dogfood the family (operator board)
- [runbooks/effect-reconciliation.md](runbooks/effect-reconciliation.md) — Effect reconciliation — the offline half
- [runbooks/fleet.md](runbooks/fleet.md) — Fleet runbook
- [runbooks/live-conformance.md](runbooks/live-conformance.md) — Live provider conformance — admission, ratification, and the nightly lane
- [runbooks/platform-refusal.md](runbooks/platform-refusal.md) — Platform refusal — what each binary does off the supported runner
- [runbooks/publication.md](runbooks/publication.md) — Exact-source aggregate publication
- [runbooks/release-build.md](runbooks/release-build.md) — Release build containment boundary
- [runbooks/schema-removal.md](runbooks/schema-removal.md) — Schema removal — `UNSUPPORTED_SCHEMA` sites and what an operator can do
- [runbooks/setup-recovery.md](runbooks/setup-recovery.md) — Setup recovery drill
- [runbooks/signer-rotation.md](runbooks/signer-rotation.md) — Signer rotation — launch-grant key today, release-signing key not provisioned
- [runbooks/source-setup.md](runbooks/source-setup.md) — Source setup and installation boundary


### Specification sources

- [spec/CENTERRAIL_FINAL_ADAPTIVE_MULTI_FRONTIER_ENGINEERING_SPEC.md](spec/CENTERRAIL_FINAL_ADAPTIVE_MULTI_FRONTIER_ENGINEERING_SPEC.md) — Centerrail
- [spec/GASTOWN_OPEN_ISSUES_RISK_AUDIT_FOR_CENTERRAIL.md](spec/GASTOWN_OPEN_ISSUES_RISK_AUDIT_FOR_CENTERRAIL.md) — Gas Town Open-Issue Risk Audit
- [spec/POTENTIAL_DRAFT.md](spec/POTENTIAL_DRAFT.md) — BULLETFARM — The Definitive Multi-Agent Coding Engine
- [spec/README.md](spec/README.md) — Historical design corpus
- [spec/git_role.md](spec/git_role.md) — Executive conclusion
- [spec/nightshift.md](spec/nightshift.md) — nightshift
- [spec/paper.md](spec/paper.md) — Bullet Farm — IEEE white-paper record

## Maintenance rule

When an implementation changes, update its generated contract or executable
test first. Change a status document in the same reviewed transaction that
adds the receipt supporting the new status. Preserve historical sources
verbatim when practical and refresh `spec/HISTORICAL_ARTIFACTS.sha256` whenever
a tracked historical Markdown source intentionally changes.
