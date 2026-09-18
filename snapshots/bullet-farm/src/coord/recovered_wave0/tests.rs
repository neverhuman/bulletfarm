use std::{collections::VecDeque, path::Path};

use super::{
    super::{
        COORD_SCHEMA_VERSION, ClaimState, ClaimSummary, CoordError, Status, StatusOrigin,
        git::wave0::Wave0MechanicalObservation,
        model::{Wave0CleanStateV1, Wave0MemberRoleV1, Wave0MemberV1},
    },
    RecoveredWave0FactsV1, observe_recovered_wave0_with,
};

const ROOT: &str = "/tmp/bullet-recovered-w0-fixture";

fn digest(prefix: &str, byte: char) -> String {
    format!("{prefix}{}", byte.to_string().repeat(64))
}

fn member(role: Wave0MemberRoleV1, identity: &str, byte: char) -> Wave0MemberV1 {
    Wave0MemberV1 {
        role,
        repository_identity: identity.to_owned(),
        commit_oid: format!("sha1:{}", byte.to_string().repeat(40)),
        tree_oid: format!("sha1:{}", ((byte as u8 + 1) as char).to_string().repeat(40)),
        index_state: Wave0CleanStateV1::Clean,
        worktree_state: Wave0CleanStateV1::Clean,
        untracked_state: Wave0CleanStateV1::Clean,
    }
}

fn mechanical() -> Wave0MechanicalObservation {
    Wave0MechanicalObservation {
        members: [
            member(Wave0MemberRoleV1::Hub, "root/bullet-farm", 'a'),
            member(Wave0MemberRoleV1::Kernel, "root/bullet-kernel", 'b'),
            member(Wave0MemberRoleV1::BulletGit, "root/bullet-git", 'c'),
            member(Wave0MemberRoleV1::Portal, "root/bullet-portal", 'd'),
        ],
        collaboration_log_path_hex: "2f746d702f4147454e545f434841542e6d64".to_owned(),
        collaboration_log_sha256: digest("sha256:", 'e'),
        collaboration_log_byte_length: 1,
    }
}

fn source(generation: &str) -> String {
    Path::new(ROOT)
        .join(".bullet-family/coord/generations")
        .join(generation)
        .join("events.jsonl")
        .to_string_lossy()
        .into_owned()
}

fn status() -> Status {
    let generation_id = digest("gen_", 'a');
    Status {
        schema_version: COORD_SCHEMA_VERSION,
        generation_id: generation_id.clone(),
        manifest_blake3: digest("blake3:", 'b'),
        origin: StatusOrigin::Recovered {
            incident_at_unix_ms: 10,
            recovered_at_unix_ms: 20,
            trusted_records: 3,
        },
        as_of_sequence: 2,
        next_sequence: 3,
        last_request_id: digest("req_", 'c'),
        last_request_blake3: digest("", 'd'),
        last_record_blake3: digest("", 'e'),
        last_envelope_blake3: digest("", 'f'),
        byte_length: 128,
        observed_at_unix_ms: 30,
        source: source(&generation_id),
        claims: Vec::new(),
    }
}

fn claim(id: char, state: ClaimState, committed: bool) -> ClaimSummary {
    ClaimSummary {
        claim_id: digest("clm_", id),
        agent: format!("agent-{id}"),
        lane: "fixture-lane".to_owned(),
        repo: "bullet-farm".to_owned(),
        paths: vec![format!("fixture/{id}")],
        claimed_at_unix_ms: 1,
        last_event_unix_ms: 2,
        expires_unix_ms: 3,
        state,
        proof_command: None,
        changed_paths: Vec::new(),
        commit_oid: committed.then(|| "a".repeat(40)),
        commit_orchestrator: committed.then(|| "orchestrator".to_owned()),
        commit_recorded_at_unix_ms: committed.then_some(4),
        recovery_adoption: None,
    }
}

fn observe(first: Status, second: Status) -> Result<RecoveredWave0FactsV1, CoordError> {
    let mut statuses = VecDeque::from([first, second]);
    observe_recovered_wave0_with(
        Path::new(ROOT),
        || {
            statuses
                .pop_front()
                .ok_or_else(|| CoordError::new("TEST", "missing status"))
        },
        || Ok(mechanical()),
    )
}

fn valid() -> RecoveredWave0FactsV1 {
    let first = status();
    let mut second = first.clone();
    second.observed_at_unix_ms += 1;
    observe(first, second).unwrap()
}

#[test]
fn binds_recovery_watermark_claim_projection_and_four_members() {
    let value = valid();
    value.validate().unwrap();
    assert!(value.facts_blake3.starts_with("blake3:"));
    assert_eq!(value.coord_schema_version, COORD_SCHEMA_VERSION);
    assert_eq!(value.claim_projection.total, 0);
    assert_eq!(value.members.len(), 4);
}

#[test]
fn genesis_is_never_a_recovered_w0_subject() {
    let mut genesis = status();
    genesis.origin = StatusOrigin::Genesis;
    assert_eq!(
        observe(genesis.clone(), genesis).unwrap_err().code(),
        "W0_RECOVERY_REQUIRED"
    );
}

#[test]
fn every_open_claim_partition_refuses() {
    for (state, committed) in [
        (ClaimState::Active, false),
        (ClaimState::HandedOff, false),
        (ClaimState::FrozenRecovery, false),
    ] {
        let mut open = status();
        open.claims = vec![claim('a', state, committed)];
        assert_eq!(
            observe(open.clone(), open).unwrap_err().code(),
            "W0_CLAIMS_OPEN"
        );
    }

    let mut partial_receipt = status();
    let mut handed = claim('a', ClaimState::HandedOff, true);
    handed.commit_orchestrator = None;
    partial_receipt.claims = vec![handed];
    assert_eq!(
        observe(partial_receipt.clone(), partial_receipt)
            .unwrap_err()
            .code(),
        "W0_CLAIMS_OPEN"
    );
}

#[test]
fn expired_and_receipted_claim_partitions_are_allowed_and_sorted() {
    let mut first = status();
    first.claims = vec![
        claim('c', ClaimState::RecoveredReceipted, true),
        claim('a', ClaimState::Expired, false),
        claim('b', ClaimState::HandedOff, true),
    ];
    let mut second = first.clone();
    second.claims.reverse();
    second.observed_at_unix_ms += 1;
    let value = observe(first, second).unwrap();
    assert_eq!(value.claim_projection.total, 3);
    assert_eq!(value.claim_projection.expired, 1);
    assert_eq!(value.claim_projection.handed_off_receipted, 1);
    assert_eq!(value.claim_projection.recovered_receipted, 1);
}

#[test]
fn duplicate_claim_identity_refuses() {
    let mut duplicate = status();
    duplicate.claims = vec![
        claim('a', ClaimState::Expired, false),
        claim('a', ClaimState::RecoveredReceipted, true),
    ];
    assert_eq!(
        observe(duplicate.clone(), duplicate).unwrap_err().code(),
        "INVALID_RECOVERED_WAVE0_FACTS"
    );
}

#[test]
fn every_authority_watermark_change_refuses_the_bracket() {
    let edits: [fn(&mut Status); 8] = [
        |value| {
            value.generation_id = digest("gen_", '9');
            value.source = source(&value.generation_id);
        },
        |value| value.manifest_blake3 = digest("blake3:", '9'),
        |value| {
            value.as_of_sequence = 3;
            value.next_sequence = 4;
        },
        |value| value.last_envelope_blake3 = digest("", '9'),
        |value| value.last_record_blake3 = digest("", '9'),
        |value| value.last_request_id = digest("req_", '9'),
        |value| value.last_request_blake3 = digest("", '9'),
        |value| value.byte_length += 1,
    ];
    for edit in edits {
        let first = status();
        let mut second = first.clone();
        second.observed_at_unix_ms += 1;
        edit(&mut second);
        assert_eq!(
            observe(first, second).unwrap_err().code(),
            "W0_SUBJECT_CHANGED"
        );
    }
}

#[test]
fn claim_content_change_refuses_the_bracket() {
    let mut first = status();
    first.claims = vec![claim('a', ClaimState::Expired, false)];
    let mut second = first.clone();
    second.claims[0].agent = "changed-agent".to_owned();
    second.observed_at_unix_ms += 1;
    assert_eq!(
        observe(first, second).unwrap_err().code(),
        "W0_SUBJECT_CHANGED"
    );
}

#[test]
fn clock_rollback_and_wrong_source_refuse() {
    let first = status();
    let mut earlier = first.clone();
    earlier.observed_at_unix_ms -= 1;
    assert_eq!(
        observe(first.clone(), earlier).unwrap_err().code(),
        "W0_SUBJECT_CHANGED"
    );

    let mut wrong = first;
    wrong.source = "/tmp/not-the-ledger".to_owned();
    assert_eq!(
        observe(wrong.clone(), wrong).unwrap_err().code(),
        "W0_SUBJECT_CHANGED"
    );
}

#[test]
fn member_order_identity_digest_and_cleanliness_are_closed() {
    let base = valid();
    let mut cases = Vec::new();
    let mut order = base.clone();
    order.members.swap(0, 1);
    cases.push(order);
    let mut identity = base.clone();
    identity.members[0].repository_identity = "root/other".to_owned();
    cases.push(identity);
    let mut digest_value = base.clone();
    digest_value.members[0].commit_oid = format!("sha1:{}", "A".repeat(40));
    cases.push(digest_value);
    let mut dirty = base;
    dirty.members[0].index_state = Wave0CleanStateV1::Dirty;
    cases.push(dirty);
    for value in cases {
        assert_eq!(
            value.validate().unwrap_err().code(),
            "INVALID_RECOVERED_WAVE0_FACTS"
        );
    }
}

#[test]
fn canonical_hostiles_and_identity_substitution_refuse() {
    let value = valid();
    let json = String::from_utf8(bullet_wire::canonical_json(&value).unwrap()).unwrap();
    let unknown = json.replacen('{', "{\"a_unknown\":true,", 1);
    let duplicate = json.replacen(
        "\"schema_version\":1",
        "\"schema_version\":1,\"schema_version\":1",
        1,
    );
    let unsafe_integer =
        json.replacen("\"byte_length\":128", "\"byte_length\":9007199254740992", 1);
    let nested_unknown = json.replacen(
        "\"claim_projection_blake3\":",
        "\"b_unknown\":true,\"claim_projection_blake3\":",
        1,
    );
    for hostile in [unknown, nested_unknown] {
        assert_eq!(
            bullet_wire::decode_canonical::<RecoveredWave0FactsV1>(hostile.as_bytes())
                .unwrap_err()
                .code(),
            "DOCUMENT_SCHEMA_INVALID"
        );
    }
    for hostile in [duplicate, unsafe_integer] {
        assert!(
            bullet_wire::decode_canonical::<RecoveredWave0FactsV1>(hostile.as_bytes()).is_err()
        );
    }

    let mut substituted = value;
    substituted.facts_blake3 = digest("blake3:", '9');
    assert_eq!(
        substituted.validate().unwrap_err().code(),
        "INVALID_RECOVERED_WAVE0_FACTS"
    );

    let mut nested_watermark = valid();
    nested_watermark.watermark.generation_id = "not-a-generation".to_owned();
    assert_eq!(
        nested_watermark.validate().unwrap_err().code(),
        "INVALID_RECOVERED_WAVE0_FACTS"
    );
}
