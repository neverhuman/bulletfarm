use super::*;

use std::{fs, os::unix::fs::PermissionsExt};

use ed25519_compact::{KeyPair, Seed};

use crate::coord::model::{RecoveryAuthorizationDecisionV1, RecoveryBootstrapSourceV1};
use crate::coord::{
    generation::recovery::adoption_fixture,
    recovery_manifest::{self, RecoveryInspectionCommand},
};

pub(in crate::coord) struct TestAuthority {
    pub(in crate::coord) authorization: RecoveryAuthorizationV1,
    pub(in crate::coord) signature: RecoveryAuthorizationSignatureV1,
    pub(in crate::coord) provenance: RecoveryBootstrapProvenanceV1,
    _guard: TestPolicyGuard,
}

pub(in crate::coord) fn test_authority(
    inspection: &RecoveryInspectionV1,
    authorized_at_unix_ms: u64,
    expires_at_unix_ms: u64,
) -> Result<TestAuthority, CoordError> {
    test_authority_with_decision(
        inspection,
        authorized_at_unix_ms,
        authorized_at_unix_ms,
        expires_at_unix_ms,
    )
}

pub(in crate::coord) fn test_authority_with_decision(
    inspection: &RecoveryInspectionV1,
    decision_at_unix_ms: u64,
    authorized_at_unix_ms: u64,
    expires_at_unix_ms: u64,
) -> Result<TestAuthority, CoordError> {
    let key_pair = KeyPair::from_seed(
        Seed::from_slice(&[42_u8; 32])
            .map_err(|_| invalid("cannot construct private test reviewer seed"))?,
    );
    let public_key: [u8; 32] = key_pair
        .pk
        .as_ref()
        .try_into()
        .map_err(|_| invalid("private test reviewer key has the wrong length"))?;
    let guard = install_test_policy(public_key);
    let (executable, _) = read_self_executable()?;
    let provenance = RecoveryBootstrapProvenanceV1 {
        kind: "bullet.coord.recovery-bootstrap-provenance.v1".to_owned(),
        schema_version: 1,
        bootstrap_commit_oid: "1".repeat(40),
        bootstrap_tree_oid: "2".repeat(40),
        archive_sha256: format!("sha256:{}", "3".repeat(64)),
        cargo_lock_sha256: format!("sha256:{}", "4".repeat(64)),
        source_files: vec![
            RecoveryBootstrapSourceV1 {
                path: "Cargo.lock".to_owned(),
                byte_length: 1,
                sha256: format!("sha256:{}", "4".repeat(64)),
            },
            RecoveryBootstrapSourceV1 {
                path: "rust-toolchain.toml".to_owned(),
                byte_length: 1,
                sha256: format!("sha256:{}", "5".repeat(64)),
            },
        ],
        rustc_version: "rustc private-test".to_owned(),
        cargo_version: "cargo private-test".to_owned(),
        executable_byte_length: executable.len() as u64,
        executable_sha256: format!("sha256:{:x}", Sha256::digest(&executable)),
    };
    let authorization = RecoveryAuthorizationV1 {
        kind: "bullet.coord.recovery-authorization.v1".to_owned(),
        schema_version: 1,
        decision: RecoveryAuthorizationDecisionV1::Approve,
        inspection_id: inspection.inspection_id.clone(),
        inspection_sha256: inspection.sealed_sha256()?.as_str().to_owned(),
        recovery_operator: OPERATOR_IDENTITY.to_owned(),
        recovery_operator_uid: rustix::process::geteuid().as_raw(),
        reviewer_principal: REVIEWER_PRINCIPAL.to_owned(),
        reviewer_fingerprint: fingerprint(&public_key),
        policy_namespace: POLICY_NAMESPACE.to_owned(),
        bootstrap_provenance_sha256: sealed_sha256(&provenance)?,
        decision_at_unix_ms,
        authorized_at_unix_ms,
        expires_at_unix_ms,
        authority_boot_id: TEST_AUTHORITY_BOOT_ID.to_owned(),
        authority_time_namespace_device: 1,
        authority_time_namespace_inode: 1,
        authorized_at_boottime_ms: authorized_at_unix_ms,
        expires_at_boottime_ms: expires_at_unix_ms,
    };
    let authorization_bytes = bullet_wire::canonical_json(&authorization).map_err(wire)?;
    let signature_value = key_pair
        .sk
        .sign(signing_message(&authorization_bytes), None);
    let signature = RecoveryAuthorizationSignatureV1 {
        kind: "bullet.coord.recovery-authorization-signature.v1".to_owned(),
        schema_version: 1,
        namespace: POLICY_NAMESPACE.to_owned(),
        reviewer_principal: REVIEWER_PRINCIPAL.to_owned(),
        reviewer_fingerprint: fingerprint(&public_key),
        authorization_sha256: sha256_line(&authorization_bytes),
        signature_ed25519: format!("ed25519:{}", encode_hex(signature_value.as_ref())),
    };
    Ok(TestAuthority {
        authorization,
        signature,
        provenance,
        _guard: guard,
    })
}

const TEST_AUTHORITY_BOOT_ID: &str = "00000000-0000-4000-8000-000000000001";

fn clock(unix_ms: u64, boottime_ms: u64) -> ClockObservation {
    ClockObservation {
        unix_ms,
        boottime_ms,
        boot_id: TEST_AUTHORITY_BOOT_ID.to_owned(),
        time_namespace_device: 1,
        time_namespace_inode: 1,
    }
}

#[test]
fn production_policy_is_disabled_without_the_offline_public_key() {
    let error = match installed_policy() {
        Ok(_) => panic!("production recovery policy unexpectedly enabled"),
        Err(error) => error,
    };
    assert_eq!(error.code(), "RECOVERY_POLICY_DISABLED");
}

#[test]
fn disabled_policy_refuses_before_clock_observation() {
    let (_fixture, inspection) = inspection_fixture();
    let authority = test_authority(&inspection, 10, 100).unwrap();
    let authorization = authority.authorization.clone();
    let signature = authority.signature.clone();
    let provenance = authority.provenance.clone();
    drop(authority);
    assert_eq!(
        code(verify_observed(
            &inspection,
            &authorization,
            &signature,
            &provenance,
            || panic!("disabled policy must refuse before observing the host clock"),
        )),
        "RECOVERY_POLICY_DISABLED"
    );
}

#[test]
fn test_policy_binds_the_supplied_public_key_fingerprint() {
    let public_key = [7_u8; 32];
    let _guard = install_test_policy(public_key);
    let policy = installed_policy().unwrap();
    assert_eq!(policy.reviewer_public_key, public_key);
    assert_eq!(policy.reviewer_fingerprint, fingerprint(&public_key));
    assert_eq!(
        decode_hex::<32>(&encode_hex(&public_key), "").unwrap(),
        public_key
    );
}

fn inspection_fixture() -> (
    adoption_fixture::AdoptionRecoveryFixture,
    RecoveryInspectionV1,
) {
    let fixture = adoption_fixture::fixture(&"1".repeat(40), &"2".repeat(40));
    fs::set_permissions(fixture.family.path(), fs::Permissions::from_mode(0o700)).unwrap();
    fs::set_permissions(&fixture.input.coord_dir, fs::Permissions::from_mode(0o700)).unwrap();
    fs::set_permissions(
        fixture.input.interrupted_capture.path.parent().unwrap(),
        fs::Permissions::from_mode(0o700),
    )
    .unwrap();
    let inspection = recovery_manifest::inspect(
        fixture.family.path(),
        &RecoveryInspectionCommand {
            interrupted_capture: fixture.input.interrupted_capture.path.clone(),
            tainted_generation: fixture.input.tainted_generation.path.clone(),
            frozen_live_source: fixture.input.frozen_live_source.path.clone(),
        },
    )
    .unwrap();
    (fixture, inspection)
}

fn code<T>(result: Result<T, CoordError>) -> &'static str {
    match result {
        Ok(_) => panic!("hostile recovery authority unexpectedly verified"),
        Err(error) => error.code(),
    }
}

#[test]
fn signed_authority_binds_exact_documents_key_and_actor_separation() {
    let (_fixture, inspection) = inspection_fixture();
    let authority = test_authority(&inspection, 10, 100).unwrap();
    verify(
        &inspection,
        &authority.authorization,
        &authority.signature,
        &authority.provenance,
        clock(20, 20),
    )
    .unwrap()
    .require_active(clock(20, 20))
    .unwrap();

    let mut changed_body = authority.authorization.clone();
    changed_body.expires_at_unix_ms -= 1;
    changed_body.expires_at_boottime_ms -= 1;
    assert_eq!(
        code(verify(
            &inspection,
            &changed_body,
            &authority.signature,
            &authority.provenance,
            clock(20, 20),
        )),
        "INVALID_RECOVERY_AUTHORIZATION"
    );

    let mut wrong_signature = authority.signature.clone();
    let last = wrong_signature.signature_ed25519.len() - 1;
    let replacement = if &wrong_signature.signature_ed25519[last..] == "0" {
        "1"
    } else {
        "0"
    };
    wrong_signature
        .signature_ed25519
        .replace_range(last.., replacement);
    assert_eq!(
        code(verify(
            &inspection,
            &authority.authorization,
            &wrong_signature,
            &authority.provenance,
            clock(20, 20),
        )),
        "INVALID_RECOVERY_AUTHORIZATION"
    );

    let mut same_actor = authority.authorization.clone();
    same_actor.reviewer_principal = same_actor.recovery_operator.clone();
    assert_eq!(
        code(verify(
            &inspection,
            &same_actor,
            &authority.signature,
            &authority.provenance,
            clock(20, 20),
        )),
        "INVALID_RECOVERY_AUTHORIZATION"
    );

    let mut changed_provenance = authority.provenance.clone();
    changed_provenance.archive_sha256 = format!("sha256:{}", "9".repeat(64));
    assert_eq!(
        code(verify(
            &inspection,
            &authority.authorization,
            &authority.signature,
            &changed_provenance,
            clock(20, 20),
        )),
        "INVALID_RECOVERY_AUTHORIZATION"
    );

    let mut contradictory_lock = authority.provenance.clone();
    contradictory_lock.cargo_lock_sha256 = format!("sha256:{}", "8".repeat(64));
    assert_eq!(
        code(contradictory_lock.validate()),
        "INVALID_RECOVERY_MANIFEST_PRODUCTION"
    );

    let _wrong_key = install_test_policy([7_u8; 32]);
    assert_eq!(
        code(verify(
            &inspection,
            &authority.authorization,
            &authority.signature,
            &authority.provenance,
            clock(20, 20),
        )),
        "INVALID_RECOVERY_AUTHORIZATION"
    );
}

#[test]
fn authorization_windows_are_closed_and_bounded() {
    let (_fixture, inspection) = inspection_fixture();
    let authority = test_authority(&inspection, 10, 100).unwrap();
    let expired = verify(
        &inspection,
        &authority.authorization,
        &authority.signature,
        &authority.provenance,
        clock(100, 100),
    )
    .unwrap();
    assert_eq!(
        code(expired.require_active(clock(100, 100))),
        "RECOVERY_AUTHORIZATION_EXPIRED"
    );
    expired.require_read_only_replay(clock(100, 100)).unwrap();

    let not_yet = verify(
        &inspection,
        &authority.authorization,
        &authority.signature,
        &authority.provenance,
        clock(9, 20),
    )
    .unwrap();
    assert_eq!(
        code(not_yet.require_active(clock(9, 20))),
        "RECOVERY_AUTHORIZATION_NOT_YET_VALID"
    );
    assert_eq!(
        code(not_yet.require_read_only_replay(clock(9, 20))),
        "RECOVERY_AUTHORIZATION_NOT_YET_VALID"
    );

    let boottime_not_yet = verify(
        &inspection,
        &authority.authorization,
        &authority.signature,
        &authority.provenance,
        clock(20, 9),
    )
    .unwrap();
    assert_eq!(
        code(boottime_not_yet.require_active(clock(20, 9))),
        "RECOVERY_AUTHORIZATION_NOT_YET_VALID"
    );
    assert_eq!(
        code(boottime_not_yet.require_read_only_replay(clock(20, 9))),
        "RECOVERY_AUTHORIZATION_NOT_YET_VALID"
    );

    let maximum = test_authority(
        &inspection,
        10,
        10_u64.checked_add(MAX_AUTHORIZATION_WINDOW_MS).unwrap(),
    )
    .unwrap();
    maximum.authorization.validate().unwrap();

    let mut overlong = maximum.authorization.clone();
    overlong.expires_at_unix_ms += 1;
    overlong.expires_at_boottime_ms += 1;
    assert_eq!(
        code(overlong.validate()),
        "INVALID_RECOVERY_MANIFEST_PRODUCTION"
    );

    let boot_expired = verify(
        &inspection,
        &authority.authorization,
        &authority.signature,
        &authority.provenance,
        clock(20, 100),
    )
    .unwrap();
    assert_eq!(
        code(boot_expired.require_active(clock(20, 100))),
        "RECOVERY_AUTHORIZATION_EXPIRED"
    );

    let unix_expired = verify(
        &inspection,
        &authority.authorization,
        &authority.signature,
        &authority.provenance,
        clock(100, 20),
    )
    .unwrap();
    assert_eq!(
        code(unix_expired.require_active(clock(100, 20))),
        "RECOVERY_AUTHORIZATION_EXPIRED"
    );
    unix_expired
        .require_read_only_replay(clock(100, 20))
        .unwrap();

    let mut wrong_boot = clock(20, 20);
    wrong_boot.boot_id = "00000000-0000-4000-8000-000000000002".to_owned();
    assert_eq!(
        code(verify(
            &inspection,
            &authority.authorization,
            &authority.signature,
            &authority.provenance,
            wrong_boot,
        )),
        "RECOVERY_AUTHORIZATION_BOOT_CHANGED"
    );
}

#[test]
fn renewed_execution_window_preserves_the_operator_decision() {
    let (_fixture, inspection) = inspection_fixture();
    let first = test_authority_with_decision(&inspection, 10, 10, 100).unwrap();
    let renewed = test_authority_with_decision(&inspection, 10, 101, 191).unwrap();
    let first_verified = verify(
        &inspection,
        &first.authorization,
        &first.signature,
        &first.provenance,
        clock(20, 20),
    )
    .unwrap();
    let renewed_verified = verify(
        &inspection,
        &renewed.authorization,
        &renewed.signature,
        &renewed.provenance,
        clock(110, 110),
    )
    .unwrap();
    assert_ne!(
        first.signature.authorization_sha256,
        renewed.signature.authorization_sha256
    );
    assert_eq!(
        first_verified.operator_decision_sha256,
        renewed_verified.operator_decision_sha256
    );
    assert_eq!(
        first_verified.decision_at_unix_ms,
        renewed_verified.decision_at_unix_ms
    );
}

#[test]
fn time_namespace_identity_refuses_offset_independent_lookalikes() {
    let (host_identity, _retained) = super::super::clock::open_time_namespace().unwrap();
    assert_ne!(host_identity.0, 0);
    assert_ne!(host_identity.1, 0);

    let wrong_namespace = fs::File::open("/proc/self/ns/uts").unwrap();
    let error = super::super::clock::validate_time_namespace_descriptor(&wrong_namespace)
        .expect_err("an NSFS descriptor for the wrong namespace type must be refused");
    assert_eq!(error.code(), "INVALID_RECOVERY_MANIFEST_PRODUCTION");

    assert!(super::super::clock::time_namespace_link_matches(
        b"time:[1]",
        1
    ));
    for wrong_type in [
        b"mnt:[1]".as_slice(),
        b"time_for_children:[1]".as_slice(),
        b"time:[2]".as_slice(),
        b"time:[01]".as_slice(),
    ] {
        assert!(!super::super::clock::time_namespace_link_matches(
            wrong_type, 1
        ));
    }
    let (_fixture, inspection) = inspection_fixture();
    let authority = test_authority(&inspection, 10, 100).unwrap();
    let verified = verify(
        &inspection,
        &authority.authorization,
        &authority.signature,
        &authority.provenance,
        clock(20, 20),
    )
    .unwrap();
    for (unix_ms, boottime_ms, device, inode) in [(20, 20, 1, 2), (0, 20, 2, 1), (20, 0, 1, 2)] {
        let mut lookalike = clock(unix_ms, boottime_ms);
        lookalike.time_namespace_device = device;
        lookalike.time_namespace_inode = inode;
        assert_eq!(
            code(verified.require_active(lookalike)),
            "RECOVERY_TIME_NAMESPACE_CHANGED"
        );
    }
}
