# xbabe2 development closeout

Status: **Implementation detail; no production or release admission**  
Owner: Bullet Farm maintainers  
Last reviewed: 2026-09-09

This is subordinate to the [full-product plan](full-product-dogfood-plan.md),
[G1–G18 register](product-gaps.md) and existing typed profile inventory.
The [health checkpoint](health-checkpoint-20260909.md) records accepted subjects
and retained failures. Those authorities and the existing DF/WP identifiers remain
unchanged; this document supplies the detailed execution and verification order.

This work order refines the existing DF packets for everyday development on xbabe2.
It does not replace the gap register, change a profile definition, or treat a
native CLI demonstration as the production transaction. Recorder, browser capture,
renderer, verifier, fixtures and local commands are product source: they must be
tracked in the supporting members and included in `neverhuman/bulletfarm`.

## Current evidence and immediate corrections

| Subject | Established evidence | Remaining qualification |
| --- | --- | --- |
| Hub `258c8467`, tree `2cfe419a` | Complete required lane: 819 distinct passing tests, all five mapped lanes, pinned models and independent review. GNU time 22:41.98; maximum child RSS 1,792,340 KiB; new private target about 4.17 GB logical. | Later changes need their own mapped proof; this is local component evidence. |
| Kernel `02bf7c5`, tree `6fd94de4` | Complete standalone required pass: 1,075 standalone plus 34 contract identities. | Later native-provider source at5858e843 and current separate edits remain subject to independent review, inventory regeneration and complete proof. |
| Hub publication `eb6c9cc` / `912dddf` | Source timeout budgets and GitHub destination integrated after independent review, 36 publication tests and 50 actual wrapper cases. | New exact-source aggregate, complete mapped proof and hosted activation remain required; Current974f5de2 adds actual BulletGit source-scan transfer/activation;51 other member profiles and five final refusals remain unavailable. |
| Portal `29d02c34`, tree `e6f8dc71` | Complete standalone required:183 unit tests,14 mocked browser tests, five bundle tests, all five lanes; independent artifact readback9c0343ca;92.234732seconds. | Actual farmd, coverage, family, hosted and native-provider campaigns retain separate acceptance. |
| Four-member family retry | BulletGit, Kernel standalone/family and Portal standalone checks completed. The run then correctly refused changed Kernel source after stage 5. | No complete family PASS. Stages 6–7 did not run; do not attach earlier successes to the changed source. Preserve both the earlier temporary-directory failure and this source-drift failure. |
| Primary GitHub | Fresh 9 September read-back still identifies `f8ce28e6`, zero workflows/runs/rulesets/open PRs, unprotected main. | Full source publication, workflow execution, required checks, review protection and tested-main read-back. |
| Auditor | The immutable Jankurai 1.7.0 artifact has verified GitHub provenance. Its selected dependency closure omits reviewed fixes. | An admitted source closure, detector regressions, reproducibility/clean-environment execution and actual full audits. Provenance alone admits no score. |
| Media | Actual recorder, browser, RGB/master and GIF-timing component tests; retained synthetic browser frames and private historical captures. | Real application/backend/provider activity, actual Codex/Claude/Cursor/Antigravity implementation tasks, production TUI capture, 1080p geometry, accessibility/color acceptance and export verification. |

The initial 165 findings and 28 cap occurrences remain triage inputs, not 193
confirmed defects. Every disposition needs the source subject, finding fingerprint,
reproducer, repair or auditor correction, independent review and acceptance receipt.

Native gate-selection and capture behavior needs exact-source qualification: validate a nonempty,
bounded selection against the authoritative gate catalog before preparing or
launching any provider. Construct request/transcript validation before dispatch.
Unknown or stale gates must cause zero provider launches. A newly added error enum
must distinguish a failure before spawn from a failed capture after spawn. Preserve
failure evidence and uncertain cost for the latter, including read failures,
truncation, canaries, nonzero exit, timeout and missing terminal frames. A forked
process, a CLI exit and a billed model turn are separate observations.

## Packet order and acceptance contracts

Each row is a workstream implemented as separately reviewed packets of at most
four claimed files. A row is not permission to claim its entire surface. Root owns
integration; two workers implement disjoint packets; an independent reviewer
challenges both the implementation and the evidence. Canonical checkouts only.

| Order / governing packet | Concrete implementation | Required evidence before advancing |
| --- | --- | --- |
| H1 / DF-W0a | Finish the reviewed timeout, GitHub destination and documentation packets. Reconcile new Kernel changes; preserve all failed runs and uncommitted source. | Focused positive/negative tests, exact file hashes, independent review, clean committed subjects and regenerated inventories. No static success badge substitutes for a run. |
| H2 / DF-W0a/b | Complete clean member checks and dependency-ordered family proof using private targets and a short, private temporary root. Obtain all writers' freeze acknowledgement first. | Exact four commit/tree pairs, raw selected/completed IDs, fresh reports, no failed or skipped selected tests, exact declared profile selections/exclusions, no unexpected zero-test execution, final source/config/tool read-back, released proof locks. A source change invalidates the combined run. |
| H3 / DF-706 | Admit the canonical Jankurai family, including policy/exit/conformance consistency, inventory classification and safe/unsafe detector pairs. Replace the Python auditor launcher with the admitted Rust executable. | Immutable component tags and full source/dependency provenance; checksum and signature verification; clean-environment execution; tests retain unsafe-case detection. Keep old reports. |
| H4 / DF-706 | Close actual security/correctness findings, contract-purpose drift, generated/manual misclassification, owner/test omissions, large authored files and duplicated verifiers. Unify each repository's full-scan policy. | Actual full audits at floor 90, zero caps and zero hard findings; all remaining medium dispositions explicitly reviewed. No lower CLI override, fake generated header or broad path exclusion. |
| C1 / DF-W0a, DF-706 | Activate exact-source aggregate CI using the current 53-job/55-invocation catalog and existing lane scripts. Preserve timeouts, dependencies, matrices, triggers and artifacts. | Every implemented job executes its real lane and validates retained artifacts; unsupported profiles fail explicitly. Complete applicable PR and deliberately triggered scheduled campaigns pass on the exact candidate. |
| C2 / DF-104, DF-502 | Synchronize accepted member refs, immutable tags and the primary aggregate. Use a new publication request for changed subjects or destination. | Fresh remote ancestry/base read-back, expected-old writes, complete history scan, exact source tags/review branch, PR, tested merge subject, approval, protected integration and resulting main/run/artifact read-back. |
| R1 / DF-R4–R7, DF-DOG0 | Complete both-location preserved-generation admission and final locked validation; persist admission references and exact retry/restart handling. | Reviewed retained inputs, authenticated independent review and the actual operator checkpoint. Preserve the incident separately. An omitted optional argument is not admission. |
| R2 / DF-204/205 | Implement supervised schema 22→23 upgrade and subsequent execution migrations through it. | Exclusive maintenance custody, prefix-aware verified backup, one transactional migration, reopen/read-back, external authority high-water, interrupted retry, collision handling, quarantine-preserving rollback. |
| T1 / DF-101–103, DF-201/202, DF-602 | Define accounts, enrollments, quota buckets, reservations, launch grants, operational runs, proposals and operator commands in the authoritative wire source; generate Rust/JSON Schema/OpenAPI/TypeScript. | Closed schemas, canonical bytes, unknown/duplicate/stale/overflow refusals, generated drift checks and contract agreement across all consumers. |
| T2 / DF-202/203 | Connect `POST /api/v1/commands` to one durable admission transaction: authority/revision/idempotency, capacity reservation, nonce consumption, run allocation and dispatch outbox. | Concurrent submissions, duplicate keys, lost response, process death at every write boundary and restart return one persisted command/run without duplicate dispatch or capacity. |
| T3 / DF-302/303 | Connect production Runner to exact account/model/effort/runtime/snapshot/checkpoint/policy/gates/reservation/deadline; persist supervision and results. | Hostile startup/configuration, resource exhaustion, ambiguous start, acknowledged interruption and process-tree teardown proofs. Capacity remains held until termination or authoritative reconciliation is evidenced. |
| T4 / DF-204, DF-401 | Repair termination ordering and durable stop ownership. Success, cleanup and lease release must follow verified termination; unresolved execution must not expire into redispatch. | Failed/false/missing acknowledgement, capture failure, supervisor death, heartbeat expiry and restart cannot report success or authorize a successor while execution may survive. Test both immediate cleanup and delayed TTL reclaim. |
| T5 / DF-401/503 | Complete preimage-bound proposals and atomic Candidate finalization: preserved Candidate, Attempt transition, lease settlement, audit event and verifier outbox. | Invalid proposal validation occurs before consuming a one-use mutation permit where no effect has occurred; stale bases, limits, interruption and response-loss proofs preserve original identities. Rebase/repair creates a new Candidate and invalidates affected review. |
| T6 / DF-501/502 | Connect independent verifier and governed human integration. | Writer cannot issue its own accepted review; exact Candidate/ProofBundle/check/merge bindings; missing, stale, skipped or failed evidence refuses; post-integration reconstruction matches source and target. |
| U1 / DF-601–605 | Add task graphs, dependencies, acceptance criteria, exact path ownership, context capsules, handoffs, review/cancel/freeze/resume/account commands and an atomic Shift Brief. | Durable command IDs survive response loss, reload, reconnect and server restart; malformed/stale/gapped SSE triggers reconciliation; organization isolation, session/CSRF/RBAC and accessibility tests pass. |
| D1 / DF-701 | Run a fake provider through the same real API/ledger/Runner/containment/BulletGit/supervisor/artifacts/verifier path before credentialed coding. | No alternate simulator-only ingress or fixture self-signing can promote this to production acceptance; retained failure matrix and independent reconstruction pass. |
| P1 / DF-303, W8 | Independently qualify Codex, Claude and Cursor subscription/runtime/account closures on xbabe2. | Each provider completes the real transaction with exact model provenance, quota evidence, safe startup, interruption, isolation and failure/restart receipts. A successful provider cannot qualify another. |
| M1 / DF-604/706 | Build and verify real 1080p TUI and web recordings locally using tracked Rust/TypeScript tooling. | Native frames, actual production task IDs and timestamps, exact tool/source subjects, pixel/timing checks, original lossless masters and an independently checked export. |
| D2 / W8 internal campaign | Run twelve bounded accepted implementations, four per provider, plus mixed-provider collaboration using Bullet on Bullet. | Exact task/attempt/model/account/runtime/source/proposal/review/test/merge/receipt chain; failed attempts retained; operators can interrupt, recover and explain every outcome. |
| D3 / W8 observation | Observe the integrated tasks for seven days against comparable manually coordinated work. | Measured human effort, time to integration, recovery/review effort, duplicates, repairs, quota and escaped defects; actual surviving changes, no synthetic productivity claim. |

T3 is provider-capable wiring only: it admits no live execution before T4
durable stop/reclaim guarantees, D1 same-stack fake-provider proof and the
required operational/account checkpoints. M1 tooling is implemented and tested
before D2; actual recordings span D2 and are retained through D3 observation.

T4 cannot be closed by moving an ignored `terminate()` error to a later return.
Current cleanup and heartbeat expiry can still release/requeue work. Paused states
also expire; retaining an absorbing quarantined Attempt with its lease can obstruct
global expiry, while releasing that lease removes writer custody. Define and test
the durable stop/redispatch ownership contract first. Reuse the existing retained
worker state and exact settlement machinery where suitable; introduce any new
execution tables only through the supervised upgrade mechanism.

## The real production connection, in implementation order

The latest instruction prioritizes fixing real production execution. A separately
launched signed-in CLI, read-only proposal receipt or harness-exception GIF does
not fulfill it. Keep simulator controls for deterministic fault tests, explicitly
selected and labelled; production selection must never fall back to them.

The source trace is pinned to Kernel `5858e843` / tree `af4b9138` and BulletGit
`6cf10e70` / tree `bb20b4af`, with Hub admission at `974f5de2`. External native
adapter changes after these subjects require separate review and new proof.

| Current boundary | Exact source and behavior | Work still required |
| --- | --- | --- |
| HTTP admission | Kernel `crates/adapters/src/sqlite/commands.rs::submit_command` atomically records command, dispatch outbox and event; exact retries validate all three. | Typed coding request, authority/revision, account capacity, run and launch-nonce binding in that same admission transaction. |
| Worker | `apps/bullet-runner/src/bin/bullet-command-worker/{child.rs,receipt/provider.rs}` selects `transaction_offline` and requires `sim`, credential-free, transaction-ineligible receipt fields. | Native production dispatch and authoritative settlement with durable recovery. Preserve the component receipt's original meaning. |
| Runner | `apps/bullet-runner/src/main.rs` accepts only `sim`; `crates/runner/src/attempt.rs::start_request` supplies no model/budget. | Native session factory and one admitted account/model/effort/runtime/reservation subject carried through the Attempt. |
| Claude | Application `dogfood_run.rs` invokes an enrolled real stream-JSON runtime under containment; it produces a read-only proposal and separately refuses general live authority. | Reviewed exit/usage fixes, explicit model/effort, cancellable native session and valid production launch admission. Provider-reported cost is not independently verified subscription billing. |
| Other providers | Native transport and protocol components exist. Codex dispatch retains a `0.0.0` runtime placeholder; Cursor and Antigravity text pings emit no normalized coding events. | Installed-schema qualification and real structured coding sessions for each account; text pings do not qualify Cursor ACP or Antigravity coding. |
| Writer | Gitd `daemon_handlers.rs::apply_proposal` consumes its permit before repository semantic validation; Kernel accepts1024 operations, Git128 plus32MiB aggregate. | Pre-effect validation, aligned contract bounds, retained final validation and direct refusal/ambiguity controls. |
| Completion | Runner `attempt/drive.rs` discards a termination error and cleans before lease release; this path does not atomically enqueue verification. | Durable stop ownership and Candidate/Attempt/lease/audit/verifier-outbox transaction before cleanup. |
| Verifier | Product `apps/bullet-verifier/src/main.rs` always refuses signed-intent admission. | Implement admitted request consumption, independently owned execution/artifacts and accepted evidence; then governed human integration. |
| Deployment | Schema inspection recognizes22-to23 but says the upgrade operation is absent. Coordinator fresh admission refuses incomplete incident input. | Supervised upgrade and complete two-location admission, with actual operator checkpoints. These are implementation tasks, not permanent excuses to use fixture authority. |

The following are subdivisions of R/T/P/U/M above, not new completion gates.
Path sets identify candidate edits; each owner must first re-read current source,
declare at most four exact paths, and split a larger change into compiling packets.
Independent review and mapped checks are required before integration.

| Packet / owner lane | Proposed implementation and existing mechanism to reuse | Direct acceptance and dependency |
| --- | --- | --- |
| T3a / worker1 | Finish Claude `src/{dogfood.rs,parse.rs,parse/results.rs}` and `tests/dogfood_dispatch_pairs.rs`: validated usage prefix, failure-terminal usage, explicit successful exit and timeout handling, bounded cost conversion and opaque-capture UNKNOWN. | Actual harmless child exit0/1/signal/timeout, late malformed output after valid usage, overflow and capture failure; original Conformance event ordering unchanged. No new account call. This packet is currently under independent review. |
| T5a / worker2 | Gitd handler and workspace validation seam, plus direct daemon tests: perform non-publishing prevalidation (temporary indexes/Git objects may be written), validate malformed/bounds/attempt/checkpoint/scope/preimage subjects before a permit is consumed, then repeat necessary validation under writer custody. | Known no-effect refusal leaves permit available and daemon usable; valid retry succeeds. A failure after a possible mutation still becomes UNKNOWN and freezes. Never classify every error as safely aborted. Can proceed alongside T3a. |
| T1a / worker1 | Reconcile Kernel proposal Rust and authored JSON limits with BulletGit's128-operation/32MiB envelope; authoritative agreement tests and per-write UTF-8 byte validation. |128/129 operations, exact/over aggregate bytes, multibyte content, duplicates, unsafe paths and stale preimages; accepted proposal cannot exceed the writer's envelope. Do not enlarge limits to silence the mismatch. Before first real Candidate. |
| T3b / native adapter owner | Keep admission/containment composition in application or Runner; keep provider protocol/process ownership in harness. Reuse `HarnessAdapter`, normalized events, prepared command factory and externally held `DispatchSignal`. | No application↔harness dependency cycle; no simulator construction for a real provider; start/send/events/terminate belong to one invocation. A late or previous TurnCompleted cannot satisfy another turn. Depends on T3a; may be proved with harmless process fixtures before live authority. |
| R1a / integration + worker | Extend complete two-location admission types/consumer and persist references into Genesis. Revalidate retained inventories, replay/dispositions, independent review and four source subjects under the final initialization lock. | Omitted input, moved incident bytes, changed review or source, partial preservation, collisions and every process-death boundary refuse or retry exactly without losing bytes. Prepare the final operator packet before requesting the checkpoint. Operating HOLD stays effective until it passes. |
| R2a / storage owner | Build `bullet farm upgrade`22-to23 using existing verified prefix backup and schema inspection. Add explicit maintenance ownership, journal, transactional migration, close/reopen readback and quarantine-preserving rollback. | Competing daemon/upgrader refuses; injected failure and process death at backup/DDL/commit/reopen boundaries preserve recoverability. Retry uses the same intent, not a new database. External authority high-water cannot move backwards. Before new execution tables. |
| T1b / contract owner | Define a closed coding-command and admitted execution subject in authoritative contracts. Generate Rust/JSON Schema/OpenAPI/TypeScript through existing generators. Bind organization, task/revision, account/credential generation, model/effort, runtime/passport, source/checkpoint/scope, policy/gates, reservation, nonce and deadline. | Unknown/duplicate/stale fields, overflow, malformed IDs, wrong tenant, missing model and mismatched generations refuse. Generator drift and cross-consumer agreement pass. No untyped payload becomes execution authority. |
| T2a / storage owner | Through supervised migrations, connect account enrollment, quota buckets, invocation reservations, operational run and dispatch state. Extend the existing immediate command transaction; reuse exact request/retry/outbox/event logic. | Concurrent duplicates allocate one run/reservation/outbox and consume the authorized nonce once. Conflicting retry refuses. Failure at every write boundary leaves all-or-nothing state. Response loss/restart and an exact retry after authority later advances return the original identity without authorizing a second launch. Depends on R2a/T1b. |
| T4a / supervision owner | Persist execution start intent, possible/confirmed start, cancellation, stop observation and reconciliation; retain the exact owner/runtime/process subject. Reuse worker retained stages and authenticated readback. | Parent death before/after spawn, capture failure, lost start/stop response, inherited descendants, timeout and heartbeat expiry cannot free capacity or redispatch while execution may survive. Never signal a recycled PID/process group. Unknown liability remains reserved. |
| T3c / worker owner | Connect `bullet-command-worker` to a production execution manifest and native Runner subject instead of the fixture child/receipt selector. Keep fixture dispatch explicitly separate by admitted profile. | The actual HTTP request reaches the admitted native executable with exact account/model/effort/deadline. Missing native handler is a typed failure, never a simulator fallback. Restart reconciles retained state; stale or forged receipts cannot settle a run. Depends on T2a/T3b/T4a and admission. |
| T5b / storage + writer | Atomically finalize preserved Candidate, Attempt transition, lease settlement, audit event and verifier outbox; store exact proposal/base/head/tree and preservation identities. Perform workspace cleanup only after durable completion. | Kill/restart and lost response before/after each boundary retain one Candidate and enqueue one verification request. No further provider call to recover completed work. Rebase/repair creates a new Candidate and invalidates previous reviews. |
| T6a / verifier owner | Wire the existing clean-room verifier to signed VerificationIntent admission, trusted key lifecycle/durable nonce use, separate service UID and artifact custody, exact Candidate and immutable gate/tool subjects. | Writer cannot mint accepted review/evidence or alter verifier artifacts. Wrong/stale intent, replay, missing/skipped gate, changed tool/source or failed gate refuses. Actual verifier service runs and independent readback checks signed outputs. Prepare deployment inputs while coding; do not leave this layer permanently open. |
| T6b / integration owner | Connect accepted evidence and human review to existing expected-old integration and effect readback. Keep Candidate publication distinct from target integration. | A stale review/changed Candidate/advanced target blocks integration. Lost publication response reconciles exact original intent. Read back tested merge subject and resulting target tree; no self-approval by the implementation worker. |
| U1a / Portal/TUI owner | Add typed coding submission and persisted command/run projections using existing pending-command reconciliation. Show real account/model, active paths, dependencies, review, quota and stop/recovery status. | Actual farmd browser/TUI tests cover response loss, reload/reconnect/restart, malformed/stale/gapped events, organization separation and accessible focus/status. A green process exit cannot make an UNKNOWN run green. |
| D1 / integration + reviewer | Exercise these production services using a deterministic provider driver solely to force failures and concurrency; do not use the simulator-only command child as a second route. | Same ingress, admission transaction, workload worker, containment, writer, independent verifier and integration boundary; full duplicate/start/stop/finalization matrix retained. No live-provider credit. Before credentialed mutation. |
| P1a / account owner | Qualify Claude's admitted installed native schema, explicit model/effort, safe subscription home/configuration, structured proposal, notifications, cancellation and quota reporting. | One bounded real implementation through the production HTTP command, independently verified and human-integrated. Repeat exact submission and restart after response loss without a second invocation. No read-only explanation prompt. |
| P1b / account owner | Qualify Codex App Server from its actual installed schema/version; thread/session/turn correlation, explicit model/effort/cwd, structured proposal, permissions and acknowledged interruption; multiple quota buckets. | Independent real coding transaction plus schema/model drift, out-of-order events, malformed proposal and cancellation controls. Claude success cannot qualify Codex. |
| P1c / account owner | Implement and qualify Cursor agent's actual ACP initialization/authentication, session/model/mode, streaming/permissions, blocking extensions and cancellation. | Independent real coding transaction; unsupported modes/extensions and permissions refuse safely. A text-only Cursor ping is not an ACP pass. |
| P1d / account owner | Inspect and admit Antigravity's installed supported coding protocol and complete dependency/account closure; drive structured proposals and bounded cancellation through the same production path. | Independent actual implementation and failure/restart proof. No guessed ACP compatibility or empty event stream accepted as coding evidence. Required for the requested four-provider media. |
| M1 / media owner | Migrate owned PTY/supervision/manifest/render checks to Rust and browser code to TypeScript. Capture the actual TUI and styled Vite/React Portal from the same production ledger/run identities. | Native1920×1080, bright accessible colors/sharp text, continuous timestamps, original lossless masters, independent decoded-pixel/timing inspection and each GIF strictly<50,000,000bytes. All code and fixtures ship in main; no staged screenshots or capture output replacing the product path. |

The first live acceptance task should be small enough to review completely: one
real Bullet Rust or TypeScript behavior change with an existing test that fails
before the change and passes afterward. Select its exact source base, paths,
expected behavior and deadline before launch. Record the authenticated command,
run/Attempt, account generation, model/effort/runtime, actual events and exit,
proposal/preimages, Candidate, gate outputs, independent review, human integration
and final reconstructed tree. Keep account secrets private; derived public evidence
identifies the account generation without exporting credentials.

First prove Claude end to end, then independently Codex and Cursor, then
Antigravity. This is execution order, not permission to omit a provider. Continue
the twelve-task campaign and mixed-provider change below; Antigravity adds its
qualified implementation tasks before claiming the four-provider demonstration.
The seven-day survival window starts after actual integrated changes exist.

## Fast verification while implementing the connection

Use direct existing lane commands in private targets. For Kernel changes, run the
focused Rust test target first, then the repository's mapped `fast`, `contract`
or `required` lane according to the owner map. For Git writer changes, run the
direct daemon/refusal tests and complete member `scripts/ci-local.sh required`.
For Portal changes, use pinned Node22.23.2/npm10.9.8 for typecheck and focused
Vitest tests, then complete `scripts/ci-local.sh required`; mocked browser tests
and real-farmd tests retain separate identities. Root alone owns full proof locks.

Do not infer a production pass from these component commands. Add the actual
credential-free production transaction campaign to the existing lane scripts and
then the separately admitted xbabe2 native-account campaign. The latter must
invoke the product command and inspect durable API/ledger/worker/verifier results,
not just call `bullet dogfood read-only` or parse a supplied transcript. Its exact
reproduction command is published only after the implementation exists and is
tested; an invented CLI example must not masquerade as an executable runbook.

Measure two completed packet cycles before forecasting dates. The critical path
is admission/upgrade → typed durable run and stop ownership → native workload
worker → atomic Candidate completion → independent verifier/integration → actual
account qualifications. Writer repairs, provider protocol work, CI adapters and
capture-tool migration can proceed alongside that path within the two-worker
limit. Provision verifier/service custody, publication App authority and dedicated
workers early with concrete reviewable packets; preserve the final operator action
as a named dependency rather than consuming credentials through ordinary PR CI.

## Rapid CI without weakening the acceptance gate

Fast feedback and complete acceptance are distinct jobs using the same reviewed
scripts. Do not create another CI engine or silently substitute a smaller suite.

| Campaign | Execution and target | Required output / failure behavior |
| --- | --- | --- |
| Local packet loop | Formatting, type checks and the owning focused Rust or Portal tests, with a private build target. Measure warm and cold latency. | Exact selected/completed identities and failures. This speeds editing and does not grant the full required check. |
| Every PR | Source scan first, mapped fast/lint/contract/security/docs partitions, generated contracts, publication reconstruction and changed-seam adversarial proofs. | Full expected job/matrix set, actual aggregate event SHA, member subjects, tool/workflow hashes, run/attempt and artifact digests. Missing tools, tests or reports fail. |
| Family candidate | Dependency-ordered BulletGit → Kernel standalone/family → Portal standalone/real-farmd → Hub contracts/models; admitted daemon path and hash. | A clean four-subject observation plus all stage reports. No concurrent source/config mutation; no inherited old observation. |
| Scheduled candidate | History secrets, links, advisories/licenses, coverage, portable Jankurai, native-platform jobs and fuzz/sanitizer/fault campaigns required by the profile. | Deliberately dispatch against the candidate, retain run/attempt/artifacts, and verify each applicable campaign. Ordinary PR success cannot stand in for it. |
| Credentialed xbabe2 | Dedicated isolated worker for provider qualification, real coding tasks, interruption/restart, account/quota and native capture. | Private credential custody, sanitized derived outputs and exact execution receipts. Ordinary PRs receive no subscription, signing or publication credentials. |
| Privileged/lifecycle/native | Separate admitted workers for containment, service identities, upgrade/install/rollback and five-platform certification. | Named infrastructure blocker until actual execution. Cross-compilation or a workflow definition does not certify a platform. |

Hub commits5b11bfb,599b2ae and974f5de integrate BulletGit's actual source-scan
adapter, cross-job verified tool transfer and generated workflow activation.
Focused36 publication tests,50 wrapper cases and23 transfer controls pass locally;
these are component tests, not hosted job execution.51 other member profiles and
five final refusals still need execution support. Implement the remaining adapters
by reusing their reviewed lane scripts and exact validators; preserve original
matrices/dependencies/triggers/timeouts/artifacts. Keep unsupported profiles and
the final incomplete-campaign gate failing until implemented and executed.

Admit Rust toolchains/MSRV and Cargo configuration, Node 22.23.2, npm 10.9.8,
browser binaries and immutable tool subjects. Cache keys include member identity,
toolchain, lockfiles, lane/profile and trust boundary; an untrusted PR cache must
not supply release authority. Measure elapsed time, maximum memory, disk and
artifact volume before choosing worker capacity. The observed Hub maximum child
RSS is not a measurement of aggregate parallel memory. Serial cold builds remain
the local default until measurements justify a different budget.

The stable final required check rejects every missing, skipped, cancelled, neutral,
malformed, stale or failed predecessor or artifact. It verifies exact matrix
identities and selected/completed test sets rather than trusting a job name or
green process exit. Preserve real `GITHUB_SHA`; record source commit/tree separately.
Configure required checks, human approval, stale-review dismissal and no force-push
or deletion on primary main after the complete checks actually operate. Read back
both the tested merge subject and the resulting protected main.

## Separate subscription qualifications

| Provider | Required protocol/runtime work | Minimum negative proofs |
| --- | --- | --- |
| Codex | Inspect the installed App Server schema; explicit account, model, effort, cwd and session identity; structured proposals, correlated notifications, acknowledged interruption and multiple vendor quota buckets. | Wrong/changed model or schema, duplicate/late events, response loss, malformed proposal, tool permission refusal, interruption without terminal acknowledgement, exhausted/contradictory quota and restart. |
| Claude | Durable stream-JSON dispatch with explicit model/effort; qualify the installed schema and structured-output mechanism; safe configuration/startup with subscription authentication preserved. | Empty/stale gates before spawn, changed runtime, unknown authority-bearing fields, hostile settings/hooks/MCP/plugins, malformed tool/result frames, missing cost, post-spawn capture failure, timeout and cancellation. |
| Cursor agent | Actual ACP initialization and authentication, session/model/mode selection, streaming, permission requests, blocking extensions and cancellation. | Unsupported initialization/extensions/mode, rejected or malformed permission, stale session, changed model, missing terminal completion, interrupted start and cancellation ambiguity. |
| Antigravity | Inspect the installed supported coding protocol and runtime; explicit account/model/session, structured proposals, permissions and cancellation through the production worker. | Unsupported protocol or model, empty coding events, changed runtime, stale account generation, malformed proposal, ambiguous start/stop and restart. |

Use actual retained native protocol observations and official installed schemas;
synthetic fixtures remain regression inputs with explicit provenance. A single new
transcript does not justify accepting every unknown field. Classify new fields and
retain corresponding unsafe-case detection. Root-stage only the specifically
admitted executable closure, with digest, ownership, loader/libraries/runtime
files and a verifiable passport. A provider auto-update requires a new runtime
generation and requalification; it must not inherit the old passport.

Each account has a private home, credential generation, serialized refresh and
one active invocation. Repository-controlled startup code must not execute before
the policy permits it. Vendor quota remains independent of local invocation usage:
sparse updates, external usage, expiry, contradictory readings and exhaustion must
survive restart. Retain the existing 80% warning, 95% alert and exhaustion pause.
Manual snapshots are explicitly `OPERATOR_REPORTED`, finite, expire within one
hour or reset and cannot override later vendor exhaustion. Unknown charge is
`UNPRICED`, not zero. Provider failure cannot silently move work to another account.

## Twelve tasks and one demonstrable collaborative change

For each of Codex, Claude and Cursor, select one small Rust implementation, one
TypeScript/React implementation, one meaningful test change and one documentation
implementation from the live Bullet backlog. Freeze each task's baseline, accepted
paths, expected behavior, negative case and review criteria before invocation.
Use exact account/model/runtime identities and retain rejected proposals as well
as successful ones. Complete tasks through the product commands and integration
path, not a separate terminal agent whose output is later pasted into Bullet.

At least one accepted change must visibly cross providers: for example a Rust
contract/backend change, its generated TypeScript/React consumer and an independent
adversarial review. Assign those roles explicitly, preserve dependencies and
handoffs, and have a human integrate the reviewed Candidate. The implementation
workers and reviewer must not share completion authority. Include a duplicate
submission, stale base, provider failure, quota fallback, explicit account change,
bounded repair/escalation and a restart in the retained campaign.

Limits remain one active invocation per account, two implementation workers,
two repairs, one escalation, eight provider invocations per task and a 60-minute
deadline. Pauses, nested delegation or account changes reset none of them. Publish
the actual model provenance and failures; do not claim comparative productivity
until the seven-day observation and matched manual baseline support it.

## Real 1080p capture as maintained source

1. Keep capture, rendering and verification implementation in the tracked main
   source distribution. Preserve the current `just demo-gif-record`,
   `just demo-gif-render` and `just demo-gif-check` entrypoints or migrate them
   explicitly with documentation and compatibility tests. The final aggregate
   must contain their complete source and fixtures; a private scratch helper is
   not delivery of this requirement.
2. Move owned process supervision, terminal recording, artifact manifests and
   validation from Python to Rust. Keep browser recording and web behavior in
   TypeScript with the actual Vite/React product. Thin shell launchers may invoke
   the admitted tools; they must not become a second policy or state authority.
   Prove Rust behavior against the retained recorder interruption, EOF, custody,
   output-bound and timing regressions before replacing the old implementation.
3. Provide an actual bright/high-contrast product theme for both TUI and web.
   Use sharp text and saturated, distinguishable status colors with textual
   labels; measure contrast and keyboard/focus behavior. Capture the selected
   real theme rather than brightening or recoloring frames afterward.
4. Capture native 1920×1080 frames. Fix and record browser viewport, device pixel
   ratio, browser/fonts and terminal grid/font/raster geometry. No upscaling,
   dim overlays, interpolated motion or substituted screenshots. Check decoded
   geometry and pixels, including text edges and bright status colors.
5. Start recorders before the real product command and retain actual timestamps
   continuously through task admission, provider/account/model selection, parallel
   work, proposal, independent review, tests, human integration and final read-back.
   Include genuine failure/reconciliation behavior where it occurs. An explanation
   prompt or five staged screenshots cannot satisfy the coding demonstration.
6. Bind both views to the same task/run/Attempt/Candidate/review/merge identities
   and ledger watermarks. The browser must use actual production API/SSE and
   durable state; the TUI must show the real CLI/session. A task on Bullet's own
   source is required for the self-dogfood demonstration. Include real Codex,
   Claude, Cursor and Antigravity activity after independent account/runtime
   qualification. The initial twelve Codex/Claude/Cursor tasks remain the earlier
   milestone; qualify and retain the corresponding Antigravity implementation
   tasks before claiming the requested four-provider demonstration complete.
7. Preserve original PNGs and terminal stream/transcript privately, with a
   pixel-exact FFV1 master and hashes. GIF has a 256-color palette and centisecond
   timing: accept a lossless GIF only when an independent decoder proves exact
   source-pixel equality and the admitted timing bounds. Otherwise fail strict
   lossless GIF acceptance and retain the exact master; never label a quantized
   derivative lossless. Any final-frame hold is an explicit display policy.
   Each 1920×1080 TUI or web GIF must be strictly below 50,000,000 bytes (decimal
   MB); equality fails. Check the actual completed file in both render and
   independent verification. Preserve oversized failures and their original
   masters; do not meet the bound by dimming, recoloring or dropping frames.
8. Test failure paths: absent or drifted tools, renderer error, dropped/colliding
   frames, wrong dimensions, timing flattening, truncation, disk exhaustion,
   interrupted capture, incomplete provider termination and mismatched task IDs.
   Refuse export on missing evidence. Retain failures instead of overwriting a
   successful-looking output file.
9. Separate private capture from reviewed public export. Credentials, private
   account homes and raw secrets never become committed assets. Select a bounded
   public task whose real output can be exported; preserve any redaction history
   and original private evidence. Exported media, transcript and reproduction
   manifest must identify their exact public source subjects.
10. Verify local reproduction from the accepted aggregate with admitted tools:
    rebuild the capture/render/check code, reproduce the encoded output from
    retained inputs and independently check it. A new provider execution has its
    own task/run identity; deterministic rendering does not imply deterministic
    model output or repeat billing authorization.

## Exit checkpoints and final read-back

The earliest **proper internal development** milestone requires clean local health,
working exact-source CI and protected GitHub delivery, admitted operational custody,
the real durable coding transaction, three separately qualified subscriptions,
reviewable collaborative work, interruption/restart recovery and retained evidence.
Useful dogfood acceptance additionally requires twelve accepted tasks and seven-day
survival. Live media may document an admitted internal campaign with that narrower
scope; it must not claim `self-hosted-v1` or universal certification early.

Full completion still requires the signed twelve-boundary campaign, production
custody separation and historical recovery, Ubuntu packaging and two clean installs,
all lifecycle/backup/upgrade/rollback proofs, durable cognition/routing/evolution,
Jeryu self-hosting, Antigravity, remaining forge adapters, five platform slices,
then `team-v1` and `saga-v1`, plus active post-V1 obligations and retired dispositions.
All 18 product profiles must pass independently under the existing typed inventory.
Do not remove a required profile to make the release page green.

For every accepted packet, reconcile this plan, the health checkpoint and G1–G18
register with the typed inventory; unchanged blocked statuses may retain identical
canonical JSON bytes. Record exact source commits/trees, review/PR/merge subjects,
workflow digests, run/attempt/matrix identities, selected/completed tests, artifact
and package digests, provider/account/runtime generations, task receipts and profile
receipts. Final read-back must come from actual local/remote consumers, not this
document or a chat claim. Forecast dates only after two measured implementation
cycles, separating infrastructure waits from the mandatory observation window.
