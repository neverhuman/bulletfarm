# Scorecard (generated)

Status: **instrumented estimate; not release authority.**

Rubric `d2-v1`. Blended **43.5** (architecture 94.5, implemented 40, stranger 3). Frozen baseline was **43.3**.

| # | Dimension | Design | Implemented |
| --- | --- | ---: | ---: |
| 1 | Concurrency and authority kernel | 95 | 64 |
| 2 | Isolation and repository safety | 96 | 78 |
| 3 | Evidence and verification integrity | 94 | 28 |
| 4 | Integration and delivery authority | 92 | 26 |
| 5 | Identity, quota and cost governance | 90 | 8 |
| 6 | Multi-agent collaboration and roles | 88 | 6 |
| 7 | Evolutionary optimization | 86 | 8 |
| 8 | Operator truth and UX | 90 | 48 |
| 9 | Installability and release engineering | 94 | 16 |
| 10 | Security posture | 92 | 58 |
| 11 | Test and assurance depth | 90 | 57 |
| 12 | Documentation honesty | 94 | 86 |

| Row | Admitted | Refusal | Claim |
| --- | --- | --- | --- |
| `d1.nonce-ledger` | no | `PINNED_FAMILY_SUBJECT_UNAVAILABLE` | Durable nonce issue/consume separated |
| `d1.signed-transport` | no | `NO_EVIDENCE_REFERENCE` | Signed lease transport mounted internally |
| `d2.egress-ci` | no | `NO_EVIDENCE_REFERENCE` | Three egress proofs run every push |
| `d3.proof-root-eight` | no | `PINNED_FAMILY_SUBJECT_UNAVAILABLE` | ProofRoot over eight inputs with tamper tests |
| `d4.attestor` | no | `NO_EVIDENCE_REFERENCE` | Attestor binary posts exact-SHA checks |
| `d4.jeryu-live` | no | `NO_EVIDENCE_REFERENCE` | release.forge.jeryu admitted |
| `d5.budgets` | no | `PINNED_FAMILY_SUBJECT_UNAVAILABLE` | Atomic dual-tree reservation/settlement |
| `d6.two-providers` | no | `NO_EVIDENCE_REFERENCE` | Two providers dispatch through the router |
| `d7.evolution-off` | yes | `-` | evolutionary_authority remains false until OD-H |
| `d8.fifteen-surfaces` | no | `NO_EVIDENCE_REFERENCE` | Fifteen portal surfaces render durable subjects |
| `d9.schema-3` | no | `NO_EVIDENCE_REFERENCE` | release.installable-lock admitted |
| `d10.jankurai-90` | no | `NO_EVIDENCE_REFERENCE` | release.jankurai-90 admitted |
| `d11.invariants-51` | no | `NO_EVIDENCE_REFERENCE` | 51/51 invariants enforced |
| `d12.signed-jeryu-tags` | no | `NO_EVIDENCE_REFERENCE` | Jeryu tags are annotated and signed |
| `g2.transaction-proof` | no | `RELEASE_GATE_NOT_ADMITTED` | release.transaction-demo admitted |

A row adds its typed implemented delta only when the kind-specific verifier re-derives the claim from committed Hub bytes or an exact pinned family subject. Mutable sibling checkouts, file presence, unsigned receipts, ignored tests, and `check release` gates do not admit. This page is not release authority.
