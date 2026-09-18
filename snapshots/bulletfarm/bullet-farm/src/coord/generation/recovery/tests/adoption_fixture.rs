use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
};

use sha2::Digest;
use tempfile::{TempDir, tempdir};

use super::super::{RecoveryInput, SourceExpectation, verifier};
use crate::coord::{
    generation::manifest::{
        CreateBodyInput, GenerationManifest, Sha256Digest, TrustedClaimOutcomeCounts,
        TrustedProjectionInventory, TrustedRecordKindCounts, create_body,
    },
    model::{ClaimState, FrozenClaimSubject, GroupReceipt, Record},
};

#[derive(Clone)]
pub(in crate::coord) struct LineRef {
    pub(in crate::coord) index: u64,
    pub(in crate::coord) start: u64,
    pub(in crate::coord) end: u64,
    pub(in crate::coord) sha256: String,
}

pub(in crate::coord) struct AdoptionRecoveryFixture {
    pub(in crate::coord) family: TempDir,
    pub(in crate::coord) input: RecoveryInput,
    pub(in crate::coord) manifest: GenerationManifest,
    pub(in crate::coord) trusted: Vec<u8>,
    pub(in crate::coord) frozen: Vec<u8>,
    pub(in crate::coord) parent_receipt: LineRef,
    pub(in crate::coord) claim_ids: [String; 2],
    pub(in crate::coord) claim_records: [LineRef; 2],
    pub(in crate::coord) handoff_records: [LineRef; 2],
    pub(in crate::coord) group_receipt: LineRef,
    pub(in crate::coord) frozen_claims: Vec<FrozenClaimSubject>,
}

pub(in crate::coord) fn fixture(parent_oid: &str, commit_oid: &str) -> AdoptionRecoveryFixture {
    let family = tempdir().unwrap();
    let coord = family.path().join(".bullet-family/coord");
    let external = family.path().join("recovery-input");
    fs::create_dir_all(&coord).unwrap();
    fs::create_dir(&external).unwrap();
    fs::set_permissions(&coord, fs::Permissions::from_mode(0o775)).unwrap();

    let parent_claim_id = claim_id('a');
    let claim_ids = [claim_id('b'), claim_id('c')];
    let trusted_records = vec![
        Record::Claim {
            schema_version: 1,
            at_unix_ms: 1,
            claim_id: parent_claim_id.clone(),
            agent: "parent-agent".to_owned(),
            lane: "trusted-parent".to_owned(),
            repo: "bullet-kernel".to_owned(),
            paths: vec!["baseline.txt".to_owned()],
            expires_unix_ms: 60_001,
        },
        Record::Handoff {
            schema_version: 1,
            at_unix_ms: 2,
            claim_id: parent_claim_id.clone(),
            agent: "parent-agent".to_owned(),
            proof_command: "trusted-parent-proof".to_owned(),
            proof_exit_code: 0,
            changed_paths: vec!["baseline.txt".to_owned()],
            commit_oid: None,
        },
        Record::CommitReceipt {
            schema_version: 1,
            at_unix_ms: 3,
            claim_id: parent_claim_id,
            orchestrator: "trusted-parent-orchestrator".to_owned(),
            commit_oid: parent_oid.to_owned(),
            committed_paths: vec!["baseline.txt".to_owned()],
        },
        Record::Claim {
            schema_version: 1,
            at_unix_ms: 4,
            claim_id: claim_ids[0].clone(),
            agent: "agent-b".to_owned(),
            lane: "recovery-group".to_owned(),
            repo: "bullet-kernel".to_owned(),
            paths: vec!["a.txt".to_owned()],
            expires_unix_ms: 60_004,
        },
        Record::Claim {
            schema_version: 1,
            at_unix_ms: 5,
            claim_id: claim_ids[1].clone(),
            agent: "agent-c".to_owned(),
            lane: "recovery-group".to_owned(),
            repo: "bullet-kernel".to_owned(),
            paths: vec!["b.txt".to_owned()],
            expires_unix_ms: 60_005,
        },
    ];
    let suffix_records = vec![
        Record::Handoff {
            schema_version: 1,
            at_unix_ms: 11,
            claim_id: claim_ids[0].clone(),
            agent: "agent-b".to_owned(),
            proof_command: "quarantined-proof-a".to_owned(),
            proof_exit_code: 0,
            changed_paths: vec!["a.txt".to_owned()],
            commit_oid: None,
        },
        Record::Handoff {
            schema_version: 1,
            at_unix_ms: 12,
            claim_id: claim_ids[1].clone(),
            agent: "agent-c".to_owned(),
            proof_command: "quarantined-proof-b".to_owned(),
            proof_exit_code: 0,
            changed_paths: vec!["b.txt".to_owned()],
            commit_oid: None,
        },
        Record::CommitReceiptGroup {
            schema_version: 1,
            at_unix_ms: 13,
            orchestrator: "quarantined-orchestrator".to_owned(),
            commit_oid: commit_oid.to_owned(),
            receipts: vec![
                GroupReceipt {
                    claim_id: claim_ids[0].clone(),
                    committed_paths: vec!["a.txt".to_owned()],
                },
                GroupReceipt {
                    claim_id: claim_ids[1].clone(),
                    committed_paths: vec!["b.txt".to_owned()],
                },
            ],
        },
    ];
    let (trusted, trusted_lines) = lines(&trusted_records);
    let (suffix, suffix_lines) = lines(&suffix_records);
    let frozen = [trusted.as_slice(), suffix.as_slice()].concat();
    let mut interrupted = trusted.clone();
    interrupted.extend_from_slice(b"{\"kind\":\"ambiguous");
    let mut tainted = interrupted.clone();
    tainted.push(b'\n');

    let interrupted_path = external.join("interrupted.partial");
    let tainted_path = external.join("tainted.jsonl");
    let legacy_path = coord.join("events.jsonl");
    private(&interrupted_path, &interrupted);
    private(&tainted_path, &tainted);
    private(&legacy_path, &frozen);
    let source_meta = fs::metadata(&legacy_path).unwrap();

    let summaries = crate::coord::state::summaries(&trusted_records, 10).unwrap();
    let trusted_state_blake3 = format!(
        "blake3:{}",
        bullet_wire::hash_canonical("bullet-family.coord.trusted-state.v2", &summaries)
            .unwrap()
            .to_hex()
    );
    let frozen_claims = claim_ids
        .iter()
        .map(|claim_id| {
            let claim = summaries.get(claim_id).unwrap();
            assert_eq!(claim.state, ClaimState::Active);
            FrozenClaimSubject {
                claim_id: claim_id.clone(),
                claim_blake3: format!(
                    "blake3:{}",
                    bullet_wire::hash_canonical("bullet-family.coord.frozen-claim.v2", claim)
                        .unwrap()
                        .to_hex()
                ),
            }
        })
        .collect::<Vec<_>>();
    let placeholder = manifest(
        &trusted,
        &interrupted,
        &tainted,
        &frozen,
        source_meta.dev(),
        source_meta.ino(),
        &trusted_state_blake3,
        &frozen_claims,
        format!("blake3:{}", "0".repeat(64)),
    );
    let inventory = verifier::compute_post_prefix_inventory(
        &mut fs::File::open(&interrupted_path).unwrap(),
        &mut fs::File::open(&tainted_path).unwrap(),
        &mut fs::File::open(&legacy_path).unwrap(),
        &placeholder,
    )
    .unwrap();
    let manifest = manifest(
        &trusted,
        &interrupted,
        &tainted,
        &frozen,
        source_meta.dev(),
        source_meta.ino(),
        &trusted_state_blake3,
        &frozen_claims,
        inventory,
    );
    AdoptionRecoveryFixture {
        input: RecoveryInput {
            coord_dir: coord,
            trusted_prefix: expectation(&trusted),
            interrupted_capture: SourceExpectation {
                path: interrupted_path,
                content: expectation(&interrupted),
            },
            tainted_generation: SourceExpectation {
                path: tainted_path,
                content: expectation(&tainted),
            },
            frozen_live_source: SourceExpectation {
                path: legacy_path,
                content: expectation(&frozen),
            },
        },
        family,
        manifest,
        parent_receipt: trusted_lines[2].clone(),
        claim_ids,
        claim_records: [trusted_lines[3].clone(), trusted_lines[4].clone()],
        handoff_records: [
            shifted(&suffix_lines[0], trusted.len() as u64, 5),
            shifted(&suffix_lines[1], trusted.len() as u64, 5),
        ],
        group_receipt: shifted(&suffix_lines[2], trusted.len() as u64, 5),
        trusted,
        frozen,
        frozen_claims,
    }
}

#[allow(clippy::too_many_arguments)]
fn manifest(
    trusted: &[u8],
    interrupted: &[u8],
    tainted: &[u8],
    frozen: &[u8],
    device: u64,
    inode: u64,
    trusted_state_blake3: &str,
    frozen_claims: &[FrozenClaimSubject],
    inventory: String,
) -> GenerationManifest {
    let artifacts = serde_json::from_value(serde_json::json!({
        "trusted_prefix": super::binding("archive/trusted-prefix.jsonl", trusted, 5, true),
        "interrupted_capture": super::binding(
            "archive/interrupted-observation.jsonl.partial", interrupted, 5, false
        ),
        "tainted_generation": super::binding("archive/tainted-generation.jsonl", tainted, 6, true),
        "frozen_live_source": super::binding("archive/frozen-live-source.jsonl", frozen, 8, true),
    }))
    .unwrap();
    GenerationManifest::from_body(
        create_body(CreateBodyInput {
            recovery_operator: "fixture-recovery-operator".to_owned(),
            recovery_policy_sha256: Sha256Digest::for_bytes(b"policy"),
            operator_decision_sha256: Sha256Digest::for_bytes(b"decision"),
            replay_contract_version: 1,
            replay_contract_sha256: Sha256Digest::for_bytes(b"replay"),
            bootstrap_commit_oid: "a".repeat(40),
            bootstrap_paths: vec!["src/coord".to_owned()],
            legacy_source_device: device,
            legacy_source_inode: inode,
            parent_generation: "legacy-v1".to_owned(),
            incident_at_unix_ms: 10,
            recovered_at_unix_ms: 20,
            trusted_record_count: 5,
            trusted_projection_inventory: TrustedProjectionInventory {
                record_kinds: TrustedRecordKindCounts {
                    claim: 3,
                    heartbeat: 0,
                    handoff: 1,
                    commit_receipt: 1,
                    commit_receipt_correction: 0,
                    commit_receipt_group: 0,
                    commit_receipt_group_correction: 0,
                },
                claim_outcomes: TrustedClaimOutcomeCounts {
                    total: 3,
                    active: 2,
                    expired: 0,
                    handed_off_unreceipted: 0,
                    receipted: 1,
                },
            },
            discarded_range: serde_json::from_value(serde_json::json!({
                "start_inclusive": trusted.len(), "end_exclusive": frozen.len()
            }))
            .unwrap(),
            ambiguous_tail_range: serde_json::from_value(serde_json::json!({
                "start_inclusive": trusted.len(), "end_exclusive": interrupted.len()
            }))
            .unwrap(),
            ambiguous_tail_sha256: Sha256Digest::for_bytes(&interrupted[trusted.len()..]),
            artifacts,
            trusted_state_blake3: trusted_state_blake3.to_owned(),
            frozen_claims: frozen_claims.to_vec(),
            post_prefix_inventory_blake3: inventory,
        })
        .unwrap(),
    )
    .unwrap()
}

fn lines(records: &[Record]) -> (Vec<u8>, Vec<LineRef>) {
    let mut bytes = Vec::new();
    let mut refs = Vec::new();
    for (index, record) in records.iter().enumerate() {
        let start = bytes.len() as u64;
        let mut line = bullet_wire::canonical_json(record).unwrap();
        line.push(b'\n');
        bytes.extend_from_slice(&line);
        refs.push(LineRef {
            index: index as u64 + 1,
            start,
            end: bytes.len() as u64,
            sha256: format!("sha256:{:x}", sha2::Sha256::digest(&line)),
        });
    }
    (bytes, refs)
}

fn shifted(reference: &LineRef, bytes: u64, records: u64) -> LineRef {
    LineRef {
        index: reference.index + records,
        start: reference.start + bytes,
        end: reference.end + bytes,
        sha256: reference.sha256.clone(),
    }
}

fn private(path: &std::path::Path, bytes: &[u8]) {
    fs::write(path, bytes).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o400)).unwrap();
}

fn expectation(bytes: &[u8]) -> super::super::ContentExpectation {
    super::super::ContentExpectation {
        byte_length: bytes.len() as u64,
        sha256: Sha256Digest::for_bytes(bytes),
    }
}

fn claim_id(marker: char) -> String {
    format!("clm_{}", marker.to_string().repeat(64))
}
