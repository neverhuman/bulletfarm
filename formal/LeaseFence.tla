------------------------------ MODULE LeaseFence ------------------------------
EXTENDS Naturals, FiniteSets, TLC

CONSTANTS Runners, MaxFence, MaxTime, MaxScope, MaxOps, MaxEpoch, MaxFreeze

ASSUME /\ Runners # {}
       /\ MaxFence \in Nat \ {0}
       /\ MaxTime \in Nat \ {0}
       /\ MaxScope \in Nat \ {0}
       /\ MaxOps \in Nat \ {0}
       /\ MaxEpoch \in Nat \ {0, 1}
       /\ MaxFreeze \in Nat \ {0}

None == "none"

VARIABLES now, leaseOwner, fence, expires, tokenFence, tokenEpoch,
          authorityEpoch, freezeGeneration, frozen, scopeRevision,
          ackRevision, barrier, inFlight, acceptedApplies, refusedApplies,
          recoveryApproved

vars == <<now, leaseOwner, fence, expires, tokenFence, tokenEpoch,
          authorityEpoch, freezeGeneration, frozen, scopeRevision,
          ackRevision, barrier, inFlight, acceptedApplies, refusedApplies,
          recoveryApproved>>

(*
--algorithm LeaseFenceProtocol {
variables now = 0, leaseOwner = "none", fence = 0, expires = 0,
          tokenFence = [r \in Runners |-> 0],
          tokenEpoch = [r \in Runners |-> 0], authorityEpoch = 1,
          freezeGeneration = 0, frozen = FALSE, scopeRevision = 1,
          ackRevision = 0, barrier = FALSE, inFlight = 0,
          acceptedApplies = 0, refusedApplies = 0;
process (runner \in Runners) {
Loop:
  while (TRUE) {
    either when leaseOwner = "none" /\ ~frozen /\ fence < MaxFence;
      with (nextFence = fence + 1) {
        leaseOwner := self || fence := nextFence ||
        expires := IF now < MaxTime THEN now + 1 ELSE now ||
        tokenFence[self] := nextFence || tokenEpoch[self] := authorityEpoch ||
        ackRevision := scopeRevision;
      };
    or when now < MaxTime; now := now + 1;
    or when leaseOwner = self /\ expires > now /\ ~frozen;
      expires := IF now < MaxTime THEN now + 1 ELSE now;
    or when leaseOwner # "none" /\ expires <= now /\ inFlight = 0;
      leaseOwner := "none" || barrier := FALSE || ackRevision := 0;
    or skip;
    end either;
  }
}
}
*)

Init ==
    /\ now = 0
    /\ leaseOwner = None
    /\ fence = 0
    /\ expires = 0
    /\ tokenFence = [r \in Runners |-> 0]
    /\ tokenEpoch = [r \in Runners |-> 0]
    /\ authorityEpoch = 1
    /\ freezeGeneration = 0
    /\ frozen = FALSE
    /\ scopeRevision = 1
    /\ ackRevision = 0
    /\ barrier = FALSE
    /\ inFlight = 0
    /\ acceptedApplies = 0
    /\ refusedApplies = 0
    /\ recoveryApproved = FALSE

Authorized(r) ==
    /\ leaseOwner = r
    /\ tokenFence[r] = fence
    /\ tokenEpoch[r] = authorityEpoch
    /\ expires > now
    /\ ~frozen
    /\ ~barrier
    /\ ackRevision = scopeRevision

Acquire(r) ==
    /\ leaseOwner = None
    /\ ~frozen
    /\ fence < MaxFence
    /\ leaseOwner' = r
    /\ fence' = fence + 1
    /\ expires' = IF now < MaxTime THEN now + 1 ELSE now
    /\ tokenFence' = [tokenFence EXCEPT ![r] = fence + 1]
    /\ tokenEpoch' = [tokenEpoch EXCEPT ![r] = authorityEpoch]
    /\ ackRevision' = scopeRevision
    /\ UNCHANGED <<now, authorityEpoch, freezeGeneration, frozen,
                    scopeRevision, barrier, inFlight, acceptedApplies,
                    refusedApplies, recoveryApproved>>

Heartbeat(r) ==
    /\ leaseOwner = r
    /\ tokenFence[r] = fence
    /\ tokenEpoch[r] = authorityEpoch
    /\ expires > now
    /\ ~frozen
    /\ expires' = IF now < MaxTime THEN now + 1 ELSE now
    /\ UNCHANGED <<now, leaseOwner, fence, tokenFence, tokenEpoch,
                    authorityEpoch, freezeGeneration, frozen, scopeRevision,
                    ackRevision, barrier, inFlight, acceptedApplies,
                    refusedApplies, recoveryApproved>>

Tick ==
    /\ now < MaxTime
    /\ now' = now + 1
    /\ UNCHANGED <<leaseOwner, fence, expires, tokenFence, tokenEpoch,
                    authorityEpoch, freezeGeneration, frozen, scopeRevision,
                    ackRevision, barrier, inFlight, acceptedApplies,
                    refusedApplies, recoveryApproved>>

Reap ==
    /\ leaseOwner # None
    /\ expires <= now
    /\ inFlight = 0
    /\ leaseOwner' = None
    /\ ackRevision' = 0
    /\ barrier' = FALSE
    /\ UNCHANGED <<now, fence, expires, tokenFence, tokenEpoch,
                    authorityEpoch, freezeGeneration, frozen, scopeRevision,
                    inFlight, acceptedApplies, refusedApplies,
                    recoveryApproved>>

Terminated ==
    /\ now = MaxTime
    /\ leaseOwner = None
    /\ inFlight = 0
    /\ ~recoveryApproved

StutterDone ==
    /\ Terminated
    /\ UNCHANGED vars

EnterBarrier(r) ==
    /\ Authorized(r)
    /\ inFlight = 0
    /\ barrier' = TRUE
    /\ UNCHANGED <<now, leaseOwner, fence, expires, tokenFence, tokenEpoch,
                    authorityEpoch, freezeGeneration, frozen, scopeRevision,
                    ackRevision, inFlight, acceptedApplies, refusedApplies,
                    recoveryApproved>>

AppendScope(r) ==
    /\ leaseOwner = r
    /\ barrier
    /\ inFlight = 0
    /\ scopeRevision < MaxScope
    /\ scopeRevision' = scopeRevision + 1
    /\ UNCHANGED <<now, leaseOwner, fence, expires, tokenFence, tokenEpoch,
                    authorityEpoch, freezeGeneration, frozen, ackRevision,
                    barrier, inFlight, acceptedApplies, refusedApplies,
                    recoveryApproved>>

AcknowledgeScope(r) ==
    /\ leaseOwner = r
    /\ barrier
    /\ ackRevision' = scopeRevision
    /\ UNCHANGED <<now, leaseOwner, fence, expires, tokenFence, tokenEpoch,
                    authorityEpoch, freezeGeneration, frozen, scopeRevision,
                    barrier, inFlight, acceptedApplies, refusedApplies,
                    recoveryApproved>>

Resume(r) ==
    /\ leaseOwner = r
    /\ barrier
    /\ ackRevision = scopeRevision
    /\ barrier' = FALSE
    /\ UNCHANGED <<now, leaseOwner, fence, expires, tokenFence, tokenEpoch,
                    authorityEpoch, freezeGeneration, frozen, scopeRevision,
                    ackRevision, inFlight, acceptedApplies, refusedApplies,
                    recoveryApproved>>

TryApply(r) ==
    /\ inFlight = 0
    /\ acceptedApplies + refusedApplies < MaxOps
    /\ IF Authorized(r)
          THEN /\ inFlight' = 1
               /\ acceptedApplies' = acceptedApplies + 1
               /\ refusedApplies' = refusedApplies
          ELSE /\ inFlight' = 0
               /\ acceptedApplies' = acceptedApplies
               /\ refusedApplies' = refusedApplies + 1
    /\ UNCHANGED <<now, leaseOwner, fence, expires, tokenFence, tokenEpoch,
                    authorityEpoch, freezeGeneration, frozen, scopeRevision,
                    ackRevision, barrier, recoveryApproved>>

CommitApply(r) ==
    /\ inFlight = 1
    /\ Authorized(r)
    /\ inFlight' = 0
    /\ UNCHANGED <<now, leaseOwner, fence, expires, tokenFence, tokenEpoch,
                    authorityEpoch, freezeGeneration, frozen, scopeRevision,
                    ackRevision, barrier, acceptedApplies, refusedApplies,
                    recoveryApproved>>

AbortInvalidApply ==
    /\ inFlight = 1
    /\ (expires <= now \/ frozen \/ barrier \/ leaseOwner = None)
    /\ inFlight' = 0
    /\ UNCHANGED <<now, leaseOwner, fence, expires, tokenFence, tokenEpoch,
                    authorityEpoch, freezeGeneration, frozen, scopeRevision,
                    ackRevision, barrier, acceptedApplies, refusedApplies,
                    recoveryApproved>>

Freeze ==
    /\ ~frozen
    /\ freezeGeneration < MaxFreeze
    /\ frozen' = TRUE
    /\ freezeGeneration' = freezeGeneration + 1
    /\ UNCHANGED <<now, leaseOwner, fence, expires, tokenFence, tokenEpoch,
                    authorityEpoch, scopeRevision, ackRevision, barrier,
                    inFlight, acceptedApplies, refusedApplies,
                    recoveryApproved>>

Restore ==
    /\ inFlight = 0
    /\ authorityEpoch < MaxEpoch
    /\ freezeGeneration < MaxFreeze
    /\ authorityEpoch' = authorityEpoch + 1
    /\ leaseOwner' = None
    /\ frozen' = TRUE
    /\ freezeGeneration' = freezeGeneration + 1
    /\ ackRevision' = 0
    /\ barrier' = FALSE
    /\ recoveryApproved' = FALSE
    /\ UNCHANGED <<now, fence, expires, tokenFence, tokenEpoch,
                    scopeRevision, inFlight, acceptedApplies, refusedApplies>>

ApproveRecovery ==
    /\ frozen
    /\ leaseOwner = None
    /\ ~recoveryApproved
    /\ recoveryApproved' = TRUE
    /\ UNCHANGED <<now, leaseOwner, fence, expires, tokenFence, tokenEpoch,
                    authorityEpoch, freezeGeneration, frozen, scopeRevision,
                    ackRevision, barrier, inFlight, acceptedApplies,
                    refusedApplies>>

RecoverActive ==
    /\ frozen
    /\ leaseOwner = None
    /\ recoveryApproved
    /\ frozen' = FALSE
    /\ recoveryApproved' = FALSE
    /\ UNCHANGED <<now, leaseOwner, fence, expires, tokenFence, tokenEpoch,
                    authorityEpoch, freezeGeneration, scopeRevision,
                    ackRevision, barrier, inFlight, acceptedApplies,
                    refusedApplies>>

Next ==
    \/ Tick
    \/ Reap
    \/ StutterDone
    \/ Freeze
    \/ Restore
    \/ ApproveRecovery
    \/ RecoverActive
    \/ AbortInvalidApply
    \/ \E r \in Runners:
        Acquire(r) \/ Heartbeat(r) \/ EnterBarrier(r) \/ AppendScope(r)
        \/ AcknowledgeScope(r) \/ Resume(r) \/ TryApply(r) \/ CommitApply(r)

TypeOK ==
    /\ now \in 0..MaxTime
    /\ leaseOwner \in Runners \cup {None}
    /\ fence \in 0..MaxFence
    /\ expires \in 0..MaxTime
    /\ tokenFence \in [Runners -> 0..MaxFence]
    /\ tokenEpoch \in [Runners -> Nat]
    /\ authorityEpoch \in 1..MaxEpoch
    /\ freezeGeneration \in 0..MaxFreeze
    /\ frozen \in BOOLEAN
    /\ scopeRevision \in 1..MaxScope
    /\ ackRevision \in 0..MaxScope
    /\ barrier \in BOOLEAN
    /\ inFlight \in 0..1
    /\ acceptedApplies \in 0..MaxOps
    /\ refusedApplies \in 0..MaxOps
    /\ recoveryApproved \in BOOLEAN

BarrierHasNoInFlightApply == barrier => inFlight = 0
RestoredAuthorityNeedsFreshAcquire == leaseOwner = None => inFlight = 0
RecoveryApprovalIsScoped == recoveryApproved => frozen /\ leaseOwner = None

\* Reaping is scheduler progress, not a promise that time itself advances.
\* Once a deadline is already due, weak fairness first aborts any invalid
\* in-flight apply and then reaps the lease. A restore may also invalidate the
\* lease and satisfy the property without a Reap step.
ExpiredLeaseEventuallyReaped ==
    (leaseOwner # None /\ expires <= now) ~> (leaseOwner = None)

\* Approval is an explicit external action and receives no fairness assumption.
\* Once approval is durable, it cannot remain pending forever. Internal
\* activation consumes it under weak fairness; a newer restore may instead
\* revoke it and require fresh approval for the new authority epoch.
RecoveryApprovalEventuallySettles ==
    recoveryApproved ~> ~recoveryApproved

Spec == Init /\ [][Next]_vars
        /\ WF_vars(AbortInvalidApply)
        /\ WF_vars(Reap)
        /\ WF_vars(RecoverActive)

=============================================================================
