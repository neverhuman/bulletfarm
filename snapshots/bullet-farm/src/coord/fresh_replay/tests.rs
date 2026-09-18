use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
};

use serde_json::{Value, json};

use super::*;
use crate::coord::model::{
    IncidentDirectoryIdentityV1, IncidentInventoryNodeV1, IncidentInventorySubjectV1,
};

fn claim(id: &str, at: u64) -> Value {
    json!({"kind":"claim", "schema_version":1, "at_unix_ms":at,
        "claim_id":id, "agent":"worker", "lane":"docs", "repo":"bullet-farm",
        "paths":["README.md"], "expires_unix_ms":at+30_000})
}

fn framed(values: &[Value]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| {
            let mut bytes = serde_json::to_vec(value).unwrap();
            bytes.push(b'\n');
            bytes
        })
        .collect()
}

fn ledgers() -> (Vec<u8>, Vec<u8>) {
    let handoff = json!({"kind":"handoff", "schema_version":1, "at_unix_ms":42_000,
        "claim_id":"beta", "agent":"worker", "proof_command":"just docs",
        "proof_exit_code":0, "changed_paths":["README.md"], "commit_oid":null});
    let receipt = json!({"kind":"commit_receipt", "schema_version":1, "at_unix_ms":43_000,
        "claim_id":"beta", "orchestrator":"integrator", "commit_oid":"a".repeat(40),
        "committed_paths":["README.md"]});
    let correction = json!({"kind":"commit_receipt_correction", "schema_version":1, "at_unix_ms":44_000,
        "claim_id":"beta", "orchestrator":"integrator", "previous_commit_oid":"a".repeat(40),
        "commit_oid":"c".repeat(40), "committed_paths":["README.md"], "reason":"fixture correction"});
    (
        framed(&[
            claim("alpha", 1_000),
            claim("beta", 41_000),
            handoff,
            receipt,
            correction,
        ]),
        framed(&[claim("alpha", 1_000)]),
    )
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
}

impl Fixture {
    fn new() -> Self {
        let (outer, hub) = ledgers();
        Self::with_ledgers(&outer, &hub)
    }

    fn with_ledgers(outer: &[u8], hub: &[u8]) -> Self {
        let family = tempfile::tempdir().unwrap();
        fs::write(
            family.path().join("repos.manifest.toml"),
            "fixture = true\n",
        )
        .unwrap();
        let records = tempfile::tempdir().unwrap();
        fs::set_permissions(records.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let pair = FreshPreservationSubjectV1::from_inventories(
            hex(family.path().as_os_str().as_encoded_bytes()),
            inventory(family.path(), Location::Outer, outer),
            inventory(family.path(), Location::Hub, hub),
        )
        .unwrap();
        let pair_path = records.path().join("pair.json");
        sealed::write(&pair_path, &pair).unwrap();
        let outer_path = records.path().join("outer.jsonl");
        let hub_path = records.path().join("hub.jsonl");
        for (path, bytes) in [(&outer_path, outer), (&hub_path, hub)] {
            fs::write(path, bytes).unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o400)).unwrap();
        }
        let pair_bytes = canonical_lf(&pair).unwrap();
        let mut dispositions = Vec::new();
        for (location, bytes) in [(Location::Outer, outer), (Location::Hub, hub)] {
            if let Ok(replayed) = replay(bytes, "fixture") {
                for summary in replayed.claims.values() {
                    dispositions.push(ClaimDisposition {
                        location,
                        claim_id: summary.claim_id.clone(),
                        claim_subject_id: subject_id("frc_", CLAIM_DOMAIN, summary).unwrap(),
                        disposition: summary
                            .commit_oid
                            .clone()
                            .map_or(Disposition::RetainForRecovery, |commit_oid| {
                                Disposition::RecordedReceipt { commit_oid }
                            }),
                    });
                }
            }
        }
        Self {
            family,
            records,
            request: FreshReplayRequestV1 {
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
            },
        }
    }

    fn run(
        &self,
        request: &FreshReplayRequestV1,
        name: &str,
    ) -> Result<ReplayPublication, CoordError> {
        let path = self.records.path().join(format!("{name}.request.json"));
        sealed::write(&path, request).unwrap();
        publish(
            self.family.path(),
            &path,
            &self.records.path().join(format!("{name}.out.json")),
        )
    }

    fn no_incidents(&self) {
        assert!(!self.family.path().join(".bullet-family").exists());
        assert!(!self.family.path().join("bullet-farm").exists());
    }
}

#[test]
fn replay_cli_binds_both_ledgers_and_every_disposition_without_incident_effects() {
    let fixture = Fixture::new();
    let request = fixture.records.path().join("request.json");
    let output = fixture.records.path().join("replay.json");
    sealed::write(&request, &fixture.request).unwrap();
    let args = [
        "bullet-family",
        "--root",
        fixture.family.path().to_str().unwrap(),
        "preservation-bind",
        "replay",
        "--request",
        request.to_str().unwrap(),
        "--out",
        output.to_str().unwrap(),
    ];
    let run = || {
        crate::cli::run(
            args.iter().map(|arg| (*arg).into()),
            Ok(fixture.family.path().into()),
        )
    };
    let first = run().unwrap();
    let first = bullet_wire::decode_unique_value(first.as_bytes()).unwrap();
    assert_eq!(first["outcome"], "CREATED");
    let bytes = fs::read(&output).unwrap();
    let value = bullet_wire::decode_canonical_value(&bytes[..bytes.len() - 1]).unwrap();
    assert_eq!(value["facts"]["purpose"], "REPLAY_FACTS_ONLY");
    assert_eq!(
        value["facts"]["outer"]["claims"]["alpha"]["state"],
        "expired"
    );
    assert_eq!(value["facts"]["hub"]["claims"]["alpha"]["state"], "active");
    assert_eq!(
        value["facts"]["outer"]["claims"]["beta"]["commit_oid"],
        "c".repeat(40)
    );
    assert_eq!(
        value["facts"]["request"]["dispositions"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        value["replay_id"],
        subject_id("fgr_", REPLAY_DOMAIN, &value["facts"]).unwrap()
    );
    let retried = run().unwrap();
    assert_eq!(
        bullet_wire::decode_unique_value(retried.as_bytes()).unwrap()["outcome"],
        "ADOPTED_EXACT_EXISTING"
    );
    assert_eq!(fs::read(&output).unwrap(), bytes);
    assert_eq!(fs::metadata(&output).unwrap().mode() & 0o7777, 0o400);
    fixture.no_incidents();
    let mut records = vec![claim("delta", 1_000), claim("gamma", 1_000)];
    for id in ["delta", "gamma"] {
        records.push(
            json!({"kind":"handoff", "schema_version":1, "at_unix_ms":2_000,
            "claim_id":id, "agent":"worker", "proof_command":"just docs", "proof_exit_code":0,
            "changed_paths":["README.md"], "commit_oid":null}),
        );
    }
    let receipts = json!([{"claim_id":"delta", "committed_paths":["README.md"]},
        {"claim_id":"gamma", "committed_paths":["README.md"]}]);
    records.push(
        json!({"kind":"commit_receipt_group", "schema_version":1, "at_unix_ms":3_000,
        "orchestrator":"integrator", "commit_oid":"a".repeat(40), "receipts":receipts}),
    );
    records.push(json!({"kind":"commit_receipt_group_correction", "schema_version":1, "at_unix_ms":4_000,
        "orchestrator":"integrator", "previous_commit_oid":"a".repeat(40), "commit_oid":"b".repeat(40),
        "receipts":receipts, "reason":"fixture correction"}));
    let fixture = Fixture::with_ledgers(&framed(&records), &ledgers().1);
    fixture.run(&fixture.request, "groups").unwrap();
    assert_eq!(
        fixture.request.dispositions[0].disposition,
        Disposition::RecordedReceipt {
            commit_oid: "b".repeat(40)
        }
    );
    fixture.no_incidents();
}

#[test]
fn replay_refuses_missing_duplicate_stale_or_invented_dispositions() {
    let fixture = Fixture::new();
    for variant in [
        "missing",
        "duplicate",
        "order",
        "role",
        "hash",
        "retain-receipt",
        "invent-receipt",
        "wrong-receipt",
    ] {
        let mut request = fixture.request.clone();
        match variant {
            "missing" => {
                request.dispositions.pop();
            }
            "duplicate" => request.dispositions[2] = request.dispositions[0].clone(),
            "order" => request.dispositions.swap(0, 1),
            "role" => request.dispositions[0].location = Location::Hub,
            "hash" => request.dispositions[0].claim_subject_id = format!("frc_{}", "a".repeat(64)),
            "retain-receipt" => {
                request.dispositions[1].disposition = Disposition::RetainForRecovery
            }
            "invent-receipt" => {
                request.dispositions[0].disposition = Disposition::RecordedReceipt {
                    commit_oid: "a".repeat(40),
                }
            }
            "wrong-receipt" => {
                request.dispositions[1].disposition = Disposition::RecordedReceipt {
                    commit_oid: "b".repeat(40),
                }
            }
            _ => unreachable!(),
        }
        assert!(fixture.run(&request, variant).is_err(), "{variant}");
        assert!(
            !fixture
                .records
                .path()
                .join(format!("{variant}.out.json"))
                .exists()
        );
    }
    fixture.no_incidents();
}

#[test]
fn replay_refuses_corrupt_complete_ledgers_and_exact_reference_drift() {
    let (outer, hub) = ledgers();
    let mut schema = claim("alpha", 1_000);
    schema["schema_version"] = json!(2);
    for bytes in [
        outer[..outer.len() - 1].to_vec(),
        b"{\"kind\":\"claim\",\"kind\":\"handoff\"}\n".to_vec(),
        [outer.as_slice(), b"\n"].concat(),
        framed(&[schema]),
        framed(&[claim("alpha", 1_000), claim("alpha", 1_000)]),
    ] {
        let fixture = Fixture::with_ledgers(&bytes, &hub);
        let error = fixture.run(&fixture.request, "corrupt").unwrap_err();
        assert!(
            matches!(error.code(), "CORRUPT_COORD_LOG" | "UNSUPPORTED_SCHEMA"),
            "{error}"
        );
        assert_eq!(fs::read(&fixture.request.outer_ledger_copy).unwrap(), bytes);
        assert!(!fixture.records.path().join("corrupt.out.json").exists());
    }
    let fixture = Fixture::new();
    for variant in ["swap", "pair-id", "pair-hash", "pair-length", "schema"] {
        let mut request = fixture.request.clone();
        match variant {
            "swap" => std::mem::swap(&mut request.outer_ledger_copy, &mut request.hub_ledger_copy),
            "pair-id" => request.preservation.preservation_id = format!("fgp_{}", "b".repeat(64)),
            "pair-hash" => {
                request.preservation.sealed_sha256 = format!("sha256:{}", "b".repeat(64))
            }
            "pair-length" => request.preservation.byte_length += 1,
            "schema" => request.schema_version = 2,
            _ => unreachable!(),
        }
        assert!(fixture.run(&request, variant).is_err(), "{variant}");
    }
}

#[test]
fn replay_refuses_unsafe_paths_modes_aliases_and_unchecked_operator_fields() {
    for variant in [
        "inside",
        "relative",
        "duplicate",
        "writable",
        "hardlink",
        "symlink",
        "operator",
        "no-events",
    ] {
        let fixture = Fixture::new();
        let mut request = fixture.request.clone();
        match variant {
            "inside" => request.outer_ledger_copy = fixture.family.path().join("events.jsonl"),
            "relative" => request.outer_ledger_copy = PathBuf::from("events.jsonl"),
            "duplicate" => request.hub_ledger_copy = request.outer_ledger_copy.clone(),
            "writable" => fs::set_permissions(
                &request.outer_ledger_copy,
                fs::Permissions::from_mode(0o600),
            )
            .unwrap(),
            "hardlink" => fs::hard_link(
                &request.outer_ledger_copy,
                fixture.records.path().join("alias"),
            )
            .unwrap(),
            "symlink" => {
                let alias = fixture.records.path().join("alias");
                std::os::unix::fs::symlink(&request.outer_ledger_copy, &alias).unwrap();
                request.outer_ledger_copy = alias;
            }
            "no-events" => {
                let mut pair: FreshPreservationSubjectV1 =
                    sealed::read(&request.preservation.path).unwrap();
                let mut body = pair.outer_inventory.subject;
                body.nodes[0].relative_path_hex = hex(b"other.jsonl");
                pair.outer_inventory = IncidentInventoryV1::from_subject(body).unwrap();
                let pair = FreshPreservationSubjectV1::from_inventories(
                    pair.family_root_path_hex,
                    pair.outer_inventory,
                    pair.hub_inventory,
                )
                .unwrap();
                request.preservation.path = fixture.records.path().join("missing-events.json");
                sealed::write(&request.preservation.path, &pair).unwrap();
                let bytes = canonical_lf(&pair).unwrap();
                request.preservation.preservation_id = pair.preservation_id;
                request.preservation.sealed_sha256 = sha256(&bytes);
                request.preservation.byte_length = bytes.len() as u64;
            }
            "operator" => {
                let mut value = serde_json::to_value(&request).unwrap();
                value["operator_approved"] = json!(true);
                let path = fixture.records.path().join("operator.json");
                sealed::write(&path, &value).unwrap();
                assert!(
                    publish(
                        fixture.family.path(),
                        &path,
                        &fixture.records.path().join("output.json")
                    )
                    .is_err()
                );
                continue;
            }
            _ => unreachable!(),
        }
        assert!(fixture.run(&request, variant).is_err(), "{variant}");
        fixture.no_incidents();
    }
}

#[test]
fn replay_preserves_conflicting_output_and_retries_after_directory_sync_failure() {
    let fixture = Fixture::new();
    let path = fixture.records.path().join("request.json");
    let output = fixture.records.path().join("output.json");
    sealed::write(&path, &fixture.request).unwrap();
    assert!(
        sealed::with_directory_sync_failure(|| publish(fixture.family.path(), &path, &output))
            .is_err()
    );
    let bytes = fs::read(&output).unwrap();
    let before = fs::metadata(&output).unwrap();
    let restored = publish(fixture.family.path(), &path, &output).unwrap();
    assert_eq!(
        restored.outcome,
        FreshGenesisPublicationOutcome::AdoptedExactExisting
    );
    assert_eq!(fs::metadata(&output).unwrap().ino(), before.ino());
    assert_eq!(fs::read(&output).unwrap(), bytes);
    let mut changed = fixture.request.clone();
    changed.outer_ledger_copy = fixture.records.path().join("outer-copy.jsonl");
    fs::write(
        &changed.outer_ledger_copy,
        fs::read(&fixture.request.outer_ledger_copy).unwrap(),
    )
    .unwrap();
    fs::set_permissions(
        &changed.outer_ledger_copy,
        fs::Permissions::from_mode(0o400),
    )
    .unwrap();
    let changed_path = fixture.records.path().join("changed.json");
    sealed::write(&changed_path, &changed).unwrap();
    assert!(publish(fixture.family.path(), &changed_path, &output).is_err());
    assert_eq!(fs::read(&output).unwrap(), bytes);
}

#[test]
fn replay_enforces_copy_request_and_combined_output_bounds() {
    let (_, hub) = ledgers();
    let fixture = Fixture::with_ledgers(&vec![b'x'; MAX_BYTES as usize + 1], &hub);
    assert!(fixture.run(&fixture.request, "large-copy").is_err());
    let fixture = Fixture::new();
    let request = fixture.records.path().join("large-request.json");
    fs::write(&request, vec![b'x'; MAX_BYTES as usize + 2]).unwrap();
    fs::set_permissions(&request, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(
        publish(
            fixture.family.path(),
            &request,
            &fixture.records.path().join("output.json")
        )
        .is_err()
    );
    let prefix = "x".repeat(128);
    let claims = (0..1_024)
        .map(|i| claim(&format!("{prefix}-{i:04}"), 1_000))
        .collect::<Vec<_>>();
    let bytes = framed(&claims);
    assert!(bytes.len() as u64 <= MAX_BYTES);
    let fixture = Fixture::with_ledgers(&bytes, &bytes);
    assert!(canonical_lf(&fixture.request).is_ok());
    assert_eq!(fixture.request.dispositions.len(), MAX_DISPOSITIONS);
    assert!(fixture.run(&fixture.request, "large-output").is_err());
    assert!(
        !fixture
            .records
            .path()
            .join("large-output.out.json")
            .exists()
    );
}
