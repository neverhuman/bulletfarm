# BULLETFARM — The Definitive Multi-Agent Coding Engine
### Final scoring of all proposals · Gas Town teardown · adjudicated red-team · the complete model · fair score

Working name **Bulletfarm** (the place that refines raw fuel — many agents — into the one thing that moves the war rig: verified, surviving code on `main`). Stack, strictly: **Rust** control plane / runners / verifier / broker; **TypeScript + Vite + React** cockpit.

**Frozen V1 (2026-08) supersedes C8.** This file is historical provenance. Release GA requires conformant Claude, Codex, Cursor, and Antigravity adapters. The C8 sentence "GA = kernel + any two certified providers" is not current release authority. See `bullet-farm/docs/assurance/product-gaps.md` C8.

---

## 0. What I verified before scoring (so nothing rests on folklore)

Across these rounds I fetched primary sources rather than trusting summaries:

- **Gas Town is real and strong** — `gastownhall/gastown`, 17.7k stars, 7,770 commits, MIT, Go. Not "screen sessions." It has a Bors-style *bisecting* merge queue (Refinery), a three-tier watchdog (Daemon→Deacon→Witness), OTEL telemetry, severity-routed escalation, a capacity-governor scheduler, Seance session-recovery, DoltHub federation (Wasteland), and an htmx dashboard. The honest bar is beating *that*.
- **The autopsy citations are real.** I read [Gas City #5511](https://github.com/gastownhall/gascity/issues/5511) in full: under load (~loadavg 100) `bd create --graph` is killed at the runner deadline **after Dolt already committed server-side**; the client sees a timeout, classifies it transient, and re-applies — yielding **3 complete live workflows from one `gc sling`** (2 batch + 1 sequential fallback) that spawn **three workers into one non-isolated checkout** editing the same files. The maintainer's own fix is idempotency-key adoption ("re-read before re-writing"). Earlier rounds confirmed #1181 (shared-worktree clobber, P1) and #742 (tmux-liveness → 687 respawns/24h). The shared thesis of every serious proposal — idempotent graph mint, private clones, OUTCOME_UNKNOWN-not-auto-retry, no-tmux-liveness — is therefore *evidence-backed*, not aesthetic.
- **The adversarial red-team's two best catches are real git/build facts** (verified below): `git clone --reference` without `--dissociate` is a genuine corruption vector under mirror GC, and `build.rs`/proc-macros execute arbitrary host code at *compile* time (before any test "runs"). Both were under-specified in every prior proposal, mine included.

---

## 1. Gas Town — the complete shortcomings register (for *this* objective)

Gas Town is an excellent orchestration laboratory. For the specific objective — **maximize accepted, surviving engineering value per human-minute and per dollar under adversarial concurrency** — here is everything that falls short, grouped by root cause. Each is design-level and durable (not bug-of-the-week), and most trace to one of five missing *bindings*: identity↔incarnation, command↔idempotent-effect, evidence↔exact-subject, config↔runtime-generation, work↔isolated-authority.

**Substrate / liveness**
1. **tmux is the liveness oracle.** Its own problems-view calls a dead agent a "Zombie: Dead tmux session"; tmux 3.0+ is *required* for `gt up` and every tmux-backed role. #742 shows the failure: a dead session left work claimed and drove 687 respawns in 24h. A fleet OS must not infer "alive" from a terminal multiplexer.
2. **Claim ≠ incarnation.** Work ownership can bind to a session/display name that a later process reuses; no monotonic fence makes a stale incarnation's writes impossible.

**Coordination store**
3. **Beads-on-Dolt-in-Git as the control store.** Versioned-SQL-inside-the-repo is fascinating and operationally heavy: install requires Dolt + ICU4C + `bd`; the sibling repo carries a Dolt compatibility floor and a writer-deadlock class that hangs backup sync under write load. It puts a young CGO database in the hot path of every lease transition.
4. **Non-idempotent graph mint (#5511).** The timeout-after-commit race mints duplicate workflows; nothing keys the launch write idempotently across *attempts*.

**Isolation**
5. **Git-worktree Hooks share one `.git`.** "Each hook is a git worktree" → shared index/refs/hooks; #1181 (P1) is the receipt: parallel agents clobbered uncommitted edits. Isolation is opt-in userland, and even worktrees don't give private `.git` state.
6. **No compile-time isolation story.** Agents run host-trusted; nothing addresses `build.rs`/proc-macro arbitrary code execution.

**Integration authority**
7. **The Refinery merges itself.** Gas Town *is* the merge authority via its own Bors-style queue. Nothing external and sovereign (GitHub required checks) sits above the LLM fleet, so a bad reviewer chain or a prompt-injected step can, in principle, influence what lands.
8. **Evidence isn't bound to an exact commit.** Verification gates exist but nothing invalidates a green result when the branch moves; a rebase can silently carry stale green.

**Review integrity**
9. **Roles are prompt/config, not type-level.** Mayor/Witness/Deacon/Refinery/polecat are real, but reviewer independence isn't a structural invariant — the same family can, in principle, shape acceptance of its own work.

**Identity, quota, credentials**
10. **Scheduler is a rate governor, not a capacity graph.** `scheduler.max_polecats` throttles to avoid 429s — sensible, but there's no named-identity model, no quota observation with source/confidence, no reservation/settlement, no login-challenge inbox, no owner-bound compliance. This is exactly the subsystem the user asked for and Gas Town most lacks.
11. **Credential drift.** A configured provider profile can silently fall back to a machine-wide account (sibling #4945 class); no effective-identity probe.

**Config / truth**
12. **Config can read active while runtime is stale** (sibling #5279/#5536 class): reload exposes new values while a component keeps old ones; a stale endpoint pins a circuit breaker.
13. **One malformed config unit can erase valid state** (sibling #5529): no per-unit quarantine with last-known-good retention.
14. **Dashboard can contradict completed work** (sibling #4840): projections aren't watermarked or reconciled against external SCM truth.

**Framing / metrics**
15. **Activity, not outcome.** The product is oriented around agents/convoys/sessions moving, not around *accepted, surviving value per dollar and human-minute*. Vanity metrics (agents spawned, sessions, terminals) crowd out landed value.

None of these means Gas Town is bad — it's the best expression of the *tmux + Beads/Dolt + worktree* substrate. It means the substrate itself is the ceiling, and Bulletfarm changes the substrate.

---

## 2. Scoring all proposals (0–100, one rubric)

Rubric weights: Concurrency kernel 20 · Identity/quota/compliance 15 · Collaboration & review integrity 12 · Integration authority & Git safety 8 · Reliability & testability 12 · KPI system 12 · Security & threat model 6 · Operator UX 5 · Buildability & stack fit 6 · Evidence & honesty 4.

| Proposal | Score | One-line |
|---|---:|---|
| **NEXUS-FLEET** | **43** | Best cockpit texture + the one novel idea (tree-sitter symbol conflict), but fencing-free leases, a TOCTOU race *inside* its own lock manager, and an auth modal whose "Verified" button just closes the dialog. |
| **Armada v1** | **83** | Correct event-sourced kernel with fencing at an effects gateway and strong anti-gaming KPIs; used worktrees, lacked evidence-staleness binding, review not blind. |
| **Keel** | **88** | Best *kernel* document (two-phase spawn, idempotency keys, 409-suicide, no-worktree clones) and only verified-issue autopsy; fence-derivation under-stated, sandbox deferred, evidence not commit-bound. |
| **Constellation** | **86** | Best *evidence + identity* (commit-bound evidence with staleness, custody modes, credential generations, quota confidence, blind calibration-weighted review); kernel asserted not specified, worktrees, scope gravity to year two. |
| **Centerrail (submitted)** | **93.5** | Most complete synthesis; transaction-processor framing, Variant-as-fence, immutable Graph Delta, five trust planes, honest non-atomic effects; latent holes at verifier-circularity, attestation loop, CONTRADICTORY-exit, and a `--reference` GC race + proc-macro gap it didn't name. |
| **Adversarial red-team of Centerrail (doc 8)** | **79 as a review** | Landed two genuinely important catches (X2 GC race, X5 proc-macro) and several good ones (X4 staged race budget, X6 disjoint batching); but two fixes are *regressions* (X3 variable-renaming, X8 blanket transitive locking), its TLA+ doesn't actually model the wound-wait it claims to prove, and its 9.85 self-score treats amendments as shipped code. Scored *as a red-team*, not a design. |
| **Consolidated Centerrail (doc 9, my prior)** | **95.2** | Centerrail + 12 no-regression hardenings (invariant tiering, oracle-split, two-track scope, attestor≠broker, fence-mediated liveness exit, probe reservation, GA-decoupling). The base the final builds on. |

### Per-proposal feedback to improve (condensed)

**NEXUS-FLEET → to reach ~70:** thread a fence token through every effect; make lock-acquire a single transaction with a partial-unique index (kill the TOCTOU); demote AST from *lock* to *conflict-forecast*; persist+reconcile the broker with reservation settlement; make the login modal's success come from an adapter probe; delete the internal merge coordinator for GitHub's; write the KPI layer it skipped.

**Keel → to reach ~92:** state the fence-derivation rule (`MAX(attempts.fence)+1`, not the deletable lease row); fix `UNIQUE ... WHERE` to a partial *index*; add runner attestation for multi-host liveness; show the race-winner's real-lock acquisition; adopt commit-bound evidence + quota-confidence; pull rootless containers into v1.

**Constellation → to reach ~92:** import Keel's kernel wholesale (transactions, idempotency keys, two-phase spawn); ban worktrees outside a flagged trusted-local class; compress milestones A–B into weekly exit gates; mark every numeric KPI target as a post-baseline hypothesis; make the ChangeIntent compatibility matrix total and deterministic (no "probably").

**Centerrail → the 12 hardenings in doc 9**, *plus* the four adjudicated additions below (the red-team's real catches).

**Adversarial red-team (doc 8) → to be a better review:** keep X2/X5/X4/X6/X7; drop X3-variable-renaming and X8-blanket-locking (both regress the very thing they protect); actually model wound-wait in the TLA+ (the shown `AcquireLock` only has the no-conflict path); and score the *design*, not the design-after-my-amendments — and never above the implementation-maturity ceiling (~0.5/10 until code runs).

---

## 3. Adjudicating the adversarial red-team (doc 8) — verified, kept, or rejected

I pressure-tested every "critical" finding against primary git/build behavior. Verdicts:

| ID | Claim | Verdict | Disposition in final |
|---|---|---|---|
| **X2** | `clone --reference` + mirror `git gc --prune=now` → `fatal: bad object` in live workspaces | **CORRECT & important.** `--reference` writes `objects/info/alternates` into the mirror's store with no cross-process refcount; a mirror repack/prune can unlink objects a borrower needs. Real, documented. Every prior proposal (mine included) said `--reference` without mandating `--dissociate`. | **ADOPT (A2):** clones are `--reference ... --dissociate` (copy needed objects at clone time) **or** CoW reflink (`cp --reflink=always`) on Btrfs/XFS/APFS. Mirror GC can never corrupt a live workspace. |
| **X5** | `build.rs`/proc-macros (Rust) and custom transformers (TS) run arbitrary host code at *compile* time | **CORRECT & one of the best catches.** Sandboxing test *execution* is insufficient; `cargo build`/`tsc` execute code before any test runs. | **ADOPT (A5):** compilation of build scripts/macros runs in the same S1/S2 sandbox as tests, **network-denied, no secret mounts, no parent env**; verifier compiles in its isolation class with virtualized-TSC clamping to blunt timing side-channels. Compile = code execution, treated as such. |
| **X4** | N-Variant races reserve N full budgets → 5-hour org quota drained | **CORRECT, and the fix is also cheaper.** | **ADOPT (A4):** staged race budgets — every Variant gets ~15% (scaffold+tests), an arbiter culls to the top 1–2, survivors get the remaining ~85%; any `CRITICAL/HIGH` single-writer task preempts speculative Variants. |
| **X6** | At arrival rate λ > 1/T_verify, every merge re-invalidates queued candidates → infinite rebase livelock | **CORRECT queueing theory; sharper than my C10 backpressure.** | **ADOPT (A6):** **tree-disjoint preservation** — when PR K lands, a queued candidate whose write-set is provably disjoint from K's committed diff keeps its evidence (fast AST 3-way re-check only), no full re-verify; only *overlapping* candidates re-verify. Combine with C10 backpressure + stale-base pre-rebase. Livelock broken by avoiding needless invalidation, not just throttling. |
| **X7** | UNKNOWN liveness stalls forever | **Converges with my C5.** STONITH timing bound (`runner self-kills at LeaseDuration − Grace` < `server Expiry`) is the same self-fence, stated crisply. | **ADOPT the timing inequality** into C5's fence-mediated exit; guarantees the old runner is dead before a successor fence is leased. |
| **X1** | Two attempts each hold a lock and request the other's → deadlock | **Real vs *base* Centerrail; largely handled by my C3** (additive expansion doesn't hold-and-wait — it acquires-or-fails). Wound-wait is a *stronger* general fix. | **ADOPT wound-wait (A1)** as the general policy for any multi-resource expansion: total resource order + older-wounds-younger + **no retained partial claims** (all-or-nothing acquire). Subsumes C3's die/retry and eliminates the deadlock class outright. |
| **X3** | LLM family fingerprintable from code style → blind review is pseudo-blind | **Real threat, but the proposed fix partly regresses.** Formatting normalization (rustfmt/prettierP0+r4B31\) and comment-watermark scrubbing are safe and good. **Variable-renaming to `var_1`/`helper_fn_a` is a REGRESSION** — it destroys the reviewer's ability to judge naming quality and catch semantically-wrong identifiers, which is core review value. | **PARTIAL ADOPT (A3):** normalize formatting + scrub comment watermarks/self-references before blind review; **reject** identifier renaming. Residual stylometry is accepted and mitigated by the *calibration-weighted, cross-family* arbitration already in the design (a fingerprint doesn't help a reviewer that's scored on catching real defects). |
| **X8** | Change a trait → all implementors may break; auto soft-read-lock every implementor | **Real risk, but blanket transitive locking is a throughput-collapse REGRESSION** — a widely-implemented trait (`Display`) would lock hundreds of files and serialize the repo. | **PARTIAL ADOPT (A8):** the tree-sitter service *forecasts* interface-impl coupling and serializes/raced-splits **only on actual predicted overlap**; no blanket auto-lock. Advisory, not a global mutex. |
| **TLA+** | Skeleton "proves" deadlock-freedom | **Incomplete.** The shown `AcquireLock` models only the no-conflict path; the prose wound-wait (rollback B) isn't in the spec, so it proves nothing about deadlock-freedom. | **Note honestly:** the formal model must actually encode wound-wait + fence monotonicity to earn the claim (see C7 — this is one of the two protocols worth model-checking). |

Net: the adversarial round contributed **four real hardenings P0+r4B33\P0+r4B34\P0+r4B35\P0+r6B42\P0+r5053\P0+r5045\(A2, A5, A4, A6)** and **two crisp refinements (A1 wound-wait, A7 STONITH timing)**, while **two of its fixes were rejected as regressions (A3-rename, A8-blanket-lock)** — kept only in their non-regressing forms. This is exactly the discipline the exercise demands: adopt what hardens, refuse what trades one failure for another.

---

## 4. BULLETFARM — the complete model

Centerrail's architecture, adopted intact, hardened by the 12 no-regression fixes from the consolidated round (C1–C12) **plus** the six adjudicated additions from the adversarial round (A1, A2, A4, A5, A6, A7). Renaming the *engineering* would be ego; the Bulletfarm name is the product skin and the lore that carries the advanced concepts in §5.

### 4.1 The shape (five trust planes, unchanged)
Control · Execution · Verification · Delivery · Evidence/Audit. Rust modular monolith (`bulletfarmd`) + separate `runnerd` / `verifierd` / `deliveryd` (real credential boundaries, not microservice fashion). SQLite-WAL (dev/small team) → Postgres HA (fleet), identical `sqlx` schema. Vite/React cockpit, types generated from Rust (`ts-rs`; CI fails on drift). Domain: Mission → Plan Revision → (Graph Delta) → Work Package → Selection Group → **Variant (the fence/lease boundary)** → Attempt → Candidate → Evidence(E0–E4) → Review → Integration → Observation Window.

### 4.2 The kernel invariants that actually make it safe
- **Fencing everywhere, checked where damage happens.** Monotonic per-Variant fence derived from `MAX(attempts.fence)+1` (survives lease-row deletion). Every side effect presents the fence at a single effects gateway; stale fence → rejected inside the ledger transaction. A zombie agent cannot push, comment, or settle.
- **Idempotent graph mint (the #5511 kill).** `POST /plans/{content-hash}/materialize` inserts the entire DAG in one SERIALIZABLE transaction or nothing; replays return the existing graph. Command idempotency keys on every mutation; ambiguous timeout → **OUTCOME_UNKNOWN, never auto-retry** — the resolver adopts identity-exact (see below) before any replay.
- **Private clones, GC-safe (A2).** `git clone --reference <bare-mirror> --dissociate` (or CoW reflink) → every writer has a *private* `.git`, index, working tree; mirror GC can never corrupt a live workspace; worktrees banned by CI grep.
- **Compile = code execution (A5).** build-script/proc-macro/transformer compilation runs network-denied, secret-free, in the writer/verifier sandbox class; verifier clamps virtualized TSC.
- **Evidence bound to exact subject.** `{subject_commit, patch_hash, env_hash, input-closure, tier E0–E4}`; a moved commit invalidates by default; reuse is a conservative whitelist, never a heuristic.
- **Deadlock-free scope expansion (A1 + C3).** Two-track: additive-within-authority-envelope is a cheap lock-acquire+fence-bump; authority-escalating uses the successor-Attempt protocol. Multi-resource expansion uses **wound-wait** (total order, older-wounds-younger, no retained partial claims). Deadlock class eliminated.
- **Identity-exact effect adoption (C9).** Logical effect key includes the fence; a remote ref is adopted iff its OID equals *this* attempt's `desired_state_hash`, else `ORPHANED_REMOTE`, quarantined. No stale-incarnation laundering.
- **Sovereign integration (C4).** GitHub required checks / branch protection / merge queue are the only merge authority. An **attestation verifier** (not the delivery Broker) recomputes the verdict from the signed proof-bundle and posts the green check with its own credential; the Broker only delivers code. No LLM output is an input to the merge decision.
- **Fence-mediated liveness exit (C5 + A7 STONITH).** Liveness is `{ALIVE, DEAD, UNKNOWN, CONTRADICTORY}`; the latter two fail closed but raise a typed Intervention with fence-mediated safe exits; runner self-kills at `LeaseDuration − Grace` < server `Expiry`, so the old runner is provably dead before a successor fence is leased.

### 4.3 Identity, quota, login (the subsystem the user asked for, Gas Town lacks)
Named profiles with `{owner, authorization_class, custody_mode, credential_generation, health}`. **Custody:** personal CLI subs stay in local custody on the owner's host (metadata only centrally); API/enterprise use KMS envelope. **Login challenges:** broker raises one-open-per-identity (partial unique index); cockpit inbox shows the device-code/OAuth link parsed from the adapter; the *named owner* completes the provider's own flow; **success comes from an adapter `auth_status` probe, never a UI button**; a re-login increments `credential_generation` and kills older grants. **Quota epistemology:** typed observations ranked official-API → structured-event → headers → typed-limit → estimator → **`unknown`, which is never scheduled as headroom**; pre-turn reservations settle against actuals; a bounded, logged **probe reservation (C6)** converts `unknown`→known without scheduling bulk work on unknowns. **Compliance as code (C6b):** `named_human_subscription` profiles are rate-governed to **one-seat-per-human regardless of seat count**, so ten consumer logins buy zero throughput advantage — the circumvention incentive is removed by construction; throughput comes from `api_credit`/`organization_seat`/`service_identity`. *(Grounds: the three vendors' published terms on credential sharing and limit circumvention; Anthropic's Agent-SDK-with-plan guidance. Operational fact, re-verify; not legal advice.)*

### 4.4 Collaboration & review integrity
Six **type-level** protocols (partition/pipeline/pursuit(race)/council/pair-shadow/investigate); a Variant is the write boundary; models cannot invent a seventh that shares a tree. Races use **staged budgets (A4)**. Review is **blind + cross-family**, verdict cites hunks + tests, arbitration is **calibration-weighted** (discount correlated agreement), and **oracle-modifying diffs (C2)** — tests/CI/fixtures — force pre-change-suite-still-passes + a different-family reviewer, with **hidden holdouts required for R2+**. Blind-review inputs are **formatting-normalized and comment-watermark-scrubbed (A3)** — but identifiers are *not* renamed (that would regress reviewability). The tree-sitter service **forecasts** interface/impl coupling and serializes only on actual overlap (A8) — never a blanket transitive lock.

### 4.5 Integration flow (beats Gas Town's Refinery at its own game)
Integration Coordinator: conflict-forecast → **stale-base pre-rebase before entering the batch** → speculative **tree-disjoint batch synthesis (A6)** (Bors-style bisection, but disjoint candidates keep their evidence and only overlapping ones re-verify) → merge-group CI bound to the exact synthetic SHA → GitHub merge queue decides → resulting target commit verified → Observation Window (survival). Verifier queue depth is a **writer-admission backpressure input (C10)**, so the re-verification amplifier is bounded and the λ>1/T_verify livelock cannot form.

### 4.6 Cockpit (honest projection)
Five screens (Control Tower / Mission Graph / Fleet+live xterm / Merge Rail / Evidence Bay), all projections over the event log; **no skip-checks affordance exists in the UI**; pending operator commands show as pending; the **freeze/stop chip resolves in two shown stages (C11)** — "recorded" → "enforced on N/M runners, 1 unreachable, leases expire in 12s" — so the operator sees true safety state under partition; multi-repo `MANUAL_RECOVERY_REQUIRED` **quarantines the saga blast radius, not the fleet (C12)**. If the board disagrees with GitHub required checks, GitHub wins and a drift badge renders.

### 4.7 KPI system (three loops by measurement latency)
Every metric: ledger-computable formula + alert + **named anti-gaming counter-metric**; vanity metrics (tokens/agents/sessions/LoC) are denominators only. **Fast loop (steer):** lease-integrity=0, first-pass CI, surprise-429, scope-expansion escalation rate, projection lag, stale-evidence-into-queue=0. **Medium (ratchet from baseline):** autonomous-merge rate, cost/accepted-PR, repair-thrash, same-basis Gini. **Slow (trust, weeks):** Survival@30d (blame-lineage), escaped-defect, revert. North star: **Landed Autonomous Value** (merged, all-evidence-satisfied, zero human code edits, unreverted@14–30d) ⇄ Survival@30d as its counter-metric. **DFI (dumpster-fire index)** first-class: orphan leases, duplicate mints, stale-fence-accepted, shared-cwd, markdown-as-lock — all target 0. **Every numeric target is a 4-week-baseline hypothesis, then ratcheted** — never asserted as physics.

### 4.8 Assurance & build plan
**Invariant tiering (C1):** every invariant tagged T1-schema / T2-gateway / T3-test; untagged = review failure. **Formal-model exactly two protocols (C7):** lease/fence/reclaim and command/effect idempotency-under-ambiguous-timeout (the #5511 class) — and the model must actually encode wound-wait + fence monotonicity (the adversarial TLA+ didn't). **GA requires all four certified providers (C8, frozen V1)** (Claude, Codex, Cursor, Antigravity); the historical "any two" proposal is superseded. 12-week Phase-A/B kernel with per-week exit gates; the gate is not "twenty agents ran" but: *one Mission materialized exactly once; one Variant took a never-reused fence; the #5511 duplicate could neither mutate state nor create an effect; the writer ran in a GC-safe private clone; compile-time code ran network-denied; work survived kill-9 at every boundary; a clean verifier produced exact-subject evidence the author couldn't weaken; GitHub got one brokered candidate via an attestor-published check with no SCM credential in the sandbox.*

---

## 5. The Bulletfarm doctrine — advanced multi-agent / role-based / evolutionary design, carried by lore

Here is the bonus the brief asked for: genuinely powerful multi-agent, role-based, and evolutionary-computation concepts, each with a real systems mechanism *and* a Bulletfarm story for why it belongs. The lore is a mnemonic; underneath each is a defensible engineering primitive that improves the system.

### 5.1 The Blood Bag — hostile QA as a survival environment
**Lore.** In the Citadel, the healthy are bled to keep warlords alive. Bulletfarm inverts the cruelty into rigor: every Candidate is a War Boy that must survive the Blood Bag before it's allowed near the rig. QA is not a checkbox; it is a *hostile life-or-die environment* the code must physically survive.
**Mechanism (real).** The Verification plane runs a **gauntlet of escalating hostility tiers** the author cannot see or weaken: E1 writer-visible → E2 clean independent → **E3 hidden/historical holdout + adversarial mutation testing + fuzzing** → E4 human. "Witness me" is literal: a Candidate that dies in the Blood Bag produces immutable death evidence (the failing subject) and its lineage is marked contaminated. This operationalizes C2's oracle-split: the hostile oracle lives *outside* the author's reach, and only survivors ride.

### 5.2 The Pursuit — evolutionary best-of-N with staged fuel
**Lore.** When the objective is uncertain and valuable, you don't send one car — you send a *pursuit*: many War Rigs down the Fury Road, and only the one that reaches the target with the cargo intact wins. Fuel is precious, so you don't fill every tank; you give each a taste, watch who's driving well, and pour the guzzoline into the survivors.
**Mechanism (real).** The `pursuit` protocol is a **(μ, λ) evolutionary strategy** over Variants: spawn λ candidates with ~15% fuel (A4), an arbiter evaluates the *phenotype* (test coverage + AST approach), culls to the μ best, pours the remaining fuel into survivors, and the Selection transaction integrates exactly one. Cross-family diversity is the mutation operator; the calibration-weighted arbiter is fitness. This is genuine evolutionary computation with a hard budget, not "run three and pick one."

### 5.3 The War Council — island-model diversity against groupthink
**Lore.** Furiosa, Max, and the Vuvalini plan the run separately before they argue — because if everyone rides in one rig and it flips, the plan dies with it. Independent councils that converge beat one convoy that agrees with itself.
**Mechanism (real).** The `council` protocol is the **island model** of evolutionary computation: each provider family plans in isolation (no cross-talk until plans lock), a deterministic contradiction-extractor finds disagreements, and synthesis merges. This prevents the correlated blind spot that a single-family plan-write-review pipeline hides — the reason arbitration is calibration-weighted (2-of-a-family agreeing is weaker than 2 independent families).

### 5.4 The Witness & the Deacon — a watchdog hierarchy that can't lie
**Lore.** Gas Town already has Witnesses and a Deacon; Bulletfarm keeps the *idea* and removes the tmux eyes. A Witness that watches a terminal is a blind priest. Bulletfarm's Witnesses read *evidence*, not scrollback.
**Mechanism (real).** A three-tier reconciler (per-Variant heartbeat → per-repo Witness → cross-repo Deacon) drives the four-valued liveness machine and the fence-mediated STONITH exit (C5/A7). The difference from Gas Town: every watchdog decision is a **projection over the hash-chained event log**, so the watchers cannot themselves become a lying source of truth. Adopting a good idea, fixing its substrate.

### 5.5 The Wasteland Ledger — federated reputation as a fitness memory
**Lore.** Gas Town's Wasteland lets towns post work and earn portable reputation stamps. Bulletfarm keeps the poetry and hardens the ledger: a rig's *reputation* is which frontier family actually lands surviving code **on this repo, this language, this task-class**.
**Mechanism (real).** The Performance Graph is an **evolutionary fitness memory**: per (family, model, task-class, repo) it records accepted-and-survived outcomes, and the router is a **constrained contextual bandit** with holdouts, exploration caps, and per-decision explanations — learning *routing*, never safety or permission rules. Reputation is earned by survival (the slow-loop KPI), not by self-report. This is the honest, non-gameable version of Wasteland stamps.

### 5.6 Mother's Milk & guzzoline — quota as a metered, life-critical resource
**Lore.** In the Citadel, water and fuel are hoarded and metered because scarcity is lethal. Bulletfarm treats provider quota the same way: you never pretend a dry tank is full, and you spend a cup to check the level before committing the convoy.
**Mechanism (real).** The capacity graph's typed observations with `unknown`-stays-`unknown`, reservation/settlement, and the **probe reservation (C6)** are exactly "spend a known-small cup to measure before committing." Compliance-as-code (C6b) is the anti-hoarding rule: stockpiling consumer seats buys no advantage. The lore makes the epistemic honesty *memorable*: a full-looking tank you haven't dipped is not fuel.

### 5.7 The Rig's dead-man switch — self-fencing under partition
**Lore.** Immortan's war rig has a dead-man's trigger: lose the driver and it doesn't keep barreling forward blind. A partitioned Bulletfarm runner does the same — cut off from the Citadel, it stops itself before it can do harm.
**Mechanism (real).** Runner self-kill at `LeaseDuration − Grace` (A7 STONITH) is the dead-man switch: a runner that can't renew its lease terminates its own processes *before* the ledger leases a successor fence, converting split-brain from "two live writers" into "one live + one self-terminated," with fencing guaranteeing no double delivery regardless. CAP says you can't prevent the partition; the dead-man switch makes it *safe*.

Each of these is a real primitive — hostile hidden-oracle QA, (μ,λ) evolutionary races, island-model planning, evidence-based watchdogs, bandit-routed fitness memory, metered-resource epistemics, dead-man self-fencing — dressed in lore that makes the doctrine teachable. The story is the mnemonic; the mechanism is the moat.

---

## 6. Final fair score

Same 10-dimension rubric. This is the design *after* the consolidated round (95.2) plus the six adjudicated adversarial hardenings — and *minus* nothing, because the two regressive "fixes" were refused.

| Dimension (wt) | Consolidated (doc 9) | **Bulletfarm** | Why it moved |
|---|---:|---:|---|
| Concurrency kernel (20) | 19.6 | **19.7** | A1 wound-wait eliminates the scope-expansion deadlock class outright; A7 STONITH tightens the split-brain bound |
| Identity/quota/compliance (15) | 14.6 | **14.6** | already near-ceiling; vendor-capped epistemics hold it below 15 honestly |
| Collaboration & review (12) | 11.6 | **11.7** | A3 (formatting-normalize + watermark-scrub, no rename) hardens blind review without regressing reviewability; A4 staged budgets |
| Integration authority & Git safety (8) | 8.0 | **8.0** | at ceiling; A2 GC-safe clones + A6 disjoint batching are *reliability* gains, scored below |
| Reliability & testability (12) | 11.6 | **11.9** | A2 (GC-safe clones) closes a real corruption vector; A6 (disjoint batching) breaks the rebase livelock; A5 compile-sandbox |
| KPI system (12) | 11.7 | **11.7** | three-loop + DFI + baseline-then-ratchet already strong; unchanged |
| Security & threat model (6) | 5.6 | **5.9** | A5 (compile = code execution, network-denied) closes the single most-cited under-specified hole across all proposals |
| Operator UX (5) | 5.0 | **5.0** | at ceiling; two-stage freeze chip + saga quarantine already in |
| Buildability & stack fit (6) | 5.2 | **5.3** | A4/A6/A2 are standard, well-understood mechanisms (reflink, Bors-disjoint, wound-wait) — they *reduce* novel risk |
| Evidence & honesty (4) | 4.0 | **4.0** | primary-source-verified throughout; regressive fixes explicitly refused; TLA+ gap named |
| **Weighted total** | **95.2** | **96.4** | |

### **Final score: 96.4 / 100 (paper architecture)**

**Why it moved up 1.2.** The adversarial round found two genuinely important, primary-source-verifiable holes that *every* prior proposal (mine included) under-specified — the `--reference` GC corruption race (A2) and compile-time arbitrary code execution (A5) — and two sharper mechanisms (A4 staged race budgets, A6 tree-disjoint batching) plus two crisp refinements (A1 wound-wait, A7 STONITH). Adopting exactly those, and *refusing* the two regressive fixes (variable-renaming, blanket transitive locking), is a strict, no-regression improvement.

**Why not higher — the load-bearing ~3.6 I won't inflate:**
1. **Implementation maturity is still ~0.5/10.** This is a design. The score is for the architecture; the *product* is unproven until the Phase-A chaos suite is green on real hardware. No prose closes this gap.
2. **Verifier independence rests on a human-maintained holdout corpus (C2/§5.1).** Perfect on paper; only as strong as the hidden tests a team actually curates and rotates.
3. **Cross-provider quota is vendor-capped (C6/§5.6).** `unknown` is made *safe and measurable*, never *known*. No architecture transcends what vendors disclose.
4. **Split-brain is bounded, not eliminated (A7/§5.7).** CAP forbids the clean win; the dead-man switch makes it safe and visible — that is the physics ceiling.
5. **The system is large.** Invariant-tiering (C1) makes assurance auditable, but a 54-invariant, five-plane, four-daemon system is a multi-year program; integration risk is real until the kernel is boring.
6. **The evolutionary/bandit layer (§5.2/5.5) can overfit.** Learned routing and staged-race fitness are powerful but need holdouts, exploration caps, and drift decay to avoid optimizing for the wrong signal — mitigated by design, not eliminated.

**Why not lower.** Every adopted change hardened or simplified with no regression; the two regressive fixes and the four cuts that would have gutted the design (drop the verifier, mandate Postgres, collapse the planes, internalize merge) were explicitly refused; the kernel is correct at the transaction level and now deadlock-free, GC-safe, compile-sandboxed, and identity-exact under adversarial adoption; and the doctrine layer adds real evolutionary and role-based sophistication rather than decoration. The last ~3.6 points are purchasable only with the one thing prose cannot fake: running code with the chaos suite green — the moment Bulletfarm stops being the best *design* and starts being the best *system*.
