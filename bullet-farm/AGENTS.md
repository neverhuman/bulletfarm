# bullet-farm Agent Instructions

Read `SPLIT.md` first. This repository is the public hub of the Bullet Farm
split family.
Discover the outermost family root from `repos.manifest.toml`. Read its
`AGENT_CHAT.md` before every claim and edit, and use `bullet-family coord` for
machine-enforced claims. Never commit a machine-local absolute chat path.

Read `agent/JANKURAI_STANDARD.md` next.
Do not edit outside requested ownership. Run the mapped test lane before the
final response.

- Canonical local Jeryu slug: `root/bullet-farm`.
- Do not add committed cross-repo `path = "../..."` dependencies.
- Do not hand-edit generated artifacts listed in `agent/generated-zones.toml`.
- Before a worker handoff, run the mapped focused tests in a private target.
  Do not run `scripts/ci-local.sh` from a worker lane: it owns the repository's
  serialized proof lock. Only the recovered sole-writer runs
  `bash scripts/ci-local.sh required`, after exact handoffs are integrated and
  the checkout is clean.
- Spec documents under `docs/spec/` are design corpus, not runtime authority.

## Fleet discipline

Eight to fifteen agents from three model families work four shared trees at
once, so every rule here exists to stop one agent's failure from becoming
everyone's. These are normative. If you believe an exception applies, post the
exception to the family `AGENT_CHAT.md` before you act on it, never after.

- **No claim outlives its owner.** A heartbeat may issue only from the process
  that owns that lane, and no loop may span more than one claim, because on
  2026-08-26 a blanket heartbeat loop outlived the session that started it: it
  kept eight ownerless claims alive for hours, blocking every serialized window
  behind them, and its last write was cut mid-record and froze the shared
  ledger for two days (`AGENT_CHAT.md` 2026-08-27T15:58:11Z;
  `docs/decisions/0015-dogfood-track.md`, "Coordinator"). Kill your loop by PID
  when the lane ends, never by pattern. The coordinator checks that a heartbeat
  names the claim's owning agent string (`CLAIM_OWNER_MISMATCH`,
  `src/coord/state.rs:221-226`) but cannot see which process sent it, so the
  rule that no claim outlives its owner is enforced by discipline; product
  enforcement is `bullet-family check dogfood --json` (ADR 0015,
  "Consequences"), unbuilt.
- **HOLD before stopping.** Before you stop for ANY reason — finished, blocked,
  interrupted, or out of provider budget — append a HOLD line naming every
  touched path with its `sha256sum`, so the next owner resumes from bytes
  instead of from your memory of them. Running low on budget is the case this
  rule was written for, not an exemption from it. Enforced by discipline; no
  product surface can observe a session that simply stops.
- **Per-lane heartbeats.** Run one per-lane heartbeat loop per claim, owned by
  the worker's own lifetime, so that a dead worker's claim expires instead of
  blocking the fleet. `--ttl-seconds` must be `30..=86400` and defaults to 600
  (`src/coord/mod.rs:393-399`, `src/coord/model.rs:42`), so beat well inside
  your own TTL. Name the loop file for your agent
  (`heartbeat-<agent-id>.sh`) and kill it by PID, because the scratchpad is
  shared between lanes and a pattern-matched `pkill` has already killed other
  lanes' loops (`AGENT_CHAT.md` 2026-08-25T10:04:35Z).
- **Four claimed files, no more.** A lane holds at most four claimed files —
  three source plus one test — because a claim is the unit of both exclusivity
  and recovery: a wider claim blocks more agents and cannot be restated
  honestly in one HOLD line. `--path` is repeatable and only requires at least
  one path (`PATH_REQUIRED`, `src/coord/state.rs:328-334`), with no upper bound
  in code, so the cap is enforced by discipline; product enforcement is
  `bullet-family check dogfood --json` (ADR 0015), unbuilt.
- **Prove in a private `CARGO_TARGET_DIR`, never the shared target and never
  the shared proof-custody lock.** Export a target directory of your own,
  outside every worktree, before any `cargo` command
  (`CARGO_TARGET_DIR=<private-dir>/<agent-id>`), so concurrent lanes cannot
  corrupt or serialize on each other's build output. A worker lane proves with
  focused commands only (`cargo test --locked -p <crate> …`, `cargo clippy
  --locked -p <crate> --all-targets -- -D warnings`, `rustfmt --check`,
  `shellcheck -x`) and does not take `<repo>/.git/bullet-ci.lock.d`: that lock
  is one per repository, and a worker killed while holding it leaves every
  other lane refusing with `CI_PROOF_LOCKED_OR_STALE` and exit 75
  (`ops/ci/family-custody.sh`, `ci_proof_acquire`/`ci_proof_refusal`). The
  observed lanes above take that lock by design — run them one at a time, only
  when your task says to, and never leave one holding it. The stale-lock
  refusal is enforced by code; keeping worker lanes out of the lock and off the
  shared target is enforced by discipline.
- **Read the physical tail of the log immediately before every append.**
  `AGENT_CHAT.md` is append-only with many concurrent writers, so a tail read
  even minutes ago is already stale and an append written against it can
  contradict a claim, HOLD, or withdrawal posted since. Re-read, then append,
  and never edit anything above your own entry. Enforced by discipline; no
  product surface reads `AGENT_CHAT.md`.

The coordinator ledger is frozen under recovery: run no `bullet-family coord`
verb until recovery completes. See
[`docs/runbooks/fleet.md`](docs/runbooks/fleet.md).

Source citations above name a symbol as well as a line, because the lines were
read from a shared working tree on 2026-08-28 and several of these files have
live owners: trust the symbol, re-find the line.
