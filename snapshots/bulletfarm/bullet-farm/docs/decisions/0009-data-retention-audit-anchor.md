# ADR 0009: Data classification, retention, and audit anchoring

Status: Accepted
Owner: Bullet Kernel maintainers
Last reviewed: 2026-08-24
Applies to: CAS, holdouts, events, and audit

## Decision

CAS writes atomically, verifies tagged digests, records size/media/classification/tenant metadata,
scans secrets at ingress, encrypts classified evidence, and uses reference-traced GC. Expiring
payloads leave hash-only tombstones or use cryptographic erasure.

Product-verification and research-confirmation holdouts use separate stores, keys, custodians, and
query ledgers. Ordered append-only audit batches chain roots, use an auditor key, and anchor each
root outside the primary database.

## Consequence

Restore must detect gaps, reorder, and rollback. Raw holdouts and sensitive telemetry are not portal
or author data.
