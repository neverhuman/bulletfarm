# Lean MVP and TUI delivery plan

Status: implementation plan, not a qualification receipt. The canonical design
is `tips/BULLETFARM_FINAL_ENGINEERING_SPEC.md` in the family tips corpus outside
this checkout; its BF3 identifiers and acceptance cases remain the source of
truth. The provenance document with `(1)` in its name is not another backlog.

This plan chooses the smallest product that can complete one real, independently
checked coding task. It keeps the durable controller and trust boundaries from
BF3 while making the operator experience one no-argument command. A feature is
added only when it closes a named acceptance case or removes a measured user
friction.

## Product promise

Running `bf` opens or reconnects to one local workbench. The operator sees the
selected project, the current goal, its plan, task progress, provider sessions,
checks, and the resulting draft PR in one screen. The same command can be
restarted after a crash without losing the conversation, task identity, grant,
evidence, or remote outcome.

The first release supports one owner, one active coding account, one selected
Codex noninteractive transport, one repository, and one bounded task graph. It
can reach a draft PR only after candidate sealing and protected checks pass. A
fixture can exercise every path without a provider key or network access. The
fixture is visibly labelled and never reports fabricated live success.

The normal journey is:

1. Select or discover a repository and type a goal.
2. Review objective, acceptance, scope, account/model, limits, and checks.
3. Approve exactly one immutable plan revision.
4. Watch jobs, provider observations, diffs, and checks in the TUI.
5. Pause, stop, retry, or take over with truthful state and durable recovery.
6. Open one stable draft PR; show **Checking** until all applicable checks and
   independent review pass, then show **Review ready**.

The React/Vite portal remains a first-class consumer of the same commands and
projections. It is embedded into the distributed binary when its browser
workflow is enabled, so normal use does not require a source checkout, Node,
manual ports, or a build directory. The TUI is the G1 control surface; the
browser workbench is the G2 surface and must not create a second authority.

## Scope and non-goals

The MVP contains one Rust authority, one bounded SQLite worker, one instance
lock, one event journal, one allocator, one provider adapter, one verifier
authority, one forge adapter, and one embedded/static UI bundle. It does not
contain a workflow DSL, general chat bus, vector database, Redis/Kafka,
active-active hubs, provider-specific schedulers, automatic account pooling,
API-key fallback, autonomous policy changes, or a new Git object store.

G2 retains two owners/runners/provider families, Claude qualification, fair
scheduling, takeover/return/respecification, scoped memory, planning modes,
protected integration, complete accounting, restore/export, and independent
rebuild. G3 retains Prime, Grok Build, controlled learning, local inference,
production delivery, Slack/Bot/SMS, and justified forge improvements. Those
items stay in the canonical backlog and cannot silently enter the MVP.

## Implementation slices

### Slice 0 — repository and release discipline

Start every product slice from the latest `origin/main` in the claimed primary
checkout. Use one short-lived branch and one PR per coherent slice; never create
a worktree. Keep `BUILD_CHECKPOINT.json` truthful by separating implemented
fragments, fixture evidence, unimplemented requirements, and live qualification.
Every PR must carry its BF3 IDs, acceptance cases, changed paths, local check
receipt, and the hosted CI run. Merge only the current head after all required
checks pass. A stale or typed refusal is evidence of a blocker, not a pass.

The first implementation PR should be BF3-001 and BF3-002 together only if the
actual source map proves they cannot be reviewed independently. Otherwise keep
the packages small enough for an independent reviewer to reproduce from a clean
checkout.

### Slice 1 — strict contracts and pure state transitions (BF3-001/002)

Define ordinary Rust types and generated TypeScript types from one versioned
contract source. Include owners, repositories, missions, conversation turns,
draft revisions, tasks, prerequisites, grants, profiles, jobs, candidates,
checks, artifacts, effects, remote objects, operations, events, and diagnostics.
Each identifier has an explicit owner, revision/generation, source digest, and
creation event. Do not infer authority from a display name or provider text.

Implement semantic validators for:

- exact enum values, byte limits, Unicode handling, timestamps, and finite
  numeric budgets;
- duplicate JSON-key rejection and exact raw HTTP request-byte preservation;
- owner, repository, scope, prerequisite, grant, deadline, and cumulative
  invocation relationships;
- immutable plan revisions and expected-version mutation checks;
- check recipes that resolve to protected runnable checks before a live launch;
- effect identities and unknown-outcome states.

Make the state machine explicit. Every command returns the committed operation
and new version, not an optimistic acknowledgement. `Takeover`, `Stop`,
`Pause`, `Resume`, `Retry`, and `Start` either perform their transition or return
a typed refusal with a durable reason. Historical replay rechecks current
authorization before exposing private data or accepting a write.

### Slice 2 — one local authority and durable storage (BF3-003/004/005)

Create one hub process with one startup lock and one bounded database command
worker. CLI and TUI actions call the hub command API; they never open a second
SQLite authority. Use append-only migrations, foreign keys, short transactions,
a bounded busy timeout, and cursor-paginated projections. Verify at startup that
the pinned SQLite runtime contains the documented WAL-reset fix before enabling
WAL; refuse startup with a diagnostic code when it does not.

Persist, at minimum:

- principal/session/revocation and project discovery records;
- mission conversation and draft plan revisions;
- task graph, grant, budget, reservation, job generation, and lease state;
- provider observations, process occupancy, cancellation, and restart
  reconciliation;
- candidate manifest, immutable diff/artifact roots, check attempts and trusted
  assertion evidence;
- publication intent, remote read-back, receipt, settlement, and unknown
  outcomes;
- an append-only event stream with monotonic sequence and retention metadata.

Bootstrap is private and authenticated. Bind sessions to an owner, enforce
origin/CSRF controls for browser requests, revoke sessions durably, and scope
every read by current authorization. Fixtures may seed data for demonstration,
but a fixture grant can never authorize live execution.

### Slice 3 — the no-argument TUI (BF3-004, initial G1 surface)

Use a pinned Rust terminal stack already accepted by the family. `bf` with no
arguments starts the local hub if needed, reconnects if it is already running,
discovers the current repository, and restores the selected project, goal, and
draft. It never asks the operator to copy an ID or port during the normal path.

The screen is deliberately small:

- **Projects**: recent repositories, discovery status, and account readiness;
- **Goal**: multiline Unicode editor with objective, non-goals, scope, and
  acceptance criteria;
- **Plan**: one review page containing objective, task graph, limits, checks,
  provider/model, and the exact revision to approve;
- **Work**: compact task list with state, generation, provider, elapsed time,
  check summary, and draft-PR state;
- **Detail**: events, diff summary, raw provider/HTTP diagnostics, why a task is
  waiting, and recovery actions.

Keyboard actions are stable and discoverable: arrows or `j/k` move, `Enter`
opens, `Tab` changes panes, `e` edits, `p` previews the plan, `a` approves,
`s` starts, `Space` pauses, `x` requests stop, `r` retries an eligible failure,
`t` takes over, `?` opens help, and `q` exits after preserving the session.
Actions are disabled with a reason when authority, scope, or state does not
permit them. Status announcements, visible focus, multiline editing, narrow
terminal layout, reconnect snapshots, heartbeats, and bounded event backpressure
are acceptance requirements.

The TUI does not run provider work, Git, checks, or forge calls in its render
loop. It subscribes to bounded projections and sends short durable commands.
The browser uses the same command names and projection shapes later.

### Slice 4 — real controller paths with deterministic fakes (BF3-006/007/008/009)

Put fake provider, runner, verifier, and forge adapters behind the production
interfaces. A fake task must exercise admission, reservation, job generation,
dependency readiness, deadlines, cancellation, evidence, publication intent,
read-back, and settlement exactly as a live task would.

Before any process starts, commit the task admission, complete reservations,
budgets, event records, and launch intent. Track writing authority separately
from physical process occupancy. Enforce dependency graphs, grants, deadlines,
cumulative invocation limits, and a protected verification reserve.

Persist crash points before and after dispatch, remote application, receipt
recording, and settlement. Repeating a fixture keeps prior source and artifacts
and proves idempotence or preserves an explicit unknown result. Stop remains
**Stopping** until termination is observed; restart reconciliation repairs
orphaned jobs without issuing an unbounded duplicate.

Required fixtures include `basic`, duplicate submission, concurrent clients,
storage failure, restart, interrupted dispatch, interrupted publication,
ambiguous spawn, ambiguous forge response, revoked session, stale revision,
missing test, zero assertion, early exit, stale evidence, wrong producer, and
unauthorized write. Each fixture records an exact receipt and exit status.

### Slice 5 — planning and one approved graph (BF3-013, BF3-017 groundwork)

Create a durable mission conversation and owner-bound draft. Planning is a
bounded mission job with the same accounting, deadline, cancellation, and
recovery rules as coding. Persist the proposed task graph, acceptance criteria,
scope, source base, account/model, limits, and checks.

`Start work` accepts one exact draft revision and activates its complete graph
once. A repeated request returns the existing operation. A changed objective,
scope, spending limit, destination, or permission creates a new draft revision
and requires a new decision. Status and why queries read projections and never
invoke a planner or model.

Every worker gets a fresh compact context containing the accepted requirements,
source base, decisions, checks, allowed paths, remaining limits, relevant failed
attempt, and task revision. The worker does not receive the entire conversation
by default, and progress updates cannot mutate the plan.

### Slice 6 — one qualified live executor (BF3-010/011)

After fake recovery is green, qualify one pinned Codex noninteractive transport
using the existing subscription and the disposable Linux runner profile. Do not
add parallel transports, account pooling, or API-key fallback for this gate.

The runner has private Git state, supervised child processes, bounded CPU/memory/
disk/time, tested filesystem and descriptor limits, isolated credentials, and an
explicit network policy. Candidate execution does not receive hub or publisher
credentials. Unsupported containment refuses before spawn and leaves a typed
diagnostic; it is never waived for the pilot.

Seal the candidate from immutable inputs. Verify complete diff, base, scope,
artifact completeness, and producer identity. Run protected checks as a separate
verifier against the sealed inputs. Acceptance requires required-test discovery,
test execution, assertion evidence, and completion evidence. Exit code, model
text, or candidate-controlled JSON alone can never manufacture a trusted pass.

The first live qualification is a disposable repository task with a deliberately
small change and a real required check. It must be interrupted, restarted, and
completed once before the route is called qualified.

### Slice 7 — recoverable draft PR (BF3-012/013)

Persist exact publication intent before dispatch. Bind it to task revision,
candidate root, selection, grant, checks, destination, and expected remote state.
Use one stable PR identity per task revision. Reconcile lost responses through
authoritative forge read-back, preserve unknown outcomes until resolved, and
respect human branch edits and closures.

If required CI needs a PR, publish a preliminary draft while the task remains
**Checking**. Only after all applicable automated checks and independent review
pass may the UI say **Review ready**. Human merge remains outside this MVP gate.

### Slice 8 — browser companion and shared projections (BF3-018 groundwork)

Once the TUI loop is reliable, expose the same typed command/projection API to
the Vite/React workbench. Embed the production build in the distributed binary
and serve it only through the authenticated local hub. Keep raw JSON, protocol
traces, and diagnostic codes in a Details drawer rather than the main flow.

The browser journey must cover first launch, returning user, project discovery,
plan editing, single acceptance, Unicode input, disconnect/reconnect, narrow
screens, keyboard navigation, and status announcements. It must remain usable
with no provider installed by showing the exact missing capability and never a
fabricated success.

## Command and state contract

The operator-facing command vocabulary is shared by the TUI, browser, and
future scripts:

| Command | Durable effect |
| --- | --- |
| `discover` | Refresh repository/account readiness projection. |
| `goal.edit` | Append a conversation turn and create an owner-bound draft revision. |
| `plan.review` | Read the exact proposed revision, scope, limits, and checks. |
| `plan.accept` | Commit one decision for one exact revision. |
| `work.start` | Activate the accepted graph once and enqueue ready jobs. |
| `work.pause` / `work.resume` | Change admission of future work with expected-version checks. |
| `work.stop` | Request cancellation and remain stopping until observed termination. |
| `work.retry` | Create a bounded new generation only for an eligible failure. |
| `work.takeover` | Transfer current writing authority after checking task state. |
| `status` / `why` / `events` / `diff` | Read projections; never call a model or provider. |
| `check` | Run or inspect protected checks against an immutable candidate. |
| `deliver` | Reconcile the durable publication intent and stable draft PR. |
| `doctor` / `demo` | Inspect prerequisites or run honest fake fixtures. |

The TUI presents these as key actions, so normal use remains parameter-free.
Explicit subcommands are retained for diagnostics, automation, and recovery.

## CI and verification gates

Every slice adds a deterministic negative case before the implementation. The
required local command is `scripts/check` (or the family equivalent) and must
cover Rust format/lint/tests, contract/schema fixtures, TypeScript typecheck,
portal build/smoke journeys, and security/source scans. Hosted CI repeats those
checks from a clean checkout with pinned tool and dependency hashes.

The gate matrix is:

1. **Contract** — schemas, examples, duplicate keys, raw bytes, graph cycles,
   scope, authority, and version semantics.
2. **Storage** — migrations, WAL runtime check, lock ownership, bounded worker,
   restart, cursor pagination, and durable operation lookup.
3. **Controller** — admission, reservation, dependencies, generations,
   cancellation, restart reconciliation, and fixture receipts.
4. **Security** — private bootstrap, anonymous refusal, revocation, CSRF/origin,
   cross-owner refusal, secret scans, and source custody.
5. **Verification** — missing tests, zero assertions, early exit, stale/wrong
   producer evidence, scope/base/artifact mismatch, and unauthorized writes.
6. **Delivery** — duplicate submission, ambiguous forge response, read-back,
   human closure/edit, stable PR identity, and Checking versus Review ready.
7. **UI** — first launch, returning user, plan editing, Unicode, keyboard/focus,
   reconnect, narrow screen, bounded SSE, slow client, and event flood.
8. **Performance** — frozen release build with 10,000 historical tasks, four
   simulated runners, 20 status reads/s, and 10 short mutations/s.

Record pass/fail distributions, workload identity, cold/warm startup, failures,
and memory. The release targets are status/why p95 under 100 ms, durable small
commands p95 under 250 ms, idle hub RSS under 150 MiB, editor interaction p95
under 100 ms, and zero model calls for status or routine monitoring.

The integrated G1 gate is one browser- or TUI-driven real task reaching an
independently checked draft PR after an interruption and recovery. Fixture-only
success, a green exit code without assertions, or an unqualified provider cannot
satisfy it.

## Ordered work packages and exit criteria

| Order | BF3 packages | Exit criterion |
| --- | --- | --- |
| 1 | 001–002 | Contracts, validators, and gate inventory reviewed; negative fixtures fail before code. |
| 2 | 003–005 | One locked hub, durable operation lookup, auth, migrations, budgets, and current-version checks pass. |
| 3 | 006–007 | Production command/job/effect paths run deterministic fakes with exact receipts. |
| 4 | 008–009 | Isolated jobs, cancellation, restart reconciliation, reservations, and generations survive fault matrix. |
| 5 | 013 (planning portion) | Goal-to-plan-to-single-acceptance graph is durable and repeat-safe. |
| 6 | 010–011 | One Codex route and independent verifier pass live qualification with interruption. |
| 7 | 012–013 | Stable draft PR is reconciled and G1 integrated gate passes. |
| 8 | 018 (browser portion) | Embedded React workbench consumes the same API and all required browser journeys pass. |

Do not begin a later row because a provider is available. Begin it only after
the preceding exit receipt is reviewed and `BUILD_CHECKPOINT.json` names the
remaining limitations. If a gate exposes a missing prerequisite, stop the
affected route, preserve the receipt, and narrow the scope rather than adding a
new service or bypass.

## Operating rules for the team

- One integrator owns the current branch; independent reviewers reproduce the
  acceptance cases from a clean checkout.
- `AGENT_CHAT.md` is a temporary coordination board, not product state. Record
  claims and releases there, archive it when it grows, and put durable decisions
  in code, migrations, receipts, or docs.
- Never write provider keys, enrollments, live grants, or operator-decision
  lines from an implementation agent.
- Never call a provider from a status, why, UI render, or routine monitoring
  path.
- Never convert `UNKNOWN`, `BLOCKED`, `STOPPING`, or source-custody refusal into
  success to improve a dashboard.
- Rebase new work on current main, keep PRs small, wait for current-head CI,
  and merge only after required checks are green.

This plan is complete when the first G1 draft PR is independently checked,
recoverable, and explainable from the TUI without opening a provider terminal.
Everything after that is measured expansion along the retained G2 and G3
backlog.
