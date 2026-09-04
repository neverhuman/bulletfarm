# Dogfood the family (operator board)

Status: Active  
Owner: Bullet Farm maintainers  
Last reviewed: 2026-08-31
Applies to: agents working the split family at the outermost `repos.manifest.toml`

## What is stopping acceleration (honest)

There are two disjoint dogfood tracks. Coordination dogfood is **N agents using
Bullet coord + Shift Brief + farmd commands as the board**. Provider dogfood is
one contained read-only proposal through a provider-specific compose. Neither
track is release evidence. The current board is **diagnostic and blocked**;
`self-hosted-v1` remains `BLOCKED`.

The single product loop that would let this family move faster is currently
**broken at the board**, not missing a fifth repo or a live provider:

1. **Machine coord cannot accept claims.** Live ledger is legacy
   `events.jsonl` without `CURRENT`. `bullet-family coord status` now returns
   `COORD_RECOVERY_REQUIRED`. Until R4/R5 finish explicit recovery and an
   orchestrator receipts it, workers race in `AGENT_CHAT.md` instead of
   `coord claim` / heartbeat / handoff. Do not chmod the 0400 ledger.
2. **The diagnostic board exists, but it is not operable.** Use
   `bullet-family check dogfood --json` as the diagnostic board. The bounded
   compatibility launcher `python3 scripts/dogfood-board.py --json` invokes
   that exact Rust command and forwards its bytes and exit status; it has no
   projection logic of its own. The current blocked board exits non-zero and is
   always `authoritative: false`. Exit 0 is reserved for an operable loop, not
   for a merely well-formed diagnostic. Its coordinator view is typed in both
   safe states: available after recovery, or unavailable with a stable error
   code before it.
3. **Claude compose exists, but no passing provider run exists.** Kernel exposes
   `bullet dogfood read-only` and the `dogfood-claude` feature. It is Claude-only,
   produces no repository mutation, and has no admitted successful receipt.
   Exit 78 is neutral, never PASS; exit 0 requires one exact proposal and its
   create-once hard-false receipt. Dogfood explicitly refuses
   `live_admission_enabled=true`. Codex, Cursor, and Antigravity compose paths
   remain unimplemented.
4. **Public `run_demo` is watchable PENDING → durable UNKNOWN.** An
   authenticated exact POST and duplicate replay survive farmd restart; the
   packaged Portal replays and polls the same command/request, a registered
   same-UID Runner claims it over `SO_PEERCRED` UDS, and the bounded exact
   worker admits the retained fixture receipt before settling
   `COMPONENT_PROOF_NOT_TRANSACTION_ELIGIBLE`. Public GET and Portal show the
   same raw-receipt BLAKE3, and worker restart reads `NO_COMMAND`. There is no
   transaction-eligible dispatch, independently admitted executor,
   `APPLIED`, or `VERIFIED`. Agents can watch the loop; they cannot ship work
   through it.
5. **The retained offline CLI bridge supplies the nested fixture receipt, but
   alone is not farmd command dispatch or transaction evidence.** The public
   wrapper binds it to the exact command/request without upgrading its evidence
   class. The bridge durably admits the ScopeGrant, uses
   peer-authenticated farmd/Runner and a Kernel-issued exact Candidate grant,
   prepares the Candidate through production Gitd, and delivers/read-backs that
   exact head through `LocalBareForge`. Inside its unsigned `COMPONENT_PROOF`,
   the harness issues purpose-separated PASETO v4.public signatures over JCS
   `VerificationIntentV1`, `EvidenceV1`, and `ProofBundleV1`, reconstructs the
   ephemeral public subjects, and reverifies the canonical chain before any
   effect. The same retained run binds exact-SHA check/proof-root read-back,
   protected expected-old-OID integration, a caller-free purpose-signed
   `ObservationV1` whose authoritative target read is `MATCHED`, and reopen
   read-back. After every child exits, the wrapper independently reopens the
   retained ordinary-Git source HEAD, Candidate HEAD/tree, and LocalBare target
   HEAD; the private SQLite ledger is retained beside those artifacts. The
   signed executor, verifier, and observer are same-process fixtures, the
   nested records are `FIXTURE_KEY_ONLY`, and every eligibility flag is hard
   false. The public binding still has no trusted key lifecycle, durable nonce
   consumption, distinct UID/credential custody, independently owned artifact
   custody, process-level response-loss hook, or twelve-boundary chaos proof.
6. **Six Portal surfaces stay UNKNOWN** (Cognitive Router, Fusion Lab, Quota,
   Struggle, Behavior, Hygiene). Do not paint them `verified`.
7. **100/100 is a different project.** OD-D tags/locks, live admission, L-24
   installed Gitd authority/tag closure, L-31 `TRANSACTION_PROOF`, stranger
   receipts, and five platforms are operator/spine work. Inventing them would
   falsify dogfood.

## Remaining build and test ladder

Do the next free row only. Skip any path that is claimed. Orchestrator-only
commits.

| Wave | What to build / test | Paths (if free) | Done when | Not done when |
| --- | --- | --- | --- | --- |
| D0 | Sanctioned coord recovery + schema-2 CURRENT | Hub `src/coord/**` (R4/R5) | independently authorized recovery preserves lineage; `coord status --json --all` exits 0; claim/heartbeat/handoff work | Fresh Genesis, ledger relocation, chmod of events.jsonl, or invented CURRENT |
| D1 | Browser proof empty hash is Shift Brief | Portal `e2e/shift-brief.spec.ts` | mocked Playwright: `/`, `#`, and `#/` → Shift Brief; unknown hash → explicit `NOT_FOUND_ROUTE`; `#/control-tower` still Control Tower; zero `.verified` | greening UNKNOWN surfaces |
| D2 | CLI diagnostic board (landed, currently blocked) | Hub `src/cli.rs` + `src/check/dogfood.rs` | blocked inputs exit non-zero; after D0 and admitted board inputs it exits 0 with `authoritative:false` while self-hosted-v1 remains BLOCKED | calling diagnostic output authority or flipping check-release |
| D3 | Leftover producers | L-15 GC (`gc.rs` / `gc_safety.rs`); L-06 cargo-fuzz+nightly only after `ops/ci` free; L-64 after L-32 receipt | focused cargo/playwright proof | inventory steal; L-64 before L-32 receipt |
| D4 | Watchable command loop | Kernel public component wrapper plus packaged Portal | authenticated exact duplicate POST → PENDING → farmd restart → packaged Portal replay/poll → same-UID UDS worker claim → retained fixture receipt → durable exact-digest UNKNOWN; worker restart reads `NO_COMMAND`; every eligibility flag stays false | emitting APPLIED/VERIFIED, claiming independent custody, or inventing process-level response-loss/chaos evidence |
| D5 | Retained offline component bridge | Kernel `just proof-transaction-offline` with an absolute digest-pinned production Gitd | exact Candidate grant/final check, one-use Candidate, fixture writer refusal + PASS, purpose-signed fixture intent/evidence/proof with reconstructed ephemeral keys, exact-head delivery/read-back, stale-fence refusal, `UNKNOWN`→`COMMITTED`, exact-SHA check/proof-root read-back, protected expected-old-OID integration, purpose-signed fixture Observation `MATCHED` and reverified, reopen read-back, plus post-exit exact source/Candidate/target Git reads and retained ledger; retained unsigned `COMPONENT_PROOF` keeps every eligibility flag false | calling the fixture-key signed chain/Observation independent or trusted, treating retained private artifacts as independent role custody, or inferring public dispatch from the D5 CLI receipt alone |
| D6 | Independent custody and evidence admission | W5 verifier, broker, attestor, integrator, observer, and auditor identities | independently registered keys, durable nonce consumption, distinct UID/credential custody, independently owned artifacts, and semantic admission preserve the already signed D5 subjects under durable claim leases | borrowing fixture keys or private harness artifacts, or changing any eligibility flag before semantic admission |
| D7 | OD-D → complete W5 → L-31 | operator tags/locks after independent verifier and effect closure | signed `TRANSACTION_PROOF` | invented LIVE_PROOF / stranger receipts / borrowed component receipt |
| D8 | Separate live four-model conformance | OD-A + general live admission | four contained conformance receipts | treating this as dogfood admission; the dogfood validator refuses `live_admission_enabled=true` |

From the `bullet-kernel` checkout, the retained component regression command is:

```bash
BULLET_GITD_BIN="${BULLET_GITD_BIN:?set exact absolute canonical daemon}" \
BULLET_GITD_SHA256="${BULLET_GITD_SHA256:?set exact lowercase SHA-256}" \
just proof-transaction-offline
```

Both variables must already name the exact canonical executable and its
lowercase SHA-256. The retained diagnostic is
`/tmp/bullet-offline-component-proof.observation-20260827T1340Z/COMPONENT_PROOF.receipt.json`
(SHA-256 `b0aeadaebc834dd20e6c8f885b48d1868c0b4005d34a05b500fd27e23b6e5eb2`).
It binds Candidate
`can_6a1825080083e84b1f6a834ba11281e13dcb5bc08ec57844531f98379e215e3a`,
base `571e25fe7171eb98d00d4477481a3223f8e45b32`, head
`c60e2d674cc27b8520e16ff4b961ade355cddc64`, tree
`618430ce7ff8883985bf50af5c074b07626ddc4d`, and proof root
`prf_9f433d2583c534e27a5a8cb853be7b42d05c31981e673f0cbf3bc6efa8f514e4`.
Its reverified verification-chain BLAKE3 is
`e562a67094f4c54d3fbf7943555df54d204500d64d4b24a9709a3269da20e216`;
its reverified signed-Observation BLAKE3 is
`571d1f416ae67d9fb50a3ba66374a82aa3beda3011fc62cbe62edc9f3b191855`,
with outcome `MATCHED`. Private relative paths retain the ordinary-Git source,
Candidate, and LocalBare target under `artifacts`, plus the ledger under
`data/ledger.sqlite`; the wrapper independently reads the three exact Git
subjects after child exit. The outer receipt remains `UNSIGNED_FIXTURE`, the
nested records remain `FIXTURE_KEY_ONLY`, and independent, transaction, and
release eligibility remain hard false. Rerun this same command after D6 lands;
do not infer independent UID/key/nonce/artifact custody, public dispatch, or
release admission from the harness-process component.

Independent review of the first dogfood slice ACCEPTed diagnostic COMPONENT
behavior and HELDs commit until D0/D1 inventories are serialized by their
owners. This runbook does not raise scorecard floors. See [`fleet.md`](fleet.md)
for claim / heartbeat / handoff / receipt. This runbook does not restate ADR 0013.

## What to look at

- Portal empty hash (`/` or `#/`) opens **Shift Brief**. Control Tower remains
  `#/control-tower`. Six surfaces without ledger subjects stay `NONE_NO_SUBJECT`.
  No row paints `verified`.
- Machine board (diagnostic only, `authoritative: false`):

```bash
bullet-family check dogfood --json
```

The Rust command composes the internal scorecard, coordinator-status, and
`self-hosted-v1` release evaluators. Release is always evaluated against a
fresh empty temporary receipt registry; this command does not accept an
admitted receipt registry. The current board is diagnostic and blocked, so it
exits non-zero. A BLOCKED release may remain diagnostic output after the loop
becomes operable, but an unavailable coordinator, dirty W0 subject, or
missing/invalid dogfood binding is a loop blocker and therefore cannot exit 0.
An unavailable coordinator exposes only its stable typed error code, never its
path-bearing detail or an ambient host `HOME`. The board never claims
`LIVE_PROOF`.

The POSIX Python script is only a timeout-bounded, byte/exit-transparent
developer compatibility launcher for that Rust command. It resolves the
explicit `BULLET_FAMILY_BIN` test hook, then an executable local debug binary,
then the locked Cargo fallback, always as an argument vector without a shell.
On timeout it sends TERM, waits one fixed grace interval, sends KILL to that
exact process group, and reaps the leader before returning 124. This covers
ordinary Cargo-owned descendants that remain in the launched group; it is
teardown hygiene, not containment or authority. The fallback is diagnostic
developer compatibility, not installed-product or release authority. The
script does not parse or reconstruct board truth.

Until `coord status --json --all` exits 0, workers still record path-exact
manual claims in `AGENT_CHAT.md`. After `CURRENT` exists, `coord claim` /
heartbeat / handoff is the board; `AGENT_CHAT.md` stays the human-decision log.

## How 8 workers run

1. Re-read `<family-root>/AGENT_CHAT.md` tail and `bullet-family coord status --json --all`.
   If coord is frozen (`COORD_RECOVERY_REQUIRED` / `COORD_IO_FAILED` / no CURRENT), do not chmod it;
   record a path-exact manual claim in `AGENT_CHAT.md` and skip any path that
   is already active there.
2. `coord claim` exact repository-relative paths only. No shared worktrees.
3. Heartbeat at least every five minutes and on proof, blocker, or handoff.
4. Hand off with the green proof command and every changed path inside the claim.
5. Only the orchestrator commits. A handed-off claim is not a second commit.

## Do not run providers

`BULLET_LIVE_PROVIDERS`, provider OAuth, and forge tokens do not authorize a
run. Do not flip live admission, invent OD-D tags, remount `/v1/leases`, or
green an UNKNOWN portal surface without a ledger subject.

The Claude-only compose is implemented, but this runbook does not authorize
invoking it. `bullet dogfood read-only` exits 0 only after one exact contained
proposal and create-once receipt; exit 78 means neutral/missing prerequisite and
is never PASS. Exit 1 is a failure. A policy with
`live_admission_enabled=true` is refused by the dogfood path, not a shortcut.

## Coordinator recovery hold

Do not run Fresh Genesis, relocate either coordinator ledger, chmod the frozen
source, invent `CURRENT`, or delete incident bytes. The current operating HOLD
requires sanctioned independent recovery and a recovery-bound clean W0 before
coordination dogfood can turn green. The family planning label “OD-L” is a
proposal pending reviewed amendment and ratification; it is not accepted by
checked-in ADR 0013 or ADR 0015 and supplies no authority today.

## Command loop (when farmd is up)

Authenticated `POST /api/v1/commands` records exact `run_demo` as `PENDING` and
returns the same command ID and request digest for an exact idempotent replay.
After farmd restart, the packaged Portal replays and polls that request while a
registered same-UID `SO_PEERCRED` UDS Runner and bounded exact worker retain the
fixture transaction receipt and atomically settle `UNKNOWN` with
`COMPONENT_PROOF_NOT_TRANSACTION_ELIGIBLE`. Public GET and the Portal command
card show the same command ID, request digest, and raw-receipt BLAKE3; worker
restart returns `NO_COMMAND`. This is `COMPONENT_PROOF` / `UNSIGNED_FIXTURE`
with nested `FIXTURE_KEY_ONLY` records and every eligibility flag hard false.
It has no process-level response-loss or twelve-boundary chaos proof.
`APPLIED` / `VERIFIED` are not a dogfood outcome.
