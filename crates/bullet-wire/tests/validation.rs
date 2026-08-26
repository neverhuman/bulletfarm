use std::str::FromStr;

use bullet_wire::{
    Blake3Digest, CandidateId, ContentId, GateId, GitOid, PatchMutation, PatchOperation,
    PatchProposal, Preimage, RepoPath, SCHEMA_VERSION,
};

fn digest(byte: u8) -> Blake3Digest {
    Blake3Digest::from_bytes([byte; 32])
}

fn id<T: FromStr>(prefix: &str, byte: char) -> T
where
    T::Err: std::fmt::Debug,
{
    format!("{prefix}{}", byte.to_string().repeat(64))
        .parse()
        .unwrap()
}

fn proposal(operations: Vec<PatchOperation>) -> PatchProposal {
    PatchProposal {
        schema_version: SCHEMA_VERSION,
        proposal_id: id::<ContentId>("cnt_", '1'),
        producing_attempt_id: id("atm_", '2'),
        base_checkpoint_id: id("ckp_", '3'),
        base_checkpoint_digest: digest(4),
        operations,
        gate_ids: vec![id::<GateId>("gat_", '5')],
    }
}

fn write(path: &str) -> PatchOperation {
    PatchOperation {
        path: path.parse().unwrap(),
        preimage: Preimage::Digest { digest: digest(6) },
        mutation: PatchMutation::Write {
            content_utf8: "content".to_owned(),
        },
    }
}

#[test]
fn ids_are_full_lowercase_256_bit_encodings() {
    let candidate = id::<CandidateId>("can_", 'a');
    assert_eq!(candidate.as_str().len(), 68);
    for denied in ["can_abc".to_owned(), format!("can_{}", "A".repeat(64))] {
        assert!(CandidateId::from_str(&denied).is_err());
    }
}

#[test]
fn git_oids_require_algorithm_tags() {
    assert!(GitOid::from_str(&format!("sha1:{}", "a".repeat(40))).is_ok());
    assert!(GitOid::from_str(&format!("sha256:{}", "b".repeat(64))).is_ok());
    assert!(GitOid::from_str(&"a".repeat(40)).is_err());
    assert!(GitOid::from_str(&format!("sha1:{}", "A".repeat(40))).is_err());
}

#[test]
fn repository_paths_reject_escape_and_platform_collisions() {
    for denied in [
        "/etc/passwd",
        "../src",
        "src/../main.rs",
        ".git/config",
        "src\\main.rs",
        "src/file:stream",
        "src/trailing.",
    ] {
        assert!(RepoPath::from_str(denied).is_err(), "accepted {denied}");
    }
}

#[test]
fn proposal_rejects_duplicate_case_paths_and_parent_conflicts() {
    let duplicate = proposal(vec![write("src/Main.rs"), write("src/main.rs")]);
    assert_eq!(duplicate.validate().unwrap_err().code(), "PATH_COLLISION");
    for (parent, child) in [
        ("src", "src/main.rs"),
        ("Src", "src/main.rs"),
        ("Étage", "étage/file.rs"),
    ] {
        let proposal = proposal(vec![write(parent), write(child)]);
        assert_eq!(
            proposal.validate().unwrap_err().code(),
            "PATH_CONFLICT",
            "accepted portable ancestor conflict {parent:?} and {child:?}"
        );
    }
}

#[test]
fn patch_proposal_matches_exact_schema_one_golden_shape() {
    let expected = serde_json::json!({
        "schema_version": 1,
        "proposal_id": format!("cnt_{}", "1".repeat(64)),
        "producing_attempt_id": format!("atm_{}", "2".repeat(64)),
        "base_checkpoint_id": format!("ckp_{}", "3".repeat(64)),
        "base_checkpoint_digest": "04".repeat(32),
        "operations": [{
            "path": "PONG.txt",
            "preimage": { "kind": "absent" },
            "mutation": { "kind": "write", "content_utf8": "PONG\n" }
        }],
        "gate_ids": [format!("gat_{}", "5".repeat(64))]
    });
    assert_eq!(expected.as_object().unwrap().len(), 7);

    let proposal: PatchProposal = serde_json::from_value(expected.clone()).unwrap();
    proposal.validate().unwrap();
    assert_eq!(serde_json::to_value(proposal).unwrap(), expected);
}

#[test]
fn delete_requires_a_bound_preimage() {
    let invalid = proposal(vec![PatchOperation {
        path: "src/main.rs".parse().unwrap(),
        preimage: Preimage::Absent,
        mutation: PatchMutation::Delete,
    }]);
    assert_eq!(invalid.validate().unwrap_err().code(), "MISSING_PREIMAGE");
}

#[test]
fn actual_scope_cannot_exceed_grant() {
    let mut manifest = bullet_wire_test_candidate();
    manifest.actual_scope = vec!["docs/readme.md".parse().unwrap()];
    assert_eq!(
        manifest.validate().unwrap_err().code(),
        "ACTUAL_SCOPE_EXCEEDS_GRANT"
    );
}

fn bullet_wire_test_candidate() -> bullet_wire::ComponentCandidateManifest {
    bullet_wire::ComponentCandidateManifest {
        schema_version: SCHEMA_VERSION,
        repository_id: id("rep_", '1'),
        change_id: id("chg_", '2'),
        producing_attempt_id: id("atm_", '3'),
        attempt_fence: 1,
        work_package_id: id("wpk_", '4'),
        variant_id: id("var_", '5'),
        plan_revision_id: id("pln_", '6'),
        graph_revision_id: id("grf_", '7'),
        base_checkpoint_id: id("ckp_", '8'),
        base_commit: format!("sha1:{}", "9".repeat(40)).parse().unwrap(),
        head_commit: format!("sha1:{}", "a".repeat(40)).parse().unwrap(),
        tree_oid: format!("sha1:{}", "b".repeat(40)).parse().unwrap(),
        patch_digest: digest(1),
        parent_candidate_ids: vec![],
        granted_scope: vec!["src".parse().unwrap()],
        actual_scope: vec!["src/lib.rs".parse().unwrap()],
        context_capsule_id: id("cnt_", 'c'),
        configuration_snapshot_id: id("cnt_", 'd'),
        policy_snapshot_id: id("cnt_", 'e'),
        routing_snapshot_id: id("cnt_", 'f'),
        environment_digest: digest(2),
        toolchain_digest: digest(3),
    }
}
