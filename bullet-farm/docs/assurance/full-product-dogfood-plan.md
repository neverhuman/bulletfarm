# Full-product dogfood bridge

Status: **ACTIVE implementation plan; all release profiles remain `BLOCKED`**  
Owner: Bullet Farm maintainers  
Last reconciled: 2026-08-27

This is the execution bridge from the frozen local family to Bullet developing
Bullet through its own complete transaction. It refines the
[finish execution plan](execution-plan.md); it does not replace the
[G1–G18 register](product-gaps.md), [Waves 0–11](closure-roadmap.md),
[WP-01–WP-23](../workplan.md), or [OD-A–OD-J](../decisions/0013-operator-decision-register.md).
The executable profile check and admitted receipts always win over this page.

## 1. Finish lines

“Dogfood” has three non-substitutable finish lines. Work is planned against the
earliest honest one, while preserving the path to the complete target.

| Finish line | Meaning | Earliest gate | Current fact |
| --- | --- | --- | --- |
| Coordination dogfood | Bullet uses the recovered schema-2 coordinator for its own claims, handoffs, sole-writer commits, receipts, exact retries, and restart read-back | Phase R + W0 | **BLOCKED** by the frozen incident and dirty subjects |
| Full-product offline dogfood | One Bullet change crosses Control, Execution/BulletGit, Verification, Delivery/integration, and Evidence/audit with all twelve fault boundaries | W7 `TRANSACTION_PROOF` | **BLOCKED**; a retained component bridge reaches purpose-signed fixture intent/evidence/proof and `MATCHED` Observation, exact Candidate delivery, local check, protected integration, reopen read-back, and post-exit retained-artifact reads, but its executor/keys/roles are harness-process fixtures, its outer receipt is unsigned/ineligible, and independent custody plus the twelve-boundary campaign are absent |
| Live self-hosted dogfood | The same product path adds admitted Claude execution and protected Jeryu integration on the signed Ubuntu family | W8 `self-hosted-v1` | **BLOCKED** by W7 and OD-A/B/D/E |
| Complete documented target | Independent evolution, provider, forge, platform, team, and saga profiles also pass | W9–W11 and all eight finish conditions | **BLOCKED**; no narrower receipt may be borrowed |

The first two lines are engineering-owned except for schema-3 signing inputs.
The third and fourth intentionally require operator custody. No percentage,
checkbox, local simulation, or chat line promotes an evidence class.

## 2. Critical path and stop rules

```text
R4 replay P1 and exact recovery subject
→ R5 supervised rollover facade
→ R6 PASS/APPROVE/request/adopt transactions
→ R7 fresh 0700 rehearsal
→ independently reviewed real incident recovery
→ W0 four clean heads and family observation
→ first schema-2 coordinated Bullet-on-Bullet change
→ W1 immutable contracts and schema-3 subjects
→ W2 durable authority
→ W3 service/isolation boundary
→ W4 production BulletGit and LocalBareForge
→ W5 independent verification and effect reconciliation
→ W6 API/Portal operational truth
→ W7 signed twelve-boundary transaction and Ubuntu lifecycle
→ W8 Claude + protected Jeryu self-hosted transaction
→ W9/W10/W11 complete the documented target
```

Every packet stops on dirty or changed subjects, failed/skipped/zero tests,
missing artifacts, stale generations, signer/identity substitution, an
unreconciled effect, or `UNKNOWN`. A retry first reads back by the original
request and desired-state identity. It never dispatches a second write merely
because a response was lost.

## 3. Phase R and W0: start useful dogfood safely

These packets are serialized. They are the only route to unfreezing normal
family coordination.

| Packet | Work and owner surface | Exit evidence | Active hold |
| --- | --- | --- | --- |
| DF-R4 | Hub recovery verifier/ledger: validate the published recovery tombstone, retired source, sibling absence, and sealed observation chain before and at final locked replay | Clean exact R3.1+R4 subject; all recovery/adoption hostile suites; strict lint; independent artifact review | Bounded logic is exact-copy green; `cargo clippy --locked --workspace --all-targets -- -D warnings` is 0/0 on this tree; R4.1 `canonical_hostile` 8/8 after the current path/attribute/dependency inventory pin. No live execution is authorized |
| DF-R5 | Hub private recovery facade and narrow CLI: exact normalized absolute inputs, descriptor-safe Linux admission, sole inner topology oracle, deterministic writer-wait/resume, typed non-Linux refusal | Facade publish/wait/resume/already-current tests; facade-level Exchange crash/restart; macOS/Windows compile and zero-mutation CLI refusal | Linux facade, Exchange restart, and static hosted-lane policy are green. This Linux host now fail-closes with `NATIVE_PLATFORM_EVIDENCE_UNAVAILABLE` (`scripts/platform-native-evidence.sh`); native `macos-15`/`windows-2025` compile/refusal remains unproved until those runners admit `ops/ci/platform-refusal.sh` |
| DF-R6 | Hub recovery producers: internally derived proof PASS, sealed independent APPROVE, canonical adoption request, explicit `adopt --request`, request-byte idempotency/conflict | Failed/SKIP/UNKNOWN/non-APPROVE cannot append; reviewer differs from orchestrator; exact proof set/watermark; retry invokes no clock/process/write | Linux component implemented: the [operator runbook](../runbooks/coordinator-recovery.md) covers the five closed CLI actions; focused model/backend/CLI chain is 5/5, public ingress 2/2, the complete coordinator library is 159/159, canonical hostiles are 8/8, and four canonical public suites are 19/19. Strict Hub lint still has 41 shared reachability/dead-code failures; native-platform execution, R7 rehearsal, independent review, and the real incident remain blocked |
| DF-R7a | Fresh owner-0700 synthetic family: run rollover → proof → review → request → adopt → restart with injected crashes at every publication and append boundary | One immutable rehearsal bundle; deterministic redacted JSON; byte-identical rerun; independent review | Unsigned COMPONENT rehearsal producer (`scripts/recovery-rehearsal.sh`) is landed: 0700 parent, byte-identical rerun, live incident hard-false. Signed independent bundle review and policy-enabled recover-rollover remain open (`RECOVERY_POLICY_DISABLED` without operator keys) |
| DF-R7b | Frozen real incident: independently compare every live input hash to the reviewed rehearsal, execute once under supervision, adopt every reviewed break-glass group, restart and read back | Current schema-2 generation, complete watermark, no unexplained frozen claim, signed human review record | Compare/refuse producer (`scripts/recovery-incident-compare.sh`) is landed: missing APPROVE and frozen live source both refuse; the script cannot chmod, recover, or adopt. Execution waits on independent human approval |
| DF-W0a | Four repository owners close active changes, run atomic and standalone `required` lanes, then sole-writer commits reviewed path sets | Four clean immutable commit/tree pairs; no missing, skipped, zero-test, dirty, or orphan partition | Waits on incident recovery |
| DF-W0b | Family order: BulletGit build → Kernel component/family with exact daemon path → Portal component/real-farmd browser → Hub contracts/models/media | Deterministic unsigned `bullet.ci-observation.v1`; second identical family run | No signed Evidence is claimed |
| DF-DOG0 | First low-risk docs/test-only Bullet change uses status → exact claim → heartbeat → handoff → proof → sole-writer commit receipt → restart read-back | Stored request IDs and complete watermark reconstruct the whole loop after process restart | Any direct commit or inferred receipt invalidates the exercise |

DF-DOG0 is the earliest useful development dogfood. After it passes, all new
Bullet work should use the coordinator, while the product still describes that
use as `COMPONENT`, never `TRANSACTION`.

## 4. W1: one immutable language and family

W1 prevents every later service from inventing its own identity or receipt
meaning.

| Packet | Deliverable | Required negative proof | Dependency |
| --- | --- | --- | --- |
| DF-101 | Publish one recursively closed `bullet-wire-v1` source for Authority, Transaction, Forge, Evolution, Release, Candidate, Evidence, Effect, and receipt records; regenerate Rust, JSON Schema, OpenAPI, and TypeScript | Unknown fields, duplicate keys, unsafe numbers, aliases, wrong tagged OID, partial IDs, and generated drift fail | Clean W0; WP-03 |
| DF-102 | Canonical bytes and identity: RFC 8785 JSON, domain-separated BLAKE3, full-width IDs, exact request digests, one-use nonces | Cross-type/domain substitution, non-canonical equivalent bytes, overflow, replay, and digest aliasing fail | DF-101 |
| DF-103 | Trust admission: PASETO authority, Ed25519 receipts, signer roles, trusted time, revocation, high-water, exact-family and dependency closure | Self-selected policy/root, wrong role/family, expiry boundary, revocation, stale high-water, and generic-receipt substitution fail | DF-101/102; OD-E custody for release evidence |
| DF-104 | Schema-3 `family.lock`, external-component lock, signed immutable member/wire/Jeryu subjects, and non-circular Hub-last tag order | Branch, sibling path, mutable URL, missing OID/tree/digest, tag/lock cycle, and second-resolution drift fail | OD-D, reviewed Jeryu tag, DF-101–103 |
| DF-105 | Exactly two model locks—authority/lease/fence and effect/check/integration—plus generated traces bound to receipts | Model drift, missing trace, extra authoritative model, and trace/receipt mismatch fail | DF-101–103 |

W1 exits only when two fresh resolutions produce identical authenticated bytes
and every receipt kind has a hostile-tested semantic verifier. It closes the
language and subject boundary, not a transaction.

## 5. W2–W3: durable authority and isolated execution

### W2 work packets

| Packet | Owning code | Exit |
| --- | --- | --- |
| DF-201 normalized truth | Kernel domain, SQLite migrations, adapters | Mission, graph revision, Variant, Attempt, scope, policy/config generation, authority epoch, budgets, leases, fences, freeze, intervention, and outbox are constrained columns/relations rather than authoritative blobs |
| DF-202 atomic authority | Kernel application/adapters | One serialized write transaction commits state, event, audit link, hash chain, and outbox; concurrent acquire, expiry, zero-row renewal, and quota races cannot over-authorize |
| DF-203 capability custody | Kernel authority and Runner transport | Durable peer registry, operator-admitted signing key, nonce/grant high-water, exact `SO_PEERCRED` binding, product Runner client, and lost-response read-back survive restart |
| DF-204 storage continuity | Kernel SQLite/CAS/GC/backup | `synchronous=FULL`, safe GC, audit-root continuity, verified backup/restore, external high-water, and `SAFE_STOPPED` on corruption or ambiguity |
| DF-205 recovery mode | Kernel control and services | `RECOVERING` invalidates leases/grants/credentials, reconciles remote truth, and requires independent recovery approval before mutation resumes |

### W3 work packets

| Packet | Owning code/operations | Exit |
| --- | --- | --- |
| DF-301 identities | Service packaging and UDS boundaries | Distinct control, runner, BulletGit, verifier, broker, attestor, integrator, observer, auditor, and Jeryu users; no cross-role state or credential read |
| DF-302 S1 workcell | Runner, rootless `crun`, cgroup/seccomp/network policy | Read-only source/root, private HOME/tmp/cache, bounded writable roots, default-deny egress, resource limits, and full process-tree teardown pass hostile isolation tests |
| DF-303 lifetime/secret projection | Runner and credential broker | Monotonic kill timer starts before Git/provider/forge; short-lived role-scoped credentials are projected only after authority; canary never appears in logs/artifacts |
| DF-304 S2 boundary | Runner/Firecracker package | S2-required policy refuses before spawn until an exact guest image has its own containment receipt; no S2→S1 downgrade |

W2/W3 exit evidence must include crash, restore, UID spoof, socket swap, escape,
egress bypass, fork bomb, resource exhaustion, canary, and survivor hostiles. A
debug registry, ephemeral key, inherited HOME, or process-local recovery map is
not production authority.

## 6. W4–W5: exact Candidate to reconciled integration

| Packet | Deliverable | Exit proof | Current component fact |
| --- | --- | --- | --- |
| DF-401 mutation permit | Kernel durably reserves a one-use operation and BulletGit repeats the final online lease/fence/subject check before I/O, then settles or reconciles it | Stale/replayed/wrong-scope permit and authority loss perform no Git mutation | The retained bridge durably admits its ScopeGrant, uses peer-authenticated farmd/Runner, and requires a Kernel-issued exact Candidate grant plus final check before production Gitd prepares the one-use Candidate. This is private component authority, not an admitted release reservation. |
| DF-402 private generation | BulletGit applies one exact `PatchProposal` through dirfd/openat2 into a private generation with inode-safe locks, fsynced journal/CAS/tree, and atomic active-generation switch | Traversal, `.git`, symlink/reparse, hostile config/filter/attribute, binary substitution, crash, and ENOSPC hostiles fail closed or resume exactly | Production Gitd is on the retained path and stale fence is refused; the full filesystem, crash, ENOSPC, and immutable published-subject exit proof remains required. |
| DF-403 Candidate | Candidate and Integration manifests bind repository, lineage, graph, Attempt, scope, policy, environment, toolchain, gates, and distinct proof roots | Candidate reconstructs byte-for-byte; rebase/merge/result-OID changes invalidate identity | Kernel chooses the exact Candidate preparation subject, production Gitd returns the one-use Candidate, fixture verification refuses writer identity and then records PASS, and purpose-signed fixture intent/evidence/proof plus every effect retain the same Candidate/head/tree. The outer receipt remains unsigned and both eligibility flags are false. |
| DF-404 forge port | Capability handshake, expected-old-OID delivery, immutable-ref read-back, idempotent check/PR, protected integration, target read-back, observation, reconciliation; one active primary profile | `LocalBareForge` proves `UNKNOWN` and `ORPHANED_REMOTE` without overwrite or duplicate logical effect | The retained `LocalBareForge` chain delivers and authoritatively reads back the exact Candidate head, refuses stale fence, reconciles lost response `UNKNOWN` to `COMMITTED`, publishes/read-backs the exact-SHA check and actual ProofBundle root, performs protected expected-old-OID integration, purpose-signs authoritative target outcome `MATCHED`, and proves all subjects again after reopen. Private ordinary-Git source/Candidate/target artifacts and the ledger are retained; the shell independently reads the exact three Git subjects after child exit. These remain local fixture components, not external-forge evidence. |
| DF-501 independent verifier | Verifier-owned identity/workcell reconstructs the exact Candidate and signs Evidence/ProofBundle under immutable `GateSpecV1` | Author/writer cannot self-qualify; zero/skipped/flaky/timeout/infra/unknown never becomes PASS | The retained harness now issues purpose-separated PASETO v4.public/JCS `VerificationIntentV1`, `EvidenceV1`, and `ProofBundleV1`, reconstructs its ephemeral public subjects, and reverifies the canonical chain. The executor and signers still share the harness process; the keys are self-asserted `FIXTURE_KEY_ONLY` subjects with no trusted lifecycle, durable nonce consumption, distinct verifier UID/credentials, or independently owned verifier artifact custody, so independent-evidence eligibility stays false. |
| DF-502 five-authority effects | Distinct broker, attestor, integrator, and observer execute the legal order only | Candidate → ProofBundle → delivery/read-back → check/read-back → integration/read-back → Observation; every retry reads remote truth first | The purpose-signed fixture ProofBundle now feeds the complete local order and reopen; the caller-free fixture observer authoritatively reads the target, signs `ObservationV1` as `MATCHED`, and reconstructably reverifies it. Broker/attestor/integrator/observer are still not distinct OS identities or credential/key owners, so the signed Observation remains ineligible component truth. |
| DF-503 audit closure | Observer/auditor bind commands, identities, subjects, outcomes, and hash-chain anchors without receiving mutation credentials | A clean read reconstructs the effect history; audit loss or mismatch blocks completion | The nested verification/Observation chain is signed and private source/Candidate/target plus ledger artifacts are retained, but the containing JSON remains an unsigned `COMPONENT_PROOF`, not a signed audit or release receipt. |

From the `bullet-kernel` checkout, preserve the current component boundary with:

```bash
BULLET_GITD_BIN="${BULLET_GITD_BIN:?set exact absolute canonical daemon}" \
BULLET_GITD_SHA256="${BULLET_GITD_SHA256:?set exact lowercase SHA-256}" \
just proof-transaction-offline
```

The retained diagnostic is
`/tmp/bullet-offline-component-proof.observation-20260827T1340Z/COMPONENT_PROOF.receipt.json`,
SHA-256 `b0aeadaebc834dd20e6c8f885b48d1868c0b4005d34a05b500fd27e23b6e5eb2`.
It binds Candidate
`can_6a1825080083e84b1f6a834ba11281e13dcb5bc08ec57844531f98379e215e3a`,
base `571e25fe7171eb98d00d4477481a3223f8e45b32`, head
`c60e2d674cc27b8520e16ff4b961ade355cddc64`, and tree
`618430ce7ff8883985bf50af5c074b07626ddc4d` through the nested signed chain
and every local effect. The actual ProofBundle/check/protection root is
`prf_9f433d2583c534e27a5a8cb853be7b42d05c31981e673f0cbf3bc6efa8f514e4`.
Reconstructed ephemeral public subjects reverify the verification chain at
BLAKE3 `e562a67094f4c54d3fbf7943555df54d204500d64d4b24a9709a3269da20e216`
and the signed `MATCHED` Observation at BLAKE3
`571d1f416ae67d9fb50a3ba66374a82aa3beda3011fc62cbe62edc9f3b191855`.
After child exit, the shell independently reopens exact source HEAD, Candidate
HEAD/tree, and target HEAD under retained sibling `artifacts`; the regular
private ledger remains at `data/ledger.sqlite`. The outer receipt remains
`UNSIGNED_FIXTURE`, nested records remain `FIXTURE_KEY_ONLY`, and independent,
transaction, and release eligibility are hard false. After independent
key/nonce/UID/credential/artifact custody and semantic receipt admission land,
this same command is the next honest read-back; the harness fixture cannot
update an eligibility flag.

This is the center of full-product dogfood. W4 may create an exact Candidate;
only W5 can independently qualify and reconcile it.

## 7. W6–W7: operable product, transaction, and package

### W6 operator surface

- DF-601 serves operator commands only at `/api/v1` and workload commands over
  peer-authenticated `/internal/v1`; legacy `/v1` refuses without mutation.
- DF-602 requires idempotency key plus expected revision and returns a durable
  `CommandReceiptV1`; signed dispatch and typed read-back survive response loss.
- DF-603 provides bounded snapshots, pagination, resumable authenticated SSE,
  atomic watermarks, queue backpressure, and `RESYNC_REQUIRED` on gaps.
- DF-604 makes every selected Portal surface durable or explicitly
  `OUT_OF_PROFILE`; missing subjects and unavailable read-back stay visible.
- DF-605 embeds manifest-verified Portal bytes at the farmd origin with CSP,
  OIDC+PKCE/off-loopback TLS, origin/CSRF/RBAC controls, WCAG 2.2 AA automation,
  and a retained manual review.

Current W6 component fact: a retained exact-subject wrapper authenticates an
idempotent public `run_demo` POST, survives farmd restart, replays and polls the
same command/request through the packaged Portal, dispatches it to a registered
same-UID `SO_PEERCRED` UDS Runner and bounded exact worker, admits the retained
fixture transaction receipt, atomically settles the same command/request/raw-
receipt BLAKE3 to durable `UNKNOWN`, and reads `NO_COMMAND` after worker restart.
The outer result remains `COMPONENT_PROOF` / `UNSIGNED_FIXTURE`, nested records
remain `FIXTURE_KEY_ONLY`, and every eligibility flag is hard false. This closes
no DF-60x exit: signed `CommandReceiptV1`, operator custody, distinct identities,
process-level response-loss and twelve-boundary chaos evidence, remaining
projection/SSE guarantees, and package/install admission are still absent.

### W7 proof and lifecycle

| Packet | Required artifact | Acceptance |
| --- | --- | --- |
| DF-701 vertical dogfood scenario | One low-risk Bullet docs/test task entered through farmd, executed by simulator under a real grant, written by production BulletGit, independently verified, integrated into `LocalBareForge`, observed, and projected in Portal | Exact IDs and subjects agree across all five authorities and after restart |
| DF-702 twelve-boundary campaign | Signed fault bundle for grant, Runner start, workspace open, provider completion, patch apply, checkpoint, Candidate prepare, verifier handoff, delivery, check, integration, and observation/cleanup | Death, timeout, response loss, stale authority, freeze, clock shift, ENOSPC, and restart never duplicate or falsely complete work |
| DF-703 `TRANSACTION_PROOF` | `just proof-transaction-offline` emits one signed exact-family `GateReceiptV1` plus sanitized artifacts | Kind-specific admission succeeds from an absolute registry; changed family/policy/toolchain/evidence fails |
| DF-704 Ubuntu package | Reproducible x86_64 Ubuntu 24.04 archive with embedded Portal and binaries, OCI/S2/Jeryu assets, checksums, CycloneDX/SPDX SBOM, provenance, signatures, and rollback bytes | Different-identity network-bounded rebuild produces identical admitted artifacts |
| DF-705 lifecycle | Two clean installs from the same schema-3 lock; activate, upgrade, backup, restore, rollback, uninstall-with-retention, and disaster recovery | Exact read-back, non-circular manifests, authority high-water preservation, no ambient tool/credential substitution |
| DF-706 assurance | Portable audit, advisories/licenses/sources, secret/workflow/CodeQL/fuzz/sanitizer/chaos/accessibility gates, exact paper/brief rebuild | Jankurai ≥90, zero caps/hard findings, no skipped/missing result, artifact hashes match documentation |

DF-703 is the first honest **full-product offline dogfood**. DF-705 is the
first stranger-installable lifecycle. Neither authorizes a live provider or
forge.

## 8. W8: first live self-hosted dogfood

1. Finish the local provider-policy/enrollment-anchor consumers, exact runtime
   probe, launch/egress/teardown receipts, budgets, and onboarding UX before
   requesting credentials.
2. Consume OD-D/E custody and the operator-admitted OD-A Claude enrollment.
   Run one bounded native Claude conformance task on the exact W7 family. Stop
   on runtime/model/profile drift, budget uncertainty, canary leak, or teardown
   ambiguity.
3. Finish the offline Jeryu semantic adapter and receipt admission before
   consuming OD-B. Admit the exact deployment passport, protected repository,
   and distinct broker/attestor/integrator/observer credential handles.
4. Send the same exact Candidate through protected Jeryu delivery, check,
   integration, target read-back, observation, backup/restore, and drift proof.
   A lost response remains `UNKNOWN` until authoritative read-back.
5. Admit provider, forge, package, operations, and two-install receipts through
   their kind-specific verifiers. Run `check release --profile self-hosted-v1`
   from the admitted absolute registry.
6. Only after PASS, record install and Claude task media from those same signed
   subjects. Media is a projection of proof, never the proof itself.

## 9. W9–W11: all work beyond first GA

| Phase | Independent branches | Completion condition |
| --- | --- | --- |
| W9 evolution | CognitiveTask/SelectionGroup/Role/Fusion persistence, budgets/routing/struggle, immutable recipes, matched-compute study, holdout custody, shadow, rollback readiness; then OD-H bounded ≤1% R0/R1 canary, promotion and drift | `evolution-v1` passes independently; it is never implied by self-hosted or universal |
| W10 providers | Codex, Cursor, Antigravity exact runtime/enrollment/conformance receipts under OD-A | Four provider receipts exist; none substitutes for another |
| W10 forges | GitHub App under OD-C, GitLab.com under OD-I, self-managed GitLab under OD-J | Jeryu plus three independent forge profiles pass exact protected integration and reconciliation |
| W10 platforms | Linux aarch64, macOS x86_64/arm64, Windows x64 build/install and certified containment or typed mutation refusal | Five platform slices install twice and pass their exact profile; Linux remains the mutation reference until separately certified |
| W10 research | Preregistered matched corpus with subjects, costs, failures, and receipts | No benchmark/superiority claim precedes admitted comparable evidence |
| W11 team | PostgreSQL authority, remote runners, workload mTLS/SPIFFE, object storage, durable stream, partition/failover fencing, distributed restore | `team-v1` passes before saga work receives credit |
| W11 saga | Staged multi-repository Candidates, dependency quarantine, forward repair, compensation as new effects, exact global read-back | `saga-v1` passes without rewriting prior history |

## 10. Completeness crosswalk

No registered work may disappear between documents and implementation.

| Closure band | Gaps | Workplan | Operator facts |
| --- | --- | --- | --- |
| R/W0 coordination and baseline | control prerequisite to all G rows | WP-01, 11, 20, 22 groundwork | none |
| W1 contracts/subjects | G1, G4, G5, G12 | WP-01, 03, 07, 19, 22 | OD-D/E |
| W2/W3 authority/isolation | G2, G3, G8, G10, G14 | WP-02, 12, 16, 21 | later service custody; no live credential |
| W4/W5 writer/verification/effects | G2, G4, G6, G7, G14, G16 | WP-02, 04–06, 16, 19 | live forge acts wait |
| W6/W7 product/package | G1, G2, G8, G9, G12–G14 | WP-01–05, 11, 13, 20, 22 | OD-D/E |
| W8 self-hosted | G5, G6, G9, G10, G12–G14 | WP-06–08, 12, 13, 19, 21, 23 | OD-A/B/D/E; OD-G only for public topology |
| W9 evolution | G11, G13, G15 | WP-15 | OD-H after offline prerequisites |
| W10 breadth | G5, G7, G9, G10, G12, G16 | WP-06, 09, 12, 15, 17, 21, 23 | OD-A/C/G/I/J |
| W11 distributed/saga | G17, G18 | future profile work routed by the gap register | distributed infrastructure custody |
| Optional/post-V1 governance | no release gap may be cleared by it | WP-09/10/14/17; WP-18 retired | OD-F/G clear no release gate |

The W0 typed bidirectional inventory must fail if any G1–G18, WP-01–WP-23,
OD-A–OD-J, W0–W11, invariant, receipt kind, runtime owner, test, or gate lacks a
reverse link. This table is a human route, not that executable inventory.

## 11. Ownership and safe parallelism

- One owner at a time edits a trust-boundary seam. Recovery verifier/ledger,
  recovery CLI, receipt producers, and live incident execution remain
  serialized.
- After Phase R, standalone lane repair can run independently in Hub, Kernel,
  BulletGit, and Portal. The exact family lane is always serialized in its
  dependency order.
- After W1 freezes wire identity, Kernel W2, local packaging, Portal projection,
  and provider-onboarding refusal work may proceed on disjoint paths. They
  converge only on clean immutable subjects.
- W4 BulletGit authority and W5 verifier/effects may develop in parallel only
  against the same frozen contract fixtures; the vertical scenario does not run
  until both exact implementations are admitted.
- W10 provider, forge, and platform slices are independent. One failed slice
  blocks itself and `universal-v1`, not a previously admitted self-hosted build.
- Operator acts are requested only after their local consumer, negative tests,
  least-privilege scope, expiry, revocation, rollback, and read-back procedure
  are complete. No live secret waits inside an unfinished path.

## 12. Owner cadence and definition of done

Every packet handoff records exact files, commit/tree, cleanliness, commands,
tool versions, test counts, outcomes, artifact hashes, evidence class,
remaining holds, and independent reviewer. The owner board is reconciled after
every packet, not after a wave-sized batch.

The next executable queue is DF-R4 → DF-R5 → DF-R6 → DF-R7a → DF-R7b →
DF-W0a/b → DF-DOG0. Once DF-DOG0 passes, the vertical product queue is DF-101
through DF-105, DF-201 through DF-205, DF-301 through DF-304, DF-401 through
DF-503, DF-601 through DF-605, and DF-701 through DF-706. W8 begins only from
that admitted W7 subject.

“100%” still means all eight conditions in the
[finish execution plan](execution-plan.md#9-definition-of-finished) hold at
once. Until then, report the exact narrower evidence class and blocker.
