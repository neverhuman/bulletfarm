# Historical Phase 9–10 sketch

Status: **SUPERSEDED planning provenance; do not execute**
Last reviewed: 2026-08-25

This phase numbering predates the active dependency graph and is preserved only
for provenance. The current order is
[`closure-roadmap.md`](assurance/closure-roadmap.md): Wave 9 is `evolution-v1`,
Wave 10 is provider/forge/platform breadth and `universal-v1`, and Wave 11 is
`team-v1` followed by `saga-v1`. Nothing below overrides that order.

## Historical Phase 9 — team distribution

Build only after the five-plane `TRANSACTION_PROOF` exists and the self-hosted release
baseline in [`release.md`](release.md) is green:

- PostgreSQL ledger (`bullet-adapters-postgres`; today `from_env()` is `NotConfigured` without `DATABASE_URL`)
- S3-compatible CAS
- mTLS runners and runner epochs
- RBAC/OIDC
- replicated farmd projections
- high-isolation verifier pools

Exit: partition/failover chaos creates no double authority or accepted stale Effect.

## Historical Phase 10 — advanced optimization

Only after empirical proof from Phase 9:

- contextual routing learner (today `bullet-router` is deterministic D0)
- larger councils and code races
- integration batching / bisection
- cross-repository sagas
- S2/S3 sandbox expansion

Exit: statistically valid uplift, no guardrail regression, every adaptive decision explainable.

## Current scaffold

`crates/adapters-postgres` compiles in required CI and refuses to open a connection unless `DATABASE_URL` is set. That is the entire Phase 9 code until G2 and the V1 release baseline are green. PostgreSQL remains a refused V1 scaffold.
