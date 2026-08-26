use std::{fs, path::Path};

use bullet_wire::{
    RELEASE_GATE_RECEIPT_SIGNATURE_DOMAIN, RELEASE_REGISTRY_OBJECT_DIGEST_DOMAIN,
    decode_canonical_value, hash_framed_bytes, release_bundle_manifest_v2_digest,
    v1alpha1::{ReleaseEvidenceKindV1, ReleaseEvidenceSubjectV1, ReleaseReceiptKindV1},
};

use super::fixture::{
    RegistryMutation, assert_registry_rejected_with, mutate_registry_manifest, rewrite_registry,
    write_canonical, write_structural_registry,
};

const TRANSACTION_CORE_KINDS: [ReleaseEvidenceKindV1; 8] = [
    ReleaseEvidenceKindV1::Candidate,
    ReleaseEvidenceKindV1::Evidence,
    ReleaseEvidenceKindV1::ProofBundle,
    ReleaseEvidenceKindV1::Effect,
    ReleaseEvidenceKindV1::Check,
    ReleaseEvidenceKindV1::Integration,
    ReleaseEvidenceKindV1::Observation,
    ReleaseEvidenceKindV1::AuditAnchor,
];

const TRANSACTION_COMMON_KINDS: [ReleaseEvidenceKindV1; 4] = [
    ReleaseEvidenceKindV1::Environment,
    ReleaseEvidenceKindV1::Policy,
    ReleaseEvidenceKindV1::Schema,
    ReleaseEvidenceKindV1::Toolchain,
];

fn add_unreferenced_signature_fillers(registry: &Path, count: usize) {
    let manifest_path = registry.join("registry-manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    let objects = manifest["objects"].as_array_mut().unwrap();
    let insertion = objects
        .iter()
        .position(|object| object["object_kind"] == "trusted-time-signature")
        .unwrap();
    let mut fillers = Vec::with_capacity(count);
    for index in 0..count {
        let bytes = vec![index as u8; 1024 * 1024];
        let path = format!("signatures/filler-{index:02}.sig");
        fs::create_dir_all(registry.join("signatures")).unwrap();
        fs::write(registry.join(&path), &bytes).unwrap();
        fillers.push(serde_json::json!({
            "schema_version": "v1alpha1",
            "object_id": format!("rob_{:064x}", 0x100usize + index),
            "object_kind": "trusted-time-signature",
            "object_digest": kind_framed_digest(b"trusted-time-signature", &bytes),
            "object_path": path,
        }));
    }
    objects.splice(insertion..insertion, fillers);
    write_canonical(&manifest_path, &manifest);
}

fn kind_framed_digest(kind: &[u8], bytes: &[u8]) -> String {
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

pub(super) fn assert_hostile_registries(registry: &Path) {
    manifest_binding_refusals(registry);
    transaction_kind_refusals(registry);
    kind_and_policy_refusals(registry);
    existing_subject_refusals(registry);
    signature_and_budget_refusals(registry);
}

fn evidence_kind_name(kind: ReleaseEvidenceKindV1) -> String {
    serde_json::to_value(kind)
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned()
}

fn transaction_subject(kind: ReleaseEvidenceKindV1, identity: usize) -> ReleaseEvidenceSubjectV1 {
    let hex = format!("{identity:064x}");
    ReleaseEvidenceSubjectV1 {
        schema_version: "v1alpha1".to_owned(),
        subject_kind: kind,
        subject_id: format!("cnt_{hex}"),
        native_subject_id: format!("{}:cnt_{hex}", evidence_kind_name(kind)),
        subject_digest: format!("blake3:{hex}"),
    }
}

fn sort_kinds(kinds: &mut [ReleaseEvidenceKindV1]) {
    kinds.sort_by_key(|kind| evidence_kind_name(*kind));
}

fn sort_subjects(subjects: &mut [ReleaseEvidenceSubjectV1]) {
    subjects.sort_by_key(|subject| {
        (
            evidence_kind_name(subject.subject_kind),
            subject.subject_id.clone(),
        )
    });
}

fn install_complete_transaction(records: &mut RegistryMutation) {
    let mut kinds = TRANSACTION_CORE_KINDS
        .into_iter()
        .chain(TRANSACTION_COMMON_KINDS)
        .collect::<Vec<_>>();
    sort_kinds(&mut kinds);
    let mut subjects = kinds
        .iter()
        .copied()
        .enumerate()
        .map(|(index, kind)| transaction_subject(kind, index + 1))
        .collect::<Vec<_>>();
    sort_subjects(&mut subjects);

    records.spec.receipt_kind = ReleaseReceiptKindV1::Transaction;
    records.spec.required_evidence_kinds = kinds;
    records.request.receipt_kind = ReleaseReceiptKindV1::Transaction;
    records.request.evidence_subjects.clone_from(&subjects);
    records.receipt.receipt_kind = ReleaseReceiptKindV1::Transaction;
    records.receipt.evidence_subjects = subjects;
}

fn mutate_receipt_json(registry: &Path, mutation: impl FnOnce(&mut serde_json::Value)) {
    let path = registry.join("receipts/provider-codex.json");
    let mut receipt = decode_canonical_value(&fs::read(&path).unwrap()).unwrap();
    mutation(&mut receipt);
    write_canonical(&path, &receipt);
}

fn transaction_kind_refusals(registry: &Path) {
    write_structural_registry(registry);
    rewrite_registry(registry, install_complete_transaction);
    assert_registry_rejected_with(
        registry,
        "provider-codex",
        Some("gate/profile coverage differs"),
    );

    for missing in TRANSACTION_CORE_KINDS {
        write_structural_registry(registry);
        rewrite_registry(registry, |records| {
            install_complete_transaction(records);
            records
                .spec
                .required_evidence_kinds
                .retain(|kind| *kind != missing);
            records
                .request
                .evidence_subjects
                .retain(|subject| subject.subject_kind != missing);
            records
                .receipt
                .evidence_subjects
                .retain(|subject| subject.subject_kind != missing);
        });
        assert_registry_rejected_with(
            registry,
            "provider-codex",
            Some(&format!(
                "receipt kind transaction is missing required evidence {}",
                evidence_kind_name(missing)
            )),
        );
    }

    for (index, repeated) in TRANSACTION_CORE_KINDS.into_iter().enumerate() {
        write_structural_registry(registry);
        rewrite_registry(registry, |records| {
            install_complete_transaction(records);
            let duplicate = transaction_subject(repeated, index + 32);
            records.request.evidence_subjects.push(duplicate.clone());
            sort_subjects(&mut records.request.evidence_subjects);
            records.receipt.evidence_subjects.push(duplicate);
            sort_subjects(&mut records.receipt.evidence_subjects);
        });
        assert_registry_rejected_with(
            registry,
            "provider-codex",
            Some(&format!(
                "receipt kind transaction repeats required evidence {}",
                evidence_kind_name(repeated)
            )),
        );
    }

    write_structural_registry(registry);
    rewrite_registry(registry, |records| {
        install_complete_transaction(records);
        for subject in records
            .request
            .evidence_subjects
            .iter_mut()
            .chain(records.receipt.evidence_subjects.iter_mut())
            .filter(|subject| subject.subject_kind == ReleaseEvidenceKindV1::Candidate)
        {
            *subject = transaction_subject(ReleaseEvidenceKindV1::Jeryu, 63);
        }
        records
            .spec
            .required_evidence_kinds
            .retain(|kind| *kind != ReleaseEvidenceKindV1::Candidate);
        records
            .spec
            .required_evidence_kinds
            .push(ReleaseEvidenceKindV1::Jeryu);
        sort_kinds(&mut records.spec.required_evidence_kinds);
        sort_subjects(&mut records.request.evidence_subjects);
        sort_subjects(&mut records.receipt.evidence_subjects);
    });
    assert_registry_rejected_with(
        registry,
        "provider-codex",
        Some("receipt kind transaction is missing required evidence candidate"),
    );

    write_structural_registry(registry);
    mutate_receipt_json(registry, |receipt| {
        receipt["receipt_kind"] = "future-receipt".into();
    });
    assert_registry_rejected_with(registry, "provider-codex", Some("invalid gate receipt"));

    write_structural_registry(registry);
    mutate_receipt_json(registry, |receipt| {
        receipt["evidence_subjects"][0]["subject_kind"] = "future-evidence".into();
    });
    assert_registry_rejected_with(registry, "provider-codex", Some("invalid gate receipt"));
}

fn manifest_binding_refusals(registry: &Path) {
    write_structural_registry(registry);
    mutate_registry_manifest(registry, |manifest| {
        manifest["objects"]
            .as_array_mut()
            .unwrap()
            .retain(|object| object["object_kind"] != "release-bundle-manifest-v2");
    });
    assert_registry_rejected_with(
        registry,
        "provider-codex",
        Some("requires exactly one registry object"),
    );

    write_structural_registry(registry);
    rewrite_registry(registry, |records| {
        for evidence in records
            .request
            .evidence_subjects
            .iter_mut()
            .chain(records.receipt.evidence_subjects.iter_mut())
            .filter(|subject| subject.subject_kind == ReleaseEvidenceKindV1::Artifact)
        {
            evidence.native_subject_id = format!("artifact:release-manifest-v2_{}", "0".repeat(64));
        }
    });
    assert_registry_rejected_with(
        registry,
        "provider-codex",
        Some("do not share one exact digest binding"),
    );

    write_structural_registry(registry);
    let duplicate_bytes = b"different duplicate manifest";
    let duplicate_digest = release_bundle_manifest_v2_digest(duplicate_bytes).unwrap();
    fs::write(
        registry.join("artifacts/release-manifest-v2-duplicate.toml"),
        duplicate_bytes,
    )
    .unwrap();
    mutate_registry_manifest(registry, |manifest| {
        let objects = manifest["objects"].as_array_mut().unwrap();
        let insertion = objects
            .iter()
            .position(|object| object["object_kind"] == "signer-policy")
            .unwrap();
        objects.insert(
            insertion,
            serde_json::json!({
                "schema_version": "v1alpha1",
                "object_id": format!("rob_{}a", "9".repeat(63)),
                "object_kind": "release-bundle-manifest-v2",
                "object_digest": duplicate_digest,
                "object_path": "artifacts/release-manifest-v2-duplicate.toml",
            }),
        );
    });
    assert_registry_rejected_with(
        registry,
        "provider-codex",
        Some("requires exactly one registry object"),
    );
}

fn kind_and_policy_refusals(registry: &Path) {
    write_structural_registry(registry);
    rewrite_registry(registry, |records| {
        records.spec.receipt_kind = ReleaseReceiptKindV1::ProfileClosure;
        records.request.receipt_kind = records.spec.receipt_kind;
        records.receipt.receipt_kind = records.spec.receipt_kind;
    });
    assert_registry_rejected_with(
        registry,
        "provider-codex",
        Some("gate/profile coverage differs"),
    );

    write_structural_registry(registry);
    rewrite_registry(registry, |records| {
        records.closure_spec.receipt_kind = ReleaseReceiptKindV1::Provider;
        records.closure_request.receipt_kind = records.closure_spec.receipt_kind;
        records.closure_receipt.receipt_kind = records.closure_spec.receipt_kind;
    });
    assert_registry_rejected_with(
        registry,
        "provider-codex",
        Some("gate/profile coverage differs"),
    );

    write_structural_registry(registry);
    rewrite_registry(registry, |records| {
        records.policy.activates_at_unix_ms = 250;
    });
    assert_registry_rejected_with(registry, "provider-codex", Some("windows are incoherent"));

    write_structural_registry(registry);
    rewrite_registry(registry, |records| {
        records.policy.expires_at_unix_ms = 725;
    });
    assert_registry_rejected_with(registry, "provider-codex", Some("windows are incoherent"));
}

fn existing_subject_refusals(registry: &Path) {
    write_structural_registry(registry);
    rewrite_registry(registry, |records| {
        records.manifest.registry_signer_key_id = "attestor".to_owned();
    });
    assert_registry_rejected_with(
        registry,
        "provider-codex",
        Some("signer policy subjects are inconsistent"),
    );

    write_structural_registry(registry);
    rewrite_registry(registry, |records| {
        records.time.trusted_time_key_id = "registry".to_owned();
    });
    assert_registry_rejected_with(
        registry,
        "provider-codex",
        Some("trusted-time observation names a key outside"),
    );

    write_structural_registry(registry);
    rewrite_registry(registry, |records| {
        records.receipt.attestor_key_id = "time".to_owned();
    });
    assert_registry_rejected_with(
        registry,
        "provider-codex",
        Some("signer key with the wrong policy role"),
    );

    write_structural_registry(registry);
    rewrite_registry(registry, |records| {
        records.request.family_subject.family = "other-family".to_owned();
        records.receipt.family_subject.family = "other-family".to_owned();
        records.time.family = "other-family".to_owned();
    });
    assert_registry_rejected_with(
        registry,
        "provider-codex",
        Some("release record names the wrong family"),
    );

    write_structural_registry(registry);
    rewrite_registry(registry, |records| {
        records.graph.profiles[0].gate_ids = vec!["release.provider.cursor".to_owned()];
        records.spec.gate_id = "release.provider.cursor".to_owned();
        records.request.gate_id = "release.provider.cursor".to_owned();
        records.receipt.gate_id = "release.provider.cursor".to_owned();
    });
    assert_registry_rejected_with(
        registry,
        "provider-codex",
        Some("changes the declared gates"),
    );

    write_structural_registry(registry);
    rewrite_registry(registry, |records| {
        records.receipt.expires_at_unix_ms = records.request.expires_at_unix_ms + 1;
    });
    assert_registry_rejected_with(registry, "provider-codex", Some("windows are incoherent"));

    write_structural_registry(registry);
    rewrite_registry(registry, |records| {
        records.policy.signer_keys[1].revoked_at_unix_ms = Some(250);
    });
    assert_registry_rejected_with(registry, "provider-codex", Some("attestor key lifecycle"));

    write_structural_registry(registry);
    mutate_registry_manifest(registry, |manifest| {
        manifest["entries"] = serde_json::json!([]);
    });
    assert_registry_rejected_with(
        registry,
        "provider-codex",
        Some("gate/profile coverage differs"),
    );
}

fn signature_and_budget_refusals(registry: &Path) {
    write_structural_registry(registry);
    let signature = fs::read(registry.join("signatures/provider-codex.sig")).unwrap();
    let legacy_digest = format!(
        "blake3:{}",
        hash_framed_bytes(RELEASE_GATE_RECEIPT_SIGNATURE_DOMAIN, &signature)
            .unwrap()
            .to_hex()
    );
    mutate_registry_manifest(registry, |manifest| {
        manifest["objects"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|object| object["object_path"] == "signatures/provider-codex.sig")
            .unwrap()["object_digest"] = legacy_digest.clone().into();
        manifest["entries"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|entry| entry["gate_id"] == "release.provider.codex")
            .unwrap()["receipt_signature_digest"] = legacy_digest.into();
    });
    assert_registry_rejected_with(
        registry,
        "provider-codex",
        Some("digest differs from its manifest binding"),
    );

    write_structural_registry(registry);
    let signature = fs::read(registry.join("signatures/provider-codex.sig")).unwrap();
    fs::write(registry.join("time/provider-codex.sig"), &signature).unwrap();
    let wrong_kind_digest = kind_framed_digest(b"gate-receipt-signature", &signature);
    mutate_registry_manifest(registry, |manifest| {
        manifest["objects"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|object| object["object_path"] == "time/provider-codex.sig")
            .unwrap()["object_digest"] = wrong_kind_digest.clone().into();
        manifest["entries"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|entry| entry["gate_id"] == "release.provider.codex")
            .unwrap()["trusted_time_signature_digest"] = wrong_kind_digest.into();
    });
    assert_registry_rejected_with(
        registry,
        "provider-codex",
        Some("digest differs from its manifest binding"),
    );

    write_structural_registry(registry);
    add_unreferenced_signature_fillers(registry, 15);
    assert_registry_rejected_with(
        registry,
        "provider-codex",
        Some("registry contains an unreferenced object"),
    );

    write_structural_registry(registry);
    add_unreferenced_signature_fillers(registry, 16);
    assert_registry_rejected_with(
        registry,
        "provider-codex",
        Some("aggregate byte budget is exhausted"),
    );
}
