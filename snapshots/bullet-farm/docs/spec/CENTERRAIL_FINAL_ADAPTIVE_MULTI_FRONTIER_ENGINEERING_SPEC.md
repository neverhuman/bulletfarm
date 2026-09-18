# Centerrail

## Final Adaptive Multi-Frontier Engineering Specification

**Product:** Centerrail  
**Descriptor:** Verified multi-frontier software delivery  
**Tagline:** Many minds. One verified line to main.  
**Specification status:** Implementation-complete architecture and build contract  
**First-party languages:** Rust and TypeScript  
**Portal:** Vite + React + TypeScript  
**Reference runner:** Linux  
**Supported operator hosts:** Linux, macOS, Windows  
**Initial harness families:** OpenAI Codex, Claude Code, Cursor Agent, Google Antigravity  
**Primary objective:** Maximize verified, surviving engineering value per human minute and per dollar while minimizing regressions, authority ambiguity, context loss, agent slop, quota waste, and operational risk.

---

# 0. Status, scope, and meaning of “final”

This document supersedes the prior Centerrail synthesis as the complete product and engineering specification for:

- transactional software-change orchestration;
- model and provider routing;
- economy-model defaulting;
- multi-model fusion and councils;
- quota and budget governance;
- task struggle detection and escalation;
- provider-portable context and verified compression;
- real-time agent supervision;
- PTY/TTY compatibility observation and control;
- deterministic bad-behavior prevention;
- private-clone Git and workspace control;
- sandboxed execution;
- independent verification;
- protected external effects and integration;
- the engineering operations portal;
- implementation sequencing, APIs, schemas, tests, and exit gates.

“Final” means that the design intentionally closes the known architectural decisions needed to start and finish an implementation. It does **not** mean that an unbuilt system is already proven, that no provider will change, or that no future product decision will occur. Correctness remains contingent on implementation, formal modeling, fault injection, benchmark evidence, security review, canary deployment, and production observation.

The system must never claim “zero regressions,” “100% autonomy,” or “exactly once physical execution” where the underlying external systems cannot support those claims. It must represent uncertainty explicitly and fail closed at authority boundaries.

---

# 1. Final product decision

Build Centerrail as a **transaction processor for software changes, model cognition, and consequential external effects**.

The previous architecture already established the correct software-change transaction:

```text
Mission
→ immutable Plan Revision
→ Work Packages and Variants
→ fenced Attempts
→ exact Candidates
→ Evidence
→ independent Review
→ protected Integration
→ Observation Window
→ surviving Outcome
```

This revision adds a first-class **Cognitive Execution Plane**:

```text
Cognitive Task
→ task classification
→ eligible provider/model lanes
→ quota and budget reservation
→ single, cascade, shadow, race, council, or fusion protocol
→ typed Cognitive Artifact
→ calibrated confidence and provenance
→ progress monitoring
→ escalation or completion
→ canonical Context Graph update
```

The core law remains:

> Many agents may reason. Multiple isolated Variants may compete. Exactly one incarnation-fenced Attempt may write within each Variant. Every consequential external mutation is brokered. Every proof names its exact subject. The repository remains sovereign.

A model saying “done,” a terminal becoming idle, a provider process exiting zero, a plan being displayed, a branch push returning success, or a reviewer saying “looks good” has no completion authority.

Completion means:

```text
exact Candidate
+ current qualifying proof bundle
+ required independent review
+ protected integration
+ verified target commit
+ successful observation window
```

---

# 2. What is new in this revision

The architecture now includes the following systems as first-class, non-optional components:

1. **Cognitive Task Taxonomy**  
   Every model call is classified by task type, risk, novelty, scope, evidence need, context need, and expected difficulty.

2. **Minimum Sufficient Intelligence Router**  
   The scheduler selects the least expensive eligible lane expected to meet the quality floor. The target is for roughly 70–90% of model invocations to use deterministic tools, economy models, or standard models, while difficult and high-risk work receives frontier models, higher effort, councils, or fusion.

3. **Model Fusion Engine**  
   Centerrail supports independent proposals, rank-and-select, rank-and-fuse, critique-revise, debate, map/reduce, planning councils, code Variants, and verifier ensembles.

4. **Struggle and Completion Monitor**  
   It detects lack of meaningful progress, repeated failures, oscillation, context saturation, low confidence, test stagnation, scope churn, tool loops, and false completion. It escalates model strength, effort, collaborators, decomposition, or human attention according to policy.

5. **Canonical Context Graph and Capsule Compiler**  
   Provider-native conversations are caches. Centerrail owns facts, decisions, questions, source references, artifacts, scope, evidence, and continuity. Provider-specific capsules are compiled from this canonical state.

6. **Verified Context Compression**  
   Compression is an explicit, versioned cognitive task with coverage metrics, loss declarations, back-references, and optional independent checking. The original source remains recoverable.

7. **Behavior Policy Engine**  
   Bad coding-agent behavior is described in a versioned machine-enforceable catalog. Rules prevent, pause, quarantine, remediate, or terminate actions such as copying repositories into temporary folders, creating junk directories, polluting Git, disabling tests, bypassing checks, using broad destructive commands, or claiming completion without delivery evidence.

8. **Structured Session Supervisor with PTY Compatibility Plane**  
   Official structured protocols are authoritative. PTY/TTY streams are mirrored for live visibility and compatibility, with cross-platform Unix PTY and Windows ConPTY support. Routine plan acceptance, permission prompts, context cycling, and provider dialogs are automated by adapter state machines and Centerrail policy.

9. **Cross-Platform Runner Matrix**  
   Linux provides the hardened reference boundary. macOS and Windows support local development and controlled execution, with explicit security-grade labels and remote Linux/microVM fallback for high-risk tasks.

10. **Complete Cognitive and Operations Portal**  
    The dashboard exposes routing, fusion, quota, progress, struggle, context lineage, session state, behavior events, workspace hygiene, diffs, verification, integration, and audit—not just terminal activity.

---

# 3. Design doctrine

Centerrail follows these doctrines:

## 3.1 Deterministic mechanisms before model judgment

Use code, schemas, policy, Git, tests, and authoritative provider APIs for facts that can be established mechanically. Models interpret ambiguity; they do not grant themselves authority or attest to their own side effects.

## 3.2 Economy by default, quality by contract

Low-cost models are preferred only when an eligible lane is calibrated to the task class and repository. Cost targets never override risk, evidence, provider authorization, data policy, or quality floors.

## 3.3 Structured protocol before terminal inference

Preferred integration order:

```text
official SDK or app server
→ documented JSON-RPC / ACP
→ stable JSON or NDJSON headless interface
→ typed non-interactive CLI
→ PTY compatibility observation
```

Terminal output is useful for humans and fallback detection, but never the sole authority for completion, authentication, rate limiting, permission acceptance, or liveness.

## 3.4 Canonical state outside provider sessions

A provider conversation may disappear, compact incorrectly, or become unavailable. No Mission, decision, acceptance condition, scope grant, Candidate, Evidence item, or unresolved question may exist only inside that conversation.

## 3.5 Verified effects, not assumed effects

A command returning success is not proof that its effect occurred. Every consequential effect is read back from an authoritative surface and recorded in an immutable receipt.

## 3.6 Safety belongs at the action boundary

Do not rely on every prompt to remember safety rules. Destructive actions, cleanup, completion, delivery, scope expansion, approval, and context cycling enforce their preconditions in Rust at the choke point.

## 3.7 Explicit uncertainty

Observation values include `UNKNOWN`, `STALE`, and `CONTRADICTORY`. Unknown quota is not capacity. A failed read is not an empty queue. A missing terminal update is not a dead process. A timeout is not proof of non-execution.

## 3.8 Repository sovereignty

Protected branch rules, required checks, merge-group verification, and repository policy remain the final integration authority. Centerrail has no “skip checks” escape hatch.

---

# 4. Non-negotiable invariants

These invariants are enforced by Rust transition logic, database constraints, sandbox policy, adapter contracts, and repository rules—not prompt instructions.

## 4.1 Authority and transaction invariants

| ID | Invariant |
|---|---|
| A1 | At most one authoritative writer lease exists per Variant. |
| A2 | Fence epochs are monotonically increasing and never reused. |
| A3 | Every authoritative transition verifies the complete Authority Token. |
| A4 | A stale Attempt cannot heartbeat, expand scope, attach authoritative Evidence, create an Effect, finalize, push, select, review, cancel, or terminate a successor. |
| A5 | Plan Revision and Graph Delta materialization is atomic, content-addressed, and idempotent. |
| A6 | Every acknowledged state-changing command is durably recorded and idempotent. |
| A7 | A timeout does not imply that an operation did not occur. |
| A8 | Unknown or contradictory liveness cannot authorize destructive cleanup or reassignment. |
| A9 | Database time is authoritative for distributed lease expiry; runner monotonic time self-fences earlier. |
| A10 | Runtime observations are projections of the ledger and external systems, never co-equal authority. |

## 4.2 Workspace and Git invariants

| ID | Invariant |
|---|---|
| W1 | No two writers share mutable checkout, `.git`, index, refs, cache, browser profile, or sandbox state. |
| W2 | Writable Attempts never use Git worktrees. |
| W3 | Every writable Attempt starts from an exact full commit SHA. |
| W4 | Every actual delivered write is covered by a granted Change Intent. |
| W5 | Out-of-scope writes pause execution before further mutation. |
| W6 | Runtime-generated provider files are outside the product repository or forcibly excluded and attested. |
| W7 | Candidate preparation requires a classified, clean workspace manifest. |
| W8 | No cleanup may delete a workspace until preservation is verified and a deletion receipt is recorded. |
| W9 | Dry-run and execution use the same decision function and input snapshot. |
| W10 | Hostile Git hooks, filters, credential helpers, includes, protocols, diff drivers, merge drivers, and submodule behavior are controlled by runner policy. |

## 4.3 Cognitive routing and fusion invariants

| ID | Invariant |
|---|---|
| C1 | Every model invocation belongs to an immutable Cognitive Task and Dispatch Decision. |
| C2 | Every invocation names provider, model, profile, harness, adapter, prompt version, context capsule, budget reservation, and task taxonomy. |
| C3 | Hard policy eligibility is evaluated before any cost or performance score. |
| C4 | A model may request escalation but cannot lower its own risk, quality floor, evidence requirement, or permission boundary. |
| C5 | Economy-share targets cannot force an ineligible low-tier model. |
| C6 | Fusion contributors are isolated and their outputs retain provenance. |
| C7 | A fuser cannot erase disagreements, unsupported claims, or source attribution. |
| C8 | Code fusion produces a new fenced Variant or Candidate; it never mutates contributor workspaces in place. |
| C9 | Author-generated confidence is advisory and cannot satisfy independent verification. |
| C10 | Routing and fusion policies are versioned, explainable, and replayable. |

## 4.4 Context invariants

| ID | Invariant |
|---|---|
| X1 | Provider-native session history is a cache, not canonical memory. |
| X2 | Every compressed capsule links to all source nodes and the original artifacts remain recoverable. |
| X3 | Compression records omissions, unresolved contradictions, confidence, and coverage. |
| X4 | Decisions, acceptance requirements, scope grants, Evidence, and unresolved blockers cannot be dropped by compression. |
| X5 | Cross-provider migration creates a new session identity while preserving the same canonical Work Package context lineage. |
| X6 | Context saturation triggers a control-plane action before the provider becomes unable to respond. |
| X7 | Clearing or compacting context cannot destroy uncommitted work or authoritative state. |
| X8 | A provider-specific formatting optimization cannot change canonical meaning. |

## 4.5 Quota and budget invariants

| ID | Invariant |
|---|---|
| Q1 | Every provider use is attributable to a named authorized profile and owner. |
| Q2 | Unknown quota remains unknown. |
| Q3 | Every turn has a durable quota reservation and budget reservation before dispatch. |
| Q4 | Actual usage settles reservations; estimation error updates the forecaster. |
| Q5 | Consumer identities are never pooled or rotated to bypass provider limits. |
| Q6 | Emergency reserve floors cannot be consumed by speculative work. |
| Q7 | A vendor-wide throttle opens a circuit breaker and de-correlates retries. |
| Q8 | Pricing and quota interpretations are versioned snapshots. |

## 4.6 Session and behavior invariants

| ID | Invariant |
|---|---|
| S1 | Structured events are authoritative when available; PTY text is compatibility evidence only. |
| S2 | Routine provider UI dialogs are handled by adapter policy, not human terminal babysitting. |
| S3 | Centerrail policy approval is distinct from a provider’s local plan/permission prompt. |
| S4 | A session cannot remain indefinitely in an unclassified blocked state. |
| S5 | Every process tree is owned by one runner session and terminable as a unit. |
| S6 | Every behavior rule has a stable ID, version, detector, severity, enforcement action, and evidence. |
| S7 | A model cannot suppress, delete, or downgrade its own behavior event. |
| S8 | Destructive behavior rules fail closed when state observation is unknown. |
| S9 | Completion requires machine-verifiable postconditions, not an agent claim. |
| S10 | Temporary artifacts are confined to declared ephemeral roots and cannot enter a Candidate without explicit classification. |

## 4.7 Evidence, review, and effect invariants

| ID | Invariant |
|---|---|
| E1 | Every Candidate has exact base, head, tree, patch, and lineage identity. |
| E2 | Evidence is exact-subject and exact-environment bound. |
| E3 | Writer Evidence cannot satisfy independent-verification requirements. |
| E4 | Only policy-approved typed `PASS` satisfies a blocking gate. |
| E5 | Evidence invalidates when its subject or conservative input closure changes. |
| E6 | Every privileged external mutation has a prior durable Effect Intent. |
| E7 | Every dispatched Effect is reconciled into a verified Receipt or explicit uncertainty state. |
| E8 | `OUTCOME_UNKNOWN` is resolved before replay unless non-execution or safe idempotence is proven. |
| E9 | A reviewer must satisfy independence policy for the reviewed subject. |
| E10 | Repository rules and merge-group checks remain final integration authority. |

Any violation of A1–A7, W1–W5, Q1–Q5, S5, or E1–E8 is a severity-one control-plane incident.

# 5. Trust-plane architecture

Centerrail has seven explicit planes.

| Plane | Responsibility |
|---|---|
| **Control Plane** | Mission, Plan, Work Package, Variant, Attempt, lease, fence, command, policy, state transition |
| **Cognitive Execution Plane** | task classification, routing, provider/model invocation, fusion, progress and escalation |
| **Repository Execution Plane** | private clones, sandbox, tools, file mutations, checkpoints, Candidate preparation |
| **Session Supervision Plane** | provider protocol, process tree, PTY/ConPTY mirror, dialog automation, interrupt and recovery |
| **Independent Verification Plane** | clean reconstruction, deterministic gates, hidden evaluators, semantic review inputs |
| **Effect and Delivery Plane** | GitHub and future external mutations, just-in-time credentials, read-back reconciliation |
| **Evidence and Audit Plane** | content-addressed artifacts, provenance, proof bundles, event history, retention, tamper evidence |

The portal is a projection across these planes. It is never an authority source.

## 5.1 Reference deployment

```text
Vite / React / TypeScript portal
        │ HTTPS JSON + SSE + terminal WebSocket
        ▼
Rust modular control-plane daemon
  domain · ledger · workflow · policy
  cognitive router · quota · context
  progress · behavior · projections · audit
        │
        ├── SQLite WAL (local) or PostgreSQL (team)
        ├── content-addressed artifact store
        ├── durable event/outbox stream
        │
        ├── mTLS runner protocol
        ▼
Rust Runner
  workspace manager · sandbox manager
  tool gateway · process supervisor
  provider adapter host · PTY/ConPTY mirror
  checkpoint · Candidate preparation
        │
        ├── Provider Harness Enclave
        ├── Untrusted Repository Sandbox
        └── local artifact spool
        │
        ▼
Independent Rust Verifier
        │
        ▼
Rust Effect Broker
        │
        ▼
GitHub App / future external providers
```

## 5.2 Architectural style

Start with a **modular Rust monolith** for control-plane domain logic. Runner, verifier, and Effect Broker are separate processes because they are trust boundaries. Do not split the control plane into network microservices until load, isolation, or deployment independence justifies the operational cost.

The ledger stores current transactional state and append-only event/audit records. The system does not depend on full event sourcing to operate, but every derived projection is rebuildable from canonical tables plus events.

## 5.3 Deployment modes

### Local developer mode

- one control-plane daemon;
- SQLite WAL;
- local content-addressed storage;
- one or more local runners;
- loopback HTTPS or Unix-domain socket;
- local OIDC-free principal;
- optional remote hardened verifier.

### Team mode

- PostgreSQL;
- S3-compatible object storage;
- replicated stateless API/projection instances;
- authenticated mTLS runners;
- OIDC and RBAC;
- independent verifier pools;
- GitHub App;
- centralized quota and cost service.

### Enterprise mode

- multi-tenancy;
- SSO/SCIM;
- KMS/HSM-backed service credentials;
- data-residency controls;
- dedicated or attested runners;
- S2/S3 verifier classes;
- tamper-evident audit roots;
- disaster recovery with independently tested restoration;
- retention and legal-hold policies.

The domain objects, APIs, and protocols remain compatible across modes.

---

# 6. Authoritative domain model

The final hierarchy is:

```text
Organization
└── Repository
    └── Mission
        ├── Acceptance Contract
        ├── Plan Revision
        │   ├── Graph Delta
        │   └── Work Package
        │       ├── Selection Group
        │       │   ├── Variant
        │       │   │   ├── Attempt
        │       │   │   │   ├── Agent Session
        │       │   │   │   ├── Cognitive Task
        │       │   │   │   │   ├── Invocation
        │       │   │   │   │   └── Collaboration Run
        │       │   │   │   ├── Tool Events
        │       │   │   │   ├── Behavior Events
        │       │   │   │   ├── Progress Observations
        │       │   │   │   ├── Context Capsules
        │       │   │   │   └── Checkpoints
        │       │   │   ├── Candidate lineage
        │       │   │   ├── Evidence
        │       │   │   └── Review
        │       │   └── Selection
        │       ├── Integration
        │       └── Observation Window
        ├── Effect Intents and Receipts
        └── Interventions
```

## 6.1 Mission

An authorized engineering objective with:

```text
mission_id
organization_id
repository_ids
title
objective
business_context
acceptance_contract_id
risk_class
data_classification
requested_by
base_commits
target_branches
budget
deadline_policy
routing_policy_id
workflow_strategy
observation_policy
status
created_at
```

The Acceptance Contract is immutable after admission. A change creates a new authorized contract version and Plan Revision and invalidates any affected work or Evidence.

## 6.2 Acceptance Contract

Contains machine-addressable requirements:

```text
requirement_id
description
kind
criticality
verification_method
required_evidence_tier
required_reviewer_independence
risk_escalation
source
status
```

Kinds include:

- functional behavior;
- invariant;
- compatibility;
- performance;
- security;
- privacy;
- migration;
- operational;
- accessibility;
- user-interface;
- documentation;
- rollback;
- observation/survival.

Every Work Package and gate maps to one or more requirements. Uncovered requirements block plan activation.

## 6.3 Plan Revision

Immutable and content-addressed:

```text
plan_revision_id
mission_id
parent_revision_id
canonical_plan_hash
planner_collaboration_run_id
work_packages
typed_dependencies
predicted_change_intents
acceptance_mapping
routing_hints
fusion_hints
budget_effect
risk_effect
validation_result
created_at
activated_at
```

`materialize(plan_hash)` inserts the complete graph, fence counters, ready rows, predicted intents, and plan event in one serializable transaction. Retrying the same canonical hash returns the same graph.

## 6.4 Graph Delta

Dynamic planning creates a new immutable delta rather than mutating a Plan:

```text
graph_delta_id
parent_plan_hash
expected_graph_sequence
deterministic_node_ids
deterministic_edge_ids
reason
acceptance_mapping
scope_effect
risk_effect
budget_effect
validator_version
authorization
canonical_hash
```

It applies atomically or not at all.

## 6.5 Work Package

The smallest independently contracted logical unit:

```text
work_package_id
mission_id
plan_revision_id
task_class
title
contract
acceptance_requirements
dependencies
read_scope
predicted_change_intent
risk_class
data_classification
base_commit
required_capabilities
provider_independence_rules
budget
timeout
retry_policy
escalation_policy
context_policy
collaboration_protocol
status
priority
created_at
```

## 6.6 Selection Group

A container for independent alternatives:

```text
selection_group_id
work_package_id
protocol
selection_rubric
minimum_diversity
max_variants
budget
selection_state
selected_variant_id
selection_evidence
```

Normal work has one default Variant. A race or code-fusion protocol creates multiple Variants.

## 6.7 Variant

The unit of writable authority:

```text
variant_id
selection_group_id
work_package_id
variant_strategy
fence_counter
budget
workspace_policy
candidate_head
state
```

Each Variant has:

- a separate permanent fence counter;
- at most one active writer;
- a private clone;
- a separate budget;
- its own Candidate lineage;
- its own Evidence;
- no mutable state shared with another Variant.

## 6.8 Attempt and Authority Token

An Attempt is one execution incarnation of a Variant.

```rust
pub struct AuthorityToken {
    pub organization_id: OrganizationId,
    pub repository_id: RepositoryId,
    pub mission_id: MissionId,
    pub acceptance_contract_id: AcceptanceContractId,
    pub plan_revision_id: PlanRevisionId,
    pub graph_sequence: u64,
    pub work_package_id: WorkPackageId,
    pub selection_group_id: SelectionGroupId,
    pub variant_id: VariantId,
    pub attempt_id: AttemptId,
    pub attempt_fence: u64,
    pub runner_id: RunnerId,
    pub runner_epoch: u64,
    pub workspace_id: WorkspaceId,
    pub workspace_nonce: [u8; 32],
    pub scope_revision: u64,
    pub context_revision: u64,
    pub config_snapshot_hash: Digest,
    pub policy_snapshot_hash: Digest,
    pub routing_policy_hash: Digest,
    pub credential_profile_id: Option<ProfileId>,
    pub credential_generation: Option<u64>,
}
```

A display name, PID, terminal title, provider thread ID, branch name, or filesystem path grants no authority.

## 6.9 Cognitive Task

Every model invocation originates from an immutable Cognitive Task:

```text
cognitive_task_id
attempt_id or verifier_run_id
parent_task_id
task_class
objective
input_manifest_hash
context_capsule_id
output_schema_id
risk_class
quality_floor
latency_class
budget
routing_policy_id
collaboration_protocol
independence_requirements
completion_contract
created_at
```

A Cognitive Task is smaller than a Work Package and may represent planning, compression, classification, code generation, critique, review, or synthesis.

## 6.10 Invocation

One provider/model turn:

```text
invocation_id
cognitive_task_id
dispatch_decision_id
provider
model
model_snapshot_id
profile_id
harness
adapter_version
native_session_id
prompt_template_version
context_capsule_id
reasoning_effort
quota_reservation_id
budget_reservation_id
started_at
completed_at
status
usage
cost
raw_event_artifact
output_artifact
```

## 6.11 Collaboration Run

A set of Invocations under one protocol:

```text
collaboration_run_id
cognitive_task_id
protocol
contributor_invocations
ranker_invocations
fuser_invocation
diversity_score
budget
state
selected_output_id
disagreement_artifact
```

## 6.12 Cognitive Artifact

Typed output from a cognitive task:

```text
artifact_id
cognitive_task_id
producer_invocation_id
kind
schema_version
claims
source_references
assumptions
uncertainties
contradictions
recommendations
proposed_actions
confidence
content_hash
created_at
```

Kinds include:

- classification;
- summary;
- context compression;
- repository map;
- plan proposal;
- test proposal;
- code proposal;
- patch explanation;
- review finding;
- risk analysis;
- synthesis;
- decision recommendation;
- escalation diagnosis.

## 6.13 Candidate

An immutable code subject:

```text
candidate_id
variant_id
attempt_id
base_commit
head_commit
tree_hash
patch_hash
lineage_subject
environment_digest
toolchain_digest
granted_scope
actual_scope
parent_candidate_id
prepared_at
```

A rebase creates a new Candidate. Descendants become `ANCESTRY_STALE` until rebuilt or explicitly revalidated.

## 6.14 Evidence

An immutable proof claim:

```text
evidence_id
subject_type
subject_id
subject_commit
subject_patch_hash
tier
kind
producer
command
tool_version
environment_digest
input_manifest
result
artifact_hash
policy_version
started_at
completed_at
valid_until
invalidation_policy
status
```

## 6.15 Effect

A proposed external mutation:

```text
effect_intent_id
logical_effect_key
provider
target_identity
desired_state_hash
remote_preconditions
authority_token_hash
policy_version
payload_hash
provider_idempotency_key
state
```

Receipt:

```text
effect_receipt_id
effect_intent_id
provider_request_id
observed_remote_identity
observed_remote_version
observed_state_hash
raw_receipt_hash
verification_method
verification_result
created_at
```

## 6.16 Intervention

A versioned human or policy command:

```text
intervention_id
mission_id
work_package_id
attempt_id
requested_by
required_role
reason
options
risk_effect
scope_effect
budget_effect
context_effect
evidence_invalidation_effect
state
resolution
created_at
resolved_at
```

Human steering is never an unrecorded terminal side channel.

# 7. Cognitive task taxonomy

The router must not infer difficulty from prompt length or changed-line count alone. Every Cognitive Task receives a structured classification.

## 7.1 Classification dimensions

```rust
pub struct TaskClassification {
    pub primary_class: TaskClass,
    pub secondary_classes: Vec<TaskClass>,
    pub determinism: DeterminismLevel,
    pub novelty: NoveltyLevel,
    pub ambiguity: AmbiguityLevel,
    pub repository_scope: ScopeSize,
    pub semantic_risk: RiskClass,
    pub data_classification: DataClass,
    pub evidence_requirement: EvidenceTier,
    pub context_requirement: ContextDemand,
    pub tool_requirement: CapabilitySet,
    pub expected_output_size: OutputSize,
    pub latency_class: LatencyClass,
    pub confidence: f32,
    pub classifier_version: VersionId,
    pub evidence: Vec<ClassificationSignal>,
}
```

Signals include:

- explicit workflow declaration;
- path and subsystem risk;
- language and framework;
- acceptance requirement kinds;
- number and type of dependency edges;
- repository history for similar tasks;
- benchmark outcomes;
- test and build topology;
- predicted Change Intent;
- issue keywords;
- current error signatures;
- novelty against repository embeddings;
- data sensitivity;
- uncertainty and contradiction density;
- human-specified minimum tier.

## 7.2 Task classes

| Class | Description | Default lane | Default collaboration |
|---|---|---|---|
| `deterministic_transform` | formatting, schema generation, exact conversion | D0 | single tool |
| `extract_structured` | extract facts into a schema | M1 | single; cheap verifier if critical |
| `classify_route` | label task, risk, provider eligibility | M1 | shadow calibration |
| `summarize_local` | summarize a bounded source with citations | M1 | single + deterministic coverage check |
| `compress_context` | produce provider-portable Context Capsule | M1/M2 | compressor + checker |
| `retrieve_rank` | rank repository facts or artifacts | D0/M1 | map/reduce |
| `repository_map` | identify architecture and relevant files | M1/M2 | parallel scouts |
| `error_triage` | interpret build/test/runtime failure | M1/M2 | cascade |
| `test_selection` | choose relevant test subset | M1/M2 | deterministic constraints |
| `test_authoring` | add bounded tests | M2 | single or pair critique |
| `documentation` | narrow docs/update | M1/M2 | single |
| `mechanical_code_edit` | renames, API substitutions, generated changes | D0/M1/M2 | deterministic or single |
| `bounded_bug_fix` | localized defect with reproducible failure | M2 | cascade on struggle |
| `feature_implementation` | multi-file product feature | M2/M3 | plan + implement + review |
| `broad_refactor` | architectural change across modules | M3 | council + staged pipeline |
| `architecture_design` | choose components, boundaries, tradeoffs | M3/M4 | independent proposals + fusion |
| `security_analysis` | auth, injection, secret, permission, boundary work | M3/M4 | adversarial council |
| `migration_design` | schema/data migration and rollback | M3/M4 | council + human policy gate |
| `performance_analysis` | profiling, benchmark, algorithmic change | M2/M3/M4 | race or scientific loop |
| `ui_visual_validation` | browser behavior, accessibility, screenshots | M2/M3 | tool-assisted verifier |
| `code_review` | semantic review of exact Candidate | M2/M3 | blind independent review |
| `incident_reproduction` | reproduce live failure and isolate mechanism | M2/M3 | investigate protocol |
| `integration_repair` | rebase, conflict, merge-group failure | M2/M3 | pipeline |
| `fusion_rank` | compare cognitive artifacts | M1/M2 | pairwise or listwise rank |
| `fusion_synthesize` | create a superior answer from alternatives | M2/M3 | provenance-preserving fusion |
| `completion_assessment` | determine whether contract is satisfied | D0 + M2 | deterministic first, independent model second |
| `struggle_diagnosis` | explain why progress has stalled | M1/M2 | independent observer |
| `behavior_triage` | classify a behavior event and remediation | D0/M1 | policy first |
| `human_decision_brief` | prepare a decision packet | M2/M3 | fusion for high risk |

## 7.3 Difficulty is not risk

A task can be easy but high-risk. Examples:

- changing one line of authentication code;
- altering a payment limit;
- changing a migration default;
- deleting an access-control check;
- editing a protected deployment manifest.

Risk is a hard policy dimension. Difficulty controls model capacity. A low-difficulty R3 task may still require a frontier reviewer, clean verifier, and human approval.

## 7.4 Classification pipeline

```text
workflow-declared class
    ↓
deterministic repository and path policy
    ↓
cheap structured classifier
    ↓
confidence and contradiction check
    ↓
optional shadow classifier
    ↓
hard risk floor and human override
    ↓
stored TaskClassification
```

When classification confidence is below policy threshold, Centerrail routes to a stronger classifier or opens a classification fusion run. It does not silently assign a low tier.

---

# 8. Model and provider registry

Centerrail routes to **lanes**, not brand names.

## 8.1 Lane identity

```rust
pub struct Lane {
    pub lane_id: LaneId,
    pub provider: ProviderId,
    pub model: ModelId,
    pub model_snapshot: ModelSnapshotId,
    pub harness: HarnessId,
    pub adapter_version: VersionId,
    pub profile_id: ProfileId,
    pub reasoning_effort: EffortLevel,
    pub sandbox_class: SandboxClass,
    pub host_class: HostClass,
    pub data_policy: DataPolicyId,
}
```

## 8.2 Model snapshot

Provider model names and capabilities change. Never hard-code lasting policy to a marketing name.

```text
model_snapshot_id
provider
provider_model_id
display_name
family
released_at
observed_at
context_window
max_output
structured_output
tool_use
vision
computer_use
reasoning_effort_levels
latency_distribution
price_snapshot
quota_dimensions
data_policy
known_limitations
benchmark_calibration
status
```

OpenAI’s current Codex family illustrates why snapshots matter: Luna, Terra, and Sol have distinct intended operating points. Centerrail maps such provider-native names into configured economy, standard, and frontier tiers, but policy refers to capabilities and calibrated outcomes rather than assuming the names remain stable.

## 8.3 Capability negotiation

Every adapter defaults to `UNSUPPORTED` until a pinned-version conformance suite proves:

```text
structured_events
structured_output_schema
native_resume
native_fork
session_export
session_import
turn_interrupt
mid_turn_steering
tool_approvals
plan_mode_control
usage_events
quota_source
auth_challenge
model_selection
reasoning_effort
browser_control
image_input
MCP
file_references
context_usage
native_compaction
headless_mode
PTY_required
multiline_prompt
```

Capabilities may be:

```text
SUPPORTED
SUPPORTED_WITH_LIMITATIONS
EXPERIMENTAL
UNSUPPORTED
UNKNOWN
```

The scheduler does not dispatch a task whose required capability is `UNKNOWN`.

## 8.4 Provider adapters

```rust
#[async_trait::async_trait]
pub trait HarnessAdapter: Send + Sync {
    fn descriptor(&self) -> HarnessDescriptor;
    async fn probe(&self, profile: &ProfileRef) -> Result<ProbeResult>;
    async fn list_models(&self, profile: &ProfileRef) -> Result<Vec<ModelSnapshot>>;
    async fn observe_quota(&self, profile: &ProfileRef) -> Result<Vec<QuotaObservation>>;
    async fn begin_login(&self, profile: &ProfileRef) -> Result<AuthChallenge>;
    async fn start(&self, request: StartSession) -> Result<SessionHandle>;
    async fn resume(&self, request: ResumeSession) -> Result<SessionHandle>;
    async fn send(&self, session: &SessionHandle, turn: Turn) -> Result<TurnHandle>;
    async fn steer(&self, session: &SessionHandle, message: SteeringMessage) -> Result<Ack>;
    async fn approve_local_plan(&self, session: &SessionHandle, decision: PlanDecision) -> Result<Ack>;
    async fn respond_permission(&self, session: &SessionHandle, decision: PermissionDecision) -> Result<Ack>;
    async fn compact(&self, session: &SessionHandle, request: CompactRequest) -> Result<ContextTransition>;
    async fn checkpoint(&self, session: &SessionHandle) -> Result<SessionCheckpoint>;
    async fn interrupt(&self, session: &SessionHandle) -> Result<Ack>;
    async fn terminate(&self, session: &SessionHandle) -> Result<Ack>;
    fn events(&self, session: &SessionHandle) -> HarnessEventStream;
}
```

## 8.5 Initial integrations

### OpenAI Codex

Preferred local integration: app-server over stdio JSONL/JSON-RPC. It provides threads, turns, streamed events, approvals, account state, and rate-limit surfaces. Experimental remote transports are not a production dependency.

### Claude Code

Preferred integration: headless stream-JSON or the Agent SDK, with bare/scripted mode where appropriate. Hooks may be used as deterministic integration points, but Centerrail does not rely on repository-controlled hooks for authority. Stream result events provide response, cost, and session metadata.

### Cursor Agent

Preferred integration: ACP over stdio JSON-RPC. Headless print mode is available for simple one-shot Cognitive Tasks. ACP is used for session lifecycle, streamed updates, permissions, and advanced clients.

### Google Antigravity

Preferred integration: documented headless NDJSON mode with model and effort selection. Slash commands that break the event stream are invoked as separate adapter operations, not injected into the prompt stream.

## 8.6 Profile identity verification

After starting or resuming a provider session, the adapter probes the effective provider account/profile. A mismatch with the authorized profile or credential generation fails closed.

No ambient home directory, default login, shell profile, or shared token may silently determine the account.

---

# 9. Minimum Sufficient Intelligence Router

The routing objective is:

> Select the least expensive eligible execution strategy that is calibrated to meet the task’s quality floor and deadline, while preserving quota reserve and minimizing expected integration failure.

The desired 70–90% economy share is a **portfolio target**, not a per-task command.

## 9.1 Tier model

| Tier | Mechanism | Typical work |
|---|---|---|
| `D0` | deterministic program, no model | formatting, exact generation, static extraction |
| `M1` | economy model | classification, bounded summaries, compression, narrow docs |
| `M2` | standard coding model | bounded fixes, tests, routine components, reviews |
| `M3` | frontier model / high effort | difficult debugging, broad refactors, architecture |
| `M4` | council, fusion, or isolated race | critical ambiguity, security, migrations, high-value architecture |

Provider mappings are configuration snapshots. Example:

```text
OpenAI: Luna → M1, Terra → M2, Sol → M3
```

Mappings for other providers are derived from current capabilities, cost, latency, and Centerrail benchmark calibration.

## 9.2 Portfolio objectives

Configurable organization objectives:

```toml
[routing.portfolio]
economy_call_share_min = 0.70
economy_call_share_max = 0.90
economy_token_share_target = 0.65
frontier_budget_reserve = 0.20
speculation_budget_share_max = 0.10
quality_floor = 0.97
```

Definitions:

- **economy call share:** fraction of Invocations on D0/M1 lanes;
- **economy token share:** fraction of model tokens on M1 lanes;
- **frontier reserve:** protected quota/budget for M3/M4 work;
- **quality floor:** accepted surviving result rate relative to the best approved baseline for the task stratum.

The router may violate the economy-share target to satisfy risk, quality, deadline, or outage policy. It records the reason.

## 9.3 Hard eligibility filters

A lane is excluded unless it satisfies:

- risk-class minimum;
- data-classification and provider policy;
- repository permission;
- task capability requirements;
- required context window;
- required structured-output support;
- sandbox class;
- platform compatibility;
- quota reservation;
- budget reservation;
- concurrency limit;
- profile ownership and authorization;
- continuity constraints;
- reviewer-independence constraints;
- active circuit breakers;
- current adapter conformance;
- task-class quality floor.

## 9.4 Soft objective

```text
utility(lane, task) =
    P(verified_pass | lane, task, repo, context) × business_value
  - λ1 × expected_model_cost
  - λ2 × expected_runner_and_CI_cost
  - λ3 × expected_latency
  - λ4 × expected_repair_cost
  - λ5 × quota_exhaustion_risk
  - λ6 × integration_conflict_risk
  - λ7 × provider_outage_risk
  - λ8 × context_migration_risk
  + λ9 × continuity_value
  + λ10 × diversity_value
  + λ11 × bounded_exploration_value
```

Every Dispatch Decision stores:

```text
classification
eligible lanes
excluded lanes and reasons
feature vector
calibration snapshot
score terms
quota shadow price
budget effect
chosen lane or collaboration protocol
fallback ladder
scheduler version
```

## 9.5 Routing stages

```text
1. Try D0 when a deterministic implementation exists.
2. Try M1 when calibrated quality ≥ floor.
3. Try M2 when M1 is ineligible or confidence is insufficient.
4. Try M3 for high difficulty, high novelty, high ambiguity, or escalation.
5. Admit M4 only when expected value exceeds added cost/latency.
```

A lower tier may return:

```text
COMPLETE
ESCALATE_DIFFICULTY
ESCALATE_RISK
NEED_MORE_CONTEXT
NEED_TOOL
AMBIGUOUS
UNSUPPORTED
```

Returning uncertainty is rewarded. Fabricating completion is penalized and may trigger a behavior rule.

## 9.6 Router learning

Start with deterministic rules and manually weighted scores. Introduce constrained learning only after attributable outcomes exist.

Training data must include:

- task classification;
- repository and language;
- exact model/harness/adapter versions;
- input context characteristics;
- accepted/rejected result;
- Evidence quality;
- repair loops;
- cost;
- latency;
- human intervention;
- integration result;
- observation survival;
- escaped defects.

Requirements:

- randomized holdouts;
- champion/challenger policies;
- capped exploration;
- inverse-propensity or randomized-treatment correction;
- calibration by repository and task stratum;
- stale-history decay;
- no learning of permissions, risk gates, evidence floors, or merge rules;
- explainable replay of every adaptive decision.

## 9.7 Shadow routing

Before moving a task class to a cheaper lane:

1. continue using the approved lane for authority;
2. run the proposed cheaper lane in read-only shadow mode on a sampled subset;
3. compare outputs with hidden acceptance and independent review;
4. calculate quality, latency, cost, and disagreement;
5. promote only when confidence intervals satisfy policy;
6. retain a continuing holdout after promotion.

---

# 10. Routing defaults by task class

| Task | Default | Promotion trigger | Fusion policy |
|---|---|---|---|
| structured classification | M1 | low confidence or policy disagreement | 2×M1 vote, M2 arbiter |
| bounded summary | M1 | low source coverage | M1 compressor + M1 checker |
| context compression | M1 | high-risk context or contradiction | M1/M2 dual compression + M2 fuse |
| repository map | M1 scouts | missing acceptance coverage | map/reduce with M2 synth |
| error triage | M1/M2 | repeated unknown error | M3 diagnosis |
| test selection | D0+M1 | coverage uncertainty | M2 checker |
| narrow docs | M1 | semantic/API impact | M2 review |
| mechanical edit | D0/M1 | tool cannot prove transform | M2 |
| bounded bug fix | M2 | struggle score threshold | M3 or second M2 critic |
| feature implementation | M2 | broad scope/novelty | M3 planner/reviewer |
| architecture design | M3 | material disagreement | two M3 proposals + M3 fuser |
| security | M3 | R3/R4 or novel boundary | adversarial multi-provider council |
| migration | M3 | irreversible/data risk | council + E3 verifier + human gate |
| performance | M2/M3 | multiple viable algorithms | isolated code race |
| code review | M2 | R3 or low reviewer confidence | M3 independent second review |
| completion assessment | D0 then M2 | contradiction/unknown | M3 arbiter |
| struggle diagnosis | M1/M2 observer | repeated escalation | M3 replanner |

These are defaults. Repository calibration and policy may raise them.

# 11. Model Fusion Engine

Fusion is not “ask several models and paste their answers together.” It is a typed collaboration protocol with independent inputs, provenance, disagreement preservation, budget admission, exact output contracts, and verification.

Research on mixture-of-agents, rank-and-fuse systems, and learned routing supports the general opportunity, but Centerrail treats those techniques as configurable protocols rather than universal improvements. Fusion adds cost, latency, and correlated failure risk; it is admitted only when expected value is positive.

## 11.1 Fusion principles

1. Contributors receive the same immutable Cognitive Task and source manifest.
2. Contributors are blind to one another unless the protocol explicitly includes critique.
3. Outputs use the same typed schema.
4. Every claim retains contributor and source provenance.
5. Ranking and synthesis are distinct operations.
6. The fuser receives the original task, not only candidate outputs.
7. Disagreements are explicit data.
8. The fuser may select one output instead of combining incompatible ideas.
9. Code contributors write only in their own Variants.
10. A fused code result is produced in a new fenced synthesis Variant.
11. Fusion output does not bypass independent verification.
12. High-risk fusion requires independence across provider/model/harness or an explicit correlation waiver.

## 11.2 Collaboration protocols

### `single`

One invocation. Default for calibrated low-risk work.

### `cascade`

```text
economy model
→ complete if confidence and deterministic checks pass
→ standard model if unresolved
→ frontier model if still unresolved
```

Best for keeping the majority of calls inexpensive.

### `shadow`

One authoritative invocation plus a non-authoritative alternative used for calibration. Shadow output cannot affect code or state.

### `pair_critique`

Contributor creates output; independent critic identifies defects; contributor or a new reviser produces a corrected result.

### `rank_select`

N independent outputs; ranker selects the strongest without synthesis. Preferred when outputs are mutually exclusive or fusion would dilute a correct concise answer.

### `triad_fusion`

The user-requested basic fusion pattern:

```text
Contributor A: provider/model family A
Contributor B: provider/model family B
Fuser C: sufficiently independent lane
```

A and B generate structured alternatives. C receives the original task, both artifacts, source references, deterministic checks, and a fusion rubric. C must:

- enumerate agreement;
- enumerate disagreement;
- identify unsupported claims;
- score each alternative;
- either select, synthesize, or declare unresolved;
- preserve citations/provenance;
- produce a machine-readable Fusion Report.

### `rank_fuse`

N contributors → pairwise/listwise ranker → top K → generative fuser. Use when there are many alternatives and ranking cost is justified.

### `critique_revise_rounds`

Alternating critique and revision with a strict maximum round count. Each round must add a new finding or produce a changed artifact; otherwise it is thrash.

### `debate`

Independent positions exchange structured objections. An arbiter decides against a rubric. Use sparingly for architecture, security, and policy—not routine coding.

### `map_reduce`

Many economy scouts analyze partitions of a repository, log set, or test matrix. A stronger reducer synthesizes a global map.

### `planner_council`

Two or more independent plan proposals, contradiction extraction, constraint validation, and a synthesis plan. The resulting Plan Revision remains subject to deterministic graph validation.

### `adversarial_council`

Builder, breaker, and judge roles across independent lanes. Used for security boundaries, migrations, incident response, and destructive effects.

### `code_race`

N isolated Variants implement the same Work Package. Each has its own fence, clone, budget, Candidate, and Evidence. A selector chooses a Candidate or opens a synthesis Variant.

### `code_synthesis`

A new synthesis Variant receives exact diffs, Candidates, test evidence, and reviewer findings from source Variants. It may cherry-pick, reimplement, or combine ideas, but all resulting writes belong to its own fence and Candidate lineage.

### `verifier_ensemble`

Independent verifiers inspect the same exact Candidate. Findings are deduplicated and confidence-calibrated. Review quorum is policy-based, not majority theater.

### `compression_ensemble`

Two compressors produce separate portable summaries; a checker compares source coverage and contradictions; a fuser creates the final capsule or retains both when fusion would lose information.

## 11.3 Fusion artifact schema

```rust
pub struct FusionReport {
    pub collaboration_run_id: CollaborationRunId,
    pub task_id: CognitiveTaskId,
    pub contributors: Vec<InvocationId>,
    pub rubric_version: VersionId,
    pub agreements: Vec<SupportedClaim>,
    pub disagreements: Vec<Disagreement>,
    pub candidate_scores: Vec<CandidateScore>,
    pub unsupported_claims: Vec<UnsupportedClaim>,
    pub selected_material: Vec<ProvenancePointer>,
    pub omitted_material: Vec<Omission>,
    pub synthesis: ArtifactRef,
    pub resolution: FusionResolution,
    pub residual_uncertainty: Vec<OpenQuestion>,
    pub confidence: f32,
}
```

Resolution:

```text
SELECT_A
SELECT_B
SELECT_OTHER
SYNTHESIZE
KEEP_MULTIPLE
ESCALATE
UNRESOLVED
INVALID_INPUT
```

## 11.4 Diversity score

Provider-name difference alone is insufficient.

```text
diversity_score =
    w1 × provider_family_difference
  + w2 × model_lineage_difference
  + w3 × harness_difference
  + w4 × prompt_strategy_difference
  + w5 × toolchain_difference
  + w6 × context_view_difference
  + w7 × evaluator_custody_difference
  - w8 × shared_training_or_gateway_correlation
```

The score and its components are stored. High-risk policy can require a minimum.

## 11.5 Fusion admission

```text
expected_fusion_value =
    P(single fails and fusion succeeds) × business_value
  + expected reviewer/test savings
  + diversity benefit
  - contributor cost
  - fuser cost
  - added latency cost
  - integration/conflict risk
  - contamination risk
```

Admit fusion when:

- task is above value threshold;
- ambiguity or novelty exceeds threshold;
- contributor lanes are sufficiently diverse;
- budget and quota reserves remain;
- deterministic solution is unavailable;
- predicted benefit exceeds configured margin;
- deadline permits added latency.

Do not admit fusion merely because idle models exist.

## 11.6 Triad fusion algorithm

```rust
async fn run_triad(task: CognitiveTask) -> Result<FusionReport> {
    let (lane_a, lane_b) = router.select_independent_contributors(&task)?;
    let reserve = quota.reserve_parallel(&task, &[lane_a, lane_b])?;

    let [a, b] = invoke_blind_parallel(task.clone(), lane_a, lane_b).await?;
    validate_schema(&a)?;
    validate_schema(&b)?;
    let disagreement = compare_artifacts(&a, &b)?;

    if deterministic_rank_can_select(&a, &b, &task) {
        return rank_select_without_fuser(task, a, b, disagreement).await;
    }

    let lane_c = router.select_fuser(&task, &[lane_a, lane_b], &disagreement)?;
    quota.reserve(&task, &lane_c)?;
    let fused = invoke_fuser(task, lane_c, &a, &b, &disagreement).await?;
    validate_fusion_provenance(&fused, &[a, b])?;
    independent_check_if_required(&task, &fused).await?;
    settle_all_reservations(reserve, &fused)?;
    Ok(fused)
}
```

## 11.7 Code fusion algorithm

```text
1. Materialize Selection Group.
2. Create Variant A and Variant B with independent fences and private clones.
3. Run contributors blind.
4. Prepare exact Candidate A and Candidate B.
5. Run deterministic gates independently.
6. Run read-only comparison:
   - acceptance coverage;
   - changed scope;
   - test outcomes;
   - performance/security evidence;
   - complexity and maintainability;
   - conflicts and complementary ideas.
7. Selector may:
   a. choose A;
   b. choose B;
   c. reject both;
   d. authorize Synthesis Variant C.
8. If C:
   - increment C fence;
   - create fresh private clone from exact base;
   - provide A/B diffs and evidence as read-only artifacts;
   - synthesize under C scope;
   - prepare Candidate C;
   - rerun all required Evidence.
9. Selection result is immutable and audited.
```

There is no in-place merge of two active writer workspaces.

## 11.8 Fusion quality metrics

- oracle win rate against hidden tests;
- accepted survival uplift;
- unique-finding yield;
- contradiction resolution rate;
- unsupported-claim rate;
- fuser regression rate;
- selected-versus-synthesized yield;
- cost and latency multiplier;
- provider diversity;
- human preference;
- repair-loop reduction;
- contamination rate.

Protocols are disabled for a task stratum when their confidence-adjusted uplift is non-positive.

---

# 12. Struggle, escalation, and completion monitoring

Centerrail must know when work is making progress, merely emitting activity, or failing in a loop.

## 12.1 Meaningful progress

Meaningful progress is a typed state change, not terminal bytes.

Examples:

- new source fact with exact provenance;
- acceptance requirement mapped;
- reproduction moved from unknown to deterministic;
- test added or failure narrowed;
- source diff changed within scope;
- compiler/test failure count reduced;
- Candidate prepared;
- review finding resolved;
- quota/context transition completed;
- blocker or question explicitly identified;
- external effect reconciled.

Non-progress activity:

- repeating the same command;
- rereading the same files without a new finding;
- rewriting equivalent plans;
- repeated handoffs on the same step;
- repeated context compression without reduced uncertainty;
- tool retries with the same inputs;
- long prose with no state update;
- fake completion or report-only cycles.

## 12.2 Progress Observation

```text
progress_observation_id
attempt_id
cognitive_task_id
kind
timestamp
source
before_hash
after_hash
delta
acceptance_coverage_delta
test_delta
scope_delta
uncertainty_delta
evidence_delta
meaningful
reason
```

## 12.3 Struggle state

```text
HEALTHY
WATCH
STRUGGLING
STALLED
BLOCKED_EXTERNAL
CONTEXT_AT_RISK
QUOTA_AT_RISK
POLICY_BLOCKED
ESCALATING
RECOVERING
FAILED
```

## 12.4 Struggle score

Use a transparent feature model first:

```text
struggle_score =
    + 0.18 × normalized_time_without_progress
    + 0.15 × repeated_tool_signature_rate
    + 0.12 × repeated_test_failure_rate
    + 0.10 × context_saturation
    + 0.10 × scope_expansion_churn
    + 0.08 × plan_revision_churn
    + 0.08 × contradiction_growth
    + 0.07 × low_confidence_events
    + 0.06 × behavior_warning_rate
    + 0.06 × quota_exhaustion_risk
    - 0.15 × acceptance_coverage_delta
    - 0.12 × verified_test_progress
    - 0.10 × candidate_quality_delta
```

Weights are policy snapshots and later calibrated. Deterministic critical triggers bypass the aggregate score.

## 12.5 Critical triggers

Immediate intervention or escalation:

- provider reports context limit or cannot accept input;
- profile authentication expires;
- quota reaches hard floor;
- repeated identical command exceeds limit;
- process is alive but structured heartbeat and output progress stop;
- workspace disappears or Git identity changes;
- out-of-scope write;
- attempted forbidden command;
- tests are deleted/disabled to obtain a pass;
- Candidate diff unexpectedly shrinks or explodes;
- model claims completion while contract remains uncovered;
- three or more handoffs on the same atomic step;
- two repair loops repeat the same finding;
- provider session identity mismatches authorized profile;
- external effect remains `OUTCOME_UNKNOWN`;
- destructive action precondition cannot be observed.

## 12.6 Escalation ladder

| Level | Action |
|---|---|
| `L0` | continue current lane |
| `L1` | provide targeted deterministic feedback or missing context |
| `L2` | raise reasoning effort or move M1→M2 |
| `L3` | move M2→M3, preserve Context Capsule and workspace |
| `L4` | add independent critic, tester, or repository scout |
| `L5` | replan/decompose Work Package or start a new Variant |
| `L6` | admit triad fusion, council, or code race |
| `L7` | open Intervention for human/domain decision |
| `L8` | quarantine/fail when policy or safety boundary is implicated |

Escalation is not always “bigger model.” Common remedies include:

- narrower task decomposition;
- missing test reproduction;
- targeted source retrieval;
- fresh session;
- context migration;
- new tool capability;
- clean workspace;
- external dependency wait;
- policy clarification;
- switching to a model with a better task-specific calibration.

## 12.7 Escalation Decision

```text
escalation_decision_id
attempt_id
trigger
struggle_snapshot
current_lane
candidate_actions
selected_action
expected_cost
expected_benefit
quota_effect
context_effect
scope_effect
continuity_plan
policy_version
created_at
```

## 12.8 Automatic task decomposition

A stalled Work Package may be superseded by a Graph Delta that decomposes it into smaller packages when:

- acceptance coverage remains equivalent;
- scope and dependencies are validated;
- no active Effect is ambiguous;
- current work is checkpointed;
- predecessor Attempt is superseded by fence;
- budget/risk changes are authorized.

## 12.9 Completion monitor

Completion is evaluated in layers:

```text
1. Cognitive Task output schema valid?
2. Required source claims supported?
3. Work Package acceptance mapping complete?
4. Actual scope within grant?
5. Workspace clean and classified?
6. Candidate exact and immutable?
7. Required deterministic gates PASS?
8. Required Evidence current?
9. Independent review satisfied?
10. Effects verified?
11. Integration and observation satisfied?
```

An agent “done” event merely requests evaluation.

## 12.10 False-completion defense

Detect:

- no Candidate;
- dirty or untracked workspace;
- branch not preserved;
- tests not run;
- claimed test result lacks Evidence;
- source bead/task closed but integration absent;
- report-only loop;
- skipped workflow steps;
- placeholder/TODO left in required path;
- acceptance items omitted;
- remote branch or PR mismatch;
- target already closed/merged and work is duplicated.

False completion produces a behavior event and either returns targeted repair instructions or escalates.

---

# 13. Canonical Context Graph

Centerrail owns a provider-neutral knowledge structure.

## 13.1 Context node types

```text
MissionObjective
AcceptanceRequirement
PlanDecision
ArchitectureDecision
RepositoryFact
SourceReference
ToolObservation
TestResult
FailureSignature
Assumption
Question
Answer
Risk
ChangeIntent
ScopeGrant
CandidateFact
EvidenceFact
ReviewFinding
HumanSteering
QuotaState
BehaviorEvent
ExternalEffectState
ContinuitySummary
```

Each node includes:

```text
node_id
type
canonical_content
source_refs
created_by
created_at
confidence
validity
supersedes
contradicts
depends_on
visibility
sensitivity
token_estimate
```

Edges preserve:

- provenance;
- supports;
- contradicts;
- supersedes;
- derived-from;
- relevant-to;
- resolves;
- blocks;
- acceptance mapping;
- Candidate binding.

## 13.2 Canonical thread

The canonical thread contains:

- objective and business context;
- Acceptance Contract;
- active Plan and Graph Deltas;
- accepted decisions;
- exact repository facts;
- tool outcomes;
- open questions;
- granted scope;
- Candidate lineage;
- Evidence and review;
- human steering;
- quota and budget state;
- behavior and incident state.

Provider messages are imported as raw artifacts and normalized nodes where useful. They are not the only copy.

## 13.3 Context Capsule

A bounded, versioned view compiled for a Cognitive Task:

```text
capsule_id
context_revision
task_id
provider/model target
objective
contract
acceptance subset
base and Candidate identity
read scope
granted write scope
relevant source facts
accepted decisions
known risks
open questions
required evidence
tool/network policy
quota/budget
continuity summary
compression records
source manifest
token estimate
```

## 13.4 Capsule compiler

```text
1. Determine task information requirements.
2. Select mandatory nodes:
   - objective;
   - acceptance;
   - scope;
   - policy;
   - unresolved blockers;
   - Candidate/Evidence identity.
3. Retrieve relevant facts by exact graph edges, lexical search, symbols, and embeddings.
4. Resolve superseded nodes.
5. Include contradictions explicitly.
6. Order by task utility and provider position sensitivity.
7. Apply provider-specific rendering.
8. Enforce token budget.
9. If over budget, schedule compression tasks.
10. Emit a source manifest and coverage report.
```

The compiler never silently omits mandatory nodes.

## 13.5 Provider renderers

Provider renderers may use:

- structured JSON;
- Markdown;
- provider file references;
- prompt caching blocks;
- system/developer/user message separation;
- provider-native rules or skills;
- token-optimized structured notation.

Rendering is a view. It cannot change canonical semantics.

## 13.6 Context continuity grades

```text
C0: no continuity; fresh task
C1: objective and contract only
C2: facts, decisions, and exact workspace state
C3: full Work Package continuity with unresolved questions
C4: provider-native session resume plus canonical capsule
```

Cross-provider migration normally targets C3. Native resume can add C4 but is never required for correctness.

---

# 14. Verified context compression

Compression is a Cognitive Task, not a side effect hidden inside a provider.

## 14.1 Compression goals

- reduce tokens and latency;
- retain requirements, decisions, risks, blockers, and exact references;
- remove redundant narration and repeated tool output;
- preserve contradictions;
- make loss explicit;
- remain reversible through source links.

## 14.2 Compression artifact

```rust
pub struct CompressionArtifact {
    pub compression_job_id: CompressionJobId,
    pub source_manifest_hash: Digest,
    pub target_task: CognitiveTaskId,
    pub compressor_invocation: InvocationId,
    pub checker_invocation: Option<InvocationId>,
    pub retained_nodes: Vec<ContextNodeId>,
    pub omitted_nodes: Vec<OmittedContext>,
    pub synthesized_nodes: Vec<SynthesizedContext>,
    pub unresolved_contradictions: Vec<ContradictionId>,
    pub decision_coverage: f32,
    pub acceptance_coverage: f32,
    pub source_reference_coverage: f32,
    pub risk_coverage: f32,
    pub open_question_coverage: f32,
    pub compression_ratio: f32,
    pub confidence: f32,
    pub content_hash: Digest,
}
```

## 14.3 Compression classes

### Lossless structural compaction

- deduplicate identical events;
- replace repeated logs with artifact pointers;
- collapse superseded state;
- normalize tables;
- preserve all semantic nodes.

Prefer D0.

### Extractive compression

Select exact source sentences, facts, code references, and test errors. Preferred for high-risk material because it minimizes generative distortion.

### Abstractive compression

Generate a shorter representation. Must retain source back-references and pass coverage checks.

### Task-conditioned compression

Select and summarize information relevant to the next Cognitive Task. A security reviewer and a documentation writer receive different capsules from the same canonical graph.

## 14.4 Default compression protocol

Low-risk:

```text
M1 compressor
→ deterministic schema/coverage validation
→ publish
```

Normal engineering:

```text
M1 compressor
→ independent M1/M2 checker
→ repair or publish
```

High-risk or large migration:

```text
compressor A + compressor B
→ M2/M3 comparison
→ provenance-preserving fuse
→ independent coverage verification
```

## 14.5 Mandatory retained information

Compression cannot drop:

- Mission objective;
- acceptance requirements relevant to task;
- current exact SHA/Candidate;
- current granted scope;
- current policy constraints;
- unresolved contradictions;
- unresolved questions/blockers;
- decisions and rejected alternatives when still relevant;
- test failures and known regressions;
- security/privacy risks;
- active Effect uncertainty;
- human steering;
- continuity actions.

## 14.6 Compression failure states

```text
PASS
INSUFFICIENT_COVERAGE
CONTRADICTION_LOSS
SOURCE_DRIFT
TOO_LOSSY
SCHEMA_INVALID
CHECKER_DISAGREES
UNSUPPORTED
```

A failed compression never replaces the prior capsule.

## 14.7 Context saturation policy

Each provider adapter reports or estimates context use.

Thresholds are configurable by model and task:

```toml
[context.thresholds]
warn = 0.45
prepare_handoff = 0.55
mandatory_transition = 0.65
hard_stop_margin = 0.15
```

Centerrail—not the model—initiates:

1. checkpoint;
2. context graph update;
3. compression/capsule build;
4. new provider session or native continuation;
5. identity verification;
6. continuity validation;
7. old session termination.

A human never needs to type `/compact` or `/clear` for routine operation.

## 14.8 Provider migration

```text
1. Freeze current turn at safe boundary or interrupt.
2. Checkpoint Git, files, process state, and context.
3. Import final provider events.
4. Compile C3 Context Capsule.
5. Select eligible target lane.
6. Reserve target quota/budget.
7. Start fresh target session.
8. Probe effective identity and capabilities.
9. Deliver capsule and continuity instruction.
10. Require structured acknowledgment of:
    - objective;
    - current state;
    - next action;
    - unresolved blockers.
11. Resume tools.
12. Terminate or archive predecessor.
```

Migration records why the switch occurred, what was transferred, what was omitted, and the independence implications.

---

# 15. Quota, capacity, and cost governance

Quota is a vector, not one percentage.

## 15.1 Provider Profile

```text
profile_id
provider
harness
owner_principal
authorization_class
commercial_class
custody_mode
permitted_hosts
permitted_repositories
permitted_use
credential_generation
authentication_state
health_state
concurrency_limit
policy
created_at
```

Authorization classes:

- API credit;
- organization seat;
- service identity;
- named human subscription;
- cloud gateway;
- local model.

## 15.2 Quota dimension

```text
requests
input_tokens
output_tokens
cached_tokens
reasoning_compute
concurrency
model_window
daily_window
weekly_window
monthly_spend
subscription_window
credits
provider-specific unit
```

## 15.3 Quota Observation

```text
quota_observation_id
profile_id
model_snapshot_id
dimension
limit
used
remaining
window_start
window_end
reset_at
observed_at
source
confidence
age
evidence_hash
pricing_version
```

Sources ordered by trust:

1. official provider/admin API;
2. structured harness account/rate-limit method;
3. official CLI status or usage command;
4. response headers;
5. typed limit response;
6. enterprise telemetry export;
7. empirical estimator;
8. unknown.

## 15.4 Current provider opportunities

- Codex app-server exposes account/auth and rate-limit surfaces suitable for structured profile state.
- Cursor Teams/Enterprise APIs expose usage, spending, and model access; enterprise telemetry can supplement near-real-time observations.
- Antigravity exposes model quota and credit views; adapters invoke usage operations outside its NDJSON prompt stream.
- Claude Code provides structured usage/cost events and organizational analytics; real-time remaining quota may be `UNKNOWN` unless an authorized structured source exists.

Centerrail must model differing freshness and confidence rather than pretending every provider offers the same API.

## 15.5 Reservations

Before every invocation:

```text
forecast = estimator(task, model, capsule, effort)
reservation = {
  expected,
  p90,
  hard_cap,
  dimensions,
  expires_at
}
```

Admission checks:

- remaining minus active reservations;
- emergency reserve floor;
- deadline;
- fallback capacity;
- vendor concentration;
- profile ownership;
- organization budget.

After invocation, usage settles the reservation. The residual is released. Overrun creates a forecast-error event.

## 15.6 Quota shadow price

The scheduler computes a scarcity cost:

```text
shadow_price =
    scarcity_curve(remaining / limit)
  + reset_distance_penalty
  + uncertainty_penalty
  + concentration_penalty
  + emergency_reserve_penalty
```

This prevents wasting scarce frontier quota on tasks that an economy lane can handle.

## 15.7 Reserve classes

```text
normal
critical
security
incident
integration_repair
human_interactive
speculative
benchmark
```

Speculative work cannot consume critical reserve.

## 15.8 Rate-limit handling

On a typed 429/throttle:

1. classify dimension and scope;
2. honor retry/reset information;
3. mark lane/profile `THROTTLED`;
4. cancel or pause speculative invocations;
5. checkpoint affected sessions;
6. apply full-jitter exponential backoff;
7. prevent synchronized restarts;
8. migrate only when profile and continuity policy allow;
9. preserve provider-specific session state;
10. settle or extend reservations.

## 15.9 Cost accounting

Every invocation records:

- list-price estimate;
- billed cost when available;
- input/output/cache/reasoning units;
- profile and commercial class;
- runner/CI cost;
- fusion allocation;
- accepted-result attribution;
- wasted/aborted cost;
- shadow cost for scarce quota.

Executive cost metrics use accepted, surviving results—not raw token volume.

---

# 16. Behavior Policy Engine

Centerrail maintains a versioned catalog of undesirable agent behaviors. Each rule is enforceable through commands, filesystem events, Git state, process/network observation, provider events, or deterministic code analysis.

## 16.1 Rule schema

```rust
pub struct BehaviorRule {
    pub rule_id: BehaviorRuleId,
    pub version: SemVer,
    pub category: BehaviorCategory,
    pub title: String,
    pub description: String,
    pub detector: DetectorSpec,
    pub scope: RuleScope,
    pub severity: Severity,
    pub action: EnforcementAction,
    pub remediation: Option<RemediationSpec>,
    pub exceptions: Vec<PolicyException>,
    pub evidence_requirement: EvidenceRequirement,
    pub enabled_by_default: bool,
}
```

Actions:

```text
OBSERVE
WARN
NUDGE
BLOCK_COMMAND
PAUSE_ATTEMPT
REQUEST_SCOPE
AUTO_REMEDIATE
QUARANTINE_CANDIDATE
TERMINATE_SESSION
FAIL_ATTEMPT
OPEN_INTERVENTION
```

## 16.2 Detector channels

- tool gateway command interception;
- filesystem watcher;
- periodic full workspace reconciliation;
- Git status/diff/tree inspection;
- process tree and resource observer;
- network proxy;
- provider structured event stream;
- PTY screen signature fallback;
- test/build result parser;
- Candidate scanner;
- static code analyzers;
- semantic model observer, advisory only;
- external Effect Broker.

## 16.3 Enforcement order

```text
prevent before action
→ detect immediately after unavoidable action
→ pause and checkpoint
→ classify and remediate
→ verify postcondition
→ resume or quarantine
```

The agent cannot acknowledge away a deterministic policy violation.

# 17. Default bad-behavior catalog

The initial catalog below is normative. Organizations may add stricter rules, but may not disable kernel safety rules without changing product policy and accepting an audited risk exception.

| ID | Category | Prohibited or monitored behavior | Detector | Default action |
|---|---|---|---|---|
| `FS001` | workspace | Writes outside assigned workspace | filesystem boundary + mount policy | `BLOCK_COMMAND` |
| `FS002` | workspace | Creates a second repository copy inside the workspace | nested .git / manifest similarity scan | `PAUSE_ATTEMPT` |
| `FS003` | workspace | Creates ad hoc copy directories such as copy, backup, old, new, final2, tmp-repo | path-pattern + tree similarity | `WARN then PAUSE on source duplication` |
| `FS004` | workspace | Writes runtime/provider configuration into product repository | known provider artifact manifest | `AUTO_REMEDIATE + QUARANTINE if committed` |
| `FS005` | workspace | Leaves unclassified untracked files at Candidate preparation | git status + artifact classifier | `BLOCK_FINALIZE` |
| `FS006` | workspace | Uses system temp paths for durable work without checkpoint | open-file/process observation | `WARN then CHECKPOINT` |
| `FS007` | workspace | Writes large binary or archive unexpectedly | size/type/entropy scan | `PAUSE_ATTEMPT` |
| `FS008` | workspace | Creates recursive directory copies or symlink cycles | filesystem graph scan | `BLOCK_COMMAND` |
| `FS009` | workspace | Changes file ownership or broad permissions | command interception + stat diff | `BLOCK_COMMAND` |
| `FS010` | workspace | Deletes repository files outside granted scope | diff-vs-intent | `PAUSE_ATTEMPT` |
| `GT001` | git | Uses Git worktree for writable task | command interception | `BLOCK_COMMAND` |
| `GT002` | git | Modifies .git internals directly | filesystem boundary | `BLOCK_COMMAND` |
| `GT003` | git | Runs reset --hard, clean -fdx, checkout -- ., restore ., or equivalent destructive command without authorized snapshot | command AST | `BLOCK_COMMAND` |
| `GT004` | git | Force pushes or deletes remote refs | tool gateway | `BLOCK_COMMAND` |
| `GT005` | git | Pushes directly from agent sandbox | network/tool policy | `BLOCK_COMMAND` |
| `GT006` | git | Changes remote URL or credential helper | git config diff | `BLOCK_COMMAND` |
| `GT007` | git | Adds provider hook files or Centerrail runtime files to Candidate | Candidate scanner | `QUARANTINE_CANDIDATE` |
| `GT008` | git | Leaves detached-head commits without preservation | Git reconciler | `AUTO_REMEDIATE` |
| `GT009` | git | Claims clean state using commits-ahead rather than exact diff/tree checks | completion evaluator | `REJECT_COMPLETION` |
| `GT010` | git | Rebases or merges target without integration coordinator authorization | command interception | `BLOCK_COMMAND` |
| `SC001` | scope | Writes outside granted Change Intent | watcher + full diff check | `PAUSE_ATTEMPT` |
| `SC002` | scope | Requests repeated broad scope expansions without new evidence | scope request history | `ESCALATE` |
| `SC003` | scope | Touches protected paths without risk upgrade | path policy | `BLOCK_COMMAND` |
| `SC004` | scope | Modifies lockfiles without declared dependency intent | diff classifier | `PAUSE_ATTEMPT` |
| `SC005` | scope | Modifies generated files without generator evidence | generated-file manifest | `BLOCK_FINALIZE` |
| `SC006` | scope | Changes CI, security, auth, migration, or deployment files while classified low risk | path-risk policy | `RECLASSIFY + PAUSE` |
| `TL001` | tools | Runs sudo, su, system package manager, Docker socket, or host service command | command interception | `BLOCK_COMMAND` |
| `TL002` | tools | Downloads or executes unpinned binary | network + command policy | `BLOCK_COMMAND` |
| `TL003` | tools | Uses curl | shell or equivalent remote execution | command AST | `BLOCK_COMMAND` |
| `TL004` | tools | Spawns unbounded background process | process supervisor | `TERMINATE_PROCESS` |
| `TL005` | tools | Starts unmanaged server or listener | socket observer | `PAUSE_ATTEMPT` |
| `TL006` | tools | Repeats identical command beyond threshold | tool signature counter | `NUDGE then ESCALATE` |
| `TL007` | tools | Runs broad repository scan repeatedly without new result | command/result hash | `NUDGE` |
| `TL008` | tools | Ignores timeout/cancellation | process supervisor | `TERMINATE_PROCESS` |
| `TL009` | tools | Attempts unsupported MCP/tool without grant | tool gateway | `BLOCK_COMMAND` |
| `TL010` | tools | Uses shell to bypass typed Centerrail operation | command policy | `BLOCK_COMMAND` |
| `NW001` | network | Connects to undeclared host | egress proxy | `BLOCK_CONNECTION` |
| `NW002` | network | Attempts cloud metadata or local network discovery | egress proxy | `BLOCK_CONNECTION + SECURITY_EVENT` |
| `NW003` | network | Exfiltrates repository content in URL/body to unapproved destination | DLP proxy | `BLOCK_CONNECTION + QUARANTINE` |
| `NW004` | network | Runs a tunnel, proxy, reverse shell, or port-forward | process/network detector | `TERMINATE_SESSION` |
| `NW005` | network | Uses personal SCM credential or SSH agent | environment/socket policy | `BLOCK_COMMAND` |
| `SE001` | secrets | Reads secret outside explicit grant | filesystem/secret broker | `BLOCK_COMMAND` |
| `SE002` | secrets | Writes secret or high-entropy credential to repository | secret scan | `QUARANTINE_CANDIDATE` |
| `SE003` | secrets | Prints secret into model context or logs | DLP redactor | `REDACT + SECURITY_EVENT` |
| `SE004` | secrets | Attempts to enumerate credential stores | command policy | `TERMINATE_SESSION` |
| `CD001` | code | Deletes or weakens tests to make suite pass | diff + test manifest + semantic check | `QUARANTINE_CANDIDATE` |
| `CD002` | code | Adds skip/ignore/only/focus markers without explicit intent | static analyzer | `BLOCK_FINALIZE` |
| `CD003` | code | Swallows errors or broadens catch to hide failure | semantic lint | `REVIEW_REQUIRED` |
| `CD004` | code | Returns hardcoded success or fake fixture in production path | semantic detector + hidden tests | `QUARANTINE_CANDIDATE` |
| `CD005` | code | Adds TODO/FIXME/placeholder in acceptance-critical path | static scan | `BLOCK_FINALIZE` |
| `CD006` | code | Duplicates large code or vendors source instead of integrating properly | similarity scan | `REVIEW_REQUIRED` |
| `CD007` | code | Introduces unnecessary dependency | dependency diff + rationale requirement | `PAUSE_FINALIZE` |
| `CD008` | code | Changes public API without compatibility evidence | API diff | `BLOCK_FINALIZE` |
| `CD009` | code | Changes schema without migration/rollback evidence | schema diff | `BLOCK_FINALIZE` |
| `CD010` | code | Introduces nondeterministic test or sleep-based synchronization | test analyzer | `REVIEW_REQUIRED` |
| `TS001` | testing | Claims test pass without captured command Evidence | completion evaluator | `REJECT_COMPLETION` |
| `TS002` | testing | Runs only a narrower test after touching broader surface without justified selection | test impact analyzer | `BLOCK_FINALIZE` |
| `TS003` | testing | Ignores flaky, timed-out, cancelled, or infra-error result as pass | typed gate evaluator | `BLOCK_FINALIZE` |
| `TS004` | testing | Modifies expected output instead of fixing behavior without rationale | diff/test correlation | `REVIEW_REQUIRED` |
| `TS005` | testing | Skips required browser/accessibility/security test | acceptance mapping | `BLOCK_FINALIZE` |
| `CP001` | completion | Emits done with dirty workspace | Git reconciler | `REJECT_COMPLETION` |
| `CP002` | completion | Emits done without exact Candidate | Candidate service | `REJECT_COMPLETION` |
| `CP003` | completion | Emits done with uncovered acceptance requirement | acceptance evaluator | `REJECT_COMPLETION` |
| `CP004` | completion | Closes task while external effect is unknown | Effect service | `REJECT_COMPLETION` |
| `CP005` | completion | Reports workflow step audit without executing steps | step receipts | `FAIL_ATTEMPT` |
| `CP006` | completion | Restarts onto a closed/already-integrated target | target-state preflight | `BLOCK_START` |
| `CX001` | context | Exceeds context transition threshold without checkpoint | context monitor | `FORCE_TRANSITION` |
| `CX002` | context | Repeatedly compresses or hands off same step without progress | context/progress history | `ESCALATE_REPLAN` |
| `CX003` | context | Drops unresolved decision, blocker, or scope from capsule | capsule coverage validator | `REJECT_CAPSULE` |
| `CX004` | context | Uses provider-native history as sole durable record | continuity validator | `BLOCK_TRANSITION` |
| `AG001` | agent | Repeatedly states confidence without evidence | event analyzer | `NUDGE` |
| `AG002` | agent | Fabricates source reference, test, or tool result | provenance validator | `FAIL_COGNITIVE_TASK` |
| `AG003` | agent | Ignores explicit interruption or cancellation | session supervisor | `TERMINATE_SESSION` |
| `AG004` | agent | Oscillates between plans without new evidence | artifact hash/history | `ESCALATE` |
| `AG005` | agent | Performs report-only activity instead of required steps | workflow receipts | `FAIL_ATTEMPT` |
| `AG006` | agent | Attempts to lower risk, waive checks, or expand permissions in prose | policy parser | `IGNORE + SECURITY_EVENT` |
| `AG007` | agent | Edits runtime status or audit files | filesystem boundary | `BLOCK_COMMAND` |
| `EF001` | effects | Attempts direct GitHub/cloud/package/deploy mutation | network/tool gateway | `BLOCK_COMMAND` |
| `EF002` | effects | Retries ambiguous external effect blindly | Effect state machine | `BLOCK_RETRY` |
| `EF003` | effects | Uses stale fence for external mutation | Effect Broker | `REJECT_EFFECT` |
| `CL001` | cleanup | Deletes workspace before verified preservation | cleanup service | `BLOCK_DELETE` |
| `CL002` | cleanup | Treats failed observation as empty/clean | three/four-valued observation guard | `BLOCK_ACTION` |
| `CL003` | cleanup | Cleanup targets resources by shared task ID rather than owned attempt identity | ownership validator | `BLOCK_ACTION` |
| `CL004` | cleanup | Reuses name/path before tombstoned cleanup completes | resource tombstone | `BLOCK_ALLOCATION` |

## 17.1 Behavior event

```text
behavior_event_id
rule_id
rule_version
organization_id
repository_id
mission_id
work_package_id
variant_id
attempt_id
invocation_id
timestamp
detector
observed_action
evidence_artifacts
severity
enforcement_action
postcondition
resolution
```

## 17.2 Real-time workspace hygiene

The runner maintains a `WorkspaceManifest` containing:

```text
base commit
current head
tracked modifications
untracked files
ignored runtime files
ephemeral files
generated files
dependency files
binary files
open file handles
running processes and cwd
nested repositories
symlinks
mounts
network listeners
```

It updates incrementally from filesystem events and periodically performs a complete reconciliation. Filesystem watchers are advisory; Candidate preparation uses a fresh authoritative scan.

## 17.3 Temporary work policy

Allowed:

```text
$CENTERRAIL_ATTEMPT_TMP/
/tmp/centerrail/{attempt_id}/
.centerrail-runtime/ mounted outside repository
declared build output directories
declared tool caches
```

Disallowed by default:

- repository copies under arbitrary temp names;
- backup directories inside source;
- runtime provider settings in the product tree;
- generated logs, transcripts, screenshots, or credentials in source;
- unexplained archive files;
- “final”, “new”, “old”, “copy”, or numbered duplicate trees;
- durable work existing only in `/tmp`.

An agent may request an artifact classification. Approved artifacts move to content-addressed storage and receive a manifest pointer.

## 17.4 Remediation

Auto-remediation must be deterministic and narrow. Examples:

- move known runtime files outside repo;
- add an entry to `.git/info/exclude`, not product `.gitignore`, when local-only;
- checkpoint dirty work before context transition;
- terminate an unmanaged child process;
- remove an empty generated temp directory;
- restore a file modified by an explicitly failed deterministic transform.

Auto-remediation never silently changes product code, discards uncommitted work, rewrites history, or deletes a branch.

## 17.5 Behavior score

A per-Attempt behavior score may summarize risk for routing and operator triage, but individual rules remain authoritative. A high aggregate score cannot invent a violation, and a low score cannot waive one.

# 18. Structured Session Supervisor and PTY/TTY compatibility plane

The correct terms are:

- **TTY:** terminal device abstraction;
- **PTY:** pseudo-terminal pair on Unix-like systems;
- **ConPTY:** Windows pseudoconsole API;
- **terminal screen model:** parsed virtual screen state;
- **provider session:** logical model conversation;
- **process session:** owned process tree;
- **agent session:** Centerrail binding between provider conversation, Attempt, context, and process.

Centerrail supports live streaming and interrupts without making terminal text authoritative.

## 18.1 Session architecture

```text
AgentSession
├── provider session identity
├── structured adapter transport
├── process tree
├── PTY/ConPTY endpoint when required
├── parsed terminal screen
├── raw terminal frame log
├── dialog controller
├── context monitor
├── progress monitor
├── quota monitor
├── behavior monitor
└── Attempt Authority Token
```

## 18.2 Session state machine

```text
CREATED
→ STARTING
→ IDENTITY_PROBING
→ CONTEXT_LOADING
→ READY
→ RUNNING
↔ WAITING_TOOL
↔ WAITING_LOCAL_PLAN
↔ WAITING_PERMISSION
↔ PAUSED
→ CHECKPOINTING
→ CONTEXT_TRANSITION
→ RESUMING
→ COMPLETING
→ TERMINATING
→ TERMINATED
```

Exception states:

```text
AUTH_REQUIRED
THROTTLED
CONTEXT_AT_RISK
UNRESPONSIVE
PROTOCOL_ERROR
SCREEN_UNKNOWN
PROCESS_UNKNOWN
POLICY_BLOCKED
QUARANTINED
CRASHED
```

No blocked state may persist without:

- a typed reason;
- last observation;
- confidence;
- next scheduled action;
- escalation deadline.

## 18.3 Structured event envelope

```rust
pub struct AgentEvent {
    pub event_id: EventId,
    pub session_id: AgentSessionId,
    pub invocation_id: Option<InvocationId>,
    pub native_session_id: Option<String>,
    pub provider: ProviderId,
    pub model: Option<ModelId>,
    pub kind: AgentEventKind,
    pub timestamp: DateTime<Utc>,
    pub sequence: u64,
    pub causation_id: Option<EventId>,
    pub payload: AgentEventPayload,
    pub raw_artifact: Option<ArtifactRef>,
}
```

Kinds include:

```text
session.started
session.ready
session.identity
turn.started
turn.delta
turn.completed
turn.failed
thinking.delta
tool.requested
tool.started
tool.completed
tool.failed
permission.requested
plan.proposed
plan.waiting
usage.reported
quota.reported
context.reported
auth.required
rate_limited
steering.acknowledged
interrupt.acknowledged
checkpoint.completed
session.compacted
session.terminated
protocol.error
```

## 18.4 PTY mirror

When a provider requires an interactive terminal:

### Unix

- create PTY with `portable-pty` or a narrow platform crate;
- spawn process in its own session/process group;
- record process group and pidfd where available;
- parse bytes using a terminal parser such as `vte`;
- maintain screen grid, cursor, modes, title, and alternate-screen state;
- stream frames to portal;
- preserve raw bytes in bounded artifacts.

### Windows

- create ConPTY through Win32 bindings;
- attach process to a Job Object;
- capture pseudoconsole stream;
- maintain equivalent screen model;
- terminate entire Job Object;
- normalize Windows short/long paths and quoting.

## 18.5 Authority rule

PTY data can support:

- live viewing;
- compatibility dialog recognition;
- forensic evidence;
- fallback progress hints;
- operator attachment;
- terminal input for explicitly authorized manual debugging.

PTY data alone cannot establish:

- successful tool execution;
- completed model turn;
- authenticated profile;
- exact quota;
- safe idleness;
- accepted plan;
- Candidate completion;
- process liveness when contradictory structured/process evidence exists.

## 18.6 Dialog Controller

The Dialog Controller automates routine provider UI states.

```rust
pub trait DialogRecognizer {
    fn recognize(&self, structured: &SessionFacts, screen: &Screen) -> DialogObservation;
}

pub trait DialogAction {
    fn preconditions(&self) -> Vec<Predicate>;
    async fn execute(&self, session: &SessionHandle) -> Result<ActionReceipt>;
    async fn verify(&self, session: &SessionHandle) -> Result<Postcondition>;
}
```

Dialog types:

```text
LOCAL_PLAN_APPROVAL
TOOL_PERMISSION
EDIT_PERMISSION
CONTEXT_LIMIT
COMPACTION_PROMPT
AUTH_LOGIN
RATE_LIMIT
SPEND_LIMIT
TRUST_WORKSPACE
UPDATE_AVAILABLE
CRASH_DIALOG
CHOICE_MENU
UNSUPPORTED
```

## 18.7 Plan approval distinction

There are two different approvals:

1. **Provider-local plan acceptance**  
   A CLI may ask whether to leave plan mode or begin editing. Centerrail automatically approves only after:
   - the plan artifact was captured;
   - the plan maps to the active Work Package;
   - predicted scope is compatible;
   - task risk permits autonomous execution;
   - no Centerrail Intervention is required.

2. **Centerrail policy or business approval**  
   R3/R4 work, irreversible effects, acceptance changes, or domain decisions may require an authorized human. A local CLI prompt cannot satisfy or waive this.

Thus no human should have to visit a terminal to click “approve plan,” while genuine governance remains intact.

## 18.8 Permission automation

Provider tool/permission prompts are answered from Tool Gateway policy:

```text
ALLOW_ONCE
ALLOW_SESSION
DENY
ESCALATE
```

The model cannot expand the policy through prose. A denied request yields a structured explanation and alternative.

## 18.9 Context automation

On context threshold or provider prompt:

1. stop new model turns;
2. allow current tool call to reach a safe boundary or interrupt;
3. checkpoint workspace;
4. import final events;
5. build verified Context Capsule;
6. choose native compact, native resume, or fresh session;
7. deliver continuity capsule;
8. verify acknowledgment;
9. archive predecessor.

Humans do not type `/compact` or `/clear` for routine cycles.

## 18.10 Interrupt and steering

Steering messages have:

```text
steering_id
session_id
attempt_id
fence
source
priority
message
delivery_mode
expires_at
acknowledged_at
applied_at
result
```

Delivery modes:

- immediate structured steer;
- next safe boundary;
- interrupt then steer;
- cancel turn;
- pause tools;
- terminate.

An acknowledgment is distinct from application. The portal shows both.

## 18.11 Hang detection

A session is not “hung” merely because it is quiet.

Observation vector:

```text
structured connection
provider turn state
process alive
child process state
CPU
I/O
network activity
tool gateway activity
workspace changes
terminal screen changes
last meaningful progress
quota/auth/context state
```

Classification:

```text
ALIVE_PROGRESSING
ALIVE_WAITING_VALID
ALIVE_BUSY_TOOL
ALIVE_NO_PROGRESS
UNRESPONSIVE
DEAD
UNKNOWN
CONTRADICTORY
```

Policy uses multiple converging signals. Unknown never authorizes destructive cleanup.

## 18.12 Process ownership

Every session process tree is placed in:

- Linux cgroup and process group;
- macOS process group plus tracked descendants, preferably within a VM;
- Windows Job Object.

Termination:

1. structured cancel;
2. graceful signal/control event;
3. bounded grace;
4. force entire owned group;
5. verify no owned processes remain;
6. record receipt.

Do not recursively discover children by racing one subprocess call per PID.

## 18.13 Live portal terminal

Portal features:

- read-only live screen;
- raw log and normalized events;
- scrollback artifact;
- exact process and provider identity;
- context/quota meters;
- tool request queue;
- interrupt/pause/terminate commands;
- optional audited manual attach;
- screen-recognition confidence;
- explicit structured-vs-terminal source labels.

Manual terminal input is disabled by default and cannot change Centerrail scope or policy.

---

# 19. Cross-platform runner and sandbox design

## 19.1 Security-grade matrix

| Platform | Local execution | Strong isolation | PTY | Process ownership | Production recommendation |
|---|---|---|---|---|---|
| Linux | full | rootless namespaces, Landlock/seccomp/cgroups; microVM for S2 | PTY | cgroup + pidfd/process group | reference |
| macOS | supported | VM-backed for meaningful strong isolation | PTY | process group; VM boundary preferred | remote Linux or VM for R2+ |
| Windows | supported | AppContainer/Windows Sandbox/Hyper-V class | ConPTY | Job Object | Hyper-V/remote Linux for R2+ |
| WSL | supported as Linux environment | depends on WSL boundary and host policy | PTY | Linux process group/cgroup limits | acceptable for dev; grade explicitly |

Centerrail never labels process-only macOS or Windows execution as equivalent to hardened Linux or a separate-kernel VM.

## 19.2 Sandbox classes

| Class | Boundary | Typical use |
|---|---|---|
| `S0` | read-only process, no write authority | scouts, classification |
| `S1` | rootless OS sandbox/container, limits, network policy | normal R1/R2 work |
| `S2` | separate-kernel microVM or equivalent | R3, hidden evaluation, untrusted repos |
| `S3` | dedicated/attested environment | exceptional R4/regulatory |

## 19.3 Linux S1

- user and mount namespaces;
- cgroup v2 CPU/memory/pids/I/O;
- seccomp;
- Landlock where available;
- read-only base image;
- private writable workspace;
- isolated `/tmp`, HOME, XDG;
- no host `/proc` beyond namespace;
- no Docker socket;
- no SSH agent;
- egress proxy;
- DNS policy;
- immutable toolchain manifest;
- controlled package caches.

## 19.4 Linux S2

Use Firecracker, Cloud Hypervisor, Kata, or equivalent separate-kernel boundary. The exact backend is pluggable, but the sandbox contract is stable:

```rust
pub trait SandboxBackend {
    async fn create(&self, spec: SandboxSpec) -> Result<SandboxHandle>;
    async fn exec(&self, handle: &SandboxHandle, req: ExecRequest) -> Result<ExecHandle>;
    async fn snapshot(&self, handle: &SandboxHandle) -> Result<SnapshotRef>;
    async fn terminate(&self, handle: &SandboxHandle) -> Result<TerminationReceipt>;
    async fn attest(&self, handle: &SandboxHandle) -> Result<Attestation>;
}
```

## 19.5 macOS

Local process mode is `S0` or `S1-lite` and visibly labeled.

For strong isolation:

- create Linux VM through an approved virtualization backend;
- mount/copy the private clone into VM;
- proxy provider transport separately from repository tools;
- use host runner only as supervisor;
- no host credential directories mounted.

## 19.6 Windows

- ConPTY for interactive harness;
- Job Object limits and tree termination;
- restricted token/AppContainer where feasible;
- Windows Defender/Application Control integration;
- Hyper-V or Windows Sandbox backend for higher grades;
- canonicalize case, drive, UNC, junction, reparse-point, and 8.3 aliases;
- block NTFS junction escape;
- isolated user profile and credential manager access.

## 19.7 Provider Harness Enclave versus repository sandbox

Provider credentials and the provider process do not need to share all repository tool privileges.

Preferred split:

```text
Provider Harness Enclave
  - authorized provider credential
  - structured session transport
  - no SCM/production credential
  - no arbitrary network
       │ typed tool requests
       ▼
Repository Tool Sandbox
  - source and build tools
  - no provider credential
  - no SCM/production/cloud-admin credential
       │ proposed external effect
       ▼
Effect Broker
  - short-lived narrow credential
  - no model execution
```

When a vendor CLI cannot be separated from tools, the runner still removes all unrelated credentials and gates every command/network action.

---

# 20. Private-clone Git and workspace control

## 20.1 Layout

```text
mirror:     /var/lib/centerrail/mirrors/{repository_id}.git
workspace:  /var/lib/centerrail/work/{attempt_id}/repo
runtime:    /var/lib/centerrail/runtime/{attempt_id}/
artifacts:  /var/lib/centerrail/spool/{attempt_id}/
branch:     centerrail/{variant_id}/{attempt_id}
```

## 20.2 Clone creation

```text
1. Update mirror under repository lock.
2. Verify requested base SHA exists.
3. Clone without checkout.
4. Share immutable objects only.
5. Dissociate or pin objects before mirror GC.
6. Disable inherited Git config.
7. Checkout exact detached base.
8. Create private branch.
9. Apply repository-specific safe config.
10. record Workspace Manifest and nonce.
```

## 20.3 Hostile Git controls

Set isolated:

```text
HOME
XDG_CONFIG_HOME
XDG_CACHE_HOME
GIT_CONFIG_NOSYSTEM=1
GIT_CONFIG_GLOBAL=/dev/null
GIT_TERMINAL_PROMPT=0
GIT_ASKPASS=<denied helper>
GIT_SSH_COMMAND=<controlled>
```

Disable/control:

- hooks;
- credential helpers;
- `include` / `includeIf`;
- clean/smudge filters;
- external diff/merge drivers;
- pager/editor;
- arbitrary file protocol;
- unsafe submodule URLs;
- LFS endpoints;
- repository aliases;
- environment-injected Git options.

## 20.4 Diff intelligence

The runner exposes typed tools:

```text
workspace.status
workspace.diff
workspace.diff_stat
workspace.changed_symbols
workspace.untracked
workspace.intent_coverage
workspace.generated_drift
workspace.test_impact
workspace.checkpoint
```

Agents receive diffs and exact source references rather than repeatedly reconstructing state.

## 20.5 Change Intent resources

Resources include:

- file;
- normalized directory prefix;
- package/crate/module;
- lockfile;
- schema/migration lane;
- generated artifact family;
- API surface;
- deployment manifest;
- localization catalog;
- security boundary;
- external effect target.

## 20.6 Dynamic scope: successor Attempt

```text
discover required resource
→ stop mutation
→ request typed amendment
→ freeze provider/tools
→ authoritative full write-set scan
→ checkpoint
→ conflict/policy/risk/budget evaluation
→ increment permanent fence
→ atomically transfer old + new grants
→ predecessor SUPERSEDED
→ rebuild sandbox
→ resume from canonical Context Capsule
```

The running process never gains surprise authority.

## 20.7 Candidate preparation

1. stop model mutation;
2. settle tool calls;
3. full workspace scan;
4. classify every untracked/ignored file;
5. verify actual scope;
6. verify runtime artifact exclusion;
7. create checkpoint;
8. create local commit with controlled identity;
9. compute base/head/tree/patch hashes;
10. upload artifact manifest;
11. run writer gates;
12. mark Candidate `PREPARED`.

## 20.8 Cleanup

Workspace deletion requires:

```text
Attempt terminal
+ no open file/process
+ preservation artifact verified
+ Candidate/checkpoint retained per policy
+ no ambiguous Effect
+ cleanup target matches workspace nonce
+ deletion receipt
```

Name/path reuse waits for cleanup tombstone completion.

---

# 21. Tool Gateway and MCP governance

## 21.1 Tool request

```text
tool_request_id
session_id
invocation_id
attempt_id
fence
tool_id
operation
arguments_hash
declared_purpose
required_scope
network_destinations
risk
timeout
```

## 21.2 Policy decision

```text
ALLOW
ALLOW_WITH_REWRITE
DENY
REQUIRE_SCOPE_AMENDMENT
REQUIRE_INTERVENTION
```

## 21.3 Command parsing

Shell commands are parsed into an AST where possible. Policy does not rely only on substring matching.

The gateway validates:

- executable identity;
- arguments;
- working directory;
- environment;
- redirections;
- pipelines;
- backgrounding;
- shell expansion;
- network destinations;
- expected outputs;
- timeout;
- resource limits.

## 21.4 MCP registry

Every MCP/tool server has:

```text
tool_server_id
publisher
version
binary/container digest
transport
capabilities
data destinations
credential needs
network needs
repository permissions
risk class
review status
```

Unknown arbitrary MCP is disabled. Tool servers run in their own sandbox or process boundary and never inherit all agent credentials.

## 21.5 Tool receipts

Every tool execution records:

- normalized request;
- policy decision;
- command/environment digest;
- start/end;
- exit status;
- typed outcome;
- stdout/stderr artifact;
- filesystem delta;
- network observations;
- resource use;
- cancellation;
- postcondition.

---

# 22. Independent verification, review, and regression control

The prior Centerrail proof architecture remains mandatory.

## 22.1 Evidence tiers

| Tier | Meaning | Can satisfy |
|---|---|---|
| E0 | model/author assertion | progress and hypotheses |
| E1 | deterministic writer-sandbox result | fast feedback, low-risk support |
| E2 | clean independent verifier | normal integration proof |
| E3 | protected hidden evaluator, merge-group CI, external security/performance system | high-confidence protected proof |
| E4 | authorized human/domain approval | business judgment and irreversible acceptance |

## 22.2 Verifier isolation

Verifier has:

- different runtime identity;
- separate credentials;
- separate caches;
- hidden-test custody;
- clean source reconstruction;
- independent environment manifest;
- no author transcript by default;
- no merge credential;
- no writable access to author workspace.

## 22.3 Gate outcomes

```text
PASS
FAIL
FLAKY
INFRA_ERROR
CANCELLED
TIMED_OUT
NOT_RUN
UNSUPPORTED
UNKNOWN
SUPERSEDED
INVALIDATED
```

Only an acceptable `PASS` satisfies readiness.

## 22.4 Verification flow

```text
test architect proposes failure cases
→ visible tests and holdouts freeze
→ writer produces exact Candidate
→ mutation stops
→ clean verifier reconstructs
→ deterministic gates
→ protected holdouts/history
→ blind independent semantic review
→ bounded repair creates new Candidate
→ affected Evidence invalidates
→ merge-group verification
→ protected integration
→ observation window
```

## 22.5 Review independence

Score dimensions:

- not author Attempt/session;
- different provider family;
- different model lineage;
- different harness/adapter;
- different prompt/context lineage;
- different tool/evaluator custody;
- no exposure to author confidence/rationale;
- no prior reviewer conclusion exposure.

Policy sets a minimum score by risk.

## 22.6 Hidden-test contamination

When a protected failure is revealed to a writer, that Candidate lineage is marked contaminated. If unbiased evaluation remains required, rotate the holdout or use an independently maintained evaluator.

## 22.7 Repair loops

- findings are typed and deduplicated;
- each loop must add Evidence or change Candidate;
- default maximum two loops;
- repeated identical finding without change is thrash;
- escalation can replan, raise model tier, add critic, or open Intervention.

---

# 23. Universal Effect Broker

Internal authorization and external physical mutation are separate transactions.

## 23.1 State machine

```text
PROPOSED
→ AUTHORIZED
→ DISPATCHING
→ RECEIPT_PENDING
→ VERIFIED
→ COMMITTED
```

Exceptions:

```text
OUTCOME_UNKNOWN
QUARANTINED
FAILED
COMPENSATION_PENDING
COMPENSATING
COMPENSATED
ORPHANED_REMOTE
```

## 23.2 Timeout rule

After dispatch, timeout becomes `OUTCOME_UNKNOWN`.

The broker:

1. reads authoritative remote state;
2. correlates request identity, logical effect key, target, precondition, and desired hash;
3. adopts original effect if it occurred;
4. retries only if non-execution or safe idempotence is established;
5. otherwise quarantines or opens Intervention.

## 23.3 GitHub v1

1. reconstruct Candidate in trusted checkout;
2. validate current fence and Selection;
3. validate lineage, scope, policy, and Evidence;
4. scan delivered tree;
5. mint narrow GitHub App token;
6. push immutable branch with expected-old-OID precondition;
7. read ref back;
8. create/update PR idempotently;
9. bind checks to exact SHA;
10. record receipt;
11. use protected merge queue;
12. verify resulting target commit.

No SCM credential enters the model or repository sandbox.

---

# 24. State machines

## 24.1 Work Package

```text
PROPOSED
→ ADMITTED
→ BLOCKED ↔ READY
→ LEASED
→ EXECUTING
→ PREPARING
→ VERIFYING
→ REVIEWING
→ INTEGRATION_READY
→ INTEGRATING
→ INTEGRATED
→ OBSERVING
→ SURVIVED
```

Exceptions:

```text
STRUGGLING
ESCALATING
QUARANTINED
CANCELLED
FAILED
REVERTED
```

## 24.2 Attempt

```text
CREATED
→ STARTING
→ RUNNING
↔ PAUSED
→ CHECKPOINTING
→ PREPARING
→ SUCCEEDED
```

Exceptions:

```text
SUPERSEDED
FAILED
CRASHED
CANCELLED
QUARANTINED
```

## 24.3 Cognitive Task

```text
CREATED
→ CLASSIFYING
→ ROUTING
→ RESERVED
→ RUNNING
→ VALIDATING
→ COMPLETE
```

Exceptions:

```text
NEEDS_CONTEXT
STRUGGLING
ESCALATING
FUSING
UNSUPPORTED
FAILED
CANCELLED
```

## 24.4 Collaboration Run

```text
PLANNED
→ CONTRIBUTORS_RUNNING
→ CANDIDATES_READY
→ RANKING
→ FUSING
→ VALIDATING
→ COMPLETE
```

Exceptions:

```text
SELECTED_WITHOUT_FUSION
KEEP_MULTIPLE
UNRESOLVED
BUDGET_EXHAUSTED
FAILED
```

## 24.5 Context Transition

```text
REQUESTED
→ CHECKPOINTING
→ GRAPH_UPDATING
→ COMPRESSING
→ STARTING_SUCCESSOR
→ VERIFYING_CONTINUITY
→ COMPLETE
```

Exceptions:

```text
COMPRESSION_REJECTED
SUCCESSOR_AUTH_FAILED
CONTINUITY_FAILED
ROLLBACK_TO_PREDECESSOR
```

## 24.6 Behavior Event

```text
OBSERVED
→ ENFORCING
→ REMEDIATING
→ VERIFYING
→ RESOLVED
```

Exceptions:

```text
QUARANTINED
INTERVENTION_REQUIRED
FAILED_ATTEMPT
```

---

# 25. Portal: complete engineering operations system

Every view displays:

```text
organization/repository/Mission
as_of_sequence
projection timestamp and lag
exact base/Candidate/target SHAs
observation source and confidence
policy/config/routing versions
```

Status vocabulary:

```text
PENDING
CONFIRMED
FAILED
UNKNOWN
STALE
CONTRADICTORY
```

## 25.1 Control Tower

Answers:

- what verified work landed;
- what survived;
- autonomous acceptance and human intervention;
- model and runner cost;
- economy-model share and quality floor;
- current quota risks;
- tasks struggling or escalating;
- fusion yield;
- behavior violations;
- integration dwell;
- control-plane health.

## 25.2 Mission Graph

Displays:

- Acceptance Contract;
- Plan Revisions and Graph Deltas;
- Work Packages;
- Selection Groups and Variants;
- dependencies and critical path;
- Attempts and fences;
- Cognitive Tasks and collaboration protocols;
- context lineage;
- Candidates;
- Evidence;
- reviews;
- Effects;
- integration;
- observation.

## 25.3 Cognitive Router

For every dispatch:

- task taxonomy;
- hard exclusions;
- eligible lanes;
- expected quality/cost/latency;
- quota shadow price;
- chosen tier;
- fallback ladder;
- economy-share impact;
- calibration confidence;
- historical comparisons;
- shadow-routing outcomes.

## 25.4 Fusion Lab

Shows:

- protocol;
- contributor lanes and diversity;
- independent artifacts;
- agreements/disagreements;
- ranker scores;
- fuser provenance;
- residual uncertainty;
- cost/latency;
- hidden-evaluation result;
- whether fusion beat selection or single-lane baseline.

## 25.5 Fleet

Shows:

- provider/model/harness/profile;
- runner/platform/sandbox;
- task and phase;
- Authority Token;
- quota and budget reservations;
- context use;
- last meaningful progress;
- struggle state;
- process and protocol state;
- behavior warnings;
- terminal availability.

## 25.6 Live Attempt

Unified causal timeline:

- model turns;
- context capsules;
- tool calls;
- file changes;
- Git diffs;
- scope requests;
- tests;
- checkpoints;
- quota;
- struggle evaluations;
- escalations;
- behavior events;
- human steering;
- Candidate preparation;
- reviews;
- Effects.

Tabs:

```text
Timeline
Terminal
Structured Events
Diff
Files
Processes
Network
Browser
Tests
Evidence
Context
Behavior
Quota
Raw Artifacts
```

## 25.7 Session Supervisor

- current structured state;
- process tree;
- PTY/ConPTY screen;
- dialog recognition;
- pending permissions/plans;
- context transition;
- steering queue and acknowledgment;
- interrupt/pause/terminate;
- profile identity;
- model/effort;
- last progress.

## 25.8 Context Lineage

- canonical nodes and edges;
- source references;
- active capsule;
- compression jobs;
- retained/omitted content;
- coverage scores;
- provider migrations;
- contradictions;
- continuity acknowledgments;
- original artifacts.

## 25.9 Quota and Capacity

Matrix by:

- profile owner;
- provider/model;
- authorization and custody;
- dimensions;
- limit/used/remaining;
- confidence and age;
- resets;
- reservations;
- emergency reserve;
- current tasks;
- cost;
- circuit breakers.

## 25.10 Struggle and Escalation Cockpit

- tasks by struggle state;
- score components;
- progress timeline;
- repeated signatures;
- context/quota/scope state;
- recommended escalation;
- expected cost/benefit;
- current escalation ladder;
- interventions.

## 25.11 Behavior Center

- rule catalog;
- live events;
- affected workspaces;
- enforcement;
- remediation receipts;
- recurring patterns by model/provider/repository;
- false-positive review;
- policy exceptions;
- Candidate quarantines.

## 25.12 Workspace and Git Hygiene

- exact branch/base/head;
- dirty/untracked/ignored files;
- classified temp/runtime files;
- nested repos;
- large binaries;
- intent coverage;
- process cwd/open files;
- preservation/checkpoint state;
- cleanup eligibility.

## 25.13 Merge Rail

```text
Preparing
Verifying
Reviewing
Changes Requested
Selection
Integration Ready
Rebase/Conflict
Merge Queue
Merge-Group Verification
Integrated
Observation
Survived
Reverted
```

## 25.14 Quality Lab

- accepted survival by task/model/provider;
- economy-share quality;
- router calibration/regret;
- fusion uplift;
- reviewer effectiveness;
- escaped defects;
- test effectiveness;
- repair thrash;
- context compression quality;
- migration continuity;
- behavior incidence;
- adapter regressions;
- benchmark replay.

## 25.15 Incidents and Audit

Reconstructs:

```text
Mission
→ command
→ policy/routing decision
→ Attempt/fence
→ session/profile
→ context
→ tool/workspace behavior
→ Candidate/Evidence/review
→ Effect
→ GitHub state
→ human action
```

## 25.16 Pending command UX

A portal click creates a durable command and displays “requested.” It becomes confirmed or failed only when the ledger records the result. The portal never optimistically changes authoritative state.

# 26. Storage and transactional kernel

## 26.1 Canonical table groups

### Identity and configuration

```text
organizations
principals
role_bindings
repositories
repository_policies
configuration_generations
policy_versions
prompt_template_versions
model_snapshots
adapter_snapshots
environment_images
evaluator_suites
```

### Mission and work graph

```text
missions
acceptance_contracts
acceptance_requirements
plan_revisions
graph_deltas
work_packages
work_dependencies
selection_groups
variants
variant_fence_counters
attempts
active_leases
runner_leases
ready_queue
change_intents
resource_claims
```

### Cognitive execution

```text
cognitive_tasks
task_classifications
dispatch_decisions
invocations
collaboration_runs
collaboration_members
cognitive_artifacts
fusion_reports
progress_observations
struggle_evaluations
escalation_decisions
```

### Context

```text
context_graphs
context_nodes
context_edges
context_capsules
capsule_members
compression_jobs
compression_artifacts
continuity_transitions
```

### Session and behavior

```text
agent_sessions
native_sessions
session_observations
terminal_sessions
terminal_frames
process_observations
dialog_observations
steering_commands
behavior_rules
behavior_events
workspace_manifests
workspace_entries
```

### Capacity and finance

```text
credential_profiles
auth_challenges
quota_dimensions
quota_observations
quota_reservations
budget_accounts
budget_reservations
usage_facts
price_snapshots
circuit_breakers
```

### Delivery and proof

```text
candidates
candidate_lineage
evidence
evidence_dependencies
reviews
review_findings
selections
effect_intents
effect_receipts
integrations
observation_windows
outcomes
interventions
artifacts
commands
outbox
events
audit_events
projection_checkpoints
```

## 26.2 Core DDL examples

```sql
CREATE TABLE variant_fence_counters (
    variant_id       TEXT PRIMARY KEY REFERENCES variants(id),
    next_fence       BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE attempts (
    id               TEXT PRIMARY KEY,
    variant_id       TEXT NOT NULL REFERENCES variants(id),
    work_package_id  TEXT NOT NULL REFERENCES work_packages(id),
    attempt_number   INTEGER NOT NULL,
    fence            BIGINT NOT NULL,
    runner_id        TEXT,
    runner_epoch     BIGINT,
    workspace_id     TEXT,
    workspace_nonce  BLOB,
    scope_revision   BIGINT NOT NULL,
    context_revision BIGINT NOT NULL,
    state            TEXT NOT NULL,
    lane_json        TEXT NOT NULL,
    started_at       TEXT,
    ended_at         TEXT,
    failure_class    TEXT,
    outcome_json     TEXT,
    UNIQUE(variant_id, fence)
);

CREATE TABLE active_leases (
    variant_id       TEXT PRIMARY KEY REFERENCES variants(id),
    attempt_id       TEXT NOT NULL UNIQUE REFERENCES attempts(id),
    fence            BIGINT NOT NULL,
    runner_id        TEXT NOT NULL,
    runner_epoch     BIGINT NOT NULL,
    workspace_nonce  BLOB NOT NULL,
    heartbeat_at     TEXT NOT NULL,
    expires_at       TEXT NOT NULL
);

CREATE TABLE cognitive_tasks (
    id                       TEXT PRIMARY KEY,
    attempt_id               TEXT REFERENCES attempts(id),
    verifier_run_id          TEXT,
    parent_task_id           TEXT REFERENCES cognitive_tasks(id),
    task_class               TEXT NOT NULL,
    objective                TEXT NOT NULL,
    input_manifest_hash      TEXT NOT NULL,
    context_capsule_id       TEXT,
    output_schema_id         TEXT NOT NULL,
    risk_class               TEXT NOT NULL,
    quality_floor            REAL NOT NULL,
    latency_class            TEXT NOT NULL,
    budget_json              TEXT NOT NULL,
    routing_policy_id        TEXT NOT NULL,
    collaboration_protocol   TEXT NOT NULL,
    completion_contract_json TEXT NOT NULL,
    state                    TEXT NOT NULL,
    created_at               TEXT NOT NULL
);

CREATE TABLE task_classifications (
    cognitive_task_id TEXT PRIMARY KEY REFERENCES cognitive_tasks(id),
    classification_json TEXT NOT NULL,
    confidence REAL NOT NULL,
    classifier_version TEXT NOT NULL,
    evidence_hash TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE dispatch_decisions (
    id                   TEXT PRIMARY KEY,
    cognitive_task_id    TEXT NOT NULL REFERENCES cognitive_tasks(id),
    classification_hash  TEXT NOT NULL,
    eligible_lanes_json  TEXT NOT NULL,
    excluded_lanes_json  TEXT NOT NULL,
    feature_vector_json  TEXT NOT NULL,
    scores_json          TEXT NOT NULL,
    selected_strategy    TEXT NOT NULL,
    selected_lanes_json  TEXT NOT NULL,
    fallback_ladder_json TEXT NOT NULL,
    quota_snapshot_hash  TEXT NOT NULL,
    budget_snapshot_hash TEXT NOT NULL,
    scheduler_version    TEXT NOT NULL,
    created_at           TEXT NOT NULL
);

CREATE TABLE invocations (
    id                    TEXT PRIMARY KEY,
    cognitive_task_id     TEXT NOT NULL REFERENCES cognitive_tasks(id),
    dispatch_decision_id  TEXT NOT NULL REFERENCES dispatch_decisions(id),
    provider              TEXT NOT NULL,
    model_snapshot_id     TEXT NOT NULL,
    profile_id            TEXT NOT NULL,
    harness               TEXT NOT NULL,
    adapter_version       TEXT NOT NULL,
    native_session_id     TEXT,
    prompt_version        TEXT NOT NULL,
    context_capsule_id    TEXT NOT NULL,
    reasoning_effort      TEXT NOT NULL,
    quota_reservation_id  TEXT NOT NULL,
    budget_reservation_id TEXT NOT NULL,
    state                 TEXT NOT NULL,
    started_at            TEXT,
    completed_at          TEXT,
    usage_json            TEXT,
    cost_json             TEXT,
    raw_event_artifact_id TEXT,
    output_artifact_id    TEXT
);

CREATE TABLE collaboration_runs (
    id                     TEXT PRIMARY KEY,
    cognitive_task_id      TEXT NOT NULL REFERENCES cognitive_tasks(id),
    protocol               TEXT NOT NULL,
    diversity_score        REAL,
    budget_json            TEXT NOT NULL,
    state                  TEXT NOT NULL,
    selected_output_id     TEXT,
    disagreement_artifact_id TEXT,
    created_at             TEXT NOT NULL,
    completed_at           TEXT
);

CREATE TABLE cognitive_artifacts (
    id                     TEXT PRIMARY KEY,
    cognitive_task_id      TEXT NOT NULL REFERENCES cognitive_tasks(id),
    producer_invocation_id TEXT NOT NULL REFERENCES invocations(id),
    kind                   TEXT NOT NULL,
    schema_version         TEXT NOT NULL,
    content_hash           TEXT NOT NULL,
    artifact_id            TEXT NOT NULL REFERENCES artifacts(id),
    confidence             REAL,
    provenance_json        TEXT NOT NULL,
    created_at             TEXT NOT NULL
);

CREATE TABLE progress_observations (
    id                         TEXT PRIMARY KEY,
    attempt_id                 TEXT NOT NULL REFERENCES attempts(id),
    cognitive_task_id          TEXT REFERENCES cognitive_tasks(id),
    kind                       TEXT NOT NULL,
    source                     TEXT NOT NULL,
    before_hash                TEXT,
    after_hash                 TEXT,
    delta_json                 TEXT NOT NULL,
    acceptance_coverage_delta  REAL NOT NULL DEFAULT 0,
    uncertainty_delta          REAL NOT NULL DEFAULT 0,
    meaningful                 BOOLEAN NOT NULL,
    reason                     TEXT NOT NULL,
    observed_at                TEXT NOT NULL
);

CREATE TABLE struggle_evaluations (
    id                   TEXT PRIMARY KEY,
    attempt_id           TEXT NOT NULL REFERENCES attempts(id),
    score                REAL NOT NULL,
    state                TEXT NOT NULL,
    feature_vector_json  TEXT NOT NULL,
    critical_triggers_json TEXT NOT NULL,
    evaluator_version    TEXT NOT NULL,
    created_at           TEXT NOT NULL
);

CREATE TABLE context_nodes (
    id                TEXT PRIMARY KEY,
    context_graph_id  TEXT NOT NULL REFERENCES context_graphs(id),
    node_type         TEXT NOT NULL,
    canonical_content TEXT NOT NULL,
    content_hash      TEXT NOT NULL,
    source_refs_json  TEXT NOT NULL,
    created_by        TEXT NOT NULL,
    confidence        REAL,
    validity          TEXT NOT NULL,
    sensitivity       TEXT NOT NULL,
    token_estimate    INTEGER NOT NULL,
    created_at        TEXT NOT NULL
);

CREATE TABLE context_edges (
    id               TEXT PRIMARY KEY,
    context_graph_id TEXT NOT NULL REFERENCES context_graphs(id),
    source_node_id   TEXT NOT NULL REFERENCES context_nodes(id),
    target_node_id   TEXT NOT NULL REFERENCES context_nodes(id),
    edge_type        TEXT NOT NULL,
    created_at       TEXT NOT NULL,
    UNIQUE(source_node_id, target_node_id, edge_type)
);

CREATE TABLE context_capsules (
    id                  TEXT PRIMARY KEY,
    context_graph_id    TEXT NOT NULL REFERENCES context_graphs(id),
    context_revision    BIGINT NOT NULL,
    cognitive_task_id   TEXT NOT NULL REFERENCES cognitive_tasks(id),
    target_lane_json    TEXT NOT NULL,
    content_hash        TEXT NOT NULL,
    source_manifest_hash TEXT NOT NULL,
    coverage_json       TEXT NOT NULL,
    token_estimate      INTEGER NOT NULL,
    artifact_id         TEXT NOT NULL REFERENCES artifacts(id),
    created_at          TEXT NOT NULL
);

CREATE TABLE behavior_events (
    id                 TEXT PRIMARY KEY,
    rule_id            TEXT NOT NULL,
    rule_version       TEXT NOT NULL,
    attempt_id         TEXT NOT NULL REFERENCES attempts(id),
    invocation_id      TEXT REFERENCES invocations(id),
    detector           TEXT NOT NULL,
    observed_action    TEXT NOT NULL,
    evidence_json      TEXT NOT NULL,
    severity           TEXT NOT NULL,
    enforcement_action TEXT NOT NULL,
    resolution         TEXT,
    created_at         TEXT NOT NULL,
    resolved_at        TEXT
);

CREATE TABLE quota_observations (
    id                TEXT PRIMARY KEY,
    profile_id        TEXT NOT NULL REFERENCES credential_profiles(id),
    model_snapshot_id TEXT,
    dimension         TEXT NOT NULL,
    limit_value       REAL,
    used_value        REAL,
    remaining_value   REAL,
    unit              TEXT NOT NULL,
    window_start      TEXT,
    window_end        TEXT,
    reset_at          TEXT,
    source            TEXT NOT NULL,
    confidence        REAL NOT NULL,
    evidence_hash     TEXT,
    observed_at       TEXT NOT NULL
);

CREATE TABLE quota_reservations (
    id                TEXT PRIMARY KEY,
    invocation_id     TEXT UNIQUE,
    profile_id        TEXT NOT NULL REFERENCES credential_profiles(id),
    model_snapshot_id TEXT NOT NULL,
    forecast_json     TEXT NOT NULL,
    hard_cap_json     TEXT NOT NULL,
    state             TEXT NOT NULL,
    expires_at        TEXT NOT NULL,
    settled_usage_json TEXT,
    created_at        TEXT NOT NULL,
    settled_at        TEXT
);

CREATE TABLE effect_intents (
    id                       TEXT PRIMARY KEY,
    logical_effect_key       TEXT NOT NULL,
    provider                 TEXT NOT NULL,
    target_identity          TEXT NOT NULL,
    desired_state_hash       TEXT NOT NULL,
    remote_preconditions_json TEXT NOT NULL,
    attempt_id               TEXT NOT NULL REFERENCES attempts(id),
    fence                    BIGINT NOT NULL,
    policy_version           TEXT NOT NULL,
    payload_hash             TEXT NOT NULL,
    provider_idempotency_key TEXT,
    state                    TEXT NOT NULL,
    created_at               TEXT NOT NULL,
    UNIQUE(provider, logical_effect_key)
);

CREATE TABLE commands (
    organization_id TEXT NOT NULL,
    command_id      TEXT NOT NULL,
    command_type    TEXT NOT NULL,
    request_hash    TEXT NOT NULL,
    status          TEXT NOT NULL,
    response_json   TEXT,
    created_at      TEXT NOT NULL,
    completed_at    TEXT,
    PRIMARY KEY(organization_id, command_id)
);
```

SQLite uses application-level transition transactions and partial indexes where supported. PostgreSQL additionally uses row locks, `SKIP LOCKED`, exclusion constraints where appropriate, and serializable retry logic.

## 26.3 Lease acquisition

```text
acquire(variant_id, lane, runner_id, command_id):
  BEGIN SERIALIZABLE
    return prior result if command exists
    lock Variant and Work Package
    require Work Package READY
    require Variant eligible
    require dependencies satisfied
    require budget/quota/policy eligibility
    require no active authoritative lease
    atomically acquire all requested resource claims
    increment permanent fence counter
    create Attempt STARTING
    create active lease
    remove ready row
    append AttemptLeased event
    enqueue dispatch outbox command
    store idempotent command result
  COMMIT
```

## 26.4 Heartbeat

```sql
UPDATE active_leases
SET heartbeat_at = CURRENT_TIMESTAMP,
    expires_at = :db_now_plus_ttl
WHERE variant_id = :variant
  AND attempt_id = :attempt
  AND fence = :fence
  AND runner_id = :runner
  AND runner_epoch = :epoch
  AND workspace_nonce = :nonce;
```

Zero rows means stale authority. Runner immediately freezes tools, checkpoints salvage if allowed, and terminates.

## 26.5 Ready queue

Ready rows are push-maintained in the same transaction that:

- closes a dependency;
- accepts a required gate;
- releases a lease;
- activates a Plan/Delta;
- completes Selection;
- resolves an Intervention.

Do not rescan every Work Package on a timer as the primary scheduler.

## 26.6 Durable outbox and inbox

All runner commands, Effect dispatches, projection updates, and external notifications use:

```text
transactional state change
+ outbox row in same commit
→ at-least-once delivery
→ receiver idempotency
→ durable acknowledgment
```

Inbound provider and runner events have unique event IDs, per-session sequence, causation ID, correlation ID, and deduplication.

---

# 27. Public API

All mutations require an `Idempotency-Key` or explicit command ID. OpenAPI is generated from Rust types and used to generate the TypeScript client.

## 27.1 Mission and graph

```text
POST /v1/missions
GET  /v1/missions/{id}
POST /v1/missions/{id}/acceptance-contracts
POST /v1/missions/{id}/plans
POST /v1/plans/{hash}/materialize
POST /v1/plans/{id}/graph-deltas
GET  /v1/work-packages/{id}
POST /v1/work-packages/{id}/commands
POST /v1/work-packages/{id}/selection-groups
```

## 27.2 Cognitive routing and fusion

```text
POST /v1/cognitive-tasks
GET  /v1/cognitive-tasks/{id}
POST /v1/cognitive-tasks/{id}/classify
POST /v1/cognitive-tasks/{id}/dispatch
POST /v1/cognitive-tasks/{id}/collaboration-runs
GET  /v1/collaboration-runs/{id}
GET  /v1/dispatch-decisions/{id}
POST /v1/routing/simulate
GET  /v1/routing/calibration
GET  /v1/fusion/metrics
```

## 27.3 Sessions and supervision

```text
GET  /v1/sessions
GET  /v1/sessions/{id}
POST /v1/sessions/{id}/steer
POST /v1/sessions/{id}/interrupt
POST /v1/sessions/{id}/pause
POST /v1/sessions/{id}/resume
POST /v1/sessions/{id}/terminate
POST /v1/sessions/{id}/context-transition
GET  /v1/sessions/{id}/terminal          WebSocket
GET  /v1/sessions/{id}/events            SSE
GET  /v1/sessions/{id}/processes
GET  /v1/sessions/{id}/dialogs
```

## 27.4 Context

```text
GET  /v1/context-graphs/{id}
POST /v1/context-graphs/{id}/nodes
POST /v1/context-capsules/compile
GET  /v1/context-capsules/{id}
POST /v1/compression-jobs
GET  /v1/compression-jobs/{id}
POST /v1/continuity-transitions
```

## 27.5 Behavior and workspaces

```text
GET  /v1/behavior/rules
POST /v1/behavior/rules
GET  /v1/behavior/events
POST /v1/behavior/events/{id}/resolve
GET  /v1/workspaces/{id}
GET  /v1/workspaces/{id}/manifest
GET  /v1/workspaces/{id}/diff
POST /v1/workspaces/{id}/checkpoint
POST /v1/workspaces/{id}/cleanup
```

## 27.6 Capacity

```text
POST /v1/profiles
POST /v1/profiles/{id}/auth-challenges
GET  /v1/profiles/{id}/quota
GET  /v1/capacity
GET  /v1/costs
POST /v1/quota/refresh
POST /v1/budgets/{id}/commands
```

## 27.7 Proof and delivery

```text
GET  /v1/candidates/{id}
POST /v1/candidates/{id}/verify
GET  /v1/evidence/{id}
POST /v1/reviews
POST /v1/selections
POST /v1/effects
GET  /v1/effects/{id}
POST /v1/integrations
GET  /v1/integrations/{id}
GET  /v1/observation-windows/{id}
```

## 27.8 Operations

```text
GET  /v1/events?after={sequence}          SSE
GET  /v1/audit
GET  /v1/kpis
GET  /v1/healthz
GET  /v1/readyz
POST /v1/repositories/{id}/freeze
POST /v1/repositories/{id}/drain
POST /v1/interventions/{id}/resolve
```

## 27.9 Error model

Typed problem details:

```json
{
  "type": "https://centerrail.dev/problems/stale-authority",
  "title": "Stale authority token",
  "status": 409,
  "code": "STALE_AUTHORITY",
  "correlation_id": "corr_...",
  "retryable": false,
  "details": {
    "expected_fence": 14,
    "received_fence": 13
  }
}
```

No handler accepts undocumented dynamic query parameters. Open-ended maps use typed request bodies.

---

# 28. Runner protocol

mTLS gRPC/Protobuf:

```text
RegisterRunner
RenewRunnerLease
LeaseCommandStream
AcknowledgeCommand
EmitRunnerEvents
HeartbeatAttempt
RequestChangeIntent
RequestScopeSuccessor
CreateCheckpoint
StartAgentSession
SteerAgentSession
InterruptAgentSession
TransitionContext
PrepareCandidate
UploadArtifact
ReportQuotaObservation
ReportUsage
ReportBehaviorEvent
ReportProgress
TerminateAttempt
CleanupWorkspace
```

Commands are at-least-once and idempotent.

## 28.1 Runner registration

```text
runner_id
runner_epoch
host identity
platform/architecture
sandbox backends
PTY/ConPTY support
toolchain inventories
provider adapters
capacity
attestation
data-residency labels
```

## 28.2 Runner self-fencing

- renew runner lease continuously;
- local monotonic self-kill deadline shorter than server expiry;
- stop accepting commands before expiry;
- freeze tool gateway on missed renewal;
- terminate all owned process groups;
- stale ledger fence blocks effects even if self-kill fails.

## 28.3 Event sequencing

Every event:

```text
event_id
stream_id
sequence
timestamp
causation_id
correlation_id
trace_id
authority_token_hash
payload_type
payload
artifact_refs
```

Sequence gaps are visible and trigger replay.

---

# 29. TypeScript workflow authoring and canonical IR

Workflow authors use a TypeScript builder that compiles to canonical signed JSON or Protobuf. TypeScript never executes inside the authoritative scheduler.

```ts
export default workflow("verified-adaptive-feature", ({ mission }) => {
  const map = mapReduce({
    map: scoutRepo({ tier: "economy", count: 6 }),
    reduce: synthesizeRepoMap({ tier: "standard" }),
  });

  const plan = triadFusion({
    contributors: [
      planner({ providerFamily: "openai", tier: "frontier" }),
      planner({ providerFamily: "anthropic", tier: "frontier" }),
    ],
    fuser: planner({ providerFamily: "google", tier: "frontier" }),
    rubric: "acceptance-covered-safe-plan-v2",
  });

  const work = boundedMap(plan.workPackages, {
    concurrency: "repo-writer-slots",
    route: minimumSufficientIntelligence(),
    struggle: escalationLadder("default"),
    run: implement(),
  });

  const proof = verify(work, {
    clean: true,
    hidden: byRisk(),
  });

  const review = blindReview(proof, {
    independence: byRisk(),
  });

  return integrate(review, {
    effect: "github-protected",
    observation: mission.observationPolicy,
  });
});
```

Required primitives:

- sequence;
- parallel;
- bounded map;
- map/reduce;
- retry/backoff;
- timeout;
- checkpoint;
- route;
- cascade;
- shadow;
- rank/select;
- fusion;
- council;
- debate;
- race;
- quorum;
- blind review;
- repair;
- escalation;
- conditional;
- budget/quota gate;
- risk gate;
- scope successor;
- human Intervention;
- external event;
- schedule;
- compensation;
- integration queue.

IR includes exact versions, schemas, resource limits, and policy references. Arbitrary code in workflow definitions is rejected.

---

# 30. Configuration examples

## 30.1 Routing policy

```toml
[routing]
policy_id = "adaptive-default-v1"
quality_floor = 0.97
max_repair_loops = 2
shadow_sample_rate = 0.05

[routing.portfolio]
economy_call_share_min = 0.70
economy_call_share_max = 0.90
frontier_budget_reserve = 0.20
speculation_budget_share_max = 0.10

[routing.tiers.M1]
allowed_task_classes = [
  "classify_route",
  "summarize_local",
  "compress_context",
  "documentation",
  "extract_structured"
]
min_calibrated_success = 0.97

[routing.tiers.M3]
required_for_risk = ["R3", "R4"]
```

## 30.2 Model mappings

```toml
[[models]]
provider = "openai"
model = "gpt-5.6-luna"
tier = "M1"
tasks = ["classify_route", "summarize_local", "compress_context"]

[[models]]
provider = "openai"
model = "gpt-5.6-terra"
tier = "M2"
tasks = ["bounded_bug_fix", "test_authoring", "code_review"]

[[models]]
provider = "openai"
model = "gpt-5.6-sol"
tier = "M3"
tasks = ["architecture_design", "security_analysis", "broad_refactor"]
```

The actual registry is refreshed and pinned into `ModelSnapshot` records. Configuration cannot assert unsupported capabilities.

## 30.3 Fusion

```toml
[fusion.triad-architecture]
protocol = "triad_fusion"
task_classes = ["architecture_design", "migration_design"]
min_business_value = 100
minimum_diversity = 0.65
contributors = 2
fuser_tier = "M3"
max_cost_multiplier = 3.2
require_independent_check = true

[fusion.context-compression]
protocol = "compression_ensemble"
task_classes = ["compress_context"]
activate_above_tokens = 60000
minimum_source_coverage = 0.98
minimum_acceptance_coverage = 1.0
```

## 30.4 Struggle policy

```toml
[struggle.default]
watch_score = 0.35
struggling_score = 0.55
stalled_score = 0.75
max_identical_commands = 3
max_same_failure_without_delta = 2
max_handoffs_per_atomic_step = 2
max_repair_loops = 2
context_transition = 0.60
```

## 30.5 Behavior

```toml
[behavior]
catalog = "centerrail-default-v1"
unknown_destructive_state = "block"
untracked_candidate_policy = "block"
runtime_files_policy = "outside_repo"

[[behavior.exceptions]]
rule_id = "CD007"
paths = ["tools/benchmark-fixtures/**"]
reason = "approved vendored benchmark fixture"
expires_at = "2026-12-31T00:00:00Z"
approved_by = "principal_..."
```

## 30.6 Provider profile

```toml
[[profiles]]
id = "openai-team"
provider = "openai"
owner = "team-platform"
authorization_class = "organization_seat"
custody = "enterprise"
repositories = ["repo_*"]
concurrency = 12

[profiles.reserve]
critical = 0.20
security = 0.10
speculative_max = 0.05
```

---

# 31. Repository layout

```text
/
├── Cargo.toml
├── crates/
│   ├── domain/
│   ├── ids/
│   ├── ledger/
│   ├── transitions/
│   ├── events/
│   ├── outbox/
│   ├── projections/
│   ├── workflow-ir/
│   ├── workflow-runtime/
│   ├── scheduler/
│   ├── task-taxonomy/
│   ├── routing/
│   ├── fusion/
│   ├── progress/
│   ├── escalation/
│   ├── quota/
│   ├── budgets/
│   ├── auth-broker/
│   ├── policy/
│   ├── behavior/
│   ├── context-graph/
│   ├── context-capsule/
│   ├── compression/
│   ├── evidence/
│   ├── review/
│   ├── effects/
│   ├── scm/
│   ├── github-effects/
│   ├── telemetry/
│   ├── audit/
│   ├── runner-protocol/
│   ├── runner/
│   ├── workspace/
│   ├── git-safe/
│   ├── sandbox/
│   ├── process-supervisor/
│   ├── terminal/
│   ├── dialog-controller/
│   ├── tool-gateway/
│   ├── mcp-registry/
│   ├── harness-core/
│   ├── harness-codex/
│   ├── harness-claude/
│   ├── harness-cursor/
│   ├── harness-antigravity/
│   ├── verifier/
│   ├── api/
│   ├── generated-types/
│   └── test-simulation/
├── apps/
│   ├── centerraild/
│   ├── centerrail-runner/
│   ├── centerrail-verifier/
│   ├── centerrail-effects/
│   └── centerrail/
├── web/
│   ├── src/
│   │   ├── control-tower/
│   │   ├── missions/
│   │   ├── graph/
│   │   ├── router/
│   │   ├── fusion/
│   │   ├── fleet/
│   │   ├── sessions/
│   │   ├── attempts/
│   │   ├── context/
│   │   ├── quota/
│   │   ├── behavior/
│   │   ├── workspaces/
│   │   ├── integrations/
│   │   ├── quality/
│   │   ├── incidents/
│   │   ├── audit/
│   │   └── settings/
│   └── vite.config.ts
├── packages/
│   ├── workflow-sdk/
│   ├── generated-api/
│   ├── event-schemas/
│   └── ui/
├── schemas/
│   ├── api/
│   ├── events/
│   ├── workflow/
│   ├── cognitive/
│   ├── fusion/
│   ├── context/
│   ├── behavior/
│   ├── harness/
│   └── evidence/
└── tests/
    ├── model/
    ├── property/
    ├── chaos/
    ├── provider-contract/
    ├── fusion/
    ├── routing/
    ├── context-migration/
    ├── compression/
    ├── behavior/
    ├── terminal/
    ├── cross-platform/
    ├── adversarial-scope/
    ├── prompt-injection/
    ├── identity/
    ├── security/
    └── end-to-end/
```

No first-party Go, Python, Java, shell orchestration, or Node backend. Rust owns backend/control/runner/verifier/effect behavior. TypeScript is limited to Vite/React, workflow authoring, generated clients, and browser tests. External vendor CLIs remain isolated dependencies.

---

# 32. Observability, KPIs, and SLOs

## 32.1 OpenTelemetry

Use OTel for traces, infrastructure metrics, and structured logs. High-cardinality Attempt IDs live in traces/events, not bounded metric labels.

Every event carries:

- organization/repository/Mission;
- Work Package/Variant/Attempt;
- Cognitive Task/Invocation/Collaboration Run;
- runner/session;
- provider/model/harness/profile;
- workflow/policy/config/routing versions;
- risk and data class;
- causation/correlation/trace IDs.

## 32.2 North-star metric

```text
Verified Surviving Mission Value
──────────────────────────────────────────────────────
human intervention + model cost + runner/CI cost
```

Mission value is pre-sized before treatment. It counts only when acceptance, proof, integration, and observation requirements pass.

## 32.3 Routing metrics

- economy call/token/cost share;
- quality floor compliance;
- success calibration;
- cost per accepted task by class;
- routing regret;
- promotion/fallback rate;
- frontier reserve use;
- quota-related delay;
- shadow-lane win rate;
- provider concentration.

## 32.4 Fusion metrics

- fusion admission rate;
- oracle win rate;
- selection versus synthesis;
- unique idea/finding yield;
- contradiction resolution;
- unsupported claim rate;
- cost/latency multiplier;
- accepted survival uplift;
- code-race yield;
- fuser regression.

## 32.5 Context metrics

- capsule token size;
- compression ratio;
- acceptance/decision/risk/source coverage;
- checker disagreement;
- migration continuity pass;
- context transition frequency;
- handoffs per atomic step;
- context-related stalls;
- retrieval precision.

## 32.6 Behavior metrics

- events by rule/model/provider/repository;
- prevented versus detected;
- remediation success;
- false positives;
- Candidate quarantines;
- destructive-action blocks;
- runtime artifact leakage;
- temp-folder incidents;
- completion fraud/false-done rate.

## 32.7 Session metrics

- structured transport availability;
- PTY fallback rate;
- dialog auto-resolution;
- steering acknowledgment and application latency;
- context-transition latency;
- unresponsive detection;
- forced termination;
- orphan process rate;
- manual terminal intervention.

## 32.8 Initial SLOs

| SLI | Objective |
|---|---|
| acknowledged control-event loss | 0 |
| concurrent writer violation | 0 |
| stale-fence mutation/effect accepted | 0 |
| duplicate plan/delta materialization | 0 |
| shared writable checkout | 0 |
| unverified destructive cleanup | 0 |
| projection lag | p95 <1s, p99 <3s |
| scheduler decision | p95 <100ms, p99 <500ms |
| lease transaction | p99 <100ms |
| warm workspace start | p95 <5s |
| structured steering ack | p95 <2s |
| cancellation ack | p95 <2s |
| forced process-tree termination | p95 <15s |
| context transition after threshold | p95 <60s |
| lost-run detection | p95 <30s |
| branch/checkpoint salvage when filesystem reachable | ≥99.99% |
| routine dialog automation | ≥99.9% |
| manual terminal approval required for routine work | 0 |
| context capsule mandatory coverage | 100% |
| behavior event audit completeness | 100% |

Economic and model-quality targets are baseline-and-ratchet hypotheses, not guarantees declared before data exists.

# 33. Testing strategy

Centerrail must be tested as a distributed safety system, not only a web application.

## 33.1 Executable state-machine model

Before broad implementation, model:

- Plan/Graph Delta materialization;
- Variant leases and fences;
- runner expiry;
- scope-successor transfer;
- command idempotency;
- Effect ambiguity;
- Selection;
- context transition;
- quota reservation;
- destructive cleanup.

Use a model checker or exhaustive small-state simulator. Generate traces into Rust property tests.

Properties:

```text
never two active authoritative writers per Variant
never reuse fence
stale token never mutates
plan/delta atomic
idempotent replay returns same result
unknown liveness never destroys
ambiguous effect never blindly replays
scope transfer has no authority gap
workspace cleanup never precedes preservation
```

## 33.2 Deterministic provider simulator

Simulate:

- token and text streaming;
- structured tool calls;
- local plans and permissions;
- usage events;
- context reports;
- auth expiry;
- 401/429/5xx;
- quota reset;
- malformed/out-of-order/duplicate events;
- delayed stale events;
- long turns;
- refusals;
- process crash;
- resume failure;
- false completion;
- context limit;
- terminal-only dialogs;
- provider version drift.

Every adapter must pass the same contract suite.

## 33.3 Routing benchmark

Build a repository/task corpus stratified by:

- task class;
- language/framework;
- scope;
- novelty;
- ambiguity;
- risk;
- repository;
- provider/model;
- harness version.

For each task:

- exact base SHA;
- hidden acceptance;
- predeclared value and risk;
- cost/latency;
- outcome and observation;
- human intervention.

Evaluate:

- always-economy;
- always-standard;
- always-frontier;
- rules router;
- learned router;
- cascade;
- fusion.

Promotion requires confidence-adjusted quality floor, not average anecdote.

## 33.4 Fusion benchmark

Test protocols on tasks where diversity can matter:

- architecture;
- security review;
- migration;
- debugging;
- performance;
- planning;
- context compression;
- code review.

Metrics:

- best individual versus selected versus fused;
- hidden-test pass;
- unsupported claims;
- disagreement retention;
- cost/latency;
- reviewer preference;
- accepted survival.

Include cases where fusion should select one candidate or declare unresolved. Penalize forced synthesis.

## 33.5 Context migration tests

For every provider pair:

```text
Codex → Claude
Codex → Cursor
Codex → Antigravity
Claude → Codex
...
```

Test:

- clean fresh migration;
- mid-turn interruption;
- context threshold;
- auth expiry;
- quota exhaustion;
- provider crash;
- source drift;
- contradictory context;
- large capsule;
- omitted native session.

Assertions:

- mandatory coverage 100%;
- exact SHA/scope preserved;
- successor acknowledgment correct;
- no lost unresolved blocker;
- no duplicate effect;
- no uncheckpointed work loss.

## 33.6 Compression tests

Golden corpora with:

- acceptance requirements;
- decisions;
- rejected alternatives;
- contradictions;
- exact source references;
- security risks;
- unresolved questions;
- noisy repeated logs.

Adversarially verify compression retains mandatory facts and marks omissions.

## 33.7 Behavior tests

For every rule:

- positive fixture;
- negative fixture;
- platform variants;
- enforcement action;
- remediation;
- postcondition;
- audit record;
- exception handling.

High-risk rules require fail-closed fault tests when detector inputs are missing.

## 33.8 PTY/terminal golden tests

Record sanitized terminal sessions for each pinned provider version:

- startup;
- plan prompt;
- permission prompt;
- rate limit;
- context limit;
- auth challenge;
- update dialog;
- crash;
- alternate screen;
- Unicode/wide characters;
- resize;
- partial escape sequence;
- Windows ConPTY differences.

Structured protocol remains preferred, but compatibility recognizers use golden screen tests and explicit confidence.

## 33.9 Cross-platform tests

CI matrix:

```text
Linux x86_64/arm64
macOS arm64/x86_64 where available
Windows x86_64
WSL
```

Test:

- path canonicalization;
- case sensitivity;
- symlink/junction/reparse escape;
- process-tree ownership;
- PTY/ConPTY;
- quoting;
- line endings;
- executable bits;
- Git config isolation;
- cleanup;
- atomic file replacement;
- long paths;
- 8.3 aliases.

## 33.10 Fault injection

Continuously inject:

- kill runner during edit/checkpoint/commit/push;
- kill control plane during lease/materialization/selection;
- partition before lease expiry;
- duplicate command;
- delayed stale event;
- database retry/failover;
- object-store timeout;
- corrupt cache;
- provider outage;
- quota exhaustion;
- credential expiry;
- terminal transport failure;
- lost structured event;
- Git conflict;
- external PR/ref mutation;
- Effect timeout after remote success;
- cleanup failure;
- projection lag.

## 33.11 Adversarial security tests

- repository prompt requests credential exfiltration;
- test script probes metadata/local network;
- malicious Git filter/hook/include;
- unsafe submodule;
- symlink/junction escape;
- agent edits protected path;
- agent deletes tests;
- reviewer attempts to waive checks;
- UI attempts to mark auth complete;
- stale worker tries to push;
- binary/high-entropy secret insertion;
- XSS in terminal/artifact;
- MCP server exfiltration;
- provider profile mismatch;
- effect replay after timeout.

## 33.12 Portal tests

- delayed/out-of-order SSE;
- projection rebuild;
- city/repository/Mission switch;
- stale response cannot overwrite new selection;
- every mutation shows pending until acknowledged;
- unknown state renders unknown;
- terminal sanitization;
- large graph virtualization;
- accessibility;
- permissions and audit.

## 33.13 End-to-end proof scenario

The first mandatory demonstration:

> One Mission materializes once; two independent planning models create proposals; a third produces a provenance-preserving Plan; one Variant receives a never-reused fence; a stale duplicate cannot mutate state or create an Effect; useful work survives process death; context migrates to another provider without losing acceptance or scope; a bad workspace action is blocked; the Candidate is exact; a clean verifier produces qualifying Evidence; and one protected GitHub integration occurs without exposing SCM credentials to the agent.

---

# 34. Exit-gated build sequence

Dates are secondary to exit gates.

## Phase 0 — semantics, schemas, and simulator

Build:

- domain vocabulary;
- state machines;
- Authority Token;
- Plan/Graph Delta canonicalization;
- task taxonomy;
- routing/fusion IR;
- context schemas;
- behavior catalog;
- provider simulator;
- SCM/effect simulator;
- executable lease/effect/scope/cleanup model.

Exit:

- exhaustive/sampled traces cannot violate invariants;
- schemas and generated types are stable;
- all command/effect handlers define idempotency.

## Phase 1 — smallest correct local kernel

Build:

- Rust modular monolith;
- SQLite WAL;
- Mission/Plan/Work Package/Variant/Attempt;
- atomic plan materialization;
- ready queue;
- leases/fences;
- commands/outbox/events;
- private-clone factory;
- local Linux runner;
- checkpoint/salvage;
- minimal portal.

Exit:

- zero duplicate graph, fence reuse, shared writable checkout, or stale transition in kill/retry suite;
- workspace preservation proven;
- portal pending-state contract proven.

## Phase 2 — Cognitive Task and economy router

Build:

- Cognitive Task;
- task classification;
- model/provider registry;
- profiles;
- quota/budget reservations;
- deterministic D0;
- M1/M2/M3 tier policy;
- Dispatch Decision;
- shadow routing;
- provider simulator adapter.

Exit:

- every invocation attributable;
- unknown quota represented honestly;
- routing replay deterministic;
- shadow evaluation pipeline produces calibration.

## Phase 3 — first structured provider and session supervisor

Choose one provider with the strongest structured local integration.

Build:

- adapter;
- profile identity probe;
- structured events;
- usage/quota source;
- process tree;
- PTY mirror;
- dialog controller;
- steering/interrupt;
- context monitor;
- Context Graph/Capsule;
- behavior hooks.

Exit:

- no human terminal action required for routine plan/permission/context dialogs;
- provider crash/auth/quota/context tests pass;
- structured event loss visible;
- process termination bounded.

## Phase 4 — Candidate and GitHub Effect loop

Build:

- actual-write enforcement;
- successor-Attempt scope;
- Candidate prepare/finalize;
- hostile Git policy;
- GitHub App Effect Broker;
- exact read-back receipts;
- protected PR flow;
- workspace cleanup.

Exit:

- writer death during commit/push preserves work;
- ambiguous push does not duplicate accepted effect;
- no SCM credential enters sandbox;
- secret/out-of-scope/runtime-polluted Candidate cannot push;
- destructive cleanup fails closed.

## Phase 5 — independent verifier

Build:

- clean reconstruction;
- verifier custody;
- E0–E3 Evidence;
- deterministic gates;
- hidden evaluators;
- Evidence invalidation;
- Proof Bundles.

Exit:

- writer cannot satisfy independent proof;
- changed Candidate cannot retain stale Evidence;
- flaky/infra/unknown never reads PASS.

## Phase 6 — fusion and escalation

Build:

- progress observations;
- struggle score;
- escalation ladder;
- independent critics;
- triad fusion;
- rank/select;
- compression ensemble;
- planning council;
- code Selection Groups and Variants;
- Fusion Lab.

Exit:

- fusion benchmark shows statistically valid uplift on admitted strata;
- forced-fusion regressions are detected;
- code fusion uses new fenced Variant;
- stalled tasks escalate without human babysitting;
- thrash limits enforce.

## Phase 7 — remaining providers

Promote Codex, Claude, Cursor, and Antigravity sequentially through:

- protocol conformance;
- profile identity;
- auth expiry;
- quota;
- structured event normalization;
- cancellation;
- malformed/delayed events;
- context migration;
- canary;
- rollback.

Exit:

- any adapter can be disabled without compromising kernel;
- all provider pairs pass C3 migration;
- unsupported capability blocks dispatch.

## Phase 8 — complete portal and quality system

Build:

- Control Tower;
- Mission Graph;
- Cognitive Router;
- Fusion Lab;
- Session Supervisor;
- Context Lineage;
- Quota/Capacity;
- Struggle Cockpit;
- Behavior Center;
- Workspace Hygiene;
- Merge Rail;
- Quality Lab;
- Incidents/Audit.

Exit:

- operators can diagnose every blocked task without SSH;
- all state shows sequence/lag/confidence;
- no optimistic authoritative UI;
- accessibility and large-scale graph tests pass.

## Phase 9 — team distribution

Build:

- PostgreSQL;
- S3-compatible CAS;
- mTLS runners;
- RBAC/OIDC;
- runner epochs;
- replicated API/projections;
- backpressure;
- high-isolation verifier pools.

Exit:

- partition/failover chaos creates no double authority or accepted stale Effect;
- committed control RPO zero in tested failover;
- mixed 100-task workload has zero invariant violations.

## Phase 10 — advanced optimization

Only after empirical proof:

- contextual routing learner;
- larger councils;
- code races;
- integration batching/bisection;
- conflict forecasting;
- cross-repository sagas;
- S2/S3 sandbox expansion.

Exit:

- statistically valid uplift;
- no guardrail regression;
- every adaptive decision explainable;
- speculation yield positive after total cost.

---

# 35. Implementation epics and ownership

Each epic has one primary Rust crate group and explicit dependencies.

| Epic | Deliverable | Depends on |
|---|---|---|
| E01 | IDs, canonical encoding, digests | none |
| E02 | Domain entities and transition types | E01 |
| E03 | SQLite/Postgres ledger | E01–E02 |
| E04 | commands, outbox, events, audit | E03 |
| E05 | Plan/Graph Delta materializer | E03–E04 |
| E06 | leases, fences, ready queue | E03–E05 |
| E07 | runner protocol and runner lease | E04–E06 |
| E08 | private clone and safe Git | E07 |
| E09 | sandbox/process supervisor | E07 |
| E10 | Tool Gateway/MCP registry | E09 |
| E11 | task taxonomy/classifier | E02–E04 |
| E12 | model/profile registry | E03 |
| E13 | quota/budget reservations | E12 |
| E14 | router and Dispatch Decision | E11–E13 |
| E15 | provider simulator | E07, E14 |
| E16 | session supervisor and terminal | E07, E09 |
| E17 | Context Graph/Capsule | E03–E04 |
| E18 | compression and migration | E14, E16–E17 |
| E19 | behavior engine/catalog | E08–E10, E16 |
| E20 | progress/struggle/escalation | E14, E16–E19 |
| E21 | fusion/collaboration protocols | E14, E17, E20 |
| E22 | Candidate service | E06, E08–E10 |
| E23 | verifier/Evidence | E22 |
| E24 | review/selection | E21, E23 |
| E25 | Effect Broker/GitHub | E22–E24 |
| E26 | integration/observation | E25 |
| E27 | public API/generated TS | all domain epics |
| E28 | portal projections and views | E27 |
| E29 | provider adapters | E12–E18 |
| E30 | distributed team mode | proven local kernel |

Each epic must ship:

- domain schema;
- transition table;
- API/protocol shape;
- unit/property tests;
- fault tests;
- observability;
- operator view;
- migration/rollback plan;
- documentation.

---

# 36. Gastown study: what it has already learned

The current Gastown repository and release history contain valuable mechanisms and incident evidence. Centerrail should adopt the lessons, not copy the authority model.

## 36.1 Current release and active work

At the research date, Gastown’s latest tagged release is v1.2.1. Recent releases include:

- provider usage-limit handling distinct from crashes;
- scheduler capacity buckets;
- stricter worker admission/reuse;
- per-role effort and cost tiers;
- static model promotion after repeated failures;
- stuck-agent restart;
- context-budget guards;
- checkpointing and patrol improvements;
- Windows/macOS/Linux support.

Current open work includes a native OpenCode server worker with authenticated loopback control, persisted session mapping, SSE lifecycle events, durable idempotent nudges without TUI keystrokes, restart/resume, and Windows path/process handling.

## 36.2 Useful ideas to carry forward

### Static escalation as a seed

Gastown’s `model-escalation.json` and redispatch logic demonstrate a practical first step: promote an agent type after repeated failure, with cooldown and attempt limits. Centerrail generalizes this into task-class routing, progress-derived struggle, cross-provider lanes, quota reservations, and a versioned Escalation Decision.

### Cost/outcome correlation

Gastown’s cost-learning work records model, tokens, time, turns, formula, and outcome, and adds preflight statistics. Centerrail should retain this spirit but bind every fact to exact task, model snapshot, Context Capsule, Candidate, Evidence, integration, and survival.

### Fresh-agent distributed stages

Gastown’s distributed workflow proposal recognizes that sequential design/implementation/review can exceed one context window and uses committed artifacts between fresh agents. Centerrail formalizes this with canonical Context Capsules, exact Candidates, private clones, and provider-portable continuity.

### Context-budget monitoring

Gastown issue history shows sessions can freeze at context limit while appearing “working,” and model instructions to hand off are not reliable. Centerrail therefore makes context transition a control-plane action.

### Structured telemetry

Gastown’s agent event telemetry normalizes text, tool, thinking, usage, session, Git, and run identity. Centerrail uses a similar normalized event envelope, but makes it durable and authoritative where supported.

### Native server workers

Gastown’s OpenCode server work correctly avoids TUI keystrokes and uses durable idempotent delivery. Centerrail applies that structured-first principle to all providers.

### Checkpoint and workspace recovery

Gastown repeatedly emphasizes preserving branches, commits, Git state, and context before cycling or cleanup. Centerrail makes preservation a hard cleanup precondition.

## 36.3 Lessons from failures

### Prompt compliance is not execution proof

A Gastown Deacon using a cheap model could report hundreds of patrol cycles without executing required steps because the workflow lacked machine-enforced step receipts. Centerrail economy routing therefore requires stronger deterministic contracts for lower models, not weaker ones.

### Context handoff cannot be model-owned

Gastown sessions have frozen for hours at context limit and needed manual tmux commands. Centerrail monitors context and transitions sessions before the hard limit.

### Provider runtime files pollute product repositories

Gastown has observed provider hook/config directories for Copilot, Cursor, Codex, Gemini, and others become committable. Centerrail keeps runtime files outside the product tree and scans the Candidate.

### Destructive cleanup must fail closed

A Gastown non-force cleanup deleted worktrees after preservation pushes failed. Centerrail uses one decision function for dry-run/execution and requires preservation verification before deletion.

### Terminal and process heuristics are dangerous

Gastown has experienced liveness misclassification and process-tree traversal bugs. Centerrail uses structured provider state plus OS-owned process groups and four-valued observation.

### Repeated handoff is a struggle signal

Frequent handoff/compaction on the same atomic work is not success; it indicates decomposition, routing, or context failure. Centerrail feeds this into struggle scoring.

## 36.4 What Centerrail does not adopt

- Beads/Dolt as authoritative orchestration ledger;
- writable Git worktrees;
- tmux panes as liveness or completion authority;
- raw keystrokes as primary messaging;
- Claude account pooling/rotation as a scale strategy;
- prompt-only step enforcement;
- role-play names as domain authority;
- mutable files as command/ownership truth;
- cleanup keyed only by reusable names/paths;
- model claims as completion.

## 36.5 Fusion gap

The reviewed Gastown tags, issues, source, and PRs show ad hoc multi-model review and static model promotion, but no first-class provider-independent rank/fuse, triad fusion, planning council, code-Variant synthesis, or calibrated economy router. Centerrail’s Cognitive Execution Plane is therefore a substantive product layer rather than a rename of existing Gastown machinery.

---

# 37. Explicit product refusals

Centerrail will not ship:

1. writable Git worktrees;
2. a shared mutable checkout;
3. terminal activity as a liveness oracle;
4. Beads, Dolt, Git, or Markdown as the control-plane ledger;
5. consumer-account cycling or anonymous profile pools;
6. model-generated permission/risk waivers;
7. arbitrary TypeScript in the scheduler;
8. unrestricted MCP/tool servers;
9. provider runtime files inside product source;
10. destructive cleanup without verified preservation;
11. blind retry after ambiguous external effect;
12. author self-review as independent proof;
13. silent Evidence transfer to a changed Candidate;
14. forced fusion when candidates are incompatible;
15. a hard 70–90% economy quota that sacrifices quality or safety;
16. UI buttons that assert auth, completion, or effect success;
17. a claim of equivalent sandbox strength across Linux, macOS, and Windows;
18. a giant microservice topology before the kernel is proven;
19. executive metrics centered on agents, terminals, tokens, or lines;
20. “100% autonomous” or “zero regression” marketing without reproducible proof.

---

# 38. Definition of implementation complete

The product is not implementation-complete until all items below pass.

## Kernel

- [ ] atomic Plan/Graph Delta;
- [ ] permanent fences;
- [ ] one writer per Variant;
- [ ] complete Authority Token checks;
- [ ] idempotent commands/outbox;
- [ ] ready queue;
- [ ] unknown/contradictory observation;
- [ ] tested backup/restore.

## Cognitive plane

- [ ] task taxonomy;
- [ ] D0/M1/M2/M3/M4 routing;
- [ ] model snapshots;
- [ ] profile identity;
- [ ] quota/budget reservation and settlement;
- [ ] Dispatch Decision explanation;
- [ ] shadow calibration;
- [ ] struggle and escalation;
- [ ] triad fusion;
- [ ] code Selection Groups and synthesis Variant;
- [ ] router/fusion benchmark.

## Context

- [ ] canonical Context Graph;
- [ ] capsule compiler;
- [ ] provider renderers;
- [ ] verified compression;
- [ ] mandatory coverage;
- [ ] context thresholds;
- [ ] every provider-pair migration;
- [ ] continuity acknowledgment.

## Session supervision

- [ ] structured adapter events;
- [ ] PTY/ConPTY mirror;
- [ ] process ownership;
- [ ] dialog controller;
- [ ] automated local plan/permission/context handling;
- [ ] steering acknowledgment;
- [ ] interrupt/cancel/terminate;
- [ ] no indefinite blocked state;
- [ ] no routine human terminal approvals.

## Workspaces and behavior

- [ ] private clones;
- [ ] hostile Git policy;
- [ ] Change Intents;
- [ ] successor-Attempt scope;
- [ ] full Workspace Manifest;
- [ ] 84-rule default behavior catalog;
- [ ] runtime file exclusion;
- [ ] Candidate hygiene;
- [ ] preservation-before-cleanup.

## Proof and effects

- [ ] exact Candidates;
- [ ] E0–E4 Evidence;
- [ ] clean verifier;
- [ ] blind review;
- [ ] Evidence invalidation;
- [ ] Proof Bundle;
- [ ] Effect Intent/Receipt;
- [ ] GitHub protected delivery;
- [ ] merge-group verification;
- [ ] observation/survival.

## Portal

- [ ] all primary views;
- [ ] sequence/lag/confidence everywhere;
- [ ] honest pending commands;
- [ ] live session/terminal;
- [ ] routing/fusion/quota/context/behavior visibility;
- [ ] incident reconstruction;
- [ ] accessibility and large-scale tests.

## Quality and operations

- [ ] state-machine model;
- [ ] property tests;
- [ ] provider simulator;
- [ ] chaos suite;
- [ ] adversarial security suite;
- [ ] cross-platform suite;
- [ ] SLO dashboards;
- [ ] canary and rollback;
- [ ] disaster recovery test;
- [ ] independent security review;
- [ ] benchmark against approved baselines.

---

# 39. Final engineering judgment

Centerrail’s differentiation is not the number of simultaneous agents. It is the ability to use many heterogeneous models aggressively while keeping authority, context, quota, behavior, proof, and external effects under one deterministic system.

The economy strategy is:

```text
deterministic tools and economy models for routine cognition
+ calibrated cascades for uncertain work
+ frontier models for high-difficulty/high-risk decisions
+ fusion only where diversity has positive expected value
+ automatic escalation when measured progress fails
```

The safety strategy is:

```text
private clones
+ permanent fences
+ mediated tools
+ verified context
+ machine-enforced behavior
+ exact Candidates
+ independent proof
+ receipt-verified effects
+ repository sovereignty
```

The operator strategy is:

```text
one portal
+ honest state
+ live sessions without terminal babysitting
+ explainable routing
+ central quota
+ visible struggle and escalation
+ full causal audit
```

Build the permanent-fence, private-clone, exact-Candidate, structured-provider, Context Capsule, quota-reservation, behavior-enforcement, and Effect-reconciliation kernel first. Once that survives fault injection, model fusion and broad parallelism become controlled acceleration instead of amplified confusion.

# 40. Reference algorithms

The pseudocode in this section is normative enough to guide implementation. Error handling must preserve the listed state transitions and idempotency behavior.

## 40.1 Task classification

```rust
pub async fn classify_task(
    ctx: &RequestContext,
    task: &CognitiveTask,
    repo: &RepositorySnapshot,
    policy: &RoutingPolicy,
) -> Result<TaskClassification> {
    let declared = task.declared_classification.clone();
    let deterministic = policy_engine::classify_paths_and_contract(
        &task.objective,
        &task.completion_contract,
        repo,
    )?;

    let input = ClassificationInput::from(task, repo, &deterministic);
    let cheap = classifier_lane.invoke_structured(input.clone()).await?;
    schema::validate(&cheap)?;

    let combined = reconcile_classification(declared, deterministic, cheap)?;
    let contradiction = classification_contradiction_score(&combined);

    let final_classification = if combined.confidence < policy.min_classifier_confidence
        || contradiction > policy.max_classifier_contradiction
    {
        let second = router
            .select_independent_classifier(task, &combined)
            .await?;
        let second_result = second.invoke_structured(input).await?;
        classification_fuser::resolve(combined, second_result, policy).await?
    } else {
        combined
    };

    let hardened = risk_policy::apply_floors(final_classification, repo, task)?;
    ledger.store_classification(task.id, &hardened).await?;
    Ok(hardened)
}
```

## 40.2 Lane routing

```rust
pub async fn route(
    task: &CognitiveTask,
    class: &TaskClassification,
    state: &RoutingState,
) -> Result<DispatchDecision> {
    let all = model_registry.active_lanes().await?;
    let mut eligible = Vec::new();
    let mut excluded = Vec::new();

    for lane in all {
        match eligibility::evaluate(task, class, &lane, state).await? {
            Eligibility::Eligible(features) => eligible.push((lane, features)),
            Eligibility::Excluded(reasons) => excluded.push((lane.id, reasons)),
        }
    }

    if eligible.is_empty() {
        return Err(Problem::NoEligibleLane { excluded });
    }

    let mut scored = Vec::new();
    for (lane, features) in eligible {
        let forecast = outcome_model.forecast(task, class, &lane, &features)?;
        let quota = quota_service.snapshot_and_shadow_price(&lane).await?;
        let cost = cost_model.estimate(task, &lane, &features)?;
        let utility = objective::score(&forecast, &quota, &cost, task, class, state);
        scored.push(ScoredLane { lane, features, forecast, quota, cost, utility });
    }

    scored.sort_by(descending_utility_then_stable_id);
    let strategy = collaboration_admission::choose(task, class, &scored, state)?;
    let reservations = reservation_service.reserve(strategy.forecasts()).await?;

    let decision = DispatchDecision::new(
        task,
        class,
        scored,
        excluded,
        strategy,
        reservations,
        scheduler_version(),
    );

    ledger.insert_dispatch_decision(&decision).await?;
    Ok(decision)
}
```

## 40.3 Economy cascade

```rust
pub async fn run_cascade(
    task: CognitiveTask,
    ladder: Vec<LaneSpec>,
) -> Result<CognitiveArtifact> {
    let mut prior = None;

    for lane_spec in ladder {
        let lane = router.resolve_eligible_lane(&task, lane_spec).await?;
        let reservation = quota.reserve_for_task(&task, &lane).await?;
        let output = invoke(task.clone(), lane, prior.as_ref()).await;
        quota.settle(reservation, output.usage()).await?;

        match output? {
            InvocationOutput::Complete(artifact) => {
                validate_task_completion(&task, &artifact).await?;
                return Ok(artifact);
            }
            InvocationOutput::Escalate(reason, artifact) => {
                prior = Some(EscalationContext { reason, artifact });
            }
            InvocationOutput::NeedContext(request) => {
                let capsule = context.compile_expanded(&task, request).await?;
                prior = Some(EscalationContext::context(capsule));
            }
            InvocationOutput::PolicyBlocked(problem) => return Err(problem),
        }
    }

    Err(Problem::EscalationLadderExhausted)
}
```

## 40.4 Fusion admission

```rust
pub fn admit_fusion(
    task: &CognitiveTask,
    class: &TaskClassification,
    lanes: &[ScoredLane],
    policy: &FusionPolicy,
) -> FusionAdmission {
    if !policy.task_classes.contains(&class.primary_class) {
        return FusionAdmission::No("task class not enabled");
    }
    if task.budget.remaining < policy.minimum_budget {
        return FusionAdmission::No("insufficient budget");
    }
    if task.deadline.slack < policy.minimum_latency_slack {
        return FusionAdmission::No("deadline");
    }

    let pair = best_diverse_pair(lanes, policy.minimum_diversity)?;
    let uplift = fusion_model.predict_uplift(task, class, &pair);
    let total_cost = pair.expected_cost + policy.expected_fuser_cost;
    let value = uplift.probability * task.business_value
        - total_cost
        - policy.latency_cost
        - policy.contamination_risk;

    if value > policy.minimum_expected_value {
        FusionAdmission::Yes { pair, expected_value: value }
    } else {
        FusionAdmission::No("non-positive expected value")
    }
}
```

## 40.5 Progress and struggle evaluation

```rust
pub async fn evaluate_struggle(attempt: AttemptId) -> Result<StruggleEvaluation> {
    let window = progress_store.recent_window(attempt, STRUGGLE_WINDOW).await?;
    let features = StruggleFeatures {
        time_without_progress: window.time_since_meaningful_progress(),
        repeated_tools: window.identical_tool_rate(),
        repeated_failures: window.same_failure_rate(),
        context_saturation: session.context_saturation(attempt).await?,
        scope_churn: window.scope_churn(),
        plan_churn: window.plan_churn(),
        contradiction_growth: window.contradiction_growth(),
        low_confidence_rate: window.low_confidence_rate(),
        behavior_rate: window.behavior_warning_rate(),
        quota_risk: quota.attempt_risk(attempt).await?,
        acceptance_delta: window.acceptance_delta(),
        verified_test_delta: window.verified_test_delta(),
        candidate_quality_delta: window.candidate_quality_delta(),
    };

    let critical = critical_triggers::evaluate(attempt, &window).await?;
    let score = struggle_model.transparent_score(&features);
    let state = struggle_policy.classify(score, &critical);

    let eval = StruggleEvaluation::new(attempt, features, critical, score, state);
    ledger.insert_struggle_evaluation(&eval).await?;

    if state.requires_action() {
        escalation_service.plan_and_apply(&eval).await?;
    }
    Ok(eval)
}
```

## 40.6 Escalation selection

```rust
pub async fn choose_escalation(
    eval: &StruggleEvaluation,
    attempt: &Attempt,
) -> Result<EscalationDecision> {
    let candidates = vec![
        Action::TargetedContext,
        Action::RaiseEffort,
        Action::PromoteTier,
        Action::AddCritic,
        Action::AddTestArchitect,
        Action::FreshSessionSameLane,
        Action::MigrateProvider,
        Action::Decompose,
        Action::Fusion,
        Action::HumanIntervention,
        Action::Quarantine,
    ];

    let eligible = policy.filter_escalations(candidates, eval, attempt).await?;
    let scored = eligible
        .into_iter()
        .map(|a| escalation_model.score(a, eval, attempt))
        .collect::<Result<Vec<_>>>()?;

    let selected = select_max_expected_value(scored)?;
    let decision = EscalationDecision::from(selected, eval, attempt);
    ledger.insert_escalation_decision(&decision).await?;
    apply_escalation_idempotently(&decision).await?;
    Ok(decision)
}
```

## 40.7 Capsule compilation

```rust
pub async fn compile_capsule(
    task: &CognitiveTask,
    target: &Lane,
    budget: TokenBudget,
) -> Result<ContextCapsule> {
    let mandatory = context_graph.mandatory_nodes(task).await?;
    let relevant = context_graph.retrieve_relevant(task, target).await?;
    let resolved = context_graph.resolve_supersession_and_contradictions(
        mandatory.union(relevant)
    )?;

    let mut content = renderer.render_nodes(target, &resolved)?;
    if tokenizer.estimate(target.model_snapshot, &content) > budget.max {
        let job = compression_service.create(task, target, resolved, budget).await?;
        let compressed = compression_service.run_and_verify(job).await?;
        content = renderer.render_compression(target, &compressed)?;
    }

    let coverage = capsule_validator.coverage(task, &content)?;
    if !coverage.meets_policy(task.context_policy) {
        return Err(Problem::InsufficientContextCoverage { coverage });
    }

    let capsule = ContextCapsule::new(task, target, content, coverage);
    ledger.insert_context_capsule(&capsule).await?;
    Ok(capsule)
}
```

## 40.8 Context transition

```rust
pub async fn transition_context(
    session: AgentSessionId,
    reason: ContextTransitionReason,
) -> Result<ContinuityTransition> {
    let command = idempotency.begin("context-transition", session, &reason).await?;
    if let Some(prior) = command.completed_result() {
        return Ok(prior);
    }

    session_supervisor.freeze_new_turns(session).await?;
    session_supervisor.reach_safe_boundary_or_interrupt(session).await?;
    let checkpoint = runner.checkpoint(session).await?;
    event_importer.drain_provider_events(session).await?;
    context_graph.update_from_session(session).await?;

    let task = session_store.active_task(session).await?;
    let target = router.select_continuity_lane(&task, session, reason).await?;
    let capsule = compile_capsule(&task, &target, target.context_budget()).await?;
    let successor = session_supervisor.start_successor(target, checkpoint, capsule).await?;
    continuity_validator.verify(successor, &task).await?;
    session_supervisor.terminate_predecessor(session).await?;

    let transition = ContinuityTransition::complete(session, successor, reason);
    idempotency.complete(command, &transition).await?;
    Ok(transition)
}
```

## 40.9 Behavior enforcement

```rust
pub async fn authorize_tool_request(
    req: ToolRequest,
    facts: &AttemptFacts,
) -> Result<ToolAuthorization> {
    authority::verify(&req.authority_token, facts).await?;
    let parsed = command_parser.parse(&req)?;
    let observations = behavior_engine.evaluate_pre_action(&parsed, facts).await?;

    let highest = observations.max_severity();
    match highest.action {
        EnforcementAction::Allow => ToolAuthorization::allow(parsed),
        EnforcementAction::Rewrite => ToolAuthorization::rewrite(parsed, highest.rewrite),
        EnforcementAction::RequestScope => {
            scope_service.pause_and_request(facts.attempt_id, highest.scope).await?;
            ToolAuthorization::deny("scope amendment required")
        }
        EnforcementAction::Block => ToolAuthorization::deny(highest.reason),
        EnforcementAction::Pause => {
            attempt_service.pause(facts.attempt_id, highest.reason).await?;
            ToolAuthorization::deny("attempt paused")
        }
        EnforcementAction::Terminate => {
            session_supervisor.terminate(facts.session_id).await?;
            ToolAuthorization::deny("session terminated")
        }
        EnforcementAction::Quarantine => {
            attempt_service.quarantine(facts.attempt_id, highest.reason).await?;
            ToolAuthorization::deny("attempt quarantined")
        }
    }
}
```

## 40.10 Candidate completion

```rust
pub async fn evaluate_completion(attempt: AttemptId) -> Result<CompletionResult> {
    authority::require_current_attempt(attempt).await?;
    let wp = work_store.for_attempt(attempt).await?;
    let workspace = workspace_service.full_manifest(attempt).await?;

    require!(workspace.within_scope);
    require!(workspace.no_unclassified_untracked);
    require!(workspace.runtime_artifacts_excluded);
    require!(workspace.no_open_mutating_processes);

    let candidate = candidate_service.prepare(attempt, &workspace).await?;
    let gates = gate_service.required_for(&wp, &candidate).await?;
    let writer_results = gate_service.run_writer_gates(&candidate, gates.writer).await?;

    if !writer_results.all_pass() {
        return Ok(CompletionResult::RepairRequired(writer_results));
    }

    ledger.transition_attempt_to_prepared(attempt, candidate.id).await?;
    Ok(CompletionResult::CandidatePrepared(candidate))
}
```

---

# 41. Event catalog

Events are immutable, typed, sequenced, and schema-versioned.

## 41.1 Control and graph

```text
MissionCreated
AcceptanceContractCreated
PlanProposed
PlanValidated
PlanMaterialized
GraphDeltaProposed
GraphDeltaApplied
WorkPackageReady
WorkPackageBlocked
SelectionGroupCreated
VariantCreated
AttemptLeased
AttemptHeartbeat
AttemptPaused
AttemptSuperseded
AttemptTerminated
LeaseExpired
ScopeAmendmentRequested
ScopeSuccessorGranted
```

## 41.2 Cognitive

```text
CognitiveTaskCreated
TaskClassified
DispatchDecided
QuotaReserved
BudgetReserved
InvocationStarted
InvocationEventReceived
InvocationCompleted
InvocationFailed
CollaborationRunStarted
FusionCandidateReady
FusionRanked
FusionCompleted
FusionUnresolved
```

## 41.3 Progress and escalation

```text
ProgressObserved
StruggleEvaluated
TaskWatchRaised
TaskStalled
EscalationProposed
EscalationApplied
TaskDecomposed
InterventionOpened
InterventionResolved
```

## 41.4 Context

```text
ContextNodeAdded
ContextNodeSuperseded
ContradictionRecorded
CapsuleCompiled
CompressionStarted
CompressionValidated
CompressionRejected
ContextTransitionStarted
SuccessorSessionStarted
ContinuityVerified
ContextTransitionCompleted
```

## 41.5 Session and terminal

```text
AgentSessionCreated
ProviderIdentityVerified
ProviderIdentityMismatch
SessionReady
TurnStarted
ToolRequested
PermissionRequested
PlanProposed
LocalPlanApproved
SteeringQueued
SteeringAcknowledged
SteeringApplied
InterruptRequested
InterruptAcknowledged
ContextAtRisk
SessionUnresponsive
ProcessTreeTerminated
TerminalFrameStored
DialogObserved
DialogResolved
```

## 41.6 Behavior/workspace

```text
BehaviorObserved
BehaviorBlocked
BehaviorRemediationStarted
BehaviorRemediated
CandidateQuarantined
WorkspaceCreated
WorkspaceManifestUpdated
WorkspaceCheckpointed
WorkspacePreservationVerified
WorkspaceCleanupAuthorized
WorkspaceDeleted
```

## 41.7 Proof and effects

```text
CandidatePrepared
EvidenceStarted
EvidenceAccepted
EvidenceInvalidated
ReviewStarted
ReviewFindingCreated
ReviewAccepted
VariantSelected
EffectProposed
EffectAuthorized
EffectDispatched
EffectOutcomeUnknown
EffectVerified
EffectFailed
IntegrationQueued
MergeGroupStarted
Integrated
ObservationStarted
OutcomeSurvived
OutcomeReverted
```

## 41.8 Capacity

```text
ProfileCreated
AuthChallengeCreated
ProfileIdentityVerified
QuotaObserved
QuotaReservationCreated
QuotaReservationSettled
ProfileThrottled
CircuitBreakerOpened
CircuitBreakerHalfOpened
CircuitBreakerClosed
BudgetThresholdReached
```

## 41.9 Audit requirements

Every event must include or derive:

- actor/principal/service;
- organization/repository;
- Mission;
- Work Package/Variant/Attempt when applicable;
- Cognitive Task/Invocation when applicable;
- Authority Token hash;
- policy/config/routing versions;
- causation/correlation/trace;
- artifact references;
- timestamp from authoritative emitter.

Unknown actor or missing correlation on a sensitive mutation is an audit defect.

---

# 42. Provider adapter conformance

Every adapter version must pass the following.

## 42.1 Initialization

- binary/version detection;
- capability declaration;
- profile identity;
- model catalog;
- structured transport;
- PTY requirement;
- workspace startup;
- sanitized environment.

## 42.2 Turns

- simple turn;
- long streaming turn;
- structured output;
- tool call;
- permission request;
- plan mode;
- mid-turn steering;
- cancel;
- timeout;
- malformed event;
- duplicate event;
- delayed event.

## 42.3 Lifecycle

- fresh session;
- native resume;
- fresh successor with Context Capsule;
- provider crash;
- runner crash;
- adapter restart;
- context limit;
- compact/clear transition;
- auth expiry;
- login challenge;
- quota throttle;
- model unavailable;
- provider version change.

## 42.4 Usage and quota

- usage event parsing;
- no double counting;
- cache/reasoning dimensions;
- rate-limit observation;
- exact source/confidence;
- reservation settlement;
- unknown handling.

## 42.5 Security

- effective profile mismatch;
- ambient credential absent;
- no SCM credential;
- no host HOME;
- network allowlist;
- tool request mediation;
- terminal injection;
- provider output treated untrusted.

## 42.6 Cross-platform

- Linux;
- macOS;
- Windows/ConPTY;
- WSL where supported;
- path/quoting;
- process-tree cleanup;
- atomic state;
- session mapping.

Promotion stages:

```text
DEVELOPMENT
→ CONTRACT_PASS
→ SYNTHETIC_CANARY
→ INTERNAL_CANARY
→ LIMITED
→ GENERAL
```

Regression automatically rolls back traffic to the prior adapter snapshot.

---

# 43. Security threat model

## 43.1 Adversaries and failures

Centerrail assumes:

- repository content may be malicious;
- tests/build scripts may be malicious;
- provider output may be incorrect or prompt-injected;
- model may take shortcuts;
- provider session may be compromised or misidentified;
- user-supplied MCP server may be malicious;
- runner may crash or partition;
- control messages may duplicate or delay;
- external effect may succeed while response is lost;
- portal content may contain XSS;
- profile credential may expire or be revoked;
- humans may make mistakes or attempt unauthorized actions.

A fully compromised authorized host/root can defeat process-only isolation. Strong workloads require separate-kernel/dedicated boundaries.

## 43.2 Assets

- source code;
- uncommitted work;
- SCM authority;
- provider credentials;
- cloud/production credentials;
- KMS keys;
- hidden tests;
- acceptance contracts;
- policy and audit;
- customer data;
- model quota and budget;
- Candidate/Evidence integrity.

## 43.3 Trust boundaries

- browser ↔ API;
- control plane ↔ ledger;
- control plane ↔ runner;
- provider enclave ↔ tool sandbox;
- runner ↔ repository;
- verifier ↔ author artifacts;
- Effect Broker ↔ external provider;
- object store ↔ consumers;
- admin/profile owner ↔ organization policy.

## 43.4 Zero ambient authority

A task receives only:

- exact source;
- explicit tools;
- declared network destinations;
- bounded resources;
- authorized provider profile;
- explicit time/budget;
- no unrelated credential;
- no privileged effect authority.

## 43.5 Prompt injection controls

- permissions outside prompt;
- policy/risk not model-controlled;
- repository text cannot grant tools/network;
- no secrets in context without explicit grant;
- suspicious instructions create events;
- tool requests separately authorized;
- delivery scans exact tree;
- reviewers cannot waive checks.

## 43.6 Exfiltration controls

- egress allowlist/DLP;
- no Git push from sandbox;
- secret/entropy scan;
- binary/size anomaly scan;
- model context redaction;
- artifact visibility labels;
- provider data-policy eligibility;
- no cloud metadata/local discovery;
- audit of all external bytes where feasible.

## 43.7 Supply chain

- pinned signed releases;
- SBOM;
- dependency scanning;
- verified vendor binary hashes where possible;
- image digests;
- adapter snapshot;
- canary and rollback;
- reproducible builds for Centerrail;
- protected CI.

## 43.8 Portal

- OIDC/SAML;
- RBAC;
- CSRF;
- strict CSP;
- sanitization of terminal/Markdown/artifacts;
- no secrets in browser state;
- audited mutations;
- same-origin and Host validation;
- rate limits;
- session expiration.

## 43.9 Audit tamper evidence

Periodically:

1. hash ordered audit batch;
2. construct Merkle root;
3. sign with audit key;
4. export root to independent storage;
5. verify during incident and backup restore.

---

# 44. Operational recovery runbooks

## 44.1 Provider quota exhaustion

```text
stop new turns
→ mark profile/model dimension exhausted
→ checkpoint affected sessions
→ build continuity capsules
→ preserve workspace
→ choose wait or authorized migration
→ reserve target capacity
→ resume
→ reconcile usage
```

## 44.2 Context freeze

```text
detect structured/screen context state
→ interrupt if possible
→ checkpoint
→ import transcript/events
→ compile verified capsule
→ start successor
→ verify continuity
→ terminate predecessor
```

If predecessor cannot invoke a handoff command, Centerrail reconstructs continuity from canonical state and workspace; it never depends on the frozen model.

## 44.3 Runner partition

```text
runner misses control renewal
→ local self-fence
→ freeze tool gateway
→ terminate children by deadline
→ server expires runner
→ expire Attempt lease
→ salvage/checkpoint if filesystem reachable
→ issue successor fence
```

## 44.4 Ambiguous GitHub push

```text
Effect timeout
→ OUTCOME_UNKNOWN
→ query exact remote ref
→ compare expected old/new OIDs and effect key
→ adopt if remote matches
→ retry only if non-execution proven
→ quarantine contradiction
```

## 44.5 Cleanup uncertainty

```text
cleanup requested
→ snapshot state
→ verify ownership nonce
→ inspect processes/open files/Git/preservation
→ if any UNKNOWN: deny and reschedule
→ if safe: delete
→ verify absence
→ tombstone completion
```

## 44.6 Database failover

- state changes commit with durable transaction;
- outbox rows replay;
- command IDs deduplicate;
- runner heartbeats retry against new primary;
- no fence counter rollback;
- projections rebuild;
- audit verifies sequence continuity.

## 44.7 Adapter regression

```text
protocol error breaker opens
→ stop new dispatch
→ preserve sessions/workspaces
→ route only compatible continuations
→ roll back adapter snapshot
→ conformance replay
→ canary before reopen
```

## 44.8 Behavior quarantine

```text
block action
→ pause Attempt
→ checkpoint
→ capture evidence
→ deterministic remediation if safe
→ independent scan
→ resume, replan, or terminate
```

---

# 45. V1 product boundary

The full domain supports the complete architecture, but the first generally available release must remain disciplined.

## Included in V1

- Linux reference runner;
- macOS/Windows operator portal and development runner;
- one repository and target branch per Mission;
- local Rust control plane and SQLite;
- CAS artifacts;
- provider simulator;
- all four provider adapters by GA, promoted sequentially;
- task taxonomy;
- D0/M1/M2/M3 routing;
- quota/budget observations and reservations;
- cascade and shadow;
- triad fusion for non-code cognitive tasks;
- planning council;
- permanent fences and leases;
- private clones;
- successor scope;
- Context Graph/Capsules/compression/migration;
- session supervisor and PTY/ConPTY view;
- behavior catalog;
- exact Candidates;
- clean verifier;
- Evidence/Review/Proof Bundle;
- GitHub-only Effect Broker;
- protected PR/check/merge queue;
- all core portal views.

## Deferred from V1

- production deployments;
- package publication;
- production data migrations;
- general cloud-resource effects;
- cross-repository atomicity;
- large code races as default workflow;
- online learned router authority;
- unrestricted MCP marketplace;
- same-file multiwriter editing;
- anonymous consumer profile pooling;
- enterprise microservice decomposition;
- universal macOS/Windows hardened sandbox equivalence.

The deferred features remain represented in the domain so V1 does not require a rewrite.

---

# 46. Research basis and engineering interpretation

The design is informed by, but does not blindly copy:

- dynamic strong/weak model routing research;
- rank-and-fuse ensembling;
- mixture-of-agents proposer/aggregator patterns;
- prompt/context compression research;
- provider official structured automation surfaces;
- Gastown’s cost, model-escalation, context, telemetry, checkpoint, and native-worker work;
- Gas City’s incident corpus and authority/effect failures;
- established distributed-systems fencing, leases, outbox, idempotency, and saga patterns.

Engineering interpretation:

1. Model routing is useful only with repository/task calibration and policy constraints.
2. Fusion can outperform individuals, but it can also add cost and synthesize errors; ranking, selection, and refusal are first-class.
3. Context compression must preserve provenance and mandatory information.
4. Provider structured APIs reduce terminal fragility but do not replace Centerrail’s authority kernel.
5. Provider quotas differ and must remain vector-valued with source/confidence.
6. Cheap models require **more** mechanical workflow enforcement, not more trust.
7. No research result removes the need for exact Candidate verification and repository rules.

# 47. Canonical task recipes

Recipes are policy defaults compiled into workflow IR. They are not prompt-only conventions.

## 47.1 Summarization

**Purpose:** Create a bounded, source-linked summary.

```text
class: summarize_local
default: M1
inputs: immutable source manifest
output: SummaryArtifact
checks:
  - schema;
  - source-reference validity;
  - required-section coverage;
  - unsupported-claim scan.
escalate:
  - coverage below threshold;
  - contradictions;
  - sensitive/high-risk source.
```

Summary schema:

```text
purpose
key_facts[]
decisions[]
risks[]
open_questions[]
source_references[]
omissions[]
confidence
```

The source remains in CAS. Summary cannot become authoritative Evidence by itself.

## 47.2 Context compression

```text
M1 compressor
→ coverage validator
→ independent checker when normal/high risk
→ publish Context Capsule
```

Mandatory coverage is 100% for applicable acceptance, scope, policy, blocker, and Effect nodes. A high compression ratio is not success if coverage falls.

## 47.3 Repository reconnaissance

```text
partition source/index by package and acceptance concern
→ 4–12 M1 read-only scouts
→ each emits RepositoryMapFragment with exact references
→ D0 deduplication
→ M2 reducer
→ optional M3 gap review
```

No scout receives write authority. Findings become proposed Context nodes until validated.

## 47.4 Planning

Routine bounded task:

```text
M2 plan
→ deterministic graph/scope/acceptance validation
```

High-value ambiguous task:

```text
M3 proposal A
+ M3 proposal B from independent family
→ contradiction extraction
→ M3 fuser C
→ deterministic Plan validator
→ optional human/domain gate by risk
```

The fuser cannot invent acceptance coverage that neither proposal supports.

## 47.5 Bounded bug fix

```text
D0/M1 reproduce and classify
→ M2 writer
→ local targeted tests
→ exact Candidate
→ E2 verifier
→ M2 blind review
```

Escalate when:

- reproduction remains nondeterministic;
- same failure repeats twice;
- scope expands beyond threshold;
- hidden test reveals conceptual misunderstanding;
- risk path raises class.

## 47.6 Feature implementation

```text
repository map
→ plan
→ partition Work Packages by Change Intent
→ M2 writers
→ M3 only for difficult package or architecture blocker
→ clean verification
→ independent review
→ merge queue
```

Do not run a frontier model for every small implementation turn when M2 is calibrated.

## 47.7 Architecture design

```text
independent source map
→ two or three M3 proposals
→ explicit tradeoff matrix
→ adversarial critic
→ provenance-preserving fusion
→ decision record
→ implementation Plan
```

Output must identify rejected alternatives and reversal conditions.

## 47.8 Security analysis

```text
threat model builder
+ adversarial attacker model
+ repository/control-flow scout
→ finding deduplication
→ M3/M4 judge
→ deterministic scanners
→ E3 where required
```

Security fusion requires independent provider/model lineage and no author-only proof.

## 47.9 Migration design

```text
schema/data map
→ forward plan
→ rollback/compensation plan
→ invariant checker
→ dry-run against copy
→ adversarial failure injection
→ human gate
```

No production migration effect in V1.

## 47.10 Performance work

When multiple algorithmic options exist:

```text
baseline benchmark
→ independent hypotheses
→ isolated code Variants
→ identical benchmark environment
→ statistical comparison
→ selector or synthesis Variant
→ regression suite
```

Optimization without a stable benchmark is rejected.

## 47.11 Code review

```text
D0 diff metadata and risk map
→ M2 blind semantic reviewer
→ M3 second reviewer for R3 or low confidence
→ deterministic gates
→ finding deduplication
```

Reviewers cite exact hunks/symbols/tests/requirements. A generic summary is invalid.

## 47.12 Test authoring

```text
test architect receives acceptance and source, not implementation rationale
→ proposes failure cases
→ tests stored outside writer authority when holdout
→ writer sees allowed subset
→ verifier runs all
```

## 47.13 Incident reproduction

```text
log/source partition scouts
→ timeline builder
→ competing root-cause hypotheses
→ minimal reproducer attempts
→ evidence-ranked synthesis
→ fix Work Packages
```

A hypothesis is not promoted to fact without a receipt or reproducible evidence.

## 47.14 UI validation

```text
browser task in isolated profile
→ semantic DOM/accessibility checks
→ screenshots/video artifacts
→ visual model review only where deterministic assertions are insufficient
→ exact Candidate binding
```

## 47.15 Completion assessment

```text
D0 contract and ledger evaluation
→ independent M2 semantic check only for non-mechanical clauses
→ M3 arbiter on contradiction
```

The writer does not decide completion.

## 47.16 Struggle diagnosis

```text
deterministic feature snapshot
→ M1/M2 independent observer
→ suggested remedy with evidence
→ policy-selected escalation
```

The observer has read-only access and cannot modify the Attempt.

## 47.17 Human decision brief

For an unavoidable domain decision:

```text
facts and exact evidence
→ two independent option analyses where high value
→ fused tradeoff brief
→ explicit decision options
→ consequence/risk/cost/evidence invalidation
→ authorized human decision
```

Human attention is used for judgment, not terminal mechanics.

---

# 48. Front-end implementation architecture

## 48.1 Technology

- Vite;
- React;
- TypeScript strict mode;
- generated OpenAPI client;
- generated event types;
- TanStack Query or equivalent for request lifecycle;
- a normalized client read model;
- virtualization for large tables/graphs;
- Web Workers for heavy graph/diff processing;
- terminal renderer with strict sanitization;
- accessible component library;
- no business-authority logic in browser.

## 48.2 State model

The browser stores:

```text
selected organization/repository/Mission
projection snapshots with as_of_sequence
pending commands keyed by command_id
SSE cursor
ephemeral UI preferences
terminal viewport buffers
```

It does not store:

- secrets;
- provider credentials;
- authoritative workflow state;
- hidden test content;
- unredacted sensitive logs unless current role is authorized.

## 48.3 Event ingestion

```text
initial typed snapshot
→ open SSE after snapshot sequence
→ deduplicate event_id
→ require monotonic sequence
→ detect gap
→ pause projection apply
→ fetch replay from last confirmed sequence
→ resume
```

Every page exposes projection lag and source health.

## 48.4 Command UX

```ts
const result = await commands.create({
  commandId: uuidv7(),
  type: "SESSION_INTERRUPT",
  target,
  request,
});

pendingCommands.add(result.commandId);
```

UI renders:

```text
Interrupt requested
```

Only `CommandCompleted` or refreshed state can render:

```text
Interrupted
```

Failed/unknown commands remain visible with recovery guidance.

## 48.5 Graph rendering

Mission Graph uses semantic zoom:

- level 1: Mission/Plan/Work Package critical path;
- level 2: Selection Groups/Variants/Attempts;
- level 3: Cognitive Tasks/Invocations;
- level 4: Evidence/Effects/events.

Use server-side aggregation and client virtualization. Do not ship thousands of raw terminal events into graph layout.

## 48.6 Live terminal

- WebSocket carries framed binary/text terminal updates;
- server sanitizes control sequences and enforces read authorization;
- client renderer does not interpret hyperlinks/scripts;
- input channel disabled unless an audited attach command grants it;
- structured event timeline is primary view.

## 48.7 Diff view

Features:

- exact base/head;
- path intent status;
- behavior annotations;
- generated/runtime classification;
- test impact;
- reviewer findings;
- contributor Variant comparison;
- fusion provenance;
- line-level Evidence links.

## 48.8 Router and Fusion visualizations

Router:

- classification chips;
- hard exclusions;
- utility decomposition;
- quota shadow price;
- quality calibration;
- economy-share target;
- fallback.

Fusion:

- contributor columns;
- source/citation links;
- agreement/disagreement graph;
- rank;
- selected/fused content;
- residual uncertainty;
- benchmark outcome.

## 48.9 Context view

- node graph;
- source references;
- active capsule;
- mandatory coverage;
- compression omissions;
- contradictions;
- migration history;
- raw source retrieval with permission checks.

## 48.10 Behavior view

- real-time violations;
- rule details;
- action evidence;
- workspace location;
- remediation;
- model/provider cohort trends;
- exception workflow.

## 48.11 Accessibility

- keyboard operation;
- WCAG 2.2 AA target;
- non-color status indicators;
- screen-reader labels for graph/table state;
- terminal alternative text/log view;
- reduced-motion mode;
- focus preservation under live updates.

## 48.12 Front-end tests

- generated type sync;
- reducer/event property tests;
- delayed SSE;
- sequence gaps;
- command pending semantics;
- role permissions;
- XSS fixtures;
- large graph;
- terminal;
- accessibility;
- visual regression.

---

# 49. Upgrade and compatibility strategy

## 49.1 Schema migrations

- forward-only migration files;
- checksum and ordered version;
- transactional where database supports;
- preflight backup;
- compatibility window between binary and schema;
- no ambient CLI tool independently migrating authoritative schema;
- rollback through restore or explicitly supported down migration;
- migration Evidence.

## 49.2 Configuration generations

A complete immutable generation includes:

```text
organization policy
repository policy
routing policy
fusion policy
behavior catalog
task taxonomy
model snapshots
profile metadata
prompt templates
workflow IR
sandbox images
tool registry
adapter versions
evaluator suites
```

Activation:

```text
validate
→ stage
→ dry-run affected objects
→ atomically mark generation active
→ components acknowledge generation
→ drain/restart incompatible sessions
→ last-known-good retained
```

Partial configuration activation is forbidden.

## 49.3 Adapter upgrades

- side-by-side versions;
- conformance;
- recorded protocol fixtures;
- canary percentage;
- compare errors, completion, usage, and behavior;
- automatic rollback;
- session compatibility declaration.

## 49.4 Model changes

A provider model alias changing behavior creates a new Model Snapshot. Routing calibration does not transfer silently. New snapshot begins in shadow/canary according to policy.

## 49.5 Prompt changes

Prompt templates are immutable and hashed. Outcome analytics include prompt version. A prompt change uses champion/challenger rollout, not immediate fleet-wide replacement.

## 49.6 Behavior catalog changes

Rules have semantic versions. Breaking enforcement changes require:

- migration note;
- dry-run report;
- expected incidence;
- canary;
- exception compatibility;
- rollback.

## 49.7 API compatibility

- versioned `/v1`;
- additive changes preferred;
- generated clients;
- explicit deprecation;
- compatibility tests;
- event payload versions;
- raw provider frames remain separately versioned.

## 49.8 Backup and restore

Back up:

- PostgreSQL/SQLite ledger;
- CAS objects and manifests;
- audit roots/keys;
- configuration generations;
- Git mirror metadata only as cache;
- KMS references, not plaintext secrets.

Restore test verifies:

- fence counters;
- commands;
- active/expired leases;
- outbox;
- Candidate/Evidence;
- effect ambiguity;
- projection rebuild;
- audit continuity.

---

# 50. Acceptance test matrix

| Requirement | Test |
|---|---|
| one writer per Variant | concurrent lease property/chaos |
| permanent fence | delete/recreate/restore sequence |
| stale Attempt blocked | delayed message/effect suite |
| atomic graph | crash at every insert boundary |
| economy routing | calibrated task corpus |
| 70–90% target | portfolio simulation without quality-floor violation |
| fusion | hidden benchmark and forced-fusion negative cases |
| quota | source/freshness/reservation/settlement tests |
| escalation | synthetic stagnation and recovery cases |
| context migration | full provider-pair matrix |
| compression | mandatory coverage/adversarial corpus |
| bad behavior | rule positive/negative fixtures |
| temp/workspace hygiene | nested copy/runtime artifact/Candidate scan |
| PTY control | Unix/ConPTY golden dialogs |
| no routine terminal approvals | end-to-end unattended sessions |
| process cleanup | owned-tree fault tests |
| sandbox | escape/network/credential tests |
| Git safety | hostile config/filter/hook/submodule tests |
| scope successor | atomic transfer and stale predecessor |
| exact Candidate | base/head/tree/patch fixtures |
| independent proof | author exclusion and clean reconstruction |
| Effect ambiguity | remote success/local timeout simulation |
| GitHub protection | App token, ref readback, merge queue |
| portal truth | pending commands/lag/unknown/contradiction |
| audit | sensitive-action completeness and Merkle root |
| recovery | runner/control/database/provider failures |
| cross-platform | Linux/macOS/Windows path/process/terminal |
| observation | revert/incident/survival outcome |

The release gate fails if any kernel test is quarantined or marked flaky.

# 51. Provider capability snapshot at specification freeze

This snapshot informs the first adapters. It is not a permanent capability contract; each adapter pins and probes the installed version.

## 51.1 OpenAI Codex

Current official opportunities:

- Codex app-server is a local JSON-RPC control surface.
- The default stdio transport uses newline-delimited JSON.
- It exposes threads, turns, approvals, history, streamed events, account/auth state, and rate-limit information.
- Local model choice currently includes a tiered GPT-5.6 family: Luna for repeatable/high-volume work, Terra as a general workhorse, and Sol for complex/high-detail work.
- Model and reasoning choices can be inspected and changed through structured or documented local surfaces.
- Experimental remote/WebSocket features are not assumed production-stable.

Centerrail use:

```text
Luna → candidate M1 economy lane
Terra → candidate M2 standard lane
Sol → candidate M3 frontier lane
```

The mapping activates only after repository/task calibration.

## 51.2 Claude Code

Current official opportunities:

- headless programmatic mode;
- stream-JSON output;
- final result with response, cost, and session metadata;
- hooks with structured lifecycle inputs/outputs;
- Agent SDK with tools, hooks, subagents, MCP, and permissions;
- session start/resume and CLI control;
- usage/cost monitoring and organizational analytics;
- gateway support.

Centerrail use:

- bare/scripted mode for one-shot Cognitive Tasks where host configuration must not leak;
- structured streaming for sessions;
- hooks as deterministic integration notifications, not authority;
- canonical Context Capsule regardless of native resume;
- quota remaining stays unknown unless an authorized structured source supplies it.

## 51.3 Cursor Agent

Current official opportunities:

- headless print mode for automation;
- ACP over stdio JSON-RPC for custom clients;
- CLI on macOS, Linux, WSL, and native Windows;
- team/enterprise Admin and Analytics APIs for usage, spending, model access, and analytics;
- enterprise OpenTelemetry export for usage and cloud-agent logs.

Centerrail use:

- ACP for interactive sessions and control;
- headless for bounded one-shot tasks;
- enterprise APIs as quota/cost observations with explicit freshness;
- rules/project files treated as untrusted repository context unless policy approves them.

## 51.4 Google Antigravity

Current official opportunities:

- headless mode with machine-readable streaming;
- model and reasoning/effort selection;
- continuous prompts over stdin;
- quota/usage commands and credit controls;
- native status-line and CLI settings;
- structured operation caveats, including commands that must run outside the streaming prompt channel.

Centerrail use:

- NDJSON/headless adapter;
- separate typed adapter calls for `/usage`, `/model`, and other stream-breaking commands;
- model quota observations with source and timestamp;
- explicit policy on credit overages;
- no terminal injection of slash commands into an active NDJSON stream.

## 51.5 Capability asymmetry is expected

Centerrail must not flatten these differences into a fictional common denominator. The common contract carries:

- supported capability;
- limitations;
- transport;
- freshness;
- confidence;
- version;
- fallback.

Scheduling uses the intersection of task requirements and verified lane capabilities.

---

# 52. Gastown source crosswalk

| Gastown work | Centerrail interpretation |
|---|---|
| v1.0.1 per-role effort and cost tiers | seed for model-tier registry |
| v1.0.1 `model-escalation.json` | seed for transparent escalation policy |
| stuck-agent-dog | seed for independent progress observer |
| quota dog/account state | demonstrates central quota need; do not adopt consumer-profile pooling |
| context-budget guard and issue #3906 | control plane must transition context before freeze |
| distributed formula PR #1553 | fresh sessions and artifacts beat one giant context; generalize with Context Graph |
| telemetry PR #2068 | normalized provider events and usage are valuable; make durable and cross-provider |
| cost-learning PR #1729 | correlate cost with outcome; extend to exact task/Candidate/survival |
| OpenCode server PR #4706 | structured native worker, durable idempotent nudges, restart/resume are the right direction |
| issue #2386 fake patrol cycles | lower models require machine-enforced step receipts |
| issue #4722 provider hook leakage | runtime files must stay outside product repo |
| issue #4397 unsafe nuke | preservation/read-back is a hard cleanup precondition |
| current worktree/session incidents | private clones and immutable incarnation identity |
| ad hoc triple-model reviews | promote into typed fusion/council protocols |

No reviewed Gastown release or active work provides the complete Centerrail combination of:

- task taxonomy;
- calibrated minimum-sufficient-intelligence routing;
- central cross-provider quota vectors;
- first-class triad fusion;
- code-Variant synthesis;
- canonical provider-portable context;
- struggle-derived escalation;
- machine-enforced behavior catalog;
- structured-first session supervisor;
- transactionally fenced software-change authority.

---

# 53. Final score and readiness statement

| Area | Paper-design readiness |
|---|---:|
| transaction/fencing | 9.6/10 |
| model routing/quota | 9.3/10 |
| fusion/collaboration | 9.2/10 |
| context portability | 9.4/10 |
| behavior/workspace safety | 9.5/10 |
| session supervision | 9.2/10 |
| sandbox/cross-platform honesty | 9.1/10 |
| verification/effects | 9.6/10 |
| portal/operability | 9.3/10 |
| implementation maturity today | not implemented |

Overall paper architecture: **9.45/10**.

It does not receive 10 because:

- acceptance contracts can remain incomplete;
- models, reviewers, tests, and humans can share blind spots;
- provider protocols, prices, and quotas change;
- macOS/Windows local isolation has limits;
- external systems cannot always prove non-execution;
- learned routing requires real outcome data;
- fusion can amplify correlated error;
- behavior detectors can have false positives;
- portal usability needs real operator testing;
- no implementation has yet survived the specified fault/security/benchmark program.

That uncertainty is a design input, not a reason to weaken the architecture.

---
# Specification metadata

```text
freeze_date: 2026-08-24
word_count_before_metadata: 23134
line_count_before_metadata: 7497
sha256_before_metadata: 7fedf389d7e7a2af512dbaf16e879df12fd25f9abf936b37daeae53c4472b674
```
