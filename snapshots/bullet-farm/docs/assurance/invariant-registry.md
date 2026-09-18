# Invariant registry contract

Status: Enforced
Owner: Bullet Farm policy
Last reviewed: 2026-08-26
Applies to: v1alpha1 Gate 0 and later-wave planning

`policy/v1alpha1/invariant-registry.json` is the machine source. Each atomic entry binds a stable ID,
legacy aliases, C1-C12/C6B or EV01-EV29 crosswalk IDs, exactly one T1/T2/T3 primary tier,
lifecycle, first applicable wave, plane, owner, target, proof, gate, failure handling, milestone,
version, and documentation anchor.

Validation refuses duplicate IDs, duplicate aliases, alias/ID collision, missing crosswalk controls,
unknown controls, incomplete traceability, an enforced entry without a target and proof, or a
planned entry assigned to Gate 0. Future controls are intentionally `planned`; they cannot satisfy
an earlier gate. The generated crosswalk is a review view, not a second source of truth.

L-36 bound three authority/lease/fence rows to named L-32 identities and moved
only those rows to `enforced`: `BF-CTL-0C5` (`missed_heartbeat_is_not_pass`),
`BF-EV-21` (`lost_effect_response_is_not_pass`), and `BF-EV-28`
(`stale_fence_is_not_pass`). Proof for those rows is `bash ops/ci/faults.sh`.
Remaining planned rows stay planned; this is not 51/51.

This is completeness for entries declared in this registry only. The validator does not prove that every
runtime invariant or enforcement site has been inventoried, nor that every implementation site points back to
one entry. That whole-product bidirectional orphan inventory is explicit Wave-0 work.

Proof: `cargo test --locked -p bullet-wire --test policy_registry`.
