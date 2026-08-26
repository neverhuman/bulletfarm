# Bullet Farm closure roadmap

Status: **ACTIVE plan; not runtime, receipt, or release authority**  
Owner: Bullet Farm maintainers  
Last reconciled: 2026-08-25

This is the dependency-ordered route from the current component-proved family
to the named release profiles. The executable decision always wins:

```bash
bullet-family check release --profile <profile> \
  --receipts <absolute-admitted-registry> --json
```

Every profile is currently `BLOCKED`. A completed engineering item is not a
release fact until its exact current-family receipt passes the condition-specific
semantic verifier. The historical 26-gate page and `linux-preview` are
diagnostics only.

## Product order

| Order | Profile or proof | Dependency | Honest meaning |
| ---: | --- | --- | --- |
| 1 | `TRANSACTION_PROOF` | Credential-free Waves 0–7 | One exact offline five-authority transaction plus its twelve-boundary fault receipt; not live or released |
| 2 | `self-hosted-v1` | Waves 0–8 | First GA: Ubuntu 24.04 x86_64/systemd, Claude, local Jeryu, signed install, operations, and security |
| 3 | `evolution-v1` | `self-hosted-v1` + Wave 9 | Separately certified offline study, shadow/canary, promotion, rollback; never implied by universal |
| 4 | Provider, forge, and platform profiles | Wave 10 | Independent certifications; no one slice certifies another |
| 5 | `universal-v1` | `self-hosted-v1` + all Wave-10 slices | Later composition of all four providers, Jeryu, GitHub, GitLab.com, self-managed GitLab, and five platforms; it does not include evolution |
| 6 | `team-v1`, then `saga-v1` | Wave 11 | Distributed team mode, then cross-repository sagas |

`linux-preview` may diagnose a subset of Ubuntu/Jeryu/Claude conditions, but
it cannot replace `self-hosted-v1` or authorize a tag.

## Seven functions, five authorities

The architecture has seven functional planes and five transaction authorities.
The mapping is intentional: a useful function does not automatically receive a
new credential or completion vote.

| Functional plane | Transaction authority | Principal boundary |
| --- | --- | --- |
| Control | Control | Kernel alone owns commands, graph state, policy, budgets, leases, fences, and dispatch admission |
| Cognitive execution | Execution | Classifiers, routers, councils, provider sessions, and fusion run only under an exact execution grant; they cannot mint authority, Evidence, or effects |
| Repository execution | Execution | Runner supervises; BulletGit is the sole Git writer; the two remain distinct principals inside the execution domain |
| Session supervision | Execution | Runner owns the provider process tree, protocol, heartbeat, interrupt, and teardown; it receives no forge or verifier credential |
| Independent verification | Verification | A separate identity reconstructs the exact Candidate and emits Evidence; writer output cannot self-qualify |
| Effect and delivery | Delivery/integration | Broker, attestor, and integrator use separate workload identities and reconcile exact desired state by read-back |
| Evidence and audit | Evidence/audit | Observer and auditor own durable observation and audit subjects; Portal is a non-authoritative reader |

No authority may substitute for another. Provider exit zero is not Candidate
identity, writer tests are not independent Evidence, dispatch success is not a
reconciled effect, and Portal green is not completion.

## Waves 0–11

### Wave 0 — freeze a truthful baseline

Objective: end with four clean canonical checkouts and no optimistic claim.

- Finish the README/media/CI union, run every standalone required lane, and run
  the exact family lane over immutable commits.
- Prove all four canonical checkouts standalone from fresh clones; remove
  Kernel's unprovisioned sibling BulletGit dependency and Portal's unprovisioned
  sibling-Kernel CI dependency without adding committed sibling paths.
- Bind each retained change to its machine claim, exact path set, commit, tree,
  commands, tool versions, and sanitized artifact hashes.
- Keep component observations unsigned and distinct from Bullet Evidence.
- Add a bidirectional control inventory over every G-id, V1-S slice, historical
  phase, invariant, paper/workplan row, runtime enforcement/owner, test, gate,
  and receipt kind; prove none of those surfaces or this Waves 0–11 roadmap is
  orphaned. Mark historical Centerrail plans as provenance and classify every
  status assertion as DESIGNED, COMPONENT, TRANSACTION, LIVE, or RELEASE.

Negative checks: dirty subjects, overlapping claims, omitted tests, zero-test
partitions, skipped jobs, missing artifacts, stale generated pages, duplicate or
non-finite JSON, and subject drift all refuse.

Exit into Wave 1: four clean observed heads, standalone/family lanes green, no
known orphan crosswalk row, and one deterministic unsigned, non-promotional
clean-family observation whose exact subject hash is frozen. This observation
is component diagnostics only—not `BaselineReceiptV1`, transaction, or release
evidence. Wave 1 implements the kind-specific verifier and admits a signed
`BaselineReceiptV1` over that unchanged frozen subject; subject drift returns
to Wave 0. Release remains `BLOCKED` throughout.

### Wave 1 — freeze contracts and receipt admission

Objective: make every later receipt refer to one immutable language and family.

- Publish signed immutable wire/member tags and the reviewed Jeryu capability tag.
  At the clean prospective Hub head, generate the schema-3 family and external-
  component locks from those authenticated non-Hub subjects, commit the lock,
  sign the Hub tag last, and verify that exact Hub tag contains the generated
  lock. Run `bullet-family lock generate --tag <prospective-version> --subjects
  <absolute-path>` before the Hub tag exists; generation reads Hub `HEAD`, so a
  pre-existing Hub tag would make the order circular.
- Publish one `bullet-wire-v1` artifact consumed by exact tag/digest from every
  repository. It owns recursively closed Authority, Transaction, Forge,
  Evolution, and Release records; eliminate duplicate Candidate/digest semantics
  and regenerate Rust, JSON Schema, OpenAPI, and TypeScript from that source.
- Enforce RFC 8785 canonical JSON, domain-separated BLAKE3, full-width typed IDs,
  tagged Git OIDs, PASETO v4.public authority, Ed25519 release receipts,
  one-use nonces, exact request digests, safe integers, and recursive
  unknown-field rejection across Rust and TypeScript hostile fixtures.
- Implement trust-root, signer-role, trusted-time, revocation, replay/high-water,
  exact-family, dependency-closure, and per-kind semantic receipt admission.
- Bind toolchains, dependency locks, policy, environment, command, and artifact
  subjects without circular manifests or self-selected signer policy.
- Retain exactly two formal models: authority/lease/fence and
  effect/check/integration. Bind model-lock digests and executable traces into
  receipts, and prove generation twice from fresh checkouts.

Negative checks: branch or sibling-path inputs, unknown schema/profile/receipt
kind, duplicate keys, unsafe numbers, replay, expired/revoked signers, wrong
family or gate, generic evidence substituted for a semantic receipt, and
manifest self-reference all refuse.

Exit: current signed schema-3 subjects resolve twice to identical immutable
bytes and every receipt kind has a hostile-tested semantic verifier. External
predecessors: OD-D, OD-E, and the reviewed Jeryu tag.

### Wave 2 — normalize durable control authority

Objective: Kernel becomes the sole durable scheduler and authority issuer.

- Cut one release SQLite baseline. Prototype databases move forward only by
  explicit export → verify → import; they are never silently migrated in
  place, adopted as release state, or destroyed.
- Normalize Mission, graph revision, Variant, Attempt, workspace generation,
  scope, policy/routing/configuration generation, authority epoch, budget,
  reservation, freeze, and intervention state; no JSON blob is authoritative.
- Allocate leases/fences and reserve token, cost, call, wall-time, concurrency,
  provider quota, invocation, CPU, memory, PIDs, disk, egress, output, artifact,
  verifier-backlog, effect, probe, and CAS-liability budgets atomically using
  database time. Unknown usage remains reserved.
- Mint short-lived one-use capabilities only from the current durable lease;
  bind audience, operation, request, subject, fence, runner, workspace, scope,
  policy, expiry, and nonce.
- Promote the component-proved Runner UDS transport: its registered Runner
  ID/epoch ↔ `SO_PEERCRED` UID binding, farmd UID plus socket GID/device/inode
  pinning, and durable server grant/nonce state are landed. Replace the
  debug-only peer registry and ephemeral process signing key with
  operator-admitted durable configuration/custody; persist client
  acquire/read-back recovery instead of process-local metadata; wire product
  Runner; and prove bounded lost-response recovery without a second write.
- Complete CAS retention, orphan-safe GC, audit-root continuity, and verified
  backup/restore with `SAFE_STOPPED` on ambiguity or corruption.
- Use a single serialized write actor, concurrent read pool, WAL, foreign keys,
  `synchronous=FULL`, normalized constrained identities, and append-only
  triggers; atomically commit state/event/hash-chain/outbox without
  `INSERT OR REPLACE` authority paths. Keep authority high-water and restore
  epoch outside the restorable database.
- Implement `RECOVERING`: invalidate leases, grants, credentials, activations,
  and cached authority, reconcile remote truth, and require independent recovery
  approval. Persist freeze generation, component acknowledgements, P0 incident
  queue, interventions, signed audit batches, and protected Jeryu audit anchors.

Negative checks: concurrent acquire, every capability-field mutation, replay,
exact-expiry boundary, zero-row renewal, authority outage, crash at every
SQLite/WAL/CAS/outbox boundary, corrupt state, quota oversubscription, and
restore mismatch yield no unauthorized mutation.

Exit: a durable-authority receipt reconstructs the exact audit root after crash
and restore. No provider or Git write is needed for this wave.

### Wave 3 — isolate services and project secrets

Objective: no provider, writer, verifier, forge effect, observer, auditor, or
Jeryu workload can borrow another role's operating-system identity or secret.

- Provision distinct control, runner, BulletGit, verifier, broker, attestor,
  integrator, observer, auditor, and Jeryu users with peer-authenticated UDS
  endpoints and role-specific state roots.
- Implement the S1 rootless `crun` OCI workcell: user/mount/PID/network
  namespaces, cgroup v2, seccomp, read-only root and repository, private
  HOME/tmp/cache, bounded scratch/artifacts, and default-deny networking.
- Start Runner's monotonic self-kill timer immediately after grant verification
  and before Git, provider, or forge startup; install cleanup guards at every
  startup, heartbeat, provider, apply, checkpoint, Candidate, verifier, and
  release boundary.
- Project only short-lived brokered credentials. No sandbox inherits host HOME,
  SCM/cloud credentials, another role's state, or a bearer-token workload
  fallback.
- Build and certify the S2 Firecracker guest path. A policy that requires S2
  refuses before provider startup until that exact guest is certified.

Negative checks: sandbox escape, fork bomb, resource exhaustion, egress bypass,
secret canary, process-tree survivor, cross-role state/credential read, UID
spoofing, timer delay, and S2 downgrade all refuse or tear down the full tree.

Exit: S1 isolation and service-identity receipts pass; S2-required work remains
fail-closed until its separate containment receipt passes.

### Wave 4 — close BulletGit authority and forge capability

Objective: BulletGit creates exact Candidates only under current Kernel
authority, and all primary forges implement the same honest semantic port.

- Replace `AUTHORITY_CONTRACT_UNAVAILABLE` with offline signature validation,
  Kernel online final lease/fence check, durable reservation, one-use operation
  permit, settlement, and restart reconciliation. Fixture authority is absent
  from release binaries; the legacy unsigned token is removed.
- Apply an exact `PatchProposal` into a private generation using dirfd/openat2
  beneath/no-symlink access and inode-safe locks. Fsync journal, CAS, tree, and
  directory before atomically switching the active generation.
- Bind Candidate and Integration manifests to mandatory toolchain, environment,
  policy, lineage, repository, graph, scope, gates, and every proof-root
  component; keep the two roots distinct. Admit an absolute digest-pinned Git
  binary, bounded output/deadlines, 128 paths, and 32 MiB aggregate content.
- Atomically persist Candidate, Attempt success, package `Prepared`, lease
  release, event, audit link, and verifier outbox. Bound replay archives and
  require signed audit linkage; cleanup requires a preservation receipt.
- Implement the forge semantic port: authenticated capability handshake,
  expected-old-OID delivery, immutable-ref read-back, idempotent PR/MR,
  exact-SHA proof check, protected integration, target read-back, observation,
  and reconciliation. Exactly one `PrimaryForgeProfileV1` generation is active
  per repository; `ReplicationIntentV1` is separate and never proves integration.
- Probe only an operator-named pinned Jeryu build. Missing capabilities are
  implemented and tagged in `/home/ubuntu/jain-split/jeryu-split`, never patched
  into Bullet or the running forge.

Negative checks: placeholder components, stale/replayed permits, preimage drift,
traversal, `.git`, symlink/reparse races, hostile Git config/filter/attribute,
sequencer states, binary substitution, crash at each generation boundary,
response ambiguity, and a second primary all refuse or become typed `UNKNOWN`/
`ORPHANED_REMOTE` without rewriting history.

Exit: one exact offline Candidate is reconstructible from immutable inputs and
production `bullet-gitd` reads it back under online authority.

### Wave 5 — independently verify and deliver through five authorities

Objective: connect exact-Candidate verification to delivery, check,
integration, and observation without identity collapse or ambiguity laundering.

- Accept only signed `VerificationIntentV1` and immutable `GateSpecV1` IDs.
  Reconstruct the Candidate/CAS/environment/toolchain in verifier-owned S1/S2
  workcells. Derive independence from OS identity, artifact custody, and
  conflict policy, never an environment boolean.
- Sign Evidence and ProofBundle with verifier-owned keys. Separate product and
  research holdout stores, users, keys, custodians, and query ledgers.
- Give broker, attestor, integrator, and observer distinct credentials and claim
  leases. Every retry reads authoritative remote state before writing.
- Enforce the only legal effect order:

```text
Candidate
→ ProofBundle
→ DeliveryGrant
→ candidate ref push/read-back
→ CheckGrant
→ exact-SHA check/read-back
→ IntegrationGrant
→ protected target integration/read-back
→ Observation
```

- Lost response remains `UNKNOWN`; an unexpected third-party value becomes
  `ORPHANED_REMOTE`. Compensation is a new authorized effect and never edits
  prior history. Zero tests, all skipped, flaky, timeout, infrastructure error,
  invalid subject, or unknown can never yield PASS.

Exit: `LocalBareForge` closes the full chain with distinct identities and proves
no author-as-independent Evidence, merge bypass, duplicate logical effect, or
ambiguity laundering.

### Wave 6 — finish production APIs, Portal, and operations

Objective: expose only durable, command-correlated product state through the
operator and workload boundaries.

- Serve operator traffic only at `/api/v1`; workload transitions use
  peer-authenticated UDS at `/internal/v1`. Legacy `/v1` returns typed
  `API_VERSION_RETIRED` and performs no mutation.
- Refuse off-loopback startup without TLS, OIDC Authorization Code + PKCE,
  origin allowlist, secure cookies, CSRF, RBAC, upstream phishing-resistant MFA,
  and two-person high-risk approval. Every mutation requires idempotency key and
  expected revision and returns `CommandReceiptV1`.
- Add signed internal dispatch, typed reconciliation, pagination, bounded
  queries, authenticated resumable SSE, bounded queues, atomic snapshot
  watermarks, and typed `RESYNC_REQUIRED`.
- Finish `/livez`, `/readyz`, restricted `/metrics`, OpenTelemetry correlation,
  capacity/backpressure, freeze countdown, incident, intervention, approval,
  restore, and audit-anchor workflows.
- Finish all 15 Portal surfaces. Under `self-hosted-v1`, every surface is durable
  or explicitly `OUT_OF_PROFILE`; under `evolution-v1`, all 15 are durable. The
  default Shift Brief names the exact unproved claim, subject, evidence class,
  freshness, blocker, and next authorized action.
- Embed exact manifest-verified Portal bytes under same-origin CSP/security
  headers; arbitrary release-time `VITE_BULLET_API` is refused. Meet WCAG 2.2 AA
  through keyboard, focus, live-announcement, responsive/auth, axe, and manual
  review gates.

Negative checks: cross-role access, CSRF, replay, stale revision, duplicate
command, queue overflow, cursor gap, snapshot/event race, missing ledger subject,
optimistic empty state, and unavailable read-back remain visible and never green.

Exit: packaged-origin browser and API/stream race suites pass; every selected
surface is durable or typed out of the selected profile.

### Wave 7 — prove the offline transaction and Ubuntu package

Objective: produce one fault-complete signed `TRANSACTION_PROOF` and one
operational Ubuntu 24.04 x86_64 family from the same schema-3 subjects.

- Add `just proof-transaction-offline` using simulator providers and
  `LocalBareForge` with distinct workload identities. Bind the exact family,
  policy, schema, toolchain, `GateSpec`, Mission, Attempt, Candidate, Evidence,
  ProofBundle, Effect, Check, Integration, Observation, fault evidence, and
  audit anchor into one signed `GateReceiptV1`.
- Inject death, timeout, response loss, stale authority, freeze, clock movement,
  ENOSPC, and restart at all twelve boundaries: (1) grant persistence,
  (2) Runner startup, (3) workspace open, (4) provider completion,
  (5) patch apply, (6) checkpoint, (7) Candidate preparation,
  (8) verifier handoff, (9) candidate delivery, (10) check publication,
  (11) integration, and (12) observation/cleanup.
- Run a different-identity, exact-OID, network-bounded builder. Build the Hub,
  embedded farmd/Portal, Kernel/Runner/Verifier/Effects, BulletGit, OCI assets,
  Firecracker guest, and exact Jeryu runtime. Produce checksums, CycloneDX and
  SPDX SBOMs, SLSA provenance, Ed25519 receipt signatures, Cosign/Sigstore
  bundles, and reproducibility evidence.
- Install immutable bytes under `/usr/lib/bullet`, config under `/etc/bullet`,
  state under `/var/lib/bullet`, independently stored authority high-water, and
  sockets under `/run/bullet`. Keep Jeryu at `/usr/lib/bullet/jeryu/<version>`
  with config/state/runtime under the locked `/etc`, `/var/lib`, and `/run`
  roots; retain old bytes for atomic rollback and refuse handshake drift.
- Exercise two clean installs, activate, upgrade, backup, restore, rollback,
  uninstall with non-destructive data retention, and disaster recovery. Make
  dependency/advisory freshness, vulnerability, license, source, secret,
  workflow, CodeQL, fuzz, sanitizer, accessibility, chaos, package, and restore
  failures blocking; require Jankurai ≥90, zero caps, and zero hard findings.
- Build the paper and executive brief twice from clean signed subjects and the
  same generated evidence registry.

Exit: the twelve-boundary offline receipt passes and every offline
`self-hosted-v1` gate is green. No provider or live forge authority is implied.

### Wave 8 — run explicit live checkpoints and release `self-hosted-v1`

Objective: first GA on Ubuntu with one approved Claude identity and protected
local Jeryu transaction.

1. **Checkpoint A — Claude.** Stop for signed operator approval of policy
   v1alpha2 generation ≥2, the
   provider-runner key, exact Claude service profile, budget, expiry, and
   rollback.
2. Run one bounded read-only Claude conformance turn; prove native protocol,
   exact binary/profile, credential isolation, egress, settlement, teardown,
   and signed receipt.
3. **Checkpoint B — Jeryu.** Only after Checkpoint A passes, stop again for
   distinct Jeryu broker, attestor, integrator, and observer credentials plus
   one exact protected test repository.
4. Run one low-risk Claude + Jeryu transaction through Candidate, proof,
   delivery, exact-SHA check, protected integration, read-back, and observation
   without sharing credentials or reusing an earlier transaction's Candidate.
5. Preserve any failure as failure; the profile remains HOLD/REJECT rather than
   synthesizing a pass or retrying an ambiguous effect.

Exit: `check release --profile self-hosted-v1 --receipts <registry> --json`
returns PASS only from current admitted receipts. External predecessors are
OD-A, OD-B, OD-D, OD-E, the exact Claude service identity, and protected Jeryu
custody.

### Wave 9 — certify `evolution-v1`

Objective: allow optimization only after the self-hosted product is stable.

- Implement a generic signed `TeamRecipeV1` compiler; the engine hardcodes no
  persona. Freeze and certify T0 single-agent fallback, T1 planner→implementer,
  T2 implementer→critic→bounded repair, T3 planner→parallel specialists→
  provenance-preserving synthesizer, T4 exact-base code race→blinded selector→
  new Candidate, and T5 planner→parallel implementer/reviewer→synthesizer→
  independent verifier.
- Give every role a distinct sandbox, profile, reservation, context, typed
  artifact contract, credential set, and conflict check. Never inherit a
  session, workspace, Candidate, Evidence, ProofBundle, or authority.
- Implement typed formulas/orders over immutable work graphs, durable mail and
  handoff, desired-state reconciliation, health patrol, interactive attach/peek,
  and event triggers. Terminals and prompts remain observations, never authority.
- Persist task classification, routing exclusions, context lineage, quota,
  struggle, dissent, fusion, selection, negative knowledge, promotion,
  activation, drift, review, and adjudication. Complete BulletGit's semantic
  moat: intent/conflict forecasting, evidence invalidation, patch algebra,
  causal blame, intent-aware compensation, and proof/context fetch.
- Dogfood only for discovery; use a frozen external corpus for confirmation.
  The first preregistered study is T0 versus fixed T3 on naturally parallel
  multi-file maintenance tasks with matched provider/profile/tools/environment
  and total compute. Use deterministic allocation, A/A calibration, clustered
  repository/lineage/time splits, blinded independent review, contamination
  tracking, intention-to-treat, multiplicity control, missingness rules, and
  all-in cost accounting.
- Implement deterministic offline MOME with descriptors parallelism,
  change-surface, and evidence burden; a 3×3×3 grid; capacity eight per cell;
  hard-violation filtering; conservative confidence bounds; deterministic
  hypervolume eviction; and ASHA reduction factor three. Disable ASHA when rung
  calibration fails.
- Freeze finalists before external T0-versus-T3 confirmation. Promotion order is
  offline confirmation→no-effect shadow→rollback-readiness proof→OD-H for one
  exact expiring R0/R1 canary≤1% with concurrent T0 reserve→canary observation
  and rollback receipt→independent promotion/drift receipts→conservative routing.
  Only observation-surviving outcomes update
  routing; fingerprint or drift changes decertify before another route.
- Separate recommender, certifier, promoter, and activation identities; rollback
  atomically to the last healthy routing generation on regression or drift.
  R2+ remains cryptographically impossible without exact signed human approval.
- Run a matched comparison against pinned Gas Town and Gas City releases. The
  primary endpoint is accepted-and-observation-surviving outcome under equal
  models, tools, budgets, and compute, counting every timeout, fallback,
  failure, abstention, intervention, and cost.

Exit: `evolution-v1` passes from study, shadow, bounded-canary, promotion, drift,
and rollback receipts. OD-H alone never satisfies this exit or authorizes R2+.
Rollback meets its SLO; comparative receipts support any competitive claim. It
depends on `self-hosted-v1` and is never implied by `universal-v1`.

### Wave 10 — certify provider, forge, and platform breadth

Objective: create non-substitutable profile receipts and compose them into the
later `universal-v1` release.

- Live-certify Codex, Cursor, and Antigravity independently; no receipt is
  reusable across provider/version/profile/service identity.
- Complete GitHub App and GitLab adapters with separate broker, attestor,
  integrator, and observer identities. Certify GitLab.com first; every
  self-managed version/endpoint has its own profile.
- Test stale expected OID, wrong target, force-moved head, lost response before
  and after mutation, auth expiry/rotation, pagination, branch-protection drift,
  check mismatch, third-party values, and restart at every phase.
- Build Linux x86_64/aarch64, macOS x86_64/arm64, and Windows x64 archives.
  Non-Linux control clients keep mutation disabled until exact containment
  passes. Certify Linux-arm64 OCI/Firecracker parity; later macOS/Windows
  mutation uses the certified Linux S2 guest behind Virtualization.framework or
  Hyper-V rather than weaker host-only isolation.
- Require current PASS receipts for `self-hosted-v1`, all four providers,
  Jeryu, GitHub, GitLab.com, self-managed GitLab, and all five platforms from
  one compatible signed family. Recheck shared schema/policy/tool/environment,
  trusted-time, expiry/revocation, dependency closure, and contradictions; rerun
  five-package smoke and the refusal/containment matrix.

Exit: every slice passes only from its own semantic receipt and
`universal-v1` passes their exact composition. Evolution stays separate.

### Wave 11 — distributed teams, then sagas

Objective: add distributed authority without weakening single-host semantics.

- `team-v1`: PostgreSQL, remote runners, mTLS/SPIFFE identities, replicated
  stateless projections, S3-compatible object storage, runner epochs, remote
  verifier pools, durable leader/fence semantics, partitions, failover, freeze,
  and restore. Prove clock movement and duplicate delivery cannot mint double
  authority.
- `saga-v1`: cross-repository staged Candidates, dependency-aware quarantine,
  ordered compatibility constraints, observations, explicit
  `PARTIALLY_INTEGRATED`, `COMPENSATING`, `FORWARD_REPAIR_REQUIRED`, and
  `SURVIVED` states, compensation, and forward repair. Never claim false
  atomicity.
- Preserve one writer per repository generation and exact evidence/effect
  subjects across every partition and recovery boundary; extend proof-aware
  query/fetch without leaking holdouts or protected context.

Exit: `team-v1` passes first; `saga-v1` then passes from a distinct receipt set.
Neither can retroactively satisfy `self-hosted-v1` or `universal-v1`.

## Gap-to-wave ownership

| Gap | Closure waves | First counting receipt |
| --- | --- | --- |
| G1 install | 0, 1, 7 | `RELEASE_PROOF` for schema-3 signed two-run lifecycle |
| G2 transaction | 2–7 | `TRANSACTION_PROOF` |
| G3 Kernel path | 2, 3 | durable authority plus isolation receipts |
| G4 BulletGit path | 1, 4 | online-authority Candidate transaction |
| G5 providers | 1, 8, 10 | provider-specific `LIVE_PROOF` after policy/enrollment-anchor admission |
| G6 Jeryu | 4, 5, 8 | Jeryu effect `LIVE_PROOF` |
| G7 GitHub adapter | 5, 10 | independent GitHub App profile receipt |
| G8 security | 3, 7 | exact-subject scanner/Jankurai release receipts |
| G9 packages | 7, 10 | target-specific then composition `RELEASE_PROOF` |
| G10 containment | 3, 10 | platform-specific containment receipt |
| G11 evolution | 9 | `release.profile.evolution-v1` receipt |
| G12 release decision | 1, 7, 8 | the explicitly requested profile condition |
| G13 Portal surfaces | 6, 9 | durable-or-`OUT_OF_PROFILE` self-hosted receipt; all-15 evolution receipt |
| G14 farmd API | 2, 5, 6 | signed-dispatch/API reconciliation receipt |
| G15 cognitive state | 9 | all-durable cognitive/evolution replay receipt; Wave 6 may mark its self-hosted Portal surface `OUT_OF_PROFILE` but does not close cognitive persistence |
| G16 GitLab adapters | 4, 5, 10 | independent GitLab.com and self-managed GitLab profile receipts |
| G17 distributed team mode | 11 | `release.profile.team-v1` receipt |
| G18 cross-repository sagas | 11 | `release.profile.saga-v1` receipt after team closure |

## Owner handoff rule

For every wave, the implementing owner must leave: exact repository commit and
tree IDs; clean status; claim and handoff IDs; commands and pinned tool versions;
zero-test/skip counts; sanitized artifact hashes; negative-test outcomes; the
remaining BLOCKED conditions; and the next executable command. A separate
reviewer must compare the changed-path set and receipt subject before the
orchestrator commits. No handoff, CI observation, demo, or prose assertion is a
substitute for the receipt class named by the exit.
