mod hostile;

use super::{test_support::DOGFOOD_PUBLIC_HEX, *};
use crate::{
    AuthorityAudience, Blake3Digest, CredentialProjectionProfileId, DogfoodProviderProtocolV1,
    KeyAlgorithmV1, KeyPurposeV1, LaunchProvider, PolicySnapshotV1, PrincipalId,
    ProviderEnrollmentId, ProviderProfileId, RuntimePassportId, WireError, canonical_json,
    decode_canonical, hash_framed_bytes, policy_snapshot_digest,
};

const POLICY: &[u8] = include_bytes!("../../../tests/fixtures/policy-v1alpha2-live-enabled.json");
const LIVE_GOLDEN: &[u8] =
    include_bytes!("../../../../../fixtures/canonical/launch-grant-golden.json");
const ISSUER: &str = "provider-enrollment-operator";
const KEY_ID: &str = "provider-enrollment-1";
const ENROLL_SECRET: &str = "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
const ENROLL_PUBLIC: &str = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
const ACT: u64 = 1_800_000_000_000;
const EXP: u64 = ACT + 10_000;

fn bytes(raw: &str) -> Vec<u8> {
    hex::decode(raw).unwrap()
}

fn digest(seed: u8) -> Blake3Digest {
    Blake3Digest::from_bytes([seed; 32])
}

fn refusal<T>(result: Result<T, WireError>, code: &'static str) -> WireError {
    let error = result.err().unwrap_or_else(|| panic!("expected {code}"));
    assert_eq!(error.code(), code, "{error}");
    error
}

fn signer() -> ProviderEnrollmentSigningKey {
    ProviderEnrollmentSigningKey::from_bytes(ISSUER, KEY_ID, &bytes(ENROLL_SECRET)).unwrap()
}

fn key_index(policy: &PolicySnapshotV1, purpose: KeyPurposeV1) -> usize {
    policy
        .issuer_keys
        .iter()
        .position(|key| key.key_purpose == purpose)
        .unwrap()
}

fn policy() -> PolicySnapshotV1 {
    let mut policy: PolicySnapshotV1 = decode_canonical(POLICY).unwrap();
    let base = policy.issuer_keys[key_index(&policy, KeyPurposeV1::AuthoritySigning)].clone();
    let make = |issuer: &str, key_id: &str, purpose, public: &str| {
        let mut key = base.clone();
        key.issuer = issuer.to_owned();
        key.key_id = key_id.to_owned();
        key.key_purpose = purpose;
        key.public_key = public.to_owned();
        key.audiences.clear();
        key
    };
    policy.issuer_keys.extend([
        make(
            ISSUER,
            KEY_ID,
            KeyPurposeV1::ProviderEnrollmentSigning,
            ENROLL_PUBLIC,
        ),
        make(
            "dogfood-operator",
            "dogfood-launch-1",
            KeyPurposeV1::DogfoodLaunchSigning,
            DOGFOOD_PUBLIC_HEX,
        ),
        make(
            "attestor.example",
            "dogfood-run-1",
            KeyPurposeV1::DogfoodRunAttestationSigning,
            &"33".repeat(32),
        ),
    ]);
    policy.validate().unwrap();
    policy
}

fn policy_digest(policy: &PolicySnapshotV1) -> Blake3Digest {
    policy_snapshot_digest(&canonical_json(policy).unwrap()).unwrap()
}

fn claims(provider: LaunchProvider, policy: &PolicySnapshotV1) -> ProviderEnrollmentClaimsV2 {
    ProviderEnrollmentClaimsV2 {
        schema_version: DOGFOOD_SCHEMA_VERSION.to_owned(),
        issuer: ISSUER.to_owned(),
        key_id: KEY_ID.to_owned(),
        signing_purpose: PROVIDER_ENROLLMENT_SIGNING_PURPOSE.to_owned(),
        claims_domain: PROVIDER_ENROLLMENT_CLAIMS_DOMAIN.to_owned(),
        provider,
        protocol: DogfoodProviderProtocolV1::required_for(provider),
        runtime_passport_id: RuntimePassportId::from_digest(digest(1)),
        provider_profile_id: ProviderProfileId::from_digest(digest(2)),
        service_identity_id: PrincipalId::from_digest(digest(3)),
        credential_projection_profile_id: CredentialProjectionProfileId::from_digest(digest(4)),
        runtime_version: "v1.2.3".to_owned(),
        enrollment_generation: 7,
        activates_at_unix_ms: ACT,
        expires_at_unix_ms: EXP,
        revoked_at_unix_ms: None,
        egress_policy_digest: digest(5),
        tool_policy_digest: digest(6),
        budget_policy_digest: digest(7),
        endpoint_observation_digest: digest(8),
        version_observation_digest: digest(9),
        profile_observation_digest: digest(10),
        policy_snapshot_digest: policy_digest(policy),
        policy_generation: policy.policy_generation,
    }
}

fn expected(claims: &ProviderEnrollmentClaimsV2) -> ProviderEnrollmentExpectationV2 {
    ProviderEnrollmentExpectationV2 {
        provider_enrollment_id: claims.enrollment_id().unwrap(),
        enrollment_generation: claims.enrollment_generation,
        policy_snapshot_digest: claims.policy_snapshot_digest,
        policy_generation: claims.policy_generation,
    }
}

fn verify(
    claims: &ProviderEnrollmentClaimsV2,
    policy: &PolicySnapshotV1,
    expected: &ProviderEnrollmentExpectationV2,
    now: u64,
) -> Result<ProviderEnrollmentClaimsV2, WireError> {
    signer().sign(claims)?.verify(policy, expected, now)
}

#[test]
fn four_providers_preserve_claim_bytes_ids_and_deterministic_envelopes() {
    assert_eq!(
        PROVIDER_ENROLLMENT_SIGNING_PURPOSE,
        "provider-enrollment-signing"
    );
    assert_eq!(
        PROVIDER_ENROLLMENT_CLAIMS_DOMAIN,
        "provider.enrollment-claims.v2"
    );
    assert_eq!(
        PROVIDER_ENROLLMENT_IMPLICIT_ASSERTION,
        b"bullet-farm.provider-enrollment.v2"
    );
    assert_eq!(
        PROVIDER_ENROLLMENT_ENVELOPE_DOMAIN,
        "authority.provider-enrollment-envelope.v2"
    );
    assert_eq!(
        canonical_json(&footer(ISSUER, KEY_ID)).unwrap(),
        br#"{"issuer":"provider-enrollment-operator","key_id":"provider-enrollment-1","purpose":"provider-enrollment-signing","schema_version":"v1alpha1"}"#
    );
    let policy = policy();
    let mut pins = Vec::new();
    for provider in [
        LaunchProvider::Claude,
        LaunchProvider::Codex,
        LaunchProvider::Cursor,
        LaunchProvider::Agy,
    ] {
        let claims = claims(provider, &policy);
        let canonical = canonical_json(&claims).unwrap();
        assert_eq!(
            decode_provider_enrollment_claims(&canonical).unwrap(),
            claims
        );
        let signed = signer().sign(&claims).unwrap();
        assert_eq!(signer().sign(&claims).unwrap(), signed);
        assert_eq!(
            signed.verify(&policy, &expected(&claims), ACT).unwrap(),
            claims
        );
        assert_eq!(
            signed.digest().unwrap(),
            hash_framed_bytes(
                PROVIDER_ENROLLMENT_ENVELOPE_DOMAIN,
                signed.paseto.as_bytes()
            )
            .unwrap()
        );
        assert_ne!(
            signed.digest().unwrap(),
            hash_framed_bytes(
                "authority.launch-grant-envelope.v1alpha1",
                signed.paseto.as_bytes()
            )
            .unwrap()
        );
        pins.push((
            claims.enrollment_id().unwrap().to_string(),
            signed.digest().unwrap().to_string(),
        ));
    }
    assert_eq!(
        pins,
        [
            (
                "pen_a3e8433f2577f0e2df59a9557497168e85c3a30d487becc36771b12c08ca5645",
                "e37a9002b4f23f89488db6aee27a7f3cbd9bc7037fc178e8c6bbc996da3e2c14",
            ),
            (
                "pen_7f6c2fa5428f53ccdc971c24387dfc383a807f00891763c162083a7a17421f9b",
                "9dacca0663230a631a3722518b47d923917156fd00617a5129d68e9f50a73f50",
            ),
            (
                "pen_8cb5789765e12bd3725b3b3d26a3634ba6aabc58852fca4b4177d6680a36be8a",
                "e8a530924590c20381495d2ab5d8455afbcc0ab9a175b9ec9554216970c2061e",
            ),
            (
                "pen_955cbfcf88be2e09c9c67a1a76d2e3610fef02c64eabd7975a4cfecc182a5e54",
                "6f23add6c7747de2e8763814c3adbdaad59de388ea0755a7124cf037c20eab45",
            ),
        ]
        .map(|(id, digest)| (id.to_owned(), digest.to_owned()))
    );
    assert_eq!(
        serde_json::to_string(&LaunchProvider::Agy).unwrap(),
        r#""agy""#
    );
    assert!(decode_canonical::<LaunchProvider>(br#""antigravity""#).is_err());
}

#[test]
fn policy_selection_is_structural_current_and_purpose_separated() {
    let policy = policy();
    let base_claims = claims(LaunchProvider::Claude, &policy);
    let signed = signer().sign(&base_claims).unwrap();
    let base_expected = expected(&base_claims);
    let enrollment = key_index(&policy, KeyPurposeV1::ProviderEnrollmentSigning);
    let code = |p: &PolicySnapshotV1, s: &SignedProviderEnrollmentV2, now| {
        s.verify(p, &base_expected, now).unwrap_err().code()
    };
    let mut malformed = policy.clone();
    malformed.issuer_keys[enrollment].public_key = "AA".repeat(32);
    assert_eq!(
        code(&malformed, &signed, ACT),
        "INVALID_PROVIDER_ENROLLMENT_PUBLIC_KEY"
    );
    let mut unsafe_policy = policy.clone();
    unsafe_policy.evidence_policy.unknown_satisfies_gate = true;
    assert_eq!(code(&unsafe_policy, &signed, ACT), "UNSAFE_POLICY");
    let mut reused = policy.clone();
    reused.issuer_keys[enrollment].public_key = reused.issuer_keys
        [key_index(&reused, KeyPurposeV1::AuthoritySigning)]
    .public_key
    .clone();
    assert_eq!(code(&reused, &signed, ACT), "SIGNER_KEY_MATERIAL_REUSED");
    for mutate in [
        (|p: &mut PolicySnapshotV1| {
            let i = key_index(p, KeyPurposeV1::ProviderEnrollmentSigning);
            p.issuer_keys[i].algorithm = KeyAlgorithmV1::SshEd25519;
        }) as fn(&mut PolicySnapshotV1),
        |p| {
            let i = key_index(p, KeyPurposeV1::ProviderEnrollmentSigning);
            p.issuer_keys[i].audiences = vec![AuthorityAudience::ProviderRunner];
        },
    ] {
        let mut bad = policy.clone();
        mutate(&mut bad);
        assert_eq!(
            code(&bad, &signed, ACT),
            "INVALID_PROVIDER_ENROLLMENT_PUBLIC_KEY"
        );
    }
    let mut unknown = signed.clone();
    unknown.key_id = "unknown-enrollment-key".to_owned();
    assert_eq!(
        code(&policy, &unknown, ACT),
        "PROVIDER_ENROLLMENT_SIGNER_KEY_UNKNOWN"
    );
    let mut removed = policy.clone();
    removed.issuer_keys.remove(enrollment);
    assert_eq!(
        code(&removed, &signed, ACT),
        "PROVIDER_ENROLLMENT_SIGNER_KEY_UNKNOWN"
    );
    for purpose in [
        KeyPurposeV1::AuthoritySigning,
        KeyPurposeV1::DogfoodLaunchSigning,
        KeyPurposeV1::DogfoodRunAttestationSigning,
        KeyPurposeV1::ReleaseSigning,
    ] {
        let key = &policy.issuer_keys[key_index(&policy, purpose)];
        let mut relabeled = signed.clone();
        relabeled.issuer = key.issuer.clone();
        relabeled.key_id = key.key_id.clone();
        assert_eq!(
            code(&policy, &relabeled, ACT),
            "PROVIDER_ENROLLMENT_SIGNER_KEY_WRONG_PURPOSE"
        );
    }
    assert_eq!(
        code(&policy, &signed, policy.activation_at_unix_ms - 1),
        "POLICY_NOT_ACTIVE"
    );
    assert_eq!(
        code(&policy, &signed, policy.expires_at_unix_ms),
        "POLICY_NOT_ACTIVE"
    );
    let mut inactive = policy.clone();
    inactive.issuer_keys[enrollment].revoked_at_unix_ms = Some(ACT);
    assert_eq!(
        code(&inactive, &signed, ACT),
        "PROVIDER_ENROLLMENT_SIGNER_KEY_INACTIVE"
    );
    let mut activates_late = policy.clone();
    activates_late.issuer_keys[enrollment].activates_at_unix_ms = ACT + 1;
    let rebound = claims(LaunchProvider::Claude, &activates_late);
    refusal(
        verify(&rebound, &activates_late, &expected(&rebound), ACT),
        "PROVIDER_ENROLLMENT_SIGNER_KEY_INACTIVE",
    );
    verify(&rebound, &activates_late, &expected(&rebound), ACT + 1).unwrap();
    let mut expires = policy.clone();
    expires.issuer_keys[enrollment].expires_at_unix_ms = ACT + 2;
    let rebound = claims(LaunchProvider::Claude, &expires);
    verify(&rebound, &expires, &expected(&rebound), ACT + 1).unwrap();
    refusal(
        verify(&rebound, &expires, &expected(&rebound), ACT + 2),
        "PROVIDER_ENROLLMENT_SIGNER_KEY_INACTIVE",
    );
    let mut nonoverlap = policy;
    let policy_expiry = nonoverlap.expires_at_unix_ms;
    let key = &mut nonoverlap.issuer_keys[enrollment];
    key.activates_at_unix_ms = policy_expiry;
    key.expires_at_unix_ms = policy_expiry + 1_000;
    key.retain_until_unix_ms = key.expires_at_unix_ms + 15_000;
    refusal(
        signed.verify(&nonoverlap, &base_expected, ACT),
        "PROVIDER_ENROLLMENT_SIGNER_KEY_INACTIVE",
    );
}

#[test]
fn generation_policy_id_and_every_aggregate_subject_are_exact() {
    let policy = policy();
    let base = claims(LaunchProvider::Claude, &policy);
    let expected = expected(&base);
    let mutations: [fn(&mut ProviderEnrollmentClaimsV2); 14] = [
        |v| {
            v.provider = LaunchProvider::Codex;
            v.protocol = DogfoodProviderProtocolV1::required_for(v.provider);
        },
        |v| v.runtime_passport_id = RuntimePassportId::from_digest(digest(20)),
        |v| v.provider_profile_id = ProviderProfileId::from_digest(digest(21)),
        |v| v.service_identity_id = PrincipalId::from_digest(digest(22)),
        |v| {
            v.credential_projection_profile_id =
                CredentialProjectionProfileId::from_digest(digest(23));
        },
        |v| v.runtime_version = "v9.9.9".to_owned(),
        |v| v.egress_policy_digest = digest(24),
        |v| v.tool_policy_digest = digest(25),
        |v| v.budget_policy_digest = digest(26),
        |v| v.endpoint_observation_digest = digest(27),
        |v| v.version_observation_digest = digest(28),
        |v| v.profile_observation_digest = digest(29),
        |v| v.policy_snapshot_digest = digest(30),
        |v| v.policy_generation += 1,
    ];
    for mutate in mutations {
        let mut changed = base.clone();
        mutate(&mut changed);
        refusal(
            verify(&changed, &policy, &expected, ACT),
            "PROVIDER_ENROLLMENT_SUBJECT_MISMATCH",
        );
    }
    let mut generation = base.clone();
    generation.enrollment_generation += 1;
    refusal(
        verify(&generation, &policy, &expected, ACT),
        "PROVIDER_ENROLLMENT_GENERATION_MISMATCH",
    );
    let mut wrong_generation = expected.clone();
    wrong_generation.enrollment_generation += 1;
    refusal(
        verify(&base, &policy, &wrong_generation, ACT),
        "PROVIDER_ENROLLMENT_GENERATION_MISMATCH",
    );
    for mutate in [
        (|e: &mut ProviderEnrollmentExpectationV2| {
            e.provider_enrollment_id = ProviderEnrollmentId::from_digest(digest(30));
        }) as fn(&mut ProviderEnrollmentExpectationV2),
        |e| e.policy_snapshot_digest = digest(31),
        |e| e.policy_generation += 1,
    ] {
        let mut changed = expected.clone();
        mutate(&mut changed);
        refusal(
            verify(&base, &policy, &changed, ACT),
            "PROVIDER_ENROLLMENT_SUBJECT_MISMATCH",
        );
    }
    let mut same_generation_other_body = policy.clone();
    same_generation_other_body
        .budget_policy
        .maximum_changed_paths -= 1;
    same_generation_other_body.validate().unwrap();
    refusal(
        verify(&base, &same_generation_other_body, &expected, ACT),
        "PROVIDER_ENROLLMENT_SUBJECT_MISMATCH",
    );
    let mut other_generation = policy;
    other_generation.policy_generation += 1;
    refusal(
        verify(&base, &other_generation, &expected, ACT),
        "PROVIDER_ENROLLMENT_SUBJECT_MISMATCH",
    );
}

#[test]
fn enrollment_time_is_half_open_and_revocation_wins_at_equal_expiry() {
    let policy = policy();
    let base = claims(LaunchProvider::Agy, &policy);
    let verify_at =
        |claims: &ProviderEnrollmentClaimsV2, now| verify(claims, &policy, &expected(claims), now);
    refusal(
        verify_at(&base, ACT - 1),
        "PROVIDER_ENROLLMENT_NOT_YET_VALID",
    );
    verify_at(&base, ACT).unwrap();
    verify_at(&base, EXP - 1).unwrap();
    refusal(verify_at(&base, EXP), "PROVIDER_ENROLLMENT_EXPIRED");
    for revoked in [ACT, ACT + 500, EXP] {
        let mut claims = base.clone();
        claims.revoked_at_unix_ms = Some(revoked);
        if revoked > ACT {
            verify_at(&claims, revoked - 1).unwrap();
        }
        refusal(verify_at(&claims, revoked), "PROVIDER_ENROLLMENT_REVOKED");
    }
    let mut key_first = policy;
    let enrollment = key_index(&key_first, KeyPurposeV1::ProviderEnrollmentSigning);
    key_first.issuer_keys[enrollment].revoked_at_unix_ms = Some(ACT);
    let mut revoked = base;
    revoked.revoked_at_unix_ms = Some(ACT);
    refusal(
        verify(&revoked, &key_first, &expected(&revoked), ACT),
        "PROVIDER_ENROLLMENT_SIGNER_KEY_INACTIVE",
    );
}
