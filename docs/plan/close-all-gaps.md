# Close all gaps: the dependency-ordered plan from `main` to the canonical 3.0 specification

Written 2026-09-18 by claude-orch. Companion documents: [`docs/closure-plan.md`](../closure-plan.md) (gates and the
G0–G3 closure matrix, codex), [`docs/acceptance-status.md`](../acceptance-status.md) (per-criterion ledger, codex),
[`docs/gaps/register.md`](../gaps/register.md) (this plan's 366-row requirement register, independent of the ledger),
[`docs/implementation-plan.md`](../implementation-plan.md) (Stages 0–7), [`docs/migration/README.md`](../migration/README.md)
(PR B and retirement gates). Where this document and `closure-plan.md` differ, the stricter statement wins and the next
docs PR reconciles them.

## 1. Where main is

Basis: `neverhuman/bulletfarm` main `1ea7aa60` for code evidence; docs PRs #2 (closure plan, merged `7579d67e`) and this register left the runtime unchanged. Queue at the time of this revision: this PR is the only open PR; no unmerged product branches remain (`codex/consolidate-repository` and `codex/complete-gap-plan` were deleted on merge). Register counts over
366 obligations (every BF3 acceptance criterion, every AT/HF/CF scenario, and the prose "must" clauses of the
spec's Parts II–V): **missing 256, partial 67, done 8, deferred (G3) 34,
not applicable 1**. Of the 90 package criteria, 0 are done.

What exists and stays: passive discovery of Claude/Codex/Cursor/Grok sessions, the append-only coordination board
(`bf claim|heartbeat|release --proof|note --to`, generated `AGENT_CHAT.md`), the ratatui screen with Agents/Board/PRs
views and transcript tails, `bf prs` over `gh`, a loopback hub with bootstrap sessions and idempotent command
receipts, cached CI in about 85 s, strict branch protection, and the cross-vendor review protocol. The controller that
existed at `9f03362` (contracts, jobs, budgets, fixture adapters, verification, fake forge, 26-table schema) is in
history only.

Four defects on main gate G0 even for what exists and are fixed first (M0/M2):

1. `commands.command_id` is a global primary key and replay never filters by actor, so one principal can read
   another's operation and detect a body mismatch (`migrations/001_core.sql`, `src/commands.rs:34-57`).
2. The command row and the operation row are two autocommit inserts with no transaction and no event row; a crash
   between them strands the command id forever (`src/commands.rs:66-75`).
3. Bootstrap always mints the human `owner` session; `principals.kind` is never read, so no agent, runner, verifier
   or publisher principal and no human-only guard exist at the API (`src/api.rs`).
4. The shipped React page posts `run/edit_draft/start_work/pause/cancel` and fetches `/v3/projects|drafts|work|events`,
   none of which the hub serves; the board keeps a second writable SQLite (`bf.sqlite`) beside `hub.sqlite`.

## 2. Rules that hold for every milestone

- Zero open PRs is the steady state: a PR is opened only when its author can stay with it to green and a reviewer of a
  different vendor is available; anything older than one working day is merged, fixed or closed with proof.
- One integrator per PR, claim before branching (`bf claim`), disjoint file sets between open PRs, branch built from a
  fresh `origin/main`, local `bash scripts/check` under Node 22.23.2 / npm 10.9.8, hosted `check` green, reviewer
  posts `REVIEW: approve <head-sha>` and rebase-merges. No worktrees, no stacked PRs.
- A criterion moves to done only with a test on main that fails without the behaviour; fixture success is not live
  qualification; the Operating HOLD stays until the human enrollment/grant ceremony.
- Every milestone ends with `docs/acceptance-status.md` and `docs/gaps/register.json` regenerated against the new
  main (register rows that became done cite the test).

## 3. Milestones in dependency order (critic-verified against spec §24.2 and Stages 0–7)

Size key: S ≤ 1 agent-day, M 1–3, L 3–7, XL > 7. The remaining-work roll-up per milestone is computed mechanically from
`docs/gaps/register.json` (rows not done or n/a, by size); the sizes themselves are reader estimates, not measurements. Owners are lanes, not people; any lane may take an unclaimed milestone after posting on the board.

| Milestone | Packages | Remaining rows (by size) | Suggested lane | Gate |
|---|---|---|---|---|
| M0 | queue hygiene, kernel defects 1–4, PR B command cutover | L 3, M 14, S 17 (shared with M7 audit rows) | codex-consolidation (PR B), claude (defects) | none |
| M1 | BF3-001, BF3-002 (+ BF3-007 fakes may start) | M 3, S 7; cross-cutting L 4, M 17, S 12 | claude | G0 |
| M2 | BF3-003 | M 1, S 4 | codex | G0 |
| M3 | BF3-004 (+ early BF3-017/018 browser slice) | L 1, M 5, S 1; operator rows L 2, M 30, S 12 | claude (API), codex (web) | G0 |
| M4 | BF3-005, BF3-006, BF3-007 | L 2, M 11, S 4; control rows L 6, M 20, S 1; execution rows L 2, M 19, S 4 | grok | G0 exit |
| M5 | BF3-008, BF3-009 | L 5, M 11, S 3, XL 1 | grok (needs sandbox decision) | G1 |
| M6 | BF3-010, BF3-011, BF3-012, BF3-013 | L 6, M 12, S 13; delivery rows L 2, M 15, S 6, XL 1 | codex (Codex profile), claude (verification), grok (forge) | G1 exit |
| M7 | BF3-014 … BF3-022 (closes the 22 core packages and the scenarios linked to them) | L 11, M 26, S 9, XL 2 | all lanes, serialized per §24.2 | G2 exit |
| M8 | BF3-023 … BF3-030 (AT-048..051, HF13/14/15/18 close here; all 80 scenarios only after M8) | L 22, M 2, S 8 | decision per package: implement, reject with reason, or no need | G3 |

The critic's ordering as recorded in the audit snapshot of 2026-09-18T20:35Z — **dated provenance, taken before bulletfarm#2 merged**: its M0 paragraph describes that moment (the closure-plan branch was then local-only and #2 did not yet exist). Current M0 state is in §1 and §4 below; the row assignments per milestone remain valid:

Inputs: §24.2 Depends-on column, implementation-plan Stages 0-7, closure-plan G0-G3 (present on codex/complete-gap-plan, not on 1ea7aa60). Source at HEAD c9dc7c60 is byte-identical to 1ea7aa60 under src/, tests/, migrations/, web/, scripts/; only docs and BUILD_CHECKPOINT.json differ. All Rust tests pass at this head (cargo test --locked --offline).

M0 — Stage 0 queue hygiene (no package). (a) The 'PR' does not exist: codex/complete-gap-plan is a local branch 3 commits ahead of origin/main=1ea7aa60 (f0048725, 0811bcef, c9dc7c60; docs + BUILD_CHECKPOINT only) and another agent committed to it during this audit. Push it over the neverhuman SSH key, open the PR, get a different-vendor `REVIEW: approve <head-sha>` comment, merge with --rebase --delete-branch. (b) Before merging, downgrade the seven refuted rows in BUILD_CHECKPOINT.json/closure-plan (§8.1-a, §AppendixA, §20.3-b, §18.4-headroom, §13-g, BF3-001-AC02 → missing; §7-b → partial). (c) Resolve origin/codex/consolidate-repository (unmerged product branch, no PR) by merge or documented deletion; closure §6.3 requires zero unmerged product branches. Exit: zero open PRs/issues/unmerged branches, clean base.

M1 — BF3-001 + BF3-002 (Stage 1; deps: none → BF3-001). Rows: BF3-001-AC01, AT-032 (restore fixtures/gates/dedup + at032_gate_oracle into scripts/check), BF3-001-AC02 (typed readiness record, not prose), BF3-001-AC03 (docs/onboarding reuse + assisted-only record), BF3-002-AC01/02/03, AT-001 and AT-002 (pure validator + cycle detection; wiring into activation is M4), HF01 (profile validator; admission is M4), §8.1-a (safe ints, Micros type, overflow-checks), §8.1-b (duplicate-key/UTF-8 regression tests), §8.2 mission shape, §8.3 task validator with check-ID resolution, §8.4 contract path rules (newline, no glob aliasing, rename/symlink/gitlink/mode/policy), §8.5 validate_profile, §AppendixA (seven schemas, one generator, CI diff of wire.ts). Also BF3-007 fakes can start here in parallel (dep: BF3-002 only).

M2 — BF3-003 (Stage 2; dep BF3-002). Rows: BF3-003-AC01 (single transaction: authorize → version → mutate → event → operation; port failed_storage_rolls_back), BF3-003-AC02 + AT-003 + AT-004 namespace (PRIMARY KEY(actor_id,command_id)), BF3-003-AC03 (schema-fingerprinting migration of the 4-table hub.sqlite and bf.sqlite; Appendix B tables + immutability triggers), HF16, §8.6 tables, §8.1-c events(seq) table, §13-a three identity columns, §20.2-a (typed envelope, deny_unknown_fields, target_id/expected_version → STALE_VERSION, 64 KiB bound), §20.2-c, §20.2-d (correlation ID, OUTCOME_UNKNOWN status, COMMAND_CONFLICT → RESOURCE_CONFLICT or documented), §7-b transaction fix.

M3 — BF3-004 (Stage 2; dep BF3-003). Rows: BF3-004-AC01 + HF17 + CF06 (principal kinds human/agent/runner/verifier/publisher, human-only gate in kernel, initiator vs actor), BF3-004-AC02 (replay recheck), BF3-004-AC03 + §13-f (bf init, one-time bootstrap exchanged for scoped identity, runner enroll/revoke, body-owner test), AT-005 + §13-b (repositories/memberships/roles), AT-006 + §20.3-a + §20.3-c (scoped reads: missions, tasks, why, work, events SSE with cursor/snapshot, artifacts), §13-c scope-matrix tests, §13-e (token out of URL fragment; record SSH-forwarding decision), §13-d grants/decisions tables (shared with M4), §7-a runner subcommand shape, §6.2 --local naming, §7-d (note/release/claim/heartbeat as command kinds, migrate bf.sqlite with provenance, CLI/TUI become API clients — quiesce other agents' claims first), §9-c importer + §13-g command-shaped-JSON negative test.

M4 — BF3-005 + BF3-006 + BF3-007 (Stage 3; closes G0). BF3-005 (deps 003,004): BF3-005-AC01/02/03, AT-007/008/009, CF05, §18.2, §18.3 ledger + metering precision, §18.4 lifetime invocations per original task (fix plan wording 'per revision'), transport retries, failure fingerprints, §13.1 revocation recheck at admission/dispatch. BF3-006 (deps 002,003,004): BF3-006-AC01/02/03, AT-010, AT-011, §9-a bf/control layout, §9-b revise mapping, §9.1 invalidations, wiring AT-001/AT-002/CHECK_MISSING/HF01 into activate_plan, §8.5 profile registry/admission, §6.1 capability table + goal-downgrade guard, command kinds create_mission/activate_plan/pause/stop/cancel/revise/resolve_decision/grant_allowance (§20.2-b). BF3-007: adapter seam, fakes, AT-012.

M5 — BF3-008 + BF3-009 (Stage 3/5; G1 start; 009 deps 003-008). Rows: §14.1 admission transaction, AT-016 one_current_writer, AT-017 multi-resource atomic acquisition bound to jobs, AT-018 launch_intents/incarnation, AT-019/AT-020/§14.2 lease (epoch compared, monotonic, holder-checked heartbeat), BF3-009-AC03/HF08/CF03 stale generation at result collection, §14.2 seal + idempotent replay + repair withdrawal, AT-022/BF3-009-AC02/§14.2 reservation lifetime replacing TTL expiry (reconcile plan §2 'expiry' wording), §14.3 + HF07 occupancy/cleanup/survivor search (needs BF3-008 sandbox), §16.2 all-job recovery, §18.4 interactive headroom + UI limitation statement, §12.2/12.3 state dimensions. BF3-008 requires the sandbox host blocker resolved first.

M6 — BF3-010, BF3-011, BF3-012, BF3-013 (Stages 5-7; closes G1). Order: 011 (deps 006,008,009) and 010 (deps 007,008,009; live-gated) in parallel → 012 (deps 004,009,011; request_merge kind, §16 effects) → 013 (deps 010,011,012; bf demo fixtures then the installed live task under HOLD release). Stage 7 dogfood PR is itself a G1 exit item and must be merged or closed with evidence.

M7 — G2, BF3-014..022 (plan §5 allows an early partial 017/018 browser slice after M3 for the first journey). Dependency order: 014 (deps 009,012,013) → 015 (011,012,014) and 016 (010,013) → 017 (006,013,014,016) → 018 (004,014,015,017), 019 (011,012,014,016), 020 (005,014,016) → 021 (003,009,011,012,015,020; carries CF07/HF20 importer) → 022 (015,017-021; fault campaign, performance workload, blind rebuild). Exit: all 22 core packages current-green. [Superseded on revision 3: M7 closes the scenarios linked to BF3-001..022; the eight G3 scenarios AT-048..051, HF13, HF14, HF15, HF18 close in M8, so "all 80" holds only after M8.]

M8 — G3, BF3-023..030: 023 (022) → 024 (016,022), 025 (019,022), 026 (018,022) → 027 (026); 028 (013,020); 029 (021,022,023); 030 (019,022). Each closes as implemented, rejected or no-need with evidence; then the closure-plan §6 zero-open-work audit.

## 4. Concrete PR sequence (ordered; not time-bounded — the G2/G3 scope is weeks of work)

Each line is one PR with a disjoint file set; the number is the order, not a GitHub id.

1. **PR B — `bulletfarm` command, `bf` alias, launcher cutover** (codex-consolidation; per migration README). Then reinstall on the host and update the container instructions.
2. **Kernel defect fixes** (claude): actor-scoped `commands` primary key + replay filter; one transaction for command → event → operation; `principals.kind` enforced with a human-only guard; correlation ids in error bodies; tests for each. Files: `migrations/`, `src/commands.rs`, `src/error.rs`, `src/api.rs`, `tests/core.rs`.
3. **Web page truth** (codex): remove or stub-route the unsupported kinds and fetches so the embedded page never 404s; typecheck asserts `wire.ts` matches the hub's kinds. Files: `web/src/*`, `src/api.rs` (read routes only).
4. **BF3-001 restore** (claude): `fixtures/gates/dedup`, `at032_gate_oracle` in `scripts/check`, a stored readiness record per repository, `docs/onboarding/`.
5. **BF3-002 contracts** (claude): `src/domain.rs` validators (task/mission/profile, cycles, check resolution, safe integers, path rules), seven schema files under `schemas/`, generated `wire.ts` diff-checked in CI.
6. **BF3-007 fakes** (grok): adapter seam `probe/start_fresh/read_events/cancel/collect_result`, `result-v3` payload, fake provider and fake forge with the malformed-frame negatives.
7. **BF3-003 store** (codex): restore the Appendix B schema behind a schema-recognizing, fingerprinted migration from today's four-table `hub.sqlite`; immutability triggers; events/outbox; rollback test.
8. **BF3-004 identity + CLI** (claude): `bf init`, one-time bootstrap exchange, runner enroll/revoke, roles and memberships, scoped reads (`missions`, `tasks`, `why`, `work`, `events` SSE with cursor), and the board verbs moved behind `POST /v3/commands` so `bf.sqlite` retires (quiesce other agents' claims first, announced on the board).
9. **BF3-005 budgets** (grok) and **BF3-006 Git intent/activation** (claude): completion reserve, holds, bounded retries; `refs/heads/bf/control`, expected-old-value updates, crash-injected activation.
10. **BF3-009 claims/leases/scheduler** (grok) on the restored jobs table; **BF3-008 isolation** only after the sandbox decision below.
11. **BF3-011 verification** (claude), **BF3-012 publication** (grok), **BF3-010 Codex profile** (codex, live-gated), then **BF3-013** demo fixtures and the first installed live task under the HOLD ceremony.
12. G2 packages in §24.2 order, one PR each; G3 decisions recorded one PR each.

## 5. Decisions only the operator can make (each blocks a specific milestone)

- **Sandbox host (blocks M5 BF3-008 and M6 BF3-010).** `apparmor_restrict_unprivileged_userns=1` makes bubblewrap fail;
  the only container runtime is a rootful Docker shared with a live database. Options: enable unprivileged user
  namespaces on xbabe2, dedicate a Linux runner host, or approve Docker with an explicit isolation profile.
- **Live authority (blocks BF3-010, BF3-013, AT-026/027, Stage 7).** The human enrollment/grant ceremony that lifts the
  Operating HOLD for one pinned Codex `exec --json` profile (installed 0.155.0, not the 0.154.0 the plan audited).
- **Second GitHub identity (blocks nothing, strengthens every merge).** `jeppsontaylor` already authenticates over SSH
  on this host; inviting it to the org and adding a `gh` token would let branch protection require one native review.
- **Repository retirement (M0/M7 audit).** `delete_repo` needs operator-assisted authentication; until then the five
  old repositories stay archived per the migration gates.

## 6. Host blockers verified on xbabe2 (audit snapshot 2026-09-18T20:35Z; the queue facts in it are superseded by §1)

- No PR exists to merge: codex/complete-gap-plan is local-only (git branch -r shows only origin/main, origin/archive/history, origin/codex/consolidate-repository); it is 3 docs-only commits ahead of origin/main 1ea7aa60 and moved from f0048725 to c9dc7c60 while this audit ran, so another agent is writing to the same checkout — claim via `bf claim` before touching it.
- origin/codex/consolidate-repository is an unmerged remote product branch with no open PR; closure-plan §6.3 requires none. gh pr list and gh issue list on neverhuman/bulletfarm are both empty.
- GitHub token scopes are gist, read:org, repo only: no `workflow` scope (cannot push .github/workflows changes with the token; must push over SSH with ~/.ssh/id_ed25519_github_neverhuman_publish as AGENTS.md:49 says) and no `delete_repo` scope (closure-plan step C.5 retirement of bf/bullet-farm/bullet-kernel/bullet-git/bullet-portal needs operator-assisted auth).
- One shared GitHub identity `neverhuman` for every agent and vendor: 'different-vendor review' can only be a comment, never a native GitHub approval; main protection requires only the strict `check` status context, so process review is unenforced by the forge.
- Sandbox qualification (BF3-008/010, Stage 5) is blocked on this host: /proc/sys/kernel/apparmor_restrict_unprivileged_userns=1; `bwrap --unshare-user --unshare-pid` fails 'setting up uid map: Permission denied' and `bwrap --unshare-all` fails 'loopback: Failed RTM_NEWADDR: Operation not permitted' (bubblewrap 0.9.0). Docker 29.2.1 daemon is reachable (user in docker group) but is rootful and shared with a live container (veox-itest-db postgres, 127.0.0.1:5434) that must not be disturbed; using it as the OCI runner needs an explicit decision and an AppArmor/privilege change or a different Linux host.
- scripts/check exits 64 on the host shell: node v26.1.0 / npm 11.13.0 are active while web/.nvmrc pins 22.23.2 and check pins npm 10.9.8. ~/.nvm has v22.23.2 installed; every local proof must run under `nvm use 22.23.2` with npm 10.9.8 installed into it.
- Operating HOLD (AGENTS.md): no live provider calls, grants or enrollments may be created by agents. BF3-010, BF3-013 live pilot, Stage 7 dogfood, AT-026/AT-027 and every G1 exit that needs a real Codex run wait on the human-controlled enrollment/grant ceremony.
- Codex CLI on host is 0.155.0 (plan §Stage 5 audited 0.154.0); the pinned qualified binary must name 0.155.0 or whatever is actually tested. Other CLIs present: claude 2.1.277, cursor-agent 2026.09.10, grok 1.0.34.
- Secondary gh host 127.0.0.1:8787 (account jeryu) has an invalid token; harmless, but per plan §4 Jeryu credentials confer no GitHub authority and must not be used as a fallback.
- Migrating bf.sqlite into the hub (§7-d, M3) touches ~/.bf shared by every agent on this host; it requires quiescing all other agents' claims/heartbeats first, and the generated AGENT_CHAT.md symlink cutover must be preserved (AGENTS.md:31).
- Verified non-blocker: libsqlite3-sys 0.38.2 bundles SQLite 3.53.2 with source id 2026-06-03 19:12:13 d6e03d8c…, matching src/storage.rs:80-82, and the full Rust suite passes offline at this head.

## 7. Obligations the first mapping missed (all now in the register)

- **BF3-007 (AC01-03, AT-012)** (M) — Adapter seam and deterministic fakes: probe/start_fresh/read_events/cancel/collect_result contract (§12.1), result-v3 payload (§12.1a), malformed/truncated frame negatives, distinct admission/end/output/cleanup events, no-network fixture proof. It is a hard dependency of BF3-008 and BF3-009, both of which the control-plane area lists.
- **BF3-008 (AC01-03, AT-013, AT-014, HF06, HF09, CF01)** (L) — Isolated workspaces and physical execution tracking: private Git/filesystem/process/network boundary, exact incarnation cleanup, survivor detection, honest occupancy under failed stop. HF07/HF08/CF01 in the received rows presuppose it.
- **BF3-010 (AC01-03, AT-026, HF02-HF05)** (L) — Certify one Codex CLI exec --json subscription profile in a qualified Linux OCI runner; fresh-session proof, cancellation/auth/quota/nested-usage fixtures, clean-room fallback. Live-gated by HOLD and the sandbox host blocker.
- **BF3-011 (AC01-03, AT-021, AT-030, AT-031, AT-033, AT-034, AT-044, HF11, CF02)** (L) — Seal candidates and run stage-specific independent checks (§15, §8.6 receipt rules): verifier identity/generation, discovery/assertion validation, prepublication vs CI-only distinction, forged-output and partial-artifact rejection.
- **BF3-012 (AC01-03, AT-015, AT-023, AT-024, AT-035-037)** (L) — Credential-separated safe import, durable effect intents (§16, §16.1), stable draft PR identity, lost-response/ambiguity/revocation/human-edit reconciliation, request_merge command kind.
- **BF3-013 (AC01-03)** (M) — First useful developer loop: bf demo --fixture basic and --fixture interrupted_publish (§24.1), then one installed browser-driven live task with interruption/restart/lost-publish recovery and recorded versions/costs/limits.
- **BF3-014 (AC01-03, AT-040) + §14.4** (M) — Owner lanes, fair rotation cursor, tunable limits (4 team writers / 2 per owner / 2 verifier slots / 1 integration path / 8 pending PRs per repo), overlap reasons, external PR reconciliation. §14.4 belongs to control-plane and may be in the truncated tail.
- **BF3-015 (AC01-03, AT-038, AT-039) + §17** (M) — Human takeover, return, submit-human and respecification: fence old writer, checkpoint disclosure, identical independent checks, allowance preserved across revisions, take/return command kinds.
- **BF3-016 (AC01-03, AT-027)** (L) — Certify a second coding-provider family through the same job/isolation/accounting contract; cross-provider checkpoint restart; no identity evasion on auth/quota failure.
- **BF3-017 (AC01-03, AT-025, HF12) + §10, §10.1, §10.2** (L) — Foreman planning as a mission job, Fast/Standard/Deep bounded modes, plan-acceptance CAS, fresh task packets, scoped memory with no automatic audience widening.
- **BF3-018 (AC01-03, HF19) + §20.4** (L) — Single React workbench on the shared typed API: Mine/Domain/Team filters, decision cards, paginated projections, bounded SSE with reconnect snapshot, slow-client bounds, zero-model monitoring, accessibility, TUI/CLI moved onto the same authority.
- **BF3-019 (AC01-03, AT-041, AT-042, CF04) + §15.3, §15.4** (M) — Native integration on the current combined subject, assembled mission outcomes, known-revert invalidation of prerequisites, publication distinct from merge.
- **BF3-020 (AC01-03, AT-028, AT-029, AT-043, AT-047, HF10, CF08) + §17.2, §18.1** (M) — Complete cost/window/route reporting: inclusive-root vs disjoint-leaf basis, late child usage attribution, attention windows and drain, advisory stall vs hard limits, human-time labelling.
- **BF3-021 (AC01-03, AT-045, AT-046, HF20, CF07) + §16.3, §16.4, §23.1, §23.2** (M) — Consistent backup, paused disaster restore with publisher quiescence and authority rotation, retention/cleanup by exact identity, portable export/import, legacy digest-scheme import (CF07 is referenced by BF3-002-AC03 but its importer is unrowed).
- **BF3-022 (AC01-03, AT-052) + §7.2 + plan §4 performance workload** (L) — Two-owner/two-runner fault campaign, frozen release-build performance workload (10k tasks, 4 runners, 20 reads/s, 10 mutations/s, p95 targets, RSS), independent fresh-agent rebuild from the kit alone.
- **BF3-023..BF3-030 (G3; AT-048, AT-049, AT-050, AT-051, HF13, HF14, HF15, HF18)** (L) — Deferred packages: shadow evaluation/profile promotion (§22), Grok Build executor, one production pipeline (§21), signed Slack thin client, Grok Bot/SMS validation, OpenJarvis boundary (§19.3), replacement/federation test, one forge-native improvement. Each needs an implemented / rejected / no-need decision with evidence.
- **§12.2 / §12.3** (M) — Three separate state dimensions (lifecycle, writing authority, occupancy) and the task state machine with orthogonal hold_reason (auth_required, dependency_invalid, budget_unavailable, review_required, resource_conflict, check_missing, outcome_unknown) and the separate publication attribute. Referenced by several control-plane rows but never listed as an obligation itself.
- **§16.2** (M) — All-job restart behavior: acquire instance lock, inspect retained authority and unsettled effects, reconnect runner incarnations, fence expired work, reconcile before conflicting writes; reused PID/pathname is not identity. Only AT-008's 'across restart' touches it.
- **§20.1 CLI contract** (L) — Required verbs bf init, serve --local, connect, runner enroll/start, submit, plan, run --goal, chat, status --mine/--team, why, take, submit-human, return, revise, decide, pause, stop <mission>, cancel, report --window, export. Rows mention only `bf init`, the runner role and `bf serve --local`; src/main.rs:28-115 implements none of the listed verbs except doctor/serve (and stop is a pid-killer, not the mission stop).
- **§24.1 scripts/check scope + bf demo fixtures** (S) — scripts/check must cover schema fixtures (it does not: scripts/check:11-18 runs only web typecheck/test, build-web --check, fmt, clippy, cargo test, build) and the product must provide bf demo --fixture basic / interrupted_publish with deterministic receipts and no network.
- **§7.1 minimum process separation** (M) — Hub never executes candidate code, builds, tests or hooks; tool-using planners/reviewers are runner jobs; candidate execution has no hub/forge/check/signing credentials; trusted reporter outside the candidate process. Nothing rows this as a testable obligation (the proof-command runner in src/board.rs runs arbitrary commands in the CLI process today).

## 8. Definition of done for this plan

All 90 criteria `D` in `docs/acceptance-status.md` with a citing test; all 80 scenarios green in CI or in the
sanitized live-qualification lane; every G3 package closed as implemented, rejected or no-need with evidence;
performance workload recorded against the release build; independent fresh-agent rebuild performed; zero open
PRs and zero unmerged product branches; only `neverhuman/bulletfarm` remains for this product.
