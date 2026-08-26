use super::*;

pub(crate) fn write_structural_registry(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    fs::create_dir_all(root).unwrap();
    let release_bundle_manifest = b"release bundle manifest v2 structural fixture";
    let release_bundle_digest = release_bundle_manifest_v2_digest(release_bundle_manifest).unwrap();
    let release_bundle_hex = release_bundle_digest.strip_prefix("blake3:").unwrap();
    let graph = ReleaseProfileGraphV1 {
        schema_version: "v1alpha1".to_owned(),
        profile_graph_id: release_id("rpg", '1'),
        family: "bullet-farm".to_owned(),
        generation: 1,
        profiles: vec![ReleaseProfileNodeV1 {
            schema_version: "v1alpha1".to_owned(),
            profile_id: "provider-codex".to_owned(),
            dependency_profile_ids: Vec::new(),
            gate_ids: vec![
                "release.profile.provider-codex".to_owned(),
                "release.provider.codex".to_owned(),
            ],
        }],
    };
    let graph_digest = canonical_digest(RELEASE_PROFILE_GRAPH_DIGEST_DOMAIN, &graph);
    let spec = ReleaseGateSpecV1 {
        schema_version: "v1alpha1".to_owned(),
        gate_spec_id: release_id("gsp", '1'),
        gate_id: "release.provider.codex".to_owned(),
        gate_version: 1,
        receipt_kind: ReleaseReceiptKindV1::Provider,
        profile_ids: vec!["provider-codex".to_owned()],
        required_evidence_kinds: vec![
            ReleaseEvidenceKindV1::Artifact,
            ReleaseEvidenceKindV1::Environment,
            ReleaseEvidenceKindV1::Policy,
            ReleaseEvidenceKindV1::Provider,
            ReleaseEvidenceKindV1::Schema,
            ReleaseEvidenceKindV1::Toolchain,
        ],
        gate_policy_digest: tagged_digest('a'),
    };
    let spec_digest = canonical_digest(RELEASE_GATE_SPEC_DIGEST_DOMAIN, &spec);
    let family_subject = ReleaseFamilySubjectV1 {
        schema_version: "v1alpha1".to_owned(),
        family: "bullet-farm".to_owned(),
        family_lock_digest: tagged_digest('b'),
        schema_bundle_digest: tagged_digest('c'),
        repositories: vec![
            registry_repository(ReleaseRepositoryNameV1::BulletFarm, '1'),
            registry_repository(ReleaseRepositoryNameV1::BulletGit, '2'),
            registry_repository(ReleaseRepositoryNameV1::BulletKernel, '3'),
            registry_repository(ReleaseRepositoryNameV1::BulletPortal, '4'),
        ],
    };
    let evidence_subjects = vec![
        ReleaseEvidenceSubjectV1 {
            schema_version: "v1alpha1".to_owned(),
            subject_kind: ReleaseEvidenceKindV1::Artifact,
            subject_id: release_id("cnt", '6'),
            native_subject_id: format!("artifact:release-manifest-v2_{release_bundle_hex}"),
            subject_digest: release_bundle_digest.clone(),
        },
        registry_evidence(ReleaseEvidenceKindV1::Environment, '1'),
        registry_evidence(ReleaseEvidenceKindV1::Policy, '2'),
        registry_evidence(ReleaseEvidenceKindV1::Provider, '3'),
        registry_evidence(ReleaseEvidenceKindV1::Schema, '4'),
        registry_evidence(ReleaseEvidenceKindV1::Toolchain, '5'),
    ];
    let request = ReleaseGateVerificationRequestV1 {
        schema_version: "v1alpha1".to_owned(),
        verification_request_id: release_id("rvr", '1'),
        gate_id: spec.gate_id.clone(),
        gate_version: spec.gate_version,
        receipt_kind: spec.receipt_kind,
        profile_ids: spec.profile_ids.clone(),
        evidence_nonce: release_hex('d', 64),
        gate_spec_digest: spec_digest.clone(),
        profile_graph_digest: graph_digest.clone(),
        gate_policy_digest: spec.gate_policy_digest.clone(),
        family_subject: family_subject.clone(),
        evidence_subjects: evidence_subjects.clone(),
        requested_at_unix_ms: 100,
        expires_at_unix_ms: 850,
    };
    let request_digest = canonical_digest(RELEASE_VERIFICATION_REQUEST_DIGEST_DOMAIN, &request);
    let receipt = GateReceiptV1 {
        schema_version: "v1alpha1".to_owned(),
        gate_receipt_id: release_id("grc", '1'),
        gate_id: request.gate_id.clone(),
        gate_version: request.gate_version,
        receipt_kind: request.receipt_kind,
        profile_ids: request.profile_ids.clone(),
        evidence_nonce: request.evidence_nonce.clone(),
        request_digest: request_digest.clone(),
        gate_spec_digest: spec_digest.clone(),
        profile_graph_digest: graph_digest.clone(),
        gate_policy_digest: request.gate_policy_digest.clone(),
        family_subject,
        evidence_subjects,
        attestor_key_id: "attestor".to_owned(),
        started_at_unix_ms: 200,
        completed_at_unix_ms: 300,
        expires_at_unix_ms: 800,
    };
    let receipt_digest = canonical_digest(RELEASE_GATE_RECEIPT_DIGEST_DOMAIN, &receipt);
    let policy = ReleaseSignerPolicyV1 {
        schema_version: "v1alpha1".to_owned(),
        family: "bullet-farm".to_owned(),
        policy_generation: 1,
        activates_at_unix_ms: 1,
        expires_at_unix_ms: 900,
        registry_signer_key_id: "registry".to_owned(),
        trusted_time_key_id: "time".to_owned(),
        signer_keys: vec![
            registry_signer("artifact", ReleaseSignerRoleV1::ArtifactRelease, 'a'),
            registry_signer("attestor", ReleaseSignerRoleV1::GateAttestor, 'b'),
            registry_signer("registry", ReleaseSignerRoleV1::RegistryCurator, 'c'),
            registry_signer("source", ReleaseSignerRoleV1::SourceTag, 'd'),
            registry_signer("time", ReleaseSignerRoleV1::TrustedTime, 'e'),
        ],
    };
    let policy_digest = canonical_digest(RELEASE_SIGNER_POLICY_DIGEST_DOMAIN, &policy);
    let time = TrustedTimeObservationV1 {
        schema_version: "v1alpha1".to_owned(),
        family: "bullet-farm".to_owned(),
        gate_receipt_id: receipt.gate_receipt_id.clone(),
        receipt_digest: receipt_digest.clone(),
        evidence_nonce: receipt.evidence_nonce.clone(),
        signer_policy_digest: policy_digest.clone(),
        observed_at_unix_ms: 350,
        valid_until_unix_ms: 750,
        restore_epoch: 1,
        trusted_time_key_id: "time".to_owned(),
    };
    let time_digest = canonical_digest(RELEASE_TRUSTED_TIME_DIGEST_DOMAIN, &time);
    let receipt_signature = b"untrusted receipt signature";
    let time_signature = b"untrusted trusted-time signature";
    let receipt_signature_digest =
        registry_blob_digest("gate-receipt-signature", receipt_signature);
    let time_signature_digest = registry_blob_digest("trusted-time-signature", time_signature);
    let closure_spec = ReleaseGateSpecV1 {
        schema_version: "v1alpha1".to_owned(),
        gate_spec_id: release_id("gsp", '2'),
        gate_id: "release.profile.provider-codex".to_owned(),
        gate_version: 1,
        receipt_kind: ReleaseReceiptKindV1::ProfileClosure,
        profile_ids: vec!["provider-codex".to_owned()],
        required_evidence_kinds: vec![
            ReleaseEvidenceKindV1::Artifact,
            ReleaseEvidenceKindV1::Environment,
            ReleaseEvidenceKindV1::Policy,
            ReleaseEvidenceKindV1::Schema,
            ReleaseEvidenceKindV1::Toolchain,
        ],
        gate_policy_digest: tagged_digest('e'),
    };
    let closure_spec_digest = canonical_digest(RELEASE_GATE_SPEC_DIGEST_DOMAIN, &closure_spec);
    let closure_request = ReleaseGateVerificationRequestV1 {
        schema_version: "v1alpha1".to_owned(),
        verification_request_id: release_id("rvr", '2'),
        gate_id: closure_spec.gate_id.clone(),
        gate_version: closure_spec.gate_version,
        receipt_kind: closure_spec.receipt_kind,
        profile_ids: closure_spec.profile_ids.clone(),
        evidence_nonce: release_hex('e', 64),
        gate_spec_digest: closure_spec_digest.clone(),
        profile_graph_digest: graph_digest.clone(),
        gate_policy_digest: closure_spec.gate_policy_digest.clone(),
        family_subject: receipt.family_subject.clone(),
        evidence_subjects: receipt.evidence_subjects.clone(),
        requested_at_unix_ms: 100,
        expires_at_unix_ms: 850,
    };
    let closure_request_digest =
        canonical_digest(RELEASE_VERIFICATION_REQUEST_DIGEST_DOMAIN, &closure_request);
    let closure_receipt = GateReceiptV1 {
        schema_version: "v1alpha1".to_owned(),
        gate_receipt_id: release_id("grc", '2'),
        gate_id: closure_request.gate_id.clone(),
        gate_version: closure_request.gate_version,
        receipt_kind: closure_request.receipt_kind,
        profile_ids: closure_request.profile_ids.clone(),
        evidence_nonce: closure_request.evidence_nonce.clone(),
        request_digest: closure_request_digest.clone(),
        gate_spec_digest: closure_spec_digest.clone(),
        profile_graph_digest: graph_digest.clone(),
        gate_policy_digest: closure_request.gate_policy_digest.clone(),
        family_subject: closure_request.family_subject.clone(),
        evidence_subjects: closure_request.evidence_subjects.clone(),
        attestor_key_id: "attestor".to_owned(),
        started_at_unix_ms: 210,
        completed_at_unix_ms: 310,
        expires_at_unix_ms: 800,
    };
    let closure_receipt_digest =
        canonical_digest(RELEASE_GATE_RECEIPT_DIGEST_DOMAIN, &closure_receipt);
    let closure_time = TrustedTimeObservationV1 {
        schema_version: "v1alpha1".to_owned(),
        family: "bullet-farm".to_owned(),
        gate_receipt_id: closure_receipt.gate_receipt_id.clone(),
        receipt_digest: closure_receipt_digest.clone(),
        evidence_nonce: closure_receipt.evidence_nonce.clone(),
        signer_policy_digest: policy_digest.clone(),
        observed_at_unix_ms: 360,
        valid_until_unix_ms: 750,
        restore_epoch: 1,
        trusted_time_key_id: "time".to_owned(),
    };
    let closure_time_digest = canonical_digest(RELEASE_TRUSTED_TIME_DIGEST_DOMAIN, &closure_time);
    let closure_receipt_signature = b"untrusted profile closure signature";
    let closure_time_signature = b"untrusted profile closure time signature";
    let closure_receipt_signature_digest =
        registry_blob_digest("gate-receipt-signature", closure_receipt_signature);
    let closure_time_signature_digest =
        registry_blob_digest("trusted-time-signature", closure_time_signature);
    let entry = ReleaseRegistryEntryV1 {
        schema_version: "v1alpha1".to_owned(),
        gate_id: receipt.gate_id.clone(),
        profile_ids: receipt.profile_ids.clone(),
        gate_receipt_id: receipt.gate_receipt_id.clone(),
        receipt_digest: receipt_digest.clone(),
        receipt_path: "receipts/provider-codex.json".to_owned(),
        receipt_signature_digest: receipt_signature_digest.clone(),
        receipt_signature_path: "signatures/provider-codex.sig".to_owned(),
        trusted_time_digest: time_digest.clone(),
        trusted_time_path: "time/provider-codex.json".to_owned(),
        trusted_time_signature_digest: time_signature_digest.clone(),
        trusted_time_signature_path: "time/provider-codex.sig".to_owned(),
    };
    let closure_entry = ReleaseRegistryEntryV1 {
        schema_version: "v1alpha1".to_owned(),
        gate_id: closure_receipt.gate_id.clone(),
        profile_ids: closure_receipt.profile_ids.clone(),
        gate_receipt_id: closure_receipt.gate_receipt_id.clone(),
        receipt_digest: closure_receipt_digest.clone(),
        receipt_path: "receipts/provider-codex-closure.json".to_owned(),
        receipt_signature_digest: closure_receipt_signature_digest.clone(),
        receipt_signature_path: "signatures/provider-codex-closure.sig".to_owned(),
        trusted_time_digest: closure_time_digest.clone(),
        trusted_time_path: "time/provider-codex-closure.json".to_owned(),
        trusted_time_signature_digest: closure_time_signature_digest.clone(),
        trusted_time_signature_path: "time/provider-codex-closure.sig".to_owned(),
    };
    let manifest = ReleaseRegistryManifestV1 {
        schema_version: "v1alpha1".to_owned(),
        registry_id: release_id("rrg", '1'),
        generation: 1,
        previous_registry_digest: tagged_digest('d'),
        signer_policy_digest: policy_digest.clone(),
        profile_graph_digest: graph_digest.clone(),
        family_lock_digest: tagged_digest('b'),
        created_at_unix_ms: 400,
        expires_at_unix_ms: 700,
        registry_signer_key_id: "registry".to_owned(),
        objects: vec![
            registry_object(
                ReleaseRegistryObjectKindV1::GateReceipt,
                '1',
                receipt_digest,
                &entry.receipt_path,
            ),
            registry_object(
                ReleaseRegistryObjectKindV1::GateReceipt,
                'a',
                closure_receipt_digest,
                &closure_entry.receipt_path,
            ),
            registry_object(
                ReleaseRegistryObjectKindV1::GateReceiptSignature,
                '2',
                receipt_signature_digest,
                &entry.receipt_signature_path,
            ),
            registry_object(
                ReleaseRegistryObjectKindV1::GateReceiptSignature,
                'b',
                closure_receipt_signature_digest,
                &closure_entry.receipt_signature_path,
            ),
            registry_object(
                ReleaseRegistryObjectKindV1::GateSpec,
                '3',
                spec_digest,
                "specs/provider-codex.json",
            ),
            registry_object(
                ReleaseRegistryObjectKindV1::GateSpec,
                'c',
                closure_spec_digest,
                "specs/provider-codex-closure.json",
            ),
            registry_object(
                ReleaseRegistryObjectKindV1::ProfileGraph,
                '4',
                graph_digest,
                "profiles/graph.json",
            ),
            registry_object(
                ReleaseRegistryObjectKindV1::ReleaseBundleManifestV2,
                '9',
                release_bundle_digest,
                "artifacts/release-manifest-v2.toml",
            ),
            registry_object(
                ReleaseRegistryObjectKindV1::SignerPolicy,
                '5',
                policy_digest,
                "policy/signers.json",
            ),
            registry_object(
                ReleaseRegistryObjectKindV1::TrustedTimeObservation,
                '6',
                time_digest,
                &entry.trusted_time_path,
            ),
            registry_object(
                ReleaseRegistryObjectKindV1::TrustedTimeObservation,
                'd',
                closure_time_digest,
                &closure_entry.trusted_time_path,
            ),
            registry_object(
                ReleaseRegistryObjectKindV1::TrustedTimeSignature,
                '7',
                time_signature_digest,
                &entry.trusted_time_signature_path,
            ),
            registry_object(
                ReleaseRegistryObjectKindV1::TrustedTimeSignature,
                'e',
                closure_time_signature_digest,
                &closure_entry.trusted_time_signature_path,
            ),
            registry_object(
                ReleaseRegistryObjectKindV1::VerificationRequest,
                '8',
                request_digest,
                "requests/provider-codex.json",
            ),
            registry_object(
                ReleaseRegistryObjectKindV1::VerificationRequest,
                'f',
                closure_request_digest,
                "requests/provider-codex-closure.json",
            ),
        ],
        entries: vec![closure_entry, entry],
    };
    write_canonical(&root.join("receipts/provider-codex.json"), &receipt);
    write_canonical(
        &root.join("receipts/provider-codex-closure.json"),
        &closure_receipt,
    );
    write_canonical(&root.join("specs/provider-codex.json"), &spec);
    write_canonical(
        &root.join("specs/provider-codex-closure.json"),
        &closure_spec,
    );
    write_canonical(&root.join("profiles/graph.json"), &graph);
    write_canonical(&root.join("policy/signers.json"), &policy);
    write_canonical(&root.join("time/provider-codex.json"), &time);
    write_canonical(
        &root.join("time/provider-codex-closure.json"),
        &closure_time,
    );
    write_canonical(&root.join("requests/provider-codex.json"), &request);
    write_canonical(
        &root.join("requests/provider-codex-closure.json"),
        &closure_request,
    );
    fs::create_dir_all(root.join("artifacts")).unwrap();
    fs::write(
        root.join("artifacts/release-manifest-v2.toml"),
        release_bundle_manifest,
    )
    .unwrap();
    fs::create_dir_all(root.join("signatures")).unwrap();
    fs::write(
        root.join("signatures/provider-codex.sig"),
        receipt_signature,
    )
    .unwrap();
    fs::write(
        root.join("signatures/provider-codex-closure.sig"),
        closure_receipt_signature,
    )
    .unwrap();
    fs::write(root.join("time/provider-codex.sig"), time_signature).unwrap();
    fs::write(
        root.join("time/provider-codex-closure.sig"),
        closure_time_signature,
    )
    .unwrap();
    write_canonical(&root.join("registry-manifest.json"), &manifest);
}
