# 0015 — The dogfood track: `DOGFOOD_RUN` operational observations and `dogfood-local-v0`

Status: **Proposed — pending operator ratification (OD-K) AND the engineering predecessors named below.**
An independent review on 2026-08-28 held this ADR and its runbook on eight findings; the corrections are
folded in. Component surfaces now exist for the `DOGFOOD_RUN` shape, the `dogfood-local-v0` release
refusal, the diagnostic `check dogfood` board, and one Claude-only `bullet dogfood read-only` compose.
No successful operational receipt or admitted provider run exists, every release profile remains
`BLOCKED`, and this ADR cannot change that.
Owner: Bullet Farm maintainers
Related: 0001 (providers propose, they never write), 0003 (five trust planes), 0005 (signed authority and
key lifecycle), 0011 (signed launch grants; `BULLET_LIVE_ADMISSION` rejected), 0012 (policy v1alpha2 live
admission), 0013 (operator decision register — OD-K lives there)

## Context

The release-assurance program is honest and it is working: 0 of 43 gates are receipted, `check release`
is fail-closed, and every lane correctly refuses to promote fixture-grade evidence. That program answers
one question — *may we ship this to a stranger?*

It has been asked to answer a second, different question — *may we use this ourselves, today, on this
machine, to do our own work?* — and it answers "no" to that too, because it has no vocabulary for it.
The consequence is visible in the family's own record: the plan of record places real providers at
Wave 8, councils at Wave 9 and the non-Claude providers at Wave 10, all after first GA, while
`docs/runbooks/dogfood.md` redefines dogfood as using the coordinator as a claims board. Meanwhile the
operator's own directive is to accelerate by running real frontier models through the loop now.

Both answers can be true at once, and conflating them costs the family the feedback that would make the
release program cheaper to finish. This ADR proposes that the second question receive its own vocabulary, its own
proposed admission path, and its own permanent ceiling.

## Decision

`DOGFOOD_RUN` is **not a sixth evidence class.** The ladder in
[`../assurance/execution-plan.md`](../assurance/execution-plan.md) §2 and
[`../release.md`](../release.md) says every evidence assertion uses exactly one of five classes
(`DESIGNED`, `COMPONENT`, `TRANSACTION`, `LIVE`, `RELEASE`), and an earlier draft of this ADR contradicted
that by inventing a sixth. Corrected: **`DOGFOOD_RUN` is a purpose-separated, non-evidence operational
observation.** Its future record would describe what an operator's own loop did on one host. It would
assert no evidence class and could never be an input to one.

The operational profile name is `dogfood-local-v0`. It does **not** implement `ReleaseProfile`:
`check release --profile dogfood-local-v0` returns typed `NOT_A_RELEASE_PROFILE`.

## Landed component surfaces and remaining gap

The following component boundaries are implemented:

- Hub source defines the hard-false `DOGFOOD_RUN` v0 template and refuses that kind at the semantic
  release registry. It is not a `GateClass` or a release receipt kind.
- `bullet-family check dogfood --json` renders a diagnostic, never-authoritative board. It retains `--track coord|dogfood|all`; each track exits 1 while its blockers remain. Configuration,
  observed operation, and release status are separate. Unavailable enrollment, runtime, conformance,
  containment, durable-launch, and operational read-back consumers block provider readiness; a binding
  alone cannot pass. A blocked release remains visible independently.
- Kernel source exposes `bullet dogfood read-only` for Claude. It can emit one create-once
  `DOGFOOD_READ_ONLY_RECEIPT` and proposal only after a contained turn succeeds. No such successful
  receipt is admitted today, and Codex, Cursor, and Antigravity compose paths do not exist.
- The typed `DogfoodBindingV1` scope and its live-path refusal are implemented. Dogfood requires an
  offline v1alpha2 policy: `live_admission_enabled=true` is refused by the dogfood validator.

The remaining record gap is still material. The Hub template is not a durable operational record, and
the Kernel receipt is not the generated, cross-repository full-loop `DOGFOOD_RUN` described below. That
future record needs explicit bounds, recursive unknown-field refusal, exact full-loop subjects, and an
admitted producer/read-back path. It must not use the `authority-signing` / `provider-runner` key as an
evidence signer: that key admits provider launch, not evidence.

A future `DOGFOOD_RUN` operational record would be limited to this assertion and nothing more:

> On one Linux host, under one operator UID, with an operator-owned generation-2 policy admitted by that
> operator, a change went from a materialized Mission through a read-only provider proposal, an exact
> `PatchProposal` applied by production `bullet-gitd`, a sealed-catalog gate, a verifier chain, and a
> local bare-forge delivery with authoritative read-back — and here are the real subjects it touched.

### What a future `DOGFOOD_RUN` record may claim

- Real, reproducible Git subjects: base, head, tree, Candidate id, delivered ref, read-back result.
- Real gate outcomes from the sealed catalog, including honest `FAIL`, `TIMED_OUT` and `INFRA_ERROR`.
- Real provider facts: enrolled executable path and digest, observed version, invocation count, and
  either a provider-reported cost or an explicit `UNPRICED`.
- That the loop is operable end to end on this host, and — via `check dogfood` — that the family board
  is in a workable state.

### What a future `DOGFOOD_RUN` record may never claim

- Any release, transaction, or independent-evidence eligibility. The future purpose-separated record must
  make all three structurally inexpressible or hard `false` **by type**, not by convention; its validator
  must reject any conflicting value, and the release registry must refuse a `DOGFOOD_RUN` record. Those
  guarantees do not exist until the generated record, validator, and hostile refusal test land.
- Any gate in the release inventory, any `ReleaseProfile` condition, or any scorecard floor.
- Independence of any kind. Executor, verifier and observer would share one process and one UID on this
  host; the future record must label that social custody `OPERATOR_LOCAL_KEY_SAME_UID`, never
  `FIXTURE_KEY_ONLY` (which would understate the key's durability) and never a trusted-custody label (which
  would overstate it).
- Provider liveness for the release program. `release.provider.claude` is closed by OD-A and its
  conformance receipt, not by a dogfood run, however many succeed.
- Recovery of the 2026-08-26 coordinator incident. See "Coordinator" below.

## Admission

The proposed admission would remain the operator's decision. Runtime scope checks must be mechanical, but
operator provenance remains social on this same-UID host. No admission exists until all of these future
requirements and the engineering predecessors below are implemented and independently safety-reviewed:

1. The landed typed dogfood audience/operation binding, carried by an admitted create-once subject whose
   custody, signer lifecycle, digest, replay state, and read-back are independently verified. The current
   environment-selected binding file is structural component machinery, not operator authority. General
   live and release paths must continue to refuse this binding.
2. An operator-owned top-level v1alpha2 policy at an absolute path **outside every repository**, mode 0600,
   with `policy_generation >= 2`, `sandbox_policy.live_admission_enabled = false`, and separate
   `dogfood-launch-signing` and `provider-enrollment-signing` keys with their dedicated purposes, plus
   the separate dogfood binding above. The prior `authority-signing` / `provider-runner` requirement
   is superseded; the older Kernel consumer must migrate before this admission. Every nested
   policy `schema_version` stays `v1alpha1`. A separately admitted producer must encode the resulting
   `PolicySnapshotV1` as RFC 8785 canonical bytes, install it create-once, reopen it, and require byte-exact
   canonical read-back; pretty or sorted output from plain `jq` is not an admissible policy subject.
   `route_policy.evolutionary_authority` stays `false`.
3. The private halves of those keys under operator custody in the data directory, plus exact provider enrollment,
   service identity, credential projection, invocation/spend bounds, validity, revocation, and containment.
4. The canonical OD-K social witness in the family log, naming every field required by ADR 0013. The line
   itself supplies no runtime authority and cannot mechanically distinguish its same-UID author.
5. The reviewed preserved fresh-development transition described in the September 7 amendment below,
   including complete two-location inventories, strict replay/dispositions and exact four-repository W0
   durably bound into Genesis and checked under the final guard and on restart. Original-incident
   recovery remains separate, unresolved and owed. This replaces the earlier recovery-only prerequisite
   as a proposal; the Operating HOLD still prohibits execution before its engineering/review/operator acts.

There is no environment-variable admission. ADR 0011 rejected `BULLET_LIVE_ADMISSION` and this ADR does
not reintroduce it under another name: a shared token is not custody.

**Two limits of that admission, stated plainly rather than implied away.**

*Structural scope is not custody.* `DogfoodBindingV1` now separates the dogfood audience/operation from
general live admission, and the dogfood validator rejects `live_admission_enabled=true`. The checked-in
board still discovers its binding through an environment-selected path, however, and no admitted signer,
create-once publisher, replay ledger, or independent read-back establishes operator custody. The type
closes the scope-confusion bug; it does not complete OD-K.

*Operator provenance on this host is social, not cryptographic.* An earlier draft claimed a consuming
command independently rejects an agent-created policy or key. That is false here: Kernel policy loading
checks regularity, size, stable inode/length and canonical content but **not** owner, mode, signature, or
any OD-K witness (`crates/application/src/policy_snapshot/load.rs:62-103`); key loading checks mode 0600
and the current UID, which every agent on this host shares; and no Hub or Kernel source consumes OD-K or
`AGENT_CHAT.md` at all. A forged `— operator —` line is refused by *people reading the log*, not by the
code. Closing this needs an operator trust root or an identity agents cannot reach; until then the honest
description is custody by convention on a single-UID host.

## What does not change

- **ADR 0001 must hold without amendment.** A future dogfood provider must run read-only and propose;
  `bullet-gitd` must remain the sole writer; `-w/--worktree/--tmux` and every documented escape hatch must
  stay hard-denied. A dogfood run would have strictly the same write topology as a release-grade run — only
  its operational purpose and assurance ceiling would differ.
- **The Jeryu forge must not be touched.** The proposed first dogfood merge target would be a local bare
  mirror under the operator data directory. OD-B would not be consumed, no credential would be minted, and
  the running instance at `127.0.0.1:8787` would be neither modified nor authenticated against.
- **Bullet must never merge into a real repository.** It would deliver a Candidate ref into the mirror and
  read it back. A human would fetch and merge.
- **The release program is unchanged.** No gate, profile, receipt kind, invariant, or scorecard input is
  added, removed, or relaxed by this ADR.

## Coordinator

The dogfood track requires a working coordinator; the 2026-08-26 incident left the live ledger frozen
(`events.jsonl`, mode 0400, no `CURRENT`). Current recovery component machinery does not authorize the
real incident: reviewer policy/custody, trusted clock publication, independent review, and the live
operator checkpoint remain open.

**Operating HOLD:** do not run Fresh Genesis, relocate either ledger, chmod the frozen source, invent
`CURRENT`, or delete incident bytes. The family plan calls the desired independent recovery amendment
“OD-L”, but that label is not accepted authority: checked-in ADR 0013 and this proposed ADR have not been
reviewed and amended to ratify it. Until a reviewed decision amendment and its operator/reviewer acts
exist, both real incident recovery and fresh initialization remain blocked; manual path-exact coordination
remains the only available coordination protocol. DF-R7a and DF-R7b remain open packets, owed in full, on their own lane.

### September 7 amendment proposal: preserved fresh development generation

The selected implementation route is a **distinct internal-development generation**, followed by
Codex, Claude, and Cursor subscription dogfood. This paragraph reconciles the proposed route with
OD-K; it is not ratification, an operator checkpoint, or recovery of the old generation. The Operating
HOLD above remains effective. The earlier recovery-only sequencing is superseded as a proposal;
DF-R7a/R7b remain separate unresolved incident-recovery obligations.

Before an operator can authorize retirement or initialization, the consuming transition must:

1. Bind complete inventories of **both** the outer family coordinator and the Hub-local coordinator,
   including original, interrupted, tainted, and quarantine bytes; preserve their original identities,
   modes, and digests. Refuse unreviewed destinations, collisions, omissions, partial relocation, and
   changed subjects. Verify retirement read-back for every inventoried location.
2. Strictly replay the reviewed ledger with the same claim/heartbeat/handoff/receipt reducer used by
   the coordinator. Expiry is not a disposition. Every abandoned claim and unresolved handoff needs
   an exact reviewed disposition; unresolved work may be retained for later recovery only when the
   admission explicitly quarantines it without adopting leases or implying integration.
3. Bind all four exact commit/tree pairs, repository identities, cleanliness, replay watermark,
   dispositions, and the independently authored review record. Revalidate them under the final
   initialization guard. Missing/changed input must leave no new generation.
4. Durably bind admission references into the new Genesis and enforce them on exact retry and restart.
   Publishing unreferenced sidecar files is insufficient. Incident-derived initialization must not
   fall back to pristine bootstrap when inputs are omitted.
5. Pass conflict, owner-heartbeat, expiry, stale-generation, exact-retry, changed-request, handoff,
   integration-receipt, and restart tests, including crashes and subject drift around publication.

Only after engineering proof and independent review may the operator approve the exact amendment
and admission packet. Provider enrollment/launch is a later, separate checkpoint after its consumers
and refusal tests exist. Agents never write keys, enrollments, or operator decisions. A fresh
coordinator alone cannot authorize provider spend or integration.

Initial dogfood requires Codex App Server, Claude headless stream JSON, and Cursor ACP; Antigravity
follows separately. Providers propose against a read-only exact snapshot. BulletGit applies the
proposal in a private clone; sealed gates and separate review qualify an exact Candidate; a human
approves integration. Subscription enrollment is per account, with private provider HOME and finite
call/time/concurrency limits. Missing monetary cost remains `UNPRICED`; unknown quota is not unlimited.
The shared wire uses separate dogfood launch and enrollment signing purposes. Kernel's older
`provider-runner` consumer must migrate before admission; labels alone confer no launch authority.

The September 7 source review found the initialization and inventory requirements above incomplete.
The admission kit and family `CEREMONY-WAVE2.md` are **NOT EXECUTABLE**. The statement that only an
operator ceremony remains is withdrawn; no operator is being asked to waive missing engineering.

## Consequences

- Landed surface: `bullet-family check dogfood --json` reports coordinator state, repository dirtiness,
  release status, and dogfood binding state. It is always diagnostic and exits non-zero when a loop
  blocker remains. `tests/dogfood-board.py` forwards its bytes and status.
- Required future authority inputs: an admitted RFC 8785 policy and binding producer/read-back, sanctioned
  transition authority, and an exact transition-bound four-repository W0 subject. Every missing, changed,
  incomplete, dirty, unresolved-claim, or replay-watermark mismatch must refuse.
- Landed refusal: `check release --profile dogfood-local-v0` returns typed `NOT_A_RELEASE_PROFILE`.
- Partial record surface: the hard-false `DOGFOOD_RUN` template and release-registry refusal exist, while
  the durable generated full-loop record and admitted producer do not. `bullet-wire`'s release
  receipt-kind vocabulary must **not** gain a dogfood kind.
- Future dogfood operational observations could cheaply expose faults, costs, and latency that guide the
  W7 chaos campaign and W5 custody-split work. They would be planning inputs, never evidence or substitutes
  for the proof those waves require.

## Falsification

This decision is wrong, and should be reverted, if any of the following is ever observed: a
`DOGFOOD_RUN` operational record admitted by a release gate or profile; a dogfood operational observation
cited as independent or transaction evidence in any document or handoff; a dogfood run performing a
repository mutation outside `bullet-gitd`; a live provider dispatched without the operator policy and launch
grant; exit 78 described as PASS; a dogfood path accepting `live_admission_enabled=true`; Fresh Genesis or
ledger relocation used as incident recovery; or the frozen coordinator generation being modified, adopted,
or described as recovered without the independently authorized recovery chain.
