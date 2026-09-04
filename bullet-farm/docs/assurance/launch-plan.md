# Historical launch-plan checkpoint

Status: **SUPERSEDED on 2026-08-25; historical planning record only**
Owner: Bullet Farm maintainers
Last reconciled: 2026-08-25

> Do not execute or quote the milestone order, scope labels, commands, counts,
> or OIDs below as current truth. The active dependency order is the
> [closure roadmap](closure-roadmap.md): credential-free `TRANSACTION_PROOF`,
> first-GA `self-hosted-v1`, separate `evolution-v1` (Wave 9), independent
> provider/forge/platform profiles and later `universal-v1` (Wave 10), then
> `team-v1` and `saga-v1` (Wave 11).
> Executable profile evaluation and admitted receipts always win.

Sources of truth this plan is built from: [`release-truth.generated.md`](release-truth.generated.md) (canonical
43-row `universal-v1` target, historical 26-gate diagnostic, and ungated blockers), [`product-gaps.md`](product-gaps.md) (G1–G15),
[`v1-closure-plan.md`](v1-closure-plan.md) (V1-S0…S8),
[`../release.md`](../release.md), [`../decisions/0013-operator-decision-register.md`](../decisions/0013-operator-decision-register.md),
family-root `TEAM_PLAN_CLAUDE.md` §10 (WI-01…WI-35), the coordination log's named blockers, and the full read-only
inventory archived at family-root `.l7-bundle/REMAINING-WORK-INVENTORY-2026-08-25.md` (≈120 rows). Where a number here
disagrees with a generated page, the generated page wins.

## 0. What "launch" can honestly mean, in order

The historical `legacy-v1-26` projection is a frozen 26-row diagnostic. Every release invocation now requires an
explicit profile and absolute receipt registry; the full frozen portable V1 target is `universal-v1`. Each gate
still needs kind-specific semantic receipt admission before it can turn green. Each milestone names the strongest
claim that is true at that point; nothing earlier is "shipped". `legacy-v1-26` and `linux-preview` are diagnostic
profiles and never release authority; narrower product profiles certify only their named dependency closures.

| Milestone | Honest claim when reached | Gates opened (cumulative) | Who must act |
| --- | --- | --- | --- |
| **M0 — Clean board** | Documentation, audits and generated pages are internally consistent; every checkout is clean. | 0 | agents |
| **M1 — Contributor-installable source family (Linux x86_64)** | A contributor clones the hub, verifies signed member tags, and sets up from source. Not "installable": no prebuilt binary, no package. | `installable-lock`; scans (`secret`, `dependency`, `license`, `workflow`) become receiptable against tagged trees | agents + **OD-D** |
| **M2 — Five-plane `TRANSACTION_PROOF`** (credential-free, offline) | One exact change was proposed, applied only through BulletGit, independently verified, delivered with reconciliation and truthfully projected through the existing authenticated minimal surface — with a signed receipt. **The product exists as a product here.** | `transaction-demo`; production BulletGit G4 stops blocking G2. Full Portal/API/cognitive product surfaces G13–G15 follow M2 | agents (uses M1's tag) |
| **M3 — Explicit live checkpoints** | After the offline proof, separately approved Claude, Codex, Cursor, and Antigravity turns pass; protected Jeryu and GitHub Candidate effects reconcile through exact read-back and observation. | all four `provider.*` gates plus `forge.jeryu` and `forge.github-app` | **OD-A, OD-B, OD-C** + agents |
| **M4 — Linux preview package** | A signed, SBOM- and provenance-bearing archive installs twice from tagged bytes on Ubuntu 24.04 x86_64, exercises systemd lifecycle/recovery, and refuses unsupported mutation platforms. This is explicitly a non-release `linux-preview`. | diagnostic Linux package, installer, operations, recovery, supply-chain, fault, and quality evidence | agents + **OD-E**, root descriptor, hosted Jankurai artifact |
| **M5 — Canonical V1 GA** | The exact five signed archives pass their release smoke, every gate in the complete `universal-v1` dependency closure has current admitted receipts, and the explicitly profiled command returns green. | complete portable V1 closure | agents + operators; five-platform builders/signers and all live approvals |

The historical twelve-dimension scoring methodology, old P0–P4 phase sketch, and superseded lane board are preserved
in [`path-to-100.md`](path-to-100.md). Neither this page nor
[`v1-closure-plan.md`](v1-closure-plan.md) is the current execution plan; only
the active [`closure roadmap`](closure-roadmap.md) defines dependency order.

The frozen V1 contract requires all four providers, Jeryu and GitHub effect receipts, and exactly five signed
archives: Linux x86_64/aarch64, macOS x86_64/arm64, and Windows x64. Linux is the full production runner; other
platforms fail closed on real mutation until their native containment passes. No live mutation occurs without a
distinct signed operator checkpoint after the offline gates pass.

## 1. Agent-closable now (no operator input) — the work queue in dependency order

Every row names inputs that are on the development host today. Lanes are path-disjoint, coord-claimed, and committed
path-exactly with receipts. IDs refer to the inventory.

### M0 (reconciled history and terminal clean-check criterion)
| Item | What | Status |
| --- | --- | --- |
| R-26 | Commit the runbooks / ADR 0013 / glossary / spec-mirror group | done (hub `b6ae0ca`) |
| R-36, R-37 | This plan; paper value/risk framing (R5) | current planning references; generated truth still wins |
| R-01 | Reconcile the stale "seven unprojected / five projections" truth after Context Lineage | done in this commit: source, golden, and generated page say six/six |
| R-27 | End every convergence with four clean canonical primary checkouts | verify at handoff with Git status and the family lane; this plan never asserts cleanliness by itself |
| R-11 | `bullet-git` fails its own audit floor (54 < 56) | **done** — git `e1b47ff1`: 54 → 65, floor ratcheted to 65, caps 9 → 3 |

### M1 (agent half)
| Item | What | Lane |
| --- | --- | --- |
| R-03, R-50 | `deny.toml` (licenses, advisories, bans, sources) in all Rust repos; `zizmor` family-wide | **done** — hub `f9edeef7`, kernel `81d549de`, git `858c533b`, portal `3aa0ec69`; plus a fail-closed advisory-freshness proof (cargo-deny 0.19.8 reports a failed fetch as success) |
| R-15, R-16 | Explicit MSRV-1.95 and pinned-1.97.1 lanes; resolve the toolchain contradiction; emit observation JSON in the receipt schema | **done** — hub `9a7237e4`, kernel `b5863b03`, git `c64b7f85`; finding: no MSRV incompatibility exists anywhere |
| R-21, WI-34, WI-30 | **LANDED** (hub `3728e798`, kernel `bb420f46`, git `54d1338b`): `NoNewDispatchAfterStop` checked as a property while `NoDispatchAfterStop` is documented as a state predicate TLC refutes at depth 4; STONITH `grace < TTL` mirrored in both validators, closing the formerly accepted zero-maximum hole; GC-under-load tests mutation-proven; reflink remains designed, not implemented | done |
| R-43…R-46, R-42 | Jankurai hygiene caps (proof-lane mapping, pre-push parity, dead markers outside contract strings, zyal sentinel, supply-chain manifest) and LOC splits — per repo | component work is tracked by current repository audit lanes; hosted/release admission remains separate |
| R-10 | Publish per-repo Jankurai numbers (kernel 57 / git 54 / portal 60) in `release.md` | BLOCKED diff for the holder |
| R-25 | Root `docs/paper.md` sanitized to the hub copy | done (family root) |
| then | `bullet-family lock generate --tag <tag> --subjects <absolute-path>` once OD-D exists | after OD-D |


### Wave 1 from the capability study (2026-08-25 10:11Z–11:03Z) — landed
| Item | What | Status |
| --- | --- | --- |
| L-04 | A crashed runner blocked its Variant forever: `expire_leases` had no production caller | **done** — kernel `4effc37d` (reclaim inside the acquisition transaction, `LeaseService::expire_due`, `bullet farm reap`) + `107c5cd5` (farmd reaper tick, ≤500 ms, never disableable, contention-tested against the acquisition path); both mutation-proved |
| L-01 | `doctor` exited 0 while reporting BLOCKED | **done** — hub `f00171ff`: exits 3 on BLOCKED; required lane asserts exit/JSON agreement |
| L-03 | OD-B told the operator to run `gh auth login` (Jeryu: do-not-run) | **done** — hub `f00171ff` (ADR 0013 OD-B: `jeryu gh-setup --token-file`); corrected in every out-of-repo copy |
| Jankurai | hub 58 / kernel 57 / portal 60 vs ≥ 90 | **raised** — hub 65 / kernel 64 / portal 68 (git 65): hub `0cc0a681`, kernel `1dcbaa21`, portal `5e4b474f`; floors for kernel/portal `ops/ci/audit.sh` await the CI lanes; hosted half EXTERNAL (portable pinned auditor) |

### M2 (agent-only, the pivot) — ordered
1. **R-07 / RUNNER-FARMD-LEASE-ROUTE-AUDIT** — promote the signed UDS RPC/client from component proof. Registered Runner ID/epoch ↔ `SO_PEERCRED` UID binding, farmd UID plus socket GID/device/inode pinning, and durable server grant/nonce state are landed. Remaining: operator-admitted durable peer registry and signing-key custody, durable client acquire/read-back recovery instead of process-local `last_acquire`, product Runner wiring through the bounded client, and connected lost-response proof.
2. **V1-S2-a / KERNEL-AUTHORITY-PREDECESSOR** — normalized truth: no JSON-blob authority, no `INSERT OR REPLACE`; lease/Attempt rows carry graph revision, workspace generation, scope digest, policy/routing generation, authority epoch, freeze generation.
3. **V1-S2-b** — short-lived PASETO mutation capabilities minted from the durable active lease.
4. **G4 / ONLINE-AUTHORITY-AUDIT (2)–(4)** — Kernel reservation + one-use permit; BulletGit positive checker replaces `AuthorityGateway::unavailable`; settlement after I/O. Honest wire binding needs the M1 tag (blocker 1).
5. **V1-S4-a/b** — real runner saga (acquire → read-only provider generation → proposal → BulletGit apply → admitted gates → ≤2 repairs → checkpoint → exact Candidate; heartbeat failure freezes, kills the tree, preserves, successor resumes from the exact checkpoint).
6. **V1-S4-c / WI-13 / R-55** — independent verifier: per-gate executable digest, multi-gate aggregation, oracle-modifying-diff classification with sealed holdouts.
7. **V1-S4-d** — effect broker intent → dispatch → lost-response `UNKNOWN` → read-back → identity-exact adoption, no second write (local forge).
8. **V1-S5-a** — farmd signed dispatch so a public command settles past PENDING.
9. **R-04** — replace `just demo`'s synthetic success with the signed `TRANSACTION_PROOF`; **R-08** build the tagged crash-boundary fault suite alongside step 5.
10. **G13 / G14 / G15** — after the minimal transaction is independently proven, add each remaining Portal surface, farmd route, and cognitive ledger subject as a projection of durable truth. These completeness lanes consume G2; they are not prerequisites of it.

### M4/M5 (agent half)
| Item | What | Lane |
| --- | --- | --- |
| R-05 / V1-S5-d | Embed the manifest-verified Portal bundle in farmd; Playwright against the packaged origin | **done** — kernel `d59c5a72` + `0c10cd9e`, portal `f4b8975e`; browser suite 2/2 against farmd's own origin |
| R-02 | Production build broker for the exact five-target matrix; checksums; both SBOM formats; provenance producer; non-circular manifest generator; release workflow (R-13) | Linux x86_64 **quarantined component only** — hub `a1b7ab38` + `ab1fd8fa` exercise deterministic bundle construction, CycloneDX, checksums, unsigned provenance, and eight-binary modes in tests; the public command refuses before parsing or mutation. Still open: different-identity exact-OID build broker, semantic admission, signing (OD-E), schema-3 lock (OD-D), four external build hosts (OD-F), and release workflow (R-13) |
| V1-S7-a…d | Signed admission of the wrapper-selected executable; clone-transport helper subjects; transaction-wide repository stability; allowed-signers admission | next |
| Post-V1 evolution | Frozen T0/T3 study, deterministic allocation/evaluation, external confirmation, R0/R1 canary and rollback evidence | explicitly post-V1; keep `evolutionary_authority=false` |

### Independent/deferred profiles

PostgreSQL team mode (R-30), remote runners, cross-repository sagas, and evolutionary self-tuning close only later
profiles. GitHub effect/attestation (R-17/WI-12), Codex/Cursor/Antigravity live certifications, and macOS/Windows
archives are not deferred: the frozen canonical V1 contract requires them.

## 2. Operator actions — the complete list (register: ADR 0013)

| ID | Action | Unblocks | Cost |
| --- | --- | --- | --- |
| **OD-A** | `bullet authority keygen`, write the v1alpha2 generation-2 policy outside the repositories, ratify the provider-runner key, budgets, all four provider service profiles, expiry, and rollback (`docs/runbooks/live-conformance.md` §2) | all four `release.provider.*` gates | minutes; bounded provider spend |
| **OD-B** | After the read-only Claude receipt, issue separate Jeryu broker/attestor/integrator credentials and name one exact protected test repository | `release.forge.jeryu` | minutes to hours |
| **OD-C** | GitHub App on one branch-protected test repo, delivery and attestation credentials separated | mandatory V1 `release.forge.github-app` (after R-17/WI-12) | hours |
| **OD-D** | Publish signed immutable member tags with authenticated Jeryu URL/slugs and a tag signer (a *new* tag: `v0.1.0-alpha.4` predates the signed authority contract `c07efb1`) | `installable-lock`; every "from tagged bytes" gate; honest wire binding for M2 step 4 | hours |
| **OD-E** | Ed25519 release-signing key under protected custody; signer policy for namespace `bullet-farm-release-receipt-v1`; `release/allowed_signers`; root-owned `/etc/bullet-farm/release-msrv-1-95-admission.toml` with three roots and signed time (R-14) | `signatures`, `receipt-contracts`, `rust-msrv-1-95`, `provenance` | hours |
| **OD-F** | Ratify either an explicitly non-release Linux preview with no release tag/V1 wording, or amend the frozen five-archive rule through a reviewed ADR | the permitted M4 description only; it does not clear either five-platform release gate | decision before M4 publication |
| **OD-G** | Ratify public names/endpoints (`git.neverhuman.org`, GitHub org), Jeryu deployment identity, backup, TLS (workplan WP-08/14/17) | public mirror, permalinks, hosted CI provisioning | a decision + ops |

## 3. External / platform

Canonical V1 needs clean Linux x86_64/aarch64, macOS x86_64/arm64, and Windows x64 build hosts, a checksum-pinned
portable Jankurai artifact for hosted CI (`jankurai-90` hosted half, R-12), and hosted family provisioning for
member CI (R-49, HOSTED-PORTAL-FAMILY-CI). Non-Linux builds must remain mutation-disabled until native containment
passes, but all five signed archives remain release blockers.

## 4. Reconciliation rule
The six implemented/six missing projection count, per-workspace Rust 1.95.0/1.97.1 assignments, and Jankurai
1.6.11 pin are reconciled. Historical `TEAM.md`, `POTENTIAL_DRAFT.md`, archived operator kits, and schema-removal
notes remain non-authoritative context where they conflict with the frozen all-four-provider `universal-v1`
profile, the current setup refusal, or generated truth. Any newly reproduced disagreement is a blocking truth bug:
fix the executable source and generated projection together; never choose the greener prose value.

## 5. Feature completeness against the vision (summary; full table in the inventory §4)
**Exists as code (COMPONENT_PROOF):** five principals; DB-clock leases/fences; idempotent graph mint; ambiguity → durable `UNKNOWN`; private dissociated clones; prior-or-complete-next generations + sealed preservation; provenance-bound `CandidateId`; signed launch grants; Linux egress isolation; four offline provider protocol subsets; 13-step live-conformance path; policy v1alpha2 rule (hub + kernel); revision-one Context Capsule; watermark-bound projections (9/15 surfaces); authenticated command ingress; two pinned TLA+ models; signed bundle verify/extract/receipt verifier; sealed setup tool subjects; coordination ledger; release-truth report.
**Design-only:** roles as capability profiles; hard-constraint routing; quota epistemology (reservations, probe, one-seat-per-human); vector fitness/selection; fusion with dissent; struggle ladder; behavior gateway; verifier backpressure; attestor ≠ broker; oracle-split + holdouts; typed freeze ack; wound-wait; staged race budgets; gates inside the egress sandbox; tree-disjoint evidence preservation; anchored audit batches; CAS/GC; topology library; SLO defaults; KPI loops; login-challenge inbox.
**Independent/deferred profiles by contract:** PostgreSQL and remote runners; cross-repo sagas; multi-tenant;
evolutionary self-tuning. The minimum typed cognitive plane is V1, but the T0-versus-T3 confirmation/canary is
post-V1. All four providers, both effects, and all five archives remain canonical V1 blockers.

## 6. Exit criteria for this plan
This plan is retired when kind-specific receipt admission is implemented and `bullet-family check release --profile
universal-v1 --receipts <admitted-absolute-registry> --json` admits current signed receipts for its complete
dependency closure—including Claude, Codex, Cursor, Antigravity, Jeryu, hosted adapters, and all five platforms—and
returns green. Diagnostic profiles may remain useful but cannot substitute for that decision. Until then the honest
status is **pre-release, blocked**, and the strongest evidence grade remains the grade actually admitted by receipts.
