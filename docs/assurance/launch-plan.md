# Launch plan — bridging every remaining issue to a shippable Bullet Farm V1

Status: **ACTIVE planning artifact; not runtime or release authority**
Owner: Bullet Farm maintainers (orchestrator: claude-orch)
Last reconciled: 2026-08-25 ~09:00Z against hub `ba7c955`, kernel `0109a90`+, git `3c90cbe`+, portal `b493f29`+
Sources of truth this plan is built from: [`release-truth.generated.md`](release-truth.generated.md) (canonical
26-gate V1 catalog + ungated blockers), [`product-gaps.md`](product-gaps.md) (G1–G15),
[`v1-closure-plan.md`](v1-closure-plan.md) (V1-S0…S8),
[`../release.md`](../release.md), [`../decisions/0013-operator-decision-register.md`](../decisions/0013-operator-decision-register.md),
family-root `TEAM_PLAN_CLAUDE.md` §10 (WI-01…WI-35), the coordination log's named blockers, and the full read-only
inventory archived at family-root `.l7-bundle/REMAINING-WORK-INVENTORY-2026-08-25.md` (≈120 rows). Where a number here
disagrees with a generated page, the generated page wins.

## 0. What "launch" can honestly mean, in order

`bullet-family check release --json` reports 26/26 BLOCKED today. The current canonical command has no generic
registry option; each gate needs kind-specific semantic receipt admission before it can turn green. Each milestone
names the strongest claim that is true at that point; nothing earlier is "shipped". Named profiles, including
`linux-preview`, are diagnostic slices and never release authority.

| Milestone | Honest claim when reached | Gates opened (cumulative) | Who must act |
| --- | --- | --- | --- |
| **M0 — Clean board** | Documentation, audits and generated pages are internally consistent; every checkout is clean. | 0 | agents |
| **M1 — Contributor-installable source family (Linux x86_64)** | A contributor clones the hub, verifies signed member tags, and sets up from source. Not "installable": no prebuilt binary, no package. | `installable-lock`; scans (`secret`, `dependency`, `license`, `workflow`) become receiptable against tagged trees | agents + **OD-D** |
| **M2 — Five-plane `TRANSACTION_PROOF`** (credential-free, offline) | One exact change was proposed, applied only through BulletGit, independently verified, delivered with reconciliation and truthfully projected — with a signed receipt. **The product exists as a product here.** | `transaction-demo`; G2's whole predecessor chain (G4, G13, G14, G15) stops blocking | agents (uses M1's tag) |
| **M3 — Explicit live checkpoints** | After the offline proof, separately approved Claude, Codex, Cursor, and Antigravity turns pass; protected Jeryu and GitHub Candidate effects reconcile through exact read-back and observation. | all four `provider.*` gates plus `forge.jeryu` and `forge.github-app` | **OD-A, OD-B, OD-C** + agents |
| **M4 — Linux preview package** | A signed, SBOM- and provenance-bearing archive installs twice from tagged bytes on Ubuntu 24.04 x86_64, exercises systemd lifecycle/recovery, and refuses unsupported mutation platforms. This is explicitly a non-release `linux-preview`. | diagnostic Linux package, installer, operations, recovery, supply-chain, fault, and quality evidence | agents + **OD-E**, root descriptor, hosted Jankurai artifact |
| **M5 — Canonical V1 GA** | The exact five signed archives pass their release smoke, all 26 canonical gates have current admitted receipts, and the unprofiled command returns green. | every canonical V1 gate | agents + operators; five-platform builders/signers and all live approvals |

The frozen V1 contract requires all four providers, Jeryu and GitHub effect receipts, and exactly five signed
archives: Linux x86_64/aarch64, macOS x86_64/arm64, and Windows x64. Linux is the full production runner; other
platforms fail closed on real mutation until their native containment passes. No live mutation occurs without a
distinct signed operator checkpoint after the offline gates pass.

## 1. Agent-closable now (no operator input) — the work queue in dependency order

Every row names inputs that are on the development host today. Lanes are path-disjoint, coord-claimed, and committed
path-exactly with receipts. IDs refer to the inventory.

### M0 (in flight / done)
| Item | What | Status |
| --- | --- | --- |
| R-26 | Commit the runbooks / ADR 0013 / glossary / spec-mirror group | done (hub `b6ae0ca`) |
| R-36, R-37 | This plan; paper value/risk framing (R5) | this file; R5 lane in flight |
| R-01 | Reconcile the stale "seven unprojected / five projections" truth after Context Lineage | done in this commit: source, golden, and generated page say six/six |
| R-27 | All four checkouts dirty with other lanes' work | continuous: commit or hand off; no audit or paper snapshot is admissible until clean |
| R-11 | `bullet-git` fails its own audit floor (54 < 56) | lane `claude-git-audit` in flight |

### M1 (agent half)
| Item | What | Lane |
| --- | --- | --- |
| R-03, R-50 | `deny.toml` (licenses, advisories, bans, sources) in all Rust repos; `zizmor` family-wide | `claude-security-lanes` in flight |
| R-15, R-16 | Explicit MSRV-1.95 and pinned-1.97.1 lanes; resolve the toolchain contradiction; emit observation JSON in the receipt schema | `claude-toolchains` in flight |
| R-21, WI-34, WI-30 | **LANDED** (hub `3728e798`, kernel `bb420f46`, git `54d1338b`): `NoNewDispatchAfterStop` checked as a property while `NoDispatchAfterStop` is documented as a state predicate TLC refutes at depth 4; STONITH `grace < TTL` mirrored in both validators, closing the formerly accepted zero-maximum hole; GC-under-load tests mutation-proven; reflink remains designed, not implemented | done |
| R-43…R-46, R-42 | Jankurai hygiene caps (proof-lane mapping, pre-push parity, dead markers outside contract strings, zyal sentinel, supply-chain manifest) and LOC splits — per repo | git: in flight; kernel/hub/portal: next |
| R-10 | Publish per-repo Jankurai numbers (kernel 57 / git 54 / portal 60) in `release.md` | BLOCKED diff for the holder |
| R-25 | Root `docs/paper.md` sanitized to the hub copy | done (family root) |
| then | `bullet-family lock generate --tag <tag>` once OD-D exists | after OD-D |

### M2 (agent-only, the pivot) — ordered
1. **R-07 / RUNNER-FARMD-LEASE-ROUTE-AUDIT** — promote the quarantined `SignedLeaseService`: durable nonce ledger, persistent `last_acquire`, `advance_attempt_with_authority`, full-subject release check in one SQLite transaction, `/internal/v1` mount only, hardened bounded client. (cursor-grok holds the WIP today.)
2. **V1-S2-a / KERNEL-AUTHORITY-PREDECESSOR** — normalized truth: no JSON-blob authority, no `INSERT OR REPLACE`; lease/Attempt rows carry graph revision, workspace generation, scope digest, policy/routing generation, authority epoch, freeze generation.
3. **V1-S2-b** — short-lived PASETO mutation capabilities minted from the durable active lease.
4. **G4 / ONLINE-AUTHORITY-AUDIT (2)–(4)** — Kernel reservation + one-use permit; BulletGit positive checker replaces `AuthorityGateway::unavailable`; settlement after I/O. Honest wire binding needs the M1 tag (blocker 1).
5. **V1-S4-a/b** — real runner saga (acquire → read-only provider generation → proposal → BulletGit apply → admitted gates → ≤2 repairs → checkpoint → exact Candidate; heartbeat failure freezes, kills the tree, preserves, successor resumes from the exact checkpoint).
6. **V1-S4-c / WI-13 / R-55** — independent verifier: per-gate executable digest, multi-gate aggregation, oracle-modifying-diff classification with sealed holdouts.
7. **V1-S4-d** — effect broker intent → dispatch → lost-response `UNKNOWN` → read-back → identity-exact adoption, no second write (local forge).
8. **V1-S5-a** — farmd signed dispatch so a public command settles past PENDING.
9. **R-04** — replace `just demo`'s synthetic success with the signed `TRANSACTION_PROOF`; **R-08** build the tagged crash-boundary fault suite alongside step 5.

### M4/M5 (agent half)
| Item | What | Lane |
| --- | --- | --- |
| R-05 / V1-S5-d | Embed the manifest-verified Portal bundle in farmd; Playwright against the packaged origin | `claude-portal-embed` in flight |
| R-02 | `bullet-family release build` for the exact five-target matrix; checksums; SBOM (Rust + npm); provenance producer; non-circular manifest generator; release workflow (R-13) | next, after R-05 |
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
| **OD-G** | Ratify public names/endpoints (`git.neverhuman.org`, GitHub org), Jeryu deployment identity, backup, TLS (workplan WP-08/14/17) | public mirror, permalinks, hosted CI provisioning | a decision + ops |

## 3. External / platform

Canonical V1 needs clean Linux x86_64/aarch64, macOS x86_64/arm64, and Windows x64 build hosts, a checksum-pinned
portable Jankurai artifact for hosted CI (`jankurai-90` hosted half, R-12), and hosted family provisioning for
member CI (R-49, HOSTED-PORTAL-FAMILY-CI). Non-Linux builds must remain mutation-disabled until native containment
passes, but all five signed archives remain release blockers.

## 4. Contradictions to reconcile (owners named in the inventory §2)
Surface counts six vs seven (`release-truth` rows); operator-decision lists (root README / product-gaps → ADR 0013 pointers); owner vocabularies (LOCAL-then-EXTERNAL vs "Engineering, predecessor-blocked" vs `LOCAL-BLOCKED`); Rust toolchain 1.95 vs 1.97.1 per workspace; Jankurai auditor 1.6.0 artifacts vs pinned 1.6.11; `TEAM.md`/`POTENTIAL_DRAFT.md` still state C8 "any two providers" (superseded: frozen V1 requires all four); `schema-removal.md` setup-wrapper wording vs hub `3039878` default-refuse; the archived operator kit understated how close OD-A is (fixed).

## 5. Feature completeness against the vision (summary; full table in the inventory §4)
**Exists as code (COMPONENT_PROOF):** five principals; DB-clock leases/fences; idempotent graph mint; ambiguity → durable `UNKNOWN`; private dissociated clones; prior-or-complete-next generations + sealed preservation; provenance-bound `CandidateId`; signed launch grants; Linux egress isolation; four offline provider protocol subsets; 13-step live-conformance path; policy v1alpha2 rule (hub + kernel); revision-one Context Capsule; watermark-bound projections (9/15 surfaces); authenticated command ingress; two pinned TLA+ models; signed bundle verify/extract/receipt verifier; sealed setup tool subjects; coordination ledger; release-truth report.
**Design-only:** roles as capability profiles; hard-constraint routing; quota epistemology (reservations, probe, one-seat-per-human); vector fitness/selection; fusion with dissent; struggle ladder; behavior gateway; verifier backpressure; attestor ≠ broker; oracle-split + holdouts; typed freeze ack; wound-wait; staged race budgets; gates inside the egress sandbox; tree-disjoint evidence preservation; anchored audit batches; CAS/GC; topology library; SLO defaults; KPI loops; login-challenge inbox.
**Independent/deferred profiles by contract:** PostgreSQL and remote runners; cross-repo sagas; multi-tenant;
evolutionary self-tuning. The minimum typed cognitive plane is V1, but the T0-versus-T3 confirmation/canary is
post-V1. All four providers, both effects, and all five archives remain canonical V1 blockers.

## 6. Exit criteria for this plan
This plan is retired when kind-specific receipt admission is implemented and `bullet-family check release --json`
admits current signed receipts for all 26 canonical gates—including Claude, Codex, Cursor, Antigravity, Jeryu,
GitHub, and all five archives—and returns green. Diagnostic profiles may remain useful but cannot substitute for that
decision. Until then the honest status is **pre-release, blocked**, and the strongest evidence grade remains the
grade actually admitted by receipts.
