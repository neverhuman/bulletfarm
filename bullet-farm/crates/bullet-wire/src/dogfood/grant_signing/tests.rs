use serde_json::json;

use super::*;
use crate::{
    AuthorityVerificationKey, LaunchGrantClaims, LaunchGrantExpectation, LaunchProvider,
    SignedLaunchGrant, WireError, decode_canonical, decode_canonical_value,
};

const SECRET_KEY: [u8; 64] = [
    180, 203, 251, 67, 223, 76, 226, 16, 114, 125, 149, 62, 74, 113, 51, 7, 250, 25, 187, 125, 159,
    133, 4, 20, 56, 217, 225, 27, 148, 42, 55, 116, 30, 185, 219, 187, 188, 4, 124, 3, 253, 112,
    96, 78, 0, 113, 240, 152, 126, 22, 178, 139, 117, 114, 37, 193, 31, 0, 65, 93, 14, 32, 177,
    162,
];
const PUBLIC_KEY: [u8; 32] = [
    30, 185, 219, 187, 188, 4, 124, 3, 253, 112, 96, 78, 0, 113, 240, 152, 126, 22, 178, 139, 117,
    114, 37, 193, 31, 0, 65, 93, 14, 32, 177, 162,
];
use super::test_support::{EXPIRES_AT, ISSUER, KEY_ID, NOT_BEFORE, fixture, providers};

fn signer() -> DogfoodLaunchSigningKey {
    DogfoodLaunchSigningKey::from_bytes(ISSUER, KEY_ID, &SECRET_KEY).unwrap()
}

fn verifier() -> DogfoodLaunchVerificationKey {
    DogfoodLaunchVerificationKey::from_bytes(ISSUER, KEY_ID, &PUBLIC_KEY).unwrap()
}

fn verify(
    claims: &DogfoodLaunchGrantClaimsV1,
    intent: &DogfoodReadOnlyIntentV1,
    enrollment: &ProviderEnrollmentClaimsV2,
    now: u64,
) -> Result<DogfoodLaunchGrantClaimsV1, WireError> {
    signer()
        .sign(claims)?
        .verify(&verifier(), intent, enrollment, now)
}

fn refusal<T>(result: Result<T, WireError>, code: &'static str) -> WireError {
    let error = result.err().unwrap_or_else(|| panic!("expected {code}"));
    assert_eq!(error.code(), code, "{error}");
    error
}

fn rebind(
    enrollment: &ProviderEnrollmentClaimsV2,
    intent: &mut DogfoodReadOnlyIntentV1,
    claims: &mut DogfoodLaunchGrantClaimsV1,
) {
    intent.subject.provider.provider_enrollment_id = enrollment.enrollment_id().unwrap();
    claims.intent_id = intent.intent_id().unwrap();
    claims.subject = intent.subject.clone();
}

fn raw_signed(
    payload: &[u8],
    purpose: &str,
    assertion: &[u8],
    secret: &[u8],
) -> SignedDogfoodLaunchGrantV1 {
    let carrier = PurposeSeparatedPasetoSigningKey::from_bytes(ISSUER, KEY_ID, secret).unwrap();
    let footer = canonical_json(&json!({
        "issuer": ISSUER,
        "key_id": KEY_ID,
        "purpose": purpose,
        "schema_version": DOGFOOD_SCHEMA_VERSION,
    }))
    .unwrap();
    SignedDogfoodLaunchGrantV1 {
        schema_version: DOGFOOD_SCHEMA_VERSION.to_owned(),
        issuer: ISSUER.to_owned(),
        key_id: KEY_ID.to_owned(),
        paseto: carrier.sign(payload, &footer, assertion).unwrap(),
    }
}

fn verify_raw(
    payload: &[u8],
    purpose: &str,
    assertion: &[u8],
) -> Result<DogfoodLaunchGrantClaimsV1, WireError> {
    let (enrollment, intent, _) = fixture(LaunchProvider::Cursor);
    raw_signed(payload, purpose, assertion, &SECRET_KEY).verify(
        &verifier(),
        &intent,
        &enrollment,
        NOT_BEFORE,
    )
}

#[test]
fn all_four_provider_grants_round_trip_with_exact_domains_and_footer() {
    assert_eq!(
        DOGFOOD_LAUNCH_GRANT_SIGNING_PURPOSE,
        "dogfood-launch-signing"
    );
    assert_eq!(
        DOGFOOD_LAUNCH_GRANT_IMPLICIT_ASSERTION,
        b"bullet-farm.dogfood-launch-grant.v1alpha1"
    );
    assert_eq!(
        DOGFOOD_LAUNCH_GRANT_ENVELOPE_DOMAIN,
        "authority.dogfood-launch-grant-envelope.v1alpha1"
    );
    assert_eq!(MAX_DOGFOOD_LAUNCH_GRANT_TOKEN_BYTES, 32_768);
    assert_eq!(
        canonical_json(&footer(ISSUER, KEY_ID)).unwrap(),
        br#"{"issuer":"kernel.example","key_id":"dogfood-launch-alpha","purpose":"dogfood-launch-signing","schema_version":"v1alpha1"}"#
    );
    let mut envelope_digests = Vec::new();
    for (provider, protocol) in providers() {
        let (enrollment, intent, claims) = fixture(provider);
        assert_eq!(enrollment.protocol, protocol);
        let signed = signer().sign(&claims).unwrap();
        assert_eq!(signer().sign(&claims).unwrap(), signed);
        assert_eq!(
            signed
                .verify(&verifier(), &intent, &enrollment, NOT_BEFORE)
                .unwrap(),
            claims
        );
        assert_eq!(
            signed.digest().unwrap(),
            hash_framed_bytes(
                DOGFOOD_LAUNCH_GRANT_ENVELOPE_DOMAIN,
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
        assert_eq!(
            raw_signed(
                &canonical_json(&claims).unwrap(),
                DOGFOOD_LAUNCH_GRANT_SIGNING_PURPOSE,
                DOGFOOD_LAUNCH_GRANT_IMPLICIT_ASSERTION,
                &SECRET_KEY,
            ),
            signed
        );
        envelope_digests.push(signed.digest().unwrap().to_string());
        let mut changed = claims.clone();
        changed.grant_nonce = Blake3Digest::from_bytes([90; 32]);
        assert_ne!(claims.grant_id().unwrap(), changed.grant_id().unwrap());
    }
    assert_eq!(
        envelope_digests,
        [
            "a8ccf38495c8d7a8732c98afa39f6fbdb2522e00ad169f7a5477107a7dbdc86a",
            "0df7d27eef33eba2af5ec7f2fc57996483bfecf00b9c342b20fef1d58b22ed9a",
            "ab7d1e3ebb0db59ee365b0ce6ee63a0b37b0442c01dfe4d6ba2f8630ddd55a00",
            "945273f6187584989ddb405b6e9544e8969da29cc59b7d3d5c8f056ce438a11c",
        ]
    );
    let (mut enrollment, intent, mut claims) = fixture(LaunchProvider::Agy);
    let valid_claims = claims.clone();
    claims.subject.execution.attempt_fence += 1;
    refusal(
        verify(&claims, &intent, &enrollment, NOT_BEFORE),
        "DOGFOOD_GRANT_SUBJECT_MISMATCH",
    );
    enrollment.enrollment_generation += 1;
    refusal(
        verify(&valid_claims, &intent, &enrollment, NOT_BEFORE),
        "PROVIDER_ENROLLMENT_SUBJECT_MISMATCH",
    );
}

#[test]
fn trusted_window_and_enrollment_boundaries_are_exact() {
    let (enrollment, intent, claims) = fixture(LaunchProvider::Claude);
    let signed = signer().sign(&claims).unwrap();
    refusal(
        signed.verify(&verifier(), &intent, &enrollment, NOT_BEFORE - 1),
        "DOGFOOD_GRANT_NOT_YET_VALID",
    );
    signed
        .verify(&verifier(), &intent, &enrollment, NOT_BEFORE)
        .unwrap();
    signed
        .verify(&verifier(), &intent, &enrollment, EXPIRES_AT - 1)
        .unwrap();
    refusal(
        signed.verify(&verifier(), &intent, &enrollment, EXPIRES_AT),
        "DOGFOOD_GRANT_EXPIRED",
    );

    let mut activated_late = enrollment.clone();
    let mut late_intent = intent.clone();
    let mut late_claims = claims.clone();
    activated_late.activates_at_unix_ms = NOT_BEFORE + 1;
    rebind(&activated_late, &mut late_intent, &mut late_claims);
    refusal(
        verify(&late_claims, &late_intent, &activated_late, NOT_BEFORE),
        "PROVIDER_ENROLLMENT_SUBJECT_MISMATCH",
    );

    let mut ended_early = enrollment.clone();
    let mut early_intent = intent.clone();
    let mut early_claims = claims.clone();
    ended_early.expires_at_unix_ms = EXPIRES_AT - 1;
    rebind(&ended_early, &mut early_intent, &mut early_claims);
    refusal(
        verify(&early_claims, &early_intent, &ended_early, NOT_BEFORE),
        "PROVIDER_ENROLLMENT_SUBJECT_MISMATCH",
    );
    ended_early.expires_at_unix_ms = EXPIRES_AT;
    rebind(&ended_early, &mut early_intent, &mut early_claims);
    verify(&early_claims, &early_intent, &ended_early, NOT_BEFORE).unwrap();

    let mut revoked = enrollment.clone();
    let mut revoked_intent = intent.clone();
    let mut revoked_claims = claims.clone();
    for revoked_at in [EXPIRES_AT - 1, EXPIRES_AT] {
        revoked.revoked_at_unix_ms = Some(revoked_at);
        rebind(&revoked, &mut revoked_intent, &mut revoked_claims);
        refusal(
            verify(&revoked_claims, &revoked_intent, &revoked, NOT_BEFORE),
            "PROVIDER_ENROLLMENT_SUBJECT_MISMATCH",
        );
    }
    revoked.revoked_at_unix_ms = Some(EXPIRES_AT + 1);
    rebind(&revoked, &mut revoked_intent, &mut revoked_claims);
    verify(&revoked_claims, &revoked_intent, &revoked, NOT_BEFORE).unwrap();

    let mut invalid = claims.clone();
    invalid.issued_at_unix_ms = NOT_BEFORE + 1;
    refusal(signer().sign(&invalid), "DOGFOOD_GRANT_INVALID");
    invalid = claims.clone();
    invalid.expires_at_unix_ms += 1;
    invalid.subject.deadline_unix_ms += 1;
    refusal(signer().sign(&invalid), "DOGFOOD_GRANT_INVALID");
    for expires_at in [NOT_BEFORE, NOT_BEFORE - 1] {
        invalid = claims.clone();
        invalid.expires_at_unix_ms = expires_at;
        refusal(signer().sign(&invalid), "DOGFOOD_GRANT_INVALID");
    }
    invalid = claims;
    invalid.subject.deadline_unix_ms = EXPIRES_AT - 1;
    refusal(signer().sign(&invalid), "DOGFOOD_GRANT_INVALID");
}

#[test]
fn key_envelope_and_signed_identity_hostiles_refuse_stably() {
    refusal(
        DogfoodLaunchSigningKey::from_bytes("", KEY_ID, &SECRET_KEY),
        "INVALID_DOGFOOD_GRANT_KEY",
    );
    refusal(
        DogfoodLaunchSigningKey::from_bytes(ISSUER, KEY_ID, &[0; 64]),
        "INVALID_DOGFOOD_GRANT_KEY",
    );
    refusal(
        DogfoodLaunchVerificationKey::from_bytes(ISSUER, KEY_ID, &[0; 32]),
        "INVALID_DOGFOOD_GRANT_KEY",
    );
    for material in [&SECRET_KEY[..63], &[1_u8; 65][..]] {
        refusal(
            DogfoodLaunchSigningKey::from_bytes(ISSUER, KEY_ID, material),
            "INVALID_DOGFOOD_GRANT_KEY",
        );
    }
    let long_identity = "x".repeat(129);
    for identity in ["bad key", long_identity.as_str()] {
        refusal(
            DogfoodLaunchVerificationKey::from_bytes(ISSUER, identity, &PUBLIC_KEY),
            "INVALID_DOGFOOD_GRANT_KEY",
        );
    }
    let public_hex = hex::encode(PUBLIC_KEY);
    assert!(
        PurposeSeparatedPasetoVerificationKey::from_lower_hex(ISSUER, KEY_ID, &public_hex)
            .is_some()
    );
    for invalid in ["0", &public_hex.to_uppercase(), &"g".repeat(64)] {
        assert!(
            PurposeSeparatedPasetoVerificationKey::from_lower_hex(ISSUER, KEY_ID, invalid)
                .is_none()
        );
    }
    let (enrollment, intent, claims) = fixture(LaunchProvider::Codex);
    let signed = signer().sign(&claims).unwrap();
    let wrong_material =
        hex::decode("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a").unwrap();
    let wrong_key =
        DogfoodLaunchVerificationKey::from_bytes(ISSUER, KEY_ID, &wrong_material).unwrap();
    refusal(
        signed.verify(&wrong_key, &intent, &enrollment, NOT_BEFORE),
        "DOGFOOD_GRANT_INVALID",
    );
    let other = DogfoodLaunchVerificationKey::from_bytes(ISSUER, "other-key", &PUBLIC_KEY).unwrap();
    refusal(
        signed.verify(&other, &intent, &enrollment, NOT_BEFORE),
        "DOGFOOD_GRANT_KEY_UNKNOWN",
    );
    let mut changed = signed.clone();
    changed.key_id = "other-key".to_owned();
    refusal(
        changed.verify(&verifier(), &intent, &enrollment, NOT_BEFORE),
        "DOGFOOD_GRANT_KEY_UNKNOWN",
    );
    changed = signed.clone();
    changed.schema_version = "v2".to_owned();
    refusal(
        changed.verify(&verifier(), &intent, &enrollment, NOT_BEFORE),
        "DOGFOOD_GRANT_INVALID",
    );
    changed = signed.clone();
    changed.paseto = changed.paseto.replacen("v4.public.", "v4.local.", 1);
    refusal(
        changed.verify(&verifier(), &intent, &enrollment, NOT_BEFORE),
        "DOGFOOD_GRANT_INVALID",
    );
    changed = signed.clone();
    let last = changed.paseto.pop().unwrap();
    changed.paseto.push(if last == 'A' { 'B' } else { 'A' });
    refusal(
        changed.verify(&verifier(), &intent, &enrollment, NOT_BEFORE),
        "DOGFOOD_GRANT_INVALID",
    );
    changed = signed.clone();
    changed.paseto.truncate(changed.paseto.len() - 8);
    refusal(
        changed.verify(&verifier(), &intent, &enrollment, NOT_BEFORE),
        "DOGFOOD_GRANT_INVALID",
    );
    changed = signed.clone();
    changed.paseto = format!(
        "v4.public.{}",
        "A".repeat(MAX_DOGFOOD_LAUNCH_GRANT_TOKEN_BYTES)
    );
    refusal(changed.digest(), "DOGFOOD_GRANT_INVALID");
    let mut wrong_claims = claims;
    wrong_claims.key_id = "other-key".to_owned();
    refusal(signer().sign(&wrong_claims), "DOGFOOD_GRANT_KEY_UNKNOWN");
    let mut envelope = serde_json::to_value(signed).unwrap();
    envelope["unknown"] = json!(true);
    assert!(serde_json::from_value::<SignedDogfoodLaunchGrantV1>(envelope).is_err());
}

#[test]
fn fixed_purpose_authentication_precedes_strict_claim_semantics() {
    let (_, _, claims) = fixture(LaunchProvider::Cursor);
    let payload = canonical_json(&claims).unwrap();
    for purpose in [
        "authority-signing",
        "mutation-permit-signing",
        "launch-grant-signing",
        "provider-enrollment-signing",
        "dogfood-run-attestation-signing",
        "release-signing",
    ] {
        refusal(
            verify_raw(&payload, purpose, DOGFOOD_LAUNCH_GRANT_IMPLICIT_ASSERTION),
            "DOGFOOD_GRANT_INVALID",
        );
    }
    for assertion in [
        b"bullet-farm.authority.v1alpha1".as_slice(),
        b"bullet-farm.mutation-permit.v1alpha1".as_slice(),
        b"bullet-farm.launch-grant.v1alpha1".as_slice(),
        b"bullet-farm.provider-enrollment.v2".as_slice(),
        b"bullet-farm.dogfood-run-attestation.v1alpha1".as_slice(),
        b"".as_slice(),
    ] {
        refusal(
            verify_raw(&payload, DOGFOOD_LAUNCH_GRANT_SIGNING_PURPOSE, assertion),
            "DOGFOOD_GRANT_INVALID",
        );
    }
    let malformed = [
        {
            let mut value = claims.clone();
            value.schema_version = "v2".to_owned();
            (value, "schema_version")
        },
        {
            let mut value = claims.clone();
            value.signing_purpose = "authority-signing".to_owned();
            (value, "signing_purpose")
        },
        {
            let mut value = claims.clone();
            value.claims_domain = "authority.claims.v1alpha1".to_owned();
            (value, "claims_domain")
        },
    ];
    for (value, reason) in &malformed {
        let error = refusal(
            verify_raw(
                &canonical_json(value).unwrap(),
                DOGFOOD_LAUNCH_GRANT_SIGNING_PURPOSE,
                DOGFOOD_LAUNCH_GRANT_IMPLICIT_ASSERTION,
            ),
            "DOGFOOD_GRANT_INVALID",
        );
        assert!(error.reason().contains(reason));
    }
    let error = refusal(
        verify_raw(
            &canonical_json(&malformed[1].0).unwrap(),
            DOGFOOD_LAUNCH_GRANT_SIGNING_PURPOSE,
            b"bullet-farm.authority.v1alpha1",
        ),
        "DOGFOOD_GRANT_INVALID",
    );
    assert!(error.reason().contains("PASETO"));

    let mut unknown = serde_json::to_value(&claims).unwrap();
    unknown["unknown"] = json!(true);
    let error = refusal(
        verify_raw(
            &canonical_json(&unknown).unwrap(),
            DOGFOOD_LAUNCH_GRANT_SIGNING_PURPOSE,
            DOGFOOD_LAUNCH_GRANT_IMPLICIT_ASSERTION,
        ),
        "DOGFOOD_GRANT_INVALID",
    );
    assert!(error.reason().contains("DOCUMENT_SCHEMA_INVALID"));
    let canonical = String::from_utf8(payload).unwrap();
    let duplicate = canonical.replacen(
        "\"audience\":\"dogfood-runner\",",
        "\"audience\":\"dogfood-runner\",\"audience\":\"dogfood-runner\",",
        1,
    );
    let error = refusal(
        verify_raw(
            duplicate.as_bytes(),
            DOGFOOD_LAUNCH_GRANT_SIGNING_PURPOSE,
            DOGFOOD_LAUNCH_GRANT_IMPLICIT_ASSERTION,
        ),
        "DOGFOOD_GRANT_INVALID",
    );
    assert!(error.reason().contains("DUPLICATE_JSON_KEY"));
    let error = refusal(
        verify_raw(
            &serde_json::to_vec_pretty(&claims).unwrap(),
            DOGFOOD_LAUNCH_GRANT_SIGNING_PURPOSE,
            DOGFOOD_LAUNCH_GRANT_IMPLICIT_ASSERTION,
        ),
        "DOGFOOD_GRANT_INVALID",
    );
    assert!(error.reason().contains("NON_CANONICAL_JSON"));
}

#[test]
fn live_and_dogfood_tokens_are_bidirectionally_non_interchangeable() {
    let (enrollment, intent, claims) = fixture(LaunchProvider::Claude);
    let golden = decode_canonical_value(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/canonical/launch-grant-golden.json"
    )))
    .unwrap();
    let live = serde_json::from_value::<SignedLaunchGrant>(golden["envelope"].clone()).unwrap();
    let as_dogfood =
        serde_json::from_value::<SignedDogfoodLaunchGrantV1>(serde_json::to_value(&live).unwrap())
            .unwrap();
    let dogfood_live_key =
        DogfoodLaunchVerificationKey::from_bytes(&live.issuer, &live.key_id, &PUBLIC_KEY).unwrap();
    refusal(
        as_dogfood.verify(&dogfood_live_key, &intent, &enrollment, NOT_BEFORE),
        "DOGFOOD_GRANT_INVALID",
    );

    let dogfood = signer().sign(&claims).unwrap();
    let as_live =
        serde_json::from_value::<SignedLaunchGrant>(serde_json::to_value(&dogfood).unwrap())
            .unwrap();
    let live_verifier = AuthorityVerificationKey::from_bytes(ISSUER, KEY_ID, &PUBLIC_KEY).unwrap();
    let live_claims = decode_canonical::<LaunchGrantClaims>(
        golden["claims_canonical_json"].as_str().unwrap().as_bytes(),
    )
    .unwrap();
    let expected = LaunchGrantExpectation {
        audience: live_claims.audience,
        lease: live_claims.lease_subject(),
        provider: live_claims.provider_subject(),
        policy_snapshot_digest: live_claims.policy_snapshot_digest,
    };
    refusal(
        as_live.verify(&live_verifier, &expected, NOT_BEFORE),
        "LAUNCH_GRANT_INVALID",
    );
}
