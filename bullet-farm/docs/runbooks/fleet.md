# Fleet runbook

Status: **the human protocol is active; every `coord` verb is suspended under coordinator recovery**
Owner: Bullet Farm maintainers
Last reviewed: 2026-08-28
Applies to: all bullet repos

## Recovery notice: run no `coord` verb until recovery completes

The 2026-08-26 incident left the live ledger frozen — `events.jsonl` at mode 0400 with no `CURRENT`
(ADR 0015, "Coordinator"). That is the `Legacy` presence, and every ledger entry point refuses it
before doing anything: `COORD_RECOVERY_REQUIRED` ("legacy `events.jsonl` exists without `CURRENT`;
explicit recovery is required", `src/coord/store/ledger.rs:451-456`), reached from both the read path
(`ledger.rs:128-138`) and the guarded write path (`store/ledger/transaction.rs:132-141`). A retired
source with no published `CURRENT` refuses `COORD_RECOVERY_IN_PROGRESS` instead.

So: **no `bullet-family coord` verb may be run until recovery completes** — not `claim`, not
`heartbeat`, not `handoff`, not `receipt`, and not `status`. Do not attempt `init` to "fix" it: a
fresh Genesis over a retained incident is an operator act gated on OD-K ratification, its typed
predecessors, and an independent reviewer distinct from the operator (ADR 0015, "Coordinator"). The
recovery producers are the recovery owner's, are documented in `docs/runbooks/coordinator-recovery.md`,
and remain blocked for the real incident.

While the freeze holds, coordination is the human protocol only: append-only entries in the family
`AGENT_CHAT.md`, read-only design and test work on disjoint paths, HOLD lines carrying `sha256sum` for
every touched path, and no claim, handoff, receipt, or commit by a worker. The fleet rules that this
runbook's loop assumes are normative in [`../../AGENTS.md`](../../AGENTS.md), "Fleet discipline".

## Coordination surfaces

Coordination is rooted at the outermost ancestor containing `repos.manifest.toml`; no public file
contains a host-specific absolute path. Human decisions remain append-only in
`<family-root>/AGENT_CHAT.md`. Machine claims are append-only in the ignored
`<family-root>/.bullet-family/coord/` generation, and are changed only through the verbs below.

Every source citation below names a symbol as well as a line. The line numbers were read from a
shared working tree on 2026-08-28 and several of these files have live owners, so the symbol is the
durable anchor and the line is a convenience.

Two identifiers appear on every mutating verb, and both are validated before anything is read:

- `--request-id` is `req_` plus exactly 64 lowercase hex characters
  (`RequestId::parse`, `src/coord/model/api.rs:11-16,133-141`). It is the idempotency key. Re-issuing
  the same `--request-id` with a byte-identical command subject returns the original record with
  `replayed: true` (`src/coord/store/projection.rs:157`); re-issuing it with a different subject is
  refused `COORD_REQUEST_CONFLICT` ("request ID already binds another normalized command subject",
  `src/coord/store/mutations.rs:315-319`). Mint one per intended command, never per attempt.
- `--expected-generation` is `gen_` plus exactly 64 lowercase hex characters
  (`GenerationId::parse`, `src/coord/model/api.rs:31-36`). It is the fence: the append refuses
  `COORD_SUBJECT_CHANGED` ("append expected another generation") when the coordinator's `CURRENT`
  generation is not the one you read (`src/coord/store/ledger/transaction.rs:142-145`). Capture it
  from a status read; never carry one across a restart without re-reading.

## Verbs, as the CLI actually takes them

Every flag below is an allow-listed option of that action in `src/cli/coord.rs`; anything else is
refused `UNKNOWN_OPTION`, and `--json`/`--all` are the only bare flags the parser accepts at all
(`src/cli/coord.rs:279-289`). Suspended under the recovery notice above; recorded here so the
successor loop is written against the real shapes, not the schema-1 ones.

```bash
# Creation-free read. Takes a shared lock, opens read-only, creates nothing.
# Rejects every --value option, including --request-id.
bullet-family --root /absolute/family coord status --json --all

# One generation, once, at genesis. No --request-id, no --expected-generation:
# it is what creates the generation the others fence against.
bullet-family coord init --operator codex-root \
  --policy-sha256 <64-hex> --replay-contract-version 1 --replay-contract-sha256 <64-hex> \
  --bootstrap-commit 0123456789abcdef0123456789abcdef01234567 --bootstrap-path crates/runner

bullet-family coord claim --request-id req_<64-hex> --expected-generation gen_<64-hex> \
  --agent codex-a --lane L4 --repo bullet-kernel --path crates/runner --ttl-seconds 600

bullet-family coord heartbeat --request-id req_<64-hex> --expected-generation gen_<64-hex> \
  --claim clm_... --agent codex-a --ttl-seconds 600 --note proof-started

bullet-family coord handoff --request-id req_<64-hex> --expected-generation gen_<64-hex> \
  --claim clm_... --agent codex-a \
  --proof 'cargo test --locked' --exit-code 0 --changed-path crates/runner/src/lib.rs

bullet-family coord receipt --request-id req_<64-hex> --expected-generation gen_<64-hex> \
  --claim clm_... --orchestrator codex-root \
  --commit 0123456789abcdef0123456789abcdef01234567 \
  --committed-path crates/runner/src/lib.rs

bullet-family coord receipt-group --request-id req_<64-hex> --expected-generation gen_<64-hex> \
  --claim clm_a... --claim clm_b... \
  --orchestrator codex-root --commit 0123456789abcdef0123456789abcdef01234567

bullet-family coord correct-receipt --request-id req_<64-hex> --expected-generation gen_<64-hex> \
  --claim clm_... --orchestrator codex-root \
  --previous-commit 0123456789abcdef0123456789abcdef01234567 \
  --commit 89abcdef0123456789abcdef0123456789abcdef \
  --committed-path crates/runner/src/lib.rs --reason 'amended commit after path-exact fixup'

bullet-family coord correct-receipt-group --request-id req_<64-hex> \
  --expected-generation gen_<64-hex> \
  --claim clm_a... --claim clm_b... --orchestrator codex-root \
  --previous-commit 0123456789abcdef0123456789abcdef01234567 \
  --commit 89abcdef0123456789abcdef0123456789abcdef \
  --reason 'split a contaminated shared-index commit into an exact replacement commit'
```

Notes that follow from the code, not from habit:

- `--path`, `--changed-path`, `--committed-path`, `--bootstrap-path` and the grouped `--claim` are
  repeatable and required — absent, they refuse `MISSING_OPTION` (`src/cli/coord.rs:328-333`). Every
  other option must appear exactly once or refuse `DUPLICATE_OPTION`.
- `--ttl-seconds` is optional on `claim` and `heartbeat`, defaults to 600
  (`DEFAULT_TTL_SECONDS`, `src/coord/model.rs:42`) and must be `30..=86400`
  (`validate_ttl`, `src/coord/mod.rs:393-399`), checked on both verbs
  (`src/coord/store/subject.rs:70,83`).
- `--note` on `heartbeat` is optional; `--exit-code` on `handoff` is optional and defaults to 0.
- `handoff` sets `commit_oid: None` unconditionally (`src/cli/coord.rs:126`): a worker cannot record a
  commit even by accident.
- `heartbeat`, `handoff` and the receipt verbs refuse `CLAIM_OWNER_MISMATCH` when the named agent is
  not the claim's owner (`src/coord/state.rs:221-226`). That checks the agent *string*; nothing in the
  product can see which process sent the beat, so "no claim outlives its owner" is enforced by
  discipline — product enforcement is `bullet-family check dogfood --json` (ADR 0015,
  "Consequences"), which is built and fail-closed: it exits non-zero whenever the
  loop is inoperable, naming each blocker (`loop_blockers`).
- The recovery actions — `recovery-inspect`, `recovery-provenance`, `recovery-manifest`,
  `recover-rollover`, `recovery-plan`, `recovery-proof`, `recovery-review`, `recovery-request`,
  `adopt` — take sealed-document paths (`--interrupted-capture`, `--tainted-generation`,
  `--frozen-live-source`, `--inspection`, `--authorization`, `--authorization-signature`,
  `--bootstrap-provenance`, `--manifest`, `--plan`, `--approval`, `--request`, `--output`,
  `--cargo-bin`, `--rustc-bin`, `--bootstrap-commit`), never `--request-id` or
  `--expected-generation`. They are the recovery owner's and are documented in
  `docs/runbooks/coordinator-recovery.md`.

## The operating loop

This is the loop the fleet actually runs, and the only order in which these verbs are safe
(`docs/assurance/execution-plan.md`, "one operating loop"). Every step is a worker step except the
sole-writer step, which is the orchestrator's.

1. **Creation-free status read.** `coord status --json --all`. It rejects every `--value` option,
   opens the coordination files read-only under a *shared* lock, and creates nothing
   (`src/cli/coord.rs:224-233`, `src/coord/store/ledger.rs:128-138`, `fs/io.rs:62-94` — no `O_CREAT`
   on any path). Read the board before you believe anything about it.
2. **Generation capture.** Take `generation_id` from that same status JSON
   (`Status`, `src/coord/model/api.rs:116-131`) and use it verbatim as `--expected-generation` for
   every command in this lane. A generation captured before a restart is stale by assumption.
3. **Bounded claim.** `coord claim` with `--request-id`, `--expected-generation`, at most four
   claimed files (four `--path` values), and a `--ttl-seconds` you can actually beat. The claim is the unit of exclusivity:
   claim only what you will edit.
4. **Per-lane heartbeat.** Run one per-lane heartbeat loop per claim, from the process that owns
   the lane, at least every five minutes and on proof, blocker, commit, or handoff. Never one loop across several claims, and
   never a loop that outlives the worker — see [`../../AGENTS.md`](../../AGENTS.md), "Fleet
   discipline", for the 2026-08-26 incident this rule comes from.
5. **Proof-bearing handoff, no commit by the worker.** `coord handoff` with the exact proof command,
   its `--exit-code`, and one `--changed-path` per file you actually changed. Handoff requires green
   proof and rejects every changed path outside the claim. The worker stages, commits, stashes,
   resets and pushes nothing.
6. **Sole-writer commit and receipt.** The orchestrator commits path-exactly, then records a
   single-claim `receipt` with every committed path, or a `receipt-group` whose handed-off path union
   exactly matches the commit. One writer, one commit, one receipt.
7. **Restart read-back by request ID.** After any interruption, re-read status and re-issue the *same*
   `--request-id` for the command you are unsure about. A byte-identical subject replays and returns
   `replayed: true`; it does not write twice.
8. **Typed reconciliation before any retry.** Never retry on a guess. Reconcile by the original
   request identity and the desired state, then act:

   | Refusal | What it means | What to do |
   | --- | --- | --- |
   | `COORD_RECOVERY_REQUIRED` / `COORD_RECOVERY_IN_PROGRESS` | the frozen generation, or a retired source with no published `CURRENT` | stop; this runbook's recovery notice applies |
   | `COORD_SUBJECT_CHANGED` | your `--expected-generation` is not `CURRENT` | re-read status, recapture the generation, do not reuse the old fence |
   | `COORD_REQUEST_CONFLICT` | that `--request-id` already binds a different command subject | mint a new request ID for a genuinely different command; never reuse one to force a change through |
   | `CLAIM_OVERLAP` | an active claim contains, or is contained by, your path | that path has an owner; post `BLOCKED` naming it |
   | `CLAIM_OWNER_MISMATCH` / `CLAIM_NOT_ACTIVE` / `CLAIM_NOT_FOUND` | not your claim, expired, or never existed | claim again; expired claims stop blocking and cannot be revived |
   | `PATH_OUTSIDE_CLAIM` / `COMMITTED_PATH_MISMATCH` / `RECEIPT_CORRECTION_MISMATCH` | the handed-off, committed, or previously receipted path or commit set does not match | fix the set; the whole locked append is rejected, nothing partial landed |
   | anything `_UNKNOWN` or `TIMEOUT` | the outcome is genuinely unobserved | run authoritative read-back and keep the subject frozen; do not dispatch a second write or switch providers |

   A stale generation, changed subject, failed proof, missing receipt, or `UNKNOWN` stops the lane.
   `UNKNOWN` is reported as `UNKNOWN` and is never reconciled into success. The full class-by-class
   contract is [`../errors.md`](../errors.md) —
   [conflict or changed subject](../errors.md#conflict-or-changed-subject),
   [outcome unknown](../errors.md#outcome-unknown),
   [receipt missing](../errors.md#receipt-missing).

## Rules that do not change

The CLI takes an exclusive file lock across replay, overlap detection, and one append. Active claims
overlap when either repository-relative path contains the other on a segment boundary. Expired
claims stop blocking and cannot be revived; claim again. Handoff requires green proof and rejects
every changed path outside the claim. After an exact-path commit, the orchestrator records either a
single-claim `receipt` with every committed path or a `receipt-group` whose handed-off path union
exactly matches the commit. `correct-receipt` rebinds one already-receipted claim to a different
commit: it refuses unless `--previous-commit` equals the commit OID currently recorded on that claim
(`RECEIPT_CORRECTION_MISMATCH`), requires the `--committed-path` set to match the handed-off changed
paths (`COMMITTED_PATH_MISMATCH`) and to equal the new commit's actual path set, and appends a
`CommitReceiptCorrection` record carrying the mandatory `--reason`; nothing is rewritten or deleted.
`correct-receipt-group` applies the same append-only repair to at least two distinct claims from one
repository. Every claim must currently name the same `--previous-commit`; the replacement commit
must exactly equal their handed-off path union, and replay deterministically reconstructs each
claim's covered leaf set. Any mismatch rejects the whole locked append.
Run a heartbeat at least every five minutes and on proof, blocker, commit, or handoff.

Handed-off claims are committed by whichever orchestrator reaches them first (codex-root or
claude-orch): that orchestrator commits the claim path-exactly and records its receipt; the other
verifies the recorded commit OID and moves on instead of committing again
(`AGENT_CHAT.md`, 2026-08-25T05:46:14Z). A claim whose `commit_oid` is already set is never staged a
second time.

Only the orchestrator commits. Worker proof is the mapped focused Cargo/npm, Rustfmt, ShellCheck, or
equivalent command in a private target; worker lanes never run `scripts/ci-local.sh` and never take
the per-repository proof-custody lock `<repo>/.git/bullet-ci.lock.d`. After exact handoffs are
integrated and the checkout is clean, the recovered sole-writer runs
`bash scripts/ci-local.sh required` and any family lane serially. Zero Git worktrees, zero pushes,
zero new remotes. A
process killed while holding it leaves every other lane refusing `CI_PROOF_LOCKED_OR_STALE` with exit
75 (`ops/ci/family-custody.sh`, `ci_proof_acquire`/`ci_proof_refusal`). The stale-lock refusal is enforced by code; keeping worker
lanes out of the lock and off the shared target is enforced by discipline.
Provider CLI execution is quarantined. `BULLET_LIVE_PROVIDERS`, provider OAuth state, and forge
tokens do not authorize a run. A signed launch-grant validator already exists (ADR 0011). Live
dispatch stays policy-disabled: committed policy is v1alpha1 / generation 1 /
`live_admission_enabled=false`. ADR 0012 ratification plus a Kernel loader mirror are operator
acts, not environment variables.
