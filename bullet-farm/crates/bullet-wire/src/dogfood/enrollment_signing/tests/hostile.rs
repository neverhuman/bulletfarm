use serde_json::json;

use super::*;
use crate::{
    AuthorityVerificationKey, DogfoodLaunchVerificationKey, LaunchGrantClaims,
    LaunchGrantExpectation, SignedDogfoodLaunchGrantV1, SignedLaunchGrant,
};

use super::super::super::grant_signing::PurposeSeparatedPasetoSigningKey;
use super::super::test_support::{DOGFOOD_SECRET_HEX, signed_dogfood_launch};

fn raw_signed(
    payload: &[u8],
    purpose: &str,
    assertion: &[u8],
    secret: &str,
) -> SignedProviderEnrollmentV2 {
    let key = PurposeSeparatedPasetoSigningKey::from_bytes(ISSUER, KEY_ID, &bytes(secret)).unwrap();
    let footer = canonical_json(&json!({
        "issuer": ISSUER,
        "key_id": KEY_ID,
        "purpose": purpose,
        "schema_version": DOGFOOD_SCHEMA_VERSION,
    }))
    .unwrap();
    SignedProviderEnrollmentV2 {
        schema_version: DOGFOOD_SCHEMA_VERSION.to_owned(),
        issuer: ISSUER.to_owned(),
        key_id: KEY_ID.to_owned(),
        paseto: key.sign(payload, &footer, assertion).unwrap(),
    }
}

fn env_refusal(
    envelope: &SignedProviderEnrollmentV2,
    policy: &PolicySnapshotV1,
    expected: &ProviderEnrollmentExpectationV2,
    now: u64,
    code: &'static str,
) {
    refusal(envelope.verify(policy, expected, now), code);
}

fn cross_use(
    mut envelope: SignedProviderEnrollmentV2,
    policy: &PolicySnapshotV1,
    expected: &ProviderEnrollmentExpectationV2,
) {
    env_refusal(
        &envelope,
        policy,
        expected,
        ACT,
        "PROVIDER_ENROLLMENT_SIGNER_KEY_WRONG_PURPOSE",
    );
    envelope.issuer = ISSUER.to_owned();
    envelope.key_id = KEY_ID.to_owned();
    env_refusal(
        &envelope,
        policy,
        expected,
        ACT,
        "PROVIDER_ENROLLMENT_INVALID",
    );
}

#[test]
fn envelope_crypto_canonical_and_claims_hostiles_keep_precedence() {
    let policy = policy();
    let claims = claims(LaunchProvider::Cursor, &policy);
    let expected = expected(&claims);
    let signed = signer().sign(&claims).unwrap();
    for mutate in [
        (|e: &mut SignedProviderEnrollmentV2| e.schema_version = "v2".to_owned())
            as fn(&mut SignedProviderEnrollmentV2),
        |e| e.issuer.clear(),
        |e| e.key_id = "x".repeat(129),
        |e| e.paseto = "v4.local.invalid".to_owned(),
        |e| {
            e.paseto.pop();
        },
        |e| e.paseto.push('A'),
    ] {
        let mut hostile = signed.clone();
        mutate(&mut hostile);
        env_refusal(
            &hostile,
            &policy,
            &expected,
            ACT - 1,
            "PROVIDER_ENROLLMENT_INVALID",
        );
    }
    let mut at_cap = signed.clone();
    at_cap.paseto = format!(
        "v4.public.{}",
        "A".repeat(MAX_PROVIDER_ENROLLMENT_TOKEN_BYTES - 10)
    );
    validate_envelope(&at_cap).unwrap();
    at_cap.paseto.push('A');
    refusal(validate_envelope(&at_cap), "PROVIDER_ENROLLMENT_INVALID");

    let encoded = String::from_utf8(canonical_json(&signed).unwrap()).unwrap();
    let duplicate = encoded.replacen('{', r#"{"schema_version":"v1alpha1","#, 1);
    for hostile in [
        br#"{"issuer":"provider-enrollment-operator","key_id":"provider-enrollment-1","schema_version":"v1alpha1"}"#.to_vec(),
        canonical_json(&json!({
            "issuer": ISSUER,
            "key_id": KEY_ID,
            "paseto": signed.paseto,
            "schema_version": DOGFOOD_SCHEMA_VERSION,
            "unknown": true,
        }))
        .unwrap(),
        duplicate.into_bytes(),
        serde_json::to_vec_pretty(&signed).unwrap(),
    ] {
        assert!(decode_canonical::<SignedProviderEnrollmentV2>(&hostile).is_err());
    }

    for (purpose, assertion, secret) in [
        (
            "dogfood-launch-signing",
            PROVIDER_ENROLLMENT_IMPLICIT_ASSERTION,
            ENROLL_SECRET,
        ),
        (
            PROVIDER_ENROLLMENT_SIGNING_PURPOSE,
            b"bullet-farm.dogfood-run-attestation.v1alpha1".as_slice(),
            ENROLL_SECRET,
        ),
        (
            PROVIDER_ENROLLMENT_SIGNING_PURPOSE,
            PROVIDER_ENROLLMENT_IMPLICIT_ASSERTION,
            DOGFOOD_SECRET_HEX,
        ),
    ] {
        let raw = raw_signed(
            &canonical_json(&claims).unwrap(),
            purpose,
            assertion,
            secret,
        );
        env_refusal(&raw, &policy, &expected, ACT, "PROVIDER_ENROLLMENT_INVALID");
    }

    let canonical = canonical_json(&claims).unwrap();
    let mut unknown = crate::decode_canonical_value(&canonical_json(&claims).unwrap()).unwrap();
    unknown["unknown"] = json!(true);
    let duplicate = format!(
        r#"{{"schema_version":"v1alpha1",{}"#,
        std::str::from_utf8(&canonical)
            .unwrap()
            .trim_start_matches('{')
    );
    for payload in [
        canonical_json(&unknown).unwrap(),
        serde_json::to_vec_pretty(&claims).unwrap(),
        duplicate.into_bytes(),
        b"{}".to_vec(),
    ] {
        let raw = raw_signed(
            &payload,
            PROVIDER_ENROLLMENT_SIGNING_PURPOSE,
            PROVIDER_ENROLLMENT_IMPLICIT_ASSERTION,
            ENROLL_SECRET,
        );
        env_refusal(&raw, &policy, &expected, ACT, "PROVIDER_ENROLLMENT_INVALID");
    }

    for mutate in [
        (|v: &mut ProviderEnrollmentClaimsV2| v.schema_version = "v2".to_owned())
            as fn(&mut ProviderEnrollmentClaimsV2),
        |v| v.signing_purpose = "dogfood-launch-signing".to_owned(),
        |v| v.claims_domain = "authority.launch-grant-claims.v1alpha1".to_owned(),
        |v| v.expires_at_unix_ms = v.activates_at_unix_ms,
    ] {
        let mut malformed = claims.clone();
        mutate(&mut malformed);
        let raw = raw_signed(
            &canonical_json(&malformed).unwrap(),
            PROVIDER_ENROLLMENT_SIGNING_PURPOSE,
            PROVIDER_ENROLLMENT_IMPLICIT_ASSERTION,
            ENROLL_SECRET,
        );
        env_refusal(
            &raw,
            &policy,
            &expected,
            ACT - 1,
            "PROVIDER_ENROLLMENT_INVALID",
        );
    }
    let mut wrong_identity = claims.clone();
    wrong_identity.issuer = "other-issuer".to_owned();
    let raw = raw_signed(
        &canonical_json(&wrong_identity).unwrap(),
        PROVIDER_ENROLLMENT_SIGNING_PURPOSE,
        PROVIDER_ENROLLMENT_IMPLICIT_ASSERTION,
        ENROLL_SECRET,
    );
    env_refusal(
        &raw,
        &policy,
        &expected,
        ACT,
        "PROVIDER_ENROLLMENT_KEY_UNKNOWN",
    );
    let mut bad_pair = claims;
    bad_pair.protocol = DogfoodProviderProtocolV1::ClaudeStreamJson;
    refusal(
        signer().sign(&bad_pair),
        "DOGFOOD_PROVIDER_PROTOCOL_MISMATCH",
    );
    for (issuer, key, material) in [
        ("", KEY_ID, vec![1; 64]),
        (ISSUER, "", vec![1; 64]),
        (ISSUER, KEY_ID, vec![1; 63]),
        (ISSUER, KEY_ID, vec![1; 65]),
        (ISSUER, KEY_ID, vec![0; 64]),
    ] {
        refusal(
            ProviderEnrollmentSigningKey::from_bytes(issuer, key, &material),
            "INVALID_PROVIDER_ENROLLMENT_KEY",
        );
    }
}

#[test]
fn live_launch_and_enrollment_tokens_are_bidirectionally_non_interchangeable() {
    let policy = policy();
    let enrollment = claims(LaunchProvider::Claude, &policy);
    let expected = expected(&enrollment);
    let (dogfood, intent, launch) = signed_dogfood_launch(&enrollment);
    let crossed = decode_canonical(&canonical_json(&dogfood).unwrap()).unwrap();
    cross_use(crossed, &policy, &expected);

    let signed = signer().sign(&enrollment).unwrap();
    let crossed: SignedDogfoodLaunchGrantV1 =
        decode_canonical(&canonical_json(&signed).unwrap()).unwrap();
    let key =
        DogfoodLaunchVerificationKey::from_bytes(ISSUER, KEY_ID, &bytes(ENROLL_PUBLIC)).unwrap();
    refusal(
        crossed.verify(&key, &intent, &enrollment, launch.not_before_unix_ms),
        "DOGFOOD_GRANT_INVALID",
    );

    let golden = crate::decode_canonical_value(LIVE_GOLDEN).unwrap();
    let claims: LaunchGrantClaims =
        decode_canonical(golden["claims_canonical_json"].as_str().unwrap().as_bytes()).unwrap();
    let live: SignedLaunchGrant =
        decode_canonical(&canonical_json(&golden["envelope"]).unwrap()).unwrap();
    let key = AuthorityVerificationKey::from_bytes(
        &live.issuer,
        &live.key_id,
        &bytes(golden["public_key_hex"].as_str().unwrap()),
    )
    .unwrap();
    live.verify(
        &key,
        &LaunchGrantExpectation {
            audience: claims.audience,
            lease: claims.lease_subject(),
            provider: claims.provider_subject(),
            policy_snapshot_digest: claims.policy_snapshot_digest,
        },
        golden["verify_at_unix_ms"].as_u64().unwrap(),
    )
    .unwrap();
    let crossed = decode_canonical(&canonical_json(&live).unwrap()).unwrap();
    cross_use(crossed, &policy, &expected);
}
