use serde_json::json;

use super::*;

const LIVE_GOLDEN: &[u8] =
    include_bytes!("../../../../../../fixtures/canonical/launch-grant-golden.json");
const LAUNCH_SECRET: &str = "b4cbfb43df4ce210727d953e4a713307fa19bb7d9f85041438d9e11b942a37741eb9dbbbbc047c03fd70604e0071f0987e16b28b757225c11f00415d0e20b1a2";
const ENROLL_SECRET: &str = "c5aa8df43f9f837bedb7442f31dcb7b166d38535076f094b85ce3a2e0b4458f7fc51cd8e6218a1a38da47ed00230f0580816ed13ba3303ac5deb911548908025";
const LIVE_SECRET: &str = "f5e5767cf153319517630f226876b86c8160cc583bc013744c6bf255f5cc0ee5278117fc144c72340f67d0f2316e8386ceffbf2b2428c9c51fef7c597f1d426e";

#[test]
fn envelope_crypto_and_recursive_body_hostiles_keep_precedence() {
    let policy = policy();
    let value = fixture(LaunchProvider::Claude, &policy);
    let envelope = signed(&value);
    let mut structurally_invalid = envelope.clone();
    structurally_invalid.schema_version = "v2".into();
    let mut unsafe_policy = policy.clone();
    unsafe_policy.evidence_policy.unknown_satisfies_gate = true;
    refusal(
        structurally_invalid.verify(&unsafe_policy, &value.subjects(), NOW),
        "DOGFOOD_RUN_ATTESTATION_INVALID",
    );
    for material in [&[0_u8; 64][..], &[1_u8; 63][..], &[1_u8; 65][..]] {
        refusal(
            DogfoodRunAttestationSigningKey::from_bytes(
                &PrincipalId::from_digest(digest(49)),
                RUN_KEY,
                material,
            ),
            "INVALID_DOGFOOD_RUN_ATTESTATION_KEY",
        );
    }
    for key_id in ["", "bad key", &"x".repeat(129)] {
        refusal(
            DogfoodRunAttestationSigningKey::from_bytes(
                &PrincipalId::from_digest(digest(49)),
                key_id,
                &bytes(RUN_SECRET),
            ),
            "INVALID_DOGFOOD_RUN_ATTESTATION_KEY",
        );
    }
    let mutations: [fn(&mut SignedDogfoodRunV1); 7] = [
        |v| v.schema_version = "v2".into(),
        |v| v.issuer = "pri_short".into(),
        |v| v.issuer = format!("bad_{}", "31".repeat(32)),
        |v| v.key_id = "x".repeat(129),
        |v| v.paseto = v.paseto.replacen("v4.public.", "v4.local.", 1),
        |v| {
            v.paseto.pop();
        },
        |v| v.paseto.push('A'),
    ];
    for mutate in mutations {
        let mut hostile = envelope.clone();
        mutate(&mut hostile);
        refusal(
            hostile.verify(&policy, &value.subjects(), NOW),
            "DOGFOOD_RUN_ATTESTATION_INVALID",
        );
    }
    let mut cap = envelope.clone();
    cap.paseto = format!(
        "v4.public.{}",
        "A".repeat(MAX_DOGFOOD_RUN_ATTESTATION_TOKEN_BYTES - 10)
    );
    validate_envelope(&cap).unwrap();
    cap.paseto.push('A');
    refusal(validate_envelope(&cap), "DOGFOOD_RUN_ATTESTATION_INVALID");
    let encoded = String::from_utf8(canonical_json(&envelope).unwrap()).unwrap();
    let duplicate = encoded.replacen('{', r#"{"schema_version":"v1alpha1","#, 1);
    for hostile in [
        br#"{"issuer":"x","key_id":"x","schema_version":"v1alpha1"}"#.to_vec(),
        canonical_json(&json!({"issuer": ATT, "key_id": RUN_KEY, "paseto": envelope.paseto, "schema_version": DOGFOOD_SCHEMA_VERSION, "unknown": true})).unwrap(),
        duplicate.into_bytes(),
        serde_json::to_vec_pretty(&envelope).unwrap(),
    ] {
        assert!(decode_canonical::<SignedDogfoodRunV1>(&hostile).is_err());
    }
    let mut wrong_material = policy.clone();
    let index = key_index(&wrong_material, KeyPurposeV1::DogfoodRunAttestationSigning);
    wrong_material.issuer_keys[index].public_key = "aa".repeat(32);
    refusal(
        envelope.verify(&wrong_material, &value.subjects(), NOW),
        "DOGFOOD_RUN_ATTESTATION_INVALID",
    );

    let mut malformed = value.run.clone();
    malformed.schema_version = "v2".into();
    refusal(
        signer().sign(&malformed, &value.subjects()),
        "DOGFOOD_RUN_INVALID",
    );
    refusal(
        raw_signed(
            &canonical_json(&malformed).unwrap(),
            DOGFOOD_RUN_ATTESTATION_SIGNING_PURPOSE,
            DOGFOOD_RUN_ATTESTATION_IMPLICIT_ASSERTION,
        )
        .verify(&policy, &value.subjects(), NOW),
        "DOGFOOD_RUN_INVALID",
    );
    let mut body = serde_json::to_value(&value.run).unwrap();
    for field in [
        "accepted",
        "audience",
        "authority",
        "candidate",
        "credential",
        "effect",
        "eligibility",
        "evidence",
        "host",
        "nonce",
        "outcome",
        "output",
        "path",
        "raw_secret",
        "status",
        "success",
    ] {
        body[field] = json!(true);
        refusal(
            raw_signed(
                &canonical_json(&body).unwrap(),
                DOGFOOD_RUN_ATTESTATION_SIGNING_PURPOSE,
                DOGFOOD_RUN_ATTESTATION_IMPLICIT_ASSERTION,
            )
            .verify(&policy, &value.subjects(), NOW),
            "DOGFOOD_RUN_INVALID",
        );
        body.as_object_mut().unwrap().remove(field);
    }
    body["process"]["caller_outcome"] = json!("pass");
    refusal(
        raw_signed(
            &canonical_json(&body).unwrap(),
            DOGFOOD_RUN_ATTESTATION_SIGNING_PURPOSE,
            DOGFOOD_RUN_ATTESTATION_IMPLICIT_ASSERTION,
        )
        .verify(&policy, &value.subjects(), NOW),
        "DOGFOOD_RUN_INVALID",
    );
    let canonical = String::from_utf8(canonical_json(&value.run).unwrap()).unwrap();
    for payload in [
        serde_json::to_vec_pretty(&value.run).unwrap(),
        canonical
            .replacen('{', r#"{"schema_version":"v1alpha1","#, 1)
            .into_bytes(),
    ] {
        refusal(
            raw_signed(
                &payload,
                DOGFOOD_RUN_ATTESTATION_SIGNING_PURPOSE,
                DOGFOOD_RUN_ATTESTATION_IMPLICIT_ASSERTION,
            )
            .verify(&policy, &value.subjects(), NOW),
            "DOGFOOD_RUN_INVALID",
        );
    }
    let error = refusal(
        raw_signed(
            &canonical_json(&malformed).unwrap(),
            DOGFOOD_RUN_ATTESTATION_SIGNING_PURPOSE,
            b"wrong-assertion",
        )
        .verify(&policy, &value.subjects(), NOW),
        "DOGFOOD_RUN_ATTESTATION_INVALID",
    );
    assert!(error.reason().contains("PASETO"));
}

#[test]
fn launch_enrollment_live_authority_and_release_purposes_do_not_cross() {
    let policy = policy();
    let value = fixture(LaunchProvider::Agy, &policy);
    let run = signed(&value);

    let launch = DogfoodLaunchSigningKey::from_bytes(LAUNCH, LAUNCH_KEY, &bytes(LAUNCH_SECRET))
        .unwrap()
        .sign(&value.grant)
        .unwrap();
    let mut crossed: SignedDogfoodRunV1 =
        decode_canonical(&canonical_json(&launch).unwrap()).unwrap();
    refusal(
        crossed.verify(&policy, &value.subjects(), NOW),
        "DOGFOOD_RUN_ATTESTOR_KEY_WRONG_PURPOSE",
    );
    crossed.issuer = ATT.into();
    crossed.key_id = RUN_KEY.into();
    refusal(
        crossed.verify(&policy, &value.subjects(), NOW),
        "DOGFOOD_RUN_ATTESTATION_INVALID",
    );
    let crossed: SignedDogfoodLaunchGrantV1 =
        decode_canonical(&canonical_json(&run).unwrap()).unwrap();
    let key = DogfoodLaunchVerificationKey::from_bytes(ATT, RUN_KEY, &bytes(RUN_PUBLIC)).unwrap();
    refusal(
        crossed.verify(&key, &value.intent, &value.enrollment, NOW),
        "DOGFOOD_GRANT_INVALID",
    );

    let enrollment =
        ProviderEnrollmentSigningKey::from_bytes(ENROLL, ENROLL_KEY, &bytes(ENROLL_SECRET))
            .unwrap()
            .sign(&value.enrollment)
            .unwrap();
    let expected = ProviderEnrollmentExpectationV2 {
        provider_enrollment_id: value.enrollment.enrollment_id().unwrap(),
        enrollment_generation: value.enrollment.enrollment_generation,
        policy_snapshot_digest: value.enrollment.policy_snapshot_digest,
        policy_generation: value.enrollment.policy_generation,
    };
    enrollment.verify(&policy, &expected, NOW).unwrap();
    let mut crossed: SignedDogfoodRunV1 =
        decode_canonical(&canonical_json(&enrollment).unwrap()).unwrap();
    refusal(
        crossed.verify(&policy, &value.subjects(), NOW),
        "DOGFOOD_RUN_ATTESTOR_KEY_WRONG_PURPOSE",
    );
    crossed.issuer = ATT.into();
    crossed.key_id = RUN_KEY.into();
    refusal(
        crossed.verify(&policy, &value.subjects(), NOW),
        "DOGFOOD_RUN_ATTESTATION_INVALID",
    );
    let mut crossed: SignedProviderEnrollmentV2 =
        decode_canonical(&canonical_json(&run).unwrap()).unwrap();
    refusal(
        crossed.verify(&policy, &expected, NOW),
        "PROVIDER_ENROLLMENT_SIGNER_KEY_WRONG_PURPOSE",
    );
    crossed.issuer = ENROLL.into();
    crossed.key_id = ENROLL_KEY.into();
    refusal(
        crossed.verify(&policy, &expected, NOW),
        "PROVIDER_ENROLLMENT_INVALID",
    );

    let golden = decode_canonical_value(LIVE_GOLDEN).unwrap();
    let mut claims: LaunchGrantClaims =
        decode_canonical(golden["claims_canonical_json"].as_str().unwrap().as_bytes()).unwrap();
    claims.issuer = LIVE.into();
    claims.key_id = LIVE_KEY.into();
    let live = AuthoritySigningKey::from_bytes(LIVE, LIVE_KEY, &bytes(LIVE_SECRET))
        .unwrap()
        .sign_launch_grant(&claims)
        .unwrap();
    let expected_live = LaunchGrantExpectation {
        audience: claims.audience,
        lease: claims.lease_subject(),
        provider: claims.provider_subject(),
        policy_snapshot_digest: claims.policy_snapshot_digest,
    };
    let live_key =
        AuthorityVerificationKey::from_bytes(LIVE, LIVE_KEY, &bytes(LIVE_PUBLIC)).unwrap();
    live.verify(&live_key, &expected_live, claims.not_before_unix_ms)
        .unwrap();
    let mut crossed: SignedDogfoodRunV1 =
        decode_canonical(&canonical_json(&live).unwrap()).unwrap();
    refusal(
        crossed.verify(&policy, &value.subjects(), NOW),
        "DOGFOOD_RUN_ATTESTOR_KEY_WRONG_PURPOSE",
    );
    crossed.issuer = ATT.into();
    crossed.key_id = RUN_KEY.into();
    refusal(
        crossed.verify(&policy, &value.subjects(), NOW),
        "DOGFOOD_RUN_ATTESTATION_INVALID",
    );
    let crossed: SignedLaunchGrant = decode_canonical(&canonical_json(&run).unwrap()).unwrap();
    let run_key = AuthorityVerificationKey::from_bytes(ATT, RUN_KEY, &bytes(RUN_PUBLIC)).unwrap();
    refusal(
        crossed.verify(&run_key, &expected_live, claims.not_before_unix_ms),
        "LAUNCH_GRANT_INVALID",
    );

    let payload = canonical_json(&value.run).unwrap();
    for (purpose, assertion) in [
        ("authority-signing", AUTHORITY_IMPLICIT_ASSERTION),
        (
            "mutation-permit-signing",
            MUTATION_PERMIT_IMPLICIT_ASSERTION,
        ),
        (
            "release-signing",
            b"bullet-farm.release.v1alpha1".as_slice(),
        ),
        (
            "dogfood-launch-signing",
            DOGFOOD_LAUNCH_GRANT_IMPLICIT_ASSERTION,
        ),
        (
            "provider-enrollment-signing",
            PROVIDER_ENROLLMENT_IMPLICIT_ASSERTION,
        ),
    ] {
        refusal(
            raw_signed(&payload, purpose, assertion).verify(&policy, &value.subjects(), NOW),
            "DOGFOOD_RUN_ATTESTATION_INVALID",
        );
    }
}
