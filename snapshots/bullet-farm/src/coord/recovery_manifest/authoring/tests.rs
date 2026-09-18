use ed25519_compact::{KeyPair, Seed};
use sha2::{Digest, Sha256};
use std::os::unix::fs::PermissionsExt;

use super::*;
use crate::coord::{
    generation::manifest::{
        ArtifactBinding, ByteRange, RelativeArtifactPath, Sha256Digest, TrustedClaimOutcomeCounts,
        TrustedProjectionInventory, TrustedRecordKindCounts,
    },
    model::{
        FrozenClaimSubject, RecoveryFileIdentityV1, RecoveryInspectionArtifactsV1,
        RecoveryInspectionSubjectV1, RecoverySourceInspectionV1,
    },
};

const BOOT_ID: &str = "00000000-0000-4000-8000-000000000001";

fn key_pair() -> KeyPair {
    KeyPair::from_seed(Seed::from_slice(&[23_u8; 32]).unwrap())
}

fn install(key_pair: &KeyPair) -> trust::TestPolicyGuard {
    let public_key = key_pair.pk.as_ref().try_into().unwrap();
    trust::install_test_policy(public_key)
}

fn binding(path: &str) -> ArtifactBinding {
    ArtifactBinding::new(
        RelativeArtifactPath::parse(path).unwrap(),
        1,
        Some(1),
        true,
        Sha256Digest::for_bytes(path.as_bytes()),
    )
    .unwrap()
}

fn source(path: &str, relative_path: &str) -> RecoverySourceInspectionV1 {
    RecoverySourceInspectionV1 {
        binding: binding(relative_path),
        identity: RecoveryFileIdentityV1 {
            path: path.to_owned(),
            device: 1,
            inode: 1,
            owner_uid: 1_000,
            owner_gid: 1_000,
            mode: 0o100400,
            link_count: 1,
            byte_length: 1,
            mtime_seconds: 1,
            mtime_nanoseconds: 0,
            ctime_seconds: 1,
            ctime_nanoseconds: 0,
        },
    }
}

fn inspection(incident_at_unix_ms: u64) -> RecoveryInspectionV1 {
    RecoveryInspectionV1::from_subject(RecoveryInspectionSubjectV1 {
        parent_generation: "legacy-v1".to_owned(),
        incident_at_unix_ms,
        trusted_record_count: 1,
        trusted_projection_inventory: TrustedProjectionInventory {
            record_kinds: TrustedRecordKindCounts {
                claim: 1,
                heartbeat: 0,
                handoff: 0,
                commit_receipt: 0,
                commit_receipt_correction: 0,
                commit_receipt_group: 0,
                commit_receipt_group_correction: 0,
            },
            claim_outcomes: TrustedClaimOutcomeCounts {
                total: 1,
                active: 1,
                expired: 0,
                handed_off_unreceipted: 0,
                receipted: 0,
            },
        },
        discarded_range: ByteRange {
            start_inclusive: 1,
            end_exclusive: 2,
        },
        ambiguous_tail_range: ByteRange {
            start_inclusive: 1,
            end_exclusive: 2,
        },
        ambiguous_tail_sha256: Sha256Digest::for_bytes(b"tail"),
        artifacts: RecoveryInspectionArtifactsV1 {
            trusted_prefix: binding("archive/trusted-prefix.jsonl"),
            interrupted_capture: source(
                "/evidence/interrupted",
                "archive/interrupted-observation.jsonl.partial",
            ),
            tainted_generation: source("/evidence/tainted", "archive/tainted-generation.jsonl"),
            frozen_live_source: source("/evidence/frozen", "archive/frozen-live-source.jsonl"),
        },
        trusted_state_blake3: format!("blake3:{}", "a".repeat(64)),
        frozen_claims: vec![FrozenClaimSubject {
            claim_id: format!("clm_{}", "b".repeat(64)),
            claim_blake3: format!("blake3:{}", "c".repeat(64)),
        }],
        post_prefix_inventory_blake3: format!("blake3:{}", "d".repeat(64)),
    })
    .unwrap()
}

fn provenance() -> RecoveryBootstrapProvenanceV1 {
    RecoveryBootstrapProvenanceV1::from_observations(
        "a".repeat(40),
        "b".repeat(40),
        format!("sha256:{}", "c".repeat(64)),
        format!("sha256:{}", "d".repeat(64)),
        vec![
            (
                "Cargo.lock".to_owned(),
                1,
                format!("sha256:{}", "d".repeat(64)),
            ),
            (
                "rust-toolchain.toml".to_owned(),
                1,
                format!("sha256:{}", "e".repeat(64)),
            ),
        ],
        ("rustc 1.95.0".to_owned(), "cargo 1.95.0".to_owned()),
        (1, format!("sha256:{}", "f".repeat(64))),
    )
    .unwrap()
}

fn input() -> RecoveryAuthorizationDraftInput {
    let policy = trust::installed_policy().unwrap();
    RecoveryAuthorizationDraftInput {
        decision: RecoveryAuthorizationDecisionV1::Approve,
        recovery_operator: policy.operator_identity,
        recovery_operator_uid: policy.operator_uid,
        reviewer_principal: policy.reviewer_principal,
        reviewer_fingerprint: policy.reviewer_fingerprint,
        policy_namespace: policy.namespace,
        decision_at_unix_ms: 11,
        authorized_at_unix_ms: 20,
        expires_at_unix_ms: 100,
        authority_boot_id: BOOT_ID.to_owned(),
        authority_time_namespace_device: 7,
        authority_time_namespace_inode: 9,
        authorized_at_boottime_ms: 200,
        expires_at_boottime_ms: 280,
        trusted_clock: TrustedRecoveryClockObservation {
            unix_ms: 50,
            boottime_ms: 230,
            boot_id: BOOT_ID.to_owned(),
            time_namespace_device: 7,
            time_namespace_inode: 9,
        },
    }
}

#[cfg(target_os = "linux")]
fn observed_input(validity_window_ms: u64) -> ObservedRecoveryAuthorizationDraftInput {
    let policy = trust::installed_policy().unwrap();
    ObservedRecoveryAuthorizationDraftInput {
        decision: RecoveryAuthorizationDecisionV1::Approve,
        recovery_operator: policy.operator_identity,
        recovery_operator_uid: policy.operator_uid,
        reviewer_principal: policy.reviewer_principal,
        reviewer_fingerprint: policy.reviewer_fingerprint,
        policy_namespace: policy.namespace,
        validity_window_ms,
    }
}

fn code<T>(result: Result<T, CoordError>) -> &'static str {
    match result {
        Ok(_) => panic!("hostile recovery authority unexpectedly succeeded"),
        Err(error) => error.code(),
    }
}

fn assert_invalid<T>(result: Result<T, CoordError>) {
    assert_eq!(code(result), "INVALID_RECOVERY_AUTHORIZATION");
}

#[test]
fn production_authoring_remains_policy_disabled() {
    let input = RecoveryAuthorizationDraftInput {
        decision: RecoveryAuthorizationDecisionV1::Approve,
        recovery_operator: "bullet-recovery-operator".to_owned(),
        recovery_operator_uid: 1_000,
        reviewer_principal: "bullet-recovery-reviewer".to_owned(),
        reviewer_fingerprint: format!("sha256:{}", "0".repeat(64)),
        policy_namespace: "bullet-family-coordinator-recovery-v1".to_owned(),
        decision_at_unix_ms: 11,
        authorized_at_unix_ms: 20,
        expires_at_unix_ms: 100,
        authority_boot_id: BOOT_ID.to_owned(),
        authority_time_namespace_device: 7,
        authority_time_namespace_inode: 9,
        authorized_at_boottime_ms: 200,
        expires_at_boottime_ms: 280,
        trusted_clock: TrustedRecoveryClockObservation {
            unix_ms: 50,
            boottime_ms: 230,
            boot_id: BOOT_ID.to_owned(),
            time_namespace_device: 7,
            time_namespace_inode: 9,
        },
    };
    assert_eq!(
        code(draft(&inspection(10), &provenance(), input)),
        "RECOVERY_POLICY_DISABLED"
    );
}

#[test]
fn draft_derives_subjects_and_refuses_foreign_policy_facts() {
    let keys = key_pair();
    let _guard = install(&keys);
    let inspection = inspection(10);
    let provenance = provenance();
    let authorization = draft(&inspection, &provenance, input()).unwrap();
    assert_eq!(authorization.inspection_id, inspection.inspection_id);
    assert_eq!(
        authorization.inspection_sha256,
        inspection.sealed_sha256().unwrap().as_str()
    );
    assert_eq!(
        authorization.bootstrap_provenance_sha256,
        trust::sealed_sha256(&provenance).unwrap()
    );

    let mut wrong = input();
    wrong.recovery_operator.push_str("-other");
    assert_invalid(draft(&inspection, &provenance, wrong));
    let mut wrong = input();
    wrong.recovery_operator_uid += 1;
    assert_invalid(draft(&inspection, &provenance, wrong));
    let mut wrong = input();
    wrong.reviewer_principal.push_str("-other");
    assert_invalid(draft(&inspection, &provenance, wrong));
    let mut wrong = input();
    wrong.reviewer_fingerprint = format!("sha256:{}", "0".repeat(64));
    assert_invalid(draft(&inspection, &provenance, wrong));
    let mut wrong = input();
    wrong.policy_namespace.push_str("-other");
    assert_invalid(draft(&inspection, &provenance, wrong));
}

#[test]
fn draft_requires_incident_window_and_exact_observed_clock() {
    let keys = key_pair();
    let _guard = install(&keys);
    let provenance = provenance();
    let mut wrong = input();
    wrong.decision_at_unix_ms = 10;
    assert_invalid(draft(&inspection(10), &provenance, wrong));
    let mut wrong = input();
    wrong.expires_at_boottime_ms += 1;
    assert_eq!(
        code(draft(&inspection(10), &provenance, wrong)),
        "INVALID_RECOVERY_MANIFEST_PRODUCTION"
    );
    let mut wrong = input();
    wrong.trusted_clock.boot_id = "00000000-0000-4000-8000-000000000002".to_owned();
    assert_eq!(
        code(draft(&inspection(10), &provenance, wrong)),
        "RECOVERY_AUTHORIZATION_BOOT_CHANGED"
    );
    let mut wrong = input();
    wrong.trusted_clock.time_namespace_inode += 1;
    assert_eq!(
        code(draft(&inspection(10), &provenance, wrong)),
        "RECOVERY_TIME_NAMESPACE_CHANGED"
    );
    let mut wrong = input();
    wrong.trusted_clock.unix_ms = wrong.authorized_at_unix_ms - 1;
    assert_eq!(
        code(draft(&inspection(10), &provenance, wrong)),
        "RECOVERY_AUTHORIZATION_NOT_YET_VALID"
    );
    let mut wrong = input();
    wrong.trusted_clock.boottime_ms = wrong.expires_at_boottime_ms;
    assert_eq!(
        code(draft(&inspection(10), &provenance, wrong)),
        "RECOVERY_AUTHORIZATION_EXPIRED"
    );
}

#[test]
fn exact_no_lf_message_and_verified_raw_signature_import_are_closed() {
    let keys = key_pair();
    let _guard = install(&keys);
    let inspection = inspection(10);
    let provenance = provenance();
    let authorization = draft(&inspection, &provenance, input()).unwrap();
    let canonical = bullet_wire::canonical_json(&authorization).unwrap();
    let message = signing_message(&authorization).unwrap();
    let domain = b"bullet-family.coord.recovery-authorization-signature.v1\0";
    assert_eq!(&message[..domain.len()], domain);
    assert_eq!(&message[domain.len()..], canonical);
    assert_ne!(message.last(), Some(&b'\n'));

    let raw = keys.sk.sign(&message, None);
    let signature = import_signature(&authorization, raw.as_ref()).unwrap();
    let mut canonical_line = canonical.clone();
    canonical_line.push(b'\n');
    assert_eq!(
        signature.authorization_sha256,
        format!("sha256:{:x}", Sha256::digest(&canonical_line))
    );
    assert_eq!(signature.signature_ed25519.len(), "ed25519:".len() + 128);

    let mut wrong_signature = raw.as_ref().to_vec();
    wrong_signature[0] ^= 1;
    assert_invalid(import_signature(&authorization, &wrong_signature));
    assert_invalid(import_signature(&authorization, &raw.as_ref()[..63]));

    let mut changed_input = input();
    changed_input.expires_at_unix_ms -= 1;
    changed_input.expires_at_boottime_ms -= 1;
    let changed = draft(&inspection, &provenance, changed_input).unwrap();
    assert_invalid(import_signature(&changed, raw.as_ref()));

    let mut message_with_lf = message;
    message_with_lf.push(b'\n');
    let wrong_domain_body = keys.sk.sign(&message_with_lf, None);
    assert_invalid(import_signature(&authorization, wrong_domain_body.as_ref()));
}

#[cfg(target_os = "linux")]
#[test]
fn observed_draft_derives_clock_fields_and_rechecks_before_publication() {
    let keys = key_pair();
    let _policy = install(&keys);
    let _clock = super::super::install_test_clock_pair(50, 230);
    let authorization = draft_observed(&inspection(10), &provenance(), observed_input(80)).unwrap();
    assert_eq!(authorization.decision_at_unix_ms, 50);
    assert_eq!(authorization.authorized_at_unix_ms, 50);
    assert_eq!(authorization.expires_at_unix_ms, 130);
    assert_eq!(authorization.authorized_at_boottime_ms, 230);
    assert_eq!(authorization.expires_at_boottime_ms, 310);
    assert_eq!(authorization.authority_time_namespace_device, 1);
    assert_eq!(authorization.authority_time_namespace_inode, 1);
    require_observed_current(&authorization).unwrap();

    super::super::set_test_clock(130, 310, (1, 1));
    assert_eq!(
        code(require_observed_current(&authorization)),
        "RECOVERY_AUTHORIZATION_EXPIRED"
    );
    super::super::set_test_clock(90, 270, (1, 2));
    assert_eq!(
        code(require_observed_current(&authorization)),
        "RECOVERY_TIME_NAMESPACE_CHANGED"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn observed_draft_refuses_invalid_windows_and_incident_ordering() {
    let keys = key_pair();
    let _policy = install(&keys);
    let _clock = super::super::install_test_clock_pair(50, 230);
    assert_eq!(
        code(draft_observed(
            &inspection(10),
            &provenance(),
            observed_input(0),
        )),
        "INVALID_RECOVERY_MANIFEST_PRODUCTION"
    );
    assert_eq!(
        code(draft_observed(
            &inspection(10),
            &provenance(),
            observed_input(24 * 60 * 60 * 1_000 + 1),
        )),
        "INVALID_RECOVERY_MANIFEST_PRODUCTION"
    );
    assert_invalid(draft_observed(
        &inspection(50),
        &provenance(),
        observed_input(80),
    ));

    super::super::set_test_clock(u64::MAX, u64::MAX, (1, 1));
    assert_invalid(draft_observed(
        &inspection(10),
        &provenance(),
        observed_input(1),
    ));
}

fn cli_options(pairs: &[(&str, String)]) -> Vec<String> {
    pairs
        .iter()
        .flat_map(|(name, value)| [format!("--{name}"), value.clone()])
        .collect()
}

#[cfg(target_os = "linux")]
#[test]
fn authorization_cli_is_closed_create_once_and_byte_exact() {
    let keys = key_pair();
    let _policy = install(&keys);
    let _clock = super::super::install_test_clock_pair(50, 230);
    let private = tempfile::tempdir().unwrap();
    std::fs::set_permissions(private.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let inspection_path = private.path().join("inspection.json");
    let provenance_path = private.path().join("provenance.json");
    let authorization_path = private.path().join("authorization.json");
    crate::coord::sealed::write(&inspection_path, &inspection(10)).unwrap();
    crate::coord::sealed::write(&provenance_path, &provenance()).unwrap();
    let input = observed_input(80);
    let draft = cli_options(&[
        ("inspection", inspection_path.display().to_string()),
        (
            "bootstrap-provenance",
            provenance_path.display().to_string(),
        ),
        ("decision", "APPROVE".to_owned()),
        ("recovery-operator", input.recovery_operator),
        (
            "recovery-operator-uid",
            input.recovery_operator_uid.to_string(),
        ),
        ("reviewer-principal", input.reviewer_principal),
        ("reviewer-fingerprint", input.reviewer_fingerprint),
        ("policy-namespace", input.policy_namespace),
        ("validity-window-ms", input.validity_window_ms.to_string()),
        ("output", authorization_path.display().to_string()),
    ]);
    for forbidden in ["authorized-at-unix-ms", "authority-boot-id", "private-key"] {
        let mut hostile = draft.clone();
        hostile.extend([format!("--{forbidden}"), "forged".to_owned()]);
        assert_eq!(
            code(crate::cli::test_recovery_action(
                "recovery-authorization-draft",
                &hostile,
            )),
            "UNKNOWN_OPTION"
        );
        assert!(!authorization_path.exists());
    }
    crate::cli::test_recovery_action("recovery-authorization-draft", &draft).unwrap();
    let authorization: RecoveryAuthorizationV1 =
        crate::coord::sealed::read(&authorization_path).unwrap();
    assert_eq!(authorization.authorized_at_unix_ms, 50);
    let original = std::fs::read(&authorization_path).unwrap();
    assert!(crate::cli::test_recovery_action("recovery-authorization-draft", &draft).is_err());
    assert_eq!(std::fs::read(&authorization_path).unwrap(), original);

    let message_path = private.path().join("message.bin");
    let message_options = cli_options(&[
        ("authorization", authorization_path.display().to_string()),
        ("output", message_path.display().to_string()),
    ]);
    crate::cli::test_recovery_action("recovery-authorization-message", &message_options).unwrap();
    let message = crate::coord::sealed::read_raw(&message_path, 1_024 * 1_024).unwrap();
    assert_eq!(message, signing_message(&authorization).unwrap());
    assert!(message.contains(&0) && message.last() != Some(&b'\n'));

    let raw = keys.sk.sign(&message, None);
    let raw_path = private.path().join("signature.ed25519");
    crate::coord::sealed::write_raw(&raw_path, raw.as_ref(), 64).unwrap();
    let signature_path = private.path().join("signature.json");
    crate::cli::test_recovery_action(
        "recovery-authorization-signature-import",
        &cli_options(&[
            ("authorization", authorization_path.display().to_string()),
            ("signature", raw_path.display().to_string()),
            ("output", signature_path.display().to_string()),
        ]),
    )
    .unwrap();
    let signature: RecoveryAuthorizationSignatureV1 =
        crate::coord::sealed::read(&signature_path).unwrap();
    assert_eq!(
        signature,
        import_signature(&authorization, raw.as_ref()).unwrap()
    );
    drop(_policy);
    assert_eq!(
        code(crate::cli::test_recovery_action(
            "recovery-authorization-message",
            &message_options,
        )),
        "RECOVERY_POLICY_DISABLED"
    );
}
