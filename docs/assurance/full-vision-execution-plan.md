# Full vision: implementation and real production acceptance

Status: **ACTIVE execution plan; delivery and production acceptance remain incomplete**  
Owner: sole family integrator, with current packet owners and independent reviewers  
Reconciled: 2026-09-09 against working sources and family coordination

This implements the user's request to finish **every active requirement**, including evolution,
distributed teams and cross-repository sagas, and demonstrate useful operation on the actual
Codex, Claude, Cursor and Antigravity subscriptions. It refines [Waves 0–11](closure-roadmap.md),
the [full-product bridge](full-product-dogfood-plan.md), [G1–G18](product-gaps.md),
[WP-01–WP-23](../workplan.md), and the existing DF packets. Those registers and condition-specific
receipt admission retain completion authority; this document creates no second status register.

The current user instruction selects sixteen real tasks across all four providers. Earlier
three-provider/twelve-task staging does not define completion. The selected preserved fresh
coordinator generation remains conditional on the complete reviewed admission and operator
checkpoint in [ADR 0015](../decisions/0015-dogfood-track.md). It does not recover the original
incident or make the older recovery-first route mandatory for the distinct generation.

## 1. What completion must mean

The production chain is:

```text
operator goal → durable command → admitted account/runtime → actual native CLI
→ validated proposal → preserved BulletGit Candidate → independent verification
→ exact-subject forge check → protected human-approved integration
→ authoritative target read-back → signed observation → truthful packaged Portal
```

Every arrow must have an implemented consumer, durable identity and executable acceptance case.
A real CLI returning text is insufficient. All four CLIs must author accepted implementations
through this chain; direct agent edits outside it do not count as campaign completions.

No mock provider, replacement executable, shim returning vendor-shaped output, fixture signer,
replayed session, fabricated service response, synthetic browser backend, seeded success state,
or manually fabricated receipt may satisfy production acceptance. Do not add another fake-provider
milestone. Real failure injection acts on actual processes, connections, resources and storage.
Use disposable, operator-admitted test projects and data on real services for destructive drills.

Keep existing component tests and historical evidence with their original classifications.
The documented provider simulator remains an isolated test utility; any narrowly required offline
simulator proof remains separate and supplies zero credit to native-provider or external-forge
acceptance. Production builds and entrypoints must not silently select or dispatch it.

Only existing subscriptions are authorized. No API billing fallback, credit purchase, reset-credit
consumption, account switch, model substitution or undocumented endpoint change. Unknown quota
or charges remain unknown/reserved; exhaustion pauses dependent work. Confirm the actual
subscription route separately for each admitted account rather than inferring it from a CLI name.

## 2. Baseline and complete requirement accounting

The inspected inventory has 648 dispositions: 33 IMPLEMENTED, 592 PLANNED, 20 SUPERSEDED and
3 REFUSED. Thus 625 units are active. These are documentation dispositions, not 625 independent
features or any production certification. The typed inventory also has 18 gaps, 12 waves,
9 V1 slices, 23 work packages, 46 gate bindings, and 18 product plus two diagnostic profiles.

Inspected heads were Hub `6aa1d901`, Kernel `b43d4303`, BulletGit `cc985434`, Portal `7f6d97ea`.
Every checkout had pending changes; these short IDs identify observations, not frozen test subjects.
Current source still routes the command worker through `transaction_offline`; the product verifier
refuses admission; Codex/Cursor session starts refuse; Runner lacks Antigravity and defaults to
`sim`; the sealed gate catalog includes PONG. Re-read exact source before each packet.
The previous corpus consistency change is independently reviewed and locally tested, but remains
uncommitted and closes none of these production gaps.

Before assigning the main campaign, extend the existing corpus and assurance machinery:

1. Inventory and hash every current member `docs/**/*.md`, referenced outer vision document,
   normative contract/policy, accepted ADR and workplan. Include root documentation and referenced
   material outside the seven original corpus documents. The observed 104 member Markdown paths
   are a discovery starting point, not a claim that all normative statements have been extracted.
2. Classify each normative statement as active, historical, refused, superseded, retired, or
   explicitly conditional. Preserve accepted refusals, supersession replacement links and WP-18.
   Resolve conflicting current requirements in the governing document; do not silently choose one.
3. Each active unit must link accepted meaning, product owner, enforcement symbol, executable
   acceptance case, negative case, gate/receipt kind, compatible source subject and dependency.
   Missing links become named gaps in the existing inventory immediately.
4. Track source implementation, actual component execution, actual production execution and
   semantic release admission separately. Re-review all 33 IMPLEMENTED entries. A symbol lookup,
   declared test, test pass, or screenshot cannot stand in for the next evidence class.
5. Prove bidirectional coverage of corpus units, invariants, waves, V1 slices, DF/WP rows, G gaps,
   operator dependencies, runtime owners, tests, all gate bindings and every receipt kind.
   Zero orphaned active obligations is required; neither a fixed 648 count nor a renamed status
   may hide additional discoveries. Keep precisely the two adopted formal models.

Exit: independent review can select any active requirement and find its exact planned execution
and required receipt; later it can reconstruct the admitted result. An unresolved dependency
keeps that requirement open and prevents the final full-vision claim.

## 3. Coordination with the other agents

Use the outer family `AGENT_CHAT.md`; read the complete current log and physical tail immediately
before each append, claim, edit and handoff. Operating HOLD permits only the documented manual,
path-exact engineering coordination. No coordinator verb or incident relocation is authorized here.

| Role or current owner | Immediate request and responsibility | Custody rule |
| --- | --- | --- |
| Existing root integrator | Reconcile pending packets; serialize integration, source freeze and complete proofs; coordinate aggregate publication | Remains the sole integrator; no competing writer |
| Claude CI owner | Return exact remaining compiler/tool, nextest, installer, workflow and documentation packets with hashes and evidence | Preserve current files until explicit handoff/release |
| Cursor/runtime owner | Return current SQLite/run_coding, Runner supervision, proposal limits and Portal command packet boundaries | Preserve current work; agree four-file seams before follow-on edits |
| BulletGit owner | Return preservation/apply/finalization packet and Kernel authority dependencies | BulletGit alone creates product Candidate clones |
| This planning/assurance lane | Reconcile scope, require real acceptance, review evidence and route unowned work | Does not claim another owner's implementation or runtime evidence |
| Independent reviewer and operator | Review exact source/evidence; separately perform installed custody and human integration acts | Author cannot certify own work; labels alone do not prove independence |

These are coordination requests, not acknowledgements. Current owners must answer before ownership
is transferred. Request: exact files, base/current hashes, remaining delta, source dependencies,
test evidence, active process/session custody, next command and explicit release disposition.
Reconnect the existing Codex session through its owner; never delete a live writer lock.

Run at most two implementation workers, each with disjoint packets of at most four claimed files.
Enforce the existing one-seat-equivalent bound per named human as well as per-account limits.
Review is separate. Use canonical checkouts only, never Git worktrees. Use private build targets;
allow one cold build at a time with the documented scratch margin. Complete family proofs run
only after acknowledged source freeze and integration. Every packet ends with hashes, retained
successes/failures and a resumable handoff or HOLD; owner heartbeats stop with the owning process.
Reconcile after every packet. A missed acknowledgement pauses only the dependent ownership seam.

## 4. Dependency-ordered implementation

### W0 and current CI queue — trustworthy sources first

Finish the existing compiler/tool-selection fixes, exact nextest inventories, pinned installers,
complete workflow routing and documentation freshness. Resolve the outstanding Cargo stderr
capture/refusal issues without leaking raw output or accepting truncated diagnostics. Admit
actual selected tools and tests, not just command exit codes. Keep prior failed runs.
Coordinate media relocation and the public `neverhuman/bulletfarm` README in the same publication
change so every link resolves. Accept the reviewed corpus packet by its exact hashes.

Qualify the portable auditor and triage all retained 165 findings/28 cap occurrences against
current source. Repair genuine causes; do not manufacture generated classifications or weaken
checks. Required floor is 90 with zero caps and zero hard findings per member. Preserve the
adopted aggregate CI graph, matrices, schedules, fork policy and required-check semantics.
Missing, skipped, neutral, cancelled or stale results cannot satisfy required checks.
WP-22 requires two consecutive successful main runs, actual merge-group handling, fork safety,
ruleset read-back and verified cache policy. Retain two independent source-bound assurance re-scores.
Exit: clean reviewed member subjects, serialized member/family proofs, aggregate workflow/run
evidence and authoritative protected-main read-back. A member green run is not aggregate proof.

### Phase R / W0 — coordinator admission, then supervised storage upgrade

Complete the two-location inventory, preservation destinations, strict ledger replay, unresolved
work dispositions, independent review bytes and four current commit/tree bindings. Under the
final exclusive initialization guard, revalidate every subject and retirement operation; bind
the complete admission into durable Genesis. Exact retries retain identity; changed retries,
partial retirement, collisions and drift refuse. Restart must revalidate the references.
Prepare the complete checkpoint packet before the operator acts; historical incident recovery
remains a separate unclosed obligation. Prove contention, owner death, expiry, handoff and restart
with real processes. No stale heartbeat or surviving process may manufacture replacement authority.

Before new execution tables, finish supervised SQLite schema 22→23 upgrade: exclusive maintenance
custody, durable intent, verified supported-prefix backup, transactional migration, reopen/read-back,
external authority high-water and quarantine-preserving recovery. Kill actual processes at each
durable boundary. Exit is exactly the valid old/new state or explicit recoverable quarantine;
restoring an old package or backup cannot revive authority. No reverse migration shortcuts.

### W1–W3 / DF-101–304 — contracts, atomic launch and containment

Prepare and admit OD-D/OD-E source-tag, build-attestor, trusted-time and release custody before
signed W7 artifacts or installation proofs; later campaign entry revalidates installed custody.
These read-only source and signing acts grant no provider or protected-forge mutation authority.

Generate Rust, JSON Schema, OpenAPI and TypeScript for four providers, accounts, enrollments,
quota buckets, reservations, commands, runs, proposals and session lifecycle. Publish immutable
wire subjects; enforce strict duplicate/unknown-field rejection, canonical encoding, exact typed
identities, signer purpose, expiry/revocation, trusted time and replay high-water.

Keep `POST /api/v1/commands` and `run_coding` as mutation ingress. Atomically validate authority
and expected revision, bind exact task/model/effort/account/runtime/policy/base/checkpoint/scope/
gates/deadline, reserve account capacity, consume an issued nonce, allocate the run and enqueue
one dispatch. Caller IDs cannot create authority. Exact retries return the original result.
Separate vendor quota buckets from local allowances; persist unknown consumption as liability.

Persist launch intent before spawn and actual process/workcell identity after spawn. Reconcile
ambiguous launches before another invocation. Implement durable stopping/reconciliation states;
interrupt acknowledgement is insufficient to release capacity. Confirm complete descendant
termination, then settle usage/reservations/leases. Prove Runner/farmd death and restart.
Provision separate control, Runner, BulletGit, verifier, broker, attestor, integrator, observer,
auditor and Jeryu identities with real credential custody and peer authentication.
Certify S1 filesystem/network/process/resource/secret isolation and the separate S2 Firecracker
boundary; S2-required work must refuse until its exact boundary is certified.

### W4–W5 / DF-401–503 — Candidate, verifier and real forge effects

Validate proposal structure, scope, paths and preimages before consuming one-use mutation authority
when no effect has occurred; retain the final online authority/preimage check at mutation.
Enforce the agreed 128-operation/path and 32 MiB aggregate-content bounds across every consumer.
Use immutable Rust, TypeScript/browser and documentation gates with admitted executable/dependency
digests, argv, environment, timeout and output contracts. Declare build output outside source scope.
Atomically preserve Candidate, transition Attempt, settle lease, append audit and enqueue verifier
handoff before cleanup. Prove stale bases, hostile paths, interrupted writes, ENOSPC and lost replies.

Implement the production verifier's authenticated intent consumer, durable nonce custody,
independently owned workcell/artifacts, real gate execution and signed Evidence/ProofBundle.
Establish independence by actual principal, credentials and underlying model provenance.
Execute Candidate verification → delivery/read-back → exact-subject check/read-back → protected
human-approved integration/read-back → signed observation. Preserve effect identities on retry.
Changed Candidates, rebases and merge-queue result OIDs invalidate affected reviews and gates.
Enforce the adopted two-track scope amendments, verifier dwell/backpressure and fence-mediated
exits from CONTRADICTORY or prolonged UNKNOWN states. Oracle-modifying diffs require the separate
R2+ holdout/human approval controls; an author must not repair acceptance by weakening its oracle.
Run real Jeryu and GitHub campaigns, then GitLab.com and the selected self-managed GitLab version.
LocalBareForge results remain local coverage and cannot certify an external forge.

### W6–W8 / DF-601–706 — complete operator product and packaging

Connect task dependencies, acceptance criteria, ownership, context, handoff, review, cancellation,
freeze, account selection and resume to durable commands. Finish all fifteen Portal surfaces;
the full vision cannot close with cognitive surfaces marked OUT_OF_PROFILE. Include Cognitive
Router, Fusion Lab, Quota/Capacity, Struggle, Behavior, Workspace Hygiene, context successor/
compression lineage and atomic Shift Brief. Persist operation identity across reload, lost
responses, SSE gaps and restart. Show queued/running/stopping/blocked/ambiguous states and the
next operator action; distinguish recorded freeze from acknowledgement by every affected Runner.
Complete same-origin packaged Portal, authentication, authorization, CSRF/origin/CSP controls,
account/session isolation and automated plus manual WCAG 2.2 AA acceptance.

Produce non-circular signed schema-3 locks, reproducible packages with embedded Portal,
provenance, SBOMs, checksums and rollback assets. Run two genuinely clean installs per native
target, then activation, upgrade, backup, restore, rollback, retained-data uninstall and disaster
recovery. Certify Linux x86_64/aarch64, macOS x86_64/aarch64 and Windows x86_64 on native hosts.
A profile-permitted mutation refusal proves only that refusal; advertised mutation requires
its separately certified containment. Rebuild paper and executive brief twice from exact subjects.
macOS/Windows mutation requires the certified Linux S2 guest through Virtualization.framework
or Hyper-V; host-only permission controls are not equivalent containment.

### W9 — complete cognition, collaboration and evolution

Persist classification, role contracts, routing exclusions, quotas, context lineage, dissent,
selection, fusion, struggle and negative knowledge. Complete BulletGit conflict forecasting,
proof invalidation, patch composition/algebra, causal provenance, compensation and proof/context
retrieval. Implement all T0–T5 recipes with distinct role budgets/artifacts/custody and a certified
T0 fallback. Deterministic replay must reproduce decisions without changing authority or safety limits.

Execute the frozen matched T0/T3 study: equal model/tool/environment/compute, deterministic
allocation, A/A calibration, repository/lineage/time holdouts, blinded review, contamination,
missingness/multiplicity rules and every attempted outcome included. Adopted MOME bounds are a
3×3×3 grid, eight per cell, hard-violation filtering, conservative confidence bounds,
deterministic hypervolume eviction and ASHA factor three; disable ASHA if calibration fails.
Retain the B0–B6 evidence ladder and untouched B3 holdout custody. Exact rung sample sizes and
T0 reserve percentage must be preregistered; they are not invented fixed requirements here.
Freeze finalists before external confirmation. Sequence confirmation → no-effect shadow → rollback
readiness → separate OD-H authorization for an expiring R0/R1 canary ≤1% with T0 reserve → observed
canary/rollback → independent promotion/drift receipts. Drift decertifies before another route;
rollback atomically restores the last healthy generation. R2+ requires exact signed human approval.
Certify `evolution-v1` independently. Retain the matched pinned Gas Town/Gas City comparison and
other active comparative-study obligations; no superiority claim precedes comparable evidence.

### W10–W11 and active follow-on work — universal, team, then saga

Maintain independent provider, forge and native-platform certifications and compose only compatible
current receipts into `universal-v1`. Provider brand does not establish model independence.
Complete `team-v1`: PostgreSQL authority, remote Runners/verifiers, mTLS/SPIFFE workload identity,
object storage, replicated projections, durable delivery, fencing, failover, freeze and distributed
restore. Real partitions, duplicate delivery and clock movement must never mint double authority.
Only after team certification, close `saga-v1`: staged cross-repository Candidates, dependency
quarantine, explicit partial integration, compensation as new effects and forward repair. Reconcile
every target; never describe partial outcomes as atomic success.

Close active publication, mirror, preservation, topology, paper and research obligations in WP-09,
10, 14, 15 and 17 as well as the V1 work. Conditional choices need their reviewed disposition;
silence is not completion. WP-18 remains RETIRED. Jeryu source stays exclusively in the existing
`/home/ubuntu/jain-split/jeryu-split` family and is consumed through admitted immutable artifacts.
WP-19 must support both connect-existing and managed Jeryu runtime modes without altering the
existing source family or shared service. WP-10 requires the signed preservation RFC, complete
ref inventories/bundles, ancestry map and rehearsal before mirror decisions. WP-14 publication
must provide immutable source/PDF links whose bytes match the reviewed evidence manifest.

## 5. Native CLI qualification on our actual installations

One product lifecycle must cover admission, start, send, streamed events, permissions, interruption,
termination, usage settlement and restart recovery. Replace refusing/inert session methods and
remove simulator defaults. An adapter may translate a documented protocol; it may never fabricate
a vendor capability, response, authentication result, usage fact or successful terminal state.

| Provider | Native implementation and qualification target |
| --- | --- |
| Codex | App Server initialization, thread/turn lifecycle, explicit model/effort, schema-constrained proposal, interruption and account/rate-limit observations; generate schema from the admitted installed version. Use the selected native stdio route and preserve the existing session writer. [Official interface](https://learn.chatgpt.com/docs/app-server). |
| Claude | Subscription-authenticated `-p` execution, stream JSON, explicit model, validated terminal structured proposal and supervised stop. Do not use `--bare` for this route: it bypasses subscription login. Explicitly contain startup hooks/plugins/MCP/config discovery. [Official headless interface](https://code.claude.com/docs/en/headless). |
| Cursor | Native `agent acp`: initialize, `cursor_login`, session/model/mode selection, updates, permission responses and cancellation. Answer blocking native extension requests; remove reliance on an invented Bullet vendor extension. [Official ACP interface](https://cursor.com/docs/cli/acp). |
| Antigravity | Native `agy` JSON/stream JSON, schema-constrained terminal proposal, explicit available model/effort, conversation identity, cumulative usage and terminal-state validation. Add it to every generated coding-provider consumer and Runner selection. [Official headless interface](https://antigravity.google/docs/cli/headless/). |

The docs establish interfaces, not installed compatibility, account availability or successful
production qualification. Codex documentation describes app-server/remote WebSocket support as
experimental and unsupported for production workloads; resolve the exact selected-version/transport
support limitation before claiming a supported deployment. Do not silently substitute a different
transport, SDK or billing route to make a check pass. An unsupported requirement remains open.

For each account, freeze executable and loader/library/interpreter/package closure, version,
startup configuration, private HOME and credential generation, subscription billing mode,
actual advertised model/effort, protocol schema and approved network destinations. Exclude ambient
API credentials and undeclared hooks/plugins/MCP. Capture observed provider identity and limits
without exposing credentials. A claimed CLI alias or successful `--version` is not admission.
Run genuine bounded conformance through the product command path before admitting campaign work.
Permission denial, truncation, missing terminal result and nonterminal status cannot become success.

## 6. Fixed sixteen-task production campaign

Freeze the actual backlog choices, accepted meanings, four-file packets, bases, gates, budgets and
independent reviewer before the first campaign invocation. The following is the proposed cohort;
owners must reconcile it with their current work before freeze. If a task is already completed,
replace it before the campaign starts and record why. After execution starts, keep the cohort and
all failed attempts in the denominator; do not substitute easier work or reset budgets.

| Implementer | Rust improvement | TypeScript/React improvement | Regression-test improvement | Documentation/tooling improvement |
| --- | --- | --- | --- | --- |
| Codex | Durable quota read model separates vendor buckets from local reservations | Quota/Capacity displays unknown consumption and next eligible reset | Real browser reload after lost command response returns the original command/run | Native Codex onboarding diagnostics verified against the admitted installation |
| Claude | Atomic Shift Brief snapshot binds one durable watermark | Shift Brief presents ownership, blockers and next actions from that snapshot | Real cancellation/Runner-restart test proves descendant termination before release | Upgrade/restore procedure reproduced on the installed package |
| Cursor | Resumable SSE/read model exposes gaps and bounded resynchronization | Context Lineage shows successor/compression provenance and unresolved context | Changed-Candidate/merge-subject test proves stale review cannot integrate on a real forge | Mixed-provider handoff runbook reproduced through the packaged application |
| Antigravity | Workspace Hygiene read model explains preservation and cleanup eligibility | Workspace Hygiene shows exact workspace/Candidate state and recovery action | Real-browser cross-account/session isolation test with actual authenticated farmd | Native Antigravity onboarding and terminal-state diagnostics verified on its installation |

Campaign entry requires the admitted W7 transaction/lifecycle subject, installed production
worker and independent verifier/effect custody, exact production account/forge operator admissions,
and native conformance. The separately classified W7 offline obligation gives no native credit.
These are improvements after the minimum production transaction is operable; no campaign task
may be used circularly as proof that its own missing launch/verifier authority already existed.
Each task must change useful tracked implementation/tests/tooling, meet frozen acceptance, pass
independent gates/review, integrate with human approval and retain authoritative final observation.
Map each task to its existing requirement/G/WP; split oversized work before cohort freeze.

Use mixed-provider handoffs: context prepared by one actual model family must be resumed by
another, retaining lineage and authority. Select the reviewer by observed underlying model and
principal provenance, not a fixed CLI rotation. Different providers using the same model family
do not supply independent model review. A human independent reviewer resolves unavailable diversity.

Limits apply across planning, implementation, repairs and review: one active invocation per
account, two implementation workers, two repairs, one escalation, eight invocations per bounded
task and 60 minutes. Native retries/subagent spending must be visible and budgeted where observable;
unobservable usage stays reserved. Pauses, account changes and lost responses reset none.

## 7. Real fault campaign and independent acceptance oracle

For each fault, retain the actual executable identities, controller action, timestamps,
before/after durable state, OS process/cgroup observations, native session evidence and exact
read-back. The injector must not provide replacement responses or substitute a provider binary.
Inject immediately before and after durable boundaries where applicable, and during the external
effect. Repeat process/session-sensitive cases for each of the four native providers. Declare
remaining shared-boundary coverage explicitly; one passing backend cannot silently cover another.

| Boundary | Real fault | Required invariant and read-back |
| --- | --- | --- |
| Grant/admission | Concurrent duplicate POSTs, dropped accepted reply, stale revision/nonce | One command, reservation, run and dispatch; exact retry returns original identity |
| Runner start | Kill before/after spawn record; restart with possible survivor | Reconcile original launch/process identity; no replacement while execution is unknown |
| Workspace open | Revoke authority, alter base, deny storage, terminate worker | No unauthorized clone/mutation; exact preserved or explicitly quarantined generation |
| Provider completion | Disconnect/kill actual CLI, interrupt, truncate capture, reach deadline | Missing/failed/nonterminal output never becomes success; usage liability retained |
| Patch application | Stale preimage, hostile path, real ENOSPC, interrupted writes | No escaped write or corrupt visible generation; known pre-effect refusal stays distinct |
| Checkpoint | Kill between durable writes/fsync and publication | Reopen reconstructs exact old/new checkpoint or named recoverable quarantine |
| Candidate preparation | Lose commit reply, exhaust disk, restart storage owner | One exact immutable Candidate; atomic state/event/outbox/lease settlement |
| Verifier handoff | Duplicate delivery, revoked signer, changed Candidate, verifier death | Durable one-use intent; independent reconstruction; no writer self-certification |
| Delivery | Drop real forge reply before/after push; expire credential | Adopt only original desired effect by authoritative read-back; no duplicate logical effect |
| Check | Change check SHA or protection configuration; lose reply | Only current exact-subject check/protection proof can satisfy readiness |
| Integration | Move target/merge result, race human action, lose response | Protected expected-old integration; stale review refuses; reconcile original effect |
| Observation/cleanup | Kill observer/cleanup, lose store, restore older backup | Completion needs exact signed read-back; preserved artifacts and high-water survive |

Additionally exercise actual secret isolation and denied egress, resource exhaustion, corrupt
storage, backup/restore rollback protection, expired admission/runtime drift, observed vendor
limits, browser reload, lost HTTP responses, SSE gaps, stale projections and account isolation.
Do not deliberately buy capacity or burn a subscription to force vendor exhaustion: capture an
actual limit when encountered; until then that vendor-limit acceptance case remains unproved.
Hardware/platform/provider-specific faults require their own real execution subjects.

Fault campaigns are separately predeclared bounded work with explicit subscription budgets;
they cannot hide failed implementation attempts or renew an exhausted task's allowance.
Keep failures as failures. Fix the defect, invalidate affected subjects and rerun the affected
case plus regression dependencies; retain the original evidence. No sleeping/retry loop may
turn missing observations into PASS.

## 8. Evidence, demonstration and usefulness

For each accepted task retain the linked goal/contract, command/idempotency receipt, enrollment,
runtime/configuration/account/model/effort, launch/session/turn/process identity, raw native
transcript, usage/termination, proposal, base/checkpoint/scope, Candidate manifest, gate definitions
and artifacts, independent review, human approval, forge check/integration/read-back and observation.
Bind member commits/trees, aggregate event SHA, workflow digests, run/attempt, actually selected
and completed test IDs, artifact hashes and exact package. Store originals with appropriate
custody, plus redacted shareable derivatives whose lineage is preserved.

An independent verifier must reconstruct the Candidate and evidence from separately owned
artifacts, query the actual protected target and agree with the recorded final subject. A status
field, screenshot, child exit0 or signed fixture is never its acceptance oracle. Reproducible
test/control/recording tooling must ship in the tracked Rust/TypeScript product repositories;
temporary diagnostic scripts alone do not constitute delivered tooling.

Demonstrate the same ledger tasks in the real TUI and packaged Portal: submit a mission, observe
all four providers implementing work, inspect quota and context, cancel/recover a real run,
review the exact Candidate, approve protected integration and verify the final target. A colleague
must complete the workflow without SQL or reconstructing ownership from chat. Retain native
1920×1080 frames, original transcripts, lossless masters and reproduction manifests. Each GIF
must be below 50,000,000 bytes; claim losslessness only after independent decoded pixel equality.

Observe every accepted integration for seven days. A material change to the observed behavior
starts a new window for the affected subject; retain earlier windows and escaped defects.
Measure first-pass acceptance, all repairs/escalations, active human review/recovery minutes,
elapsed integration time, coordination overhead, provider/local quota use, duplicate/lost work,
escaped defects and reversions. Preregister comparable manual work, task difficulty and accounting.
Target at least 25% less active human time without worse correctness; report paired distributions,
sample size and uncertainty. Sixteen tasks do not establish broad statistical superiority.
Freeze a first-pass target before execution; proposed target is at least 13/16, with all16
eventually accepted within their original bounds. Missing that target remains improvement work.
An exhausted task fails the original cohort; later remediation is a separately identified attempt
and cannot replace that task or improve the original campaign denominator.
Any authority duplication, secret escape, silent data loss or unaccounted work fails acceptance;
all correctness regressions need repair and renewed observation before the full-delivery claim.

## 9. Exact closure matrix and final audit

| Gap | Required closeout, using existing designated receipts |
| --- | --- |
| G1 | Signed schema-3 installation and two clean real lifecycle runs |
| G2 | Complete five-authority transaction plus signed twelve-boundary fault campaign |
| G3 | Durable Kernel authority, atomic admission, supervision and recovery |
| G4 | Online-authorized BulletGit writes, preserved Candidate and Integration proof |
| G5 | Four separate actual native provider/account/runtime conformance and live receipts |
| G6 | Admitted Jeryu deployment, protected integration, observation and restore |
| G7 | Actual GitHub App/protected project, exact checks/integration and reconciliation |
| G8 | Admitted portable auditor ≥90, zero caps/hard findings and security evidence |
| G9 | Reproducible signed packages, provenance/SBOM and native lifecycle receipts |
| G10 | Separate S1/S2 and native-platform containment/mutation-support acceptance |
| G11 | Independent evolution study/shadow/canary/promotion/drift/rollback receipts |
| G12 | Semantic receipt admission and actual PASS for every requested profile |
| G13 | All15 durable Portal surfaces and real-browser accessibility/operations evidence |
| G14 | Authenticated durable farmd API, command recovery, SSE and exact read-back |
| G15 | Durable cognition, context, routing, fusion, struggle and deterministic replay |
| G16 | Independent real GitLab.com and selected self-managed GitLab certifications |
| G17 | Actual distributed team authority, partitions/failover and restore |
| G18 | Team-dependent saga partial integration, compensation and forward repair |

All18 product profiles must independently PASS: self-hosted-v1, evolution-v1;
provider-claude/codex/cursor/antigravity; jeryu-forge-v1, github-adapter-v1,
gitlab-adapter-v1, gitlab-self-managed-v1; platform-linux-x86_64/aarch64,
platform-macos-x86_64/aarch64, platform-windows-x86_64; universal-v1, team-v1, saga-v1.
Run the existing `bullet-family check release --profile PROFILE --receipts ABSOLUTE_REGISTRY --json`
for each exact profile; an absent-registry diagnostic is never an admitted release result.

Final acceptance requires simultaneously: no active requirement merely planned or orphaned;
all designated G1–G18 receipts admitted for compatible current subjects; all18 profiles PASS;
all16 tasks and required fault cases complete with original failures retained; seven-day observation
and usefulness evaluation; native lifecycle/security/accessibility complete; protected public main
and exact successful applicable CI/artifacts; and independent reconstruction with zero reliance
on mocks, shims, fixture authority or synthetic application state for production results.

## 10. Operator packets and next two implementation cycles

Prepare complete consumer code, negative tests, exact scope/expiry, revocation/rollback and read-back
before requesting each operator act: coordinator checkpoint; service/signing custody; four valid
subscription enrollments; protected forge projects; native hosts; immutable release publication;
and later the separately authorized evolutionary canary. Never create keys, enrollments or
operator-decision lines on the operator's behalf. Existing authorization is not requested again.
OD-K internal dogfood admission and DOGFOOD_RUN observations clear no production, independence
or release gate. Preserve separate OD-D→OD-E→OD-A production custody and OD-B/C/I/J forge acts;
do not add an OD-D→OD-B dependency. Route all active OD-A–OD-K facts through existing decisions.

Frozen bootstrap commit/tree and W0 input bindings must exist before the Genesis checkpoint.
The source-maintenance exception prepares those subjects; operational W0 read-back and the first
coordinated change follow admitted Genesis. W0 observation is component evidence; W1 admits the
BaselineReceiptV1 over that exact subject. Source drift returns the affected work to W0.
OD-D/OD-E and the reviewed Jeryu tag precede W1 signed closure; local consumer engineering may
proceed earlier, but cannot claim a W1/W2/W3 release exit or activate missing authority.

The first queue uses the existing W/DF packets below. These are handoff boundaries, not new IDs
or status records. Split each implementation into acknowledged claims of at most four files;
the table does not transfer ownership or authorize an entire directory.

| Existing work / owner | Required entry | Reviewable output and stop condition |
| --- | --- | --- |
| Current CI/media/runtime packets / their existing owners | Owner acknowledges exact current scope and session/process custody | At most four claimed paths per packet, base/current hashes, retained pass/failure evidence, next command and explicit release. Silence, changed bytes or a live competing writer stops transfer. |
| W0 bootstrap preparation / sole integrator | Released packets, compiler/tool subjects and accepted corpus changes | Clean frozen member commit/tree subjects, matching W0 preparation, serialized proofs and family observation; coordinate aggregate README/media publication. Dirty union, missing tools or source drift refuses the freeze. |
| Phase R/W0 Genesis consumer / acknowledged Hub owner plus independent reviewer | Both preservation locations and frozen bootstrap/W0 inputs are available | Complete admission/replay/disposition references, locked final validation, durable Genesis consumer and real-process restart/owner-death evidence; review the whole checkpoint packet before the operator act. Missing location, custody or changed reference keeps Operating HOLD. |
| DF-201/204 storage continuity / acknowledged Kernel owner | Released SQLite seam and reviewed supported schema-22 prefix | Exclusive maintenance intent, verified backup, atomic 22→23 migration, reopen/read-back, external high-water and quarantine recovery. Real process death cannot revive stale authority; no new execution tables before this mechanism passes. |
| DF-101–103/202–203/301–304 command and native seams / acknowledged contract/runtime owners | Exact wire dependencies, disjoint claims and admitted migration for durable execution changes | Generated four-provider lifecycle contracts, atomic command/reservation/run/dispatch consumers and supervised stop/recovery seams with focused evidence. Missing installed custody, runtime/account admission or campaign prerequisites refuses live execution. |

Cycle1 targets owner handoffs, bootstrap/corpus integration and the first reviewed coordinator/
storage packets. Cycle2 continues those packets and the contract/atomic-command/native seams.
Neither cycle promises a whole wave or a native launch. Provider and Portal work proceeds only
on acknowledged disjoint dependencies; infrastructure waiting does not stop unrelated engineering.

After two measured cycles, forecast by observed accepted-packet throughput, unresolved dependencies,
review/repair time, subscription resets, native host/forge provisioning and the mandatory seven-day
window. Publish a range and assumptions; revise after each cycle. Do not promise a completion date
or claim the full vision from a plan, a successful recording, or component-only evidence.
