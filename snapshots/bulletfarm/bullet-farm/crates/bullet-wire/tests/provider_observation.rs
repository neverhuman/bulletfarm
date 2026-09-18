use bullet_wire::{
    Blake3Digest, CredentialProjectionProfileId, DOGFOOD_SCHEMA_VERSION, DogfoodProviderProtocolV1,
    LaunchProvider, PrincipalId, ProviderEndpointObservationV1, ProviderEnrollmentClaimsV2,
    ProviderObservationSubjectV1, ProviderProbeObservationV1, ProviderProfileId,
    ProviderProfileObservationV1, ProviderRuntimePassportV1, ProviderVersionObservationV1,
    RuntimeExecutionV1, RuntimeFileRoleV1, RuntimeFileV1, RuntimeLoaderV1, WireError,
    canonical_json, decode_provider_endpoint_observation, decode_provider_probe_observation,
    decode_provider_profile_observation, decode_provider_version_observation,
    verify_provider_observations,
};
use serde_json::{Value, json};

/// Decoder under test; aliased so the per-field tables stay readable.
type DecodeFn = fn(&[u8]) -> Result<(), WireError>;

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const PROVIDERS: [LaunchProvider; 4] = [
    LaunchProvider::Claude,
    LaunchProvider::Codex,
    LaunchProvider::Cursor,
    LaunchProvider::Agy,
];

#[derive(Clone)]
struct Fixture {
    passport: ProviderRuntimePassportV1,
    enrollment: ProviderEnrollmentClaimsV2,
    probe: ProviderProbeObservationV1,
    endpoint: ProviderEndpointObservationV1,
    version: ProviderVersionObservationV1,
    profile: ProviderProfileObservationV1,
}

fn digest(seed: u8) -> Blake3Digest {
    Blake3Digest::from_bytes([seed; 32])
}

fn version(provider: LaunchProvider) -> &'static str {
    match provider {
        LaunchProvider::Claude => "2.1.251",
        LaunchProvider::Codex => "0.150.1",
        LaunchProvider::Cursor => "2026.08.11",
        LaunchProvider::Agy => "1.1.19",
    }
}

fn entrypoint(provider: LaunchProvider) -> &'static str {
    match provider {
        LaunchProvider::Claude => "bin/claude",
        LaunchProvider::Codex => "bin/codex",
        LaunchProvider::Cursor => "bin/cursor-agent",
        LaunchProvider::Agy => "bin/agy",
    }
}

fn passport(provider: LaunchProvider) -> ProviderRuntimePassportV1 {
    let version = version(provider);
    let entrypoint = entrypoint(provider);
    ProviderRuntimePassportV1 {
        schema_version: 1,
        provider,
        protocol: DogfoodProviderProtocolV1::required_for(provider),
        version: version.into(),
        deployment_root: format!("/usr/lib/bullet/providers/{}/{version}", provider.as_str()),
        entrypoint: entrypoint.into(),
        execution: RuntimeExecutionV1::Native {
            loader: RuntimeLoaderV1::Static,
        },
        files: vec![RuntimeFileV1 {
            path: entrypoint.into(),
            role: RuntimeFileRoleV1::Entrypoint,
            mode: 0o555,
            size: 1,
            blake3: "11".repeat(32),
        }],
        aggregate_file_count: 1,
        aggregate_size_bytes: 1,
    }
}

fn enrollment(passport: &ProviderRuntimePassportV1) -> ProviderEnrollmentClaimsV2 {
    ProviderEnrollmentClaimsV2 {
        schema_version: DOGFOOD_SCHEMA_VERSION.into(),
        issuer: "operator.example".into(),
        key_id: "provider-enrollment-alpha".into(),
        signing_purpose: "provider-enrollment-signing".into(),
        claims_domain: "provider.enrollment-claims.v2".into(),
        provider: passport.provider,
        protocol: passport.protocol,
        runtime_passport_id: passport.passport_id().unwrap(),
        provider_profile_id: ProviderProfileId::from_digest(digest(2)),
        service_identity_id: PrincipalId::from_digest(digest(3)),
        credential_projection_profile_id: CredentialProjectionProfileId::from_digest(digest(4)),
        runtime_version: passport.version.clone(),
        enrollment_generation: 2,
        activates_at_unix_ms: 400_000,
        expires_at_unix_ms: 800_000,
        revoked_at_unix_ms: None,
        egress_policy_digest: digest(5),
        tool_policy_digest: digest(6),
        budget_policy_digest: digest(7),
        endpoint_observation_digest: digest(8),
        version_observation_digest: digest(9),
        profile_observation_digest: digest(10),
        policy_snapshot_digest: digest(11),
        policy_generation: 2,
    }
}

fn fixture(provider: LaunchProvider) -> Fixture {
    let passport = passport(provider);
    let mut enrollment = enrollment(&passport);
    let subject = ProviderObservationSubjectV1 {
        provider,
        protocol: passport.protocol,
        runtime_passport_id: passport.passport_id().unwrap(),
        provider_profile_id: enrollment.provider_profile_id.clone(),
        service_identity_id: enrollment.service_identity_id.clone(),
        credential_projection_profile_id: enrollment.credential_projection_profile_id.clone(),
        policy_snapshot_digest: enrollment.policy_snapshot_digest,
        policy_generation: enrollment.policy_generation,
    };
    let probe = ProviderProbeObservationV1 {
        schema_version: DOGFOOD_SCHEMA_VERSION.into(),
        subject: subject.clone(),
        probe_grant_digest: digest(12),
        containment_receipt_digest: digest(13),
        protocol_transcript_digest: digest(14),
        observed_at_unix_ms: 100_000,
    };
    let probe_observation_digest = probe.digest().unwrap();
    let endpoint = ProviderEndpointObservationV1 {
        schema_version: DOGFOOD_SCHEMA_VERSION.into(),
        subject: subject.clone(),
        probe_observation_digest,
        entrypoint_blake3: digest(0x11),
    };
    let version = ProviderVersionObservationV1 {
        schema_version: DOGFOOD_SCHEMA_VERSION.into(),
        subject: subject.clone(),
        probe_observation_digest,
        runtime_version: passport.version.clone(),
        native_version_artifact_digest: digest(15),
    };
    let profile = ProviderProfileObservationV1 {
        schema_version: DOGFOOD_SCHEMA_VERSION.into(),
        subject,
        probe_observation_digest,
        effective_identity_artifact_digest: digest(16),
    };
    enrollment.endpoint_observation_digest = endpoint.digest().unwrap();
    enrollment.version_observation_digest = version.digest().unwrap();
    enrollment.profile_observation_digest = profile.digest().unwrap();
    Fixture {
        passport,
        enrollment,
        probe,
        endpoint,
        version,
        profile,
    }
}

fn verify(value: &Fixture) -> Result<(), WireError> {
    verify_provider_observations(
        &value.enrollment,
        &value.passport,
        &value.probe,
        &value.endpoint,
        &value.version,
        &value.profile,
    )
}

fn refusal<T>(result: Result<T, WireError>, expected: &'static str) {
    match result {
        Err(error) => assert_eq!(error.code(), expected, "{error}"),
        Ok(_) => panic!("expected {expected}"),
    }
}

#[test]
fn all_four_provider_pairs_round_trip_and_pin_the_wire() {
    for provider in PROVIDERS {
        let value = fixture(provider);
        verify(&value).unwrap();
        assert_eq!(
            decode_provider_probe_observation(&canonical_json(&value.probe).unwrap()).unwrap(),
            value.probe
        );
        assert_eq!(
            decode_provider_endpoint_observation(&canonical_json(&value.endpoint).unwrap())
                .unwrap(),
            value.endpoint
        );
        assert_eq!(
            decode_provider_version_observation(&canonical_json(&value.version).unwrap()).unwrap(),
            value.version
        );
        assert_eq!(
            decode_provider_profile_observation(&canonical_json(&value.profile).unwrap()).unwrap(),
            value.profile
        );
        let provider_json = serde_json::to_value(&value.probe).unwrap();
        assert_eq!(provider_json["subject"]["provider"], provider.as_str());
    }
    let value = fixture(LaunchProvider::Claude);
    let expected = concat!(
        r#"{"containment_receipt_digest":"0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d","observed_at_unix_ms":100000,"#,
        r#""probe_grant_digest":"0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c","#,
        r#""protocol_transcript_digest":"0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e","schema_version":"v1alpha1","subject":{"#,
        r#""credential_projection_profile_id":"cpp_0404040404040404040404040404040404040404040404040404040404040404","policy_generation":2,"#,
        r#""policy_snapshot_digest":"0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b","protocol":"claude_stream_json","provider":"claude","#,
        r#""provider_profile_id":"prf_0202020202020202020202020202020202020202020202020202020202020202","#,
        r#""runtime_passport_id":"rtp_31babe99f6ad6fb20cd4cf5f376ac2f413785d20a6dadc877e3bc6c46f02d07e","#,
        r#""service_identity_id":"pri_0303030303030303030303030303030303030303030303030303030303030303"}}"#,
    );
    assert_eq!(canonical_json(&value.probe).unwrap(), expected.as_bytes());
    assert_eq!(
        [
            value.probe.digest().unwrap().to_hex(),
            value.endpoint.digest().unwrap().to_hex(),
            value.version.digest().unwrap().to_hex(),
            value.profile.digest().unwrap().to_hex(),
        ],
        [
            "2978e36064e245a95f580e009590d46432fdde16447d339950d63313d2ae6c8e",
            "c77e83e9085b7df5c42eb27ddcaa2853b896067077995902ffaa6117b6396cc0",
            "c7453d918ad38f5dfb0bbc2062b707d3fc922cb961305de31d93689f205a6dba",
            "2dabc1bf5e976930a9c126d1ab53abaf998be6e123d8aca475655cf8953ce35a",
        ]
        .map(str::to_owned),
    );
}

#[test]
fn decoders_are_bounded_recursively_closed_and_hostile_to_aliases() {
    let value = fixture(LaunchProvider::Claude);
    let mut probe = serde_json::to_value(&value.probe).unwrap();
    for name in "issuer key_id audience signature nonce authority admitted pass success outcome conformance release secret token cookie email account subscription auth_method provider_output credential_projection_id path deployment_root device inode uid gid pid home workdir environment argv stdout stderr host ip url port tls proposal candidate evidence usage cost model".split_whitespace() {
        let mut changed = probe.clone();
        changed[name] = json!(true);
        refusal(
            decode_provider_probe_observation(&serde_jcs::to_vec(&changed).unwrap()),
            "PROVIDER_PROBE_OBSERVATION_INVALID",
        );
    }
    probe["subject"]["caller_state"] = json!("trusted");
    refusal(
        decode_provider_probe_observation(&serde_jcs::to_vec(&probe).unwrap()),
        "PROVIDER_PROBE_OBSERVATION_INVALID",
    );
    for (field, code) in [
        ("probe_grant_digest", "PROVIDER_PROBE_OBSERVATION_INVALID"),
        ("entrypoint_blake3", "PROVIDER_ENDPOINT_OBSERVATION_INVALID"),
        ("runtime_version", "PROVIDER_VERSION_OBSERVATION_INVALID"),
        (
            "effective_identity_artifact_digest",
            "PROVIDER_PROFILE_OBSERVATION_INVALID",
        ),
    ] {
        let (object, decode): (Value, DecodeFn) = match field {
            "probe_grant_digest" => (serde_json::to_value(&value.probe).unwrap(), |bytes| {
                decode_provider_probe_observation(bytes).map(|_| ())
            }),
            "entrypoint_blake3" => (serde_json::to_value(&value.endpoint).unwrap(), |bytes| {
                decode_provider_endpoint_observation(bytes).map(|_| ())
            }),
            "runtime_version" => (serde_json::to_value(&value.version).unwrap(), |bytes| {
                decode_provider_version_observation(bytes).map(|_| ())
            }),
            _ => (serde_json::to_value(&value.profile).unwrap(), |bytes| {
                decode_provider_profile_observation(bytes).map(|_| ())
            }),
        };
        let mut missing = object.clone();
        missing.as_object_mut().unwrap().remove(field);
        refusal(decode(&serde_jcs::to_vec(&missing).unwrap()), code);
        let mut unknown = object;
        unknown["unknown"] = json!(true);
        refusal(decode(&serde_jcs::to_vec(&unknown).unwrap()), code);
    }
    let text = String::from_utf8(canonical_json(&value.probe).unwrap()).unwrap();
    refusal(
        decode_provider_probe_observation(format!(" {text}").as_bytes()),
        "PROVIDER_PROBE_OBSERVATION_INVALID",
    );
    let duplicate = text.replacen(
        "\"observed_at_unix_ms\":100000",
        "\"observed_at_unix_ms\":100000,\"observed_at_unix_ms\":100000",
        1,
    );
    refusal(
        decode_provider_probe_observation(duplicate.as_bytes()),
        "PROVIDER_PROBE_OBSERVATION_INVALID",
    );
    let mut alias = serde_json::to_value(&value.probe).unwrap();
    alias["subject"]["provider"] = json!("antigravity");
    refusal(
        decode_provider_probe_observation(&serde_jcs::to_vec(&alias).unwrap()),
        "PROVIDER_PROBE_OBSERVATION_INVALID",
    );
    for (field, prefix) in [
        ("runtime_passport_id", "prf_"),
        ("provider_profile_id", "rtp_"),
        ("service_identity_id", "cpp_"),
        ("credential_projection_profile_id", "pri_"),
    ] {
        alias = serde_json::to_value(&value.probe).unwrap();
        alias["subject"][field] = json!(format!("{prefix}{}", "a".repeat(64)));
        refusal(
            decode_provider_probe_observation(&serde_jcs::to_vec(&alias).unwrap()),
            "PROVIDER_PROBE_OBSERVATION_INVALID",
        );
    }
    let oversized = vec![b' '; 8_193];
    let decoders: [(DecodeFn, &str); 4] = [
        (
            |b| decode_provider_probe_observation(b).map(|_| ()),
            "PROVIDER_PROBE_OBSERVATION_INVALID",
        ),
        (
            |b| decode_provider_endpoint_observation(b).map(|_| ()),
            "PROVIDER_ENDPOINT_OBSERVATION_INVALID",
        ),
        (
            |b| decode_provider_version_observation(b).map(|_| ()),
            "PROVIDER_VERSION_OBSERVATION_INVALID",
        ),
        (
            |b| decode_provider_profile_observation(b).map(|_| ()),
            "PROVIDER_PROFILE_OBSERVATION_INVALID",
        ),
    ];
    for (decode, code) in decoders {
        refusal(decode(&oversized), code);
    }
}

#[test]
fn every_valid_field_changes_its_domain_digest() {
    let base = fixture(LaunchProvider::Claude);
    let probe_digest = base.probe.digest().unwrap();
    let subject_mutations: [fn(&mut ProviderObservationSubjectV1); 8] = [
        |v| {
            v.provider = LaunchProvider::Codex;
            v.protocol = DogfoodProviderProtocolV1::CodexAppServerJsonl;
        },
        |v| v.runtime_passport_id = bullet_wire::RuntimePassportId::from_digest(digest(30)),
        |v| v.provider_profile_id = ProviderProfileId::from_digest(digest(31)),
        |v| v.service_identity_id = PrincipalId::from_digest(digest(32)),
        |v| {
            v.credential_projection_profile_id =
                CredentialProjectionProfileId::from_digest(digest(33))
        },
        |v| v.policy_snapshot_digest = digest(34),
        |v| v.policy_generation += 1,
        |v| {
            v.provider = LaunchProvider::Agy;
            v.protocol = DogfoodProviderProtocolV1::AntigravityHeadlessStructured;
        },
    ];
    for mutate in subject_mutations {
        let mut changed = base.probe.clone();
        mutate(&mut changed.subject);
        assert_ne!(changed.digest().unwrap(), probe_digest);
        let mut endpoint = base.endpoint.clone();
        endpoint.subject = changed.subject.clone();
        assert_ne!(endpoint.digest().unwrap(), base.endpoint.digest().unwrap());
        let mut version = base.version.clone();
        version.subject = changed.subject.clone();
        assert_ne!(version.digest().unwrap(), base.version.digest().unwrap());
        let mut profile = base.profile.clone();
        profile.subject = changed.subject;
        assert_ne!(profile.digest().unwrap(), base.profile.digest().unwrap());
    }
    let probe_mutations: [fn(&mut ProviderProbeObservationV1); 4] = [
        |v| v.probe_grant_digest = digest(40),
        |v| v.containment_receipt_digest = digest(41),
        |v| v.protocol_transcript_digest = digest(42),
        |v| v.observed_at_unix_ms += 1,
    ];
    for mutate in probe_mutations {
        let mut changed = base.probe.clone();
        mutate(&mut changed);
        assert_ne!(changed.digest().unwrap(), probe_digest);
    }
    let mut endpoint = base.endpoint.clone();
    endpoint.entrypoint_blake3 = digest(43);
    assert_ne!(endpoint.digest().unwrap(), base.endpoint.digest().unwrap());
    endpoint = base.endpoint.clone();
    endpoint.probe_observation_digest = digest(44);
    assert_ne!(endpoint.digest().unwrap(), base.endpoint.digest().unwrap());
    let mut version = base.version.clone();
    version.runtime_version = "2.1.252".into();
    assert_ne!(version.digest().unwrap(), base.version.digest().unwrap());
    version = base.version.clone();
    version.native_version_artifact_digest = digest(45);
    assert_ne!(version.digest().unwrap(), base.version.digest().unwrap());
    version = base.version.clone();
    version.probe_observation_digest = digest(46);
    assert_ne!(version.digest().unwrap(), base.version.digest().unwrap());
    let mut profile = base.profile.clone();
    profile.effective_identity_artifact_digest = digest(47);
    assert_ne!(profile.digest().unwrap(), base.profile.digest().unwrap());
    profile = base.profile.clone();
    profile.probe_observation_digest = digest(48);
    assert_ne!(profile.digest().unwrap(), base.profile.digest().unwrap());
}

#[test]
fn aggregate_verifier_derives_time_subject_probe_and_record_mismatches() {
    let base = fixture(LaunchProvider::Claude);
    let mut changed = base.clone();
    changed.probe.observed_at_unix_ms = 400_000;
    rebind_probe(&mut changed);
    verify(&changed).unwrap();
    for observed in [99_999, 400_001] {
        changed = base.clone();
        changed.probe.observed_at_unix_ms = observed;
        rebind_probe(&mut changed);
        refusal(verify(&changed), "PROVIDER_OBSERVATION_TIME_MISMATCH");
    }
    changed = base.clone();
    changed.probe.observed_at_unix_ms = 0;
    refusal(
        changed.probe.validate(),
        "PROVIDER_PROBE_OBSERVATION_INVALID",
    );
    changed.probe.observed_at_unix_ms = MAX_SAFE_INTEGER + 1;
    refusal(
        changed.probe.validate(),
        "PROVIDER_PROBE_OBSERVATION_INVALID",
    );
    changed = base.clone();
    changed.probe.subject.policy_generation = 0;
    refusal(
        changed.probe.validate(),
        "PROVIDER_PROBE_OBSERVATION_INVALID",
    );
    changed = base.clone();
    changed.version.runtime_version = "x".repeat(129);
    refusal(
        changed.version.validate(),
        "PROVIDER_VERSION_OBSERVATION_INVALID",
    );

    changed = base.clone();
    changed.endpoint.subject.service_identity_id = PrincipalId::from_digest(digest(50));
    refusal(verify(&changed), "PROVIDER_OBSERVATION_SUBJECT_MISMATCH");
    changed = base.clone();
    changed.enrollment.credential_projection_profile_id =
        CredentialProjectionProfileId::from_digest(digest(51));
    refusal(verify(&changed), "PROVIDER_OBSERVATION_SUBJECT_MISMATCH");
    changed = base.clone();
    changed.enrollment.provider_profile_id = ProviderProfileId::from_digest(digest(52));
    refusal(verify(&changed), "PROVIDER_OBSERVATION_SUBJECT_MISMATCH");
    changed = base.clone();
    changed.enrollment.policy_snapshot_digest = digest(53);
    refusal(verify(&changed), "PROVIDER_OBSERVATION_SUBJECT_MISMATCH");
    changed = base.clone();
    changed.passport.files[0].blake3 = "22".repeat(32);
    refusal(verify(&changed), "PROVIDER_OBSERVATION_SUBJECT_MISMATCH");
    changed = base.clone();
    changed.endpoint.probe_observation_digest = digest(54);
    refusal(verify(&changed), "PROVIDER_PROBE_OBSERVATION_MISMATCH");
    changed = base.clone();
    changed.endpoint.entrypoint_blake3 = digest(55);
    refusal(verify(&changed), "PROVIDER_ENDPOINT_OBSERVATION_MISMATCH");
    changed = base.clone();
    changed.enrollment.endpoint_observation_digest = digest(56);
    refusal(verify(&changed), "PROVIDER_ENDPOINT_OBSERVATION_MISMATCH");
    changed = base.clone();
    changed.version.runtime_version = "2.1.252".into();
    refusal(verify(&changed), "PROVIDER_VERSION_OBSERVATION_MISMATCH");
    changed = base.clone();
    changed.enrollment.version_observation_digest = digest(57);
    refusal(verify(&changed), "PROVIDER_VERSION_OBSERVATION_MISMATCH");
    changed = base.clone();
    changed.profile.effective_identity_artifact_digest = digest(58);
    refusal(verify(&changed), "PROVIDER_PROFILE_OBSERVATION_MISMATCH");
    changed = base;
    changed.enrollment.profile_observation_digest = digest(59);
    refusal(verify(&changed), "PROVIDER_PROFILE_OBSERVATION_MISMATCH");
}

fn rebind_probe(value: &mut Fixture) {
    let digest = value.probe.digest().unwrap();
    value.endpoint.probe_observation_digest = digest;
    value.version.probe_observation_digest = digest;
    value.profile.probe_observation_digest = digest;
    value.enrollment.endpoint_observation_digest = value.endpoint.digest().unwrap();
    value.enrollment.version_observation_digest = value.version.digest().unwrap();
    value.enrollment.profile_observation_digest = value.profile.digest().unwrap();
}
