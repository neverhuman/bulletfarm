# ADR 0008: Local Jeryu and GitHub are separate gates

Status: Accepted
Owner: Bullet Farm maintainers
Last reviewed: 2026-08-24
Applies to: forge certification

## Decision

LocalBareForge is the deterministic fault-test boundary. Local Jeryu is the first self-hosted
production gate and is consumed only through pinned releases from the permitted Jeryu family.
GitHub is a separately certified later adapter. Neither gate substitutes for the other.

If Jeryu lacks protected-target expected-OID, exact-SHA checks, branch protection, or protected
integration, the capability is implemented and released in the permitted Jeryu family. Bullet does
not simulate forge sovereignty.

## Consequence

No committed sibling path dependency, direct agent-to-forge mutation, or synthetic remote receipt
can satisfy either gate.
