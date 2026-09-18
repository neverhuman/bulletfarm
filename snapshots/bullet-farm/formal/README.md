# Gate 0 formal assurance

Status: Enforced
Owner: Bullet Farm maintainers
Last reviewed: 2026-08-25
Applies to: Gate 0

This corpus intentionally contains exactly two bounded models:

- `LeaseFence.tla` covers acquisition, permanent fences, expiry, reclaim, scope-revision
  acknowledgement, freeze, and restore epoch invalidation.
- `EffectCheck.tla` covers durable intent, ambiguous dispatch, exact read-back, third-party remote
  state, proof-before-check, policy expiry, freeze, crash, and restore.

`toolchain.lock.json` pins the stable TLC 1.7.4 release asset by its published SHA-1 and a locally
recorded SHA-256. `model-lock.json` pins every module/config hash and the deterministic
single-worker state counts and depth. `model-check.sh` downloads only that exact asset, verifies
both digests, requires Java 21, rejects a third model, and fails on any lock-shape,
source/config/state/depth drift. Every invocation uses a unique private run root with separate
per-model metadata and logs; `model-check-concurrency-test.sh` overlaps two real pinned checks and
refuses shared metadata, incomplete output, count/depth drift, or leaked run state.

The JSON traces under `formal/traces/` are executable conformance fixtures, not model-check
success claims. Rust tests replay the same refusal/adoption decisions. Run `just model-check`.

## Exactly what is checked

The pinned counts are for exactly these configurations. Weak fairness is applied only to internal
scheduler work that is already continuously enabled; no fairness assumption manufactures time,
operator approval, policy renewal, or a remote outcome.

- `LeaseFence.cfg` (`Runners = {r1, r2}`, `MaxFence = 3`, `MaxTime = 4`, `MaxScope = 3`,
  `MaxOps = 2`, `MaxEpoch = 2`, `MaxFreeze = 3`) — INVARIANTS `TypeOK`,
  `BarrierHasNoInFlightApply`, `RestoredAuthorityNeedsFreshAcquire`,
  `RecoveryApprovalIsScoped`; PROPERTIES `ExpiredLeaseEventuallyReaped` and
  `RecoveryApprovalEventuallySettles`; weak fairness on `AbortInvalidApply`, `Reap`,
  and `RecoverActive`. `Reap` is the single scheduler reaping transition;
  `Terminated` / `StutterDone` is the named terminal successor when the bound is
  exhausted and the holder is empty. `CHECK_DEADLOCK` is `TRUE`. State graph:
  578,478 generated, 141,963 distinct, depth 27.
- `EffectCheck.cfg` (`MaxDispatches = 2`, `MaxEpoch = 2`) — INVARIANTS `TypeOK`,
  `AtMostOneLogicalEffect`, `VerifiedEffectWasReadBack`, `CheckRequiresDurableProof`,
  `ThirdPartyStateIsNeverAdopted`; PROPERTIES `NoNewDispatchAfterStop`,
  `UnknownEffectEventuallyReconciled`, and `UnknownCheckEventuallyReconciled`; weak fairness on
  `ReadBackEffect` and `ReadBackCheck`. State graph: 1,585 generated, 378 distinct, depth 18.

`ExpiredLeaseEventuallyReaped` starts only after the deadline is already due. It does not assume
that `Tick` or wall time progresses. Weak fairness clears an invalid in-flight apply and then reaps
the due lease; restore may also invalidate it. Recovery approval is an explicit separate action and
has no fairness assumption. Once present, `RecoveryApprovalEventuallySettles` requires the approval
to stop pending: weakly fair activation consumes it, while a newer restore may revoke it and require
fresh approval for the new authority epoch.

An effect or check in `unknown` has durable reconciliation work. The two weak-fair read-back actions
require it eventually to leave `unknown` for exactly one typed state: `verified` for the desired
remote value, `intent` when no mutation occurred, or `orphaned` for a third-party value. The model
does not turn timeout, ambiguity, or a foreign value into PASS.

Deadlock checking is enabled for both models. `EffectCheck` has its explicit `Crash`
self-loop. Every executable `LeaseFence` assignment clamps `expires` to `0..MaxTime`;
when time is exhausted, a due holder can be reaped after an invalid in-flight apply is
aborted, and `StutterDone` is the explicit successor once `Terminated` holds. Weak
fairness on the single `Reap` transition is the scheduler progress that empties an
expired holder (`ExpiredLeaseEventuallyReaped`); it does not manufacture wall-clock
progress or require that `Reap` occur when `Restore` already invalidates the lease.

`NoNewDispatchAfterStop == [][(~policyLive \/ frozen) => UNCHANGED <<effectDispatches,
checkDispatches>>]_vars` states the guarantee the protocol actually gives: after policy expiry or
freeze, no *new* dispatch leaves. `DispatchEffect` and `DispatchCheck` are the only actions that
raise a dispatch counter and both are guarded by `policyLive /\ ~frozen`, so the property is the
regression guard on those two guards.

`NoDispatchAfterStop` stays defined in `EffectCheck.tla` and stays unchecked, with the reason in
the module. As a state predicate it is false, and TLC refutes it at depth 4 (`Init` →
`PersistEffectIntent` → `DispatchEffect` → `ExpirePolicy`, and symmetrically `Freeze`): a policy
expiry or freeze may arrive while `effectPhase = "dispatching"`, because a stop cannot recall a
request already on the wire. Listing it as an invariant would assert something false; weakening it
would hide the boundary. An in-flight dispatch surviving a stop is therefore a modelled reality the
read-back path must absorb, not a case the model rules out.

Re-pinning after a model change: run `bash formal/model-check.sh write`. This is the only supported
generator; it runs both pinned TLC subjects to completion and atomically replaces `model-lock.json`
only after exact schema/inventory, hashes, counts, and depth validate. Do not hand-edit the lock.
Then run `bash formal/model-check.sh check` twice and require identical hashes, counts, and depths.
The default mode is `check`, and neither default nor explicit `check` writes the lock. The V1
contract of exactly two models is unchanged; `model-check.sh` still refuses a third.
