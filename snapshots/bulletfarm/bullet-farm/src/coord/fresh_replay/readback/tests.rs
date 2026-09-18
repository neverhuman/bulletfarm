use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
};

use serde_json::{Value, json};

use super::*;
use crate::coord::model::{
    IncidentDirectoryIdentityV1, IncidentInventoryNodeV1, IncidentInventorySubjectV1,
};

fn write_raw(path: &Path, bytes: &[u8]) {
    if path.exists() {
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    }
    fs::write(path, bytes).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o400)).unwrap();
}

fn inventory(root: &Path, location: Location, bytes: &[u8]) -> IncidentInventoryV1 {
    let relative = if location == Location::Outer {
        ".bullet-family/coord"
    } else {
        "bullet-farm/.bullet-family/coord"
    };
    IncidentInventoryV1::from_subject(IncidentInventorySubjectV1 {
        source_directory: IncidentDirectoryIdentityV1 {
            absolute_path_hex: hex(root.join(relative).as_os_str().as_encoded_bytes()),
            device: 1,
            inode: if location == Location::Outer { 2 } else { 3 },
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
        regular_file_byte_length: bytes.len() as u64,
        nodes: vec![IncidentInventoryNodeV1 {
            relative_path_hex: hex(b"events.jsonl"),
            node_type: IncidentInventoryNodeTypeV1::RegularFile,
            owner_uid: 1000,
            owner_gid: 1000,
            mode: 0o400,
            link_count: 1,
            byte_length: bytes.len() as u64,
            content_sha256: Some(sha256(bytes)),
        }],
    })
    .unwrap()
}

struct Fixture {
    family: tempfile::TempDir,
    records: tempfile::TempDir,
    request: FreshReplayRequestV1,
    record: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let family = tempfile::tempdir().unwrap();
        fs::write(
            family.path().join("repos.manifest.toml"),
            "fixture = true\n",
        )
        .unwrap();
        let records = tempfile::tempdir().unwrap();
        fs::set_permissions(records.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let claim = json!({"kind":"claim","schema_version":1,"at_unix_ms":1000,
            "claim_id":"same","agent":"worker","lane":"docs","repo":"bullet-farm",
            "paths":["README.md"],"expires_unix_ms":31000});
        let frame = |values: &[Value]| {
            values
                .iter()
                .flat_map(|value| {
                    let mut bytes = serde_json::to_vec(value).unwrap();
                    bytes.push(b'\n');
                    bytes
                })
                .collect::<Vec<_>>()
        };
        let outer = frame(&[
            claim.clone(),
            json!({"kind":"handoff","schema_version":1,"at_unix_ms":2000,"claim_id":"same",
                "agent":"worker","proof_command":"just docs","proof_exit_code":0,
                "changed_paths":["README.md"],"commit_oid":null}),
            json!({"kind":"commit_receipt","schema_version":1,"at_unix_ms":3000,"claim_id":"same",
                "orchestrator":"integrator","commit_oid":"a".repeat(40),"committed_paths":["README.md"]}),
        ]);
        let hub = frame(&[claim]);
        let pair = FreshPreservationSubjectV1::from_inventories(
            hex(family.path().as_os_str().as_encoded_bytes()),
            inventory(family.path(), Location::Outer, &outer),
            inventory(family.path(), Location::Hub, &hub),
        )
        .unwrap();
        let pair_path = records.path().join("pair.json");
        sealed::write(&pair_path, &pair).unwrap();
        let pair_bytes = canonical_lf(&pair).unwrap();
        let outer_path = records.path().join("outer.jsonl");
        let hub_path = records.path().join("hub.jsonl");
        write_raw(&outer_path, &outer);
        write_raw(&hub_path, &hub);
        let dispositions = [(Location::Outer, &outer), (Location::Hub, &hub)]
            .into_iter()
            .flat_map(|(location, bytes)| {
                replay(bytes, "fixture")
                    .unwrap()
                    .claims
                    .into_values()
                    .map(move |claim| ClaimDisposition {
                        location,
                        claim_id: claim.claim_id.clone(),
                        claim_subject_id: subject_id("frc_", CLAIM_DOMAIN, &claim).unwrap(),
                        disposition: claim
                            .commit_oid
                            .map_or(Disposition::RetainForRecovery, |commit_oid| {
                                Disposition::RecordedReceipt { commit_oid }
                            }),
                    })
            })
            .collect();
        let request = FreshReplayRequestV1 {
            kind: RequestKind::FreshReplayRequestV1,
            schema_version: 1,
            preservation: PreservationReference {
                path: pair_path,
                preservation_id: pair.preservation_id,
                sealed_sha256: sha256(&pair_bytes),
                byte_length: pair_bytes.len() as u64,
            },
            outer_ledger_copy: outer_path,
            hub_ledger_copy: hub_path,
            dispositions,
        };
        let request_path = records.path().join("request.json");
        sealed::write(&request_path, &request).unwrap();
        let record = records.path().join("replay.json");
        publish(family.path(), &request_path, &record).unwrap();
        Self {
            family,
            records,
            request,
            record,
        }
    }

    fn cli(&self, args: &[&str]) -> Result<crate::cli::CliOutcome, CoordError> {
        let mut options = vec![
            "bullet-family".into(),
            "--root".into(),
            self.family.path().into(),
            "preservation-bind".into(),
            "replay-verify".into(),
        ];
        options.extend(args.iter().map(std::ffi::OsString::from));
        crate::cli::execute(options, Ok(self.family.path().into()))
    }

    fn snapshot(&self) -> Vec<(std::ffi::OsString, u64, u32, Vec<u8>)> {
        let mut entries = fs::read_dir(self.records.path())
            .unwrap()
            .map(|entry| {
                let entry = entry.unwrap();
                let metadata = entry.metadata().unwrap();
                (
                    entry.file_name(),
                    metadata.ino(),
                    metadata.mode(),
                    fs::read(entry.path()).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        entries.sort();
        entries
    }

    fn no_incidents(&self) {
        assert!(!self.family.path().join(".bullet-family").exists());
        assert!(!self.family.path().join("bullet-farm").exists());
    }
}

#[test]
fn replay_verify_cli_reconstructs_exact_facts_without_writes() {
    let fixture = Fixture::new();
    // The producer embeds its complete closed request; no external request path
    // was retained in this record, and none may be invented during read-back.
    fs::remove_file(fixture.records.path().join("request.json")).unwrap();
    let before = fixture.snapshot();
    let bytes = fs::read(&fixture.record).unwrap();
    let subject: Value = sealed::read(&fixture.record).unwrap();
    assert_eq!(
        subject["facts"]["outer"]["claims"]["same"]["state"],
        "handed_off"
    );
    assert_eq!(subject["facts"]["hub"]["claims"]["same"]["state"], "active");
    for _ in 0..2 {
        let outcome = fixture
            .cli(&["--record", fixture.record.to_str().unwrap()])
            .unwrap();
        assert_eq!(outcome.exit_code(), 0);
        let verified: Value = serde_json::from_str(outcome.output()).unwrap();
        assert_eq!(
            verified,
            json!({"kind":"FRESH_REPLAY_VERIFICATION_V1","schema_version":1,
            "purpose":"REPLAY_FACTS_READBACK_ONLY","replay_id":subject["replay_id"],
            "record":fixture.record,"sealed_sha256":sha256(&bytes),"byte_length":bytes.len()})
        );
        assert_eq!(fixture.snapshot(), before);
    }
    for args in [
        vec![],
        vec!["--record"],
        vec!["--record", "relative.json"],
        vec![
            "--record",
            fixture.record.to_str().unwrap(),
            "--out",
            "other",
        ],
        vec![
            "--record",
            fixture.record.to_str().unwrap(),
            "--record",
            fixture.record.to_str().unwrap(),
        ],
        vec![
            "--record",
            fixture.record.to_str().unwrap(),
            "--operator-approved",
        ],
    ] {
        assert!(fixture.cli(&args).is_err(), "{args:?}");
        assert_eq!(fixture.snapshot(), before);
    }
    fixture.no_incidents();
}

#[test]
fn replay_verify_refuses_unclosed_noncanonical_and_substituted_records() {
    let fixture = Fixture::new();
    let original: Value = sealed::read(&fixture.record).unwrap();
    let bytes = fs::read(&fixture.record).unwrap();
    for variant in [
        "kind",
        "schema",
        "purpose",
        "id",
        "computed",
        "claim",
        "outer-extra",
        "facts-extra",
        "request-extra",
        "missing-request",
        "missing-disposition",
        "claim-hash",
        "pair-hash",
        "role-swap",
        "duplicate-path",
        "relative-path",
        "inside-path",
        "record-path",
    ] {
        let mut value = original.clone();
        match variant {
            "kind" => value["facts"]["kind"] = json!("OPERATOR_APPROVED"),
            "schema" => value["facts"]["schema_version"] = json!(2),
            "purpose" => value["facts"]["purpose"] = json!("GENESIS_ADMISSION"),
            "id" => value["replay_id"] = json!(format!("fgr_{}", "0".repeat(64))),
            "computed" => value["facts"]["outer"]["record_count"] = json!(1),
            "claim" => value["facts"]["hub"]["claims"]["same"]["state"] = json!("handed_off"),
            "outer-extra" => value["operator_approved"] = json!(true),
            "facts-extra" => value["facts"]["reviewer_approved"] = json!(true),
            "request-extra" => value["facts"]["request"]["operator_approved"] = json!(true),
            "missing-request" => {
                value["facts"].as_object_mut().unwrap().remove("request");
            }
            "missing-disposition" => {
                value["facts"]["request"]["dispositions"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            }
            "claim-hash" => {
                value["facts"]["request"]["dispositions"][0]["claim_subject_id"] =
                    json!(format!("frc_{}", "0".repeat(64)))
            }
            "pair-hash" => {
                value["facts"]["request"]["preservation"]["sealed_sha256"] =
                    json!(format!("sha256:{}", "0".repeat(64)))
            }
            "role-swap" => {
                value["facts"]["request"]["outer_ledger_copy"] =
                    json!(fixture.request.hub_ledger_copy);
                value["facts"]["request"]["hub_ledger_copy"] =
                    json!(fixture.request.outer_ledger_copy);
            }
            "duplicate-path" => {
                value["facts"]["request"]["hub_ledger_copy"] =
                    json!(fixture.request.outer_ledger_copy)
            }
            "relative-path" => value["facts"]["request"]["hub_ledger_copy"] = json!("hub.jsonl"),
            "inside-path" => {
                value["facts"]["request"]["hub_ledger_copy"] =
                    json!(fixture.family.path().join("hub.jsonl"))
            }
            "record-path" => value["facts"]["request"]["hub_ledger_copy"] = json!(fixture.record),
            _ => unreachable!(),
        }
        if variant != "id" {
            value["replay_id"] = json!(subject_id("fgr_", REPLAY_DOMAIN, &value["facts"]).unwrap());
        }
        write_raw(&fixture.record, &canonical_lf(&value).unwrap());
        let before = fixture.snapshot();
        assert!(
            fixture
                .cli(&["--record", fixture.record.to_str().unwrap()])
                .is_err(),
            "{variant}"
        );
        assert_eq!(fixture.snapshot(), before);
    }
    for malformed in [
        bytes[..bytes.len() - 1].to_vec(),
        [b" ".as_slice(), &bytes].concat(),
        [bytes.as_slice(), b"\n"].concat(),
        [b"{\"replay_id\":\"duplicate\",".as_slice(), &bytes[1..]].concat(),
        b"{\"facts\":NaN}\n".to_vec(),
        vec![b'x'; MAX_BYTES as usize + 2],
    ] {
        write_raw(&fixture.record, &malformed);
        assert!(verify(fixture.family.path(), &fixture.record).is_err());
        assert_eq!(fs::read(&fixture.record).unwrap(), malformed);
    }
    write_raw(&fixture.record, &bytes);
    let alias = fixture.records.path().join("alias");
    std::os::unix::fs::symlink(&fixture.record, &alias).unwrap();
    assert!(verify(fixture.family.path(), &alias).is_err());
    fs::remove_file(&alias).unwrap();
    fs::hard_link(&fixture.record, &alias).unwrap();
    assert!(verify(fixture.family.path(), &fixture.record).is_err());
    fs::remove_file(&alias).unwrap();
    fs::set_permissions(&fixture.record, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(verify(fixture.family.path(), &fixture.record).is_err());
    fs::set_permissions(&fixture.record, fs::Permissions::from_mode(0o400)).unwrap();
    let wrong_root = tempfile::tempdir().unwrap();
    assert!(verify(wrong_root.path(), &fixture.record).is_err());
    assert!(verify(fixture.family.path(), &fixture.record).is_ok());
    fixture.no_incidents();
}

fn with_drift<T>(path: PathBuf, bytes: Vec<u8>, operation: impl FnOnce() -> T) -> T {
    struct Reset(Option<Box<dyn FnOnce()>>);
    impl Drop for Reset {
        fn drop(&mut self) {
            AFTER_RECONSTRUCTION.with(|slot| *slot.borrow_mut() = self.0.take());
        }
    }
    let _reset = Reset(
        AFTER_RECONSTRUCTION
            .with(|slot| slot.replace(Some(Box::new(move || write_raw(&path, &bytes))))),
    );
    operation()
}

#[test]
fn replay_verify_revalidates_every_input_and_refuses_bound_interrupted_history() {
    let fixture = Fixture::new();
    for path in [
        &fixture.request.preservation.path,
        &fixture.request.outer_ledger_copy,
        &fixture.request.hub_ledger_copy,
        &fixture.record,
    ] {
        let original = fs::read(path).unwrap();
        let changed = [original.as_slice(), b" "].concat();
        let error = with_drift(path.to_owned(), changed.clone(), || {
            verify(fixture.family.path(), &fixture.record)
        })
        .unwrap_err();
        assert_eq!(
            error.code(),
            "FRESH_GENESIS_SUBJECT_CHANGED",
            "{}: {error}",
            path.display()
        );
        assert_eq!(fs::read(path).unwrap(), changed);
        assert!(verify(fixture.family.path(), &fixture.record).is_err());
        write_raw(path, &original);
        assert!(verify(fixture.family.path(), &fixture.record).is_ok());
    }
    let mut value: Value = sealed::read(&fixture.record).unwrap();
    let mut outer = fs::read(&fixture.request.outer_ledger_copy).unwrap();
    outer.pop();
    write_raw(&fixture.request.outer_ledger_copy, &outer);
    let pair = FreshPreservationSubjectV1::from_inventories(
        hex(fixture.family.path().as_os_str().as_encoded_bytes()),
        inventory(fixture.family.path(), Location::Outer, &outer),
        inventory(
            fixture.family.path(),
            Location::Hub,
            &fs::read(&fixture.request.hub_ledger_copy).unwrap(),
        ),
    )
    .unwrap();
    let pair_bytes = canonical_lf(&pair).unwrap();
    write_raw(&fixture.request.preservation.path, &pair_bytes);
    value["facts"]["request"]["preservation"] = json!({"path":fixture.request.preservation.path,
        "preservation_id":pair.preservation_id,"sealed_sha256":sha256(&pair_bytes),"byte_length":pair_bytes.len()});
    value["replay_id"] = json!(subject_id("fgr_", REPLAY_DOMAIN, &value["facts"]).unwrap());
    write_raw(&fixture.record, &canonical_lf(&value).unwrap());
    let before = fixture.snapshot();
    let error = verify(fixture.family.path(), &fixture.record).unwrap_err();
    assert_eq!(error.code(), "CORRUPT_COORD_LOG", "{error}");
    assert_eq!(fixture.snapshot(), before);
    fixture.no_incidents();
}
