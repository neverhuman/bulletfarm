use bullet_wire::{
    RELEASE_GATE_SPEC_DIGEST_DOMAIN, RELEASE_PROFILE_GRAPH_DIGEST_DOMAIN,
    RELEASE_VERIFICATION_REQUEST_DIGEST_DOMAIN, hash_canonical,
    v1alpha1::{
        GateReceiptV1, ReleaseEvidenceKindV1, ReleaseEvidenceSubjectV1, ReleaseFamilySubjectV1,
        ReleaseGateSpecV1, ReleaseGateVerificationRequestV1, ReleaseProfileGraphV1,
        ReleaseProfileNodeV1, ReleaseReceiptKindV1, ReleaseRegistryEntryV1,
        ReleaseRegistryManifestV1, ReleaseRegistryObjectKindV1, ReleaseRegistryObjectV1,
        ReleaseReplayBindingV1, ReleaseReplayStateV1, ReleaseRepositoryNameV1,
        ReleaseRepositorySubjectV1, ReleaseSignerKeyV1, ReleaseSignerPolicyV1, ReleaseSignerRoleV1,
        TrustedTimeObservationV1,
    },
};

pub(super) fn hex(character: char, length: usize) -> String {
    std::iter::repeat_n(character, length).collect()
}

pub(super) fn digest(character: char) -> String {
    format!("blake3:{}", hex(character, 64))
}

pub(super) fn typed_id(prefix: &str, character: char) -> String {
    format!("{prefix}_{}", hex(character, 64))
}

fn repository(repository: ReleaseRepositoryNameV1, character: char) -> ReleaseRepositorySubjectV1 {
    ReleaseRepositorySubjectV1 {
        schema_version: "v1alpha1".to_owned(),
        repository,
        tag: "v1.0.0".to_owned(),
        commit_oid: format!("sha1:{}", hex(character, 40)),
        tree_oid: format!("sha1:{}", hex(character, 40)),
        release_signing_identity: format!(
            "release-{character}@bullet.farm|ed25519|SHA256:{}",
            hex(character.to_ascii_uppercase(), 24)
        ),
        source_subject_digest: digest(character),
    }
}

pub(super) fn evidence(
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
        subject_id: typed_id("cnt", character),
        native_subject_id: format!("{namespace}:cnt_{}", hex(character, 64)),
        subject_digest: digest(character),
    }
}

pub(super) fn receipt() -> GateReceiptV1 {
    GateReceiptV1 {
        schema_version: "v1alpha1".to_owned(),
        gate_receipt_id: typed_id("grc", 'a'),
        gate_id: "release.rust-toolchain".to_owned(),
        gate_version: 1,
        receipt_kind: ReleaseReceiptKindV1::RustToolchain,
        profile_ids: vec!["self-hosted-v1".to_owned()],
        evidence_nonce: hex('b', 64),
        request_digest: digest('c'),
        gate_spec_digest: digest('d'),
        profile_graph_digest: digest('d'),
        gate_policy_digest: digest('e'),
        family_subject: ReleaseFamilySubjectV1 {
            schema_version: "v1alpha1".to_owned(),
            family: "bullet-farm".to_owned(),
            family_lock_digest: digest('f'),
            schema_bundle_digest: digest('a'),
            repositories: vec![
                repository(ReleaseRepositoryNameV1::BulletFarm, '1'),
                repository(ReleaseRepositoryNameV1::BulletGit, '2'),
                repository(ReleaseRepositoryNameV1::BulletKernel, '3'),
                repository(ReleaseRepositoryNameV1::BulletPortal, '4'),
            ],
        },
        evidence_subjects: vec![
            evidence(ReleaseEvidenceKindV1::Environment, '1'),
            evidence(ReleaseEvidenceKindV1::Policy, '2'),
            evidence(ReleaseEvidenceKindV1::Schema, '3'),
            evidence(ReleaseEvidenceKindV1::Toolchain, '4'),
        ],
        attestor_key_id: "gate-attestor".to_owned(),
        started_at_unix_ms: 1_000,
        completed_at_unix_ms: 2_000,
        expires_at_unix_ms: 3_000,
    }
}

fn signer_key(key_id: &str, role: ReleaseSignerRoleV1, character: char) -> ReleaseSignerKeyV1 {
    ReleaseSignerKeyV1 {
        schema_version: "v1alpha1".to_owned(),
        key_id: key_id.to_owned(),
        role,
        signing_identity: format!(
            "{key_id}@bullet.farm|ed25519|SHA256:{}",
            hex(character.to_ascii_uppercase(), 24)
        ),
        public_key: format!("ssh-ed25519 {}", hex(character.to_ascii_uppercase(), 44)),
        activates_at_unix_ms: 1_000,
        expires_at_unix_ms: 3_000,
        revoked_at_unix_ms: None,
        retain_until_unix_ms: 4_000,
    }
}

pub(super) fn signer_policy() -> ReleaseSignerPolicyV1 {
    ReleaseSignerPolicyV1 {
        schema_version: "v1alpha1".to_owned(),
        family: "bullet-farm".to_owned(),
        policy_generation: 1,
        activates_at_unix_ms: 1_000,
        expires_at_unix_ms: 3_000,
        registry_signer_key_id: "registry".to_owned(),
        trusted_time_key_id: "time".to_owned(),
        signer_keys: vec![
            signer_key("artifact", ReleaseSignerRoleV1::ArtifactRelease, 'a'),
            signer_key("attestor", ReleaseSignerRoleV1::GateAttestor, 'b'),
            signer_key("registry", ReleaseSignerRoleV1::RegistryCurator, 'c'),
            signer_key("source", ReleaseSignerRoleV1::SourceTag, 'd'),
            signer_key("time", ReleaseSignerRoleV1::TrustedTime, 'e'),
        ],
    }
}

fn registry_entry() -> ReleaseRegistryEntryV1 {
    ReleaseRegistryEntryV1 {
        schema_version: "v1alpha1".to_owned(),
        gate_id: "release.rust-toolchain".to_owned(),
        profile_ids: vec!["self-hosted-v1".to_owned()],
        gate_receipt_id: typed_id("grc", 'a'),
        receipt_digest: digest('a'),
        receipt_path: "receipts/rust-toolchain.json".to_owned(),
        receipt_signature_digest: digest('b'),
        receipt_signature_path: "signatures/rust-toolchain.sig".to_owned(),
        trusted_time_digest: digest('c'),
        trusted_time_path: "time/rust-toolchain.json".to_owned(),
        trusted_time_signature_digest: digest('d'),
        trusted_time_signature_path: "time/rust-toolchain.sig".to_owned(),
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
        object_id: typed_id("rob", character),
        object_kind,
        object_digest,
        object_path: object_path.to_owned(),
    }
}

pub(super) fn registry_manifest() -> ReleaseRegistryManifestV1 {
    ReleaseRegistryManifestV1 {
        schema_version: "v1alpha1".to_owned(),
        registry_id: typed_id("rrg", 'a'),
        generation: 1,
        previous_registry_digest: digest('a'),
        signer_policy_digest: digest('b'),
        profile_graph_digest: digest('c'),
        family_lock_digest: digest('d'),
        created_at_unix_ms: 1_000,
        expires_at_unix_ms: 3_000,
        registry_signer_key_id: "registry".to_owned(),
        objects: vec![
            registry_object(
                ReleaseRegistryObjectKindV1::GateReceipt,
                '1',
                digest('a'),
                "receipts/rust-toolchain.json",
            ),
            registry_object(
                ReleaseRegistryObjectKindV1::GateReceiptSignature,
                '2',
                digest('b'),
                "signatures/rust-toolchain.sig",
            ),
            registry_object(
                ReleaseRegistryObjectKindV1::GateSpec,
                '3',
                digest('e'),
                "specs/rust-toolchain.json",
            ),
            registry_object(
                ReleaseRegistryObjectKindV1::ProfileGraph,
                '4',
                digest('c'),
                "profiles/graph.json",
            ),
            registry_object(
                ReleaseRegistryObjectKindV1::SignerPolicy,
                '5',
                digest('b'),
                "policy/signers.json",
            ),
            registry_object(
                ReleaseRegistryObjectKindV1::TrustedTimeObservation,
                '6',
                digest('c'),
                "time/rust-toolchain.json",
            ),
            registry_object(
                ReleaseRegistryObjectKindV1::TrustedTimeSignature,
                '7',
                digest('d'),
                "time/rust-toolchain.sig",
            ),
            registry_object(
                ReleaseRegistryObjectKindV1::VerificationRequest,
                '8',
                digest('f'),
                "requests/rust-toolchain.json",
            ),
        ],
        entries: vec![registry_entry()],
    }
}

fn profile_graph() -> ReleaseProfileGraphV1 {
    ReleaseProfileGraphV1 {
        schema_version: "v1alpha1".to_owned(),
        profile_graph_id: typed_id("rpg", 'a'),
        family: "bullet-farm".to_owned(),
        generation: 1,
        profiles: vec![
            ReleaseProfileNodeV1 {
                schema_version: "v1alpha1".to_owned(),
                profile_id: "evolution-v1".to_owned(),
                dependency_profile_ids: vec!["self-hosted-v1".to_owned()],
                gate_ids: vec!["release.evolution".to_owned()],
            },
            ReleaseProfileNodeV1 {
                schema_version: "v1alpha1".to_owned(),
                profile_id: "self-hosted-v1".to_owned(),
                dependency_profile_ids: Vec::new(),
                gate_ids: vec!["release.rust-toolchain".to_owned()],
            },
        ],
    }
}

fn gate_spec() -> ReleaseGateSpecV1 {
    ReleaseGateSpecV1 {
        schema_version: "v1alpha1".to_owned(),
        gate_spec_id: typed_id("gsp", 'a'),
        gate_id: "release.rust-toolchain".to_owned(),
        gate_version: 1,
        receipt_kind: ReleaseReceiptKindV1::RustToolchain,
        profile_ids: vec!["self-hosted-v1".to_owned()],
        required_evidence_kinds: vec![
            ReleaseEvidenceKindV1::Environment,
            ReleaseEvidenceKindV1::Policy,
            ReleaseEvidenceKindV1::Schema,
            ReleaseEvidenceKindV1::Toolchain,
        ],
        gate_policy_digest: digest('e'),
    }
}

pub(super) fn binding_bundle() -> (
    ReleaseProfileGraphV1,
    ReleaseGateSpecV1,
    ReleaseGateVerificationRequestV1,
    GateReceiptV1,
) {
    let graph = profile_graph();
    let spec = gate_spec();
    let mut receipt = receipt();
    let graph_digest = format!(
        "blake3:{}",
        hash_canonical(RELEASE_PROFILE_GRAPH_DIGEST_DOMAIN, &graph)
            .unwrap()
            .to_hex()
    );
    let spec_digest = format!(
        "blake3:{}",
        hash_canonical(RELEASE_GATE_SPEC_DIGEST_DOMAIN, &spec)
            .unwrap()
            .to_hex()
    );
    let request = ReleaseGateVerificationRequestV1 {
        schema_version: "v1alpha1".to_owned(),
        verification_request_id: typed_id("rvr", 'a'),
        gate_id: spec.gate_id.clone(),
        gate_version: spec.gate_version,
        receipt_kind: spec.receipt_kind,
        profile_ids: spec.profile_ids.clone(),
        evidence_nonce: receipt.evidence_nonce.clone(),
        gate_spec_digest: spec_digest.clone(),
        profile_graph_digest: graph_digest.clone(),
        gate_policy_digest: spec.gate_policy_digest.clone(),
        family_subject: receipt.family_subject.clone(),
        evidence_subjects: receipt.evidence_subjects.clone(),
        requested_at_unix_ms: 500,
        expires_at_unix_ms: 2_500,
    };
    receipt.request_digest = format!(
        "blake3:{}",
        hash_canonical(RELEASE_VERIFICATION_REQUEST_DIGEST_DOMAIN, &request)
            .unwrap()
            .to_hex()
    );
    receipt.gate_spec_digest = spec_digest;
    receipt.profile_graph_digest = graph_digest;
    (graph, spec, request, receipt)
}

pub(super) fn replay_state() -> ReleaseReplayStateV1 {
    ReleaseReplayStateV1 {
        schema_version: "v1alpha1".to_owned(),
        registry_id: typed_id("rrg", 'a'),
        generation: 1,
        registry_manifest_digest: digest('a'),
        previous_state_digest: digest('b'),
        restore_epoch: 7,
        trusted_time_floor_unix_ms: 1_000,
        bindings: vec![ReleaseReplayBindingV1 {
            schema_version: "v1alpha1".to_owned(),
            evidence_nonce: hex('a', 64),
            gate_receipt_id: typed_id("grc", 'a'),
            gate_id: "release.rust-toolchain".to_owned(),
            request_digest: digest('c'),
            receipt_digest: digest('d'),
        }],
        registry_signer_key_id: "registry".to_owned(),
    }
}

pub(super) fn trusted_time() -> TrustedTimeObservationV1 {
    TrustedTimeObservationV1 {
        schema_version: "v1alpha1".to_owned(),
        family: "bullet-farm".to_owned(),
        gate_receipt_id: typed_id("grc", 'a'),
        receipt_digest: digest('a'),
        evidence_nonce: hex('b', 64),
        signer_policy_digest: digest('c'),
        observed_at_unix_ms: 1_000,
        valid_until_unix_ms: 3_000,
        restore_epoch: 7,
        trusted_time_key_id: "time".to_owned(),
    }
}
