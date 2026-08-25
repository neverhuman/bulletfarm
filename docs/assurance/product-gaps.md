# Product gap register

Status: **operator index; not runtime or release authority**  
Last reviewed: 2026-08-25  
Owner: Bullet Farm maintainers

This page answers “what is still missing before Bullet Farm is a product?”
It does not make a gate green. Authoritative status remains
[`release.md`](../release.md) and the receipt rows in
[`v1-closure-plan.md`](v1-closure-plan.md). A newer commit invalidates a row
until those documents are replayed.

There is no remaining *undocumented* V1 product gap: every blocked capability
below already has an owner document, a fail-closed checker, and a typed
refusal. The remaining work is implementation, operator ratification, and
receipts — not missing prose. Closing a row in this file is not closing the
product.

## How to read a row

| Field | Meaning |
| --- | --- |
| Gap | The product capability an operator still cannot honestly claim |
| Why it is still open | The exact missing subject, not a vibe |
| What already exists | Component proof that must not be promoted |
| Closer | Who or what can close it |
| Authority | Document that may flip the status |

`bullet-family check release --json` is the executable form of the canonical
26-gate V1 GA contract. The narrower `linux-preview` profile is a non-release
diagnostic; it cannot waive any canonical provider, effect, or package gate.
If this page and the canonical command disagree, the command wins.

## What an agent may close versus what it may not

Gaps are closed only by a receipt. Prose cannot close G1–G15. Agents may close
a gap only when every closer in the row is code or a mapped test **and** no
operator secret, signer, or policy generation flip is required.

| Class | IDs | Agent action |
| --- | --- | --- |
| Operator-blocked | G1, G5, G6, G7 | Document the exact act. Do not invent a lock, flip `live_admission_enabled`, or patch Jeryu/GitHub to look green. |
| Engineering, predecessor-blocked | G2, G3, G4, G9, G12, G13, G14, G15 | Implement only behind an unexpired `coord claim`. A component receipt does not clear the family gate. |
| Quality / platform | G8, G10 | Reduce hard findings; add a native backend. A local Jankurai binary is not CI evidence. |
| Explicitly post-V1 | G11 | Keep `evolutionary_authority=false`. The `linux-preview` evolution diagnostic cannot alter the canonical V1 gate set or start an evolutionary campaign. |

A gap that is *fully specified, fail-closed, and indexed* is a **documented
open gap**, not a missing product definition. That is the only sense in which
documentation can “close” G1–G15 today.

## Remaining V1 product gaps

| ID | Gap | Why it is still open | What already exists | Closer | Authority |
| --- | --- | --- | --- | --- | --- |
| G1 | Hub-only signed install | Checked-in lock is schema 2 and is refused on purpose. The build-free `scripts/setup.sh` refuses unless the operator selects an external `bullet-family` executable, but that selection is not signed package admission. Clone transport helpers still use path-selected Git, and no production Jeryu/validator two-run, package lifecycle, or signed prebuilt exists | Descriptor-relative setup, no-replace publish, sealed Linux Cargo/Node/Bash/npm/setup-mutation/family-lock/checkout Git subjects, default-refusing wrapper, two-run component fixture | Operator publishes schema-3 lock + signed prebuilt `bullet-family`; engineering admits every remaining Git/helper subject and replays production setup | [`release.md`](../release.md), [`runbooks/source-setup.md`](../runbooks/source-setup.md) |
| G2 | Connected five-plane transaction | No signed `TRANSACTION_PROOF` covering crash, salvage, verify, ambiguous effect, integration, portal truth | Atomic lease/command/outbox; fail-closed `bullet-gitd`; fixture E2; Portal `PENDING→UNKNOWN` | Kernel + BulletGit + Portal owners land one exact offline saga | [`v1-closure-plan.md`](v1-closure-plan.md) V1-S4 |
| G3 | Production Kernel write path | Signed full-subject lease transport, durable reservation, JSON-RPC, provider/effect dispatch, CAS/GC, production restore remain | DB-clock leases; authenticated ingress; UNKNOWN/FAILED worker; launch-grant + Linux egress components | Kernel V1-S2/S4; do not remount unauthenticated `HttpLeaseClient` | [`release.md`](../release.md), [ADR 0011](../decisions/0011-signed-launch-grant-and-egress-isolation.md) |
| G4 | Production BulletGit write path | Public `clone` still returns `AUTHORITY_CONTRACT_UNAVAILABLE`; no published immutable `bullet-wire` tag, online reservation/settlement, complete Integration proof, or tagged Jeryu service | Dissociate clone, hostile-git, generations, preservation, honest cleanup UNKNOWN, and a complete provenance-bound local Candidate manifest/identity | Operator publishes wire/Jeryu tags; Kernel supplies online reservation/settlement | [`v1-closure-plan.md`](v1-closure-plan.md) V1-S3 |
| G5 | Live provider conformance | Committed policy is v1alpha1 / generation 1 / `live_admission_enabled=false`; the common production path therefore refuses before provider spawn and has no live receipt | Four bounded adapters; signed launch grant; Linux egress; common policy-to-sealed-receipt orchestration; neutral four-provider zero-spawn nightly; deep fake-process proof for Claude only | Operator ratifies v1alpha2 + runner key and proves conformant native read-only turns for Claude, Codex, Cursor, and Antigravity against the same frozen release subject | [ADR 0012](../decisions/0012-policy-v1alpha2-live-admission.md), [`runbooks/live-conformance.md`](../runbooks/live-conformance.md) |
| G6 | Jeryu live effect | No authenticated read-back/reconciliation receipt | Local bare-forge component; Jeryu adapter is typed quarantine | Operator restores scoped Jeryu auth on an unmodified forge | [`release.md`](../release.md) |
| G7 | GitHub live effect | No App-test-repo integration receipt; the frozen V1 GA contract requires this second effect adapter in addition to Jeryu | Effect adapter is specified, not certified | Operator configures a GitHub App test repository and produces an exact reconciliation receipt | [`release.md`](../release.md), [ADR 0002](../decisions/0002-jeryu-forge-requirements.md), [0008](../decisions/0008-forge-gates.md) |
| G8 | Security release floor | Hub Jankurai 58 (raw 58), 10 caps, 29 hard findings; no portable CI artifact | Pinned local scan that fails closed | Hard findings to zero; score ≥90; checksum-pinned CI binary | [`release.md`](../release.md) |
| G9 | Signed five-target release | No reproducible builder, dual SBOMs, provenance, protected signing, lifecycle smoke, or signed archives exist for the complete five-target V1 matrix | Linux verify + safe extract of an already-signed archive; Portal bundle manifest | Release engineering produces and verifies Linux x86_64/aarch64, macOS x86_64/arm64, and Windows x64 archives; Linux remains the only production mutation platform until native containment passes | [`release.md`](../release.md), [ADR 0010](../decisions/0010-supply-chain-policy.md) |
| G10 | Non-Linux containment | Mutation on macOS/Windows fails until a native backend passes | Linux is the strong-isolation reference | Platform owners | [ADR 0007](../decisions/0007-sandbox-secret-taint.md) |
| G11 | Evolutionary runtime | Self-tuning optimization and evolutionary campaigns are explicitly post-V1 | [`evolutionary-control.md`](../architecture/evolutionary-control.md); policy `evolutionary_authority=false`; a preview-only diagnostic | Keep disabled for V1. A later release may close the bounded offline study and R0/R1 canary gates before any R2+ exact signed human approval | [`phase-9-10.md`](../phase-9-10.md) |
| G12 | Family `check release` | The canonical 26-gate V1 catalog remains 26/26 `BLOCKED`; `linux-preview` separately remains 25/25 `BLOCKED` | Fail-closed canonical and named diagnostic reports; supplied generic registries cannot clear a gate | Kind-specific semantic receipt admission for every canonical V1 gate | `bullet-family check release --json`; [`release-truth.generated.md`](release-truth.generated.md) |
| G13 | Portal product surfaces | Six of fifteen spec surfaces have no durable ledger subject and stay explicit UNKNOWN; Context Lineage exposes revision-one subjects only, and Portal is not packaged/embedded | Control Tower, Mission Graph, Live Attempt, Incidents and Audit, Fleet, Session Supervisor, Merge Rail, Quality Lab, and Context Lineage projections; CSRF/202; `PENDING→UNKNOWN`; SSE STALE | Portal + farmd owners after G2/G3 add Cognitive Router, Fusion Lab, Quota/Capacity, Struggle, Behavior, and Workspace Hygiene, plus successor/compression lineage | Portal architecture; V1-S5 |
| G14 | farmd production API | Public surface is the authenticated command/snapshot/SSE subset plus six read-only operational projections, not the designed ~80-route control plane | Loopback origin, no-wildcard CORS, command 202, ready/outbox/missions plus Fleet/Session/Merge/Quality/Audit/Context snapshots | Kernel API after signed dispatch and the missing ledger subjects exist | V1-S5 |
| G15 | Cognitive persistence | One immutable revision-one Context Capsule is now normalized and atomically bound to graph materialization, lease, fence, and Attempt; CognitiveTask / SelectionGroup / Role / Fusion, quotas, budgets, routing decisions, successor lineage, and compression remain absent or design-only | Context Capsule schema/migration/replay and exact projection; wire shapes; offline provider parsers | Kernel V1-S6 after G2 | [`evolutionary-control.md`](../architecture/evolutionary-control.md) |

## Canonical 26-gate V1 catalog and diagnostic profiles

The unprofiled 26-gate catalog is the canonical frozen V1 GA contract. It
requires Claude, Codex, Cursor, Antigravity, Jeryu, GitHub App reconciliation,
and five signed archives. `linux-preview` is a deliberately narrower Ubuntu
x86_64/Jeryu/Claude diagnostic and is not release authority. Other named
provider, forge, platform, and team profiles are also diagnostic slices: a
receipt for one never certifies another or clears the canonical catalog.

The profiled JSON report uses schema 3 and names its `profile`. The current
registry boundary is intentionally conservative: an absolute registry may be
selected, but generic signed envelopes cannot clear gates until kind-specific
semantic validators and externally admitted signer/trusted-time roots exist.

The following 26 rows are the canonical V1 catalog used by the generated
portable truth page.

These IDs are the static negative inventory in `src/check/prerequisites.rs`.
Every row is `BLOCKED`. A green component crate cannot clear any of them.

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
| `release.receipt-contracts` | G9 | Release |
| `release.rust-msrv-1-95` | G9 | Release |
| `release.rust-pinned-1-97-1` | G9 | Release |
| `release.sbom` | G9 | Release |
| `release.signatures` | G9 | Release |
| `release.platform-containment` | G10 | Release |

G4, G11, G13, G14, and G15 are product gaps that are not themselves a canonical
catalog `release.*` id. G4, G13, G14, and G15 still block G2 and therefore
`release.transaction-demo`. G11 is explicitly post-V1; the extra evolution
diagnostic surfaced by `linux-preview` cannot alter the canonical 26-gate
contract. G12 is the inventory of this table. The generated page
([`release-truth.generated.md`](release-truth.generated.md)) binds this table's
26 `release.*` rows and G-ID list by digest: a crosswalk change requires
`just release-truth` in the same commit or `required` fails on drift.

## How to verify each gap is still open

From the hub checkout:

```bash
bullet-family doctor --json          # G1: BLOCKED / UNSUPPORTED_SCHEMA is honest
bullet-family check release --json   # canonical V1 GA: 26/26 BLOCKED
bullet-family check release --profile linux-preview --receipts /absolute/registry --json
just fast && just contract           # component lanes; never G2–G15
```

Do not convert a green `just fast` into a closed G-row.

## Which command proves what

| Command | Proves | Does not prove |
| --- | --- | --- |
| `just fast` | Mapped component lanes on this checkout | G2–G15, live, install, release |
| `just contract` | Generated wire/schema identity | A running issuer or published tag |
| `bullet-family doctor --json` | Honest refusal of schema-2 hub-only install | That schema-3 exists |
| `bullet-family check release --json` | The canonical 26-gate V1 GA inventory and exact blockers | That an absent/generic registry is evidence |
| `bullet-family check release --profile linux-preview --receipts <registry> --json` | A non-release Ubuntu/Jeryu/Claude diagnostic slice | Any omitted provider, GitHub, package, or canonical GA gate |
| Archived 2026-08-24 live demo | That one past tree spawned under then-policy | HEAD conformance |

`check required` adds six more static blockers (`required.installable-lock`, `required.jankurai-ratchet`, `required.packaged-browser-e2e`, `required.pinned-scans`, `required.recovery-faults`, `required.transaction-proof`). They are the same gaps, not a second product list.

## One-hop operator answers

| Question | Answer |
| --- | --- |
| How do I install from a hub-only clone? | You cannot, honestly. Schema 2 is refused. Contributor bootstrap is [`runbooks/source-setup.md`](../runbooks/source-setup.md); signed install is G1. |
| Can I turn on live Claude/Codex/Cursor/Antigravity? | Not from this tree. Ratify ADR 0012 with a real runner key (G5). The v1alpha2 validator is not a live policy. |
| Why does `doctor` fail? | The checked-in lock is schema 2. That refusal is the product. |
| Did the white paper close the product? | No. The paper inventories G1–G15. Closing prose is not a receipt. |
| Is `just fast` enough to ship? | No. It is a component lane. Canonical V1 remains 26/26 `BLOCKED`; `linux-preview` also remains 25/25 `BLOCKED`. |
| What is the same-UID install hole? | The Rust boundary seals Cargo/Node/Bash/npm plus setup mutation, family-lock verification, and checkout verification Git bytes, and the wrapper no longer invokes ambient Cargo. G1 still includes unsigned selection of the external prebuilt, clone transport Git/helpers, transient and between-child repository object/ref/index/config/file races, non-Git work-tree traversal, and allowed-signers path admission; signed prebuilt admission plus complete Git/helper isolation closes those surfaces. |
| Does ADR 0012 mean the committed policy enables live providers? | No. The Kernel loader can validate v1alpha2 since `0d848f6`, but the committed v1alpha1 generation-1 policy still disables live admission and refuses before spawn. |
| Where is `docs/INDEX.md`? | It must not exist. This family's index is [`../README.md`](../README.md). |

## Slice leftovers (V1-S0..S8)

This is the same work as G1–G15, indexed by the closure-plan slices so an
implementer cannot “lose” a leftover by reading only the G-table.

| Slice | Status class | Leftover that still blocks a product claim |
| --- | --- | --- |
| V1-S0 | Local complete; release continuous | Exact-path commits and claim receipts remain an orchestrator obligation |
| V1-S1 | LOCAL-BLOCKED | Immutable published `bullet-wire` tag; consumers still carry duplicate or legacy semantics; production JSON-RPC hello/version/frame contract |
| V1-S2 | LOCAL-BLOCKED | Normalized full truth, signed capabilities, CAS/GC, production restore admission, fault-complete recovery |
| V1-S3 | LOCAL-BLOCKED | Positive online authority/settlement; immutable shared-wire tag; complete Integration proof; reviewed tagged `jeryu-gitd` (the local Candidate manifest/identity is complete) |
| V1-S4 | LOCAL-BLOCKED | Signed internal lease transport; runner/verifier/effect saga; credential-free `TRANSACTION_PROOF` |
| V1-S5 | LOCAL-BLOCKED | APPLIED/VERIFIED dispatch; six Portal surfaces without durable ledger subjects; successor/compression Context lineage; packaged farmd-served Portal |
| V1-S6 | LOCAL-BLOCKED | Cognitive objects beyond the revision-one Context Capsule; operator-ratified native/live receipts for all four providers; quota/budget/routing/fusion replay from persisted inputs |
| V1-S7 | LOCAL-BLOCKED | Schema-3 lock, signed admission of the build-free wrapper's external executable and remaining clone Git/helper/non-Git filesystem subjects, five signed archives, SBOM/provenance, hosted Jankurai artifact, docs that wait on typed commands |
| V1-S8 | EXTERNAL-BLOCKED | Operator-issued Jeryu and GitHub effect credentials, all four provider service profiles, five-platform signing authority, and exact protected test repositories |

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
| C1 | Every invariant is T1 schema / T2 gateway / T3 test | Registry exists; most later-wave rows remain `planned` |
| C2 | Oracle-modifying diffs + required holdouts for R2+ | Designed; implemented verifier is one fixture E2 |
| C3 | Two-track scope expansion ([ADR 0004](../decisions/0004-scope-amendment-tracks.md)) | Accepted decision; not a live Attempt path |
| C4 | Attestor ≠ broker; reconstructible check from proof bundle | Designed; live forge blocked |
| C5 | `CONTRADICTORY` / prolonged `UNKNOWN` has fence-mediated exits | Designed |
| C6 | Bounded probe reservation; `unknown` is never headroom | Designed |
| C6b | One seat-equivalent per named human | Designed; not enforced in the kernel |
| C7 | Formal-model exactly two protocols in Phase 0 | Adopted and component-complete |
| C8 | Historical proposal: GA = kernel + any two certified providers | Superseded by the frozen V1 contract requiring conformant Claude, Codex, Cursor, and Antigravity adapters |
| C9 | Identity-exact effect adoption (fence + desired OID) | Command idempotency component; graph mint not a live path |
| C10 | Verifier dwell is writer-admission backpressure | Designed |
| C11 | Freeze chip shows recorded vs enforced-on-N/M runners | Portal honesty component; freeze countdown designed |
| C12 | Multi-repo saga quarantines blast radius, not the fleet | Out of V1; [`phase-9-10.md`](../phase-9-10.md) |

Rejected critiques stay rejected: do not drop the verifier plane, do not
mandate Postgres for V1, do not collapse the five planes, do not replace
forge sovereignty with an internal merge queue.

## Documentation that is closed (do not reopen as a gap)

These used to look like missing product definition. They are defined and
fail-closed.

| Topic | Where it is closed |
| --- | --- |
| Public name, five planes, providers-propose | [`architecture/overview.md`](../architecture/overview.md), [ADR 0001](../decisions/0001-provider-execution-mode.md), [0003](../decisions/0003-five-trust-planes.md) |
| Competitor pins (Gas Town, Gas City, DeepSeek, Omnigent) | [`competitor-snapshot.md`](competitor-snapshot.md) |
| IEEE preprint source | [`../paper/`](../paper/) |
| Why authority-bearing evolution is post-V1 | [`evolutionary-control.md`](../architecture/evolutionary-control.md) |
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
- That finishing the white paper, or this register, closed G1–G15.
