# Architecture entrypoint

Status: **normative topology; implementation and release remain incomplete**  
Owner: Bullet Farm maintainers  
Last reviewed: 2026-08-25

Bullet Farm is a local-first transaction processor for multi-agent engineering,
not a shared-checkout agent launcher. A Mission is decomposed into immutable
graph revisions and role-specific Variants. Each writable Attempt receives a
private workspace and never-reused fence. Model sessions can propose work, but
only Kernel authority, BulletGit mutation, independent verification, and a
reconciled effect can advance repository truth.

## System shape

| Component | Authoritative responsibility | Explicitly not authoritative for |
| --- | --- | --- |
| `bullet-kernel` | Missions, graph/command ledger, leases/fences, policy, routing, outbox, projections | Git credentials, writer-produced proof, provider claims |
| `bullet-git` | Private clones, atomic proposals, journals/CAS, checkpoints, Candidates, preservation | Scheduling, provider credentials, protected integration |
| Runner | Supervision, heartbeat, read-only provider sessions, admitted gate execution | Git/effect credentials or self-issued authority |
| Verifier | Clean reconstruction and exact-Candidate Evidence | Writer state, integration, scheduling |
| Effect broker | Intent reservation, Jeryu/GitHub dispatch, read-back and reconciliation | Provider execution or Candidate authorship |
| `bullet-portal` | Sequence-bound projections and pending/unknown operator state | Authority inference or optimistic success |

Jeryu is the source authority and preferred localhost forge, consumed as an
independently versioned, signed external component. It is not copied into the
four-repository Bullet family. The first GA profile, `self-hosted-v1`, binds
Ubuntu 24.04 x86_64/systemd, Claude, and local Jeryu. GitHub, GitLab.com, and
self-managed GitLab are separately certified effect profiles; the later
`universal-v1` composition requires all of them, all four providers, and all
five platform profiles.

Baseline V1 is a scope description, not a CLI profile—there is no executable
`v1-ga` profile. Release decisions must name an implemented profile and an
absolute admitted receipt registry; today every such decision remains blocked.

## Seven functions and five authorities

The seven functional planes map onto five authority domains; adding a function
does not add a completion vote. Control maps to Control. Cognitive Execution,
Repository Execution, and Session Supervision map to Execution while keeping
provider sessions, Runner, and BulletGit as distinct principals. Independent
Verification maps to Verification. Effect and Delivery maps to
Delivery/integration with separate broker, attestor, and integrator identities.
Evidence and Audit maps to Evidence/audit; Portal remains a non-authoritative
reader. The exact responsibility and credential mapping is tabulated in the
[`assurance/closure-roadmap.md`](assurance/closure-roadmap.md).

## One engineering transaction

1. Kernel durably admits a command, graph subject, role/task contract, budget,
   and policy/configuration/routing snapshots.
2. Kernel allocates a lease and monotonically increasing fence in one durable
   transaction; Runner receives only the exact dispatch authority it needs.
3. A read-only provider produces a validated `PatchProposal` with admitted gate
   IDs. It never receives Git, forge, cloud, or verification authority.
4. BulletGit checks Kernel authority online immediately before mutation and
   applies the proposal into a new generation atomically.
5. Runner prepares an exact provenance-bound Candidate. A separate verifier
   reconstructs that Candidate and emits exact-subject Evidence.
6. The effect broker reserves one exact desired remote state, dispatches, reads
   back, and reconciles. Lost response is `UNKNOWN`, never an automatic retry.
7. Portal renders only durable snapshots and ordered event watermarks. Green
   requires runtime-valid `VERIFIED` Evidence and Effect receipts for the same
   command subject.

## Evolutionary control

Roles, dissent, selection, and fusion operate over immutable artifacts and new
Variants, never by allowing multiple writers into one checkout. Fitness is an
evidence vector bound to the Candidate, environment, oracle custody, cost, and
policy; model self-rating is not fitness. Routing must satisfy hard risk,
capability, quota, budget, context, and independence constraints before any
optimization. See [`architecture/evolutionary-control.md`](architecture/evolutionary-control.md).

## Current proof boundary

The wire contract, component ledger, private-workspace simulator, portal
projection, and deterministic child-process demo have substantial component
proof. Production online authority, durable Runner transport, full BulletGit
generation semantics, independent verifier custody, protected Jeryu/GitHub
effects, all four providers, signed packages, and installer evidence remain
blocked. The first-GA executable decision is `bullet-family check release
--profile self-hosted-v1 --receipts <admitted-absolute-registry> --json`; the
later maximum-scope decision uses `universal-v1`. Neither is replaced by this
summary.

Continue with [`boundaries.md`](boundaries.md),
[`architecture/overview.md`](architecture/overview.md), the current
[`assurance/closure-roadmap.md`](assurance/closure-roadmap.md), and the generated
[`assurance/release-truth.generated.md`](assurance/release-truth.generated.md).
The [`assurance/v1-closure-plan.md`](assurance/v1-closure-plan.md) is preserved
only as superseded planning provenance.
