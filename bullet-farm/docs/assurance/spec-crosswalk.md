# Spec §38 implementation crosswalk

Status: **tracked crosswalk; not evidence, runtime, or release authority**  
Last reviewed: 2026-09-10
Owner: Bullet Farm maintainers

This page answers "which box of the historical spec's *definition of
implementation complete* exists in code today, where, which CI lane exercises
it, and who owns what is still missing?" It carries **one row per checkbox** of
[§38](../spec/CENTERRAIL_FINAL_ADAPTIVE_MULTI_FRONTIER_ENGINEERING_SPEC.md#38-definition-of-implementation-complete)
in the spec's own grouping (73 boxes; every box is still `- [ ]` in the spec).
The spec is provenance, not authority ([`../spec/README.md`](../spec/README.md));
a row here proves nothing. Release status stays with
[`../release.md`](../release.md) and the explicitly profiled `check release`;
the remaining-gap index is [`product-gaps.md`](product-gaps.md); the dependency
order is [`closure-roadmap.md`](closure-roadmap.md). A newer commit in any
member repository invalidates a row until its pointer is re-verified.

## How to read a row

| Column | Meaning |
| --- | --- |
| §38 item | The checkbox text, verbatim |
| Spec § | The section that defines the mechanism |
| Where it lives today | Repository-relative crate, file, table, route, or surface; `none` when nothing exists. Pointers were verified against the committed heads `bullet-kernel 7295e71`, `bullet-git 6c81f52`, `bullet-portal c55aa03`, `bullet-farm 6aa1d90` |
| Proof lane | The CI lane and test that exercises the pointer (`fast` = standalone nextest partition, `contract` = harness offline suites, `family` = connected lane with `BULLET_GITD_BIN`, `faults`/`egress` = named kernel filters); `none` when nothing runs |
| Status | `IMPLEMENTED` (component code with a passing mapped test; the same meaning as the corpus instrument, never release evidence) · `PARTIAL` · `LIBRARY-ONLY` (code exists but no binary links it: `crates/router`, `fusion`, `behavior`, `budgets`, `projections` have zero dependents) · `DESIGNED` · `ABSENT` · `REFUSED-BY-ADR` · `DEFERRED` |
| Disposition | `DO-NOW W<n>` (built in the current program, workstreams W1–W7) · `LANE` (engineering lane opened, lands after the DO items) · `OD-<x>` (operator decision in [ADR 0013](../decisions/0013-operator-decision-register.md)) · `DEFER` (tracked here, out of scope for the program) · `REFUSED (ADR id)` |

When Status is `IMPLEMENTED` the Disposition names only the remaining admission
owner (a G-row or an OD), because no program lane is owed. No §38 box is
`REFUSED-BY-ADR`: the ADR refusals apply to *mechanisms* (listed below), and no
box is `DEFERRED` by the spec itself (§45 defers "online learned router
authority", which lands on the shadow-calibration row as `DEFER`).

**Corpus instrument.** `policy/corpus-coverage-v1.json` carries 648 units =
33 `IMPLEMENTED` / 592 `PLANNED` / 20 `SUPERSEDED` / 3 `REFUSED`, rendered in
[`corpus-coverage.generated.md`](corpus-coverage.generated.md) as
625 active requirements (33 + 592). `IMPLEMENTED` there means exactly one
passing test symbol (`Anchor::Test`; `src/check/corpus/validate.rs`), never
release evidence, and the 33 anchors are 25 kernel, 6 git, 2 farm test symbols.
This page is coarser (one row per §38 box, not per normative unit) and never
promotes a corpus row; promotion requires a test anchor in the same change
([ADR 0014](../decisions/0014-corpus-dispositions.md)).

## Kernel (8 rows)

| §38 item | Spec § | Where it lives today | Proof lane | Status | Disposition |
| --- | --- | --- | --- | --- | --- |
| atomic Plan/Graph Delta | §6.3–6.4; §4.1 | `bullet-kernel/crates/domain/src/entities.rs` (PlanRevision, GraphDelta), `crates/adapters/src/sqlite/graph.rs`; the legacy wire record `GraphDeltaV1` is retired (ADR 0016) | bullet-kernel fast: `crates/adapters/tests/graph_delta_atomicity.rs`, `crates/application/tests/graph_delta_atomicity.rs` | PARTIAL | DEFER (DF-201 normalized columns; roadmap Wave 2) |
| permanent fences | §4.1; §26.3 | `bullet-kernel/db/migrations/0002_authority.sql`, `crates/domain/src/authority.rs`, `crates/adapters/src/sqlite/leases/acquire.rs`; hub model `formal/LeaseFence.tla` | bullet-kernel fast + faults: `crates/adapters/tests/chaos.rs` (`killed_writer_is_reclaimed_and_successor_gets_next_fence`); bullet-farm `just model-check` | IMPLEMENTED | DEFER (no lane owed; admission stays G2/G3) |
| one writer per Variant | §4.1–4.2 | `bullet-kernel/crates/domain/src/mutation_guard.rs`, `crates/adapters/src/sqlite/leases/acquire.rs`, `db/migrations/0017_mutation_authority.sql` | bullet-kernel family: `crates/runner/tests/kill_retry.rs` (`successor_refusals_never_reuse_a_fence_or_create_a_clone`); fast: `crates/adapters/tests/lease_command_atomicity.rs` | IMPLEMENTED | DEFER (no lane owed; admission G2/G3) |
| complete Authority Token checks | §6.8; §4.1 | `bullet-kernel/crates/runner/src/signed_lease.rs`, `crates/runner/src/candidate_authority.rs`, `apps/bullet-farmd/src/kernel_authority.rs`, `db/migrations/0020_authority_scope_admission.sql` | bullet-kernel fast: `crates/adapters/tests/authority.rs`, `authority_scope.rs`; family: `crates/runner/tests/heartbeat_stale.rs` | PARTIAL | DEFER (DF-203 custody across restart; same-UID fixture keys until OD-D/OD-E) |
| idempotent commands/outbox | §26.6; §27.9 | `bullet-kernel/db/migrations/0001_ledger.sql` (outbox), `0006_command_correlation.sql`, `0021_command_dispatch_claims.sql`; `apps/bullet-farmd/src/commands.rs` (`POST /api/v1/commands`, `GET /api/v1/outbox`) | bullet-kernel fast: `crates/adapters/tests/command_ingress_atomicity.rs`, `command_dispatch_claim.rs`, `apps/bullet-farmd/tests/api.rs` | PARTIAL | DO-NOW W4.1 (redacted outbox payload; no nonce in any `/api/v1` body); LANE T2/DF-602 (single-transaction admission, signed `CommandReceiptV1`) |
| ready queue | §26.5 | `bullet-kernel/db/migrations/0002_authority.sql` (`ready_queue`), `crates/application/src/queue.rs`; `GET /api/v1/ready` | bullet-kernel fast: `crates/adapters/tests/lease_command_atomicity.rs`, `materialization_atomicity.rs` | IMPLEMENTED | DEFER (single unpartitioned queue; Gastown R32 per-repository partitions) |
| unknown/contradictory observation | §4.7; §23.1 | `bullet-kernel/crates/application/src/effect_state.rs` (`OUTCOME_UNKNOWN`), `crates/domain/src/observation.rs` (`Contradictory`), `crates/application/src/effect_recovery.rs`, `db/migrations/0023_effect_recovery_claims.sql` | bullet-kernel fast: `crates/effects/tests/reconcile.rs`, `restart_reconcile.rs`, `crates/adapters/tests/effect_recovery_claim.rs` | IMPLEMENTED | DEFER (proved on `LocalBareForge` only; live forges wait on OD-B/OD-C) |
| tested backup/restore | §49.8; §44.6 | `bullet-kernel/apps/bullet/src/maintenance.rs` (backup, quarantined restore), `crates/adapters/src/sqlite/backup/`; [`../runbooks/backup-restore.md`](../runbooks/backup-restore.md) | bullet-kernel fast: `crates/adapters/src/sqlite/backup/tests.rs`, `prefix_tests.rs` | PARTIAL | DEFER (DF-705 two-install lifecycle and DR; needs OD-D/OD-E) |

## Cognitive plane (11 rows)

| §38 item | Spec § | Where it lives today | Proof lane | Status | Disposition |
| --- | --- | --- | --- | --- | --- |
| task taxonomy | §7 | `bullet-kernel/crates/domain` (`TaskClass`, `TaskClassification`), `crates/router/src/lib.rs` (`classify`); no binary depends on `bullet-router` | bullet-kernel fast: router unit tests | LIBRARY-ONLY | LANE (recorded, non-gating routing-provenance row in `run_coding` admission; plan §9.2) |
| D0/M1/M2/M3/M4 routing | §9.1; §10; §40.2 | `bullet-kernel/crates/router/src/lib.rs` (`dispatch`, `lane_for`: universal T0 simulator fallback, `transaction_gate_eligible` always false) | bullet-kernel fast: router unit tests | LIBRARY-ONLY | LANE (provenance only); tiers, eligibility filters, soft objective DEFER |
| model snapshots | §8.2 | `bullet-kernel/crates/harness-core/src/adapter.rs` (`ModelSnapshot`, `list_models`) implemented per adapter; no durable snapshot table (§26.1) | bullet-kernel contract: `binary_id(bullet-harness-*::offline)` | PARTIAL | DO-NOW W4.1 (provider/model/effort on attempts); durable snapshots DEFER (G15) |
| profile identity | §8.6 | `bullet-kernel/crates/harness-core/src/probe.rs` (`ProfileIdentity`, `RUNTIME_PROBE_UNAVAILABLE` default), `runtime_passport.rs`; signed enrollment component (M1.3) without a runtime consumer | bullet-kernel fast: `crates/harness-core/tests/launch_grant_admission.rs`; contract lane | PARTIAL | OD-A (live admission, per-provider enrollment); DO-NOW W7.4 (`dogfood stage-runtime` passport + enrollment) |
| quota/budget reservation and settlement | §15.5; §15.9 | `bullet-kernel/crates/budgets/` (in-memory, zero dependents), `db/migrations/0014_reservations.sql` (`budget_reservations`), `0022_lease_transport_settlements.sql`; no provider quota observation or cost rows | bullet-kernel fast: `crates/budgets/tests/conservation.rs`, `crates/adapters/tests/lease_transport_settlement.rs` | PARTIAL (budgets crate LIBRARY-ONLY) | DO-NOW W4.2 (link `crates/budgets`; per-attempt cost/usage; settle reservations; `quota_capacity` on the operator snapshot) |
| Dispatch Decision explanation | §9.5; §27.2 | `bullet-kernel/crates/router/src/lib.rs` (`DispatchDecision`), never persisted; no `/routing` route in the 21-route farmd catalog | bullet-kernel fast: router unit tests | LIBRARY-ONLY | LANE (persist the decision as a provenance row → Cognitive Router surface) |
| shadow calibration | §9.6–9.7 | `bullet-kernel/crates/router/src/lib.rs` (`shadow` returns a `quarantined` `ShadowRecord`); no shadow dispatch, holdouts, bandit, or `/routing/calibration` | bullet-kernel fast: `shadow_is_not_the_chosen_lane` | LIBRARY-ONLY | DEFER (plan §9.2; §45 defers online learned router authority) |
| struggle and escalation | §12; §40.5–40.6 | `bullet-kernel/crates/fusion/src/lib.rs` (`struggle_score`, `escalate`: pure, non-gating); no Progress Observation rows, critical triggers, or decomposition | bullet-kernel fast: fusion unit tests | LIBRARY-ONLY | LANE (persist Progress Observations from W4.3 events → Struggle Cockpit read-only); gating DEFER |
| triad fusion | §11.3–11.6 | `bullet-kernel/crates/fusion/src/lib.rs` (`fuse`, protocol `offline-digest-order-scaffold`); no councils, diversity score, or admission | bullet-kernel fast: fusion unit tests | LIBRARY-ONLY | LANE after W2.9 (two conforming providers; ADR 0015 "councils once ≥ 2 providers conform"); protocol catalog DEFER |
| code Selection Groups and synthesis Variant | §6.6–6.7; §11.7 | `bullet-kernel/crates/domain/src/entities.rs` (`selection_group_id`), `crates/domain/src/ids.rs`; no selections table, transaction, or code fusion | bullet-kernel fast: `crates/domain/tests/invariants.rs` | DESIGNED | DEFER (git_role #5; G15, roadmap Wave 9) |
| router/fusion benchmark | §33.3–33.4 | none; [`competitor-snapshot.md`](competitor-snapshot.md#fair-benchmark-contract) refuses any benchmark before a signed transaction | none | ABSENT | DEFER (L-71/WP-15 after `TRANSACTION_PROOF`) |

## Context (8 rows)

| §38 item | Spec § | Where it lives today | Proof lane | Status | Disposition |
| --- | --- | --- | --- | --- | --- |
| canonical Context Graph | §13.1–13.2 | `bullet-kernel/db/migrations/0011_context_capsules.sql` (immutable revision-one capsules only), `crates/application/src/context.rs`, `crates/runner/src/capsule.rs`; `GET /api/v1/context-lineage`; `bullet-portal/src/pages/ContextLineagePage.tsx` | bullet-kernel fast: `crates/application/tests/context_capsules.rs`, `crates/adapters/tests/context_capsules.rs`, `apps/bullet-farmd/tests/projections.rs` | PARTIAL | DEFER (G15 cognitive persistence; roadmap Wave 9) |
| capsule compiler | §13.4; §40.7 | none | none | ABSENT | DEFER (G15) |
| provider renderers | §13.5 | none | none | ABSENT | DEFER (G15) |
| verified compression | §14.1–14.4 | none (the §27.4 compression-jobs API is absent) | none | ABSENT | DEFER (G15) |
| mandatory coverage | §14.5 | none | none | ABSENT | DEFER (G15) |
| context thresholds | §14.7; §18.9 | none | none | ABSENT | DEFER (G15; context automation, plan §9.2) |
| every provider-pair migration | §14.8; §33.5 | none | none | ABSENT | DEFER (G15; presupposes ≥ 2 conforming providers, W2.9 first) |
| continuity acknowledgment | §13.6; §14.8 | none | none | ABSENT | DEFER (G15) |

## Session supervision (9 rows)

| §38 item | Spec § | Where it lives today | Proof lane | Status | Disposition |
| --- | --- | --- | --- | --- | --- |
| structured adapter events | §18.3; §41.5 | `bullet-kernel/crates/harness-core/src/event.rs` (`AgentEventKind`, `AgentEvent`); parsers in `crates/harness-{claude,codex,cursor,antigravity}`; consumed by `crates/runner/src/attempt/session.rs` then discarded (no `session_events` table) | bullet-kernel contract: `binary_id(bullet-harness-*::offline)`; fast: `crates/harness-claude/tests/dogfood.rs` | PARTIAL | DO-NOW W4.3 (`attempt_session_events` over the signed lease RPC; `GET /api/v1/attempts/{id}/events`) |
| PTY/ConPTY mirror | §18.4; §33.8 | none (`Capability::PtyRequired` flag only, `crates/harness-core/src/capability.rs`); root's uncommitted PTY-driven console tests drive the TUI, not a mirror | none | ABSENT | LANE (forensic, non-authoritative, after the structured plane; Gastown R37) |
| process ownership | §18.12; §28.2 | `bullet-kernel/crates/harness-core/src/spawnrun.rs` (process-group kill), `crates/runner/src/clock.rs` (self-kill deadline), `crates/harness-egress` teardown; no cgroup/Job Object ownership or termination receipt | bullet-kernel faults: `spawnrun::tests::provider_crash_is_nonzero_and_descendants_are_killed`, `timeout_kills_the_process_group_and_keeps_partial_output`; egress: `teardown_kills_holder_uplink_proxy_and_group_children` | PARTIAL | LANE W4.5 (T4/T4a truthful termination and durable stop ownership) |
| dialog controller | §18.6 | none; `crates/harness-claude/src/protocol.rs` bounds authority with `permissionMode: "plan"` and a tool allowlist instead of recognizing dialogs | none | ABSENT | DEFER (ADR 0001 read-only providers leave most dialog classes unreachable; plan §9.2) |
| automated local plan/permission/context handling | §18.7–18.9; §21 | none (no Tool Gateway, MCP registry, or tool receipts; `crates/mcp-mock` only) | none | ABSENT | DEFER (plan §9.2; ADR 0001 scope) |
| steering acknowledgment | §18.10 | none; `crates/harness-claude/src/protocol.rs` (`interrupt_request`) returns a typed unsupported-cancellation refusal | bullet-kernel contract: `crates/harness-claude/tests/offline.rs` | ABSENT | DEFER (steer/pause; plan §9.2) |
| interrupt/cancel/terminate | §18.10; §27.3 | process-level only: `crates/harness-core/src/spawnrun.rs`, `crates/runner/src/gate/process.rs` (`terminate`); no `cancel_attempt` command kind and no `/api/v1/attempts/{id}/*` mutation route | bullet-kernel faults: `spawnrun::tests::explicit_cancel_kills_the_process_group` | PARTIAL | LANE W4.5 (durable `cancel_attempt` with expected fence → `coding stop`, TUI `x`) |
| no indefinite blocked state | §18.11; §4.6 | heartbeat and self-kill (`crates/runner/src/signed_lease.rs`, `clock.rs`), lease reaper (`crates/adapters/src/sqlite/leases.rs`); no eight-value hang classification | bullet-kernel family: `crates/runner/tests/heartbeat_stale.rs`; faults: `spawnrun::tests::heartbeat_failure_kills_the_process_group`; fast: `crates/adapters/tests/lease_reaper.rs` | PARTIAL | LANE (hang classification over W4.3 events; plan §9.2) |
| no routine human terminal approvals | §18.8; §4.6 | achieved by refusal, not automation: providers run read-only under a closed argv (ADR 0001; `crates/harness-claude/src/protocol.rs`), so no terminal approval is reachable | bullet-kernel contract: harness offline suites | PARTIAL | DEFER (permission ladder; ADR 0001 keeps the write-dialog classes unreachable) |

## Workspaces and behavior (9 rows)

| §38 item | Spec § | Where it lives today | Proof lane | Status | Disposition |
| --- | --- | --- | --- | --- | --- |
| private clones | §20.1–20.2 | `bullet-git/crates/bullet-git-workspace/src/clone.rs`, `reflink.rs`, `mirror.rs` (`--dissociate`, reflink; POTENTIAL A2) | bullet-git contract: `crates/bullet-git-workspace/tests/clone_safety.rs`, `mirror_clone.rs`, `reflink.rs`, `gc_safety.rs` | IMPLEMENTED | DEFER (no lane owed; admission G4) |
| hostile Git policy | §20.3 | `bullet-git/crates/bullet-git-workspace/src/safe_git.rs` (pinned binary, digest, deadline, output caps), `scope.rs`, `gc.rs`; `bullet-kernel/crates/verifier/src/safe_git.rs` | bullet-git contract: `tests/clone_safety.rs`, `gc_safety.rs`; bullet-kernel fast: `crates/runner/src/gitd/workspace_binding/tests.rs` | PARTIAL | DEFER (G4; roadmap Wave 4 `openat2` private generation) |
| Change Intents | §20.5; §4.2 W4 | none | none | ABSENT | DEFER (git_role #8; plan §9.5) |
| successor-Attempt scope | §20.6 | [ADR 0004](../decisions/0004-scope-amendment-tracks.md) accepted; `bullet-kernel/db/migrations/0020_authority_scope_admission.sql`, `crates/application/src/authority_scope/`; no successor-Attempt protocol | bullet-kernel fast: `crates/adapters/tests/authority_scope.rs` | DESIGNED | DEFER (plan §9.2) |
| full Workspace Manifest | §20.2 step 10; §4.2 W7; §25.12 | `bullet-git/crates/bullet-git-workspace/src/clone.rs` (`WorkspaceManifest` recorded at clone time); no dirty/untracked/temp classification or cleanup-eligibility rows | bullet-git contract: `tests/clone_safety.rs` | PARTIAL | DO-NOW W4.6 (workspace hygiene projection from `candidate_preparation_*` + preservation receipts); classification DEFER |
| 84-rule default behavior catalog | §16; §17 | `bullet-kernel/crates/behavior/src/catalog.rs` (`SPEC_ROWS`, 84 planned rows), `detect.rs` (`DETECTOR_SCAFFOLD_IDS`, strict subset, non-authoritative); zero dependents | bullet-kernel fast: behavior unit tests (assert 84) | LIBRARY-ONLY | LANE (run detectors over W4.3 events as recorded observations → Behavior Center read-only); enforcement order DEFER |
| runtime file exclusion | §17.2–17.3; rule GT007 | none as a classifier; Candidate preparation lists untracked files without `--exclude-standard` today | none | ABSENT | DO-NOW W2.6 (per-gate `ls-files --others --exclude-standard` with `.gitignore` asserted inside the checkpoint); temp-work classifier DEFER |
| Candidate hygiene | §20.7 | `bullet-kernel/crates/runner/src/candidate_authority.rs`, `crates/application/src/candidate_preparation/`, `db/migrations/0019_candidate_preparation.sql`; `bullet-git/crates/bullet-gitd/src/daemon_handlers.rs` (`apply_proposal`) | bullet-kernel fast: `crates/adapters/tests/candidate_preparation.rs`; family: `crates/runner/tests/loop_sim.rs` | IMPLEMENTED | DO-NOW W4.6 (`files_changed`/per-path digests on `CandidateRow`); W2.1 validate-before-permit |
| preservation-before-cleanup | §20.8; Gastown R8 | `bullet-git/crates/bullet-git-workspace/src/preservation.rs`, `preservation_cleanup.rs`, `crates/bullet-gitd/src/daemon_handlers.rs` (`handle_cleanup`); `bullet-kernel/apps/bullet-runner/src/bin/bullet-command-worker/receipt/preservation.rs`, `crates/runner/src/attempt/drive.rs` | bullet-git contract: `package(bullet-gitd)`; bullet-kernel fast: `crates/runner/src/attempt/simulation_tests/` | PARTIAL | DO-NOW W2.1 (receipt-gated cleanup path, no online lease read-back) + W2.2 (typed post-preservation outcome) |

## Proof and effects (10 rows)

| §38 item | Spec § | Where it lives today | Proof lane | Status | Disposition |
| --- | --- | --- | --- | --- | --- |
| exact Candidates | §6.13; §20.7 | `bullet-kernel/crates/application/src/candidate_preparation/`, `crates/runner/src/gitd/candidate.rs`; `bullet-git/crates/bullet-git-types` (integration manifest, proof root) | bullet-kernel fast: `crates/adapters/tests/candidate_preparation.rs`; bullet-git fast: `crates/bullet-git-types/tests/integration_manifest.rs`, `proof_root.rs` | IMPLEMENTED | DO-NOW W2.7 (`bullet candidate publish` to a persistent bare mirror); admission G4 |
| E0–E4 Evidence | §22.1 | `bullet-kernel/crates/domain/src/entities.rs` (tier `E0..=E4`), `crates/domain/src/gates.rs`, `crates/verifier/src/evidence.rs` (clean-room E2), `crates/verifier/src/e3.rs` | bullet-kernel fast: `crates/verifier/tests/signed_chain.rs` | PARTIAL | LANE (honest increment: UID check, E1 downgrade); DEFER DF-501 independent custody |
| clean verifier | §22.2 | `bullet-kernel/crates/verifier/src/run.rs`, `workspace.rs`, `safe_git.rs`; `apps/bullet-verifier/src/main.rs` refuses (`VERIFICATION_INTENT_ADMISSION_UNAVAILABLE`); fixture executor behind `bullet-verifier/fixture-executor`, same UID, `FIXTURE_KEY_ONLY` | bullet-kernel fast (with `--features bullet-verifier/fixture-executor`) | PARTIAL | DEFER (DF-501/T6; stated open on every receipt); LANE minimum increment |
| blind review | §22.5; POTENTIAL A3 | none | none | ABSENT | DEFER (plan §9.6) |
| Evidence invalidation | §4.7 E5; §22.3–22.4 | typed `GateOutcome::Invalidated` only (`bullet-kernel/crates/domain/src/gates.rs`; projected by `apps/bullet-farmd/src/projections/quality_lab.rs`); no dependency edges, no automatic invalidation on rebase or head movement | bullet-kernel fast: `apps/bullet-farmd/tests/projections.rs` | PARTIAL | DEFER (git_role #7; Gastown R18; plan §9.2) |
| Proof Bundle | §6.14; §22.4 | `bullet-kernel/crates/verifier/src/signed_chain/`, `apps/bullet-runner/src/bin/bullet-command-worker/receipt.rs`, `apps/bullet/src/bin/transaction_offline/` | bullet-kernel fast: `crates/verifier/tests/signed_chain.rs`, `apps/bullet/tests/transaction_component_verify.rs`; family: `apps/bullet/tests/transaction_demo.rs` | IMPLEMENTED | DEFER (fixture-signed COMPONENT; DF-503 signed audit needs OD-E) |
| Effect Intent/Receipt | §6.15; §23.1 | `bullet-kernel/crates/effects/src/broker.rs` (`IntentInput`, `dispatch`), `db/migrations/0003_effects.sql`, `0009_effect_receipt_identity.sql`, `crates/application/src/effect_state.rs`; `apps/bullet-effects` | bullet-kernel fast: `crates/effects/tests/`, `crates/adapters/tests/effects.rs`, `apps/bullet-effects/tests/bin.rs` | IMPLEMENTED | DO-NOW W2.7 (dispatch to `LocalBareForge` at `refs/heads/bullet/candidate/<candidate_id>`) |
| GitHub protected delivery | §23.3 | `bullet-kernel/crates/effects/src/github.rs` (`GitHubForge`, specified, uncertified), `integration.rs` (`ProtectionState`, `CheckReceipt`) | bullet-kernel fast: `crates/effects/tests/integration.rs` (local forge only) | DESIGNED | OD-C (GitHub App test authority; G7) |
| merge-group verification | §23.3 step 11; §22.4 | none; Merge Rail projects three states (`bullet-kernel/apps/bullet-farmd/src/projections/merge_rail.rs`, `bullet-portal/src/pages/MergeRailPage.tsx`) | none | ABSENT | DEFER (forge program; plan §9.3 Merge Rail) |
| observation/survival | §23.1; §25.13 | `bullet-kernel/crates/effects/src/observation.rs` (`ObservationV1`, `SignedObservationV1`), `local_integration.rs` (MATCHED), `recovery.rs`; no observation window, Survived, or Reverted | bullet-kernel fast: `crates/effects/tests/observation.rs`; family: `apps/bullet/tests/transaction_demo.rs` | PARTIAL | DEFER (forge program; live read-back needs OD-B/OD-C) |

## Portal (7 rows)

| §38 item | Spec § | Where it lives today | Proof lane | Status | Disposition |
| --- | --- | --- | --- | --- | --- |
| all primary views | §25 | `bullet-portal/src/surfaces.ts` (15 declared, 6 `unknownReason`); `bullet-portal/src/pages/` (ControlTower, ShiftBrief, Fleet, SessionSupervisor, ContextLineage, MergeRail, QualityLab, IncidentsAudit, ProjectedSurface) | bullet-portal fast: vitest (214 pinned); e2e `e2e/*.spec.ts`; `ops/ci/real-farmd.sh` | PARTIAL | DO-NOW W4.2/W4.6 (two surfaces gain a ledger subject) + LANE (router/fusion/struggle/behavior read-only rows); G13 |
| sequence/lag/confidence everywhere | §48.2–48.3; §25 | `bullet-portal/src/components/StatusHeader.tsx` (`as_of_sequence`, event continuity, stale badge), `src/hooks/useEventStream.ts`, `src/sse.ts`; STALE-on-mount, no atomic watermark | bullet-portal fast: `src/operatorSnapshot.test.ts`, `src/sse.test.ts` | PARTIAL | DO-NOW W5.7 (`unproven`/`stale`/`current`); DF-603 atomic snapshot |
| honest pending commands | §25.16; §48.4 | `bullet-portal/src/pendingCommand.ts`, `src/components/CommandCard.tsx`, `OutboxCard.tsx`; `bullet-kernel/apps/bullet-farmd/src/commands.rs` (`run_demo`, `run_coding` only; receipt `UNSIGNED_FIXTURE`) | bullet-portal fast: `src/pendingCommand.test.ts`, `src/api.command.test.ts`; bullet-kernel fast: `apps/bullet-farmd/tests/api.rs` | PARTIAL | DO-NOW W5.5/W5.6 (provider "requested, not observed"; stop/steer greyed with refusal text) + W3.4 (TUI submit) |
| live session/terminal | §18.13; §48.6 | none (no WebSocket route; `bullet-portal/src/pages/SessionSupervisorPage.tsx` reads `GET /api/v1/sessions` only) | none | ABSENT | DEFER (PTY forensic-only per Gastown R37; plan §9.3) |
| routing/fusion/quota/context/behavior visibility | §25.3, §25.4, §25.9, §25.8, §25.11 | `bullet-portal/src/surfaces.ts` honest `unknownReason` cards (cognitive-router, fusion-lab, quota-capacity, struggle-cockpit, behavior-center, workspace-hygiene); `ContextLineagePage.tsx` shows capsules only | bullet-portal fast: surfaces tests | DESIGNED | DO-NOW W4.2 (quota) + W4.6 (hygiene); LANE router/fusion/struggle/behavior rows; context DEFER |
| incident reconstruction | §25.15 | `bullet-portal/src/pages/IncidentsAuditPage.tsx` over `GET /api/v1/audit` and `GET /api/v1/events`; no correlation-grouped timeline | bullet-portal fast: `IncidentsAuditPage.test.tsx` | PARTIAL | DO-NOW W5.2 (`RunTimeline` by `correlation_id`) + W4.3 (`stream_id`/`correlation_id` on `EventEnvelope`) |
| accessibility and large-scale tests | §48.11–48.12; §33.12 | `bullet-portal/src/components/Nav.tsx` (`aria-current`), `CommandPalette.tsx` (`aria-*`), `e2e/control-tower.spec.ts`; no reduced-motion mode, focus preservation, or large-graph tests | bullet-portal fast + e2e | PARTIAL | DO-NOW W5.1/W5.8 (tokens, skip link, landmarks, `aria-live`, hosted real-farmd job); large-scale DEFER |

## Quality and operations (11 rows)

| §38 item | Spec § | Where it lives today | Proof lane | Status | Disposition |
| --- | --- | --- | --- | --- | --- |
| state-machine model | §33.1 | hub `formal/LeaseFence.tla`, `formal/EffectCheck.tla`, `formal/model-lock.json` (exactly two pinned models by decision C7); `bullet-kernel/crates/adapters/tests/formal_traces.rs` replays exported TLC traces | bullet-farm `just model-check` (`formal/model-check.sh`); bullet-kernel fast: `formal_traces.rs` | PARTIAL | DEFER (DF-105 traces bound into receipts; the two-model scope is a decision, not a gap) |
| property tests | §33.1 | `bullet-kernel/crates/application/tests/properties.rs`, `crates/domain/tests/invariants.rs`, `crates/budgets/tests/conservation.rs`, `dimensions.rs` (proptest) | bullet-kernel fast | IMPLEMENTED | DEFER (no lane owed; coverage grows with each lane) |
| provider simulator | §33.2 | `bullet-kernel/crates/harness-sim/src/lib.rs` (`SimAdapter`), `crates/test-simulation`; `apps/bullet-runner/src/main.rs` (`--provider sim` default; real providers admitted at the CLI since kernel `7295e71`) | bullet-kernel contract: `package(bullet-test-simulation)`; family: `crates/runner/tests/loop_sim.rs` | IMPLEMENTED | LANE D1 (fake-provider dry run through the real path after W2.8; `--replay` fixtures) |
| chaos suite | §33.10 | `bullet-kernel/ops/ci/faults.sh` (13 named component fault tests), `crates/adapters/tests/chaos.rs`, `cross_process.rs`, `restart_process/`; `ops/ci/proof-transaction-offline-chaos.sh` | bullet-kernel faults lane; nightly | PARTIAL | DEFER (DF-702 twelve-boundary fault campaign; release program) |
| adversarial security suite | §33.11; §43 | `bullet-kernel/crates/harness-egress/tests/`, `crates/effects/tests/observation.rs` hostiles, `crates/runner/src/gitd/workspace_binding/tests.rs`; hub `tests/coord_phase_r.rs` (canonical hostiles), `ops/ci/security.sh`, secret canaries | bullet-kernel egress + fast; bullet-farm security + fast | PARTIAL | DEFER (G8 floor ≥ 90 with zero caps; DF-706 fuzz/mutation/sanitizer tiers) |
| cross-platform suite | §33.9; §19.5–19.6 | `bullet-kernel/ops/ci/portable-refusal.sh`, `bullet-git/ops/ci/platform-refusal.sh`, `bullet-farm/ops/ci/platform-refusal.sh`, `bullet-portal/ops/ci/platform-refusal.mjs`: compile every target, execute the typed refusal; Linux is the only real runner | each repository's platform-refusal lane | PARTIAL | OD-F (platform decision); DEFER macOS/Windows runner and sandbox classes (plan §9.4) |
| SLO dashboards | §32.8; §32.1 | none (no OpenTelemetry dependency; no KPI computation) | none | ABSENT | DEFER (plan §9.3 OTel/KPIs) |
| canary and rollback | §43.7; §49.3; §34 Phase 7 | evolution: `evolutionary_authority=false` by design, roadmap Wave 9 shadow → canary → rollback receipts; upgrade: `bullet-kernel/apps/bullet/src/maintenance.rs` quarantined restore, supervised schema upgrade (closeout R2) PARTIAL | bullet-kernel fast: `crates/adapters/src/sqlite/migrations/tests/`; bullet-farm fast: `tests/release_truth.rs` | DESIGNED | OD-H (one expiring ≤ 1 % R0/R1 canary; G11); LANE R2 (supervised upgrade; root/atomic_portal own migrations) |
| disaster recovery test | §44.6; §49.8 | [`../runbooks/backup-restore.md`](../runbooks/backup-restore.md), [`../runbooks/coordinator-recovery.md`](../runbooks/coordinator-recovery.md), hub `scripts/recovery-rehearsal.sh` (synthetic COMPONENT rerun, DF-R7a) | bullet-farm fast: `tests/coord_phase_r.rs` | PARTIAL | DEFER (DF-705 lifecycle + DR; DF-R7b waits on an operator checkpoint) |
| independent security review | §43; §34 Phase 8 | none; Jankurai audits (`ops/ci/audit.sh` in each repository) are internal diagnostics scoring 64–68 | none | ABSENT | DEFER (X-2 external review; separate audit program, plan §9.6) |
| benchmark against approved baselines | §9.2 quality floor; §33.3–33.4 | none; [`competitor-snapshot.md`](competitor-snapshot.md#fair-benchmark-contract) refuses any benchmark before a signed transaction | none | ABSENT | DEFER (L-71/WP-15 after `TRANSACTION_PROOF`) |

## Totals

Counts are over the 73 rows above; a row's primary disposition is the first
token of its Disposition cell.

| Status | Rows | | Primary disposition | Rows |
| --- | --- | --- | --- | --- |
| IMPLEMENTED (component) | @IMPL@ | | DO-NOW (W1–W7) | @DO@ |
| PARTIAL | @PART@ | | LANE | @LANE@ |
| LIBRARY-ONLY | @LIB@ | | OD-x | @OD@ |
| DESIGNED | @DES@ | | DEFER | @DEFER@ |
| ABSENT | @ABS@ | | REFUSED | @REF@ |

Reading: the kernel, proof, and workspace groups are component-real; the
cognitive plane is library code no binary links; the context group and most of
session supervision are absent and deferred to G15 and the forge/session
programs; the portal is honest about its six `UNKNOWN` surfaces. Today's code
matches the spec's §45 V1 boundary far better than its §38 completion list.

## REFUSED and SUPERSEDED mechanisms (do not resurrect)

These are the items the program must not rebuild, verbatim from the approved
plan §9.7, with the deciding record. They are mechanisms, not §38 boxes; the
corpus instrument counts 3 `REFUSED` and 20 `SUPERSEDED` units at a finer
granularity than this list.

| Item (plan §9.7) | Class | Deciding record |
| --- | --- | --- |
| `DOGFOOD_RUN` as a sixth evidence class or receipt kind | REFUSED | [ADR 0015](../decisions/0015-dogfood-track.md) |
| provider write/mutation authority or worktree escape | REFUSED | [ADR 0001](../decisions/0001-provider-execution-mode.md); ADR 0015 "what does not change" |
| `dogfood-local-v0` as a `ReleaseProfile` (typed `NOT_A_RELEASE_PROFILE`) | REFUSED (landed) | ADR 0015 |
| provider-runner key as dogfood evidence signer | REFUSED | ADR 0015 landed surfaces; gates M1.2 |
| env token / Cargo feature / mutation claims as launch authority | REFUSED | [ADR 0011](../decisions/0011-signed-launch-grant-and-egress-isolation.md) |
| public farmd lease routes | REFUSED | ADR 0011 rejected alternatives |
| v1alpha1 live admission and the immutable conservatism set | REFUSED | [ADR 0012](../decisions/0012-policy-v1alpha2-live-admission.md) |
| `LaunchGrantV1` / `GraphDeltaV1` | SUPERSEDED | [ADR 0016](../decisions/0016-legacy-contract-semantic-closure.md) |
| a second type-expression registry | REFUSED | [ADR 0017](../decisions/0017-catalog-type-expression-vocabulary.md) |
| generic/raw signer or fixture-key evidence | REFUSED | [ADR 0018](../decisions/0018-evidence-authenticity-publication.md) |
| vendored Jeryu / jeryu-split recreation / fifth source member | REFUSED | family plans (`NEXT_EVOLUTION_PLAN.md` §3, §7; `OWNER-CLOSE-PLAN.md` §1); no ADR id |
| GitHub as sovereign source truth | SUPERSEDED | `NEXT_EVOLUTION_PLAN.md` §3; [`../workplan.md`](../workplan.md) |
| Fresh Genesis, ledger relocation or chmod as recovery | REFUSED | ADR 0015 (coordinator); [`../runbooks/coordinator-recovery.md`](../runbooks/coordinator-recovery.md) |
| the OD-K historical template | SUPERSEDED | [ADR 0013](../decisions/0013-operator-decision-register.md) OD-K |
| CLOSE-100 26-gate catalog and G1–G15 vocabulary | SUPERSEDED | `CLOSE-100-BOARD.md` / `CLOSE-100-RECEIPTS.md` banners; live authority is `check release` (43 gates) and [`product-gaps.md`](product-gaps.md) G1–G18 |
| the Sept-4 Wave-2 ceremony | SUPERSEDED | `CEREMONY-WAVE2.md` header; `OD-D-OPERATOR.md` banner |
| Nightshift/veox-auto as runtime authority | REFUSED | `DOGFOOD-MULTI-CLI-GATES.md` §20; [`nightshift-fusion-plan.md`](nightshift-fusion-plan.md) |
| `codex exec --json` / Cursor print modes as protocol equivalence | REFUSED | ADR 0001 adapter order; gates §7 |
| TEAM.md C8 any-two-providers GA / `TeamRecipeV1` execution | SUPERSEDED | ADR 0001; [`product-gaps.md`](product-gaps.md#centerrail-c1c12-product-status) C8 |

## Contradictions resolved by disposition

Recorded from the approved plan §9.8. Each is resolved by the disposition
named here; an ADR follows only if root and the operator agree.

- DE-SIM's "kernel `MAX_OPERATIONS = 1024`" is stale (128 since `52cbcf1`); the
  live mismatch is bullet-wire 1024 and the missing kernel aggregate bound →
  DO-NOW W2.3 (128 ops + 32 MiB family-wide, one equality test per repo).
- `check dogfood --track coord|dogfood|all` (source) vs
  `coordination|provider-local-v0` (gates doc M0.2) → source wins; note in the
  gates doc.
- OD-L / OD-M / OD-N appear only in family plans, not in ADR 0013 → cited as
  plan labels only, never as live operator decisions.
- Spec §27 `/v1/...` vs the mounted `/api/v1/...` (`/v1` answers 410
  `API_VERSION_RETIRED`) → propose an ADR line renumbering §27; every route
  cited above uses `/api/v1`.
- Spec §25.1 Control Tower vs the Nightshift Shift Brief as landing page →
  Shift Brief stays `DEFAULT_ROUTE` (`bullet-portal/src/surfaces.ts`); noted
  here, spec left as provenance.
- Portal test count 183 vs 195 vs 214 → pinned by commit in the ratchet commit
  message (the `fast` lane pins 214 at `c55aa03`).
- A real Claude turn executed while OD-K is OPEN → it is a `DOGFOOD_READ_ONLY`
  observation outside admission, which ADR 0015 permits; it is not the M1.12
  checkpoint.
- Three-provider pilot (ADR 0015 / exec plan) vs four-provider (gates M2 /
  ADR 0001) → every recording states which milestone it claims.

Found while building this crosswalk (not in plan §9.8; recorded, not decided):

- Spec §45 lists "Context Graph/Capsules/compression/migration" and
  "session supervisor and PTY/ConPTY view" under *Included in V1*, while plan
  §9.2 defers the context group citing §45. The disposition (`DEFER`) stands
  because the owning register row is G15 / roadmap Wave 9; the citation should
  read G15, not §45.
- The family-root survey line "bullet-runner admits only `--provider sim`" is
  stale: kernel `7295e71` admits `claude|codex|cursor|agy|antigravity` at the
  CLI (DE-SIM L2.2 landed). Admission at the CLI is not live admission (OD-A).

## Maintenance rule

Re-verify a pointer before citing it; a moved file or renamed test makes the
row false, not merely stale. Change Status only in the same reviewed change
that lands the test anchor supporting it, and never above the corpus meaning
of `IMPLEMENTED`. Dispositions change only when the plan, an ADR, or ADR 0013
changes; this page never closes a G-row, an OD, or a release gate.
