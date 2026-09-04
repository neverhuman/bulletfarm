# Bullet Farm code map

Status: **maintainer routing guide; not runtime or release authority**

Last reviewed: 2026-08-25

Bullet Farm is a family of four independent repositories. A change should begin in the repository that owns the relevant authority; cross-repository generated artifacts move through versioned contracts, never committed sibling path dependencies.

## Repository ownership

### `bullet-farm`: family and public contract

| Concern | Primary paths |
| --- | --- |
| Public onboarding and developer commands | `README.md`, `Justfile`, `scripts/` |
| Family discovery and diagnostics | `src/doctor/`, `src/check/`, `repos.manifest.toml` |
| Source setup and release verification | `src/setup/`, `src/release/`, `family.lock`, `release/` |
| Shared wire and schemas | `contracts/`, `crates/bullet-wire/` |
| Policy and invariant registry | `policy/` |
| Canonical/hostile fixtures | `fixtures/` |
| Formal models and assurance | `formal/`, `docs/assurance/` |
| Public documentation and reproducible media | `docs/`, `docs/readme-media/` |

The checked-in schema-2 `family.lock` is diagnostic only. Do not turn source-setup prose into install authority.

### `bullet-kernel`: durable control and trust boundaries

| Concern | Primary paths |
| --- | --- |
| Domain state and invariants | `crates/domain/` |
| Application transactions and authority | `crates/application/` |
| SQLite ledger and migrations | `crates/adapters/src/sqlite/`, `db/migrations/` |
| Provider-neutral protocol/admission | `crates/harness-core/` |
| Provider protocol adapters | `crates/harness-claude/`, `crates/harness-codex/`, `crates/harness-cursor/`, `crates/harness-antigravity/` |
| Runner supervision and self-fencing | `crates/runner/`, `apps/bullet-runner/` |
| Independent verification | `crates/verifier/`, `apps/bullet-verifier/` |
| Effects and reconciliation | `crates/effects/`, `apps/bullet-effects/` |
| HTTP/SSE API and projections | `apps/bullet-farmd/` |
| Read-only MCP projection adapter | `apps/bullet-mcpd/`, `docs/mcp.md` |

Kernel owns durable admission. A Runner or adapter must not mint authority for itself.

### `bullet-git`: sole repository writer

| Concern | Primary paths |
| --- | --- |
| Change, Candidate, and capability types | `crates/bullet-git-types/` |
| Private clones and repository mutation | `crates/bullet-git-workspace/` |
| Journal, CAS, and recovery | `crates/bullet-git-journal/` |
| Capability RPC and writer daemon | `crates/bullet-gitd/` |

Provider processes and Portal code never receive a general Git write path. Candidate identity changes when any bound repository subject changes.

### `bullet-portal`: projection only

| Concern | Primary paths |
| --- | --- |
| Generated API client and schemas | `src/generated/` |
| Route/page projections | `src/pages/` |
| Reusable view components | `src/components/` |
| Query/replay behavior | `src/hooks/`, `src/api.ts` |
| Browser/component proof | colocated `src/**/*.test.*`, `e2e/`, `ops/ci/` |

Never hand-edit `src/generated/`. Change the Hub/Kernel contract first, regenerate, and review the byte diff. Portal state cannot grant leases, verify Candidates, or settle effects.

## Change X here

| You need to change… | Begin here | Required neighboring proof |
| --- | --- | --- |
| A provider protocol or transcript parser | Kernel `crates/harness-core/` and one `crates/harness-*` adapter | Offline hostile/mutation suite; live status remains blocked without a sealed receipt |
| Provider process supervision | Kernel `crates/runner/` | Lease/fence, timeout, cancellation, process-tree death, and canary-negative tests |
| Git mutation or private-clone behavior | BulletGit `crates/bullet-git-workspace/` and `crates/bullet-gitd/` | Hostile repository/config tests, journal/CAS recovery, exact Candidate sensitivity |
| Durable authority or policy enforcement | Kernel application/domain/SQLite paths | Atomic persistence, restart/replay, stale-fence refusal, contract compatibility |
| Shared policy, DTO, or schema | Hub `policy/`, `contracts/`, `crates/bullet-wire/` | Regenerate canonical bundles/fixtures, then update consumers by immutable subject |
| farmd API | Kernel `apps/bullet-farmd/` | OpenAPI/contract tests, authentication boundary, watermark/replay behavior |
| Portal UI | Portal pages/components/hooks | Generated client unchanged or regenerated from the owning contract; component and browser tests |
| Effect adapter or forge reconciliation | Kernel `crates/effects/` | Idempotency key, timeout to `UNKNOWN`, exact remote read-back, no duplicate write |
| Family setup, package, or release behavior | Hub `src/setup/`, `src/release/`, `src/check/` | Signed schema-3 subjects, hostile filesystem/package fixtures, receipt verification |
| Evidence or release language | Hub generator inputs and `docs/assurance/` | Executable receipt first; regenerate release truth; prose cannot promote a gate |
| CI policy | Each repository's `ops/ci/`, `scripts/ci-local.sh`, `.github/workflows/`, and `ci.toml` | Local lane parity, meta-tests, secretless fork behavior, stable aggregator context |

## Cross-repository order

When a family proof needs more than one repository, use this dependency order:

1. Run BulletGit standalone checks and build the exact `bullet-gitd` subject.
2. Run Kernel standalone checks, then its explicit family inventory with the absolute daemon path.
3. Run Portal standalone checks, then the real-farmd browser proof.
4. Run Hub contracts and the two pinned formal models last.

This order is component observation only until authenticated immutable family provisioning can bind every source OID and hosted result.
