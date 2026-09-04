#[cfg(unix)]
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::Path,
    process::Command,
};

const DEMO_LAUNCHER: &str = include_str!("../scripts/demo.sh");
const PREVIEW_LAUNCHER: &str = include_str!("../scripts/preview.sh");

#[cfg(unix)]
fn rejected_data_directory(path: &Path) -> std::process::Output {
    Command::new("bash")
        .arg("scripts/demo.sh")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("BULLET_DATA_DIR", path)
        .output()
        .expect("launcher must execute")
}

#[test]
fn demo_launcher_uses_a_fresh_default_and_preserves_explicit_data() {
    assert!(
        DEMO_LAUNCHER.contains("if [[ -n \"${BULLET_DATA_DIR:-}\" ]]"),
        "an explicit demo data directory must remain supported"
    );
    assert!(
        DEMO_LAUNCHER.contains("DATA=\"$BULLET_DATA_DIR\""),
        "the explicit demo data directory must be used exactly"
    );
    assert!(
        DEMO_LAUNCHER.contains("realpath -e -- \"$DATA\""),
        "the explicit directory must reject symlinked ancestors"
    );
    assert!(
        !DEMO_LAUNCHER.contains("chmod") && !DEMO_LAUNCHER.contains("mkdir -p \"$DATA\""),
        "the launcher must not mutate an unadmitted explicit path"
    );
    assert!(
        DEMO_LAUNCHER.contains("DATA=\"$(mktemp -d /tmp/bullet-txn.XXXXXX)\""),
        "the default demo must allocate a fresh bounded run directory"
    );
    assert!(
        !DEMO_LAUNCHER.contains("DATA=\"${BULLET_DATA_DIR:-$KERNEL/target/demo}\""),
        "the launcher must not reuse one schema-sensitive default database"
    );
    assert!(
        !DEMO_LAUNCHER.contains("rm -"),
        "the launcher must preserve prior demo runs instead of deleting them"
    );

    #[cfg(unix)]
    {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("target");
        fs::create_dir(&target).unwrap();
        fs::set_permissions(&target, fs::Permissions::from_mode(0o755)).unwrap();
        let bad_mode = rejected_data_directory(&target);
        assert!(!bad_mode.status.success());
        assert_eq!(
            fs::metadata(&target).unwrap().permissions().mode() & 0o777,
            0o755
        );

        fs::set_permissions(&target, fs::Permissions::from_mode(0o700)).unwrap();
        let alias = root.path().join("alias");
        symlink(&target, &alias).unwrap();
        let linked = rejected_data_directory(&alias);
        assert!(!linked.status.success());
        assert_eq!(
            fs::metadata(&target).unwrap().permissions().mode() & 0o777,
            0o700
        );

        let relative = rejected_data_directory(Path::new("relative-demo-data"));
        assert!(!relative.status.success());
    }
}

#[test]
fn demo_launcher_labels_component_evidence_before_running_the_fixture() {
    let evidence_label = DEMO_LAUNCHER
        .find("echo \"evidence_class: COMPONENT_PROOF\"")
        .expect("demo must print its component evidence class");
    let fixture_trust_label = DEMO_LAUNCHER
        .find("echo \"verifier_fixture_trust: UNSIGNED_FIXTURE\"")
        .expect("demo must print that verifier execution is an unsigned fixture");
    let independence_label = DEMO_LAUNCHER
        .find("echo \"independent_verification_eligible: false\"")
        .expect("demo must print that fixture verification is not independent");
    let release_label = DEMO_LAUNCHER
        .find("echo \"release_gate_eligible: false\"")
        .expect("demo must print that it cannot clear a release gate");
    let transaction_label = DEMO_LAUNCHER
        .find("echo \"transaction_proof: absent\"")
        .expect("demo must print that transaction proof remains absent");
    let kernel_run = DEMO_LAUNCHER
        .find("cargo run --locked -q -p bullet --bin transaction_demo")
        .expect("demo must run the offline fixture saga");

    let fixture_build = DEMO_LAUNCHER
        .find("cargo build --locked -q -p bullet-verifier --features fixture-executor --bin bullet-verifier-fixture")
        .expect("demo must build only the explicit fixture verifier");
    let stage_create = DEMO_LAUNCHER
        .find("VERIFIER_FIXTURE_STAGE=\"$(mktemp -d /tmp/bullet-verifier-fixture.XXXXXX)\"")
        .expect("demo must allocate a fresh private verifier stage");
    let stage_canonical = DEMO_LAUNCHER
        .find("CANONICAL_VERIFIER_FIXTURE_STAGE=\"$(realpath -e -- \"$VERIFIER_FIXTURE_STAGE\")\"")
        .expect("demo must canonicalize its verifier stage");
    let stage_mode = DEMO_LAUNCHER
        .find("$(stat -c '%u:%a' -- \"$VERIFIER_FIXTURE_STAGE\")")
        .expect("demo must require caller ownership and exact stage mode");
    let staged_path = DEMO_LAUNCHER
        .find("BULLET_VERIFIER_FIXTURE_BIN=\"$VERIFIER_FIXTURE_STAGE/bullet-verifier-fixture\"")
        .expect("demo must bind the verifier to the private stage");
    let stage_copy = DEMO_LAUNCHER
        .find("cp --reflink=never -- \"$VERIFIER_FIXTURE_BUILD_BIN\" \"$BULLET_VERIFIER_FIXTURE_BIN\"")
        .expect("demo must copy bytes without a reflink");
    let staged_canonical = DEMO_LAUNCHER
        .find(
            "CANONICAL_VERIFIER_FIXTURE_BIN=\"$(realpath -e -- \"$BULLET_VERIFIER_FIXTURE_BIN\")\"",
        )
        .expect("demo must canonicalize the staged verifier");
    let staged_single_link = DEMO_LAUNCHER
        .find("$(stat -c '%u:%h' -- \"$BULLET_VERIFIER_FIXTURE_BIN\")")
        .expect("demo must require a caller-owned single-link staged verifier");
    let staged_bytes = DEMO_LAUNCHER
        .find("cmp -s -- \"$VERIFIER_FIXTURE_BUILD_BIN\" \"$BULLET_VERIFIER_FIXTURE_BIN\"")
        .expect("demo must require the staged bytes to equal the built fixture");
    let staged_digest = DEMO_LAUNCHER
        .find("BULLET_VERIFIER_FIXTURE_SHA256=\"$(sha256sum -- \"$BULLET_VERIFIER_FIXTURE_BIN\")\"")
        .expect("demo must hash only the admitted staged verifier");
    let staged_export = DEMO_LAUNCHER
        .find("export BULLET_VERIFIER_FIXTURE_BIN")
        .expect("demo must export the admitted staged verifier");

    assert!(evidence_label < kernel_run);
    assert!(fixture_trust_label < kernel_run);
    assert!(independence_label < kernel_run);
    assert!(release_label < kernel_run);
    assert!(transaction_label < kernel_run);
    for ordered in [
        fixture_build,
        stage_create,
        stage_canonical,
        stage_mode,
        staged_path,
        stage_copy,
        staged_canonical,
        staged_single_link,
        staged_bytes,
        staged_digest,
        staged_export,
        kernel_run,
    ]
    .windows(2)
    {
        assert!(
            ordered[0] < ordered[1],
            "fixture staging order must be exact"
        );
    }

    for exact_binding in [
        "cargo build --locked -q -p bullet-verifier --features fixture-executor --bin bullet-verifier-fixture",
        "export BULLET_GITD_BIN=\"$GIT/target/debug/bullet-gitd\"",
        "export BULLET_GITD_FIXTURE_BIN=\"$GIT/target/debug/bullet-gitd-fixture\"",
        "BULLET_GITD_SHA256=\"$(sha256sum -- \"$BULLET_GITD_BIN\")\"",
        "BULLET_GITD_SHA256=\"${BULLET_GITD_SHA256%% *}\"",
        "export BULLET_GITD_SHA256",
        "BULLET_GITD_FIXTURE_SHA256=\"$(sha256sum -- \"$BULLET_GITD_FIXTURE_BIN\")\"",
        "BULLET_GITD_FIXTURE_SHA256=\"${BULLET_GITD_FIXTURE_SHA256%% *}\"",
        "export BULLET_GITD_FIXTURE_SHA256",
        "export BULLET_FARMD_BIN=\"$KERNEL/target/debug/bullet-farmd\"",
        "VERIFIER_FIXTURE_BUILD_BIN=\"$KERNEL/target/debug/bullet-verifier-fixture\"",
        "BULLET_VERIFIER_FIXTURE_BIN=\"$VERIFIER_FIXTURE_STAGE/bullet-verifier-fixture\"",
        "BULLET_VERIFIER_FIXTURE_SHA256=\"$(sha256sum -- \"$BULLET_VERIFIER_FIXTURE_BIN\")\"",
        "BULLET_VERIFIER_FIXTURE_SHA256=\"${BULLET_VERIFIER_FIXTURE_SHA256%% *}\"",
        "export BULLET_VERIFIER_FIXTURE_BIN",
        "export BULLET_VERIFIER_FIXTURE_SHA256",
    ] {
        assert!(DEMO_LAUNCHER.contains(exact_binding));
    }
    assert!(!DEMO_LAUNCHER.contains("${BULLET_GITD_BIN:-"));
    assert!(!DEMO_LAUNCHER.contains("export BULLET_VERIFIER_BIN="));
    assert!(
        !DEMO_LAUNCHER.contains("cargo build --locked -q -p bullet-verifier --bin bullet-verifier")
    );
    assert!(!DEMO_LAUNCHER.contains("export VERIFIER_FIXTURE_BUILD_BIN"));
    assert!(
        !DEMO_LAUNCHER.contains(
            "BULLET_VERIFIER_FIXTURE_BIN=\"$KERNEL/target/debug/bullet-verifier-fixture\""
        )
    );
    assert!(!DEMO_LAUNCHER.contains(
        "export BULLET_VERIFIER_FIXTURE_BIN=\"$KERNEL/target/debug/bullet-verifier-fixture\""
    ));
    assert!(
        PREVIEW_LAUNCHER.contains(".effect_unknown_outcome == \"NOT_DISPATCHED\""),
        "current preview must distinguish a non-dispatched effect from an ambiguous UNKNOWN effect"
    );
    assert!(!PREVIEW_LAUNCHER.contains(".effect_unknown_outcome == \"unknown\""));
    assert!(
        PREVIEW_LAUNCHER.contains("cargo run --locked --quiet -p bullet --bin bullet -- demo"),
        "preview must select the component CLI explicitly when the package has multiple binaries"
    );
    for exact_release_assertion in [
        "--profile self-hosted-v1",
        ".profile == \"self-hosted-v1\"",
        "(.gates | length) == 27",
        "all(.gates[]; .status == \"BLOCKED\")",
        "([.gates[] | select(.status == \"PASS\")] | length) == 0",
        "([.gates[] | select(.id == \"release.transaction-demo\")] | length) == 1",
        "([.gates[] | select(.id == \"release.profile.self-hosted-v1\")] | length) == 1",
    ] {
        assert!(
            PREVIEW_LAUNCHER.contains(exact_release_assertion),
            "preview must bind self-hosted release truth: {exact_release_assertion}"
        );
    }
    for sentinel in [
        ".candidate_head == \"NOT_PRODUCED\"",
        ".evidence_result == \"NOT_RUN\"",
        ".effect_outcome == \"NOT_DISPATCHED\"",
        ".effect_unknown_outcome == \"NOT_DISPATCHED\"",
    ] {
        assert!(
            PREVIEW_LAUNCHER.contains(sentinel),
            "preview must bind negative demo sentinel {sentinel}"
        );
    }
}
