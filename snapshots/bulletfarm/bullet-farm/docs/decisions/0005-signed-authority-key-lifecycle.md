# ADR 0005: Signed authority and key lifecycle

Status: Accepted
Owner: Bullet Kernel maintainers
Last reviewed: 2026-08-24
Applies to: every mutation gateway

## Decision

Use compact PASETO v4.public. Its payload is the exact RFC 8785 encoding of strict
`AuthorityClaimsV1`; its footer is the exact RFC 8785 encoding of schema, issuer, key ID, and the
`authority-signing` purpose; its fixed implicit assertion is `bullet-farm.authority.v1alpha1`.
Do not implement a second custom Ed25519 preimage or accept an algorithm field from the caller.

Claims bind a closed audience and operation, a unique Mutation ID, exact domain-separated request
digest, authenticated principal, full graph/repository/Attempt/fence/runner/workspace/scope/context/
configuration/policy/routing/provider closure, authority and freeze generations, a validity window
of at most 15 seconds, and a 256-bit nonce. Every ID is a full lowercase 256-bit prefixed identity.
`not_before` is inclusive and `expires_at` is exclusive.

Request digests use RFC 8785 plus the existing `bullet-wire.v1` length-framed BLAKE3 construction.
Their domains are `authority.request.<operation>.v1alpha1`; the token, transport correlation ID, and
transport deadline are excluded, while every semantic mutation precondition is included. Callers
cannot select a domain for arbitrary JSON: a sealed Rust trait maps only the nine generated strict
request records to their fixed operation. Clone binds the exact ScopeGrant and digest; apply binds the
full validated PatchProposal; checkpoint, preservation, cleanup, candidate preparation, effect
dispatch, and reconciliation bind their complete typed preconditions. The exact compact PASETO bytes
are hashed as `authority.envelope.v1alpha1` for receipts and final checks.

Authority and release keys have distinct purposes and encodings. Keys bind allowed audiences and
have activation, expiry, optional revocation, and retention at least one maximum token lifetime past
expiry. Gateways validate current policy, key use, registered principal, nonce replay, and durable
lease/scope/generation state—not only the local PASETO signature.

Local verification is necessary but insufficient. Immediately before a mutation, the gateway sends
the exact envelope/digest, Mutation ID, audience, operation, and request digest to Kernel's private
final-check boundary. Kernel uses database time and atomically creates or replays a bounded in-flight
mutation reservation. Supersession, freeze, restore, and cleanup must respect that reservation; a
point-in-time lease read alone leaves a post-check/pre-write race. Unavailable or malformed authority
is a refusal, never degraded operation.

An authorization response carries a compact PASETO v4.public `SignedMutationPermitV1`, not opaque
text. Its fixed implicit assertion is `bullet-farm.mutation-permit.v1alpha1`, its footer purpose is
`mutation-permit-signing`, and its lifetime is at most one second. It binds the authority envelope and
token nonce, Mutation and reservation IDs, request, repository/workspace/generation, Attempt/fence,
authority/freeze generations, and its own nonce. BulletGit validates the full permit immediately before
the corresponding operation and consumes an operation-specific private permit by value.

Final decisions are closed `authorized | settled | refused` records with explicit
`fresh | exact-replay | conflict` disposition. A settled exact replay returns the typed durable result,
never another write authorization. After the atomic mutation boundary, BulletGit submits the signed
permit and its digest in a typed `committed | aborted | unknown` settlement. Kernel returns a typed
accepted, exact-replay, conflict, or refused result. Nullable fields are required on the wire and
constrained by the decision/state branch; omission and contradictory branches fail closed.

## Consequence

Unsigned claims, a flat claims-plus-signature JSON object, and placeholder digests never authorize.
Tokens cannot travel through argv, environment, URLs, browser storage, or ordinary logs. The
committed authority golden fixes the canonical claims, request digest, compact PASETO, public key,
envelope digest, mutation permit, decision, and settlement across Rust and TypeScript consumers. The
golden signing key is an upstream deterministic test vector: its private half exists only in test and
generator code, its public half exists only in the fixture, and normative policy must never trust it.
The generated Gate 0 policy contains only the offline release-verification key. Runtime authority trust
must be supplied by a separately generated and protected key bundle chained to a signed family lock.
Runtime issuance, nonce reservation, and online checks remain a Wave 2 prerequisite; this ADR does not
make the offline policy live-admissible.
