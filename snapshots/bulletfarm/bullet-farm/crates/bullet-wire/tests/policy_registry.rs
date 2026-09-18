use std::{fs, path::PathBuf};

use bullet_wire::{
    AuthorityAudience, ContractCatalogV1, ContractMode, InvariantRegistryV1, PolicySnapshotV1,
    canonical_json, decode_canonical, execute_contract_tool,
};
use sha2::{Digest, Sha256};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap()
        .to_path_buf()
}

#[test]
fn registry_is_complete_tiered_and_phase_honest() {
    let bytes = fs::read(root().join("policy/v1alpha1/invariant-registry.json")).unwrap();
    let registry = decode_canonical::<InvariantRegistryV1>(&bytes).unwrap();
    registry.validate().unwrap();
    assert!(registry.entries.iter().any(|entry| {
        entry.lifecycle == bullet_wire::InvariantLifecycle::Planned
            && entry.first_applicable_wave == 11
    }));
    assert!(registry.entries.iter().all(|entry| {
        entry.first_applicable_wave > 1
            || entry.lifecycle == bullet_wire::InvariantLifecycle::Enforced
    }));
}

#[test]
fn duplicate_alias_and_unknown_registry_field_fail_closed() {
    let bytes = fs::read(root().join("policy/v1alpha1/invariant-registry.json")).unwrap();
    let mut registry = decode_canonical::<InvariantRegistryV1>(&bytes).unwrap();
    let alias = registry.entries[0].legacy_aliases[0].clone();
    registry.entries[1].legacy_aliases.push(alias);
    assert_eq!(
        registry.validate().unwrap_err().code(),
        "DUPLICATE_INVARIANT_ID"
    );

    let mut value = serde_json::from_slice::<serde_json::Value>(&bytes).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("authority".to_owned(), serde_json::json!(true));
    let changed = canonical_json(&value).unwrap();
    assert_eq!(
        decode_canonical::<InvariantRegistryV1>(&changed)
            .unwrap_err()
            .code(),
        "DOCUMENT_SCHEMA_INVALID"
    );
}

#[test]
fn catalog_names_cannot_inject_generated_languages() {
    let bytes = fs::read(root().join("contracts/v1alpha1/contract-catalog.json")).unwrap();
    let mut catalog = decode_canonical::<ContractCatalogV1>(&bytes).unwrap();
    catalog.records[0].name = "Injected{Code".to_owned();
    assert_eq!(
        catalog.validate().unwrap_err().code(),
        "INVALID_CONTRACT_RECORD"
    );

    let mut catalog = decode_canonical::<ContractCatalogV1>(&bytes).unwrap();
    catalog.records[0].fields[0].name = "type".to_owned();
    assert_eq!(
        catalog.validate().unwrap_err().code(),
        "INVALID_CONTRACT_FIELD"
    );
}

#[test]
fn policy_and_catalog_are_strict_complete_and_offline() {
    let policy_bytes = fs::read(root().join("policy/v1alpha1/policy.json")).unwrap();
    let policy = decode_canonical::<PolicySnapshotV1>(&policy_bytes).unwrap();
    policy.validate().unwrap();
    assert!(!policy.sandbox_policy.live_admission_enabled);
    let generated =
        serde_json::from_slice::<bullet_wire::v1alpha1::PolicySnapshotV1>(&policy_bytes).unwrap();
    assert_eq!(generated.route_policy.universal_incumbent, "T0");
    assert_eq!(generated.issuer_keys.len(), 1);
    assert_eq!(generated.issuer_keys[0].key_id, "release-signing-alpha");

    let mut unknown_nested = serde_json::from_slice::<serde_json::Value>(&policy_bytes).unwrap();
    unknown_nested["route_policy"]["surprise"] = serde_json::json!(true);
    assert!(
        serde_json::from_value::<bullet_wire::v1alpha1::PolicySnapshotV1>(unknown_nested).is_err()
    );

    let catalog = decode_canonical::<ContractCatalogV1>(
        &fs::read(root().join("contracts/v1alpha1/contract-catalog.json")).unwrap(),
    )
    .unwrap();
    let bundle = catalog.json_schema_bundle().unwrap();
    assert_eq!(
        bundle["schemas"]["SignedAuthorityEnvelopeV1"]["additionalProperties"],
        false
    );
    assert_eq!(
        bundle["schemas"]["RouteDecision"]["additionalProperties"],
        false
    );
    assert_eq!(
        bundle["schemas"]["ApplyPatchRequestV1"]["properties"]["proposal"]["$ref"],
        "#/schemas/PatchProposalV1"
    );
    assert_eq!(
        bundle["schemas"]["CloneWorkspaceRequestV1"]["properties"]["scope_grant"]["$ref"],
        "#/schemas/ScopeGrantV1"
    );
    assert_eq!(
        bundle["schemas"]["FinalAuthorityDecisionV1"]["allOf"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        bundle["schemas"]["MutationSettlementResultV1"]["allOf"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(
        bundle["schemas"]["PatchOperationV1"]["allOf"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
}

#[test]
fn policy_key_purpose_lifecycle_and_identity_fail_closed() {
    let bytes = fs::read(root().join("policy/v1alpha1/policy.json")).unwrap();
    let policy = decode_canonical::<PolicySnapshotV1>(&bytes).unwrap();

    let mut wrong_use = policy.clone();
    wrong_use.issuer_keys[0].key_purpose = bullet_wire::KeyPurposeV1::AuthoritySigning;
    assert_eq!(wrong_use.validate().unwrap_err().code(), "INVALID_KEY_USE");

    let mut duplicate = policy.clone();
    duplicate.issuer_keys.push(duplicate.issuer_keys[0].clone());
    assert_eq!(
        duplicate.validate().unwrap_err().code(),
        "INVALID_ISSUER_KEY_LIFECYCLE"
    );

    let mut short_retention = policy;
    short_retention.issuer_keys[0].retain_until_unix_ms =
        short_retention.issuer_keys[0].expires_at_unix_ms;
    assert_eq!(
        short_retention.validate().unwrap_err().code(),
        "INVALID_ISSUER_KEY_LIFECYCLE"
    );
}

#[test]
fn authority_key_lookup_enforces_lifecycle_audience_and_material() {
    let bytes = fs::read(root().join("policy/v1alpha1/policy.json")).unwrap();
    let mut policy = decode_canonical::<PolicySnapshotV1>(&bytes).unwrap();
    let mut authority = policy.issuer_keys[0].clone();
    authority.issuer = "fixture-only-kernel".to_owned();
    authority.key_id = "fixture-only-authority".to_owned();
    authority.key_purpose = bullet_wire::KeyPurposeV1::AuthoritySigning;
    authority.algorithm = bullet_wire::KeyAlgorithmV1::PasetoV4Public;
    authority.public_key =
        "1eb9dbbbbc047c03fd70604e0071f0987e16b28b757225c11f00415d0e20b1a2".to_owned();
    authority.audiences = vec![AuthorityAudience::BulletGitd];
    policy.issuer_keys.push(authority);
    policy.validate().unwrap();

    let key = policy
        .authority_key_at(
            "fixture-only-kernel",
            "fixture-only-authority",
            AuthorityAudience::BulletGitd,
            policy.activation_at_unix_ms,
        )
        .unwrap();
    assert_eq!(key.key_id, "fixture-only-authority");
    assert_eq!(
        policy
            .authority_key_at(
                "fixture-only-kernel",
                "fixture-only-authority",
                AuthorityAudience::EffectBroker,
                policy.activation_at_unix_ms,
            )
            .unwrap_err()
            .code(),
        "AUTHORITY_KEY_AUDIENCE_MISMATCH"
    );
    assert_eq!(
        policy
            .authority_key_at(
                "fixture-only-kernel",
                "fixture-only-authority",
                AuthorityAudience::BulletGitd,
                policy.expires_at_unix_ms,
            )
            .unwrap_err()
            .code(),
        "POLICY_NOT_ACTIVE"
    );

    let authority_index = policy.issuer_keys.len() - 1;
    policy.issuer_keys[authority_index].revoked_at_unix_ms = Some(policy.activation_at_unix_ms + 1);
    assert_eq!(
        policy
            .authority_key_at(
                "fixture-only-kernel",
                "fixture-only-authority",
                AuthorityAudience::BulletGitd,
                policy.activation_at_unix_ms + 1,
            )
            .unwrap_err()
            .code(),
        "AUTHORITY_KEY_INACTIVE"
    );

    policy.issuer_keys[authority_index].revoked_at_unix_ms = None;
    policy.issuer_keys[authority_index]
        .audiences
        .push(AuthorityAudience::BulletGitd);
    assert_eq!(
        policy.validate().unwrap_err().code(),
        "INVALID_ISSUER_KEY_LIFECYCLE"
    );
    policy.issuer_keys[authority_index].audiences.pop();
    policy.issuer_keys[authority_index].public_key = "0".repeat(64);
    assert_eq!(
        policy.validate().unwrap_err().code(),
        "INVALID_AUTHORITY_KEY"
    );
}

#[test]
fn hostile_team_fixture_preserves_the_audited_bytes() {
    let bytes = fs::read(root().join("fixtures/hostile/team-original.bin")).unwrap();
    assert_eq!(bytes.len(), 31_835);
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytes)),
        "013f19032017ea5e27f69717f5e294e46aa56fbdabe31619ecfa0f77ac4007bf"
    );
}

#[test]
fn committed_generated_contracts_have_zero_byte_drift() {
    execute_contract_tool(&root(), ContractMode::Check).unwrap();
}

#[test]
fn generated_pin_accepts_only_exact_canonical_contract_bytes() {
    use bullet_wire::v1alpha1::{PinnedContract, verify_pinned_contract};

    let fixtures = [
        (
            PinnedContract::SchemaBundle,
            "contracts/v1alpha1/schema-bundle.json",
        ),
        (
            PinnedContract::InvariantRegistry,
            "policy/v1alpha1/invariant-registry.json",
        ),
        (
            PinnedContract::PolicySnapshot,
            "policy/v1alpha1/policy.json",
        ),
    ];
    for (contract, path) in fixtures {
        let bytes = fs::read(root().join(path)).unwrap();
        verify_pinned_contract(contract, &bytes).unwrap();
        let mut changed = bytes;
        changed.push(b' ');
        let error = verify_pinned_contract(contract, &changed).unwrap_err();
        assert_eq!(error.reason_code(), "UNPINNED_CONTRACT");
    }

    verify_pinned_contract(
        PinnedContract::CanonicalGolden,
        bullet_wire::v1alpha1::CANONICAL_GOLDEN_JSON.as_bytes(),
    )
    .unwrap();
    assert!(
        verify_pinned_contract(
            PinnedContract::SchemaBundle,
            bullet_wire::v1alpha1::CANONICAL_GOLDEN_JSON.as_bytes(),
        )
        .is_err()
    );
}

#[test]
fn enforced_floor_does_not_fall_below_four() {
    let bytes = fs::read(root().join("policy/v1alpha1/invariant-registry.json")).unwrap();
    let registry = decode_canonical::<InvariantRegistryV1>(&bytes).unwrap();
    let enforced: Vec<_> = registry
        .entries
        .iter()
        .filter(|entry| entry.lifecycle == bullet_wire::InvariantLifecycle::Enforced)
        .collect();
    assert!(
        enforced.len() >= 4,
        "enforced count {} fell below the committed floor of 4",
        enforced.len()
    );
    assert_eq!(registry.entries.len(), 51);
    for entry in enforced {
        assert!(
            !entry.proof_command.is_empty(),
            "{} missing proof_command",
            entry.id
        );
        assert!(
            !entry.enforcement_target.is_empty(),
            "{} missing enforcement_target",
            entry.id
        );
    }
}

#[test]
fn generated_records_are_strict_runtime_types() {
    let value = serde_json::json!({
        "schema_version": "v1alpha1",
        "failure_class_id": format!("failure_{}", "0".repeat(64)),
        "taxonomy_version": "v1",
        "class_name": "boundary",
        "definition": "defined",
        "surprise": true
    });
    assert!(serde_json::from_value::<bullet_wire::v1alpha1::FailureClass>(value).is_err());
}
