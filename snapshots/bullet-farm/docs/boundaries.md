# Trust and repository boundaries

Status: **normative; violations fail closed**  
Owner: Bullet Farm maintainers  
Last reviewed: 2026-08-25

## Repository family

The family root is a container, not a repository. Its four independently
versioned members are `bullet-farm`, `bullet-kernel`, `bullet-git`, and
`bullet-portal`. Committed sibling `path` dependencies and Git worktrees are
forbidden. Development may create ignored local composition, but release and
setup consume signed tags, exact commit/tree subjects, and a schema-3 family
lock.

Jeryu remains an external source family. A managed localhost forge will consume
a separately signed, content-addressed runtime artifact through an external
component lock bound by the Bullet release manifest. `~/bullet/bullet-jeryu`
must never become a copied source tree or fifth family member.

## Authority separation

| Boundary crossing | Required input | Required output | Failure posture |
| --- | --- | --- | --- |
| Human/Portal → Kernel | authenticated command envelope, CSRF/session binding | durable command ID, initially `PENDING` | transport success is never verification |
| Kernel → Runner | exact dispatch/lease authority, fence, limits, snapshots | heartbeats and typed operation requests | renewal failure freezes and kills the process tree |
| Provider → Runner | bounded native events and schema-valid proposal | invocation receipt plus proposal | malformed, delayed, duplicate, timeout, or quota unknown refuses |
| Runner → BulletGit | Kernel-minted operation permit and exact request digest | journaled generation/Candidate subject | authority outage or stale fence mutates nothing |
| Candidate → Verifier | immutable Candidate and independent oracle custody | Evidence bound to exact reconstruction | zero tests, flaky, infra error, timeout, unsupported, or unknown is not PASS |
| Kernel → Forge | reserved effect intent, expected/desired OIDs, Candidate/proof roots | dispatch/read-back/reconciliation receipt | lost response stays `UNKNOWN`; no second write |
| Kernel → Portal | atomic snapshot watermark and ordered event envelopes | projection only | gaps remain `STALE` until a covering snapshot succeeds |

## Credential custody

- Providers receive minimum read-only provider identity in an ephemeral HOME;
  no SCM, cloud, SSH, effect, or verifier credentials.
- Kernel owns authority signing and durable nonce/reservation state. Runner must
  never mint its own permit or load the Kernel issuing key.
- BulletGit receives no forge integration credential.
- Delivery, attestation, integration, and observation principals are distinct;
  one role cannot satisfy another role's gate.
- Portal receives an HttpOnly local session, never a raw signing key or effect
  credential.

## Evidence and ambiguity

Evidence is useful only for the exact Candidate, policy, configuration,
routing, environment, toolchain, oracle custody, and verifier identity it
binds. Rebase or subject change invalidates it by default. Writer evidence
cannot satisfy an independent tier. A receipt proves an observation, not
authority to perform another operation.

Every remote write is physically at-least-once and logically exactly-once by
reservation plus reconciliation. A timeout after dispatch is `UNKNOWN`. The
resolver adopts only the original desired state for the original fence and
quarantines a third value; it never retries across Jeryu, GitHub, or GitLab.

## Ownership and proof routing

Before editing, consult repository-local `AGENTS.md`, `agent/owner-map.json`,
and `agent/test-map.json`; coordinate exact paths through `bullet-family coord`.
The orchestrator commits only handed-off leaves and records claim IDs in the
commit receipt. Generated zones are regenerated and diffed, never edited by
hand. Stable error codes and repair steps are indexed in [`errors.md`](errors.md).
