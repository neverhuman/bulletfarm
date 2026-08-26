# Invariant registry contract

Status: Enforced
Owner: Bullet Farm policy
Last reviewed: 2026-08-24
Applies to: v1alpha1 Gate 0 and later-wave planning

`policy/v1alpha1/invariant-registry.json` is the machine source. Each atomic entry binds a stable ID,
legacy aliases, C1-C12/C6B or EV01-EV29 crosswalk IDs, exactly one T1/T2/T3 primary tier,
lifecycle, first applicable wave, plane, owner, target, proof, gate, failure handling, milestone,
version, and documentation anchor.

Validation refuses duplicate IDs, duplicate aliases, alias/ID collision, missing crosswalk controls,
unknown controls, incomplete traceability, an enforced entry without a target and proof, or a
planned entry assigned to Gate 0. Future controls are intentionally `planned`; they cannot satisfy
an earlier gate. The generated crosswalk is a review view, not a second source of truth.

This is completeness for entries declared in this registry only. The validator does not prove that every
runtime invariant or enforcement site has been inventoried, nor that every implementation site points back to
one entry. That whole-product bidirectional orphan inventory is explicit Wave-0 work.

Proof: `cargo test --locked -p bullet-wire --test policy_registry`.
