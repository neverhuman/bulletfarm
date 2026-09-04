use super::*;

fn path_hex(value: &[u8]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn digest(prefix: &str, marker: char) -> String {
    format!("{prefix}{}", marker.to_string().repeat(64))
}

fn member(role: Wave0MemberRoleV1, identity: &str, marker: char) -> Wave0MemberV1 {
    Wave0MemberV1 {
        role,
        repository_identity: identity.to_owned(),
        commit_oid: format!("sha1:{}", marker.to_string().repeat(40)),
        tree_oid: format!(
            "sha1:{}",
            (((marker as u8) + 1) as char).to_string().repeat(40)
        ),
        index_state: Wave0CleanStateV1::Clean,
        worktree_state: Wave0CleanStateV1::Clean,
        untracked_state: Wave0CleanStateV1::Clean,
    }
}

fn facts() -> Wave0FactsV1 {
    Wave0FactsV1 {
        producer_principal: "baseline-producer".to_owned(),
        claim_high_water: Wave0ClaimHighWaterV1 {
            claim_ledger_path_hex: path_hex(b"/home/ubuntu/bullet/AGENT_CHAT.md"),
            claim_ledger_sha256: digest("sha256:", 'a'),
            claim_projection_blake3: digest("blake3:", 'b'),
            byte_length: 64,
            entry_count: 1,
            active_claim_count: 0,
        },
        members: vec![
            member(Wave0MemberRoleV1::Hub, "root/bullet-farm", '1'),
            member(Wave0MemberRoleV1::Kernel, "root/bullet-kernel", '3'),
            member(Wave0MemberRoleV1::BulletGit, "root/bullet-git", '5'),
            member(Wave0MemberRoleV1::Portal, "root/bullet-portal", '7'),
        ],
    }
}

fn wave0() -> Wave0SubjectV1 {
    Wave0SubjectV1::from_reviewed(
        facts(),
        "independent-reviewer".to_owned(),
        path_hex(b"/var/lib/bullet/reviews/w0.json"),
        digest("sha256:", 'c'),
        64,
    )
    .unwrap()
}

fn directory(path: &[u8]) -> IncidentInventoryNodeV1 {
    IncidentInventoryNodeV1 {
        relative_path_hex: path_hex(path),
        node_type: IncidentInventoryNodeTypeV1::Directory,
        owner_uid: 1000,
        owner_gid: 1000,
        mode: 0o700,
        link_count: 2,
        byte_length: 64,
        content_sha256: None,
    }
}

fn regular(path: &[u8], marker: char, byte_length: u64) -> IncidentInventoryNodeV1 {
    IncidentInventoryNodeV1 {
        relative_path_hex: path_hex(path),
        node_type: IncidentInventoryNodeTypeV1::RegularFile,
        owner_uid: 1000,
        owner_gid: 1000,
        mode: 0o400,
        link_count: 1,
        byte_length,
        content_sha256: Some(digest("sha256:", marker)),
    }
}

fn inventory_subject() -> IncidentInventorySubjectV1 {
    IncidentInventorySubjectV1 {
        source_directory: IncidentDirectoryIdentityV1 {
            absolute_path_hex: path_hex(b"/home/ubuntu/bullet/.bullet-family/coord"),
            device: 1,
            inode: 2,
            owner_uid: 1000,
            owner_gid: 1000,
            mode: 0o700,
            link_count: 3,
            byte_length: 96,
        },
        destination_name_hex: path_hex(b"coord-incident-v1"),
        node_count: 3,
        directory_count: 1,
        regular_file_count: 2,
        regular_file_byte_length: 15,
        nodes: vec![
            directory(b"claims"),
            regular(b"claims/active.jsonl", 'd', 10),
            regular(b"events.jsonl", 'e', 5),
        ],
    }
}

fn inventory() -> IncidentInventoryV1 {
    IncidentInventoryV1::from_subject(inventory_subject()).unwrap()
}

fn references() -> FreshGenesisAdmissionReferencesV1 {
    let inventory = inventory();
    let wave0 = wave0();
    FreshGenesisAdmissionReferencesV1 {
        incident_inventory: FreshGenesisSealedRecordRefV1 {
            record_kind: FreshGenesisRecordKindV1::IncidentInventoryV1,
            absolute_path_hex: path_hex(b"/var/lib/bullet/incidents/inventory.json"),
            record_id: inventory.inventory_id,
            sealed_sha256: digest("sha256:", 'f'),
            byte_length: 100,
        },
        wave0_subject: FreshGenesisSealedRecordRefV1 {
            record_kind: FreshGenesisRecordKindV1::Wave0SubjectV1,
            absolute_path_hex: path_hex(b"/var/lib/bullet/baselines/w0.json"),
            record_id: wave0.subject_id,
            sealed_sha256: digest("sha256:", '0'),
            byte_length: 200,
        },
    }
}

#[test]
fn valid_records_are_canonical_and_domain_bound() {
    let inventory = inventory();
    let inventory_bytes = bullet_wire::canonical_json(&inventory).unwrap();
    let decoded: IncidentInventoryV1 = bullet_wire::decode_canonical(&inventory_bytes).unwrap();
    decoded.validate().unwrap();
    let mut inventory_drift = inventory;
    inventory_drift.inventory_id = format!("fgi_{}", "9".repeat(64));
    assert!(inventory_drift.validate().is_err());

    let wave0 = wave0();
    let wave0_bytes = bullet_wire::canonical_json(&wave0).unwrap();
    let decoded: Wave0SubjectV1 = bullet_wire::decode_canonical(&wave0_bytes).unwrap();
    decoded.validate().unwrap();

    let refs = references();
    let identity = refs.subject_blake3().unwrap();
    let mut changed = refs.clone();
    changed.wave0_subject.byte_length += 1;
    assert_ne!(changed.subject_blake3().unwrap(), identity);
}

#[test]
fn canonical_decoders_refuse_unknown_duplicate_and_open_enums() {
    let inventory = inventory();
    let mut unknown = serde_json::to_value(&inventory).unwrap();
    unknown
        .as_object_mut()
        .unwrap()
        .insert("unknown".to_owned(), serde_json::json!(true));
    let unknown = bullet_wire::canonical_json(&unknown).unwrap();
    assert!(bullet_wire::decode_canonical::<IncidentInventoryV1>(&unknown).is_err());

    let canonical = String::from_utf8(bullet_wire::canonical_json(&inventory).unwrap()).unwrap();
    let duplicate = canonical.replacen(
        "\"schema_version\":1",
        "\"schema_version\":1,\"schema_version\":1",
        1,
    );
    assert!(bullet_wire::decode_canonical::<IncidentInventoryV1>(duplicate.as_bytes()).is_err());

    let open_type = canonical.replace("\"DIRECTORY\"", "\"SYMLINK\"");
    assert!(bullet_wire::decode_canonical::<IncidentInventoryV1>(open_type.as_bytes()).is_err());
    let wave0 = String::from_utf8(bullet_wire::canonical_json(&wave0()).unwrap()).unwrap();
    let open_role = wave0.replace("\"HUB\"", "\"UNKNOWN_REPOSITORY\"");
    assert!(bullet_wire::decode_canonical::<Wave0SubjectV1>(open_role.as_bytes()).is_err());
}

#[test]
fn inventory_refuses_duplicate_oversize_unsafe_path_mode_digest_and_count() {
    let mut invalid_subjects = Vec::new();

    let mut duplicate = inventory_subject();
    duplicate.nodes[1].relative_path_hex = duplicate.nodes[0].relative_path_hex.clone();
    invalid_subjects.push(duplicate);

    let mut reordered = inventory_subject();
    reordered.nodes.swap(0, 1);
    invalid_subjects.push(reordered);

    let mut missing_parent = inventory_subject();
    missing_parent.nodes.remove(0);
    missing_parent.node_count -= 1;
    missing_parent.directory_count -= 1;
    invalid_subjects.push(missing_parent);

    let mut oversized = inventory_subject();
    oversized.nodes = (0..=inventory::MAX_INVENTORY_NODES)
        .map(|index| regular(format!("files/{index:04}").as_bytes(), 'a', 1))
        .collect();
    oversized.node_count = oversized.nodes.len() as u64;
    oversized.directory_count = 0;
    oversized.regular_file_count = oversized.node_count;
    oversized.regular_file_byte_length = oversized.node_count;
    invalid_subjects.push(oversized);

    let mut unsafe_number = inventory_subject();
    unsafe_number.source_directory.device = MAX_SAFE_INTEGER + 1;
    invalid_subjects.push(unsafe_number);

    let mut unsafe_path = inventory_subject();
    unsafe_path.nodes[0].relative_path_hex = path_hex(b"../claims");
    invalid_subjects.push(unsafe_path);

    let mut overlong_path = inventory_subject();
    overlong_path.nodes[0].relative_path_hex = path_hex(&vec![b'a'; MAX_PATH_BYTES + 1]);
    invalid_subjects.push(overlong_path);

    let mut unsafe_mode = inventory_subject();
    unsafe_mode.nodes[0].mode = 0o10000;
    invalid_subjects.push(unsafe_mode);

    let mut missing_digest = inventory_subject();
    missing_digest.nodes[1].content_sha256 = None;
    invalid_subjects.push(missing_digest);

    let mut bad_digest = inventory_subject();
    bad_digest.nodes[1].content_sha256 = Some(digest("sha256:", 'A'));
    invalid_subjects.push(bad_digest);

    let mut bad_count = inventory_subject();
    bad_count.regular_file_count += 1;
    invalid_subjects.push(bad_count);

    for subject in invalid_subjects {
        assert!(IncidentInventoryV1::from_subject(subject).is_err());
    }
}

#[test]
fn wave0_refuses_member_claim_review_and_identity_drift() {
    let valid = wave0();
    let mut invalid = Vec::new();

    let mut missing = valid.clone();
    missing.facts.members.pop();
    invalid.push(missing);

    let mut extra = valid.clone();
    extra.facts.members.push(extra.facts.members[0].clone());
    invalid.push(extra);

    let mut reordered = valid.clone();
    reordered.facts.members.swap(0, 1);
    invalid.push(reordered);

    let mut dirty = valid.clone();
    dirty.facts.members[0].untracked_state = Wave0CleanStateV1::Dirty;
    invalid.push(dirty);

    let mut claimed = valid.clone();
    claimed.facts.claim_high_water.active_claim_count = 1;
    invalid.push(claimed);

    let mut unsafe_number = valid.clone();
    unsafe_number.facts.claim_high_water.byte_length = MAX_SAFE_INTEGER + 1;
    invalid.push(unsafe_number);

    let mut oversized_ledger = valid.clone();
    oversized_ledger.facts.claim_high_water.byte_length = MAX_CLAIM_LEDGER_BYTES + 1;
    invalid.push(oversized_ledger);

    let mut self_review = valid.clone();
    self_review.review.reviewer_principal = self_review.facts.producer_principal.clone();
    invalid.push(self_review);

    let mut repo_drift = valid.clone();
    repo_drift.facts.members[0].repository_identity = "root/not-bullet-farm".to_owned();
    invalid.push(repo_drift);

    let mut review_drift = valid.clone();
    review_drift.review.reviewed_facts_blake3 = digest("blake3:", '9');
    invalid.push(review_drift);

    let mut identity_drift = valid;
    identity_drift.subject_id = format!("w0_{}", "9".repeat(64));
    invalid.push(identity_drift);

    for subject in invalid {
        assert!(subject.validate().is_err());
    }
}

#[test]
fn sealed_references_refuse_substitution_and_unsafe_subjects() {
    let valid = references();
    let mut invalid = Vec::new();

    let mut wrong_kind = valid.clone();
    wrong_kind.incident_inventory.record_kind = FreshGenesisRecordKindV1::Wave0SubjectV1;
    invalid.push(wrong_kind);

    let mut same_path = valid.clone();
    same_path.wave0_subject.absolute_path_hex =
        same_path.incident_inventory.absolute_path_hex.clone();
    invalid.push(same_path);

    let mut relative = valid.clone();
    relative.wave0_subject.absolute_path_hex = path_hex(b"relative/w0.json");
    invalid.push(relative);

    let mut bad_digest = valid.clone();
    bad_digest.incident_inventory.sealed_sha256 = digest("sha256:", 'A');
    invalid.push(bad_digest);

    let mut oversized_record = valid;
    oversized_record.wave0_subject.byte_length = MAX_SEALED_RECORD_BYTES + 1;
    invalid.push(oversized_record);

    for refs in invalid {
        assert!(refs.validate().is_err());
    }
}
