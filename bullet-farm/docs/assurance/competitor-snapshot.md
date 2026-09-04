# Competitor comparison snapshot

Status: **RESEARCH INPUT — not release evidence**
Observed: 2026-08-25 UTC; Gas City parity correction verified 2026-08-26 UTC
Owner: Bullet Farm maintainers

Revision 2026-08-26: Gas City v1.4.1 was already pinned in the paper evidence
lock but was accidentally absent from this shorter README snapshot. This
append-only research correction adds that exact subject and does not change any
previous rating or make a benchmark or superiority claim.

This bounded snapshot prevents the design comparison from silently tracking a
moving branch. It records public upstream subjects and extracts mechanisms to
test. It is not a benchmark result and makes no superiority claim.

## Exact upstream subjects

| Project | Subject observed | Source |
| --- | --- | --- |
| Gas Town | release `v1.2.1`, peeled commit `319d33a91b2deca59bba6dd26be6b9daf8eaacf6`, observed 2026-08-25 | [release](https://github.com/gastownhall/gastown/releases/tag/v1.2.1), [commit](https://github.com/gastownhall/gastown/commit/319d33a91b2deca59bba6dd26be6b9daf8eaacf6) |
| Gas City | release `v1.4.1`, peeled commit `58ef17e3bd685fd5cf7f21286277b208d3324590`, observed 2026-08-25 and reverified 2026-08-26 | [release](https://github.com/gastownhall/gascity/releases/tag/v1.4.1), [commit](https://github.com/gastownhall/gascity/commit/58ef17e3bd685fd5cf7f21286277b208d3324590) |
| DeepSeek Harness | developer-preview tag `dsh-v0.1.1-rc.2`, commit `b150a551b8d465e31e418e1b2eaf5e79bbb7d28e` (2026-08-21) | [tag](https://github.com/deepseek-ai/DeepSeek-Harness/releases/tag/dsh-v0.1.1-rc.2), [commit](https://github.com/deepseek-ai/DeepSeek-Harness/commit/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e) |
| Omnigent | release `v0.10.0` (2026-08-19), commit `40755dd8dddb07e1eb6e4055d1d9936e184ceb9b` | [release](https://github.com/omnigent-ai/omnigent/releases/tag/v0.10.0), [commit](https://github.com/omnigent-ai/omnigent/commit/40755dd8dddb07e1eb6e4055d1d9936e184ceb9b) |

These four tags are the only subjects adjudicated below. DeepSeek Harness
labels rc.2 a developer preview; that is a scope fact, not a stability promise.
No different product, upstream development branch, or post-tag commit
contributes to a rating. A later comparison must create a new dated snapshot
rather than silently moving any subject.

## Pinned dimension notes

These are documentation ratings, not execution results. Every external source
below is an immutable commit permalink. `Not documented` means the reviewed
pinned contract does not specify the Bullet-style property; it is not proof
that no implementation can exist. `N/A` means the pinned product scope does
not own that integration role.

| Subject and dimension | Rating | Pinned basis and limit |
| --- | --- | --- |
| Gas Town — writer identity | Partial/configuration-dependent | Gas Town documents selectable [agent runtimes](https://github.com/gastownhall/gastown/blob/319d33a91b2deca59bba6dd26be6b9daf8eaacf6/README.md#L556-L566) and a designated [refinery engineer](https://github.com/gastownhall/gastown/blob/319d33a91b2deca59bba6dd26be6b9daf8eaacf6/internal/refinery/engineer.go#L157-L175), but not one universally exclusive repository-writer principal. |
| Gas Town — incarnation fence | Not documented | The refinery has [claim and lifecycle checks](https://github.com/gastownhall/gastown/blob/319d33a91b2deca59bba6dd26be6b9daf8eaacf6/internal/refinery/engineer.go#L1647-L1665), but the pinned contract does not document a monotonically advancing fence carried by every mutation attempt. |
| Gas Town — exact verification subject | Partial/configuration-dependent | Merge work is bound to [merge-request Git state](https://github.com/gastownhall/gastown/blob/319d33a91b2deca59bba6dd26be6b9daf8eaacf6/internal/refinery/engineer.go#L661-L680) and [revalidated before completion](https://github.com/gastownhall/gastown/blob/319d33a91b2deca59bba6dd26be6b9daf8eaacf6/internal/refinery/engineer.go#L1143-L1159), but no complete Candidate-style base/head/tree/attempt identity is documented. |
| Gas Town — effect read-back | Partial/configuration-dependent | Git operations include [post-operation repository inspection](https://github.com/gastownhall/gastown/blob/319d33a91b2deca59bba6dd26be6b9daf8eaacf6/internal/git/git.go#L1902-L1934) and the refinery [rechecks head state](https://github.com/gastownhall/gastown/blob/319d33a91b2deca59bba6dd26be6b9daf8eaacf6/internal/refinery/engineer.go#L718-L738); this is not a universal idempotency-keyed effect-reconciliation contract. |
| Gas Town — protected integration | Partial/configuration-dependent | A designated [refinery merge actor](https://github.com/gastownhall/gastown/blob/319d33a91b2deca59bba6dd26be6b9daf8eaacf6/internal/refinery/engineer.go#L163-L175) applies [merge and verification gates](https://github.com/gastownhall/gastown/blob/319d33a91b2deca59bba6dd26be6b9daf8eaacf6/internal/refinery/engineer.go#L750-L835), subject to deployment and repository configuration. |
| Gas Town — truthful uncertainty | Partial/configuration-dependent | The refinery preserves [non-success and recovery paths](https://github.com/gastownhall/gastown/blob/319d33a91b2deca59bba6dd26be6b9daf8eaacf6/internal/refinery/engineer.go#L1317-L1330) and typed [merge-request outcomes](https://github.com/gastownhall/gastown/blob/319d33a91b2deca59bba6dd26be6b9daf8eaacf6/internal/refinery/types.go#L64-L103), but not Bullet's durable `UNKNOWN` effect state. |
| Gas City — writer identity | Partial/configuration-dependent | The pinned contract documents [multiple runtime providers, Beads-backed work, a controller, packs, and rig-scoped orchestration](https://github.com/gastownhall/gascity/blob/58ef17e3bd685fd5cf7f21286277b208d3324590/README.md#L12-L36), but not one universally exclusive repository-writer principal. |
| Gas City — incarnation fence | Not documented | The pinned [controller/session/runtime map](https://github.com/gastownhall/gascity/blob/58ef17e3bd685fd5cf7f21286277b208d3324590/README.md#L145-L167) documents supervision and session identity utilities, but not a monotonically advancing fence carried by every repository or effect mutation. |
| Gas City — exact verification subject | Not documented | The pinned contract lists [optional GitHub gates](https://github.com/gastownhall/gascity/blob/58ef17e3bd685fd5cf7f21286277b208d3324590/README.md#L43-L56) and convergence gate handling, but does not document a complete Candidate-style base/head/tree/Attempt subject for independent verification. |
| Gas City — effect read-back | Not documented | The pinned contract has a [controller that reconciles desired and running process state](https://github.com/gastownhall/gascity/blob/58ef17e3bd685fd5cf7f21286277b208d3324590/README.md#L30-L36), but does not document idempotency-keyed remote-effect read-back or adoption. |
| Gas City — protected integration | Partial/configuration-dependent | The pinned prerequisites expose [optional GitHub gates](https://github.com/gastownhall/gascity/blob/58ef17e3bd685fd5cf7f21286277b208d3324590/README.md#L43-L56) and the repository map names convergence gate handling, but protected integration depends on the configured workflow and forge. |
| Gas City — truthful uncertainty | Not documented | The pinned contract documents a [controller/supervisor and health patrol](https://github.com/gastownhall/gascity/blob/58ef17e3bd685fd5cf7f21286277b208d3324590/README.md#L12-L36), but does not document a durable ambiguous-effect state or its reconciliation rules. |
| DeepSeek Harness — writer identity | Not documented | The experimental agent-team package exposes [team tools](https://github.com/deepseek-ai/DeepSeek-Harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/packages/experimental/tool-agent-team/src/index.ts#L30-L37) and a [delegation contract](https://github.com/deepseek-ai/DeepSeek-Harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/packages/experimental/tool-agent-team/README.md#L45-L48), but no sole repository-writer identity is documented. |
| DeepSeek Harness — incarnation fence | Not documented | [Subagent lifecycle controls](https://github.com/deepseek-ai/DeepSeek-Harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/docs/subsystems/subagent.md#L145-L159) and [job lifecycle controls](https://github.com/deepseek-ai/DeepSeek-Harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/docs/subsystems/jobs.md#L173-L178) do not document a monotonically advancing mutation-authority fence. |
| DeepSeek Harness — exact verification subject | Not documented | The pinned [filesystem tool contract](https://github.com/deepseek-ai/DeepSeek-Harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/docs/subsystems/filesystem.md#L114-L151) and [kernel architecture](https://github.com/deepseek-ai/DeepSeek-Harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/docs/architecture.md#L92-L104) do not specify an immutable Candidate-like verification subject. |
| DeepSeek Harness — effect read-back | N/A | The pinned [architecture scope](https://github.com/deepseek-ai/DeepSeek-Harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/docs/architecture.md#L98-L120) is a composable cognition and execution harness, not a protected-integration effect broker. |
| DeepSeek Harness — protected integration | N/A | The pinned [architecture scope](https://github.com/deepseek-ai/DeepSeek-Harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/docs/architecture.md#L98-L120) does not assign the harness ownership of a protected-ref integration role. |
| DeepSeek Harness — truthful uncertainty | Partial/configuration-dependent | The sandbox contract exposes [explicit isolation outcomes](https://github.com/deepseek-ai/DeepSeek-Harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/docs/subsystems/sandbox.md#L30-L38) and [typed failure behavior](https://github.com/deepseek-ai/DeepSeek-Harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/docs/subsystems/sandbox.md#L118-L154), but not Bullet's durable ambiguous-effect state. |
| Omnigent — writer identity | Partial/configuration-dependent | Omnigent documents [native harness execution](https://github.com/omnigent-ai/omnigent/blob/40755dd8dddb07e1eb6e4055d1d9936e184ceb9b/README.md#L288-L292), [bot Git setup](https://github.com/omnigent-ai/omnigent/blob/40755dd8dddb07e1eb6e4055d1d9936e184ceb9b/docs/OMNIGENT_BOT_SETUP.md#L101-L121), and a [Git policy boundary](https://github.com/omnigent-ai/omnigent/blob/40755dd8dddb07e1eb6e4055d1d9936e184ceb9b/omnigent/policies/builtins/github.py#L1-L35), but no universally sole repository writer. |
| Omnigent — incarnation fence | Partial/configuration-dependent | The runner carries [lease/session state](https://github.com/omnigent-ai/omnigent/blob/40755dd8dddb07e1eb6e4055d1d9936e184ceb9b/omnigent/runner/app.py#L2081-L2092), performs [generation-aware renewal checks](https://github.com/omnigent-ai/omnigent/blob/40755dd8dddb07e1eb6e4055d1d9936e184ceb9b/omnigent/runner/app.py#L5191-L5211), and rejects [conflicting ownership state](https://github.com/omnigent-ai/omnigent/blob/40755dd8dddb07e1eb6e4055d1d9936e184ceb9b/omnigent/runner/app.py#L5230-L5249); coverage is not a universal repository-mutation fence. |
| Omnigent — exact verification subject | Not documented | The reviewed [policy contract](https://github.com/omnigent-ai/omnigent/blob/40755dd8dddb07e1eb6e4055d1d9936e184ceb9b/docs/POLICIES.md#L286-L312) and [Git policy checks](https://github.com/omnigent-ai/omnigent/blob/40755dd8dddb07e1eb6e4055d1d9936e184ceb9b/omnigent/policies/builtins/github.py#L942-L981) do not specify a Candidate-like base/head/tree identity for independent verification. |
| Omnigent — effect read-back | Partial/configuration-dependent | Sandbox provisioning performs [remote state inspection and reconciliation](https://github.com/omnigent-ai/omnigent/blob/40755dd8dddb07e1eb6e4055d1d9936e184ceb9b/omnigent/onboarding/sandboxes/blaxel.py#L793-L848), but the pinned contract does not generalize this to every external effect. |
| Omnigent — protected integration | Partial/configuration-dependent | [Policy restrictions](https://github.com/omnigent-ai/omnigent/blob/40755dd8dddb07e1eb6e4055d1d9936e184ceb9b/docs/POLICIES.md#L286-L312) and [GitHub-operation checks](https://github.com/omnigent-ai/omnigent/blob/40755dd8dddb07e1eb6e4055d1d9936e184ceb9b/omnigent/policies/builtins/github.py#L942-L981) can constrain integration, subject to policy and deployment configuration. |
| Omnigent — truthful uncertainty | Partial/configuration-dependent | Native delivery distinguishes [delivery classifications](https://github.com/omnigent-ai/omnigent/blob/40755dd8dddb07e1eb6e4055d1d9936e184ceb9b/omnigent/_native_post_delivery.py#L135-L162), applies [post-delivery reconciliation](https://github.com/omnigent-ai/omnigent/blob/40755dd8dddb07e1eb6e4055d1d9936e184ceb9b/omnigent/_native_post_delivery.py#L213-L271), and preserves [non-success terminal outcomes](https://github.com/omnigent-ai/omnigent/blob/40755dd8dddb07e1eb6e4055d1d9936e184ceb9b/omnigent/_native_post_delivery.py#L302-L375), but not Bullet's exact `UNKNOWN` contract. |

## Mechanisms worth preserving

The pinned, cited dimension notes above identify useful orchestration, harness,
and delivery mechanisms without inferring undocumented behavior or comparative
quality. Bullet Farm should evaluate its own product contract through these
exact measurements:

| Property | Required Bullet measurement |
| --- | --- |
| Durable work | Restart from every transaction boundary retains one exact Mission/Attempt/Candidate history |
| Role flexibility | A role is typed policy/configuration, not hidden authority or a hard-coded persona |
| Parallel fan-out | Receipts report useful surviving outcomes, contention, cost, latency, and verifier backlog |
| Provider plurality | Identical proposal/evidence contracts pass independently for Claude, Codex, Cursor, and Antigravity |
| Health reconciliation | Lease/fence and observation state—not terminal names or silence—drive recovery |
| API and projection scale | Typed pagination, replay watermarks, warm-read behavior, and degraded-state truth remain correct under a measured run/session corpus |
| Declarative workflow control | Retry, fan-out, drain, cancellation, and scope transitions preserve one exact graph/effect identity under faults |
| Review and gap filling | Writer, verifier, and effect attestor remain independent exact-subject principals |
| Portability | One signed family lock and package manifest reproduces clean ordinary clones from a hub-only start |
| Release evidence | Archives, checksums, SBOMs, attestations, real-inference gates, and installer receipts are compared by exact subject rather than feature presence |

## Deliberate Bullet boundaries

Bullet defines an authority boundary in which transport success, task closure,
session state, and model judgment cannot confer mutation, verification, or
integration authority. This is a statement of Bullet's contract, not a claim
that its implementation is stronger than a compared product.

The resulting split is intentional:

- Kernel is the durable transaction and lease authority.
- BulletGit is the exact repository-subject and Candidate authority.
- providers are read-only proposal producers;
- a clean verifier owns independent Evidence;
- an effect broker owns forge credentials and ambiguity reconciliation; and
- Portal is a sequence-bound projection only.

These boundaries cost more machinery than a role/session orchestrator. The cost
is justified only if receipts show fewer destroyed changes, false completions,
duplicate effects, unrecoverable sessions, and escaped regressions at acceptable
latency and spend.

## Fair benchmark contract

Run the same repository/task corpus, tool budget, wall-clock limit, provider
profiles, retry cap, and integration policy. Report distributions and failures,
not a single success rate. At minimum capture:

- candidate survival and protected-integration rate;
- escaped-defect and revert rate;
- false-completion and zero-test rate;
- duplicate/ambiguous effect rate;
- crash recovery time and lost-work bytes;
- model/tool spend, elapsed time, and human interventions; and
- p50/p95 verifier backlog and end-to-end latency.

No benchmark begins while Bullet lacks one connected, independently signed
`TRANSACTION_PROOF` for its five-plane transaction. No public benchmark,
performance, or superiority comparison is published without raw task subjects,
configuration, receipts, exclusions, and exact upstream commits. The bounded
source-documentation status table above is not a benchmark or superiority claim.

## Refresh procedure

1. Resolve the released Gas Town, Gas City, DeepSeek Harness, and Omnigent tags,
   including annotated tag objects and peeled commits. Do not mix a different
   product, development branch, or post-tag commit into the same rating snapshot.
2. Record observation date, immutable URLs, configurations, and benchmark
   corpus digest.
3. Review upstream architecture and release notes for changed primitives.
4. Add a new dated snapshot or append a clearly delimited revision; do not
   silently replace the subjects used by an existing result.
5. Re-run only after Bullet's exact build and live receipts are current.
