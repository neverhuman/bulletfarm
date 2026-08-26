# Historical Safety-Complete V1 checkpoint

Status: **SUPERSEDED on 2026-08-25; historical component inventory only**
Owner: Bullet Farm maintainers
Last reconciled: 2026-08-25
Scope: preserved point-in-time four-repository component inventory

> The scope order, commands, counts, OIDs, “V1” labels, and terminal criteria
> below are frozen provenance and are not current launch authority. Use the
> active [closure roadmap](closure-roadmap.md), the
> [product-gap register](product-gaps.md), and an explicitly profiled
> `bullet-family check release` decision. Current order is
> `TRANSACTION_PROOF` → first-GA `self-hosted-v1` → separate
> `evolution-v1` (Wave 9) → independent profiles and later `universal-v1`
> (Wave 10) → `team-v1` → `saga-v1` (Wave 11).
> Every profile remains `BLOCKED`.

Only exact generated/ledger/Git/Evidence/effect/release subjects can make a
gate green.

## Status vocabulary

| Status | Meaning |
| --- | --- |
| `COMPLETE` | Exact committed subject and mapped receipt exist for this bounded claim |
| `IN PROGRESS` | Claimed work or focused evidence exists, but no completed commit receipt exists |
| `LOCAL-BLOCKED` | Implementable offline work remains or a predecessor safety gate is not green |
| `EXTERNAL-BLOCKED` | Promotion needs operator-controlled service, credential, signer, or platform evidence |

A newer subject invalidates its receipt until the mapped gate is replayed.

## Frozen Safety-Complete V1 contract

- The public name is **Bullet Farm**. Centerrail and `TEAM.md` are historical
  design provenance. Where they conflict with this reviewed plan, this plan's
  frozen V1 choices win; historical bytes remain preserved and hashed.
- Kernel owns Missions, immutable graph revisions, commands, leases/fences,
  routing and policy snapshots, the event log, and the outbox. It never owns
  Git credentials or unverified engineering truth.
- BulletGit owns private clones, atomic patch application, journal/CAS,
  checkpoints, exact Candidates, and Candidate proof roots. It never schedules
  Missions, holds provider credentials, or grants protected-ref authority.
- Providers are read-only proposal producers. Runner supervises them and may
  execute only admitted gate IDs. Verifier reconstructs the exact Candidate in
  an independent clean environment. The effect broker alone holds forge
  credentials. Portal is a sequence-bound projection and never an authority.
- Initial source distribution is Jeryu-only from immutable signed tags. The
  public discovery index is `github.com/neverhuman/bulletfarm` (not
  `neverhuman/bullet-farm`). No hyphenated `neverhuman/bullet-*` member
  namespace is assumed. GitHub is not source
  authority, but V1 GA still requires its independently reconciled effect
  receipt in addition to Jeryu.
- The shared wire uses RFC 8785 canonical JSON, domain-separated BLAKE3, full
  256-bit lowercase IDs, and algorithm-tagged Git OIDs. `ContentId` is distinct
  from provenance-bound `CandidateId`; Candidate and Integration proof roots
  are distinct.
- Public mutations use authenticated `POST /api/v1/commands`, initially return
  `202 PENDING`, and reconcile through `PENDING|APPLIED|VERIFIED|FAILED|UNKNOWN`.
  Transport success never implies verification.
- Runner, verifier, effects, and `bullet-gitd` use negotiated, bounded,
  versioned JSON-RPC 2.0 over JSONL stdio. Stdout is protocol-only.
- Local V1 is single-user and loopback-only. PostgreSQL, distributed teams,
  remote runners, cross-repository sagas, and semantic merge synthesis close
  only later profiles. Self-tuning optimization and evolutionary campaigns are
  post-V1; the minimum typed cognitive plane remains required, with
  `evolutionary_authority=false`.
- Exactly **two** protocols are model-checked: lease/fence/reclaim and command/
  effect ambiguity under timeout. No third formal model is a V1 gate.
- V1 GA requires conformant **Claude, Codex, Cursor, and Antigravity**
  service/API profiles against the same frozen subject.
- V1 GA ships exactly five signed archives: Linux x86_64/aarch64, macOS
  x86_64/arm64, and Windows x64. Linux is the full production runner; the other
  platforms fail closed on real mutation until native containment passes.
- Pre-1.0 schemas are disposable. Unknown/legacy databases fail with typed
  `UNSUPPORTED_SCHEMA` plus explicit export/removal guidance.
- Product comparison is [commit/date-pinned](competitor-snapshot.md); no performance
  claim is valid before the same receipt-bearing corpus runs on both systems.

## Historical repository checkpoints and component receipts

The table below preserves the committed subjects observed when this plan was
first reconciled. They are historical checkpoints, not the current family
heads; the generated release-truth page and fresh local lane inventories take
precedence over every count below.

| Repository | Commit | Tree | Checkout truth |
| --- | --- | --- | --- |
| Hub | `ba7c9550ea687b8acbc2707b744c0c5c9bab480a` | `817063b853ea646b3902b715b3fbb0c1263c41a1` | Exact coordination/release truth, build-free setup, sealed setup/family-lock/checkout tool subjects, runbook index, glossary, operator-decision register, and historical-spec integrity are committed; this documentation edit is excluded |
| Kernel | `7cdf850c49695c1387604a7fc677e011a51e625c` | `73a7f314140d0950d93c496b3ad27c44c3dd5b90` | Initial immutable Context Capsules are normalized and committed atomically with graph/lease/fence/Attempt truth. Current v1alpha1 policy still refuses before provider spawn; no signed internal transaction or live receipt exists |
| BulletGit | `4c508e4173aaee43083921ab457ff65744754176` | `d5e785e0d8934bde273d1044a02d3d33753e2a68` | Complete provenance-bound local Candidate identity and strict Hub canonical vectors are committed; immutable wire-tag consumption, online authority/settlement, Integration proof, and Jeryu remain blocked |
| Portal | `3033b67074a1042362789b090e3226b1e0420e8e` | `9a3a386908b2ace6c623e56d4e0f3c1e70fb3530` | Revision-one Context Lineage is strictly runtime-validated, bringing the catalog to nine projected and six explicit UNKNOWN surfaces. Successor/compression lineage and package embedding remain open |

There is no family transaction or release receipt. The full explicit
`universal-v1` dependency closure remains `BLOCKED`; the historical
`legacy-v1-26` projection remains 26/26 `BLOCKED`, and the narrower non-release
`linux-preview` diagnostic remains 25/25 `BLOCKED`. None can turn missing
transaction, live, recovery, package, security, operations, or signing evidence
green.

| Receipt | Status | Exact evidence | Boundary |
| --- | --- | --- | --- |
| Pure signed wire | `COMPLETE` | Hub `c07efb10639d500c3e82ccc282265090ff63a4aa` | DTO/canonical/signature/golden proof; no running issuer or published immutable tag |
| Tool admission | `COMPLETE` | Hub `68d0fb92b52df8d4631ac346428f341f0bb492bc`, `7efe2f3e8227`, `3039878371b6`, `e8f0180d272e`, `34f3326391a5`, `093a0e2f6b8f`; hostile replacement/family-lock/checkout subject negatives green, required green | Linux runs immutable sealed read-only descriptor subjects for Cargo, Node, Bash, npm, setup mutation Git, family-lock verification Git, and checkout verification Git, and reports replacement/subject drift. The build-free wrapper refuses by default and selects no ambient Cargo or `bullet-family`; signed admission of its external executable, clone transport Git/helpers, transient/between-child repository and non-Git filesystem stability, allowed-signers admission, production Jeryu/validator, schema-3, and signed installation remain open |
| Historical Rust 1.95 receipt admission | `COMPLETE` | Hub `d762f86b4de2`; historical 26-gate inventory remains BLOCKED without an admission | The legacy evaluator alone can select the fixed root-owned no-follow descriptor and distinct source/attestor/time Ed25519 roots over exact signed family/tool/argv/time subjects. No admission exists; profiled evaluation uses the structural-only semantic registry and cannot inherit this path |
| Deterministic release-truth projection | `COMPLETE` | Hub `0cc7eecffd4a`; 205 Hub tests, strict Clippy, required drift/negative proof, contract, deterministic regeneration | Explicit `legacy-v1-26 --report --portable` keeps decision exit 3, renders every historical catalog row, reports 0/26 receipts, and separates mechanical/evidence/review/deployment/survival truth. It is a diagnostic projection, not evidence or the full `universal-v1` authority, and cannot clear a gate |
| Bounded JSON-RPC session contract | `COMPLETE` | Hub `b0b9be55199d7d58bc795c5252b27106a5c310b3`; wire 57/57, hostile IPC 11/11 | Pure hello/frame/deadline/cancel/correlation state machines; runtime consumers still use legacy boundaries |
| Executable exact-subject checks | `COMPLETE` | Hub `24d05af9db762a72bd4a54cddbb1807c9800ea64`; local fusion `17aa92885b2fdd1807100ad8b1ab335de8b72e5b` | Fixed bounded commands and unchanged subjects; synthetic/component results cannot promote release |
| Same-origin development | `COMPLETE` | Hub `59675c8`; launcher regression and Hub required pass | Vite development proxy only; no browser command/auth or embedded production proof |
| Checksummed SQLite schema | `COMPLETE` | Kernel `9d0e5c2232342789dc889d25b34c4035059d6e4b`; 260/260 required | Migration/FK/disposable-schema component proof; not full normalized recovery |
| Database-clock lease authority | `COMPLETE` | Kernel `63285a0`; 271/271 required, clock 6/6, farmd lease 4/4 | TTL `1..=15`, DB-owned windows, restart fence, bounded heartbeat; no public command/auth or live final check |
| Atomic lease command/outbox | `COMPLETE` | Kernel `544f43ff50b92ab864cb4bcfc31c2f0d880c36f6`; required 277/277 | Fence, Attempt, lease, graph, event, exact result, and correlated outbox commit once; public wire command/auth remains open |
| Admitted gates and authoritative snapshots | `COMPLETE` | Kernel `cdfd6f2a085faeb8201a52745bec13ce444047db`, `20032074526605e1708aa72789defbd87fa75b58`, `fef4aba1f687d67107e5bafffa9adba8545becc3`; required 293/293 | Provider text cannot name shell; atomic SQLite projections carry source/time/watermark; empty ready is verified `data:null` |
| Provider process admission | `COMPLETE` | Kernel `03baa0ed7bd6746f7e7458cca6f56a60ddfda617`; focused 55/55 plus strict checks | Absolute binary/digest, 0700 HOME, 0400 OAuth copy, positive environment, canary scan, cleanup; dispatch remains blocked without signed authority and egress |
| Signed launch-grant admission | `COMPLETE` | Hub `a2d6b2ab003c`; Kernel `d38873392bfe`; wire 72/72 and Kernel 410/410 | PASETO v4.public exact-subject verifier, active-lease issuer, policy key lifecycle, and single-use nonce ledger. Policy generation 1 keeps live admission disabled; durable authority epoch/budget reservation and authenticated Runner transport remain open |
| Operator-ratifiable live policy schema | `COMPLETE` | Hub `bf5c64245d21`; Kernel loader mirror `0d848f6`; cross-family consumers `236f4ef`, `8272844`; wire 83/83 | v1alpha2 may represent live admission only at generation >=2 with an active provider-runner PASETO key while every conservative invariant remains fixed. ADR 0012 is proposed; the committed policy remains v1alpha1 generation 1 and enables no live dispatch |
| Exact checkpoint-bound proposals | `COMPLETE` | Hub `65a5ea77`; Kernel `ca380bc4`; BulletGit `fb715b25`; Hub required 211/211, Kernel 460/460, BulletGit 131/131 | Provider output carries admitted `gate_ids` and exact base checkpoint/preimage/scope subjects; application rejects stale proposals and portable ancestor/case collisions before mutation. Consumers still mirror the unpublished wire contract, and no production online authority or live adapter is enabled |
| Initial Context Capsule authority | `COMPLETE` | Kernel `7cdf850c4969`; required 469/469 with 3 explicit live skips | Normalized immutable revision-one capsule identity and package membership commit atomically with graph materialization, lease, fence, and Attempt; replay rejects cross-graph/corrupt membership. Successor/compression lineage and the remaining cognitive scheduler objects do not exist |
| Linux provider egress isolation | `COMPLETE` | Kernel `d38873392bfe`; live lane 3/3 | User/network namespace, nft default-drop, allow-listing CONNECT proxy, counter-bound receipt, and process-tree teardown block direct internet, host Jeryu, decoy, DNS, and disallowed CONNECT. This is containment-component evidence, not provider conformance |
| Policy-gated provider conformance path | `COMPLETE` | Kernel `ba485d5b5f5`, nightly real-mode wrapper `b4735da7797`; required 439/439 with 3 intentional live skips, egress 3/3, four-provider nightly refusal | One common path orders policy, key, lease, admission, signed grant, exact executable re-observation, nonce, egress, one read-only turn, canary scan, and sealed receipt. Current v1alpha1 policy produces neutral refusal with zero spawn. Only Claude has a deep positive fake-process proof; Codex, Cursor, and Antigravity lack equivalent deep fake/live receipts, and no provider has a live receipt |
| Atomic operational projections | `COMPLETE` | Kernel `529bad1f8a77`, `7cdf850c4969`; Portal `3033b67074a1`; Kernel required 469/469 with 3 intentional live skips; Portal 104/104 unit, 10/10 mocked browser, 2/2 real farmd | Fleet, Session Supervisor, Merge Rail, Quality Lab, Audit, and revision-one Context Lineage are atomic watermark-bound farmd projections with generated/strictly validated Portal consumers. Six designed surfaces still report explicit UNKNOWN; successor/compression lineage and the generated AJV root remain open |
| Authenticated command ingress | `COMPLETE` | Kernel `19d1d47cb03956eb92dfa3f27e409c87d1ab5203`; required 307/307 | Loopback bootstrap/session/origin/CSRF and atomic PENDING command/outbox/event |
| Operation-specific local authority | `COMPLETE` | Kernel `e697f8aa457ed6289e82cf51428f5e9de809436d`; required 311/311 | Exact typed mutation/terminal/preservation decisions; signed request capabilities, daemon receipt validation, and online cleanup settlement remain open |
| Authenticated offline command reconciliation | `COMPLETE` | Kernel `77a0ecd0079d030e944ebf3a7b9077b7d64aabcc`; required 317/317 | Exact-ID worker settles one command/outbox/event to honest UNKNOWN/FAILED; no dispatch, APPLIED, VERIFIED, provider, verifier, or effect path |
| Receipt-bound backup/quarantined restore | `COMPLETE` | Kernel `798f0c814cc4dde1fc510eadc46ce14653380772`; backup 9/9, migrations 14/14, CLI 1/1 | WAL-consistent exact-size/digest copy and verified restore epoch; restored data stays quarantined because authenticity and production admission remain open |
| Four offline provider message subsets | `COMPLETE` | Kernel provider commits `ca376e4`, `c34d578`, `ea89929`, `5badc85`; strict recursive JSON `1bb32bd`; required 330/330 | Pure bounded parsers with public dispatch blocked; RFC 8785 identity, signed runtime isolation, native Cursor proposal extension, and every live receipt remain open |
| BulletGit fail-closed gateway | `COMPLETE` | BulletGit `79bf1e2129fbe50ed85d424fe6e4416407bb17f4`; 82/82 required at consumer head `7df926c` | Production refuses unavailable authority; no positive checker or Jeryu backend |
| Atomic generations and preservation | `COMPLETE` | BulletGit `61bf76dd06753df1ce37715582ab56fbf5d75cff`, `9d527b9e2d8da2fe4ca76c45787851f6d6dab8c2`; required 105/105 | Prior-or-complete-next generation plus sealed exact-state salvage before cleanup; positive online authority and Jeryu remain open |
| Exact mutation reservation | `COMPLETE` | BulletGit `f8121142cd337e243bdc97cdeec9dacea9554b04`; required 107/107 | Durable reservation binds request/envelope/Attempt/fence/workspace generation; production checker and Kernel settlement remain unavailable |
| Provenance-bound Candidate identity | `COMPLETE` | BulletGit `4c508e4173aa`; required 139/139, affected packages 130/130 | Separate Content/Candidate IDs bind the complete local Candidate provenance manifest to repository-derived facts and exact Hub canonical vectors, with preflight before permit consumption and a final writer check. Immutable Hub-tag consumption, Kernel caller convergence, online signed authority, Integration proof, and Jeryu remain open |
| Projection and real-process browser truth | `COMPLETE` | Portal `cfba6f72f6fd55cc0477182b74b63ade49821d07`; unit 51/51, mocked browser 10/10, real farmd browser 1/1 | Strict snapshots, STALE recovery, server provenance, ready-null; packaging remains open |
| Strict browser command reconciliation | `COMPLETE` | Kernel `35b64847459aefb88ef17c37427d9b7b9754ae97`; Portal `181cd00cc6f9d20b079bdcecea88eebde70c47c3`; Kernel 342/342, Portal 67/67, real farmd 1/1 | Sole exact outbox/submitted/reconciled truth; SSE ID/sequence conflicts stay STALE; authenticated worker proves correlated `PENDING→UNKNOWN`, never green. Vite preview is not packaged/embedded evidence |
| Transaction-safe source setup | `COMPLETE` | Hub `5148a52a122da46e749be3e2169f81bd6d4b8116`; Hub required 34/34 plus integrations | Fallible validation precedes no-replace publication and final durable manifest; signed schema-3 inputs and prebuilt installer remain open |
| Pinned assurance controls | `COMPLETE` | Hub `0440d446190c20c2be24620fa1d91d1b39fa3073`; required 34/34 | Pinned secret/dependency/license/workflow scans and release blocker report; passing components do not promote a release |
| Signed five-target bundle verification | `COMPLETE` | Hub `352f963c75ce1939898a26d94d39be13de321f86`; required 39/39 plus 3/3 bundle integration | Linux read-only exact-byte/signature verification; no package build, semantics, extraction, install, signer provisioning, or intermediate-directory race proof |
| Safe signed archive extraction | `COMPLETE` | Hub `ba0905604b6c743306f245837a0621781478e4cf`; hostile archive/publication proof | Exact signed bytes materialize descriptor-relatively at one absent destination; no activation, rollback, semantic package admission, or installer receipt |
| Descriptor-relative setup publication | `COMPLETE` | Hub `94b6549aa24ee4bc2110627c994b4d476042864a`; focused 13/13, required and fast | Retained root/staging identity, private staging, no-replace publication, bounded no-follow cleanup; schema-3/prebuilt install and same-UID path-based Git containment remain open |
| Signed release-receipt contract | `COMPLETE` | Hub `143f8b963586ad162a97fcd0d5ca7f18fa034796`; focused 5/5 and Rust 1.95 strict proof | Canonical TOML, exact policy digest, signer/namespace/interval, and sealed-input verification only; external policy, trusted time/revocation/custody, semantic adjudication, registry/replay, and real receipts remain open |
| BulletGit recovery and wire-shaped subjects | `COMPLETE` | BulletGit `274fd6d6655ce88979bcaa80eca758c44886963e`, `f55173622613e7ce55d9e1366ee6434c7e32158e`; required 121/121 | Freeze/recovery, full IDs, tagged OIDs, strict manifests; provenance-bound Candidate identity, shared immutable wire tag, and production Jeryu remain open |
| BulletGit cleanup outcome and CI parity | `COMPLETE` | BulletGit `2d22c28f9619`, `5dac98e12c1a`; required 123/123 | Cleanup success requires a new synced tombstone binding preservation receipt/artifact/destination; post-delete ambiguity is typed UNKNOWN. Production positive online authority, signed shared-wire tag, Jeryu, and Jankurai >=90 remain open |
| Generated Kernel-to-Portal runtime contract | `COMPLETE` | Kernel `043b8fddd59cef8a67ad98f45d9c190fc11bd94f`; Portal `c294ec7bddb7dd217eb4bd360b6c810c0391a31d`; Hub `601cb82c9a5f66cc627677244e230f289e9acc65` | Dependency-closed JSON Schema and AJV for consumed command/mission/readiness DTOs; raw events and absent Candidate/Evidence/Effect DTOs remain predecessors |
| Exact Portal bundle subject | `COMPLETE` | Portal `8272844e44d8`; required 67/67, bundle 5/5, browser 6/6 mocked + 1/1 real farmd; root `blake3:556d91f8504299230bc75e027108d1d943fd9ff99303cea7c2a155ee69edd973` | Clean commit/tree, lock, exact Git/Node, whole npm tree, and three emitted files are bound. Manifest-verified same-origin Rust embedding now has component proof; toolchain/environment signatures, a signed release archive, activation, and rollback remain open |
| Shared admitted gate and clean verifier | `COMPLETE` | Kernel `528348fae6038def88ecd6d6b4f4f54e78747cd4`; required 339/339, contract plus 3 simulators | Caller/model shell and timeout authority removed; exact gate/argv/timeout/base/head/tree Evidence. One fixture gate exists; executable digest, framing/source-path admission, and multi-gate aggregation remain open |
| Bounded verifier transport | `COMPLETE` | Kernel `365bb5d32ac31791f338a58c9c9d0b94b0b74f18`; required/contract 345/345 plus 3 simulators | Strict one-shot 64 KiB request and single bounded Evidence frame; contaminated/unknown/lossy/overflow output refuses and infinite writer is killed/reaped. JSON-RPC, signed source admission, and process-tree supervision remain open |

Hub Jankurai is 65/raw 66 with 5 caps and 48 findings: 21 high/hard and 27
medium/soft. It fails the V1 >=90, zero-cap/zero-hard gate.
Hosted CI also lacks a portable checksum-pinned Jankurai artifact; a machine-local
binary cannot be converted into a skip-green workflow.

## Dependency graph

```text
V1-S0 coordination/provenance
  -> V1-S1 wire, schemas, IPC
       -> V1-S2 Kernel durable authority
            -> V1-S3 BulletGit durable subjects + Jeryu service
                 -> V1-S4 runner/verifier/effect transaction
                      -> V1-S5 authenticated API + truthful Portal
                           -> V1-S6 cognitive plane + four providers
                                -> V1-S8 live promotion/release
       -> V1-S7 installer/CI/docs/package mechanics ----------------^
```

`V1-S7` may build after `V1-S1`, but cannot exit before `V1-S2..S6`. No slice
advances while it or a predecessor has a known safety counterexample.

## V1-S0 — coordination and provenance

Status: `COMPLETE` locally; release receipts remain continuous obligations.

Remaining work:

1. Require exact, non-overlapping `coord claim|heartbeat|handoff|status` records
   for every edit and short-lived heartbeat at least every five minutes and on
   every proof, blocker, commit, or handoff.
2. The orchestrator stages only completed claimed paths and records claim IDs
   against exact commit path sets. Never stage an entire dirty checkout.
3. Preserve mixed provenance through corrective commits. Append coordination
   history; never reset, rewrite, or treat a lane label as proof.

Exit gates:

```bash
bullet-family coord status --json
bash scripts/ci-local.sh required
```

Mandatory negatives: concurrent overlapping claims have one winner; expired,
corrupt, truncated, replayed, path-traversing, or mismatched-repository records
fail closed; a commit containing an unclaimed path cannot receive a receipt.

## V1-S1 — frozen cross-repository contracts and IPC

Status: `LOCAL-BLOCKED`. Wire/IPC component machines exist, but no immutable
Jeryu publication exists and consumers duplicate or use legacy semantics.

Required work:

1. Publish `bullet-wire` from an immutable signed Jeryu tag. Generate ignored
   `.fusion` development overrides only; committed sibling `path` dependencies
   remain forbidden.
2. Make Kernel and BulletGit consume the authoritative validated Candidate,
   digest, Evidence, effect, proposal, checkpoint, and proof semantics; delete
   duplicate local meanings rather than translating around them.
3. Replace model-owned `tests_to_run` and shell command fields with admitted
   `gate_ids`. A proposal binds producing Attempt, base checkpoint ID+digest,
   per-operation preimages, scope, and bounded content.
4. Generate OpenAPI, JSON Schema, Rust consumers, TypeScript client/types, and
   AJV validators into a temporary directory and diff tracked artifacts without
   mutating them.
5. Land negotiated JSON-RPC 2.0 hello/version/frame/deadline/cancel/request-ID
   conformance before any trust-boundary process is treated as production.

Exit gates:

```bash
just contract
just family-contract
```

Mandatory negatives: mutate every ID/digest/authority/Candidate bound field;
unknown fields, duplicate keys, non-canonical JSON, algorithm-confused Git OIDs,
oversized frames, missing hello, duplicate request IDs, late events, and legacy
schemas all refuse. Golden JSON/hash bytes match in every repository and
generation leaves zero tracked drift.

## V1-S2 — Kernel durable authority, commands, and recovery

Status: `LOCAL-BLOCKED`. Checksummed migrations/FKs, database lease time,
atomic lease-command/event/outbox, admitted gates, and authoritative snapshots
are complete components. Authenticated public command admission and typed local
operation decisions, an UNKNOWN/FAILED-only worker, and receipt-bound backup
with quarantined restore are complete; normalized truth, signed capabilities,
restore admission, fault-complete recovery, and CAS remain.

Required work:

1. Normalize immutable identities/transitions, graph revisions, Candidates,
   Evidence, commands, leases, events, projections, effects, and outbox rows.
   Remove JSON-blob truth and `INSERT OR REPLACE` authority.
2. Preserve lease/fence allocation in one SQLite `BEGIN IMMEDIATE` transaction
   using database time. Fence counters never rewind across replay, expiry,
   supersession, restart, backup, or restore.
3. Preserve authenticated admission and the exact-ID offline worker. Extend it
   only through signed dispatch; persist each result, transition, event, and
   outbox settlement atomically, returning `UNKNOWN` when truth is unavailable.
4. Mint short-lived PASETO v4.public capabilities from the durable active lease
   only. Bind audience, operation/request digest, Mission/repository/graph/
   package/Variant/Attempt/fence/runner/workspace/scope/context/configuration/
   policy/routing, authority epoch, freeze generation, expiry, and nonce.
   Lease and token maximums are separately enforced at 15 seconds.
5. Put WAL and filesystem CAS under platform data directories. Preserve the
   exact backup/restore receipt and quarantine, then add receipt authenticity,
   restore admission, fault boundaries, retention, orphan-safe GC, and anchors.
6. Preserve operation-specific decisions and delete every remaining hard-coded
   SHA/PASS/demo-success branch.

Exit gates: Kernel `bash scripts/ci-local.sh required`, locked Rust 1.95 strict
Clippy, generated drift, backup/restore/fault suites, and Hub `just model-check`
with exactly the two pinned models.

Mandatory negatives: simultaneous acquire, replay, TTL 0/16, exact expiry,
heartbeat-versus-expiry, supersession, freeze, restore, every capability-field
mutation, wrong operation/request, nonce reuse, malformed/corrupt persisted
values, authority outage/timeout/zero-row, and crash at every SQLite/WAL/CAS/
event/outbox boundary yield zero unauthorized mutation and exactly old or
complete-new durable state.

## V1-S3 — BulletGit durable subjects and Jeryu capability service

Status: `LOCAL-BLOCKED`. Immutable CAS/journal/checkpoints, prior-or-complete-next
generations, and preservation-bound cleanup are complete. Complete shared-wire
manifests and online authority remain. `jeryu-gitd` promotion is
`EXTERNAL-BLOCKED` on a reviewed capability tag and operator authentication.

Required work, in order:

1. Preserve the committed CAS-first journal and exact checkpoint identity.
   Reopen must continue to reject a missing or altered referenced object.
2. Preserve generation-atomic publication and sealed-cleanup negative coverage.
3. Consume the immutable `bullet-wire` proposal and enforce base checkpoint,
   absent/digest preimages, duplicate/conflict/path/scope/content constraints.
4. Harden an absolute verified Git binary, hostile config/attributes/filters/
   hooks/alternates, all sequencer states, exact `-z` status parsing, no-follow
   writes, validated IDs, and safe cleanup targets.
5. Complete Checkpoint/Candidate/Integration proof manifests; retain the sealed
   preservation receipt binding Attempt/fence/nonce/tree/dirty/journal/CAS and
   an external destination.
6. Add Kernel online final check and durable settlement immediately around each
   mutation. Keep the current local Git backend simulator-only. Production calls
   versioned `jeryu-gitd` from a separately reviewed immutable Jeryu tag.

Exit gates: BulletGit required/contract, hostile-repository and deterministic
fault suites, strict toolchain/audit checks, then the exact-head family contract.

Mandatory negatives: duplicate/case/NFC-colliding paths, absolute/dot/backslash/
`.git` paths, ADS/trailing-dot-space, stale/missing preimages, symlink/reparse
escape, oversized content, non-UTF-8/newline Git status, hostile filters or
alternates, incomplete sequencers, partial writes, crash at allocate/write/
file-sync/publish/dir-sync/generation-switch, proof mutation/rebase, forged or
mismatched preservation, unsafe cleanup, and authority outage all preserve the
prior authoritative generation or return typed `UNKNOWN`; none yields success.

## V1-S4 — runner, verifier, effects, and offline transaction

Status: `LOCAL-BLOCKED` on `V1-S2/S3`; bounded transport and a signed internal
UDS RPC/client exist as components, but the product Runner refuses admission
before using either that prototype or the deliberately unmounted public lease
routes. Every `/v1` route is retired with typed `API_VERSION_RETIRED`;
operator traffic is `/api/v1`, workload traffic is `/internal/v1`, and the
credential-free demo remains a component-only CLI path rather than an API.

The existing `HttpLeaseClient` is not safe to mount: it is unauthenticated,
acquire self-asserts scheduling and runner identity fields, and release/advance
are not full-subject atomic operations. Keep public `/api/v1` lease routes
absent. The signed UDS component already binds runner, authority epoch, request
digest, operation, and lease subject; checks each registered Runner ID/epoch
against the connected `SO_PEERCRED` UID; pins farmd UID plus socket GID/device/inode
across connect; and persists server grant/nonce state. Production admission still
needs an operator-admitted durable peer registry and signing-key custody in place
of debug-only registry injection and the ephemeral farmd process key, durable
client acquire/read-back recovery instead of process-local metadata, product
Runner wiring, and a connected `TRANSACTION_PROOF` before it can own
reserve/acquire/advance/release or lost-response reconciliation.

Required work:

1. Runner acquires authority, starts a private read-only provider generation,
   validates a proposal, applies only through BulletGit, executes admitted gates,
   allows at most two repair turns, checkpoints, and prepares an exact Candidate.
2. Failed or zero-row heartbeat freezes mutation, terminates the entire provider
   tree, preserves the workspace, and permits a successor fence to resume only
   from the exact checkpoint.
3. Verifier reconstructs the Candidate independently. Writer state or
   writer-produced proof cannot satisfy independent gates; oracle-modifying
   diffs receive distinct review and holdout treatment.
4. Effect broker records intent before dispatch, authorizes, dispatches, records
   ambiguous response as `UNKNOWN`, reads authoritative remote state, and adopts
   only the exact desired subject without a second write.
5. `just demo` stays credential-free and deterministic but uses real child
   boundaries and a protected local forge simulator, ending in one signed
   `TRANSACTION_PROOF`.

Exit gate: `just demo` plus Kernel/BulletGit required and an exact-head family
contract proves materialization replay, fences `1→2`, stale rejection, process
death/salvage, isolated clones, exact Candidate, independent Evidence, ambiguous
effect recovery, protected integration, preservation-before-cleanup, and
truthful projection.

Mandatory negatives: provider crash/cancel/timeout/malformed/duplicate/delayed
events, missed heartbeat, surviving grandchild, stale fence, zero admitted
tests, timeout/flaky/unsupported/infra/UNKNOWN verification, writer-modified
oracle, lost effect response, conflicting remote OID, reconciliation restart,
and cleanup-before-preservation never become PASS or cause a duplicate effect.

## V1-S5 — authenticated API, SSE, and projection truth

Status: `LOCAL-BLOCKED`. Atomic snapshots, strict Portal validation, exact-pair
SSE recovery, authenticated commands, and real farmd `PENDING→UNKNOWN` are
components. Nine of the fifteen designed Portal surfaces now have durable
farmd projections; the other six remain explicit UNKNOWN. Context Lineage
contains revision-one capsule subjects only. Manifest-verified same-origin
embedding and packaged-farmd browser tests are component-proved, but there is
no APPLIED/VERIFIED dispatch or signed package-served Portal.

Required work:

1. Snapshot responses carry `{data,as_of_sequence,observed_at,source}` from one
   atomic ledger read. SSE emits only generated default `EventEnvelope` values;
   `after` and `Last-Event-ID` are exclusive, conflicting cursors fail, replay is
   bounded/ordered, and live tail includes keepalives.
2. Preserve the one-time bootstrap, HttpOnly/SameSite session, CSRF,
   loopback/origin, and no-wildcard-CORS boundary.
3. Preserve generated client/AJV validation. Correlate each displayed
   receipt to its command ID; timeout is `UNKNOWN`, and an old green receipt
   cannot satisfy a new failed command.
4. Preserve the six new atomic Fleet, Session Supervisor, Merge Rail, Quality
   Lab, Audit, and Context Lineage projections. Implement durable ledger subjects
   and projections for the six remaining surfaces: Cognitive Router, Fusion Lab,
   Quota/Capacity, Struggle, Behavior, and Workspace Hygiene. Extend Context
   Lineage only after durable successor/compression subjects exist.
5. Embed the built Portal in the Rust distribution and run Playwright against a
   real packaged farmd. Vite/mock lanes remain focused component tests only.

Exit gates: Portal required/build, generated drift, and packaged farmd/browser
E2E with authenticated command reconciliation.

Mandatory negatives: SSE `1,2,4` plus failed snapshot stays STALE at cursor 2;
watermark `>=4` clears it. Cover first-event gaps, reconnect, retention gap,
malformed/oversized/CRLF streams, body timeout, conflicting cursors, forged or
missing session/CSRF/origin, command timeout, mismatched command receipt, older
green result, and farmd restart. None may render false green.

## V1-S6 — cognitive plane and four provider adapters

Status: `LOCAL-BLOCKED`. Fail-closed offline Claude, Codex, Cursor, and
Antigravity message subsets plus strict recursive JSON and one common
policy-to-receipt orchestration path are committed. The production v1alpha1
policy refuses before key read, ledger mutation, namespace creation, or spawn.
Real identity/profile admission, operator-ratified v1alpha2 policy, native
protocol conformance, and live receipts remain absent.

Required work:

1. Persist typed Cognitive Tasks, role/capability/profile snapshots, context
   capsules, behavior rules, budget/quota reservations, routing provenance,
   struggle/escalation, fusion, dissent, selection, and negative knowledge.
2. Treat roles as constrained policy profiles: planner decomposes, researchers
   cite observations, implementers submit proposals, critics create
   counterexamples, fusion preserves agreement/dissent, verifier supplies
   Evidence, and broker performs effects. No role receives ambient authority.
3. Route hard constraints before optimization: risk/authority envelope,
   capability maturity, exact profile, quota, context budget, independence, and
   verifier capacity. UNKNOWN paid capacity blocks ordinary dispatch; a bounded
   read-only probe requires explicit policy.
4. Preserve the bounded offline Claude stream-JSON, Codex App Server JSONL,
   Cursor ACP, and Antigravity headless subsets while closing decoder/native-
   extension gaps. Runtime probing, including Antigravity `-p=` ordering and
   schema support, determines capability.
5. Execute absolute verified binaries with allowlisted environments, ephemeral
   HOME, minimum provider OAuth, provider-only egress, and no SCM/cloud/SSH/host
   secrets. Persist exact binary/model/config/profile receipts.

Exit gate: Claude, Codex, Cursor, and Antigravity pass the same common
conformance matrix for the frozen V1 subject, and routing/fusion decisions
replay from persisted inputs.

Mandatory negatives: missing/unknown quota, expired reservation, profile or
model substitution, capability downgrade, malicious output, malformed/
duplicate/delayed events, cancel/timeout/crash, resume/fork mismatch, process
escape, direct filesystem/Git mutation, network escape, and canary secrets in
environment/output/patch/log all fail closed. A losing Variant remains immutable
and cannot be rewritten into the selected Candidate.

## V1-S7 — installer, checks, CI, packaging, and documentation

Status: `LOCAL-BLOCKED`. Coordination/checks/fusion, strict schema-3 verification,
descriptor-relative setup, a build-free default-refusing wrapper, signed bundle
verification, and safe extraction exist. Schema 2 refuses; a deterministic Portal
bundle subject exists, but there is no signed admission for the wrapper-selected
external executable, embedded farmd, package builder, activation/rollback installer,
authenticated prebuilt installer, signer, or release.

Required command surface:

```text
bullet-family doctor --json
bullet-family setup --root <path> --source jeryu [--offline]
bullet-family lock generate|verify
bullet-family checkout verify
bullet-family fuse --source local|lock
bullet-family check fast|required
bullet-family check release --profile universal-v1 --receipts <admitted-absolute-registry> --json
bullet-family check release --profile <named-profile> --receipts <absolute-registry> --json
bullet-family coord claim|heartbeat|handoff|status
```

Installer closure, in dependency order:

1. Decide and implement public signed Jeryu objects or a short-lived,
   destination-bound credential channel that never enters URL/argv/env/logs.
2. Preserve the exact Portal bundle manifest/root, stage it into an opt-in Kernel
   build through `OUT_DIR`, then publish signed `bullet-family` and embedded
   `bullet-farmd`; never use a sibling include or tracked `dist` as authority.
3. Verify hub tag, non-circular manifest, schema-3 lock, authenticated member
   URL/slug, signed tags, exact commit/tree, lockfiles, artifacts, and canonical
   Git/Bash/Cargo/Node/npm binary digests and admitted versions before mutation.
4. Preserve the committed rule that dependency/generated/exact-family checks
   finish before no-replace member publication and the final manifest marker.
5. Create ordinary clones at exact OIDs; reject dirty/symlinked/conflicting
   paths; use `cargo --locked` and `npm ci`; generate/diff in temporary trees.
   Preserve validated Rust fusion and its byte-idempotent ignored output.
6. Run hub-only setup twice in a fresh HOME and crash-inject every clone,
   checkout, fsync, publish, dependency, and drift boundary. End at exact clean
   member OIDs with no worktree and either prior or complete setup state.

CI closure:

| Profile | Required executable evidence |
| --- | --- |
| `fast` | warm under 60s; fmt, strict Clippy/unit, TypeScript type/unit, production Portal build, generated drift, affected-path routing |
| `required` | locked build/test/doc-test, contracts, migrations, real packaged browser E2E, transaction demo, pinned secret/dependency/license/workflow scans, family lock, Jankurai ratchet; no skip-green |
| `universal-v1` release | Complete profile closure: five signed archives and installer smoke, Rust 1.95 and pinned 1.97.1, SBOM/checksum/signature/provenance, backup/restore/faults, Jeryu and hosted-adapter reconciliation, all four providers, operations, and Jankurai >=90 with zero caps/hard findings |
| `linux-preview` and other named profiles | Non-release diagnostic slices only; omitted provider, forge, platform, or package subjects remain mandatory for canonical V1 GA |

Jeryu CI must run the same local commands through workflow IR. GitHub workflows
are a portable mirror, not release evidence until a public mirror exists.
Nightly exists only for meaningful fuzz/soak/live work. Missing optional live
registration is neutral; missing a required tool or receipt fails.

Documentation closure follows executable ownership:

- `docs/release.md`: concise stable release contract and blockers;
- this file: stable slice graph, exact subjects, receipts, and next gates;
- generated check report: current machine status, never manually copied scores;
- README: separate contributor use from future signed hub-only install;
- ADRs: accepted choices changed only with implementing evidence;
- runbooks: tested setup recovery, upgrade/rollback/uninstall, signer rotation,
  schema removal, backup/restore, SAFE_STOPPED, effect reconciliation, and
  platform refusal after the typed commands exist;
- `docs/spec/` and `TEAM.md`: immutable hashed historical provenance.

Exit gates: `bullet-family check fast|required` plus explicit `bullet-family check release --profile
universal-v1 --receipts <admitted-absolute-registry>`, signed lifecycle smoke
for all five V1 archives, two-run hub-only setup, generated drift in a temporary
directory, docs/link/source/license/workflow scans, and zero tracked changes.
`linux-preview` may diagnose Ubuntu/Jeryu/Claude readiness but cannot waive any
canonical gate.

Mandatory negatives: schema-2/future/corrupt lock, bad signature/checksum/tag/
tree/lockfile/tool digest, PATH shim, credential/canary leak, offline cache miss,
dirty/symlinked/non-empty destination, partial setup/crash, rerun conflict,
unsupported platform mutation, package mismatch, circular manifest, missing
scanner, zero routed tests, optional-neutral promoted to required PASS, broken
links, machine-local paths, or generated drift all block with repair guidance.

## V1-S8 — credentialed live proof and release promotion

Status: `EXTERNAL-BLOCKED`; it cannot begin before `V1-S0..S7` are green from
tagged bytes.

Required order:

1. An authorized Jeryu lane reviews, tags, and publishes the capability service.
   The operator restores authentication; run read-only probes before protected
   integration and UNKNOWN/read-back reconciliation. Never alter the running
   forge to work around missing capability.
2. After the offline gates pass, stop for distinct signed Claude, Codex, Cursor,
   and Antigravity service approvals and run one bounded read-only turn per
   provider against the same frozen subject.
3. Stop again for separate Jeryu and GitHub broker/attestor/integrator
   credentials and exact protected test repositories. Run low-risk Candidate
   transactions with authoritative read-back and observation for both effects.
   Failure remains evidence and cannot be relabeled.
4. Build and smoke all five signed archives, publish SBOM/checksums/signatures/
   provenance, verify backup/restore and containment, then sign the final
   non-circular release manifest.

Exit gate: after kind-specific semantic receipt admission exists for every row,
`bullet-family check release --profile universal-v1 --receipts
<admitted-absolute-registry> --json` passes its complete dependency closure from
exact signed tags with current Jeryu and hosted effects, all four providers, all
five platforms, security, recovery, operations, package, and signer receipts.
No generic receipt may clear a gate without its kind-specific semantic verifier.

Mandatory negatives: lost remote response becomes `UNKNOWN`; read-back adopts
the original exact OID without a second write. Wrong fence/OID/check/proof root,
expired or missing credential, protected-ref refusal, provider/profile drift,
platform containment absence, signature/provenance mismatch, revoked signer,
or unavailable required service blocks promotion. Optional unregistered lanes
may be neutral only when the requested release profile does not require them.

## Immediate closure queue

1. Promote the component-proved signed UDS Runner lease transport described in
   V1-S4, then extend offline command reconciliation into signed dispatch,
   effect read-back, and independent verification without synthetic APPLIED or VERIFIED.
2. Complete `V1-S1` immutable publication/runtime-consumer convergence and
   `V1-S2` normalized truth, capability, CAS, backup/restore, and fault receipts.
3. Complete shared-wire manifests and positive online BulletGit authority, then
   consume the reviewed tagged `jeryu-gitd` capability.
4. Produce the `V1-S4` credential-free five-plane transaction receipt; replace,
   rather than rename, synthetic success.
5. Carry the manifest-verified embedded Portal through signed packaging; add the
   six missing ledger-backed surfaces; persist the minimum typed cognitive
   routing/fusion plane; and close the common live conformance matrix for all
   four providers. Keep evolutionary campaigns and self-tuning optimization
   post-V1.
6. Publish signed schema-3 install subjects and a prebuilt installer, then build
   all five archives with lifecycle smoke, SBOM, checksums, signatures,
   provenance, containment, and Jankurai/security evidence.
7. Run `V1-S8` only with operator-provided Jeryu and GitHub credentials, all
   four provider service approvals, protected test repositories, and
   five-platform signing authority.

## Terminal definition of done

V1 GA is done only when every canonical `V1-S0..S8` gate has a current
independently verifiable receipt from the same signed subjects;
setup/demo/fast/required/release and the two pinned models pass; Jankurai
reaches 90 with zero caps/hard findings; Claude, Codex, Cursor, Antigravity,
Jeryu, GitHub, all five archives, signature, recovery, fault, installer,
operations, and containment receipts are current; every checkout is clean;
Portal has no synthetic authority; and no safety counterexample remains.
No diagnostic profile can substitute for that result.
Until then: **pre-release, blocked**.
