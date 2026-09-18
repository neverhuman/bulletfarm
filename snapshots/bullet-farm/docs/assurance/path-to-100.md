# Path to 100 — closing every gap, fairly

Status: **HISTORICAL SNAPSHOT — frozen at 2026-08-25T12:40:00Z; not current runtime, release, planning, or scoring
authority.** Commands, claims, scores, holders, and dirty-tree statements below describe that snapshot and must not
be executed or reported as current. Every executable decision requires an explicitly selected profile and
absolute receipt registry: `bullet-family check release --profile <profile> --receipts <absolute-registry>
--json`. The current release projection is [`release-truth.generated.md`](release-truth.generated.md); the active
dependency order is the [`closure roadmap`](closure-roadmap.md): `self-hosted-v1` first, `evolution-v1` in
Wave 9, breadth and later `universal-v1` in Wave 10, then `team-v1`/`saga-v1` in Wave 11. The V1 and launch plans
linked below are snapshot provenance only.
Owner at snapshot: Bullet Farm maintainers. Written against hub `5d5001d`, kernel `1dcbaa2`, git `b5c3512`, portal
`5e4b474`.
Companion pages: [`launch-plan.md`](launch-plan.md) (historical M0–M5 snapshot), [`product-gaps.md`](product-gaps.md)
(G1–G18), [`v1-closure-plan.md`](v1-closure-plan.md) (historical V1-S0…S8 snapshot),
[`../decisions/0013-operator-decision-register.md`](../decisions/0013-operator-decision-register.md) (current OD-A…OD-J register),
the capability study at family-root `.l7-bundle/EVOLUTION-2026-08-25/` (D1 understanding, D2 scorecard,
D3 beyond-Gas-Town design, D4 forge strategy, D5 lane plan). Lane IDs `L-nn` below are D5's; new lanes continue
its numbering so the two documents can be read together.

---

## 0. What "100/100, fairly" means — and what it can never mean

The number is the D2 scorecard's blended headline: **0.20 × architecture + 0.60 × implemented product + 0.20 ×
stranger-usable**, over twelve weighted dimensions, each scored twice (design as specified, implementation as
measured). Today it is **42** (D2), roughly **43** after the 2026-08-25 landings (§1; an estimate, not a re-score).

**Fair** means five rules, none negotiable:

1. **The rubric is frozen.** Twelve dimensions, the D2 weights, the 20/60/20 blend. Changing any of them needs a
   reviewed ADR and re-scores every historical point. No point on this path comes from re-weighting.
2. **Unproven capability scores as design.** A type without a caller is 0 implemented; an `#[ignore]`d test that
   CI never runs is not a proof; a doc claim with no code behind it is 0. This rule already produced 42; it
   stays.
3. **Every point has an evidence subject.** A score row moves only when it can cite a receipt id, a test name in
   a CI lane that ran, a commit OID, or a `check release` gate state. §8 makes that mechanical: the scorecard
   becomes a generated, drift-gated page like `release-truth.generated.md`.
4. **Nobody scores their own lane.** After each phase an agent from a *different provider family* re-scores from
   the evidence table alone. A disagreement over five points on any dimension is recorded as an adjudicated
   inconsistency, glossary-style, and the *lower* number stands until resolved.
5. **The score can go down.** A regression, a reverted receipt, a refuted invariant, or an expired live proof
   lowers the row. Ratchets protect audit floors, not scores.

**What 100 never means.** It is not "better than Gas Town", "faster than Omnigent", or any performance claim.
`competitor-snapshot.md` forbids benchmark, performance, and superiority comparisons until the matched corpus
(L-71) exists; its bounded source-documentation table is not such a comparison. Even after L-71, the only
performance-independent public property claim is the D3 §5 claim — every byte on `main` traceable to an exact Candidate, an
independent reproduction, an attestation by a principal that cannot write code, and a read-back receipt, or
`UNKNOWN` and stop. 100 means: **every capability the design specifies is implemented, proven where it counts,
installable by a stranger on all five platforms, and re-scored by someone who did not build it.**

---

## 1. The board this plan starts from (2026-08-25T12:40Z)

Landed since the D2 snapshot (09:37Z), all path-exact and receipted:

| Landed | Dimension moved | Evidence |
| --- | --- | --- |
| Lease reclamation at the acquisition boundary + farmd reaper tick (≤500 ms, never disableable) | 1 (crashed runner no longer blocks its Variant forever) | kernel `4effc37d`, `107c5cd5`; mutation-proved |
| `doctor` exits 3 on BLOCKED; required lane asserts exit/JSON agreement | 8 | hub `f00171ff` |
| OD-B corrected to `jeryu gh-setup --token-file` everywhere | 12 | hub `f00171ff`, `87ed67f` |
| `deny.toml` + fail-closed advisory freshness + `zizmor` in all four repos | 10 | hub `f9edeef7`, kernel `81d549de`, git `858c533b`, portal `3aa0ec69` |
| MSRV-1.95 / pinned-1.97.1 lanes | 9, 11 | hub `9a7237e4`, kernel `b5863b03`, git `c64b7f85` |
| `NoNewDispatchAfterStop` checked; STONITH `grace < TTL` in both validators | 1, 11 | hub `3728e798`, kernel `bb420f46` |
| GC-under-hostile-load proofs mutation-proven | 2 | git `54d1338b`, `b637bd3` |
| Portal embedded in farmd behind `embedded-portal`; browser suite against the packaged origin | 8, 9 | kernel `d59c5a72`, `0c10cd9e`; portal `f4b8975e` |
| Quarantined Linux x86_64 builder component (deterministic bundle, CycloneDX SBOM, checksums, unsigned provenance); archive exec-bit defect fixed; eight-binary completeness ratchet; public build still refuses | 9 | hub `a1b7ab38`, `ab1fd8fa` |
| Jankurai hub 65 / kernel 63–64 / git 69 / portal 68 (was 58/57/54/60) | 10 | hub `0cc0a681`, kernel `1dcbaa21`, git `e1b47ff1`, portal `5e4b474f` |
| Runner id: exact `run_<32hex>` or `INVALID_RUNNER_ID` exit 2 (in flight, codex-root) | 1 | claim `clm_ad5877…` |

Still exactly as D2 measured them (verified by probe at 12:40Z): `MemoryNonceLedger` is the only nonce ledger;
the three egress proofs are `#[ignore]`d and run in no CI lane; zero fuzz targets; zero `proptest!` in
bullet-git; invariants **4 enforced / 47 planned**; `ForgeEffects` has three operations; no hub `forge`
subsystem; `family.lock` is schema 2; no release workflow; no attestor, budgets, roles, or holdout crate;
`TRANSACTION_PROOF` appears in zero Rust files; `ProofRoot::compute` has one caller with four literals;
`bullet-runner` accepts `--provider sim` only; six portal surfaces are refusal strings.

**Estimated position: implemented ≈ 40, blended ≈ 43.** The gap to 100 is 57 blended points, of which roughly
36 are implementation, 11 are stranger-usability, and 1 is architecture (§2).

---

## 2. Exit criteria per dimension — what 100 requires, as evidence subjects

Each row: the D2 "A 100 requires" list turned into checkable subjects, the lanes that produce them, and the
class of proof that is admissible. **I** = today's implemented score.

| # | Dimension (weight) | I | Exit criteria for I = 100 (each must have a receipt or CI-run test) | Lanes |
| --- | --- | ---: | --- | --- |
| 1 | Concurrency and authority kernel (14) | 64 | Durable nonce ledger with issue/consume separated (table + property test). Server-side reaper with a **liveness** property in TLC (`WF_vars(Reap)` ⇒ ◇ no expired holder) and `CHECK_DEADLOCK TRUE` on both models. Signed, versioned `AuthorityToken` end to end (kernel ↔ gitd), no literal digests in `token_for`. Real authority-epoch and freeze-generation counters (no `= 1` / `= 0` constants). `advance` implemented on both ends of the lease transport. Two-process race test: exactly one writer per Variant across processes. | L-05, L-13, L-20, L-21, L-23, L-24, L-60 |
| 2 | Isolation and repository safety (11) | 78 | Three egress proofs run in a CI lane on a capable runner every push. Admitted gates execute inside the egress sandbox (`build.rs` cannot reach the network — negative test). Reflink fast path with a fallback proof. Trust-aware GC: retention classes and object tombstones, with the hostile-GC suite extended to prove tombstoned objects survive. | L-33, L-34, L-14, L-15 |
| 3 | Evidence and verification integrity (12) | 28 | `ProofRoot` populated over the eight `git_role.md:281` inputs, recomputed and **verified** on every read, reachable via RPC; a tamper test flips each input. Holdout custody service: sealed suite, query budgets (failed query consumes budget), exposure edges, contamination decisions — with the N4 build-failure test. Mutation testing with a threshold gate (per repo, ratcheted). Fuzz targets on `decode_canonical_value` and the patch applier, run nightly with corpus checked in. Hash-chained ledger (`prev_hash` column) with externally anchored batches and a verifier. Verifier: per-gate executable digest, multi-gate aggregation, oracle-modifying-diff classification. | L-25, L-53, L-16, L-06, L-61, L-27 |
| 4 | Integration and delivery authority (11) | 26 | Attestor binary with its own credential, exact-SHA read-back, `CHECK_SUBJECT_MISMATCH` fail-closed; negative tests: broker cannot post a check, attestor cannot push. Five missing port operations (protected-ref, check-run, merge-group, PR, integration-subject) with honest per-adapter refusals. Merge-group verification bound to the group head SHA against a forge that has merge groups (Jeryu after J-3, GitHub). Tree-disjoint evidence preservation (A6) with a livelock test. Real `JeryuForge` (authenticated push, expected-old OID, server read-back) and a real GitHub adapter, both with `LIVE_PROOF` receipts. Broker is a daemon, not a demo `main`. | L-28, L-12, L-57, L-45, L-46, L-29, J-1…J-4 |
| 5 | Identity, quota and cost governance (8) | 8 | `crates/budgets` + `0017_reservations.sql`: atomic dual-tree reservation/settlement with unknown-liability retention and a conservation property test. The 8-level typed observation ladder with `unknown` never scheduled as headroom (negative test). Probe reservation with its own budget, non-reusable for strategy exploration. One-seat-per-human rate governance for `named_human_subscription` profiles (negative test: two profiles, one owner, one seat of throughput). `LIVE_PROOF`: real usage settling against real actuals for all four providers. | L-22, L-55, L-62, L-44 |
| 6 | Multi-agent collaboration and roles (8) | 6 | `crates/roles`: `RoleSpec` runtime, typed artifact edges, conflict graph validated at registration **and** dispatch. T0–T6 topologies as reviewed data with caps. Real router: eligibility filters, deterministic rule routing, abstention to T0 (no `lane_for` constant). Fusion runtime: typed protocols, `FusionReport`, 8-term `diversity_score`, rank-select short-circuit, forced-synthesis penalty; the four "must fail if they pass" tests from D3 C1. **At least two providers dispatching through the router with `LIVE_PROOF` receipts**, and one `TRANSACTION_PROOF` from a real Selection Group producing two exact Candidates. | L-50, L-51, L-52, L-54, L-44 |
| 7 | Evolutionary optimization (5) | 2 | Everything in D3 C2, in order: feasibility shield (deterministic, before spend, negative-knowledge retention), lexicographic objective over complete `AggregateEvaluationV1`, bounded MOME archive with four frozen axes, B0–B6 promotion service **separate from the optimizer**, ASHA inside B1/B2 only, drift rollback restoring the incumbent. The five falsification tests from D3 C2. Started only after every gate in D3 §3 C2 is green, by operator ADR + policy generation bump (OD-H). `evolutionary_authority=true` stays `UNSAFE_POLICY` until then. | L-70a…e, L-71, OD-H |
| 8 | Operator truth and UX (7) | 48 | All fifteen surfaces render durable subjects with provenance headers (no refusal strings, no `<pre>` JSON). Two-stage freeze chip (`recorded` → `enforced N/M, k unreachable, leases expire in Ns`) driven by a real freeze table. Saga quarantine of blast radius, not fleet (multi-repo fault test). Shift Brief as the portal's default screen, every line opening to the exact unproved claim. `doctor`, `check release`, and the portal never disagree (drift test). | L-58, L-63, L-60, L-64, L-59 |
| 9 | Installability and release engineering (8) | 16 | Schema-3 lock generated from signed member tags (OD-D). A different-identity exact-OID broker builds all five targets; each archive is signed (OD-E), semantically admitted with both SBOM formats and provenance, and published by a tag-triggered workflow whose write permission is scoped to the release job. A signed prebuilt `bullet-family` performs setup twice on each platform; systemd lifecycle/recovery passes on Linux and unsupported mutation platforms refuse. Five archives pass release smoke; the complete `universal-v1` gate set is green. | OD-D, L-40, L-41, L-42, L-43, L-65, L-66, L-49 |
| 10 | Security posture (7) | 58 | Jankurai ≥ 90 in all four repos with **zero caps and zero hard findings**, floors ratcheted, hosted half running on the pinned portable auditor. Signed tokens on both paths (from dim 1). Chained, anchored audit log (from dim 3). Fuzz in CI nightly. **Independent external security review** with findings closed or accepted by ADR. Secrets never in argv/env/logs (existing tests kept). | L-47 ×4, L-48, L-68, L-06, L-61 |
| 11 | Test and assurance depth (6) | 57 | Fuzz and mutation thresholds in CI. Liveness/fairness properties checked in both TLC models, state counts re-pinned. Invariant registry **51/51 `enforced`**, each bound to a named test or typed refusal in a CI lane, with a ratchet that fails if the count falls. Egress proofs in CI. Fault suite (R-08) tagged and run: every crash boundary, none may become PASS. Two-process and multi-machine race tests. | L-06, L-16, L-13, L-35 + L-36…L-39, L-33, L-32 |
| 12 | Documentation honesty (3) | 86 | Symbol-anchored citations checked by a resolver (a cited `path::symbol` must exist at HEAD). Doc-freshness gate: "reviewed against HEAD X" fails when X is not an ancestor within N commits. No false contract claims (the "signed Jeryu tags" wording becomes true only after J-5, and stays corrected until then). `TEAM.md`/`POTENTIAL_DRAFT.md` C8 "two providers" reconciled to the frozen four. | L-69, L-08b, L-02, L-02b |
| — | **Architecture** (20 % of blend) | 94.5 | The two D2 deductions closed: the paper and the frozen contract agree on four providers (docs); the default forge (LocalJeryu) gains merge groups, protected refs with expected-old-OID, and exact-SHA check runs so the design is achievable at full strength on its own default deployment. | L-02b, J-1…J-5 |
| — | **Stranger-usable** (20 % of blend) | 3 | A person with no access to this box or its agents follows the README from a tagged release: installs twice, runs `doctor` (exit 0), completes the offline transaction demo, and — with their own provider login — one live conformance turn, on each of the five platforms. Recorded as a **stranger-trial receipt** signed by the trial runner, one per platform, repeated at each release. | L-66, L-67 |

Each row's exit criteria map one-to-one onto rows of the generated scorecard (§8), so "closing a gap" and
"moving the score" are the same act.

---

## 3. The phases

Five phases. Each exits on **receipts, not dates**; the week numbers are planning estimates under the
parallelism in §5 and assume the operator acts named in §6 land when they are first needed. Lane columns:
repo · paths (claimed path-exactly through `bullet-family coord`), predecessor, proof command (the command whose
exit code is logged at handoff), size (XS/S/M/L/XL), class (`AGENT` on-box · `OPERATOR` · `EXTERNAL` · `JERYU`
in the jeryu-split family), and the dimension row it moves with the target score after the lane.

### P0 — Honest instruments and the Wave-1 remainder (weeks 0–1; ≈ 43 → 47)

Goal: make the score itself falsifiable, then close every zero-predecessor defect the scorecard named.
All lanes are path-disjoint from the claims active at 12:40Z (`README-CI-FAMILY-R1-R3` in the hub;
`KERNEL-CI-PROTECTED-CONTEXT-R1` on kernel `ci.yml`; `RUNNER-ID-REFUSAL-R1`).

| Lane | Goal | Repo · paths | Pred | Proof | Size | Class | Dim → |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **S-00 `HUB-SCORECARD-GENERATED`** | The rubric as data: `policy/scorecard-v1.json` (12 dimensions, D2 weights, one row per §2 exit criterion, each with an evidence-subject kind: `receipt`, `ci-test`, `gate`, `commit`, `external-review`). `bullet-family check scorecard --json` scores each row as `min(claimed, evidence-admitted)`; rows with no admitted subject cap at the design score. Renders `docs/assurance/scorecard.generated.md`, drift-gated by digest in `required` like release-truth. | farm · `policy/scorecard-v1.json` (new), `src/check/scorecard/**` (new), `src/cli.rs`, `tests/scorecard.rs` (new), `docs/assurance/scorecard.generated.md` (new), `Justfile` (`scorecard` recipe; after `README-CI` hands it off) | — | `cargo test --locked -p bullet-family --test scorecard; echo EXIT=$?` | M | AGENT | §8 |
| **S-01 `INDEPENDENT-RESCORE-P0`** | A Codex- or Cursor-family agent re-scores every row from `scorecard.generated.md`'s evidence column only, without reading this plan. Disagreements > 5 recorded in `docs/glossary.md` "Adjudicated inconsistencies". | farm · `docs/glossary.md` (adjudication rows only) | S-00 | `just scorecard; echo EXIT=$?` | S | AGENT (other family) | §8 |
| **L-02 / L-02b `FAMILY-CONTRACT-TRUTH`** | Verify the "signed Jeryu tags" correction landed everywhere (probe at 12:40Z found no residue in root `AGENTS.md`/`README.md`/ADR 0002 — confirm kernel and git `SPLIT.md`); reconcile C8 "any two providers" in `TEAM.md` and `POTENTIAL_DRAFT.md` to the frozen four. | root docs (not a repo) · kernel `SPLIT.md`, git `SPLIT.md` | — | `grep -rn "signed.*tag" …; bash ops/ci/fast.sh; echo EXIT=$?` | S | AGENT | 12 → 90 |
| **L-05 `KERNEL-DURABLE-NONCE-LEDGER`** | `0012_nonce_ledger.sql`; `NonceLedger` port with `issue` and `consume` as separate operations; verification never registers. Replay across two processes refused (`NONCE_CONSUMED`). | kernel · `crates/application/src/lease_transport/nonce.rs` (new), `db/migrations/0012_nonce_ledger.sql` (new), `crates/adapters/src/sqlite/nonces.rs` (new), `crates/application/tests/nonce_ledger.rs` (new) | — | `cargo test --locked -p bullet-application -- nonce; echo EXIT=$?` | M | AGENT | 1 → 70 |
| **L-06 `FAMILY-FUZZ-TARGETS`** | First fuzz targets: `decode_canonical_value` (hub) and the patch applier + `.git/config` parser (git). Seed corpus committed; 100 k runs in a nightly lane; crashes are typed refusals, never panics. | farm · `crates/bullet-wire/fuzz/**` (new), `ops/ci/fuzz.sh` (new, after `README-CI` hand-off of `ops/ci`); git · `crates/bullet-git-workspace/fuzz/**` (new), `ops/ci/fuzz.sh` (new) | — | `cargo +nightly fuzz run canonical -- -runs=100000; echo EXIT=$?` | M | AGENT | 3 → 32, 11 → 62 |
| **L-07 `GIT-PROPTEST-FLOOR`** | Property tests for scope coverage, journal chaining, generation lineage, canonical-ID field sensitivity. | git · `crates/bullet-git-workspace/tests/properties.rs` (new), `crates/bullet-git-journal/tests/properties.rs` (new), `crates/bullet-git-types/tests/properties.rs` (new) | — | `cargo test --locked -p bullet-git-workspace --test properties; echo EXIT=$?` | M | AGENT | 11 → 65 |
| **L-08 / L-08b `DOC-FRESHNESS-GATE`** | Fix the pessimistic "reviewed against HEAD `ca380bc`" claims; add a checker that fails when a doc's reviewed-HEAD is not an ancestor of HEAD within 25 commits or names a route/file that does not exist. | kernel · `README.md`, `docs/cli.md`, `ops/ci/docs-freshness.sh` (new); farm · `tests/docs_freshness.rs` (new) | — | `bash ops/ci/docs-freshness.sh; echo EXIT=$?` | S | AGENT | 12 → 93 |
| **L-10 `HUB-LOCK-VALIDATION`** | Lock-generation validation (D4 §6.2) and the seven negative tests (D4 §6.3), incl. `html_content_type_is_never_a_capability` and `unsigned_tag_is_refused`. | farm · `src/family_lock.rs`, `tests/family_lock.rs` | — | `cargo test --locked -p bullet-family --test family_lock; echo EXIT=$?` | S | AGENT | 9 (enables L-40) |
| **L-12 `KERNEL-FORGE-INTEGRATION-PORT`** | Extend `ForgeEffects` with protected-ref, check-run, merge-group, PR, integration-subject operations and six typed refusals; `LocalBareForge` answers honestly (`Unsupported` where true); `JeryuForge` still refuses live. | kernel · `crates/effects/src/integration.rs` (new), `crates/effects/src/{error,local,jeryu,lib}.rs`, `crates/effects/tests/forge_conformance.rs` (new) | — | `cargo test --locked -p bullet-effects; echo EXIT=$?` | M | AGENT | 4 → 32 |
| **L-13 `FORMAL-LIVENESS`** | Add a `Reap` action with weak fairness to `LeaseFence.tla`; property `ExpiredHolderEventuallyReclaimed`; `CHECK_DEADLOCK TRUE` in both `.cfg`s with an explicit terminal-state predicate; re-pin state counts in `model-lock.json`; replay the new traces against SQLite. | farm · `formal/{LeaseFence.tla,LeaseFence.cfg,EffectCheck.cfg,model-lock.json,README.md}`, `tests/formal_traces.rs` | — | `bash scripts/model-check.sh; echo EXIT=$?` | M | AGENT | 1 → 74, 11 → 68 |
| **L-16a `MUTATION-BASELINE`** | Run `cargo-mutants` per Rust repo on the authority, effects, and canonicalization crates; commit the baseline report and a `mutants.toml`; no threshold yet (P2 sets it). | farm/kernel/git · `mutants.toml` (new), `ops/ci/mutants.sh` (new), `docs/testing.md` (after hand-off) | — | `bash ops/ci/mutants.sh --baseline; echo EXIT=$?` | M | AGENT | 11 → 70 |
| **L-33 `KERNEL-EGRESS-CI`** | A dedicated `egress.yml` on a runner with `unshare`, `slirp4netns`, `nft`; runs the three `#[ignore]`d sandbox proofs on every push; the lane observation feeds the `required` aggregator (coordinate: `ci.yml` is claimed by `codex-kernel-ci-integrator`). | kernel · `.github/workflows/egress.yml` (new), `ops/ci/egress.sh` | — | `bash ops/ci/egress.sh; echo EXIT=$?` | M | AGENT | 2 → 88 |
| **L-35 `HUB-INVARIANT-RATCHET`** | Lifecycle report; a ratchet test that fails if `enforced` falls below the committed floor (4 today); each enforced row must name its test or refusal and a proof lane that runs it. | farm · `policy/v1alpha1/invariant-registry.json`, `docs/assurance/invariant-registry.md`, `crates/bullet-wire/tests/policy_registry.rs` | — | `cargo test --locked -p bullet-wire --test policy_registry; echo EXIT=$?` | M | AGENT | 11 (enables L-36…L-39) |
| **L-47a `JANKURAI-75`** | Next hygiene step per repo (65/63/69/68 → ≥ 75), caps only from real defects, floors ratcheted in `ops/ci/audit.sh` once the CI lanes hand off. | all four · `.jankurai/`, per-finding paths | — | `bash ops/ci/audit.sh; echo EXIT=$?` | M | AGENT | 10 → 62 |
| **OD-D** | Publish new signed immutable member tags (see §6). | operator | — | `bullet-family lock verify --tag <tag>` | hours | OPERATOR | 9 → 30 |

P0 exits when: `scorecard.generated.md` exists and is drift-gated; S-01's independent number is recorded;
every lane above is receipted; OD-D's tags are verifiable. **If OD-D has not landed by the end of P0, P1's
release-preparation half (L-40/L-42; L-43 remains P2-blocked) and its authority half (L-24 onward) both stall —
there is no agent workaround.**

### P1 — M1 + M2: the product exists (weeks 1–6; ≈ 47 → 60)

Goal: `bullet-family check release --profile universal-v1 --receipts <admitted-absolute-registry> --json`
shows `installable-lock` and `transaction-demo` receipted. The
kernel spine is serial by construction (each lane extends the previous lane's tables and types); the hub,
git, and portal lanes run alongside it.

| Lane | Goal | Repo · paths | Pred | Proof | Size | Class | Dim → |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **L-20 `KERNEL-SIGNED-LEASE-PROMOTE`** (R-07) | Promote the signed UDS RPC/client from component proof. Landed: registered Runner ID/epoch ↔ `SO_PEERCRED` UID binding, farmd UID plus socket GID/device/inode pinning across connect, and durable server grant/nonce state. Remaining: operator-admitted durable peer registry and signing-key custody, durable client acquire/read-back recovery instead of process-local metadata, product Runner wiring, and connected lost-response proof. | kernel · `crates/application/src/lease_transport.rs`, `crates/harness-core/src/lease_transport.rs`, `apps/bullet-farmd/src/lease_transport_rpc.rs`, `crates/runner/src/signed_lease_rpc.rs` | L-05 | `bash scripts/ci-local.sh required; echo EXIT=$?` | L | AGENT | 1 → 80 |
| **L-21 `KERNEL-AUTHORITY-NORMALIZE`** (V1-S2-a) | Normalized authority rows (graph revision, workspace generation, scope digest, policy/routing generation, authority epoch, freeze generation); no JSON-blob authority; no `INSERT OR REPLACE`; retire the `= 1` / `= 0` constants; `token_for` digests computed from real subjects. | kernel · `crates/adapters/src/sqlite/leases/**`, `db/migrations/0013_normalized_authority.sql` (new), `crates/application/src/leases.rs`, `crates/domain/src/authority.rs` | L-20 | `cargo test --locked -p bullet-adapters; echo EXIT=$?` | L | AGENT | 1 → 86 |
| **L-22 `KERNEL-BUDGETS`** (WI-07) | `crates/budgets` + `0014_reservations.sql`: reservation → settlement, dual-tree atomicity, unknown-liability retention, conservation property test; `budget_reservation_id` becomes a row. | kernel · `crates/budgets/**` (new), `db/migrations/0014_reservations.sql` (new), `Cargo.toml` (member) | L-21 | `cargo test --locked -p bullet-budgets; echo EXIT=$?` | L | AGENT | 5 → 40 |
| **L-23 `KERNEL-MUTATION-CAPABILITY`** (V1-S2-b) | Short-lived PASETO mutation permits minted from the durable active lease; `SignedMutationPermitV1` gets its first use site. | kernel · `crates/application/src/mutation_permit/**` (new), `crates/harness-core/src/admission/**` | L-22 | `cargo test --locked -p bullet-application -- permit; echo EXIT=$?` | M | AGENT | 1 → 88 |
| **L-24 `GIT-PRODUCTION-CHECKER`** (G4) | Replace `AuthorityGateway::unavailable` with `FinalAuthorityCheck` bound to the schema-3 lock's `bullet-wire` tag; close the forgeable unsigned `WireAuthorityToken` in the same change (signed, versioned, private constructor). | git · `crates/bullet-gitd/src/authority_gateway.rs`, `crates/bullet-git-types/src/authority.rs`, `crates/bullet-gitd/tests/authority_live.rs` (new) | L-23, **OD-D** | `bash scripts/ci-local.sh required; echo EXIT=$?` | L | AGENT | 1 → 90, 10 → 68 |
| **L-25 `GIT-PROOF-ROOT-REAL`** | Populate the proof root over the eight `git_role.md:281` inputs; `bind_proof` reachable via RPC; `verify_proof_root` recomputes on read; a tamper test per input. | git · `crates/bullet-git-types/src/change.rs`, `crates/bullet-gitd/src/lib.rs`, `crates/bullet-git-types/tests/proof_root.rs` (new) | L-24 | `cargo test --locked -p bullet-git-types -- proof_root; echo EXIT=$?` | M | AGENT | 3 → 45 |
| **L-26 `KERNEL-RUNNER-SAGA`** (V1-S4-a) | The real saga: acquire → read-only provider → `PatchProposal` → gitd apply → admitted gates → ≤ 2 repairs → checkpoint → exact Candidate; heartbeat miss freezes, kills the tree, preserves; successor resumes from the checkpoint. `--provider` accepts the four real adapters (still refused without a grant). | kernel · `crates/runner/src/attempt.rs`, `crates/runner/src/saga/**` (new), `apps/bullet-runner/src/main.rs` (after `RUNNER-ID-REFUSAL-R1` receipts) | L-24 | `cargo test --locked -p bullet-runner -- saga; echo EXIT=$?` | XL | AGENT | 1 → 92, 6 → 12 |
| **L-32 `KERNEL-FAULT-SUITE`** (R-08) | Tagged crash-boundary suite alongside L-26: provider crash/cancel/timeout, missed heartbeat, surviving grandchild, stale fence, zero admitted tests, writer-modified oracle, lost effect response, conflicting remote OID, cleanup-before-preservation. None may become PASS. | kernel · `crates/runner/tests/faults/**` (new), `ops/ci/faults.sh` (new) | L-26 | `bash ops/ci/faults.sh; echo EXIT=$?` | L | AGENT | 11 → 76 |
| **L-27 `KERNEL-VERIFIER-REAL`** (V1-S4-c) | Per-gate executable digest, multi-gate aggregation, oracle-modifying-diff classification, `ZERO_TESTS`/`INFRA_ERROR`/`TIMED_OUT` never PASS (kept), writer identity refused (kept). | kernel · `crates/verifier/**`, `apps/bullet-verifier/src/main.rs` | L-25 | `cargo test --locked -p bullet-verifier; echo EXIT=$?` | L | AGENT | 3 → 55 |
| **L-28 `KERNEL-ATTESTOR`** (WI-12) | The missing principal: `apps/bullet-attestor` with its own credential, recomputes the verdict from the signed proof bundle, publishes a check for one exact SHA, `CHECK_SUBJECT_MISMATCH` on read-back drift. Negative tests: cannot push, broker cannot attest. | kernel · `apps/bullet-attestor/**` (new), `crates/effects/src/attest.rs` (new), `Cargo.toml` (member) | L-27, L-12 | `cargo test --locked -p bullet-effects -- attest; echo EXIT=$?` | L | AGENT | 4 → 50 |
| **L-29 `KERNEL-EFFECT-READBACK`** (V1-S4-d) | intent → dispatch → lost response = `UNKNOWN` → read-back → identity-exact adoption or quarantine; the broker becomes a daemon with a durable queue, not a `tempdir` demo. | kernel · `crates/effects/src/broker.rs`, `crates/effects/src/lost.rs`, `apps/bullet-effects/src/main.rs` | L-28 | `cargo test --locked -p bullet-effects -- reconcile; echo EXIT=$?` | M | AGENT | 4 → 58 |
| **L-30 `KERNEL-FARMD-SIGNED-DISPATCH`** (V1-S5-a) | Signed dispatch so a public command leaves `PENDING`; `EXECUTION_ADAPTER_UNAVAILABLE` becomes a real settlement path. | kernel · `apps/bullet-farmd/src/commands.rs`, `apps/bullet-farmd/src/dispatch.rs` (new) | L-26 | `cargo test --locked -p bullet-farmd -- dispatch; echo EXIT=$?` | M | AGENT | 8 → 56 |
| **L-31 `KERNEL-TRANSACTION-PROOF`** (R-04, G2, **M2**) | Replace `just demo`'s synthetic success with the signed five-plane `TRANSACTION_PROOF`; `bullet transaction --json` emits the receipt; hub `check release` admits it for `transaction-demo`. | kernel · `apps/bullet/src/transaction/**` (new), `apps/bullet/src/main.rs`; farm · `src/check/prerequisites.rs`, `src/check/truth/rows/*.rs` | L-29, L-30 | `cargo run --locked -p bullet -- transaction --json; echo EXIT=$?` | XL | AGENT | 1/3/4 (+10 blended) |
| **L-36 `INVARIANTS-BATCH-1`** | Move the authority/lease/fence invariants (≈ 14 rows) from `planned` to `enforced`, each bound to a test from L-05/L-13/L-20/L-21/L-32. | farm · registry + crosswalk; kernel tests as named | L-35, L-32 | `cargo test --locked -p bullet-wire --test policy_registry; echo EXIT=$?` | M | AGENT | 11 → 80 |
| **L-11 `HUB-FORGE-SUBSYSTEM`** | `bullet-family forge probe\|pin\|status` with the `text/html` SPA-fallthrough guard; read-only against 127.0.0.1:8787. | farm · `src/forge/**` (new), `src/cli.rs`, `tests/forge.rs` (new) | L-10 | `cargo test --locked -p bullet-family --test forge; echo EXIT=$?` | M | AGENT | 9 → 35 |
| **L-40 `HUB-LOCK-GENERATE`** | Schema-3 lock from the signed tags: `jeryu_url`, `jeryu_slug`, `tree_oid`, `lockfile`, `artifact` per member; `required` stops asserting the refusal. | farm · `family.lock`, `src/family_lock/**`, `ops/ci/required.sh` (after hand-off) | OD-D, L-10 | `bullet-family lock verify --tag <tag>; echo EXIT=$?` | M | AGENT | 9 → 40 |
| **L-42 `HUB-RELEASE-WORKFLOW-PREP`** | Prepare a cache-free, non-publishing build/verification diagnostic for the quarantined Linux component. It has no tag trigger, release creation, package-byte upload, or write permission; signed publication remains L-42b after OD-E and the five-target matrix. | farm · future release workflow source | L-40 | workflow policy, zizmor, and source-bound hostile tests | M | AGENT | 9 → 43 |
| **L-43 `HUB-SETUP-BOOTSTRAP`** | `just setup` runs only a separately admitted signed prebuilt `bullet-family` whose package manifest, checksum, signature, and schema-3 lock were verified outside the source family; source-built bytes never self-admit. Replay the two-run component on this host; clean-host lifecycle and release evidence remain L-66. | farm · `scripts/setup.sh`, `src/setup/**`, `ops/ci/required.sh` | L-40, L-41b | `bash ops/ci/required.sh; echo EXIT=$?` | L | AGENT | 9 → 45, stranger → 25 |
| **L-14 `GIT-REFLINK`** | Reflink fast path for private clones on reflink-capable filesystems with a byte-identical fallback proof. | git · `crates/bullet-git-workspace/src/clone.rs`, `tests/reflink.rs` (new) | — | `cargo test --locked -p bullet-git-workspace --test reflink; echo EXIT=$?` | M | AGENT | 2 → 90 |
| **L-34 `KERNEL-GATES-IN-SANDBOX`** (WI-32) | Admitted gates run inside the egress sandbox; a `build.rs` that opens a socket is a failing negative test. | kernel · `crates/harness-egress/src/gate_exec.rs` (new), `crates/runner/src/gates.rs` | L-33, L-26 | `bash ops/ci/egress.sh; echo EXIT=$?` | M | AGENT | 2 → 94 |
| **S-01 `INDEPENDENT-RESCORE-P1`** | Re-score as in P0. | farm · glossary adjudication rows | all of P1 | `just scorecard` | S | AGENT (other family) | §8 |

P1 exits when: `transaction-demo` and `installable-lock` are admitted; the fault suite runs in CI; the
independent re-score is recorded. The former blended ≈60 (implemented ≈60, stranger ≈25) P1 projection is not
admissible: L-43 is dependency-indexed above but cannot start or contribute its projected score in P1 because
its signed-prebuilt predecessor L-41b is a P2 release-custody lane. Re-score only from completed receipts.

### P2 — M3 + M4: live and packaged (weeks 6–10; ≈ 60 → 73)

Goal: four provider receipts, two forge receipts, a signed Linux package a stranger installs twice.

| Lane | Goal | Repo · paths | Pred | Proof | Size | Class | Dim → |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **OD-A / OD-B / OD-C / OD-E / OD-F** | See §6. Needed at the start of P2. | operator | — | — | hours | OPERATOR | 9, 4, 5 |
| **L-44 `LIVE-PROVIDER-CONFORMANCE`** | One ≤ 1-turn conformance turn per provider under the ratified generation-2 policy, inside the egress sandbox, under a 15-s grant; receipts admitted for all four `release.provider.*` gates; real usage settles against L-22 reservations. | kernel · `ops/ci/nightly.sh` receipts, `crates/budgets` settlement | OD-A, L-31, L-22 | `BULLET_LIVE_REAL=1 bash ops/ci/nightly.sh; echo EXIT=$?` | M | AGENT | 5 → 55, 6 → 20 |
| **L-45 `FORGE-JERYU-LIVE`** | Real `JeryuForge`: authenticated push under the OD-B credentials, expected-old OID, server read-back, reconciliation; `release.forge.jeryu` admitted. Read-only probes first; the running 8787 instance is never modified. | kernel · `crates/effects/src/jeryu.rs`, `crates/effects/tests/jeryu_live.rs` (new) | OD-B, L-12, L-29 | `cargo test --locked -p bullet-effects -- jeryu_live; echo EXIT=$?` | L | AGENT | 4 → 70 |
| **L-46 `FORGE-GITHUB-ADAPTER`** (R-17) | GitHub App adapter: PR, check-run (attestor credential), merge-group subject, protected-ref read-back; `release.forge.github-app` admitted. | kernel · `crates/effects/src/github.rs` (new), `crates/effects/tests/github_live.rs` (new) | OD-C, L-12, L-28 | `cargo test --locked -p bullet-effects -- github; echo EXIT=$?` | L | AGENT | 4 → 82 |
| **L-41b `HUB-RELEASE-SIGN`** | Detached signatures over the Linux bundle with the OD-E key; signer policy; `allowed_signers`; root descriptor admission for `rust-msrv-1-95`. `release verify` returns typed refusals (not `COORD_IO_FAILED`) for unsigned bundles. | farm · `src/release/sign/**` (new), `src/release/verify.rs`, `release/allowed_signers` | OD-E, L-42 | `bullet-family release verify --bundle <path>; echo EXIT=$?` | M | AGENT | 9 → 55 |
| **L-66 `INSTALL-TWICE-LINUX`** | Two-run install from tagged, signed bytes on a clean Ubuntu 24.04 x86_64 VM; systemd lifecycle and recovery; unsupported-platform refusal; receipt admitted for the `linux-preview` profile. | farm · `docs/runbooks/install.md`, `ops/ci/install-twice.sh` (new) | L-41b, L-43 | `bash ops/ci/install-twice.sh; echo EXIT=$?` | M | AGENT | 9 → 65 |
| **L-67 `STRANGER-TRIAL-LINUX`** | A person with no access to this box or its coordination log follows only the README from the tagged release: install twice, `doctor` exit 0, offline transaction demo, one live turn with their own login. Signed stranger-trial receipt; every friction point becomes a docs or product defect with a lane. | farm · `docs/assurance/stranger-trials/<date>-linux.md` (new) | L-66 | receipt present and admitted by `check scorecard` | S | EXTERNAL (a human) | stranger → 60 |
| **L-47b `JANKURAI-90`** ×4 | ≥ 90, zero caps, zero hard findings in each repo; floors ratcheted; two principled-reversion classes stay documented. | all four · `.jankurai/`, per-finding paths | L-47a | `bash ops/ci/audit.sh; echo EXIT=$?` | L | AGENT | 10 → 78 |
| **L-48 `EXT-JANKURAI-ARTIFACT`** | Checksum-pinned portable auditor so hosted CI runs the audit lane. | — | — | hosted `audit` lane green | M | EXTERNAL | 10 → 84 |
| **L-61 `KERNEL-AUDIT-ANCHOR`** (WI-15) | `prev_hash` chaining on `events`; batch roots anchored to an external append-only subject (the forge's protected ref); `bullet audit verify` walks the chain. | kernel · `crates/application/src/audit/**` (new), `db/migrations/0015_audit_chain.sql` (new) | L-21 | `cargo test --locked -p bullet-application -- audit_chain; echo EXIT=$?` | L | AGENT | 3 → 62, 10 → 86 |
| **L-60 `KERNEL-FREEZE-TWO-STAGE`** (WI-11, C11) | Freeze table; `recorded` → `enforced N/M runners, k unreachable, leases expire in Ns`; real freeze generation counter feeds L-21's column. | kernel · `db/migrations/0016_freeze.sql` (new), `crates/application/src/freeze.rs` (new); portal · `src/surfaces.ts`, `src/pages/Freeze.tsx` (new) | L-21 | `cargo test --locked -p bullet-application -- freeze; echo EXIT=$?` | M | AGENT | 8 → 64, 1 → 94 |
| **L-64 `KERNEL-SAGA-QUARANTINE`** (C12) | Multi-repo `MANUAL_RECOVERY_REQUIRED` quarantines the saga's blast radius; a fault test proves an unrelated repository keeps processing. | kernel · `crates/application/src/saga_quarantine.rs` (new), `crates/runner/tests/faults/quarantine.rs` (new) | L-32 | `bash ops/ci/faults.sh; echo EXIT=$?` | M | AGENT | 8 → 70 |
| **L-15 `GIT-GC-RETENTION`** | Trust-aware GC: retention classes, object tombstones; the hostile-GC suite proves tombstoned objects survive `gc --prune=now`. | git · `crates/bullet-git-workspace/src/gc.rs` (new), `tests/gc_safety.rs` | — | `cargo test --locked -p bullet-git-workspace --test gc_safety; echo EXIT=$?` | M | AGENT | 2 → 98 |
| **L-16b `MUTATION-THRESHOLD`** | Mutation score threshold per crate from the P0 baseline, ratcheted; the lane fails below it. | farm/kernel/git · `mutants.toml`, `ops/ci/mutants.sh` | L-16a, L-32 | `bash ops/ci/mutants.sh; echo EXIT=$?` | M | AGENT | 3 → 66, 11 → 84 |
| **L-37 `INVARIANTS-BATCH-2`** | Effects, evidence, verification invariants (≈ 12 rows) → `enforced`. | farm registry; kernel/git tests | L-36, L-29, L-27 | registry test | M | AGENT | 11 → 86 |
| **S-01 `INDEPENDENT-RESCORE-P2`** | Re-score. | — | all of P2 | `just scorecard` | S | AGENT (other family) | §8 |

P2 exits when: all six live gates are admitted; the Linux stranger receipt exists; Jankurai ≥ 90 ×4.
Expected blended ≈ 73 (implemented ≈ 70, stranger ≈ 60).

### P3 — The capability leap and the forge that unlocks it (weeks 10–18; ≈ 73 → 86)

Goal: everything D3 places in V1.1, plus the Jeryu release that closes the architecture deduction. None of
this starts before L-31's signed `TRANSACTION_PROOF` (normative in `evolutionary-control.md`).

| Lane | Goal | Repo · paths | Pred | Proof | Size | Class | Dim → |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **L-50 `KERNEL-ROLES`** (WI-17) | `RoleSpec` runtime, typed artifact edges, conflict graph at registration and dispatch; N11 negative test (a `RoleSpec` cannot carry a credential or evidence floor). | kernel · `crates/roles/**` (new) | L-31 | `cargo test --locked -p bullet-roles; echo EXIT=$?` | L | AGENT | 6 → 40 |
| **L-51 `HUB-TOPOLOGIES`** (WI-18) | T0–T6 as reviewed data with per-topology caps and certified strata; catalog set-equality check. | farm · `policy/v1alpha1/topologies.json` (new), `crates/bullet-wire/src/catalog.rs` | L-50 | `cargo test --locked -p bullet-wire; echo EXIT=$?` | M | AGENT | 6 → 50 |
| **L-52 `KERNEL-ROUTER-REAL`** (WI-19) | Eligibility filters, deterministic rule routing, abstention to T0; `lane_for` retired; regret and abstention counters. | kernel · `crates/router/src/**` | L-22, L-50 | `cargo test --locked -p bullet-router; echo EXIT=$?` | L | AGENT | 6 → 65 |
| **L-53 `KERNEL-HOLDOUT`** (WI-13, C3) | Sealed custody, query budgets (a failed query consumes), exposure edges, contamination decisions, N4 build-failure test. | kernel · `crates/holdout/**` (new), `db/migrations/0017_holdout.sql` (new) | L-27 | `cargo test --locked -p bullet-holdout; echo EXIT=$?` | L | AGENT | 3 → 82 |
| **L-54 `KERNEL-FUSION-REAL`** (WI-25) | Typed protocols, `FusionReport`, 8-term `diversity_score`, rank-select short-circuit, forced-synthesis penalty; the four "fail if they pass" tests. | kernel · `crates/fusion/src/**` | L-50, L-53 | `cargo test --locked -p bullet-fusion; echo EXIT=$?` | L | AGENT | 6 → 80 |
| **L-55 `KERNEL-QUOTA-LADDER`** (C5) | 8-level typed observation ladder; probe reservation with its own budget; `unknown` never headroom (negative test). | kernel · `crates/budgets/**` | L-22, L-44 | `cargo test --locked -p bullet-budgets -- ladder; echo EXIT=$?` | L | AGENT | 5 → 80 |
| **L-62 `KERNEL-SEAT-GOVERNANCE`** (C6b) | One seat-equivalent per human for `named_human_subscription` profiles, by profile ownership within the single-tenant contract; negative test with two profiles, one owner. | kernel · `crates/budgets/src/seats.rs` (new) | L-55 | `cargo test --locked -p bullet-budgets -- seats; echo EXIT=$?` | M | AGENT | 5 → 92 |
| **L-56 `KERNEL-BACKPRESSURE`** (WI-14) | Verifier/effect dwell → writer admission, fair durable P0 spill. | kernel · `crates/application/src/queue.rs` | L-27 | `cargo test --locked -p bullet-application -- queue; echo EXIT=$?` | M | AGENT | 1 → 96 |
| **L-57 `KERNEL-TREE-DISJOINT`** (WI-33, A6) | Provably disjoint write-sets keep evidence; overlapping ones re-verify; livelock test at λ > 1/T_verify. | kernel · `crates/application/src/integration/**` (new) | L-29 | `cargo test --locked -p bullet-application -- disjoint; echo EXIT=$?` | M | AGENT | 4 → 88 |
| **L-58 / L-63 `PORTAL-FIFTEEN-SURFACES`** | The six refusal-string surfaces (Cognitive Router, Fusion Lab, Quota/Capacity, Struggle, Behavior, Workspace Hygiene) and the two `<pre>` surfaces (mission-graph, live-attempt) become real projections with provenance headers and watermarks. | portal · `src/pages/**`, `src/surfaces.ts`; kernel · `crates/projections/**`, `apps/bullet-farmd/src/projections/**` | L-52, L-54, L-55 | `npm test && npm run e2e; echo EXIT=$?` | L | AGENT | 8 → 90 |
| **L-59 `PORTAL-SHIFT-BRIEF`** (C6) | Shift Brief as the default screen; every line opens into the exact unproved claim; sourced from the release-truth and scorecard facts. | portal · `src/pages/ShiftBrief.tsx` (new), `src/App.tsx` | L-31 | `npm run e2e; echo EXIT=$?` | M | AGENT | 8 → 95 |
| **J-1…J-6 `JERYU-FORGE-RELEASE`** | See §7: protected refs with expected-old OID, exact-SHA check runs, merge groups, PR parity, signed-tag immutability, release to `git.neverhuman.org`, side-by-side deployment. | jeryu-split family (separate coordination log) | L-12, OD-G | `cargo test` in jeryu-split; `bullet-family forge probe` against the new instance | XL | JERYU + OPERATOR | 4 → 95, arch → 100 |
| **L-38 `INVARIANTS-BATCH-3`** | Collaboration, routing, quota, projection invariants (≈ 13 rows) → `enforced`. | farm registry; kernel/portal tests | L-52, L-54, L-55, L-58 | registry test | M | AGENT | 11 → 92 |
| **L-69 `DOC-SYMBOL-ANCHORS`** | Citations as `path::symbol`; a resolver test fails on a missing symbol; line-number citations retired from normative docs. | farm · `tests/doc_anchors.rs` (new), `docs/**` citations; kernel/git docs | — | `cargo test --locked -p bullet-family --test doc_anchors; echo EXIT=$?` | M | AGENT | 12 → 100 |
| **L-68 `EXT-SECURITY-REVIEW`** (commission) | Independent external review scoped to the authority chain, egress sandbox, forge adapters, and release pipeline; findings tracked as lanes. | — | L-31, L-45, L-46 | signed report on file | L | EXTERNAL | 10 (closes in P4) |
| **S-01 `INDEPENDENT-RESCORE-P3`** | Re-score. | — | all of P3 | `just scorecard` | S | AGENT (other family) | §8 |

P3 exits when: a real Selection Group produces two exact Candidates under `TRANSACTION_PROOF`; two providers
dispatch through the router with live receipts; fifteen surfaces render; merge-group verification runs
against the new Jeryu. Expected blended ≈ 86 (architecture 100, implemented ≈ 87, stranger ≈ 70).

### P4 — Five platforms, evolution, benchmark (weeks 18–30; ≈ 86 → 100)

| Lane | Goal | Repo · paths | Pred | Proof | Size | Class | Dim → |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **L-49 / L-65 `FIVE-TARGET-RELEASE`** | Clean Linux aarch64, macOS x86_64/arm64, Windows x64 hosts (or reproducible cross builders where the target permits); `release build` for all five; signed; mutation-disabled on non-Linux until native containment passes (kept). | farm · `src/release/build/**`; external hosts | OD-F, L-41b | `bullet-family release verify` ×5; `check release --profile universal-v1 --receipts <admitted-absolute-registry> --json` | L | EXTERNAL + AGENT | 9 → 95 |
| **L-66 / L-67 ×4** | Install-twice and stranger trials on the remaining four platforms. | farm · runbooks, stranger-trial receipts | L-65 | receipts admitted | M | EXTERNAL (humans) | 9 → 100, stranger → 100 |
| **L-42b `RELEASE-PUBLISH-SIGNED`** | After OD-E and five-target admission, a tag-triggered workflow publishes only the five signed archives with signed provenance; `release smoke` passes on each and the publication is read back. | farm · `.github/workflows/release.yml` | OD-E, L-65 | `bullet-family release smoke; echo EXIT=$?` plus forge API read-back | M | AGENT | 9 → 100 |
| **OD-H `EVOLUTION-ADR`** | Operator ADR and policy generation bump enabling `evolutionary_authority` only after the seven D3 §3 C2 gates are green. | operator | L-31, L-22, L-53, L-28, L-71 corpus, promotion service | — | a decision | OPERATOR | 7 |
| **L-70a `EVO-FEASIBILITY-SHIELD`** | Deterministic shield before spend; negative knowledge retained; the "recipe field named `attempt_fence` fails schema" test. | kernel · `crates/recipe/src/shield.rs` (new) | L-50, L-51 | `cargo test --locked -p bullet-recipe -- shield` | M | AGENT | 7 → 20 |
| **L-70b `EVO-EVALUATION`** (WI-20) | Corpus ingestion, evaluation vectors, matched-compute accounting, complete `AggregateEvaluationV1` with every pre-outcome assignment. | kernel · `crates/evaluation/**` (new) | L-53, L-22 | `cargo test --locked -p bullet-evaluation` | L | AGENT | 7 → 40 |
| **L-70c `EVO-ARCHIVE`** | Bounded MOME archive, four frozen axes, epsilon-dominance, append-only history. | kernel · `crates/recipe/src/archive.rs` (new), `db/migrations/0018_archive.sql` (new) | L-70b | `cargo test --locked -p bullet-recipe -- archive` | L | AGENT | 7 → 55 |
| **L-70d `EVO-PROMOTION-SERVICE`** | B0–B6 promotion as a separate principal from the optimizer; ASHA inside B1/B2 only; B3 against L-53 custody; canary with automatic rollback. | kernel · `apps/bullet-promoter/**` (new) | L-70c | `cargo test --locked -p bullet-promoter` | L | AGENT | 7 → 75 |
| **L-70e `EVO-DRIFT-ROLLBACK`** | Bound breach removes the recipe from routing, preserves lineage, restores the certified incumbent; the five D3 C2 falsification tests. | kernel · `crates/recipe/src/drift.rs` (new) | L-70d | `cargo test --locked -p bullet-recipe -- drift` | M | AGENT | 7 → 90 |
| **L-71 `WP-15-BENCHMARK`** | The matched corpus: raw task subjects, configuration, receipts, exclusions, exact upstream commits; the only admissible source of any comparison claim. | farm · `docs/assurance/benchmark/**` (new); evaluation data outside the repos | L-70b | corpus receipt admitted | XL | AGENT + EXTERNAL | 7 → 100 |
| **L-39 `INVARIANTS-51`** | Remaining rows → `enforced`; 51/51; ratchet at 51. | farm registry | L-70e | registry test | M | AGENT | 11 → 100 |
| **L-68 closure** | External review findings closed or accepted by ADR. | per finding | L-68 | signed closure | M | AGENT + EXTERNAL | 10 → 100 |
| **S-01 `INDEPENDENT-RESCORE-FINAL`** | Two independent re-scores from different families; 100 stands only if both agree within 2 on every dimension. | — | everything | `just scorecard` | S | AGENT (two families) | §8 |

P4 exits when the complete `universal-v1` dependency closure is green and its explicitly profiled
`check release` against the admitted absolute registry returns exit 0; five stranger receipts;
evolution running under B0–B6 with a proven rollback; the final double re-score agrees. **That is 100.**

---

## 4. Dependency graph

```
P0  S-00 scorecard ─► S-01 rescore              (runs again at the end of every phase)
    L-05 nonce ──────────────────────────┐
    L-13 liveness   L-06 fuzz   L-07 prop │   L-33 egress-CI ─► L-34 gates-in-sandbox
    L-12 forge port ────────────┐         │   L-35 ratchet ─► L-36 ─► L-37 ─► L-38 ─► L-39 (51/51)
    L-10 lock validation ──┐    │         │
    OD-D ◄── THE PIVOT ────┼────┼─────────┼──────────────────────────────────────────┐
                           ▼    │         ▼                                          │
P1  L-40 lock ─► L-42 wf                    L-20 ─► L-21 ─► L-22 budgets ─► L-23 ─► L-24 gitd checker
    L-11 hub forge                                              │                      │
                                                                │          L-25 proof root ─► L-27 verifier ─► L-28 attestor ─► L-29 readback
                                                                │                      │                                           │
                                                                │          L-26 saga ─► L-32 faults          L-30 dispatch         │
                                                                │                      └──────────────────────────┬─────────────────┘
                                                                │                                                 ▼
                                                                │                                     L-31 TRANSACTION_PROOF  [M2]
P2  OD-A ─► L-44 live providers ◄────────────────────────────────┘                                                 │
    OD-B ─► L-45 Jeryu live      OD-C ─► L-46 GitHub      OD-E ─► L-41b sign ─► L-43 setup ─► L-66 install×2 ─► L-67 stranger   │
    L-61 audit chain   L-60 freeze   L-64 quarantine   L-15 GC retention   L-16b mutation   L-47b Jankurai 90      │
                                                                                                                  │
P3  L-50 roles ─► L-51 topologies ─► L-52 router ─┐        L-53 holdout ─► L-54 fusion ─► L-58/63 surfaces        │
    L-55 ladder ─► L-62 seats     L-56 backpressure│        L-57 tree-disjoint     L-59 shift brief ◄──────────────┘
    J-1…J-6 Jeryu release ─► merge-group verification against the default forge   L-68 review (commissioned)
                                                                                                                  │
P4  OD-F ─► L-49/L-65 five targets ─► L-42b publish ─► L-66/L-67 ×4      OD-H ─► L-70a…e evolution ─► L-71 corpus │
    L-39 invariants 51/51    L-68 closure    S-01 final double re-score  ═══════════════════════════════► 100
```

**OD-D is still the pivot.** It gates the release track (L-40 onward) and, through the immutable `bullet-wire`
source that `bullet-gitd`'s gateway names as its reason to refuse, the authority track (L-24 onward) — and
therefore M2, M3, and everything in P3/P4. Nothing an agent can do substitutes for it.

---

## 5. Parallelism and schedule

**Constraints that bound the fleet.** The zero-worktree rule means lanes are partitioned by path inside single
checkouts; only orchestrators commit; every commit is path-exact against a coord claim. The kernel spine in P1
is serial by construction (each lane extends the previous lane's schema and types), so the kernel carries at
most **three concurrent lanes** on disjoint crates (spine + one `crates/effects`/`crates/verifier` lane + one
`harness-egress`/`runner/tests` lane). Hub, git, and portal each carry one to two lanes.

| Phase | Concurrent lanes | Agents | Wall-clock estimate | Gate to exit |
| --- | --- | --- | ---: | --- |
| P0 | 13 (all parallel) | 8–10 | 1 week | scorecard drift-gated; OD-D verifiable |
| P1 | spine (serial) + 6 side lanes | 6–8 | 5 weeks (spine ≈ 4) | `transaction-demo` + `installable-lock` admitted |
| P2 | 12 | 7–9 | 4 weeks (bounded by operator acts and the stranger trial) | six live gates; Linux stranger receipt; Jankurai ≥ 90 ×4 |
| P3 | 14 + the Jeryu family | 9–12 | 8 weeks | Selection-Group `TRANSACTION_PROOF`; 15 surfaces; merge groups on the default forge |
| P4 | 10 + external hosts | 6–8 + humans | 12 weeks (bounded by five-platform hosts and the corpus) | 26/26 green; five stranger receipts; double re-score |

Total ≈ 30 weeks if every operator and external act lands the week it is first needed. Two sensitivities
dominate: **OD-D late by N weeks slides everything by N**; the five-platform hosts (OD-F/L-49) bound P4 and
cannot be compressed by adding agents. Everything else is agent-elastic — more agents shorten P0/P2/P3, not P1.

Weekly cadence, every phase: Monday `coord status` + `check release --profile universal-v1 --receipts
<admitted-absolute-registry> --json` + `check scorecard --json` posted
to the coordination log; lanes claim path-exactly, heartbeat ≤ 5 min, hand off with per-path hunk attestations;
orchestrators clear the index → stage exact leaves → verify staged == claim → commit → receipt. Friday: the
generated scorecard is regenerated and diffed; any row that moved without a new evidence subject is reverted.

---

## 6. Operator and external register — with need-by phase and the cost of delay

Register of record: [ADR 0013](../decisions/0013-operator-decision-register.md). No agent writes an
`— operator —` line. This table adds *when* each act is first needed and what stalls without it.

| ID | Act | First needed | Stalls without it | Cost |
| --- | --- | --- | --- | --- |
| **OD-D** | New signed immutable member tags with authenticated Jeryu URL/slugs and a tag signer (`v0.1.0-alpha.4` predates the signed authority contract `c07efb1`; use L-10's frozen URL contract) | **P0, week 1** | L-40…L-43 (release), L-24 → L-31 (M2), and everything after | hours |
| **OD-A** | `bullet authority keygen`; generation-2 v1alpha2 policy outside the repositories; ratify the provider-runner key, budgets, four provider profiles, expiry, rollback in the coordination log (`docs/runbooks/live-conformance.md` §2) | start of P2 | all four `release.provider.*`; L-44; L-55's live half; dimension 6's "two providers dispatching" | minutes + bounded spend |
| **OD-B** | `jeryu gh-setup --host http://127.0.0.1:8787 --token-file <0600 file>`; separate broker/attestor/integrator credentials; one exact protected test repository | start of P2 | `release.forge.jeryu`; L-45 | minutes–hours |
| **OD-C** | GitHub App on one branch-protected test repo; delivery and attestation credentials separated | start of P2 | `release.forge.github-app`; L-46; merge-group verification against GitHub | hours |
| **OD-E** | Ed25519 release-signing key under protected custody; signer policy; `release/allowed_signers`; root-owned `/etc/bullet-farm/release-msrv-1-95-admission.toml` with three distinct roots | start of P2 | `signatures`, `receipt-contracts`, `rust-msrv-1-95`, `provenance`; L-41b; L-43; every stranger trial | hours |
| **OD-F** | Linux preview outside the release gate **or** amend the five-archive rule by reviewed ADR; then provision the four non-Linux build hosts | decision in P2; hosts by start of P4 | L-49/L-65; dimension 9 above 65; stranger above 60 | a decision + hosts |
| **OD-G** | Ratify public names/endpoints (`git.neverhuman.org`, GitHub org), Jeryu deployment identity, backup, TLS | before J-6 (P3) | the Jeryu release deployment; hosted CI provisioning; public mirror | a decision + ops |
| **OD-H** | Operator ADR + policy generation bump enabling `evolutionary_authority` after the seven D3 §3 C2 gates are green | P4 | L-70d/e activation (the code can be built and tested with the flag off) | a decision |
| **X-1** | Portable checksum-pinned Jankurai artifact (L-48) | P2 | hosted `audit` lane; `release.jankurai-90` hosted half | external |
| **X-2** | Independent external security review (L-68) | commissioned P3, closed P4 | dimension 10 above 92 | external |
| **X-3** | Five stranger-trial volunteers (L-67), one per platform, no access to this box | P2 (Linux), P4 (others) | stranger dimension above 25 | people |

---

## 7. The Jeryu release lanes — what localhost unlocks

The user's standing directive: key git features become a **new Jeryu binary/version released to
`git.neverhuman.org`**, and the running `127.0.0.1:8787` instance — which many repositories depend on — is
never disturbed. ADR 0002 and D4 settled the shape: no vendored Jeryu source; Jeryu consumed only via pinned
tags from the jeryu-split family; `ForgeEffects` extended in the kernel (L-12); Jeryu is the recommended
default forge because it is the only one that can be offline, deterministic, credential-free, and — after
these lanes — merge-group-capable. GitHub/GitLab remain configurable targets that get the same port with honest
refusals where the forge lacks a primitive.

These lanes run in the jeryu-split family with its own coordination log and tag discipline
(`*-v5.x-split.N`). Each is the closer for a specific §2 criterion.

| Lane | Feature | Closes | Contract with the kernel |
| --- | --- | --- | --- |
| **J-1 protected refs + expected-old-OID** | Ruleset per ref namespace; server-side compare-and-swap on push; typed rejection naming the observed OID | dimension 4 "protected-ref op"; A2 exact-OID CAS on the default forge | `ForgeEffects::protect_ref`, `push_candidate_ref` read-back sees the server's OID, never the client's |
| **J-2 exact-SHA check runs** | Check-run API keyed by `{sha, name, external_id}`; immutable once concluded; conclusion by a distinct credential | dimension 4 "check-run op"; the attestor's publish target | `ForgeEffects::publish_check` with `proof_bundle_id` and `attestor_policy_hash` in `external_id` |
| **J-3 merge groups** | Server-composed merge group with a stable group head SHA; checks bind to the group head; group invalidated on base movement | **the architecture deduction** (D2 §3.2): merge-group verification on the default deployment; dimension 4 "merge-group op" | `ForgeEffects::open_merge_group` / `read_merge_group_head`; L-57 falls back to tree-disjoint batching only when the forge reports `Unsupported` |
| **J-4 PR parity** | PR create/read/list with exact head/base OIDs and a required-checks view | dimension 4 "PR op"; portal Merge Rail | `ForgeEffects::open_pull_request`, `read_required_checks` |
| **J-5 signed-tag immutability** | Tags as annotated objects; optional signature verification against a configured allowed-signers file; deletion/move refused for immutable namespaces | dimension 12: "immutable signed Jeryu tags" becomes a true sentence; OD-D's tags verifiable server-side | `bullet-family lock verify --tag` checks the server's signature verdict as well as the local one |
| **J-6 release + side-by-side deployment** | New `jeryu-*-v5.x-split.N` tags; binary published to `git.neverhuman.org`; deployed as a **second instance on a new port** with a migrated copy of the data; `bullet-family forge probe` certifies it; the operator (OD-G) chooses the cutover moment; the old instance is untouched until then | dimension 9 hosted half; the do-not-disturb rule stays satisfied | `family.lock` `jeryu_url` may point at either instance; both must pass `forge probe` |

Nothing in J-1…J-6 runs against 8787 except read-only probes. The kernel adapters are written against the port
contract and the conformance suite (L-12's `forge_conformance.rs`), so they are testable against
`LocalBareForge` before any Jeryu build exists.

---

## 8. The fair-scoring protocol

**The scorecard becomes an instrument, not an opinion** (S-00). `policy/scorecard-v1.json` holds the rubric;
every row of §2 is a record `{dimension, criterion, design_score, evidence_kind, evidence_subject,
implemented_score}`. `bullet-family check scorecard --json`:

1. Reads each row's evidence subject: a receipt id in `.bullet-family/coord`, a test name that must appear in
   a proof-lane observation from the last seven days, a `check release` gate state, a commit OID that must be
   an ancestor of HEAD, or a signed external document on file.
2. Scores the row as `min(claimed, admitted)`. A row whose subject is missing, expired, or refuted scores its
   **design** value into column A and **0** into column I — the D2 rule, mechanised.
3. Renders `docs/assurance/scorecard.generated.md` with the twelve dimension totals, the blend, and a
   *"What moved and why"* diff against the previous generation, digest-gated in `required` like release-truth.

**Independence.** After each phase an agent from a different provider family runs `just scorecard`, reads only
the generated page, and writes its own twelve numbers. A gap over five on any dimension is an adjudicated
inconsistency in the glossary with both numbers and a ruling; the lower stands until the ruling closes.
The final 100 requires two independent re-scores from two families agreeing within two on every dimension.

**Frozen rubric.** Dimensions, weights, and the 20/60/20 blend are `scorecard-v1`. A `scorecard-v2` needs a
reviewed ADR and re-renders every historical generation under both versions side by side.

**What would make a 100 false — the falsifiers, checked continuously:**
- any `#[ignore]`d test cited as evidence that no CI lane ran in the last seven days;
- any receipt older than its subject's TTL (live receipts expire; the score drops until re-run);
- any invariant marked `enforced` whose named test is absent from the last proof-lane observation;
- any stranger-trial receipt from someone with a coordination-log entry;
- any Jankurai score obtained by suppressing a rule without a reasoned `jankurai:allow`;
- any comparison claim anywhere in `docs/` before L-71's corpus receipt exists.

---

## 9. Risks and the anti-patterns this plan refuses

| Risk | Refusal |
| --- | --- |
| Starting the collaboration or evolution runtime before M2 | Normative in `evolutionary-control.md`; P3 has L-31 as a hard predecessor; `evolutionary_authority=true` stays `UNSAFE_POLICY` until OD-H. |
| Buying score by re-weighting or by counting design as done | §0 rules 1–2; the generated scorecard mechanises them. |
| Buying Jankurai points from the auditor's own defects | Two principled reversions are already documented; L-47 keeps the rule. |
| Synthetic PASS surviving into a `TRANSACTION_PROOF` | L-31 replaces the demo; L-32 forbids every crash boundary from becoming PASS; `unknown_satisfies_gate: false` stays immutable. |
| A comparison claim against Gas Town before the corpus exists | Forbidden by `competitor-snapshot.md`; a falsifier in §8. |
| Disturbing the running Jeryu | J-6 is side-by-side on a new port; 8787 sees read-only probes only; OD-G chooses the cutover. |
| Operator acts arriving late | §6 names the need-by phase and what stalls; agents build everything testable offline first (adapters against `LocalBareForge`, evolution with the flag off). |
| Shared-tree contamination in a fleet of 8–12 agents | Path-exact claims, index cleared before staging, staged-set == claim verified, per-path hunk attestations at handoff — the discipline that caught two mistakes on 2026-08-25. |
| Five-platform hosts never materialising | Then the honest ceiling is ≈ 96 (dimension 9 at 65, stranger at 60); OD-F's "amend the five-archive rule by reviewed ADR" is the only other path, and it would be a `scorecard-v2` change, scored side by side. |

---

## 10. Definition of done, and the trajectory

100 is declared when **all** of the following are true at once and re-scored independently by two families:

- [ ] `bullet-family check release --profile universal-v1 --receipts <admitted-absolute-registry> --json`
      returns exit 0 with every gate in the full dependency closure admitted from current receipts.
- [ ] Every §2 row has an admitted evidence subject; `scorecard.generated.md` shows I = 100 on all twelve.
- [ ] Architecture 100: paper and contract agree on four providers; the default forge runs merge groups.
- [ ] Five stranger-trial receipts, one per platform, from people with no coordination-log entry.
- [ ] Invariant registry 51/51 `enforced`, ratchet at 51.
- [ ] Jankurai ≥ 90 ×4, zero caps, zero hard findings, hosted audit lane green.
- [ ] External security review closed.
- [ ] Evolution running under B0–B6 with a proven automatic rollback, enabled by OD-H, never before.
- [ ] Matched benchmark corpus receipted; no comparison claim published before it.
- [ ] `docs/paper.md` status macros regenerated from the final `evidence.json`; the paper's concerns table
      shows each concern's closing receipt.

| Checkpoint | Architecture | Implemented | Stranger | **Blended** |
| --- | ---: | ---: | ---: | ---: |
| D2 snapshot (2026-08-25 09:37Z) | 94.5 | 37.4 | 3 | **42** |
| Today (estimate, §1) | 94.5 | ≈ 40 | 3 | **≈ 43** |
| P0 exit | 94.5 | ≈ 45 | 3 | **≈ 47** |
| P1 exit (M2) | 94.5 | ≈ 60 | 25 | **≈ 60** |
| P2 exit (M3 + M4) | 94.5 | ≈ 70 | 60 | **≈ 73** |
| P3 exit (capability leap) | 100 | ≈ 87 | 70 | **≈ 86** |
| P4 exit (GA + evolution + corpus) | 100 | 100 | 100 | **100** |

Every number in the last five rows is a forecast under the D2 rubric; the generated scorecard and the
independent re-score replace them as each phase lands. If a forecast turns out generous, the lower number is
the one this page will show.
