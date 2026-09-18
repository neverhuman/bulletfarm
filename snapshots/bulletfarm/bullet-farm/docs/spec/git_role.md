# Executive conclusion

**A 100% Rust, GitHub-native, custom Git layer could become Bullet Farm’s deepest and most defensible technical advantage.** The multi-model scheduler is valuable, but models and harnesses will commoditize. An agent-first source-control substrate that understands authority, change identity, proof, lineage, failed attempts, semantic conflict, and protected integration would be much harder to replicate.

The decisive move is **not** to turn Git into the entire Bullet Farm database. The existing design is right to reject Git, Markdown, and task stores as the live control-plane ledger. Leases, fences, quota reservations, commands, runtime identity, and unsettled external effects belong in a transactional database. 

Instead, divide canonical truth deliberately:

```text
Bullet Ledger
  Canonical for mutable authority:
  leases, fences, commands, profiles, quota, policy activation,
  runner epochs, effect intents, effect settlement

BulletGit
  Canonical for immutable engineering truth:
  changes, revisions, candidates, lineage, checkpoints, scope,
  evidence, reviews, decisions, selections, integration subjects

GitHub
  Canonical for remote collaboration and final integration:
  refs, pull requests, checks, rulesets, merge queue, merged target SHA
```

This preserves Bullet Farm’s central software-change lifecycle—Mission, immutable Plan, fenced Attempt, exact Candidate, independent verification, protected integration, and survival—while moving enforcement closer to the lowest mutable layer. 

I would call the internal subsystem **BulletGit**, while keeping **Bullet Farm** as the overall platform.

---

# The 10 most profound insights

## 1. Do not create one universal source of truth; create three non-overlapping sovereign truths

A common architectural instinct would be: “Since we control Git, put the Mission, locks, leases, evidence, reviews, and agent state into Git.”

That would be a mistake.

Git is exceptional at immutable, content-addressed history. It is poor as a high-churn coordination database for:

* renewable leases;
* heartbeats;
* quota reservations;
* authentication state;
* unsettled remote effects;
* scheduler queues;
* transient runner observations;
* strongly serialized resource ownership.

The correct boundary is:

| Domain                        | Canonical system |
| ----------------------------- | ---------------- |
| Mutable operational authority | SQL ledger       |
| Immutable engineering history | BulletGit        |
| Remote integration state      | GitHub           |

This gives each system the state model it handles best. BulletGit can still cryptographically anchor selected ledger decisions—fence, policy snapshot, scope revision, Candidate identity—without becoming responsible for current liveness.

**Profound consequence:** Bullet Farm gains a formal answer to “where is truth?” without pretending that one storage technology is appropriate for every kind of truth.

---

## 2. A stable logical Change identity matters more than a better merge algorithm

A normal Git commit identifies one exact revision. The commit object records a tree, parent objects, author and committer information, and a message; it has no first-class stable identity for “the same logical change evolving through repair, rebase, or review.” ([Git][1])

Agent engineering needs two identities:

```text
ChangeId
  Stable identity of the engineering intention

CandidateId
  Exact immutable identity of one implementation revision
```

Example:

```text
Change chg_auth_refresh
├── Candidate C1: initial implementation
├── Candidate C2: repaired after verifier failure
├── Candidate C3: rebased onto updated main
└── Candidate C4: final merge-group composition
```

Jujutsu demonstrates the value of a stable Change ID that survives revisions to a change, but its Git interoperability documentation also illustrates why a nonstandard commit header is not a sufficient authoritative mechanism: not all Git operations preserve it reliably. ([JJ VCS][2])

BulletGit should therefore maintain stable Change identity in its own Merkle graph, not only in commit messages or trailers.

This single design unlocks:

* review continuity without stale-review mistakes;
* explicit rebase lineage;
* repair iteration tracking;
* Candidate comparison;
* best-of-(N) selection;
* stable analytics across rewritten commits;
* causal blame;
* precise evidence invalidation;
* meaningful revert and compensation.

**This is the most important new primitive.**

---

## 3. Agents should not “use Git”; they should call a capability-secure repository service

Giving an agent a shell containing `git` is too much ambient authority.

Even in a private clone, general Git exposes:

* arbitrary ref creation and rewriting;
* config and environment interpretation;
* hooks and filters;
* credential helpers;
* external diff and merge drivers;
* submodule transports;
* LFS behavior;
* filesystem-level repository internals.

BulletGit should instead run as a Rust daemon—`bullet-gitd`—and expose a typed capability API:

```rust
trait AgentRepository {
    async fn read_tree(&self, auth: &AuthorityToken, at: ObjectId)
        -> Result<TreeView>;

    async fn apply_change(&self, auth: &AuthorityToken, patch: PatchSet)
        -> Result<WorkspaceRevision>;

    async fn checkpoint(&self, auth: &AuthorityToken)
        -> Result<CheckpointId>;

    async fn request_scope(&self, auth: &AuthorityToken, request: ScopeRequest)
        -> Result<ScopeDecision>;

    async fn prepare_candidate(&self, auth: &AuthorityToken)
        -> Result<CandidateId>;

    async fn query_lineage(&self, id: ChangeId)
        -> Result<ChangeEvolution>;
}
```

The model process would receive no raw GitHub credential and no unrestricted ref-update capability. Every mutation would carry:

```text
Mission
Plan Revision
Work Package
Variant
Attempt
permanent fence
runner epoch
workspace nonce
scope revision
configuration snapshot
policy snapshot
```

That extends Bullet Farm’s existing rule that a display name, PID, terminal, provider thread, or path carries no authority. 

**Profound consequence:** the authority kernel moves below prompts and shell conventions into the repository mutation path itself.

---

## 4. Branches are human navigation aids; Variants and Selections should be the machine coordination model

A Git branch is a mutable ref. That is useful for humans, but semantically weak for autonomous competition.

BulletGit should represent:

```text
Selection Group
├── Variant A
│   ├── Attempt A1
│   └── Candidate CA
├── Variant B
│   ├── Attempt B1
│   └── Candidate CB
├── Variant C
│   ├── Attempt C1
│   └── Candidate CC
└── Selection
    └── Candidate CB selected by rubric R
```

Each Variant receives:

* a different permanent fence;
* a different private workspace;
* a different budget;
* separate Evidence;
* separate review context;
* no shared mutable branch.

The GitHub-compatible branch is merely an export representation:

```text
refs/heads/bullet/candidate/<candidate-id>
```

The actual selection is a distinct signed object. Moving a branch pointer must never silently mean “this is now the winner.”

Git already supports atomic ref updates and conditional pushes, which BulletGit can use at the compatibility boundary, but those mechanisms do not provide Variant, selection-rubric, or loser-retention semantics. ([Git][3])

**Profound consequence:** speculative parallelism becomes safe and measurable instead of being disguised as branch proliferation.

---

## 5. The working copy should become an append-only transaction journal

Current Git has three distinct mutable concepts:

```text
HEAD tree
index
working tree
```

For agents, this creates an attribution gap. A process can die with important work present only as untracked or unstaged filesystem state.

BulletGit should treat the working copy as a continuously journaled transaction:

```text
Workspace base
  → operation 1: write src/auth.rs
  → operation 2: rename fixture
  → operation 3: generator output
  → operation 4: revert operation 2
  → checkpoint K1
  → operation 5: test repair
  → checkpoint K2
  → Candidate C1
```

Every mutation should record:

* path identity;
* before and after content hash;
* mode change;
* originating tool or process;
* Attempt and fence;
* scope-grant revision;
* causal model turn or tool call;
* timestamp and operation sequence.

A Rust FUSE or overlay-filesystem layer could reject unauthorized paths before mutation and hash content incrementally. The Git index could disappear from the agent-facing abstraction.

Git worktrees have some per-worktree metadata but still share most repository metadata and ordinary refs, which is one reason Bullet Farm correctly prefers private writable clones. ([Git][4])

**Profound consequence:** “uncommitted work” stops being an unstructured accident. It becomes recoverable, attributable, inspectable engineering state.

---

## 6. A mergeable change should be a proof-carrying object

A commit answers:

> What tree and parents does this revision have?

It does not answer:

* Which Mission authorized it?
* Which scope was granted?
* Which agent incarnation created it?
* Which environment built it?
* Which tests passed?
* Were those tests independent?
* Which policy version approved it?
* Was the Candidate rebased after review?
* What actual merge-group composition was tested?

Git notes can attach supplementary information without modifying the commit, but they live in separate refs and are not a strong cross-tool authority contract. ([Git][5])

BulletGit should produce a **Proof Root**:

```text
ProofRoot
├── Candidate identity
├── base, head, tree and patch hashes
├── Change and lineage
├── scope grant and actual write set
├── runner and sandbox attestation
├── toolchain and dependency manifests
├── deterministic Evidence
├── independent verifier Evidence
├── reviews and independence calculation
├── policy decision
├── human approvals
└── Effect receipts
```

GitHub’s artifact-attestation system already supports cryptographically signed provenance for build artifacts. BulletGit should integrate with that capability where useful, but extend the concept to the complete software-change proof graph. ([GitHub Docs][6])

The GitHub required check becomes:

```text
Bullet Farm / Proof Complete
external_subject = exact Candidate or merge-group SHA
proof_root = BLAKE3(...)
```

**Profound consequence:** merge readiness becomes a verifiable property of an exact subject, not a collection of loosely associated green badges.

---

## 7. Failed and rejected work should become first-class negative knowledge

Traditional repository history is biased toward what won.

Autonomous systems also need to know:

* what was attempted;
* what failed;
* why it failed;
* which tests exposed it;
* which reviewer finding rejected it;
* which model/provider combination produced it;
* whether the approach was intrinsically wrong or merely poorly executed;
* whether a later change invalidates the old failure.

BulletGit should retain bounded, access-controlled **Negative Knowledge Objects**:

```text
RejectedApproach
FailureHypothesis
InvalidatedAssumption
VerifierCounterexample
ReviewFinding
DiscardedVariant
RepairOutcome
```

This does not mean storing unrestricted chain-of-thought. It means preserving useful, typed, externally observable engineering facts:

```text
Approach:
  cache refresh result in process-global state

Rejected because:
  verifier test auth_refresh_cross_tenant failed

Scope:
  src/auth/cache.rs, src/auth/session.rs

Candidate:
  C_0192

Superseded by:
  C_0195
```

This gives future agents a memory of what **not** to repeat. It also creates much better routing and evaluation data than “PR merged” or “task failed.”

**Profound consequence:** the repository becomes an organizational learning system rather than only a ledger of surviving code.

---

## 8. Rebase, squash, repair, synthesis, and merge must be explicit subject transitions

The word “rebase” encourages people to think the change remained the same.

For proof purposes, it did not.

BulletGit should represent an explicit evolution edge:

```rust
enum EvolutionKind {
    Amend,
    Repair,
    Rebase,
    Squash,
    Split,
    Synthesis,
    CherryPick,
    MergeComposition,
    GeneratedRefresh,
}
```

Each edge records:

```text
predecessor Candidate
successor Candidate
operation kind
old and new bases
semantic delta
tool and actor
evidence retained
evidence invalidated
reviews retained
reviews invalidated
```

The stable Change ID may survive, but the Candidate ID never does.

Git notes include rewrite-related configuration, and Jujutsu’s stable Change concept shows the usefulness of preserving logical identity across commit changes. Neither provides Bullet Farm’s required evidence-invalidation semantics by itself. ([Git][5])

**Profound consequence:** stale proof becomes structurally detectable instead of relying on reviewer discipline.

---

## 9. Concurrency should be decided by semantic intent before edits, not by textual conflicts afterward

Git’s primary conflict model is reactive: compare trees and discover conflicts during merge. `rerere` can remember and reuse prior conflict resolutions, but it still starts from a conflict that has already occurred. ([Git][7])

Agent-first version control can operate earlier.

Before writing, every Variant declares a **Change Intent**:

```text
files and directory prefixes
packages/crates/modules
symbols
public APIs
schemas
migration lanes
lockfiles
generated artifact families
build targets
external-effect targets
```

BulletGit can combine this with repository indexes to forecast:

* direct path collision;
* overlapping symbol mutation;
* caller/callee interaction;
* schema-consumer impact;
* generated-file ownership;
* incompatible migration ordering;
* test-fixture interference;
* API version conflict;
* lockfile contention.

The scope graph remains conservative. AST and semantic analysis are advisers, not absolute mutexes. Final authority still comes from exact object-level write-set verification.

**Profound consequence:** the platform avoids many conflicts before wasting model, CI, and reviewer capacity.

---

## 10. GitHub should remain sovereign for integration, but should not need to understand BulletGit internals

A custom Git object format that GitHub cannot store or transport would create a proprietary island.

The correct architecture is a **Git-compatible superset**:

```text
Internally:
  richer Bullet object graph

At the GitHub boundary:
  standard blobs
  standard trees
  standard commits
  standard refs
  pull requests
  Check Runs
  artifact attestations
  rulesets
  merge queue
```

GitHub’s merge queue tests a separate merge-group composition and requires checks to report against that group’s head SHA. That is exactly the correct remote integration subject for Bullet Farm. ([GitHub Docs][8])

GitHub Apps provide repository- and permission-scoped installation tokens that expire after one hour; the token itself does not replace Bullet Farm’s branch, Candidate, fence, or scope checks. Those remain the Effect Broker’s responsibility. ([GitHub Docs][9])

The documented custom pre-receive-hook facility is a GitHub Enterprise Server capability. Therefore, a GitHub Cloud architecture should not depend on custom server-side receive hooks. It should use an exclusive GitHub App writer, rulesets, required checks, immutable Candidate refs, and merge queue. ([GitHub Docs][10])

**Profound consequence:** BulletGit can innovate rapidly without sacrificing the GitHub ecosystem or locking the company into its own forge.

---

# Ranked top 20 agent-first Git features

## Summary ranking

| Rank | Feature                                            | Strategic value | Recommended stage |
| ---: | -------------------------------------------------- | --------------: | ----------------- |
|    1 | Stable Change ID and Candidate Evolution Graph     |         10.0/10 | Kernel            |
|    2 | Proof-Carrying Candidate Root                      |         10.0/10 | Kernel            |
|    3 | Fenced Capability-Secure Repository API            |         10.0/10 | Kernel            |
|    4 | Transactional Workspace Journal and Checkpoints    |          9.9/10 | Kernel            |
|    5 | First-Class Variant and Selection DAG              |          9.7/10 | V1                |
|    6 | Explicit Rewrite/Rebase Successor Semantics        |          9.7/10 | Kernel            |
|    7 | Evidence Dependency and Invalidation Graph         |          9.6/10 | Kernel            |
|    8 | Semantic Change Intent and Resource Leases         |          9.5/10 | V1                |
|    9 | First-Class Integration Subject                    |          9.4/10 | V1                |
|   10 | Semantic Merge and Conflict Forecasting            |          9.3/10 | V1.1              |
|   11 | GitHub Effect Ledger and Remote Receipts           |          9.2/10 | Kernel/V1         |
|   12 | Negative Knowledge and Rejected-Variant Archive    |          9.1/10 | V1.1              |
|   13 | Reproducible Execution Envelope and Attestation    |          9.0/10 | V1                |
|   14 | Patch Algebra and Composable Change Fragments      |          8.9/10 | V1.1              |
|   15 | Acceptance, Context, and Decision Graph            |          8.8/10 | V1                |
|   16 | Agent-Aware Blame and Causal Provenance            |          8.7/10 | V1.1              |
|   17 | Intent-Aware Revert and Compensation               |          8.6/10 | V1.1              |
|   18 | Trust-Aware CAS, Retention, and Garbage Collection |          8.5/10 | V1                |
|   19 | Agent-Native Fetch, Query, and Proof-Pack Protocol |          8.4/10 | Scale             |
|   20 | Cross-Repository Mission and Saga Objects          |          8.3/10 | Post-V1           |

---

## 1. Stable Change ID and Candidate Evolution Graph

### What it is

A permanent `ChangeId` identifies the engineering intention. Every exact implementation receives a new `CandidateId`.

```text
ChangeId = stable logical identity
CandidateId = exact immutable implementation identity
GitOid = exported Git-compatible commit identity
```

The graph records all Candidate evolution:

```text
C1 --repair--> C2 --rebase--> C3 --merge-group--> C4
```

### Why it matters

This solves the fundamental mismatch between:

* human discussion about “the change”;
* Git’s exact commit hashes;
* agent repair loops;
* rebases;
* Candidate races;
* evidence validity.

### Minimum design

```rust
struct Change {
    id: ChangeId,
    mission: MissionId,
    acceptance_root: Digest,
}

struct Candidate {
    id: CandidateId,
    change: ChangeId,
    git_commit: GitOid,
    tree: GitOid,
    patch_digest: Digest,
    lineage_subject: LineageSubject,
    environment_digest: Digest,
}
```

### Main danger

Never let `ChangeId` authorize integration. Only an exact `CandidateId` or integration subject may satisfy proof or merge policy.

---

## 2. Proof-Carrying Candidate Root

### What it is

One Merkle root binds all qualifying proof to the exact Candidate.

```text
candidate_proof_root =
    H(candidate
      || scope
      || environment
      || evidence
      || reviews
      || policy
      || approvals
      || effect_receipts)
```

### Why it matters

Today, code, tests, review, CI, artifacts, and policy live in different systems with weak cross-linkage. BulletGit turns them into one verifiable closure.

### GitHub representation

* Candidate branch contains normal Git commits.
* Required Check Run references Candidate ID and proof root.
* Full proof lives in Bullet CAS.
* Build artifacts may receive GitHub attestations.
* Merge-group proof produces a different root for the actual landing composition.

### Main danger

A proof root proves that recorded claims are bound together. It does not prove the acceptance contract is complete or that tests are sufficient.

---

## 3. Fenced Capability-Secure Repository API

### What it is

The agent receives narrowly scoped repository capabilities instead of a general Git executable.

```text
read object
read symbol
apply patch
write allowed path
checkpoint
prepare Candidate
request scope expansion
query proof
```

Every call validates the current Authority Token.

### Why it matters

This prevents a stale agent from bypassing Bullet Farm through a raw `git commit`, `git update-ref`, or credential-bearing push.

### Ideal sandbox

```text
No Git binary
No .git write access
No GitHub credential
No credential helper
No SSH agent
No hooks
No arbitrary submodule transport

Only:
  repository RPC socket
  scoped workspace filesystem
  approved tool gateway
```

### Main danger

The filesystem itself must also be mediated or independently verified. A capability-secure Git API cannot prevent an agent from writing arbitrary files if the raw workspace remains unrestricted.

---

## 4. Transactional Workspace Journal and First-Class Checkpoints

### What it is

Every filesystem mutation becomes an operation in a durable journal. A checkpoint is a first-class immutable tree plus operation boundary, not merely a stash or patch file.

```text
Checkpoint
  base tree
  current tree
  operation range
  untracked-file manifest
  scope revision
  Attempt and fence
  tool/process provenance
```

### Why it matters

It provides:

* recovery after kill;
* safe successor-Attempt scope expansion;
* time travel;
* precise human intervention;
* partial Candidate salvage;
* better model continuation;
* reproducible incident reconstruction.

### Main danger

Continuous journaling could be expensive. Use content hashing, overlay semantics, batched operation records, and configurable checkpoint frequency.

---

## 5. First-Class Variant and Selection DAG

### What it is

Best-of-(N) work becomes a typed graph rather than multiple branches competing informally.

```text
SelectionGroup
  rubric
  budget
  Candidates[]
  Evidence[]
  selection decision
  losing-candidate disposition
```

### Why it matters

It allows safe use of:

* different models;
* different architectures;
* security hardening alternatives;
* performance approaches;
* competing repair strategies.

The selected Candidate is explicit. Losing Candidates can be retained for negative knowledge or later synthesis.

### Main danger

Races can burn enormous capacity. Admission requires a predeclared rubric and positive expected value.

---

## 6. Explicit Rewrite, Rebase, and Merge Successor Semantics

### What it is

Every history-rewriting operation emits a typed Candidate transition.

```text
Candidate C1
  --rebase(base_old, base_new)-->
Candidate C2
```

The transition service computes:

* tree delta;
* patch-equivalence analysis;
* changed dependency closure;
* Evidence invalidation;
* review invalidation;
* conflict resolutions introduced;
* new integration risk.

### Why it matters

It eliminates the dangerous phrase:

> “It was only a rebase.”

For proof purposes, there is no “only.”

### Main danger

Do not attempt unsound automatic proof reuse. Default to invalidation and require deterministic repository-authored closure checkers for exceptions.

---

## 7. Evidence Dependency and Invalidation Graph

### What it is

Every Evidence object declares the inputs on which it depended:

```text
source paths
generated inputs
toolchain
dependency locks
environment
service fixtures
schema versions
base commit
Candidate tree
```

When a Candidate changes, BulletGit calculates which proof becomes stale.

### Why it matters

This makes evidence reuse safe enough to optimize while remaining conservative.

Example:

```text
Change:
  docs/architecture.md

Potentially reusable:
  Rust unit-test result

Change:
  Cargo.lock

Invalidated:
  build
  tests
  security scan
  license scan
  binary attestation
```

### Main danger

Dependency closure is difficult and language-specific. Over-approximation is safe but expensive; under-approximation is dangerous.

---

## 8. Semantic Change Intent and Resource Leases

### What it is

A Candidate declares intended resources before mutation:

```text
path: src/auth/**
symbol: Session::rotate
crate: auth
schema: public.session_v3
migration_lane: postgres.primary
generated_family: api-client
lockfile: Cargo.lock
```

Bullet Farm grants compatible leases and rejects or serializes collisions.

### Why it matters

It moves coordination ahead of expensive implementation.

It also makes scope compliance enforceable:

```text
Predicted scope != authority
Granted scope = authority
Actual write set must be covered
```

### Main danger

Semantic resources are advisory unless backed by exact object/path verification. Never use AST ranges as the sole mutex.

---

## 9. First-Class Integration Subject

### What it is

The actual composition that may land becomes its own object:

```text
IntegrationSubject
  target SHA
  Candidate set
  merge method
  generated merge-group SHA
  conflict resolutions
  combined proof requirements
  integration Evidence
```

### Why it matters

A Candidate can pass independently yet fail when combined with current main or another queued Candidate.

GitHub’s merge queue already creates and tests a distinct merge-group head SHA. BulletGit should model that SHA as a first-class subject, not as incidental CI metadata. ([GitHub Docs][8])

### Main danger

Integration Evidence must never be incorrectly copied back onto the pre-merge Candidate.

---

## 10. Semantic Merge and Conflict Forecasting

### What it is

A layered merge engine:

```text
Layer 1: standard three-way tree merge
Layer 2: language-aware structural merge
Layer 3: API/schema compatibility analysis
Layer 4: generated-output ownership
Layer 5: dependency/test-impact graph
Layer 6: model-assisted explanation or repair proposal
```

The model may propose resolution, but deterministic services own the merge subject and validation.

### Why it matters

It addresses conflicts that are invisible to textual merge:

* two agents add incompatible API parameters;
* one changes a schema while another changes its consumer;
* two migrations use the same ordering slot;
* generated output diverges from source;
* two changes separately pass but violate a shared invariant.

### Main danger

Semantic merge cannot be trusted universally. Unsupported languages and macros must fall back to conservative conflict states.

---

## 11. GitHub Effect Ledger and Remote Receipts

### What it is

Git operations that cross the GitHub boundary become explicit Effects:

```text
push Candidate ref
create/update PR
publish Check Run
enqueue merge
delete stale branch
record merged target
```

Every Effect has:

```text
logical key
desired remote state
expected current state
Candidate and fence
GitHub request identity
observed remote result
receipt
```

### Why it matters

A lost HTTP or Git response cannot be treated as non-execution. Bullet Farm resolves the original effect before retrying.

Git can request atomic remote ref updates when supported, but no Git operation creates an ACID transaction across the local ledger, GitHub refs, pull requests, status checks, and merge queue. ([Git][3])

### Main danger

Some remote outcomes cannot be proven absent. Preserve `OUTCOME_UNKNOWN` and quarantine rather than inventing certainty.

---

## 12. Negative Knowledge and Rejected-Variant Archive

### What it is

Structured retention for failed engineering attempts:

```text
Candidate rejected
Verifier counterexample
Review finding
Broken assumption
Known-bad conflict resolution
Unproductive tool sequence
Provider-specific failure
```

### Why it matters

Future agents can ask:

```text
Has this approach already failed?
What test disproved it?
Which Candidate fixed the issue?
Was failure implementation-specific or architectural?
```

It also provides high-quality routing data:

```text
Model A succeeds at auth debugging
Model B repeatedly misses generated-code effects
Harness C loses context after scope amendment
```

### Main danger

Retention must be privacy-aware. Do not store hidden chain-of-thought, secrets, or unnecessary provider transcripts.

---

## 13. Reproducible Execution Envelope and Attestation

### What it is

A Candidate records the exact environment that produced and verified it:

```text
runner image
kernel/sandbox tier
compiler
toolchain
dependency locks
environment variables by name and approved digest
network policy
provider/model/harness version
prompt package
MCP/tool versions
test fixture versions
```

### Why it matters

It distinguishes:

> “The tests passed somewhere”

from:

> “These exact tests passed for this exact Candidate in this exact independently controlled environment.”

### GitHub integration

Use GitHub artifact attestations for produced binaries or containers, while BulletGit retains the wider Candidate and verification envelope. ([GitHub Docs][6])

### Main danger

Perfect reproducibility is not always available. Record environmental uncertainty explicitly.

---

## 14. Patch Algebra and Composable Change Fragments

### What it is

Represent a Candidate as a graph of logical fragments rather than one indivisible diff:

```text
Fragment F1: API type addition
Fragment F2: implementation
Fragment F3: tests
Fragment F4: generated client
Fragment F5: documentation
```

Fragments declare dependencies:

```text
F2 requires F1
F3 validates F1 + F2
F4 derives from F1
```

### Why it matters

This enables:

* safe Candidate synthesis;
* selective adoption from losing Variants;
* automatic patch splitting;
* focused review;
* partial reverts;
* causal evidence mapping;
* parallel work with explicit dependencies.

### Main danger

Fragments cannot always be applied independently. The system must preserve whole-tree Candidate identity as final authority.

---

## 15. Acceptance, Context, and Decision Graph

### What it is

Code is linked to the requirements and decisions that produced it:

```text
Acceptance requirement
  → Plan node
  → Work Package
  → Change fragment
  → Candidate
  → Evidence
  → Review finding
  → Integration outcome
```

### Why it matters

An agent can ask:

```text
Why does this branch exist?
Which requirement does this line serve?
What evidence established this behavior?
Which decision rejected the alternative?
```

This is more valuable than storing longer chat transcripts.

### Main danger

Links must be typed and evidence-backed. Do not let model-generated narratives become accepted facts automatically.

---

## 16. Agent-Aware Blame and Causal Provenance

### What it is

Extend blame from:

```text
Who last changed this line?
```

to:

```text
Which logical Change introduced it?
Which Candidate revision?
Which agent/human/tool performed the mutation?
Which Work Package and acceptance requirement?
Which verifier covered it?
Which review findings affected it?
Which generated source produced it?
```

### Why it matters

This would dramatically improve:

* incident analysis;
* ownership routing;
* regression localization;
* security investigations;
* model evaluation;
* maintenance agents.

### Main danger

Avoid turning model identity into simplistic blame. The causal graph should distinguish author, planner, tool, reviewer, approver, and integrator.

---

## 17. Intent-Aware Revert and Compensation

### What it is

A normal revert applies an inverse patch. BulletGit should understand logical effects.

```text
Revert Change X
  remove implementation
  restore affected API
  regenerate clients
  reverse migration where safe
  update feature flag
  invalidate dependent Candidates
  create compensation plan
```

### Why it matters

This is especially valuable for:

* stacked changes;
* multi-repository Missions;
* schema migrations;
* generated code;
* feature flags;
* partial rollout.

### Main danger

Many effects are irreversible. The system must distinguish:

```text
revertible
compensatable
forward-repair only
human recovery required
```

---

## 18. Trust-Aware CAS, Retention, and Garbage Collection

### What it is

The object store understands trust and dependency roots:

```text
source objects
Candidate objects
writer artifacts
verifier artifacts
hidden evaluator artifacts
reviews
effect receipts
negative knowledge
audit roots
```

GC follows the entire proof and lineage graph, not only Git commit reachability.

### Why it matters

Without this, aggressive cleanup may delete:

* Evidence needed for audit;
* a Candidate required for regression reproduction;
* a parent Candidate of a stacked change;
* a verifier artifact referenced by a merged commit;
* a rejected Variant still used by routing evaluation.

### Main danger

Unlimited history will become expensive and may create data-governance risk. Use explicit retention classes and cryptographic tombstones.

---

## 19. Agent-Native Fetch, Query, and Proof-Pack Protocol

### What it is

A protocol optimized for agents rather than full developer clones:

```text
fetch exact symbols and dependency neighborhoods
fetch relevant test history
fetch Candidate proof closure
fetch negative knowledge for touched resources
fetch only required blobs
stream changed semantic units
```

Git protocol v2 already supports filtered transfers and bundle discovery, providing useful compatibility foundations for specialized clone acceleration. ([Git][11])

A Rust implementation is feasible without shelling out to C Git: `gitoxide`/`gix` is an existing Git implementation written in Rust and designed for correctness and performance. ([GitHub][12])

### Why it matters

Agents frequently need a precise context slice, not the entire repository and all historical blobs.

### Main danger

Over-filtering can hide crucial context. Context selection should be explainable and allow escalation to wider source access.

---

## 20. Cross-Repository Mission and Saga Objects

### What it is

A higher-level object references Candidates across repositories:

```text
MissionChange
├── API repository Candidate A
├── client repository Candidate B
├── deployment repository Candidate C
├── compatibility constraints
├── integration order
├── observation gates
└── compensation plan
```

### Why it matters

Git has repository-local history. Modern systems often require coordinated changes across:

* service and client;
* schema and consumer;
* library and dependents;
* infrastructure and application;
* generated SDK repositories.

The object should not claim false atomicity. It should model:

```text
STAGED
PARTIALLY_INTEGRATED
OBSERVING
COMPENSATING
FORWARD_REPAIR_REQUIRED
SURVIVED
```

### Main danger

Cross-repository operations are sagas, not one atomic merge. The portal must communicate partial completion honestly.

---

# Recommended internal architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                     Bullet Farm Control Plane                │
│ SQL authority ledger: leases, fences, commands, quota, policy│
└───────────────────────────────┬─────────────────────────────┘
                                │ AuthorityToken
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                         bullet-gitd                          │
│                                                             │
│  Git compatibility core     Bullet Change Graph             │
│  ───────────────────────     ───────────────────────────     │
│  blobs                       Change                          │
│  trees                       Candidate                       │
│  commits                     EvolutionEdge                   │
│  tags                        Checkpoint                      │
│  packs                       ScopeGrant                      │
│  protocol v2                 ProofRoot                       │
│                              Review                          │
│  Workspace journal           Selection                       │
│  Semantic merge              IntegrationSubject              │
│  Scope enforcement           EffectReceipt                   │
└──────────────┬────────────────────────────┬─────────────────┘
               │                            │
               ▼                            ▼
     Private agent workspace        Bullet CAS / History Engine
               │
               ▼
┌─────────────────────────────────────────────────────────────┐
│                       GitHub Effect Broker                   │
│ Fence recheck · Candidate verification · JIT App token      │
│ Push · PR · Check Run · Merge queue · read-back receipt     │
└───────────────────────────────┬─────────────────────────────┘
                                ▼
                    GitHub protected repository
```

## Compatibility rule

**Never require GitHub to understand custom object types.**

Internally, Bullet objects may use canonical CBOR and BLAKE3 or another modern digest. At the GitHub boundary, export:

* ordinary Git trees and commits;
* ordinary Candidate branches;
* Check Runs carrying Candidate and proof-root identity;
* artifact attestations where applicable;
* standard pull requests;
* required rulesets;
* merge queue.

The full Bullet graph lives in the Bullet CAS and may optionally be mirrored into standard Git blobs and metadata commits for portability. Commit headers or trailers can be included for convenience, but they should not be the sole authoritative mapping because nonstandard metadata is not preserved by every Git workflow. ([JJ VCS][13])

---

# What I would build first

## Phase 1: the category-defining kernel

Build these five together:

1. `ChangeId` versus `CandidateId`;
2. `bullet-gitd` capability API;
3. transactional workspace journal;
4. Candidate evolution edges;
5. proof-root object.

That immediately creates an agent-first repository model.

## Phase 2: enforce authority at the repository layer

Add:

* Authority Token validation;
* permanent-fence checks;
* scope enforcement;
* private workspace mounts;
* immutable Candidate refs;
* evidence invalidation;
* checkpoint recovery.

## Phase 3: close the GitHub loop

Add:

* exclusive GitHub App writer;
* expected-old-OID push;
* remote read-back;
* idempotent PR effects;
* Candidate Check Runs;
* proof-complete required rule;
* merge-group Integration Subjects;
* post-merge Outcome objects.

GitHub rulesets can require status checks, and merge queue can require the combined merge-group subject to pass before integration. ([GitHub Docs][14])

## Phase 4: build the durable moat

Then add:

* Variant selection;
* negative knowledge;
* semantic conflict forecasting;
* patch algebra;
* causal blame;
* intent-aware reverts;
* agent-native proof packs;
* cross-repository sagas.

---

# What I would explicitly refuse

Do **not** build:

1. A new Git format that cannot round-trip through normal GitHub repositories.
2. Git as the live lease, heartbeat, or quota ledger.
3. Commit messages or trailers as the only Change identity.
4. Git notes as the only Evidence authority.
5. An unrestricted Git CLI inside agent sandboxes.
6. A mutable shared branch on which multiple agents collaborate.
7. Automatic Evidence reuse after rebase.
8. Model-generated semantic locks treated as authoritative.
9. A merge engine that silently applies model resolutions without proof.
10. A claim that GitHub App token scope itself enforces Bullet Farm fences.
11. Unlimited retention of transcripts, failed Candidates, or sensitive artifacts.
12. “Smart” conflict resolution before the authority and proof kernel is correct.

---

# Final ranking of the strategic bet

The value stack is:

```text
1. Stable logical change identity
2. Proof-carrying exact Candidates
3. Authority enforced inside repository mutation
4. Transactional, recoverable working state
5. Explicit lineage and invalidation
6. Safe Variant competition
7. Semantic intent and conflict prediction
8. Negative engineering knowledge
9. GitHub receipt-verified integration
10. Agent-native repository query and learning
```

The deepest insight is this:

> **Git was designed to preserve snapshots and ancestry. BulletGit should preserve engineering intention, authority, proof, and causal evolution—while still exporting ordinary Git to GitHub.**

That would make Bullet Farm much more than an orchestration platform. It would become the first serious candidate for an **agent-native software change operating system**: many models may reason, but the repository itself understands which change is authorized, which revision is exact, which proof still applies, which Candidate won, and why anything is allowed to reach main.

[1]: https://git-scm.com/docs/git-commit-tree "https://git-scm.com/docs/git-commit-tree"
[2]: https://jj-vcs.github.io/jj/latest/tutorial/ "https://jj-vcs.github.io/jj/latest/tutorial/"
[3]: https://git-scm.com/docs/git-push/2.53.0 "https://git-scm.com/docs/git-push/2.53.0"
[4]: https://git-scm.com/docs/git-worktree "https://git-scm.com/docs/git-worktree"
[5]: https://git-scm.com/docs/git-notes "https://git-scm.com/docs/git-notes"
[6]: https://docs.github.com/en/actions/concepts/security/artifact-attestations "https://docs.github.com/en/actions/concepts/security/artifact-attestations"
[7]: https://git-scm.com/docs/git-rerere "https://git-scm.com/docs/git-rerere"
[8]: https://docs.github.com/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue "https://docs.github.com/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue"
[9]: https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/authenticating-as-a-github-app-installation "https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/authenticating-as-a-github-app-installation"
[10]: https://docs.github.com/en/enterprise-server%403.20/admin/enforcing-policies/enforcing-policy-with-pre-receive-hooks/managing-pre-receive-hooks-on-your-instance "https://docs.github.com/en/enterprise-server%403.20/admin/enforcing-policies/enforcing-policy-with-pre-receive-hooks/managing-pre-receive-hooks-on-your-instance"
[11]: https://git-scm.com/docs/bundle-uri "https://git-scm.com/docs/bundle-uri"
[12]: https://github.com/Byron/gitoxide?ref=cve.news "https://github.com/Byron/gitoxide?ref=cve.news"
[13]: https://jj-vcs.github.io/jj/latest/git-compatibility/ "https://jj-vcs.github.io/jj/latest/git-compatibility/"
[14]: https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets "https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets"

