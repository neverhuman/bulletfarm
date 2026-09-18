use std::{
    fs,
    os::unix::{
        ffi::OsStringExt,
        fs::{MetadataExt, PermissionsExt},
        net::UnixListener,
    },
    path::{Path, PathBuf},
};

#[cfg(target_os = "linux")]
use super::incident::{observe_incident_inventory, verify_retired_incident_inventory};
use super::*;
use crate::coord::model::{
    IncidentDirectoryIdentityV1, IncidentInventoryNodeTypeV1, IncidentInventoryNodeV1,
    IncidentInventorySubjectV1, Wave0ClaimHighWaterV1, Wave0CleanStateV1, Wave0FactsV1,
    Wave0MemberRoleV1, Wave0MemberV1,
};

fn hex(value: &[u8]) -> String {
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

fn inventory() -> IncidentInventoryV1 {
    IncidentInventoryV1::from_subject(IncidentInventorySubjectV1 {
        source_directory: IncidentDirectoryIdentityV1 {
            absolute_path_hex: hex(b"/home/ubuntu/bullet/.bullet-family/coord"),
            device: 1,
            inode: 2,
            owner_uid: 1000,
            owner_gid: 1000,
            mode: 0o700,
            link_count: 2,
            byte_length: 64,
        },
        destination_name_hex: hex(b"coord-incident-v1"),
        node_count: 1,
        directory_count: 0,
        regular_file_count: 1,
        regular_file_byte_length: 8,
        nodes: vec![IncidentInventoryNodeV1 {
            relative_path_hex: hex(b"events.jsonl"),
            node_type: IncidentInventoryNodeTypeV1::RegularFile,
            owner_uid: 1000,
            owner_gid: 1000,
            mode: 0o400,
            link_count: 1,
            byte_length: 8,
            content_sha256: Some(digest("sha256:", 'a')),
        }],
    })
    .unwrap()
}

fn wave0() -> Wave0SubjectV1 {
    Wave0SubjectV1::from_reviewed(
        Wave0FactsV1 {
            producer_principal: "baseline-producer".to_owned(),
            claim_high_water: Wave0ClaimHighWaterV1 {
                claim_ledger_path_hex: hex(b"/home/ubuntu/bullet/AGENT_CHAT.md"),
                claim_ledger_sha256: digest("sha256:", 'b'),
                claim_projection_blake3: digest("blake3:", 'c'),
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
        },
        "independent-reviewer".to_owned(),
        hex(b"/var/lib/bullet/reviews/w0.json"),
        digest("sha256:", 'd'),
        64,
    )
    .unwrap()
}

fn private_dir(parent: &Path, name: &str) -> PathBuf {
    let path = parent.join(name);
    fs::create_dir(&path).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    path
}

fn write_existing(path: &Path, bytes: &[u8], mode: u32) {
    fs::write(path, bytes).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

#[test]
fn create_once_records_are_exact_and_response_loss_replays_by_readback() {
    let checkout = tempfile::tempdir().unwrap();
    let output = tempfile::tempdir().unwrap();
    fs::set_permissions(output.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let inventory_path = output.path().join("inventory.json");
    let wave0_path = output.path().join("w0.json");
    let inventory = inventory();
    let wave0 = wave0();

    let created = publish_records(
        checkout.path(),
        &inventory_path,
        &wave0_path,
        &inventory,
        &wave0,
    )
    .unwrap();
    assert_eq!(
        (
            created.inventory_outcome,
            created.wave0_outcome,
            created.references.subject_blake3().unwrap(),
        ),
        (
            FreshGenesisPublicationOutcome::Created,
            FreshGenesisPublicationOutcome::Created,
            created.references_subject_blake3.clone(),
        )
    );
    for (path, expected) in [
        (&inventory_path, canonical_lf(&inventory).unwrap()),
        (&wave0_path, canonical_lf(&wave0).unwrap()),
    ] {
        assert_eq!(fs::read(path).unwrap(), expected);
        let metadata = fs::symlink_metadata(path).unwrap();
        assert!(metadata.is_file());
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o400);
        assert_eq!(metadata.nlink(), 1);
    }

    let replay = publish_records(
        checkout.path(),
        &inventory_path,
        &wave0_path,
        &inventory,
        &wave0,
    )
    .unwrap();
    assert_eq!(
        (
            replay.inventory_outcome,
            replay.wave0_outcome,
            replay.references,
            replay.references_subject_blake3,
        ),
        (
            FreshGenesisPublicationOutcome::AdoptedExactExisting,
            FreshGenesisPublicationOutcome::AdoptedExactExisting,
            created.references,
            created.references_subject_blake3,
        )
    );
}

#[test]
fn invalid_paths_and_invalid_models_refuse_before_publication() {
    let checkout = tempfile::tempdir().unwrap();
    let output = tempfile::tempdir().unwrap();
    fs::set_permissions(output.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let inventory = inventory();
    let wave0 = wave0();

    let same = output.path().join("same.json");
    assert!(publish_records(checkout.path(), &same, &same, &inventory, &wave0).is_err());
    assert!(!same.exists());

    let inside = private_dir(checkout.path(), "sealed");
    let inside_inventory = inside.join("inventory.json");
    let inside_wave0 = inside.join("w0.json");
    assert!(
        publish_records(
            checkout.path(),
            &inside_inventory,
            &inside_wave0,
            &inventory,
            &wave0,
        )
        .is_err()
    );
    assert!(!inside_inventory.exists() && !inside_wave0.exists());

    let alias_inventory = output.path().join("missing/../inventory.json");
    let alias_wave0 = output.path().join("w0-alias.json");
    assert!(
        publish_records(
            checkout.path(),
            &alias_inventory,
            &alias_wave0,
            &inventory,
            &wave0,
        )
        .is_err()
    );
    assert!(!alias_wave0.exists());

    let mut invalid_wave0 = wave0;
    invalid_wave0.facts.producer_principal = "x".repeat(1_025);
    let invalid_inventory = output.path().join("invalid-inventory.json");
    let invalid_wave0_path = output.path().join("invalid-w0.json");
    assert!(
        publish_records(
            checkout.path(),
            &invalid_inventory,
            &invalid_wave0_path,
            &inventory,
            &invalid_wave0,
        )
        .is_err()
    );
    assert!(!invalid_inventory.exists() && !invalid_wave0_path.exists());
}

#[test]
fn unsafe_inventory_outputs_refuse_without_a_later_wave0_artifact() {
    let checkout = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let inventory = inventory();
    let wave0 = wave0();
    let exact = canonical_lf(&inventory).unwrap();

    for name in ["differing", "writable", "hardlink", "symlink", "oversize"] {
        let case = private_dir(root.path(), name);
        let inventory_path = case.join("inventory.json");
        let wave0_path = case.join("w0.json");
        match name {
            "differing" => write_existing(&inventory_path, b"{}\n", 0o400),
            "writable" => write_existing(&inventory_path, &exact, 0o600),
            "hardlink" => {
                write_existing(&inventory_path, &exact, 0o400);
                fs::hard_link(&inventory_path, case.join("second-link")).unwrap();
            }
            "symlink" => {
                let target = case.join("target");
                write_existing(&target, &exact, 0o400);
                std::os::unix::fs::symlink(&target, &inventory_path).unwrap();
            }
            "oversize" => write_existing(
                &inventory_path,
                &vec![b'x'; (MAX_RECORD_BYTES + 1) as usize],
                0o400,
            ),
            _ => unreachable!(),
        }
        assert!(
            publish_records(
                checkout.path(),
                &inventory_path,
                &wave0_path,
                &inventory,
                &wave0,
            )
            .is_err(),
            "accepted {name} inventory output"
        );
        assert!(!wave0_path.exists(), "published W0 after {name} refusal");
    }

    let real = private_dir(root.path(), "real-parent");
    let alias = root.path().join("alias-parent");
    std::os::unix::fs::symlink(&real, &alias).unwrap();
    assert!(
        publish_records(
            checkout.path(),
            &alias.join("inventory.json"),
            &alias.join("w0.json"),
            &inventory,
            &wave0,
        )
        .is_err()
    );
    assert!(!real.join("w0.json").exists());

    let unsafe_parent = root.path().join("unsafe-parent");
    fs::create_dir(&unsafe_parent).unwrap();
    fs::set_permissions(&unsafe_parent, fs::Permissions::from_mode(0o750)).unwrap();
    assert!(
        publish_records(
            checkout.path(),
            &unsafe_parent.join("inventory.json"),
            &unsafe_parent.join("w0.json"),
            &inventory,
            &wave0,
        )
        .is_err()
    );
    assert!(!unsafe_parent.join("w0.json").exists());
}

#[test]
fn exact_inventory_adopts_but_a_differing_later_record_still_refuses() {
    let checkout = tempfile::tempdir().unwrap();
    let output = tempfile::tempdir().unwrap();
    fs::set_permissions(output.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let inventory_path = output.path().join("inventory.json");
    let wave0_path = output.path().join("w0.json");
    let inventory = inventory();
    let wave0 = wave0();
    let inventory_bytes = canonical_lf(&inventory).unwrap();
    write_existing(&inventory_path, &inventory_bytes, 0o400);
    write_existing(&wave0_path, b"{}\n", 0o400);

    assert!(
        publish_records(
            checkout.path(),
            &inventory_path,
            &wave0_path,
            &inventory,
            &wave0,
        )
        .is_err()
    );
    assert_eq!(fs::read(&inventory_path).unwrap(), inventory_bytes);
    assert_eq!(fs::read(&wave0_path).unwrap(), b"{}\n");

    fs::remove_file(&wave0_path).unwrap();
    write_existing(&wave0_path, &canonical_lf(&wave0).unwrap(), 0o400);
    let adopted = publish_records(
        checkout.path(),
        &inventory_path,
        &wave0_path,
        &inventory,
        &wave0,
    )
    .unwrap();
    assert_eq!(
        (adopted.inventory_outcome, adopted.wave0_outcome),
        (
            FreshGenesisPublicationOutcome::AdoptedExactExisting,
            FreshGenesisPublicationOutcome::AdoptedExactExisting,
        )
    );
}

#[cfg(target_os = "linux")]
#[test]
fn descriptor_observer_binds_complete_tree_and_exact_moved_readback() {
    let parent = tempfile::tempdir().unwrap();
    let source = private_dir(parent.path(), "coord");
    let claims = private_dir(&source, "claims");
    write_existing(&source.join("events.jsonl"), b"events\n", 0o400);
    write_existing(&claims.join("active.jsonl"), b"claims\n", 0o400);
    let raw_name = std::ffi::OsString::from_vec(vec![b'f', 0xff]);
    write_existing(&source.join(&raw_name), b"raw\n", 0o400);
    assert!(observe_incident_inventory(&source, std::ffi::OsStr::new("coord")).is_err());

    let observed =
        observe_incident_inventory(&source, std::ffi::OsStr::new("coord-incident-v1")).unwrap();
    observed.validate().unwrap();
    assert_eq!(observed.subject.node_count, 4);
    assert_eq!(observed.subject.directory_count, 1);
    assert_eq!(observed.subject.regular_file_count, 3);
    assert_eq!(observed.subject.regular_file_byte_length, 18);
    assert!(
        observed
            .subject
            .nodes
            .windows(2)
            .all(|pair| pair[0].relative_path_hex < pair[1].relative_path_hex)
    );
    assert!(observed.subject.nodes.iter().any(|node| {
        node.relative_path_hex == hex(raw_name.as_encoded_bytes()) && node.content_sha256.is_some()
    }));

    assert!(verify_retired_incident_inventory(&observed).is_err());
    let retired = parent.path().join("coord-incident-v1");
    fs::rename(&source, &retired).unwrap();
    verify_retired_incident_inventory(&observed).unwrap();
    fs::set_permissions(
        retired.join("events.jsonl"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    assert!(verify_retired_incident_inventory(&observed).is_err());
}

#[cfg(target_os = "linux")]
#[test]
fn descriptor_observer_refuses_symlinks_and_special_nodes() {
    let parent = tempfile::tempdir().unwrap();
    let source = private_dir(parent.path(), "coord");
    write_existing(&source.join("events.jsonl"), b"events\n", 0o400);
    std::os::unix::fs::symlink("events.jsonl", source.join("alias")).unwrap();
    assert!(
        observe_incident_inventory(&source, std::ffi::OsStr::new("coord-incident-v1")).is_err()
    );
    fs::remove_file(source.join("alias")).unwrap();
    let _socket = UnixListener::bind(source.join("socket")).unwrap();
    assert!(
        observe_incident_inventory(&source, std::ffi::OsStr::new("coord-incident-v1")).is_err()
    );
}

#[test]
fn consume_refuses_missing_inventory() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing.json");
    let wave0 = dir.path().join("wave0.json");
    std::fs::write(&wave0, b"{}\n").unwrap();
    let error = consume_wave0_and_inventory(&missing, &wave0, dir.path()).unwrap_err();
    assert_eq!(error.code(), "INVALID_FRESH_GENESIS_PRODUCTION");
}
