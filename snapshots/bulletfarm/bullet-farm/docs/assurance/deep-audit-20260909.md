# Bullet Farm: production and delivery audit, 9 September 2026

Status: source-grounded audit on **xbabe2**; production coding, complete hosted CI, useful dogfood and release remain **incomplete**. This report is a dated observation and repair queue under the [full-product plan](full-product-dogfood-plan.md), [G1–G18 register](product-gaps.md) and typed assurance inventory. It creates no new release authority and changes no product-profile result.

The implementation contains substantial tested custody, command, Git and browser components. The missing bridge is one durable coding transaction through those components, followed by three independently qualified subscription accounts. Installed CLIs, a green component suite, a good auditor score and a polished recording each establish different facts. None currently establishes that bridge.

The latest user instruction selects **GitHub `neverhuman/bulletfarm`** as the main public repository for all accepted work and CI. Supporting repositories may remain independent. Current configuration and several operating documents still select JeRyu; reconcile them without rewriting retained publication requests or dropping separately accepted forge-certification obligations.

**Audit coverage and evidence limits**

We studied all **101 Markdown files in the four member `docs/` trees**, totaling **1,845,777 bytes**, plus all six outer `docs/` files, which are exact byte mirrors. Ninety-nine member documents were read in full; the two large generated corpus/release pages received complete structured row comparison against their authoritative inputs. The source work inspected selected production boundaries, all eight nested workflow definitions and their execution routes, publication admission, current recorder scripts, installed executable identities and authenticated GitHub metadata. This is not an exhaustive code or penetration audit.

The reading was divided among root, CI/assurance, runtime/runbook review and implementation reviewers, with exact source snapshots and frequent family-chat coordination. All 101 documentation hashes matched the frozen inventory at final coverage read-back. A transient missing untracked media README during concurrent work was retained as an observation; it was present with its original hash at that read-back. Claims below distinguish **source inspection**, **executed reproducer**, **historical accepted proof**, and **external read-back**.

The local evidence bundle is `deep-audit-20260909` within the retained September 8 health implementation directory on xbabe2. Full SHA-256 subjects are recorded below; it is not yet a published or admitted release artifact. Existing failed attempts, incident objects and older reports remain intact. No provider turn, account enrollment, credential refresh or coordinator operation was executed by this audit.

**What is working**

| Area | Supported behavior | Practical limit |
| --- | --- | --- |
| Wire and authority components | Strict canonical inputs, generated contract checks, typed capabilities, stale-preimage/fence refusal and explicit UNKNOWN outcomes. | Contract completeness and signed production authority custody remain separate. |
| Kernel command ledger | Session-authenticated command submission, idempotency, durable command/outbox/event insertion and reconciliation exist. | The implemented dispatch kind is `run_demo`; settlement deliberately reports component UNKNOWN. |
| SQLite backup | Explicit connection closure, authentic current/prefix backups, retained failed staging, reopen and quarantine checks have accepted local proof. | A complete supervised 22→23 serving upgrade, interruption recovery and external authority high-water admission are still required. |
| Runner and BulletGit | An admitted UDS component, durable lease registry, private-clone proposals, preimage checks, Candidate preservation and hostile Git/configuration tests exist. | The production Runner chooses only `sim`; account-aware execution and atomic final completion are not connected. |
| Portal | Session/CSRF transport, validated unknown values, event-gap handling, command correlation, projections and a real-farmd component lane exist. | Lost-response identity, normal-event projection refresh, atomic Shift Brief and six operational surfaces remain open. |
| Publication | Deterministic aggregate construction/reconstruction, preserved v1 bytes, expected-old ref updates and ambiguous-response read-back exist. | Governed GitHub workflow publication, complete root CI and protected integration are not activated. |
| Media | Pinned offline generation produced four actual 1200×675 GIFs; two full output trees and an independent reconstruction matched. | The three CLI GIFs replay the same retained text; the Portal uses five still images. They demonstrate offline rendering, not three native coding sessions. |

Supporting component descriptions: [Kernel architecture](https://github.com/neverhuman/bullet-kernel/blob/8ddad3444da7ea548460b8b0c89600ad88e8d8b5/docs/architecture.md), [BulletGit architecture](https://github.com/neverhuman/bullet-git/blob/6cf10e70f1f03ab504f084ffa0b25cf97c7ab42a/docs/architecture.md), [Portal projections](https://github.com/neverhuman/bullet-portal/blob/e76eeb485eb8b438cafac0a05327cfbf38c38619/docs/projections.md), [current admission requirements](../runbooks/dogfood-admission-kit.md).

Accepted complete local proofs remain bound to their recorded subjects:

| Member | Accepted subject | Executed required evidence | Current-source limitation |
| --- | --- | --- | --- |
| Kernel | `21e504a486f6e1d3aa94dfcd4036a967bdd31d6a` | 1,074 fast + 34 contract tests; 470.949 seconds | Newer foreign `8ddad344` needs independent acceptance; three egress and nine family cases were outside the standalone run. |
| BulletGit | `f0c8605595564bf990b6a52735e8710b4e9a6440` | 62 fast + 160 contract tests; 135.802 seconds | Newer foreign `6cf10e70` is a different source subject. |
| Portal | `e76eeb485eb8b438cafac0a05327cfbf38c38619` | 153 unit tests, 14 mocked browser cases and five Node/Vite bundle contract cases; separate coverage passed unchanged floors | This is component/browser evidence, not account or production accessibility acceptance. |
| Hub/family | Historical exact receipts retained | Earlier member/family, publication and focused media proofs are useful | A complete clean four-member proof on the final accepted heads has not been established by this audit. |

Cursor reported additional required passes on newer review heads. Those reports require exact source/artifact read-back and independent change review; they are not silently substituted for the accepted receipts above. At source read-back the Hub was dirty, while Kernel, Git and Portal were clean. Cleanliness alone does not establish coordinator admission.

A later supporting-repository API read-back found all four GitHub source repositories also had zero workflows, zero runs and unprotected main. [Kernel PR 1](https://github.com/neverhuman/bullet-kernel/pull/1), [Git PR 1](https://github.com/neverhuman/bullet-git/pull/1) and [Portal PR 1](https://github.com/neverhuman/bullet-portal/pull/1) were merged at approximately 02:43 UTC to `8c9b8e8a`, `9600d668` and `22359616`. Those merges establish publication events, not successful hosted CI or independent acceptance of every change. Hub source main was `109ec83d` at that sample.

**Confirmed findings and the shortest repair**

| ID / severity | Evidence and consequence | Repair and direct acceptance test |
| --- | --- | --- |
| BF-A01 / high | Source: command insertion is atomic, but lacks production account authority, expected revision, launch nonce, quota reservation and allocated run in that transaction. `command_dispatch.rs` accepts only `run_demo`. | Extend the existing `/api/v1/commands` route. Concurrent identical requests allocate exactly one reservation/run/outbox; changed payload, stale revision, exhausted capacity or invalid authority leave no partial admission. |
| BF-A02 / high | Source: Runner CLI accepts only `sim`; `StartSession` uses `model: None` and `max_budget_usd: None`. Frozen provider probes do not enter this production route. | Carry admitted account, model/effort, runtime, snapshot/checkpoint, paths, policy/gates, reservation and deadline into one real Runner path. Prove it first with a fake provider through the actual API and ledger. |
| BF-A03 / high | Source: successful attempt discards `adapter.terminate` errors, records a successful termination message and returns success. | Propagate termination uncertainty; retain capacity and work until termination is evidenced. Pair normal completion with a successful task whose termination fails and a cancelled task with a surviving child. |
| BF-A04 / high | Source: SupervisorJournal logs write errors, sets its started flag before the first write succeeds and cannot return failure. Checkpoint publication writes/renames without file/directory synchronization. | Make required transitions fallible and durable. Inject write, rename, sync, malformed checkpoint and restart failures; none may become success or erase exact retry identity. |
| BF-A05 / high | Source: Candidate preservation is checked, but workspace cleanup precedes lease release; preservation, Attempt transition, audit and verifier enqueue are not one durable completion transaction. | Persist finalization atomically before cleanup. Kill at each boundary and reconstruct the same Candidate; repairs/rebases create new Candidates and invalidate affected review. |
| BF-A06 / high | Source: ControlTower creates an envelope inside submission and stores accepted state only in memory. Response loss/reload loses the operation key; another click creates a new key. | Persist the scoped pending envelope before sending. Test commit-then-drop-response, reload, terminal exact retry, conflicting digest and organization switch. Reconcile before offering a new operation. |
| BF-A07 / medium | Source: contiguous SSE events advance the cursor without refreshing views; other projections load once. Shift Brief composes independent changing row snapshots. | Refresh/invalidate by sequence and keep stale status until the snapshot covers it. Add one atomic brief read. Test normal contiguous updates, gaps, reordered responses, reconnect and restart. |
| BF-A08 / high | Executed: the native recorder returned zero after a child printed `CHILD_EXIT_1` and exited 1. Source also maps 129/130/143 to success, waits without a hard reap deadline and accumulates unbounded output. | Preserve exact child status; bound capture and owned-group teardown; retain partial/failed evidence. Test failing/signal exits, ignored TERM/HUP, descendants, infinite output and recorder interruption under an independent outer timeout. |
| BF-A09 / high | Source: the capture shell deletes ledger/WAL/SHM in reused cache storage, overwrites fallback attempts, writes a planned Portal transcript, and says no live provider was spawned after three native launches. | Use exclusive fresh private run directories; never delete old generation bytes. Retain every attempt and actual observation. Separate native invocation, capture success, structured completion and Bullet admission fields. |
| BF-A10 / high | Source: 16 new Hub generated-zone stubs are declaration-only; their named generators do not emit them, and the mapped sync checker does not read them. A Kernel binding stub names a non-writing test as its generator. | Remove the false declarations through reviewed corrective commits, preserving their history. Admit the auditor's reviewed-manual handling. Test actual generator ownership and drift, not the presence of a header. |
| BF-A11 / high | Source: three canonical formal JSON copies gained `//` headers and the consumer now strips comment lines. This changes the parser/input contract to satisfy an old auditor. | Restore strict JSON bytes and parser behavior unless a real contract change is independently justified. Keep canonical JSON hash equivalence and malformed/commented-input refusal. |
| BF-A12 / high | Source/read-back: generated root CI leaves 52 member definitions and all five final gates as deliberate failures; public main has no active workflow and is unprotected. | Implement real lane execution and exact result admission; publish with suitable authority; require the final stable check and reviewed integration. Reject absent/skipped/cancelled/neutral/malformed/stale results. |

Primary implementation locations: Kernel [command transaction](https://github.com/neverhuman/bullet-kernel/blob/8ddad3444da7ea548460b8b0c89600ad88e8d8b5/crates/adapters/src/sqlite/commands.rs), [dispatch](https://github.com/neverhuman/bullet-kernel/blob/8ddad3444da7ea548460b8b0c89600ad88e8d8b5/crates/adapters/src/sqlite/command_dispatch.rs), [Runner entry](https://github.com/neverhuman/bullet-kernel/blob/8ddad3444da7ea548460b8b0c89600ad88e8d8b5/apps/bullet-runner/src/main.rs), [attempt configuration](https://github.com/neverhuman/bullet-kernel/blob/8ddad3444da7ea548460b8b0c89600ad88e8d8b5/crates/runner/src/attempt.rs), [attempt completion](https://github.com/neverhuman/bullet-kernel/blob/8ddad3444da7ea548460b8b0c89600ad88e8d8b5/crates/runner/src/attempt/drive.rs), [checkpoint storage](https://github.com/neverhuman/bullet-kernel/blob/8ddad3444da7ea548460b8b0c89600ad88e8d8b5/apps/bullet-runner/src/supervisor.rs); Portal [ControlTower](https://github.com/neverhuman/bullet-portal/blob/e76eeb485eb8b438cafac0a05327cfbf38c38619/src/pages/ControlTower.tsx), [event stream](https://github.com/neverhuman/bullet-portal/blob/e76eeb485eb8b438cafac0a05327cfbf38c38619/src/hooks/useEventStream.ts), [Shift Brief](https://github.com/neverhuman/bullet-portal/blob/e76eeb485eb8b438cafac0a05327cfbf38c38619/src/pages/ShiftBriefPage.tsx). Snapshot ledgers retain exact bytes and source lines; supporting-repository permalinks bind the observed source commits.

BF-A10/11 refer specifically to Hub `32bae50e4bb22fdb41226db0fbf9a845301aa3db` and Kernel `8ddad3444da7ea548460b8b0c89600ad88e8d8b5`. Their full diffs are preserved. Root challenged both in family chat and requested independent review. The genuine schema bundle content was moved rather than regenerated; relocation itself is not evidence of a correctness defect. The false generator declarations and changed JSON parsing are the demonstrated defects. An independent scratch reproduction ran the exact copied sync checker with all 16 stubs absent and again with all 16 corrupted: both exited zero, while actual API consumer drift exited one. An emulation of the new formal reader also converted an illegal raw JSON string newline into valid text. No Rust execution is claimed for that emulation. No accusation about intent is needed to reject these changes as health acceptance. Subsequent Hub commit `4ecaa3374e90399abdc0a0d4e27a44e8615ec457` corrected the corpus test anchor and restored byte-copy synchronization to the older Kernel fixture path; it did not remove the inspected Kernel comment-stripping consumer or generated-binding stub. Its assurance test now expects the Hub stub paths. These later changes require their own review; the frozen documentation inventory and original reproducer are not relabeled as this newer subject.

Browser session/CSRF storage and authorization deserve direct negative testing, but an old heuristic report is not proof of a stolen session or broken boundary. Preserve validated `unknown` handling, legitimate `provider-runner` vocabulary, constant SQL fragments, hostile fixtures and crash-test children when repairing the actual findings.

**Jankurai: qualify the measuring tool before accepting the score**

The initial full score-90 audit produced Hub 65, Kernel 64, Git 64 and Portal 68, with 109 high and 56 medium findings plus 28 cap occurrences. These are **165 findings and 28 cap occurrences**, not 193 proven product defects. The queue must keep repository, fingerprint, source subject, owner, reproducer, disposition and acceptance evidence for each.

The installed auditor identifies as 1.6.11; the earlier report embedded 1.6.0. Canonical source contains locally reviewed policy/conformance/exit, generated-governance and heuristic corrections that the selected dependency graph does not yet prove it includes. Local Core 619-test, tools-kernel 136-test and analyzer 76-test results do not by themselves qualify a portable artifact. Later reported scores above 90 remain diagnostics until exact executable/dependency provenance, effective policy and scanned inputs are admitted.

The four wrappers are inconsistent: Hub policy floor 65; other policies 85 but CLI overrides lower effective floors to Kernel 57, Git 65 and Portal 59 and weaken severity selection. All four remove previous output. Some optionally run a ratchet before the required full scan. Fix this with one explicit policy per repository: **full scan, floor at least 90, zero caps, zero hard findings**, JSON/Markdown/repair artifacts and retained output generations. A diagnostic file or comment catalog does not prove the audit executed.

Admit existing canonical fixes before writing replacements. Reconcile family lock, immutable signed tags, Cargo selection, executable version, embedded report version and build provenance. Complete regression pairs for real authored `build/archive/backup` paths versus demonstrated outputs; validated versus unsafe unknown values; bounded versus unbounded frames; constant versus injected SQL; crash fixtures; prohibited-pattern catalogs; `.gitignore` prose; CSRF values versus authentication credentials. Headers and directory names must not exempt authored source from security scans. Reviewed-manual JSON remains byte-identical authored input.

Then build a checksum-pinned portable artifact and execute it in a clean CI-equivalent environment with source-preserving output handling. Keep security scans on their broader declared inputs. Explicitly review every remaining medium disposition and narrow exception. Current score stubs are a reason to repair the auditor/adoption boundary, not a reason to lower the acceptance floor. See [audit policy](../../agent/audit-policy.toml) and [standard binding](../../agent/JANKURAI_STANDARD.md).

**CI inventory and missing execution**

There are **53 nested job definitions / 55 expanded invocations**: 27 required and 26 scheduled definitions. Two OS matrices add the extra invocations. The root renderer adds five final gates, producing **58 definitions / 60 expanded invocations**. Only the Hub source-scan definition is connected to a real lane; the other 52 definitions refuse. A separate bootstrap implementation provides useful publication/reconstruction proof but is not another completed member job.

| Member | Required definitions | Scheduled definitions / expanded | Existing original timeout range |
| --- | ---: | ---: | --- |
| Hub | 7 | 8 / 8 | 8–35 minutes |
| Kernel | 7 | 7 / 8 | 5–35 minutes |
| BulletGit | 7 | 8 / 8 | 5–25 minutes |
| Portal | 6 | 3 / 4 | 5–25 minutes |

The renderer currently retains identity/dependency/matrix structure but replaces executable steps with refusals, drops original artifact/tool details and substitutes five-minute timeouts. It advertises `merge_group`, while event admission rejects that event. Preserve every original dependency, trigger, timeout, matrix and artifact requirement when implementing execution. The actual event `GITHUB_SHA` must remain unchanged; record member commits/trees separately. [Inventory](../../src/publication/ci_inventory.rs), [renderer](../../src/publication/ci_render/jobs.rs), [bootstrap validator](../../publication/ci-required.sh).

| Scheduling boundary | Required CI work |
| --- | --- |
| Ordinary credential-free PR / merge subject | Real member fast/lint/contract/security/docs lanes; existing source scans; exact expected-result final gate; one dependency-ordered family job; complementary admitted Rust toolchains/MSRV; real-farmd browser and embedded bundle; rendered keyboard/accessibility cases. |
| Scheduled and explicitly selected release candidate | Four portable Jankurai audits, history secrets, advisory/link freshness, coverage, bounded fuzz/soak, actual mutation threshold, sanitizer and CodeQL campaigns. Existing fuzz corpus replay and mutant configuration listing are useful but do not prove those campaigns. |
| Isolated privileged worker | Three Linux egress cases, real containment, hostile configuration, process death/restart, complete twelve-boundary fault campaign. Missing namespaces/tools or a sampler result must remain a named blocker. |
| Isolated credential worker | Individual Codex/Claude/Cursor account qualification, quota/refresh/cancellation/reconciliation, mixed-provider tasks and forge effects. Untrusted PR code receives no subscription home or release authority. |
| Clean installation / native workers | Signed Ubuntu package, two clean installs and lifecycle/upgrade/rollback/uninstall; later all five platform certifications and universal package checks. Portable refusal jobs are not native certification. |
| Deliberate product campaign | Twelve accepted tasks and seven-day survival; cognition/calibration/evolution study, shadow, authorized canary/rollback, team-v1 then saga acceptance at their prescribed later gates. |

Reuse `scripts/ci-local.sh` and the existing family orchestrator. Its seven stages already order Git proof/build, Kernel proof/family subset, Portal proof/real browser, and Hub contract/model checks. `family-contract` is an alias, not additional coverage. Reconcile `ops/ci/family.sh`'s rustup 1.29.0 guard with the admitted 1.29.1 subject before rerun. The complementary toolchain scripts can return success with `receipt_grade:false`; a gate requiring receipt-grade evidence must inspect that result. Node **22.23.2**, npm **10.9.8**, Cargo configuration and full tool subjects need consistent admission.

Each observation must bind member commit/tree, actual aggregate event SHA, workflow digest, run/attempt, matrix, selected/completed test identities and artifact digests. Preserve honest zero-test source-scan semantics; a required test lane cannot silently complete zero tests. Final success rejects missing, skipped, cancelled, neutral, failed, malformed or stale results. Measure cold-build duration, peak memory, disk and artifact volume before assigning hosted capacity or caches. Keep targets private and cold builds serialized during local qualification.

**GitHub publication and operator dependencies**

Authenticated read-back at **02:28:30 UTC on 9 September** found [public main](https://github.com/neverhuman/bulletfarm) at `f8ce28e6b0583160519e5898250904f63eee753d`: **zero workflows, zero runs, zero open PRs, zero rulesets, and main unprotected**. A second authoritative read-back at **02:51:36 UTC** observed the same main, zero workflows/runs/PRs/rulesets and unprotected main; Actions itself is enabled. These are dated facts; later source PRs are not automatically root aggregate CI. [Actions](https://github.com/neverhuman/bulletfarm/actions) must be read back again after publication.

The available `gh` token has repository admin/push capability but lacks the `workflow` OAuth scope; the connector is pull-only. Separately, governed publication requires a repository-scoped App installation token and Bot PR author. A personal admin token does not satisfy that admission unchanged. Distinguish permission, token scope and product policy; do not claim no GitHub access exists. [Transport admission](../../src/publication/transport.rs), [PR checks](../../src/publication/pull_request.rs), [currently stale destination](../../publication/config.json).

Prepare the complete GitHub-destination source/workflow packet before the final credential checkpoint. Use the existing expected-old and read-back machinery, preserve private origins/source objects and old request identities, and create a new request after source or destination changes. Protect main with the stable required check, human approval, stale-review dismissal and force-push/deletion prohibition. Verify both the tested merge subject and resulting main, with run/attempt/artifact read-back. Refresh remote ancestry immediately before any ref change; a newly advanced base requires reconciliation and renewed proof. Do not purge retained incident or dangling objects as part of synchronization.

The remaining external dependencies are concrete: admitted workflow-capable publication authority; preserved-generation operator checkpoint and enrollment; authenticated independent review; isolated credential/privileged/native workers; and elapsed survival observation. Engineering that does not depend on these should continue. An absent credential does not explain missing Runner/CI implementation.

**Three real subscription accounts on xbabe2**

Isolated `--version` probes observed Codex **0.153.4**, Claude **2.1.266**, and Cursor/agent **2026.09.08-6caf4ff**. Wrapper hashes are discovery evidence, not complete executable dependency closures. No authentication or account-read probe was executed by this audit. The account may already be logged in; that still does not establish a Bullet enrollment, reservation or qualified runtime.

| Provider | Qualification that must run against the installed subject |
| --- | --- |
| Codex | Inspect installed App Server schema; select explicit model/effort/cwd; preserve native session and notifications; validate structured proposals; acknowledge interruption; reconcile restart and multiple rate-limit buckets. Official account endpoints distinguish account identity, legacy rate limits and optional per-limit buckets. Missing/null quota is not zero usage or unlimited allowance. [Official App Server account documentation](https://learn.chatgpt.com/docs/app-server#auth-endpoints). |
| Claude | Qualify stream-JSON events, explicit model/effort, structured proposal, session/termination and subscription authentication under controlled startup configuration. `--bare` skips OAuth/keychain and therefore is not a subscription-preserving containment shortcut. Non-bare headless execution can load settings/hooks/plugins; inspect actual installed behavior and prevent repository startup execution. [Official programmatic guide](https://code.claude.com/docs/en/headless). |
| Cursor | Use actual ACP initialization, `cursor_login` authentication, session creation/loading, model/mode selection, streaming, permission responses, forbidden extensions and cancellation. An unanswered permission request can block a session. The existing headless plan helper does not establish native ACP qualification. [Official ACP guide](https://cursor.com/docs/cli/acp). |

Current online documentation informs the qualification plan; it does not prove that each installed version supports every described feature. Bind account identity/private home, credential generation and serialized refresh, complete runtime passport, actual requested/observed model/effort, snapshot/checkpoint, paths, gates, reservation and deadline. Prevent repository-controlled configuration from executing before the containment boundary is effective.

Persist vendor quota separately from Bullet invocation usage: sparse updates, external usage, expiry, contradictions and exhaustion matter. Manual snapshots stay `OPERATOR_REPORTED`, have finite allowance, expire within one hour or reset, and cannot override later vendor exhaustion. Warn at 80%, alert at 95%, pause at exhaustion; unknown charges remain `UNPRICED`. Retain one invocation per account, two implementation workers, two repairs, one escalation, eight invocations per task and a 60-minute deadline. Pauses/account changes/delegation do not reset limits.

The coordinator Operating HOLD remains effective. Complete both incident locations, retained bytes/replay dispositions, four source subjects, authenticated independent review, omitted-input refusal, final-lock revalidation and restart-safe admission before the operator checkpoint. A fresh preserved development generation does not recover the original incident. New execution tables must enter through the supervised upgrade mechanism: qualify 22→23 first and add new numbered migrations for new tables. Require exclusive maintenance custody, verified prefix backup, transactional migration and reopen/read-back. Rollback retains quarantine and external authority high water. [Operating rule](../runbooks/fleet.md), [admission kit](../runbooks/dogfood-admission-kit.md).

**What a convincing demonstration must show**

The requested coding-harness exception permits a clearly labeled native-account illustration. It does not confer production admission on an unfinished product path. Current new recorder prompts request a six-line explanation without file changes; that cannot prove coding on Bullet Farm. A production demonstration should follow one actual bounded issue from task submission through account/model selection, progress, interruption/reconciliation, proposal diff, independent review, human integration and exact resulting tests/commit. Include a mixed-provider handoff and a retained failed attempt. After the path exists, select genuine small Bullet Farm fixes so the product visibly improves itself.

The already verified offline renderer produced 46 matching artifacts across two complete generations and a fresh checker reconstruction. Three CLI GIFs are byte-identical, eight-second text replays; the Portal GIF is nine seconds from five screenshots. These are real rendered files with honest UNKNOWN metadata, but they do not show native provider TUIs or live production work. Retain them as component history.

The untracked HQ pipeline needs repair before live use: exact exit/status preservation, hard bounds, fresh private ledgers, exclusive per-attempt evidence, actual Portal observations, checksum-admitted tools/fonts, secret-aware sanitized derivatives and final artifact read-back. Its prompt-echo grep is not completion evidence; its hardcoded transcript and account fields must become observed data. Raw private captures must not auto-publish. Process groups cover only processes remaining in that group; they cannot prove containment of daemonized descendants.

For image quality, capture at final display resolution with a bright high-contrast theme, readable large text, stable viewport and no fade/dimming. Keep original RGB PNG frames and timestamps, or an admitted lossless video master. Avoid WebM→MP4→GIF as the source path: enlarging a lossy intermediate cannot restore detail. Pin renderer, fonts and every tool dependency; use nearest-neighbor only where appropriate for pixel art, and avoid unnecessary resizing of text.

GIF has a palette limit of at most 256 colors per frame; disabling dithering does not make arbitrary RGB capture lossless. [FFmpeg palette documentation](https://ffmpeg.org/ffmpeg-filters.html#palettegen-1). Preserve a lossless master, compare decoded GIF pixels against it, and report exact equality or quantization error honestly. If exact GIF equality is required, use a capture/theme whose frame color set fits the palette and prove round-trip equality. Otherwise distribute the lossless master alongside the GIF and label the latter a palette-quantized derivative. Brightness/stddev thresholds alone prove neither sharpness nor fidelity.

Acceptance should inspect actual first/last/transition frames, verify dimensions, frame count/timing, sharp text, high contrast, palette/decoded-pixel results, input/tool/output hashes and no unintended private data. Keep actual failures and attempted cancellations. A good-looking image must never change UNKNOWN into VERIFIED.

**Vision that should drive the remaining work**

The strongest common goal across Centerrail, Nightshift and the current plans is **verified engineering changes that survive, per human-minute and provider cost**. Agent count, transcript volume, test-count growth and audit-score movement are not that outcome. Keep SQLite authority, immutable Candidate/Change lineage, permanent fences, typed proposals, independent verification and one externally observed effect/read-back. These controls address real failure modes.

Start with a fixed small task graph: dependencies, acceptance criteria, exact owned paths, context capsule, bounded handoff, independent review and human integration. One useful Shift Brief should show the current task/account/model, quota source/age, exact unproved claim, next action, proposal/review state and evidence. Connection status and a moving cursor are insufficient. Distinguish mechanically tested, reviewed, integrated, deployed and survived states.

Adaptive councils, broad fusion dashboards, learned routing, cognition studies and distributed sagas remain accepted later obligations. They need not precede the first useful three-provider transaction. Historical self-scores, pseudocode, old model lists and dated competitor studies are not implementation evidence. The corpus has **33 IMPLEMENTED, 592 PLANNED, 20 SUPERSEDED and 3 REFUSED units**; this is a disposition count, not a percent-complete estimate. All 43 release claims and all 18 product plus two diagnostic profiles are currently BLOCKED. The invariant inventory has 51 rows, only seven enforced; closed cross-references are not runtime closure.

**Concrete overengineering and avoidable churn**

1. Remove declaration-only generated stubs and false generator metadata. Repair the auditor where it misclassifies reviewed-manual contracts; do not change protocol bytes to please it.
2. Replace formatter compression with responsibility-based modules. [ADR 0019](../decisions/0019-w11-proof-support-correction.md) documents 66 `rustfmt::skip` sites added to satisfy physical line limits; that makes the metric defeat readability.
3. Derive test inventories once and review semantic deltas. [ADR 0020](../decisions/0020-w11-test-inventory-baseline-correction.md) requires a new decision and two adjudications for six inventory literals. Repeated frozen counts/ADRs create drift; retain independent completion checks without duplicating administrative state.
4. Keep one current checkpoint, a dependency plan and a gap register with distinct purposes. Link historical receipts instead of copying the same long narrative into three active documents after every packet.
5. Use existing lane and family scripts. Avoid a second universal CI engine, duplicate validator schema or another orchestration service merely to execute them.
6. Reduce repeated command-history scans and per-row API reads. An indexed command projection and atomic brief can remove unbounded polling work and inconsistent N+1 snapshots while retaining mismatch refusal.
7. Separate ordinary decoder/component feedback from the later independently custodied release-host evidence requirement. Keep the evidence ceiling explicit; a narrow governing amendment is needed where current W11 rules require more. Do not silently weaken signing or hostile-runtime controls.
8. Consolidate process ownership only after preserving actual bounds, descriptors, teardown and failure evidence. The existing Rust pipe supervisor is not a drop-in PTY recorder; an adapter should be a bounded follow-up, not a third broad framework.

These changes reduce work without removing actual authority, retention, interruption or review boundaries. The accepted full program remains intact.

**Active documentation repairs**

| Current contradiction | Required correction |
| --- | --- |
| JeRyu-primary publication/config/source-setup language | Apply the user's GitHub primary destination, preserve historical requests and keep remaining forge profiles explicit. |
| `dogfood.md` recommends coordinator status while fleet HOLD forbids every coordinator verb | Point active status guidance to the non-coordinator fail-closed board; keep initialization unavailable until admission. |
| `live-conformance.md` / signer rotation recommend policy deletion or replacement | Use implemented immutable successor/revocation/high-water semantics; mark production rollback unavailable until proved. Deleting a policy is not revocation. |
| Old schema-removal advice says fresh data is the only route | Document current verified prefix backup/export without claiming serving upgrade completion or recommending data deletion. |
| Old error/status/test-count and Portal embedding statements | Reconcile with current source and exact observed inventories; distinguish mock, real daemon and signed installed package evidence. |
| Source bootstrap mixes incompatible family schemas and first-Ubuntu/five-platform manifests | Give one supported command/input profile and an honest refusal until its prerequisites exist. |
| Spec README says outer paper copies differ; they now match | Refresh the mirror statement from exact bytes while preserving dated design provenance. |
| Frozen generated corpus points a test at `main.rs`; it moved to `main_tests.rs` | Later Hub `4ecaa337` repairs the authoritative anchor and generated row; verify that packet. The original issue was routing drift, not a missing test. |

Do not rewrite historical ADRs as though old observations were current. Repair active commands and add clear superseding links. Security rollback and lost-operation advice take precedence over cosmetic documentation cleanup.

**Implementation sequence and measurable finish lines**

| Stage | Concrete work | Exit evidence |
| --- | --- | --- |
| 1. Trustworthy health | Reconcile foreign changes, correct generated-workaround defects, admit canonical Jankurai and full policies, finish mapped repairs and clean member/family proof. | Four exact accepted commits/trees; full scans ≥90, zero caps/hard findings; reviewed medium dispositions; no receipt rebinding. |
| 2. GitHub delivery | Implement real root profiles, family/toolchain jobs and observation/final-gate admission; correct destination; prepare governed publication and protections. | Exact aggregate reconstruction, applicable hosted jobs pass, protected tested merge and resulting main with artifact read-back. Scheduled/profile campaigns remain separately named. |
| 3. Reliable production core | Complete preserved-generation admission and supervised upgrade; extend typed contracts; wire atomic coding admission, Runner, termination/journal and Candidate finalization. | Fake provider completes API→ledger→Runner→containment→BulletGit→supervisor→artifacts→verifier with duplicate, response-loss, process-death and restart negatives. |
| 4. Three accounts and useful operator loop | Qualify each actual subscription and runtime closure; fixed task graph, durable controls, quota, account selection, review and atomic Shift Brief. | Each provider independently completes the same production route; operator can understand, interrupt, review and reconstruct it after reload/restart. |
| 5. Useful dogfood | One Rust, TS/React, test and documentation implementation per provider; mixed work, independent review, failures/retries/fallback and real media. | Twelve accepted tasks with retained failures, exact model/source evidence and seven-day survival measurement against comparable manual coordination. |
| 6. Full completion | Complete custody/fault/lifecycle/Ubuntu/Jeryu, cognition/evolution, remaining forge/platform/universal, team-v1/saga and active post-v1 obligations. | All G1–G18 evidence, all 18 independently passing product profiles, every accepted change on public main and complete applicable release CI. |

Measure active human minutes, elapsed integration time, review/recovery effort, duplicate work, repairs, quota use and escaped defects. Forecast dates only after two measured implementation cycles; account separately for worker/credential waits and the mandatory seven-day observation. A burst of successful CLI output cannot compress that window.

**Retained evidence subjects**

| Artifact in the local audit bundle, unless stated otherwise | SHA-256 / exact identity |
| --- | --- |
| Frozen `docs-inventory-r1.json` | `5b67ed6d3ab33566d1fe4088180df39d5ac1e123c92b56e69671e503c4c82b47` |
| `complete-document-coverage.json` | `a9e66fba54405fd3e6805dc3b190ebe08f6c2d2b9d27ced400bc6fb7d82ba19b` |
| `generated-docs-analysis.json` | `92250a8aaebef6208bfcb88c12574746964321549dd0004afd31f4c2a9ddeae2` |
| CI source/report packet `ci-assurance-packet.json` | `05db39dc5f48923e40e2c31d7b275865af2dde8a5f04586f48751f9b81e72859` |
| Runtime/runbook report `reviewer-runtime.md` | `c8a3768cd88368e7b59609384e120ceb624c8bae692ea5515edcff1ed16f68e9` |
| Runtime full-read ledger `reviewer-coverage.json` | `a56028589bdb2aeb57745f54030fb36120ae27fca5f0b59430bed83b0b1a26b0` |
| `implementation-complexity.md` | `2fbb8de57d19fb38d785574b6308ab5ec2c5ba72045854b70af3b01318b7a428` |
| `github-readback-r1/readback.json` | `1e72d091a0affde75a7bbfb58167483df59758cc713dce96917dde3900ae875c` |
| `github-readback-r2/readback.json` | `37fd1f4ccb7cd93a22e6671e31f87592963e72ee508ba2ad3fb6ec9b0adea41d` |
| Supporting GitHub source read-back `source-github-readback-r1/readback.json` | `bee54f7fad6d58cd2c6d376b4b2f2a057c7245aaccf8259b17b358db6490aa82` |
| `source-readback-r2.json` | `4e0ec5bd53824ed80123adf92a2bad6ded377c8d0f486b0c8b55883542ce4a90` |
| `installed-tools/versions.json` | `2ebfd87f83d949bf9a7ea31a42c869de4461cb502206ee8f38659280370cb304` |
| Recorder failing test `media-negative-r1/result.json` | `b5c242cf3378a8cebba2ef38751c334ccca59fcdd6b5d9c6ad8fc46df1c9c341` |
| `recording-proposal-r1/proposal.md` | `b9ca0fe49d2149da492af3165f2d27eb9800710e7c7233a52f399fb3eb23f38d` |
| Offline media evidence in sibling `readme-offline-runtime-r1/media-evidence.json` | `6f25e68742cdf0ecbfccc50583f5b153b0aee710928c8e5c127cf024ed94126a` |
| Independent `offline-media-review.json` | `d0421630c3c44b7ea952deaf00d38e59cf1ae02fc3614cea4cf336412b37f573` |
| Independent score-workaround review `score-workaround-review-r1/review.md` | `a6c2ac763a3ca0052f0d9ffa84a09555ad00a0f8e3f9c9393ca4b01bbffcd003` |
| Score-workaround negative reproduction `score-workaround-review-r1/reproducer.json` | `510477dd914fc4485dd9fb42a7654f371c6324af57abaaff13bf47d5902f479c` |
| Hub score-workaround diff | `a14df7fb45d7d3d2181ff37c58586aec30321745da3cccfa6031f57216c2fa49` |
| Kernel score-workaround diff | `98090f53714c48505e9dcc8766cacdf3d0f9885850c85ba9d9203f89a24e3138` |

The four observed source commits/trees are Hub `32bae50e4bb22fdb41226db0fbf9a845301aa3db` / `615b8826b2bcee9de4d611c82d653ef5f7fdd570`, Kernel `8ddad3444da7ea548460b8b0c89600ad88e8d8b5` / `7c59cf6c51451034bdc15a3f975e75f3fe26c9cd`, Git `6cf10e70f1f03ab504f084ffa0b25cf97c7ab42a` / `bb20b4afdfbf894b684ade83548fec861387b6b4`, and Portal `e76eeb485eb8b438cafac0a05327cfbf38c38619` / `7671ce0a135e13827543ac4111624d14c8f4c8f5`. These are observations, not blanket acceptance of foreign commits.

Final release read-back must additionally name source/public PRs, merge commits, workflow digests, runs/attempts, artifacts, packages and each profile receipt. They do not exist as a complete successful set today.
