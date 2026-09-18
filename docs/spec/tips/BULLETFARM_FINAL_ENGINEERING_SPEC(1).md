# BulletFarm 3.0
## Comparative adjudication and complete engineering specification

**Prepared:** 16 September 2026. **For:** Jepson Taylor, Alton and the implementation team.  
**Selected stack:** Rust hub/runner/custom CLI; Vite + TypeScript + React; one operational database and one authority.  
**Design assessment:** **94.38/100 (94.4 rounded)** under the same rubric applied to the five full input plans.  
**Status:** Proposed build contract. No application implementation, live provider qualification, security certification or independent fresh-agent rebuild is claimed.

> **People own outcomes. Small execution steps preserve clarity. Independent evidence earns shared consequences.**

## Decision in one page

Use E's coherent-task/ordered-step model as the work-unit foundation, C's operational restraint, D's precise accounting and reconstruction discipline, and A/B's harness, profile, privacy and learning contracts. Preserve Alton's human ownership, gate adequacy, shipping-versus-experiment separation and ability to replace the machinery. Do not concatenate the input architectures or adopt any upstream as a wholesale fork on documentary evidence alone.

One code task is one coherent reviewable PR, containing one to eight ordered execution steps. Each step starts fresh from a verified checkpoint and has bounded acceptance. The whole task retains ownership while its author stops and its candidate proceeds through independent verification and protected delivery. A shared hub coordinates multiple people without taking ownership of their product decisions.

The new contract explicitly selects one pause policy, one conservative boot policy, one greenfield Codex transport, one purpose-tagged Job model, staged draft/review/merge meanings, and a plan-sized first-pass allowance plus a finite shared repair pool. These close real handoff ambiguities; they are not additional services.

The common architecture scores are **A 92.08, B 92.26, C 92.74, D 93.08, E 93.62**, and **Final 94.38**. Differences this small are subjective judgment, not measured productivity or safety. The brief and two build guides/backlog are assessed separately for their actual roles. No original self-score is used as an empirical baseline.

## Reading map

| Need | Read |
|---|---|
| Understand the comparison and deductions | Part I: every full proposal, all supporting artifacts and the common rubric |
| Build the selected runtime | Part II §§0–24, then the exact API and wire appendices |
| Start without previous chat | `AGENT_START_HERE.md`; canonical BF3 backlog and Q3 cases |
| Inspect concrete machine contracts | `schemas/contracts.schema.json`, SQL, examples, defaults and qualification files |
| Verify old requirements were retained | Appendix F and `review/PRESERVATION.json`; archived source hashes |
| Distinguish evidence from promises | Validation section below and Appendix H |

## What was actually validated

**123 offline package/reference checks passed; 0 failed.** These check schema/examples, selected reference guards, in-memory SQL constraints, graph/case relationships, source hashes, test-inventory behavior and score arithmetic. They do not establish application operation. **All 124 runtime acceptance scenarios remain `not_run`.** The package has 24 core build packages and seven explicitly optional extensions; it contains no implemented `bf` executable.

The validation environment was Python 3.13.5, JSON Schema validator 4.26.0, and SQLite 3.46.1 **in memory**. This is not certification of the selected durable-disk SQLite runtime. Real OS isolation, provider behavior, forge races, performance, economics and the fresh-agent rebuild are separate required evidence.

Source-derived descriptions are labeled A–E and Legacy; freshly checked primary integration facts use W references. The final selections are new reviewer decisions, not claims that all source authors agreed. `archive/` contains historical inputs and cannot override the selected 3.0 instructions.



---

# Part I — Comparative review and final adjudication

**Review date:** 16 September 2026. **Scope:** five distinct full proposals, all ten current uploaded artifacts, and retained earlier requirements. This is a review of specified behavior and build instructions, not an audit of a deployed product.

## 1. Inventory: ten artifacts are not ten competing architectures

| Label | Current supplied artifact | Role in this review |
|---|---|---|
| A | `BULLETFARM_HARNESS_INFORMED_FINAL_SPEC(1).md` | Full harness-informed architecture, historical self-score 94.50. |
| A-brief | `BULLETFARM_DECISION_BRIEF(1).md` and `.pdf` | Two representations of A's decision brief; not two additional architectures. |
| B | `BULLETFARM_FINAL_PROPOSAL_AND_SPEC(1).pdf` | Complete 84-page harness-integrated architecture, historical self-score 93.65. |
| C | `BULLETFARM_FINAL_PROPOSAL_AND_ENGINEERING_SPEC(1).md` | Full human-owned build contract, historical self-score 93.43. |
| D | `BULLETFARM_FINAL_VISION_AND_REBUILD_SPEC(1).md` | Full three-crate rebuild baseline, historical self-score 94.06. |
| D-backlog | `BACKLOG(1)(1).json` | D's 24 core / six extension work packages. |
| D-guide | `START_HERE(1)(1).md` | D's three-crate, Codex-exec build entry point. |
| E | `BULLETFARM_VISION_AND_REBUILD_SPEC(1).md` | Full ordered-step workbench, historical self-score 93.86. |
| E-guide | `BULLETFARM_START_HERE(1).md` | E's one-package, one-to-eight-step build entry point. |

The current B PDF is byte-identical to the earlier harness-integrated PDF in this conversation. Its matching Markdown and prior rebuild ZIP were available and used for programmatic comparison. That is an established file identity, not a filename inference. A's MD/PDF brief expresses the same recommendation; it is assessed for communication, not independently counted in architecture scoring. Source hashes and exact filenames are in `review/SOURCE_MANIFEST.json`.

The current D backlog has 30 unique package IDs, 24 marked core, six marked optional, and 120 unique acceptance IDs. Its dependency references exist and its graph is acyclic. These checks do not establish that its future cargo tests exist or pass. Its package names and three-crate paths align with D, not E. E expects `BUILD_BACKLOG.json` and BF-R identifiers, whereas D's supplied file uses BF2 identifiers. B also uses BF2 identifiers for different work. Combining them by ID would silently assign different requirements to the same identifier.

A, C, D and E refer to additional companion schemas, SQL or validators that were not supplied as part of their current standalone attachments. That is a handoff-availability limitation, not proof those files never existed. This review does not invent their contents or substitute B's similarly named artifacts. The final package supplies its own complete, internally consistent representations.

## 2. Method and fair comparison

Each full proposal is scored against the same twelve criteria below. The question is: how well does the written plan satisfy this user's entire engineering-product request, with a restrained build path and explicit interactions? Declared features receive design credit; declarations that tests previously passed do not receive runtime credit. More pages, more tests listed, more references and a higher original self-score receive no automatic bonus.

Historical self-scores use different weights and are not averaged. The current common scores use `sum(weight × score) / 100`. A one-point difference is not measured superiority. All five proposals share substantial ancestry; the useful result is selecting mechanisms, not proclaiming a decisive winner from decimals.

Anchors: 0–39 missing or contradictory; 40–59 aspirational with major gaps; 60–74 specified happy path; 75–89 meaningful mechanisms with material open interactions; 90–94 coherent, bounded and testable with explicit residuals; 95–99 exceptional closure of that written criterion; 100 would require no material scope-specific deficiency and convincing independent demonstration. Implementation qualification is reported separately at every score.

### Common weighted scorecard

| Dimension | Weight | A | B | C | D | E | Final |
|---|---:|---:|---:|---:|---:|---:|---:|
| People and product fit | 6% | 94 | 96 | 94 | 95 | 96 | 96 |
| MVP simplicity and sequence | 12% | 91 | 91 | 94 | 91 | 91 | 93 |
| Fresh-agent build precision | 12% | 89 | 91 | 91 | 93 | 92 | 94 |
| Task decomposition and planning | 10% | 90 | 90 | 91 | 89 | 96 | 95 |
| Unattended execution | 8% | 93 | 94 | 93 | 93 | 94 | 95 |
| Durability and recovery | 12% | 93 | 92 | 94 | 95 | 94 | 95 |
| Verification and integration | 10% | 93 | 92 | 94 | 96 | 96 | 95 |
| Team coordination and humans | 8% | 94 | 94 | 95 | 95 | 95 | 95 |
| Security and data boundaries | 8% | 92 | 92 | 92 | 93 | 93 | 93 |
| Economics and controlled learning | 8% | 93 | 93 | 93 | 95 | 94 | 94 |
| Harness and provider realism | 4% | 95 | 94 | 86 | 86 | 87 | 94 |
| Operations and release boundary | 2% | 91 | 91 | 92 | 94 | 93 | 93 |
| **Total** | **100%** | **92.08** | **92.26** | **92.74** | **93.08** | **93.62** | **94.38** |

E is the strongest whole-plan fit in this comparison, principally because its bounded ordered steps directly address the user's small-agent workflow without imposing an intermediate human merge. D is the strongest compact authority/accounting reference. C offers particularly clear minimal delivery and operator semantics. A and B contain the strongest harness-informed additions. None is selected unchanged.

### What each criterion examines

Within each category, four equally weighted inspection questions anchor the judgment. They are prompts for consistent evaluation, not fictitious measured sub-scores.

| Category | Four inspection questions |
|---|---|
| People/product | Does the owner retain intent? Are solo/team coherent? Are decisions actionable? Is the promise truthful? |
| Simplicity | Is deployment small? Is a first transaction isolated? Are optional capabilities truly optional? Is working code reused? |
| Build precision | Are definitions self-contained? Are schemas/algorithms aligned? Are prerequisites explicit? Can a new agent find the right artifact? |
| Tasks/planning | Is work independently understandable? Are PR units coherent? Are dependencies/checkpoints precise? Is debate bounded? |
| Autonomy | Is fresh context genuine? Are waits classified? Are retries finite? Are extra permissions distinguished from continuation? |
| Recovery | Is intent durable before effects? Are authority lifetimes separate? Are unknown remote outcomes retained? Is restore fenced? |
| Verification | Does the oracle detect behavior? Is the issuer trusted? Is the exact subject current? Is combined mission behavior checked? |
| Team/humans | Do claims outlive writers? Is scheduling fair? Is takeover safe? Are unmanaged-work limits explicit? |
| Security | Are code and credentials separated? Is intake constrained? Is current authorization checked? Is context audience-safe? |
| Economics/learning | Are all costs counted once? Do revisions preserve allowance? Are comparisons controlled? Can changes and machinery be rolled back? |
| Harness realism | Is one transport chosen? Are profiles qualified by role? Are children/memory bounded? Are reuse decisions reversible? |
| Operations/releases | Are blockers diagnosable? Are observations timestamped? Is release authority separate? Are targets distinguished from measurements? |

## 3. Feedback on A — Harness-informed baseline 2.1

**Common score: 92.08/100. Original self-score: 94.50/100 under its own rubric.**

**Source position.** A keeps a small Rust authority kernel and gives Prime RPC an early, concrete executor qualification opportunity. It treats OpenJarvis's engine/core boundary as a source-reuse candidate, and QM as a personal/shared-context and human-only-authority reference. It preserves incumbent execution where conformant rather than insisting on a greenfield provider migration. [A: Executive decision; R2–R6; H1–H7]

**Strongest contributions.** A is unusually specific about what the upstream sources actually establish: a model endpoint is not a coding worker; a trace database is not an immutable acceptance ledger; a Prime child admission handle is not a completed answer; and outer session-reset success is not proof a new session exists. It also separates task-local adaptation from promotion of shared skills, specifies current audience checks, and treats a restricted native credential profile as restricted rather than magically hardened. Its discussion of unknown PR outcomes correctly rejects a single eventually consistent “not found” as proof of nonapplication.

**Deductions and remedies.** A retains a task RunSpec plus a parallel model-job path and adds a separate ExecutionBinding to preserve previous schemas. That is defensible for migration but unnecessarily indirect for a clean build. The final contract uses one tagged Job and a required immutable profile reference for every model job. A's useful route-priority scores compare a Prime adapter, an OpenJarvis subset and QM patterns: different integration units. They cannot be read as a like-for-like whole-project ranking or combined with B's whole-foundation scores. The final review separates foundation replacement from component qualification, without inventing a numeric universal winner.

A retains single-invocation task/checklist semantics, so the user still needs a concrete mechanism to hand sequential small pieces to fresh cheap workers inside one coherent PR. E supplies that mechanism. Its preserved research/front matter, baseline body and appended harness contracts also require a reader to reconcile multiple layers. The final document integrates each chosen rule once; historical text is archived, not executable precedence.

**Keep / change.** Keep profile qualification, early Prime trial, engine/core spike and scoped learning. Simplify job and schema paths, add bounded step semantics, and make all research scores visibly about a specified integration decision. A is a strong harness/reliability reference, not the sole final build authority.

**What would raise confidence.** One pinned Prime RPC adapter demonstrates a real fresh session, full-root cancellation, contained children, accurate aggregate usage and post-author delivery under the same gates as the incumbent. A separate small OpenJarvis build spike reports actual dependencies and cancellation behavior. No upstream benchmark substitutes for either result.

## 4. Feedback on B — Harness-integrated Personal Agency edition

**Common score: 92.26/100. Original self-score: 93.65/100.**

**Source position.** B specifies one Rust package, a unified Job abstraction, personal/domain ownership, independent verification and a full execution-profile amendment. It uses Codex App Server and Claude headless as reference adapters, with useful method-level restrictions. It explicitly handles the PR-only dependency trap using an intermediate-merge policy. [B: core §§5–8, 10–18; Part III N1–N9]

**Strongest contributions.** The distinction between personal control and shared guarantees is clear. The code/research result separation is concrete; planning jobs do not need fake existing implementation tasks. Its accounting recognizes cumulative/inclusive child usage and explicitly avoids charging parent and child totals twice. Private summaries inherit source access restrictions, and revoked access invalidates cached inclusions. Excluding the documented out-of-sandbox Codex shell method from the agent interface is a valuable precise control, not merely a broad safety slogan.

**Deductions and remedies.** B is the most layered handoff: research, preserved core, new normative amendments, then long generated appendices. A retained core sentence still describes Prime as uninspected while its new parts describe selected source inspection. The truthful final state is “selected source reviewed; runtime not qualified.” This is an editorial/precedence defect, not a discovered security vulnerability. Its new profile binding is again a sensible migration patch but should become a direct core field in a clean-install contract.

Its `pr_ready` label can exclude some remote CI obligations, whereas other proposals use readiness more strictly. Neither naming convention is inherently impossible, but a pooled spec must choose one: the final contract distinguishes `draft_open`, `review_ready` and `merge_eligible`. B correctly exposes the intermediate-merge issue but postpones executable serial microsteps. E's bounded cursor removes that limitation without adding a same-PR DAG. B places a new Prime trial after the team/shadow path by default; the final selection gives a potentially working incumbent an earlier qualification opportunity, bounded so it cannot delay the first win indefinitely.

**Keep / change.** Keep the unified jobs, human experience, data-aware profiles, current access checks and method allowlist. Remove stale source-status wording, collapse the layered normative documents, promote profile identity into the core schema and adopt E's narrowly bounded steps.

**What would raise confidence.** A fresh agent implements the first path without needing to ask which layer overrides another. A two-step feature reaches a reviewable PR and accurately distinguishes incomplete CI, completed review and actual merge. Performance targets remain targets until a disclosed benchmark runs.

## 5. Feedback on C — Human-owned engineering build contract

**Common score: 92.74/100. Original self-score: 93.43/100.**

**Source position.** C favors a one-package Rust application, a dedicated SQLite worker, a headless first provider and one coherent task per PR. It explicitly distinguishes provisional draft publication from readiness when CI must run on the remote branch. It defines human parking and conservative boot-epoch recovery. [C: §§4, 10–16, 20, 23–24]

**Strongest contributions.** C is strong on the real delivery chain: untrusted bundle intake, exact candidate identity, independent reporting, staged PR publication, current integration, and actual merge-content mapping. Its `park` workflow acknowledges that retained reservations otherwise become operational hostage situations. Its accounting permits recording actual overspend rather than rejecting reality to preserve a misleading budget invariant. Human work does not require a fake provider heartbeat. These are valuable practical details.

**Deductions and remedies.** C keeps both a selection version and delivery generation with overlapping uses. The final clean-build contract uses one delivery version for selection/revocation and a separate writer generation; an implementation needs more counters only when it can name independently changing meanings. C's default single-task three-invocation model needs E's checkpoint cursor to serve sequential lightweight agents without expanding PR count.

The planning backlog asks the critic to form risks before exposure to the author's proposal while the ordinary flow is described as three operations. That independence needs an explicit protocol; a model cannot “unsee” text present in its input. The final Standard mode is candidly sequential draft/critique/revise. Deep uses two genuinely independent proposals followed by one synthesis. C's harness assessment predates the profile/Prime/OpenJarvis/QM refinements, so those are additions, not claims that C opposed them.

C rotates the boot epoch at every restart, deliberately interrupting active authors. This is a legitimate conservative choice, not a correctness defect. Its cost and operator effect should be acknowledged; the final MVP adopts this simpler policy, with sealed candidates preserved and reauthorized rather than maintaining implicit process reattachment.

**Keep / change.** Keep the minimal deployment, staged draft path, safe collector, parking and simple restart. Clarify planning independence, unify counters, add bounded steps and role-qualified full profiles.

**What would raise confidence.** Demonstrate a CI-only acceptance task progressing from an authorized provisional draft to actual readiness without a circular prerequisite; park an unresolved task without permitting its old PR to merge as current; and verify that a restart never turns an unknown effect into a duplicate create.

## 6. Feedback on D — Personal engineering agency rebuild baseline 2.0

**Common score: 93.08/100. Original self-score: 94.06/100.**

**Source position.** D uses three Rust crates inside one deployable binary, a unified bounded-job protocol, strong accounting and explicit root-work history. It includes a small controlled comparison demonstration in the team gate. Its current supplied JSON provides 24 core packages and six gated extensions. [D: §§7–16, 26–35; D-backlog]

**Strongest contributions.** This is the clearest compact reference for transaction guards and accounting. Budget scopes constrain the same leaf spend rather than representing additive charges. An actual bill above the estimate is recorded, not rejected by a permanent `spent + held <= limit` CHECK. Unknown exposure persists through cancellation and rollover. Explicit authoritative flags make the one-writer index meaningful without clock-dependent SQL predicates. It requires an actual second coding executor, not merely a second reviewer, and tests isolation across repositories.

**Deductions and remedies.** Three crates are not three services and are not a major problem. The final greenfield implementation selects one Cargo package to minimize bootstrapping choices; working compatible crates should remain. Root-level lifetime allowance is valuable, but plan admission must budget all initially intended steps/tasks as well as repair. A fixed three-invocation default for a root split into several planned jobs could block legitimate execution unless the allocator explicitly resolves that allowance. The final specification formalizes normal allocations plus a finite repair pool and never regenerates them from edited text.

D has no concrete ordered-step cursor, and its generic harness qualification does not yet contain A/B's detailed profile/memory/Prime constraints. Adding a paired comparison to the team release is defensible, but it should first validate the comparison mechanism with fakes; paid production experiments are not a hidden prerequisite for shipping ordinary work. The supplied backlog's generic `source_sections` text is less useful than exact section references, and proposed filtered cargo commands can accidentally match zero tests. The final acceptance harness fails when an expected test ID is missing or zero assertions execute.

**Keep / change.** Keep the accounting, explicit authority model, dependency-complete backlog and cross-repository isolation test. Add ordered steps and profiles; simplify clean-build packaging; tie allowances and test selection to explicit admitted plans.

**What would raise confidence.** Run the actual two-owner/provider demonstration, including real overage settlement, a task resplit under exhausted root allowance, and a restored backup containing outdated credential revocations. Its existing reference-test claims are not a substitute for those tests.

## 7. Feedback on E — Ordered-step workbench

**Common score: 93.62/100. Original self-score: 93.86/100.**

**Source position.** E separates a coherent PR-sized task from one to eight ordered execution steps. Each step uses a fresh session and private workspace from a verified immutable checkpoint, with a cumulative gate and a single `next_step` cursor. It preserves whole-task reservations and defines step-count-plus-two shared repair allocations. [E: §§5–9, 11, 24–27]

**Strongest contributions.** This is the most consequential functional improvement in the set. It preserves the user's small-agent strategy without requiring a review/merge after every handoff or adding a nested workflow graph. Cursor advancement is compare-and-set on current selection and exact evidence; changing an earlier step invalidates the affected suffix. One-step tasks use the same representation, so the first build slice is not throwaway code. The aggregate acceptance check and explicit zero-discovery/neutral/skipped handling also make the integration bar more precise.

**Deductions and remedies.** The cursor, checkpoint ancestry, suffix invalidation and shared retry pool are real added state. Keep one step as the default, implement multiple steps only after the single-step path works, and reject artificial checkpoints that do not verify useful behavior. E's BF-R015 combines planning, step execution and failure packets; that is too broad as a single implementation package. The final backlog separates the step engine from the planner while retaining one runtime mechanism.

E permits preserving an epoch on a normal restart but rotates on disaster restore; C/D choose new epochs at every boot. Both require careful implementation. The final MVP chooses conservative new-boot authority, states its interruption cost and leaves seamless reattachment later. E's generic harness interface still needs A/B's full-profile, child and private-memory requirements. Its current standalone upload does not contain the mentioned schema/SQL/qualification files, so those cannot be reported as inspected executable fixtures in this comparison.

**Keep / change.** Use E's task/step model as the work-unit foundation, plus its explicit contracts, strict acceptance aggregation and human semantics. Keep its extra state bounded. Add the more concrete donor-harness, accounting, effect and context requirements from A–D.

**What would raise confidence.** A two-step real feature completes with no intermediate human merge, fails correctly when the first checkpoint is revised, stays within its original repair pool, and reaches exact current integration evidence. A separate agent reproduces those behaviors using only the final kit.

## 8. Supporting artifacts: scored for their actual jobs

These role scores are not comparable to the twelve-category architecture score. A six-page brief should not be penalized for not containing a database schema; a backlog should be judged on executable handoff rather than brand language.

| Artifact | Role-specific score | Principal feedback |
|---|---:|---|
| A decision brief, Markdown | 93.70 | Strong decision clarity and source boundaries. Keep its route scores labeled as different reuse proposals; make its exact required full-spec filename unmistakable. |
| A decision brief, six-page PDF | 93.70 | Same decision content and score, not a second architecture. Selected page images are readable; avoid treating historical validation numbers on the cover as tests run in this review. |
| D JSON backlog | 91.15 | Strong IDs, scopes, dependency closure and future-test honesty. Generic source-section references and absent companion test inventory weaken standalone handoff. Bind exact section and case IDs; detect zero-test cargo filters. |
| D Start Here | 91.40 | Clear fixed choices and future-versus-current command distinction. Add a manifest/missing-file preflight so an agent cannot substitute B/E companions with colliding names. |
| E Start Here | 90.50 | Clearly describes ordered steps and original build gates. Map `SPEC.md` and companion filenames to the actual distributed files; require schema validation in CI rather than allowing a skip to be mistaken for full qualification. |

Brief role weights: decision clarity 30%, fidelity 25%, limits/evidence 20%, navigation 15%, next action 10%. Scores 96/94/96/88/90 yield **93.70** for both representations of the same brief.

Backlog/guide weights: entry clarity 20%, contract consistency 25%, dependency/handoff precision 20%, evidence honesty 20%, companion availability 15%. D-backlog scores 92/94/96/94/75 yield **91.15**; D-guide 95/94/94/98/70 yield **91.40**; E-guide 94/92/93/98/70 yield **90.50**. Availability refers only to verified attachments for that particular package, not whether the author created a complete kit elsewhere. These support scores must not be substituted for design scores.

## 9. Adjudications adopted in the final contract

| Issue | Source alternatives | Final choice and reason |
|---|---|---|
| Work unit | A–D task/checklist; E ordered execution steps | E's bounded linear steps, one default, maximum eight; one task remains one coherent PR. |
| Controller | One Cargo package versus three crates | One greenfield package, preserve working conforming code; no module becomes a service by default. |
| Job protocol | Separate model jobs/attempts versus unified purpose-tagged job | One Job shape, task reference optional for mission planning; profile required for model jobs. |
| Codex reference | exec in A/C/D; App Server in B/E | App Server over local stdio for greenfield, method allowlist; retain an already conforming exec adapter rather than implementing both. |
| First executor | Prime early in A; defaults/incumbent elsewhere | Conforming incumbent first; bounded Prime trial at that seam; Codex fallback. Neither Prime nor a second provider blocks the first verified PR. |
| Stop semantics | Pause drains effects versus holds them | Pause holds new jobs and unsent privileged effects; stop also revokes active work/delivery; cancel terminalizes intent; withdraw safely releases reservations. |
| Restart | Preserve after reconciliation versus revoke each boot | New boot authority, bounded author interruption; sealed candidates survive and get reauthorized. No transparent reattachment in MVP. |
| Readiness | PR ready may exclude CI in B; stricter check aggregation elsewhere | Separate provisional draft, review-ready, merge-eligible, merged and mission accepted. |
| Learning | Generic metadata versus complete profile and promotion | A/B full immutable profiles; D accounting; protected qualification and audience-scoped promotion. |
| Budget | Per-task three, root-wide cap, E steps plus two | Allocate first passes plus a shared finite repair pool at plan admission; preserve root totals across split/revision. |
| Multiple repositories | Global plans or linked missions | One repository per executable mission; linked explicit cross-repo checkpoints, no atomic multi-repo release. |
| Preservation | Copy every old clause versus consolidate | Archive exact inputs and map durable requirements to new normative sections/tests; do not execute contradictory archived prose. |

## 10. Final assessment

The consolidated specified-design score is **94.38/100**, displayed as **94.4**. This is 0.76 points above E under this rubric, a small subjective difference, not a measured productivity improvement. The change earns its place through explicit contract closure: bounded steps, coherent state names, direct job-profile binding, staged remote verification, closed allowance accounting, one boot policy and one test-selection contract.

The principal remaining deductions are actual provider/OS credential boundaries, finite test adequacy, semantic conflicts, real-world cost and usability, and the unperformed independent fresh-agent rebuild. All runtime qualification statuses remain `not_run`. A weighted score cannot waive unauthorized spending, stale publication, forged evidence, wrong-subject integration, privileged execution of candidate code, or loss of acknowledged control state.


---

# Part II — The final engineering contract

## 0. Authority, scope and how to build

**BulletFarm 3.0 — adjudicated implementation contract, 16 September 2026.** This is a proposed product, not a running orchestrator. Its commands, endpoints and acceptance tests are interfaces to implement. A source review, schema test or in-memory transaction test is not provider qualification, OS isolation certification or an independent rebuild.

This contract replaces competing implementation instructions from A–E for a clean installation. It does not replace organizational security/data policy, current repository permissions, or human decisions. Earlier inputs remain archived and mapped for provenance. Do not make an agent reconcile two different meanings of `BF2-001`, `pr_ready`, `pause` or “first provider.”

`MUST` is a release obligation, `SHOULD` a default requiring a recorded reason to deviate, and `LATER` is not a hidden dependency. Follow the active organization policy, then authorized repository policy, then this versioned contract, then an accepted mission/task inside that policy. A conflict among normative prose, schema, example and test is a specification defect: stop the affected path, record an erratum, correct all representations, and rerun the affected tests. Do not silently choose the permissive interpretation.

The deliverable includes a single machine-readable contract schema, a reference SQL migration, valid examples, one canonical backlog and a case inventory. `AGENT_START_HERE.md` gives the order. An existing codebase may satisfy obligations through different internal files; map and preserve it rather than rewriting working software to match a diagram. The first-party runtime remains Rust. Python in this package is offline specification-validation tooling only.

## 1. Vision: personal leverage, shared evidence

**Give each person the ability to direct more engineering work without making the team absorb more uncertainty.** The person owns the objective, priorities, acceptable tradeoffs and delegated limits. BulletFarm makes those choices durable, turns uncertainty into bounded investigations, turns clarity into small execution packets, and returns evidence-linked progress.

The product is one lead, one first-party CLI, and one simple web view. It is not one immortal model session. A provider thread can disappear while the work, accountable owner, selected code, remaining money and next action remain intact.

The important experience is: **ask, inspect, steer, take over**. Routine status, lease renewal, accounting, cancellation and scheduling use deterministic code. Models are invoked for reasoning, implementation, relevant review and explanation. They cannot promote themselves into administrators, pronounce their own patch verified or rewrite the acceptance bar.

Five assets outlive this particular architecture: task/requirement corpus; meaningful gates; accepted decisions and bounded failure lessons; ownership and delivery policy; outcome and cost history. The scheduler, prompts, providers, batching and UI are replaceable. This adopts Alton's durable/disposable distinction while deliberately choosing one shared allocator rather than independently scheduling factories. [Legacy Alton §§1–3; A–E]

Success measures are accepted **root outcomes**, human active minutes, end-to-end lead time, all attributable machine cost and observed rework/defects. Keep units separate unless the organization explicitly selects a money/time conversion. Agent count, token share, commits, TODOs and PR counts are diagnostics, not the denominator of productivity.

A safe block is neither completed work nor an unsafe acceptance. Report those outcomes separately. A useful research result may disprove its hypothesis and still meet its contract. Passing finite checks establishes those checks on identified inputs, not arbitrary program correctness.

## 2. The product boundary and first useful wins

| Stage | What must be demonstrated | What it does not claim |
|---|---|---|
| G0: foundation | Fake provider/forge, current authority, contracts, budget/claim transactions, meaningful gate fixtures | Live providers, distributed reliability or full OS security |
| G1: first useful transaction | One owner, repo and conforming executor; one-step task; independent check; actual draft PR; post-author delivery; restart/lost-response handling | Automatic decomposition, whole-team operation or autonomous merge |
| G2: product/team MVP | Two owners, two runners, two real coding-provider families; owned domains; one-to-eight steps; lead/CLI/web; human flows; protected integration and recovery | All providers, active-active availability or production rollout |
| G3: earned extensions | A bounded adopter proves the specific extra provider/channel/local/evaluation/release feature | Permission to skip any core authority or evidence rule |

The **first product win** additionally requires a natural-language objective to become valid tasks and useful code without provider-terminal handoffs. A manually authored contract reaching a PR is the earlier engineering transaction, not that full product demonstration.

One repository belongs to an executable mission. A program spanning repositories uses linked missions and explicit version/artifact checkpoints. Report partial delivery honestly. There is no cross-repository atomic merge or release protocol in MVP.

Scope includes multiple registered repositories and role-based access, but start with one enabled repo. Test two repositories for data isolation. One local installation uses the same hub/runner protocol as team mode. An offline engineer may work in their own clone; an offline team runner may not invent new shared authority. Import/join is explicit and starts historical work paused.

**Binding exclusions:** no new Git implementation, forge UI clone, distributed broker, workflow language, workflow editor, vector database, active-active hub, peer database sync, unrestricted recursive swarm, native model training, learned scheduler, universal terminal scraper, automatic account pooling, new CI/deployment platform or always-running model supervisor. Do not deploy QM, OpenJarvis and Prime under one another. An optional feature is added to close a named failed case or measured limitation, not because it appeared in a reference architecture.

## 3. A small fixed architecture

```text
bf CLI                 React / TypeScript / Vite
     \                         /
      authenticated HTTP commands + SSE
                        |
              one Rust hub process
     current state / policy / scheduler / outbox
                |                  |
      local SQLite + artifacts     trusted forge publisher
                |                  |
       outbound Rust runners       existing forge / CI
                |                  |
    isolated provider and verifier jobs
                |                  |
       sealed candidates -> exact evidence
                                   |
                         protected integration
                                   |
                         existing release system
```

Use one Cargo package with a library and executable. `bf hub`, `bf runner` and CLI clients are process roles, not independent services. The hub serves static web assets. The publisher is a trusted role/module; source builds never run under it. Verification reuses isolated jobs with trusted reporting outside the candidate process, not another compulsory network service.

Greenfield choices: Tokio, Axum, Serde, rusqlite, Reqwest, Clap, tracing, a maintained UUID implementation and SHA-256 library. Resolve actual compatible versions, pin a supported Rust toolchain and commit lockfiles during bootstrap. Pin the web toolchain and package lock too. A document's guessed “latest” version is not evidence of compilation.

```text
Cargo.toml / Cargo.lock / rust-toolchain.toml
src/
  lib.rs, main.rs
  domain/         contracts, IDs, semantic validation, pure guards
  store/          SQLite worker, migrations, transaction helpers
  api/            authentication, commands, queries, SSE
  cli/            custom commands; never a second allocator
  control/        readiness, steps, budgets, reservations, recovery
  intent/         Git contract activation and source mappings
  runner/         spool, process identity, sandbox and artifacts
  providers/      fake, selected Codex, Claude; optional Prime/Grok
  verification/   recipes, independent results, aggregate checks
  delivery/       sanitized intake, forge effects and observations
  lead/           bounded planning, targeted context, findings
  people/         ownership, takeover, adoption, withdrawal
  reporting/      current projections, receipts and portable export
web/src/          one page, Mine/Team filters, detail drawer
schemas/          authoritative JSON wire contracts
migrations/       one schema lineage
fixtures/         deterministic provider/forge and tiny source fixtures
tests/            unit, contract, integration, fault and live suites
```

Use ordinary functions and typed modules before traits with only one implementation. No plugin registers new authority transitions. A provider adapter normalizes a harness; it does not assign tasks, settle authoritative evidence or call the forge directly.

### 3.1 Database and instance boundary

One database thread owns the SQLite connection. Apply short transactions; never hold one over model calls, networking, Git, sandbox creation, compilation or artifact upload. State mutations, audit events and pending-effect records commit together. Ordinary current tables plus audit/outbox are sufficient; full event sourcing is not required.

The database is on the hub's persistent local disk. Runners communicate over the API. Use foreign keys, WAL, `synchronous=FULL`, a bounded busy timeout, tested migrations and consistent backups. SQLite documents the same-host WAL constraints and a WAL-reset fix in 3.51.3, with selected earlier backports. Record the actual linked runtime and require the fix, not merely a recent Rust crate. Package tests on an older in-memory SQLite do not qualify production WAL. [W3]

Hold an OS-level installation lock. At each new hub process boot, generate a random boot authority ID outside rollback-prone backup state, start in reconciliation mode and revoke prior active execution. This simple policy may interrupt models and incur another bounded job; it is an explicit MVP availability/cost tradeoff. Sealed candidates and historical receipts survive. A cloned installation on another machine is not fenced by a local file lock: disable the old publisher or revoke its credentials before activating a replacement.

An existing conforming PostgreSQL backend can remain. Do not build two database engines or migrate working infrastructure for cosmetic consistency. One active allocator remains a requirement regardless of database.

## 4. People, identity and the scope of team awareness

Every mission/task has a human owner and domain. A domain identifies product responsibility; it is not a scheduler. Each runner, provider account, verifier profile and service installation also has an accountable owner. These identities can differ. Spare compute does not transfer credentials or domain authority.

Owners can prioritize their work, select eligible profiles, set budgets within shared limits, pause, take over and resolve product decisions. Organizational policy controls data/provider eligibility, required checks, credential privileges and release boundaries. Cross-domain work requires standing delegation or a current decision from affected owners. A common queue does not authorize silently taking over another roadmap.

Three basic human roles suffice: administrator, engineer and observer, each narrowed by repository membership and specific grants. Domain owner is an assignment, not automatic administrator status. Reuse organization identity. For an internal pilot, use narrow revocable tokens behind TLS; a loopback bootstrap is not a public multi-tenant auth design. Browser sessions need secure HttpOnly cookies, origin checks and CSRF protection. CLI secrets belong in the OS store or approved protected mechanism, never Git/browser local storage.

Authenticate before checking idempotency or returning resource details. Request payloads may request ownership but cannot assert the caller's identity, reviewer status or authority. Recheck membership on reads, artifact access, search, event delivery and queued effect execution. A digest is not an access token. Close or refilter streams after revocation.

Authorized collaborators see outcomes, owners, intended scopes, selected diffs, acceptance and cost quality. Raw exploratory conversations remain owner/operator restricted according to declared retention. Do not pretend administrators and backups cannot access data they legitimately control. Private drafts do not reserve shared work or grant execution. The hub observes registered work and open PRs, not unsaved/unpushed edits on an unregistered laptop.

## 5. Vocabulary and one authority per fact

| Record | Definition |
|---|---|
| Mission | Owned root outcomes, repository, delegated envelope, total budget, terminal boundary |
| Plan revision | Immutable accepted tasks, decisions, dependencies and requirement coverage |
| Task | One coherent code change/PR, or one bounded research deliverable; stable across revisions |
| Step | Ordered executable packet within a code task; no separate PR, ownership or scheduler |
| Job | One recorded planning, writing, research, review, verification or human invocation |
| Candidate | Exact admitted code commit/tree and ancestry, not a branch name |
| Evidence | Trusted observation about an exact subject and recipe; immutable history |
| Decision | A current bounded question/action, owner, options, subject and expiry |
| Receipt | Exportable snapshot of known outcomes, evidence, uncertainty and next obligations |

Git owns immutable intent content. Protected configuration owns permissions and trusted gate definitions. SQLite owns current operational authority, jobs, budgets, reservations and selections. The forge owns actual remote refs/PR/merge facts. Trusted execution/reporting owns check observations. The release system owns deployed artifact and health observations. Every displayed fact identifies its authority and freshness.

A task phase is a projection, not a writable status supplied by a model. A successful provider exit only establishes that the invocation ended. A locally passing check is not a remote integration result. No `done=true` field can bypass these distinctions.

## 6. Wire contracts and immutable identity

Use versioned strict JSON, UTF-8, duplicate-key rejection and bounded bodies. IDs are opaque ASCII identifiers; versions/cursors are positive integers bounded to JavaScript-safe range. Monetary microunits are nonnegative decimal strings on the wire, parsed with checked signed-64-bit arithmetic. Zero is valid; missing authority is not zero or unlimited. MVP uses one explicit USD accounting domain and performs no implicit currency conversion.

Artifact/contract digests are SHA-256 of the **exact stored bytes**. Retain those bytes; do not reserialize and claim the same byte identity. No self-hash or enclosing not-yet-created Git commit belongs inside its own hashed input. Command replay hashes the exact submitted body; clients retain and retransmit it. Equivalent JSON with different bytes under the same command key is a conflict, not an excuse to infer equivalence at a privileged boundary.

Git objects carry their repository's object format. Accept SHA-1 or SHA-256 shapes only when the repository declares that format and the object exists with expected type/reachability. A syntactically correct fixture OID is not a live binding. Examples in this pack deliberately have no enabled live authority.

The single `schemas/contracts.schema.json` defines Task, ExecutionProfile, Job, ProposedResult, Evidence, Plan and Command shapes through `$defs`. Semantic validation is additional: current rights, cross-references, same repository, dependency existence/revision/cycles, scope containment, parent requirements, actual object identity, budgets and trusted producers cannot be established by JSON Schema alone.

### 6.1 Semantic admission rules

Reject empty/vague outcomes; missing owner/root/requirement coverage; unknown or canceled/superseded dependency revisions; cycles; invalid source paths; unsupported profiles; missing gate methods; unauthorized costs; a cross-repo executable edge; a code task with zero or more than eight steps; a research task with code-writing steps; and a step writing outside the full task scope. Structural validity is not a claim of a good plan: semantic review remains required where policy specifies it.

Path scopes are literal `{path, kind: file|directory}`. Reject absolute paths, NULs, `.`/`..` components, repeated separators and backslashes. A directory matches descendants on segment boundaries: `src/a` does not include `src/ab`. Normalize actual tree entries safely. A string-prefix function does not contain symlink/archive attacks. Scope defines admissible delivered changes; an OS sandbox confines transient behavior.

A new feature's future test function need not exist before implementation. Its trusted **method of evaluation** must be defined or independently adopted. The executor can write tests; it cannot redefine what the tests must demonstrate. Separate method/recipe custody from the candidate's test implementation. Use a no-op candidate to check that a new feature cannot pass merely because the old test suite stays green.

## 7. Git intent and versioned dependencies

Use one protected `refs/heads/bulletfarm/control` per repo. Only the metadata publisher updates it, with expected-old-object and fast-forward requirements. Metadata updates run contract checks, not application deployment. The candidate's local `.bulletfarm` files never become effective policy.

```text
missions/<mission>/revisions/<n>/mission.json
missions/<mission>/revisions/<n>/plan.json
missions/<mission>/revisions/<n>/tasks/<task>.json
missions/<mission>/revisions/<n>/decisions/<decision>.json
```

Do not put heartbeats, token streams or continuously changing task state in Git. Markdown and JSONL views are exports. Import old TODO Markdown and `AGENT_CHAT.md` as proposed context, preserve IDs/provenance, and require validated adoption; absence/deletion is never completion.

Plan activation is a durable sequence: validate proposal; finalize immutable artifacts; record metadata-publication intent; publish/observe exact Git revision; atomically activate the whole graph in SQL with event and eligible work. Repeated activation of the same digest returns the existing graph. A crash after Git publication leaves an inert revision, not a partially authorized task list. Git has expected-old-value ref updates, but they do not replace the whole claim/budget protocol. [W4]

Each dependency pins predecessor task ID and revision. Separate code tasks wait for the predecessor's observed protected merge; research dependencies wait for accepted artifacts. A PR-ready predecessor does not satisfy `merged`. Contract revision supersession explicitly preserves unchanged work or invalidates affected unfinished tasks. Already merged changes remain history and need new corrective work, not a rollback of accounting or evidence.

A PR-only mission with code dependencies must declare intermediate protected merges as `human_checkpoints` or `preauthorized_protected_merge`; the latter requires an actual grant. With `none`, combine coupled work into steps of one coherent task or use independently deliverable tasks. Do not promise an uninterrupted overnight chain while hiding a necessary human merge. Program-level cross-repo coordination uses an explicit observed artifact/version checkpoint; there is no atomic multi-repo mission.

## 8. Small agents without fragmented pull requests

**A task is the review unit. A step is the fresh-worker unit.** Default to one step. Permit two through eight only when each produces a meaningful checkpoint and saves context/coordination effort. This is E's bounded improvement, narrowed to one linear cursor rather than a second graph. [E §§5–9]

Example: CSV export may need a serializer, endpoint wiring and browser flow that share one evolving interface. Three independent PRs could each be unbuildable or need repeated human merges. One task can contain two sequential steps: implement serializer plus regression behavior; then wire endpoint and browser behavior, rerunning serializer obligations. Final acceptance covers the whole export and JSON compatibility. Independent unrelated fixes remain separate tasks and run concurrently.

Each Step contains an ID, outcome, instructions, write-scope subset and trusted checkpoint checks. It has no separate owner, global queue, publication authority or free budget. `next_step` ranges from 0 to N. The task reserves its full declared path/interface union until delivered, withdrawn or explicitly transferred. Only one production writer exists per task at a time.

### 8.1 Checkpoint protocol

For cursor `k`, start fresh from the admitted checkpoint for `k-1` or the resolved base if `k=0`. Pin task revision, base, checkpoint, current selection version, profile and allowance. Run the writer in private mutable state. Freeze/stop writing, finalize the bundle, admit the exact candidate under still-current author authority, and release the model's compute slot. Independent verification can continue after the author exits.

For step `k`, run the union of checkpoint obligations for steps `0..k` on the **current candidate**, not a collection of old green SHAs. Also run protected always-required hygiene checks. Advance from `k` to `k+1` only inside a transaction checking current task revision, selected candidate, delivery version, exact step identity and complete trusted evidence. Duplicate or delayed results cannot advance twice.

When `k+1=N`, run every mandatory final task acceptance check and required semantic review. Earlier checkpoint evidence is not final acceptance; a later change may invalidate earlier behavior. Final publication follows §15. Step checkpoints are not separate PRs by default.

### 8.2 Repair, replan and suffix invalidation

Repair creates a new Job and fresh context from the appropriate sealed candidate, with precise failed-check facts. A new candidate increments delivery version and invalidates earlier current-evidence eligibility. Histories remain immutable.

If repairing a later step can remain inside its scope while satisfying cumulative checks, retry that step. If an earlier interface/step changes, record an explicit rewind point, invalidate the dependent suffix, and rerun it against the revised checkpoint. No hidden reuse of downstream results is allowed. Rewind does not create extra allocations; insufficient remaining allowance creates a bounded owner decision.

Do not invent checkpoints simply to use weak models. A tightly coupled change without meaningful intermediate acceptance should be one step and may need a stronger model. A test-only red commit intended as temporary scaffolding is not an independently accepted checkpoint. The final task can include tests and implementation together.

### 8.3 An allowance that supports the plan

At plan admission allocate one initial writer invocation per planned step plus a finite shared repair pool, normally two per code task. Thus one-step work defaults to three; a two-step task has four total writer allocations, not six. At most three invocations may target the same step under the current allowance. Default planning has three substantive jobs; reviews/research have distinct small caps but share monetary limits.

Root-work limits aggregate all descendant task allocations. Splitting/respecifying consumes the **remaining previously authorized root allowance**, not a newly regenerated `N+2` for each child. When the initial accepted plan deliberately contains multiple tasks for one root outcome, its root allowance must cover their intended first passes and authorized repairs. A later expansion requires an explicit allowance/budget revision. This closes both the retry-reset loophole and the fixed-three-jobs-for-many-planned-steps trap.

Reserve an invocation when the prepared job commits. A replay of that launch does not consume another allocation. An attempt canceled after dispatch still consumes its allocation; a pre-dispatch reservation can be released only with proof no invocation began. All observed spend remains. Transport retries with provider-ambiguous admission cannot silently create an unlimited series of jobs under one allocation.

## 9. Planning, research and the lead

Use one fixed flow, not a society of permanent managers.

| Mode | Model jobs | When |
|---|---|---|
| Fast | One proposal, then structural/policy admission | Small clear bounded changes |
| Standard | Draft, different eligible provider critique, revision | Ordinary meaningful feature work |
| Deep | Two independent proposals given identical initial context, then one synthesis | Important architectural or research uncertainty |

Standard critique is **not called blind** when the proposal is in its input. Deep proposals are formed before either sees the other's output. Three different brands do not establish statistically independent errors. A fourth job needs a named unresolved issue and available preauthorized allowance; do not debate until everybody agrees.

Findings identify requirement/step, affected source, severity, evidence/reason, suggested correction and a discriminating check where possible. Each material finding is addressed, rejected with a reason or explicitly unresolved. Neither praise nor a numerical confidence can waive a gate. Product tradeoffs belong to the owner. Testable disagreements should become bounded experiments.

Before code tasks exist, planning Jobs reference the mission, a read-only source snapshot and mission budget; `task_id` is null. They do not create fake implementation tasks or an unmetered second job engine. The same capability and accounting infrastructure serves all purposes.

A research task specifies question, hypotheses, frozen inputs, baseline, evaluation method, compute/time cap and artifact deliverable. Negative findings can satisfy it. No candidate Git commit is required for a report-only outcome. A proposed code change gets a normal code task. Distinguish mathematical proof, test evidence, measured benchmark and conjecture in the report. Numerical/performance claims specify datasets, seeds, environment, precision/tolerance and measurement protocol; do not tune against hidden acceptance cases.

The lead retrieves current authorized state before status-bearing answers. Cards/counts come directly from deterministic projections; free prose cannot change them. The lead may propose mission/task revisions, profile choices and actions through typed commands. It cannot approve its own new grant. A person's request in the first-party interface can resolve an explicit in-scope command, but a model paraphrase or repository instruction is not another human authorization.

## 10. Execution profiles and practical harness adoption

An immutable ExecutionProfile identifies harness/adapter revision and transport; configured provider/model and exposed version or alias limitation; inference engine and location; tool/skill configuration; context strategy; initial memory snapshot; environment/isolation profile; native delegation limits; and usage reporting basis. Credentials are references in private deployment configuration, never fields containing secrets in portable profile artifacts.

A profile digest does not certify itself. The trusted qualification registry records which profile passed which role, workload/risk, environment and test suite, when, by whom, and under which executable/config digests. A read-only planning qualification does not establish coding ability. Any relevant change invalidates affected qualification until a canary passes. Record when a provider alias can change silently; don't pretend to pin invisible weights.

The default install has no QM, OpenJarvis or Prime dependency. Preserve a functioning incumbent if it passes the same contract. Give Prime an early bounded qualification trial at the first-live-executor seam when authorized; do not make a successful experiment a prerequisite to G1. Greenfield fallback is Codex App Server, then Claude as a genuinely different coding-provider family. An already conforming exec adapter remains a reuse exception, not a requirement to implement two Codex transports.

### 10.1 Codex

Use App Server over local stdio, pin/generate protocol fixtures from the actual binary, initialize, create a new thread, start the permitted turn, consume normalized events and interrupt through the supported path. Do not resume/fork a planner's history and label it fresh. Use a narrow method allowlist. Exclude `thread/shellCommand` from unattended/Foreman access: official documentation says it executes outside the thread sandbox. The server itself must still be inside the declared external execution boundary. [W1]

Approval responses come from current controller policy and the exact request subject. Unknown authority-affecting requests fail closed. Do not expose arbitrary App Server methods to the user-facing model. A terminal menu's wording or option number is never a dependency.

### 10.2 Claude

Use one pinned headless structured interface controlled by Rust. Explicitly define auth, HOME, settings, hooks, skills and MCP loading. Current documentation states that ordinary headless use can load project hooks without the interactive trust dialog, whereas bare mode skips automatic discovery but changes authentication behavior. Use an API-oriented bare profile when eligible; do not apply it blindly to subscription auth. Probe version-specific unattended controls rather than hard-coding a flag from a different installation. [W2]

Native signals and exit codes are observations; confirm the managed sandbox/process tree stopped. A final result followed by pending descendants is not complete physical termination. No unapproved shared host background process is retained.

### 10.3 Prime Agent

Prime is an executor candidate, not the team authority. Use one pinned RPC adapter if the trial is selected. Verify an actual fresh session ID/history, not just a successful reset response: the inspected RPC contract can return `success: true` with `cancelled: true`. Prompt/spawn acknowledgements are not final results. A resident daemon or schedule surviving closed stdin must remain inside the job's containment and accounting; don't attach a production task to a user's global daemon. [W6]

Disable schedules, cross-session goals, background persistence and children unless a particular certified profile enforces their lifetime. Keep persistent REPL state inside one bounded Job where useful; a new step/owner/provider handoff starts with a fresh authorized packet. Shared learned skills require separate promotion. Cancel the exact root/daemon/kernel/child tree without a global shutdown that affects someone else's work.

### 10.4 OpenJarvis and local execution

OpenJarvis's inspected Rust workspace and engine trait make a narrow reuse test worthwhile. The engine is an inference boundary, not a complete coding-worker lifecycle or cancellation contract. [W7, W8] Start with the smallest useful engine/core boundary or trace-field mapping. Measure the compiled dependencies and runtime services; keep it only if it reduces owned code and maintenance. Its trace storage never becomes a second authoritative work/evidence database.

A local model endpoint needs a tested harness/tool loop for code-writing use. Freeze engine, model artifact/quantization/context settings and hardware. Qualification covers tool support, truncation, out-of-memory, timeout, cancellation and accounting. No silent local-to-cloud fallback when source policy forbids transmission. Loading and inference stay outside the hub. Local purchase/energy opportunity cost is not inferred from API token prices.

### 10.5 QM and other surfaces

Adopt explicit private/domain/shared context and human-only authority operations. A model calling a self-API remains a model actor. QM's inspected security document also states limitations around command heuristics and materialized secrets; those are reasons to test actual boundaries, not a security certificate. [W9]

An existing QM installation may call a scoped BulletFarm client for mission creation/status/proposals. It does not become another allocator, verifier or approver. A provisioned computer is not presumed to contain source or inherit a parent's access. Shared forum files are not a safe concurrent candidate workspace. Keep the engineering core independent of QM's complete company-assistant surface.

### 10.6 Fork decision and replacement test

No wholesale donor fork is selected. A pinned unmodified runtime is preferred. Fork a narrow component only when a failing necessary fixture proves a patch is required; record upstream revision, license/notice obligations, patch scope, owner and upgrade suite. A TypeScript/Python-to-Rust rewrite of a whole application is not a low-cost fork.

Reverse this decision if an equal-outcome prototype meets the entire relevant contract with less first-party implementation and lower operational burden. Do not compare an unbuilt spec score to an existing project's adoption-fit score as proof of superiority. Root licenses do not settle third-party dependencies, model-weight licenses or service-account terms. Live entitlements are onboarding inputs, not invented by the build agent.

## 11. The execution protocol and genuine automatic continuation

```text
probe(profile) -> role capabilities + exact installed identity
start_fresh(JobPacket) -> handle + observed session identity
events(handle) -> bounded normalized observations
respond(handle, exact permitted decision) -> acknowledgement, if supported
cancel(handle) -> requested / confirmed / unknown
collect(handle) -> candidate | artifact | findings | blocker | failure
```

The runner owns framing, process lifetime, limits and stop enforcement. A provider adapter does not set task state. Every input/output frame binds job ID and source identity; trusted infrastructure assigns actor, time and committed sequence. Preserve bounded raw frames for diagnostics but do not make the scheduler depend on vendor-specific nested JSON.

Launch flow: authorized `bf run`; bounded planning; validated immutable plan activation; atomic scope/capacity/budget claim; new provider session; task/step-local context; execution. There is no extra “Implement this plan?” prompt. `bf plan` remains deliberately draft-only, while its planning spend still needs authorization.

The compact packet includes exact task/step, code base and prior checkpoint, accepted interfaces/decisions, relevant source paths/symbols, check descriptions, allowed profile, remaining allowance, current failure facts and expected output shape. Use ordinary repository search before introducing semantic indexing. Do not transmit full transcripts to every worker. A useful negative lesson records applicability and exact evidence, not an eternal claim that a method never works.

| Observation | Response |
|---|---|
| In-policy routine continuation | Proceed automatically under the approved profile |
| Unknown or prohibited authority request | Deny/stop and create a typed owned exception |
| Product ambiguity | Ask the owner; disjoint work continues |
| Authentication/MFA failure | Disable that profile, expose supported owner reauthentication |
| Quota/rate limit | Respect cooldown; bounded retry or eligible fallback |
| Protocol/schema drift | Quarantine adapter version; no guessed success |
| Missing tools/disk/LFS/submodule | Block preflight before avoidable paid calls |
| Context exhaustion | Seal known work, fresh successor inside remaining allowance |
| Repeated same verified failure | Change tactic, use permitted escalation, or block |
| Healthy quiet build | Continue inside declared tool/job deadline and authority |
| Expired lease/cancellation | Stop managed tree without model cooperation |
| Ambiguous provider admission | Track possible execution/spend; no blind duplicate job |
| Ambiguous forge effect | Reconcile exact remote subject before retry |

Automatic fallback never bypasses a denial, data restriction or provider policy. Native background agents are disabled unless bounded and attributed. One generation owns one mutable candidate workspace; child helpers are read-only or use separate scratch space with an enforced policy. An instruction telling a child not to write is not enforcement. Unsupported containment means the profile is ineligible for shipping, not that the controller pretends it succeeded.

## 12. Three authority lifetimes, explicit transitions

### 12.1 Writer execution

A writer Job binds boot authority, task/revision, step, runner incarnation, writer generation, packet digest and expiring lease. The hub increments generation on a new writer admission, and checks current authorization on every renewal/result. At most one current production writer exists per task; the database's partial unique index uses an explicit authoritative flag, not `expires_at > now()`.

Initial timing is ten-second heartbeats, sixty-second leases and a fifteen-second conservative margin. Compute the local deadline from the monotonic renewal-request start time, accounting for elapsed round-trip and suspend behavior; never extend from delayed response arrival. Stale generations cannot be revived by reconnecting. Server-side checks remain decisive even if a stale process physically continues.

### 12.2 Pending-change ownership

Reserve the full task path/interface union at first writer admission. This reservation persists through checkpoints, verification, human review and integration. Author exit releases compute, not the pending change. Age produces one owned attention item rather than silent expiration that admits a conflict.

A stopped human's laptop is not a reason to assign their work to a bot. Human reservations persist until explicit owner/repository resolution. Actual work on an unregistered branch is visible only after an observed PR or registered intent; do not claim perfect semantic conflict prevention.

### 12.3 Selected candidate and delivery

Admission of a sealed candidate under current writer authority increments a single `delivery_version` and selects its immutable identity. Candidate changes, cancellation, withdrawal or relevant revocation increment/invalidate that same delivery version. A separate redundant selection counter is not needed in the clean model. Writer generation and delivery version remain distinct because execution can end while delivery continues.

The author may exit normally immediately after sealing. Verifier/reviewer jobs bind the selected subject but not the author's live lease. A returned check for an older subject can be stored as historical evidence but cannot advance current state. Every privileged effect rechecks current delivery version, candidate, contract, policy and control state before issuance.

### 12.4 Stop, cancel and withdrawal

| Command | Selected behavior |
|---|---|
| `pause` | Hold new jobs and unsent privileged effects; observe active work and receive sealed results/evidence; do not launch a new verification job while paused |
| `resume` | Revalidate remaining authority, profiles, budgets, subjects and remote state before releasing holds |
| `stop` | Pause plus revoke current writer/delivery authority and request managed-tree termination; intent remains available for explicit later resume |
| `cancel` | Stop and terminalize intended work; retain unsettled effects and all history |
| `task withdraw` | Stop/revoke, reconcile outstanding effects, make old PR non-current/unmergeable through the protected path, then release claims while retaining salvage |

Already-sent remote actions may complete after these commands. Report requested, dispatch committed, termination confirmed, remote unknown and final observed outcome separately. A stopped or canceled intent cannot be silently marked accepted because a previously sent merge later succeeded; record the actual remote fact and the cancellation race for owner resolution.

If the forge cannot ensure an old PR is no longer eligible to merge, do not claim withdrawing a local grant closes that race. Keep conservative overlap handling and require a maintainer to settle the external change before releasing a conflicting managed reservation. A withdrawal is not a generic undo of merged code.

## 13. Scheduling and transactional accounting

The scheduler is an event/deadline-driven deterministic loop. It performs reconciliation and hard stop work first. Within an admitted priority band, rotate owners fairly; within each owner, prefer waiting age and explicit dependency-unblocking work with stable IDs as tie-breakers. Promote long-waiting low-priority work under a configured aging rule. Show the reason a task runs or waits. Do not let a frontier model decide which lease “looks free.”

Admit a writer only when current intent is active, control permits execution, dependencies and profile qualification hold, source/checkpoint is exact, all declared claims are available, an eligible runner slot exists, downstream capacity is not saturated, all monetary holds fit and an invocation allocation remains. Recheck these inside the transaction after selecting a queue entry.

```text
BEGIN IMMEDIATE
  check authenticated actor, task version and current policy
  check ready predicate, exact dependency receipts and profile qualification
  check all path/interface reservations or retain own existing reservation
  check owner/repo/team capacity and downstream watermark
  check task/root invocation allocations
  reserve exposure in every applicable money scope or none
  acquire complete claim set or none
  increment writer generation; create prepared job
  decrement allocation / record allocation identity once
  append audit event; persist unique launch intent
COMMIT
send only the recorded launch intent
```

All-or-none claims prevent holding half a desired expansion while waiting forever for the rest. An unauthorized scope expansion preserves sealed work and asks for a contract revision; it does not silently broaden the current reservation. Read access does not lock the whole repo. Named interfaces/migrations/lockfiles capture known semantic coupling; undeclared coupling is checked through integration and remains a residual risk.

### 13.1 Money once, limits in many scopes

Applicable scopes include organization/project, owner, mission, root outcome, task, operating window and experiment. They are constraints on the **same** leaf spend. A $1 job affects several limit views but remains $1 total expenditure. Reserve `spent + held + proposed <= limit` atomically for every applicable scope, with checked arithmetic. The signed integer upper bound is enforced on parse and arithmetic. Missing budget/currency/profile denies live admission.

Do not encode `spent + held <= limit` as a permanent database CHECK: a late actual overrun must still be recorded. Stop new admission and show the overrun rather than clipping it. Keep unresolved exposure through cancel, day-window rollover and restart. A window does not reset lifetime money or usage. An explicit transfer first reserves remaining exposure in the new window before releasing the old hold.

Parse whether usage reports are incremental, cumulative or inclusive of descendants. Deduplicate by provider report identity and digest. Cumulative updates apply only the new difference, with corrections handled as explicit reconciled observations rather than negative invented spend. Inclusive parent totals and separately visible child totals cannot both be charged. Preserve raw reports and the selected charge basis. Unknown final usage is not zero and does not automatically release its hold.

Planning, critique, repairs, reviews, verification compute and trials all consume the appropriate allowance. Reserve a protected portion of task budget for required acceptance/review before admitting expensive writing; otherwise the agent can spend the entire budget generating code that cannot be evaluated. This is monetary headroom, not a verifier slot held idle throughout writing.

A strict invoice cap requires an enforceable provider-side or metered request boundary. With delayed CLI telemetry, advertise a stop threshold plus possible in-flight exposure, not a precise hard cap. Authentication eligibility, quota availability and invoice spend are separate. Separate keys are not necessarily separate quota buckets.

### 13.2 Work-in-progress and local resources

Initial configurable defaults: four team writers, two per owner and two per repository; two verifier jobs; one integration lane per repo; six outstanding managed PRs per repo; at most two refined not-running tasks per useful worker slot. New implementation slows when verification exceeds two waves or outstanding PRs reach their cap. Required repair/verification/reconciliation retains capacity. These are pilot defaults, not claims of optimal hardware throughput.

Preflight CPU/RAM/disk and required GPU/device capability through runner profiles. Reserve exclusive scarce devices when needed. Long research tools get explicit job-specific deadlines, not a universal inactivity kill. Concurrency grows only when accepted throughput improves without quality regression. Do not make scheduler intelligence another model expense.

## 14. Runner, process and artifact lifecycle

The trusted supervisor stores a small local durable spool before launch: job/packet digest, boot/writer generation, sandbox ID, runner incarnation, intended command, last acknowledged event and sealed artifacts. It is not a replica task scheduler. Label the sandbox by that exact identity. On restart, inspect/reconcile or terminate that instance before accepting replay; PID alone may be reused and directory names are untrusted.

A duplicate launch with identical identity returns current observation. A duplicate ID with different packet bytes is a protocol incident. An unknown process state blocks a conflicting authoritative launch until fenced/reconciled; isolated stale physical processes never share a successor's directory. No global `pkill provider` or shared-daemon shutdown is used to cancel one task.

Generated code runs unprivileged in one approved disposable Linux container/VM profile with private mutable source, HOME, process namespace and temporary directories. No developer home, SSH agent, host container socket, hub database/token, publisher/reporter/signing/deployment credentials or other job mounts. Restrict egress, CPU, memory, PIDs and disk. Build scripts, proc macros, compiler plugins, install hooks and tests are executable code. A private Git clone alone is not a sandbox.

Prepare source from an exact base, then safe immutable image/dependency/object caches. Never share writable `.git` or an author-controlled build cache with the trusted verifier/control path. Retained alternates require a GC/retention guarantee. Submodules and LFS are explicitly supported by a tested profile or rejected before model spend, not silently omitted. Large repos use established safe snapshots/reflinks after measurement, not repeated blind full downloads or a new storage platform.

### 14.1 Sealing and intake

Freeze/stop writer mutations, form a bounded local Git bundle, upload to a supervisor-owned temporary location, stream-verify digest/length/quota, flush and atomically finalize, then record complete artifact metadata. A crash before completion leaves an orphan, not evidence. Incomplete or missing artifacts never pass admission.

An unprivileged collector imports into a clean quarantined repository under trusted Git configuration. Require expected object format/types/reachability/base ancestry; bound object count/size; reject unexpected refs, replace refs, grafts, alternates, unsafe paths/modes and unauthorized symlink/submodule changes. Do not run the author's Git hooks, credential helper, filter, diff driver or URL-supplied remote in the publisher identity. Use argument arrays, explicit trusted configuration and no source checkout/build in the hub.

Transfer only admitted immutable objects to the publisher's clean repository. Run source checkout/tests only in a separate verification sandbox. A hash establishes content integrity, not behavioral trust. Keep useful failed patches and the last completed checkpoint; do not promise recovery of uncheckpointed scratch lost in a crash.

### 14.2 Events and cancellation

Separate heartbeat and state ingress from high-volume logs. Batch text deltas into bounded artifacts. Sequence normalized events per job/runner, then assign a hub committed sequence. Same sequence plus same digest is replay; different payload is quarantine. Backpressure may truncate declared diagnostic text, never silently discard a terminal authority transition. Persist a final status independently of stdout.

Request graceful interruption, then terminate the exact owned tree at the hard deadline. Confirm through the supervisor, not a model message. Unreachable descendants leave termination unconfirmed. Local service liveness, provider activity and useful engineering progress are distinct observations. Heuristic stall warnings are advisory; lease, cancellation and resource deadlines are enforced.

## 15. Verification, draft publication and integration

### 15.1 Gate adequacy before scaling

For the first enabled task class, select meaningful recent defects or representative seeded faults. Keep the accepted regression test and trusted recipe while reintroducing the faulty behavior. Require a correct baseline pass and defective variant failure for the intended reason. Blind whole-PR reverts can remove tests too. An unrelated broken build or documentation-only revert is not useful behavioral detection. A new feature's no-op candidate must not satisfy its new requirement.

Record cases, exact good/bad subjects, fixtures, environment, expected failure, observed result and coverage limitations. Approximately ten cases are a diagnostic, not statistical certification. A missed material behavior blocks unattended acceptance for the affected class until coverage or named human evaluation is strengthened; it need not block unrelated safe classes or explicitly incomplete draft assistance.

### 15.2 Trusted method, untrusted code

A protected Recipe defines check ID, executable/arguments, relative working directory, permitted environment, fixture/input construction, time/resource limits, expected discovery/completeness, result parser/version, artifact requirements, risk class and location `local|ci|integration`. Mount protected evaluation inputs read-only as needed; source-authored tests are supplementary until independently adopted. New fixtures are permitted but do not let a learner inspect holdouts or redefine pass conditions after seeing failures.

The trusted reporter obtains actual execution observations from the independent supervisor and sends them with its separate credential. A candidate process printing a JSON PASS is not that reporter. A clean environment alone is not an adequate oracle: code can affect its test process, so sensitive behaviors may need out-of-process black-box checks, independent fixtures and human review. Do not claim finite tests are an uncheatable proof system.

Evidence binds the exact task revision/digest, candidate or research artifact, relevant base/integration composition, recipe/policy/fixture/environment digests, assigned verification job and authenticated producer, observation time, discovered/failed/skipped required cases and complete logs/artifacts. Store PASS/FAIL/ERROR/MISSING/SKIPPED/UNKNOWN distinctly. A PASS requires the recipe's explicit completeness/discovery rules. Zero tests, missing artifacts and skipped required cases are not success. Known flakes have a bounded protected policy and replacement coverage; retain first failure and every rerun rather than sampling until green.

### 15.3 Publication solves the remote-CI bootstrap honestly

A final selected candidate can be published as a **provisional draft** after safe intake, scope/policy checks and the repository's configured local pre-publication minimum. This may be necessary to trigger CI. Remote-only obligations remain pending. Checkpoints within a task are not published as separate drafts unless an explicit future mode is implemented.

The provisional-draft grant is narrow: correct owner/repo/head namespace, no ready status, no merge request and no release authority. A repository may disallow even provisional publication until stronger local checks; honor that configuration. Do not claim “nothing reaches origin until every check passes” while a required check can run only on origin.

After all required **task** checks, including any CI-only task checks, and designated independent task review pass for the current head, mark **review_ready**. Repository-required human/Code Owner approval can still be pending for integration; it is explicitly shown. **merge_eligible** additionally requires current protected integration evidence and all human/forge rules. A draft exists before either label. This vocabulary resolves different source uses of `pr_ready` without silently weakening acceptance.

| State | Establishes | Does not establish |
|---|---|---|
| checkpoint_verified | Current cumulative step checks passed | Entire task, PR or merge is accepted |
| draft_open | Exact remote draft observed after authorized local minimum | All task/CI/review obligations passed |
| review_ready | All required task-level acceptance and independent task review complete on current head | Required final human review or current merge-group checks complete |
| merge_eligible | Required human/forge rules and current integration checks satisfied | Merge actually occurred |
| merged | Actual protected integration/content relationship observed | Deployment or observation window complete |
| mission accepted | Declared root outcomes achieved at the named boundary | Broader unrequested quality/performance guarantees |

### 15.4 Independent review and trusted aggregate

Use a fresh review session for ordinary behavioral changes; policy may allow lighter deterministic handling for calibrated low-risk mechanical changes. High-risk actual surfaces require named human/domain approval and stronger checks. Effective risk is the maximum of mission intent, actual changed surfaces and protected minimum; the writer cannot down-classify its own work. Reviewers receive requirements, exact diff, related code and evidence, not just the author's persuasive narrative.

Publish a trusted aggregate `bulletfarm/acceptance` for each relevant subject, with the designated issuer. It succeeds only when the required non-skipped obligations actually hold. GitHub documentation allows some skipped/neutral conclusions in generic required-status semantics, so merely observing generic mergeability or a familiar check name is insufficient for this stricter contract. Preserve expected-source enforcement. [W5]

### 15.5 Current integration and mission outcome

Use the existing certified protected merge queue. GitHub's queue validates the updated target plus queued composition and requires `merge_group` workflow triggering; these are not the original PR-head checks. [W10] Confirm equivalent behavior on the owned forge: actual check issuers, protection/permissions, expected head, tested composition, merge results and timeout recovery. API-name parity is not this qualification.

Where no proven protected current-subject path exists, stop at review-ready and the existing protected human workflow. Do not implement a client-side “test then push sometime later” fallback and call it atomic. A human route still uses the existing protection; manual does not mean unchecked.

Repairs use forward commits, not force-pushing away human/history. Squash/rebase can change commit identity; record original candidate, tested integration tree, actual merge OID and established content relation. Failure to establish the relation is unknown plus post-merge verification/owner attention, not an assertion that the original SHA merged. Two individually green changes can fail together, and integration failure may be non-monotonic; no assumption that bisection finds a single guilty PR.

Mission acceptance covers assembled root behavior, not counts of step checks. `review_ready` missions explicitly report remaining integration obligations and may require intermediate human merges for separate dependent tasks. `merged` missions require observed code integration and final mission checks. Research uses accepted artifacts. Reject production-observed goals until the release adapter is qualified. Do not accept an unsupported target and leave it mysteriously running forever.

## 16. External effects and honest recovery

Every consequential external mutation has a persisted immutable effect record: metadata push, candidate push, PR create/update, acceptance check publication, merge request, notification or release request. Its key identifies the logical operation, serialization scope, payload digest, subject and initial authorization. The current handler still reauthorizes before issuance.

```text
pending -> sending -> confirmed
                  -> unknown -> confirmed | not_applied | blocked
pending -> cancelled          (revoked before issuance)
```

One unresolved effect occupies a serialization key at a time. The fixed controller stages dependent operations: observe branch push before PR creation; observe PR before updating its state. Independent tasks/refs can progress. New conflicting effects never leapfrog an unknown outcome.

Persist `sending` and the checked dispatch authorization before network issuance. A crash here is treated as possibly sent, even if the call never left the process. Reconcile by exact remote repository, branch, head, task/revision marker and known operation IDs. A lost create response should adopt only the exact existing PR. Same marker with a different head, or a human-closed PR, is not permission to reopen/overwrite it. An eventually consistent empty result does not prove absence while a request may remain in flight. Retry only documented remote idempotency or authoritative `not_applied` evidence; otherwise retain one owned unknown incident.

Webhook authenticity and delivery ID are verified and durably stored before acknowledgement. Duplicates and reordered events trigger current-state reconciliation; never apply a stale “open” event to overwrite an observed merge. Periodic bounded polling repairs missed notifications. Webhook payload identity alone is not a permanent current-fact oracle.

### 16.1 Boot and disaster restore

Acquire instance lock, verify runtime/schema/artifact custody, enter paused reconciliation, generate new boot authority and revoke old active execution. Runners on prior authority stop; they may upload diagnostic salvage but cannot submit new current candidates. Revalidate selected sealed candidates and unsent effects under current policy. Effects previously sending/unknown remain unknown until reconciled. The old author need not be relaunched for delivery.

For a restored/cloned installation, additionally ensure the old publisher is stopped or its credentials revoked, invalidate credentials/registrations whose revocations may have been rolled back, verify all required artifacts and reconcile forge changes newer than the backup. Remain paused if these facts cannot be established. A random new epoch is not a remote revoke API.

Use supported consistent database backup with an artifact manifest and required Git intent objects. A copy of only the live main SQLite file is not sufficient. Test actual disk durability separately from in-memory SQL. Migrations are explicit, backed up and checked on fixtures; no unbounded destructive migration merely because a process started.

### 16.2 Retention and operational incidents

Cleanup requires exact job/sandbox/artifact identity, terminal or authorized withdrawn state, no active reader/reference or unsettled obligation and elapsed retention. Age alone is not permission. Disk pressure stops new admission and reports an incident; it does not delete active work. Preserve meaningful failures and root history when bulky workspaces expire. A historical receipt can say raw logs were pruned; a fresh acceptance cannot depend on missing required artifacts.

A provider outage or expired shared credential becomes one incident with affected tasks and an owner, not one duplicate approval per run. Diagnostics must remain useful without any available model. No automatic `doctor --fix` weakens policy or deletes work; remediation is a separate scoped command.

## 17. Human takeover, adoption and escalation drain

`bf task take` checks current actor/task version, revokes the old writer and invalidates delivery, persists a stop/checkpoint request and preserves the pending-change reservation. The human receives a new clone from an exact known checkpoint. Never transfer a directory still writable by a stale process. When a runner is unreachable, offer the last actually stored checkpoint or an explicitly selected clean start and identify the possibly lost interval.

A human assignment is not a model process. It does not need sixty-second “thinking” heartbeats. Its named reservation remains until acknowledged withdrawal, cancellation or owner reassignment. Reconnection/submission revalidates current assignment, scope, task revision and policy. The system does not kill the person's editor, reset their clone or claim surveillance over their whole machine.

`bf task submit` seals human output through the same constrained intake, independent verification and protected delivery path. Record human/tool metadata only as known, and human active time as measured/reported/estimated/unknown. Missing time is not zero. Existing organizational permissions may differ between people and bots, but neither gets an invisible acceptance bypass.

Every material exception has primary and fallback owners, creation/last-observed time, affected revision/subject, evidence and bounded options. Drain by clarify/respecify, decompose, take over, withdraw, cancel or explicitly extend remaining allowance. Edits and child task IDs preserve root history/cost and cannot erase a failed attempt. Close the decision only after the exact version-bound action applies; generic “yes” cannot approve a changed action.

Ad-hoc branches are normal. `bf task adopt` creates a retrospective task/provenance record before managed acceptance, checks current overlaps and never invents earlier supervision timestamps. Preserve the person's clone and existing PR. Urgent work can use the ordinary protected human flow without waiting for a reporting window. Do not represent an unknown old branch as prequalified merely because the human trusts it.

## 18. Scoped memory, learning and empirical routing

Three classes suffice: disposable task-local working state; approved owner/domain knowledge and skills; protected control material. The first two can inform execution but do not grant authority. The third is owned by administrators/gate owners and is not an ordinary learning target.

A summary inherits its source ACL and data classification. Current audience checks occur before retrieval, caching and delivery. Include source revision, scope/policy version and profile memory snapshot in relevant cache identity. Removing a name is not a privacy proof. Explicit owner review can promote a sanitized artifact; no ambient merge of everyone's private conversations occurs. Already transmitted data cannot be retroactively unsent by changing the local ACL; record affected jobs and prevent further disclosure.

The whole execution profile is the experiment unit. Freeze task/base/gates/environment/tools/context/initial memory and resource ceilings. Record which factor differs. Compare warm with warm or declare warmth as the treatment. Never attribute a complete system comparison solely to model weights.

From the first job retain task/root/class, profile, inputs, observed outcomes, failures, costs, intervention and eventual integration/defect links. An optional shadow execution has its own experiment budget and namespace and no production selection or publication grant. The incumbent ships independently. A real defect discovered in shadow can enter ordinary trusted triage; “shadow does not block” is not a rule to ignore evidence.

Promote an enduring lesson through a small versioned proposal, source/privacy review, fixed acceptance and held-out comparison, designated owner decision, canary and reversible rollback. Default experimental spending is zero until authorized. Model-weight training and automatic bandit routing are later experiments, not core dependencies. A learner cannot edit its own tests, exclude inconvenient cases or publish a permissive profile as already approved.

Report eligibility, selection bias, failures, cancellations, missing observations and uncertainty. A shadow not deployed has no observed production-survival outcome. Human baselines often select harder tasks; stratification does not magically eliminate every confounder. Twenty or thirty observations is not a universal statistical threshold. An inconclusive trial is a valid reason to retain the incumbent and stop spending.

A named service owner periodically evaluates whether simpler machinery preserves the durable assets at lower total cost/attention. The exit rehearsal exports work and continues through an alternative runner/human without native conversation files. This document assigns an operating responsibility; it does not create calendar events or background automations.

## 19. CLI, API, events and browser contract

Appendix A supplies the exact command payload grammar and MissionRequest intake fields before a task graph exists. These are part of the build contract, not additional optional design choices.

```text
bf init                         initialize with live dispatch off
bf hub start                    start one authority/API/static web
bf runner enroll / start        connect an explicitly authorized runner
bf doctor                       read-only actual capability/health report
bf plan "outcome"                draft only, within planning allowance
bf run "outcome"                 authorized plan + execution
bf chat MISSION                 discuss/steer through typed proposals
bf status --mine / --team        deterministic state, no model required
bf why TASK / bf evidence TASK   exact blocker, subject and next action
bf pause / resume TARGET        hold/revalidate new jobs and effects
bf stop / cancel TARGET         revoke/stop; cancel terminalizes intent
bf task take / submit TASK      safe human handoff and candidate intake
bf task adopt BRANCH             retrospective registration
bf task revise TASK              immutable revision, preserved allowances
bf task withdraw TASK            reconcile and safely release pending work
bf decide DECISION --option ID  exact current subject-bound decision
bf export PROJECT               non-live portable history
bf demo --offline               real controller/store with fake adapters
```

Commands above are the build target, not an installed executable in the ZIP. `bf plan` cannot silently execute code. `bf run` may automatically activate in-policy plans without another menu, but no action widens its envelope through conversation inference.

### 19.1 Minimal routes

| Route | Contract |
|---|---|
| `POST /v1/commands` | Strict enumerated action, command ID, target/repo, expected version, typed payload |
| `GET /v1/operations/{id}` | Durable accepted/pending/confirmed/blocked/unknown result |
| `GET /v1/work` | Authorized paginated Mine/Team projection |
| `GET /v1/missions/{id}` | Outcome, active plan, task dependencies, decisions, costs and boundary |
| `GET /v1/tasks/{id}` | Contract, next step, selected candidate, claims, jobs and evidence |
| `GET /v1/tasks/{id}/why` | Deterministic reason and next action |
| `GET /v1/events?after=N` | Authorized SSE replay or explicit resnapshot requirement |
| `POST /v1/runners/enroll` | Single-use expiring code exchanged for scoped revocable identity |
| `POST /v1/runners/{id}/poll` | Capacity/cursor; returns already-persisted launch/stop assignments |
| `POST /v1/jobs/{id}/heartbeat` | Current assigned runner, boot authority and generation only |
| `POST /v1/jobs/{id}/events` | Bounded sequenced observations, not state-setting commands |
| `POST /v1/jobs/{id}/result` | Weak ProposedResult; trusted admission still required |
| `POST /v1/verifications/{id}/result` | Assigned trusted producer, exact subject and completeness checked |
| `POST /v1/artifacts` | Quota/size/digest-checked upload, no client-selected server path |
| `GET /v1/artifacts/{digest}` | Current ACL and completeness checked |
| `POST /v1/hooks/forge` | Verified durable delivery inbox |
| `GET /v1/receipts/{id}` | Immutable known-state snapshot and successor references |

Authentication precedes disclosure, including replay results. The request body cannot supply an authoritative actor. Unique `(principal, command_id)` plus exact-body digest implements idempotency; the expected version is checked for new mutations, while identical already-admitted retries return their original operation after current read authorization. Different content under one key is 409. Pending dependency status is not an HTTP transport error.

Use 400 malformed, 401 unauthenticated, 403 denied, nondisclosing 404 where appropriate, 409 stale/idempotency/selection conflict, 422 valid-shaped but invalid contract/unsupported capability, 429 pressure and 503 unavailable durable storage. An accepted 202 means durably recorded, not that a PR exists. Include `{code,message,correlation_id,operation_id,current_version,next_action}` with explicit nulls where unavailable and no secrets.

Core actions: mission create/message; plan propose/activate; task revise/take/submit/adopt/withdraw; pause/resume/stop/cancel; decision resolve; budget/allowance extension by a designated approver; receipt/export. Each has a finite handler-specific payload validator. There is no generic “execute shell” or arbitrary method dispatcher. Agent proposals call only allowed subset handlers; authority-changing approval is a human/admin control.

SSE uses committed monotonic sequence and current repository filters. A cursor older than retention gets `RESYNC_REQUIRED` plus a coherent snapshot/cursor. Permission-filtered gaps are normal. A disconnected UI cannot infer a lost mutation succeeded or resubmit it with a different ID. Bound logs, escape HTML/Markdown/terminal controls and never let a slow client stall durable state transitions.

### 19.2 One operational page

Mine and Team are filters on the same data, not two trackers. Show mission outcome and delivery boundary; owner/domain; phase and current step; actual next blocker and decision owner; selected candidate/PR; local/CI/integration evidence; actual/held/unknown cost; provider health and observation freshness. The detail drawer holds source decisions, full contract, exact diff, check findings, attempt history and raw logs on demand.

Use keyboard navigation, visible focus, readable contrast and text labels rather than color-only status. Render count/status cards deterministically; a model can explain them but cannot write a green status. No invented percent complete based on token use or planner-created task count. Group infrastructure incidents and show useful next actions rather than a wall of thinking agents.

## 20. Reference database and transaction contracts

`migrations/001_initial.sql` implements a compact relational starting point. It is not a production engine or proof of the application guards. Explicit relationships include principals/memberships/repos; missions/roots/tasks and immutable task revisions; jobs/profiles; claims; artifacts/candidates/evidence; budgets/holds/usage; commands/events/effects; observations/decisions and receipts. Tables are not services.

Use a partial unique index for one authoritative writer per task, nonnegative counters/limits and monotonic writer/delivery/version triggers. Immutable accepted contract/profile/candidate/evidence rows cannot be rewritten. Foreign keys establish references; they do not by themselves validate same-repository policy or exact intended subject. Scope overlap and multi-scope budget admission occur inside the single serialized transaction path.

| Transaction | Atomic requirements |
|---|---|
| Accept command | Current authorization, immutable request receipt, operation, state/event/outbox as appropriate |
| Activate plan | Confirmed Git/artifact identity, complete graph and requirement mapping, active revision, event |
| Prepare job | Current ready predicate, complete claims/holds, allowance consumption, generation and unique launch intent |
| Admit sealed result | Artifact and object validation, current author/revision, immutable candidate, selected delivery version, event |
| Admit evidence | Trusted assigned issuer/job, exact subject, complete artifacts, immutable history, eligible cursor/state transition |
| Advance step | Current cursor/candidate/version and cumulative check coverage; one advancement only |
| Revoke/withdraw | New generations/control state, invalidated grants, stop/effect intents, retained claims until settled |
| Settle usage | Report dedupe, raw observed bill, one leaf delta and updates to all affected holds/scopes |
| Observe remote result | Exact remote identities, settled effect, current projection and audit |

External operations never occur inside a SQL transaction. The outbox establishes durable intent, not distributed exactly-once effects. Retaining an unknown state is an intentional result when the remote system cannot resolve the ambiguity.

See the appendices for the wire contracts, generated backlog and scenarios. A schema enum mismatch, unimplemented case ID or query returning zero expected tests is a build failure—not a reason to weaken validation.

## 21. Defaults, performance and operations

| Setting | Initial default / rule |
|---|---|
| Live execution, money, auto-merge, experiments | Disabled / zero until specifically authorized |
| Reference deployment | Linux execution, one local-disk SQLite hub, one Rust package |
| Steps | One default; maximum eight linear steps per code task |
| Writer allocations | N initial step allocations plus two shared repairs; max three per step |
| Planning | Three substantive jobs; extra issue requires allowance |
| Team/owner/repo writers | 4 / 2 / 2, tunable downward |
| Verifiers / integration lanes | 2 / one per repository |
| Open managed PR cap | Six per repository |
| Heartbeat / lease / safety margin | 10 / 60 / 15 seconds |
| Initial job wall time | 3,600 seconds unless qualified long-job profile |
| Tool timeout | 600 seconds unless protected recipe/profile overrides |
| Command/frame | 1 MiB maximum; artifacts separate |
| Event batch | 100 events and 4 MiB maximum |
| Page size | 100 default, 500 maximum |
| Long poll | 30 seconds maximum |
| Raw diagnostic retention | 14 days by proposed policy; no deletion of active obligations |
| Failed workspace retention | 48 hours after durable salvage and safe terminal state |
| Provider self-update | Off in managed jobs; pin and canary upgrades |

These are starting configurations, not evidence of optimal throughput or compliance requirements. The operator supplies actual gates, devices, source size, budget, account eligibility, allowed data location, retention and rollback policy.

For a disclosed 4-vCPU/8-GiB Linux reference host with local SSD, 10,000 task records, 100 status clients and 50 compact state events/second, propose p95 authorized status under 100 ms; p95 durable command acknowledgement under 250 ms; observed-event to persisted dispatch under 250 ms; idle hub RSS under 200 MiB; compressed initial web assets under 600 KiB. Report host/runtime and data distribution. These are **unmeasured targets**. Include authorization and fsync in local timings, and report model/build/forge latency separately. Never omit correctness controls to satisfy a timing slogan.

`bf doctor` is read-only and reports actual linked SQLite/Git/provider versions, profile qualification, source/LFS/submodule readiness, credentials by reference, gate/issuer/protection capability, disk/clock/process health, unknown effects, missing evidence and reconciliation freshness. Remediation is narrow and authorized. New provider versions run recorded conformance plus a live canary before promotion; model aliases with hidden changes remain a declared measurement limitation.

## 22. Production, channels and the owned forge

The core ends at protected engineering delivery. A later single-target adapter reuses the existing release pipeline:

```text
protected merge -> immutable artifact -> staging checks
 -> exact artifact/environment authorization -> deployment receipt
 -> actual running version and health over the required interval
```

Approval binds artifact digest, environment, release configuration, action, actor and expiry. A rebuild or substitution changes the subject. A successful trigger is not deployment; an initial healthy probe is not the observation window. Workers never hold signing/deployment keys. Reversible rollback must be rehearsed; a previous binary does not undo irreversible migration/data deletion. Record compatibility/forward recovery explicitly.

Slack is the first extra channel. Validate its signed transport and identity mapping, deduplicate, enforce current repository/subject authority and use the same commands. SMS initially carries opt-in summaries and authenticated links, not sensitive code or a generic “yes” sufficient for irreversible actions. Grok Bot requires an actual supported client/tool capability trial; Grok Build's coding protocol does not establish a Bot messaging API. No channel contains a second scheduler or independent permission memory.

Owning the forge is useful where writes become real: current-generation/expected-head checks, trusted acceptance provenance, registered intent visibility and actual merge-subject enforcement. Start with existing interfaces. Add one measured native improvement, not custom Git storage or a duplicate forge UI. Preserve ordinary Git/protected human workflows with the extension disabled.

## 23. Build order and the independent fresh-agent test

A fresh agent receives this package, an empty or authorized existing repository, supported compiler/toolchain and explicit environment bindings. It does not need the old conversation, Alton's private memory, a missing earlier “final” or copied provider sessions. Missing authorization and credentials remain missing; fixture values never become live grants.

For an existing source tree, produce an obligation-to-code/evidence/gap mapping before changes. For an empty tree, create one Cargo package and Vite directory, lock versions, implement contracts/store/fake flows and then the first gated live path. Do not create all empty modules or twenty parallel agents before one accepted command works.

The canonical `BACKLOG.json` uses BF3 identifiers, preserving source IDs only in traceability. Every package has exact sections, prerequisites, write scopes, non-goals, acceptance IDs and test bindings. Full application tests must assert behavior and fail when an expected test selector matches nothing. A future `cargo test` command shown here is not a command claimed to pass in this delivered spec-only kit.

Expected product commands after implementation:

```text
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
npm --prefix web ci
npm --prefix web run typecheck
npm --prefix web test -- --run
npm --prefix web run build
bf demo --offline --scenario first-win
bf doctor --repo <actual-approved-repository>
```

`bf demo` uses the real controller/store paths with deterministic fake provider, fake forge, tiny fixture repo and fake clock. It requires no network or secrets, and must never send its simulated PASS into a live issuer registry. Demonstrate failure and recovery, not a happy-path animation.

Each build handoff records package, changed files, exact code revision, actual commands/results, current failing case, missing evidence and next eligible package. A reviewer runs the frozen acceptance suite. A second fresh agent must continue from only the repo and packet. The independent clean rebuild later repeats G1 and G2, records every ambiguity as an erratum, and compares observable behavior rather than source-style similarity. This exercise has not been performed by writing the specification.

## 24. Acceptance, preservation and final score

All application cases in `qualification/CASES.json` begin `not_run`. Offline package tests validate schema examples, referential/graph integrity, score arithmetic, selected SQL constraints and a small semantic reference model; they do not compile Rust, call providers, perform real attacks or certify a forge. The validation report states exactly which checks ran.

Hard release failures include unauthorized spending/read/mutation; a stale writer changing current selection; a forged/wrong-subject check satisfying acceptance; source execution reaching privileged credentials; a blind duplicate effect after uncertainty; a successful code change bypassing required integration; an imported live grant; acknowledged state silently lost; or an untrue completed status. These disable the affected capability regardless of the design score.

G1 proves a real one-step task, no routine native menu, actual evidence, author exit before delivery and lost-response/restart handling. G2 adds two owners/runners/coding families, independent and conflicting work, two-step execution with no intermediate human merge, suffix invalidation, human takeover, exact current integration, restore/export and operator comprehension. Include at least two repositories in access-isolation cases even when the first live pilot uses one.

A representative 30–50 parent-outcome pilot is a practical diagnostic, not a universal sample-size proof. Freeze workload eligibility, acceptance boundary and comparable baseline. Retain failures, safe blocks, unsupported cases, cost exclusions and user minutes. Report whether the product reduces handling or whole accepted-outcome cost without relaxed quality; do not promise an arbitrary 25% saving or 90% autonomy before measurement.

The final common-rubric score is **94.38/100 (94.4 rounded)**. The category table is in Part I and `review/SCORES.json`. High scores describe the written contract; they do not imply 94.38% reliability. The main deductions remain provider/OS qualification, gate adequacy, hidden semantic conflicts, real economics/usability and the unperformed independent rebuild.

All ten current inputs and nine earlier source artifacts are retained with hashes. `review/PRESERVATION.json` maps the material requirements to this contract and BF3 cases. Preservation means retaining the user need and traceable decisions—not executing contradictory archived implementation instructions. Neither a larger document nor another layer of agents raises the score by itself.

**Final directive:** ship the smallest dependable owned work loop, with linear steps where they genuinely help. Preserve people, intent, useful code and evidence across provider and process changes. Increase concurrency and sophistication only when the accepted-outcome data shows they help.


---

# Appendix A — Exact command payload and intake contract

This appendix closes the finite command grammar. It is normative, not illustrative. `Command` has one authenticated caller, one `command_id`, repository, target/version and eight named payload slots. Every payload slot is present; unused slots MUST be null. No caller-supplied role, actor, shell command, server path or arbitrary endpoint is accepted. Action-specific nullability and target/version rules are encoded in the JSON schema; cross-record authorization remains a controller transaction.

## A1. Mission intake before a task graph exists

The CLI/web first constructs a `MissionRequest` from explicit user inputs plus visible, already-authorized project defaults. `bf plan` selects `mode=plan`; `bf run` selects `mode=run`. The user sees the resolved owner/domain, outcome, delivery boundary, planning limit and total budget. Unconfigured limits are zero/disabled, not guessed from an example. A request to spend inside an existing allocation does not grant the caller permission to create that allocation.

`MissionRequest` names the repository, requested human owner/domain, objective, mode, supported delivery boundary, intermediate-merge policy, protected policy/gate profile, planning execution profile, parent budget scopes, mission/planning monetary ceilings, maximum planning jobs, initial requirements and non-goals. Initial requirements may be empty while a plan is being drafted; an activated Plan may not lack mandatory outcome coverage. Policy and gate identities are protected registry references, not caller-defined permissions.

Admission verifies the authenticated caller may act for the requested owner, all referenced scopes belong to the same authorized accounting/data domain, the planning ceiling does not exceed the mission ceiling, and the requested profiles/boundary are permitted. Create the mission and child budget rows atomically with the command receipt. Generate the actual mission ID at this point. Subsequent model jobs reference it even before a task exists. The first paid planning job separately reserves against all relevant limits. A rejected command consumes no job allowance; a transmitted model request retains its real/unknown usage even when planning fails.

Upload the request bytes as an authenticated immutable artifact. `mission.create` references that digest. The returned operation supplies the new mission ID and version. Creating a mission in plan mode cannot later enable implementation because a model put `execution_requested=true` into a proposed Plan. An explicit authorized run/adoption decision must enable it within current policy; otherwise store the plan as draft only. The initial implementation may use a new run-mode mission referring to the approved draft rather than introduce an undocumented mode-toggle command.

Plan proposal and activation preserve the mission's original requirements and scope. Material additions require a recorded authorized amendment; omissions cannot silently improve the apparent completion percentage. The activated Plan budget and summed root allocations fit the mission's current ceilings. Planned writer allocations include each first pass plus the shared finite repair pool. The planner cannot mint budget by creating more root IDs.

## A2. Payload matrix

All listed fields MUST be non-null unless described as optional. Other payload fields MUST be null. “Existing” means `target_id` and `expected_version` are non-null and refer to the named current object. “Create” means both are null. A same-key replay uses the exact original body bytes.

| Action | Target | Required payload | Meaning and guard |
|---|---|---|---|
| `mission.create` | Create | `artifact_digest` | Digest of a strict MissionRequest; caller may create for owner/domain under existing policy and parent budgets. |
| `mission.message` | Existing mission | `text` | Record scoped conversation. Any paid response uses existing planning/interaction allowance. Text itself cannot grant authority. |
| `plan.propose` | Existing mission | `artifact_digest` | Strict Plan artifact; record a proposal only. Read-only/model proposal authority never activates work. |
| `plan.activate` | Existing mission | `artifact_digest` | Same admitted Plan bytes, confirmed Git intent operation, current authorized adoption. A raw digest alone cannot assert Git publication. |
| `task.revise` | Existing task | `artifact_digest` | New strict Task revision. No current writer; conflicting effects settled; selected delivery invalidated; root lineage/limits preserved. |
| `task.take` | Existing task | `option_id`; optional `checkpoint_id` | Option is `last_confirmed`, `clean_start` or `checkpoint`. The last requires a known checkpoint ID; other options require it null. Revoke old writer/delivery and retain change ownership. |
| `task.submit` | Existing human-assigned task | `artifact_digest`, `head_oid` | Artifact is a quarantined candidate bundle; named head must match its inspected contents. Current assignment, revision, scope and intake rules apply. |
| `task.adopt` | Create | `artifact_digest`, `head_oid` | Artifact is a strict proposed Task. Head must already be available through authorized registered-repository intake or a previously uploaded quarantined bundle. Adopt creates current provenance, not retroactive supervision. |
| `task.withdraw` | Existing task | `text` | Reason for withdrawing pending work. Revoke delivery, reconcile remote effects/proposal state, then release claims when the old change can no longer integrate through managed authority. |
| `target.pause` | Existing mission or task | None | Hold new jobs and unsent effects. Existing isolated jobs/observations may finish; no new verification job starts while held. |
| `target.resume` | Existing mission or task | None | Recheck current membership, policy, limits, checkpoint and effects; never resurrect an expired writer generation. |
| `target.stop` | Existing mission or task | None | Pause plus revoke current worker/delivery authority and request managed-process termination; resumable intent remains. |
| `target.cancel` | Existing mission or task | `text` | Cancelled intent is terminal. Retain unsettled physical/remote observations, claims needed for reconciliation, and historical costs. |
| `decision.resolve` | Existing decision | `option_id`, `subject_digest` | Exact current decision subject, unexpired designated approver, permitted option. Models cannot self-approve protected human decisions. |
| `budget.extend` | Existing budget scope | `amount_microusd`, `text` | New absolute ceiling, not a delta. Must exceed current ceiling, fit parent constraints and caller authority; record reason and version. No implicit currency conversion. |
| `allowance.extend` | Existing root allowance | `allocation_count`, `text` | New absolute invocation ceiling, not a delta. Explicit authorized extension; consumed counts and all descendant history remain. |
| `receipt.export` | Existing mission | None | Permission-filtered snapshot at expected mission version; no live credentials/grants. Project-wide CLI export enumerates authorized mission exports plus a root manifest. |

Task adoption rejects an unknown head instead of asking a privileged publisher to fetch a worker-supplied URL. An already registered external PR is observed first, then explicitly associated after exact head/owner checks. The adopted Task must belong to an existing compatible mission or be created through a separately accepted MissionRequest first. No hidden create-mission privilege is implied by adoption.

`target_id` resolves to a typed object in the specified repository. A task's parent pause/cancel affects its effective admission. A child cannot resume through a still-paused parent. A cancellation remains terminal; continuing the objective uses a new explicitly linked task or mission. A paused object records which transitions are held and why so the lead can explain it without interpreting free text.

## A3. Response, version and error semantics

A successfully persisted command returns `operation_id`, `target_id`, `accepted_version`, `event_sequence` and operation state. For pending remote work use HTTP 202. The result says nothing about actual remote completion until the corresponding observation is confirmed. A request rejected before durable admission returns a typed error and no effect is dispatched.

Authentication and repository authorization run before idempotency-receipt disclosure. The key namespace is the authenticated principal plus installation and command ID. The digest covers exact request bytes including action/repository/target. Same key and different bytes returns `409 IDEMPOTENCY_CONFLICT`; the original operation remains unchanged. After authorization, an exact replay returns the original operation even when its target has subsequently changed, rather than attempting the mutation again.

Use 400 for invalid syntax/UTF-8/duplicate keys; 401 for missing authentication; 403 or privacy-preserving 404 for denied access; 409 for stale version, selection, generation or key conflict; 422 for a well-shaped but invalid task/profile/goal; 429 for request pressure; and 503 when durable storage/control service is unavailable. A normal dependency wait is a task state, not a failed HTTP transport. Errors carry stable code, correlation ID, authorized current-version information and a next action. They never contain secrets or unauthorized object-existence details.

Operation status is one of `accepted`, `pending`, `confirmed`, `blocked`, `failed`, `unknown`, or `cancelled`. These are not identical to task phases. An accepted cancel operation can coexist with an unknown previously sent remote effect; present both. An old event cannot downgrade an observed merged PR into open state. Reconciliation uses authoritative current remote observations, not last-arriving payload order.

## A4. Uploads, heartbeats and event batches

`POST /v1/artifacts` accepts an authenticated, bounded binary stream and declared digest/length. The server assigns its temporary path, streams the hash, enforces the repository/job quota, safely finalizes the bytes, and returns an artifact ID/digest and completeness state. Finalization does not mean the bytes are a valid candidate. Candidate bundles subsequently pass the independent intake checks. A digest is not an access token. Retrieval checks current actor/repository/audience even for deduplicated storage.

Runner poll names its enrolled incarnation, last acknowledged command cursor and measured available slots. It returns persisted assignments only. Heartbeat names job, boot authority, writer generation when applicable and observed process state; only the assigned runner can renew. Local deadline uses the renewal request's monotonic start plus granted TTL minus safety margin, never delayed receipt time. A response arriving after the local stop deadline does not revive work.

Each event names job, incarnation, local sequence, kind, observed timestamp and bounded payload. The supervisor adds/authenticates identity; model JSON cannot supply it. Duplicate sequence with identical digest is acknowledged; changed content under the same sequence is a protocol violation. Terminal results, cancellation and evidence do not share a lossy progress buffer. Required state is durable; optional textual progress may be truncated with an explicit marker. Event filtering and reconnect snapshots use the same ACL and produce no implicit command retries.

The complete provider event enum is `session_started`, `progress`, `tool_started`, `tool_finished`, `permission_requested`, `input_requested`, `usage_observed`, `result_proposed`, `run_failed`, `run_ended`. A missing final result is not inferred from text. Unknown informational fields may remain diagnostic; unknown authority-affecting requests are denied/quarantined. Raw provider timestamps are observations, not the hub's ordering authority.

## A5. Bootstrap and real environment bindings

Bootstrap locally with live dispatch disabled. An owner/admin uses the OS-protected local configuration path to establish the installation, repository registry, protected policy, token custody and runner enrollment. Do not expose a public unauthenticated bootstrap endpoint. A team installation uses its existing authenticated TLS ingress or a documented certificate arrangement. An enrollment code is expiring, scoped, one-use, hashed at rest and exchanged outside candidate execution. The first runtime version resolves concrete dependency/provider versions and writes lockfiles plus capability receipts; this kit invents none of those values.

A model may discover read-only facts when authorized. It may not fabricate an owner identity, money allocation, provider entitlement, verifier issuer or production destination merely because a schema requires a value. `examples/ENVIRONMENT_BINDINGS.json` intentionally leaves those values unresolved. The fake demo needs none of them and cannot select a live provider or publisher by falling back from an unknown fixture.



---

# Appendix B — Canonical wire types

Generated from `schemas/contracts.schema.json`, Draft 2020-12, schema version 3. The JSON schema is the machine representation. Exact-byte digests are external to the document they identify. Shape validation does not grant authority; cross-record and policy checks in the main specification are mandatory.

## Scope

| Field | Required | Constraint |
|---|---|---|
| `path` | yes | `{"type":"string","minLength":1,"maxLength":1024,"description":"Literal repo-relative canonical slash-separated path; semantic traversal, segment and symlink checks required."}` |
| `kind` | yes | `{"enum":["file","directory"]}` |

## Dependency

| Field | Required | Constraint |
|---|---|---|
| `task_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `revision` | yes | `{"type":"integer","minimum":1,"maximum":9007199254740991}` |
| `goal` | yes | `{"enum":["merged","accepted_artifact"]}` |

## Acceptance

| Field | Required | Constraint |
|---|---|---|
| `id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `requirement` | yes | `{"type":"string","minLength":1,"maxLength":65536}` |
| `check_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |

## Step

| Field | Required | Constraint |
|---|---|---|
| `id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `outcome` | yes | `{"type":"string","minLength":1,"maxLength":65536}` |
| `instructions` | yes | `{"type":"array","items":{"type":"string","minLength":1,"maxLength":8192},"minItems":1,"maxItems":32}` |
| `write_scope` | yes | `{"type":"array","items":{"$ref":"#/$defs/Scope"},"minItems":1,"maxItems":64}` |
| `checkpoint_checks` | yes | `{"type":"array","items":{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"},"minItems":1,"maxItems":64,"uniqueItems":true}` |

## Task

| Field | Required | Constraint |
|---|---|---|
| `schema_version` | yes | `{"const":3}` |
| `task_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `revision` | yes | `{"type":"integer","minimum":1,"maximum":9007199254740991}` |
| `mission_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `root_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `repo_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `owner_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `domain_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `kind` | yes | `{"enum":["code","research"]}` |
| `title` | yes | `{"type":"string","minLength":1,"maxLength":8192}` |
| `outcome` | yes | `{"type":"string","minLength":1,"maxLength":65536}` |
| `requirement_ids` | yes | `{"type":"array","items":{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"},"minItems":1,"maxItems":64,"uniqueItems":true}` |
| `non_goals` | yes | `{"type":"array","items":{"type":"string","minLength":1,"maxLength":8192},"minItems":1,"maxItems":32}` |
| `depends_on` | yes | `{"type":"array","items":{"$ref":"#/$defs/Dependency"},"minItems":0,"maxItems":64}` |
| `write_scope` | yes | `{"type":"array","items":{"$ref":"#/$defs/Scope"},"minItems":0,"maxItems":64}` |
| `resources` | yes | `{"type":"array","items":{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"},"minItems":0,"maxItems":64,"uniqueItems":true}` |
| `steps` | yes | `{"type":"array","items":{"$ref":"#/$defs/Step"},"minItems":0,"maxItems":8}` |
| `acceptance` | yes | `{"type":"array","items":{"$ref":"#/$defs/Acceptance"},"minItems":1,"maxItems":64}` |
| `gate_profile` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `risk` | yes | `{"enum":["low","standard","high"]}` |
| `preferred_profile_digest` | yes | `{"anyOf":[{"type":"string","pattern":"^[0-9a-f]{64}$"},{"type":"null"}]}` |
| `research_method` | yes | `{"anyOf":[{"type":"string","minLength":1,"maxLength":65536},{"type":"null"}]}` |
| `limits` | yes | `{"type":"object","additionalProperties":false,"properties":{"writer_allocations":{"type":"integer","minimum":0,"maximum":9007199254740991},"per_step_max":{"type":"integer","minimum":1,"maximum":3},"research_allocations":{"type":"integer","minimum":0,"maximum":9007199254740991},"wall_time_ms":{"type":"integer","minimum":1,"maximum":9007199254740991},"budget_microusd":{"type":"string","pattern":"^(0&#124;[1-9][0-9]{0,18})$","description":"USD microunits, additionally enforce <= 9223372036854775807 with checked arithmetic."}},"required":["writer_allocations","per_step_max","research_allocations","wall_time_ms","budget_microusd"]}` |

Conditional constraints:

```json
[
  {
    "if": {
      "properties": {
        "kind": {
          "const": "code"
        }
      }
    },
    "then": {
      "properties": {
        "steps": {
          "minItems": 1
        },
        "write_scope": {
          "minItems": 1
        },
        "research_method": {
          "type": "null"
        },
        "limits": {
          "properties": {
            "writer_allocations": {
              "minimum": 1
            },
            "research_allocations": {
              "const": 0
            }
          }
        }
      }
    },
    "else": {
      "properties": {
        "steps": {
          "maxItems": 0
        },
        "write_scope": {
          "maxItems": 0
        },
        "research_method": {
          "type": "string",
          "minLength": 1,
          "maxLength": 65536
        },
        "limits": {
          "properties": {
            "writer_allocations": {
              "const": 0
            },
            "research_allocations": {
              "minimum": 1
            }
          }
        }
      }
    }
  }
]
```

## ExecutionProfile

| Field | Required | Constraint |
|---|---|---|
| `schema_version` | yes | `{"const":3}` |
| `profile_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `harness` | yes | `{"type":"object","additionalProperties":false,"properties":{"name":{"type":"string","minLength":1,"maxLength":8192},"revision":{"type":"string","minLength":1,"maxLength":8192},"adapter_revision":{"type":"string","minLength":1,"maxLength":8192},"transport":{"enum":["fake","codex_app_server","codex_exec_existing","claude_headless","prime_rpc","grok_acp","qualified_other"]}},"required":["name","revision","adapter_revision","transport"]}` |
| `model` | yes | `{"type":"object","additionalProperties":false,"properties":{"provider":{"type":"string","minLength":1,"maxLength":8192},"configured_id":{"type":"string","minLength":1,"maxLength":8192},"observed_revision":{"anyOf":[{"type":"string","minLength":1,"maxLength":8192},{"type":"null"}]},"alias_may_change":{"type":"boolean"},"settings":{"type":"object","additionalProperties":{"type":"string","maxLength":1024},"maxProperties":32}},"required":["provider","configured_id","observed_revision","alias_may_change","settings"]}` |
| `engine` | yes | `{"type":"object","additionalProperties":false,"properties":{"name":{"type":"string","minLength":1,"maxLength":8192},"revision":{"type":"string","minLength":1,"maxLength":8192},"endpoint_ref":{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"},"location":{"enum":["local","approved_remote"]},"model_artifact_digest":{"anyOf":[{"type":"string","pattern":"^[0-9a-f]{64}$"},{"type":"null"}]}},"required":["name","revision","endpoint_ref","location","model_artifact_digest"]}` |
| `context_strategy` | yes | `{"type":"string","minLength":1,"maxLength":8192}` |
| `initial_memory_digest` | yes | `{"anyOf":[{"type":"string","pattern":"^[0-9a-f]{64}$"},{"type":"null"}]}` |
| `skill_digests` | yes | `{"type":"array","items":{"type":"string","pattern":"^[0-9a-f]{64}$"},"minItems":0,"maxItems":64,"uniqueItems":true}` |
| `environment_digest` | yes | `{"type":"string","pattern":"^[0-9a-f]{64}$"}` |
| `isolation_profile` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `data_classification` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `delegation` | yes | `{"type":"object","additionalProperties":false,"properties":{"max_children":{"type":"integer","minimum":0,"maximum":16},"max_depth":{"type":"integer","minimum":0,"maximum":2},"child_writes":{"const":"no_shared_candidate_writes"},"schedules":{"const":"disabled"}},"required":["max_children","max_depth","child_writes","schedules"]}` |
| `usage_basis` | yes | `{"enum":["exclusive_leaf","inclusive_root","estimated","unknown"]}` |
| `grants_authority` | yes | `{"const":false,"description":"Portable profiles cannot self-authorize. Live eligibility is a separate trusted registry/grant."}` |

## Subject

| Field | Required | Constraint |
|---|---|---|
| `kind` | yes | `{"enum":["code","research","integration"]}` |
| `repo_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `task_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `task_revision` | yes | `{"type":"integer","minimum":1,"maximum":9007199254740991}` |
| `contract_digest` | yes | `{"type":"string","pattern":"^[0-9a-f]{64}$"}` |
| `candidate_id` | yes | `{"anyOf":[{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"},{"type":"null"}]}` |
| `artifact_digest` | yes | `{"type":"string","pattern":"^[0-9a-f]{64}$"}` |
| `base_oid` | yes | `{"anyOf":[{"type":"string","pattern":"^(?:[0-9a-f]{40}&#124;[0-9a-f]{64})$"},{"type":"null"}]}` |
| `head_oid` | yes | `{"anyOf":[{"type":"string","pattern":"^(?:[0-9a-f]{40}&#124;[0-9a-f]{64})$"},{"type":"null"}]}` |
| `tree_oid` | yes | `{"anyOf":[{"type":"string","pattern":"^(?:[0-9a-f]{40}&#124;[0-9a-f]{64})$"},{"type":"null"}]}` |
| `integration_oid` | yes | `{"anyOf":[{"type":"string","pattern":"^(?:[0-9a-f]{40}&#124;[0-9a-f]{64})$"},{"type":"null"}]}` |
| `delivery_version` | yes | `{"type":"integer","minimum":0,"maximum":9007199254740991}` |

## Job

| Field | Required | Constraint |
|---|---|---|
| `schema_version` | yes | `{"const":3}` |
| `job_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `mission_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `repo_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `root_id` | yes | `{"anyOf":[{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"},{"type":"null"}]}` |
| `purpose` | yes | `{"enum":["plan","critique","synthesis","implement","research","review","verify","human","evaluate"]}` |
| `executor_kind` | yes | `{"enum":["model","verifier","human","fake"]}` |
| `task_id` | yes | `{"anyOf":[{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"},{"type":"null"}]}` |
| `task_revision` | yes | `{"anyOf":[{"type":"integer","minimum":1,"maximum":9007199254740991},{"type":"null"}]}` |
| `step_index` | yes | `{"anyOf":[{"type":"integer","minimum":0,"maximum":7},{"type":"null"}]}` |
| `profile_digest` | yes | `{"anyOf":[{"type":"string","pattern":"^[0-9a-f]{64}$"},{"type":"null"}]}` |
| `packet_digest` | yes | `{"type":"string","pattern":"^[0-9a-f]{64}$"}` |
| `boot_authority` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `runner_id` | yes | `{"anyOf":[{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"},{"type":"null"}]}` |
| `runner_incarnation` | yes | `{"anyOf":[{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"},{"type":"null"}]}` |
| `writer_generation` | yes | `{"anyOf":[{"type":"integer","minimum":1,"maximum":9007199254740991},{"type":"null"}]}` |
| `expected_delivery_version` | yes | `{"type":"integer","minimum":0,"maximum":9007199254740991}` |
| `base_oid` | yes | `{"anyOf":[{"type":"string","pattern":"^(?:[0-9a-f]{40}&#124;[0-9a-f]{64})$"},{"type":"null"}]}` |
| `parent_checkpoint_id` | yes | `{"anyOf":[{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"},{"type":"null"}]}` |
| `subject` | yes | `{"anyOf":[{"$ref":"#/$defs/Subject"},{"type":"null"}]}` |
| `budget_scope_ids` | yes | `{"type":"array","items":{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"},"minItems":1,"maxItems":16,"uniqueItems":true}` |
| `reservation_microusd` | yes | `{"type":"string","pattern":"^(0&#124;[1-9][0-9]{0,18})$","description":"USD microunits, additionally enforce <= 9223372036854775807 with checked arithmetic."}` |
| `wall_time_ms` | yes | `{"type":"integer","minimum":1,"maximum":9007199254740991}` |
| `lease_ms` | yes | `{"type":"integer","minimum":1,"maximum":9007199254740991}` |
| `new_session_required` | yes | `{"type":"boolean"}` |
| `grants_authority` | yes | `{"const":false,"description":"Example/portable packet cannot mint authority; runtime admission is a trusted transaction."}` |

Conditional constraints:

```json
[
  {
    "if": {
      "properties": {
        "executor_kind": {
          "const": "model"
        }
      }
    },
    "then": {
      "properties": {
        "profile_digest": {
          "type": "string",
          "pattern": "^[0-9a-f]{64}$"
        },
        "runner_id": {
          "type": "string",
          "pattern": "^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"
        },
        "runner_incarnation": {
          "type": "string",
          "pattern": "^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"
        },
        "new_session_required": {
          "const": true
        }
      }
    }
  },
  {
    "if": {
      "properties": {
        "purpose": {
          "enum": [
            "plan",
            "critique",
            "synthesis"
          ]
        }
      }
    },
    "then": {
      "properties": {
        "task_id": {
          "type": "null"
        },
        "task_revision": {
          "type": "null"
        },
        "step_index": {
          "type": "null"
        },
        "writer_generation": {
          "type": "null"
        }
      }
    }
  },
  {
    "if": {
      "properties": {
        "purpose": {
          "const": "implement"
        }
      }
    },
    "then": {
      "properties": {
        "task_id": {
          "type": "string",
          "pattern": "^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"
        },
        "task_revision": {
          "type": "integer",
          "minimum": 1,
          "maximum": 9007199254740991
        },
        "step_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 7
        },
        "writer_generation": {
          "type": "integer",
          "minimum": 1,
          "maximum": 9007199254740991
        },
        "base_oid": {
          "type": "string",
          "pattern": "^(?:[0-9a-f]{40}|[0-9a-f]{64})$"
        }
      }
    }
  }
]
```

## ProposedResult

| Field | Required | Constraint |
|---|---|---|
| `schema_version` | yes | `{"const":3}` |
| `job_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `kind` | yes | `{"enum":["candidate","artifact","findings","blocked","failed"]}` |
| `artifact_digest` | yes | `{"anyOf":[{"type":"string","pattern":"^[0-9a-f]{64}$"},{"type":"null"}]}` |
| `head_oid` | yes | `{"anyOf":[{"type":"string","pattern":"^(?:[0-9a-f]{40}&#124;[0-9a-f]{64})$"},{"type":"null"}]}` |
| `tree_oid` | yes | `{"anyOf":[{"type":"string","pattern":"^(?:[0-9a-f]{40}&#124;[0-9a-f]{64})$"},{"type":"null"}]}` |
| `summary` | yes | `{"type":"string","minLength":1,"maxLength":8192}` |
| `blocker_code` | yes | `{"anyOf":[{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"},{"type":"null"}]}` |
| `check_claims_are_untrusted` | yes | `{"const":true}` |

Conditional constraints:

```json
[
  {
    "if": {
      "properties": {
        "kind": {
          "const": "candidate"
        }
      }
    },
    "then": {
      "properties": {
        "artifact_digest": {
          "type": "string",
          "pattern": "^[0-9a-f]{64}$"
        },
        "head_oid": {
          "type": "string",
          "pattern": "^(?:[0-9a-f]{40}|[0-9a-f]{64})$"
        },
        "tree_oid": {
          "type": "string",
          "pattern": "^(?:[0-9a-f]{40}|[0-9a-f]{64})$"
        }
      }
    }
  },
  {
    "if": {
      "properties": {
        "kind": {
          "const": "artifact"
        }
      }
    },
    "then": {
      "properties": {
        "artifact_digest": {
          "type": "string",
          "pattern": "^[0-9a-f]{64}$"
        },
        "head_oid": {
          "type": "null"
        },
        "tree_oid": {
          "type": "null"
        }
      }
    }
  },
  {
    "if": {
      "properties": {
        "kind": {
          "const": "blocked"
        }
      }
    },
    "then": {
      "properties": {
        "blocker_code": {
          "type": "string",
          "pattern": "^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"
        }
      }
    }
  }
]
```

## Evidence

| Field | Required | Constraint |
|---|---|---|
| `schema_version` | yes | `{"const":3}` |
| `evidence_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `verification_job_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `producer_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `subject` | yes | `{"$ref":"#/$defs/Subject"}` |
| `check_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `recipe_digest` | yes | `{"type":"string","pattern":"^[0-9a-f]{64}$"}` |
| `fixture_digest` | yes | `{"type":"string","pattern":"^[0-9a-f]{64}$"}` |
| `environment_digest` | yes | `{"type":"string","pattern":"^[0-9a-f]{64}$"}` |
| `policy_digest` | yes | `{"type":"string","pattern":"^[0-9a-f]{64}$"}` |
| `outcome` | yes | `{"enum":["pass","fail","error","missing","skipped","unknown"]}` |
| `exit_code` | yes | `{"anyOf":[{"type":"integer","minimum":-255,"maximum":255},{"type":"null"}]}` |
| `required_cases` | yes | `{"type":"integer","minimum":1,"maximum":9007199254740991}` |
| `observed_cases` | yes | `{"type":"integer","minimum":0,"maximum":9007199254740991}` |
| `failed_cases` | yes | `{"type":"integer","minimum":0,"maximum":9007199254740991}` |
| `skipped_cases` | yes | `{"type":"integer","minimum":0,"maximum":9007199254740991}` |
| `logs_digest` | yes | `{"type":"string","pattern":"^[0-9a-f]{64}$"}` |
| `artifacts_complete` | yes | `{"type":"boolean"}` |
| `observed_at_ms` | yes | `{"type":"integer","minimum":0,"maximum":9007199254740991}` |

## Plan

| Field | Required | Constraint |
|---|---|---|
| `schema_version` | yes | `{"const":3}` |
| `mission_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `repo_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `revision` | yes | `{"type":"integer","minimum":1,"maximum":9007199254740991}` |
| `owner_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `objective` | yes | `{"type":"string","minLength":1,"maxLength":65536}` |
| `delivery_boundary` | yes | `{"enum":["review_ready","merged","accepted_artifact","production_observed"]}` |
| `execution_requested` | yes | `{"type":"boolean"}` |
| `intermediate_merge_policy` | yes | `{"enum":["none","human_checkpoints","preauthorized_protected_merge"]}` |
| `requirements` | yes | `{"type":"array","items":{"type":"object","additionalProperties":false,"properties":{"id":{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"},"statement":{"type":"string","minLength":1,"maxLength":65536}},"required":["id","statement"]},"minItems":1,"maxItems":128}` |
| `tasks` | yes | `{"type":"array","items":{"$ref":"#/$defs/Task"},"minItems":1,"maxItems":128}` |
| `decision_artifacts` | yes | `{"type":"array","items":{"type":"string","pattern":"^[0-9a-f]{64}$"},"minItems":0,"maxItems":128,"uniqueItems":true}` |
| `root_allowances` | yes | `{"type":"array","items":{"type":"object","additionalProperties":false,"properties":{"root_id":{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"},"writer_allocations":{"type":"integer","minimum":0,"maximum":9007199254740991},"research_allocations":{"type":"integer","minimum":0,"maximum":9007199254740991},"budget_microusd":{"type":"string","pattern":"^(0&#124;[1-9][0-9]{0,18})$","description":"USD microunits, additionally enforce <= 9223372036854775807 with checked arithmetic."}},"required":["root_id","writer_allocations","research_allocations","budget_microusd"]},"minItems":1,"maxItems":128}` |
| `mission_budget_microusd` | yes | `{"type":"string","pattern":"^(0&#124;[1-9][0-9]{0,18})$","description":"USD microunits, additionally enforce <= 9223372036854775807 with checked arithmetic."}` |

## Command

| Field | Required | Constraint |
|---|---|---|
| `schema_version` | yes | `{"const":3}` |
| `command_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `repo_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `action` | yes | `{"enum":["mission.create","mission.message","plan.propose","plan.activate","task.revise","task.take","task.submit","task.adopt","task.withdraw","target.pause","target.resume","target.stop","target.cancel","decision.resolve","budget.extend","allowance.extend","receipt.export"]}` |
| `target_id` | yes | `{"anyOf":[{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"},{"type":"null"}]}` |
| `expected_version` | yes | `{"anyOf":[{"type":"integer","minimum":0,"maximum":9007199254740991},{"type":"null"}]}` |
| `payload` | yes | `{"type":"object","additionalProperties":false,"properties":{"artifact_digest":{"anyOf":[{"type":"string","pattern":"^[0-9a-f]{64}$"},{"type":"null"}]},"text":{"anyOf":[{"type":"string","minLength":1,"maxLength":65536},{"type":"null"}]},"option_id":{"anyOf":[{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"},{"type":"null"}]},"subject_digest":{"anyOf":[{"type":"string","pattern":"^[0-9a-f]{64}$"},{"type":"null"}]},"amount_microusd":{"anyOf":[{"type":"string","pattern":"^(0&#124;[1-9][0-9]{0,18})$","description":"USD microunits, additionally enforce <= 9223372036854775807 with checked arithmetic."},{"type":"null"}]},"allocation_count":{"anyOf":[{"type":"integer","minimum":0,"maximum":9007199254740991},{"type":"null"}]},"head_oid":{"anyOf":[{"type":"string","pattern":"^(?:[0-9a-f]{40}&#124;[0-9a-f]{64})$"},{"type":"null"}]},"checkpoint_id":{"anyOf":[{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"},{"type":"null"}]}},"required":["artifact_digest","text","option_id","subject_digest","amount_microusd","allocation_count","head_oid","checkpoint_id"]}` |

Action-specific non-null/null and creation/version requirements are additionally encoded as schema conditionals and reproduced in Appendix A of the full engineering specification.

## LearningProposal

| Field | Required | Constraint |
|---|---|---|
| `schema_version` | yes | `{"const":3}` |
| `proposal_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `root_evidence_ids` | yes | `{"type":"array","items":{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"},"minItems":1,"maxItems":32,"uniqueItems":true}` |
| `owner_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `scope` | yes | `{"enum":["owner","domain","repository"]}` |
| `baseline_profile_digest` | yes | `{"type":"string","pattern":"^[0-9a-f]{64}$"}` |
| `candidate_artifact_digest` | yes | `{"type":"string","pattern":"^[0-9a-f]{64}$"}` |
| `target` | yes | `{"enum":["prompt","skill","context_strategy","routing_profile","engine_profile"]}` |
| `hypothesis` | yes | `{"type":"string","minLength":1,"maxLength":65536}` |
| `evaluation_manifest_digest` | yes | `{"type":"string","pattern":"^[0-9a-f]{64}$"}` |
| `status` | yes | `{"const":"proposed"}` |
| `can_self_approve` | yes | `{"const":false}` |

## MissionRequest

| Field | Required | Constraint |
|---|---|---|
| `schema_version` | yes | `{"const":3}` |
| `repo_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `owner_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `domain_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `objective` | yes | `{"type":"string","minLength":1,"maxLength":65536}` |
| `mode` | yes | `{"enum":["plan","run"]}` |
| `delivery_boundary` | yes | `{"enum":["review_ready","merged","accepted_artifact"]}` |
| `intermediate_merge_policy` | yes | `{"enum":["none","human_checkpoints","preauthorized_protected_merge"]}` |
| `policy_id` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `gate_profile` | yes | `{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}` |
| `planning_profile_digest` | yes | `{"type":"string","pattern":"^[0-9a-f]{64}$"}` |
| `parent_budget_scope_ids` | yes | `{"type":"array","minItems":1,"uniqueItems":true,"items":{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"}}` |
| `mission_budget_microusd` | yes | `{"type":"string","pattern":"^(0&#124;[1-9][0-9]{0,18})$","description":"USD microunits, additionally enforce <= 9223372036854775807 with checked arithmetic."}` |
| `planning_budget_microusd` | yes | `{"type":"string","pattern":"^(0&#124;[1-9][0-9]{0,18})$","description":"USD microunits, additionally enforce <= 9223372036854775807 with checked arithmetic."}` |
| `max_planning_jobs` | yes | `{"type":"integer","minimum":1,"maximum":4}` |
| `requirements` | yes | `{"type":"array","items":{"type":"object","additionalProperties":false,"properties":{"id":{"type":"string","pattern":"^[A-Za-z][A-Za-z0-9_.:-]{0,127}$"},"statement":{"type":"string","minLength":1,"maxLength":65536}},"required":["id","statement"]},"minItems":0,"maxItems":128}` |
| `non_goals` | yes | `{"type":"array","items":{"type":"string","minLength":1,"maxLength":4096},"maxItems":100}` |



---

# Appendix C — Reference SQL, defaults and examples

## C1. Initial migration

The following is the complete reference DDL in `migrations/001_initial.sql`. It is a starting relational contract, not the scheduler, a security boundary, or the whole application. Database constraints cover selected immutable/unique relationships; the semantic and authorization checks in Part II are mandatory transactions around these tables. In particular, all repository/profile/subject relationships, path overlap, grant validity, artifact truth and remote effects require the controller and trusted processes.

```sql
-- BulletFarm 3.0 reference persistence contract.
-- This is starter DDL, not an implemented controller or a WAL qualification.
PRAGMA foreign_keys = ON;

CREATE TABLE principals (
  id TEXT PRIMARY KEY, kind TEXT NOT NULL CHECK(kind IN ('human','runner','verifier','publisher')),
  revoked INTEGER NOT NULL DEFAULT 0 CHECK(revoked IN (0,1))
);
CREATE TABLE repositories (
  id TEXT PRIMARY KEY, object_format TEXT NOT NULL CHECK(object_format IN ('sha1','sha256')),
  owner_id TEXT NOT NULL REFERENCES principals(id), policy_digest TEXT NOT NULL,
  live_enabled INTEGER NOT NULL DEFAULT 0 CHECK(live_enabled IN (0,1))
);
CREATE TABLE memberships (
  repo_id TEXT NOT NULL REFERENCES repositories(id), principal_id TEXT NOT NULL REFERENCES principals(id),
  role TEXT NOT NULL CHECK(role IN ('admin','engineer','observer','runner','verifier','publisher')),
  version INTEGER NOT NULL DEFAULT 1 CHECK(version>=1), PRIMARY KEY(repo_id,principal_id)
);
CREATE TABLE artifacts (
  digest TEXT PRIMARY KEY CHECK(length(digest)=64), repo_id TEXT NOT NULL REFERENCES repositories(id),
  size_bytes INTEGER NOT NULL CHECK(size_bytes>=0), media_type TEXT NOT NULL,
  storage_key TEXT NOT NULL UNIQUE, availability TEXT NOT NULL DEFAULT 'present'
    CHECK(availability IN ('present','pruned'))
);
CREATE TABLE profiles (
  digest TEXT PRIMARY KEY CHECK(length(digest)=64), bytes BLOB NOT NULL, profile_id TEXT NOT NULL
);
CREATE TABLE qualifications (
  profile_digest TEXT NOT NULL REFERENCES profiles(digest), repo_id TEXT NOT NULL REFERENCES repositories(id),
  purpose TEXT NOT NULL, environment_digest TEXT NOT NULL, report_digest TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 0 CHECK(enabled IN (0,1)),
  PRIMARY KEY(profile_digest,repo_id,purpose,environment_digest)
);
CREATE TABLE missions (
  id TEXT PRIMARY KEY, repo_id TEXT NOT NULL REFERENCES repositories(id), owner_id TEXT NOT NULL REFERENCES principals(id),
  version INTEGER NOT NULL DEFAULT 1 CHECK(version>=1),
  control TEXT NOT NULL DEFAULT 'paused' CHECK(control IN ('running','paused','stopped','cancelled')),
  boundary TEXT NOT NULL CHECK(boundary IN ('review_ready','merged','accepted_artifact','production_observed')),
  active_plan_digest TEXT, accepted_receipt TEXT
);
CREATE TABLE roots (
  id TEXT PRIMARY KEY, mission_id TEXT NOT NULL REFERENCES missions(id),
  writer_allowance INTEGER NOT NULL CHECK(writer_allowance>=0), writer_used INTEGER NOT NULL DEFAULT 0 CHECK(writer_used>=0),
  research_allowance INTEGER NOT NULL CHECK(research_allowance>=0), research_used INTEGER NOT NULL DEFAULT 0 CHECK(research_used>=0)
);
CREATE TABLE plans (
  mission_id TEXT NOT NULL REFERENCES missions(id), revision INTEGER NOT NULL CHECK(revision>=1),
  digest TEXT NOT NULL CHECK(length(digest)=64), bytes BLOB NOT NULL,
  git_oid TEXT NOT NULL, PRIMARY KEY(mission_id,revision), UNIQUE(mission_id,digest)
);
CREATE TABLE tasks (
  id TEXT PRIMARY KEY, mission_id TEXT NOT NULL REFERENCES missions(id), repo_id TEXT NOT NULL REFERENCES repositories(id),
  root_id TEXT NOT NULL REFERENCES roots(id), owner_id TEXT NOT NULL REFERENCES principals(id),
  current_revision INTEGER NOT NULL CHECK(current_revision>=1), version INTEGER NOT NULL DEFAULT 1 CHECK(version>=1),
  kind TEXT NOT NULL CHECK(kind IN ('code','research')), phase TEXT NOT NULL,
  control TEXT NOT NULL DEFAULT 'paused' CHECK(control IN ('running','paused','stopped','cancelled','withdrawn')),
  writer_generation INTEGER NOT NULL DEFAULT 0 CHECK(writer_generation>=0),
  delivery_version INTEGER NOT NULL DEFAULT 0 CHECK(delivery_version>=0),
  next_step INTEGER NOT NULL DEFAULT 0 CHECK(next_step BETWEEN 0 AND 8),
  selected_candidate_id TEXT REFERENCES candidates(id), selected_artifact_digest TEXT REFERENCES artifacts(digest)
);
CREATE TABLE task_revisions (
  task_id TEXT NOT NULL REFERENCES tasks(id), revision INTEGER NOT NULL CHECK(revision>=1),
  digest TEXT NOT NULL CHECK(length(digest)=64), bytes BLOB NOT NULL,
  step_count INTEGER NOT NULL CHECK(step_count BETWEEN 0 AND 8), PRIMARY KEY(task_id,revision)
);
CREATE TABLE dependencies (
  task_id TEXT NOT NULL, task_revision INTEGER NOT NULL, predecessor_id TEXT NOT NULL,
  predecessor_revision INTEGER NOT NULL, goal TEXT NOT NULL CHECK(goal IN ('merged','accepted_artifact')),
  PRIMARY KEY(task_id,task_revision,predecessor_id,predecessor_revision),
  FOREIGN KEY(task_id,task_revision) REFERENCES task_revisions(task_id,revision),
  FOREIGN KEY(predecessor_id,predecessor_revision) REFERENCES task_revisions(task_id,revision),
  CHECK(task_id<>predecessor_id)
);
CREATE TABLE jobs (
  id TEXT PRIMARY KEY, mission_id TEXT NOT NULL REFERENCES missions(id), repo_id TEXT NOT NULL REFERENCES repositories(id),
  task_id TEXT, task_revision INTEGER, root_id TEXT REFERENCES roots(id),
  purpose TEXT NOT NULL CHECK(purpose IN ('plan','critique','synthesis','implement','research','review','verify','human','evaluate')),
  executor_kind TEXT NOT NULL CHECK(executor_kind IN ('model','verifier','human','fake')),
  profile_digest TEXT REFERENCES profiles(digest), runner_id TEXT REFERENCES principals(id), runner_incarnation TEXT,
  packet_digest TEXT NOT NULL, boot_authority TEXT NOT NULL,
  writer_generation INTEGER, step_index INTEGER CHECK(step_index BETWEEN 0 AND 7),
  expected_delivery_version INTEGER NOT NULL DEFAULT 0 CHECK(expected_delivery_version>=0),
  state TEXT NOT NULL CHECK(state IN ('prepared','starting','running','waiting_tool','waiting_provider','stopping','finished','failed','cancelled','unknown')),
  authoritative INTEGER NOT NULL DEFAULT 0 CHECK(authoritative IN (0,1)), lease_expires_ms INTEGER,
  FOREIGN KEY(task_id,task_revision) REFERENCES task_revisions(task_id,revision),
  CHECK((task_id IS NULL AND task_revision IS NULL) OR (task_id IS NOT NULL AND task_revision IS NOT NULL)),
  CHECK(executor_kind<>'model' OR profile_digest IS NOT NULL),
  CHECK(authoritative=0 OR (task_id IS NOT NULL AND writer_generation>0 AND purpose IN ('implement','human')))
);
CREATE UNIQUE INDEX one_current_writer ON jobs(task_id) WHERE authoritative=1;
CREATE TABLE claims (
  id TEXT PRIMARY KEY, task_id TEXT NOT NULL REFERENCES tasks(id), repo_id TEXT NOT NULL REFERENCES repositories(id),
  scope_json TEXT NOT NULL, state TEXT NOT NULL CHECK(state IN ('held','released'))
);
CREATE UNIQUE INDEX one_task_claim_set ON claims(task_id) WHERE state='held';
CREATE TABLE candidates (
  id TEXT PRIMARY KEY, task_id TEXT NOT NULL, task_revision INTEGER NOT NULL,
  job_id TEXT NOT NULL REFERENCES jobs(id), bundle_digest TEXT NOT NULL REFERENCES artifacts(digest),
  base_oid TEXT NOT NULL, head_oid TEXT NOT NULL, tree_oid TEXT NOT NULL,
  step_index INTEGER NOT NULL CHECK(step_index BETWEEN 0 AND 7), parent_checkpoint_id TEXT REFERENCES candidates(id),
  FOREIGN KEY(task_id,task_revision) REFERENCES task_revisions(task_id,revision)
);
CREATE TABLE checkpoints (
  task_id TEXT NOT NULL, task_revision INTEGER NOT NULL, step_index INTEGER NOT NULL,
  candidate_id TEXT NOT NULL REFERENCES candidates(id), delivery_version INTEGER NOT NULL CHECK(delivery_version>0),
  evidence_group_digest TEXT NOT NULL, active INTEGER NOT NULL CHECK(active IN (0,1)),
  PRIMARY KEY(task_id,task_revision,step_index,candidate_id),
  FOREIGN KEY(task_id,task_revision) REFERENCES task_revisions(task_id,revision)
);
CREATE UNIQUE INDEX one_current_checkpoint ON checkpoints(task_id,task_revision,step_index) WHERE active=1;
CREATE TABLE evidence (
  id TEXT PRIMARY KEY, job_id TEXT NOT NULL REFERENCES jobs(id), producer_id TEXT NOT NULL REFERENCES principals(id),
  task_id TEXT NOT NULL, task_revision INTEGER NOT NULL, candidate_id TEXT REFERENCES candidates(id),
  subject_digest TEXT NOT NULL, check_id TEXT NOT NULL, recipe_digest TEXT NOT NULL,
  environment_digest TEXT NOT NULL, policy_digest TEXT NOT NULL,
  outcome TEXT NOT NULL CHECK(outcome IN ('pass','fail','error','missing','skipped','unknown')),
  required_cases INTEGER NOT NULL CHECK(required_cases>0), observed_cases INTEGER NOT NULL CHECK(observed_cases>=0),
  failed_cases INTEGER NOT NULL CHECK(failed_cases>=0), skipped_cases INTEGER NOT NULL CHECK(skipped_cases>=0),
  logs_digest TEXT NOT NULL REFERENCES artifacts(digest), bytes BLOB NOT NULL,
  FOREIGN KEY(task_id,task_revision) REFERENCES task_revisions(task_id,revision)
);
CREATE TABLE budget_scopes (
  id TEXT PRIMARY KEY, kind TEXT NOT NULL, limit_microusd INTEGER NOT NULL CHECK(limit_microusd>=0),
  spent_microusd INTEGER NOT NULL DEFAULT 0 CHECK(spent_microusd>=0),
  held_microusd INTEGER NOT NULL DEFAULT 0 CHECK(held_microusd>=0)
  -- Do not CHECK spent+held<=limit: real late overspend must be recordable.
);
CREATE TABLE holds (
  job_id TEXT NOT NULL REFERENCES jobs(id), budget_id TEXT NOT NULL REFERENCES budget_scopes(id),
  reserved_microusd INTEGER NOT NULL CHECK(reserved_microusd>=0),
  unresolved_microusd INTEGER NOT NULL CHECK(unresolved_microusd>=0),
  PRIMARY KEY(job_id,budget_id)
);
CREATE TABLE usage_reports (
  source TEXT NOT NULL, report_id TEXT NOT NULL, digest TEXT NOT NULL,
  job_id TEXT NOT NULL REFERENCES jobs(id), basis TEXT NOT NULL,
  amount_microusd INTEGER CHECK(amount_microusd>=0), final INTEGER NOT NULL CHECK(final IN (0,1)),
  bytes BLOB NOT NULL, PRIMARY KEY(source,report_id)
);
CREATE TABLE job_charges (
  job_id TEXT PRIMARY KEY REFERENCES jobs(id), charged_microusd INTEGER NOT NULL DEFAULT 0 CHECK(charged_microusd>=0),
  accounting_final INTEGER NOT NULL DEFAULT 0 CHECK(accounting_final IN (0,1))
);
CREATE TABLE commands (
  principal_id TEXT NOT NULL REFERENCES principals(id), command_id TEXT NOT NULL,
  repo_id TEXT NOT NULL REFERENCES repositories(id), request_digest TEXT NOT NULL,
  operation_id TEXT NOT NULL UNIQUE, bytes BLOB NOT NULL, state TEXT NOT NULL,
  PRIMARY KEY(principal_id,command_id)
);
CREATE TABLE events (
  sequence INTEGER PRIMARY KEY AUTOINCREMENT, repo_id TEXT NOT NULL REFERENCES repositories(id),
  kind TEXT NOT NULL, subject_id TEXT NOT NULL, bytes BLOB NOT NULL
);
CREATE TABLE effects (
  id TEXT PRIMARY KEY, logical_key TEXT NOT NULL UNIQUE, serialization_key TEXT NOT NULL,
  repo_id TEXT NOT NULL REFERENCES repositories(id), task_id TEXT REFERENCES tasks(id),
  candidate_id TEXT REFERENCES candidates(id), delivery_version INTEGER,
  kind TEXT NOT NULL, request_digest TEXT NOT NULL, bytes BLOB NOT NULL,
  state TEXT NOT NULL CHECK(state IN ('pending','sending','unknown','confirmed','not_applied','blocked','cancelled')),
  remote_receipt TEXT
);
CREATE UNIQUE INDEX one_unresolved_effect ON effects(serialization_key)
  WHERE state IN ('pending','sending','unknown','blocked');
CREATE TABLE inbox (
  source TEXT NOT NULL, delivery_id TEXT NOT NULL, repo_id TEXT NOT NULL REFERENCES repositories(id),
  digest TEXT NOT NULL, bytes BLOB NOT NULL, PRIMARY KEY(source,delivery_id)
);
CREATE TABLE observations (
  repo_id TEXT NOT NULL REFERENCES repositories(id), object_key TEXT NOT NULL, observed_at_ms INTEGER NOT NULL,
  bytes BLOB NOT NULL, PRIMARY KEY(repo_id,object_key)
);
CREATE TABLE decisions (
  id TEXT PRIMARY KEY, repo_id TEXT NOT NULL REFERENCES repositories(id), owner_id TEXT NOT NULL REFERENCES principals(id),
  subject_digest TEXT NOT NULL, version INTEGER NOT NULL CHECK(version>=1),
  expires_ms INTEGER NOT NULL, state TEXT NOT NULL CHECK(state IN ('open','applied','rejected','expired','superseded')),
  request_bytes BLOB NOT NULL, resolution_bytes BLOB
);
CREATE TABLE receipts (
  id TEXT PRIMARY KEY, repo_id TEXT NOT NULL REFERENCES repositories(id), mission_id TEXT NOT NULL REFERENCES missions(id),
  digest TEXT NOT NULL, bytes BLOB NOT NULL, observed_sequence INTEGER NOT NULL
);

CREATE TRIGGER immutable_task_revisions_update BEFORE UPDATE ON task_revisions BEGIN SELECT RAISE(ABORT,'immutable task revision'); END;
CREATE TRIGGER immutable_task_revisions_delete BEFORE DELETE ON task_revisions BEGIN SELECT RAISE(ABORT,'retain task revision'); END;
CREATE TRIGGER immutable_profiles_update BEFORE UPDATE ON profiles BEGIN SELECT RAISE(ABORT,'immutable profile'); END;
CREATE TRIGGER immutable_profiles_delete BEFORE DELETE ON profiles BEGIN SELECT RAISE(ABORT,'retain profile history'); END;
CREATE TRIGGER immutable_plans_update BEFORE UPDATE ON plans BEGIN SELECT RAISE(ABORT,'immutable plan'); END;
CREATE TRIGGER immutable_candidates_update BEFORE UPDATE ON candidates BEGIN SELECT RAISE(ABORT,'immutable candidate'); END;
CREATE TRIGGER immutable_evidence_update BEFORE UPDATE ON evidence BEGIN SELECT RAISE(ABORT,'immutable evidence'); END;
CREATE TRIGGER immutable_receipts_update BEFORE UPDATE ON receipts BEGIN SELECT RAISE(ABORT,'immutable receipt'); END;
CREATE TRIGGER monotonic_task_counters BEFORE UPDATE ON tasks
WHEN NEW.version<OLD.version OR NEW.writer_generation<OLD.writer_generation OR NEW.delivery_version<OLD.delivery_version
BEGIN SELECT RAISE(ABORT,'counter moved backwards'); END;
CREATE TRIGGER immutable_job_binding BEFORE UPDATE ON jobs
WHEN NEW.packet_digest<>OLD.packet_digest OR NEW.boot_authority<>OLD.boot_authority
 OR NEW.profile_digest IS NOT OLD.profile_digest OR NEW.task_id IS NOT OLD.task_id
 OR NEW.task_revision IS NOT OLD.task_revision OR NEW.writer_generation IS NOT OLD.writer_generation
BEGIN SELECT RAISE(ABORT,'immutable job binding'); END;
CREATE TRIGGER immutable_effect_intent BEFORE UPDATE ON effects
WHEN NEW.logical_key<>OLD.logical_key OR NEW.serialization_key<>OLD.serialization_key
 OR NEW.request_digest<>OLD.request_digest OR NEW.candidate_id IS NOT OLD.candidate_id
 OR NEW.delivery_version IS NOT OLD.delivery_version OR NEW.bytes<>OLD.bytes
BEGIN SELECT RAISE(ABORT,'immutable effect intent'); END;
CREATE TRIGGER immutable_event_update BEFORE UPDATE ON events BEGIN SELECT RAISE(ABORT,'append-only audit'); END;
CREATE TRIGGER retain_plans_delete BEFORE DELETE ON plans BEGIN SELECT RAISE(ABORT,'retain plans history'); END;
CREATE TRIGGER retain_candidates_delete BEFORE DELETE ON candidates BEGIN SELECT RAISE(ABORT,'retain candidates history'); END;
CREATE TRIGGER retain_evidence_delete BEFORE DELETE ON evidence BEGIN SELECT RAISE(ABORT,'retain evidence history'); END;
CREATE TRIGGER retain_receipts_delete BEFORE DELETE ON receipts BEGIN SELECT RAISE(ABORT,'retain receipts history'); END;
CREATE TRIGGER retain_events_delete BEFORE DELETE ON events BEGIN SELECT RAISE(ABORT,'retain events history'); END;
```

## C2. Default configuration

These are configurable starting values, not measured optimal operating settings. Live grants and experiment spending are disabled until authorized.

```json
{
  "schema_version": 3,
  "live_dispatch": false,
  "auto_merge_classes": [],
  "experimental_budget_microusd": "0",
  "steps_default": 1,
  "steps_max": 8,
  "repair_pool": 2,
  "per_step_max": 3,
  "planning_jobs_max": 3,
  "team_writers": 4,
  "owner_writers": 2,
  "repo_writers": 2,
  "verifiers": 2,
  "integration_per_repo": 1,
  "outstanding_pr_cap": 6,
  "heartbeat_ms": 10000,
  "lease_ms": 60000,
  "lease_margin_ms": 15000,
  "job_wall_ms": 3600000,
  "tool_wall_ms": 600000,
  "command_bytes": 1048576,
  "event_batch_bytes": 4194304,
  "event_batch_count": 100,
  "page_size": 100,
  "page_max": 500,
  "poll_ms": 30000,
  "raw_log_retention_days": 14,
  "failed_workspace_retention_hours": 48
}
```

## C3.mission_request.json

Synthetic fixture, not live authority. The exact file is `examples/mission_request.json`. Its IDs, revisions and allocations are examples; referenced digests are meaningful only for the included exact fixture bytes.

```json
{
  "schema_version": 3,
  "repo_id": "repo_fixture",
  "owner_id": "owner_fixture",
  "domain_id": "domain_results",
  "objective": "Produce a coherent CSV export change while preserving JSON behavior.",
  "mode": "plan",
  "delivery_boundary": "review_ready",
  "intermediate_merge_policy": "none",
  "policy_id": "policy_fixture",
  "gate_profile": "gate_fixture",
  "planning_profile_digest": "be31ff2c40bf03a56fe149aafb1194ac97405524025e641e50ccc61ae6b80e37",
  "parent_budget_scope_ids": [
    "budget_fixture_parent"
  ],
  "mission_budget_microusd": "0",
  "planning_budget_microusd": "0",
  "max_planning_jobs": 3,
  "requirements": [
    {
      "id": "REQ_export",
      "statement": "Users obtain correctly quoted CSV without breaking JSON."
    }
  ],
  "non_goals": [
    "No production deployment."
  ]
}
```

## C3.task.json

Synthetic fixture, not live authority. The exact file is `examples/task.json`. Its IDs, revisions and allocations are examples; referenced digests are meaningful only for the included exact fixture bytes.

```json
{
  "schema_version": 3,
  "task_id": "task_csv",
  "revision": 1,
  "mission_id": "mission_fixture",
  "root_id": "root_export",
  "repo_id": "repo_fixture",
  "owner_id": "owner_fixture",
  "domain_id": "domain_results",
  "kind": "code",
  "title": "Export CSV without changing JSON behavior",
  "outcome": "Users obtain correctly quoted CSV while existing JSON behavior stays unchanged.",
  "requirement_ids": [
    "REQ_export"
  ],
  "non_goals": [
    "No new authentication, data schema, or release behavior."
  ],
  "depends_on": [],
  "write_scope": [
    {
      "path": "src/export",
      "kind": "directory"
    },
    {
      "path": "tests/export",
      "kind": "directory"
    }
  ],
  "resources": [
    "interface_export"
  ],
  "steps": [
    {
      "id": "step_serializer",
      "outcome": "CSV serialization handles delimiters, quotes, nulls and newlines.",
      "instructions": [
        "Implement the specified serializer and regression coverage together."
      ],
      "write_scope": [
        {
          "path": "src/export",
          "kind": "directory"
        },
        {
          "path": "tests/export",
          "kind": "directory"
        }
      ],
      "checkpoint_checks": [
        "csv_roundtrip",
        "json_compat"
      ]
    },
    {
      "id": "step_download",
      "outcome": "Download path uses the serializer and cancels cleanly.",
      "instructions": [
        "Wire the download handler and its request/cancellation regression checks."
      ],
      "write_scope": [
        {
          "path": "src/export",
          "kind": "directory"
        },
        {
          "path": "tests/export",
          "kind": "directory"
        }
      ],
      "checkpoint_checks": [
        "csv_download",
        "csv_cancel"
      ]
    }
  ],
  "acceptance": [
    {
      "id": "AC_serializer",
      "requirement": "Delimiter, quote, newline and null cases follow the approved CSV contract.",
      "check_id": "csv_roundtrip"
    },
    {
      "id": "AC_json",
      "requirement": "Existing JSON responses are unchanged.",
      "check_id": "json_compat"
    },
    {
      "id": "AC_download",
      "requirement": "The download path returns the required artifact and headers.",
      "check_id": "csv_download"
    },
    {
      "id": "AC_cancel",
      "requirement": "Cancellation does not publish a partial artifact.",
      "check_id": "csv_cancel"
    }
  ],
  "gate_profile": "gate_export_fixture",
  "risk": "standard",
  "preferred_profile_digest": "be31ff2c40bf03a56fe149aafb1194ac97405524025e641e50ccc61ae6b80e37",
  "research_method": null,
  "limits": {
    "writer_allocations": 4,
    "per_step_max": 3,
    "research_allocations": 0,
    "wall_time_ms": 3600000,
    "budget_microusd": "8000000"
  }
}
```

## C3.research_task.json

Synthetic fixture, not live authority. The exact file is `examples/research_task.json`. Its IDs, revisions and allocations are examples; referenced digests are meaningful only for the included exact fixture bytes.

```json
{
  "schema_version": 3,
  "task_id": "task_research",
  "revision": 1,
  "mission_id": "mission_fixture",
  "root_id": "root_research",
  "repo_id": "repo_fixture",
  "owner_id": "owner_fixture",
  "domain_id": "domain_results",
  "kind": "research",
  "title": "Investigate CSV cancellation strategy",
  "outcome": "Compare two cancellation strategies on frozen synthetic inputs and report a bounded conclusion.",
  "requirement_ids": [
    "REQ_research"
  ],
  "non_goals": [
    "No new authentication, data schema, or release behavior."
  ],
  "depends_on": [],
  "write_scope": [],
  "resources": [],
  "steps": [],
  "acceptance": [
    {
      "id": "AC_report",
      "requirement": "The report identifies input, method, observed outcomes, uncertainty and limitations.",
      "check_id": "research_report"
    }
  ],
  "gate_profile": "gate_export_fixture",
  "risk": "standard",
  "preferred_profile_digest": "be31ff2c40bf03a56fe149aafb1194ac97405524025e641e50ccc61ae6b80e37",
  "research_method": "Run two frozen synthetic workloads with the same inputs; a negative result is valid. No source publication.",
  "limits": {
    "writer_allocations": 0,
    "per_step_max": 3,
    "research_allocations": 2,
    "wall_time_ms": 3600000,
    "budget_microusd": "2000000"
  }
}
```

## C3.profile.json

Synthetic fixture, not live authority. The exact file is `examples/profile.json`. Its IDs, revisions and allocations are examples; referenced digests are meaningful only for the included exact fixture bytes.

```json
{
  "schema_version": 3,
  "profile_id": "profile_fixture",
  "harness": {
    "name": "fixture-harness",
    "revision": "fixture-only",
    "adapter_revision": "fixture-v1",
    "transport": "fake"
  },
  "model": {
    "provider": "fake",
    "configured_id": "fixture-model",
    "observed_revision": "fixture-v1",
    "alias_may_change": false,
    "settings": {}
  },
  "engine": {
    "name": "fake",
    "revision": "v1",
    "endpoint_ref": "endpoint_fixture",
    "location": "local",
    "model_artifact_digest": null
  },
  "context_strategy": "exact-contract-and-checkpoint",
  "initial_memory_digest": null,
  "skill_digests": [],
  "environment_digest": "1111111111111111111111111111111111111111111111111111111111111111",
  "isolation_profile": "fixture_no_network",
  "data_classification": "synthetic",
  "delegation": {
    "max_children": 0,
    "max_depth": 0,
    "child_writes": "no_shared_candidate_writes",
    "schedules": "disabled"
  },
  "usage_basis": "exclusive_leaf",
  "grants_authority": false
}
```

## C3.plan.json

Synthetic fixture, not live authority. The exact file is `examples/plan.json`. Its IDs, revisions and allocations are examples; referenced digests are meaningful only for the included exact fixture bytes.

```json
{
  "schema_version": 3,
  "mission_id": "mission_fixture",
  "repo_id": "repo_fixture",
  "revision": 1,
  "owner_id": "owner_fixture",
  "objective": "Produce a coherent CSV export change with preserved JSON compatibility.",
  "delivery_boundary": "review_ready",
  "execution_requested": false,
  "intermediate_merge_policy": "none",
  "requirements": [
    {
      "id": "REQ_export",
      "statement": "Users obtain correctly quoted CSV while existing JSON behavior stays unchanged."
    }
  ],
  "tasks": [
    {
      "schema_version": 3,
      "task_id": "task_csv",
      "revision": 1,
      "mission_id": "mission_fixture",
      "root_id": "root_export",
      "repo_id": "repo_fixture",
      "owner_id": "owner_fixture",
      "domain_id": "domain_results",
      "kind": "code",
      "title": "Export CSV without changing JSON behavior",
      "outcome": "Users obtain correctly quoted CSV while existing JSON behavior stays unchanged.",
      "requirement_ids": [
        "REQ_export"
      ],
      "non_goals": [
        "No new authentication, data schema, or release behavior."
      ],
      "depends_on": [],
      "write_scope": [
        {
          "path": "src/export",
          "kind": "directory"
        },
        {
          "path": "tests/export",
          "kind": "directory"
        }
      ],
      "resources": [
        "interface_export"
      ],
      "steps": [
        {
          "id": "step_serializer",
          "outcome": "CSV serialization handles delimiters, quotes, nulls and newlines.",
          "instructions": [
            "Implement the specified serializer and regression coverage together."
          ],
          "write_scope": [
            {
              "path": "src/export",
              "kind": "directory"
            },
            {
              "path": "tests/export",
              "kind": "directory"
            }
          ],
          "checkpoint_checks": [
            "csv_roundtrip",
            "json_compat"
          ]
        },
        {
          "id": "step_download",
          "outcome": "Download path uses the serializer and cancels cleanly.",
          "instructions": [
            "Wire the download handler and its request/cancellation regression checks."
          ],
          "write_scope": [
            {
              "path": "src/export",
              "kind": "directory"
            },
            {
              "path": "tests/export",
              "kind": "directory"
            }
          ],
          "checkpoint_checks": [
            "csv_download",
            "csv_cancel"
          ]
        }
      ],
      "acceptance": [
        {
          "id": "AC_serializer",
          "requirement": "Delimiter, quote, newline and null cases follow the approved CSV contract.",
          "check_id": "csv_roundtrip"
        },
        {
          "id": "AC_json",
          "requirement": "Existing JSON responses are unchanged.",
          "check_id": "json_compat"
        },
        {
          "id": "AC_download",
          "requirement": "The download path returns the required artifact and headers.",
          "check_id": "csv_download"
        },
        {
          "id": "AC_cancel",
          "requirement": "Cancellation does not publish a partial artifact.",
          "check_id": "csv_cancel"
        }
      ],
      "gate_profile": "gate_export_fixture",
      "risk": "standard",
      "preferred_profile_digest": "be31ff2c40bf03a56fe149aafb1194ac97405524025e641e50ccc61ae6b80e37",
      "research_method": null,
      "limits": {
        "writer_allocations": 4,
        "per_step_max": 3,
        "research_allocations": 0,
        "wall_time_ms": 3600000,
        "budget_microusd": "8000000"
      }
    }
  ],
  "decision_artifacts": [],
  "root_allowances": [
    {
      "root_id": "root_export",
      "writer_allocations": 4,
      "research_allocations": 0,
      "budget_microusd": "8000000"
    }
  ],
  "mission_budget_microusd": "10000000"
}
```

## C3.job.json

Synthetic fixture, not live authority. The exact file is `examples/job.json`. Its IDs, revisions and allocations are examples; referenced digests are meaningful only for the included exact fixture bytes.

```json
{
  "schema_version": 3,
  "job_id": "job_fixture",
  "mission_id": "mission_fixture",
  "repo_id": "repo_fixture",
  "root_id": "root_export",
  "purpose": "implement",
  "executor_kind": "model",
  "task_id": "task_csv",
  "task_revision": 1,
  "step_index": 0,
  "profile_digest": "be31ff2c40bf03a56fe149aafb1194ac97405524025e641e50ccc61ae6b80e37",
  "packet_digest": "2222222222222222222222222222222222222222222222222222222222222222",
  "boot_authority": "boot_fixture",
  "runner_id": "runner_fixture",
  "runner_incarnation": "inc_fixture",
  "writer_generation": 1,
  "expected_delivery_version": 0,
  "base_oid": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "parent_checkpoint_id": null,
  "subject": null,
  "budget_scope_ids": [
    "budget_mission",
    "budget_task"
  ],
  "reservation_microusd": "1000000",
  "wall_time_ms": 3600000,
  "lease_ms": 60000,
  "new_session_required": true,
  "grants_authority": false
}
```

## C3.planning_job.json

Synthetic fixture, not live authority. The exact file is `examples/planning_job.json`. Its IDs, revisions and allocations are examples; referenced digests are meaningful only for the included exact fixture bytes.

```json
{
  "schema_version": 3,
  "job_id": "job_plan",
  "mission_id": "mission_fixture",
  "repo_id": "repo_fixture",
  "root_id": null,
  "purpose": "plan",
  "executor_kind": "model",
  "task_id": null,
  "task_revision": null,
  "step_index": null,
  "profile_digest": "be31ff2c40bf03a56fe149aafb1194ac97405524025e641e50ccc61ae6b80e37",
  "packet_digest": "2222222222222222222222222222222222222222222222222222222222222222",
  "boot_authority": "boot_fixture",
  "runner_id": "runner_fixture",
  "runner_incarnation": "inc_fixture",
  "writer_generation": null,
  "expected_delivery_version": 0,
  "base_oid": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "parent_checkpoint_id": null,
  "subject": null,
  "budget_scope_ids": [
    "budget_mission"
  ],
  "reservation_microusd": "1000000",
  "wall_time_ms": 3600000,
  "lease_ms": 60000,
  "new_session_required": true,
  "grants_authority": false
}
```

## C3.evidence.json

Synthetic fixture, not live authority. The exact file is `examples/evidence.json`. Its IDs, revisions and allocations are examples; referenced digests are meaningful only for the included exact fixture bytes.

```json
{
  "schema_version": 3,
  "evidence_id": "evidence_fixture",
  "verification_job_id": "verify_fixture",
  "producer_id": "verifier_fixture",
  "subject": {
    "kind": "code",
    "repo_id": "repo_fixture",
    "task_id": "task_csv",
    "task_revision": 1,
    "contract_digest": "a55357275f57a876bb988f941bcd635eebf6050cfc0386bba6f3189e2c130375",
    "candidate_id": "candidate_fixture",
    "artifact_digest": "3333333333333333333333333333333333333333333333333333333333333333",
    "base_oid": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "head_oid": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "tree_oid": "cccccccccccccccccccccccccccccccccccccccc",
    "integration_oid": null,
    "delivery_version": 1
  },
  "check_id": "csv_roundtrip",
  "recipe_digest": "4444444444444444444444444444444444444444444444444444444444444444",
  "fixture_digest": "5555555555555555555555555555555555555555555555555555555555555555",
  "environment_digest": "1111111111111111111111111111111111111111111111111111111111111111",
  "policy_digest": "6666666666666666666666666666666666666666666666666666666666666666",
  "outcome": "pass",
  "exit_code": 0,
  "required_cases": 4,
  "observed_cases": 4,
  "failed_cases": 0,
  "skipped_cases": 0,
  "logs_digest": "7777777777777777777777777777777777777777777777777777777777777777",
  "artifacts_complete": true,
  "observed_at_ms": 1789560000000
}
```

## C3.result.json

Synthetic fixture, not live authority. The exact file is `examples/result.json`. Its IDs, revisions and allocations are examples; referenced digests are meaningful only for the included exact fixture bytes.

```json
{
  "schema_version": 3,
  "job_id": "job_fixture",
  "kind": "candidate",
  "artifact_digest": "3333333333333333333333333333333333333333333333333333333333333333",
  "head_oid": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  "tree_oid": "cccccccccccccccccccccccccccccccccccccccc",
  "summary": "Proposed synthetic serializer candidate; independent checks still required.",
  "blocker_code": null,
  "check_claims_are_untrusted": true
}
```

## C3.command.json

Synthetic fixture, not live authority. The exact file is `examples/command.json`. Its IDs, revisions and allocations are examples; referenced digests are meaningful only for the included exact fixture bytes.

```json
{
  "schema_version": 3,
  "command_id": "command_pause",
  "repo_id": "repo_fixture",
  "action": "target.pause",
  "target_id": "mission_fixture",
  "expected_version": 1,
  "payload": {
    "artifact_digest": null,
    "text": null,
    "option_id": null,
    "subject_digest": null,
    "amount_microusd": null,
    "allocation_count": null,
    "head_oid": null,
    "checkpoint_id": null
  }
}
```

## C3.learning_proposal.json

Synthetic fixture, not live authority. The exact file is `examples/learning_proposal.json`. Its IDs, revisions and allocations are examples; referenced digests are meaningful only for the included exact fixture bytes.

```json
{
  "schema_version": 3,
  "proposal_id": "lesson_fixture",
  "root_evidence_ids": [
    "evidence_fixture"
  ],
  "owner_id": "owner_fixture",
  "scope": "repository",
  "baseline_profile_digest": "be31ff2c40bf03a56fe149aafb1194ac97405524025e641e50ccc61ae6b80e37",
  "candidate_artifact_digest": "8888888888888888888888888888888888888888888888888888888888888888",
  "target": "context_strategy",
  "hypothesis": "A smaller source packet reduces handling cost without losing required behaviors.",
  "evaluation_manifest_digest": "9999999999999999999999999999999999999999999999999999999999999999",
  "status": "proposed",
  "can_self_approve": false
}
```

## C3.ENVIRONMENT_BINDINGS.json

Synthetic fixture, not live authority. The exact file is `examples/ENVIRONMENT_BINDINGS.json`. Its IDs, revisions and allocations are examples; referenced digests are meaningful only for the included exact fixture bytes.

```json
{
  "live_dispatch": false,
  "repository_url": null,
  "owner_membership": null,
  "provider_account_ref": null,
  "allowed_model_id": null,
  "runtime_binary_digest": null,
  "sandbox_qualification": null,
  "gate_registry": null,
  "trusted_reporter": null,
  "forge_qualification": null,
  "budget_grant": null,
  "production_authority": null,
  "note": "Nulls are unresolved required inputs, not permissions for the agent to invent. Read-only discovery may resolve facts, not consent."
}
```


---

# Appendix D — Canonical implementation backlog

Generated from `BACKLOG.json`; edit the JSON and regenerate, never maintain conflicting lists. Build packages are not services or automatically dispatchable jobs. All real calls require authorized environment bindings.

## BF3-001 — Contracts, fixtures and semantic admission

**Stage:** G0. **Depends on:** none. **Normative sections:** 6, 7, 8.

**Write scope:** `src/domain/`, `schemas/`, `fixtures/contracts/`.

Contracts, fixtures and semantic admission. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-001-AC01 | Valid data round-trips; ambiguous JSON and out-of-range money fail before any authority is created | `bf3_001_ac01` |
| BF3-001-AC02 | The exact invalid edge and uncovered root requirement are rejected; no task becomes runnable | `bf3_001_ac02` |
| BF3-001-AC03 | Segment boundaries and subset checks hold; invalid steps or zero code steps fail | `bf3_001_ac03` |
| BF3-001-AC04 | Mission planning uses a null task reference; every model job requires a profile; human work does not require a fake provider | `bf3_001_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-002 — Durable store and command receipts

**Stage:** G0. **Depends on:** BF3-001. **Normative sections:** 3, 19, 20.

**Write scope:** `src/store/`, `migrations/`, `tests/store/`.

Durable store and command receipts. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-002-AC01 | All three roll back; no successful acknowledgement is emitted | `bf3_002_ac01` |
| BF3-002-AC02 | The identical request returns the same operation; different bytes conflict and revoked readers receive no old response | `bf3_002_ac02` |
| BF3-002-AC03 | Database and handler guards reject changes and retain history | `bf3_002_ac03` |
| BF3-002-AC04 | The database transaction is already released; unrelated operations stay usable | `bf3_002_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-003 — Identity, minimal API and first-party CLI

**Stage:** G0. **Depends on:** BF3-002. **Normative sections:** 4, 19.

**Write scope:** `src/api/`, `src/cli/`, `src/main.rs`.

Identity, minimal API and first-party CLI. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-003-AC01 | Server-derived identity controls authorization; the payload cannot impersonate a human | `bf3_003_ac01` |
| BF3-003-AC02 | Every surface enforces current repository access including guessed IDs and old cursors | `bf3_003_ac02` |
| BF3-003-AC03 | One exchange succeeds; stored secret material is protected; no runner credential enters a candidate | `bf3_003_ac03` |
| BF3-003-AC04 | Controls use durable facts with zero model calls and no terminal fallback | `bf3_003_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-004 — Money, invocation allowances and capacity

**Stage:** G0. **Depends on:** BF3-002, BF3-003. **Normative sections:** 8.3, 13.

**Write scope:** `src/control/budgets.rs`, `src/control/allowances.rs`, `tests/budget/`.

Money, invocation allowances and capacity. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-004-AC01 | Only admissible requests commit; any failure leaves every hold and allocation unchanged | `bf3_004_ac01` |
| BF3-004-AC02 | One leaf charge basis is counted once; unknown final usage retains exposure | `bf3_004_ac02` |
| BF3-004-AC03 | Actual overrun is stored, not clipped/rejected, and new admission stops | `bf3_004_ac03` |
| BF3-004-AC04 | No new default pool is manufactured; approved first passes and remaining repairs must fit explicit root limits | `bf3_004_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-005 — Immutable Git intent and activation

**Stage:** G0. **Depends on:** BF3-001, BF3-002, BF3-003. **Normative sections:** 7.

**Write scope:** `src/intent/`, `tests/intent/`.

Immutable Git intent and activation. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-005-AC01 | The revision is inert until the complete graph is atomically adopted; no half graph dispatches | `bf3_005_ac01` |
| BF3-005-AC02 | One set of task revisions and dependencies exists | `bf3_005_ac02` |
| BF3-005-AC03 | Only trusted current policy applies; metadata cannot trigger source deployment or force-push | `bf3_005_ac03` |
| BF3-005-AC04 | It is rejected or explicitly revised to steps/independent work/human checkpoints; PR-ready never impersonates merged | `bf3_005_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-006 — Runner journal and isolated fake execution

**Stage:** G0. **Depends on:** BF3-003, BF3-004, BF3-005. **Normative sections:** 11, 14.

**Write scope:** `src/runner/`, `src/providers/fake.rs`, `fixtures/provider/`.

Runner journal and isolated fake execution. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-006-AC01 | The exact existing run is reconciled or blocked; no blind duplicate process is launched | `bf3_006_ac01` |
| BF3-006-AC02 | Response arrival does not extend authority; the job self-stops conservatively and reports uncertainty | `bf3_006_ac02` |
| BF3-006-AC03 | Only its recorded tree is stopped; surviving/unreachable descendants remain unconfirmed | `bf3_006_ac03` |
| BF3-006-AC04 | Bounded diagnostic handling does not lose terminal state or falsely kill the healthy tool for silence | `bf3_006_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-007 — Shared claims, writer generations and scheduler

**Stage:** G0. **Depends on:** BF3-002, BF3-004, BF3-006. **Normative sections:** 12, 13.

**Write scope:** `src/control/claims.rs`, `src/control/scheduler.rs`, `tests/authority/`.

Shared claims, writer generations and scheduler. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-007-AC01 | One authoritative generation wins; the loser consumes no budget or partial claims | `bf3_007_ac01` |
| BF3-007-AC02 | Conflicting work waits, disjoint work proceeds and no partial claim deadlock occurs | `bf3_007_ac02` |
| BF3-007-AC03 | Whole-task reservations remain through verification/review without holding compute | `bf3_007_ac03` |
| BF3-007-AC04 | Old output remains salvage only; it cannot change current selection or authorize effects | `bf3_007_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-008 — Sealed candidates and three-lifetime handoff

**Stage:** G0. **Depends on:** BF3-005, BF3-006, BF3-007. **Normative sections:** 12, 14.1.

**Write scope:** `src/delivery/intake.rs`, `src/control/candidates.rs`, `tests/intake/`.

Sealed candidates and three-lifetime handoff. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-008-AC01 | The malformed artifact is rejected/contained; publisher identity executes no candidate-controlled code | `bf3_008_ac01` |
| BF3-008-AC02 | Exact base/head/tree, contract, artifact and delivery version are recorded before current-author status is released | `bf3_008_ac02` |
| BF3-008-AC03 | The author need not retain a live lease; current delivery authority remains required | `bf3_008_ac03` |
| BF3-008-AC04 | History can be retained but current state cannot advance; a new candidate needs new evidence | `bf3_008_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-009 — Gate adequacy and independent verification

**Stage:** G0. **Depends on:** BF3-001, BF3-006, BF3-008. **Normative sections:** 15.1, 15.2.

**Write scope:** `src/verification/`, `fixtures/gates/`, `tests/verification/`.

Gate adequacy and independent verification. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-009-AC01 | Baseline passes and wrong behavior fails for its intended reason; inapplicable or infrastructure failures are labeled | `bf3_009_ac01` |
| BF3-009-AC02 | Self-report or wrong issuer cannot satisfy the assigned verifier record | `bf3_009_ac02` |
| BF3-009-AC03 | Readiness fails despite exit zero; logs and required discovery are checked | `bf3_009_ac03` |
| BF3-009-AC04 | Store it only for its original subject; it cannot certify the new selection | `bf3_009_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-010 — Forge outbox, provisional drafts and reconciliation

**Stage:** G0. **Depends on:** BF3-002, BF3-003, BF3-008, BF3-009. **Normative sections:** 15.3, 16.

**Write scope:** `src/delivery/forge.rs`, `src/delivery/effects.rs`, `fixtures/forge/`.

Forge outbox, provisional drafts and reconciliation. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-010-AC01 | Find the exact PR/head/task marker; do not blindly create another | `bf3_010_ac01` |
| BF3-010-AC02 | Remain unknown until safe nonapplication or exact success is established; conflicting effect stays blocked | `bf3_010_ac02` |
| BF3-010-AC03 | Draft opens after local minimum but stays unready; no merge authority appears until full obligations hold | `bf3_010_ac03` |
| BF3-010-AC04 | Do not reopen, overwrite or force-push automatically; surface exact divergence | `bf3_010_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-011 — First executor qualification and incumbent/Prime trial

**Stage:** G1. **Depends on:** BF3-004, BF3-006, BF3-008, BF3-009, BF3-010. **Normative sections:** 10, 11.

**Write scope:** `src/providers/`, `fixtures/providers/`, `qualification/executors/`.

First executor qualification and incumbent/Prime trial. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-011-AC01 | A genuine new session executes without routine TUI menus; inherited planner history is not relabeled fresh | `bf3_011_ac01` |
| BF3-011-AC02 | Each yields a bounded typed state; Codex unsandboxed shell methods and new authority are not exposed | `bf3_011_ac02` |
| BF3-011-AC03 | One conforming path is used; a failed/over-budget trial falls back without making three platforms mandatory | `bf3_011_ac03` |
| BF3-011-AC04 | Actual credential/child/usage boundaries are recorded; unsupported profiles are explicitly ineligible | `bf3_011_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-012 — First useful transaction and offline product demo

**Stage:** G1. **Depends on:** BF3-001, BF3-002, BF3-003, BF3-004, BF3-005, BF3-006, BF3-007, BF3-008, BF3-009, BF3-010, BF3-011. **Normative sections:** 2, 23, 24.

**Write scope:** `tests/e2e/first_win.rs`, `src/cli/demo.rs`, `docs/first-win.md`.

First useful transaction and offline product demo. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-012-AC01 | Success and failure scenarios produce honest simulated results, never live acceptance | `bf3_012_ac01` |
| BF3-012-AC02 | A real independently checked draft appears with actual SHA, evidence and known limitations | `bf3_012_ac02` |
| BF3-012-AC03 | Candidate survives, ownership stays reserved and one logical PR is observed | `bf3_012_ac03` |
| BF3-012-AC04 | Draft, review-ready, merge-eligible and merged are not conflated; no deployment is claimed | `bf3_012_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-013 — Ordered step engine and finite repair pool

**Stage:** G2. **Depends on:** BF3-012. **Normative sections:** 8, 12.

**Write scope:** `src/control/steps.rs`, `tests/steps/`.

Ordered step engine and finite repair pool. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-013-AC01 | One coherent task reaches a final candidate without an intermediate human merge | `bf3_013_ac01` |
| BF3-013-AC02 | Only matching current cumulative evidence advances the cursor once | `bf3_013_ac02` |
| BF3-013-AC03 | All dependent suffix evidence loses eligibility; historical results remain | `bf3_013_ac03` |
| BF3-013-AC04 | Limits remain cumulative; no hidden third repair or pool reset is admitted | `bf3_013_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-014 — Second complete coding provider

**Stage:** G2. **Depends on:** BF3-012. **Normative sections:** 10, 11.

**Write scope:** `src/providers/claude.rs`, `tests/providers/`, `fixtures/providers/`.

Second complete coding provider. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-014-AC01 | Read-only summarization alone does not pass the second-executor gate | `bf3_014_ac01` |
| BF3-014-AC02 | Only approved explicit configuration loads; bare/auth assumptions are actually tested | `bf3_014_ac02` |
| BF3-014-AC03 | New session receives exact task/evidence and remaining allowance without policy widening | `bf3_014_ac03` |
| BF3-014-AC04 | Capability gaps remain visible; neither adapter changes core authority semantics | `bf3_014_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-015 — Bounded planning, task packets and lead

**Stage:** G2. **Depends on:** BF3-005, BF3-012, BF3-013, BF3-014. **Normative sections:** 7, 8, 9.

**Write scope:** `src/lead/`, `tests/planning/`.

Bounded planning, task packets and lead. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-015-AC01 | Standard does not claim blindness; Deep proposals precede cross-exposure; every job is metered | `bf3_015_ac01` |
| BF3-015-AC02 | Run starts fresh execution automatically; plan never launches a writer | `bf3_015_ac02` |
| BF3-015-AC03 | Owner/experiment path is explicit; model votes cannot override missing gates or new authority | `bf3_015_ac03` |
| BF3-015-AC04 | Only authorized exact source/decisions/relevant failure facts enter the compact packet; not every transcript | `bf3_015_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-016 — Independent review and strict acceptance aggregate

**Stage:** G2. **Depends on:** BF3-009, BF3-014, BF3-015. **Normative sections:** 15.2, 15.4.

**Write scope:** `src/verification/review.rs`, `src/verification/aggregate.rs`, `tests/review/`.

Independent review and strict acceptance aggregate. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-016-AC01 | Risk rises to policy minimum; writer cannot select lighter acceptance | `bf3_016_ac01` |
| BF3-016-AC02 | A separate qualified job reviews exact requirements/diff/evidence; actor spoofing fails | `bf3_016_ac02` |
| BF3-016-AC03 | Only complete trusted matching evidence can succeed | `bf3_016_ac03` |
| BF3-016-AC04 | Old review remains historical and cannot authorize changed code without required recheck | `bf3_016_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-017 — Human ownership, takeover and safe withdrawal

**Stage:** G2. **Depends on:** BF3-008, BF3-010, BF3-012, BF3-013. **Normative sections:** 4, 12.4, 17.

**Write scope:** `src/people/`, `tests/people/`.

Human ownership, takeover and safe withdrawal. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-017-AC01 | Old authority is revoked; a new private clone uses only actually captured state | `bf3_017_ac01` |
| BF3-017-AC02 | No bot silently takes over and no editor is killed; submission revalidates current assignment | `bf3_017_ac02` |
| BF3-017-AC03 | Provenance is retrospective and lifetime cost/allowance/history remain intact | `bf3_017_ac03` |
| BF3-017-AC04 | Claims release only after relevant effects and remote eligibility are settled; no claimed undo of merged code | `bf3_017_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-018 — Team fairness, backpressure and external PR awareness

**Stage:** G2. **Depends on:** BF3-007, BF3-010, BF3-014, BF3-017. **Normative sections:** 4, 13.

**Write scope:** `src/control/fairness.rs`, `src/delivery/observations.rs`, `tests/team/`.

Team fairness, backpressure and external PR awareness. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-018-AC01 | Preferences hold within shared policy; idle capacity never authorizes stealing another roadmap/account | `bf3_018_ac01` |
| BF3-018-AC02 | New writing slows while verification/repair/reconciliation continue | `bf3_018_ac02` |
| BF3-018-AC03 | Observed overlap is shown with timestamp; invisible work is not claimed controlled | `bf3_018_ac03` |
| BF3-018-AC04 | The configured fairness/aging policy gives the second owner progress and explainable ordering | `bf3_018_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-019 — Mine/Team web interface and event consistency

**Stage:** G2. **Depends on:** BF3-003, BF3-015, BF3-017, BF3-018. **Normative sections:** 19.

**Write scope:** `web/`, `src/api/events.rs`, `tests/ui/`.

Mine/Team web interface and event consistency. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-019-AC01 | Deterministic committed projections win; prose cannot create a successful status | `bf3_019_ac01` |
| BF3-019-AC02 | Replay or snapshot resync repairs state without duplicate commands or leaked repository data | `bf3_019_ac02` |
| BF3-019-AC03 | 409/current-subject rejection prevents changed action authorization | `bf3_019_ac03` |
| BF3-019-AC04 | Focus, labels and bounded rendering work; one incident replaces repetitive per-task alerts | `bf3_019_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-020 — Protected integration and final mission acceptance

**Stage:** G2. **Depends on:** BF3-010, BF3-013, BF3-016, BF3-018. **Normative sections:** 7, 15.5, 22.

**Write scope:** `src/delivery/integration.rs`, `src/control/mission_acceptance.rs`, `tests/integration/`.

Protected integration and final mission acceptance. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-020-AC01 | Current combined checks run and block incompatible composition; PR-head PASS is insufficient | `bf3_020_ac01` |
| BF3-020-AC02 | Candidate/tested-tree/actual-merge relation is established or explicitly unknown | `bf3_020_ac02` |
| BF3-020-AC03 | Stop at the existing protected manual path; no unsafe scripted fallback is invented | `bf3_020_ac03` |
| BF3-020-AC04 | Mission stays incomplete; production-observed requests fail early if release adapter is unavailable | `bf3_020_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-021 — Boot recovery, backups and retention

**Stage:** G2. **Depends on:** BF3-008, BF3-010, BF3-017, BF3-020. **Normative sections:** 3.1, 16.

**Write scope:** `src/control/recovery.rs`, `src/store/backup.rs`, `tests/recovery/`.

Boot recovery, backups and retention. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-021-AC01 | New authority revokes authors, retains candidates and revalidates delivery; no automatic duplicate create | `bf3_021_ac01` |
| BF3-021-AC02 | Remain paused until old credentials/authority are disabled and remote state/artifacts are reconciled | `bf3_021_ac02` |
| BF3-021-AC03 | No false durable acknowledgement or accepted incomplete proof appears | `bf3_021_ac03` |
| BF3-021-AC04 | It is retained; deletion needs exact identity, terminal/reconciled status and safe reference checks | `bf3_021_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-022 — Portable receipts and complete outcome accounting

**Stage:** G2. **Depends on:** BF3-004, BF3-005, BF3-009, BF3-017, BF3-021. **Normative sections:** 18, 23, 24.

**Write scope:** `src/reporting/`, `tests/export/`.

Portable receipts and complete outcome accounting. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-022-AC01 | Portable history preserves uncertainty and excludes credentials/live grants; import starts paused | `bf3_022_ac01` |
| BF3-022-AC02 | Root delivered value is not multiplied; scope-aggregate money is not double-counted | `bf3_022_ac02` |
| BF3-022-AC03 | Quality labels remain and missing active minutes are not zero | `bf3_022_ac03` |
| BF3-022-AC04 | Intent, selected code and check definitions are intelligible without native provider sessions | `bf3_022_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-023 — Profile privacy, diagnostics and performance qualification

**Stage:** G2. **Depends on:** BF3-011, BF3-014, BF3-018, BF3-019, BF3-021, BF3-022. **Normative sections:** 10, 18, 21.

**Write scope:** `src/control/profiles.rs`, `src/cli/doctor.rs`, `tests/security/`, `tests/performance/`.

Profile privacy, diagnostics and performance qualification. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-023-AC01 | Current ACL applies to sources/derived summaries/cache hits; no accidental team-wide promotion | `bf3_023_ac01` |
| BF3-023-AC02 | A new digest and appropriate requalification are required; old self-declared PASS cannot certify it | `bf3_023_ac02` |
| BF3-023-AC03 | Actual bindings and missing capabilities are shown without modifying policy or spending on unapproved calls | `bf3_023_ac03` |
| BF3-023-AC04 | Report actual p95/RSS/assets and all exclusions; auth/durability stay enabled and targets are not fabricated as results | `bf3_023_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-024 — Team release and independent rebuild gate

**Stage:** G2. **Depends on:** BF3-001, BF3-002, BF3-003, BF3-004, BF3-005, BF3-006, BF3-007, BF3-008, BF3-009, BF3-010, BF3-011, BF3-012, BF3-013, BF3-014, BF3-015, BF3-016, BF3-017, BF3-018, BF3-019, BF3-020, BF3-021, BF3-022, BF3-023. **Normative sections:** 2, 23, 24.

**Write scope:** `qualification/`, `tests/e2e/team.rs`, `docs/release-evidence.md`.

Team release and independent rebuild gate. Implement the cited contract behavior with positive and negative assertions.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-024-AC01 | All declared core obligations are demonstrated through owned interfaces without routine vendor unpausing | `bf3_024_ac01` |
| BF3-024-AC02 | Release fails; green exit with zero selected tests cannot satisfy a case | `bf3_024_ac02` |
| BF3-024-AC03 | Cost/time/quality and observation boundaries are reported without cherry-picking or inflated task counts | `bf3_024_ac03` |
| BF3-024-AC04 | Observable contract tests pass or explicit errata remain; prior chat and another proposal are not required | `bf3_024_ac04` |

**Do not:** Do not introduce another scheduler, database authority or CI platform. Do not weaken higher-level authorization, accepted gates or retained lineage.

## BF3-X01 — Prime bounded-tree profile

**Stage:** G3. **Depends on:** BF3-024. **Normative sections:** 10.3, 11, 13.1.

**Write scope:** `src/providers/prime.rs`, `tests/prime/`.

Prime bounded-tree profile. Complete only after the core and new scoped authorization.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-X01-AC01 | No fresh state is claimed until an actual new session/history is observed | `bf3_x01_ac01` |
| BF3-X01-AC02 | Its root/daemon/kernel/children are contained; no uncontrolled resident schedule remains | `bf3_x01_ac02` |
| BF3-X01-AC03 | Parent data/authority/allowance limits apply; no concurrent writer shares the candidate tree | `bf3_x01_ac03` |
| BF3-X01-AC04 | Exactly one declared charge basis is used; missing descendants remain uncertainty | `bf3_x01_ac04` |

**Do not:** Do not make the extension a hidden dependency of G1/G2. Do not inherit a new authoritative task store or bypass the core contract.

## BF3-X02 — OpenJarvis/local-engine component trial

**Stage:** G3. **Depends on:** BF3-024. **Normative sections:** 10.4, 18.

**Write scope:** `spikes/openjarvis/`, `src/providers/local.rs`.

OpenJarvis/local-engine component trial. Complete only after the core and new scoped authorization.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-X02-AC01 | Report compiled/runtime dependency cost; no second task ledger/scheduler is adopted | `bf3_x02_ac01` |
| BF3-X02-AC02 | Reject unless a separately qualified tool loop supplies the worker contract | `bf3_x02_ac02` |
| BF3-X02-AC03 | Typed failures retain usage exposure; no silent model substitution | `bf3_x02_ac03` |
| BF3-X02-AC04 | Policy prohibits transmission unless newly authorized; measurement distinguishes setup/inference/tools | `bf3_x02_ac04` |

**Do not:** Do not make the extension a hidden dependency of G1/G2. Do not inherit a new authoritative task store or bypass the core contract.

## BF3-X03 — Grok Build ACP adapter

**Stage:** G3. **Depends on:** BF3-024. **Normative sections:** 10, 11.

**Write scope:** `src/providers/grok.rs`, `tests/grok/`.

Grok Build ACP adapter. Complete only after the core and new scoped authorization.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-X03-AC01 | Require the actual supported product/version/protocol, not its filename | `bf3_x03_ac01` |
| BF3-X03-AC02 | Use certified noninteractive semantics or a typed block, never guessed keypresses | `bf3_x03_ac02` |
| BF3-X03-AC03 | The core job/step/authority vocabulary remains unchanged | `bf3_x03_ac03` |
| BF3-X03-AC04 | No reroute evades a prohibited action or data restriction | `bf3_x03_ac04` |

**Do not:** Do not make the extension a hidden dependency of G1/G2. Do not inherit a new authoritative task store or bypass the core contract.

## BF3-X04 — Controlled profile learning and warm-state experiments

**Stage:** G3. **Depends on:** BF3-024. **Normative sections:** 18.

**Write scope:** `src/reporting/evaluation.rs`, `evals/`.

Controlled profile learning and warm-state experiments. Complete only after the core and new scoped authorization.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-X04-AC01 | Incumbent delivery is independent; shadow cannot change production selection or publish | `bf3_x04_ac01` |
| BF3-X04-AC02 | Freeze them or explicitly name the changed factor; do not attribute confounded result to model alone | `bf3_x04_ac02` |
| BF3-X04-AC03 | Protected oracle/privacy checks reject it; owner approval/canary required for allowed targets | `bf3_x04_ac03` |
| BF3-X04-AC04 | Retain failures/censoring; do not fabricate survival or universal sample-size sufficiency | `bf3_x04_ac04` |

**Do not:** Do not make the extension a hidden dependency of G1/G2. Do not inherit a new authoritative task store or bypass the core contract.

## BF3-X05 — Thin Slack, Bot and SMS clients

**Stage:** G3. **Depends on:** BF3-024. **Normative sections:** 19, 22.

**Write scope:** `src/api/channels/`, `tests/channels/`.

Thin Slack, Bot and SMS clients. Complete only after the core and new scoped authorization.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-X05-AC01 | Signature/delivery checks reject it and real transport still needs repository authority | `bf3_x05_ac01` |
| BF3-X05-AC02 | Reject mismatch exactly as CLI/web; channel cannot self-approve privilege | `bf3_x05_ac02` |
| BF3-X05-AC03 | Require an actual capability trial; do not infer it from coding-CLI docs | `bf3_x05_ac03` |
| BF3-X05-AC04 | No sensitive disclosure or irreversible action; use authenticated decision links | `bf3_x05_ac04` |

**Do not:** Do not make the extension a hidden dependency of G1/G2. Do not inherit a new authoritative task store or bypass the core contract.

## BF3-X06 — One existing release pipeline

**Stage:** G3. **Depends on:** BF3-024. **Normative sections:** 22.

**Write scope:** `src/delivery/releases.rs`, `tests/releases/`.

One existing release pipeline. Complete only after the core and new scoped authorization.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-X06-AC01 | Invalidate approval and require current subject authorization | `bf3_x06_ac01` |
| BF3-X06-AC02 | No deployment/healthy claim until exact receipt and required observations establish it | `bf3_x06_ac02` |
| BF3-X06-AC03 | Use explicit compatibility/forward recovery; no generic rollback promise | `bf3_x06_ac03` |
| BF3-X06-AC04 | Existing protected pipeline owns them; writer receives only permitted observations | `bf3_x06_ac04` |

**Do not:** Do not make the extension a hidden dependency of G1/G2. Do not inherit a new authoritative task store or bypass the core contract.

## BF3-X07 — One measured forge improvement or replacement

**Stage:** G3. **Depends on:** BF3-024. **Normative sections:** 10.6, 22, 18.

**Write scope:** `src/delivery/extensions/`, `docs/replacement/`.

One measured forge improvement or replacement. Complete only after the core and new scoped authorization.

| Acceptance ID | Required behavior | Test binding |
|---|---|---|
| BF3-X07-AC01 | Patch only the necessary contract with pinned source, notices and upgrade tests | `bf3_x07_ac01` |
| BF3-X07-AC02 | Remote mutation rejects it at the enforcement boundary | `bf3_x07_ac02` |
| BF3-X07-AC03 | Durable work/evidence remains usable without the extension | `bf3_x07_ac03` |
| BF3-X07-AC04 | Judge accepted outcomes and maintenance, not conformance to contingent batching style | `bf3_x07_ac04` |

**Do not:** Do not make the extension a hidden dependency of G1/G2. Do not inherit a new authoritative task store or bypass the core contract.


---

# Appendix E — Runtime acceptance scenarios

These are implementation obligations, all initially `not_run`. Offline specification checks do not change these statuses. Each test binding must exist in the future Rust acceptance target; missing/zero selected tests fail release.

## Q3-001-01 — Strict records

**Owner:** BF3-001 / BF3-001-AC01. **Required for:** G0. **Status:** not run.

**Given** a valid fixture and variants with duplicate keys, extra fields or bad amounts. **When** decode them through the real boundary. **Then** valid data round-trips; ambiguous JSON and out-of-range money fail before any authority is created.

## Q3-001-02 — Dependencies and requirements

**Owner:** BF3-001 / BF3-001-AC02. **Required for:** G0. **Status:** not run.

**Given** a plan with missing, wrong-revision, cyclic or cross-repo dependencies. **When** request activation. **Then** the exact invalid edge and uncovered root requirement are rejected; no task becomes runnable.

## Q3-001-03 — Scope and steps

**Owner:** BF3-001 / BF3-001-AC03. **Required for:** G0. **Status:** not run.

**Given** a two-step task with literal file/directory scopes. **When** test src/a versus src/ab, traversal and an out-of-task step. **Then** segment boundaries and subset checks hold; invalid steps or zero code steps fail.

## Q3-001-04 — Profiles and purpose

**Owner:** BF3-001 / BF3-001-AC04. **Required for:** G0. **Status:** not run.

**Given** planning before implementation tasks and a model job without a profile. **When** validate job role contracts. **Then** mission planning uses a null task reference; every model job requires a profile; human work does not require a fake provider.

## Q3-002-01 — Atomic local mutation

**Owner:** BF3-002 / BF3-002-AC01. **Required for:** G0. **Status:** not run.

**Given** a mutation that creates state, audit and outbox. **When** inject failure before commit. **Then** all three roll back; no successful acknowledgement is emitted.

## Q3-002-02 — Exact command replay

**Owner:** BF3-002 / BF3-002-AC02. **Required for:** G0. **Status:** not run.

**Given** one admitted principal/key/body and a changed-body retry. **When** replay under current read authorization. **Then** the identical request returns the same operation; different bytes conflict and revoked readers receive no old response.

## Q3-002-03 — Immutable history

**Owner:** BF3-002 / BF3-002-AC03. **Required for:** G0. **Status:** not run.

**Given** stored task/profile/candidate/evidence records. **When** attempt in-place identity changes or backwards generations. **Then** database and handler guards reject changes and retain history.

## Q3-002-04 — No I/O under transaction

**Owner:** BF3-002 / BF3-002-AC04. **Required for:** G0. **Status:** not run.

**Given** a fake network call held indefinitely. **When** drive other read/control commands. **Then** the database transaction is already released; unrelated operations stay usable.

## Q3-003-01 — Actor provenance

**Owner:** BF3-003 / BF3-003-AC01. **Required for:** G0. **Status:** not run.

**Given** an observer and a model payload asserting admin/owner. **When** submit paid and authority-changing commands. **Then** server-derived identity controls authorization; the payload cannot impersonate a human.

## Q3-003-02 — Read isolation

**Owner:** BF3-003 / BF3-003-AC02. **Required for:** G0. **Status:** not run.

**Given** two repositories and a revoked user. **When** read status, artifacts, search and reconnect SSE. **Then** every surface enforces current repository access including guessed IDs and old cursors.

## Q3-003-03 — Enrollment

**Owner:** BF3-003 / BF3-003-AC03. **Required for:** G0. **Status:** not run.

**Given** a scoped expiring enrollment token. **When** exchange it twice or after expiry. **Then** one exchange succeeds; stored secret material is protected; no runner credential enters a candidate.

## Q3-003-04 — Provider-independent controls

**Owner:** BF3-003 / BF3-003-AC04. **Required for:** G0. **Status:** not run.

**Given** every provider is disabled. **When** run status, why, pause and operation status through CLI/API. **Then** controls use durable facts with zero model calls and no terminal fallback.

## Q3-004-01 — All-scope admission

**Owner:** BF3-004 / BF3-004-AC01. **Required for:** G0. **Status:** not run.

**Given** two concurrent requests near a shared limit. **When** reserve task/root/mission/owner exposure. **Then** only admissible requests commit; any failure leaves every hold and allocation unchanged.

## Q3-004-02 — Usage replay

**Owner:** BF3-004 / BF3-004-AC02. **Required for:** G0. **Status:** not run.

**Given** duplicate and reordered cumulative/inclusive parent-child reports. **When** settle them. **Then** one leaf charge basis is counted once; unknown final usage retains exposure.

## Q3-004-03 — Late overrun

**Owner:** BF3-004 / BF3-004-AC03. **Required for:** G0. **Status:** not run.

**Given** a completed job bills above its estimate. **When** record the verified bill. **Then** actual overrun is stored, not clipped/rejected, and new admission stops.

## Q3-004-04 — Split allowance

**Owner:** BF3-004 / BF3-004-AC04. **Required for:** G0. **Status:** not run.

**Given** one root with partially used allowance and a rewritten split plan. **When** activate child work or rewind a step. **Then** no new default pool is manufactured; approved first passes and remaining repairs must fit explicit root limits.

## Q3-005-01 — Git/SQL gap

**Owner:** BF3-005 / BF3-005-AC01. **Required for:** G0. **Status:** not run.

**Given** a confirmed metadata push and uncommitted activation. **When** crash then restart. **Then** the revision is inert until the complete graph is atomically adopted; no half graph dispatches.

## Q3-005-02 — Activation replay

**Owner:** BF3-005 / BF3-005-AC02. **Required for:** G0. **Status:** not run.

**Given** the same mission and plan digest. **When** activate twice. **Then** one set of task revisions and dependencies exists.

## Q3-005-03 — Protected metadata

**Owner:** BF3-005 / BF3-005-AC03. **Required for:** G0. **Status:** not run.

**Given** candidate content modifies its own .bulletfarm or CI files. **When** activate or publish control metadata. **Then** only trusted current policy applies; metadata cannot trigger source deployment or force-push.

## Q3-005-04 — PR-only trap

**Owner:** BF3-005 / BF3-005-AC04. **Required for:** G0. **Status:** not run.

**Given** separate code tasks depend on merged output but no intermediate merge is authorized. **When** admit the plan. **Then** it is rejected or explicitly revised to steps/independent work/human checkpoints; PR-ready never impersonates merged.

## Q3-006-01 — Spawn replay

**Owner:** BF3-006 / BF3-006-AC01. **Required for:** G0. **Status:** not run.

**Given** a durable local job ID and process label. **When** crash between spawn and acknowledgement then replay. **Then** the exact existing run is reconciled or blocked; no blind duplicate process is launched.

## Q3-006-02 — Lease timing

**Owner:** BF3-006 / BF3-006-AC02. **Required for:** G0. **Status:** not run.

**Given** a renewal reply delayed beyond the remaining deadline or machine suspend. **When** compute local continuation. **Then** response arrival does not extend authority; the job self-stops conservatively and reports uncertainty.

## Q3-006-03 — Process-tree stop

**Owner:** BF3-006 / BF3-006-AC03. **Required for:** G0. **Status:** not run.

**Given** a job with an owned child and a separate owner job. **When** cancel the first root. **Then** only its recorded tree is stopped; surviving/unreachable descendants remain unconfirmed.

## Q3-006-04 — Framing and backpressure

**Owner:** BF3-006 / BF3-006-AC04. **Required for:** G0. **Status:** not run.

**Given** oversized output, a slow UI and quiet healthy tool. **When** ingest logs and renew leases. **Then** bounded diagnostic handling does not lose terminal state or falsely kill the healthy tool for silence.

## Q3-007-01 — One current writer

**Owner:** BF3-007 / BF3-007-AC01. **Required for:** G0. **Status:** not run.

**Given** two enrolled runners race for one task. **When** claim concurrently. **Then** one authoritative generation wins; the loser consumes no budget or partial claims.

## Q3-007-02 — All-or-none claims

**Owner:** BF3-007 / BF3-007-AC02. **Required for:** G0. **Status:** not run.

**Given** two owners request overlapping literal paths and interface keys. **When** schedule their tasks. **Then** conflicting work waits, disjoint work proceeds and no partial claim deadlock occurs.

## Q3-007-03 — Pending ownership

**Owner:** BF3-007 / BF3-007-AC03. **Required for:** G0. **Status:** not run.

**Given** a writer ends with an admitted unmerged candidate. **When** release its execution slot. **Then** whole-task reservations remain through verification/review without holding compute.

## Q3-007-04 — Stale return

**Owner:** BF3-007 / BF3-007-AC04. **Required for:** G0. **Status:** not run.

**Given** a disconnected old writer returns after reassignment. **When** submit candidate or privileged proposal. **Then** old output remains salvage only; it cannot change current selection or authorize effects.

## Q3-008-01 — Safe object intake

**Owner:** BF3-008 / BF3-008-AC01. **Required for:** G0. **Status:** not run.

**Given** a bundle with bad paths, hooks, replace refs, helpers or truncated bytes. **When** collect through the unprivileged quarantine. **Then** the malformed artifact is rejected/contained; publisher identity executes no candidate-controlled code.

## Q3-008-02 — Immutable candidate

**Owner:** BF3-008 / BF3-008-AC02. **Required for:** G0. **Status:** not run.

**Given** a frozen valid candidate and still-current writer. **When** admit and select it transactionally. **Then** exact base/head/tree, contract, artifact and delivery version are recorded before current-author status is released.

## Q3-008-03 — Post-author delivery

**Owner:** BF3-008 / BF3-008-AC03. **Required for:** G0. **Status:** not run.

**Given** a selected candidate and an exited author. **When** launch independent verification and later authorized publication. **Then** the author need not retain a live lease; current delivery authority remains required.

## Q3-008-04 — Revoked selection

**Owner:** BF3-008 / BF3-008-AC04. **Required for:** G0. **Status:** not run.

**Given** a delayed result/effect for a superseded delivery version. **When** process it. **Then** history can be retained but current state cannot advance; a new candidate needs new evidence.

## Q3-009-01 — Adequacy diagnostic

**Owner:** BF3-009 / BF3-009-AC01. **Required for:** G0. **Status:** not run.

**Given** correct baseline, no-op new feature and retained-test defective variant. **When** run the adopted checks. **Then** baseline passes and wrong behavior fails for its intended reason; inapplicable or infrastructure failures are labeled.

## Q3-009-02 — Trusted producer

**Owner:** BF3-009 / BF3-009-AC02. **Required for:** G0. **Status:** not run.

**Given** a writer prints PASS using the check name. **When** admit the result. **Then** self-report or wrong issuer cannot satisfy the assigned verifier record.

## Q3-009-03 — Discovery completeness

**Owner:** BF3-009 / BF3-009-AC03. **Required for:** G0. **Status:** not run.

**Given** a test filter matches zero tests or skips a required case. **When** aggregate acceptance. **Then** readiness fails despite exit zero; logs and required discovery are checked.

## Q3-009-04 — Stale verification

**Owner:** BF3-009 / BF3-009-AC04. **Required for:** G0. **Status:** not run.

**Given** candidate/policy/recipe changes while a verifier runs. **When** receive its otherwise passing result. **Then** store it only for its original subject; it cannot certify the new selection.

## Q3-010-01 — Lost PR reply

**Owner:** BF3-010 / BF3-010-AC01. **Required for:** G0. **Status:** not run.

**Given** remote create succeeds but its response disappears. **When** recover from sending/unknown. **Then** find the exact PR/head/task marker; do not blindly create another.

## Q3-010-02 — Unknown versus absent

**Owner:** BF3-010 / BF3-010-AC02. **Required for:** G0. **Status:** not run.

**Given** an eventually consistent lookup returns no PR while create may still execute. **When** retry reconciliation. **Then** remain unknown until safe nonapplication or exact success is established; conflicting effect stays blocked.

## Q3-010-03 — CI-only gate

**Owner:** BF3-010 / BF3-010-AC03. **Required for:** G0. **Status:** not run.

**Given** a required check can run only after an authorized branch exists. **When** publish under the provisional-draft grant. **Then** draft opens after local minimum but stays unready; no merge authority appears until full obligations hold.

## Q3-010-04 — Human remote action

**Owner:** BF3-010 / BF3-010-AC04. **Required for:** G0. **Status:** not run.

**Given** a human closes or changes the branch while a queued update exists. **When** reconcile before sending. **Then** do not reopen, overwrite or force-push automatically; surface exact divergence.

## Q3-011-01 — Fresh handoff

**Owner:** BF3-011 / BF3-011-AC01. **Required for:** G1. **Status:** not run.

**Given** an authorized single-step task after planning. **When** run the selected native transport. **Then** a genuine new session executes without routine TUI menus; inherited planner history is not relabeled fresh.

## Q3-011-02 — Typed failures

**Owner:** BF3-011 / BF3-011-AC02. **Required for:** G1. **Status:** not run.

**Given** auth/quota/denial/malformed/partial events and a forbidden method. **When** exercise the adapter contract. **Then** each yields a bounded typed state; Codex unsandboxed shell methods and new authority are not exposed.

## Q3-011-03 — Selected incumbent

**Owner:** BF3-011 / BF3-011-AC03. **Required for:** G1. **Status:** not run.

**Given** an available incumbent and an authorized bounded Prime RPC trial. **When** choose the first execution profile. **Then** one conforming path is used; a failed/over-budget trial falls back without making three platforms mandatory.

## Q3-011-04 — Containment and account

**Owner:** BF3-011 / BF3-011-AC04. **Required for:** G1. **Status:** not run.

**Given** real approved provider credentials and disposable isolation. **When** perform the authorized low-risk live qualification. **Then** actual credential/child/usage boundaries are recorded; unsupported profiles are explicitly ineligible.

## Q3-012-01 — No-network demo

**Owner:** BF3-012 / BF3-012-AC01. **Required for:** G1. **Status:** not run.

**Given** no provider keys or external network. **When** run the fixture through real controller/store. **Then** success and failure scenarios produce honest simulated results, never live acceptance.

## Q3-012-02 — Real draft PR

**Owner:** BF3-012 / BF3-012-AC02. **Required for:** G1. **Status:** not run.

**Given** one explicit authorized task and real qualified executor/forge. **When** run from the custom CLI. **Then** a real independently checked draft appears with actual SHA, evidence and known limitations.

## Q3-012-03 — Failure recovery

**Owner:** BF3-012 / BF3-012-AC03. **Required for:** G1. **Status:** not run.

**Given** the author exits and hub/create-response failure is injected. **When** recover the same task. **Then** candidate survives, ownership stays reserved and one logical PR is observed.

## Q3-012-04 — Boundary receipt

**Owner:** BF3-012 / BF3-012-AC04. **Required for:** G1. **Status:** not run.

**Given** a PR with some final human/integration obligations pending. **When** read status and export receipt. **Then** draft, review-ready, merge-eligible and merged are not conflated; no deployment is claimed.

## Q3-013-01 — Two steps one PR

**Owner:** BF3-013 / BF3-013-AC01. **Required for:** G2. **Status:** not run.

**Given** the two-step CSV fixture and permitted allocations. **When** complete both fresh jobs. **Then** one coherent task reaches a final candidate without an intermediate human merge.

## Q3-013-02 — Cursor compare-and-set

**Owner:** BF3-013 / BF3-013-AC02. **Required for:** G2. **Status:** not run.

**Given** duplicate/wrong-subject checkpoint results. **When** advance from step k. **Then** only matching current cumulative evidence advances the cursor once.

## Q3-013-03 — Suffix invalidation

**Owner:** BF3-013 / BF3-013-AC03. **Required for:** G2. **Status:** not run.

**Given** an earlier accepted checkpoint is changed. **When** rewind and rerun affected work. **Then** all dependent suffix evidence loses eligibility; historical results remain.

## Q3-013-04 — Repair limits

**Owner:** BF3-013 / BF3-013-AC04. **Required for:** G2. **Status:** not run.

**Given** a two-step task with four total writer allocations and max three per step. **When** inject repeated failures, context resets and rewinds. **Then** limits remain cumulative; no hidden third repair or pool reset is admitted.

## Q3-014-01 — Real second family

**Owner:** BF3-014 / BF3-014-AC01. **Required for:** G2. **Status:** not run.

**Given** a different qualified coding-provider family. **When** execute the same bounded task and eligible review. **Then** read-only summarization alone does not pass the second-executor gate.

## Q3-014-02 — Trusted startup

**Owner:** BF3-014 / BF3-014-AC02. **Required for:** G2. **Status:** not run.

**Given** repo/user hooks, MCP or ambient skills are present. **When** start a headless profile. **Then** only approved explicit configuration loads; bare/auth assumptions are actually tested.

## Q3-014-03 — Cross-provider handoff

**Owner:** BF3-014 / BF3-014-AC03. **Required for:** G2. **Status:** not run.

**Given** a failed first provider has a sealed checkpoint. **When** replace it with the eligible second provider. **Then** new session receives exact task/evidence and remaining allowance without policy widening.

## Q3-014-04 — Same conformance

**Owner:** BF3-014 / BF3-014-AC04. **Required for:** G2. **Status:** not run.

**Given** both profiles see denial, cancel, partial output and unknown usage. **When** run the fixture family and authorized live checks. **Then** capability gaps remain visible; neither adapter changes core authority semantics.

## Q3-015-01 — Three planning modes

**Owner:** BF3-015 / BF3-015-AC01. **Required for:** G2. **Status:** not run.

**Given** Fast, Standard and Deep requests. **When** execute their fixed bounded flows. **Then** Standard does not claim blindness; Deep proposals precede cross-exposure; every job is metered.

## Q3-015-02 — Accepted plan executes

**Owner:** BF3-015 / BF3-015-AC02. **Required for:** G2. **Status:** not run.

**Given** bf run within standing scope versus bf plan draft-only. **When** admit a valid plan. **Then** run starts fresh execution automatically; plan never launches a writer.

## Q3-015-03 — Critical disagreement

**Owner:** BF3-015 / BF3-015-AC03. **Required for:** G2. **Status:** not run.

**Given** a missing requirement or material unresolved finding. **When** synthesize the plan. **Then** owner/experiment path is explicit; model votes cannot override missing gates or new authority.

## Q3-015-04 — Just-in-time packets

**Owner:** BF3-015 / BF3-015-AC04. **Required for:** G2. **Status:** not run.

**Given** a task with earlier failures and substantial repo history. **When** construct a worker input. **Then** only authorized exact source/decisions/relevant failure facts enter the compact packet; not every transcript.

## Q3-016-01 — Risk by actual diff

**Owner:** BF3-016 / BF3-016-AC01. **Required for:** G2. **Status:** not run.

**Given** a nominally low-risk change modifies auth/CI/permissions. **When** admit the changed candidate. **Then** risk rises to policy minimum; writer cannot select lighter acceptance.

## Q3-016-02 — Independent review

**Owner:** BF3-016 / BF3-016-AC02. **Required for:** G2. **Status:** not run.

**Given** the writer submits a persuasive summary and counterfeit reviewer field. **When** request review. **Then** a separate qualified job reviews exact requirements/diff/evidence; actor spoofing fails.

## Q3-016-03 — Aggregate status

**Owner:** BF3-016 / BF3-016-AC03. **Required for:** G2. **Status:** not run.

**Given** native checks are skipped/neutral or same-name from wrong issuer. **When** compute bulletfarm/acceptance. **Then** only complete trusted matching evidence can succeed.

## Q3-016-04 — Review revision

**Owner:** BF3-016 / BF3-016-AC04. **Required for:** G2. **Status:** not run.

**Given** a reviewer passes one candidate then new commits arrive. **When** compute readiness. **Then** old review remains historical and cannot authorize changed code without required recheck.

## Q3-017-01 — Takeover checkpoint

**Owner:** BF3-017 / BF3-017-AC01. **Required for:** G2. **Status:** not run.

**Given** an active or unreachable writer with last sealed work. **When** a person takes the task. **Then** old authority is revoked; a new private clone uses only actually captured state.

## Q3-017-02 — Offline human

**Owner:** BF3-017 / BF3-017-AC02. **Required for:** G2. **Status:** not run.

**Given** a person closes their laptop under a held task reservation. **When** lease/connection observations expire. **Then** no bot silently takes over and no editor is killed; submission revalidates current assignment.

## Q3-017-03 — Adopt and respecify

**Owner:** BF3-017 / BF3-017-AC03. **Required for:** G2. **Status:** not run.

**Given** ad-hoc human work and a previously failed root. **When** register or split/revise it. **Then** provenance is retrospective and lifetime cost/allowance/history remain intact.

## Q3-017-04 — Withdraw unresolved PR

**Owner:** BF3-017 / BF3-017-AC04. **Required for:** G2. **Status:** not run.

**Given** a task has held claims and an old remotely mergeable change. **When** withdraw. **Then** claims release only after relevant effects and remote eligibility are settled; no claimed undo of merged code.

## Q3-018-01 — Owner autonomy

**Owner:** BF3-018 / BF3-018-AC01. **Required for:** G2. **Status:** not run.

**Given** two owners with allowed priorities/profiles. **When** schedule independent domain tasks. **Then** preferences hold within shared policy; idle capacity never authorizes stealing another roadmap/account.

## Q3-018-02 — Queue saturation

**Owner:** BF3-018 / BF3-018-AC02. **Required for:** G2. **Status:** not run.

**Given** verifier or PR backlog reaches its watermark. **When** offer more writer tasks. **Then** new writing slows while verification/repair/reconciliation continue.

## Q3-018-03 — External work

**Owner:** BF3-018 / BF3-018-AC03. **Required for:** G2. **Status:** not run.

**Given** an overlapping human PR and invisible unpushed local changes. **When** refresh team status. **Then** observed overlap is shown with timestamp; invisible work is not claimed controlled.

## Q3-018-04 — Fair progress

**Owner:** BF3-018 / BF3-018-AC04. **Required for:** G2. **Status:** not run.

**Given** one owner continuously submits high volume and another eligible task ages. **When** run deterministic selection. **Then** the configured fairness/aging policy gives the second owner progress and explainable ordering.

## Q3-019-01 — One truth

**Owner:** BF3-019 / BF3-019-AC01. **Required for:** G2. **Status:** not run.

**Given** CLI, two browsers and lead prose disagree. **When** render current state. **Then** deterministic committed projections win; prose cannot create a successful status.

## Q3-019-02 — Stream recovery

**Owner:** BF3-019 / BF3-019-AC02. **Required for:** G2. **Status:** not run.

**Given** event loss, duplicate events and an expired cursor. **When** reconnect. **Then** replay or snapshot resync repairs state without duplicate commands or leaked repository data.

## Q3-019-03 — Decision freshness

**Owner:** BF3-019 / BF3-019-AC03. **Required for:** G2. **Status:** not run.

**Given** an old approval button is clicked after subject/version change. **When** process decision. **Then** 409/current-subject rejection prevents changed action authorization.

## Q3-019-04 — Accessible operators

**Owner:** BF3-019 / BF3-019-AC04. **Required for:** G2. **Status:** not run.

**Given** keyboard-only user, large logs and a shared provider incident. **When** navigate the work view. **Then** focus, labels and bounded rendering work; one incident replaces repetitive per-task alerts.

## Q3-020-01 — Current composition

**Owner:** BF3-020 / BF3-020-AC01. **Required for:** G2. **Status:** not run.

**Given** two PRs pass alone but fail together or target changes. **When** enter protected integration. **Then** current combined checks run and block incompatible composition; PR-head PASS is insufficient.

## Q3-020-02 — Actual merge identity

**Owner:** BF3-020 / BF3-020-AC02. **Required for:** G2. **Status:** not run.

**Given** squash/rebase creates a different OID. **When** reconcile merge. **Then** candidate/tested-tree/actual-merge relation is established or explicitly unknown.

## Q3-020-03 — Unsupported forge

**Owner:** BF3-020 / BF3-020-AC03. **Required for:** G2. **Status:** not run.

**Given** the endpoint lacks exact-subject/check-issuer protection. **When** request automatic merge. **Then** stop at the existing protected manual path; no unsafe scripted fallback is invented.

## Q3-020-04 — Mission boundary

**Owner:** BF3-020 / BF3-020-AC04. **Required for:** G2. **Status:** not run.

**Given** all steps pass but assembled root behavior or required artifact is missing. **When** evaluate completion. **Then** mission stays incomplete; production-observed requests fail early if release adapter is unavailable.

## Q3-021-01 — Ordinary boot

**Owner:** BF3-021 / BF3-021-AC01. **Required for:** G2. **Status:** not run.

**Given** active authors and selected sealed candidates exist. **When** restart the single hub. **Then** new authority revokes authors, retains candidates and revalidates delivery; no automatic duplicate create.

## Q3-021-02 — Disaster clone

**Owner:** BF3-021 / BF3-021-AC02. **Required for:** G2. **Status:** not run.

**Given** old publisher may still run and backup omits later revocations. **When** restore. **Then** remain paused until old credentials/authority are disabled and remote state/artifacts are reconciled.

## Q3-021-03 — Storage failure

**Owner:** BF3-021 / BF3-021-AC03. **Required for:** G2. **Status:** not run.

**Given** disk full or truncated required evidence occurs. **When** acknowledge or use artifact. **Then** no false durable acknowledgement or accepted incomplete proof appears.

## Q3-021-04 — Exact cleanup

**Owner:** BF3-021 / BF3-021-AC04. **Required for:** G2. **Status:** not run.

**Given** an old-looking directory belongs to a live or referenced job. **When** run retention cleanup. **Then** it is retained; deletion needs exact identity, terminal/reconciled status and safe reference checks.

## Q3-022-01 — Non-live export

**Owner:** BF3-022 / BF3-022-AC01. **Required for:** G2. **Status:** not run.

**Given** mission has contracts, failures, current candidate and unknown cost. **When** export/import. **Then** portable history preserves uncertainty and excludes credentials/live grants; import starts paused.

## Q3-022-02 — Root metrics

**Owner:** BF3-022 / BF3-022-AC02. **Required for:** G2. **Status:** not run.

**Given** one root is split into many steps/PRs. **When** report throughput and cost. **Then** root delivered value is not multiplied; scope-aggregate money is not double-counted.

## Q3-022-03 — Human timing

**Owner:** BF3-022 / BF3-022-AC03. **Required for:** G2. **Status:** not run.

**Given** human duration is missing or self-reported. **When** report economics. **Then** quality labels remain and missing active minutes are not zero.

## Q3-022-04 — Ability to leave

**Owner:** BF3-022 / BF3-022-AC04. **Required for:** G2. **Status:** not run.

**Given** the coordinator is disabled after export. **When** a human or alternative executor continues. **Then** intent, selected code and check definitions are intelligible without native provider sessions.

## Q3-023-01 — Current scoped memory

**Owner:** BF3-023 / BF3-023-AC01. **Required for:** G2. **Status:** not run.

**Given** private summary/cache exists before membership revocation. **When** retrieve for a new job or channel. **Then** current ACL applies to sources/derived summaries/cache hits; no accidental team-wide promotion.

## Q3-023-02 — Immutable profile drift

**Owner:** BF3-023 / BF3-023-AC02. **Required for:** G2. **Status:** not run.

**Given** harness, tools, memory or environment changes. **When** admit a model job. **Then** a new digest and appropriate requalification are required; old self-declared PASS cannot certify it.

## Q3-023-03 — Read-only diagnostics

**Owner:** BF3-023 / BF3-023-AC03. **Required for:** G2. **Status:** not run.

**Given** provider/isolation/forge facts are incomplete. **When** run doctor. **Then** actual bindings and missing capabilities are shown without modifying policy or spending on unapproved calls.

## Q3-023-04 — Measured overhead

**Owner:** BF3-023 / BF3-023-AC04. **Required for:** G2. **Status:** not run.

**Given** the declared reference host/load is available. **When** run authorized local benchmark. **Then** report actual p95/RSS/assets and all exclusions; auth/durability stay enabled and targets are not fabricated as results.

## Q3-024-01 — Full two-owner path

**Owner:** BF3-024 / BF3-024-AC01. **Required for:** G2. **Status:** not run.

**Given** two owners/runners/repos and two coding families. **When** complete independent, overlapping and two-step work. **Then** all declared core obligations are demonstrated through owned interfaces without routine vendor unpausing.

## Q3-024-02 — No empty test success

**Owner:** BF3-024 / BF3-024-AC02. **Required for:** G2. **Status:** not run.

**Given** an acceptance selector or named test is missing. **When** run release inventory and suite. **Then** release fails; green exit with zero selected tests cannot satisfy a case.

## Q3-024-03 — Honest pilot

**Owner:** BF3-024 / BF3-024-AC03. **Required for:** G2. **Status:** not run.

**Given** representative matched workloads include failures and safe blocks. **When** publish the pilot report. **Then** cost/time/quality and observation boundaries are reported without cherry-picking or inflated task counts.

## Q3-024-04 — Fresh agent rebuild

**Owner:** BF3-024 / BF3-024-AC04. **Required for:** G2. **Status:** not run.

**Given** a separate agent has only this kit, repo and authorized environment facts. **When** rebuild G1 then G2 under an independent reviewer. **Then** observable contract tests pass or explicit errata remain; prior chat and another proposal are not required.

## Q3-X01-01 — Fresh-session veto

**Owner:** BF3-X01 / BF3-X01-AC01. **Required for:** G3. **Status:** not run.

**Given** Prime returns success with cancelled=true. **When** prepare a fresh job. **Then** no fresh state is claimed until an actual new session/history is observed.

## Q3-X01-02 — Resident schedules

**Owner:** BF3-X01 / BF3-X01-AC02. **Required for:** G3. **Status:** not run.

**Given** a runtime can survive RPC stdin closure. **When** cancel the exact job. **Then** its root/daemon/kernel/children are contained; no uncontrolled resident schedule remains.

## Q3-X01-03 — Child authority

**Owner:** BF3-X01 / BF3-X01-AC03. **Required for:** G3. **Status:** not run.

**Given** a child requests another provider or candidate write. **When** admit nested work. **Then** parent data/authority/allowance limits apply; no concurrent writer shares the candidate tree.

## Q3-X01-04 — Tree accounting

**Owner:** BF3-X01 / BF3-X01-AC04. **Required for:** G3. **Status:** not run.

**Given** inclusive parent and child reports arrive duplicated/out of order. **When** settle total cost. **Then** exactly one declared charge basis is used; missing descendants remain uncertainty.

## Q3-X02-01 — Narrow reuse

**Owner:** BF3-X02 / BF3-X02-AC01. **Required for:** G3. **Status:** not run.

**Given** pinned engine/core sources and their dependencies. **When** build an isolated component spike. **Then** report compiled/runtime dependency cost; no second task ledger/scheduler is adopted.

## Q3-X02-02 — Endpoint not harness

**Owner:** BF3-X02 / BF3-X02-AC02. **Required for:** G3. **Status:** not run.

**Given** only an inference endpoint exists. **When** request code execution. **Then** reject unless a separately qualified tool loop supplies the worker contract.

## Q3-X02-03 — Local failure

**Owner:** BF3-X02 / BF3-X02-AC03. **Required for:** G3. **Status:** not run.

**Given** OOM, truncation, timeout or unsupported tools occur. **When** execute the local profile. **Then** typed failures retain usage exposure; no silent model substitution.

## Q3-X02-04 — Privacy fallback

**Owner:** BF3-X02 / BF3-X02-AC04. **Required for:** G3. **Status:** not run.

**Given** local-only source would need cloud fallback. **When** choose a route. **Then** policy prohibits transmission unless newly authorized; measurement distinguishes setup/inference/tools.

## Q3-X03-01 — Official identity

**Owner:** BF3-X03 / BF3-X03-AC01. **Required for:** G3. **Status:** not run.

**Given** a binary called grok with unknown provenance. **When** qualify it. **Then** require the actual supported product/version/protocol, not its filename.

## Q3-X03-02 — Plan review

**Owner:** BF3-X03 / BF3-X03-AC02. **Required for:** G3. **Status:** not run.

**Given** always-approve leaves native plan review pending. **When** execute accepted work. **Then** use certified noninteractive semantics or a typed block, never guessed keypresses.

## Q3-X03-03 — Shared contracts

**Owner:** BF3-X03 / BF3-X03-AC03. **Required for:** G3. **Status:** not run.

**Given** Grok runs an eligible code task and failure fixtures. **When** normalize events. **Then** the core job/step/authority vocabulary remains unchanged.

## Q3-X03-04 — Safe fallback

**Owner:** BF3-X03 / BF3-X03-AC04. **Required for:** G3. **Status:** not run.

**Given** a Grok denial differs from outage/quota. **When** consider another provider. **Then** no reroute evades a prohibited action or data restriction.

## Q3-X04-01 — Shadow isolation

**Owner:** BF3-X04 / BF3-X04-AC01. **Required for:** G3. **Status:** not run.

**Given** a challenger produces a better-looking candidate. **When** finish a matched trial. **Then** incumbent delivery is independent; shadow cannot change production selection or publish.

## Q3-X04-02 — Comparable treatment

**Owner:** BF3-X04 / BF3-X04-AC02. **Required for:** G3. **Status:** not run.

**Given** different warm state, base, memory or gates exist. **When** compare profiles. **Then** freeze them or explicitly name the changed factor; do not attribute confounded result to model alone.

## Q3-X04-03 — Protected promotion

**Owner:** BF3-X04 / BF3-X04-AC03. **Required for:** G3. **Status:** not run.

**Given** a learned skill attempts to modify gates or contains private source. **When** propose promotion. **Then** protected oracle/privacy checks reject it; owner approval/canary required for allowed targets.

## Q3-X04-04 — Uncertain result

**Owner:** BF3-X04 / BF3-X04-AC04. **Required for:** G3. **Status:** not run.

**Given** small sample, failed tasks and undeployed shadow outcomes. **When** report and decide. **Then** retain failures/censoring; do not fabricate survival or universal sample-size sufficiency.

## Q3-X05-01 — Authenticated transport

**Owner:** BF3-X05 / BF3-X05-AC01. **Required for:** G3. **Status:** not run.

**Given** forged/replayed Slack payload. **When** submit mission command. **Then** signature/delivery checks reject it and real transport still needs repository authority.

## Q3-X05-02 — Stale decision

**Owner:** BF3-X05 / BF3-X05-AC02. **Required for:** G3. **Status:** not run.

**Given** a channel approval references an old artifact/version. **When** apply it. **Then** reject mismatch exactly as CLI/web; channel cannot self-approve privilege.

## Q3-X05-03 — Bot capability

**Owner:** BF3-X05 / BF3-X05-AC03. **Required for:** G3. **Status:** not run.

**Given** Grok Bot interface has not been demonstrated. **When** enable connector. **Then** require an actual capability trial; do not infer it from coding-CLI docs.

## Q3-X05-04 — SMS restraint

**Owner:** BF3-X05 / BF3-X05-AC04. **Required for:** G3. **Status:** not run.

**Given** an ambiguous yes or source-containing notification. **When** process it. **Then** no sensitive disclosure or irreversible action; use authenticated decision links.

## Q3-X06-01 — Artifact approval

**Owner:** BF3-X06 / BF3-X06-AC01. **Required for:** G3. **Status:** not run.

**Given** the approved artifact/environment/config changes. **When** promote. **Then** invalidate approval and require current subject authorization.

## Q3-X06-02 — Actual deployment

**Owner:** BF3-X06 / BF3-X06-AC02. **Required for:** G3. **Status:** not run.

**Given** pipeline trigger succeeds but observed running version differs. **When** evaluate goal. **Then** no deployment/healthy claim until exact receipt and required observations establish it.

## Q3-X06-03 — Irreversible migration

**Owner:** BF3-X06 / BF3-X06-AC03. **Required for:** G3. **Status:** not run.

**Given** a binary rollback would leave incompatible data. **When** recover. **Then** use explicit compatibility/forward recovery; no generic rollback promise.

## Q3-X06-04 — Secret custody

**Owner:** BF3-X06 / BF3-X06-AC04. **Required for:** G3. **Status:** not run.

**Given** a code worker requests signing/deployment credentials. **When** execute release handoff. **Then** existing protected pipeline owns them; writer receives only permitted observations.

## Q3-X07-01 — Minimal patch

**Owner:** BF3-X07 / BF3-X07-AC01. **Required for:** G3. **Status:** not run.

**Given** a measured missing mutation-boundary check exists. **When** propose upstream/local change. **Then** patch only the necessary contract with pinned source, notices and upgrade tests.

## Q3-X07-02 — Server fence

**Owner:** BF3-X07 / BF3-X07-AC02. **Required for:** G3. **Status:** not run.

**Given** a stale generation or changed expected head reaches the forge. **When** apply the qualified extension. **Then** remote mutation rejects it at the enforcement boundary.

## Q3-X07-03 — Compatibility

**Owner:** BF3-X07 / BF3-X07-AC03. **Required for:** G3. **Status:** not run.

**Given** extension is disabled or coordinator replaced. **When** use normal Git/protected PR workflow. **Then** durable work/evidence remains usable without the extension.

## Q3-X07-04 — Equal-outcome reversal

**Owner:** BF3-X07 / BF3-X07-AC04. **Required for:** G3. **Status:** not run.

**Given** another architecture meets the same test boundary with less burden. **When** review replacement. **Then** judge accepted outcomes and maintenance, not conformance to contingent batching style.


---

# Appendix F — Requirement preservation and deliberate changes

The ten current artifacts and nine earlier sources are archived byte-for-byte with hashes. This map preserves the material needs and their acceptance obligations. It does not claim every earlier clause or test remains unchanged: contradictory conventions are explicitly superseded by the selected 3.0 contract. Archived BF/BF2/BF-R identifiers are provenance, not current dispatch IDs.

| ID | Requirement | Source | Final treatment | Implementation |
|---|---|---|---|---|
| KEEP-01 | Personal/domain ownership | Alton §§1,12; A–E | Retain; §§1,4,17 | BF3-017, BF3-018 |
| KEEP-02 | One owned CLI/web/lead | Original request; A–E | Retain; §§3,9,19 | BF3-003, BF3-015, BF3-019 |
| KEEP-03 | No routine provider approval babysitting | Original request; A–E | Retain; §§10–11 | BF3-011, BF3-014 |
| KEEP-04 | Accepted plan starts fresh execution | Original request; A–E | Retain and test actual session identity; §§9–11 | BF3-011, BF3-015 |
| KEEP-05 | Git-versioned immutable task intent | A–E; earlier Verified Changes | Retain; §§6–7 | BF3-001, BF3-005 |
| KEEP-06 | No absent-file dependency success | Alton §5 alternative; A–E correction | Explicit state replaces absence convention; §§6–7 | BF3-001, BF3-005 |
| KEEP-07 | Small work packets for lighter models | Original request; Alton; E | Add bounded linear execution steps inside coherent task; §8 | BF3-013 |
| KEEP-08 | Coherent PRs rather than one PR per TODO | A–E | Retain, separate worker and review unit; §§8,15 | BF3-013, BF3-020 |
| KEEP-09 | Bounded cross-frontier challenge | Original request; A–E | Retain fixed Fast/Standard/Deep; §9 | BF3-015 |
| KEEP-10 | Questions/findings/decisions replace shared chat authority | Earlier Core/One Lead; A–E | Retain; §§5,9,19 | BF3-015 |
| KEEP-11 | Shared allocation across owners/runners | A–E; Alton federation alternative | One hub; independent factories deferred; §§2–4,13 | BF3-007, BF3-018 |
| KEEP-12 | One writer and retained pending-change reservation | Core; alignment; A–E | Retain distinct lifetimes; §12 | BF3-007, BF3-008 |
| KEEP-13 | Author may stop before verification/publication | Alignment; A–E | Retain selected-candidate delivery authority; §12 | BF3-008, BF3-010 |
| KEEP-14 | Gate adequacy before affected-class autonomy | Alton §13; A–E | Retain with test-preserving defect reintroduction; §15 | BF3-009 |
| KEEP-15 | Independent exact-subject verification | A–E | Retain and reject skipped/empty/wrong issuer; §15 | BF3-009, BF3-016 |
| KEEP-16 | Current combined integration and mission check | A–E | Retain; §15 | BF3-020 |
| KEEP-17 | Safe bootstrap where CI requires a pushed candidate | C; B different readiness convention | Explicit draft_open before review_ready; §15 | BF3-010, BF3-016 |
| KEEP-18 | Durable intent before spawn; runner deduplication | A–E | Retain; §§12–14 | BF3-006, BF3-007 |
| KEEP-19 | Unknown external effects are not failed effects | A–E | Retain current authorization + reconciliation; §16 | BF3-010, BF3-021 |
| KEEP-20 | Human takeover/adoption/escalation drain | Alton §10; A–E | Retain no fake human heartbeat; §17 | BF3-017 |
| KEEP-21 | Revision/split preserves history and budgets | Alton reset alternative; A–E | Retain shared root history; explicitly allocate N+2; §§8,13,17 | BF3-004, BF3-013 |
| KEEP-22 | Whole-outcome cost and uncertainty | Original request; A–E | Retain cost once with multiple enforcing scopes; §13 | BF3-004, BF3-022 |
| KEEP-23 | Fairness and downstream backpressure | Earlier Core/Delivery; A–E | Retain; §13 | BF3-018 |
| KEEP-24 | Day/night headroom and honest reports | Alton blocks; A–E | Retain policies, not mandatory branch lifetimes; §§13,19,21 | BF3-018, BF3-019 |
| KEEP-25 | Quiet long healthy tools are not killed for silence | Original workload; A–E | Retain job-specific deadlines; §14 | BF3-006, BF3-011 |
| KEEP-26 | Fresh sessions with safe setup/cache reuse | A–E; Alton warm-shift alternative | Retain one task/step boundary; warm experiment later; §§8,14,18 | BF3-013, BF3-X04 |
| KEEP-27 | Prime practical executor candidate | A; B; Alton | Qualify early at incumbent seam, no launch dependency; §10 | BF3-011, BF3-X01 |
| KEEP-28 | OpenJarvis local inference and Rust reuse | A/B | Narrow engine/core trial, not new platform; §10 | BF3-X02 |
| KEEP-29 | QM personal/shared context and human-only controls | A/B | Retain scope rules, optional client only; §§4,10,18 | BF3-023, BF3-X05 |
| KEEP-30 | Complete immutable execution profiles | A/B | Direct Job binding in clean build; §10 | BF3-001, BF3-011, BF3-023 |
| KEEP-31 | Children contained, metered and not team allocators | A/B | Zero by default; bounded qualified tree later; §§10–11,13 | BF3-X01 |
| KEEP-32 | Private notes are not automatically shared memory | QM-informed A/B | Current audience/source ACL before cache and promotion; §18 | BF3-023, BF3-X04 |
| KEEP-33 | Shipping distinct from controlled learning | Alton §11; A–E | Retain incumbent; no publishing shadow authority; §18 | BF3-X04 |
| KEEP-34 | Difficult research permits negative artifact results | Original research direction; A–E | Retain separate accepted_artifact outcome; §9 | BF3-001, BF3-015 |
| KEEP-35 | Existing QA/production pipeline and immutable promotion | Original request; A–E | Retain separate certified extension; §22 | BF3-X06 |
| KEEP-36 | Slack, Grok Bot and SMS | Original request; A–E | Retain thin clients, actual capability test; §22 | BF3-X03, BF3-X05 |
| KEEP-37 | Owned forge source/parity advantage | Original request; A–E | Behavioral qualification then one measured extension; §§15,22 | BF3-020, BF3-X07 |
| KEEP-38 | Replace entire machinery, preserve durable assets | Alton §2; A–E | Portable paused import and replacement exercise; §§18,23 | BF3-022, BF3-X07 |
| KEEP-39 | No hidden earlier-chat or companion dependencies | Latest user; A–E | One new BF3 package with complete schemas/examples/tests; §§0,23; appendices | BF3-001, BF3-024 |
| KEEP-40 | No premature services/custom Git/DSL | Original constraint; A–E | Retain one package/one DB, external harness languages allowed; §§2–3 | BF3-001, BF3-024 |
| KEEP-41 | Score separate from observed reliability | All full proposals | Retain honest unrun runtime cases; §24; review | BF3-024 |
| KEEP-42 | Pause/stop/restart consistency | A–E differ | Select hold-pause and conservative new-boot authority; §§12,16 | BF3-021 |
| KEEP-43 | Actual test selection, not zero-test success | D guide; rebuilt precision requirement | Mandatory expected-test inventory and nonzero assertion counts; §23; agent guide | BF3-024 |


---

# Appendix G — Sources, provenance and evidence limits

The attached plans are the primary basis of comparison. A–E below identify their exact current versions; new selections in Part II are reviewer decisions, not claims of mutual agreement or implemented capability. The common score concerns written fit and build precision, not measured upstream coding performance.

## Current design sources

| Label | File | Role |
|---|---|---|
| A | `BULLETFARM_HARNESS_INFORMED_FINAL_SPEC(1).md` | Harness-informed baseline 2.1 |
| A-brief | `BULLETFARM_DECISION_BRIEF(1).md` | Decision brief, Markdown |
| A-brief-PDF | `BULLETFARM_DECISION_BRIEF(1).pdf` | Same decision content; six-page representation |
| B | `BULLETFARM_FINAL_PROPOSAL_AND_SPEC(1).pdf` | 84-page harness-integrated specification |
| C | `BULLETFARM_FINAL_PROPOSAL_AND_ENGINEERING_SPEC(1).md` | Human-owned build contract |
| D | `BULLETFARM_FINAL_VISION_AND_REBUILD_SPEC(1).md` | Three-crate rebuild baseline 2.0 |
| D-backlog | `BACKLOG(1)(1).json` | 24 core + 6 extensions, 120 acceptance clauses |
| D-guide | `START_HERE(1)(1).md` | D’s matching first-agent instructions |
| E | `BULLETFARM_VISION_AND_REBUILD_SPEC(1).md` | Ordered-step rebuild edition |
| E-guide | `BULLETFARM_START_HERE(1).md` | E’s matching first-agent instructions |

The matching earlier B PDF was byte-identical and its complete earlier Markdown/rebuild ZIP was available for inspection. No similarly named schemas from B were silently treated as A/C/D/E’s missing companions. Hashes of all nineteen archived inputs are in `review/SOURCE_MANIFEST.json`.

## Earlier requirements retained

Alton’s *Shared vision: agent work*, the five earlier distinct architecture families (Core, Verified Changes, One Lead, Engineering Foreman, Verified Delivery), the two earlier JSON backlogs and the alignment brief are retained in `archive/legacy/`. Their repeated filename suffixes are not independent architecture votes. Their decisive requirements are mapped in Appendix F. Source self-scores are historical judgments and are not reused as measurements.

## Targeted external verification

Primary documents below were rechecked on 16 September 2026 for narrow integration claims. Selection of a current documentation page does not establish installed-version compatibility. The previously studied upstream snapshots remain provenance anchors; a released binary/image digest must be separately qualified. No private implementation source tree, live coding comparison, upstream test suite, OS attack test, runtime benchmark or deployment was executed. No remote fork or branch was created.

### W1. OpenAI — Codex App Server

Current primary documentation rechecked. New threads, lifecycle, method allowlisting and the outside-thread-sandbox shell method. Not an installed-binary test.

[OpenAI — Codex App Server](https://developers.openai.com/codex/app-server)

### W2. Anthropic — Run Claude Code programmatically

Current primary documentation rechecked. Structured headless output, trusted startup and bare/auth differences. Not a determination of the team’s account entitlement.

[Anthropic — Run Claude Code programmatically](https://code.claude.com/docs/en/headless)

### W3. SQLite — Write-ahead logging

Local/same-host WAL, one writer, durability and documented WAL-reset fix. In-memory validation here does not qualify the linked production runtime.

[SQLite — Write-ahead logging](https://sqlite.org/wal.html)

### W4. Git — update-ref

Expected-old-value reference updates exist. They are not the complete task/claim/budget protocol.

[Git — update-ref](https://git-scm.com/docs/git-update-ref)

### W5. GitHub — Protected branches

Expected status issuer and permitted native status conclusions; BulletFarm’s acceptance aggregate is intentionally stricter.

[GitHub — Protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)

### W6. Prime Agent — pinned RPC contract

Selected primary protocol ranges revisited, including acknowledged but cancelled new-session switch. Runtime not installed or qualified.

[Prime Agent — pinned RPC contract](https://github.com/PrimeIntellect-ai/prime-agent/blob/66abc2a604fc42a220292a1ca4cf33ee60cb5733/packages/coding-agent/docs/rpc.md)

### W7. OpenJarvis — pinned engine trait

Inference boundary inspected; not a complete coding-job authority/cancellation interface.

[OpenJarvis — pinned engine trait](https://raw.githubusercontent.com/open-jarvis/OpenJarvis/a6b22bac1791feebf4688789d1aacb70bf0dd8fd/rust/crates/openjarvis-engine/src/traits.rs)

### W8. OpenJarvis — pinned Rust workspace

Seventeen workspace crates; do not characterize as Python-only or infer the entire default product is one Rust binary.

[OpenJarvis — pinned Rust workspace](https://raw.githubusercontent.com/open-jarvis/OpenJarvis/a6b22bac1791feebf4688789d1aacb70bf0dd8fd/rust/Cargo.toml)

### W9. QM — pinned security policy

Selected primary security documentation rechecked; useful authority boundaries and explicit limitations, not a full security audit.

[QM — pinned security policy](https://github.com/yc-software/qm/blob/ec86d8b603a6cc07b21aa4d55359b88c425b61ef/SECURITY.md)

### W10. GitHub — Managing a merge queue

Combined integration subjects and required merge_group workflow event. Custom-forge equivalence remains to be tested.

[GitHub — Managing a merge queue](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue)

### W11. xAI — Grok Build Plan Mode

Plan review and tool approval are distinct. Deployed binary behavior still needs conformance.

[xAI — Grok Build Plan Mode](https://docs.x.ai/build/features/plan-mode)

### W12. Anthropic — Securely deploying AI agents

External credential boundary and sandboxing principles, not proof of any actual BulletFarm deployment.

[Anthropic — Securely deploying AI agents](https://code.claude.com/docs/en/agent-sdk/secure-deployment)

## YC discussion and limitations

The supplied harness-informed A/B documents report a previous automatically transcribed mirror review of the YC video `n9xKblqyQ28`. This adjudication retains its qualitative motivation—evaluate the execution system, not only a model name—and checks consequential engineering details against primary sources above. It does not claim a new direct audiovisual viewing or independent reproduction of any video benchmark. No score depends on transferring a demonstration’s multiplier to BulletFarm.

Root licenses, vendored components, model weights, service terms and redistribution notices must be checked for the exact component before copying code. No upstream source or font files are distributed in this new kit. Source-derived observations, design choices and unrun deployment obligations remain distinct.

## What the measurements mean

Package validation executes selected schema/semantic-reference and in-memory SQL checks. Application acceptance scenarios remain marked `not_run`. The reference Python code is deliberately small and cannot replace Rust runtime tests. No percent design score is a percent reliability claim. PDF rendering/layout checks establish document usability, not software correctness.


---

# Appendix H — Final detailed rubric and residual work

**Final design score: 94.38/100; rounded 94.4.** Same twelve criteria and weights as A–E. This is a written-contract assessment with several points of subjective uncertainty, not measured performance, security or implementation completeness. A 0.76-point difference from E is not a statistically established product improvement.

## People and product fit — 96/100; weight 6%

**What earns credit:** Keeps personal/domain objectives and one useful product while sharing authority and evidence.

**Remaining deduction:** Real owner/engineer ergonomics and domain-change exceptions are unobserved.

**Evidence needed:** Two people steer and take over work, identify the next accountable actor and understand delivery limits without vendor logs.

## MVP simplicity and sequence — 93/100; weight 12%

**What earns credit:** One package, one store, one selected transport each, capability-gated stages and no hidden mandatory platform.

**Remaining deduction:** Isolated execution and safe delivery are still substantial work; ordered steps add a bounded mechanism.

**Evidence needed:** A clean G1 install reaches a real verified draft without optional providers, training, extra services or a framework rewrite.

## Fresh-agent build precision — 94/100; weight 12%

**What earns credit:** Exact payload grammar, schema/examples/SQL, package/test linkage, supplied companions and canonical generated appendices.

**Remaining deduction:** The independent full Rust rebuild has not been performed; deployment facts require real bindings.

**Evidence needed:** A fresh agent builds G1/G2 from this kit and an independent reviewer records all invented decisions as defects.

## Task decomposition and planning — 95/100; weight 10%

**What earns credit:** Coherent tasks contain one-to-eight linear worker steps, meaningful cumulative checkpoints, finite repair and explicit research.

**Remaining deduction:** Schemas do not establish semantic completeness or optimal task size; not every problem splits well.

**Evidence needed:** Two-step completion, suffix invalidation and a bounded negative research result; compare fragmentation cost on actual work.

## Unattended execution — 95/100; weight 8%

**What earns credit:** Fresh actual sessions, qualified supported interfaces, typed failures and deterministic stop/status.

**Remaining deduction:** Provider authentication, refusal, incomplete telemetry and genuine product decisions remain external constraints.

**Evidence needed:** Two execution-capable provider families pass fresh/handoff/denial/long-tool/termination fixtures without routine TUI actions.

## Durability and recovery — 95/100; weight 12%

**What earns credit:** Three lifetimes, durable local/runner intent, explicit dispatch authorization, unknown effects, conservative boot/restore.

**Remaining deduction:** Remote APIs cannot be made globally atomic by local bookkeeping, and a single hub trades availability for clarity.

**Evidence needed:** Inject crashes, partitions, delayed renewals, ambiguous PR writes and old-backup restore against actual process/forge behavior.

## Verification and integration — 95/100; weight 10%

**What earns credit:** Gate integrity and adequacy, exact producer/subject, cumulative steps, staged CI bootstrap and current combined integration.

**Remaining deduction:** Finite tests and human/model reviews remain incomplete oracles; hostile candidate code can target weak test designs.

**Evidence needed:** Good/no-op/defect controls, skipped/zero/wrong-issuer checks and individually green but interacting changes fail as specified.

## Team coordination and humans — 95/100; weight 8%

**What earns credit:** Full task reservations, owner fairness, human takeover/adoption, safe withdrawal and visible unmanaged work boundaries.

**Remaining deduction:** Unregistered local edits and undeclared semantic coupling cannot be completely observed.

**Evidence needed:** Two owners/runners coordinate overlap and independent work; paused human work is not silently stolen; withdrawals reconcile effects.

## Security and data boundaries — 93/100; weight 8%

**What earns credit:** Current identity/data policy, protected control/evidence, separate credentials, constrained intake and privacy-aware memory.

**Remaining deduction:** Actual host/provider-key isolation and deployment-specific defense have not been demonstrated.

**Evidence needed:** Credential, hook, cache, object-import, egress, prompt-injection and cross-repo disclosure fixtures pass on each enabled profile.

## Economics and controlled learning — 94/100; weight 8%

**What earns credit:** One physical cost with many enforcing scopes, whole-outcome accounting, no hidden child money, controlled profile learning.

**Remaining deduction:** Cost savings, acceptable uncertainty and causal harness improvements need real data; billing may arrive late.

**Evidence needed:** Matched cohort includes all failed/review/CI costs and human time; known/unknown exposure reconciles without losing overruns.

## Harness and provider realism — 94/100; weight 4%

**What earns credit:** Early incumbent/Prime test, concrete OpenJarvis subset and QM boundary contributions; one role-qualified adapter contract.

**Remaining deduction:** Selected source review is not runtime conformance or proof that a fork would be cheaper.

**Evidence needed:** Run equal-outcome trials; retain the smallest conforming implementation, measured maintenance and supported upgrade behavior.

## Operations and release boundary — 93/100; weight 2%

**What earns credit:** Deterministic diagnostics, bounded load, safe retention/export and one existing immutable-artifact release path.

**Remaining deduction:** Actual load, recovery objectives, release environment and reversible rollback remain unqualified.

**Evidence needed:** Measure the disclosed fixture without weakened durability; rehearse restore/export and a single approved staging/production target.

## Non-compensable release failures

Unauthorized spend/access/mutation, stale authoritative publication, writer-minted acceptance, wrong-subject integration, untrusted execution holding privileged delivery credentials, blind duplicate effects after an unresolved timeout, silently revived imported grants and loss of acknowledged control intent block the affected capability. A high average or cheaper route never waives these gates.

Do not raise the score merely after adding another reference or a longer appendix. The next useful improvement comes from demonstrated conformance, less operator handling, matched-quality economics, or a simpler equivalent implementation.
