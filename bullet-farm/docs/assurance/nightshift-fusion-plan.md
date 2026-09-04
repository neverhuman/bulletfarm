# Nightshift fusion plan

Status: **Proposed; implementation and release authority remain blocked**  
Owner: Bullet family maintainers  
Last reviewed: 2026-08-26  
Applies to: `bullet-farm`, `bullet-kernel`, `bullet-git`, and `bullet-portal`

This is the implementation blueprint for translating the useful Nightshift
operating model into Bullet. It is not a receipt, an architecture approval, a
release decision, or permission to bypass the dependency order in
[`closure-roadmap.md`](closure-roadmap.md). The authoritative gap register is
[`product-gaps.md`](product-gaps.md), and an explicit profiled
`bullet-family check release` result wins over this document.

## Executive decision

Fuse Nightshift's **operator experience and scheduling lessons**, not its
runtime implementation.

The result should feel like Nightshift to an operator: one excellent morning
brief, a legible left-to-right delivery pipeline, clear blocked work, exact next
actions, bounded progress when one item cannot run, retained attempt knowledge,
and eventually one agent-facing conversation. Underneath that experience,
Bullet keeps its stricter authority model:

- Kernel is the only durable scheduler and control authority.
- BulletGit is the only Git mutation authority.
- Independent verifiers produce Evidence; neither an agent nor the Portal can.
- Effects, integration, deployment observation, and release review remain
  distinct authorities.
- Portal projects exact durable state and submits typed commands; it never
  computes completion from browser state.
- A conversational agent may investigate and propose. It does not mint
  authority, write Git directly, verify itself, or turn a missing observation
  into success.

No Nightshift Python, shell, HTML template, skip file, branch-name claim, or
Markdown parser should be copied into a production Bullet path. The source is a
valuable behavior study and adversarial fixture, not a component dependency.

## Exact study subject

The read-only study used this source subject on `xbabe0`:

| Item | Captured value |
| --- | --- |
| Checkout | `/home/ubuntu/veox-auto` |
| Branch | `nightshift/blocked-chaining-and-branch-pruning` |
| HEAD | `8451f95674bfcfc466f92b3cbefa334fc92effea` |
| HEAD tree | `b802dfdb963c84b27c0865b65b52da9aa418d726` |
| HEAD parents | `728e8d9fa1cf3c0e4f348be67c45b8f43366e644`, `9e8d8a311737f41208d9310e6a1eac442c49abee` |
| Upstream branch | `728e8d9fa1cf3c0e4f348be67c45b8f43366e644` |
| Working state | two commits ahead of upstream; tracked tree clean; pre-existing untracked `.claude/` |
| Block-chain/prune change | `77a24e8545cfb5883ca951985ebdd0f1c4aabe31` |
| Dashboard dependency change | `cecaf470753312ae4f10c134178f74a14265865e` |
| Dashboard write-up | `docs/pipeline-dashboard.md` from `9e8d8a311737f41208d9310e6a1eac442c49abee` |
| Handoff template | `docs/jain-handoff-template.md` |
| Research | `docs/research/gastown-2026-08.md`, `graphify-serena-2026-08.md`, `omnigent-2026-08.md` |

The bounded audit manifest is the following path-to-Git-blob map. These are Git
object identities, not filesystem checksums:

| Path | Blob |
| --- | --- |
| `docs/pipeline-dashboard.md` | `cce41f25ce8131c46fae1b5ceb55a64fac45cc3a` |
| `docs/jain-handoff-template.md` | `b2064b796b65c86c90f32b8ecddd3fa50b13be52` |
| `docs/research/gastown-2026-08.md` | `09781b7ca9123d80529fc3ce3130fe8b8d23127f` |
| `docs/research/graphify-serena-2026-08.md` | `70d92c1ddce1a2c0af218bfd4d44751f6b301a8b` |
| `docs/research/omnigent-2026-08.md` | `5e5c588b3c3d330f0fc8adde0229ba3498b46c90` |
| `bin/todo-driver.sh` | `f1c2465edeec3abf83517771310de01e6833d169` |
| `bin/secondary-driver.sh` | `95da46351b3d6e00158584a4d6388caa1e6d4379` |
| `bin/pipeline.py` | `14219b17f8bfc52de3f726d98ff8dc363ce6b8ff` |
| `bin/pipeline.tmpl.html` | `ded8b9999d9ee638ff0f919918f9cb27ebece36d` |
| `bin/console-server.py` | `42581664922aec16813eac767c13e62c7424e76a` |
| `bin/jain-handoff.py` | `4aa323fc61c728536bd0806d9779698213ab98b3` |
| `bin/publish-jain-handoff.sh` | `8b1b59e5ec4b702a20f195431de150cec48e64ef` |

This is not an immutable published dependency: the branch is ahead of its
remote and the checkout is not pristine. Any later comparison must recapture
the exact commit, tree, relevant blob identities, status, and tool versions.
There will be no cherry-pick, subtree, path dependency, new worktree, or runtime
link to this checkout.

### What is implemented in Nightshift

| Behavior | Implementation | Useful lesson | Limitation that must not cross into Bullet |
| --- | --- | --- | --- |
| Continue after a blocked item | `.auto-blocked`/exit 75 re-executes the driver with `VEOX_BLOCKED_CHAIN`; primary and secondary paths recheck most outer guards | One blocked item should not waste an unattended dispatch window | Shell recursion is not a durable transition; the named maximum allows the initial item plus four continuations; a continuation is another tick and may switch repository; reasons remain free-form |
| Retire merged auto branches | startup and success paths call direct remote deletion after an ancestry check | Completed automation refs should not accumulate forever | There is no expected-old-OID CAS, preservation receipt, effect record, or reliable reconciliation; a concurrent ref update can be deleted |
| Dependency cascade | `pipeline.py` parses `Needs`, `Depends on`, or `Requires` Markdown and computes a fixed-point held set | Operators need transitive blocker visibility | It is presentation-only; neither scheduler consumes it, identifiers are weak, and there is no committed behavioral fixture |
| Pipeline dashboard | loopback `/pipeline` shows queue, local, published, trunk, running, and delivery cards across four hard-coded lanes | A supply-chain view is much easier to operate than raw logs | The JSON is unversioned and heterogeneous, sources are scraped at different times, a 90-second last-good cache can hide failure, and no per-card watermark exists |
| Jain handoff | a Markdown template and helper publish a proposal branch for a human | Cross-boundary work needs an explicit request, evidence, and acceptance criteria | It has no durable accept/reject/resolve lifecycle, exact command identity, atomic saga, or replay/reconciliation contract |
| Review interaction | the wider console stores reviewer checkbox state | Review must be visible and easy to act on | A browser checkbox is not Evidence or a durable Review subject |

The study also found concrete reasons to translate rather than port:

- queue, run, and inflight identities can collide across lanes because several
  lookups key only on a repo-local todo number;
- the dashboard can inspect a feature-branch `HEAD` while the scheduler reads a
  release-candidate ref;
- untracked files can be omitted from cleanliness presentation;
- advertised `off-lane` and `stale-trunk` health states are not assigned by the
  current builder;
- Git, GitHub, deployment files, caches, and live state can be combined from
  different epochs into one apparently coherent response;
- several command failures collapse to empty data, allowing false-zero or
  false-healthy presentation;
- repository-controlled text is serialized into an inline script without a
  robust `</script>` boundary;
- the service uses ambient Git/network/process authority and an external font;
- the generic Jain template itself passes the current shape validator;
- the direct branch deletion has a time-of-check/time-of-use race and lacks
  secondary-lane parity;
- `bin/pipeline.py` and `bin/pipeline.tmpl.html` are respectively about 1,180
  and 589 lines, above the shape appropriate for a typed Bullet component.

Read-only smoke checks established that the shell files parse, the Python files
parse, the tracked diff is clean, the loopback service is active, and
`/healthz` answers `ok`. The pipeline GET path was not accepted as read-only
proof because it may refresh network-backed Jain cache state.
`bin/check-console.py pipeline` could not run on the source host because `node`
was absent. That check is a DOM/CSS-class smoke check in any case, not schema,
scheduler, security, accessibility, or effect proof. No committed Nightshift
unit/contract/fault suite covers the behaviors above.

### What is only proposed in Nightshift

These ideas are documents or todos, not scheduler behavior:

- conditional `until`, `until-pr`, and `until-todo` blocks;
- a bounded summary of the preceding failed attempt for a retry;
- a narrow ADR convention around retry behavior.

The distinction matters. Bullet should test these ideas against its existing
domain and authority model; it must not cite the Nightshift proposal as
implemented evidence.

### Research disposition

The three Nightshift research notes support a deliberately narrow import:

- From Gas Town/Beads, retain the conclusion of a failed attempt and treat a
  conditional blocker as “not yet,” while preserving atomic code/work
  transitions inside Bullet's own ledger. Do not add a second task database as
  authority.
- From Graphify/Serena, stamp every derived artifact with its exact source and
  refuse stale or suspiciously shrunken projections. Do not adopt a low-recall
  code graph or a write-capable editor side channel.
- From Omnigent, preserve fail-closed sandboxing and default-deny egress ideas.
  Do not introduce a second control plane, database, retry loop, or permissive
  wrapper parser.

## Current Bullet boundary

The Bullet source already contains the stronger conceptual destination:

- [`corpus-coverage.generated.md`](corpus-coverage.generated.md) records ten
  Nightshift requirements. Only the separation of the five truths is currently
  implemented; the default Shift Brief and detailed proof drill-down remain
  planned for Wave 6.
- [`closure-roadmap.md`](closure-roadmap.md) requires Kernel authority in Wave
  2, BulletGit authority in Wave 4, the five-authority effect chain in Wave 5,
  production APIs and the default Shift Brief in Wave 6, packaging and fault
  proof in Wave 7, live self-hosted proof in Wave 8, durable evolution and
  `TeamRecipeV1` in Wave 9, and distributed/saga behavior in Wave 11.
- [`product-gaps.md`](product-gaps.md) keeps G1-G18 open until the named
  semantic receipts exist. Nightshift most directly informs G2-G4 and
  G13-G15/G18; it does not close any of them.
- Hub release truth already separates mechanical execution, evidence
  completeness, release review, deployment match, and post-deployment
  survival. It is code-backed and Markdown-rendered; current stable JSON lacks
  the richer per-claim narrative needed by a Portal wire snapshot.
- Portal already has stricter projection behavior than Nightshift: generated
  DTO validation, exact header/body watermarks, honest `UNKNOWN`, bounded SSE
  gap recovery, same-origin command bootstrap, and `PENDING` to `UNKNOWN`
  reconciliation. It has nine projected spec surfaces and six explicit
  `UNKNOWN` surfaces, but no Shift Brief and no managed pipeline workspace.
- Kernel has `WorkPackage` and a ready queue, but no authoritative typed work
  dependency graph or conditional wait reducer. Current materialization makes
  packages ready rather than deriving readiness from dependencies.
- BulletGit and Kernel already model preservation and cleanup authority. That
  makes Nightshift's direct ref deletion specifically inadmissible.
- `bullet-mcpd` is intentionally read-only. There is no command tool and no
  conversational authority boundary yet.

The generated release-truth report remains `BLOCKED`; this plan does not change
that fact or the scorecard.

## Fusion rules

Every implementation slice must satisfy these rules:

1. **One authority per fact.** A Portal card, agent sentence, Markdown row,
   cache entry, process exit, or Git branch name cannot become durable work,
   Evidence, an effect, integration, deployment, or release truth.
2. **One exact subject per claim.** Work, attempts, candidates, evidence,
   commands, reviews, effects, and observations name immutable typed subjects.
3. **Unknown stays unknown.** Missing, stale, contradictory, response-lost, or
   mixed-watermark state never collapses to empty, healthy, complete, or safe
   to delete.
4. **Read models are atomic.** A screen obtains one server-composed snapshot or
   displays an explicit clock vector; the browser does not synthesize truth
   from unrelated response times.
5. **Commands are durable before execution.** Mutations require identity,
   idempotency, expected revision, scope, policy, and reconciliation.
6. **Cleanup is a separate authorized effect.** Integration or inactivity does
   not itself authorize deletion. Preservation, exact identity, current remote
   observation, expected OID, read-back, and a tombstone are mandatory.
7. **Retry knowledge is bounded and typed.** Preserve the prior attempt's
   conclusion and exact supporting subjects, not an unlimited transcript.
8. **The agent is a liaison.** It may read, explain, draft, and later submit a
   scoped typed command. It cannot use conversation text as a permission,
   credential, verifier result, or durable decision.
9. **No second control plane.** No Beads, Omnigent, Markdown scheduler, or
   dashboard scraper duplicates Kernel state.
10. **No implementation wave skips its predecessor.** Read-only UX can land
    early with honest `UNKNOWN`/`OUT_OF_PROFILE`; authority-bearing behavior
    waits for the transaction, verifier, and effect spine.

## Adopt, translate, and refuse

| Nightshift concept | Decision | Bullet form | Owner | Earliest roadmap fit |
| --- | --- | --- | --- | --- |
| Morning handoff | Adopt | default Shift Brief with exact selected profile, family subject, five truths, blocking claims, freshness, and next authorized actions | Hub → Kernel projection → Portal | Wave 6 |
| Left-to-right pipeline | Adopt | atomic `MissionPipelineSnapshotV1` over work, attempt, Candidate, Evidence, effect, integration, and observation subjects | Kernel/farmd/Portal | Waves 5-6 |
| Continue after block | Translate | readiness reducer plus bounded dispatch scan using count and monotonic deadline; no recursive driver | Kernel | Wave 2 |
| Conditional dependency | Translate | closed typed dependency/wait union and durable observations; Markdown is import-only | wire/Kernel | Waves 1-2 |
| Prior-attempt context | Translate | bounded `AttemptOutcomeV1` referenced by a successor Context Capsule | Kernel | Waves 2 and 9 |
| Merged branch pruning | Translate | preservation-gated, expected-OID ref-retirement effect with reconciliation and tombstone | BulletGit + Kernel effects | Waves 4-5 |
| Jain handoff | Translate | typed `WorkReferralV1`; read/import projection first, local command lifecycle after the Wave 6 API command gate, and cross-repository enactment only as a Wave 11 saga | wire/Kernel/Portal | Waves 6 and 11 |
| Reviewer checkbox | Translate | typed review intent and command-backed decision bound to exact Candidate/policy/Evidence; durable review/adjudication remains evolution work | reviewer principal → Kernel persistence → Portal projection; verifier remains Evidence-only | Wave 6 command/API; Wave 9 review/adjudication |
| One coordinator | Adopt as UX | one operator liaison over read-only queries, then scoped typed command submission and later team recipes | Kernel/Runner/farmd/MCP/Portal | Wave 6 reads/command gate; Wave 9 cognition/orchestration |
| Python/HTML dashboard | Refuse | generated contracts and small Rust/React modules | — | — |
| Git/files/cache scraping in web server | Refuse | admitted adapters and ledger projections | — | — |
| Direct `git push --delete` | Refuse | BulletGit effect protocol | — | — |
| Branch name as claim/history | Refuse | lease/fence/work/Candidate identities | — | — |
| Markdown/TSV as scheduler authority | Refuse | closed wire DTOs and normalized rows | — | — |
| Review UI as Evidence | Refuse | independent review/evidence authority | — | — |
| Beads/Omnigent control plane | Refuse | existing Kernel control plane | — | — |
| Graphify or write-capable Serena | Refuse | exact-source derived artifacts and existing guarded provider boundary | — | — |

## Target product experience

### Information architecture

The Shift Brief should become the default home. It is a composition over
existing product truths, not a sixteenth sovereign operational surface.
Control Tower remains a deep link while the new work-oriented navigation is:

- `#/` — Shift Brief;
- `#/work` — filterable work pipeline/list/graph;
- `#/work/<id>` — exact work subject, dependencies, attempt history,
  acceptance, Candidate, Evidence, effects, and activity;
- `#/activity` — correlated event and audit timeline;
- `#/operations/*` — grouped fleet, sessions, context, merge, quality, and
  audit views;
- a persistent conversation drawer for the single operator liaison.

These routes are views over the existing surface catalog, not an implicit
catalog expansion:

| Route | Existing surface ownership |
| --- | --- |
| Shift Brief | Control Tower composition plus Merge Rail, Quality Lab, and Incidents & Audit facts |
| Work list/detail/graph | Mission Graph, Live Attempt, Merge Rail, and Quality Lab |
| Activity | Incidents & Audit |
| Operations | the existing Fleet, Session Supervisor, Context Lineage, Merge Rail, Quality Lab, and Incidents & Audit surfaces |
| Conversation drawer | a separate Wave 9 cognition capability; read-only overlay in Wave 6, `OUT_OF_PROFILE` until its catalog/invariant ADR is approved |

All fifteen current hash routes keep stable aliases or typed redirects. Direct
reload, bookmark, back/forward navigation, query/filter restoration, subject
drawer restoration, unknown hashes, focus transfer, and focus return are
compatibility tests. Any true surface-catalog expansion requires a reviewed ADR
and invariant/gap-profile update rather than a navigation-only edit.

Hash routing should remain initially because packaged farmd currently serves
only the entry document and assets. History routing requires an explicit server
fallback and separate security review.

### Shift Brief layout

The screen should answer “what is true, what is stuck, and what may I do next?”
without requiring the operator to know repository internals.

```text
+-----------------------------------------------------------------------+
| selected profile · exact family subject · observed time · freshness   |
+-----------------------------------------------------------------------+
| Mechanical | Evidence | Review | Deployment | Survival                |
+-----------------------------------------------------------------------+
| Exact release blockers              | Next authorized actions         |
| claim · subject · why · acceptance  | preview · authority · owner     |
+-----------------------------------------------------------------------+
| Queue -> Attempt -> Candidate -> Proof -> Delivery -> Integration     |
|                                                -> Deployment/Survival |
+-----------------------------------------------------------------------+
| Blocked/deferred work | Recent outcomes | Referrals | Incidents       |
+-----------------------------------------------------------------------+
| Persistent agent drawer: ask, inspect, draft; never silently execute  |
+-----------------------------------------------------------------------+
```

Required behavior:

- The five truths are separate fields and visual groups. Mechanical green
  cannot visually imply evidence, review, deployment, or survival green.
- Every blocking line opens to its exact gap/requirement, subjects, evidence
  class, observed evidence, invalidation, owner, and next authorized action.
- Every count declares scope and watermark. Counts from different clocks are
  never added, compared, or shown as one total.
- `UNKNOWN`, `STALE`, `CONTRADICTORY`, `PENDING`, and `OUT_OF_PROFILE` have
  distinct text, icon, and accessible names; color is supplementary.
- Empty is a proven empty collection at an exact watermark. It is not the
  fallback for an error or absent subject.
- The default ordering is release impact, then blocker age/deadline, then
  stable work identity. The operator can filter without changing authority.
- Risk, effort, release blocking, acceptance completeness, and confidence are
  separate dimensions. Zero acceptance criteria must not render as low risk or
  low effort.

### Work pipeline

The work screen should offer three synchronized projections of one server-side
snapshot:

- a compact left-to-right pipeline for operational scanning;
- an accessible table for precise filtering, sorting, and keyboard use;
- a dependency graph only when it materially clarifies blocking.

Each card names the durable work ID, exact revision, state, blocking condition,
attempt/candidate subject if one exists, owner, age, deadline, and snapshot
watermark. Cards do not infer Git or CI state in the browser. Selecting a card
opens a subject drawer rather than navigating through unrelated raw JSON pages.

### One-agent experience

“Communicate with one agent” is a product interaction, not an authority merge.
The browser does not run `bullet-mcpd`, a model, or provider credentials. The
runtime path is:

```text
Portal
  -> authenticated farmd conversation/command API
  -> durable Kernel turn command
  -> Runner-owned liaison process with a scoped provider identity
  -> stdio read-only bullet-mcpd / admitted farmd queries
  -> durable settled turn/provenance record
  -> farmd snapshot/SSE projection back to Portal
```

Farmd does not inherit the provider credential, and the browser receives only
bounded deltas plus durable turn/command identities. Kernel owns create,
resume, cancel, expiry, concurrency, and settlement. Provider response loss is
`UNKNOWN`; browser reload resumes by turn ID rather than starting a second
turn. Streamed deltas are presentation-only until the settled turn record is
durable.

The liaison evolves in four gated levels:

1. **Read-only investigator.** It queries Shift Brief, work detail, and audit
   through closed MCP tools. Every answer cites exact subject IDs and the
   snapshot sequence. A stale sequence forces refresh.
2. **Drafting assistant.** It can draft a plan, dependency, referral, or command
   preview. The draft is explicitly non-authoritative and names its intended
   scope, cost, risks, and required approvals.
3. **Scoped command submitter.** Only after G2-G4 and the MCP command gate close,
   including the complete Wave 6 command/API receipt, a separate machine
   principal can submit an already-reviewed generated DTO.
   The response is a durable command ID in `PENDING`; response loss is
   reconciled by command/idempotency ID and never blindly retried.
4. **Single facade over team recipes.** In Wave 9, T0 remains the stable
   fallback while specialist roles or a `TeamRecipeV1` may execute behind the
   liaison. The UI can hide that complexity by default, but provenance,
   dissent, budgets, Candidates, and independent verification stay separately
   inspectable.

No Mission, decision, question, scope grant, Candidate, Evidence, or approval
may exist only in a provider transcript. Each settled agent answer uses a
machine-validated structured response envelope: every factual claim references
the exact tool-result ID, subject, and watermark that was actually returned,
while inference is visibly labeled. Invented IDs, unreturned citations, mixed
watermarks, stale citations, and tool-output prompt injection refuse settlement.
Streaming deltas are ephemeral; the settled, redacted turn/provenance record
and any accepted command are durable Kernel subjects.

### UI engineering quality bar

The Portal implementation should use feature-oriented modules, shared truth
and observation components, and the generated API client:

```text
src/features/shift-brief/
src/features/work-pipeline/
src/features/evidence/
src/features/activity/
src/features/conversation/
src/components/TruthBadge.tsx
src/components/ObservationMeta.tsx
src/components/SubjectLink.tsx
src/components/CommandTimeline.tsx
```

Add a small sequence-aware query cache with request cancellation,
deduplication, stale/fresh policy, and event-driven invalidation. SSE events are
refresh signals; opaque event bodies do not become browser authority. Avoid a
framework migration until route and query requirements prove the need.

The release bar includes WCAG 2.2 AA, semantic landmarks, skip navigation,
route title/focus management, `aria-current`, table captions and headers,
live-region announcements, visible focus, reduced motion, responsive overflow,
and keyboard-complete drawers/dialogs. Use packaged local assets, precompiled
validators, and a CSP that removes `unsafe-eval` and inline style dependence.
Zero known WCAG A/AA violations are allowed; severity labels cannot waive an
A/AA failure.

## Working contracts

Names below are design handles, not approved wire names. Wave 1 freezes the
exact schema, canonicalization, identifiers, enums, and compatibility rules.
Shared domain records have one `bullet-wire-v1` authority. Kernel
`contracts/openapi.yaml` remains the HTTP API source of truth, and the Portal
transport client remains generated from it. A field is never independently
authored in both sources; references/adapters make the boundary explicit. Any
proposal to generate OpenAPI from `bullet-wire` first requires an ADR changing
the repository-local contract rule and proving there is still one source per
type with no circular generation.

| Contract | Minimum content | Authority and invariants |
| --- | --- | --- |
| `ShiftBriefSnapshotV1` | selected profile; exact installed family/registry/report subjects; issuer/signer role; five truth records; proof gaps; advisory scorecard facts; scope; section clocks; trusted generation time; generation/high-water identity | Hub derives and signs it from one explicit check operation and exact admitted receipt registry. Kernel admits it through a named command/principal, verifies the signer/trust root, installed family/profile/registry, freshness and monotonic high-water, and projects without recomputation. Forged, replayed, superseded, future-dated, clock-rollback, stale, invalid, or mixed input is `UNKNOWN` and cannot upgrade authority |
| `AcceptanceGapV1` | gap/requirement IDs; exact subject; claim; why; acceptance; required/observed evidence class; blocker; owner; invalidation; action preview or explicit none | One row is falsifiable and stable. Any action preview binds principal/role, subject and policy revisions, approval requirements, expected revision, and expiry; it is refetched before confirmation and is not itself a capability |
| `MissionPipelineSnapshotV1` | bounded mission/plan/work summary; dependencies; current attempts/Candidates/Evidence/effects/checks/integration/deployment observation; excluded subjects; source/sequence/time; child-page cursors | Built in one SQLite read transaction. Deterministic order and one watermark; histories are paged rather than embedded without bound. Otherwise return a typed refusal or explicit clock vector |
| `WorkDependencyV1` | graph revision; predecessor and successor work IDs; closed dependency kind; condition subject; creation authority | Self, missing, duplicate, and cyclic edges refuse during graph-revision admission. An edge is immutable within the revision |
| `WaitConditionV1` | kind; exact awaited subject; expected observation; current observation status; source/sequence/time; reevaluation trigger; deadline/escalation | `UNKNOWN` and `CONTRADICTORY` never satisfy. Trusted time, operator acts, capacity, external checks, predecessor outcomes, and receipts are distinct variants |
| `ReadinessDecisionV1` | work/revision; evaluated dependency set; decision; reasons; evaluation sequence | Kernel reducer is the only readiness authority. Ready means every required condition is satisfied at one transaction boundary |
| `AttemptOutcomeV1` | attempt/fence; terminal class; bounded conclusion; exact failure/evidence/log/checkpoint subjects; retry recommendation; supersession | The successor references the outcome digest. Raw transcript is neither required context nor durable authority |
| `RefRetirementIntentV1` | repository/ref; expected OID; Candidate/Attempt owner; integration observation; preservation receipt; operation ID | BulletGit rejects missing/moved/ambiguous subjects. No prefix/name-only target, no deletion before preservation, no blind retry |
| `RefRetirementObservationV1` | requested and observed OIDs; remote result; read-back; status; tombstone subject | Response loss is `UNKNOWN`; reconcile before a new write. Success includes durable read-back and tombstone |
| `WorkReferralV1` | origin repository/work/attempt; target repository; rationale; priority; dependency/block relation; requested work; evidence; acceptance; deadline; lifecycle | Markdown is import/export only. Proposal, acknowledgement, acceptance, rejection, resolution, reopen, and escalation are durable idempotent transitions |
| `ReviewIntentV1` / review record | exact Candidate/integration subject; evidence digest; policy revision; reviewer principal; decision; supersession | A UI action produces a command, not Evidence. Subject or policy change invalidates the review; only the independent durable record can render complete |
| `ConversationTurnRecordV1` | conversation/turn IDs; context sequence; attached subject IDs; prompt/response digests; redacted content reference; tool-call provenance | Provider text cannot grant tools or scope. Sensitive content is separated from general event/audit projection |
| `AgentResponseEnvelopeV1` | settled claim/inference segments; exact tool-result/subject/watermark citations; model/provider subject; turn identity | Host validates every citation against results returned in that turn. An unsupported claim cannot settle as an observation |
| `ActionConfirmationV1` | human/approved principal; canonical command DTO digest; subject/scope/expected revision; policy generation; cost/destructive class; approval references; expiry/nonce | The liaison principal cannot mint it. One-byte payload change, stale preview, replay, principal/session swap, or expired approval refuses before command persistence |

### Dependency semantics

Do not add a generic `Blocked` boolean. Preserve `Pending` as the durable work
state and derive ready/not-ready plus exact reasons from typed conditions.

The reducer must:

1. validate the entire immutable dependency graph at plan-revision admission;
2. evaluate conditions in one Kernel transaction at a recorded sequence;
3. persist the decision and reasons with the state/event/outbox transition;
4. wake work from events, trusted-time deadlines, or explicit observations;
5. distinguish permanent failure, temporary deferral, capacity backpressure,
   missing authority, external unknown, and required operator action;
6. leave work pending when any required observation is unknown or stale;
7. enqueue a work package exactly once when all conditions become satisfied.

The ready row carries the exact graph revision and every condition observation
revision/freshness used by the decision. `claim_ready` revalidates those values
inside the same transaction that mints the lease/fence. If a predecessor,
external check, approval, time window, or capability is revoked, expires,
stales, or revises between enqueue and claim, no lease or Attempt is minted and
the row returns to pending with a new reason. Terminal predecessor failure,
cancellation, expiry, and escalation each have an explicit policy transition so
dependents cannot remain pending forever without a visible owner/action.

The dispatcher may inspect another ready item after one candidate becomes
deferred, but the loop is bounded by both a configured count and a monotonic
time budget. It revalidates freeze, lease, policy, capacity, and trusted time
between selections. Only a wholly local admission/readiness check may defer
without minting an Attempt or consuming execution budget. Any provider,
session, network probe, or other external work first creates a fenced Attempt
and reserves/settles its applicable budgets, even when it encounters rate
limits, timeout, crash, or response loss. Observation work has its own bounded
and settled cost.

### Cleanup semantics

Nightshift's branch pruning becomes this state machine:

```text
eligible candidate
  -> exact integration observation
  -> sealed preservation receipt
  -> retirement intent with expected remote OID
  -> BulletGit effect permit
  -> remote compare-and-swap/delete
  -> read-back
  -> retired tombstone | UNKNOWN | ORPHANED_REMOTE
```

There is no abandonment bypass in this profile. The implementation must never
enumerate a prefix and delete by name alone, and it must refuse default,
protected, integration, tag, reserved-namespace, non-ephemeral, non-enrolled,
or non-owned refs. Enrollment at Attempt creation binds the exact ref purpose
and owner. Every permit and preservation receipt is authenticated, current,
purpose-bound, and subject-exact; forged, stale, replayed, or wrong-purpose
material refuses.
Every retry reads remote truth first. A moved ref is `ORPHANED_REMOTE`, not a
reason to force deletion. Lost response is reconciled without issuing a second
logical effect. Secondary repositories use the same port and proof suite; there
is no special weaker cleanup path.

## Repository ownership

### `bullet-farm`

- Freeze shared domain records and hostile fixtures in `crates/bullet-wire` and
  the generated contract catalog while preserving Kernel OpenAPI ownership for
  HTTP DTOs.
- Refactor release-truth JSON and Markdown to render from one typed model. Do
  not parse `release-truth.generated.md`.
- Produce a portable `ShiftBriefSnapshotV1` from one explicit profile/family/
  registry invocation, with exact input identity and section clocks.
- Keep scorecard facts advisory and visibly separate from release authority.
- Extend invariant/corpus coverage only in the same reviewed change as the
  relevant code/test anchor. Never mark implementation complete from this plan.

### `bullet-kernel`

- Add normalized dependency/wait/readiness, attempt outcome, referral/review,
  and later conversation records under domain/application/adapters ownership.
- Add append-only migrations; prototype data is export/verify/import, never
  silently adopted.
- Admit the exact Hub snapshot as an observation and project it without running
  a sibling checkout or reimplementing the release evaluator.
- Add server-composed Shift Brief and mission-pipeline projections from one
  read transaction.
- Add only the smallest `/api/v1` reads and generated command kinds supported by
  durable subjects.
- Extend `bullet-mcpd` with closed read-only tools first. A command tool remains
  separately gated by its existing security requirements.

### `bullet-git`

- Implement ref retirement as an authority-checked, expected-OID,
  preservation-gated effect with reconciliation and a cleanup tombstone.
- Bind every target to repository, ref, exact tagged OID, Candidate/Attempt,
  integration observation, and permit. Never accept a prefix or ambient cwd.
- Preserve only Git workspace, journal, checkpoint, preservation, cleanup, and
  recovery artifacts, and return typed receipts/observations for Kernel to
  link. General Attempt conclusions remain solely in Kernel.

### `bullet-portal`

- Consume generated DTOs only; do not hand-edit generated clients.
- Add Shift Brief, work pipeline/detail, activity, and the persistent liaison
  drawer in feature-oriented modules.
- Replace client-side multi-request truth composition with atomic server views.
- Retain honest loading/empty/unknown/stale/contradictory states and make every
  action show its command lifecycle.
- Keep the six missing operational surfaces `UNKNOWN` or selected-profile
  `OUT_OF_PROFILE` until Kernel has durable subjects.

### Jeryu and providers

Use only the pinned Jeryu family at `/home/ubuntu/jain-split/jeryu-split` and
only through reviewed tags/capabilities. Do not patch a live forge to mimic
Nightshift deletion. Providers remain read-only proposal producers until the
Kernel/Runner/BulletGit/verifier/effect gates explicitly authorize more.

## Dependency-ordered delivery plan

| Slice | Outcome | Roadmap fit | Hard predecessor |
| --- | --- | --- | --- |
| NS-0 | Source pin, fusion boundary, traceable plan | Wave 0 documentation | exact read-only study |
| NS-1 | Shared domain/HTTP contract freeze with one authority per type | Wave 1 | frozen clean-family subject, OD-D, OD-E, reviewed Jeryu tag, and contract governance |
| NS-2 | Durable dependency graph and readiness reducer | Wave 2 | NS-1 and Kernel durable-authority baseline |
| NS-3 | Bounded blocked continuation and operational attempt outcomes; successor Context Capsule later | Wave 2; Wave 9 context integration | NS-2, budgets, leases/fences; G15 for successor cognition |
| NS-4 | Preservation-gated ref retirement | Wave 4 semantics; Waves 5/7 local proof; Wave 8 Jeryu; Wave 10 hosted forges | online BulletGit authority and forge-specific effect reconciliation |
| NS-5 | Typed Shift Brief model, then signed admission | Wave 1 model; Wave 6 admission | NS-1, exact release-report subject, trusted signer/time/high-water |
| NS-6 | Atomic farmd/MCP projections | Wave 6 | NS-2, NS-3, NS-5 and durable rows |
| NS-7 | Accessible Shift Brief and managed pipeline UI | Wave 6 | NS-6 and deferred rendered-UX proof lane |
| NS-8 | Review plus referral read/import, local lifecycle, durable adjudication, then saga enactment | Wave 6 read/import and command-gated local lifecycle; Wave 9 review/adjudication; Wave 11 cross-repo enactment | distinct reviewer, API command/approval, evolution persistence, then saga prerequisites |
| NS-9 | Single-agent read facade, then durable conversation/scoped commands/team recipes | Wave 6 reads/command gate; Wave 9 cognition/orchestration | NS-6; mutation requires G2-G4, Wave 5, and the complete Wave 6 command/API receipt |
| NS-10 | Packaged/fault/live/evolution/saga proof | Waves 7-11 | all applicable earlier slices |

### Counting receipt and operator-predecessor matrix

| Slice | Counting condition / semantic verifier | Required external/operator predecessor |
| --- | --- | --- |
| NS-0 | none; documentation-only component observation | none |
| NS-1 | Wave 1 kind-specific schema/tag and `BaselineReceiptV1` admission over the unchanged frozen family subject | OD-D, OD-E, reviewed signed Jeryu tag |
| NS-2/NS-3 | G2 `TRANSACTION_PROOF` plus G3 durable-authority receipt; Hub kind-specific semantic verifier | NS-1; no prose/demo substitution |
| NS-4 | G4 online-authority Candidate transaction; Wave 7 boundary-12 cleanup in `TRANSACTION_PROOF`; forge-specific `LIVE_PROOF` at Waves 8/10 | Jeryu custody at Wave 8; OD-C and other named forge decisions at Wave 10 as applicable |
| NS-5/NS-6/NS-7 | G13 selected-profile durable-or-`OUT_OF_PROFILE` Portal condition plus G14 signed-dispatch/API reconciliation and packaged browser proof | installed signed family; Wave 6 auth policy; Wave 8 OD-A, OD-B, OD-D, OD-E for live self-hosted proof |
| NS-8 | local decisions count only within the existing Wave 6 G13/G14 command/API condition; durable review/adjudication requires the Wave 9 G11/G15 evolution receipt; cross-repository enactment requires G18 `saga-v1` | reviewer/approval custody; `team-v1` must pass before `saga-v1` |
| NS-9 | read facade counts only with the G13/G14 Portal/API condition; durable cognition/team recipes require G11/G15 `evolution-v1` receipts | `self-hosted-v1` PASS; OD-H only at the exact expiring canary point |
| NS-10 | exact `self-hosted-v1`, provider/forge/platform, `evolution-v1`, `universal-v1`, `team-v1`, and `saga-v1` semantic verifiers as selected | Wave 8 OD-A/B/D/E; Wave 10 OD-A/C/I/J and independent provider/forge/platform receipts; `team-v1` before `saga-v1` |

OD-F and OD-G, if resolved, clear no product or release gate by themselves.
[`0013-operator-decision-register.md`](../decisions/0013-operator-decision-register.md)
remains the source for operator-decision meaning and current status.

### NS-0 — preserve the decision boundary

Deliverables:

- this source-pinned adopt/translate/refuse plan;
- a docs index link;
- no imported runtime byte and no changed status assertion.

Exit: documentation checks pass, the exact paths are handed off under a machine
claim, and every release profile remains `BLOCKED`.

### NS-1 — freeze contracts before screens or schedulers

Deliverables:

- approve exact domain-wire and HTTP DTO ownership plus recursively closed
  schemas for Shift Brief,
  acceptance gaps, work dependencies/waits, readiness decisions, attempt
  outcomes, ref retirement, review, and referral;
- specify canonical JSON, domain-separated identity, safe integers, tagged OIDs,
  time semantics, forward compatibility, and maximum sizes;
- generate Rust/JSON Schema from shared domain authority and HTTP/TypeScript
  transport from Kernel OpenAPI, with drift checks proving no duplicate source;
- publish immutable family/wire subjects as required by Wave 1.

Exit tests:

- duplicate/unknown fields, unsafe/non-finite numbers, invalid Unicode, invalid
  enum/tag, oversized fields/collections, wrong family/schema/profile, and
  non-canonical bytes refuse in Rust and TypeScript;
- two clean generations are byte-identical;
- every derived artifact names the exact generator/input subjects and refuses
  stale or partial regeneration.

### NS-2 — make dependencies authoritative

Deliverables:

- normalized dependency/wait rows and append-only migration;
- immutable plan-revision graph admission with cycle/missing/self checks;
- one pure readiness reducer shared by dry-run/execution decisions;
- atomic state, event, audit-link, and ready-queue update;
- event- and trusted-time-driven reevaluation.

Exit tests:

- self, missing, duplicate, and cyclic edges refuse without partial rows;
- the Nightshift-style 185 → 184 → 190 chain holds transitively and releases
  exactly once, in stable order, after its exact conditions resolve;
- unknown external checks, clock ambiguity, or lost observation stay pending;
- an observation satisfied at enqueue and revoked/staled before `claim_ready`
  produces no lease or Attempt;
- crash/replay, concurrent satisfaction, cancellation, revision replacement,
  and restore do not double-enqueue or lease blocked work;
- property tests cover graph insertion order and topological permutations.

### NS-3 — continue safely and remember why

Deliverables:

- bounded ready-item scanning after a pre-attempt deferral;
- count, monotonic-time, capacity, and fairness limits;
- durable operational `AttemptOutcomeV1`; the bounded successor Context Capsule
  reference remains a Wave 9/G15 integration;
- explicit rate-limit/session-capacity behavior shared across all providers and
  lanes.

Exit tests:

- zero, one, limit, and limit-plus-one block cases prove no off-by-one;
- freeze, lease, policy, authority, capacity, and trusted time are rechecked
  between selections;
- repeated local blocker observations are idempotent and do not burn execution
  budget; any provider/session/probe path proves Attempt reservation/settlement;
- rate-limit, session-limit, provider crash, timeout, cancel, and response loss
  produce the correct outcome class;
- a retry receives the exact predecessor conclusion/digest/fence and no
  unrelated or transcript-only state;
- fairness prevents a permanently deferred item or lane from starving ready
  work.

### NS-4 — retire refs without deleting history by accident

Deliverables:

- BulletGit ref-retirement intent, permit, journal, reconciliation, and
  tombstone;
- expected-old-OID remote operation and post-operation read-back;
- exact integration plus mandatory preservation prerequisite;
- one semantic port for every repository/forge capability profile, with each
  profile remaining blocked until its own roadmap proof wave.

Exit tests:

- absent preservation, stale/moved OID, wrong repository, wrong owner,
  branch-name/prefix collision, missing integration observation, and ambiguous
  ancestry refuse before mutation;
- default/protected/integration/tag/reserved/non-enrolled refs and forged,
  stale, replayed, or wrong-purpose permits/receipts refuse before mutation;
- a concurrent remote update can never be deleted;
- response loss before/after mutation, already absent ref, third-party value,
  auth expiry, restart, and unavailable read-back settle to exact success,
  `UNKNOWN`, or `ORPHANED_REMOTE` without a duplicate logical effect;
- tombstone persistence failure prevents cleanup completion;
- the same suite covers primary and secondary repositories.

### NS-5 — produce one exact Shift Brief

Deliverables:

- one typed release-truth model that renders both stable JSON and the existing
  Markdown diagnostic;
- exact selected profile, installed family subject, registry subject,
  issuer/signer role, policy/tool inputs, facts, evidence completeness, release
  review, deployment, survival, trusted time, and monotonic generation fields;
- advisory scorecard facts attached with their own subject/clock;
- explicit portability/redaction and invalidation rules;
- one named authenticated Kernel import command with signer/trust-root,
  installed-family/profile/registry selection, trusted-time freshness,
  supersession/high-water, replay/downgrade, and invalidation rules; storage
  cannot promote the report.

Exit tests:

- JSON/Markdown golden tests share one model and one row per selected gate;
- subject or receipt mutation changes identity and invalidates prior admission;
- unreceipted or partially receipted profiles render exact missing claims, not
  the blanket statement that no receipt exists;
- incompatible scorecard/release clocks are refused or separately displayed;
- forged-but-well-formed, old-valid replay, wrong installed family, wrong
  registry/profile, superseded generation, future time, and clock rollback
  refuse admission;
- absolute paths, host names, secrets, and nondeterministic time do not leak in
  portable mode;
- two runs from the same exact subject are byte-identical.

### NS-6 — expose atomic projections, not a second control plane

Initial API shape should remain small:

- `GET /api/v1/shift-brief`;
- `GET /api/v1/work-items` with bounded stable sort/filter/cursor;
- `GET /api/v1/work-items/{id}` or one exact mission-pipeline snapshot route;
- typed SSE invalidation/resync metadata;
- read-only MCP tools for Shift Brief, gap detail, work detail, and next
  authorized actions.

Deliverables:

- one-transaction SQLite projections with deterministic ordering;
- exact `data`, `as_of_sequence`, `observed_at`, `source`, scope, and response
  header agreement;
- closed event and command discriminators with subject/revision/correlation;
- body, collection, cursor, replay, queue, and deadline bounds;
- `RESYNC_REQUIRED` for unrecoverable stream gaps.

Opaque cursors bind snapshot watermark, installed family/profile, filter, sort,
query version, principal scope, and expiry. Pagination either holds the exact
repeatable snapshot or returns `RESYNC_REQUIRED`; it never silently crosses an
epoch. Work-detail histories are paged, and every MCP/API decoded response has
an explicit maximum no larger than the current 256 KiB MCP ceiling unless a
separately reviewed bound replaces it.

Until authenticated projection reads and resumable SSE exist, this feature is
loopback-only and off-loopback is typed `OUT_OF_PROFILE`. Same-origin is not
authentication. Wave 6 adds OIDC Authorization Code + PKCE, RBAC, session
revocation/logout, CSRF rotation, rate limiting, MFA/two-person approval where
required, and scoped authenticated SSE.

Exit tests:

- missing/corrupt/stale/wrong-family/wrong-profile Hub observation is visible as
  `UNKNOWN` and cannot grant release authority;
- no farmd path shells into Hub, reads a sibling checkout, invokes ambient Git,
  or scrapes Markdown;
- mixed-watermark composition refuses;
- duplicate, out-of-order, truncated, and replay-window-exceeded events force
  the correct refresh/resync behavior;
- empty, unknown, stale, contradictory, and out-of-profile fixtures are
  distinct;
- pagination is stable through concurrent inserts and request sizes are
  bounded.
- cursor/watermark/filter/sort/principal/expiry mutation, last-accepted versus
  first-refused response size, and child-history overflow refuse or resync;
- authenticated resume covers `Last-Event-ID` versus query-cursor conflict,
  session expiry/revocation, permission change, slow consumers, bounded client
  and server queues, reconnect storms, and cross-scope disclosure refusal.

### NS-7 — deliver the operator-grade web GUI

Deliverables:

- default Shift Brief, work pipeline/table/detail, activity timeline, and
  shared observation/subject components;
- sequence-aware query cache with real request cancellation and SSE
  invalidation;
- stable deep links to exact subjects and gaps;
- responsive desktop/mobile designs for PASS/HOLD/BLOCKED/UNKNOWN/STALE;
- a closed UI truth-state union covering `PASS`, `HOLD`/`BLOCKED`, `UNKNOWN`,
  `STALE`, `CONTRADICTORY`, `PENDING`, `OUT_OF_PROFILE`, loading, proven empty,
  unavailable/offline, and resync;
- local assets, precompiled validation, hardened CSP, content redaction, and no
  raw JSON as the main interaction.

Exit tests:

- component tests prove no unknown/pending state can use a success label;
- TypeScript exhaustiveness checks and visual/semantic baselines cover every
  member of the closed truth-state union with no default-to-success or
  error-to-empty fallback;
- mocked browser tests cover mechanical PASS with release HOLD, blocker
  drill-down, mixed-clock refusal, stale data, event gaps, and command states;
- real packaged-farmd tests cover same-origin reads, bootstrap/session expiry,
  reconnect, `PENDING` to `UNKNOWN`, and positive `VERIFIED` only when a real
  durable worker can produce it;
- reload, route navigation, back/forward, and session expiry while a command is
  `PENDING` resume bounded monitoring/reconciliation of the same durable command
  ID and prove that no second POST occurs;
- automated and manual review report zero known WCAG A/AA violations; keyboard,
  focus, route title,
  live-region, reduced-motion, responsive overflow, and table semantics are
  asserted and manually reviewed;
- intentional desktop/mobile visual baselines exist for all truth states;
- packaged-header tests prove CSP contains neither `unsafe-eval` nor unsafe
  inline script/style allowances; precompiled validators still work and
  hostile content/URLs cannot create script, style, or DOM injection;
- before UI implementation, a checked-in performance budget pins CI hardware,
  browser versions, a 10,000-item server fixture, 100-row maximum client page,
  graph/page bounds, event rate/duration, p50/p95 response/interaction/render
  thresholds, heap/queue ceilings, and typed overload behavior. Chromium,
  Firefox, and WebKit must meet that exact budget; “recorded bounds” alone do
  not pass.

### NS-8 — make review and referrals durable

Deliverables are split by authority:

- Wave 6 exact-subject review intent/command record with idempotency, expected
  revision, reviewer identity, policy, Evidence digest, decision, and
  supersession; it adds no new receipt kind and counts only through the existing
  G13/G14 command/API condition;
- Wave 6 read/import validation and projection for `WorkReferralV1`;
- only after the complete Wave 6 command/API gate, local proposal,
  acknowledge/accept/reject/resolve/reopen lifecycle;
- Wave 9 durable review/adjudication persistence under G11/G15 evolution;
- human approval and escalation projections;
- only in Wave 11, dependency-aware cross-repository saga execution,
  compensation, and forward repair.

Exit tests:

- a checkbox/local storage/browser success cannot qualify Evidence;
- Candidate review, verifier Evidence, and Hub Release Review are three
  non-substitutable subjects: the verifier cannot review itself, Candidate
  review cannot satisfy a release-review fact, and release review cannot
  fabricate Evidence. A distinct reviewer principal decides; Kernel persists;
  Portal submits/projects; Hub consumes only an admitted semantic record;
- Candidate, evidence, policy, or integration-subject change invalidates the
  prior review;
- unauthorized target, placeholder template, duplicate submission, response
  loss, stale revision, expiry, accept/reject/resolve/reopen, and crash at each
  transition preserve one durable truth;
- high-risk approval requires two distinct currently eligible principals;
  same-principal double approval, revoked role/MFA, insufficient approvals,
  payload/policy revision after approval, and concurrent approval races refuse
  or reconcile without execution;
- cross-repository partial integration is explicit and never presented as
  atomic success.

### NS-9 — add the liaison without collapsing authorities

Deliverables:

- read-only MCP tools with exact subject/sequence citations;
- durable, bounded, redacted conversation provenance;
- preview/confirm UX for typed proposals;
- after G2-G4, Wave 5, and the full Wave 6 command/API receipt, a separately
  authenticated, narrowly scoped command tool;
- in Wave 9, T0 fallback and optional team recipes behind one visible liaison.

Exit tests:

- prompt/repository content cannot expand scope, tools, network, credentials, or
  approval;
- turn create/resume/cancel, reload during streaming, bounded concurrency, and
  provider response loss preserve one durable turn identity;
- stale context forces refresh/replan;
- direct Git/effect/self-verification requests refuse;
- one confirmed intent produces at most one durable command through idempotency
  and response-loss reconciliation;
- one-byte DTO substitution, stale/expired preview, confirmation replay,
  session/principal swap, and unknown/malformed/aliased tool call refuse;
- transcript deletion does not delete Mission, decision, question, Candidate,
  Evidence, review, or command truth;
- specialist provenance, dissent, budgets, and verifier independence remain
  inspectable even when the default UI shows one agent.

### NS-10 — bind the experience to real proof

The feature is not complete when a dashboard renders. It joins the existing
roadmap exits:

- Wave 7 twelve-boundary offline transaction and packaged-origin browser proof;
- Wave 8 explicit Claude and Jeryu live checkpoints;
- Wave 9 study/shadow/canary/rollback proof for team recipes and the agent
  facade;
- Wave 10 independent provider, GitHub/GitLab/Jeryu forge, and platform profile
  proofs plus compatible `universal-v1` composition;
- Wave 11 partition/failover and cross-repository saga proof.

The morning brief must be generated after an unattended fault-injected shift
and point to the exact observed Mission, Attempt, Candidate, Evidence, effects,
integration, deployment, and survival subjects. No synthetic UI fixture can
substitute for these receipts.

## Proof matrix

| Layer | Mandatory positive proof | Mandatory hostile/fault proof |
| --- | --- | --- |
| Wire | generated Rust/schema/OpenAPI/TypeScript agree; two-run deterministic identity | duplicate/unknown keys, unsafe numbers, invalid tags/OIDs/times, oversize, wrong family/version |
| Domain graph | stable acyclic graph, exact-once readiness, deterministic replay, observation-bound atomic lease claim | self/missing/cycle, insertion permutations, unknown/revoked/stale condition, terminal predecessor, cancel/revision race |
| Scheduler | ready work progresses after defer within count/time/fairness bounds | off-by-one, freeze/policy/lease change between selections, rate/session limit, starvation |
| Attempt memory | exact bounded predecessor outcome reaches successor | wrong fence/digest, oversized transcript, stale/superseded outcome, missing artifact |
| Persistence | graph/outcome/command/event/audit/outbox commit atomically and replay | crash at transaction/WAL/outbox/restore boundaries, corrupt row, duplicate event |
| BulletGit cleanup | preserved exact ref is retired and read back with tombstone | concurrent move, stale OID, wrong owner, response loss, already absent, third-party value |
| Shift Brief | one exact profile/family/report and five distinct truths | partial receipts, mixed clocks, stale source, path/secret leakage, subject mutation |
| Projection/API | one-transaction snapshots, header/body watermark equality, snapshot/principal-bound cursor, authenticated resumable SSE | mixed epochs, cache/error-as-empty, cursor mutation/expiry, gap/replay overflow, auth/scope change, malformed/oversized response |
| Portal | accurate states, deep links, filters, responsive/a11y visuals | no green for unknown, XSS strings including `</script>`, Unicode, stale and contradictory data |
| Commands/review | 202 durable pending, exact revision, later verified read-back | CSRF/origin/auth/RBAC, duplicate, response loss, self-review, subject change |
| Conversation | host-validated claim-level citations, exact confirmation digest, redacted durable provenance, safe read tools | invented/unreturned/mixed-clock citations, prompt injection, secret reflection, stale context/preview, payload substitution, tool expansion, blind retry |
| End to end | packaged same-origin transaction and morning brief over exact subjects | every required fault class injected at each of all twelve transaction boundaries, restart/reconnect, observation loss, deployment drift |

The Wave 7 fault requirement is a boundary-by-fault matrix, not twelve chosen
examples: death, timeout, response loss, stale authority, freeze, clock
movement, ENOSPC, and restart are exercised at every one of the twelve roadmap
boundaries wherever the fault is semantically injectable, with an explicit
not-applicable justification otherwise.

Add a concrete identity-isolation fixture in which app, www, and automation
lanes in two repositories all use the same human todo number and similarly
shaped branch labels. Scheduler rows, attempts, blockers, outcomes, counts,
deep links, cursors, and Portal cards must remain distinct by full typed
repository/work identity.

For every implementation claim, run the repository's focused tests and
`bash scripts/ci-local.sh required`. From the Hub, the family handoff retains:

```bash
cargo run --locked --quiet --bin bullet-family -- doctor --json
just fast
just demo
```

`doctor --json` is allowed to refuse for a documented missing or unsupported
authority subject; that nonzero refusal must be preserved, not relabeled green.
Once the exact clean family subject and bootstrap prerequisites exist, run the
full family lane required by the active roadmap. Record tool versions, exact
heads/trees, zero-test/skip counts, sanitized artifact identities, claim,
handoff, and every remaining blocked condition.

## Gap traceability

Nightshift fusion is a cross-cutting contribution, not a replacement for the
G1-G18 roadmap.

| Gap | Contribution from this plan | What still closes the gap |
| --- | --- | --- |
| G1 signed install | packaged Shift Brief/Portal becomes part of the exact family subject | signed schema-3 install lifecycle and release receipt |
| G2 transaction | durable dependencies, command/review/referral transitions, fault cases | the full signed five-plane `TRANSACTION_PROOF` |
| G3 Kernel write path | readiness and agent commands use the production authority path | admitted lease transport, recovery, read-back, and durable-authority receipt |
| G4 BulletGit write path | replaces unsafe prune with exact ref-retirement semantics | online authority, immutable wire/Jeryu subjects, Candidate/integration proof |
| G5 providers | liaison exposes provider state without granting provider authority | provider-specific live conformance and policy/enrollment receipts |
| G6 Jeryu | pipeline shows exact Jeryu effect/integration observations | protected Jeryu capability and live effect receipt |
| G7 GitHub | same forge-neutral projection and retirement protocol | separately certified GitHub App profile |
| G8 security | hostile UI/API/agent/cleanup and accessibility gates are explicit | exact-subject security/Jankurai release receipts |
| G9 package | packaged-origin GUI and local assets are required | signed package, lifecycle, reproducibility, and target receipts |
| G10 containment | liaison/provider/effect boundaries retain sandbox requirements | target-specific S1/S2 containment receipts |
| G11 evolution | one liaison can front T0/team recipes without hiding provenance | Wave 9 study, shadow, canary, drift, rollback, and promotion receipts |
| G12 release evidence | Shift Brief projects exact selected-profile truth | current admitted semantic receipt set and explicit `check release` result |
| G13 Portal | directly supplies the default Shift Brief and managed work GUI | durable or `OUT_OF_PROFILE` subjects, packaged browser/accessibility proof |
| G14 farmd | directly supplies small atomic projections, pagination, typed SSE/MCP | production dispatch/reconciliation and API receipt |
| G15 cognition | attempt outcomes, context lineage, conversation provenance, later recipes | all durable cognitive/evolution rows and replay receipt |
| G16 GitLab | uses the same forge-neutral projection/effect semantics | independent GitLab.com and self-managed receipts |
| G17 team | one-agent facade remains stable across later remote execution | Wave 11 distributed authority/failover receipt |
| G18 sagas | typed referrals become dependency-aware cross-repo workflows | `team-v1` first, then distinct `saga-v1` compensation/forward-repair proof |

## Rollout and compatibility

1. Land schemas and read-only models before routes or UI. Unknown schema
   versions refuse; there is no permissive fallback parser.
2. Add farmd reads as loopback-only first; current same-origin projection/SSE
   reads are not authenticated. Missing data renders typed `UNKNOWN`; it does
   not silently fall back to scraped data. Off-loopback remains
   `OUT_OF_PROFILE` until the full Wave 6 auth/RBAC/SSE controls pass.
3. Make Shift Brief the default only after the packaged-origin, accessibility,
   mixed-watermark, and truthful-empty suites pass. Keep Control Tower deep
   links stable.
4. Introduce typed dependencies for new plan revisions. Existing prototype
   rows require explicit export/verify/import or remain on their old frozen
   subject; no silent migration manufactures dependencies.
5. Run ref retirement in observe/dry-run mode first using the same pure decision
   function. Mutation remains policy-disabled until expected-OID,
   preservation, response-loss, and read-back proofs pass.
6. Launch the liaison read-only. Drafting is visibly non-authoritative. Command
   submission remains absent until the transaction and MCP command gates close.
7. Preserve audit and telemetry for resyncs, stale sources, dependency wait
   duration, dispatch skips, retry outcomes, retirement unknowns, command
   reconciliation, and agent proposals. These are observations, not release
   receipts.

Rollback is append-only and subject-preserving: disable a route/command/policy
generation, revert the default navigation, or activate the prior healthy
routing generation. Never delete missions, outcomes, Candidates, Evidence,
reviews, commands, effects, or cleanup tombstones to make a UI rollback look
clean. Conversation content is encrypted and access-logged with explicit
retention/expiry, scoped redaction, and secret-erasure policy. Expiry or an
authorized content deletion preserves the immutable turn/command/provenance
identity and a deletion tombstone while cryptographically erasing protected
content; tests prove backups, projections, logs, and caches cannot recover an
erased secret.

## Principal risks and controls

| Risk | Control |
| --- | --- |
| Building a parallel Nightshift control plane | Kernel rows and commands are the only scheduler authority; no runtime Nightshift dependency |
| Portal green laundering | closed truth states, exact subjects/watermarks, unknown never green, browser is projection-only |
| Mixed-epoch dashboard | server-side atomic snapshot or explicit section clock vector; no client arithmetic across clocks |
| Destructive ref race | exact tagged OID, preservation, effect permit, compare-and-swap, read-back, tombstone |
| Retry context bloat or prompt injection | bounded typed outcome, digests, redaction, tool/scope policy outside provider text |
| One agent becoming hidden authority | Runner-hosted liaison principal has narrow tools; exact human confirmation digest, durable commands, specialists, reviewer, verifier, and effects remain separate |
| Cross-repo handoff pretending atomicity | proposal first; Wave 11 saga states expose partial integration, compensation, and forward repair |
| UI scope outrunning durable state | small route set; missing subjects stay `UNKNOWN`/`OUT_OF_PROFILE`; no designed 80-route expansion |
| Secret or untrusted-content leakage | separate sensitive store, scoped projections, CSP hardening, XSS/redaction fixtures, no external fonts |
| Proof reduced to screenshots | contract/domain/fault/packaged/live receipts remain mandatory; visual proof is only one layer |

## Tiered completion criteria

Nightshift fusion is not one hidden universal prerequisite. It has three
profile-specific exits.

### A. `self-hosted-v1` operator experience

- the packaged Shift Brief is default and separates all five truths;
- every displayed claim opens to its exact subject, requirement, evidence,
  freshness, blocker, owner, and revision/expiry-bound next action preview;
- work dependencies/waits drive atomic readiness-to-lease, bounded continuation
  is fair, and operational retries receive exact bounded outcomes;
- LocalBareForge/Jeryu ref retirement is integration-observed,
  preservation-gated, expected-OID, reconciled, and tombstoned;
- local review/referral projection and any enabled local lifecycle pass the
  distinct review plus Wave 6 command/API gates;
- farmd and Portal pass atomic projection, authenticated/loopback profile,
  packaged-origin, accessibility, CSP, responsive, security, race, scale, and
  truthful-state suites;
- the liaison is read-only unless its full scoped command gate passes; G15
  cognition and G18 sagas may remain explicit `OUT_OF_PROFILE`;
- Wave 7 boundary-by-fault proof and Wave 8 live checkpoints bind the morning
  brief to exact Candidate/Evidence/effect/integration/deployment/survival
  subjects.

Counting authority: G2 `TRANSACTION_PROOF`, G3 durable-authority, G4
online-authority Candidate/local forge proof, G13 durable-or-`OUT_OF_PROFILE`
Portal condition, G14 API reconciliation, and the exact selected
`self-hosted-v1` release condition.

### B. `evolution-v1` cognition and one-agent orchestration

- successor Context Capsules, encrypted/retained conversation provenance,
  claim-level citations, T0 fallback, optional team recipes, budgets, dissent,
  selection, independent review/verification, drift, and rollback are durable;
- the one-agent facade never hides or inherits specialist, reviewer, verifier,
  Candidate, Evidence, or effect authority;
- study, shadow, exact OD-H canary, rollback, and promotion receipts pass.

Counting authority: G11/G15 and the distinct `evolution-v1` semantic receipt.
This tier depends on `self-hosted-v1` and is not implied by Portal completion.

### C. `team-v1` then `saga-v1`

- remote authority/failover first passes `team-v1`;
- referral enactment across repositories then exposes partial integration,
  quarantine, compensation, forward repair, and survival without claiming
  false atomicity;
- the single liaison remains a facade over exact distributed subjects.

Counting authority: G17 `team-v1` first, then G18 `saga-v1` from a distinct
receipt set. Neither is a prerequisite for first GA.

Until the applicable tier and active roadmap exits are met, this document
describes the destination and sequence only. It never turns the current
`BLOCKED` release decision into a pass, and no vague reference to “named
semantic receipts” substitutes for the counting matrix above.
