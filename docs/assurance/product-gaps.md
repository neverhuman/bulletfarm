# Product gap register

Status: **operator index; not runtime or release authority**  
Last reviewed: 2026-09-10
Owner: Bullet Farm maintainers

This page answers “what is still missing before Bullet Farm is a product?”
It does not make a gate green. Authoritative status remains
[`release.md`](../release.md), the explicitly profiled release command, and
generated [`release-truth.generated.md`](release-truth.generated.md). The active
dependency order is [`closure-roadmap.md`](closure-roadmap.md). A newer commit
invalidates a row until those sources are replayed.

The latest operator instruction selects **GitHub `neverhuman/bulletfarm`** as the
primary integration repository, containing the exact accepted sources of all four
supporting repositories. Commit `912dddf` selects that destination for new
requests; historical request identities and destinations remain intact. Supporting
source PRs and audit scores do not establish complete aggregate CI. Separately
accepted Jeryu self-hosting and forge-certification obligations remain in scope.

The [9 September deep audit](deep-audit-20260909.md) records the current production
bridge, all CI execution gaps, actual account/runtime discovery, media defects and
concrete complexity reductions after studying all 101 member documentation files.
Its BF-A01–BF-A12 findings are source/reproducer-backed repair work under this
register, not new product-profile statuses or a replacement governing plan.

The [post-audit health checkpoint](health-checkpoint-20260909.md) records complete
Kernel `02bf7c5` and Hub `258c8467` required passes, actual execution of all 819
Hub/wire identities, reviewed capture/timing/custody repairs and publication
timeout/destination packets. Later changes need new mapped proof. The family
retry refused Kernel source drift after stage 5; no complete family pass follows.
Auditor admission, hosted CI and real-account production acceptance remain open.
The [detailed xbabe2 work order](xbabe2-development-closeout.md) refines the existing
plan with dependencies, CI campaigns, initial three-provider qualification and
Antigravity qualification for the requested four-provider demonstration. Maintained
real 1080p capture source and GIFs strictly below 50,000,000 bytes are explicit
acceptance requirements. It changes no G/WP/profile definition or status.
Reviewed local CI adapter/transfer/generator packets (`5b11bfb3`, `599b2ae8`,
`974f5de2`) and Portal custody/inventory packets (`9f53fe6a`, `29d02c34`) add
component evidence. Portal passes 183 unit tests; final complete and hosted checks
remain required. The typed inventory stays unchanged and blocked.

Exact historical accepted subjects and failed attempts remain in the
[checkpoint](production-prerequisite-checkpoints.md),
[health observations](health-observations-20260908.md) and
[prerequisite observations](prerequisite-observations-20260908.md). The current
branches need their own complete proof. Offline media generation and independent
reconstruction passed. The historical text/five-still replay and later continuous
synthetic browser capture do not qualify native coding or production operation.

The [full audit baseline](health-audit-baseline.json) and
[repair queue](health-repair-queue.jsonl) preserve all 165 findings and 28 cap
occurrences as unresolved triage inputs. An admitted portable auditor, explicit
score-90/zero-cap/zero-hard policy, actual repair evidence and reviewed medium
dispositions are still required. The deep audit independently rejected new
unverified generated stubs and weakened formal JSON parsing as health acceptance.

Production admission, supervised upgrades, account-aware dispatch, termination
and durable finalization, browser response-loss recovery, complete hosted CI and
all profile campaigns remain open. All G1–G18 remain `DESIGNED`; all 18 product
plus two diagnostic profiles remain `BLOCKED`. The typed inventory is unchanged:
this documentation packet claims no newly admitted runtime evidence.

This is the best-known current gap inventory, not a completeness proof. Every
blocked capability listed below has an owner document, a fail-closed checker,
and a typed refusal, but the Wave-0 bidirectional implementation↔invariant
inventory may discover additional orphaned requirements or enforcement sites.
Any discovery becomes a new explicit row before implementation proceeds.
Closing a row in this file is not closing the product.

Portal29d02c34 now has an independently accepted complete standalone required pass with183 units,14 mocked browser cases and five bundle tests; Hub6ad38025 integrates reviewed media CI routing after its direct suites passed. Complete current Hub/family/hosted and actual production proofs remain pending.

The latest instruction explicitly rejects a harness-exception recording as the
production milestone. The [expanded real-production work order](xbabe2-development-closeout.md)
pins the simulator-only worker/Runner and unavailable product verifier to exact
source, retains the working atomic command ledger and native Claude component,
and specifies bounded repairs through real account execution, independent
verification and human integration. Recordings follow that connected path. All
four requested providers must execute real tasks before the four-provider demo
is complete; no fixture or read-only explanation prompt can substitute.

## How to read a row

| Field | Meaning |
| --- | --- |
| Gap | The product capability an operator still cannot honestly claim |
| Why it is still open | The exact missing subject, not a vibe |
| What already exists | Component proof that must not be promoted |
| Closer | Who or what can close it |
| Authority | Document that may flip the status |

`bullet-family check release --profile self-hosted-v1 --receipts
<admitted-absolute-registry> --json` is the first-GA decision. The later
`universal-v1` composition adds every provider, forge, and platform profile
without implicitly adding evolution. `legacy-v1-26` and `linux-preview` are
diagnostics only. If this page and an explicitly profiled command disagree, the
command wins.

## What an agent may close versus what it may not

Gaps are closed only by a receipt. Prose cannot close G1–G18. Agents may close
a gap only when every closer in the row is code or a mapped test **and** no
operator secret, signer, or policy generation flip is required.

| Class | IDs | Agent action |
| --- | --- | --- |
| Engineering then operator | G1, G5, G6, G7, G16 | Implement and prove only the local producer/admission half behind an unexpired claim. Then document the exact operator act; do not invent a lock, flip `live_admission_enabled`, credentials, or patch a forge to look green. |
| Engineering, predecessor-blocked | G2, G3, G4, G9, G12, G13, G14, G15, G17, G18 | Use an unexpired `coord claim` after the admitted transition; during Operating HOLD use the authorized bounded manual path-exact maintenance protocol. A component receipt does not clear the family gate. |
| Quality / platform | G8, G10 | Reduce hard findings; add a native backend. A local Jankurai binary is not CI evidence. |
| Dependent certification | G11 | Keep `evolutionary_authority=false` through `self-hosted-v1`; implement and certify `evolution-v1` separately afterwards. Universal never implies it. |

A gap that is *fully specified, fail-closed, and indexed* is a **documented
open gap**, not a missing product definition. That is the only sense in which
documentation can “close” G1–G18 today.

## Remaining V1 product gaps

| ID | Gap | Why it is still open | What already exists | Closer | Authority |
| --- | --- | --- | --- | --- | --- |
| G1 | Hub-only signed install | Checked-in lock is schema 2 and is refused on purpose. The build-free `scripts/setup.sh` refuses unless the operator selects an external `bullet-family` executable, but that selection is not signed package admission. Clone transport helpers still use path-selected Git, and no production Jeryu/validator two-run, package lifecycle, or signed prebuilt exists | Descriptor-relative setup, no-replace publish, sealed Linux Cargo/Node/Bash/npm/setup-mutation/family-lock/checkout Git subjects, default-refusing wrapper, two-run component fixture | Engineering admits every remaining Git/helper subject and production lifecycle transition; then release custody publishes the signed schema-3 lock and prebuilt `bullet-family` and runs the fresh-host replay | [`release.md`](../release.md), [`runbooks/source-setup.md`](../runbooks/source-setup.md) |
| G2 | Connected five-plane transaction | No signed `TRANSACTION_PROOF`; the retained public loop reaches durable `UNKNOWN` through a harness-process executor and fixture keys, with independent, transaction, and release eligibility hard false. Trusted key lifecycle and durable nonce consumption, distinct verifier/effect UIDs and credential custody, independently owned artifacts, transaction-grade public dispatch and Portal truth, and the twelve-boundary campaign remain absent | A retained exact-digest command connects durable ScopeGrant admission, peer-authenticated farmd/Runner, Kernel-issued Candidate grant/final check, production Gitd one-use Candidate preparation, fixture writer refusal + PASS, purpose-signed PASETO v4.public/JCS `VerificationIntentV1`, `EvidenceV1`, `ProofBundleV1`, and caller-free `MATCHED` `ObservationV1` over the exact Candidate/base/head/tree, ProofBundle/check/protection/integration/target subjects with reconstructed ephemeral public keys and canonical-chain digests; exact Candidate-head `LocalBareForge` delivery/read-back, stale-fence refusal, lost-response `UNKNOWN`→`COMMITTED`, protected expected-old-OID integration, and reopen read-back; and private retained source/Candidate/target Git plus ledger artifacts whose exact Git subjects are independently reopened by the shell after child exit. A separate retained public wrapper authenticates exact duplicate `run_demo` POSTs, survives farmd restart, replays and polls through the Vite-preview Portal, dispatches the same request through a registered same-UID `SO_PEERCRED` UDS Runner and bounded exact worker, admits that fixture receipt, settles the same command/request/receipt digest durably to `UNKNOWN`, and reads back `NO_COMMAND` after worker restart | Independent verification/effect/audit owners issue new product-owned intents and evidence over independently reconstructed exact Candidates under registered keys, durable nonces, distinct UID/credential custody and independently owned artifacts; fixture records remain historical; semantic admission, transaction-grade farmd/Portal, and chaos owners close the remaining W6/W7 path | [`closure-roadmap.md`](closure-roadmap.md) Waves 2–7; [`runbooks/dogfood.md`](../runbooks/dogfood.md) |
| G3 | Production Kernel write path | Authenticated public `run_demo` now settles `PENDING`→durable `UNKNOWN` through the retained private component path and survives farmd and worker restart. It still uses same-UID fixture custody; operator-admitted long-lived peer/key custody, durable recovery/read-back beyond the private proof root, production provider/effect dispatch, CAS/GC, and production restore remain absent | DB-clock leases; authenticated ingress; exact duplicate public POST; product-provisioned private signing key; durable private peer registry and server grant/nonce state; peer- and socket-bound signed UDS component; bounded exact worker state/read-back; exact Candidate grant/final-check component; retained fixture receipt; UNKNOWN/FAILED worker; launch-grant + Linux egress components | Kernel V1-S2/S4 promotes the proven private path with operator-admitted durable registry/key custody, distinct service identities, recovery outside the fixture root, and production command/provider/effect dispatch; do not remount unauthenticated `HttpLeaseClient` | [`release.md`](../release.md), [ADR 0011](../decisions/0011-signed-launch-grant-and-egress-isolation.md) |
| G4 | Production BulletGit write path | Ordinary public `clone` remains fail-closed outside the scoped Kernel path; there is no published immutable `bullet-wire` tag, admitted installed authority custody, signed Integration proof, or tagged Jeryu service | The retained bridge uses the production daemon with a Kernel-issued exact Candidate grant/final check to prepare one one-use Candidate, refuses stale fence, preserves that exact Candidate/head through delivery, local exact-SHA check, protected integration, signed fixture Observation, and reopen read-back, then retains private source/Candidate/LocalBare ordinary-Git subjects and independently reopens their exact HEAD/tree after child exit; dissociate clone, hostile-git, generations, preservation, and honest cleanup UNKNOWN remain component-proved | Operator publishes wire/Jeryu tags; Kernel/BulletGit owners bind the retained permit/final-check path to installed durable custody and signed Integration evidence | [`closure-roadmap.md`](closure-roadmap.md) Waves 1 and 4 |
| G5 | Live provider conformance | The retained v1alpha1 policy refusal is `POLICY`. The default runtime-conformance observation returns `RUNTIME_PROBE_UNAVAILABLE`; this default contract does not describe every feature-selected probe path. A feature-gated Claude probe also exists at the inspected Kernel `5858e843`. The separately granted Claude read-only composition at5858e843 contains real enrolled-runtime dispatch under containment, but its general-live eligibility remains false; complete production policy/enrollment admission, all-provider onboarding and semantic live-receipt registration remain absent | Four bounded adapters; signed launch-grant and Linux-egress components; sealed 13-step refusal receipts; neutral four-provider zero-spawn nightly; positive PONG/conformance synthesis in a strict `cfg(test)` dispatcher; separate native Claude read-only source component with later exit/usage repairs under review | Engineering qualifies the existing Claude component and each other native protocol, connects admitted production execution, and completes hostile-tested schema-3 policy/enrollment-anchor admission, provider onboarding and semantic sealed-receipt registration; then the operator ratifies the policy and enrollments, supplies exact executables/profiles/credentials, and proves native read-only turns against the same frozen release subject | [ADR 0012](../decisions/0012-policy-v1alpha2-live-admission.md), [`runbooks/live-conformance.md`](../runbooks/live-conformance.md) |
| G6 | Jeryu live effect | No authenticated external Jeryu check, protected integration/target read-back, signed Observation, reconciliation, backup/restore, or drift receipt | The retained `LocalBareForge` component delivers/read-backs the exact Candidate, publishes/read-backs its exact-SHA check/ProofBundle root, performs protected expected-old-OID integration, purpose-signs caller-free target outcome `MATCHED` and reverifies it, reconciles lost response, reopens the same records, and retains/reopens its private Git target after child exit; the external Jeryu adapter remains typed quarantine and no local fixture substitutes for it | Engineering completes typed Jeryu probes and semantic receipt admission over the same port; the operator later restores scoped auth on an unmodified pinned forge and registers exact integration/reconciliation/backup/restore/drift receipts | [`release.md`](../release.md) |
| G7 | GitHub live effect | No App-test-repo integration receipt exists for the independent GitHub adapter profile | Effect adapter is specified, not certified | Engineering lands the typed capability/delivery/check/integration/read-back/reconciliation adapter and semantic receipt admission; then the operator configures a GitHub App test repository with role-separated credentials and registers the exact receipt | [`release.md`](../release.md), [ADR 0002](../decisions/0002-jeryu-forge-requirements.md), [0008](../decisions/0008-forge-gates.md) |
| G8 | Security release floor | September 8 full diagnostic audits scored Hub 65, Kernel 64, BulletGit 64 and Portal 68, with 109 high and 56 medium findings requiring contextual triage; the auditor is not yet qualified and no portable hosted artifact is admitted | Retained full diagnostic scans; policy/outcome consistency qualification remains open | Score ≥90 with zero caps and zero hard findings; checksum-pinned CI binary and portable exact-subject report | [`release.md`](../release.md) |
| G9 | Signed profile-selected release | No profile has an admitted signed package set. One quarantined component builds an unsigned Linux x86_64 archive; the frozen verifier expects the later five-target universal envelope and cannot admit the first-GA single-target profile | The component builder embeds the Portal and eight binaries, then re-reads its archive, CycloneDX SBOM, provenance, BLAKE3 checksums, and non-circular build manifest; signed-bundle verify + safe extract also exist as incompatible components | Release engineering first produces and lifecycle-smokes the signed Ubuntu 24.04 x86_64/systemd archive selected by `self-hosted-v1`; independent platform profiles add four targets and later `universal-v1` composes all five | [`release.md`](../release.md), [ADR 0010](../decisions/0010-supply-chain-policy.md) |
| G10 | Platform containment | Linux production containment has component proof only; every selected target still lacks an admitted target/profile-bound containment or typed refusal receipt | Linux is the strong-isolation reference; non-Linux mutation is fail-closed | Platform owners certify each named platform independently; `self-hosted-v1` selects Linux x86_64 only and `universal-v1` composes all five | [ADR 0007](../decisions/0007-sandbox-secret-taint.md) |
| G11 | Evolutionary runtime | `evolution-v1` has no study, canary, promotion, or rollback receipt and is deliberately not selected by self-hosted or universal | [`evolutionary-control.md`](../architecture/evolutionary-control.md); policy `evolutionary_authority=false` | After `self-hosted-v1`, complete the frozen offline study, no-effect shadow, and rollback-readiness proof; OD-H then authorizes one exact expiring ≤1% R0/R1 canary, followed by independent canary, promotion, drift, and rollback receipts | [`closure-roadmap.md`](closure-roadmap.md) Wave 9 |
| G12 | Family `check release` | Every named product profile remains `BLOCKED`; the historical `legacy-v1-26` and `linux-preview` diagnostics also remain blocked | Fail-closed explicit profiles and reports; supplied generic registries cannot clear a gate | Both kind-specific semantic admission through `release.receipt-contracts` and the explicitly requested profile-condition receipt over its exact dependency closure; neither half substitutes for the other | `bullet-family check release --profile <profile> --receipts <admitted-absolute-registry> --json`; [`release-truth.generated.md`](release-truth.generated.md) |
| G13 | Portal product surfaces | Six of fifteen spec surfaces have no durable ledger subject and stay explicit UNKNOWN; Context Lineage exposes revision-one subjects only; same-origin embedding is component-proved, but no signed package or installation receipt exists | Control Tower, Mission Graph, Live Attempt, Incidents and Audit, Fleet, Session Supervisor, Merge Rail, Quality Lab, and Context Lineage projections; CSRF/202; `PENDING→UNKNOWN`; SSE STALE; real-farmd browser component proof | Portal + farmd owners after G2/G3 add Cognitive Router, Fusion Lab, Quota/Capacity, Struggle, Behavior, and Workspace Hygiene, plus successor/compression lineage; release engineering supplies the signed package/install receipt under G9 | [`closure-roadmap.md`](closure-roadmap.md) Waves 6 and 9 |
| G14 | farmd production API | Authenticated `/api/v1/commands` dispatch for exact `run_demo` is component-proved from `PENDING` to durable `UNKNOWN` with `COMPONENT_PROOF_NOT_TRANSACTION_ELIGIBLE`, but only through same-UID private fixture custody. The outer receipt is `UNSIGNED_FIXTURE`, nested records are `FIXTURE_KEY_ONLY`, every eligibility flag is hard false, and there is no process-level response-loss hook, twelve-boundary chaos campaign, signed transaction receipt, independent executor/effect identity, or complete designed control plane | Loopback origin, no-wildcard CORS, command 202, ready/outbox/missions plus Fleet/Session/Merge/Quality/Audit/Context snapshots; exact duplicate POST before restart; farmd restart; Vite-preview Portal duplicate replay/poll; registered `SO_PEERCRED` UDS Runner claim; bounded exact worker; retained fixture transaction receipt; atomic `UNKNOWN` settlement whose command ID, request digest, and raw-receipt BLAKE3 agree in public GET and Portal; worker restart `NO_COMMAND`; post-exit retained-artifact reopen | Promote this component only after the minimal offline G2 transaction, independent custody, semantic admission, and response-loss/chaos exits settle honestly; add routes only after each missing ledger subject exists | [`closure-roadmap.md`](closure-roadmap.md) Waves 2, 5, and 6 |
| G15 | Cognitive persistence | One immutable revision-one Context Capsule is now normalized and atomically bound to graph materialization, lease, fence, and Attempt; CognitiveTask / SelectionGroup / Role / Fusion, quotas, budgets, routing decisions, successor lineage, and compression remain absent or design-only | Context Capsule schema/migration/replay and exact projection; wire shapes; offline provider parsers | Kernel evolution owners after the self-hosted substrate | [`closure-roadmap.md`](closure-roadmap.md) Wave 9; [`evolutionary-control.md`](../architecture/evolutionary-control.md) |
| G16 | GitLab adapter effects | Neither GitLab.com nor one exact self-managed GitLab endpoint/version has a typed effect adapter or a protected integration, exact-SHA status, read-back, UNKNOWN reconciliation, or drift receipt | Two explicit profile nodes and structural receipt bindings; no live adapter proof | Effects owners land the typed capability/delivery/check/integration/read-back/reconciliation adapters and semantic receipt admission; then operators provide scoped test projects and credentials and certify GitLab.com and self-managed GitLab separately | [`closure-roadmap.md`](closure-roadmap.md) Waves 4, 5, and 10 |
| G17 | Distributed team mode | PostgreSQL, remote runners, workload mTLS/SPIFFE, replicated projections, object storage, partition/failover, and distributed restore are absent | `team-v1` is an explicit fail-closed profile depending on self-hosted | Distributed runtime owners implement and certify `team-v1` only after self-hosted | [`closure-roadmap.md`](closure-roadmap.md) Wave 11 |
| G18 | Cross-repository sagas | No staged multi-repository Candidate, dependency-aware quarantine, compensation, or forward-repair transaction exists | `saga-v1` is an explicit fail-closed profile depending on team mode | Saga owners implement and certify `saga-v1` only after `team-v1` | [`closure-roadmap.md`](closure-roadmap.md) Wave 11 |

## 9 September operator-loop and experience audit

The CLI, terminal console and packaged Portal must operate the same durable work.
The 9 September inspection found two immediate safety defects: `coding submit`
printed session credentials, and the native adapter acknowledged termination
without terminating anything. The released local repairs remove the logging and
refuse unavailable termination. These repairs do not establish a complete native
lifecycle or qualify the current source for release.
This review covers all 25 direct member `docs/*.md` documents and all six outer
historical `docs/*.md` references, with the current roadmap and ADRs 0015–0018.
It does not establish whole-corpus requirement completeness. The current Hub and
accepted decisions take precedence over older five-screen, provider and forge
claims. This is additional detail under the existing G/W/DF registers.

| Existing obligation | Inspected gap and required exit |
| --- | --- |
| G13/G14; W6, DF601–605 | Local `bullet tui` now uses pinned Ratatui 0.30.2/Crossterm 0.29, an authenticated atomic snapshot, stable selection, navigation, monochrome fallback and detach/reconnect. Actual CLI/PTY component tests cover navigation, malformed refresh and terminal restoration. Initial queue blockers, approval inbox, native session actions and equivalent Portal mutations remain incomplete. |
| G3/G5/G14; W1–3, DF101–304 | The reviewed schema-26 packet accepts immutable task/repository/base/scope/criteria/gates/dependencies/budget/deadline and server-derived revision/run tracking. Exact retries preserve current phase. It allocates no nonce, reservation, lease or worker claim and reports `CODING_BINDING_ADMISSION_UNAVAILABLE`. Complete server-owned runnable authority, account eligibility, dependency evidence, monetary liabilities, verifier backpressure and the two-worker limit; historical process-wide `BULLET_HARNESS_*` configuration is not this admission. |
| G3/G5; W2–3 | Native lifecycle qualification is incomplete. The new raw CLI adapter emits stdout/exit without Runner's validated proposal; Cursor's inspected launch omits its requested model. Codex's raw `exec` route is not the planned App Server lifecycle. Implement and observe model, native identity, prompts, steering, interruption and complete process-tree termination for each provider. |
| G3/G14; W2–3/W6 | `coding stop` refuses; durable session controls, exact prompt responses, resume, pause/freeze enforcement acknowledgements and process restart reconciliation are absent. Queued cancellation and active STOPPING must be distinct; unknown termination keeps capacity reserved. |
| G14; W6, DF601–605 | Local CLI authentication uses hidden/stdin bootstrap input and private durable credentials. Reqwest plus generated Rust/JSON Schema models reject malformed replies; a private request journal precedes POST and binds exact retries and subsequent status reads to the original request. Durable daemon sessions/revocation, operator-scoped empty-cache discovery and exact retry returning the current phase now have independently reviewed component implementations. Retries retain original nonce/reservation checks without fresh epoch/quota admission. Private daemon bootstrap files replace credential logs. The task-shaped packet and Portal consumer have current component CI; installed execution and recovery remain required. |
| G13/G14; W6 | Control Tower and Shift Brief now consume one authenticated SQLite read transaction through `/api/v1/operator-snapshot`; generated consumers validate its watermark and relational consistency. Eight other projected pages retain ordinary-event refresh. Transport reconnection and event continuity remain separate; full current-source CI and installed browser acceptance are still required. |
| G13/G15; W6/W9 | Nine narrower projections do not complete fifteen operational surfaces. Six still lack ledger subjects. Persist routing exclusions, fusion dissent/selection, quota liabilities, meaningful progress, behavior/remediation, workspace preservation and successor/compression lineage before presenting those controls or insights as operational. |
| G2/G4; W4, DF401–403 | Retain Cursor's prevalidation repair in BulletGit. The current 65 MiB frame bound assumes two-times JSON expansion, while valid control bytes can expand six times. Metadata bounds and the shared sixteen-gate limit also need alignment with 128 operations, 1 MiB/write and 32 MiB decoded content. Prove interrupted writes, stale subjects, hostile paths and ENOSPC. |
| G2/G3/G4; W3–5 | Receipt-bound cleanup and acknowledged Runner finalization are implemented components. Candidate preservation, Attempt completion, lease/reservation settlement and verifier outbox still need one finalization transaction. Complete descendant termination evidence, unknown-capacity retention and post-delete response-loss reconciliation before claiming the full lifecycle. |
| G2/G6/G7/G16; W4–5, DF401–503 | PONG gates and fixture chains do not provide independent coding verification. The production verifier refuses with `VERIFICATION_INTENT_ADMISSION_UNAVAILABLE`. Implement immutable meaningful gates, independent workcells/identities and purpose-separated publication, then certified delivery/check/integration/read-back/observation for each forge. |
| G2/G12; verification predecessor | ADR 0018 requires the separately accepted operator-approval carrier/policy and two distinct authorized OD-L approvers before signer activation, final lock or live/release admission. Component contract publication may proceed. The coordinator proposal's similarly named checkpoint must not be treated as this approval. |
| G1/G8–12/G17/G18 | Signed packages, native lifecycle/security/accessibility, the sixteen real tasks, twelve-boundary faults, seven-day observation, comparative human-time study, cognition/evolution, distributed teams and sagas remain independent obligations. A local UI proof closes none of these. |

The experience target follows W6/DF601–605 and the current fifteen-surface design:

- Make a conversation with the head of the farm the default for `bullet` and
  `bulletfarm`, the React Portal, Slack and Telegram. Preserve complete user and
  assistant turns, goals, corrections, progress and pending decisions in one
  operator-owned farmd ledger. The head handles task decomposition, delegation
  and handoffs; task-file commands remain advanced controls.
- Make `bullet setup` provide guided repository/provider onboarding and optional
  Slack/Telegram connections. Make `bullet serve` start the packaged web
  experience with durable service/state custody, usable defaults and an
  authentication handoff. Existence checks or helper-only smoke tests cannot
  establish a ready installation.
- Bind each head turn to its exact input range, goal revision and runtime claim.
  New user messages remain durable while work runs; stale proposed actions must
  revalidate current subjects. A saved-message acknowledgement establishes
  neither an assistant response nor engineering success.
- Implement Rust Slack Socket Mode and Telegram adapters with admitted
  workspace/chat/user/thread bindings, durable duplicate suppression before
  acknowledgement, and recorded outbound delivery/reconciliation. All clients
  use the same conversation and command consumers. Keep transcript bodies out
  of generic audit/outbox/SSE projections. Native head execution and live
  messaging remain subject to their actual admission.

- Start with HOLD/enforcement, the exact active subject, unresolved blocker,
  evidence class and next authorized action. Prepared, verified, integrated,
  observed and seven-day survival remain separate states.
- Navigate mission → task → attempt → native session, with a causal timeline,
  tools/artifacts, context handoff and an approval inbox. Preserve keyboard focus
  during live updates; offer search, clear shortcuts and text alongside color.
- Show requested, delivered and applied controls separately. Detach changes only
  the view. Pending prompts are distinct from governance/integration approvals.
- Explain queue position and dependency/account/quota/freeze/termination blockers;
  never infer useful work or available capacity from a process count or absent data.
- Build the CLI and Portal against the same accepted generated command/query
  contracts. CLI environment presence is not admitted farmd runtime readiness;
  escape untrusted terminal output and refuse malformed/incompatible snapshots.

The initial historical 10 September checkpoint below is superseded by the later
dated inventories and authenticated connected cases; its original findings and
failures remain retained. That pass implemented a responsive Portal navigation rail, searchable Ctrl/Cmd+K
palette, visible keyboard selection and focus return; event-driven refresh on the
eight shared-loader surfaces; and explicit UNKNOWN/STALE snapshot presentation.
An authenticated SSE opening comment fixes idle farms timing out before response
headers without fabricating an event. The final Portal suite passes 195 tests on
pinned Node 22.23.2 and the typed production build passes. Farmd passes 87 default
component tests and strict Clippy; the formatted final read-auth subset passes four.
Five native Chromium checks against an actual private farmd verify authentication,
real command-event refresh, palette focus/scroll, mobile fit and detach/read-back.
No provider runs: the one accepted demo command remains PENDING without a worker.
These are local component proofs, not packaged-browser or production acceptance.
Exact sources, retained failures, screenshots and execution records are handed off
through the family coordination log; current full/hosted proofs remain required.

Verification gaps recorded at that initial checkpoint remain historical evidence:
the cited historical `bullet-offline-component-proof.observation-20260827T1340Z`
receipt was absent at its recorded local path; retrieve the preserved original
before relying on it. Existing real-farmd browser specs still assume anonymous
reads and the older demo submit button and need current-source reconciliation.
Documentation test counts and older Jeryu-only/eight-worker runbook statements
need alignment with current inventories, GitHub integration and Operating HOLD.
No historical receipt is silently invalidated, replaced or promoted by this audit.

An earlier 10 September checkpoint recorded local component proofs: 39 CLI unit tests,
two real-PTY authentication tests, a TUI PTY smoke using a clearly synthetic
authenticated server, and a further saved-request status-correlation regression.
The TUI smoke initially failed because it matched raw differential terminal
bytes; a pinned terminal decoder now checks the rendered screen. Strict CLI
Clippy passes. The atomic snapshot packet reports 35 focused farmd tests,
75 focused Portal tests and ten explicitly mocked browser regressions. At that checkpoint Portal's
discovered inventory was 206 tests and its 185-test CI pins were stale. These
observations are retained as history and superseded by the current record below.

At the 10 September 02:13 UTC checkpoint, all 45 CLI unit tests, two real-PTY
authentication tests and the TUI PTY smoke pass. The durable command-owner wave
has 85 distinct focused tests and separate static review; these are component
fixtures. The complete Kernel inventory check accounts for 1,237 identities
(1,191 standalone + 34 contract + 3 egress + 9 family); inventory enumeration
is not a successful execution of the full required lane.

Portal [PR #5](https://github.com/neverhuman/bullet-portal/pull/5), commit
`c55aa038894e2f002b09f3727d85da5452763d93`, passes the full local standalone lane
and current GitHub push/PR checks, including 214 unit and 14 mocked browser tests.
The unit/browser identity pins match actual reports. Later local generated-model
and private-bootstrap launcher changes are outside that checked commit and need
their own qualification. No count here establishes daily-use or showcase acceptance.

GitHub and installation findings belong to G1/G8–12/G17/G18 and W8–11 in this
same register. The public aggregate still serves older member snapshots and has
no established root CI execution. Its historical README overstates the requested
authenticated Bullet workflow; the canonical publication template now preserves
the recordings with explicit limits. All three retained bridge GIFs fail the
required geometry. Fresh paired installed TUI/Portal captures, sixteen accepted
provider tasks and published-asset hash/playback read-back remain required.
The previously checked Portal and BulletGit PRs were merged with successful main
checks; newer local bytes are outside those checks. Kernel PR #3's observed lint
failure lacks `zizmor`. Its reviewed provisioning repair and matching workflow
pins are committed locally at `7295e7136ce93a04d5e17ee035e94517ba8d8ca8`, but
GitHub rejected the push because the OAuth credential lacks workflow scope.
Those bytes have no hosted qualification. Portal main now has effective strict
protection requiring its six GitHub Actions checks, prohibiting force pushes and
deletion, and applying to administrators. Equivalent protection for the other
four repositories remains unverified at this checkpoint. Hub tool,
scanner, media, decoder-inventory and observation failures and aggregate member
lane execution remain open; missing or stale evidence cannot pass.

At the later **10 September, 02:26 UTC checkpoint**, the existing authorized SSH
remote successfully delivered Kernel commit `7295e7136ce93a04d5e17ee035e94517ba8d8ca8`;
GitHub PR #3 read-back confirms that exact head. The earlier OAuth push refusal
remains part of the delivery history. The new run passes source admission, fast,
contract, security and docs, but lint now fails in the offline-wrapper tests:
their `b3sum` dependency is absent from the hosted installation. The local repair
adds locked `b3sum` 1.8.2 installation and retains the refusal assertions; current
review and hosted qualification of those new bytes remain required.

At the **10 September, 03:42 UTC checkpoint**, the reviewed operator components
have reached protected GitHub main branches:

- Kernel [PR #4](https://github.com/neverhuman/bullet-kernel/pull/4) merged reviewed
  head `f0f1bdcb9c894b49674276b2050d754de8829368` as
  `ef98cd3c420f1e8ada96acd57989a87a2e4c55f8`. Its complete local required lane
  passes 1,191 standalone and 34 contract tests plus lint, security and docs.
  Both head runs and the [main run](https://github.com/neverhuman/bullet-kernel/actions/runs/34433379926)
  pass all seven required jobs. The observed scanner, missing-tool, schema-test
  and license failures were repaired with their assertions retained.
- Portal [PR #5](https://github.com/neverhuman/bullet-portal/pull/5) and
  [PR #6](https://github.com/neverhuman/bullet-portal/pull/6) merged; current main is
  `f1fdb4bc3c3c3ea03d076d268d9cc8745dca1e1a`. Reviewed head
  `4cd0fc89fa44e6f8b1f4c475f63519d9a8ff454c` passes full local required with
  245 distinct unit and 14 mocked browser tests. Both head runs and the
  [main run](https://github.com/neverhuman/bullet-portal/actions/runs/34434144426)
  pass all six required jobs. Actual identity comparison accounts for every
  added or intentionally replaced test; prior counts remain historical.

Both member main branches enforce their required checks, include administrators,
and prohibit force pushes and deletion. This is member delivery evidence;
aggregate source selection, root CI and signed package admission remain separate.
Browser command history now supports operator-scoped discovery, and response
correlation binds the original command ID, kind and Rust-compatible request
digest. A pre-POST journal stores validated bytes, resists serialization-hook
substitution and restores saved intent without an automatic submission. These
component tests do not establish installed recovery or native provider execution.
A subsequent unmerged Kernel packet rejects duplicate decoded JSON fields before
durable command admission; ten focused HTTP tests and strict Clippy pass, while
its full current-source qualification remains pending.

Per-task server allocation, durable account/dependency scheduling, acknowledged
native controls and termination, atomic Candidate settlement, independent
verification/integration and the six missing operational subjects remain open.
Unknown termination must retain capacity even after lease expiry; neither a
successful signal nor leader exit establishes complete descendant termination.

Schema-2 bootstrap is still refused, source setup still selects Jeryu, package
verification cannot yet admit the intended Ubuntu-only profile, and the
development launcher deletes temporary state on exit. Implement authenticated
GitHub aggregate onboarding, profile-bound signed releases and supervised
persistent services. Two clean installs, upgrade, backup/restore, rollback and
disaster recovery remain unproved. Ubuntu installation does not certify
`self-hosted-v1` without its independent Claude/Jeryu obligations. Operating HOLD,
the separate two-approver carrier, twelve-boundary faults, seven-day observation
and all remaining product-profile requirements remain effective.

At the **10 September, 15:18 UTC source checkpoint**, reviewed Kernel
[PR #10](https://github.com/neverhuman/bullet-kernel/pull/10) merged as
`1aca8855cae5940ffabda0b899f80544aa4c52de`, tree
`42d36180b7d07a9645b0f9253e5d8d62852c8ff3`. The tree equals reviewed head
`e0e386f6d1e09ed5bce7b2f3aac5c6893423da57`. Complete local required passed
1,243 component tests, 34 contract tests, lint, security and docs; both
[push](https://github.com/neverhuman/bullet-kernel/actions/runs/34493021858)
and [PR](https://github.com/neverhuman/bullet-kernel/actions/runs/34493236469)
runs passed all seven required jobs. The resulting
[main run](https://github.com/neverhuman/bullet-kernel/actions/runs/34494198501)
subsequently passed all seven jobs at the 15:20 UTC read-back. Strict main protection requires current
checks and prohibits force pushes and deletion. Earlier failed schema-backup
assertions and stale documentation checks remain retained; their exact repairs
received independent review before the passing run.

The closed `bullet.run-coding.v2` contract and schema-26 transaction persist
immutable task/dependency/run tracking. Fresh legacy authority-bearing submission
is refused; historical owned retries preserve their exact bytes and current phase.
The CLI supports task files, saved-request retry and digest-validated discovery.
Intent allocates no execution authority and reports
`CODING_BINDING_ADMISSION_UNAVAILABLE`. Eligible-account scheduling, monetary
liabilities and runnable task authority remain incomplete.

Schema 27 now persists closed human-message commands, server-generated conversation,
message and head-request identities, and atomic ownership/message/audit/outbox
receipts. Exact retries precede current-tip comparison; owner-scoped paged reads
validate complete immutable history, including earlier content and parent custody.
Unqualified assistant rows are refused. Twenty-two new test identities cover
nine transaction rollback boundaries, response loss, competing clients, restart
discovery, paging, HTTP authentication/revocation and backup/restore. The inventory
remains exhaustive: 1,289 total, partitioned as 1,243 standalone, 34 contract,
nine family and three egress. A saved message does not establish a native reply.
`HEAD_RUNTIME_BINDING_REQUIRED` remains explicit; native replies, delegation,
conversational CLI/Portal consumers and Slack/Telegram delivery are unfinished.

The CLI's HTTP and private-journal readers now use the strict JSON decoder before
model validation. Existing regressions reproduce root, nested and escaped-equivalent
duplicate keys, invalid UTF-8 and private-input non-disclosure; all 52 CLI unit
tests passed within this reviewed wave. New conversation journals must still bind
the authenticated operator identity so an uncommitted message cannot replay as
another operator after local login changes. Historical coding journal semantics
must be preserved explicitly.

Portal [draft PR #7](https://github.com/neverhuman/bullet-portal/pull/7), head
`dbb57bee978bb870555029e18cd14ea5b87e2413`, passed its then-current full local
required suite and hosted checks, including 258 unit and fourteen mocked browser
tests. Its task form restores journaled intent and shows owner-scoped task
snapshots. Subsequent local routing changes are outside those checked bytes:
all fourteen mocked browser cases move to an explicit `rendered` lane; connected
and packaged lanes retain three and seven cases. Real host guards reject execution
outside Linux xbabe2 or with `CI`/`GITHUB_ACTIONS` present, and hosted workflow
policy separately refuses browser tools and local-only lanes. Portable contract
checks retain the five actual bundle tests with an exact passing-log guard.
Focused parser, failure-propagation and all 34 artifact-lifecycle regressions
passed. Review found the first host-refusal test could pass because of a wrong
hostname while missing a CI-variable defect. A pure policy matrix now isolates
hostname, platform and each CI-variable presence; the actual-helper subprocess
cases also clear inherited CI variables. The repaired focused checks passed;
their final independent review and full current-source required remain pending.
Fresh local rendered,
family and packaged proofs remain required before integration. Earlier connected
cleanup failures remain evidence.

Runner finalization now requires a positive termination acknowledgement before
terminal release, retains token-free Candidate/receipt references and both
primary/recovery-journal errors, and prevents generic fallback release/requeue
after finalization begins. Post-delete response loss remains unresolved.
Native descendant observation, capacity retention beyond legacy lease expiry,
earlier failure-path negative acknowledgements, atomic Candidate/quota/verifier
settlement and duplicate-cleanup reconciliation remain separate obligations.

BulletGit main `4e33103673f535fde871ef26cffe05460674293b` has green main CI.
The owning integrator reconciled the canonical metadata after verifying all 206
accepted path blobs and modes, and released the clean checkout for a fresh Gitd
build. Reusing the older component Gitd binary cannot prove the current source.
Hub PR #3 at `12b43866e3d7f1be21af3f17fc52d04db226fe43` remains open with green
checks. Its agent reports an automatic approval classifier rejected the merge
action itself; that refusal has not been bypassed. Hub delivery is incomplete.

The aggregate generator's exact source review found 51 of 53 member jobs were
refusal stubs, an unconditional final refusal and inconsistent merge-queue/push
admission. Matching job names is not execution. Actual member lanes, validated
artifact custody and event qualification remain required before aggregate claims.

An external integrator installed unmerged development `serve`/`setup` binaries.
Independent review rejected the packet and its second revision: rotation deletes
an arbitrary small owned 0600 file, with a metadata/unlink race and loss on later
provisioning refusal. The owner withdrew its completion claim and agreed to remove
rotation entirely; repaired bytes are not yet admitted. State-directory agreement,
child-bound readiness, safe signal/child custody and terminal sanitization also
require review and meaningful regressions. A release-profile binary and local
web/auth smoke do not establish signed installation or native provider execution.

The installed TUI's first local harness run had two concurrent first-paint timeouts
and four idle-wait timeouts; the latter reproduced serially and were a test defect.
After replacing idle waits with rendered-text waits, eight serial flows passed.
That does not satisfy concurrent-client acceptance. Canonical source review found
credential readers briefly take an exclusive nonblocking lock, which can reject
simultaneous startups; the TUI does not retain that lock during rendering. Implement
a private read snapshot path while preserving writer/journal exclusivity, and test
six simultaneously active clients, detach independence and logout/relogin races.
Exact installed binary subjects and retained failed PTY receipts remain necessary.

The operator requires **all Tuiwright and Playwright execution on xbabe2 only,
outside normal CI**, including mocked cases. Portal retains fourteen mocked,
three connected and seven packaged scenarios; Hub retains six browser-capture
regressions. Hub's current browser-availability early success must become a typed
refusal. Preserve exhaustive portable/local partitions and require fresh
source-bound local receipts for delivery. The excluded TUI proof harness still
needs manifest, lock, executable and tool custody; a blanket directory exclusion
would hide proof dependencies and build scripts. A hostname, login, or synthetic
browser result does not establish authenticated native-provider execution.

The legacy Claude worker omits credential-descriptor flags while clearing its
environment, and its dogfood adapter drops the requested model before constructing
provider arguments. Forward bounded admitted credential metadata through durable
invocation configuration and bind the actual account/model/settings. Neither an
echoed model nor a logged-in executable qualifies the provider.

The conversational default, setup/serve and Slack/Telegram onboarding extend
G3/G13/G14/G15 and W1–3/W6. Rust owns durable control and supervision;
Vite/TypeScript/React owns the web client. Rejected local-history, keyword-reply
and refusal-only prototypes remain private development history. All G1–G18,
W/DF/WP, fifteen surfaces, media, seven-day observation and product profiles
remain in scope. Operating HOLD requires its actual predecessor admission and
operator checkpoint; no development build or component test lifts it.

## Historical 26-gate catalog and release profiles

`self-hosted-v1` is the first GA profile: Ubuntu 24.04 x86_64/systemd, Claude,
and local Jeryu. `evolution-v1` depends on self-hosted and certifies separately.
Provider, forge, and platform profiles are independent. The later
`universal-v1` composition requires all four providers, Jeryu, GitHub,
GitLab.com, self-managed GitLab, and all five platforms without implicitly
admitting evolution, team, or saga. `legacy-v1-26` and `linux-preview` are
diagnostics only. A receipt for one profile never certifies another.
The initial coding campaign additionally requires Codex, Claude, and Cursor;
that campaign requirement does not change the currently Claude-selected
`self-hosted-v1` contract or replace independent provider certification.

The profiled JSON report uses schema 3 and names its `profile`. The current
registry boundary is intentionally conservative: an absolute registry may be
selected, but generic signed envelopes cannot clear gates until kind-specific
semantic validators and externally admitted signer/trusted-time roots exist.

The first 26 rows are the historical catalog preserved by the generated
portable page.

These IDs are the static negative inventory preserved by `legacy-v1-26` in
`src/check/prerequisites.rs`. Every row is `BLOCKED`. A green component crate
cannot clear any of them, and this historical list is not a substitute for
evaluating the full `universal-v1` dependency closure.

| Gate ID | Product gap | Class |
| --- | --- | --- |
| `release.installable-lock` | G1 | Release |
| `release.installer-twice` | G1 | Release |
| `release.transaction-demo` | G2 | Transaction |
| `release.fault-suite` | G2, G3 | Release |
| `release.backup-restore` | G3, G9 | Release |
| `release.provider.claude` | G5 | Live |
| `release.provider.codex` | G5 | Live |
| `release.provider.cursor` | G5 | Live |
| `release.provider.antigravity` | G5 | Live |
| `release.forge.jeryu` | G6 | Live |
| `release.forge.github-app` | G7 | Live |
| `release.jankurai-90` | G8 | Release |
| `release.scan.dependency` | G8, G9 | Release |
| `release.scan.license` | G8, G9 | Release |
| `release.scan.secret` | G8, G9 | Release |
| `release.scan.workflow` | G8, G9 | Release |
| `release.checksums` | G9 | Release |
| `release.manifest-non-circular` | G9 | Release |
| `release.package-matrix` | G9 | Release |
| `release.provenance` | G9 | Release |
| `release.receipt-contracts` | G12 | Release |
| `release.rust-msrv-1-95` | G9 | Release |
| `release.rust-pinned-1-97-1` | G9 | Release |
| `release.sbom` | G9 | Release |
| `release.signatures` | G9 | Release |
| `release.platform-containment` | G10 | Release |

Through its Linux x86_64 platform dependency, `universal-v1` also selects two
native lifecycle gates. Both remain blocked; neither is implied by the generic
platform condition.

| Gate ID | Product gap | Class |
| --- | --- | --- |
| `release.package-linux-x86_64` | G9 | Release |
| `release.systemd-v1` | G1, G9 | Release |

The global product-profile catalog has eighteen condition gates. The universal
projection selects fifteen of them; evolution, team, and saga remain separate.
Conditions make dependency closure visible; they do not duplicate or clear the
historical or native capability gates above.

| Gate ID | Product gap | Class |
| --- | --- | --- |
| `release.profile.evolution-v1` | G11, G13, G14, G15 | Release |
| `release.profile.github-adapter-v1` | G7 | Release |
| `release.profile.gitlab-adapter-v1` | G16 | Release |
| `release.profile.gitlab-self-managed-v1` | G16 | Release |
| `release.profile.jeryu-forge-v1` | G6 | Release |
| `release.profile.platform-linux-aarch64` | G9, G10 | Release |
| `release.profile.platform-linux-x86_64` | G9, G10 | Release |
| `release.profile.platform-macos-aarch64` | G9, G10 | Release |
| `release.profile.platform-macos-x86_64` | G9, G10 | Release |
| `release.profile.platform-windows-x86_64` | G9, G10 | Release |
| `release.profile.provider-antigravity` | G5 | Release |
| `release.profile.provider-claude` | G5 | Release |
| `release.profile.provider-codex` | G5 | Release |
| `release.profile.provider-cursor` | G5 | Release |
| `release.profile.saga-v1` | G18 | Release |
| `release.profile.self-hosted-v1` | G1, G2, G3, G5, G6, G8, G9, G10, G13, G14 | Release |
| `release.profile.team-v1` | G17 | Release |
| `release.profile.universal-v1` | G1, G2, G3, G5, G6, G7, G8, G9, G10, G16 | Release |

G4, G13, G14, and G15 have no dedicated historical catalog `release.*` id. G4
blocks G2 and therefore `release.transaction-demo`. G13 and G14 follow the
minimal authenticated offline transaction, so they do not form a cycle by
blocking their own prerequisite. They are nevertheless mechanically owned by
the `self-hosted-v1` condition (durable or typed `OUT_OF_PROFILE`) and also by
the `evolution-v1` condition (all fifteen surfaces durable). G15 is owned only
by `evolution-v1`: a Wave 6 `OUT_OF_PROFILE` projection is honest but does not
close cognitive persistence before Wave 9. Universal inherits the self-hosted
condition but never implies evolution. G11 and G16–G18 are owned by
their explicit profile-condition gates. G12 is the inventory of these tables
plus semantic admission for profiles that evaluate a selected registry;
`legacy-v1-26` is inventory-only.
The generated universal page
([`release-truth.generated.md`](release-truth.generated.md)) selects 43 gates
while binding all 46 global `release.*` crosswalk rows and the 18-G-id list by
digest: a crosswalk change requires
`just release-truth` in the same commit or `required` fails on drift.

## How to verify each gap is still open

From the hub checkout:

```bash
bullet-family doctor --json          # G1: BLOCKED / UNSUPPORTED_SCHEMA is honest
bullet-family check release --profile self-hosted-v1 --receipts /absolute/admitted-registry --json
bullet-family check release --profile evolution-v1 --receipts /absolute/admitted-registry --json
bullet-family check release --profile universal-v1 --receipts /absolute/admitted-registry --json
bullet-family check release --profile legacy-v1-26 --receipts /absolute/empty-registry --report --portable
bullet-family check release --profile linux-preview --receipts /absolute/registry --json
just fast && just contract           # component lanes; never G2–G18
```

Do not convert a green `just fast` into a closed G-row.

## Which command proves what

| Command | Proves | Does not prove |
| --- | --- | --- |
| `just fast` | Mapped component lanes on this checkout | G2–G18, live, install, release |
| `just contract` | Generated wire/schema identity | A running issuer or published tag |
| `bullet-family doctor --json` | Honest refusal of schema-2 hub-only install | That schema-3 exists |
| `bullet-family check release --profile self-hosted-v1 --receipts <admitted-absolute-registry> --json` | First-GA Ubuntu/Jeryu/Claude closure and exact blockers | Any independent provider, hosted forge, non-Linux platform, or evolution profile |
| `bullet-family check release --profile universal-v1 --receipts <admitted-absolute-registry> --json` | Later maximum-scope composition and exact blockers | Evolution, team, saga, or evidence from an absent/generic registry |
| `bullet-family check release --profile legacy-v1-26 --receipts <empty-registry> --report --portable` | The historical 26-gate diagnostic projection and exact blockers | Complete `universal-v1` release authority |
| `bullet-family check release --profile linux-preview --receipts <registry> --json` | A non-release Ubuntu/Jeryu/Claude diagnostic slice | Any omitted provider, GitHub, package, or canonical GA gate |
| Archived 2026-08-24 live demo | That one past tree spawned under then-policy | HEAD conformance |

`check required` adds six more static blockers (`required.installable-lock`, `required.jankurai-ratchet`, `required.packaged-browser-e2e`, `required.pinned-scans`, `required.recovery-faults`, `required.transaction-proof`). They are the same gaps, not a second product list.

## One-hop operator answers

| Question | Answer |
| --- | --- |
| How do I install from a hub-only clone? | You cannot, honestly. Schema 2 is refused. Contributor bootstrap is [`runbooks/source-setup.md`](../runbooks/source-setup.md); signed install is G1. |
| Can I turn on production Claude/Codex/Cursor/Antigravity? | Production coding is not admitted. The default general-live path refuses at `ADMISSION` with `RUNTIME_PROBE_UNAVAILABLE`; separately granted native Claude read-only composition exists and is component-only. Complete per-provider qualification, production account/runtime admission, durable command-to-Runner execution, independent verification and the actual policy/enrollment/operator checkpoints before production use (G5). The [development closeout](xbabe2-development-closeout.md) gives the concrete implementation order. |
| Why does `doctor` fail? | The checked-in lock is schema 2. That refusal is the product. |
| Did the white paper close the product? | No. The paper's G1–G15 inventory is extended here by explicit GitLab/team/saga profile gaps G16–G18. Closing prose is not a receipt. |
| Is `just fast` enough to ship? | No. It is a component lane. First-GA `self-hosted-v1`, later `universal-v1`, the historical 26-gate projection, and the `linux-preview` diagnostic all remain `BLOCKED`. |
| What is the same-UID install hole? | The Rust boundary seals Cargo/Node/Bash/npm plus setup mutation, family-lock verification, and checkout verification Git bytes, and the wrapper no longer invokes ambient Cargo. G1 still includes unsigned selection of the external prebuilt, clone transport Git/helpers, transient and between-child repository object/ref/index/config/file races, non-Git work-tree traversal, and allowed-signers path admission; signed prebuilt admission plus complete Git/helper isolation closes those surfaces. |
| Does ADR 0012 mean the committed policy enables live providers? | No. The retained policy/default-observation refusals authorize no production execution. At inspected Kernel `5858e843`, a feature-gated Claude probe and a separate contained Claude read-only composition exist and require their own exact authority and inputs; the historical zero-spawn refusal does not describe every such path. Neither component qualifies production coding or lifts general live admission. Each production account still needs independently admitted policy, enrollment, runtime and launch authority. |
| Where is `docs/INDEX.md`? | It must not exist. This family's index is [`../README.md`](../README.md). |

## Slice leftovers (V1-S0..S8)

This is the same predecessor work as G1–G18, indexed by the closure-plan slices so an
implementer cannot “lose” a leftover by reading only the G-table.

| Slice | Status class | Leftover that still blocks a product claim |
| --- | --- | --- |
| V1-S0 | Local complete; release continuous | Exact-path commits and claim receipts remain an orchestrator obligation |
| V1-S1 | LOCAL-BLOCKED | Immutable published `bullet-wire` tag; consumers still carry duplicate or legacy semantics; production JSON-RPC hello/version/frame contract |
| V1-S2 | LOCAL-BLOCKED | Normalized full truth, signed capabilities, CAS/GC, production restore admission, fault-complete recovery |
| V1-S3 | LOCAL-BLOCKED | Positive online authority/settlement; immutable shared-wire tag; complete Integration proof; reviewed tagged `jeryu-gitd` (the local Candidate manifest/identity is complete) |
| V1-S4 | LOCAL-BLOCKED | Signed internal lease transport; runner/verifier/effect saga; credential-free `TRANSACTION_PROOF` |
| V1-S5 | LOCAL-BLOCKED | APPLIED/VERIFIED dispatch; six Portal surfaces without durable ledger subjects; successor/compression Context lineage; packaged farmd-served Portal |
| V1-S6 | LOCAL-BLOCKED | Cognitive objects beyond the revision-one Context Capsule; schema-3 provider policy/enrollment-anchor admission, provider onboarding/runtime probing, semantic receipt registration, and quota/budget/routing/fusion replay from persisted inputs |
| V1-S7 | LOCAL-BLOCKED | Schema-3 lock, signed admission of the build-free wrapper's external executable and remaining clone Git/helper/non-Git filesystem subjects, the profile-selected signed archive set (one for first GA; five only for universal), SBOM/provenance, hosted Jankurai artifact, docs that wait on typed commands |
| V1-S8 | EXTERNAL-BLOCKED | Operator-issued first-GA Jeryu/Claude authority and Ubuntu signing custody; later independent GitHub/GitLab, three-provider, and four-additional-platform authority; exact protected test repositories |

V1 is done only when every `V1-S0..S8` gate has a current independently
verifiable receipt from the same signed subjects. “Agents ran” is not that
receipt.

## Operator decisions that are not code

The authoritative list, owners, evidence required, and expiry/reversal rules live in
[ADR 0013](../decisions/0013-operator-decision-register.md). Agents must not copy that register into status prose,
flip live/evolutionary policy, alter the running forge, invent schema-3 subjects, or substitute local credentials.

## Centerrail C1–C12: product status

The family `TEAM.md` red-team is historical provenance. Living control is
[`evolutionary-control.md`](../architecture/evolutionary-control.md) plus this
register. Disposition of each critique:

| ID | Adopted meaning | Product status |
| --- | --- | --- |
| C1 | Every entry declared in the registry has one T1 schema / T2 gateway / T3 test primary tier | Validator proves registry-internal completeness only; the whole-product bidirectional orphan inventory remains open Wave-0 work |
| C2 | Oracle-modifying diffs + required holdouts for R2+ | Designed; implemented verifier is one fixture E2 |
| C3 | Two-track scope expansion ([ADR 0004](../decisions/0004-scope-amendment-tracks.md)) | Accepted decision; not a live Attempt path |
| C4 | Attestor ≠ broker; reconstructible check from proof bundle | Designed; live forge blocked |
| C5 | `CONTRADICTORY` / prolonged `UNKNOWN` has fence-mediated exits | Designed |
| C6 | Bounded probe reservation; `unknown` is never headroom | Designed |
| C6b | One seat-equivalent per named human | Designed; not enforced in the kernel |
| C7 | Formal-model exactly two protocols in Phase 0 | Adopted and component-complete |
| C8 | Historical proposal: GA = kernel + any two certified providers | Superseded: first-GA `self-hosted-v1` names Claude exactly; independent provider profiles name Codex, Cursor, and Antigravity; later `universal-v1` composes all four, and no receipt substitutes for another |
| C9 | Identity-exact effect adoption (fence + desired OID) | Command idempotency component; graph mint not a live path |
| C10 | Verifier dwell is writer-admission backpressure | Designed |
| C11 | Freeze chip shows recorded vs enforced-on-N/M runners | Portal honesty component; freeze countdown designed |
| C12 | Multi-repo saga quarantines blast radius, not the fleet | Explicit `saga-v1` profile after `team-v1`; [closure roadmap](closure-roadmap.md) Wave 11 |

Rejected critiques stay rejected: do not drop the verifier plane, do not
mandate Postgres for V1, do not collapse the five planes, do not replace
forge sovereignty with an internal merge queue.

## Documentation that is closed (do not reopen as a gap)

These used to look like missing product definition. They are defined and
fail-closed.

| Topic | Where it is closed |
| --- | --- |
| Public name, five planes, providers-propose | [`architecture/overview.md`](../architecture/overview.md), [ADR 0001](../decisions/0001-provider-execution-mode.md), [0003](../decisions/0003-five-trust-planes.md) |
| Competitor pins (README and paper: Gas Town, Gas City, DeepSeek Harness, Omnigent) | [Dated README snapshot](competitor-snapshot.md); [paper evidence lock](../paper/evidence.json) |
| IEEE preprint source | [`../paper/`](../paper/) |
| Why authority-bearing evolution is independently certified | [`evolutionary-control.md`](../architecture/evolutionary-control.md) |
| Evidence classes and skip-green ban | [`testing.md`](../testing.md) |
| C1–C12 / TEAM.md red-team | This page + paper Section “Red-Team Disposition”; `TEAM.md` is provenance |
| Mascot / brand briefs | [`../brand/mascots/`](../brand/mascots/) |
| Family-root `/docs/*.md` copies | Not authority. Hub `bullet-farm/docs/` wins |

## Documentation that must wait on typed commands

These are not missing definitions. Writing them now would invent a CLI that
does not exist. They become runbook work after the named command is real.

| Deferred doc | Blocked on |
| --- | --- |
| Upgrade / rollback / uninstall runbook | Signed prebuilt installer (G1, G9) |
| Signer rotation and schema-removal runbook | Release signing keys (G9) |
| SAFE_STOPPED / freeze-enforced operator card | Runner ack generation (C11, G3) |
| Effect-reconciliation operator card | Identity-exact adoption path (C9, G2) |
| Platform-refusal after mutation attempt | Native backends (G10) |
| Live-admission operator recipe | ADR 0012 ratification (G5) |
| Jeryu restore / read-back recipe | Operator forge auth (G6) |
| GitHub App test-repo recipe | Operator App + protected repo (G7) |
| Launch-grant keygen recipe | ADR 0011 + operator key (G5) |
| Jankurai CI pin admission | Portable checksum-pinned binary (G8) |
| `bullet-wire` / `jeryu-gitd` publication | Operator tags (G4) |
| Generated `check release` report as a committed score | Forbidden; the command wins |

## What this page will never say

- That a green `just fast` or a green component crate is a release.
- That the archived 2026-08-24 live demo certifies HEAD.
- That Bullet Farm is measured faster, cheaper, or safer than Gas Town,
  DeepSeek Harness, or Omnigent.
- That an agent may enable live admission or invent a schema-3 lock.
- That finishing the white paper, or this register, closed G1–G18.
