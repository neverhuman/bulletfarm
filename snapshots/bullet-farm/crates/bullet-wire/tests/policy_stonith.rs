//! A7 STONITH inequality as a policy invariant (WI-34): the Kernel runner's
//! self-kill budget is 4/5 of the admitted lease TTL, so every admissible
//! maximum must leave both that budget and the remaining grace strictly inside
//! the TTL. A zero maximum is the one violating configuration and is refused
//! as `UNSAFE_POLICY` with a dedicated reason, checked after the immutable
//! conservatism set and before the live-admission rule.

use std::{fs, path::PathBuf};

use bullet_wire::{PolicySnapshotV1, decode_canonical};

const COMMITTED_POLICY: &str = "policy/v1alpha1/policy.json";
const LIVE_FIXTURE: &str = "crates/bullet-wire/tests/fixtures/policy-v1alpha2-live-enabled.json";
const STONITH_REASON: &str = "self-kill grace must be strictly less than lease TTL";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap()
        .to_path_buf()
}

fn policy(path: &str) -> PolicySnapshotV1 {
    decode_canonical(&fs::read(root().join(path)).unwrap()).unwrap()
}

#[test]
fn committed_maximum_and_every_smaller_maximum_satisfy_the_inequality() {
    let committed = policy(COMMITTED_POLICY);
    assert_eq!(committed.budget_policy.maximum_lease_ttl_seconds, 15);
    committed.validate().unwrap();
    for maximum in 1..=15 {
        let mut candidate = policy(COMMITTED_POLICY);
        candidate.budget_policy.maximum_lease_ttl_seconds = maximum;
        candidate.validate().unwrap();
    }
}

#[test]
fn zero_maximum_is_unsafe_policy_with_the_stonith_reason_in_both_versions() {
    for (name, path) in [("v1alpha1", COMMITTED_POLICY), ("v1alpha2", LIVE_FIXTURE)] {
        let mut zero = policy(path);
        zero.budget_policy.maximum_lease_ttl_seconds = 0;
        let error = zero.validate().unwrap_err();
        assert_eq!(error.code(), "UNSAFE_POLICY", "{name}");
        assert_eq!(error.reason(), STONITH_REASON, "{name}");
    }
}

#[test]
fn conservatism_set_is_checked_before_the_stonith_rule() {
    let mut high = policy(COMMITTED_POLICY);
    high.budget_policy.maximum_lease_ttl_seconds = 16;
    let error = high.validate().unwrap_err();
    assert_eq!(error.code(), "UNSAFE_POLICY");
    assert!(error.reason().starts_with("v1alpha1"), "{}", error.reason());

    let mut stacked = policy(COMMITTED_POLICY);
    stacked.budget_policy.maximum_lease_ttl_seconds = 0;
    stacked.route_policy.evolutionary_authority = true;
    let error = stacked.validate().unwrap_err();
    assert_eq!(error.code(), "UNSAFE_POLICY");
    assert!(error.reason().starts_with("v1alpha1"), "{}", error.reason());
}
