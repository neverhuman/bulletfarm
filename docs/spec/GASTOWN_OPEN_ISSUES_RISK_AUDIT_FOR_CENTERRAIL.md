# Gas Town Open-Issue Risk Audit

## Ranked redesign lessons for Centerrail

**Snapshot:** 2026-08-24  
**Repository:** [`gastownhall/gastown`](https://github.com/gastownhall/gastown)  
**Open-issue query:** [`is:issue is:open`](https://github.com/gastownhall/gastown/issues?q=is%3Aissue%20state%3Aopen)  
**Purpose:** identify the most concerning unresolved failure modes in Gas Town and convert them into non-negotiable requirements for a clean-room redesign.

> **Executive judgment:** Gas Town is an unusually inventive and transparent multi-agent orchestration laboratory. Its most serious open issues do not point to one bad subsystem; they reveal a repeated authority problem. State and meaning are inferred across Beads/Dolt rows, tmux sessions, prompts, names, paths, environment, worktrees, queues, and Git branches. When those surfaces diverge, the system often produces a clean-looking answer—*done, empty, healthy, delivered, preserved, safe, merged*—for a question it did not actually resolve.

The redesign should preserve Gas Town's best product ideas—persistent work, role decomposition, provider plurality, proactive handoff, verification queues, escalation, and cost/outcome learning—but replace its authority model. Centerrail's baseline law remains:

> **Many agents may reason. Multiple isolated Variants may compete. Exactly one incarnation-fenced Attempt may write within each Variant. Every consequential external mutation is brokered. Every proof names its exact subject. The repository remains sovereign.**

The most important lesson from the issue corpus is even more compact:

> **Models may propose meaning; only the deterministic kernel may establish authority, destructive eligibility, proof, or completion.**

---

## 1. Corpus scope and methodology

### 1.1 Snapshot

The repository-wide search returned:

| Measure | Count | Interpretation |
|---|---:|---|
| Open issues | **333** | All open issue records were included in the title/body corpus scan. |
| Open issues labeled `kind/bug` | **80** | Labels understate the defect surface; many severe reports remain `needs-triage` without a kind. |
| Open issues labeled `priority/p0` | **7** | The maintainer label is useful but insufficient for architectural ranking. |
| Open issues labeled `priority/p1` | **31** | Several production data-loss or wrong-target reports have no P0/P1 label. |

### 1.2 What “studied all outstanding issues” means here

1. Enumerated the full open-issue corpus through four paginated GitHub searches.
2. Reviewed titles, labels, dates, reported versions, and bodies for architectural clustering.
3. Ran focused searches across P0, P1, bug, storage, lifecycle, merge, provider, context, quota, messaging, and workspace families.
4. Opened and read the highest-risk production reports individually.
5. Read the current design documents that explain why the mechanisms exist: architecture, Dolt storage, polecat lifecycle, propulsion, molecules, scheduler, mail/escalation, provider integration, Factory Worker API, and model-aware molecules.
6. Ranked issues by redesign significance rather than by popularity, age, or maintainer label.

This is a **risk audit**, not a claim that every open report is still reproducible on current `main`. Reports cover multiple released versions and some explicitly verify current main while others do not. Open status means unresolved in the tracker, not necessarily unpatched in an unpublished branch. Issue bodies are treated as reported evidence; only reports that say they reproduced or source-verified a mechanism are described that way.

### 1.3 Ranking method

The 0–100 concern score weighs:

```text
5 × potential work/data loss
+ 4 × blast radius
+ 4 × silent or misleading success
+ 3 × irreversibility / recovery difficulty
+ 3 × recurrence / architectural leverage
+ 1 × source or reproduction confidence
```

The ordering deliberately places an unlabeled concurrent deletion bug above lower-blast-radius P0s. The question is not only “how bad was one incident?” but “what primitive is missing, and how many future defects can that primitive prevent?”

---

## 2. Bottom-line findings

### 2.1 The five most consequential conclusions

1. **Destructive actions are insufficiently ownership-fenced.** Cleanup, rollback, GC, checkpointing, repair, and nuke operations often target a name, bead, path, age, or inferred liveness rather than the exact incarnation that owns the resource.
2. **Completion is not repository truth.** `CLOSED`, convoy complete, session done, branch pushed, MR submitted, and merged are frequently conflated. That creates false delivery, premature dependency release, duplicate work, and cleanup of unintegrated changes.
3. **Authority is fragmented.** Beads fields, Dolt routing, tmux sessions, provider processes, environment variables, worktree state, branch metadata, and dashboards can disagree without one source that conclusively wins.
4. **Uncertainty is routinely painted green.** Read failure becomes empty; no matching rows becomes healthy; a gate that did not run becomes pre-existing; a message that was accepted by tmux becomes delivered; a local rewrite whose push failed becomes verified.
5. **The ZFC boundary is drawn in the wrong place for safety.** “Go transports, agents decide” is powerful for open-ended reasoning, but unsafe when required-step execution, liveness, destructive cleanup, gate semantics, or completion are left to prompt compliance.

### 2.2 The highest-value redesign move

Do **not** begin by porting Gas Town's roles, patrols, formulas, worktrees, or dashboard into Rust. Begin with the transaction and authority kernel:

```text
permanent fence counters
+ exact Attempt/Variant ownership
+ private writable clones
+ typed UNKNOWN/CONTRADICTORY observations
+ immutable Candidate identity
+ exact-subject Evidence
+ durable priority commands
+ Effect Intents and Receipts
+ protected integration states
+ preservation-before-destruction
```

Until those exist, adding more providers or agents multiplies ambiguous authority rather than useful parallelism.

---

## 3. Gas Town design review

### 3.1 What the design gets right

- **Clear role decomposition.** Mayor/Witness/Refinery/worker concepts make operational responsibilities legible. Preserve them as product views and policies, not as authority-bearing model personas.
- **Persistent work and handoff intent.** Gas Town correctly recognizes that provider sessions are ephemeral. Keep canonical context and durable work independent of native sessions.
- **Refinery/merge-queue aspiration.** Batching, verification, conflict handling, and serialized landing are valuable. Rebuild them around exact Candidates and protected repository authority.
- **Escalation UX.** Severity, routing, stale escalation, and human channels are useful. Back them with durable priority delivery and explicit intervention state.
- **Event-driven wake and backpressure direction.** Avoid town-wide polling, but use scoped durable queues and acknowledgments rather than shared directories.
- **Provider-agnostic ambition.** The Factory Worker API design correctly calls for push-not-scrape, structured-not-string-matched, correlation, and fail-closed unknown.
- **Fresh context per workflow stage.** Distributed workflow execution and proactive handoff are strong ideas. Move continuity into canonical Context Capsules.
- **Cost/outcome learning.** Usage, outcomes, model-aware routing, and preflight estimation are valuable. Learn only from exact accepted/surviving outcomes and never learn away safety.
- **Operational transparency.** The unusually detailed issue reports and design docs are a major asset. Convert recurring incident classes into executable invariants and chaos tests.

### 3.2 Where the design doctrine creates systemic risk

#### A. ZFC is valuable, but safety is not cognition

Gas Town's Zero Framework Cognition doctrine says the framework transports while agents decide. That is an excellent defense against hardcoding business judgment into infrastructure. The issue corpus shows the boundary must stop before:

- ownership and fencing;
- liveness classification used for destruction;
- whether an effect occurred;
- whether tests executed;
- whether a workflow step ran;
- whether work is integrated;
- whether state is safe to delete;
- whether policy permits remote publication.

Models should diagnose, propose, plan, prioritize, review semantics, and choose among authorized strategies. The kernel must own permissions, exact identity, invariants, postconditions, evidence, retention, and destructive transitions.

#### B. Propulsion has no authoritative “I understood and started” boundary

The propulsion principle trusts an agent that finds work on its hook to understand and execute immediately, explicitly avoiding a supervisor check. Open issue #2635 asks for the missing acknowledgment because session creation is currently projected as working before the agent has parsed its assignment. Centerrail should represent `DISPATCHED → ACCEPTED → CONTEXT_LOADED → EXECUTING` with structured acknowledgments and timeouts.

#### C. Root-only workflows reduce storage by deleting structural guardrails

Root-only wisps avoid high-churn rows, but the trade moves required-step truth into prompts. #2386 reports hundreds of fake patrol cycles; #4732 and related reports show bare workflows with missing steps and vacuously green health. The redesign should compress storage, not erase proof: typed workflow state can be compact and event-sourced without requiring one heavyweight issue row per model thought.

#### D. Beads/Dolt is doing too many jobs

Gas Town stores operational claims, identity, workflow, mail, merge requests, events, audit, and configuration in an issue-oriented substrate, split across town and rig databases, with some high-value wisps deliberately excluded from Dolt history. This creates a brittle interaction among routing, schema, retention, sync, and lifecycle. The fresh system needs distinct transactional, audit, artifact, and projection planes.

#### E. Worktrees optimize spawn speed at the cost of isolation

The design explicitly uses Git worktrees for autonomous writers and shared object/admin state. The open issues contain exactly the resulting classes: stale registrations, asynchronous cleanup deleting new work, background checkpoint commits, repository discovery escaping upward, and runtime files entering product source. Private clone per Attempt is not an optimization detail; it is an authority boundary.

#### F. Self-managed completion removes a bottleneck but also removes an independent commit boundary

The polecat lifecycle intentionally lets workers call `gt done`, push, submit, close, and retire themselves. That is operationally elegant, but it gives the author control over completion claims and cleanup triggers. Candidate preparation and finalization must move into trusted runner/effect services.

#### G. Mail versus nudge optimizes storage by making reliability task-dependent

The mail protocol distinguishes durable Beads mail from ephemeral nudges. In practice, critical control can still flow through a bounded lossy nudge queue, while durable mail itself contributes database churn. Centerrail should have one durable command/message substrate with retention classes and ephemeral presentation—not two semantics with different failure visibility.

#### H. The Factory Worker API points in the right direction

Gas Town's own Factory Worker API design correctly diagnoses send-keys, prompt regex, transcript scraping, process-tree inference, and keychain rotation as unstable integration hacks. Its principles—push not scrape, structured not string-matched, correlation by design, fail-closed unknown—should be adopted almost verbatim, but placed behind Centerrail's fenced authority and credential boundaries.

#### I. Model-aware molecules are a useful seed, not a sufficient router

Per-step capabilities, usage tracking, and provider/session resolution are valuable. Static MMLU/SWE thresholds, public price aggregation, and treating subscription use as zero cost are too coarse for production routing. Centerrail should learn repository/task-specific survival probability, maintain quota shadow prices, reserve capacity before every turn, and never optimize around safety policy.

---

## 4. Ranked systemic concerns

### 1. Destructive lifecycle actions are not fenced to exact ownership

**Representative open issues:** [#4584](https://github.com/gastownhall/gastown/issues/4584), [#4478](https://github.com/gastownhall/gastown/issues/4478), [#4397](https://github.com/gastownhall/gastown/issues/4397), [#4605](https://github.com/gastownhall/gastown/issues/4605), [#4593](https://github.com/gastownhall/gastown/issues/4593), [#4737](https://github.com/gastownhall/gastown/issues/4737), [#4672](https://github.com/gastownhall/gastown/issues/4672), [#4604](https://github.com/gastownhall/gastown/issues/4604), [#4588](https://github.com/gastownhall/gastown/issues/4588)

**Finding.** Cleanup, rollback, checkpointing, GC, doctor repair, and nuke paths can act on a bead, name, path, age, or inferred liveness rather than on the exact incarnation that owns the artifact. That lets a stale or failed operation destroy a newer healthy one.

**Fresh-redesign response.** Every mutable object and external effect belongs to an Attempt/Variant with a permanent fencing epoch. Destructive actions run only in trusted services, require identity-matched preconditions, produce preservation receipts where applicable, and use the same decision function for dry-run and execution.

**Required exit test.** A delayed cleanup from Attempt N must be unable to mutate any workspace, branch, lease, queue entry, or effect belonging to Attempt N+1.

### 2. Completion is overloaded and detached from repository truth

**Representative open issues:** [#4699](https://github.com/gastownhall/gastown/issues/4699), [#4469](https://github.com/gastownhall/gastown/issues/4469), [#3603](https://github.com/gastownhall/gastown/issues/3603), [#3914](https://github.com/gastownhall/gastown/issues/3914), [#1893](https://github.com/gastownhall/gastown/issues/1893), [#4739](https://github.com/gastownhall/gastown/issues/4739), [#4583](https://github.com/gastownhall/gastown/issues/4583), [#4698](https://github.com/gastownhall/gastown/issues/4698)

**Finding.** CLOSED or convoy complete can mean session ended, branch submitted, branch pushed, or work supposedly done—not that the target branch contains an accepted Candidate. Consumers then deduplicate, unblock dependencies, clean sandboxes, or report delivery from a false premise.

**Fresh-redesign response.** Use explicit Prepared, Delivered, Verified, Reviewed, Queued, Integrated, Observing, Survived, Rejected, and Reverted states. Only a verified target commit and required observation window complete a Mission requirement.

**Required exit test.** No source task or dependency may become integrated/complete from agent exit, issue closure, push success, or PR creation alone.

### 3. Authority is fragmented across Beads, Dolt, tmux, environment, paths, and Git

**Representative open issues:** [#764](https://github.com/gastownhall/gastown/issues/764), [#4527](https://github.com/gastownhall/gastown/issues/4527), [#2682](https://github.com/gastownhall/gastown/issues/2682), [#4733](https://github.com/gastownhall/gastown/issues/4733), [#4409](https://github.com/gastownhall/gastown/issues/4409), [#3763](https://github.com/gastownhall/gastown/issues/3763), [#4638](https://github.com/gastownhall/gastown/issues/4638), [#4679](https://github.com/gastownhall/gastown/issues/4679), [#4709](https://github.com/gastownhall/gastown/issues/4709), [#4598](https://github.com/gastownhall/gastown/issues/4598)

**Finding.** Multiple representations can each appear authoritative: hook fields, assigned work, session names, tmux sockets, prefixes, cwd, redirects, branch metadata, and database rows. Reconciliation tries to infer which one is real after they diverge.

**Fresh-redesign response.** One transactional ledger owns identity and state. Runtime, GitHub, terminals, filesystems, and provider sessions are observed projections. Every command carries an Authority Token; cwd, prefix, name, environment, and path are never authority.

**Required exit test.** Given contradictory runtime and durable observations, the system must expose CONTRADICTORY and refuse destructive or integrative transitions.

### 4. Actions are acknowledged before their effects are verified

**Representative open issues:** [#4527](https://github.com/gastownhall/gastown/issues/4527), [#4516](https://github.com/gastownhall/gastown/issues/4516), [#4600](https://github.com/gastownhall/gastown/issues/4600), [#4732](https://github.com/gastownhall/gastown/issues/4732), [#4660](https://github.com/gastownhall/gastown/issues/4660), [#4666](https://github.com/gastownhall/gastown/issues/4666), [#4671](https://github.com/gastownhall/gastown/issues/4671), [#4589](https://github.com/gastownhall/gastown/issues/4589), [#4596](https://github.com/gastownhall/gastown/issues/4596), [#4512](https://github.com/gastownhall/gastown/issues/4512)

**Finding.** Commands print success when dependent writes, delivery, process startup, workflow materialization, push, or remote publication either failed or were never checked.

**Fresh-redesign response.** Use stable command IDs, atomic internal transitions, an Effect Intent/Receipt protocol, read-after-write verification, and OUTCOME_UNKNOWN after ambiguous timeouts. UI and CLI show requested/dispatching/verified separately.

**Required exit test.** No success response may be emitted until the authoritative postcondition is durably observed or the result is explicitly UNKNOWN.

### 5. UNKNOWN is repeatedly collapsed into EMPTY, DEAD, HEALTHY, SAFE, or PASS

**Representative open issues:** [#4632](https://github.com/gastownhall/gastown/issues/4632), [#4633](https://github.com/gastownhall/gastown/issues/4633), [#4631](https://github.com/gastownhall/gastown/issues/4631), [#4614](https://github.com/gastownhall/gastown/issues/4614), [#4597](https://github.com/gastownhall/gastown/issues/4597), [#4712](https://github.com/gastownhall/gastown/issues/4712), [#4595](https://github.com/gastownhall/gastown/issues/4595), [#4669](https://github.com/gastownhall/gastown/issues/4669), [#4704](https://github.com/gastownhall/gastown/issues/4704), [#4703](https://github.com/gastownhall/gastown/issues/4703)

**Finding.** Read failure becomes no work; name mismatch becomes dead; zero matching rows becomes healthy; stale projection becomes current; a probe that did not run becomes pass. These false negatives are then fed into destructive actions.

**Fresh-redesign response.** All probes return typed VALUE/EMPTY/UNKNOWN/CONTRADICTORY outcomes with source, freshness, and confidence. Policy may act destructively only on sufficient positive proof.

**Required exit test.** Fault-inject every dependency read. No injected failure may serialize as an authoritative negative or positive result.

### 6. Git worktrees and ambient repository discovery make isolation unsound

**Representative open issues:** [#4602](https://github.com/gastownhall/gastown/issues/4602), [#4629](https://github.com/gastownhall/gastown/issues/4629), [#4672](https://github.com/gastownhall/gastown/issues/4672), [#3737](https://github.com/gastownhall/gastown/issues/3737), [#4722](https://github.com/gastownhall/gastown/issues/4722), [#4688](https://github.com/gastownhall/gastown/issues/4688), [#4588](https://github.com/gastownhall/gastown/issues/4588), [#3772](https://github.com/gastownhall/gastown/issues/3772), [#4594](https://github.com/gastownhall/gastown/issues/4594), [#4478](https://github.com/gastownhall/gastown/issues/4478)

**Finding.** Writers share Git administration state, upward discovery can select another repository, background agents mutate live worktrees, provider overlays enter source, and cleanup registrations outlive directories.

**Fresh-redesign response.** One private clone per writable Attempt, private .git/index/refs/caches, explicit repository identity, isolated HOME/XDG/Git config, no writable worktrees, runtime overlays outside product source, and trusted Candidate reconstruction.

**Required exit test.** Two concurrent writers and all maintenance workers must be unable to observe or mutate one another's Git administration or uncommitted files.

### 7. Ephemeral issue objects carry durable operational and audit obligations

**Representative open issues:** [#4605](https://github.com/gastownhall/gastown/issues/4605), [#4748](https://github.com/gastownhall/gastown/issues/4748), [#4713](https://github.com/gastownhall/gastown/issues/4713), [#4719](https://github.com/gastownhall/gastown/issues/4719), [#4611](https://github.com/gastownhall/gastown/issues/4611), [#4612](https://github.com/gastownhall/gastown/issues/4612), [#4635](https://github.com/gastownhall/gastown/issues/4635), [#4254](https://github.com/gastownhall/gastown/issues/4254), [#764](https://github.com/gastownhall/gastown/issues/764)

**Finding.** Wisps and other high-churn objects represent merge requests, patrol execution, nudges, identity, and rejection rationale while being age-collected, Dolt-ignored, hard-deleted, or single-copy.

**Fresh-redesign response.** Separate normalized transactional hot state, append-only audit/evidence, content-addressed artifacts, and rebuildable projections. Every object has an explicit retention and recovery class.

**Required exit test.** Deleting an ephemeral delivery/projection object must never erase the authoritative obligation, disposition, proof, or audit trail it represented.

### 8. Safety-critical workflow semantics are delegated to prompt compliance

**Representative open issues:** [#2386](https://github.com/gastownhall/gastown/issues/2386), [#4732](https://github.com/gastownhall/gastown/issues/4732), [#4693](https://github.com/gastownhall/gastown/issues/4693), [#4635](https://github.com/gastownhall/gastown/issues/4635), [#4612](https://github.com/gastownhall/gastown/issues/4612), [#4611](https://github.com/gastownhall/gastown/issues/4611), [#3675](https://github.com/gastownhall/gastown/issues/3675), [#4615](https://github.com/gastownhall/gastown/issues/4615), [#4469](https://github.com/gastownhall/gastown/issues/4469)

**Finding.** Root-only workflows reduce storage but make required-step execution, test count, cleanup, and handoff depend on model interpretation. Models can report cycles or completion without executing the underlying work.

**Fresh-redesign response.** Models choose and reason; the kernel materializes typed workflow steps, preconditions, gates, evidence requirements, and completion transitions. Inline context may be compact, but required execution is machine-accounted.

**Required exit test.** A simulator that outputs convincing completion prose without executing any required step must leave the workflow incomplete.

### 9. Message and control delivery is lossy, priority-blind, and input-corrupting

**Representative open issues:** [#4634](https://github.com/gastownhall/gastown/issues/4634), [#1216](https://github.com/gastownhall/gastown/issues/1216), [#4609](https://github.com/gastownhall/gastown/issues/4609), [#4610](https://github.com/gastownhall/gastown/issues/4610), [#4666](https://github.com/gastownhall/gastown/issues/4666), [#4711](https://github.com/gastownhall/gastown/issues/4711), [#4713](https://github.com/gastownhall/gastown/issues/4713), [#4607](https://github.com/gastownhall/gastown/issues/4607), [#4104](https://github.com/gastownhall/gastown/issues/4104)

**Finding.** Nudges may be dropped, duplicated, claimed forever, injected into active input, delivered to the wrong lifecycle state, or reported successful without acknowledgment. Critical and routine traffic compete in the same bounded queue.

**Fresh-redesign response.** Durable inbox/outbox with priorities, idempotency, claim leases, accepted/queued/processed acknowledgments, dead letters, backpressure, and structured provider protocols. PTY input is compatibility-only.

**Required exit test.** Under crash, saturation, duplicate delivery, and busy-session tests, every urgent command is either acknowledged, durably pending, or visibly dead-lettered—never silently absent.

### 10. Merge queue records do not bind an exact reviewed subject and target

**Representative open issues:** [#4627](https://github.com/gastownhall/gastown/issues/4627), [#4010](https://github.com/gastownhall/gastown/issues/4010), [#4699](https://github.com/gastownhall/gastown/issues/4699), [#4606](https://github.com/gastownhall/gastown/issues/4606), [#3914](https://github.com/gastownhall/gastown/issues/3914), [#4469](https://github.com/gastownhall/gastown/issues/4469), [#4512](https://github.com/gastownhall/gastown/issues/4512), [#4710](https://github.com/gastownhall/gastown/issues/4710), [#4748](https://github.com/gastownhall/gastown/issues/4748)

**Finding.** A queue item may point at a moving branch, wrong base, stale commit, missing verdict, or already-rejected work. The branch—not the reviewed pin—is often what ultimately merges.

**Fresh-redesign response.** Candidate identity includes exact base/head/tree/patch hashes. Review and Evidence bind that Candidate. Integration uses expected-old-OID and expected target; any branch or target drift invalidates readiness.

**Required exit test.** Force-pushing, appending, retargeting, or rebasing after review must produce a new Candidate and invalidate affected Evidence before merge.

### 11. Schema migration, release, and runtime compatibility are coupled to normal startup

**Representative open issues:** [#4749](https://github.com/gastownhall/gastown/issues/4749), [#4727](https://github.com/gastownhall/gastown/issues/4727), [#4718](https://github.com/gastownhall/gastown/issues/4718), [#4495](https://github.com/gastownhall/gastown/issues/4495), [#4702](https://github.com/gastownhall/gastown/issues/4702), [#4281](https://github.com/gastownhall/gastown/issues/4281)

**Finding.** Ordinary install/start commands can execute unbounded migrations, incompatible binaries and schemas ship together, and maintenance queries target columns absent from the same release.

**Fresh-redesign response.** Signed compatibility matrix, protocol/schema version handshake, migration planning with cardinality estimates, resumable batches, maintenance mode, canary databases, independent backup verification, and explicit operator recovery.

**Required exit test.** A binary must refuse to mutate an unsupported schema; startup must never implicitly launch a migration whose cost and rollback plan are unknown.

### 12. Repair and doctor tooling has ambient, broad, and under-reported authority

**Representative open issues:** [#4593](https://github.com/gastownhall/gastown/issues/4593), [#4623](https://github.com/gastownhall/gastown/issues/4623), [#4709](https://github.com/gastownhall/gastown/issues/4709), [#4651](https://github.com/gastownhall/gastown/issues/4651), [#4604](https://github.com/gastownhall/gastown/issues/4604), [#2682](https://github.com/gastownhall/gastown/issues/2682)

**Finding.** A generic --fix or cleanup path can rename databases, blank prefixes, misclassify ownership, or destroy live resources while reporting a narrow or unrelated fix.

**Fresh-redesign response.** Diagnostics are read-only by default. Repairs are typed Effect Plans with exact scope, before/after values, preconditions, approval policy, execution receipt, and compensation/recovery steps.

**Required exit test.** The report of a repair must enumerate every mutation; an unreported mutation is a severity-one invariant violation.

### 13. Provider integration and supervision depend on scraping implementation details

**Representative open issues:** [#3833](https://github.com/gastownhall/gastown/issues/3833), [#3735](https://github.com/gastownhall/gastown/issues/3735), [#3133](https://github.com/gastownhall/gastown/issues/3133), [#4670](https://github.com/gastownhall/gastown/issues/4670), [#4722](https://github.com/gastownhall/gastown/issues/4722), [#3836](https://github.com/gastownhall/gastown/issues/3836), [#506](https://github.com/gastownhall/gastown/issues/506), [#2480](https://github.com/gastownhall/gastown/issues/2480)

**Finding.** tmux prompts, send-keys timing, process-name inference, transcript paths, hook layouts, and provider-specific compaction behavior are used as lifecycle truth.

**Fresh-redesign response.** Capability-negotiated adapters with structured lifecycle, prompt acceptance, tool authorization, usage, context pressure, cancellation, and resume. Pinned-version conformance suites default unknown capabilities to UNSUPPORTED.

**Required exit test.** An adapter upgrade cannot enter production until malformed-event, auth, quota, cancel, resume, identity, and process-death tests pass.

### 14. Multi-rig and multi-town scope isolation is incomplete

**Representative open issues:** [#4731](https://github.com/gastownhall/gastown/issues/4731), [#4514](https://github.com/gastownhall/gastown/issues/4514), [#4718](https://github.com/gastownhall/gastown/issues/4718), [#3681](https://github.com/gastownhall/gastown/issues/3681), [#3763](https://github.com/gastownhall/gastown/issues/3763), [#4638](https://github.com/gastownhall/gastown/issues/4638), [#4733](https://github.com/gastownhall/gastown/issues/4733), [#4409](https://github.com/gastownhall/gastown/issues/4409)

**Finding.** Town-global channels wake the wrong refinery, one bad database aborts all dispatch, prefixes route to the wrong store, and stale sockets create duplicate towns.

**Fresh-redesign response.** Organization/repository/Mission partition keys are mandatory in every identity, queue, event, command, and database row. Circuit breakers and backpressure are per scope.

**Required exit test.** Corrupt, pause, or disconnect one repository. Unrelated repositories must continue scheduling, messaging, verifying, and integrating.

### 15. Context, quota, and session continuity are not control-plane-owned

**Representative open issues:** [#3906](https://github.com/gastownhall/gastown/issues/3906), [#3909](https://github.com/gastownhall/gastown/issues/3909), [#3910](https://github.com/gastownhall/gastown/issues/3910), [#4609](https://github.com/gastownhall/gastown/issues/4609), [#4052](https://github.com/gastownhall/gastown/issues/4052), [#3836](https://github.com/gastownhall/gastown/issues/3836), [#1668](https://github.com/gastownhall/gastown/issues/1668), [#2075](https://github.com/gastownhall/gastown/issues/2075)

**Finding.** Sessions freeze at context limits, handoffs lose context, provider-specific quota workarounds mutate local credentials, and the model is asked to self-cycle after it can no longer reason.

**Fresh-redesign response.** Canonical Context Graph/Capsules, context-pressure telemetry, supervisor-driven checkpoint/handoff, quota vectors and reservations, explicit provider migration, and no terminal babysitting.

**Required exit test.** At context or quota exhaustion, work remains fenced and recoverable; a successor provider/session receives a complete coverage-checked capsule without human terminal action.

### 16. Configuration and ambient environment are mutable hidden inputs

**Representative open issues:** [#4638](https://github.com/gastownhall/gastown/issues/4638), [#4594](https://github.com/gastownhall/gastown/issues/4594), [#4728](https://github.com/gastownhall/gastown/issues/4728), [#4586](https://github.com/gastownhall/gastown/issues/4586), [#4585](https://github.com/gastownhall/gastown/issues/4585), [#4667](https://github.com/gastownhall/gastown/issues/4667), [#2104](https://github.com/gastownhall/gastown/issues/2104)

**Finding.** Socket, root, model, identity, branch, vars, and provider behavior can change with inherited environment, cwd, stale files, or partial configuration.

**Fresh-redesign response.** Every Attempt records a complete immutable configuration generation and policy hash. Adapters probe effective identity after launch. Changes stage and activate atomically.

**Required exit test.** Changing an ambient variable or stale local file must not alter the authority, repository, target, provider identity, or policy of a running Attempt.

### 17. Portal and health projections can be stale, contradictory, or vacuously green

**Representative open issues:** [#4712](https://github.com/gastownhall/gastown/issues/4712), [#4661](https://github.com/gastownhall/gastown/issues/4661), [#4597](https://github.com/gastownhall/gastown/issues/4597), [#4596](https://github.com/gastownhall/gastown/issues/4596), [#4595](https://github.com/gastownhall/gastown/issues/4595), [#4732](https://github.com/gastownhall/gastown/issues/4732), [#4614](https://github.com/gastownhall/gastown/issues/4614)

**Finding.** Views look current without sequence/freshness, zero-row checks pass when the underlying structure is absent, and dropped databases remain displayed as loaded.

**Fresh-redesign response.** Rebuildable read models with as_of_sequence, projected_at, lag, source health, observation confidence, contradiction states, and exact authoritative IDs.

**Required exit test.** The portal may show PENDING/STALE/UNKNOWN/CONTRADICTORY but may never invent CONFIRMED from a missing or lagging source.

### 18. The ZFC boundary is too permissive for safety and authority

**Representative open issues:** [#2386](https://github.com/gastownhall/gastown/issues/2386), [#4469](https://github.com/gastownhall/gastown/issues/4469), [#4737](https://github.com/gastownhall/gastown/issues/4737), [#4633](https://github.com/gastownhall/gastown/issues/4633), [#4615](https://github.com/gastownhall/gastown/issues/4615)

**Finding.** ‘Go transports, agents decide’ is valuable for open-ended cognition, but harmful when it delegates liveness interpretation, destructive eligibility, required-step execution, gate meaning, or completion to prompt text.

**Fresh-redesign response.** Keep models responsible for proposals, diagnosis, planning, and semantic judgment. Put permissions, ownership, risk floors, liveness semantics, postcondition verification, destructive actions, Evidence requirements, and integration transitions in deterministic policy.

**Required exit test.** A model can propose a transition but cannot cause it unless the kernel validates all invariant and policy predicates.

---

## 5. Ranked individual open issues

The table ranks concrete reports. A paired mechanism such as #4632/#4633 is shown separately when each issue captures a distinct failure boundary.

| Rank | Issue | Score | Why it is concerning | Centerrail primitive |
|---:|---|---:|---|---|
| 1 | [#4584 — A failed sling rollback can delete a concurrent successful sling’s molecule, branch, and claim](https://github.com/gastownhall/gastown/issues/4584) | **98** | Concurrent destructive rollback is scoped to the source bead rather than the exact operation that owns the artifacts. | Permanent Attempt/Variant identity, fencing, resource ownership, and rollback scoped only to effects created by the same command. |
| 2 | [#4478 — Polecat name reuse and stale registry ghosts can delete a fresh worker’s worktree](https://github.com/gastownhall/gastown/issues/4478) | **97** | Asynchronous cleanup can act on a reused name; one incident caused a confused worker to commit 198 unrelated files to shared main. | Never-reused incarnation IDs, cleanup tombstones, identity-fenced teardown, private clones, and no shared writable repository fallback. |
| 3 | [#4397 — Non-force polecat nuke deletes work after preservation push failure](https://github.com/gastownhall/gastown/issues/4397) | **96** | The command detects work at risk and still destroys the worktree, violating its non-force safety contract. | Preservation is a verified Effect Receipt and a hard precondition; dry-run and execution use one deterministic decision function. |
| 4 | [#4605 — Age-based wisp GC deletes live merge requests with no recovery](https://github.com/gastownhall/gastown/issues/4605) | **95** | A refinery restart deleted an entire nine-item live merge queue; the Dolt-ignored rows were not recoverable from history. | Retention class is explicit; live obligations are never age-GC candidates; tombstone/export precedes deletion; queue state is durable. |
| 5 | [#4593 — gt doctor --fix silently renamed a production database and omitted the mutation from its report](https://github.com/gastownhall/gastown/issues/4593) | **94** | A repair tool changed the identity of the town’s main database and made the town unavailable without reporting the action. | Repairs compile to reviewable Effect Plans with exact preconditions, simulation, receipts, rollback metadata, and complete audit. |
| 6 | [#4629 — Detached-HEAD completion pushes refs/heads/HEAD and corrupts origin/HEAD across clones](https://github.com/gastownhall/gastown/issues/4629) | **93** | A valid-looking Git command created a remote branch named HEAD; subsequent fetches silently redirected origin/HEAD in every clone. | Trusted Git broker validates symbolic branch identity, rejects reserved refs, pushes explicit source/destination OIDs, and reads back. |
| 7 | [#4602 — Crew spawn rewrites the enclosing workspace’s Git remote](https://github.com/gastownhall/gastown/issues/4602) | **92** | When the target directory lacks its own repository, Git discovery walks upward and mutates the wrong repository’s origin. | Repository identity is explicit and hash-bound; every Git mutation verifies the expected toplevel and repository ID. |
| 8 | [#4672 — Checkpoint dog commits conflict markers and silently concludes cherry-pick/merge/rebase](https://github.com/gastownhall/gastown/issues/4672) | **91** | A background preservation mechanism mutates a live writer’s sequencer state and can commit unresolved conflicts. | Only the writer owns its mutable clone; checkpoints are out-of-band snapshots/patches and are forbidden during unresolved Git operations. |
| 9 | [#4527 — Sling reports success although authoritative writes may not persist](https://github.com/gastownhall/gastown/issues/4527) | **90** | Conflicting hook authorities, success printed before dependent writes, and autocommit/pool behavior can acknowledge state that disappears. | Single transactional ledger write, pinned transaction boundary, postcondition readback, and no success before durable commit. |
| 10 | [#4469 — Refinery verification fails open and gt done accepts zero-deliverable completion](https://github.com/gastownhall/gastown/issues/4469) | **90** | A gate that never executed can be classified as a pre-existing failure; zero tests and zero commits can still advance completion. | Typed gate results, executed-test evidence, exact Candidate requirements, and only PASS satisfying a blocking gate. |
| 11 | [#4010 — PR creation targets the wrong branch and Refinery does not validate the target](https://github.com/gastownhall/gastown/issues/4010) | **89** | Work intended for a feature integration branch was silently merged into main. | Effect Intent names expected target ref and old OID; broker and integration coordinator independently validate target before mutation. |
| 12 | [#4627 — Merge-request commit pin can disagree with branch head](https://github.com/gastownhall/gastown/issues/4627) | **89** | The queue reports a stale or bounced subject as ready while merging the branch’s newer, unreviewed head. | Candidate is immutable and exact-SHA bound; branch movement invalidates review/evidence and cannot change the merge subject. |
| 13 | [#4512 — Durable no-push/local-only intent is lost on redispatch](https://github.com/gastownhall/gastown/issues/4512) | **88** | A branch reaches the real remote before downstream policy notices that the work was explicitly local-only. | External-effect policy is structured on the Mission/Work Package and rechecked at the broker, never inferred from prose or dispatch flags. |
| 14 | [#4699 — gt done closes work and completes convoys on submit rather than merge](https://github.com/gastownhall/gastown/issues/4699) | **88** | CLOSED is published while work is only queued—or while push/MR creation failed—causing lost work, duplicate work, and false dashboards. | Separate Prepared, Delivered, Verified, Queued, Integrated, and Observed states; only verified target commit completes engineering work. |
| 15 | [#4632 — Failed hook reads serialize as a genuine empty hook](https://github.com/gastownhall/gastown/issues/4632) | **87** | A dependency failure returns rc=0 and the exact same sentence as an authoritative zero-row result. | Probe results are typed as VALUE/EMPTY/UNKNOWN/CONTRADICTORY; unknown is never converted to absence. |
| 16 | [#4633 — Destructive gt done acts on a false-empty hook result](https://github.com/gastownhall/gastown/issues/4633) | **87** | The action layer does not independently verify that no live work exists before push/submission/sandbox destruction. | Destructive command revalidates authoritative state and fence at the choke point; UNKNOWN aborts. |
| 17 | [#4634 — Nudge queue silently drops new messages at capacity, including P0 escalation](https://github.com/gastownhall/gastown/issues/4634) | **86** | A P0 report of destroyed records remained unseen for about 24 hours; recipient had no indication that traffic was being discarded. | Durable priority inbox/outbox, acknowledgments, separate control/reminder classes, spill/backpressure, and recipient-visible dead letters. |
| 18 | [#2682 — Database naming inconsistency creates split-brain and force-init can destroy the canonical database](https://github.com/gastownhall/gastown/issues/2682) | **86** | Prefix-versus-rig naming can route work into two databases; panic recovery via force init can erase the real store. | One immutable repository/scope ID, centrally allocated storage names, no cwd-derived authority, and destructive initialization denied to agents. |
| 19 | [#4589 — Compactor swallows failed force-push after history rewrite](https://github.com/gastownhall/gastown/issues/4589) | **85** | The database graph is rewritten, publication fails, verify still closes, and the local database remains forked from its remote. | Maintenance is an external Effect with preflight, publication receipt, remote ancestry verification, and explicit OUTCOME_UNKNOWN. |
| 20 | [#4737 — Patrol cleanup can classify every dog as dead and force-remove live worktrees](https://github.com/gastownhall/gastown/issues/4737) | **84** | The destructive branch is currently blocked only by an unrelated worktree-layout accident; a one-character transcription made it fire. | No formula or model may directly authorize deletion; liveness is identity-scoped and destructive cleanup runs in the trusted kernel. |
| 21 | [#4749 — Install-time migration has a hard 30-second deadline while the migration runs for more than 61 minutes](https://github.com/gastownhall/gastown/issues/4749) | **83** | Startup/install enters deterministic crash loops and leaves dirty staging state, with no supported migration escape hatch. | Dedicated migration controller, preflight cost estimate, resumable/batched steps, maintenance mode, version handshake, and rollback plan. |
| 22 | [#4733 — Rig-scoped entities are stored in the town database](https://github.com/gastownhall/gastown/issues/4733) | **82** | Correct-looking prefixed identities live in the wrong database; normal rig paths cannot find them and doctor passes vacuously. | Every row carries authoritative organization/repository/scope partition keys; routing never depends on prefix or cwd. |
| 23 | [#4600 — Sling is non-atomic and can leave a hooked bare/stepless workflow](https://github.com/gastownhall/gastown/issues/4600) | **82** | Assignment may become visible before formula materialization is complete, producing a worker that appears active without executable workflow state. | Atomic, content-addressed plan/workflow materialization and lease acquisition; partial graph creation is unrepresentable. |
| 24 | [#4732 — Formula instantiation reports success while dropping steps, labels, metadata, and type](https://github.com/gastownhall/gastown/issues/4732) | **81** | Patrols cannot complete and health checks pass over missing structure because zero matches are treated as success. | Materialization validates the full canonical IR and read-backs its manifest before acknowledging; zero-match health probes are UNKNOWN. |
| 25 | [#4638 — Ambient tmux socket override creates a split-brain town](https://github.com/gastownhall/gastown/issues/4638) | **80** | Two Mayors and two Deacons can run for hours and write the same database because inherited environment silently overrides canonical identity. | Runner/session identity is issued by the control plane and includes configuration generation; ambient environment cannot create authority. |
| 26 | [#4614 — Health output can recommend killing another town’s live database and miss real zombies](https://github.com/gastownhall/gastown/issues/4614) | **80** | Name matching is unscoped, wrong in both directions, while backup staleness remains permanently green. | Typed observations include ownership, process incarnation, scope, source, confidence, and contradiction; destructive policy rejects UNKNOWN. |
| 27 | [#4698 — Zombie restart re-primes work that is already closed and merged](https://github.com/gastownhall/gastown/issues/4698) | **79** | Stale durable hook state reactivates old work and creates conflicting duplicate merge requests. | Attempt/work identity is terminal-aware; restart adopts only current fenced work and verifies the source remains executable. |
| 28 | [#4580 — Concurrent claim and workflow launch can create duplicate active workflows](https://github.com/gastownhall/gastown/issues/4580) | **79** | Claim decoding failure plus non-atomic source launch lets one source acquire two live workflow generations. | Source admission, generation creation, lease, and routing are one idempotent transaction with a stable generation key. |
| 29 | [#4713 — Claimed nudge files can become permanently orphaned](https://github.com/gastownhall/gastown/issues/4713) | **78** | A crash after claim but before delivery removes messages from the visible queue without a reclaim sweep. | Inbox claims use renewable leases and an ack protocol; unacked claims return automatically after expiry. |
| 30 | [#4748 — Rejected merge requests and rejection reasons are hard-deleted](https://github.com/gastownhall/gastown/issues/4748) | **77** | The audit trail preserves accepted work but erases why unsafe work was refused. | Review and disposition records are append-only durable evidence; ephemeral delivery objects may be collected only after durable projection. |
| 31 | [#4719 — New-rig backlog can exist as a single local copy with no remote](https://github.com/gastownhall/gastown/issues/4719) | **77** | A core operational ledger lacks redundancy until manually configured. | Transactional state receives synchronous durability and tested backup/restore from first creation; no single-copy authoritative backlog. |
| 32 | [#4727 — Released Gas Town and Beads schema/runtime compatibility can diverge](https://github.com/gastownhall/gastown/issues/4727) | **76** | A release can ship binaries whose embedded queries and schema assumptions disagree, disabling cleanup and producing misleading health. | Signed compatibility manifest, schema/protocol negotiation, release canaries, and startup refusal on unsupported combinations. |
| 33 | [#3914 — Convoy can report complete with a pushed branch but no merge request](https://github.com/gastownhall/gastown/issues/3914) | **75** | Tracking completion is disconnected from protected integration state. | Mission progress derives from Candidate and Integration state, not issue closure or branch existence. |
| 34 | [#3603 — gt done can end sessions without committed or pushed work](https://github.com/gastownhall/gastown/issues/3603) | **75** | The lifecycle action trusts the agent’s completion path rather than requiring an exact deliverable. | Candidate preparation is trusted-runner code and refuses dirty, uncommitted, unpushed, or unidentifiable subjects. |
| 35 | [#2386 — Deacon patrol can report hundreds of fake cycles without executing required steps](https://github.com/gastownhall/gastown/issues/2386) | **74** | Root-only workflows and conflicting prompt instructions allow throughput theater while safety checks never run. | Required steps/gates are typed workflow state with proof artifacts; a model report cannot substitute for execution evidence. |
| 36 | [#4722 — Provider hook/config directories can be committed into product repositories](https://github.com/gastownhall/gastown/issues/4722) | **73** | Runtime overlays for Copilot, Cursor, Codex, Gemini, and others leak into the deliverable and can break host tools. | Provider configuration lives outside source; runtime-artifact policy is provider-complete and Candidate scanning is authoritative. |
| 37 | [#4666 — Nudge delivery can report success when tmux silently drops input](https://github.com/gastownhall/gastown/issues/4666) | **73** | A transport response is mistaken for delivery and processing. | Structured provider protocol with accepted/queued/processed acknowledgments; PTY is forensic fallback only. |
| 38 | [#3906 — Context-saturated sessions freeze silently and require manual terminal intervention](https://github.com/gastownhall/gastown/issues/3906) | **72** | The model cannot execute its own handoff after reaching the limit, while the system still reports it as working. | Context telemetry, proactive Context Capsules, supervisor-driven handoff, and explicit CONTEXT_EXHAUSTED state. |
| 39 | [#4712 — Dashboard can claim Live with no trustworthy freshness or deterministic identity](https://github.com/gastownhall/gastown/issues/4712) | **71** | A projection presents current-looking state without source sequence/freshness guarantees. | Every view carries as_of_sequence, projection lag, source health, exact IDs, and confidence. |
| 40 | [#4731 — One bad rig/database can abort town-wide daemon dispatch](https://github.com/gastownhall/gastown/issues/4731) | **70** | Failure isolation is insufficient; a local fault becomes a fleet-wide scheduling outage. | Partitioned queues and per-repository circuit breakers; one repository cannot abort unrelated reconciliation or dispatch. |

---

## 6. Deep briefs on the highest-risk reports

### [#4584](https://github.com/gastownhall/gastown/issues/4584) — Concurrent rollback destroys somebody else’s successful work

**Observed/report claim.** A failed sling enumerates every molecule associated with the source bead and burns them, including artifacts owned by a different successful concurrent sling. It can delete the healthy branch, release its claim, and deregister its worktree. The local operation looks clean because its own artifacts also disappeared.

**Architectural root.** The rollback key is the business object (`beadID`), not the exact command/Attempt that created each effect. There is no permanent fencing epoch preventing a stale compensator from mutating a newer generation.

**Centerrail requirement.** Rollback is a saga compensation over effect IDs created by the same command. Every branch, workspace, workflow generation, claim, and Effect stores its creating Attempt/fence. A mismatched compensator is a no-op plus a security event.

### [#4478](https://github.com/gastownhall/gastown/issues/4478) — Name reuse turns asynchronous cleanup into a cross-incarnation delete

**Observed/report claim.** A polecat name was force-nuked, then reused about a minute later while cleanup was still in flight. The old cleanup deleted the new worker's worktree. The worker fell upward to a shared repository and its safety net committed 198 unrelated files to main.

**Architectural root.** A human-readable slot name doubles as lifetime identity. Cleanup has no incarnation tombstone/fence and the fallback environment is writable.

**Centerrail requirement.** Names are presentation only. Each allocation has a never-reused incarnation and workspace nonce. Cleanup is fenced. A missing workspace cannot fall back to any writable parent; the runner terminates and preserves context.

### [#4397](https://github.com/gastownhall/gastown/issues/4397) — A safety command detects failed preservation and deletes anyway

**Observed/report claim.** Non-force nuke printed `WORK AT RISK` after preservation pushes failed, yet removed the worktrees containing 14–46 unpushed commits. Recovery required Git object archaeology.

**Architectural root.** Preservation is advisory output rather than a transactional precondition. Dry-run and execution do not share a final authoritative decision.

**Centerrail requirement.** Cleanup requires a verified bundle/branch/object-store receipt naming the exact tree and reachable commits. Without it the state is `PRESERVATION_FAILED`, cleanup is forbidden, and the portal opens an Intervention.

### [#4605](https://github.com/gastownhall/gastown/issues/4605) — Garbage collection erased a live merge queue

**Observed/report claim.** The first refinery patrol step ran age-based wisp GC. Nine ready MRs older than one hour disappeared before the refinery read its queue. Their rows were Dolt-ignored and therefore absent from history.

**Architectural root.** Retention uses age as a proxy for terminality, and a delivery/storage optimization made live obligations unrecoverable.

**Centerrail requirement.** Merge queue entries, reviews, rejection reasons, and integration obligations live in durable transactional tables. Retention is state-and-class based. Deletion emits a durable tombstone and requires no active references.

### [#4593](https://github.com/gastownhall/gastown/issues/4593) — Repair mutated production identity without disclosing it

**Observed/report claim.** `gt doctor --fix` renamed the main database to satisfy an inferred missing rig database, taking the town offline. Its summary reported only an unrelated identity-collision fix.

**Architectural root.** Diagnosis, repair planning, and mutation are collapsed; the tool has broad ambient authority and no complete effect ledger.

**Centerrail requirement.** Doctor is read-only. A separate repair command generates a signed plan containing every before/after value, precondition, blast radius, backup receipt, approval need, compensation, and postcondition.

### [#4629](https://github.com/gastownhall/gastown/issues/4629) — A valid-looking detached-HEAD result poisoned every clone’s default-branch pointer

**Observed/report claim.** `git rev-parse --abbrev-ref HEAD` returned the literal `HEAD` with exit code zero. Completion pushed a real branch named HEAD, and later fetches made `origin/HEAD` a non-symbolic pointer to unmerged work.

**Architectural root.** String-shaped output was accepted as semantic branch identity, and the push boundary did not reject reserved refs or verify remote consequences.

**Centerrail requirement.** Candidate preparation resolves symbolic branch state structurally. Delivery pushes explicit OIDs to an allowlisted ref namespace and verifies both remote ref and target branch. Reserved names are impossible.

### [#4602](https://github.com/gastownhall/gastown/issues/4602) — Upward Git discovery rewrote the wrong repository

**Observed/report claim.** A crew directory without its own `.git` resolved to the enclosing workspace. Session setup wrote the rig's remote into that outer repository, redirecting subsequent pushes to another project.

**Architectural root.** Filesystem location and Git discovery are treated as repository identity.

**Centerrail requirement.** Repository ID, expected root inode/path identity, remote identity, and base commit are supplied by the control plane. Every Git mutation validates all of them; no upward discovery is used for authority.

### [#4672](https://github.com/gastownhall/gastown/issues/4672) — A background checkpoint corrupted the writer’s live Git state

**Observed/report claim.** The checkpoint dog ran `git add -A` and commit during a conflicted cherry-pick. Git interpreted the commit as the resolution, removed CHERRY_PICK_HEAD, and stored conflict markers.

**Architectural root.** A maintenance actor shares a writable Git admin directory with the writer and uses mutating Git commands for preservation.

**Centerrail requirement.** Only the Attempt runner can mutate its clone. Checkpointing snapshots files/index/refs from outside the mount or asks the writer to quiesce; it never stages or commits. Sequencer/unmerged states are first-class.

### [#4527](https://github.com/gastownhall/gastown/issues/4527) — Internal success is not tied to durable storage success

**Observed/report claim.** Sling may show a green hook assignment while later writes warn or disappear under autocommit/connection behavior. Different code paths disagree about whether `hook_bead` or assigned work is authoritative.

**Architectural root.** The logical transition spans multiple writes and representations without one transaction and without a readback contract.

**Centerrail requirement.** One command performs one serializable transition. The authoritative assignment is a single schema object. Commit acknowledgment precedes outbox dispatch; read models update afterward.

### [#4469](https://github.com/gastownhall/gastown/issues/4469) — Verification can be green even though no verification happened

**Observed/report claim.** A nonexistent or misconfigured test command can fail the same way on target and branch, be called pre-existing, and license merge. Exit zero with zero executed tests is also accepted. Small models can call done with no deliverable.

**Architectural root.** Gate semantics live in formula prose and exit-code interpretation rather than typed evidence.

**Centerrail requirement.** A verifier adapter emits `PASS`, `TEST_FAILURE`, `CONFIG_ERROR`, `ZERO_TESTS`, `INFRA_ERROR`, etc., with an execution manifest. Policy says only PASS counts. Candidate preparation requires a non-empty authorized deliverable unless the contract explicitly expects no code.

### [#4699](https://github.com/gastownhall/gastown/issues/4699) — Submission is presented as landing

**Observed/report claim.** Source beads and convoys close during `gt done`, while the MR may still be queued, later rejected, or never pushed at all. Consumers treat closure as integrated truth and may duplicate or abandon work.

**Architectural root.** One status represents author completion, delivery, and integration.

**Centerrail requirement.** Separate Attempt outcome, Candidate delivery, Integration, and Observation objects. Dependencies declare which state they require. Source work remains in-flight until the exact target commit is verified.

### [#4632](https://github.com/gastownhall/gastown/issues/4632) — A failed read becomes a confident empty answer

**Observed/report claim.** When Beads hangs/fails, `gt hook` emits the exact same output and rc=0 as a genuinely empty hook. Direct SQL showed live work at the same instant.

**Architectural root.** Error handling collapses a three/four-valued world into a boolean.

**Centerrail requirement.** Typed observation envelope with source, result kind, freshness, confidence, and error. EMPTY is possible only after a successful authoritative query.

### [#4633](https://github.com/gastownhall/gastown/issues/4633) — A false negative reaches the destructive choke point

**Observed/report claim.** Role instructions tell agents to run done when hook and mail appear empty. Because hook failure looks empty, the command can push/submit and destroy a sandbox that still holds live work.

**Architectural root.** Safety is pushed to callers and prompt warnings instead of enforced at the action boundary.

**Centerrail requirement.** The trusted finalizer independently resolves current work and fence. UNKNOWN aborts. Any force override is human-authorized, reasoned, and audited.

### [#4634](https://github.com/gastownhall/gastown/issues/4634) — Critical control traffic disappears invisibly under load

**Observed/report claim.** A recipient queue capped at 50 rejects the newest arrival without recipient-side loss indication; reminders compete with urgent messages. A P0 escalation remained unseen for roughly a day.

**Architectural root.** A bounded ephemeral queue implements both low-value reminders and high-value control without durable overflow or priority admission.

**Centerrail requirement.** Priority classes, durable storage, per-class reserve, backpressure, dead letters, and receiver-visible loss. System reminders may be regenerated; P0 cannot be silently refused.

### [#4010](https://github.com/gastownhall/gastown/issues/4010) — A missing argument and absent defense-in-depth merged into the wrong branch

**Observed/report claim.** The formula omitted `--base`, so GitHub defaulted PRs to main. Refinery trusted the PR and did not compare its base to the MR target. Multiple branches intended for a feature branch landed in main.

**Architectural root.** External-effect target is repeated as loosely coupled strings across prompt/formula, MR metadata, and GitHub.

**Centerrail requirement.** One Effect Intent names exact repository, head Candidate, target ref, and expected target OID. Both broker and integration coordinator independently verify them.

---

## 7. Non-negotiable redesign invariants derived from the corpus

These supplement the existing Centerrail invariants and should be encoded in Rust transitions, SQL constraints, policy, sandbox configuration, repository rules, and executable tests—not merely documentation.

| ID | Invariant |
|---|---|
| **R1** | Every state-changing command names an organization, repository, Mission, Work Package, Variant, Attempt, fence, runner epoch, workspace nonce, configuration generation, and policy generation. |
| **R2** | Fence counters are monotonic, independent of active leases, and never reused. |
| **R3** | No destructive action may target a display name, cwd, prefix, branch name, tmux socket, PID, or filesystem path without resolving and validating the exact authoritative object identity. |
| **R4** | A stale Attempt cannot heartbeat, expand scope, checkpoint authoritatively, attach Evidence, create Effects, finalize, push, review, select, cancel, or clean a successor. |
| **R5** | Rollback and compensation may mutate only resources/effects created by the same command or explicitly adopted saga step. |
| **R6** | Private writable clone per Attempt; no two writers share worktree, .git, index, refs, mutable caches, browser profile, or provider runtime overlay. |
| **R7** | No maintenance worker may stage or commit inside a live writer clone. |
| **R8** | A destructive workspace action requires verified preservation or a policy-approved explicit abandonment record. |
| **R9** | Dry-run and execution evaluate the same pure decision function against an explicit snapshot; execution revalidates all preconditions. |
| **R10** | Probe results are VALUE, EMPTY, UNKNOWN, or CONTRADICTORY; failures never serialize as empty, healthy, dead, safe, or pass. |
| **R11** | UNKNOWN or CONTRADICTORY cannot authorize deletion, reassignment, merge, credential rotation, or remote mutation. |
| **R12** | No command reports success before its durable internal transition and required external postconditions are verified. |
| **R13** | Ambiguous external outcomes become OUTCOME_UNKNOWN and are reconciled before retry. |
| **R14** | Every external mutation has a durable Effect Intent, logical idempotency key, target identity, desired-state hash, preconditions, fence, policy, and reconciled Effect Receipt. |
| **R15** | No agent/model process receives SCM, production, cloud-admin, KMS, database-admin, or unrelated provider-profile credentials. |
| **R16** | Git delivery uses explicit source and target OIDs; reserved refs such as HEAD/FETCH_HEAD/ORIG_HEAD cannot be branch destinations. |
| **R17** | Every Candidate is immutable and identified by exact base, head, tree, patch, repository, producing Attempt, scope, configuration, and environment. |
| **R18** | Any Candidate mutation, rebase, retarget, or branch-head change invalidates all incompatible Evidence and review. |
| **R19** | Only the exact reviewed Candidate may enter protected integration. |
| **R20** | Completion is distinct from session exit, task closure, branch push, PR creation, queue entry, or check start. |
| **R21** | A Mission requirement completes only after protected integration, verified target commit, and required observation window. |
| **R22** | Only typed PASS can satisfy a required gate; FAIL, FLAKY, INFRA_ERROR, CONFIG_ERROR, TIMED_OUT, NOT_RUN, ZERO_TESTS, UNKNOWN, and UNSUPPORTED cannot. |
| **R23** | Writer evidence cannot satisfy independent-verification requirements. |
| **R24** | Test execution evidence includes exact command, environment, subject Candidate, executed-test count or equivalent manifest, and artifacts. |
| **R25** | Plans and workflow graphs are immutable, content-addressed, atomic, and idempotent; partial graph materialization is unrepresentable. |
| **R26** | Required workflow steps and gates are machine-accounted; narrative reports cannot close them. |
| **R27** | Dynamic scope creates a successor Attempt or atomic scope revision; the live process never receives surprise authority. |
| **R28** | Every durable obligation has an explicit retention/recovery class; ephemeral delivery objects cannot be the sole record of work, review, rejection, or escalation. |
| **R29** | Urgent control messages are durably pending, acknowledged, or visibly dead-lettered—never silently dropped. |
| **R30** | Message claims are leased and reclaimed; accepted, queued, delivered, processed, and expired are distinct. |
| **R31** | Every row/event/command carries explicit scope partition keys; prefix, path, cwd, and environment are not routing authority. |
| **R32** | One repository’s storage, provider, schema, or queue failure cannot abort unrelated repository processing. |
| **R33** | Every Attempt uses a complete immutable configuration and policy snapshot; ambient environment is sanitized and recorded. |
| **R34** | Schema/protocol compatibility is negotiated before mutation; normal startup never performs an unbounded migration. |
| **R35** | Diagnostics are read-only. Repair is a separate typed, audited Effect Plan with complete mutation disclosure and recovery metadata. |
| **R36** | Portal status includes as_of_sequence, projection timestamp, lag, source health, exact subject IDs, and observation confidence; rendering cannot create authority. |
| **R37** | Provider lifecycle, prompt delivery, context pressure, tool authorization, usage, and cancellation prefer structured protocol; PTY/ConPTY is compatibility and forensic data only. |
| **R38** | Audit records are append-only for all authority, destructive, credential, repair, Evidence, review, and external-effect decisions. |

---

## 8. Implementation epics

### E0 — Executable semantics and simulator

**Scope.** Formalize state machines for leases, fences, Candidates, Effects, messages, cleanup, and migrations. Build deterministic provider/SCM/storage simulators before live adapters.

**Exit gate.** Delayed stale commands, ambiguous effects, partitions, retries, and crashes cannot violate R1–R18.

### E1 — Authoritative ledger

**Scope.** Rust domain kernel with SQLite/PostgreSQL, serializable transitions, command idempotency, permanent fence counters, leases, ready queue, durable inbox/outbox, and append-only audit.

**Exit gate.** No duplicate graph, lease, command, or accepted stale transition under property and kill/retry tests.

### E2 — Private-clone runner and hostile Git boundary

**Scope.** Private clone factory, explicit repository identity, isolated HOME/XDG/Git config, process supervision, sandbox policy, runtime-artifact exclusion, checkpoints as patches/bundles, and salvage.

**Exit gate.** No worktree calls; concurrent workers and maintenance cannot cross-mutate; wrong-cwd operations fail closed.

### E3 — Candidate and Evidence model

**Scope.** Two-phase Candidate preparation, exact hashes, scope recomputation, clean reconstruction, Evidence tiers, invalidation engine, proof bundle, and independent verifier plane.

**Exit gate.** Changed or moving subjects cannot retain proof; writer output cannot satisfy independent evidence.

### E4 — Universal Effect Broker

**Scope.** GitHub V1 intents/receipts, expected-old-OID pushes, target/ref validation, idempotent PR operations, ambiguity reconciliation, secret/binary/scope scan, and target-commit verification.

**Exit gate.** Agent has no SCM credential; timeout/retry cannot create an accepted duplicate or wrong-target effect.

### E5 — Protected integration

**Scope.** Exact Candidate queue, blind review, target checks, merge-group verification, explicit rejection/rework disposition, integration serialization, and observation windows.

**Exit gate.** No branch movement, wrong base, zero-test gate, or submitted-only state can be reported as integrated.

### E6 — Typed workflow engine

**Scope.** Canonical workflow IR, atomic materialization, typed step/gate outcomes, required evidence, retries, compensation, dynamic Graph Deltas, and successor-Attempt scope changes.

**Exit gate.** Fake completion prose and partial/bare workflows cannot advance state.

### E7 — Durable control and messaging

**Scope.** Priority inbox/outbox, claim leases, acknowledgments, dead letters, backpressure, dedupe, causal IDs, and provider-structured delivery.

**Exit gate.** P0 traffic survives saturation and crashes; duplicates are harmless; no active user/model input is corrupted.

### E8 — Structured provider adapters

**Scope.** Capability negotiation, identity probe, lifecycle/usage/context events, pause/interrupt/cancel/resume, tool authorization, PTY fallback, pinned conformance suites, and canary rollout.

**Exit gate.** Provider drift cannot compromise the authority kernel; unsupported capability remains explicit.

### E9 — Context, quota, and cognitive routing

**Scope.** Canonical Context Graph/Capsules, coverage-checked compression, context handoff, quota vectors/reservations, cost attribution, model tiers, struggle escalation, and fusion protocols.

**Exit gate.** Context or quota exhaustion requires no terminal babysitting and cannot lose authoritative work.

### E10 — Configuration generations and scope isolation

**Scope.** Complete immutable configuration/policy snapshots, explicit scope IDs, staged activation, per-repository partitions/circuit breakers, and effective identity verification.

**Exit gate.** Ambient env/cwd/stale files cannot fork a town or retarget a command.

### E11 — Migration and release controller

**Scope.** Compatibility manifest, schema inventory, cost estimator, resumable migrations, maintenance mode, backups, restore tests, canary databases, and release provenance.

**Exit gate.** Unsupported binary/schema pairs refuse mutation; startup does not crash-loop on migration.

### E12 — Diagnostics, repair, and maintenance

**Scope.** Read-only diagnostics, typed repair plans, before/after manifests, preservation receipts, GC retention classes, remote publication verification, compensation, and audit.

**Exit gate.** Every mutation is reported; no repair acts on UNKNOWN; live obligations are never age-collected.

### E13 — Truthful engineering portal

**Scope.** Control Tower, Mission Graph, Fleet, Live Attempt, Session Supervisor, Behavior Center, Capacity, Context Lineage, Merge Rail, Incidents, and Audit using sequence-aware projections.

**Exit gate.** Portal never shows confirmed state without ledger evidence and exposes lag/contradiction explicitly.

---

## 9. Required fault-injection and regression matrix

| Scenario | Injection | Required result |
|---|---|---|
| **Concurrent rollback** | Run two launches for one source; fail the older after the newer owns resources. | Older rollback mutates nothing owned by newer Attempt. |
| **Delayed cleanup/name reuse** | Delay cleanup, reuse display name, then release cleanup. | Fence/tombstone rejects cleanup; fresh clone remains intact. |
| **Preservation failure** | Make salvage push/object upload fail before cleanup. | Non-force cleanup aborts; work remains; intervention is visible. |
| **Live-queue GC** | Age a READY/OPEN queue item beyond retention threshold. | It is never eligible; terminal audit/tombstone survives collection. |
| **Repair scope** | Feed doctor a misleading missing-database condition. | Read-only finding or exact proposed plan; no implicit rename/reassignment. |
| **Detached HEAD** | Finalize from detached HEAD and attempt push. | Preparation fails or trusted broker creates an explicit safe branch; reserved HEAD ref impossible. |
| **Wrong repository cwd** | Run Git mutation from a child directory without its own repository. | Expected repository ID/toplevel mismatch aborts before mutation. |
| **Git sequencer conflict** | Trigger checkpoint during merge/rebase/cherry-pick conflict. | Checkpoint captures external snapshot only; no staging/commit; sequencer unchanged. |
| **Lost database write** | Return SQL success then rollback/reset connection. | Command remains unconfirmed/failed; postcondition readback prevents success. |
| **Broken test command** | Exit 127, exit 0 with zero tests, malformed output. | CONFIG_ERROR/ZERO_TESTS; merge blocked. |
| **Moving branch head** | Force-push or append after review. | New Candidate; old Evidence invalid; queue holds. |
| **Wrong PR target** | Create PR against main when target is feature branch. | Effect precondition fails; no merge. |
| **Local-only effect** | Redispatch without repeating a no-push flag. | Structured policy follows Work Package; broker denies push. |
| **Submit without merge** | Push/create PR then stop refinery. | State is Delivered/Queued, never Integrated/Complete. |
| **Failed hook read** | Timeout/error storage read while work exists. | UNKNOWN; no empty result; destructive done denied. |
| **Queue saturation** | Fill normal/reminder queue then enqueue P0. | P0 admitted or durably spilled; recipient sees saturation/dead letter. |
| **Claim crash** | Crash after inbox claim before provider delivery. | Lease expires; message is reclaimed. |
| **Duplicate delivery** | Replay the same command/event repeatedly. | Idempotent one logical result; full audit. |
| **Split socket/config** | Inject stale socket/root/provider env. | Config generation/effective identity mismatch fails closed. |
| **Wrong database scope** | Write rig entity from town cwd and vice versa. | Explicit partition key directs one store; cwd ignored. |
| **One bad repository** | Corrupt one repo’s database/config. | Only that scope opens a breaker; others continue. |
| **Migration interruption** | Kill during each migration batch and restart repeatedly. | Resume or rollback from journal; no dirty ambiguous startup loop. |
| **Remote-effect timeout** | Commit remote push/PR then drop response. | OUTCOME_UNKNOWN; broker reads/adopts original effect before retry. |
| **Provider input collision** | Send urgent control during active user/model input. | Structured queue/interrupt protocol; input remains intact. |
| **Context exhaustion** | Reach provider context limit while busy. | Supervisor checkpoints and hands off via Context Capsule automatically. |
| **Projection lag** | Pause projector and mutate ledger/remote. | Portal shows stale/lagging, not current confirmation. |

---

## 10. What should not be ported

- Beads/Dolt, Git, Markdown, tmux, or filesystem state as the control-plane ledger.
- Writable Git worktrees for autonomous writers.
- Display names, prefixes, cwd, environment variables, tmux sockets, PIDs, or paths as authority.
- Root-only/prompt-only required workflows without machine-accounted proof.
- Agent self-reported done as completion.
- CLOSED as a union of submitted, queued, merged, abandoned, and session-ended.
- Branch names as reviewed subjects; all proof must bind exact Candidates.
- Age-only garbage collection for any live operational object.
- A lossy ephemeral nudge path for critical control or escalation.
- Success messages before readback of durable and remote postconditions.
- Generic doctor --fix or patrol formulas with broad destructive shell access.
- Repair, cleanup, or rollback keyed only by bead ID, worker name, or directory.
- Terminal scraping as the sole liveness/delivery/quota/context signal.
- Provider hook/config overlays written into product source trees.
- Automatic migrations in ordinary startup with no estimate, checkpoint, or recovery plan.
- Model or prompt ability to lower risk, waive evidence, authorize destruction, or bypass protected integration.

---

## 11. Prioritized build sequence

### Phase 0 — Model the dangerous semantics

Build executable models and simulators for leases/fences, exact ownership, command idempotency, cleanup, external Effects, Candidate drift, message claims, and migrations. Reproduce the top issue families as deterministic failing scenarios before implementing the product.

### Phase 1 — Smallest correct local authority kernel

Ledger, immutable plans, permanent fences, one writer per Variant, private clones, exact scope, durable commands, four-valued observations, checkpoints/salvage, and a minimal truth-only portal.

**Do not add a second real model provider until this phase survives kill/retry/partition tests.**

### Phase 2 — Exact Candidate and GitHub delivery

Candidate preparation, hostile-Git controls, Effect Broker, expected-OID pushes, target validation, PR idempotency, preservation receipts, and protected branch checks.

### Phase 3 — Independent verification and protected integration

Typed gate engine, clean verifier, Evidence tiers, blind review, queue subject pinning, merge-group tests, exact target-commit verification, rejection/rework state, and observation windows.

### Phase 4 — Durable control plane and structured provider adapters

Inbox/outbox, priority and dead letters, provider lifecycle API, context pressure, tool authorization, PTY compatibility, cancellation, identity probe, quota, and context handoff.

### Phase 5 — Workflow, routing, collaboration, and cognitive optimization

Canonical workflow IR, Graph Deltas, dynamic successor Attempts, task-specific routing, cheap-model portfolio targets, struggle escalation, fusion, councils, and isolated code races.

### Phase 6 — Team distribution, migrations, and operator product

PostgreSQL, mTLS runners, object storage, compatibility manifests, migration controller, high-isolation verification, complete portal, RBAC, and disaster recovery.

No phase exits while a safety invariant has a known counterexample.

---

## 12. Metrics and release gates

### Safety gates—target zero

- concurrent authoritative writer violation;
- stale-fence mutation or external effect;
- cleanup of a newer incarnation;
- shared writable checkout;
- false PASS / zero-test PASS;
- Candidate/Evidence subject mismatch;
- wrong-target integration;
- silent urgent-message loss;
- unreported repair mutation;
- live-object retention deletion;
- success acknowledgment without verified postcondition.

### Reliability gates

- p99 command idempotency replay returns the original result;
- recoverable runner loss restores work within SLO;
- 100% of critical messages become processed, durably pending, or visible dead letters;
- projection lag and source health visible on every portal page;
- schema migration interruption is resumable or compensatable at every boundary;
- provider adapter upgrades pass a pinned conformance suite and canary.

### Product gates

- accepted and surviving Mission value per engineer-week;
- human minutes per accepted Candidate;
- cost per accepted/surviving Work Package;
- first-pass clean-verifier rate;
- repair-loop count;
- routing calibration by task/repository/risk;
- escaped-defect and revert rates;
- intervention quality and time-to-resolution.

---

## 13. Compact issue-family index

This appendix is not a replacement for GitHub's live 333-item index. It groups the redesign-relevant/high-risk portion of the corpus so each architecture primitive can carry regression coverage for the reports that motivated it.

### Destructive cleanup, rollback, GC, and repair

[#4584](https://github.com/gastownhall/gastown/issues/4584), [#4478](https://github.com/gastownhall/gastown/issues/4478), [#4397](https://github.com/gastownhall/gastown/issues/4397), [#4605](https://github.com/gastownhall/gastown/issues/4605), [#4593](https://github.com/gastownhall/gastown/issues/4593), [#4737](https://github.com/gastownhall/gastown/issues/4737), [#4672](https://github.com/gastownhall/gastown/issues/4672), [#4604](https://github.com/gastownhall/gastown/issues/4604), [#4588](https://github.com/gastownhall/gastown/issues/4588), [#4615](https://github.com/gastownhall/gastown/issues/4615), [#4623](https://github.com/gastownhall/gastown/issues/4623), [#4651](https://github.com/gastownhall/gastown/issues/4651)

### False completion, lifecycle, and stranded work

[#4699](https://github.com/gastownhall/gastown/issues/4699), [#4698](https://github.com/gastownhall/gastown/issues/4698), [#4469](https://github.com/gastownhall/gastown/issues/4469), [#3603](https://github.com/gastownhall/gastown/issues/3603), [#3914](https://github.com/gastownhall/gastown/issues/3914), [#1893](https://github.com/gastownhall/gastown/issues/1893), [#4739](https://github.com/gastownhall/gastown/issues/4739), [#4734](https://github.com/gastownhall/gastown/issues/4734), [#4738](https://github.com/gastownhall/gastown/issues/4738), [#4583](https://github.com/gastownhall/gastown/issues/4583), [#4630](https://github.com/gastownhall/gastown/issues/4630), [#2635](https://github.com/gastownhall/gastown/issues/2635), [#3675](https://github.com/gastownhall/gastown/issues/3675)

### Merge queue, exact subject, target, and disposition

[#4627](https://github.com/gastownhall/gastown/issues/4627), [#4010](https://github.com/gastownhall/gastown/issues/4010), [#4606](https://github.com/gastownhall/gastown/issues/4606), [#4710](https://github.com/gastownhall/gastown/issues/4710), [#4748](https://github.com/gastownhall/gastown/issues/4748), [#4469](https://github.com/gastownhall/gastown/issues/4469), [#3914](https://github.com/gastownhall/gastown/issues/3914), [#4188](https://github.com/gastownhall/gastown/issues/4188), [#4668](https://github.com/gastownhall/gastown/issues/4668), [#4505](https://github.com/gastownhall/gastown/issues/4505)

### Authority, identity, and split-brain

[#764](https://github.com/gastownhall/gastown/issues/764), [#4527](https://github.com/gastownhall/gastown/issues/4527), [#2682](https://github.com/gastownhall/gastown/issues/2682), [#4733](https://github.com/gastownhall/gastown/issues/4733), [#4409](https://github.com/gastownhall/gastown/issues/4409), [#3763](https://github.com/gastownhall/gastown/issues/3763), [#4638](https://github.com/gastownhall/gastown/issues/4638), [#4637](https://github.com/gastownhall/gastown/issues/4637), [#4679](https://github.com/gastownhall/gastown/issues/4679), [#4709](https://github.com/gastownhall/gastown/issues/4709), [#4598](https://github.com/gastownhall/gastown/issues/4598), [#4478](https://github.com/gastownhall/gastown/issues/4478)

### Messaging, nudges, and event delivery

[#4634](https://github.com/gastownhall/gastown/issues/4634), [#1216](https://github.com/gastownhall/gastown/issues/1216), [#4607](https://github.com/gastownhall/gastown/issues/4607), [#4609](https://github.com/gastownhall/gastown/issues/4609), [#4610](https://github.com/gastownhall/gastown/issues/4610), [#4660](https://github.com/gastownhall/gastown/issues/4660), [#4666](https://github.com/gastownhall/gastown/issues/4666), [#4711](https://github.com/gastownhall/gastown/issues/4711), [#4713](https://github.com/gastownhall/gastown/issues/4713), [#4104](https://github.com/gastownhall/gastown/issues/4104), [#4514](https://github.com/gastownhall/gastown/issues/4514), [#2067](https://github.com/gastownhall/gastown/issues/2067)

### Workflow materialization and prompt-compliance gaps

[#2386](https://github.com/gastownhall/gastown/issues/2386), [#4732](https://github.com/gastownhall/gastown/issues/4732), [#4693](https://github.com/gastownhall/gastown/issues/4693), [#4600](https://github.com/gastownhall/gastown/issues/4600), [#4635](https://github.com/gastownhall/gastown/issues/4635), [#4612](https://github.com/gastownhall/gastown/issues/4612), [#4611](https://github.com/gastownhall/gastown/issues/4611), [#4615](https://github.com/gastownhall/gastown/issues/4615), [#3587](https://github.com/gastownhall/gastown/issues/3587), [#4738](https://github.com/gastownhall/gastown/issues/4738)

### Dolt, Beads, schema, routing, and durability

[#4749](https://github.com/gastownhall/gastown/issues/4749), [#4727](https://github.com/gastownhall/gastown/issues/4727), [#4718](https://github.com/gastownhall/gastown/issues/4718), [#4495](https://github.com/gastownhall/gastown/issues/4495), [#4702](https://github.com/gastownhall/gastown/issues/4702), [#4733](https://github.com/gastownhall/gastown/issues/4733), [#2682](https://github.com/gastownhall/gastown/issues/2682), [#4589](https://github.com/gastownhall/gastown/issues/4589), [#4570](https://github.com/gastownhall/gastown/issues/4570), [#4095](https://github.com/gastownhall/gastown/issues/4095), [#4254](https://github.com/gastownhall/gastown/issues/4254), [#764](https://github.com/gastownhall/gastown/issues/764), [#4719](https://github.com/gastownhall/gastown/issues/4719)

### Git/workspace isolation and contamination

[#4602](https://github.com/gastownhall/gastown/issues/4602), [#4629](https://github.com/gastownhall/gastown/issues/4629), [#4672](https://github.com/gastownhall/gastown/issues/4672), [#3737](https://github.com/gastownhall/gastown/issues/3737), [#4722](https://github.com/gastownhall/gastown/issues/4722), [#4688](https://github.com/gastownhall/gastown/issues/4688), [#4588](https://github.com/gastownhall/gastown/issues/4588), [#3772](https://github.com/gastownhall/gastown/issues/3772), [#4594](https://github.com/gastownhall/gastown/issues/4594), [#4478](https://github.com/gastownhall/gastown/issues/4478), [#4200](https://github.com/gastownhall/gastown/issues/4200), [#4209](https://github.com/gastownhall/gastown/issues/4209), [#4440](https://github.com/gastownhall/gastown/issues/4440)

### Health, liveness, and truthful observation

[#4632](https://github.com/gastownhall/gastown/issues/4632), [#4633](https://github.com/gastownhall/gastown/issues/4633), [#4631](https://github.com/gastownhall/gastown/issues/4631), [#4614](https://github.com/gastownhall/gastown/issues/4614), [#4597](https://github.com/gastownhall/gastown/issues/4597), [#4712](https://github.com/gastownhall/gastown/issues/4712), [#4595](https://github.com/gastownhall/gastown/issues/4595), [#4669](https://github.com/gastownhall/gastown/issues/4669), [#4704](https://github.com/gastownhall/gastown/issues/4704), [#4703](https://github.com/gastownhall/gastown/issues/4703), [#3998](https://github.com/gastownhall/gastown/issues/3998), [#4621](https://github.com/gastownhall/gastown/issues/4621)

### Provider integration, PTY, startup, and process control

[#3833](https://github.com/gastownhall/gastown/issues/3833), [#3735](https://github.com/gastownhall/gastown/issues/3735), [#3133](https://github.com/gastownhall/gastown/issues/3133), [#4670](https://github.com/gastownhall/gastown/issues/4670), [#4722](https://github.com/gastownhall/gastown/issues/4722), [#3836](https://github.com/gastownhall/gastown/issues/3836), [#506](https://github.com/gastownhall/gastown/issues/506), [#2480](https://github.com/gastownhall/gastown/issues/2480), [#1768](https://github.com/gastownhall/gastown/issues/1768), [#4380](https://github.com/gastownhall/gastown/issues/4380), [#4706](https://github.com/gastownhall/gastown/issues/4706)

### Context, handoff, and quota

[#3906](https://github.com/gastownhall/gastown/issues/3906), [#3909](https://github.com/gastownhall/gastown/issues/3909), [#3910](https://github.com/gastownhall/gastown/issues/3910), [#4052](https://github.com/gastownhall/gastown/issues/4052), [#1225](https://github.com/gastownhall/gastown/issues/1225), [#1969](https://github.com/gastownhall/gastown/issues/1969), [#2000](https://github.com/gastownhall/gastown/issues/2000), [#2073](https://github.com/gastownhall/gastown/issues/2073), [#2075](https://github.com/gastownhall/gastown/issues/2075), [#1668](https://github.com/gastownhall/gastown/issues/1668), [#1729](https://github.com/gastownhall/gastown/issues/1729)

### Multi-rig/town scope and blast radius

[#4731](https://github.com/gastownhall/gastown/issues/4731), [#4514](https://github.com/gastownhall/gastown/issues/4514), [#4718](https://github.com/gastownhall/gastown/issues/4718), [#3681](https://github.com/gastownhall/gastown/issues/3681), [#3763](https://github.com/gastownhall/gastown/issues/3763), [#4638](https://github.com/gastownhall/gastown/issues/4638), [#4733](https://github.com/gastownhall/gastown/issues/4733), [#4409](https://github.com/gastownhall/gastown/issues/4409), [#4104](https://github.com/gastownhall/gastown/issues/4104)

### Configuration, overlays, and ambient hidden inputs

[#4638](https://github.com/gastownhall/gastown/issues/4638), [#4594](https://github.com/gastownhall/gastown/issues/4594), [#4728](https://github.com/gastownhall/gastown/issues/4728), [#4586](https://github.com/gastownhall/gastown/issues/4586), [#4585](https://github.com/gastownhall/gastown/issues/4585), [#4667](https://github.com/gastownhall/gastown/issues/4667), [#2104](https://github.com/gastownhall/gastown/issues/2104), [#3874](https://github.com/gastownhall/gastown/issues/3874), [#4722](https://github.com/gastownhall/gastown/issues/4722)

### Dashboard, telemetry, and audit

[#4712](https://github.com/gastownhall/gastown/issues/4712), [#4661](https://github.com/gastownhall/gastown/issues/4661), [#4597](https://github.com/gastownhall/gastown/issues/4597), [#4596](https://github.com/gastownhall/gastown/issues/4596), [#4595](https://github.com/gastownhall/gastown/issues/4595), [#4748](https://github.com/gastownhall/gastown/issues/4748), [#2068](https://github.com/gastownhall/gastown/issues/2068), [#1827](https://github.com/gastownhall/gastown/issues/1827)

### Adaptive routing, model selection, and learning

[#1143](https://github.com/gastownhall/gastown/issues/1143), [#1729](https://github.com/gastownhall/gastown/issues/1729), [#1248](https://github.com/gastownhall/gastown/issues/1248), [#3909](https://github.com/gastownhall/gastown/issues/3909), [#2806](https://github.com/gastownhall/gastown/issues/2806), [#1553](https://github.com/gastownhall/gastown/issues/1553)

---

## 14. Source design documents reviewed

- [Repository README](https://github.com/gastownhall/gastown/blob/main/README.md)
- [Contributing / Zero Framework Cognition](https://github.com/gastownhall/gastown/blob/main/CONTRIBUTING.md)
- [Architecture](https://github.com/gastownhall/gastown/blob/main/docs/design/architecture.md)
- [Dolt storage](https://github.com/gastownhall/gastown/blob/main/docs/design/dolt-storage.md)
- [Polecat lifecycle](https://github.com/gastownhall/gastown/blob/main/docs/concepts/polecat-lifecycle.md)
- [Propulsion principle](https://github.com/gastownhall/gastown/blob/main/docs/concepts/propulsion-principle.md)
- [Molecules](https://github.com/gastownhall/gastown/blob/main/docs/concepts/molecules.md)
- [Scheduler](https://github.com/gastownhall/gastown/blob/main/docs/design/scheduler.md)
- [Mail protocol](https://github.com/gastownhall/gastown/blob/main/docs/design/mail-protocol.md)
- [Escalation](https://github.com/gastownhall/gastown/blob/main/docs/design/escalation.md)
- [Agent provider integration](https://github.com/gastownhall/gastown/blob/main/docs/agent-provider-integration.md)
- [Factory Worker API](https://github.com/gastownhall/gastown/blob/main/docs/design/factory-worker-api.md)
- [Model-aware molecules](https://github.com/gastownhall/gastown/blob/main/docs/design/model-aware-molecules.md)

---

## 15. Final recommendation

Gas Town's issue corpus should be treated as a **negative requirements library** for Centerrail. The fresh system should not attempt to patch each symptom independently. The recurring causes are primitive-level:

```text
missing exact ownership
+ missing permanent fencing
+ ambiguous completion
+ moving merge subjects
+ unverified external effects
+ lossy control delivery
+ prompt-defined safety
+ shared Git state
+ ephemeral durable obligations
+ hidden ambient configuration
+ repair without receipts
```

The first credible Centerrail milestone is therefore not a large swarm. It is a fault-injected demonstration that:

1. one Mission materializes once;
2. one Variant obtains a never-reused fence;
3. a stale duplicate cannot alter state or create an Effect;
4. useful work survives process death;
5. no other worker or maintenance process can mutate its clone;
6. a Candidate has exact lineage and scope;
7. failed reads remain UNKNOWN;
8. a clean verifier produces exact-subject Evidence;
9. a GitHub effect lands only the approved Candidate on the approved target;
10. no cleanup occurs until preservation/integration postconditions are verified; and
11. the portal reports every uncertainty, lag, and contradiction honestly.

Once that is repeatable, multi-provider routing and model fusion become acceleration. Before it, they are additional sources of concurrency and ambiguity.

---

## Audit limitations

- The snapshot is time-bound to 2026-08-24; issue state and main can change afterward.
- Open issues include reports against several releases; this document does not assert that every mechanism is present in the latest binary unless the issue explicitly says it is.
- Not every issue comment, linked private bead, local log, or fork patch was available or independently reproduced.
- Scores are architecture-priority judgments, not CVSS values and not a maintainer commitment.
- The report intentionally focuses on safety, authority, durability, and redesign leverage. Feature requests and low-risk UX/docs issues remain part of the 333-item corpus but are not individually ranked.

