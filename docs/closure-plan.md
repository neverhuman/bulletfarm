# BulletFarm complete gap-closure plan

Status date: 2026-09-18. This is the execution companion to the canonical
[BulletFarm 3.0 specification](spec/BULLETFARM_FINAL_ENGINEERING_SPEC.md). It covers
repository consolidation, command identity, every canonical BF3 package, every acceptance
criterion and every runtime scenario. The more narrative [implementation plan](implementation-plan.md)
remains useful for product journey and engineering rationale; this document is the exhaustive
queue and exit criteria. The [acceptance-status ledger](acceptance-status.md) records the
done/partial/missing state and current-main code evidence for each of the 90 criteria.

The current human instruction is to close all gaps between `neverhuman/bulletfarm` and the
supplied `tips/*.md`, close every remaining feature/issue, merge the current PR, and leave no
open PRs. This selects the full canonical browser/controller roadmap. The existing TUI remains
a supported client of the same API. Operating HOLD remains effective: documentation, fakes,
local implementation and secret-free qualification may proceed; live provider calls still
require the existing human-controlled authority, enrollment, grants and HOLD release.

## 1. Queue rule and definition of done

There is one product repository and one implementation queue. PR #1 was independently reviewed,
rebase-merged and deleted; its result is
`neverhuman/bulletfarm@1ea7aa60f536e3f3fe8ca186acd64b8433acc294`. Immediately after that
merge, GitHub had zero open issues and zero open PRs across `bulletfarm` and all five retiring
repositories. The documentation PR carrying this plan must be the only open product PR until it
is reviewed and merged, after which the queue returns to zero before command work begins.

From now on:

1. Never stack implementation PRs. Open at most one PR for the active, dependency-ready
   slice. Merge or explicitly close it before branching for the next slice.
2. Every PR names its BF3 acceptance/scenario scope, starts from current `main`, passes
   `scripts/check`, hosted `check`, and different-vendor exact-head process review.
3. A requested-changes review must identify a violation of current human instructions,
   canonical acceptance, or code evidence. Earlier product direction cannot override a later
   explicit human instruction. Preserve all reviews as process evidence.
4. After every merge, verify `main`, delete the merged branch, and prove the GitHub issue and
   PR queues are empty before the next slice. A planned future package is a row in this file,
   not a lingering placeholder issue or PR.
5. A package closes only when all three `BF3-...-AC` criteria and linked AT/HF/CF scenarios
   execute against the current candidate. Historical PASS, fixture-only behavior, source
   comments, local author claims and untrusted stdout are never sufficient.
6. `BUILD_CHECKPOINT.json` is updated on every slice with exact source/spec IDs, implemented
   behavior, checks, limitations, live qualification and next dependency-ready work. Partial
   behavior never marks the whole package complete.
7. Final completion means BF3-001–030 are either implemented and proven or, where the
   specification calls for an evaluation, the evaluation produced its explicit retain,
   replace, reject or no-need decision with evidence. There are then zero open issues and PRs.

## 2. Input reconciliation and current gaps

| Gap | Observed state | Required closure | Gate |
| --- | --- | --- | --- |
| GAP-001 — incomplete tips mirror | Merged PR #1 carried only the canonical 3.0 specification. | Preserve all three supplied Markdown files byte-for-byte with hashes and identify canonical versus provenance. | Closure-plan PR |
| GAP-002 — referenced package files absent | `START_HERE.md` names seven schemas, nine examples, a backlog, requirements file, validator and report, but `tips/` contains only three Markdown files. | Record absence now. Recreate normative v3 schemas/examples and a repository-owned validator from the canonical spec under BF3-002; never claim the unavailable original validator ran. | BF3-002 |
| GAP-003 — active source is a reduced shell | Current code provides discovery, coordination board, TUI, PR listing, process stop, a bounded SQLite worker, bootstrap/session skeleton and embedded React assets. It lacks the delivery controller removed after `9f03362`. | Selectively restore conforming pieces with new tests and migrations. Do not reset `main` or import historical completion claims. | BF3-001–013 |
| GAP-004 — browser calls missing routes | React requests projects, drafts, work and events; current server exposes root/bootstrap/session/doctor/commands/operation only. | Add owner-scoped typed reads, paginated projections and bounded event recovery before calling the workbench functional. | BF3-017–018 |
| GAP-005 — commands are receipts, not mutations | `/v3/commands` accepts only `note`, `release`, `stop` and stores a `NOT_IMPLEMENTED` result. CLI/TUI board mutations bypass the hub. | Generate typed command/request responses, atomically authorize/mutate/audit/create operations, and move all clients onto one hub authority after preserving migration. | BF3-002–004, BF3-009, BF3-018 |
| GAP-006 — two writable databases | Hub uses `hub.sqlite`; coordination uses `bf.sqlite` directly. Current startup only distinguishes “migration table exists.” | Fingerprint every actual schema variant, consistently back up, migrate/import into one bounded worker, revoke obsolete sessions, pause execution, refuse unknown schemas without deletion, then fence the old writer. | BF3-003–004, BF3-009, BF3-021 |
| GAP-007 — no budget, graph or scheduler authority | No complete grants, holds, graph activation, pending-change ownership, allocator or durable all-job recovery exists on current main. | Implement one transactional allocator and separate write permission, physical occupancy, selection, reservations, verification and publication. | BF3-005–009 |
| GAP-008 — no qualified executor | `bf run` is a stub; no current isolation/conformance receipt exists. | Qualify one pinned Codex subscription transport in the specified Linux OCI boundary without secrets first; reject before spawn if containment fails. HOLD still gates live calls. | BF3-008–010 |
| GAP-009 — no trusted candidate verification | Historical fixtures and local tests do not bind a current immutable Candidate to protected evidence. | Seal candidate bytes/modes/base, run verifier-owned checks, reject missing/wrong/stale/forged evidence, retain completion reserve and job identity. | BF3-011 |
| GAP-010 — no recoverable real GitHub publication | Current PR display is observation; there is no credential-separated durable publish outbox/reconciliation. | Persist exact intent before dispatch, import candidate safely, reconcile lost responses and human edits, maintain one stable draft PR identity. | BF3-012 |
| GAP-011 — no browser-to-PR release proof | No installed browser-driven real task has survived interruption and produced an independently checked draft PR. | Complete the first useful task and recorded recovery demonstration before claiming G1/release. | BF3-013 |
| GAP-012 — team and planning surface incomplete | Two owners/providers, takeover/return, fair lanes, Foreman planning, shared API, integration, accounting and restore are incomplete. | Close BF3-014–022 in dependency order and run the independent rebuild gate. | G2 |
| GAP-013 — measured extensions absent | Learning, Grok Build, production pipeline, thin clients, OpenJarvis trial, architecture decision and forge improvement are unimplemented/evaluations outstanding. | Close BF3-023–030 with measured decisions; optional experiments close through evidence-backed rejection/no-need as allowed by their acceptance. | G3 |
| GAP-014 — repository/command identity split | Package/binary is `bf`; installed `bulletfarm` points to the legacy `bullet`; parent instructions and old repository descriptions still identify retired paths. | Complete the command-identity PR, install one executable with `bf` alias, preserve `~/.bf`/`BF_DATA_DIR`, map only five retired identities in PR discovery, update active entry points and retire the legacy launcher. | Consolidation |
| GAP-015 — five redundant GitHub repositories | `bf` and four `bullet-*` repositories remain public and unarchived; descriptions point to `neverhuman/bf`. | After all preservation/cutover gates, update descriptions to `bulletfarm`, archive, recheck, obtain `delete_repo`, delete exactly those five, and verify only `neverhuman/bulletfarm` remains. | Consolidation |

### Provenance backlog reconciliation

The larger `(1)` document does not add a second queue. Its 24 core packages and seven
experimental packages map into the canonical queue as follows; a row closes only through
the canonical acceptance, including any stricter later requirement.

| Provenance package | Canonical destination |
| --- | --- |
| old BF3-001 contracts/fixtures | BF3-001–002 |
| old BF3-002 durable store | BF3-003 |
| old BF3-003 identity/API/CLI | BF3-004 |
| old BF3-004 money/allowance/capacity | BF3-005 |
| old BF3-005 Git intent/activation | BF3-006 |
| old BF3-006 runner/fake execution | BF3-007–008 |
| old BF3-007 claims/scheduler | BF3-009 |
| old BF3-008 candidate handoff | BF3-009, BF3-011 |
| old BF3-009 gate adequacy/verification | BF3-001, BF3-011 |
| old BF3-010 forge reconciliation | BF3-012 |
| old BF3-011 executor qualification | BF3-010 |
| old BF3-012 first useful transaction | BF3-013 |
| old BF3-013 ordered steps/repair pool | BF3-009, BF3-013, BF3-017 |
| old BF3-014 second provider | BF3-016 |
| old BF3-015 planning/lead | BF3-017 |
| old BF3-016 independent review | BF3-011–013, BF3-019 |
| old BF3-017 takeover/withdrawal | BF3-015 |
| old BF3-018 fairness/external PRs | BF3-014 |
| old BF3-019 web/events | BF3-018 |
| old BF3-020 integration/mission outcome | BF3-019 |
| old BF3-021 recovery/retention | BF3-021 |
| old BF3-022 receipts/accounting | BF3-020–021 |
| old BF3-023 privacy/diagnostics/performance | BF3-010, BF3-018, BF3-020, BF3-022 |
| old BF3-024 team release/rebuild | BF3-022 |
| BF3-X01 Prime trial | BF3-010 evaluation input |
| BF3-X02 OpenJarvis trial | BF3-028 |
| BF3-X03 Grok Build | BF3-024 |
| BF3-X04 learning/warm state | BF3-023 |
| BF3-X05 Slack/Bot/SMS | BF3-026–027 |
| BF3-X06 production pipeline | BF3-025 |
| BF3-X07 forge improvement/replacement | BF3-029–030 |

The 80 `Q3` scenarios in the provenance document are preserved as prior formulations.
Current proof uses the canonical 52 AT, 20 HF and eight CF scenarios because those include
later closure refinements such as current prerequisite validity, completion reserve, every-job
recovery identity, non-leaking human authority and paused legacy import.

## 3. Merge and consolidation sequence

### A. Completed repository-establishment PR

PR #1 preserved the existing `bulletfarm` history, imported the current application without
runtime drift, carried forward the browser/controller roadmap, published the historical archive
index and verified the independent private backup. It passed local, fresh-checkout and hosted
checks plus different-vendor exact-head process review. The reviewer rebase-merged it as
`1ea7aa60f536e3f3fe8ca186acd64b8433acc294` and deleted the branch. This completed
repository establishment only; it did not certify a BF3 runtime package.

### B. Merge this closure-plan PR

1. Add this closure plan, the per-criterion evidence ledger, complete byte-identical
   `tips/*.md` mirror, hash/authority index and truthful `BUILD_CHECKPOINT.json`. Update active
   instructions and documentation links. Runtime source stays unchanged.
2. Verify the tip copies with `cmp` and SHA-256. Verify the checkpoint contains exactly
   BF3-001–030, 90 unique AC IDs, AT-001–052, HF01–HF20 and CF01–CF08 exactly once in its
   coverage inventory. Verify each AC has one explicit status in the ledger and that no supplied
   artifact is represented as available when absent.
3. Run `scripts/check` with pinned Node/npm, source-import blob/mode verification, archive
   ref/evidence verification, `git diff --check` excluding only byte-preserved Markdown
   hard-break whitespace, and a fresh-checkout build.
4. Push one final head. Obtain a different-vendor `REVIEW: approve <exact-sha>` against the
   current instruction and hosted `check`. If review finds a concrete defect, fix it, rerun
   affected proof and request review of the new head.
5. The reviewer rebases/merges and deletes the branch. Fetch `origin/main`, verify the merge,
   then prove all six PR and issue queues are empty before the first runtime slice.

### C. One command-and-cutover PR

1. Branch from merged `main`; claim only the root package, installer, PR discovery, UI branding,
   active docs and identity tests. Do not begin controller restoration in this PR.
2. Rename the Cargo package/default binary to `bulletfarm`; keep `[lib] name = "bf"` so Rust
   imports do not churn. Update diagnostics/help to use the actual invoked supported name while
   documenting `bulletfarm` first.
3. Add a repository-owned installer that copies the verified `bulletfarm` binary and creates
   `bf` as a relative symlink to that exact file. Replace `bulletfarm -> bullet`. Preserve the
   old `bullet` privately and replace its installed path with a small retirement launcher that
   exits nonzero and directs users to `bulletfarm`.
4. Keep the unchanged data resolution: `BF_DATA_DIR`, otherwise `~/.bf`. Do not rename, copy or
   initialize another database. Test both names against one temporary data directory and prove
   identical version, board, claims and notes.
5. Canonicalize only these identities in PR discovery: `neverhuman/bf`, `bullet-farm`,
   `bullet-kernel`, `bullet-git`, `bullet-portal` → `neverhuman/bulletfarm`. Deduplicate before
   calling GitHub. Preserve original strings in historical notes/URLs; unrelated repositories
   keep their identity. Test current board history returns one request and zero retired requests.
6. Add an active-identity CI scan with narrow exceptions for `archive/`, `docs/spec/tips/`, the
   migration index and compatibility tests. Update package metadata, UI brand, install docs,
   badges and repository settings.
7. Pass local/fresh/hosted checks and different-vendor exact-head review. Merge immediately,
   delete the branch, install from merged `main`, run the dual-name/data tests against the real
   preserved board, and prove the PR/issue queues return to zero.

### D. Cut over the host and retire repositories

1. Change `/home/ubuntu/bullet/{AGENTS.md,README.md,repos.manifest.toml}` and active launch
   entry points to the sole checkout `/home/ubuntu/bulletfarm`; keep the coordination digest
   symlink/history. Release old claims normally and acquire any new claim under the new path.
2. Make retired local checkouts read-only development references by removing push URLs and
   adding an untracked local historical marker. Never rewrite their source/history.
3. Re-export/recheck all old refs, PR/issue queues, logs and artifacts. Capture any delta. Run
   the fresh remote mirror/fsck, evidence checksum verification and private restore drill again.
4. Update the five old GitHub descriptions to the final repository, archive them, and rerun all
   gates while archived. Do not delete if any ref/evidence/install/data/queue check fails.
5. At the actual final deletion step, obtain operator-assisted GitHub authentication containing
   `delete_repo`. Delete exactly `bf`, `bullet-farm`, `bullet-kernel`, `bullet-git` and
   `bullet-portal`. Keep `bulletfarm` and private backups. Verify GitHub inventory, installed
   commands and `bf prs`: one product repository, zero retired requests, zero open issues/PRs.

## 4. Exhaustive canonical package closure matrix

No package is currently complete on the consolidated candidate. “Partial” names reusable
behavior only; every linked acceptance and scenario must still be executed against current code.
All packages have three criteria: `BF3-NNN-AC01`, `AC02`, and `AC03`.

### G0 — deterministic substrate

| Package and complete coverage | Current truth | Closure slice and required evidence |
| --- | --- | --- |
| BF3-001 · AC01–03 · AT-032 | Partial: source/archive audit exists; historical gate fixture is outside active tree. | Commit gate adequacy inventory, owners/deployment inputs, reusable-module map and known-good/injected-defect oracle. Missing authority stays unresolved and non-mutating. |
| BF3-002 · AC01–03 · AT-001, AT-002, HF01 | Partial: duplicate-key/UTF-8 parsing exists; canonical types/schemas and semantic validation do not. | Generate Rust/TypeScript request and response types plus strict schemas/examples; reject unsafe integers, unknown fields/versions, cycles, bad paths/profiles and digest ambiguity. Add repository validator replacing only the absent supplied tool. |
| BF3-003 · AC01–03 · AT-003, AT-004, HF16 | Partial: bounded SQLite worker and exact raw command replay exist; schema covers only skeleton commands/sessions. | Add versioned migrations, state/events/jobs/outbox/effects/candidate/evidence relations and atomic rollback/idempotency. Keep all I/O outside transactions. Test disk failure and immutable evidence. |
| BF3-004 · AC01–03 · AT-005, AT-006, HF17, CF06 | Partial: private bootstrap/session and actor-scoped operation lookup exist. | Implement principals/membership/capabilities, enrollment/revocation and owner-scoped reads; remove body-owner trust and anonymous fallback; route CLI/TUI mutations through authenticated API after migration. |
| BF3-005 · AC01–03 · AT-007–009, CF05 | Missing. | Implement grants, hierarchical holds, invocation limits, mandatory completion reserve, unknown/overrun handling and concurrent conservation tests. |
| BF3-006 · AC01–03 · AT-010, AT-011 | Missing. | Publish exact-byte immutable plan/task/profile contracts, expected-old metadata refs, atomic activation and current dependency validity including known reverts. |
| BF3-007 · AC01–03 · AT-012 | Missing on active tree; historical deterministic fakes are evidence only. | Implement one bounded job envelope and deterministic fake provider/forge, explicit admission/end/output/cleanup events, malformed/truncated framing negatives and no-network fixture proof. |

G0 exit: every contract and fake path works without provider credentials; database, authority,
budget, Git activation and adapter seams pass all linked negatives. `BUILD_CHECKPOINT.json` may
then mark BF3-001–007 complete, never sooner.

### G1 — one useful verified delivery loop

| Package and complete coverage | Current truth | Closure slice and required evidence |
| --- | --- | --- |
| BF3-008 · AC01–03 · AT-013, AT-014, HF06, HF09, CF01 | Missing. Process-group stop is useful but not a certified workspace boundary. | Qualify private Git/filesystem/process/network/resource isolation, exact incarnation cleanup, survivor detection and honest occupancy under failed stop. |
| BF3-009 · AC01–03 · AT-016–020, AT-022, HF07, HF08, CF03 | Partial: path claims exist in a separate coordination DB; no unified scheduler/all-job recovery. | Migrate claims into one authority; atomically acquire repository/path/account/capacity; persist launch intent/generation/deadlines; recover planners, writers, reviewers and verifiers; reject stale/duplicate results. |
| BF3-010 · AC01–03 · AT-026, HF02–HF05 | Missing and live-gated. | Pin one Codex CLI subscription transport/profile, prove fresh identity, typed failures, cancellation/accounting/credential boundaries and clean-room containment. Secret-free qualification precedes credentials; HOLD precedes live calls. |
| BF3-011 · AC01–03 · AT-021, AT-030, AT-031, AT-033, AT-034, AT-044, HF11, CF02 | Missing on active tree. | Seal exact candidate/base/scope/modes/artifacts; run independent verifier jobs; validate discovery/assertion count/producer/generation; distinguish prepublication, candidate, CI and integration stages; reject forgery/staleness/missing bytes. |
| BF3-012 · AC01–03 · AT-015, AT-023, AT-024, AT-035–037 | Missing. Current GitHub support is read-only observation. | Add credential-separated safe import, durable effect intent, stable draft identity and authoritative reconciliation for lost response, ambiguity, revocation and human edits/closure. |
| BF3-013 · AC01–03 | Missing. Historical fixture demo cannot close a real release gate. | Run no-key fixture demo, then one authorized installed browser-driven Codex task yielding an independently checked draft PR; interrupt job/restart hub/lose publish response and record exact versions, costs, limits and intervention. |

G1 exit: a person uses installed `bulletfarm` to take one real approved goal through fresh bounded
execution, independent verification and recoverable draft publication. The author can end while
checking/delivery continue; no logical publication duplicates. Zero open PRs means the dogfood PR
itself is reviewed and merged or explicitly closed after its evidence is captured.

### G2 — individual-and-team MVP

| Package and complete coverage | Current truth | Closure slice and required evidence |
| --- | --- | --- |
| BF3-014 · AC01–03 · AT-040 | Partial observation only: discovery and PR listing exist. | Add two owner lanes, applicable grants, overlap reasons, fairness/backpressure and external PR reconciliation without claiming unregistered local work. |
| BF3-015 · AC01–03 · AT-038, AT-039 | Partial human-only stop; no durable takeover/return/respecification. | Implement takeover from last durable checkpoint, return, scope/version rebinding, cancellation fencing and honest pending-change disposition. |
| BF3-016 · AC01–03 · AT-027 | Missing. | Qualify a genuinely second coding-provider family through the same job/isolation/accounting contract; prove cross-provider handoff and startup trust. |
| BF3-017 · AC01–03 · AT-025, HF12 | Missing. React draft UI calls unimplemented routes. | Persist owner-bound conversations/revisioned drafts; implement Fast/Standard/Deep bounded planning, plan acceptance CAS, scoped fresh task packets and source-restricted memory. Goal entry never authorizes work. |
| BF3-018 · AC01–03 · HF19 | Partial assets/UI only; projects/drafts/work/events return 404. | Ship one responsive accessible workbench and shared typed API with paginated projections, bounded events/reconnect snapshot, slow-client handling and zero-model monitoring. Move TUI/CLI onto the same authority. |
| BF3-019 · AC01–03 · AT-041, AT-042, CF04 | Missing. | Protect native integration/current-base checks, assembled outcomes and dependency invalidation after known revert; distinguish product publication from merge. |
| BF3-020 · AC01–03 · AT-028, AT-029, AT-043, AT-047, HF10, CF08 | Missing. | Record complete disjoint/inclusive usage without double charge, attention windows, costs/delay/human minutes and calibrated route evidence; expose unknowns. |
| BF3-021 · AC01–03 · AT-045, AT-046, CF07, HF20 | Partial migration archive evidence only; runtime restore/export is absent. | Rehearse consistent backup, disaster restore, retention/cleanup, portable export/import, legacy digest mapping, paused authority and fresh admission. |
| BF3-022 · AC01–03 · AT-052 | Missing. | Run two-owner/two-runner fault campaign, frozen performance workload, release gates and independent fresh-agent rebuild. Record missing assumptions; no prior conversation may fill gaps. |

G2 exit: two people, two qualified runners and two coding-provider families use the browser/shared
API with fair ownership, takeover, planning, integration, complete accounting and proven restore.
BF3-001–022 and every scenario linked to those packages are current-green. Scenarios assigned to
the measured G3 extensions remain open until the corresponding evaluation closes.

### G3 — measured extensions and explicit architecture decisions

| Package and complete coverage | Current truth | Closure slice and required evidence |
| --- | --- | --- |
| BF3-023 · AC01–03 · AT-048, AT-049, HF13, HF14, HF18 | Missing. | Implement evidence-only shadow proposals, held-out fixed evaluation, human promotion/canary/rollback and separation of online adaptation cohorts. Profiles never authorize themselves. |
| BF3-024 · AC01–03 | Missing. | Qualify Grok Build as a distinct executor only after identity, framing, isolation, cancellation and accounting pass; otherwise record a rejected adapter decision. |
| BF3-025 · AC01–03 · AT-050 | Missing. | Connect one named existing production pipeline with scoped authority, interruption recovery, release receipts and rollback; no generic unbound integration claim. |
| BF3-026 · AC01–03 · AT-051 | Missing. | Add signed, replay-safe, rate-limited Slack thin-client commands over the same API; no channel message can manufacture human-only authority. |
| BF3-027 · AC01–03 | Missing. | Identify and separately validate Grok Bot and SMS capabilities/identities; implement only conforming thin clients or record evidence-backed rejection. |
| BF3-028 · AC01–03 · HF15 | Missing. | Benchmark the OpenJarvis engine/core subset behind the existing boundary; prove timeout/continued-server accounting and adopt, reject or replace explicitly. |
| BF3-029 · AC01–03 | Missing. | Use G2/G3 performance/fault evidence to test whether the modular monolith needs replacement/federation; retain it when no measured need exists and record that as the successful decision. |
| BF3-030 · AC01–03 | Missing. | Select exactly one measured forge-native improvement justified by BF3-019/022 evidence, implement it behind current authority/recovery contracts, or record no justified change. |

G3 exit: every extension has an evidence-backed implemented/rejected/no-need outcome, protected by
the same authority, accounting and recovery contracts. No experiment remains an unbounded backlog item.

## 5. Cross-cutting release gates

Each package-specific proof is necessary but the following end-to-end gates also remain:

- **Contracts/security:** strict UTF-8/JSON and safe integers; current authorization on replay;
  private bootstrap; Host/Origin controls; no credential in candidate, logs, archive or runner.
- **Storage/recovery:** one SQLite authority and bounded worker; known-schema migration with backup;
  atomic failure rollback; queue saturation; restart/lease/generation/effect reconciliation.
- **Execution:** qualified OCI profile; immutable admitted inputs; default three cumulative writing
  invocations per task revision; honest unknown effect/usage; complete descendant termination proof.
- **Verification/publication:** immutable Candidate and verifier-owned evidence; no forged output;
  one stable PR; lost-response reconciliation; human remote edits/closure respected.
- **UX/accessibility:** no IDs/raw JSON/provider terminal in the normal journey; keyboard and
  Unicode/multiline editing; visible focus/announcements/narrow layouts; reconnect and stale-edit
  recovery; monitoring performs zero model calls.
- **Performance:** release build with 10,000 historical tasks, four simulated runners, 20 status
  reads/s, 10 durable small mutations/s and slow/event-flood variants for at least 60 steady-state
  seconds. Status/Why p95 <100 ms, durable command p95 <250 ms, editor p95 <100 ms, idle RSS
  <150 MiB, with distributions, errors, workload identity and hardware recorded.
- **Release proof:** installed browser-driven real task, interruption/recovery, independently checked
  draft PR and current hosted checks. Fixture success, provider exit, dashboard/TUI and historical
  evidence alone cannot pass.

## 6. Final zero-open-work audit

After BF3-030 closes, perform one final read-only audit before declaring completion:

1. `git status`, `git fsck`, fresh clone and full `scripts/check` at exact `main`.
2. `BUILD_CHECKPOINT.json` names the current commit/spec hash, all 30 packages complete/decided,
   all 90 acceptance IDs and all 80 scenario IDs with current evidence; no “partial”, “deferred”,
   “planned”, “unknown” or unowned blocker remains.
3. GitHub shows only `neverhuman/bulletfarm` for the product, zero open issues, zero open PRs,
   no unmerged product branch, required `check` green and protections/rulesets active.
4. Installed `bulletfarm` and `bf` are the same executable/data authority; `bullet` retires with
   guidance; PR discovery makes one canonical request and historical URLs remain readable only in
   preserved evidence.
5. Private backups and immutable archive tags restore successfully; the public migration index maps
   every old repository/ref/PR to preserved evidence. No credential or live authority is published.
6. Operating HOLD status and any remaining operator-only authority are reported exactly. A HOLD may
   prevent live execution but cannot be relabeled as completed qualification.

Only this audit closes “all remaining features/issues.” Until then the checkpoint and UI must state
the precise incomplete package or external authority, without inventing completion.
