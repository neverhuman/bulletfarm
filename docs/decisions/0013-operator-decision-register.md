# ADR 0013: Operator decision register

Status: Accepted (register)
Owner: Bullet Farm maintainers
Last reviewed: 2026-08-25
Applies to: every gate whose closer is an operator act rather than code

## Decision

There is exactly one list of open operator decisions, and it is this file. It supersedes the lists formerly
kept in the family-root `README.md` ("Open operator decisions"), in
[`../assurance/product-gaps.md`](../assurance/product-gaps.md) ("Operator decisions that are not code"), and
in `TEAM_PLAN_CLAUDE.md` §10.5 (OD-1). Those locations point here or will be redirected by the orchestrator;
if any of them disagrees with this register, this register wins and the other is stale.

An operator decision is a fact only a person with custody can create — a ratified policy generation, an
authenticated forge, a protected key, a repository the family may mutate. Agents must not manufacture,
simulate, or flip any of them. Each entry names the gate ids it unblocks
([`../assurance/release-truth.generated.md`](../assurance/release-truth.generated.md); the current generated
`universal-v1` diagnostic selects 43 `BLOCKED` gates and binds all 46 global crosswalk rows), the procedure
that consumes it, the exact `AGENT_CHAT.md` line that ratifies it, and its
status. A decision is `OPEN` until its ratification line exists in the family-root `AGENT_CHAT.md`, but that
line is only an auditable coordination witness. It is never cryptographic or runtime authority: the consuming
command must independently admit and read back the separately protected policy, key, repository, deployment
passport, budget, validity, and schema-3 trust anchor named by the decision. An agent-created key or policy plus
a forged `— operator —` line cannot qualify a LIVE_PROOF or profile receipt
([`../runbooks/live-conformance.md`](../runbooks/live-conformance.md) §2.3).

Ratification lines use the family log's heading form, with `operator` as the actor:

```text
## <UTC ISO 8601 Z> — operator — <DECISION-TAG> — RATIFIED: <exact facts>
```

## Register

### OD-A — Live provider admission and non-substitutable provider enrollments

- Decision: admit an operator-owned v1alpha2 policy at `policy_generation = 2`, its `provider-runner`
  launch-grant key, and one independent enrollment row for each provider that may run. The common policy keeps
  `route_policy.evolutionary_authority=false`. Each provider row binds the exact executable path and digest,
  native protocol/model/profile, service identity and brokered credential handle (never the secret), invocation
  and spend budgets, validity interval, revocation handle, and rollback owner. A row for one provider never
  certifies another.
- Trust boundary: the policy path must be OS/operator-owned with admitted owner/mode, exact digest, key
  fingerprint/custody, signer and approval identity, and the schema-3 trust anchor that admits it. The current
  loader validates v1alpha2 shape but does not yet prove that complete anchor; that is Wave-1/G5 product debt,
  not something an `AGENT_CHAT.md` line can waive.
- Unblocks: Wave 8 may use only the Claude enrollment for `release.provider.claude`. Wave 10 independently
  enrolls Codex, Cursor, and Antigravity for their three profile gates. OD-A is not a predecessor of the
  credential-free `release.transaction-demo`.
- Procedure: [`../runbooks/live-conformance.md`](../runbooks/live-conformance.md) §2–§3; rotation in
  [`../runbooks/signer-rotation.md`](../runbooks/signer-rotation.md) §1.
- Common-policy witness:
  `## <UTC> — operator — POLICY-GENERATION-2 — RATIFIED: policy <absolute path> owner/mode <facts> digest <digest>; schema-3 anchor <digest>; provider-runner key <fingerprint/custody>; signer/approver <ids>; valid <interval>; evolutionary authority false; revocation/rollback <handles>.`
- Per-provider witness, repeated separately for `claude`, `codex`, `cursor`, and `antigravity`:
  `## <UTC> — operator — PROVIDER-<NAME>-ENROLLMENT — RATIFIED: executable <path/digest>; protocol/model/profile <facts>; service identity <id>; credential handle <handle>; budget <limits>; valid <interval>; revocation <handle>; rollback <owner>.`
- Negative acceptance: a syntactically valid agent-created policy/key, a forged operator-looking line, an
  enrollment borrowed from another provider, or an unanchored policy can never qualify a LIVE_PROOF.
- Status: `OPEN` — no common-policy witness or provider enrollment has been machine-admitted.

### OD-B — Jeryu test authority: authentication and a scoped test repository

- Decision: after the offline Wave-4/5 forge contract is green, name one exact
  protected Jeryu test repository and the immutable signed Jeryu deployment
  passport, then provision four distinct workload identities for broker,
  attestor, integrator, and observer. Their credentials use brokered,
  short-lived projection into their separate service identities; no shared
  per-user token, `~/.jeryu` state, or GitHub CLI `hosts.yml` entry may satisfy
  this decision. The running forge must not be modified to work around a
  missing capability.
- **Do not run `gh auth login`, `gh auth refresh`, or any credential-store token hunting against a Jeryu
  host.** All three are on Jeryu's own do-not-run list, published by the running instance at
  `GET http://127.0.0.1:8787/.jeryu/capabilities` under `gh_auth_policy.do_not_run`, and repeated in
  `jeryu-deploy/docs/errors.md`: "GitHub.com auth and local Jeryu host auth are separate; do not run
  gh auth login for Jeryu hosts." A login flow against a Jeryu host returns a guided refusal, not a device
  flow, so the older instruction to use `gh auth login -h 127.0.0.1:8787` could never have succeeded.
  If an operator diagnostic uses `gh`, it remains outside Bullet workload
  authority and cannot supply any of the four credentials above.
- Why not code: credentials and repository custody. Every Jeryu integration path today refuses without
  positive online authority (BulletGit fail-closed gateway; effect broker read-back).
- Unblocks: `release.forge.jeryu` (LIVE_PROOF: protected integration plus UNKNOWN/read-back reconciliation
  receipt) and the separate `release.profile.jeryu-forge-v1` condition whose semantic receipt also binds
  capability, backup/restore, and drift. It is an input to the live half of
  [`../runbooks/effect-reconciliation.md`](../runbooks/effect-reconciliation.md).
- Procedure: active [`closure-roadmap.md`](../assurance/closure-roadmap.md)
  Wave 8, Checkpoint B. First verify the authenticated deployment passport and
  capability handshake, then stop for the distinct credential handles, exact
  repository, budget, expiry, and rollback approval. No credential-generation
  shell recipe is committed before the broker and four service identities can
  project and revoke those secrets safely. The live run must execute the exact
  Candidate → proof → delivery → check → protected integration →
  observation chain and preserve ambiguous outcomes.
- Ratification line:
  `## <UTC> — operator — JERYU-TEST-AUTHORITY — RATIFIED: deployment <version/capability digest>; protected repository <owner/slug>; distinct broker <id>, attestor <id>, integrator <id>, observer <id> credential handles; budget <subject>; expiry <time>; rollback <subject>; forge unmodified.`
- Status: `OPEN`.

### OD-C — GitHub App test authority

- Decision: create or designate a GitHub App, install it on one exact protected test repository, bind the
  deployment passport and capability digest, and provision distinct broker, attestor, integrator, and observer
  workload identities and credential handles. Bind repository protection, budget, expiry, revocation, rollback,
  and the sole mutation scope; no shared user token or two-role credential qualifies.
- Why not code: an external account, App registration, and branch-protection settings. GitHub is a required,
  separately configured effect adapter, never source authority (ADR 0002, ADR 0008).
- Unblocks: `release.forge.github-app` (LIVE_PROOF: exact-subject integration and reconciliation receipt).
- Procedure: none written; the adapter is specified, not certified
  ([`../assurance/product-gaps.md`](../assurance/product-gaps.md) G7).
- Ratification line:
  `## <UTC> — operator — GITHUB-APP-TEST-AUTHORITY — RATIFIED: App <id> passport/capability <digest>; repository <owner/repo>; protection <digest>; broker <id/handle>; attestor <id/handle>; integrator <id/handle>; observer <id/handle>; budget <limits>; expiry <time>; revocation <handle>; rollback <owner>; this repository only.`
- Status: `OPEN`.

### OD-D — Signed schema-3 lock inputs

- Decision: supply the authenticated read-only Jeryu source URL, deployment passport/capability digest and member slugs, the signer identity that will sign member
  tags and the hub tag, and the exact member subjects (tag, commit/tree, dependency locks, generated-artifact
  digests, checksums), so that `bullet-family lock generate --tag VERSION --subjects <absolute-path>` can produce a schema-3
  `family.lock` and `checkout verify` can return a clean verdict.
- Why not code: authenticated source metadata and a tag signer. The checked-in lock is schema 2 and is refused
  before mutation by design ([`../runbooks/schema-removal.md`](../runbooks/schema-removal.md)).
- Unblocks: `release.installable-lock`, then `release.installer-twice` and `release.manifest-non-circular`
  (which bind the schema-3 lock); indirectly every RELEASE_PROOF gate that starts "from tagged bytes".
- Procedure: [`../runbooks/source-setup.md`](../runbooks/source-setup.md) "Future trusted installation";
  [`../runbooks/setup-recovery.md`](../runbooks/setup-recovery.md) for the two-run proof afterwards.
- Ratification line:
  `## <UTC> — operator — SCHEMA-3-LOCK-INPUTS — RATIFIED: read-only Jeryu source <url> passport/capability <digest>; slugs <hub, kernel, git, portal>; tag signer <principal/fingerprint/custody>; member tags and immutable subjects <facts>; lock generate may read these subjects; no mutation authority.`
- Status: `OPEN`. This act authenticates only the read-only source/tag/passport subjects used by schema-3
  installation. It grants no protected-repository mutation authority and does not depend on OD-B. OD-B remains
  a later Wave-8 prerequisite only for Jeryu LIVE_PROOF.

### OD-E — Release, build-attestor, and trusted-time custody

- Decision: provision distinct release-signing, build-attestor, and trusted-time keys under protected custody.
  OD-D separately owns the source-tag signer. Publish an allowed-signers policy that maps each non-substitutable
  fingerprint to its exact receipt roles, namespaces, validity, revocation, rotation, and custody, and bind the
  policy digest into the schema-3 trust root.
- Why not code: key custody and the external signer policy (ADR 0005 distinct purposes; ADR 0010 explicit
  policy input). The verifiers exist; the key does not.
- Unblocks: `release.signatures`, `release.receipt-contracts`, `release.provenance` (RELEASE_PROOF).
- Procedure: [`../runbooks/signer-rotation.md`](../runbooks/signer-rotation.md) §2 records what exists and the
  invariants a future procedure must satisfy; no signing procedure is written before the key exists.
- Ratification line:
  `## <UTC> — operator — RELEASE-TRUST-CUSTODY — RATIFIED: release signer <fingerprint/custody>; build attestor <distinct fingerprint/custody>; trusted time <distinct fingerprint/custody>; source-tag signer <OD-D fingerprint>; role/namespace policy <path/digest>; validity/revocation/rotation <facts>; schema-3 anchor <digest>; allowed_signers commit <oid>.`
- Status: `OPEN`.

### OD-F — Optional publication of the `linux-preview` diagnostic

Status: OPEN. First-GA `self-hosted-v1` already selects the signed Ubuntu 24.04
x86_64/systemd distribution, so OD-F does not choose or amend GA scope. It is
needed only if operators want to publish the separate `linux-preview`
diagnostic without a release tag or GA wording. Such publication clears no
gate and cannot substitute for `self-hosted-v1`. Ratification line:
`## <UTC> — operator — OD-F — RATIFIED: publish linux-preview as a non-release diagnostic at <location>; no tag or GA wording.`

### OD-G — Public names, endpoints, and deployment identity

Status: OPEN (added 2026-08-25). Public forge/mirror names, exact endpoints, deployment passport, DNS/TLS identity,
hosted-runner identities, review expiry, and rollback owner are unratified (`docs/workplan.md` WP-08/WP-14/WP-17).
This act owns public topology only; it grants no source, provider, effect, signing, or release authority. Unblocks
public mirror topology, immutable paper/source permalinks, and hosted family CI provisioning (R-49). Ratification
line: `## <UTC> — operator — OD-G — RATIFIED: public names/endpoints <facts>; deployment passport <digest>; DNS/TLS <identity>; hosted runners <ids>; reviewed until <time>; rollback <owner/plan>.`

### OD-H — Evolutionary authority activation

Status: OPEN. After `self-hosted-v1` passes, the exact frozen offline study and
no-effect shadow complete, and automatic rollback readiness is independently
proved, an operator may ratify a successor policy generation that authorizes
one exact expiring R0/R1 canary at no more than 1% traffic. OD-H does not
authorize R2+, conservative production routing, or `evolution-v1` PASS. The
canary must then survive its observation window (or roll back), after which
distinct certifier, promoter, and activator identities produce promotion and
drift receipts. It unblocks only that bounded canary and never changes
`self-hosted-v1` or `universal-v1`. Ratification line:
`## <UTC> — operator — OD-H — RATIFIED: frozen study/shadow <digests>; rollback-readiness <digest/owner/subject>; successor policy <path/digest/generation>; exact recipe/corpus <ids>; canary class R0/R1; traffic <=1%; budget/wall-clock <limits>; signer/certifier/promoter/activator <distinct ids>; validity/expiry <facts>; automatic rollback <handle>; R2+ forbidden.`

### OD-I — GitLab.com test authority

Status: OPEN. Designate one exact protected GitLab.com project and capability/passport digest, then provision
distinct broker, attestor, integrator, and observer workload identities with separately scoped credential handles.
Bind protection, exact mutation scope, budget, expiry, revocation, and rollback. This act unblocks only
`release.profile.gitlab-adapter-v1`/G16 and cannot certify a self-managed endpoint. Ratification line:
`## <UTC> — operator — OD-I — RATIFIED: GitLab.com project <namespace/project>; capability/protection <digests>; broker/attestor/integrator/observer <distinct ids/handles>; budget/expiry/revocation <facts>; rollback <owner>.`

### OD-J — Self-managed GitLab test authority

Status: OPEN. Designate one exact self-managed GitLab endpoint, version, deployment/capability digest, and protected
project, with four distinct workload identities/credential handles, protection, mutation scope, budget, expiry,
revocation, and rollback. This act unblocks only `release.profile.gitlab-self-managed-v1`/G16; neither its receipt
nor OD-I's receipt certifies the other. Ratification line:
`## <UTC> — operator — OD-J — RATIFIED: endpoint/version <facts>; deployment/capability <digests>; project/protection <facts>; broker/attestor/integrator/observer <distinct ids/handles>; budget/expiry/revocation <facts>; rollback <owner>.`

## Dependency DAG

- OD-D is a read-only source/tag/passport act with no dependency on OD-B. Schema-3 generation and the two-install
  lifecycle can close before any live provider or forge mutation credential exists.
- OD-E may bind OD-D's distinct source-tag signer; OD-A then consumes the admitted schema-3 trust anchor from
  OD-D/OD-E. None of these acts grants live forge mutation.
- OD-B, OD-C, OD-I, and OD-J each wait on their own offline adapter contract and are mutually independent.
- OD-H waits on passing self-hosted, frozen offline study, no-effect shadow,
  and rollback-readiness evidence. Canary, promotion, drift, and evolution
  profile receipts follow OD-H. OD-F and OD-G clear no release gate.

The graph is acyclic: `OD-D → OD-E → OD-A`; the live forge acts branch only after
their offline contracts; and `self-hosted PASS → offline study → no-effect
shadow → rollback readiness → OD-H → bounded canary → promotion/drift receipts
→ evolution PASS`. In particular there is no `OD-D → OD-B` edge.

## Reverse crosswalk

| Decision | Exact selected gate/profile condition | Gap | Earliest wave | Consuming receipt/procedure | Non-substitution |
| --- | --- | --- | ---: | --- | --- |
| OD-A | `release.provider.claude`; later Codex/Cursor/Antigravity conditions | G5 | 8 / 10 | provider-specific LIVE_PROOF; live-conformance runbook | each provider enrollment counts only for itself |
| OD-B | `release.forge.jeryu` + `release.profile.jeryu-forge-v1` | G6 | 8 | Jeryu LIVE_PROOF + semantic profile receipt | not source admission; no other forge |
| OD-C | `release.forge.github-app` + GitHub profile condition | G7 | 10 | GitHub App LIVE_PROOF | not Jeryu or GitLab |
| OD-D | installable lock, two-install lifecycle, manifest lock subject | G1, G9 | 1 / 7 | schema-3 lock + RELEASE_PROOF | read-only source authority only |
| OD-E | signatures, provenance, `release.receipt-contracts` trust roots | G9, G12 | 1 / 7 | kind-specific signed RELEASE_PROOF admission | source/build/time/release roles remain distinct |
| OD-F | none (optional diagnostic publication) | N/A | N/A | no receipt | clears no gate |
| OD-G | none (public topology/hosted CI activation) | N/A | N/A | API read-back/hosting observation | grants no product authority |
| OD-H | bounded R0/R1 canary predecessor of `release.profile.evolution-v1` | G11 | 9 | study/shadow/rollback-readiness before; canary/promotion/drift receipts after | never R2+, self-hosted, or universal; never closes evolution alone |
| OD-I | `release.profile.gitlab-adapter-v1` | G16 | 10 | GitLab.com LIVE_PROOF | never self-managed GitLab |
| OD-J | `release.profile.gitlab-self-managed-v1` | G16 | 10 | self-managed GitLab LIVE_PROOF | never GitLab.com |

## Consequence

- A gate named above moves only when machine admission and read-back register its exact receipt, never when a
  decision is merely witnessed in prose.
- Adding a decision means adding an entry, dependency edge, reverse-crosswalk row, exact gate id, and ratification
  format here in the same reviewed change as its consuming procedure. Ratified entries are retained with their
  timestamp and log reference; they are not deleted.
- No agent creates an operator fact. An operator-authored line is an audit witness only, and a separate machine
  verifier must reject forged lines, self-selected trust roots, role substitution, expiry, revocation, or drift.
