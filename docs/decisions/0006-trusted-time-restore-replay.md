# ADR 0006: Trusted time, restore epoch, and replay

Status: Accepted
Owner: Bullet Kernel maintainers
Last reviewed: 2026-08-24
Applies to: leases, grants, restore, and effects

## Decision

Database time decides server expiry; runner monotonic time self-fences earlier. Expiry predicates are
inside each update, and an expired heartbeat cannot revive authority. Phase-1 lease TTL is at most
15 seconds.

Authority high-water state lives outside restorable snapshots. Restore increments authority epoch,
invalidates leases/tokens/credentials, enters RECOVERING, reconciles uncertainty, verifies audit
roots, and requires independent activation or remains SAFE_STOPPED.

## Consequence

Clock rollback, suspend, snapshots, and replay cannot extend a grant. Timeouts produce uncertainty,
not inferred non-execution.
