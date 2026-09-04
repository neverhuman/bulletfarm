use std::str::FromStr;

use bullet_wire::{
    Blake3Digest, CandidateId, ChangeId, CheckpointId, ComponentCandidateManifest, ContentId,
    GitOid, GraphRevisionId, PlanRevisionId, RepoPath, RepositoryId, SCHEMA_VERSION, VariantId,
    WorkPackageId, canonical_json, hash_canonical,
};

fn digest(byte: u8) -> Blake3Digest {
    Blake3Digest::from_bytes([byte; 32])
}

fn id<T: FromStr>(prefix: &str, hex: char) -> T
where
    T::Err: std::fmt::Debug,
{
    format!("{prefix}{}", hex.to_string().repeat(64))
        .parse()
        .unwrap()
}

fn candidate() -> ComponentCandidateManifest {
    ComponentCandidateManifest {
        schema_version: SCHEMA_VERSION,
        repository_id: id::<RepositoryId>("rep_", '1'),
        change_id: id::<ChangeId>("chg_", '2'),
        producing_attempt_id: id("atm_", '3'),
        attempt_fence: 17,
        work_package_id: id::<WorkPackageId>("wpk_", '4'),
        variant_id: id::<VariantId>("var_", '5'),
        plan_revision_id: id::<PlanRevisionId>("pln_", '6'),
        graph_revision_id: id::<GraphRevisionId>("grf_", '7'),
        base_checkpoint_id: id::<CheckpointId>("ckp_", '8'),
        base_commit: GitOid::from_str(&format!("sha1:{}", "9".repeat(40))).unwrap(),
        head_commit: GitOid::from_str(&format!("sha1:{}", "a".repeat(40))).unwrap(),
        tree_oid: GitOid::from_str(&format!("sha1:{}", "b".repeat(40))).unwrap(),
        patch_digest: digest(12),
        parent_candidate_ids: vec![id::<CandidateId>("can_", 'd')],
        granted_scope: vec![RepoPath::from_str("src").unwrap()],
        actual_scope: vec![RepoPath::from_str("src/lib.rs").unwrap()],
        context_capsule_id: id::<ContentId>("cnt_", 'e'),
        configuration_snapshot_id: id::<ContentId>("cnt_", '1'),
        policy_snapshot_id: id::<ContentId>("cnt_", '2'),
        routing_snapshot_id: id::<ContentId>("cnt_", '3'),
        environment_digest: digest(14),
        toolchain_digest: digest(15),
    }
}

#[test]
fn rfc8785_orders_object_keys() {
    let value = serde_json::json!({"z": 1, "a": {"y": true, "b": null}});
    assert_eq!(
        String::from_utf8(canonical_json(&value).unwrap()).unwrap(),
        r#"{"a":{"b":null,"y":true},"z":1}"#
    );
}

#[test]
fn candidate_golden_encoding_and_id_are_stable() {
    let manifest = candidate();
    let json = String::from_utf8(canonical_json(&manifest).unwrap()).unwrap();
    assert!(json.starts_with(r#"{"actual_scope":["src/lib.rs"],"attempt_fence":17,"#));
    assert_eq!(
        manifest.candidate_id().unwrap().to_string(),
        "can_66da272ac2783b2e7c67ff8c3e88dc941b4853838ea24a0d133de6c619e5cdf1"
    );
    assert_eq!(
        hash_canonical("candidate.content", &manifest)
            .unwrap()
            .to_string(),
        "fb001f98979f770d2dad0b3f47ec96662eec339d1816ee3957749faf0b196762"
    );
}

#[test]
fn every_manifest_field_changes_candidate_identity() {
    let manifest = candidate();
    let original = manifest.candidate_id().unwrap();
    let value = serde_json::to_value(&manifest).unwrap();
    let object = value.as_object().unwrap();
    for key in object.keys() {
        let mut changed = value.clone();
        changed.as_object_mut().unwrap().remove(key);
        let digest = hash_canonical("candidate.provenance", &changed).unwrap();
        assert_ne!(
            CandidateId::from_digest(digest),
            original,
            "field {key} was unbound"
        );
    }
}

#[test]
fn provenance_changes_do_not_change_content_identity() {
    let original = candidate();
    let mut successor = original.clone();
    successor.attempt_fence += 1;
    successor.producing_attempt_id = id("atm_", 'f');
    assert_eq!(
        original.content_id().unwrap(),
        successor.content_id().unwrap()
    );
    assert_ne!(
        original.candidate_id().unwrap(),
        successor.candidate_id().unwrap()
    );
}

#[test]
fn domains_separate_equal_content() {
    let value = serde_json::json!({"same": true});
    assert_ne!(
        hash_canonical("candidate.content", &value).unwrap(),
        hash_canonical("checkpoint.manifest", &value).unwrap()
    );
}
