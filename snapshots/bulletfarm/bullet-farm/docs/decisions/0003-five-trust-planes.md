# ADR 0003: Five trust planes and principal separation

Status: Accepted
Owner: Bullet Farm maintainers
Last reviewed: 2026-08-25
Applies to: all runtime work

## Decision

Keep seven functional planes, but authorize transactions through exactly five
domains. The mapping is closed:

| Functional plane | Transaction authority |
| --- | --- |
| Control | Control |
| Cognitive execution | Execution |
| Repository execution | Execution |
| Session supervision | Execution |
| Independent verification | Verification |
| Effect and delivery | Delivery/integration |
| Evidence and audit | Evidence/audit |

Provider sessions, Runner, and BulletGit remain separate workload principals
inside Execution. Broker, attestor, and integrator remain separate principals
inside Delivery/integration. Observer and auditor remain separate principals
inside Evidence/audit. Portal is a projection-only reader; the evolution engine
is a decision aid with no mutation credential.

Role names in a recipe never establish identity. One-host deployment uses distinct OS identities
and peer credentials; distributed deployment later uses distinct SPIFFE-compatible identities.

## Consequence

A plane may consume only grants addressed to its authenticated principal.
Compromise of evolution, Portal, author, or broker cannot manufacture
independent Evidence or integration authority. The seven-versus-five vocabulary
is therefore not a topology contradiction: one describes useful functions, the
other describes independently authorized transaction domains.
