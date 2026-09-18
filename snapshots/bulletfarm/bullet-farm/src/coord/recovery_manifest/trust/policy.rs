#[cfg(test)]
use std::cell::RefCell;

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::coord::{
    CoordError,
    model::{RecoveryAuthorizationSignatureV1, RecoveryAuthorizationV1},
};

const POLICY_KIND: &str = "bullet.coord.recovery-trust-policy.v1";
pub(super) const POLICY_NAMESPACE: &str = "bullet-family-coordinator-recovery-v1";
pub(super) const REPLAY_CONTRACT_IDENTITY: &str = "bullet-family-coordinator-generation-replay-v1";
pub(super) const REPLAY_CONTRACT_VERSION: u32 = 1;
pub(super) const OPERATOR_IDENTITY: &str = "bullet-recovery-operator";
const OPERATOR_UID: u32 = 1_000;
pub(super) const REVIEWER_PRINCIPAL: &str = "bullet-recovery-reviewer";
const REVIEWER_FINGERPRINT: &str = "";
const REVIEWER_PUBLIC_KEY_HEX: &str = "";
pub(super) const MAX_AUTHORIZATION_WINDOW_MS: u64 = 24 * 60 * 60 * 1_000;

#[derive(Clone)]
pub(in crate::coord::recovery_manifest) struct InstalledPolicy {
    pub(in crate::coord::recovery_manifest) namespace: String,
    pub(in crate::coord::recovery_manifest) operator_identity: String,
    pub(in crate::coord::recovery_manifest) operator_uid: u32,
    pub(in crate::coord::recovery_manifest) reviewer_principal: String,
    pub(in crate::coord::recovery_manifest) reviewer_fingerprint: String,
    pub(in crate::coord::recovery_manifest) reviewer_public_key: [u8; 32],
}

impl InstalledPolicy {
    pub(in crate::coord::recovery_manifest) fn require_authorization_identity(
        &self,
        authorization: &RecoveryAuthorizationV1,
    ) -> Result<(), CoordError> {
        if authorization.recovery_operator != self.operator_identity
            || authorization.recovery_operator_uid != self.operator_uid
        {
            return Err(invalid(
                "recovery operator identity or effective UID is not admitted by policy",
            ));
        }
        if authorization.reviewer_principal != self.reviewer_principal
            || authorization.reviewer_fingerprint != self.reviewer_fingerprint
            || authorization.reviewer_principal == authorization.recovery_operator
            || authorization.policy_namespace != self.namespace
        {
            return Err(invalid(
                "reviewer identity, separation, fingerprint, or namespace is not admitted",
            ));
        }
        Ok(())
    }

    pub(in crate::coord::recovery_manifest) fn require_signature_identity(
        &self,
        signature: &RecoveryAuthorizationSignatureV1,
    ) -> Result<(), CoordError> {
        if signature.namespace != self.namespace
            || signature.reviewer_principal != self.reviewer_principal
            || signature.reviewer_fingerprint != self.reviewer_fingerprint
        {
            return Err(invalid(
                "detached signature does not name the installed reviewer policy",
            ));
        }
        Ok(())
    }
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct PolicyDigestSubject<'a> {
    kind: &'static str,
    schema_version: u32,
    namespace: &'a str,
    operator_identity: &'a str,
    operator_uid: u32,
    reviewer_principal: &'a str,
    reviewer_fingerprint: &'a str,
    reviewer_public_key_ed25519: String,
    replay_contract_identity: &'static str,
    replay_contract_version: u32,
    maximum_authorization_window_ms: u64,
    execution_clock_identity: &'static str,
    execution_clock_expectation_path: &'static str,
}

pub(in crate::coord::recovery_manifest) fn installed_policy() -> Result<InstalledPolicy, CoordError>
{
    #[cfg(test)]
    if let Some(policy) = TEST_POLICY.with(|value| value.borrow().clone()) {
        return Ok(policy);
    }
    if REVIEWER_PUBLIC_KEY_HEX.is_empty() || REVIEWER_FINGERPRINT.is_empty() {
        return Err(disabled(
            "the dedicated recovery reviewer public key and fingerprint are not installed",
        ));
    }
    let reviewer_public_key = super::decode_hex::<32>(REVIEWER_PUBLIC_KEY_HEX, "")?;
    let policy = InstalledPolicy {
        namespace: POLICY_NAMESPACE.to_owned(),
        operator_identity: OPERATOR_IDENTITY.to_owned(),
        operator_uid: OPERATOR_UID,
        reviewer_principal: REVIEWER_PRINCIPAL.to_owned(),
        reviewer_fingerprint: REVIEWER_FINGERPRINT.to_owned(),
        reviewer_public_key,
    };
    if fingerprint(&policy.reviewer_public_key) != policy.reviewer_fingerprint {
        return Err(disabled(
            "installed recovery reviewer fingerprint does not match its public key",
        ));
    }
    Ok(policy)
}

pub(in crate::coord::recovery_manifest) fn policy_sha256(
    policy: &InstalledPolicy,
) -> Result<String, CoordError> {
    let subject = PolicyDigestSubject {
        kind: POLICY_KIND,
        schema_version: 1,
        namespace: &policy.namespace,
        operator_identity: &policy.operator_identity,
        operator_uid: policy.operator_uid,
        reviewer_principal: &policy.reviewer_principal,
        reviewer_fingerprint: &policy.reviewer_fingerprint,
        reviewer_public_key_ed25519: super::encode_hex(&policy.reviewer_public_key),
        replay_contract_identity: REPLAY_CONTRACT_IDENTITY,
        replay_contract_version: REPLAY_CONTRACT_VERSION,
        maximum_authorization_window_ms: MAX_AUTHORIZATION_WINDOW_MS,
        execution_clock_identity: "linux-boot-id+retained-exact-time-nsfs+clock-boottime+root-run-tmpfs-noxdev-record.v1",
        execution_clock_expectation_path: super::super::clock::EXPECTATION_PATH,
    };
    let bytes = bullet_wire::canonical_json(&subject).map_err(super::wire)?;
    Ok(super::sha256_domain(
        b"bullet-family.coord.recovery-policy.v1\0",
        &bytes,
    ))
}

pub(super) fn fingerprint(public_key: &[u8; 32]) -> String {
    format!("sha256:{:x}", Sha256::digest(public_key))
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RECOVERY_AUTHORIZATION", reason)
}

fn disabled(reason: impl Into<String>) -> CoordError {
    CoordError::new("RECOVERY_POLICY_DISABLED", reason)
}

#[cfg(test)]
thread_local! {
    static TEST_POLICY: RefCell<Option<InstalledPolicy>> = const { RefCell::new(None) };
}

#[cfg(test)]
pub(in crate::coord) struct TestPolicyGuard(Option<InstalledPolicy>);

#[cfg(test)]
impl Drop for TestPolicyGuard {
    fn drop(&mut self) {
        TEST_POLICY.with(|value| value.replace(self.0.take()));
    }
}

#[cfg(test)]
pub(in crate::coord) fn install_test_policy(public_key: [u8; 32]) -> TestPolicyGuard {
    let policy = InstalledPolicy {
        namespace: POLICY_NAMESPACE.to_owned(),
        operator_identity: OPERATOR_IDENTITY.to_owned(),
        operator_uid: rustix::process::geteuid().as_raw(),
        reviewer_principal: REVIEWER_PRINCIPAL.to_owned(),
        reviewer_fingerprint: fingerprint(&public_key),
        reviewer_public_key: public_key,
    };
    let previous = TEST_POLICY.with(|value| value.replace(Some(policy)));
    TestPolicyGuard(previous)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_policy_remains_compiled_disabled() {
        assert!(REVIEWER_PUBLIC_KEY_HEX.is_empty());
        assert!(REVIEWER_FINGERPRINT.is_empty());
        let error = match installed_policy() {
            Ok(_) => panic!("production recovery policy unexpectedly enabled"),
            Err(error) => error,
        };
        assert_eq!(error.code(), "RECOVERY_POLICY_DISABLED");
    }

    #[test]
    fn test_policy_binds_exact_public_facts() {
        let key = [7_u8; 32];
        let _guard = install_test_policy(key);
        let policy = installed_policy().unwrap();
        assert_eq!(policy.namespace, POLICY_NAMESPACE);
        assert_eq!(policy.operator_identity, OPERATOR_IDENTITY);
        assert_eq!(policy.reviewer_principal, REVIEWER_PRINCIPAL);
        assert_eq!(policy.reviewer_public_key, key);
        assert_eq!(policy.reviewer_fingerprint, fingerprint(&key));
    }
}
