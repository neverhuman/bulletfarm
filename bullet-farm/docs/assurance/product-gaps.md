# Product gap register

Status: **operator index; not runtime or release authority**  
Last reviewed: 2026-08-27
Owner: Bullet Farm maintainers

This page answers “what is still missing before Bullet Farm is a product?”
It does not make a gate green. Authoritative status remains
[`release.md`](../release.md), the explicitly profiled release command, and
generated [`release-truth.generated.md`](release-truth.generated.md). The active
dependency order is [`closure-roadmap.md`](closure-roadmap.md). A newer commit
invalidates a row until those sources are replayed.

This is the best-known current gap inventory, not a completeness proof. Every
blocked capability listed below has an owner document, a fail-closed checker,
and a typed refusal, but the Wave-0 bidirectional implementation↔invariant
inventory may discover additional orphaned requirements or enforcement sites.
Any discovery becomes a new explicit row before implementation proceeds.
Closing a row in this file is not closing the product.

## How to read a row

| Field | Meaning |
| --- | --- |
| Gap | The product capability an operator still cannot honestly claim |
| Why it is still open | The exact missing subject, not a vibe |
| What already exists | Component proof that must not be promoted |
| Closer | Who or what can close it |
| Authority | Document that may flip the status |

`bullet-family check release --profile self-hosted-v1 --receipts
<admitted-absolute-registry> --json` is the first-GA decision. The later
`universal-v1` composition adds every provider, forge, and platform profile
without implicitly adding evolution. `legacy-v1-26` and `linux-preview` are
diagnostics only. If this page and an explicitly profiled command disagree, the
command wins.

## What an agent may close versus what it may not

Gaps are closed only by a receipt. Prose cannot close G1–G18. Agents may close
a gap only when every closer in the row is code or a mapped test **and** no
operator secret, signer, or policy generation flip is required.

| Class | IDs | Agent action |
| --- | --- | --- |
| Engineering then operator | G1, G5, G6, G7, G16 | Implement and prove only the local producer/admission half behind an unexpired claim. Then document the exact operator act; do not invent a lock, flip `live_admission_enabled`, credentials, or patch a forge to look green. |
| Engineering, predecessor-blocked | G2, G3, G4, G9, G12, G13, G14, G15, G17, G18 | Implement only behind an unexpired `coord claim`. A component receipt does not clear the family gate. |
| Quality / platform | G8, G10 | Reduce hard findings; add a native backend. A local Jankurai binary is not CI evidence. |
| Dependent certification | G11 | Keep `evolutionary_authority=false` through `self-hosted-v1`; implement and certify `evolution-v1` separately afterwards. Universal never implies it. |

A gap that is *fully specified, fail-closed, and indexed* is a **documented
open gap**, not a missing product definition. That is the only sense in which
documentation can “close” G1–G18 today.

## Remaining V1 product gaps

| ID | Gap | Why it is still open | What already exists | Closer | Authority |
| --- | --- | --- | --- | --- | --- |
| G1 | Hub-only signed install | Checked-in lock is schema 2 and is refused on purpose. The build-free `scripts/setup.sh` refuses unless the operator selects an external `bullet-family` executable, but that selection is not signed package admission. Clone transport helpers still use path-selected Git, and no production Jeryu/validator two-run, package lifecycle, or signed prebuilt exists | Descriptor-relative setup, no-replace publish, sealed Linux Cargo/Node/Bash/npm/setup-mutation/family-lock/checkout Git subjects, default-refusing wrapper, two-run component fixture | Engineering admits every remaining Git/helper subject and production lifecycle transition; then release custody publishes the signed schema-3 lock and prebuilt `bullet-family` and runs the fresh-host replay | [`release.md`](../release.md), [`runbooks/source-setup.md`](../runbooks/source-setup.md) |
| G2 | Connected five-plane transaction | No signed `TRANSACTION_PROOF`; the retained public loop reaches durable `UNKNOWN` through a harness-process executor and fixture keys, with independent, transaction, and release eligibility hard false. Trusted key lifecycle and durable nonce consumption, distinct verifier/effect UIDs and credential custody, independently owned artifacts, transaction-grade public dispatch and Portal truth, and the twelve-boundary campaign remain absent | A retained exact-digest command connects durable ScopeGrant admission, peer-authenticated farmd/Runner, Kernel-issued Candidate grant/final check, production Gitd one-use Candidate preparation, fixture writer refusal + PASS, purpose-signed PASETO v4.public/JCS `VerificationIntentV1`, `EvidenceV1`, `ProofBundleV1`, and caller-free `MATCHED` `ObservationV1` over the exact Candidate/base/head/tree, ProofBundle/check/protection/integration/target subjects with reconstructed ephemeral public keys and canonical-chain digests; exact Candidate-head `LocalBareForge` delivery/read-back, stale-fence refusal, lost-response `UNKNOWN`→`COMMITTED`, protected expected-old-OID integration, and reopen read-back; and private retained source/Candidate/target Git plus ledger artifacts whose exact Git subjects are independently reopened by the shell after child exit. A separate retained public wrapper authenticates exact duplicate `run_demo` POSTs, survives farmd restart, replays and polls through the packaged Portal, dispatches the same request through a registered same-UID `SO_PEERCRED` UDS Runner and bounded exact worker, admits that fixture receipt, settles the same command/request/receipt digest durably to `UNKNOWN`, and reads back `NO_COMMAND` after worker restart | Independent verification/effect/audit owners reissue the existing subjects under registered keys, durable nonces, distinct UID/credential custody, and independently owned artifacts; semantic admission, transaction-grade farmd/Portal, and chaos owners close the remaining W6/W7 path | [`closure-roadmap.md`](closure-roadmap.md) Waves 2–7; [`runbooks/dogfood.md`](../runbooks/dogfood.md) |
| G3 | Production Kernel write path | Authenticated public `run_demo` now settles `PENDING`→durable `UNKNOWN` through the retained private component path and survives farmd and worker restart. It still uses same-UID fixture custody; operator-admitted long-lived peer/key custody, durable recovery/read-back beyond the private proof root, production provider/effect dispatch, CAS/GC, and production restore remain absent | DB-clock leases; authenticated ingress; exact duplicate public POST; product-provisioned private signing key; durable private peer registry and server grant/nonce state; peer- and socket-bound signed UDS component; bounded exact worker state/read-back; exact Candidate grant/final-check component; retained fixture receipt; UNKNOWN/FAILED worker; launch-grant + Linux egress components | Kernel V1-S2/S4 promotes the proven private path with operator-admitted durable registry/key custody, distinct service identities, recovery outside the fixture root, and production command/provider/effect dispatch; do not remount unauthenticated `HttpLeaseClient` | [`release.md`](../release.md), [ADR 0011](../decisions/0011-signed-launch-grant-and-egress-isolation.md) |
| G4 | Production BulletGit write path | Ordinary public `clone` remains fail-closed outside the scoped Kernel path; there is no published immutable `bullet-wire` tag, admitted installed authority custody, signed Integration proof, or tagged Jeryu service | The retained bridge uses the production daemon with a Kernel-issued exact Candidate grant/final check to prepare one one-use Candidate, refuses stale fence, preserves that exact Candidate/head through delivery, local exact-SHA check, protected integration, signed fixture Observation, and reopen read-back, then retains private source/Candidate/LocalBare ordinary-Git subjects and independently reopens their exact HEAD/tree after child exit; dissociate clone, hostile-git, generations, preservation, and honest cleanup UNKNOWN remain component-proved | Operator publishes wire/Jeryu tags; Kernel/BulletGit owners bind the retained permit/final-check path to installed durable custody and signed Integration evidence | [`closure-roadmap.md`](closure-roadmap.md) Waves 1 and 4 |
| G5 | Live provider conformance | The committed v1alpha1 policy refuses at `POLICY`. A structurally valid v1alpha2 policy reaches the production adapter observation port but every adapter defaults to `RUNTIME_PROBE_UNAVAILABLE` at `ADMISSION`, before operator-key read, graph/Mission, lease, or nonce writes, egress preparation, or child spawn. No separately authorized and contained read-only probe, complete external policy/enrollment anchor, provider onboarding, or semantic live-receipt registration exists | Four bounded adapters; signed launch-grant and Linux-egress components; sealed 13-step refusal receipts; neutral four-provider zero-spawn nightly; positive PONG/conformance synthesis only in a strict `cfg(test)` dispatcher | Engineering lands a separately granted and contained real runtime probe, hostile-tested schema-3 policy/enrollment-anchor admission, provider onboarding, and semantic sealed-receipt registration; then the operator ratifies the policy and enrollments, supplies exact executables/profiles/credentials, and proves native read-only turns against the same frozen release subject | [ADR 0012](../decisions/0012-policy-v1alpha2-live-admission.md), [`runbooks/live-conformance.md`](../runbooks/live-conformance.md) |
| G6 | Jeryu live effect | No authenticated external Jeryu check, protected integration/target read-back, signed Observation, reconciliation, backup/restore, or drift receipt | The retained `LocalBareForge` component delivers/read-backs the exact Candidate, publishes/read-backs its exact-SHA check/ProofBundle root, performs protected expected-old-OID integration, purpose-signs caller-free target outcome `MATCHED` and reverifies it, reconciles lost response, reopens the same records, and retains/reopens its private Git target after child exit; the external Jeryu adapter remains typed quarantine and no local fixture substitutes for it | Engineering completes typed Jeryu probes and semantic receipt admission over the same port; the operator later restores scoped auth on an unmodified pinned forge and registers exact integration/reconciliation/backup/restore/drift receipts | [`release.md`](../release.md) |
| G7 | GitHub live effect | No App-test-repo integration receipt exists for the independent GitHub adapter profile | Effect adapter is specified, not certified | Engineering lands the typed capability/delivery/check/integration/read-back/reconciliation adapter and semantic receipt admission; then the operator configures a GitHub App test repository with role-separated credentials and registers the exact receipt | [`release.md`](../release.md), [ADR 0002](../decisions/0002-jeryu-forge-requirements.md), [0008](../decisions/0008-forge-gates.md) |
| G8 | Security release floor | Historical Hub Jankurai checkpoint 65 (raw 66), five caps, 21 high/hard and 27 medium/soft findings; no portable hosted CI artifact | Pinned local scan that fails closed | Score ≥90 with zero caps and zero hard findings; checksum-pinned CI binary and portable exact-subject report | [`release.md`](../release.md) |
| G9 | Signed profile-selected release | No profile has an admitted signed package set. One quarantined component builds an unsigned Linux x86_64 archive; the frozen verifier expects the later five-target universal envelope and cannot admit the first-GA single-target profile | The component builder embeds the Portal and eight binaries, then re-reads its archive, CycloneDX SBOM, provenance, BLAKE3 checksums, and non-circular build manifest; signed-bundle verify + safe extract also exist as incompatible components | Release engineering first produces and lifecycle-smokes the signed Ubuntu 24.04 x86_64/systemd archive selected by `self-hosted-v1`; independent platform profiles add four targets and later `universal-v1` composes all five | [`release.md`](../release.md), [ADR 0010](../decisions/0010-supply-chain-policy.md) |
| G10 | Platform containment | Linux production containment has component proof only; every selected target still lacks an admitted target/profile-bound containment or typed refusal receipt | Linux is the strong-isolation reference; non-Linux mutation is fail-closed | Platform owners certify each named platform independently; `self-hosted-v1` selects Linux x86_64 only and `universal-v1` composes all five | [ADR 0007](../decisions/0007-sandbox-secret-taint.md) |
| G11 | Evolutionary runtime | `evolution-v1` has no study, canary, promotion, or rollback receipt and is deliberately not selected by self-hosted or universal | [`evolutionary-control.md`](../architecture/evolutionary-control.md); policy `evolutionary_authority=false` | After `self-hosted-v1`, complete the frozen offline study, no-effect shadow, and rollback-readiness proof; OD-H then authorizes one exact expiring ≤1% R0/R1 canary, followed by independent canary, promotion, drift, and rollback receipts | [`closure-roadmap.md`](closure-roadmap.md) Wave 9 |
| G12 | Family `check release` | Every named product profile remains `BLOCKED`; the historical `legacy-v1-26` and `linux-preview` diagnostics also remain blocked | Fail-closed explicit profiles and reports; supplied generic registries cannot clear a gate | Both kind-specific semantic admission through `release.receipt-contracts` and the explicitly requested profile-condition receipt over its exact dependency closure; neither half substitutes for the other | `bullet-family check release --profile <profile> --receipts <admitted-absolute-registry> --json`; [`release-truth.generated.md`](release-truth.generated.md) |
| G13 | Portal product surfaces | Six of fifteen spec surfaces have no durable ledger subject and stay explicit UNKNOWN; Context Lineage exposes revision-one subjects only; same-origin embedding is component-proved, but no signed package or installation receipt exists | Control Tower, Mission Graph, Live Attempt, Incidents and Audit, Fleet, Session Supervisor, Merge Rail, Quality Lab, and Context Lineage projections; CSRF/202; `PENDING→UNKNOWN`; SSE STALE; packaged-farmd browser proof | Portal + farmd owners after G2/G3 add Cognitive Router, Fusion Lab, Quota/Capacity, Struggle, Behavior, and Workspace Hygiene, plus successor/compression lineage; release engineering supplies the signed package/install receipt under G9 | [`closure-roadmap.md`](closure-roadmap.md) Waves 6 and 9 |
| G14 | farmd production API | Authenticated `/api/v1/commands` dispatch for exact `run_demo` is component-proved from `PENDING` to durable `UNKNOWN` with `COMPONENT_PROOF_NOT_TRANSACTION_ELIGIBLE`, but only through same-UID private fixture custody. The outer receipt is `UNSIGNED_FIXTURE`, nested records are `FIXTURE_KEY_ONLY`, every eligibility flag is hard false, and there is no process-level response-loss hook, twelve-boundary chaos campaign, signed transaction receipt, independent executor/effect identity, or complete designed control plane | Loopback origin, no-wildcard CORS, command 202, ready/outbox/missions plus Fleet/Session/Merge/Quality/Audit/Context snapshots; exact duplicate POST before restart; farmd restart; packaged Portal duplicate replay/poll; registered `SO_PEERCRED` UDS Runner claim; bounded exact worker; retained fixture transaction receipt; atomic `UNKNOWN` settlement whose command ID, request digest, and raw-receipt BLAKE3 agree in public GET and Portal; worker restart `NO_COMMAND`; post-exit retained-artifact reopen | Promote this component only after the minimal offline G2 transaction, independent custody, semantic admission, and response-loss/chaos exits settle honestly; add routes only after each missing ledger subject exists | [`closure-roadmap.md`](closure-roadmap.md) Waves 2, 5, and 6 |
| G15 | Cognitive persistence | One immutable revision-one Context Capsule is now normalized and atomically bound to graph materialization, lease, fence, and Attempt; CognitiveTask / SelectionGroup / Role / Fusion, quotas, budgets, routing decisions, successor lineage, and compression remain absent or design-only | Context Capsule schema/migration/replay and exact projection; wire shapes; offline provider parsers | Kernel evolution owners after the self-hosted substrate | [`closure-roadmap.md`](closure-roadmap.md) Wave 9; [`evolutionary-control.md`](../architecture/evolutionary-control.md) |
| G16 | GitLab adapter effects | Neither GitLab.com nor one exact self-managed GitLab endpoint/version has a typed effect adapter or a protected integration, exact-SHA status, read-back, UNKNOWN reconciliation, or drift receipt | Two explicit profile nodes and structural receipt bindings; no live adapter proof | Effects owners land the typed capability/delivery/check/integration/read-back/reconciliation adapters and semantic receipt admission; then operators provide scoped test projects and credentials and certify GitLab.com and self-managed GitLab separately | [`closure-roadmap.md`](closure-roadmap.md) Waves 4, 5, and 10 |
| G17 | Distributed team mode | PostgreSQL, remote runners, workload mTLS/SPIFFE, replicated projections, object storage, partition/failover, and distributed restore are absent | `team-v1` is an explicit fail-closed profile depending on self-hosted | Distributed runtime owners implement and certify `team-v1` only after self-hosted | [`closure-roadmap.md`](closure-roadmap.md) Wave 11 |
| G18 | Cross-repository sagas | No staged multi-repository Candidate, dependency-aware quarantine, compensation, or forward-repair transaction exists | `saga-v1` is an explicit fail-closed profile depending on team mode | Saga owners implement and certify `saga-v1` only after `team-v1` | [`closure-roadmap.md`](closure-roadmap.md) Wave 11 |

## Historical 26-gate catalog and release profiles

`self-hosted-v1` is the first GA profile: Ubuntu 24.04 x86_64/systemd, Claude,
and local Jeryu. `evolution-v1` depends on self-hosted and certifies separately.
Provider, forge, and platform profiles are independent. The later
`universal-v1` composition requires all four providers, Jeryu, GitHub,
GitLab.com, self-managed GitLab, and all five platforms without implicitly
admitting evolution, team, or saga. `legacy-v1-26` and `linux-preview` are
diagnostics only. A receipt for one profile never certifies another.

The profiled JSON report uses schema 3 and names its `profile`. The current
registry boundary is intentionally conservative: an absolute registry may be
selected, but generic signed envelopes cannot clear gates until kind-specific
semantic validators and externally admitted signer/trusted-time roots exist.

The first 26 rows are the historical catalog preserved by the generated
portable page.

These IDs are the static negative inventory preserved by `legacy-v1-26` in
`src/check/prerequisites.rs`. Every row is `BLOCKED`. A green component crate
cannot clear any of them, and this historical list is not a substitute for
evaluating the full `universal-v1` dependency closure.

| Gate ID | Product gap | Class |
| --- | --- | --- |
| `release.installable-lock` | G1 | Release |
| `release.installer-twice` | G1 | Release |
| `release.transaction-demo` | G2 | Transaction |
| `release.fault-suite` | G2, G3 | Release |
| `release.backup-restore` | G3, G9 | Release |
| `release.provider.claude` | G5 | Live |
| `release.provider.codex` | G5 | Live |
| `release.provider.cursor` | G5 | Live |
| `release.provider.antigravity` | G5 | Live |
| `release.forge.jeryu` | G6 | Live |
| `release.forge.github-app` | G7 | Live |
| `release.jankurai-90` | G8 | Release |
| `release.scan.dependency` | G8, G9 | Release |
| `release.scan.license` | G8, G9 | Release |
| `release.scan.secret` | G8, G9 | Release |
| `release.scan.workflow` | G8, G9 | Release |
| `release.checksums` | G9 | Release |
| `release.manifest-non-circular` | G9 | Release |
| `release.package-matrix` | G9 | Release |
| `release.provenance` | G9 | Release |
| `release.receipt-contracts` | G12 | Release |
| `release.rust-msrv-1-95` | G9 | Release |
| `release.rust-pinned-1-97-1` | G9 | Release |
| `release.sbom` | G9 | Release |
| `release.signatures` | G9 | Release |
| `release.platform-containment` | G10 | Release |

Through its Linux x86_64 platform dependency, `universal-v1` also selects two
native lifecycle gates. Both remain blocked; neither is implied by the generic
platform condition.

| Gate ID | Product gap | Class |
| --- | --- | --- |
| `release.package-linux-x86_64` | G9 | Release |
| `release.systemd-v1` | G1, G9 | Release |

The global product-profile catalog has eighteen condition gates. The universal
projection selects fifteen of them; evolution, team, and saga remain separate.
Conditions make dependency closure visible; they do not duplicate or clear the
historical or native capability gates above.

| Gate ID | Product gap | Class |
| --- | --- | --- |
| `release.profile.evolution-v1` | G11, G13, G14, G15 | Release |
| `release.profile.github-adapter-v1` | G7 | Release |
| `release.profile.gitlab-adapter-v1` | G16 | Release |
| `release.profile.gitlab-self-managed-v1` | G16 | Release |
| `release.profile.jeryu-forge-v1` | G6 | Release |
| `release.profile.platform-linux-aarch64` | G9, G10 | Release |
| `release.profile.platform-linux-x86_64` | G9, G10 | Release |
| `release.profile.platform-macos-aarch64` | G9, G10 | Release |
| `release.profile.platform-macos-x86_64` | G9, G10 | Release |
| `release.profile.platform-windows-x86_64` | G9, G10 | Release |
| `release.profile.provider-antigravity` | G5 | Release |
| `release.profile.provider-claude` | G5 | Release |
| `release.profile.provider-codex` | G5 | Release |
| `release.profile.provider-cursor` | G5 | Release |
| `release.profile.saga-v1` | G18 | Release |
| `release.profile.self-hosted-v1` | G1, G2, G3, G5, G6, G8, G9, G10, G13, G14 | Release |
| `release.profile.team-v1` | G17 | Release |
| `release.profile.universal-v1` | G1, G2, G3, G5, G6, G7, G8, G9, G10, G16 | Release |

G4, G13, G14, and G15 have no dedicated historical catalog `release.*` id. G4
blocks G2 and therefore `release.transaction-demo`. G13 and G14 follow the
minimal authenticated offline transaction, so they do not form a cycle by
blocking their own prerequisite. They are nevertheless mechanically owned by
the `self-hosted-v1` condition (durable or typed `OUT_OF_PROFILE`) and also by
the `evolution-v1` condition (all fifteen surfaces durable). G15 is owned only
by `evolution-v1`: a Wave 6 `OUT_OF_PROFILE` projection is honest but does not
close cognitive persistence before Wave 9. Universal inherits the self-hosted
condition but never implies evolution. G11 and G16–G18 are owned by
their explicit profile-condition gates. G12 is the inventory of these tables
plus semantic admission for profiles that evaluate a selected registry;
`legacy-v1-26` is inventory-only.
The generated universal page
([`release-truth.generated.md`](release-truth.generated.md)) selects 43 gates
while binding all 46 global `release.*` crosswalk rows and the 18-G-id list by
digest: a crosswalk change requires
`just release-truth` in the same commit or `required` fails on drift.

## How to verify each gap is still open

From the hub checkout:

```bash
bullet-family doctor --json          # G1: BLOCKED / UNSUPPORTED_SCHEMA is honest
bullet-family check release --profile self-hosted-v1 --receipts /absolute/admitted-registry --json
bullet-family check release --profile evolution-v1 --receipts /absolute/admitted-registry --json
bullet-family check release --profile universal-v1 --receipts /absolute/admitted-registry --json
bullet-family check release --profile legacy-v1-26 --receipts /absolute/empty-registry --report --portable
bullet-family check release --profile linux-preview --receipts /absolute/registry --json
just fast && just contract           # component lanes; never G2–G18
```

Do not convert a green `just fast` into a closed G-row.

## Which command proves what

| Command | Proves | Does not prove |
| --- | --- | --- |
| `just fast` | Mapped component lanes on this checkout | G2–G18, live, install, release |
| `just contract` | Generated wire/schema identity | A running issuer or published tag |
| `bullet-family doctor --json` | Honest refusal of schema-2 hub-only install | That schema-3 exists |
| `bullet-family check release --profile self-hosted-v1 --receipts <admitted-absolute-registry> --json` | First-GA Ubuntu/Jeryu/Claude closure and exact blockers | Any independent provider, hosted forge, non-Linux platform, or evolution profile |
| `bullet-family check release --profile universal-v1 --receipts <admitted-absolute-registry> --json` | Later maximum-scope composition and exact blockers | Evolution, team, saga, or evidence from an absent/generic registry |
| `bullet-family check release --profile legacy-v1-26 --receipts <empty-registry> --report --portable` | The historical 26-gate diagnostic projection and exact blockers | Complete `universal-v1` release authority |
| `bullet-family check release --profile linux-preview --receipts <registry> --json` | A non-release Ubuntu/Jeryu/Claude diagnostic slice | Any omitted provider, GitHub, package, or canonical GA gate |
| Archived 2026-08-24 live demo | That one past tree spawned under then-policy | HEAD conformance |

`check required` adds six more static blockers (`required.installable-lock`, `required.jankurai-ratchet`, `required.packaged-browser-e2e`, `required.pinned-scans`, `required.recovery-faults`, `required.transaction-proof`). They are the same gaps, not a second product list.

## One-hop operator answers

| Question | Answer |
| --- | --- |
| How do I install from a hub-only clone? | You cannot, honestly. Schema 2 is refused. Contributor bootstrap is [`runbooks/source-setup.md`](../runbooks/source-setup.md); signed install is G1. |
| Can I turn on live Claude/Codex/Cursor/Antigravity? | Not from this tree. Even a structurally valid v1alpha2 policy stops at `ADMISSION` with `RUNTIME_PROBE_UNAVAILABLE` before operator-key read, graph/Mission, lease, or nonce writes, egress preparation, or spawn. Engineering must first land a separately granted and contained real probe, schema-3 policy/enrollment-anchor admission, provider onboarding, and semantic receipt registration; only then may operators ratify ADR 0012, enroll exact providers, and supply profiles/credentials for native runs (G5). |
| Why does `doctor` fail? | The checked-in lock is schema 2. That refusal is the product. |
| Did the white paper close the product? | No. The paper's G1–G15 inventory is extended here by explicit GitLab/team/saga profile gaps G16–G18. Closing prose is not a receipt. |
| Is `just fast` enough to ship? | No. It is a component lane. First-GA `self-hosted-v1`, later `universal-v1`, the historical 26-gate projection, and the `linux-preview` diagnostic all remain `BLOCKED`. |
| What is the same-UID install hole? | The Rust boundary seals Cargo/Node/Bash/npm plus setup mutation, family-lock verification, and checkout verification Git bytes, and the wrapper no longer invokes ambient Cargo. G1 still includes unsigned selection of the external prebuilt, clone transport Git/helpers, transient and between-child repository object/ref/index/config/file races, non-Git work-tree traversal, and allowed-signers path admission; signed prebuilt admission plus complete Git/helper isolation closes those surfaces. |
| Does ADR 0012 mean the committed policy enables live providers? | No. The committed v1alpha1 generation-1 policy refuses at `POLICY`; a shape-valid v1alpha2 policy instead reaches the default adapter observation port and refuses with `RUNTIME_PROBE_UNAVAILABLE` at `ADMISSION`. Neither path reads the operator key, writes graph/Mission, lease, or nonce state, prepares egress, or spawns a provider, and neither is LIVE_PROOF. |
| Where is `docs/INDEX.md`? | It must not exist. This family's index is [`../README.md`](../README.md). |

## Slice leftovers (V1-S0..S8)

This is the same predecessor work as G1–G18, indexed by the closure-plan slices so an
implementer cannot “lose” a leftover by reading only the G-table.

| Slice | Status class | Leftover that still blocks a product claim |
| --- | --- | --- |
| V1-S0 | Local complete; release continuous | Exact-path commits and claim receipts remain an orchestrator obligation |
| V1-S1 | LOCAL-BLOCKED | Immutable published `bullet-wire` tag; consumers still carry duplicate or legacy semantics; production JSON-RPC hello/version/frame contract |
| V1-S2 | LOCAL-BLOCKED | Normalized full truth, signed capabilities, CAS/GC, production restore admission, fault-complete recovery |
| V1-S3 | LOCAL-BLOCKED | Positive online authority/settlement; immutable shared-wire tag; complete Integration proof; reviewed tagged `jeryu-gitd` (the local Candidate manifest/identity is complete) |
| V1-S4 | LOCAL-BLOCKED | Signed internal lease transport; runner/verifier/effect saga; credential-free `TRANSACTION_PROOF` |
| V1-S5 | LOCAL-BLOCKED | APPLIED/VERIFIED dispatch; six Portal surfaces without durable ledger subjects; successor/compression Context lineage; packaged farmd-served Portal |
| V1-S6 | LOCAL-BLOCKED | Cognitive objects beyond the revision-one Context Capsule; schema-3 provider policy/enrollment-anchor admission, provider onboarding/runtime probing, semantic receipt registration, and quota/budget/routing/fusion replay from persisted inputs |
| V1-S7 | LOCAL-BLOCKED | Schema-3 lock, signed admission of the build-free wrapper's external executable and remaining clone Git/helper/non-Git filesystem subjects, the profile-selected signed archive set (one for first GA; five only for universal), SBOM/provenance, hosted Jankurai artifact, docs that wait on typed commands |
| V1-S8 | EXTERNAL-BLOCKED | Operator-issued first-GA Jeryu/Claude authority and Ubuntu signing custody; later independent GitHub/GitLab, three-provider, and four-additional-platform authority; exact protected test repositories |

V1 is done only when every `V1-S0..S8` gate has a current independently
verifiable receipt from the same signed subjects. “Agents ran” is not that
receipt.

## Operator decisions that are not code

The authoritative list, owners, evidence required, and expiry/reversal rules live in
[ADR 0013](../decisions/0013-operator-decision-register.md). Agents must not copy that register into status prose,
flip live/evolutionary policy, alter the running forge, invent schema-3 subjects, or substitute local credentials.

## Centerrail C1–C12: product status

The family `TEAM.md` red-team is historical provenance. Living control is
[`evolutionary-control.md`](../architecture/evolutionary-control.md) plus this
register. Disposition of each critique:

| ID | Adopted meaning | Product status |
| --- | --- | --- |
| C1 | Every entry declared in the registry has one T1 schema / T2 gateway / T3 test primary tier | Validator proves registry-internal completeness only; the whole-product bidirectional orphan inventory remains open Wave-0 work |
| C2 | Oracle-modifying diffs + required holdouts for R2+ | Designed; implemented verifier is one fixture E2 |
| C3 | Two-track scope expansion ([ADR 0004](../decisions/0004-scope-amendment-tracks.md)) | Accepted decision; not a live Attempt path |
| C4 | Attestor ≠ broker; reconstructible check from proof bundle | Designed; live forge blocked |
| C5 | `CONTRADICTORY` / prolonged `UNKNOWN` has fence-mediated exits | Designed |
| C6 | Bounded probe reservation; `unknown` is never headroom | Designed |
| C6b | One seat-equivalent per named human | Designed; not enforced in the kernel |
| C7 | Formal-model exactly two protocols in Phase 0 | Adopted and component-complete |
| C8 | Historical proposal: GA = kernel + any two certified providers | Superseded: first-GA `self-hosted-v1` names Claude exactly; independent provider profiles name Codex, Cursor, and Antigravity; later `universal-v1` composes all four, and no receipt substitutes for another |
| C9 | Identity-exact effect adoption (fence + desired OID) | Command idempotency component; graph mint not a live path |
| C10 | Verifier dwell is writer-admission backpressure | Designed |
| C11 | Freeze chip shows recorded vs enforced-on-N/M runners | Portal honesty component; freeze countdown designed |
| C12 | Multi-repo saga quarantines blast radius, not the fleet | Explicit `saga-v1` profile after `team-v1`; [closure roadmap](closure-roadmap.md) Wave 11 |

Rejected critiques stay rejected: do not drop the verifier plane, do not
mandate Postgres for V1, do not collapse the five planes, do not replace
forge sovereignty with an internal merge queue.

## Documentation that is closed (do not reopen as a gap)

These used to look like missing product definition. They are defined and
fail-closed.

| Topic | Where it is closed |
| --- | --- |
| Public name, five planes, providers-propose | [`architecture/overview.md`](../architecture/overview.md), [ADR 0001](../decisions/0001-provider-execution-mode.md), [0003](../decisions/0003-five-trust-planes.md) |
| Competitor pins (README and paper: Gas Town, Gas City, DeepSeek Harness, Omnigent) | [Dated README snapshot](competitor-snapshot.md); [paper evidence lock](../paper/evidence.json) |
| IEEE preprint source | [`../paper/`](../paper/) |
| Why authority-bearing evolution is independently certified | [`evolutionary-control.md`](../architecture/evolutionary-control.md) |
| Evidence classes and skip-green ban | [`testing.md`](../testing.md) |
| C1–C12 / TEAM.md red-team | This page + paper Section “Red-Team Disposition”; `TEAM.md` is provenance |
| Mascot / brand briefs | [`../brand/mascots/`](../brand/mascots/) |
| Family-root `/docs/*.md` copies | Not authority. Hub `bullet-farm/docs/` wins |

## Documentation that must wait on typed commands

These are not missing definitions. Writing them now would invent a CLI that
does not exist. They become runbook work after the named command is real.

| Deferred doc | Blocked on |
| --- | --- |
| Upgrade / rollback / uninstall runbook | Signed prebuilt installer (G1, G9) |
| Signer rotation and schema-removal runbook | Release signing keys (G9) |
| SAFE_STOPPED / freeze-enforced operator card | Runner ack generation (C11, G3) |
| Effect-reconciliation operator card | Identity-exact adoption path (C9, G2) |
| Platform-refusal after mutation attempt | Native backends (G10) |
| Live-admission operator recipe | ADR 0012 ratification (G5) |
| Jeryu restore / read-back recipe | Operator forge auth (G6) |
| GitHub App test-repo recipe | Operator App + protected repo (G7) |
| Launch-grant keygen recipe | ADR 0011 + operator key (G5) |
| Jankurai CI pin admission | Portable checksum-pinned binary (G8) |
| `bullet-wire` / `jeryu-gitd` publication | Operator tags (G4) |
| Generated `check release` report as a committed score | Forbidden; the command wins |

## What this page will never say

- That a green `just fast` or a green component crate is a release.
- That the archived 2026-08-24 live demo certifies HEAD.
- That Bullet Farm is measured faster, cheaper, or safer than Gas Town,
  DeepSeek Harness, or Omnigent.
- That an agent may enable live admission or invent a schema-3 lock.
- That finishing the white paper, or this register, closed G1–G18.
