# Bullet Farm opportunity workplan

Status: **non-authoritative backlog; not release authority**
Owner: Bullet Farm maintainers
Last reviewed: 2026-08-26

This workplan records opportunities exposed by the architecture paper and its
evidence audit. It does not compete with the authoritative
[`assurance/closure-roadmap.md`](assurance/closure-roadmap.md), generated
contracts, the Kernel ledger, or explicit `bullet-family check release --profile
<profile> --receipts <admitted-absolute-registry> --json`. If this
file disagrees with any of them, this file loses. `COMPLETE` here would mean
only that the named acceptance receipt exists; it cannot promote a family gate.
The owner-oriented [finish execution plan](assurance/execution-plan.md) serializes
these rows against Waves 0–11, G1–G18, and OD-A–OD-J without creating a new
source of runtime or release truth. The
[full-product dogfood bridge](assurance/full-product-dogfood-plan.md) and
[coordinator recovery runbook](runbooks/coordinator-recovery.md) route the
current component-only recovery path without promoting this backlog.

## Frozen V1 topology and operator-blocked proposals

- Initial source distribution is Jeryu-only from immutable signed tags. No
  authenticated Jeryu URL is admitted by the manifest. The operator named
  `https://github.com/neverhuman/bulletfarm` as the public discovery index
  (not `neverhuman/bullet-farm`); GitHub is not source authority, and OD-G
  remainder (DNS/TLS, hosted runners, rollback) stays open.
- The only permitted local Jeryu family is
  `/home/ubuntu/jain-split/jeryu-split`. Never recreate
  `/home/ubuntu/jeryu-split`. The branded hub owns onboarding, its manifest,
  and pinned component mapping; component repositories remain independent.
- Users are encouraged to run localhost Jeryu so CAS, read-back, protected
  refs, offline CI, and local context services can be certified together.
  GitHub and GitLab stay separately certified effect adapters. WP-19 consumes
  a signed immutable Jeryu runtime at `/usr/lib/bullet/jeryu/<version>`, with
  configuration under `/etc/bullet/jeryu`, state under `/var/lib/bullet/jeryu`,
  and runtime state under `/run/bullet/jeryu`; it never creates a Jeryu source
  checkout or family member under this tree.
- `git.neverhuman.org`, `github.com/neverhuman/jeryu`, and
  `github.com/neverhuman/jankurai` are unratified future topology proposals,
  not current authority, authenticated endpoints, or public front doors. An
  operator decision must precede any use of those names in a lock, credential,
  publication, or receipt.
- If an operator later ratifies a public mirror, it remains a separately
  configured effect adapter rather than source authority. Publication must use
  expected-old-OID CAS, post-write read-back, divergence quarantine, and
  backups proved independently of the mirror.
- Jankurai remains the monorepo at `/home/ubuntu/jankurai`. No public front door
  is assumed; any future external-history reconciliation requires a
  preservation-first RFC and explicit ancestry receipts, never a force-push.

## Backlog

Priority is `P0` (release/safety predecessor), `P1` (important follow-on), or
`P2` (research/product opportunity). Class is `V1` or `post-V1`. Status values
are descriptive only: `BACKLOG`, `IN PROGRESS`, `LOCAL-then-EXTERNAL`, `EXTERNAL-BLOCKED`, or
`COMPONENT-PROVED`, `COMPLETE WITH RECEIPT`; `COMPONENT-PROVED` records a
bounded unsigned component observation, not an acceptance receipt or gate
authority. `RETIRED` preserves a superseded proposal without authorizing its
implementation.

| ID | Priority | Class | Dependencies | Owning subsystem | Current evidence | Acceptance receipt | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| WP-01 | P1 | V1 | Stable family subjects; paper toolchain | Hub assurance | Hand-curated dated snapshot and deterministic TeX build | CI regenerates `bullet.paper-evidence.v1`, competitor pins, macros, and byte-identical PDFs from clean signed subjects | IN PROGRESS |
| WP-02 | P0 | V1 | V1-S1 through V1-S4 | Kernel, Runner, Verifier, BulletGit, Effects | Component receipts only; no connected proof | Signed `TRANSACTION_PROOF` for one exact offline five-plane transaction, followed by a matched receipt-bearing benchmark corpus | BACKLOG |
| WP-03 | P0 | V1 | Signed shared wire tag; Jeryu authority | Hub, Kernel, BulletGit | Local wire tests; no immutable publication | Signed immutable `bullet-wire` tag is independently resolved and consumed without committed sibling paths | EXTERNAL-BLOCKED |
| WP-04 | P0 | V1 | Setup admission and hostile-filesystem fixtures | Hub installer, BulletGit | Sealed tool subjects and partial descriptor-relative operations | Receipts admit the Git binary by digest, parse status as NUL-delimited bytes, mutate descriptor-relatively/no-follow, bind stronger lock identity, and prove support or typed refusal on every target OS | BACKLOG |
| WP-05 | P0 | V1 | Candidate/Evidence wire publication | BulletGit, Verifier, Effects | Strict local manifests; one fixture gate | Candidate and integration manifests prove invalidation across rebase, merge, and merge-queue result OIDs | BACKLOG |
| WP-06 | P0 | V1 | Authenticated Jeryu service and broker credentials | Effects, BulletGit, Jeryu | Local bare-forge component only; typed live capability/integration admission remains | Engineering lands typed probes, protected integration/read-back reconciliation, and semantic receipt admission; then the protected integration receipt binds operator-authenticated push, expected-old OID, required checks, server read-back, and observed protected ref | LOCAL-then-EXTERNAL |
| WP-07 | P1 | V1 | Jeryu governance review | Jeryu hub, release operations | Documentation assigns overlapping release roles to `jeryu-release-ops` and `jeryu-deploy` | Accepted authority RFC plus generated owner/credential crosswalk with one release principal per effect | BACKLOG |
| WP-08 | P0 | V1 | Operator ratification of a Jeryu endpoint, DNS, TLS, auth, protection, backup, deployment | Jeryu operations | No endpoint selected; no authenticated URL or live receipts | Dated operator decision plus TLS chain, authenticated protection policy, deployment identity, backup digest, destructive restore drill, and independent read-back receipts for the ratified endpoint | EXTERNAL-BLOCKED |
| WP-09 | P1 | post-V1 | WP-06, WP-08, and a separately ratified public mirror | Jeryu mirror broker | No public mirror selected; no component-mirror proof | If ratified, each Jeryu component tag mirrors one way under expected-old-OID CAS; divergence is quarantined without overwrite | BACKLOG |
| WP-10 | P1 | post-V1 | Preservation RFC and complete ref inventories | Jankurai governance | No public mirror selected; preservation/reconciliation RFC absent | Signed RFC, full bundle backups, ancestry map, rehearsal receipts, and an approved non-destructive reconciliation before any mirror decision | BACKLOG |
| WP-11 | P0 | V1 | Reproducible Jankurai build and hosted runner | Hub assurance, Jankurai | Machine-local audit only; release score blocked | Portable signed Jankurai artifact digest and pinned hosted audit receipt from exact source and policy | BACKLOG |
| WP-12 | P0 | V1 | Provider adapters, live policy, containment | Runner, Providers, Effects | Offline parsers and bounded component paths; complete schema-3 policy/enrollment-anchor admission and live receipt registration remain local; live admission false | Engineering lands hostile-tested policy/enrollment admission and semantic receipt registration; operators then supply four provider enrollments, exact executables/profiles/credentials, and protocol-conformance receipts plus signed launch, egress, teardown, and no-canary-leak evidence | LOCAL-then-EXTERNAL |
| WP-13 | P0 | V1 | Five target builders and signing custody | Release engineering | Verifier/extractor components; no produced package | Deterministic archives, checksums, SBOM, provenance, signatures, and two-run schema-3 installation receipts from the same family lock | BACKLOG |
| WP-14 | P1 | V1 | Operator-ratified immutable publication endpoint and paper preflight | Hub documentation, Jeryu | Local PDF/source only; no public endpoint selected | Immutable source and PDF permalinks whose hashes match the checked evidence manifest before any external announcement | EXTERNAL-BLOCKED |
| WP-15 | P2 | post-V1 | WP-02 matched corpus | Research/evaluation | Capability comparison only | Preregistered workloads, costs, failures, exact subjects, and comparable receipts; superiority claims are admitted only from this corpus | BACKLOG |
| WP-16 | P0 | V1 | Existing G3/G4; signed shared wire tag | Kernel, BulletGit | Signed lease-transport and fail-closed production clone are component-only; no durable Mutation reservation | One Kernel reservation write that repeats the active-lease check, issues a one-use permit, and is verified by production BulletGit before I/O | BACKLOG |
| WP-17 | P1 | post-V1 | WP-08, WP-09, operator topology RFC | Jeryu operations, Hub | Public discovery index named at `github.com/neverhuman/bulletfarm` (not `neverhuman/bullet-farm`); `git.neverhuman.org` and hosted CI remain unratified; GitHub is not source authority | Dated decision: GitHub as public index and/or backup; independent Jeryu Git as hosted front door for selected families; neither becomes source authority without CAS, read-back, and backup receipts | BACKLOG |
| WP-18 | P1 | post-V1 | A future operator relocation ADR | Jeryu governance | Jeryu already has the sole permitted independent source family at `/home/ubuntu/jain-split/jeryu-split`; WP-19 consumes a signed runtime artifact without copying source | No implementation under this item. Preserve the existing family; any future relocation must use a new ratified ADR, immutable source identity, signed migration/rollback receipts, and no alternate checkout | RETIRED |
| WP-19 | P0 | V1 | WP-03, WP-08, OD-D; must not wait on WP-18 | Hub installer, family container | Dedicated forge-topology audit; family-root `NEXT_EVOLUTION_PLAN.md` remains non-authoritative until corrected | Keep source only in `/home/ubuntu/jain-split/jeryu-split`. Bind a signed Jeryu runtime artifact in a non-circular external-component lock; support read-only `connect-existing` and isolated `managed` modes with immutable bytes at `/usr/lib/bullet/jeryu/<version>`, config at `/etc/bullet/jeryu`, state at `/var/lib/bullet/jeryu`, runtime at `/run/bullet/jeryu`, and retained prior-version bytes for rollback. Never copy, symlink, vendor, restart, upgrade, or reconfigure the existing source family or shared service. Refuse startup or activation when the exact version/capability handshake drifts from the lock. Default effect profile is localhost Jeryu; GitHub/GitLab remain separately certified adapters. Freeze `/git/{owner}/{repo}.git`, REST, capability, upgrade/rollback, and receipt contracts before implementation. This row is not a G6 live receipt | BACKLOG |
| WP-20 | P1 | V1 | Clean immutable component subjects | Hub documentation and CI | The [frozen snapshot](readme-media/snapshot.json), [component manifest](readme-media/component-preview/manifest.json), and [provider-safety manifest](readme-media/provider-safety/manifest.json) bind two credential-free `bullet.readme-demo.v1` sets; `just readme-record`, `just readme-render`, and `just readme-check` pass | Two manifests bind exact clean subjects, normalized transcripts, observations, tapes, static fallbacks, GIFs, frame hashes, redaction checks, and deterministic double rendering; this remains unsigned component observation | COMPONENT-PROVED |
| WP-21 | P1 | V1 | WP-12 | Runner, provider adapters, Hub onboarding | Offline protocol contracts only; provider-specific onboarding/task UX and receipt binding remain local; live admission disabled | Engineering lands provider-specific onboarding, exact runtime probing, complex-task UX, and truthful retry/recovery guidance; operator runs then bind signed launch grants, sealed live receipts, and teardown/canary proof for each provider | LOCAL-then-EXTERNAL |
| WP-22 | P0 | V1 | WP-03, WP-13, OD-G, immutable hosted runner provisioning | CI and release operations | Five atomic local lanes, source-first mirror jobs, hostile aggregation/observation tests, scheduled diagnostics, and an eight-node fail-closed `ci.toml` skeleton are implemented locally; Jeryu predecessor-status/artifact convergence semantics, a ratified public mirror, hosted runs, ruleset read-back, and runner authority remain absent | Engineering completes Jeryu predecessor/artifact convergence against the same local lanes; operators then provision ratified Jeryu and mirror runners so immutable source OIDs, fork safety, failed/skipped/cancelled/missing predecessor rejection, two consecutive main runs, merge-group context, ruleset read-back, and cache policy are recorded without treating mirror CI as release Evidence | LOCAL-then-EXTERNAL |
| WP-23 | P1 | V1 | WP-02, WP-12, WP-13 | Hub onboarding, release engineering, providers | Component-only recordings; provider onboarding/task UX remains local and public install/live task evidence remains external | After the local onboarding and transaction prerequisites land, publish install media only from two clean signed schema-3 installations and provider task media only from exact runtime probes, onboarding commands, sealed live receipts, and a connected `TRANSACTION_PROOF` | LOCAL-then-EXTERNAL |

## Maintenance

Every update must retain all eight row fields, link the exact receipt when one
exists, and state the boundary it does not prove. V1 items must also update the
authoritative closure plan in the same reviewed transaction; changing this
backlog alone never changes product truth.
