use std::path::Path;

use super::*;
use crate::coord::model::{
    IncidentDirectoryIdentityV1, IncidentInventoryNodeTypeV1, IncidentInventoryNodeV1,
    IncidentInventorySubjectV1,
};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn inventory(root: &Path, hub: bool) -> IncidentInventoryV1 {
    let source = root.join(if hub {
        "bullet-farm/.bullet-family/coord"
    } else {
        ".bullet-family/coord"
    });
    IncidentInventoryV1::from_subject(IncidentInventorySubjectV1 {
        source_directory: IncidentDirectoryIdentityV1 {
            absolute_path_hex: hex(source.as_os_str().as_encoded_bytes()),
            device: 1,
            inode: if hub { 3 } else { 2 },
            owner_uid: 1000,
            owner_gid: 1000,
            mode: 0o700,
            link_count: 2,
            byte_length: 64,
        },
        destination_name_hex: hex(b"coord-preserved"),
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
            content_sha256: Some(format!("sha256:{}", "a".repeat(64))),
        }],
    })
    .unwrap()
}

fn pair(root: &Path) -> FreshPreservationSubjectV1 {
    FreshPreservationSubjectV1::from_inventories(
        hex(root.as_os_str().as_encoded_bytes()),
        inventory(root, false),
        inventory(root, true),
    )
    .unwrap()
}

#[test]
fn preservation_pair_is_canonical_and_binds_both_complete_inventories() {
    let root = Path::new("/preservation-fixture");
    let subject = pair(root);
    let bytes = bullet_wire::canonical_json(&subject).unwrap();
    let decoded: FreshPreservationSubjectV1 = bullet_wire::decode_canonical(&bytes).unwrap();
    decoded.validate().unwrap();
    assert_eq!(decoded, subject);
    assert!(subject.preservation_id.starts_with("fgp_"));
    assert_ne!(
        subject.preservation_id,
        subject.outer_inventory.inventory_id
    );

    for hub in [false, true] {
        let mut changed = subject.clone();
        let original = if hub {
            &mut changed.hub_inventory
        } else {
            &mut changed.outer_inventory
        };
        let mut body = original.subject.clone();
        body.nodes[0].content_sha256 = Some(format!("sha256:{}", "b".repeat(64)));
        *original = IncidentInventoryV1::from_subject(body).unwrap();
        assert!(changed.validate().is_err());
        let rebound = FreshPreservationSubjectV1::from_inventories(
            changed.family_root_path_hex,
            changed.outer_inventory,
            changed.hub_inventory,
        )
        .unwrap();
        assert_ne!(rebound.preservation_id, subject.preservation_id);
    }
}

#[test]
fn preservation_pair_refuses_omissions_wrong_roles_and_malformed_originals() {
    let root = Path::new("/preservation-fixture");
    let original = pair(root);
    for missing in ["outer_inventory", "hub_inventory", "family_root_path_hex"] {
        let mut value = serde_json::to_value(&original).unwrap();
        value.as_object_mut().unwrap().remove(missing);
        assert!(
            bullet_wire::decode_canonical::<FreshPreservationSubjectV1>(
                &bullet_wire::canonical_json(&value).unwrap()
            )
            .is_err()
        );
    }
    let mut invented = serde_json::to_value(&original).unwrap();
    invented["operator_approved"] = serde_json::json!(true);
    assert!(
        bullet_wire::decode_canonical::<FreshPreservationSubjectV1>(
            &bullet_wire::canonical_json(&invented).unwrap()
        )
        .is_err()
    );

    for variant in [
        "root",
        "roles",
        "identity",
        "destination",
        "nodes",
        "schema",
        "id",
    ] {
        let mut subject = original.clone();
        match variant {
            "root" => subject.family_root_path_hex = hex(b"/preservation-fixture/.."),
            "roles" => std::mem::swap(&mut subject.outer_inventory, &mut subject.hub_inventory),
            "identity" => {
                let mut body = subject.hub_inventory.subject.clone();
                body.source_directory.inode =
                    subject.outer_inventory.subject.source_directory.inode;
                subject.hub_inventory = IncidentInventoryV1::from_subject(body).unwrap();
            }
            "destination" => subject.hub_inventory.subject.destination_name_hex = hex(b"coord"),
            "nodes" => subject.outer_inventory.subject.nodes.clear(),
            "schema" => subject.schema_version = 2,
            "id" => subject.preservation_id = format!("fgp_{}", "a".repeat(64)),
            _ => unreachable!(),
        }
        assert!(subject.validate().is_err(), "accepted {variant}");
        if !matches!(variant, "schema" | "id") {
            assert!(
                FreshPreservationSubjectV1::from_inventories(
                    subject.family_root_path_hex,
                    subject.outer_inventory,
                    subject.hub_inventory,
                )
                .is_err(),
                "rebound {variant}"
            );
        }
    }
}

#[test]
fn preservation_pair_enforces_the_combined_canonical_document_bound() {
    let root = Path::new("/preservation-fixture");
    let large = |hub| {
        let mut body = inventory(root, hub).subject;
        let template = body.nodes[0].clone();
        body.nodes = (0..IncidentInventoryV1::maximum_nodes())
            .map(|index| IncidentInventoryNodeV1 {
                relative_path_hex: hex(format!("preservation-record-{index:04}").as_bytes()),
                ..template.clone()
            })
            .collect();
        body.node_count = body.nodes.len() as u64;
        body.regular_file_count = body.node_count;
        body.regular_file_byte_length = body.node_count * template.byte_length;
        IncidentInventoryV1::from_subject(body).unwrap()
    };
    let outer = large(false);
    let hub = large(true);
    let outer_bytes = bullet_wire::canonical_json(&outer).unwrap();
    let hub_bytes = bullet_wire::canonical_json(&hub).unwrap();
    assert!(outer_bytes.len() <= bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES);
    assert!(hub_bytes.len() <= bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES);
    assert!(outer_bytes.len() + hub_bytes.len() > bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES);
    assert!(
        FreshPreservationSubjectV1::from_inventories(
            hex(root.as_os_str().as_encoded_bytes()),
            outer,
            hub,
        )
        .is_err()
    );
}

#[cfg(target_os = "linux")]
#[test]
fn preservation_publication_retries_exactly_and_preserves_conflicting_bytes() {
    use sha2::{Digest, Sha256};
    use std::{
        fs,
        os::unix::fs::{MetadataExt, PermissionsExt},
    };

    use crate::coord::fresh_genesis::{
        FreshGenesisPublicationOutcome, publish_preservation_record,
    };

    let family = tempfile::tempdir().unwrap();
    let outputs = tempfile::tempdir().unwrap();
    fs::set_permissions(outputs.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let output = outputs.path().join("preservation.json");
    let original = pair(family.path());
    let publish = |subject: &FreshPreservationSubjectV1| {
        publish_preservation_record(
            family.path(),
            &output,
            &subject.outer_inventory,
            &subject.hub_inventory,
        )
    };
    let first = publish(&original).unwrap();
    assert_eq!(first.outcome, FreshGenesisPublicationOutcome::Created);
    assert_eq!(first.subject, original);
    let bytes = fs::read(&output).unwrap();
    let mut expected = bullet_wire::canonical_json(&original).unwrap();
    expected.push(b'\n');
    assert_eq!(bytes, expected);
    assert_eq!(first.byte_length, bytes.len() as u64);
    assert_eq!(
        first.sealed_sha256,
        format!("sha256:{:x}", Sha256::digest(&bytes))
    );
    assert_eq!(
        fs::metadata(&output).unwrap().permissions().mode() & 0o7777,
        0o400
    );
    let retried = publish(&original).unwrap();
    assert_eq!(
        retried.outcome,
        FreshGenesisPublicationOutcome::AdoptedExactExisting
    );
    assert_eq!(retried.subject, first.subject);
    assert_eq!(retried.sealed_sha256, first.sealed_sha256);

    let interrupted_output = outputs.path().join("sync-interrupted.json");
    let interrupted = || {
        publish_preservation_record(
            family.path(),
            &interrupted_output,
            &original.outer_inventory,
            &original.hub_inventory,
        )
    };
    let failure = crate::coord::sealed::with_directory_sync_failure(|| interrupted().unwrap_err());
    assert_eq!(failure.code(), "FRESH_GENESIS_SUBJECT_CHANGED");
    assert_eq!(fs::read(&interrupted_output).unwrap(), bytes);
    let retained = fs::metadata(&interrupted_output).unwrap();
    let recovered = interrupted().unwrap();
    assert_eq!(
        recovered.outcome,
        FreshGenesisPublicationOutcome::AdoptedExactExisting
    );
    assert_eq!(recovered.subject, first.subject);
    assert_eq!(recovered.sealed_sha256, first.sealed_sha256);
    let after = fs::metadata(&interrupted_output).unwrap();
    assert_eq!((after.dev(), after.ino()), (retained.dev(), retained.ino()));
    assert_eq!(fs::read(&interrupted_output).unwrap(), bytes);

    let mut changed = original;
    let mut body = changed.hub_inventory.subject.clone();
    body.destination_name_hex = hex(b"different-preservation");
    changed.hub_inventory = IncidentInventoryV1::from_subject(body).unwrap();
    assert_eq!(
        publish(&changed).unwrap_err().code(),
        "FRESH_GENESIS_SUBJECT_CHANGED"
    );
    assert_eq!(fs::read(&output).unwrap(), bytes);
    assert!(!family.path().join(".bullet-family").exists());
}

#[cfg(target_os = "linux")]
#[test]
fn preservation_publication_refuses_unsafe_output_without_incident_effects() {
    use std::{fs, os::unix::fs::PermissionsExt};

    use crate::coord::fresh_genesis::publish_preservation_record;

    let family = tempfile::tempdir().unwrap();
    let inputs = pair(family.path());
    let publish = |output: &Path| {
        publish_preservation_record(
            family.path(),
            output,
            &inputs.outer_inventory,
            &inputs.hub_inventory,
        )
    };
    assert!(publish(&family.path().join("preservation.json")).is_err());
    assert!(publish(Path::new("relative.json")).is_err());
    let mut bytes = bullet_wire::canonical_json(&inputs).unwrap();
    bytes.push(b'\n');
    for variant in ["different", "writable", "symlink", "hardlink", "no-lf"] {
        let directory = tempfile::tempdir().unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let output = directory.path().join("preservation.json");
        let written = match variant {
            "different" => b"{}\n".as_slice(),
            "no-lf" => &bytes[..bytes.len() - 1],
            _ => &bytes,
        };
        let stored = if variant == "symlink" {
            directory.path().join("target.json")
        } else {
            output.clone()
        };
        fs::write(&stored, written).unwrap();
        fs::set_permissions(
            &stored,
            fs::Permissions::from_mode(if variant == "writable" { 0o600 } else { 0o400 }),
        )
        .unwrap();
        match variant {
            "symlink" => std::os::unix::fs::symlink(&stored, &output).unwrap(),
            "hardlink" => fs::hard_link(&output, directory.path().join("alias.json")).unwrap(),
            _ => {}
        }
        assert!(publish(&output).is_err(), "accepted {variant}");
        assert_eq!(fs::read(&stored).unwrap(), written);
    }
    assert!(!family.path().join(".bullet-family").exists());
}
