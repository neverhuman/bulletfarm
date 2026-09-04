use std::fs::{File, OpenOptions};

use super::*;

const FENCE_TRANSITION_PHASES: [&str; 10] = [
    "creation_plan_published",
    "sibling_created",
    "creation_observation_published",
    "fence_intent_published",
    "fence_sealed",
    "seal_observation_published",
    "publication_plan_published",
    "fence_renamed",
    "fence_parent_synced",
    "publication_observation_published",
];

type ShallowNode = (PathBuf, u8, u32, u64, u64, u64, Vec<u8>);

fn shallow_snapshot(directory: &Path) -> Vec<ShallowNode> {
    use std::os::unix::fs::MetadataExt;

    let mut nodes = fs::read_dir(directory)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            let metadata = fs::symlink_metadata(&path).unwrap();
            let kind = if metadata.is_dir() {
                1
            } else if metadata.is_file() {
                2
            } else {
                3
            };
            (
                PathBuf::from(path.file_name().unwrap()),
                kind as u8,
                metadata.mode() & 0o7777,
                metadata.ino(),
                metadata.nlink(),
                metadata.len(),
                if metadata.is_file() {
                    fs::read(&path).unwrap()
                } else {
                    Vec::new()
                },
            )
        })
        .collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.0.cmp(&right.0));
    nodes
}

#[test]
fn sealed_fence_without_observation_is_unknown_before_generation_creation() {
    let root = tempfile::tempdir().unwrap();
    let coord = coord_path(root.path());
    super::super::fs::ensure_layout(root.path(), &coord).unwrap();
    let tombstone = coord.join("events.jsonl");
    fs::create_dir(&tombstone).unwrap();
    set_mode(&tombstone, 0);

    let error = ledger(root.path())
        .initialize_genesis(&provenance(), || Ok(30))
        .unwrap_err();
    assert_eq!(error.code(), "COORD_FENCE_UNKNOWN");
    assert!(!coord.join("CURRENT").exists());
    assert!(!coord.join("generations").exists());
    assert_eq!(mode(&tombstone), 0);
}

#[test]
fn missing_publication_observation_reconstructs_from_exact_predecessors() {
    use std::os::unix::fs::MetadataExt;

    let root = tempfile::tempdir().unwrap();
    let ledger = ledger(root.path());
    let view = initialize(&ledger);
    let coord = coord_path(root.path());
    let authority = coord.join("events.jsonl");
    let before = fs::symlink_metadata(&authority).unwrap();
    remove_current(root.path());
    fs::remove_file(coord.join("genesis-fence-observation.json")).unwrap();

    let resumed = ledger
        .initialize_genesis(&provenance(), || {
            panic!("clock invoked during exact reconciliation")
        })
        .unwrap();
    let after = fs::symlink_metadata(&authority).unwrap();
    assert_eq!((after.dev(), after.ino()), (before.dev(), before.ino()));
    assert_eq!(resumed.watermark, view.watermark);
    assert!(coord.join("genesis-fence-observation.json").exists());
}

#[test]
fn missing_partial_or_tampered_fence_predecessors_are_unknown_without_mutation() {
    for variant in [
        "missing-create-plan",
        "missing-create-observation",
        "missing-fence-intent",
        "missing-seal-observation",
        "missing-publication-plan",
        "partial-observation",
        "initialization-intent",
        "create-plan",
        "create-observation",
        "fence-intent",
        "seal-observation",
        "publication-plan",
        "observation",
    ] {
        let root = tempfile::tempdir().unwrap();
        let ledger = ledger(root.path());
        let view = initialize(&ledger);
        let segment = view.source;
        let segment_length = fs::metadata(&segment).unwrap().len();
        let coord = coord_path(root.path());
        remove_current(root.path());
        let intent = coord.join("genesis-init-intent.json");
        let create_plan = coord.join("genesis-fence-create-plan.json");
        let create_observation = coord.join("genesis-fence-create-observation.json");
        let fence_intent = coord.join("genesis-fence-publish-intent.json");
        let seal_observation = coord.join("genesis-fence-seal-observation.json");
        let publication_plan = coord.join("genesis-fence-publication-plan.json");
        let observation = coord.join("genesis-fence-observation.json");
        match variant {
            "missing-create-plan" => fs::remove_file(&create_plan).unwrap(),
            "missing-create-observation" => fs::remove_file(&create_observation).unwrap(),
            "missing-fence-intent" => fs::remove_file(&fence_intent).unwrap(),
            "missing-seal-observation" => fs::remove_file(&seal_observation).unwrap(),
            "missing-publication-plan" => fs::remove_file(&publication_plan).unwrap(),
            "partial-observation" => {
                set_mode(&observation, 0o600);
                fs::write(&observation, b"{").unwrap();
                set_mode(&observation, 0o400);
            }
            "initialization-intent" => tamper_canonical_file(&intent),
            "create-plan" => tamper_canonical_file(&create_plan),
            "create-observation" => tamper_canonical_file(&create_observation),
            "fence-intent" => tamper_canonical_file(&fence_intent),
            "seal-observation" => tamper_canonical_file(&seal_observation),
            "publication-plan" => tamper_canonical_file(&publication_plan),
            "observation" => tamper_canonical_file(&observation),
            _ => unreachable!(),
        }
        let before = shallow_snapshot(&coord);

        let error = ledger
            .initialize_genesis(&provenance(), || Ok(31))
            .unwrap_err();
        assert_eq!(error.code(), "COORD_FENCE_UNKNOWN", "variant={variant}");
        assert!(!coord.join("CURRENT").exists());
        assert_eq!(fs::metadata(&segment).unwrap().len(), segment_length);
        assert_eq!(mode(&coord.join("events.jsonl")), 0);
        assert_eq!(shallow_snapshot(&coord), before, "variant={variant}");
    }
}

#[test]
fn exact_fence_observation_resumes_without_recreating_or_unsealing() {
    use std::os::unix::fs::MetadataExt;

    let root = tempfile::tempdir().unwrap();
    let ledger = ledger(root.path());
    let first = initialize(&ledger);
    let coord = coord_path(root.path());
    let tombstone = coord.join("events.jsonl");
    let before = fs::symlink_metadata(&tombstone).unwrap();
    let segment_length = fs::metadata(&first.source).unwrap().len();
    remove_current(root.path());

    let resumed = ledger
        .initialize_genesis(&provenance(), || panic!("clock invoked on retry"))
        .unwrap();
    let after = fs::symlink_metadata(&tombstone).unwrap();
    assert_eq!((after.dev(), after.ino()), (before.dev(), before.ino()));
    assert_eq!(mode(&tombstone), 0);
    assert_eq!(fs::metadata(&resumed.source).unwrap().len(), segment_length);
    assert!(coord.join("CURRENT").exists());
    let append_error = OpenOptions::new()
        .append(true)
        .open(&tombstone)
        .unwrap_err();
    let create_error = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&tombstone)
        .unwrap_err();
    assert_eq!(append_error.raw_os_error(), Some(21));
    assert_eq!(create_error.raw_os_error(), Some(21));
}

#[test]
fn retained_directory_inventory_survives_mode_zero_while_reopen_fails() {
    use std::os::unix::fs::MetadataExt;

    let root = tempfile::tempdir().unwrap();
    let directory = root.path().join("fence");
    fs::create_dir(&directory).unwrap();
    set_mode(&directory, 0o700);
    let retained = File::open(&directory).unwrap();
    let before = retained.metadata().unwrap();
    set_mode(&directory, 0);

    super::super::fs::test_inventory_retained_directory(&retained).unwrap();
    let after = retained.metadata().unwrap();
    assert_eq!((after.dev(), after.ino()), (before.dev(), before.ino()));
    assert!(rustix::fs::Dir::read_from(&retained).is_err());
}

#[test]
fn retained_directory_inventory_rewinds_and_detects_insertions_after_mode_zero() {
    let root = tempfile::tempdir().unwrap();
    let directory = root.path().join("fence");
    fs::create_dir(&directory).unwrap();
    set_mode(&directory, 0o700);
    fs::write(directory.join("intruder"), b"x").unwrap();
    let retained = File::open(&directory).unwrap();

    assert!(super::super::fs::test_inventory_retained_directory(&retained).is_err());
    set_mode(&directory, 0);
    assert!(super::super::fs::test_inventory_retained_directory(&retained).is_err());
}

#[test]
fn published_initialization_intent_resumes_without_reinvoking_clock() {
    let root = tempfile::tempdir().unwrap();
    let coord = coord_path(root.path());
    super::super::fs::ensure_layout(root.path(), &coord).unwrap();
    let prepared = super::super::genesis::prepare(&provenance(), 46).unwrap();
    let lock = super::super::fs::CoordLock::acquire(&coord, true).unwrap();
    super::super::fs::publish_genesis_intent(&lock, &prepared.intent_bytes).unwrap();
    drop(lock);

    let view = ledger(root.path())
        .initialize_genesis(&provenance(), || {
            panic!("clock invoked after durable intent")
        })
        .unwrap();
    assert_eq!(
        view.watermark.generation_id,
        prepared.manifest.generation_id().as_str()
    );
}

#[test]
fn genesis_fence_crash_boundaries_all_resume_exactly() {
    for phase in FENCE_TRANSITION_PHASES {
        let root = tempfile::tempdir().unwrap();
        let ledger = ledger(root.path());
        super::super::fs::test_crash_genesis_fence_after(phase);
        let error = ledger
            .initialize_genesis(&provenance(), || Ok(47))
            .unwrap_err();
        assert_eq!(error.code(), "COORD_TEST_CRASH", "phase={phase}");
        assert!(!coord_path(root.path()).join("CURRENT").exists());

        let resumed = ledger
            .initialize_genesis(&provenance(), || {
                panic!("clock invoked after durable initialization intent")
            })
            .unwrap_or_else(|error| panic!("phase={phase}: {error:?}"));
        assert_eq!(resumed.watermark.last_sequence, 1, "phase={phase}");
        assert_eq!(ledger.status().unwrap().watermark, resumed.watermark);
    }
}

#[test]
fn genesis_fence_sigkill_boundaries_all_resume_exactly() {
    use std::{os::unix::process::ExitStatusExt, process::Command};

    const CHILD_ROOT: &str = "BULLET_GENESIS_SIGKILL_ROOT";
    const CHILD_PHASE: &str = "BULLET_GENESIS_SIGKILL_PHASE";
    const TEST_NAME: &str =
        "coord::store::ledger::tests::fence::genesis_fence_sigkill_boundaries_all_resume_exactly";

    if let (Some(root), Some(phase)) = (std::env::var_os(CHILD_ROOT), std::env::var_os(CHILD_PHASE))
    {
        let phase = phase.to_str().unwrap();
        let phase = FENCE_TRANSITION_PHASES
            .iter()
            .copied()
            .find(|candidate| *candidate == phase)
            .expect("known Genesis transition phase");
        super::super::fs::test_kill_genesis_fence_after(phase);
        let _ = ledger(Path::new(&root)).initialize_genesis(&provenance(), || Ok(51));
        panic!("child survived the requested SIGKILL checkpoint");
    }

    for phase in FENCE_TRANSITION_PHASES {
        let root = tempfile::tempdir().unwrap();
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", TEST_NAME, "--nocapture"])
            .env(CHILD_ROOT, root.path())
            .env(CHILD_PHASE, phase)
            .output()
            .unwrap();
        assert_eq!(output.status.signal(), Some(9), "phase={phase}");

        let resumed = ledger(root.path())
            .initialize_genesis(&provenance(), || {
                panic!("clock invoked after durable initialization intent")
            })
            .unwrap_or_else(|error| panic!("phase={phase}: {error:?}"));
        assert_eq!(resumed.watermark.last_sequence, 1, "phase={phase}");
        assert_eq!(
            ledger(root.path()).status().unwrap().watermark,
            resumed.watermark
        );
    }
}

#[test]
fn unsealed_fence_insertion_and_substitution_fail_closed() {
    use std::os::unix::fs::MetadataExt;

    for variant in ["insert", "substitute"] {
        let root = tempfile::tempdir().unwrap();
        let ledger = ledger(root.path());
        super::super::fs::test_crash_genesis_fence_after("fence_intent_published");
        assert_eq!(
            ledger
                .initialize_genesis(&provenance(), || Ok(48))
                .unwrap_err()
                .code(),
            "COORD_TEST_CRASH"
        );
        let coord = coord_path(root.path());
        let prepared = super::super::genesis::prepare(&provenance(), 48).unwrap();
        let sibling = coord.join(format!(
            ".events.jsonl.genesis-next-{}",
            prepared.manifest.generation_id().as_str()
        ));
        let before = fs::symlink_metadata(&sibling).unwrap();
        if variant == "insert" {
            fs::write(sibling.join("intruder"), b"x").unwrap();
        } else {
            let displaced = coord.join("displaced-sibling");
            fs::rename(&sibling, &displaced).unwrap();
            fs::create_dir(&sibling).unwrap();
            set_mode(&sibling, 0o700);
            let after = fs::symlink_metadata(&sibling).unwrap();
            assert_ne!((after.dev(), after.ino()), (before.dev(), before.ino()));
        }

        let error = ledger
            .initialize_genesis(&provenance(), || panic!("clock invoked on retry"))
            .unwrap_err();
        assert!(
            matches!(
                error.code(),
                "INVALID_COORD_STORAGE" | "COORD_SUBJECT_CHANGED" | "COORD_FENCE_UNKNOWN"
            ),
            "variant={variant}: {error:?}"
        );
        assert!(!coord.join("CURRENT").exists());
        assert!(!coord.join("events.jsonl").exists());
    }
}

#[test]
fn insertions_between_seal_inventory_evidence_and_publication_are_rejected() {
    for phase in ["fence_sealed", "seal_observation_published"] {
        let root = tempfile::tempdir().unwrap();
        let ledger = ledger(root.path());
        super::super::fs::test_insert_genesis_fence_after(phase);
        let error = ledger
            .initialize_genesis(&provenance(), || Ok(49))
            .unwrap_err();
        assert_eq!(error.code(), "INVALID_COORD_STORAGE", "phase={phase}");
        let coord = coord_path(root.path());
        assert!(!coord.join("CURRENT").exists());
        assert!(!coord.join("events.jsonl").exists());
    }
}

#[test]
fn published_fence_rejects_siblings_stages_insertion_and_substitution() {
    for variant in ["sibling", "stage", "insert", "remove-insert", "substitute"] {
        let root = tempfile::tempdir().unwrap();
        let ledger = ledger(root.path());
        let initialized = initialize(&ledger);
        let coord = coord_path(root.path());
        let authority = coord.join("events.jsonl");
        let source_length = fs::metadata(&initialized.source).unwrap().len();
        match variant {
            "sibling" => {
                let path = coord.join(format!(".events.jsonl.genesis-next-gen_{}", "f".repeat(64)));
                fs::create_dir(path).unwrap();
            }
            "stage" => {
                fs::write(
                    coord.join(".genesis-fence-observation.json.next-dead"),
                    b"x",
                )
                .unwrap();
            }
            "insert" | "remove-insert" => {
                set_mode(&authority, 0o700);
                fs::write(authority.join("intruder"), b"x").unwrap();
                if variant == "remove-insert" {
                    fs::remove_file(authority.join("intruder")).unwrap();
                }
                set_mode(&authority, 0);
            }
            "substitute" => {
                set_mode(&authority, 0o700);
                fs::remove_dir(&authority).unwrap();
                fs::create_dir(&authority).unwrap();
                set_mode(&authority, 0);
            }
            _ => unreachable!(),
        }

        let error = ledger.status().unwrap_err();
        assert!(
            matches!(
                error.code(),
                "COORD_FENCE_UNKNOWN" | "COORD_SUBJECT_CHANGED" | "INVALID_COORD_STORAGE"
            ),
            "variant={variant}: {error:?}"
        );
        assert_eq!(
            fs::metadata(&initialized.source).unwrap().len(),
            source_length
        );
    }
}

#[test]
fn anonymous_link_sigkill_recovers_exact_genesis_and_current() {
    use std::{os::unix::process::ExitStatusExt, process::Command};

    const CHILD_ROOT: &str = "BULLET_LEDGER_SIGKILL_ROOT";
    const CHILD_TARGET: &str = "BULLET_LEDGER_SIGKILL_TARGET";
    const TEST_NAME: &str = "coord::store::ledger::tests::fence::anonymous_link_sigkill_recovers_exact_genesis_and_current";

    if let (Some(root), Some(target)) =
        (std::env::var_os(CHILD_ROOT), std::env::var_os(CHILD_TARGET))
    {
        let target = target.to_str().unwrap();
        let target = match target {
            "genesis-init-intent.json" => "genesis-init-intent.json",
            "CURRENT" => "CURRENT",
            _ => panic!("unexpected kill target"),
        };
        super::super::fs::test_kill_publish_after_link(target);
        let _ = ledger(Path::new(&root)).initialize_genesis(&provenance(), || Ok(50));
        panic!("child survived the requested SIGKILL checkpoint");
    }

    for target in ["genesis-init-intent.json", "CURRENT"] {
        let root = tempfile::tempdir().unwrap();
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", TEST_NAME, "--nocapture"])
            .env(CHILD_ROOT, root.path())
            .env(CHILD_TARGET, target)
            .output()
            .unwrap();
        assert_eq!(output.status.signal(), Some(9), "target={target}");

        let view = ledger(root.path())
            .initialize_genesis(&provenance(), || {
                panic!("clock invoked after exact linked Genesis intent")
            })
            .unwrap();
        assert_eq!(view.watermark.last_sequence, 1, "target={target}");
        assert_eq!(
            ledger(root.path()).status().unwrap().watermark,
            view.watermark
        );
    }
}
