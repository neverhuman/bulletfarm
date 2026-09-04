# ADR 0004: Scope amendment tracks

Status: Accepted
Owner: Bullet Kernel maintainers
Last reviewed: 2026-08-24
Applies to: scope changes after launch

## Decision

Track A applies only within an unchanged sandbox/network/secret/protected-resource envelope. It
enters a mutation barrier, proves zero in-flight applies, serializably acquires the free path lock,
appends and revokes scope revisions, waits for sole-writer acknowledgement, then resumes the same
Attempt and fence.

Track B covers every envelope or authority increase and creates a successor Attempt, permanent new
fence, fresh sandbox, nonce, grants, reservations, and credentials.

## Consequence

An old scope revision cannot apply after acknowledgement. A prose claim that expansion is “small”
does not select Track A; normalized envelope classification does.
