use std::{fs, path::PathBuf};

use bullet_wire::{
    RELEASE_BUNDLE_MANIFEST_V2_DIGEST_DOMAIN, RELEASE_BUNDLE_MANIFEST_V2_NATIVE_SUBJECT_PREFIX,
    RELEASE_GATE_RECEIPT_SIGNATURE_DOMAIN, RELEASE_REGISTRY_MANIFEST_SIGNATURE_DOMAIN,
    RELEASE_SIGNER_POLICY_SIGNATURE_DOMAIN, RELEASE_SOURCE_SUBJECT_DIGEST_DOMAIN,
    RELEASE_TRUSTED_TIME_SIGNATURE_DOMAIN, ReleaseWireRecord, canonical_json,
    decode_release_record, release_bundle_manifest_v2_digest,
    v1alpha1::{
        GateReceiptV1, ReleaseEvidenceKindV1, ReleaseEvidenceSubjectV1, ReleaseProfileGraphV1,
        ReleaseRegistryObjectKindV1, ReleaseRegistryObjectV1,
    },
    validate_release_bindings, validate_release_bundle_manifest_v2_binding,
};
use serde_json::{Value, json};

#[path = "release_registry_contract/fixtures.rs"]
mod fixtures;
use fixtures::*;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap()
        .to_path_buf()
}

fn assert_release_error<T: ReleaseWireRecord + std::fmt::Debug>(value: &T) {
    let bytes = canonical_json(value).unwrap();
    assert_eq!(
        decode_release_record::<T>(&bytes).unwrap_err().code(),
        "INVALID_RELEASE_WIRE_RECORD"
    );
}

fn add_unknown(mut value: Value, pointer: &str, field: &str) -> Vec<u8> {
    value
        .pointer_mut(pointer)
        .and_then(Value::as_object_mut)
        .unwrap()
        .insert(field.to_owned(), json!("caller-selected"));
    canonical_json(&value).unwrap()
}

#[test]
fn release_registry_records_are_closed_exact_and_hostile() {
    let receipt = receipt();
    let bytes = canonical_json(&receipt).unwrap();
    assert_eq!(
        decode_release_record::<GateReceiptV1>(&bytes).unwrap(),
        receipt
    );
    signer_policy().validate_release().unwrap();
    registry_manifest().validate_release().unwrap();
    replay_state().validate_release().unwrap();
    trusted_time().validate_release().unwrap();
    let (graph, spec, request, bound_receipt) = binding_bundle();
    validate_release_bindings(&graph, &spec, &request, &bound_receipt).unwrap();

    assert_eq!(
        RELEASE_GATE_RECEIPT_SIGNATURE_DOMAIN,
        "release.gate-receipt-signature.v1alpha1"
    );
    assert_eq!(
        RELEASE_REGISTRY_MANIFEST_SIGNATURE_DOMAIN,
        "release.registry-manifest-signature.v1alpha1"
    );
    assert_eq!(
        RELEASE_SIGNER_POLICY_SIGNATURE_DOMAIN,
        "release.signer-policy-signature.v1alpha1"
    );
    assert_eq!(
        RELEASE_TRUSTED_TIME_SIGNATURE_DOMAIN,
        "release.trusted-time-signature.v1alpha1"
    );
    assert_eq!(
        RELEASE_SOURCE_SUBJECT_DIGEST_DOMAIN,
        "release.source-subject.v1alpha1"
    );
    assert_eq!(
        RELEASE_BUNDLE_MANIFEST_V2_DIGEST_DOMAIN,
        "release.bundle-manifest-v2.v1alpha1"
    );
    assert_eq!(
        RELEASE_BUNDLE_MANIFEST_V2_NATIVE_SUBJECT_PREFIX,
        "artifact:release-manifest-v2_"
    );

    let manifest_bytes = b"release_manifest_schema_version = \"2\"\n";
    let manifest_digest = release_bundle_manifest_v2_digest(manifest_bytes).unwrap();
    let manifest_hex = manifest_digest.strip_prefix("blake3:").unwrap();
    let manifest_subject = ReleaseEvidenceSubjectV1 {
        schema_version: "v1alpha1".to_owned(),
        subject_kind: ReleaseEvidenceKindV1::Artifact,
        subject_id: typed_id("cnt", '9'),
        native_subject_id: format!(
            "{RELEASE_BUNDLE_MANIFEST_V2_NATIVE_SUBJECT_PREFIX}{manifest_hex}"
        ),
        subject_digest: manifest_digest.clone(),
    };
    let manifest_object = ReleaseRegistryObjectV1 {
        schema_version: "v1alpha1".to_owned(),
        object_id: typed_id("rob", '9'),
        object_kind: ReleaseRegistryObjectKindV1::ReleaseBundleManifestV2,
        object_digest: manifest_digest,
        object_path: "evidence/release-manifest.toml".to_owned(),
    };
    validate_release_bundle_manifest_v2_binding(
        std::slice::from_ref(&manifest_subject),
        std::slice::from_ref(&manifest_object),
        manifest_bytes,
    )
    .unwrap();
    for (subjects, objects, bytes) in [
        (
            Vec::new(),
            vec![manifest_object.clone()],
            manifest_bytes.as_slice(),
        ),
        (
            vec![manifest_subject.clone(), manifest_subject.clone()],
            vec![manifest_object.clone()],
            manifest_bytes.as_slice(),
        ),
        (
            vec![manifest_subject.clone()],
            Vec::new(),
            manifest_bytes.as_slice(),
        ),
        (
            vec![manifest_subject.clone()],
            vec![manifest_object.clone(), manifest_object.clone()],
            manifest_bytes.as_slice(),
        ),
        (
            vec![manifest_subject.clone()],
            vec![manifest_object.clone()],
            b"release_manifest_schema_version = \"1\"\n".as_slice(),
        ),
    ] {
        assert_eq!(
            validate_release_bundle_manifest_v2_binding(&subjects, &objects, bytes)
                .unwrap_err()
                .code(),
            "INVALID_RELEASE_WIRE_RECORD"
        );
    }
    let mut wrong_native = manifest_subject.clone();
    wrong_native.native_subject_id = format!(
        "{RELEASE_BUNDLE_MANIFEST_V2_NATIVE_SUBJECT_PREFIX}{}",
        hex('0', 64)
    );
    let mut wrong_subject_digest = manifest_subject.clone();
    wrong_subject_digest.subject_digest = digest('0');
    let mut wrong_object_digest = manifest_object.clone();
    wrong_object_digest.object_digest = digest('0');
    let mut wrong_object_kind = manifest_object.clone();
    wrong_object_kind.object_kind = ReleaseRegistryObjectKindV1::GateSpec;
    for (subject, object) in [
        (wrong_native, manifest_object.clone()),
        (wrong_subject_digest, manifest_object.clone()),
        (manifest_subject.clone(), wrong_object_digest),
        (manifest_subject.clone(), wrong_object_kind),
    ] {
        assert_eq!(
            validate_release_bundle_manifest_v2_binding(&[subject], &[object], manifest_bytes)
                .unwrap_err()
                .code(),
            "INVALID_RELEASE_WIRE_RECORD"
        );
    }
    assert_eq!(
        release_bundle_manifest_v2_digest(&[]).unwrap_err().code(),
        "INVALID_RELEASE_WIRE_RECORD"
    );
    let oversized_manifest = vec![0_u8; 1024 * 1024 + 1];
    assert_eq!(
        release_bundle_manifest_v2_digest(&oversized_manifest)
            .unwrap_err()
            .code(),
        "INVALID_RELEASE_WIRE_RECORD"
    );

    let value = serde_json::to_value(&receipt).unwrap();
    for (pointer, field) in [
        ("", "result"),
        ("", "signature"),
        ("/family_subject", "surprise"),
        ("/family_subject/repositories/0", "commit_signature"),
        ("/evidence_subjects/0", "payload"),
    ] {
        let error =
            decode_release_record::<GateReceiptV1>(&add_unknown(value.clone(), pointer, field))
                .unwrap_err();
        assert_eq!(error.code(), "DOCUMENT_SCHEMA_INVALID", "{pointer}/{field}");
    }
    let pretty = serde_json::to_string_pretty(&receipt).unwrap();
    assert_eq!(
        decode_release_record::<GateReceiptV1>(pretty.as_bytes())
            .unwrap_err()
            .code(),
        "NON_CANONICAL_JSON"
    );
    assert_eq!(
        decode_release_record::<GateReceiptV1>(
            br#"{"schema_version":"v1alpha1","schema_version":"v1alpha1"}"#
        )
        .unwrap_err()
        .code(),
        "DUPLICATE_JSON_KEY"
    );
    let unsafe_integer = String::from_utf8(bytes.clone())
        .unwrap()
        .replace("\"gate_version\":1", "\"gate_version\":9007199254740992");
    assert_eq!(
        decode_release_record::<GateReceiptV1>(unsafe_integer.as_bytes())
            .unwrap_err()
            .code(),
        "UNSAFE_JSON_INTEGER"
    );

    let mut unsorted_family = receipt.clone();
    unsorted_family.family_subject.repositories.swap(0, 1);
    assert_release_error(&unsorted_family);
    let mut incomplete_evidence = receipt.clone();
    incomplete_evidence.evidence_subjects.pop();
    assert_release_error(&incomplete_evidence);
    let mut wrong_native_namespace = receipt.clone();
    wrong_native_namespace.evidence_subjects[0].native_subject_id =
        format!("policy:cnt_{}", hex('1', 64));
    assert_release_error(&wrong_native_namespace);
    let mut repeated_kind = receipt.clone();
    repeated_kind
        .evidence_subjects
        .insert(1, evidence(ReleaseEvidenceKindV1::Environment, '5'));
    repeated_kind.validate_release().unwrap();

    let mut shared_key = signer_policy();
    shared_key.signer_keys[1].public_key = shared_key.signer_keys[0].public_key.clone();
    assert_release_error(&shared_key);
    let mut revoked_after_expiry = signer_policy();
    revoked_after_expiry.signer_keys[0].revoked_at_unix_ms = Some(3_000);
    assert_release_error(&revoked_after_expiry);
    let mut traversal = registry_manifest();
    traversal.entries[0].receipt_path = "../receipt.json".to_owned();
    assert_release_error(&traversal);
    let mut unmanifested = registry_manifest();
    unmanifested.objects.remove(0);
    assert_release_error(&unmanifested);
    let mut overlapping = registry_manifest();
    let mut second = overlapping.entries[0].clone();
    second.gate_receipt_id = typed_id("grc", 'b');
    overlapping.entries.push(second);
    assert_release_error(&overlapping);
    let mut disjoint = registry_manifest();
    let mut evolution = disjoint.entries[0].clone();
    evolution.profile_ids = vec!["evolution-v1".to_owned()];
    disjoint.entries.insert(0, evolution);
    disjoint.validate_release().unwrap();

    let mut unsorted_graph = graph.clone();
    unsorted_graph.profiles.swap(0, 1);
    assert_release_error(&unsorted_graph);
    let mut unknown_dependency = graph.clone();
    unknown_dependency.profiles[0].dependency_profile_ids = vec!["unknown-v1".to_owned()];
    assert_release_error(&unknown_dependency);
    let mut cyclic = graph.clone();
    cyclic.profiles[1].dependency_profile_ids = vec!["evolution-v1".to_owned()];
    assert_release_error(&cyclic);
    let graph_value = serde_json::to_value(&graph).unwrap();
    assert_eq!(
        decode_release_record::<ReleaseProfileGraphV1>(&add_unknown(
            graph_value,
            "/profiles/0",
            "outcome"
        ))
        .unwrap_err()
        .code(),
        "DOCUMENT_SCHEMA_INVALID"
    );

    let mut stale_request = request.clone();
    stale_request.gate_policy_digest = digest('f');
    assert_eq!(
        validate_release_bindings(&graph, &spec, &stale_request, &bound_receipt)
            .unwrap_err()
            .code(),
        "INVALID_RELEASE_WIRE_RECORD"
    );
    let mut stale_receipt = bound_receipt.clone();
    stale_receipt.profile_ids = vec!["evolution-v1".to_owned()];
    assert_eq!(
        validate_release_bindings(&graph, &spec, &request, &stale_receipt)
            .unwrap_err()
            .code(),
        "INVALID_RELEASE_WIRE_RECORD"
    );
    let mut replay = replay_state();
    replay.bindings.push(replay.bindings[0].clone());
    assert_release_error(&replay);
    let mut pre_restore_replay = replay_state();
    pre_restore_replay.restore_epoch = 0;
    assert_release_error(&pre_restore_replay);
    let mut pre_restore_time = trusted_time();
    pre_restore_time.restore_epoch = 0;
    assert_release_error(&pre_restore_time);

    let generated_rust =
        fs::read_to_string(root().join("contracts/generated/rust/schema_bundle.rs")).unwrap();
    assert!(generated_rust.contains("ReleaseBundleManifestV2"));
    let rust_gate = generated_rust
        .split("pub struct GateReceiptV1")
        .nth(1)
        .unwrap()
        .split("pub struct GraphDeltaV1")
        .next()
        .unwrap();
    assert!(!rust_gate.contains("serde_json::Value"));
    assert!(!rust_gate.contains("pub result:"));
    assert!(!rust_gate.contains("pub signature:"));
    let rust_repository = generated_rust
        .split("pub struct ReleaseRepositorySubjectV1")
        .nth(1)
        .unwrap()
        .split("pub struct ReleaseSignerKeyV1")
        .next()
        .unwrap();
    assert!(rust_repository.contains("pub source_subject_digest: String"));
    assert!(!rust_repository.contains("dependency_lock_digest"));
    assert!(!rust_repository.contains("artifact_manifest_digest"));

    let generated_ts =
        fs::read_to_string(root().join("contracts/generated/typescript/schemaBundle.ts")).unwrap();
    assert!(generated_ts.contains("\"release-bundle-manifest-v2\""));
    let ts_gate = generated_ts
        .split("export interface GateReceiptV1")
        .nth(1)
        .unwrap()
        .split("export interface GraphDeltaV1")
        .next()
        .unwrap();
    assert!(ts_gate.contains("family_subject: ReleaseFamilySubjectV1;"));
    assert!(ts_gate.contains("evidence_subjects: ReleaseEvidenceSubjectV1[];"));
    assert!(!ts_gate.contains("Record<string, unknown>"));
    assert!(!ts_gate.contains("result:"));
    assert!(!ts_gate.contains("signature:"));
    for record in [
        "ReleaseGateSpecV1",
        "ReleaseGateVerificationRequestV1",
        "ReleaseProfileGraphV1",
        "ReleaseProfileNodeV1",
        "ReleaseRegistryObjectV1",
    ] {
        let body = generated_ts
            .split(&format!("export interface {record}"))
            .nth(1)
            .unwrap()
            .split("export interface ")
            .next()
            .unwrap();
        assert!(!body.contains("Record<string, unknown>"), "{record}");
    }

    let schema_bundle: Value = serde_json::from_slice(
        &fs::read(root().join("contracts/v1alpha1/schema-bundle.json")).unwrap(),
    )
    .unwrap();
    let gate_schema = &schema_bundle["schemas"]["GateReceiptV1"];
    assert_eq!(gate_schema["additionalProperties"], false);
    assert_eq!(
        gate_schema["properties"]["family_subject"]["$ref"],
        "#/schemas/ReleaseFamilySubjectV1"
    );
    assert!(gate_schema["properties"].get("result").is_none());
    assert!(gate_schema["properties"].get("signature").is_none());
    assert_eq!(
        schema_bundle["schemas"]["ReleaseRegistryObjectV1"]["properties"]["object_kind"]["enum"],
        json!([
            "gate-receipt",
            "gate-receipt-signature",
            "gate-spec",
            "profile-graph",
            "release-bundle-manifest-v2",
            "signer-policy",
            "trusted-time-observation",
            "trusted-time-signature",
            "verification-request"
        ])
    );
    assert_eq!(
        schema_bundle["schemas"]["ReleaseEvidenceSubjectV1"]["properties"]["native_subject_id"]["pattern"],
        "^[a-z][a-z0-9-]{0,63}:[a-z][a-z0-9-]{1,31}_[0-9a-f]{64}$"
    );
    assert_eq!(
        schema_bundle["schemas"]["ReleaseProfileNodeV1"]["properties"]["gate_ids"]["items"]["pattern"],
        "^release\\.[a-z0-9][a-z0-9._-]{0,119}$"
    );
}
