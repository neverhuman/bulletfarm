# ADR 0016: Legacy contract semantic closure

Status: Accepted (DESIGNED; no wire bytes)
Owner: Bullet Farm maintainers
Related: 0003 (principal separation), 0005 (signed authority), 0009 (custody), 0011 (signed launch grant), 0014 (historical corpus disposition), 0017 (catalog type-expression vocabulary)
## Context and scope

Catalog `v1alpha1.0` has 40 recursively open leaves in 17 records. This proposal closes or replaces 12 retained coordinates, retires two obsolete `LaunchGrantV1` leaves, and retires legacy `GraphDeltaV1` after its closed successor. It is alpha-breaking. Until accepted and implemented, it defines no catalog bytes, authority, Evidence, dogfood run, transaction, live operation, evolution activation, receipt, or release state.
## Decision choices

| Coordinate | Rejected alternatives | Recommended decision |
| --- | --- | --- |
| Acceptance | Kernel three-field scaffold | Historical ten-field closed requirement |
| Evidence custody | caller independence enum; opaque attestation | fact-bearing custody with no independence verdict |
| Graph | overloaded `GraphDeltaV1`; unimplemented planning topology | runtime `GraphTransitionDeltaV1`; reserve future `PlanGraphDeltaV1` |
| Estimands | opaque digest; arbitrary registry | fixed V1 estimands plus analysis digest |
| Missingness | counts only | assignment-manifest-bound counts and equations |
| All-in cost | scalar; sparse map | mandatory dimensions and explicit unknown liability |
| Attack | opaque digest; executable inline payload | sanitized typed CAS proposal and bounded request |
| Behavior | 18 nullable columns; four-field scaffold | normalized event, typed subject/actions/resolution |
| Drift | opaque bundle | fixed observations; policy-derived non-authorizing action |
| Objectives | arbitrary metric map | fixed V1 raw observations and hard filters |
| Costs | sparse reported dimensions | mandatory dimensions; unreported is UNKNOWN |
| Sentinel | alias drift metrics | separate fixed task-threshold set |

The five recommended maintainer answers are: retain the numeric caps and millionth fixed point; retain four criticalities and escalation vocabulary; split runtime Graph transitions from future planning topology; fix V1 metric vocabularies; and defer exact custody/Evidence envelopes and signer policy to SD-B while making SD-B a hard publication/admission predecessor. ADR 0017 must be accepted before W11 or LC-A encodes this decision.
## Common closed-wire rules

- Named records and union branches are recursively closed with `additionalProperties:false`; embedded records forbid `schema_version`, while stored records require exact `v1alpha1`.
- Fields use the order shown; RFC 8785 orders canonical object keys. A `set` is unique and lexicographically sorted by its stated key; other arrays are semantically ordered.
- `Digest` is 64 lowercase BLAKE3 hex; `TypedId<p>` is `p_` plus that digest. New exact IDs are `RequirementId=req`, `ServiceIdentityId=svc`, `BehaviorEventId=bev`, `BehaviorRuleId=brl`, `DetectorId=det`, `FailureClassId=fcl`, `AggregateEvaluationId=aev`, `ExperimentProtocolId=epr`, `TeamRecipeId=rcp`, `AttackProposalId=atp`, `BehaviorTraceId=btr`, `TaskId=tsk`, `DriftSignalId=drs`, `SystemFingerprintId=sfp`, `EvaluationId=evl`, `SentinelResultId=snr`, `InvocationId=inv`, `AllocationId=alc`, `ProofBundleId=prb`, `IntegrationId=int`, `ObservationId=obs`, `UsageSettlementId=ust`, `ReviewReceiptId=rev`, `AuditBatchId=aub`, `OracleResourceId=orc`, and `CommandId=cmd`; existing `EvidenceId=evd` and `GateReceiptId=grc` remain.
- `InvariantId` is 8..64 ASCII bytes matching `^BF-[A-Z0-9]+(?:-[A-Z0-9]+)*$`. `SafeU64` is `0..9007199254740991`; `PositiveSafeU64` starts at 1; `SafeI64` is the symmetric safe interval; `U32` is `0..4294967295`; timestamps are Unix milliseconds in `SafeU64`.
- `Text<N>` is NFC UTF-8, 1..N bytes, refusing NUL, controls, bidi controls, and noncharacters; `Code<N>` is 1..N ASCII matching `^[A-Za-z0-9][A-Za-z0-9._:/+-]*$`; SemVer is 1..64 ASCII, parses as SemVer 2.0, and has no leading `v`.
- Nullable fields are required and exact-type-or-null. Numbers are integers; floats, NaN, Infinity, exponents, and numeric strings refuse. Arrays/documents are bounded; IDs/digests resolve.

Validation order is canonical bytes/duplicate keys, closed schema/types, ID/digest/local bounds, equations, subject resolution, then policy/lineage. Structural failures use `DOCUMENT_SCHEMA_INVALID`; semantic classes are `INVALID_ACCEPTANCE_REQUIREMENT`, `INVALID_EVIDENCE_CUSTODY`, `INVALID_GRAPH_TRANSITION`, `INVALID_AGGREGATE_EVALUATION`, `INVALID_ATTACK_PROPOSAL`, `INVALID_BEHAVIOR_TRACE`, `INVALID_DRIFT_SIGNAL`, `INVALID_EVALUATION_VECTOR`, and `INVALID_SENTINEL_RESULT`. No error is PASS.
## Shared records

```text
MeasuredScalarV1, tagged by state
  exact: state="exact"; value:SafeI64
  interval: state="interval"; lower_bound:SafeI64; upper_bound:SafeI64
  unknown: state="unknown"; reason:not_reported|not_observed|unsupported|
           redacted|infrastructure_failure|contradictory|zero_denominator|
           unbounded_liability
```
Intervals require `lower_bound <= upper_bound`; UNKNOWN has no numeric field, never ranks better, and cannot satisfy a hard constraint. Metric ranges further restrict it.
`MetricSourceV1` is tagged on `subject_kind`; its nine branches are `allocation:AllocationId`, `evidence:EvidenceId`, `proof_bundle:ProofBundleId`, `gate_receipt:GateReceiptId`, `integration:IntegrationId`, `observation:ObservationId`, `usage_settlement:UsageSettlementId`, `review:ReviewReceiptId`, and `audit_batch:AuditBatchId`. Each contains, in order, constant `subject_kind`, exact `subject_id`, and `subject_digest:Digest`. Sets have 1..16 entries ordered by `(subject_kind,subject_id)`; duplicates/conflicts refuse.
Allocation sources resolve exact `AllocationReceiptV1` bytes and prove assignment provenance only. For every metric, referenced policy fixes deterministic extraction, windowing, aggregation, rounding, and missingness over exact resolved source bytes; metric, unit, value, status, and source set must equal recomputation. A value not mechanically derivable requires a purpose-fixed evaluator receipt binding parent ID/digest, policy, metric, unit, value/status, window, and sorted sources. Missing/invalid derivation is UNKNOWN/ineligible; Evidence, ProofBundle, GateReceipt, release, and other-purpose signatures cannot substitute for metric evaluation.
```text
TimeWindowV1
  start_unix_ms:Timestamp; end_unix_ms:Timestamp
```
Require `start < end` and duration at most 2,678,400,000 ms (31 days).
## 1. Acceptance requirements

`AcceptanceContractV1.requirements` is a set of 1..256 entries ordered by
`requirement_id`:
```text
AcceptanceRequirementV1
  requirement_id:RequirementId; description:Text<4096>
  kind:functional_behavior|invariant|compatibility|performance|security|
       privacy|migration|operational|accessibility|user_interface|
       documentation|rollback|observation_survival
  criticality:critical|high|medium|low
  verification_method:RequirementVerificationV1
  required_evidence_tier:E0|E1|E2|E3|E4
  required_reviewer_independence:ReviewerIndependenceRequirementV1
  risk_escalation:RequirementRiskEscalationV1; source:RequirementSourceV1
  status:"active"
RequirementVerificationV1
  gate_ids:set<GateId,1..16>; satisfaction_rule:"all"
  observation_policy_digest:Digest|null
ReviewerIndependenceRequirementV1
  minimum_distinct_principals:integer[1..8]
  forbidden_shared_dimensions:set<principal|service_identity|session|workspace|
    artifact_owner|provider_family|model_family|holdout_custodian,0..8>
  conflict_policy_digest:Digest
RequirementRiskEscalationV1
  minimum_risk_class:R0|R1|R2|R3
  on_fail:block_activation|quarantine_candidate|open_intervention|require_replan
  on_unknown:block_activation|quarantine_candidate|open_intervention|require_replan
RequirementSourceV1
  source_descriptor_id:SourceDescriptorId; source_digest:Digest
```
Recompute `requirement_id` under `bullet.acceptance-requirement.v1` over the record without that field. Resolve every gate/policy/source. Every activated package and gate maps to a requirement and every requirement is mapped; mapping is admission logic. Conflict policy derives independence. History uses an immutable predecessor contract, never a status edit.
## 2. Evidence custody

```text
EvidenceCustodyV1
  producer_principal_id:PrincipalId
  producer_service_identity_id:ServiceIdentityId; producer_key_id:KeyId
  artifact_manifest_digest:Digest
  artifact_owner_service_identity_id:ServiceIdentityId
  reconstructor_service_identity_id:ServiceIdentityId
  author_principal_ids:set<PrincipalId,1..64>
  author_service_identity_ids:set<ServiceIdentityId,1..64>
  conflict_policy_digest:Digest
  custody_facts:set<CustodyFactRefV1,exactly 3>
  derivation_version:"evidence-independence-v1"; observed_at_unix_ms:Timestamp
CustodyFactRefV1
  fact_kind:producer_identity|artifact_ownership|reconstruction_identity
  media_type:identity-observation-v1|artifact-ownership-observation-v1
  fact_digest:Digest; attestor_principal_id:PrincipalId; attestor_key_id:KeyId
  observed_at_unix_ms:Timestamp
```
Facts contain each kind once in enum order. Producer/reconstructor facts use identity media; artifact ownership uses ownership media. Resolve classified-CAS bytes, exact digest, active identity-observer signature/role, service identity, and peer credential. Fact time is no later than custody time and all are within the parent Evidence window. Author principal/service sets equal the complete immutable Candidate/Attempt author lineage. Every forbidden dimension resolves from a typed, authenticated, Candidate- and Evidence-window-bound source; a missing, stale, contradictory, or unresolvable author, principal, service, session, workspace, artifact-owner, provider-family, model-family, or holdout-custodian fact yields UNKNOWN, never independence. The custody body has no independence, tier, outcome, or authority field; facts report facts only and callers/attestors never select the independence verdict. Conflict policy derives it from exact lineage, identities, ownership, environment, current facts, and forbidden dimensions.

```text
GateOutcome = PASS|FAIL|FLAKY|INFRA_ERROR|CANCELLED|TIMED_OUT|NOT_RUN|
  UNSUPPORTED|UNKNOWN|SUPERSEDED|INVALIDATED
EvidenceTier = E0|E1|E2|E3|E4
EvidenceV1
  schema_version:"v1alpha1"; evidence_id:EvidenceId; candidate_id:CandidateId
  subject_hash:Digest; outcome:GateOutcome; tier:EvidenceTier
  reason_code:Code<128>|null; verifier_principal_id:PrincipalId
  environment_hash:Digest; custody:EvidenceCustodyV1
  started_at_unix_ms:Timestamp; completed_at_unix_ms:Timestamp
```
Require `started_at <= completed_at`; custody and fact observations lie inside the inclusive window, and verifier principal equals custody producer. PASS alone requires null reason and alone may satisfy; every other outcome requires a non-null stable reason and never satisfies. Tier is policy-derived from resolved custody, never caller-selected. Recompute `evidence_id` under `bullet.evidence.v1` over canonical Evidence without that ID. There is no inline signature.

SD-B is a hard predecessor to W12 publication and every Evidence admission. Until it defines purpose-fixed signed Evidence and custody-fact envelopes, signer policy/lifecycle/read-back, authenticated derivation of outcome/tier/reason from exact GateSpec, Candidate, reconstruction, and gate results, and one-way Evidence -> ProofBundle -> GateReceipt binding, Evidence and every dependent ProofBundle, GateReceipt, MetricSource, acceptance, evaluation, route, promotion, integration, or release use is unavailable or UNKNOWN. Generic authority, launch, release, review, audit, metric-evaluator, and custody-fact signatures cannot substitute or re-sign an unauthenticated verdict.
## 3. Graph transitions

```text
GraphPackageStateV1
  work_package_id:WorkPackageId; state:WorkPackageState
GraphVariantFenceV1
  variant_id:VariantId; fence:SafeU64
GraphMaterializationBodyV1
  mission_id:MissionId; plan_revision_id:PlanRevisionId
  graph_sequence:PositiveSafeU64
  package_states:set<GraphPackageStateV1,1..4096>
  variant_fences:set<GraphVariantFenceV1,1..4096>
GraphGenesisV1
  schema_version:"v1alpha1"; graph_revision_id:GraphRevisionId
  materialization:GraphMaterializationBodyV1; materialization_hash:Digest
GraphTransitionCommandRefV1
  command_id:CommandId; request_digest:Digest; authority_decision_digest:Digest
  command:tagged union of exact admitted Kernel command-request record refs
GraphTransitionDeltaV1
  schema_version:"v1alpha1"; graph_revision_id:GraphRevisionId
  parent_graph_revision_id:GraphRevisionId; graph_sequence:PositiveSafeU64
  command_ref:GraphTransitionCommandRefV1
  operations:array<GraphTransitionOperationV1,1..256>
  materialization_hash:Digest
GraphTransitionOperationV1, tagged by op
  set_package_state: op:"set_package_state"; id:WorkPackageId
    from:WorkPackageState; to:WorkPackageState
  bump_fence: op:"bump_fence"; variant_id:VariantId
    from:SafeU64; to:PositiveSafeU64
```
`WorkPackageState` is `pending|ready|leased|executing|prepared|verifying| verified|reviewing|integration_ready|integrating|integrated|observing|survived| struggling|escalating|quarantined|cancelled|failed|reverted|rejected`.
Package/fence sets are unique and sorted by typed ID and cover the resolved immutable plan topology exactly. `materialization_hash = BLAKE3("bullet.graph-materialization.v1", JCS(materialization))`. Genesis has sequence 1 and `graph_revision_id = grf(BLAKE3("bullet.graph-genesis.v1", JCS(genesis without graph_revision_id)))`; persist it atomically with initial graph/projections. A transition requires the exact current parent and checked `graph_sequence=parent.graph_sequence+1`. Operations have at most one entry per `(op,subject_id)` and sort lexicographically by that unique key. State changes require exact current `from`, `from!=to`, and a legal Kernel edge; fence bumps require exact `from`, checked `to=from+1`. Apply operations to the parent, derive the complete successor materialization, and require its sequence and digest to equal the delta. Then `graph_revision_id = grf(BLAKE3("bullet.graph-transition.v1", JCS(delta without graph_revision_id)))`. Exact revision/command replay returns the stored successor; another stale parent conflicts, and coincidental current target state is never replay.

The delta is a Kernel-produced, non-authorizing post-decision commit fact, never a caller command or grant. Its command is a closed tagged union with one branch per admitted typed Kernel request; each request names expected parent/idempotency identity, and a separately authenticated decision binds its exact digest. Resolver policy fixes which branch may produce each edge/op set. One serialized Kernel transaction commits successor revision/materialization, normalized package state, ready queue, Variant high-water, Attempt/active-lease state, event, outbox, and CommandReceipt binding delta ID/post digest, or none; full materialization is recomputed for read-back before commit. `bump_fence` is legal only for a fence-owning lease/recovery branch, updates permanent high-water, and follows that branch's active-lease invalidation/replacement rules. Verification, integration, observation, revert, and terminal edges likewise require typed prerequisites. The delta alone never changes state or advances a fence. Future topology uses separately decided `PlanGraphDeltaV1`.
## 4. Aggregate estimands

```text
AggregateEstimandSetV1
  estimand_kind:architecture_effect|best_system_effect
  analysis_policy_digest:Digest; evaluation_set_digest:Digest
  entries:set<EstimandResultV1,1..16>
EstimandResultV1
  endpoint:observation_surviving_rate|all_in_cost_per_survivor|
    wall_time_per_survivor|human_intervention_rate|safety_violation_rate
  contrast_digest:Digest
  effect_measure:risk_difference_millionths|ratio_millionths|
    difference_micro_usd|difference_milliseconds
  estimate:MeasuredScalarV1[exact|unknown]
  confidence_interval:MeasuredScalarV1[interval|unknown]
  confidence_level_millionths:integer[500001..999999]
  cluster_count:integer[0..4096]; assignment_count:integer[0..4096]
  analysis_digest:Digest
```
Order/uniqueness is `(endpoint,contrast_digest)`; the evaluation-set digest is
`bullet.evaluation-set.v1` over sorted unique parent IDs; contrast resolves to preregistration.
Rates use risk difference/ratio, cost micro-USD difference/ratio, and time millisecond
difference/ratio. Risk difference is `[-1000000,1000000]`; ratio is nonnegative and 1,000,000 is
one. Exact estimates require containing intervals; unknown requires unknown. A ratio whose exact
preregistered denominator is zero requires estimate and interval UNKNOWN/`zero_denominator`; that
reason is forbidden for nonzero or merely unknown denominators and outside ratio estimands. No
zero, infinity, or finite sentinel substitutes. Counts and analysis obey section 5; estimand kinds
never mix.
## 5. Aggregate missingness

```text
AggregateMissingnessV1
  assignment_manifest_digest:Digest; missingness_policy_digest:Digest
  preassigned_count:integer[1..4096]; valid_assignment_count:integer[0..4096]
  excluded_invalid_task_count:integer[0..4096]
  evaluation_count:integer[0..4096]; missing_evaluation_count:integer[0..4096]
  outcome_counts:array<OutcomeCountV1,exactly 11>
  missing_reason_counts:array<MissingReasonCountV1,exactly 7>
  invalid_exclusion_manifest_digest:Digest|null
OutcomeCountV1 outcome:SURVIVED|FAILED|TIMED_OUT|INFRA_ERROR|ABSTAINED|
  ESCALATED|FALLBACK|CANCELLED|UNRESOLVED|INVALIDATED|UNKNOWN; count:[0..4096]
MissingReasonCountV1 reason:provider_unavailable|quota_exhausted|
  containment_unavailable|infrastructure_failure|cancelled_before_start|
  record_lost|unknown; count:[0..4096]
AssignmentManifestBodyV1
  allocation_ids:set<AllocationId,1..4096>
```
Arrays contain every literal once in displayed order. `assignment_manifest_digest` is the `bullet.assignment-manifest.v1`
digest of the closed body; each Allocation ID resolves and recomputes under `bullet.allocation-receipt.v1` over canonical
`AllocationReceiptV1` with its ID omitted. Let `P` be manifest allocations, `X` blinded invalid-exclusion allocations
(empty for null), `V=P∖X`, and `E` allocations with non-null section-6 Evaluation refs. Require
`|P|=preassigned`, `X⊆P`, `|X|=excluded`, `|V|=valid`, `E⊆V`, `|E|=evaluation`, and
`|V∖E|=missing`; count arrays sum to evaluation/missing, and non-null Evaluation IDs equal the
parent set/digest. Every estimand has `assignment_count=|V|`; its analysis commits exact sorted
`V`, both sibling manifest/policy digests, and applies the preregistered missingness rule to every
`V∖E`; only `X` is excluded. Require `cluster_count=0` iff assignment count is zero, otherwise
`1<=cluster_count<=assignment_count`; at zero, differences are UNKNOWN/`not_observed` and ratios
UNKNOWN/`zero_denominator`. Zero exclusions require null manifest; nonzero resolves one blinded,
preregistered decision per excluded allocation. Post-outcome deletion, survivor-only selection,
or fallback-as-challenger-success refuses.
## 6. Aggregate all-in cost

```text
AggregateAllInCostV1
  cost_policy_digest:Digest; evaluation_set_digest:Digest
  preassigned_count:integer[1..4096]; survivor_count:integer[0..4096]
  assignment_costs:set<AssignmentCostRefV1,1..4096>
  entries:array<AggregateCostEntryV1,exactly 20>
AssignmentCostRefV1
  allocation_id:AllocationId; evaluation_id:EvaluationId|null
  cost_vector_digest:Digest
AggregateCostEntryV1
  metric:CostMetricV1; unit:count|tokens|micro_usd|milliseconds|
    cpu_milliseconds|bytes
  known_total:SafeU64; bounded_unknown_count:SafeU64
  bounded_unknown_upper:SafeU64; unbounded_unknown_count:SafeU64
  conservative_total:DerivedNonNegativeValueV1
  per_survivor:DerivedNonNegativeValueV1
DerivedNonNegativeValueV1, tagged by state
  finite: state:"finite"; value:SafeU64
  infinite: state:"infinite"
  unknown: state:"unknown"; reason:"unbounded_liability"
```
Fixed metric/unit order is `input_tokens/tokens`, `output_tokens/tokens`, `cached_tokens/tokens`,
`reasoning_tokens/tokens`, `provider_cost_micro_usd/micro_usd`, `wall_time_ms/milliseconds`,
`queue_time_ms/milliseconds`, `tool_time_ms/milliseconds`, `runner_cpu_ms/cpu_milliseconds`,
`verifier_cpu_ms/cpu_milliseconds`, `verifier_wall_time_ms/milliseconds`,
`verifier_queue_time_ms/milliseconds`, `retry_count/count`, `redundant_work_count/count`,
`coordination_message_count/count`, `coordination_token_count/tokens`,
`coordination_time_ms/milliseconds`, `human_review_time_ms/milliseconds`,
`human_intervention_count/count`, `artifact_bytes/bytes`.
Assignment refs are unique/sorted by allocation ID, cover `P` exactly, and count equals
preassigned. Non-null Evaluation IDs are unique and equal the parent set, count `evaluation`; null
count is `missing+excluded`. Each digest resolves the section-11 canonical body under
`bullet.evaluation-cost-vector.v1`; non-null refs equal embedded Evaluation costs. Aggregate all
20 observations across every ref, including failed/cancelled/fallback/unresolved/excluded;
missing is UNKNOWN. Exact values sum to `known_total`; interval upper bounds sum to
`bounded_unknown_upper`; interval/unknown counts equal their fields, sum at most preassigned, and
the remainder is exact.
Cost intervals require `0 <= lower < upper`, making bounded count zero iff its upper sum is zero.
All arithmetic is checked. Without unbounded unknowns, total is finite `known+bounded-upper`;
otherwise UNKNOWN. Zero survivors makes per-survivor INFINITE; otherwise it is ceiling division
when finite and UNKNOWN for unbounded liability. Counts, evaluation digest, and survivors
cross-bind missingness/estimands; UNKNOWN cannot win efficiency.
## 7. Attack proposal body

```text
AttackProposalBodyV1
  attack_kind:regression_test|property_test|fuzz_seed|fault_injection|
    security_probe|reproduction|counterexample
  candidate_surface_manifest_digest:Digest; failure_class_id:FailureClassId|null
  expected_invariant_ids:set<InvariantId,1..32>
  input_artifact_digests:set<Digest,1..64>
  proposed_test_manifest_digest:Digest; sanitization_policy_digest:Digest
  requested_budget:AttackBudgetV1; oracle_access:"none"; hypothesis:Text<8192>
AttackBudgetV1
  max_invocations:[1..64]; max_wall_clock_ms:[1..3600000]
  max_cpu_ms:[1..3600000]; max_memory_bytes:[1..8589934592]
  max_output_bytes:[1..33554432]; max_artifact_bytes:[1..268435456]
```
Resolve all invariants/CAS subjects and bind the parent target. The budget is a
request bounded by separate reservation and grants nothing. Closed decode
forbids result, verdict, answer/oracle, credential, authority, effect, and
attestation fields. Null failure class means proposed novel class. Executable
bytes remain in classified CAS.
## 8. Behavior events

```text
BehaviorEventV1
  behavior_event_id:BehaviorEventId; rule_definition_id:BehaviorRuleId
  rule_code:Code<32>; rule_version:SemVer; subject:BehaviorSubjectV1
  observed_at_unix_ms:Timestamp; detector_id:DetectorId
  observed_action_digest:Digest; evidence_artifact_digests:set<Digest,1..32>
  severity:info|low|medium|high|critical
  enforcement_actions:ordered-unique array<EnforcementPrimitiveV1,1..8>
  postcondition:satisfied|failed|unknown|not_applicable
  resolution:BehaviorResolutionV1
```
`BehaviorSubjectV1` is a tagged union on `subject_kind` with exact branches
`repository:RepositoryId`, `mission:MissionId`, `work_package:WorkPackageId`,
`variant:VariantId`, `attempt:AttemptId`, `invocation:InvocationId`, and
`candidate:CandidateId`; each also carries `lineage_digest:Digest`.
`EnforcementPrimitiveV1` is `observe|warn|nudge|redact|block_action|
pause_attempt|checkpoint_attempt|request_scope|reclassify|auto_remediate|
quarantine_candidate|terminate_process|terminate_attempt|fail_attempt|
open_intervention|escalate|reject_completion`.
`BehaviorResolutionV1` is tagged: `unresolved{state}`, `not_required{state}`,
or `resolved{state,resolved_at_unix_ms:Timestamp,
resolution_artifact_digest:Digest}`. Resolved requires postcondition
satisfied/failed and `observed_at <= resolved_at <= trace.completed_at`;
not-required iff not-applicable; unresolved requires failed/unknown. Recompute
event ID under `bullet.behavior-event.v1` without its ID. Rule/detector resolve.
The parent adds after `environment_hash`: `started_at_unix_ms:Timestamp`,
`completed_at_unix_ms:Timestamp`, `detector_policy_digest:Digest`. It admits
0..4096 events ordered `(observed_at,id)`, all inside the inclusive window,
duration at most 31 days, and canonical trace size at most 8 MiB.
## 9. Drift evidence

```text
DriftEvidenceV1
  baseline_system_fingerprint_id:SystemFingerprintId
  baseline_window:TimeWindowV1; observation_window:TimeWindowV1
  drift_policy_digest:Digest
  metrics:array<DriftMetricObservationV1,exactly 12>
DriftMetricObservationV1
  metric:DriftMetricV1; unit:millionths|micro_usd|milliseconds|count|boolean
  baseline:MeasuredScalarV1; observed:MeasuredScalarV1
  bound_kind:upper|lower|exact_match; bound_value:SafeI64
  status:within_bound|breached|unknown; sources:set<MetricSourceV1,1..16>
```
Fixed order/unit: `escaped_defect_rate/millionths`, `revert_rate/millionths`,
`duplicate_effect_rate/millionths`, `ambiguous_effect_rate/millionths`,
`false_completion_rate/millionths`, `cost_per_survivor/micro_usd`,
`latency_p95_ms/milliseconds`, `verifier_backlog/count`,
`human_intervention_rate/millionths`, `provider_fingerprint_match/boolean`,
`profile_fingerprint_match/boolean`, `task_mixture_distance/millionths`.
Windows do not overlap; baseline ends no later than observation starts.
Millionths are 0..1,000,000, booleans 0/1, others nonnegative. The bound applies
to observed, not an implicit baseline delta; baseline is calibration provenance.
Exact/interval values wholly satisfy/refute the bound; overlap or UNKNOWN is
unknown. Sources resolve in-window. Derive parent `drift_kind` as the sole
non-within metric, otherwise `aggregate`. Action precedence is
`decertify > fallback_and_quarantine > pause_exploration > none`: fingerprint
mismatch/UNKNOWN decertifies; another breach falls back and quarantines; no
breach plus another UNKNOWN pauses; all within gives none. The field authorizes
nothing; separate signed policy transitions act.
## 10. Evaluation objectives

```text
EvaluationObjectiveVectorV1
  objective_policy_digest:Digest
  hard_constraints:set<HardConstraintObservationV1,1..128>
  entries:array<ObjectiveObservationV1,exactly 15>
HardConstraintObservationV1
  invariant_id:InvariantId; status:PASS|FAIL|UNKNOWN
  sources:set<MetricSourceV1,1..16>
ObjectiveObservationV1
  metric:ObjectiveMetricV1; unit:millionths|evidence_tier|boolean|count|bytes
  value:MeasuredScalarV1; sources:set<MetricSourceV1,1..16>
```
Fixed order/unit: `acceptance_clause_pass_rate/millionths`,
`gate_pass_rate/millionths`, `evidence_tier_floor/evidence_tier`,
`observation_survival/boolean`, `security_violation_count/count`,
`policy_violation_count/count`, `regression_strength/millionths`,
`mutation_strength/millionths`, `changed_path_count/count`,
`changed_byte_count/bytes`, `complexity_delta/millionths`,
`reviewability/millionths`, `validated_failure_class_count/count`,
`dissent_coverage/millionths`, `reliability/millionths`.
Millionths are 0..1,000,000 except signed complexity delta; tier is 0..4;
boolean is 0/1; counts/bytes are nonnegative. Policy owns direction, filtering,
Pareto order, and ties. Hard constraints are unique and lexicographically sorted by
`invariant_id`; every applicable constraint is present, and FAIL or UNKNOWN is ineligible.
Observation survival stays UNKNOWN until its window
closes and only exact 1 supports SURVIVED. Missing metrics are UNKNOWN; sources
bind the exact Candidate and closure.
## 11. Evaluation costs

```text
EvaluationCostVectorV1
  cost_policy_digest:Digest; entries:array<CostObservationV1,exactly 20>
CostObservationV1
  metric:CostMetricV1; unit:fixed unit; value:MeasuredScalarV1
  sources:set<MetricSourceV1,1..16>
```
Use the exact 20 metric/unit order from aggregate cost. Exact/interval values are nonnegative; cost intervals have strict lower<upper. Single-agent coordination is exact zero only when the recipe proves no coordination.
Every vector referenced by `assignment_costs` has exactly one allocation source total in every
entry and it matches the enclosing `AssignmentCostRefV1.allocation_id`; exact/interval values also
require one or more non-allocation fact sources, while UNKNOWN carries only that allocation source.
Allocation provenance alone proves no usage, cost, or outcome. Unreported usage is UNKNOWN, never zero.
Aggregation includes every assignment with checked arithmetic and preserves bounded/unbounded liability.
## 12. Sentinel metrics

```text
SentinelMetricSetV1
  sentinel_policy_digest:Digest
  entries:array<SentinelMetricObservationV1,exactly 8>
SentinelMetricObservationV1
  metric:acceptance_pass|gate_pass_rate|evidence_tier|observation_survival|
    wall_time_ms|provider_cost_micro_usd|policy_violation_count|
    escaped_defect_count
  unit:boolean|millionths|evidence_tier|milliseconds|micro_usd|count
  observed:MeasuredScalarV1; bound_kind:upper|lower|exact_match
  bound_value:SafeI64; status:within_bound|breached|unknown
  sources:set<MetricSourceV1,1..16>
```
Fixed order/units follow the displayed metric order: boolean, millionths,
evidence tier, boolean, milliseconds, micro-USD, count, count. Apply objective
ranges and drift bound rules. Sources bind parent task/fingerprint. Parent
outcome is PASS iff all within, FAIL if any breach, UNKNOWN if none breach and
any unknown, and INVALIDATED only for stale task/fingerprint/input closure.
Sentinel output alone never certifies or promotes.
## Exact parent normalization

In the same catalog bump, require:
```text
AcceptanceContractV1: contract_id:AcceptanceContractId;
  organization_id:OrganizationId; repository_id:RepositoryId;
  target:Text<1024>; risk_class:R0|R1|R2|R3; evidence_floor:E0|E1|E2|E3|E4;
  oracle_resources:set<OracleResourceId,0..64>;
  observation_window_seconds:integer[0..2678400]
AggregateEvaluationV1: aggregate_id:AggregateEvaluationId;
  protocol_id:ExperimentProtocolId; recipe_id:TeamRecipeId;
  evaluation_ids:set<EvaluationId,0..4096>
AttackProposal: attack_proposal_id:AttackProposalId; recipe_id:TeamRecipeId;
  proposer_class:breaker|adversary|reproducer|human_red_team
BehaviorTraceV1: trace_id:BehaviorTraceId; recipe_id:TeamRecipeId; task_id:TaskId
DriftSignal: drift_signal_id:DriftSignalId;
  system_fingerprint_id:SystemFingerprintId; drift_kind:DriftMetricV1|aggregate;
  required_action:none|pause_exploration|fallback_and_quarantine|decertify
EvaluationVectorV1: evaluation_id:EvaluationId; task_id:TaskId;
  recipe_id:TeamRecipeId; candidate_id:CandidateId;
  outcome:SURVIVED|FAILED|TIMED_OUT|INFRA_ERROR|ABSTAINED|ESCALATED|FALLBACK|
    CANCELLED|UNRESOLVED|INVALIDATED|UNKNOWN
SentinelResult: sentinel_result_id:SentinelResultId;
  system_fingerprint_id:SystemFingerprintId; task_id:TaskId;
  outcome:PASS|FAIL|UNKNOWN|INVALIDATED
```
All versioned parents use exact schema version; existing digest fields remain
exact. Evidence uses the complete ordered section-2 body. This supersedes handwritten
`outcome.rs::{Evidence,EvidenceOutcome}` and Kernel
verifier DTOs: their nine-value outcome and caller `verifier_is_independent`
are not canonical.
`EvaluationVectorV1.closure_hash` recomputes over Candidate, environment,
toolchain, policy, route, scope, provider, evaluator, custody, exposure, and
proof subjects; any change creates a new Evaluation. No free outcome/tier/class
string remains.
## Legacy launch retirement and errors

At the bump, delete `LaunchGrantV1` from catalog and `required_records()`, and
delete its generated Rust, TypeScript, and Schema outputs. Assert the name and
fields `sandbox_manifest`/`budget_reservation` are absent. Do not create target
records or deserialize it into claims. The only launch wire remains
`SignedLaunchGrantV1` plus `LaunchGrantClaimsV1` under ADR 0011.
Named-record dispatch of legacy `LaunchGrantV1` returns
`UNSUPPORTED_CONTRACT_RECORD` before body decode, nonce spending, or admission.
At the same bump delete `GraphDeltaV1` from catalog, `required_records()`, and every generated
projection. Old-name dispatch returns `UNSUPPORTED_CONTRACT_RECORD` before body decode; generation
asserts `GraphDeltaV1` and its open `operations` are absent and only `GraphTransitionDeltaV1`
exists. The two Graph DTOs can never coexist.
Then structural precedence is `INVALID_CONTRACT_RECORD_SHAPE`,
`INVALID_CONTRACT_FIELD_REFERENCE`, `INVALID_CONTRACT_FIELD_BOUNDS`,
`INVALID_CONTRACT_TAGGED_UNION`, `CONTRACT_TYPE_CYCLE`, version-gated
`OPEN_CONTRACT_FIELD`, and generated `DOCUMENT_SCHEMA_INVALID`; successful
structure then reaches the semantic codes above.
## Migration and consequences

1. ADR 0017 must first be accepted. W11 then adds generic closed refs, bounded arrays/sets,
   unions, embedded records,
   validated numeric/text/enum/ID vocabulary, cycles/bounds checks, and strict
   Rust/TypeScript generation while preserving `v1alpha1.0` bytes exactly.
2. LC-A1 encodes Acceptance, Evidence, Graph, Attack, Behavior, and Launch/GraphDelta
   retirement; LC-A2 encodes Aggregate, Drift, Evaluation, and Sentinel. Both
   remain dormant, split to keep every source/test below 500 LOC.
3. LC-B, LC-C, and dogfood constraints close the remaining 26 legacy leaves and
   W0-W10 records. SD-B must also land. Only then does W12 bump the catalog, publish these targets,
   remove legacy Launch and GraphDelta, and regenerate every projection from the one catalog.
4. Old open records receive no defaults/aliases. Prototype data migrates only
   by export, strict validation, and import; invalid rows stay quarantined.
5. Publication requires two byte-identical clean generations, compiled Rust,
   TypeScript no-emit, nested hostiles, and zero open `serde_json::Value` or
   `Record<string, unknown>` in emitted/public DTO fields and union branches. Generator-private
   JSON ASTs are outside that phrase but must never escape as DTO fields. Handwritten duplicate
   DTOs are deleted or private adapters, never a second maintained definition.
## Risks and evidence ceiling

- Fixed caps, millionth precision, four criticalities, and fixed metrics trade
  extensibility for strictness; changing them requires a catalog/schema bump.
- Custody references cannot establish independence until SD-B's classified fact formats,
  purpose-fixed signer roles/envelopes, derivation, lifecycle, replay, and read-back land; until
  then every Evidence-dependent use is unavailable or UNKNOWN.
- Runtime Graph transitions do not implement future planning topology.
- Descriptions/hypotheses may be sensitive despite bounds; classification,
  retention, projection, and holdout rules still apply.
- Derived drift, behavior, evaluation, and sentinel fields authorize no effect.
  Separate signed policy transitions remain mandatory.
- A documentation review can accept design semantics only. Until source,
  generated bytes, consumers, hostiles, and publication are complete, the
  maximum evidence class remains DESIGNED.
