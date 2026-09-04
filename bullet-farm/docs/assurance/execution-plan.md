# Bullet Farm finish execution plan

Status: **ACTIVE owner board; planning only; not runtime, Evidence, receipt, or release authority**  
Owner: Bullet Farm maintainers  
Last reconciled: 2026-08-27

This page turns the [closure roadmap](closure-roadmap.md), the
[G1–G18 product-gap register](product-gaps.md), the
[opportunity workplan](../workplan.md), and the
[operator decision register](../decisions/0013-operator-decision-register.md)
into one serialized finish sequence. Those sources remain authoritative for
architecture, product truth, opportunity scope, and operator decisions. The
packet-level [full-product dogfood bridge](full-product-dogfood-plan.md) maps
this sequence to concrete implementation, hostile proof, and handoff exits.
The
executable release check always wins:

```bash
bullet-family check release --profile <profile> \
  --receipts <absolute-admitted-registry> --json
```

Every profile is currently `BLOCKED`. A checked box in this plan would report
work completed; it could not promote a gate by itself.

## 1. Current truth and immediate constraint

- The four repositories remain independent. No consolidation, sibling-path
  dependency, new worktree, remote change, publication, rule change, tag, or
  release is part of local closure work.
- README, code-map, credential-free Stage-1 media, and forge-neutral CI are
  implemented as component work. Public installation, live provider use,
  hosted CI proof, and connected transaction claims remain unavailable.
- The checked-in family lock is schema 2. It cannot authorize setup, immutable
  family proof, signed receipt admission, or release.
- The family coordination ledger is frozen around an ambiguous legacy append.
  Recovery must finish before new authoritative claims, handoffs, receipts, or
  clean-family evidence can be created.
- Dirty canonical trees and active path custody prohibit a family proof. A
  diagnostic over that union is never committed Evidence.
- Operator facts OD-A–OD-J remain open. Agents may implement the local
  consumers and typed refusals, but cannot manufacture keys, credentials,
  endpoints, protected repositories, provider enrollments, or ratifications.

The immediate dependency chain is therefore:

```text
coordinator recovery
→ exact recovery-adoption records
→ clean four-repository Wave-0 baseline
→ signed schema-3 contract and family subjects
→ durable authority and service isolation
→ BulletGit + independent verification + five-authority effects
→ API/Portal closure
→ offline TRANSACTION_PROOF + signed Ubuntu lifecycle
→ self-hosted-v1
  ├─ W9 evolution-v1 (separate; never an input to universal-v1)
  ├─ W10 provider/forge/platform slices → universal-v1
  └─ W11 team-v1 → saga-v1 (separate; team passes first)
```

The waves remain chronologically ordered. The branches above show profile
dependency only: `universal-v1` is self-hosted plus the Wave-10 breadth slices,
not evolution, team, or saga.

### Reading “100/100” without moving the goalposts

The roughly 43/100 estimate in [Path to 100](path-to-100.md) is a frozen
2026-08-25 snapshot, not today's score. It must not be carried forward as if
the current dirty subjects had been independently re-scored. The current
executable release fact is simpler: every selected profile is `BLOCKED`.

This owner board therefore reports dependency progress, not a substitute
percentage. A row advances only on its named clean subject and exit proof:

| Threshold | Current state | What has actually advanced | Next fact required |
| --- | --- | --- | --- |
| D0 recovery consumer | **IN PROGRESS** | R1–R3.1 contracts and the R4 consumer/hostile implementation exist on a clean combined proof subject | Warning-fatal static gates, complete exact-subject rerun, frozen delta/tree, and independent review |
| D1 operable recovery | **NOT YET PROVED** | Interfaces and rehearsal shape are specified | Fixed-output proof/review producers, supervised CLI, and fresh-family rehearsal |
| D2 real incident | **BLOCKED BY D1** | Immutable incident inputs are retained; no live recovery is claimed | Independent review, supervised execution, and restart read-back |
| D3 coordination dogfood | **BLOCKED BY D2/W0** | The schema-2 coordination contract is component-tested | Four clean heads and one Bullet-owned change through the complete coordination loop |
| D4 offline product | **BLOCKED** | A retained watchable public component connects authenticated duplicate `run_demo` POST, farmd restart, packaged Portal replay/poll, same-UID UDS worker claim, retained fixture transaction, durable exact-digest `UNKNOWN`, and worker restart; it remains `COMPONENT_PROOF` / `UNSIGNED_FIXTURE` with every eligibility flag false | One signed five-authority `TRANSACTION_PROOF` over all twelve boundaries |
| D5 live self-hosted | **BLOCKED** | Offline provider protocols are contract-tested | D4 plus admitted Claude/Jeryu authority and provider/forge LIVE receipts |
| D6 documented target | **BLOCKED** | G1–G18, WP-01–WP-23, OD-A–OD-J, and Waves 0–11 are inventoried | Every profile-specific exit and all eight finish conditions in section 9 |

The historical score can be regenerated only after immutable Wave-0 subjects
exist and a different provider family re-scores admitted evidence. Until then,
“distance to 100” means the unclosed D0–D6 rows and their routed G/WP/OD gaps.

### Fastest honest self-dogfood runway

“Dogfood” has three different meanings here. They must not be collapsed:

1. **Development-coordination dogfood** uses the recovered schema-2 ledger for
   Bullet's own claims, heartbeats, handoffs, sole-writer receipts, exact retry,
   and restart read-back. This can begin after Phase R and the clean W0 baseline.
   It is `COMPONENT` use, not a connected coding-agent transaction.
2. **Offline transaction dogfood** runs one Bullet change through all five
   authorities and emits the signed twelve-boundary `TRANSACTION_PROOF`. This is
   the W7 exit and the first honest product transaction.
3. **Live self-hosted dogfood** adds an operator-admitted Claude subject and
   protected Jeryu effect after W7. This is W8 `LIVE`; it is not universal-v1.

The owner queue below is the shortest path to the first threshold and then the
connected product. “Now” means local implementation may proceed; it does not
authorize the frozen incident or an external effect.

| Order | Gate and current gap | Required implementation/output | Exact exit proof and stop condition | Primary owner surface |
| --- | --- | --- | --- | --- |
| D0 | Freeze recovery-adoption consumer (`R4`, now) | Exact delta over accepted R3.1; atomic group adoption; sealed record constructors; complete forensic, generation-envelope, Git, replay, and idempotency checks | Clean ordinary clone; serialized coordinator suite; public coordinator suites; all targets; rustfmt/diff/fsck; independent exact-subject review. Any missing/reordered evidence, lazy fetch, alternate object store, changed checkout, stale watermark, or replay effect is red | Hub coordinator model/state/store/ledger/recovery verifier |
| D1 | Make recovery operable (`R5/R6`, next) | Two fixed-output store transactions: coordinator-executed proof PASS and independent review APPROVE; creation-free incident inspection; canonical request generation; explicit supervised adopt command; no caller-selected disposition, actors, hashes, time, or result | Synthetic recovery→proof→review→adopt→restart rehearsal from a fresh 0700 family; exact retry invokes no process/clock/write; changed request conflicts; failed/skipped/unknown proof cannot append; reviewer cannot equal recovery orchestrator; CLI JSON is deterministic and redacted | Hub coordinator store plus narrow `bullet-family coord recovery` CLI |
| D2 | Resolve the frozen real incident (`R7`) | Independently review the immutable incident manifest, retained artifacts, recovery commit groups, proof commands, and adoption requests; execute the already-designed recovery once; adopt every reviewed break-glass group; read back after restart | Preflight and rehearsal hashes equal reviewed subjects; live operation has no unreviewed path; current schema-2 generation and complete watermark read back; no frozen/unreceipted claim remains unexplained. Any drift, writer ambiguity, `UNKNOWN`, missing reviewer, or different Git object halts without retrying a write | One supervised recovery operator plus independent reviewer; no provider or forge credential |
| D3 | Establish W0 and begin coordination dogfood | Commit reviewed paths in each independent repo; run standalone atomic/required lanes; run family order BulletGit→Kernel→Portal→Hub twice; freeze the unsigned baseline observation; require all new Bullet work to claim and hand off through schema 2 | Four clean immutable heads; byte-stable family reports/media; zero missing/skipped/zero-test/orphan partition; claim→heartbeat→handoff→sole-writer receipt→restart read-back succeeds on the first low-risk docs/test change. `AGENT_CHAT.md` remains collaboration context, never machine authority | Four repo owners; Hub coordinator is claim authority; orchestrator remains sole Git writer |
| D4 | Close the offline product (`W1–W7`) | Execute the existing W1–W7 queue: immutable signed wire/schema-3 authority; durable SQLite/Runner/mutation permits; S1 isolation; production BulletGit; LocalBareForge; independent verifier; five-authority effects; API/Portal; signed offline proof | Existing `just proof-transaction-offline` and the retained public wrapper remain component diagnostics. D4 exits only when the product emits one admitted signed `TRANSACTION_PROOF` and all twelve crash/timeout/loss/stale/freeze/clock/ENOSPC boundaries reconcile without duplicate or false-subject success | Hub contracts/release, Kernel authority/runner/verifier/effects/API, BulletGit writer, Portal projection |
| D5 | Start live self-hosted dogfood (`W8`) | Consume OD-A Claude enrollment only after D4; run exact runtime/egress/budget/teardown conformance; then consume OD-B Jeryu test authority and execute one low-risk protected integration | Provider-specific sealed LIVE receipt, protected-ref/check/integration observation, canary/credential non-leak, rollback readiness, and selected `self-hosted-v1` release check PASS. Any ambiguous remote effect remains `UNKNOWN` pending authoritative read-back | Operator custody plus Runner/provider, Effects/Jeryu, release verifier |
| D6 | Reach the complete documented target | Finish independent W9 evolution, W10 provider/forge/platform breadth, and W11 team then saga profiles; close every G/WP/OD row rather than borrowing another profile's receipt | All eight conditions in “Definition of finished” below hold simultaneously from the same admitted subjects | Profile owners and external operators named by ADR 0013 |

The immediate executable sequence is therefore:

```text
D0 exact R4 freeze
→ D1 proof/review producers + inspect/adopt CLI
→ D1 sealed rehearsal
→ D2 independently reviewed incident recovery/adoption
→ D3 clean W0 baseline
→ first schema-2 coordinated Bullet-on-Bullet docs/test change
→ W1–W7 connected offline transaction
→ W8 first live self-hosted change
→ W9/W10/W11 complete target
```

For D3 development work, every agent must follow one operating loop: creation-
free status read; exact generation/watermark capture; bounded claim; heartbeat;
proof-bearing handoff with no commit; sole-writer Git mutation and receipt;
restart read-back by request ID; typed reconciliation before any retry. A stale
generation, changed subject, failed proof, missing receipt, or `UNKNOWN` stops
the lane. No direct commit, second writer, inferred success, provider launch,
or remote mutation is permitted by “dogfood.”

The next implementation owner should not start W1 product work before D0–D3
settle. Read-only design, tests, and package preparation may proceed on disjoint
paths, but the fastest useful feedback comes from making the coordinator safe
enough to govern Bullet's own clean Wave-0 work first.

## 2. Rules for taking credit

Every evidence assertion uses exactly one of the five classes below. Skipping a
class is a defect.

| Evidence class | Required fact | Explicitly does not prove |
| --- | --- | --- |
| `DESIGNED` | Closed schema, invariants, negative cases, owner, and verifier are reviewed | Implementation |
| `COMPONENT` | Atomic local lane passes on an exact clean repository subject | Connected transaction, live use, or release |
| `TRANSACTION` | One signed exact-family offline transaction crosses all five authorities and fault boundaries | Live provider, live forge, or package lifecycle |
| `LIVE` | One operator-admitted provider or forge subject passes its own semantic receipt and read-back | Another provider/forge or release |
| `RELEASE` | Signed package/install/operations receipts and the selected profile's semantic verifier pass | Any unselected profile |

“Implemented” may describe internal work progress only. It is not an evidence
class or status assertion and earns no promotion until an exact `COMPONENT` or
higher subject is admitted.

For every transition, record the repository commit and tree, clean status,
command, tool versions, test count, outcome, artifact hashes, evidence class,
and independent reviewer. Failed, skipped, cancelled, missing, zero-test,
timed-out, stale, dirty, or `UNKNOWN` inputs never become PASS.

## 3. Phase R — restore coordination authority

This phase precedes Wave 0 and is local-only. It restores custody; it is not a
product or release milestone.

### R1. Freeze and bind the incident

- Preserve the exact interrupted, tainted, and frozen-live artifacts and the
  shared trusted prefix through retained, no-follow descriptors.
- Bind device/inode, owner, mode, link count, byte length, LF shape, SHA-256,
  trusted record-kind inventory, terminal claim-outcome inventory, and the
  forked post-prefix lineage in a closed manifest.
- Keep incident constants in the concrete rollover fixture/CLI boundary. Keep
  reusable manifest validation generic. Schema-1 recovery must identify its
  parent as `legacy-v1`.
- Derive the unique recovery baseline from the manifest. Never accept a
  caller-selected baseline, frozen projection, digest, or outcome.

### R2. Prove creation-free preflight and publication

- Perform every possible owner/mode/link/path, artifact, prefix, inventory,
  strict replay, and projection validation before creating or changing the
  coordination namespace.
- Bootstrap one permanent owner-0600 empty `LOCK` by retained coord-directory
  descriptor with create-exclusive/no-follow semantics, fsync, reopen, and
  identity read-back. Adopt a race only if it created that exact subject.
- Build, fsync, reopen, and byte/hash-verify the full immutable generation and
  all forensic copies before exchanging the legacy path.
- Bind the completed generation, manifest, source identity, baseline request,
  and internally derived request digest in the durable intent.
- Exchange the legacy name for a permanent mode-000 directory tombstone. A
  legacy binary must fail before append after that point.
- Prove writer quiescence as scan → exact retired-inode read → scan → exact
  retired-inode read. Any `/proc` ambiguity is `LEGACY_WRITER_UNKNOWN`.
- If a pre-open writer remains, stay frozen. If it later changes the retired
  inode, retry must refuse publication rather than blessing changed bytes.
- Publish `CURRENT` by staged, fsynced, exact read-back and no-replace rename;
  never write a partial final pointer.

### R3. Close replay, append, and hostile proof

- Reconstruct status from the exact manifest-bound schema-1 trusted prefix,
  the unique schema-2 recovery baseline, and the schema-2 suffix. Never treat
  the suffix alone as historical authority.
- Keep shared status creation-free. Type absent, legacy, frozen tombstone,
  corrupt, and current states without creating files or directories.
- Carry retained coord → generations → generation → segment/pending authority
  through reconcile and append. Bind segment and pending descriptors to the
  same exact current generation and revalidate before and after mutation.
- Require caller `request_id`, expected generation, and the complete watermark.
  Lookup idempotency before choosing time or identity. Derive IDs from canonical
  stable inputs, never PID, process order, or wall time.
- Prove fresh rollover, every durable crash prefix, resume, AlreadyCurrent,
  LOCK races, old writable FDs, `/proc` uncertainty, all artifact and inventory
  substitutions, owner/mode/link/path swaps, cross-generation descriptor
  substitution, equal-length rewrites, pending-intent replacement, response
  loss, and the pinned clean legacy executable.
- Require meta-tests that fail on a missing private test module or a zero-test
  partition. Keep every production file below 500 lines.

### R4. Adopt break-glass commits without rewriting history

- Add a distinct `RecoveryReceiptAdoptionV1`; do not reuse ordinary handoff or
  commit-receipt semantics for `FROZEN_RECOVERY` claims.
- Bind exact forensic record ranges, immutable Git commit/tree/leaf bytes,
  proof subjects, and an independent review to one canonical request and the
  complete current watermark.
- Transition a whole disjoint claim group atomically to
  `RECOVERED_RECEIPTED`; response loss replays the stored result, while the
  same group under a different request conflicts.
- First adopt the already inventoried Kernel and BulletGit break-glass commit
  groups, then every reviewed Hub recovery commit. Local OS recovery authority
  remains COMPONENT evidence only.

Phase R exits only when recovery and adoption hostile suites pass on a clean
exact subject, the permanent ledger is read back after restart, and ordinary
claim/heartbeat/handoff/receipt operations use only schema 2. Until then,
coordinator mutation and all family proof remain frozen.

## 4. Phase W0 — establish the clean family baseline

1. Finish and independently review each active Hub, Kernel, BulletGit, and
   Portal change without crossing path custody.
2. Run each repository's atomic `fast`, `lint`, `contract`, `security`, and
   `docs` lanes, then its sequential local `required` lane. Repair the first
   real failure; do not suppress it or hide it behind an aggregator.
3. Close standalone-family leaks: Kernel family-only BulletGit tests live in
   an explicit inventory, and Portal real-farmd browser proof runs only in the
   provisioned family lane.
4. Commit each repository independently from reviewed paths. Record exact
   commit/tree/cleanliness and keep unrelated user changes intact.
5. Build BulletGit first; pass its exact absolute daemon path to Kernel; run
   Kernel component and family tests; run Portal component and real-farmd
   browser tests; run Hub contracts/models last.
6. Run `doctor`, `fast`, `demo`, README media checks, generated-drift checks,
   and the full family lane from those immutable clean subjects.
7. Replace the held Markdown-scraping orphan report with a typed bidirectional
   inventory over every G-id, V1-S slice, historical phase, invariant,
   paper/workplan row, runtime enforcement/owner, test, gate, receipt kind, and
   the complete Waves 0–11 roadmap. Mark historical Centerrail plans as
   provenance and classify every status assertion as exactly `DESIGNED`,
   `COMPONENT`, `TRANSACTION`, `LIVE`, or `RELEASE`.
8. Freeze one deterministic unsigned `bullet.ci-observation.v1` baseline. It
   remains component diagnostics until Wave 1 admits a signed
   `BaselineReceiptV1` over the unchanged subject.

W0 exit: four clean heads; standalone and exact-family lanes green; no missing,
skipped, or zero-test partition; no orphan control row; README media reproducible
twice; release still `BLOCKED`.

## 5. Phases W1–W7 — close the offline product

| Phase | Local deliverable | Required hostile/read-back exit | Counting gap ownership; workplan routing |
| --- | --- | --- | --- |
| W1 contracts | One signed immutable `bullet-wire-v1`; signed member tags and reviewed Jeryu capability tag; schema-3 family/external lock; closed receipt schemas and semantic registry; trust roots, revocation, replay, trusted time, exact-family closure; two model locks | Two clean resolutions produce identical bytes; every receipt kind rejects substitution, unsafe JSON, wrong role/family, replay, expiry, and self-selected policy | G1, G4, G5, G12; WP-01, 03, 22; OD-D/E inputs plus reviewed Jeryu tag |
| W2 authority | Normalized SQLite authority; durable budgets, leases, fences, nonce/grant state, peer registry, Runner UDS recovery, backup/restore and high-water | Crash/restore reconstructs exact audit root; concurrency, expiry, response loss, corruption, quota, and stale authority cannot mutate | G2, G3, G14; WP-02, 16 |
| W3 isolation | Distinct service users; S1 rootless OCI; brokered short-lived secrets; monotonic self-kill; S2 typed refusal until certified | Escape, egress, canary, survivor, UID spoof, resource exhaustion, and required-S2 downgrade fail closed | G2, G3, G8, G10; WP-12, 21 are non-counting provider prerequisites |
| W4 writer/forge | Online-final BulletGit permit; private generation/CAS/journal; exact Candidate; semantic forge port and `LocalBareForge` | Traversal, hostile Git, stale permit, binary drift, crash, response ambiguity, and second-primary cases yield refusal/`UNKNOWN`/`ORPHANED_REMOTE` | G2, G4, G6, G16; WP-04, 05, 06, 16, 19 |
| W5 verify/effects | Independent reconstruction, Evidence/ProofBundle, distinct broker/attestor/integrator/observer, exact desired-state reconciliation | Full Candidate→proof→delivery→check→integration→observation chain; author cannot self-attest; no duplicate effect or ambiguity laundering | G2, G6, G7, G14, G16; WP-02, 05, 06 |
| W6 API/Portal | `/api/v1`, internal UDS commands, command receipts, durable projections, SSE resync, selected-profile surfaces, Shift Brief, same-origin bundle, WCAG gates | Cross-role, CSRF, replay, stale revision, queue/cursor/snapshot races, missing subjects, and optimistic empty state never render green | G2, G13, G14; provider UX portion of WP-21 is non-counting |
| W7 proof/package | Signed twelve-boundary offline `TRANSACTION_PROOF`; reproducible Ubuntu 24.04 x86_64 package; install/upgrade/backup/restore/rollback/uninstall; paper build | Fault injection at all 12 boundaries; two clean installs from one schema-3 lock; SBOM/provenance/signature/read-back; security floor and exact artifact parity | G1, G2, G8, G9, G12; WP-01–05, 11, 13, 19, 20, 22 |

W7 exit is the first connected product proof. It still grants no provider
credential, live forge authority, public endpoint, or release profile PASS.

## 6. Phases W8–W11 — close live, breadth, and later profiles

### W8: `self-hosted-v1`

- At Checkpoint A, consume—never create—the admitted Claude enrollment and
  release custody. Run the bounded Claude conformance turn and admit its exact
  provider receipt before continuing.
- Run exact executable/model/profile probing, provider onboarding, launch grant,
  egress containment, heartbeat, interruption, teardown, canary, cost/budget,
  and retry/reconciliation tests.
- Only after Checkpoint A passes, consume the Checkpoint-B Jeryu test repository
  and distinct effect credentials. Run the same exact Candidate through the
  protected Jeryu chain and independent observation. Ambiguous remote outcomes
  remain `UNKNOWN` until read-back.
- Admit two signed installation receipts and the provider/forge/operations
  receipts through kind-specific verifiers. Only then record Stage-2 install
  and live-agent media.
- Exit only when the selected `self-hosted-v1` release check returns PASS from
  an admitted absolute registry. OD-A/B/D/E are external predecessors; their
  chat witnesses alone do not qualify. OD-G clears no self-hosted release gate.

### W9: `evolution-v1`

- Implement feasibility shield, roles, routing, budgets, struggle and negative
  knowledge, immutable recipes, evaluation vectors, matched-compute accounting,
  holdout custody, MOME/ASHA study, archive, and provenance-preserving fusion.
- Complete a frozen offline study and no-effect shadow with independent
  certifier/promoter/activator roles and automatic rollback readiness.
- Only after those pass may OD-H authorize one expiring ≤1% R0/R1 canary.
  Promotion, drift, and rollback receipts must then pass independently.
- Evolution never becomes an implicit property of `self-hosted-v1` or
  `universal-v1`.

### W10: independent breadth slices and `universal-v1`

- Certify Codex, Cursor, and Antigravity separately; one provider receipt never
  substitutes for another. Keep Cursor ACP unknown until an exact native entry
  point is positively probed.
- Certify GitHub App, GitLab.com, and self-managed GitLab independently from
  Jeryu, each with distinct capabilities, protection, identities, credentials,
  budgets, expiry, reconciliation, and rollback.
- Build, install twice, and prove typed mutation refusal or certified native
  containment on Linux aarch64, macOS x86_64/arm64, and Windows x64.
- Run the matched, preregistered comparison corpus before making any benchmark
  or superiority claim.
- Compose `universal-v1` only after every selected independent slice is admitted.
  It does not include evolution.

### W11: `team-v1`, then `saga-v1`

- Add PostgreSQL authority, mTLS remote runners, object storage, durable event
  streaming, failover/partition fencing, and remote secret/identity custody.
- Certify `team-v1` before implementing cross-repository saga authority.
- Add staged multi-repository Candidates, forward repair, compensation as a new
  authorized effect, partial-integration quarantine, and exact global read-back.
- No prior history is rewritten and no local single-host receipt qualifies.

## 7. Complete gap and dependency crosswalk

| Gap | Earliest closure | Final admissible proof | External dependency |
| --- | --- | --- | --- |
| G1 setup/install | W0, W1, W7 | RELEASE | OD-D/E |
| G2 five-authority transaction | W2–W7 | TRANSACTION | none for offline |
| G3 Runner/authority transport | W2, W3 | COMPONENT then TRANSACTION | operator service custody for release |
| G4 immutable wire/Candidate | W1, W4 | signed contract + TRANSACTION | OD-D/E |
| G5 live providers | W1, W8, W10 | provider-specific LIVE | OD-A per provider |
| G6 Jeryu effects | W4, W5, W8 | forge-specific LIVE | OD-B |
| G7 GitHub effects | W5, W10 | independent forge-profile LIVE | OD-C |
| G8 assurance floor | W3, W7 | RELEASE | portable Jankurai/external review inputs |
| G9 packaging/signing | W7, W10 | RELEASE | OD-D/E |
| G10 containment/platform | W3, W10 | containment + platform RELEASE | builders/custody |
| G11 evolution | W9 | evolution profile RELEASE | OD-H after offline gates |
| G12 semantic receipt admission | W1, W7, W8 | kind-specific signed admission plus requested profile condition | OD-E trust custody |
| G13 Portal product surfaces | W6, W9 | selected-profile projection plus packaged browser/install receipt | manual accessibility review for final AA |
| G14 farmd production API | W2, W5, W6 | connected TRANSACTION then selected-profile RELEASE | none for offline |
| G15 cognitive persistence | W9 | evolution profile RELEASE | OD-H only after offline study/shadow/rollback readiness |
| G16 GitLab adapters | W4, W5, W10 | independent `release.profile.gitlab-adapter-v1` and `release.profile.gitlab-self-managed-v1` receipts | OD-I and OD-J |
| G17 distributed team mode | W11 after self-hosted | team profile RELEASE | distributed infrastructure custody |
| G18 cross-repository sagas | W11 after G17/team PASS | saga profile RELEASE | team authority and distributed infrastructure custody |

The Wave-0 bidirectional inventory is a mandatory control and baseline subject;
it does not invent or replace a product-gap ID.

### Workplan routing

| Workplan | Execution owner | Dependency or boundary |
| --- | --- | --- |
| WP-01 paper evidence | W0/W1 reproducibility; W7 signed rebuild | Clean immutable subjects; public links wait on WP-14 |
| WP-02 connected proof | W2–W7 transaction path | Credential-free; no live operator act needed |
| WP-03 shared wire | W1 contract publication | OD-D/E supply authenticated subjects/custody |
| WP-04 Git admission | W0/W4 setup and writer paths | Exact digest-pinned Git; per-platform typed proof/refusal |
| WP-05 Candidate/Evidence identity | W4/W5 | Published wire and exact merge/invalidation subjects |
| WP-06 forge semantics | W4/W5 offline, W8/W10 live | Local adapters first; each live forge needs its own operator authority |
| WP-07 Jeryu governance | W1/W8 governance | Accepted authority RFC; no overlapping effect principal |
| WP-08 Jeryu endpoint | W8 operations | OD-B/OD-G facts, protection, backup, restore, and rollback |
| WP-09 optional mirror | W10/post-V1 operations | WP-06/08 plus separately ratified mirror; never source authority |
| WP-10 Jankurai preservation | Post-V1 governance | Preservation RFC and full ref/bundle inventory before any mirror act |
| WP-11 portable audit | W0/W7 security | Checksum-pinned hosted artifact; local binary is insufficient |
| WP-12 provider admission | W3 local, W8/W10 live | OD-A common policy plus one non-substitutable provider enrollment |
| WP-13 signed package | W7 | WP-03, OD-D/E, exact builders, signer and trusted time |
| WP-14 paper publication | After W7 signed preflight | OD-G immutable endpoint; publication cannot precede hash match |
| WP-15 matched benchmark | W9/W10 research | WP-02 exact corpus; no superiority claim before receipt admission |
| WP-16 mutation permit | W2/W4 | Durable Kernel reservation plus production BulletGit verification |
| WP-17 public topology | W8/W10/post-V1 | OD-G; clears no source or release gate by itself |
| WP-18 relocation | Retired | No implementation; preserve the existing Jeryu family |
| WP-19 locked Jeryu runtime | W1/W7/W8 | WP-03/08 and OD-D; consume signed runtime, never copy source |
| WP-20 Stage-1 README media | W0 | `COMPONENT` media only; clean immutable snapshot still required for refresh |
| WP-21 provider onboarding UX | W3 then W8/W10 | Local probes/guidance before OD-A live runs |
| WP-22 forge-neutral CI | W0/W1 local, hosted later | WP-03, WP-13, immutable hosted-runner provisioning, then OD-G runners/ruleset read-back; hosted diagnostics are not Bullet Evidence |
| WP-23 Stage-2 recordings | Install media after W7; provider-task media in W8/W10 | Install requires WP-13 two-install subject; provider tasks separately require WP-02 transaction, WP-12 exact probe/live receipt, and the same signed package |

### Operator routing

| Decision | Earliest consumer | Exact boundary |
| --- | --- | --- |
| OD-A provider policy/enrollments | W8 Claude; W10 other providers | Follows OD-D→OD-E trust anchor; each enrollment proves only itself |
| OD-B Jeryu test authority | W8 | Waits on offline adapter; does not depend on or substitute for OD-D |
| OD-C GitHub App authority | W10 | One protected test repository and four role-separated credentials |
| OD-D schema-3 source inputs | W1/W7 | Read-only source/tag/passport authority; no mutation credential |
| OD-E release/build/time custody | W1/W7 | Distinct roles and allowed-signers policy; follows OD-D signer identity |
| OD-F optional preview publication | Optional after local diagnostic | Clears no gate and permits no GA wording |
| OD-G public topology/hosted runners | Hosted CI and publication only | Clears no release gate; grants no product credential |
| OD-H bounded evolution canary | W9 | Only after self-hosted, offline study, shadow, and rollback readiness |
| OD-I GitLab.com authority | W10 | Certifies only GitLab.com |
| OD-J self-managed GitLab authority | W10 | Certifies only the exact self-managed endpoint/version |

## 8. Parallelism and ownership

Only independent paths may proceed concurrently:

- During Phase R, recovery, manifest inventory, retained descriptor/ledger,
  public store/CLI, and read-only adversarial review are serialized at their
  shared seams. One owner edits each exact path set; reviewers do not patch it.
- After Phase R, the four standalone repositories may close independent atomic
  lanes concurrently. The family proof remains serialized BulletGit → Kernel →
  Portal → Hub.
- In W1–W7, packaging can develop beside the transaction path, and docs/media
  can develop beside code, but neither receives credit before exact clean
  subjects and semantic receipt admission exist.
- In W10, provider, forge, and platform certifications are independent branches.
  Failure in one slice blocks only that slice plus `universal-v1`, not an
  already admitted `self-hosted-v1`.
- Operator work begins only when its local typed consumer, negative tests,
  runbook, least-privilege scope, expiry, revocation, and rollback path are
  complete. No credential should wait in a half-built product.

Every handoff states exact files, remaining holds, tests actually run, dirty
status, and evidence class. The next owner re-reads the coordination log before
claiming or editing. Any subject drift invalidates the pending observation and
returns that item to its last proven state.

## 9. Definition of finished

Bullet Farm is “100% of the documented target” only when all of the following
are simultaneously true:

1. G1–G18 have no unowned or unverified row in the typed bidirectional inventory.
2. Waves 0–11 have their exact exit receipts, including team before saga.
3. Every selected release condition passes its kind-specific semantic verifier
   from an admitted absolute registry; generated release truth matches it.
4. Four provider, four forge, and five platform slices are individually proven;
   no slice borrows another's receipt.
5. The twelve-boundary offline transaction, package lifecycle, live self-hosted
   chain, evolution study/canary/rollback, distributed failover, and saga repair
   subjects remain independently reproducible from immutable inputs.
6. README, paper, Portal, code map, runbooks, workplan, gap register, invariant
   registry, and release projection are generated or drift-checked against the
   same exact subjects and make no stronger claim than the admitted evidence.
7. External security/accessibility review findings are closed or explicitly
   accepted by ADR, and two independent re-scores agree within the documented
   tolerance without changing the rubric.
8. A clean signed tag can be installed twice by a stranger, operated, backed up,
   restored, rolled back, and uninstalled on every selected platform; protected
   integration and publication are read back from authoritative systems.

Until all eight are true, the truthful result is the exact narrower component,
transaction, live, or profile claim—or `BLOCKED`/`UNKNOWN`.
