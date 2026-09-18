use std::{ffi::OsString, fs, os::unix::fs::PermissionsExt, path::Path};

use crate::coord::{CoordStore, GenesisInput};

use super::require_pristine_context;

fn genesis() -> GenesisInput {
    GenesisInput {
        operator: "test-operator".to_owned(),
        policy_sha256: format!("sha256:{}", "a".repeat(64)),
        replay_contract_version: 1,
        replay_contract_sha256: format!("sha256:{}", "b".repeat(64)),
        bootstrap_commit_oid: "c".repeat(40),
        bootstrap_paths: vec!["src".to_owned()],
    }
}

fn private(path: &Path) {
    fs::create_dir_all(path).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

#[test]
fn omitted_inputs_refuse_retained_material_at_either_location_without_writes() {
    for relative in [
        ".bullet-family/arbitrarily-renamed-incident/original",
        ".bullet-family/fresh-genesis/inventory.json",
        "bullet-farm/.bullet-family/coord/events.jsonl",
        "bullet-farm/.bullet-family/retained-any-name/bytes",
        ".bullet-family/coord/quarantine/tainted-events",
        ".bullet-family/coord/generations/retained-generation",
    ] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join(relative);
        private(path.parent().unwrap());
        fs::write(&path, b"retained incident bytes").unwrap();
        let store = CoordStore::with_clock(root.path().to_owned(), || panic!("clock used"));
        assert_eq!(
            store.initialize(&genesis()).unwrap_err().code(),
            "FRESH_GENESIS_ADMISSION_REQUIRED",
            "{relative}"
        );
        assert_eq!(fs::read(&path).unwrap(), b"retained incident bytes");
        assert!(!root.path().join(".bullet-family/coord/LOCK").exists());
        assert!(
            !root
                .path()
                .join(".bullet-family/coord/genesis-init-intent.json")
                .exists()
        );
    }
}

#[test]
fn pristine_guard_refuses_aliases_and_unbounded_or_unrecognized_topology() {
    for relative in [
        ".bullet-family",
        "bullet-farm",
        "bullet-farm/.bullet-family",
        ".bullet-family/coord",
    ] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join(relative);
        private(path.parent().unwrap());
        std::os::unix::fs::symlink(root.path().join("absent"), &path).unwrap();
        assert_eq!(
            require_pristine_context(root.path()).unwrap_err().code(),
            "FRESH_GENESIS_ADMISSION_REQUIRED",
            "{relative}"
        );
    }
    let root = tempfile::tempdir().unwrap();
    let coord = root.path().join(".bullet-family/coord");
    private(&coord);
    fs::write(coord.join("genesis-init-intent.json"), b"invalid").unwrap();
    fs::write(coord.join(".CURRENT.next-short"), b"retained").unwrap();
    assert!(require_pristine_context(root.path()).is_err());
    fs::remove_file(coord.join(".CURRENT.next-short")).unwrap();
    // Recognized names are only delegated; their contents still must pass the
    // existing exact intent/fence decoder before a generation may be created.
    assert!(
        CoordStore::new(root.path().to_owned())
            .initialize(&genesis())
            .is_err()
    );
    assert!(!coord.join("generations").exists());
    for index in 0..65 {
        fs::write(coord.join(format!("entry-{index}")), b"").unwrap();
    }
    assert!(require_pristine_context(root.path()).is_err());
}

#[test]
fn late_incident_insertion_under_initialization_lock_never_publishes_intent() {
    let root = tempfile::tempdir().unwrap();
    let hub = root.path().join("bullet-farm/.bullet-family/coord");
    let store = CoordStore::with_clock(root.path().to_owned(), move || {
        private(&hub);
        fs::write(hub.join("events.jsonl"), b"late incident").unwrap();
        Ok(1000)
    });
    assert_eq!(
        store.initialize(&genesis()).unwrap_err().code(),
        "FRESH_GENESIS_ADMISSION_REQUIRED"
    );
    let coord = root.path().join(".bullet-family/coord");
    assert!(!coord.join("genesis-init-intent.json").exists());
    assert!(!coord.join("generations").exists());
    assert!(!coord.join("CURRENT").exists());
}

#[test]
fn pristine_initialization_and_exact_restart_remain_available() {
    let root = tempfile::tempdir().unwrap();
    let store = CoordStore::with_clock(root.path().to_owned(), || Ok(1000));
    let initialized = store.initialize(&genesis()).unwrap();
    let reopened = CoordStore::with_clock(root.path().to_owned(), || panic!("retry clock used"));
    assert_eq!(reopened.initialize(&genesis()).unwrap(), initialized);
    fs::write(
        root.path().join(".bullet-family/retained-incident"),
        b"history",
    )
    .unwrap();
    assert_eq!(
        reopened.initialize(&genesis()).unwrap_err().code(),
        "FRESH_GENESIS_ADMISSION_REQUIRED"
    );
}

#[test]
fn exact_retry_refuses_forged_stages_and_foreign_generations() {
    resume_refuses_unconsumed_stages();
    for relative in [
        format!(".LOCK.next-{}", "a".repeat(64)),
        format!(".CURRENT.next-{}", "a".repeat(64)),
        format!(".genesis-init-intent.json.next-{}", "a".repeat(64)),
        format!("generations/gen_{}/retained", "a".repeat(64)),
    ] {
        let root = tempfile::tempdir().unwrap();
        let store = CoordStore::with_clock(root.path().to_owned(), || Ok(1000));
        store.initialize(&genesis()).unwrap();
        let retained = root.path().join(".bullet-family/coord").join(relative);
        private(retained.parent().unwrap());
        fs::write(&retained, b"unreferenced history").unwrap();
        assert_eq!(
            store.initialize(&genesis()).unwrap_err().code(),
            "FRESH_GENESIS_ADMISSION_REQUIRED"
        );
        assert_eq!(fs::read(&retained).unwrap(), b"unreferenced history");
    }
}

fn resume_refuses_unconsumed_stages() {
    for record in ["genesis-init-intent.json", "manifest.json"] {
        for variant in ["foreign-bytes", "symlink", "hardlink"] {
            let root = tempfile::tempdir().unwrap();
            let store = CoordStore::with_clock(root.path().to_owned(), || Ok(1000));
            let status = store.initialize(&genesis()).unwrap();
            let coord = root.path().join(".bullet-family/coord");
            fs::remove_file(coord.join("CURRENT")).unwrap();
            let directory = if record == "manifest.json" {
                coord.join("generations").join(status.generation_id)
            } else {
                coord.clone()
            };
            let canonical = directory.join(record);
            let expected = fs::read(&canonical).unwrap();
            let digest =
                bullet_wire::hash_framed_bytes("bullet.coord.staged-file.v2", &expected).unwrap();
            let stage = directory.join(format!(".{record}.next-{}", digest.to_hex()));
            match variant {
                "symlink" => std::os::unix::fs::symlink(&canonical, &stage).unwrap(),
                "hardlink" => fs::hard_link(&canonical, &stage).unwrap(),
                _ => fs::write(&stage, b"foreign retained bytes").unwrap(),
            }
            assert_eq!(
                store.initialize(&genesis()).unwrap_err().code(),
                "FRESH_GENESIS_ADMISSION_REQUIRED",
                "{record} {variant}"
            );
            assert!(fs::symlink_metadata(&stage).is_ok());
            assert!(!coord.join("CURRENT").exists());
        }
    }
}

#[test]
fn structurally_valid_v1_pair_cannot_publish_sidecars_or_genesis() {
    use crate::coord::{fresh_genesis::incident::observe_incident_inventory, model::*};
    let root = tempfile::tempdir().unwrap();
    let inputs = tempfile::tempdir().unwrap();
    let incident = inputs.path().join("incident");
    private(&incident);
    fs::write(incident.join("events.jsonl"), b"retained\n").unwrap();
    let inventory = observe_incident_inventory(&incident, std::ffi::OsStr::new("retired")).unwrap();
    let hex_path = |value: &Path| {
        value
            .as_os_str()
            .as_encoded_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    let sha = format!("sha256:{}", "a".repeat(64));
    let facts = Wave0FactsV1 {
        producer_principal: "producer".to_owned(),
        claim_high_water: Wave0ClaimHighWaterV1 {
            claim_ledger_path_hex: hex_path(&incident.join("events.jsonl")),
            claim_ledger_sha256: sha.clone(),
            claim_projection_blake3: format!("blake3:{}", "b".repeat(64)),
            byte_length: 0,
            entry_count: 0,
            active_claim_count: 0,
        },
        members: [
            (Wave0MemberRoleV1::Hub, "root/bullet-farm"),
            (Wave0MemberRoleV1::Kernel, "root/bullet-kernel"),
            (Wave0MemberRoleV1::BulletGit, "root/bullet-git"),
            (Wave0MemberRoleV1::Portal, "root/bullet-portal"),
        ]
        .into_iter()
        .map(|(role, identity)| Wave0MemberV1 {
            role,
            repository_identity: identity.to_owned(),
            commit_oid: format!("sha1:{}", "a".repeat(40)),
            tree_oid: format!("sha1:{}", "b".repeat(40)),
            index_state: Wave0CleanStateV1::Clean,
            worktree_state: Wave0CleanStateV1::Clean,
            untracked_state: Wave0CleanStateV1::Clean,
        })
        .collect(),
    };
    let wave0 = Wave0SubjectV1::from_reviewed(
        facts,
        "reviewer".to_owned(),
        hex_path(&inputs.path().join("review")),
        sha,
        1,
    )
    .unwrap();
    let inventory_path = inputs.path().join("inventory.json");
    let wave0_path = inputs.path().join("wave0.json");
    fs::write(
        &inventory_path,
        bullet_wire::canonical_json(&inventory).unwrap(),
    )
    .unwrap();
    fs::write(&wave0_path, bullet_wire::canonical_json(&wave0).unwrap()).unwrap();
    fs::write(
        root.path().join("repos.manifest.toml"),
        "schema_version = \"1.2.0\"\n",
    )
    .unwrap();
    let argv: Vec<OsString> = vec![
        "bullet-family".into(),
        "--root".into(),
        root.path().as_os_str().to_owned(),
        "coord".into(),
        "init".into(),
        "--incident-inventory".into(),
        inventory_path.into(),
        "--wave0-subject".into(),
        wave0_path.into(),
    ];
    let error = crate::cli::run(argv, Ok(root.path().to_owned())).unwrap_err();
    assert_eq!(error.code(), "FRESH_GENESIS_ADMISSION_UNAVAILABLE");
    assert_eq!(
        error.repair_hint(),
        "retain Operating HOLD until ADR 0015 admission is reviewed"
    );
    assert!(!root.path().join(".bullet-family").exists());
}
