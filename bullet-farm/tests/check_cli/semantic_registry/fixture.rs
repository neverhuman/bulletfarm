use std::{fs, path::Path, process::Output};

use bullet_wire::{
    RELEASE_GATE_RECEIPT_DIGEST_DOMAIN, RELEASE_GATE_SPEC_DIGEST_DOMAIN,
    RELEASE_PROFILE_GRAPH_DIGEST_DOMAIN, RELEASE_REGISTRY_OBJECT_DIGEST_DOMAIN,
    RELEASE_SIGNER_POLICY_DIGEST_DOMAIN, RELEASE_TRUSTED_TIME_DIGEST_DOMAIN,
    RELEASE_VERIFICATION_REQUEST_DIGEST_DOMAIN, canonical_json, hash_canonical, hash_framed_bytes,
    release_bundle_manifest_v2_digest,
    v1alpha1::{
        GateReceiptV1, ReleaseEvidenceKindV1, ReleaseEvidenceSubjectV1, ReleaseFamilySubjectV1,
        ReleaseGateSpecV1, ReleaseGateVerificationRequestV1, ReleaseProfileGraphV1,
        ReleaseProfileNodeV1, ReleaseReceiptKindV1, ReleaseRegistryEntryV1,
        ReleaseRegistryManifestV1, ReleaseRegistryObjectKindV1, ReleaseRegistryObjectV1,
        ReleaseRepositoryNameV1, ReleaseRepositorySubjectV1, ReleaseSignerKeyV1,
        ReleaseSignerPolicyV1, ReleaseSignerRoleV1, TrustedTimeObservationV1,
    },
};

use super::super::command;

fn release_hex(character: char, length: usize) -> String {
    std::iter::repeat_n(character, length).collect()
}

pub(super) fn release_id(prefix: &str, character: char) -> String {
    format!("{prefix}_{}", release_hex(character, 64))
}

fn tagged_digest(character: char) -> String {
    format!("blake3:{}", release_hex(character, 64))
}

#[path = "fixture/family_anchor.rs"]
mod family_anchor;
pub(super) use family_anchor::{
    fixture_family_lock_path, fixture_hub, rewrite_family_anchor_policy,
    write_matching_family_anchor,
};

fn canonical_digest<T: serde::Serialize>(domain: &str, value: &T) -> String {
    format!("blake3:{}", hash_canonical(domain, value).unwrap().to_hex())
}

fn registry_blob_digest(kind: &str, bytes: &[u8]) -> String {
    let kind = kind.as_bytes();
    let mut framed = Vec::with_capacity(16 + kind.len() + bytes.len());
    framed.extend_from_slice(&(kind.len() as u64).to_le_bytes());
    framed.extend_from_slice(kind);
    framed.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    framed.extend_from_slice(bytes);
    format!(
        "blake3:{}",
        hash_framed_bytes(RELEASE_REGISTRY_OBJECT_DIGEST_DOMAIN, &framed)
            .unwrap()
            .to_hex()
    )
}

pub(super) fn write_canonical<T: serde::Serialize>(path: &Path, value: &T) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, canonical_json(value).unwrap()).unwrap();
}

fn registry_repository(
    repository: ReleaseRepositoryNameV1,
    character: char,
) -> ReleaseRepositorySubjectV1 {
    ReleaseRepositorySubjectV1 {
        schema_version: "v1alpha1".to_owned(),
        repository,
        tag: "v1.0.0".to_owned(),
        commit_oid: format!("sha1:{}", release_hex(character, 40)),
        tree_oid: format!("sha1:{}", release_hex(character, 40)),
        release_signing_identity: format!(
            "release-{character}@bullet.farm|ed25519|SHA256:{}",
            release_hex(character.to_ascii_uppercase(), 24)
        ),
        source_subject_digest: tagged_digest(character),
    }
}

fn registry_evidence(
    subject_kind: ReleaseEvidenceKindV1,
    character: char,
) -> ReleaseEvidenceSubjectV1 {
    let namespace = serde_json::to_value(subject_kind)
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned();
    ReleaseEvidenceSubjectV1 {
        schema_version: "v1alpha1".to_owned(),
        subject_kind,
        subject_id: release_id("cnt", character),
        native_subject_id: format!("{namespace}:cnt_{}", release_hex(character, 64)),
        subject_digest: tagged_digest(character),
    }
}

fn registry_signer(key_id: &str, role: ReleaseSignerRoleV1, character: char) -> ReleaseSignerKeyV1 {
    ReleaseSignerKeyV1 {
        schema_version: "v1alpha1".to_owned(),
        key_id: key_id.to_owned(),
        role,
        signing_identity: format!(
            "{key_id}@bullet.farm|ed25519|SHA256:{}",
            release_hex(character.to_ascii_uppercase(), 24)
        ),
        public_key: format!(
            "ssh-ed25519 {}",
            release_hex(character.to_ascii_uppercase(), 44)
        ),
        activates_at_unix_ms: 1,
        expires_at_unix_ms: 900,
        revoked_at_unix_ms: None,
        retain_until_unix_ms: 1_000,
    }
}

fn registry_object(
    object_kind: ReleaseRegistryObjectKindV1,
    character: char,
    object_digest: String,
    object_path: &str,
) -> ReleaseRegistryObjectV1 {
    ReleaseRegistryObjectV1 {
        schema_version: "v1alpha1".to_owned(),
        object_id: release_id("rob", character),
        object_kind,
        object_digest,
        object_path: object_path.to_owned(),
    }
}

#[path = "fixture/structural.rs"]
mod structural;

pub(super) use structural::write_structural_registry;
pub(super) fn mutate_registry_manifest(root: &Path, mutation: impl FnOnce(&mut serde_json::Value)) {
    let path = root.join("registry-manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    mutation(&mut manifest);
    write_canonical(&path, &manifest);
}

pub(super) struct RegistryMutation {
    pub(super) graph: ReleaseProfileGraphV1,
    pub(super) spec: ReleaseGateSpecV1,
    pub(super) closure_spec: ReleaseGateSpecV1,
    pub(super) request: ReleaseGateVerificationRequestV1,
    pub(super) closure_request: ReleaseGateVerificationRequestV1,
    pub(super) receipt: GateReceiptV1,
    pub(super) closure_receipt: GateReceiptV1,
    pub(super) policy: ReleaseSignerPolicyV1,
    pub(super) time: TrustedTimeObservationV1,
    pub(super) closure_time: TrustedTimeObservationV1,
    pub(super) manifest: ReleaseRegistryManifestV1,
}

pub(super) fn rewrite_registry(root: &Path, mutation: impl FnOnce(&mut RegistryMutation)) {
    rewrite_registry_inner(root, mutation, true);
}

fn rewrite_registry_inner(
    root: &Path,
    mutation: impl FnOnce(&mut RegistryMutation),
    refresh_family_anchor: bool,
) {
    fn load<T: serde::de::DeserializeOwned>(root: &Path, path: &str) -> T {
        serde_json::from_slice(&fs::read(root.join(path)).unwrap()).unwrap()
    }

    let mut records = RegistryMutation {
        graph: load(root, "profiles/graph.json"),
        spec: load(root, "specs/provider-codex.json"),
        closure_spec: load(root, "specs/provider-codex-closure.json"),
        request: load(root, "requests/provider-codex.json"),
        closure_request: load(root, "requests/provider-codex-closure.json"),
        receipt: load(root, "receipts/provider-codex.json"),
        closure_receipt: load(root, "receipts/provider-codex-closure.json"),
        policy: load(root, "policy/signers.json"),
        time: load(root, "time/provider-codex.json"),
        closure_time: load(root, "time/provider-codex-closure.json"),
        manifest: load(root, "registry-manifest.json"),
    };
    mutation(&mut records);

    let policy_digest = canonical_digest(RELEASE_SIGNER_POLICY_DIGEST_DOMAIN, &records.policy);
    if refresh_family_anchor {
        let lock_digest = write_matching_family_anchor(&policy_digest);
        records
            .request
            .family_subject
            .family_lock_digest
            .clone_from(&lock_digest);
        records
            .closure_request
            .family_subject
            .family_lock_digest
            .clone_from(&lock_digest);
        records
            .receipt
            .family_subject
            .family_lock_digest
            .clone_from(&lock_digest);
        records
            .closure_receipt
            .family_subject
            .family_lock_digest
            .clone_from(&lock_digest);
        records.manifest.family_lock_digest.clone_from(&lock_digest);
    }

    let graph_digest = canonical_digest(RELEASE_PROFILE_GRAPH_DIGEST_DOMAIN, &records.graph);
    let spec_digest = canonical_digest(RELEASE_GATE_SPEC_DIGEST_DOMAIN, &records.spec);
    records
        .request
        .profile_graph_digest
        .clone_from(&graph_digest);
    records.request.gate_spec_digest.clone_from(&spec_digest);
    let request_digest =
        canonical_digest(RELEASE_VERIFICATION_REQUEST_DIGEST_DOMAIN, &records.request);
    records.receipt.request_digest.clone_from(&request_digest);
    records
        .receipt
        .profile_graph_digest
        .clone_from(&graph_digest);
    records.receipt.gate_spec_digest.clone_from(&spec_digest);
    let receipt_digest = canonical_digest(RELEASE_GATE_RECEIPT_DIGEST_DOMAIN, &records.receipt);
    let closure_spec_digest =
        canonical_digest(RELEASE_GATE_SPEC_DIGEST_DOMAIN, &records.closure_spec);
    records
        .closure_request
        .profile_graph_digest
        .clone_from(&graph_digest);
    records
        .closure_request
        .gate_spec_digest
        .clone_from(&closure_spec_digest);
    let closure_request_digest = canonical_digest(
        RELEASE_VERIFICATION_REQUEST_DIGEST_DOMAIN,
        &records.closure_request,
    );
    records
        .closure_receipt
        .request_digest
        .clone_from(&closure_request_digest);
    records
        .closure_receipt
        .profile_graph_digest
        .clone_from(&graph_digest);
    records
        .closure_receipt
        .gate_spec_digest
        .clone_from(&closure_spec_digest);
    let closure_receipt_digest =
        canonical_digest(RELEASE_GATE_RECEIPT_DIGEST_DOMAIN, &records.closure_receipt);
    records.time.receipt_digest.clone_from(&receipt_digest);
    records.time.signer_policy_digest.clone_from(&policy_digest);
    let time_digest = canonical_digest(RELEASE_TRUSTED_TIME_DIGEST_DOMAIN, &records.time);
    records
        .closure_time
        .receipt_digest
        .clone_from(&closure_receipt_digest);
    records
        .closure_time
        .signer_policy_digest
        .clone_from(&policy_digest);
    let closure_time_digest =
        canonical_digest(RELEASE_TRUSTED_TIME_DIGEST_DOMAIN, &records.closure_time);

    records
        .manifest
        .profile_graph_digest
        .clone_from(&graph_digest);
    records
        .manifest
        .signer_policy_digest
        .clone_from(&policy_digest);
    let entry = records
        .manifest
        .entries
        .iter_mut()
        .find(|entry| entry.gate_id == "release.provider.codex")
        .unwrap();
    entry.gate_id.clone_from(&records.receipt.gate_id);
    entry.profile_ids.clone_from(&records.receipt.profile_ids);
    entry
        .gate_receipt_id
        .clone_from(&records.receipt.gate_receipt_id);
    entry.receipt_digest.clone_from(&receipt_digest);
    entry.trusted_time_digest.clone_from(&time_digest);
    let closure_entry = records
        .manifest
        .entries
        .iter_mut()
        .find(|entry| entry.gate_id == "release.profile.provider-codex")
        .unwrap();
    closure_entry
        .gate_id
        .clone_from(&records.closure_receipt.gate_id);
    closure_entry
        .profile_ids
        .clone_from(&records.closure_receipt.profile_ids);
    closure_entry
        .gate_receipt_id
        .clone_from(&records.closure_receipt.gate_receipt_id);
    closure_entry
        .receipt_digest
        .clone_from(&closure_receipt_digest);
    closure_entry
        .trusted_time_digest
        .clone_from(&closure_time_digest);
    for object in &mut records.manifest.objects {
        object.object_digest = match object.object_path.as_str() {
            "receipts/provider-codex.json" => receipt_digest.clone(),
            "receipts/provider-codex-closure.json" => closure_receipt_digest.clone(),
            "specs/provider-codex.json" => spec_digest.clone(),
            "specs/provider-codex-closure.json" => closure_spec_digest.clone(),
            "profiles/graph.json" => graph_digest.clone(),
            "policy/signers.json" => policy_digest.clone(),
            "time/provider-codex.json" => time_digest.clone(),
            "time/provider-codex-closure.json" => closure_time_digest.clone(),
            "requests/provider-codex.json" => request_digest.clone(),
            "requests/provider-codex-closure.json" => closure_request_digest.clone(),
            _ => continue,
        };
    }

    write_canonical(&root.join("profiles/graph.json"), &records.graph);
    write_canonical(&root.join("specs/provider-codex.json"), &records.spec);
    write_canonical(
        &root.join("specs/provider-codex-closure.json"),
        &records.closure_spec,
    );
    write_canonical(&root.join("requests/provider-codex.json"), &records.request);
    write_canonical(
        &root.join("requests/provider-codex-closure.json"),
        &records.closure_request,
    );
    write_canonical(&root.join("receipts/provider-codex.json"), &records.receipt);
    write_canonical(
        &root.join("receipts/provider-codex-closure.json"),
        &records.closure_receipt,
    );
    write_canonical(&root.join("policy/signers.json"), &records.policy);
    write_canonical(&root.join("time/provider-codex.json"), &records.time);
    write_canonical(
        &root.join("time/provider-codex-closure.json"),
        &records.closure_time,
    );
    write_canonical(&root.join("registry-manifest.json"), &records.manifest);
}

pub(super) fn profiled_registry_output(registry: &Path, profile: &str) -> Output {
    profiled_registry_output_for_hub(&fixture_hub(), registry, profile)
}

pub(super) fn profiled_registry_output_for_hub(
    hub: &Path,
    registry: &Path,
    profile: &str,
) -> Output {
    command(&[
        "--root",
        hub.to_str().unwrap(),
        "check",
        "release",
        "--profile",
        profile,
        "--receipts",
        registry.to_str().unwrap(),
        "--json",
    ])
}

pub(super) fn cleanup_registry_fixture(registry: &Path) {
    if registry.exists() {
        fs::remove_dir_all(registry).unwrap();
    }
    let hub = fixture_hub();
    if hub.exists() {
        fs::remove_dir_all(hub).unwrap();
    }
}

pub(super) fn assert_registry_rejected(registry: &Path, profile: &str) {
    assert_registry_rejected_with(registry, profile, None);
}

pub(super) fn assert_registry_rejected_with(
    registry: &Path,
    profile: &str,
    expected_detail: Option<&str>,
) {
    let output = profiled_registry_output(registry, profile);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let receipt_gate = report["gates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|gate| gate["id"] == "release.receipt-contracts")
        .unwrap();
    assert_eq!(receipt_gate["status"], "FAIL");
    if let Some(expected) = expected_detail {
        assert!(
            receipt_gate["detail"].as_str().unwrap().contains(expected),
            "unexpected rejection detail: {}",
            receipt_gate["detail"]
        );
    }
    assert!(
        report["gates"]
            .as_array()
            .unwrap()
            .iter()
            .all(|gate| gate["status"] != "PASS")
    );
}
