use std::{ffi::OsString, fs, path::Path};

use bullet_family::coord::CoordError;

fn run_cli(root: &Path, args: &[String]) -> Result<String, CoordError> {
    let mut argv = vec![
        OsString::from("bullet-family"),
        OsString::from("--root"),
        root.as_os_str().to_os_string(),
        OsString::from("coord"),
        OsString::from("recover-rollover"),
    ];
    argv.extend(args.iter().map(OsString::from));
    bullet_family::cli::run(argv, Ok(root.to_path_buf()))
}

fn recovery_args(root: &Path) -> Vec<String> {
    [
        ("manifest", "manifest.json"),
        ("inspection", "inspection.json"),
        ("authorization", "authorization.json"),
        ("authorization-signature", "authorization-signature.json"),
        ("bootstrap-provenance", "bootstrap-provenance.json"),
        ("interrupted-capture", "interrupted.jsonl"),
        ("tainted-generation", "tainted.jsonl"),
        ("frozen-live-source", ".bullet-family/coord/events.jsonl"),
    ]
    .into_iter()
    .flat_map(|(name, relative)| {
        [
            format!("--{name}"),
            root.join(relative).to_str().unwrap().to_owned(),
        ]
    })
    .collect()
}

fn family() -> tempfile::TempDir {
    let family = tempfile::tempdir().unwrap();
    fs::write(family.path().join("repos.manifest.toml"), "version = 1\n").unwrap();
    family
}

#[test]
fn closed_option_refusals_are_creation_free() {
    let family = family();
    for missing_index in 0..8 {
        let mut missing_subject = recovery_args(family.path());
        missing_subject.drain(missing_index * 2..missing_index * 2 + 2);
        let missing = run_cli(family.path(), &missing_subject).unwrap_err();
        assert_eq!(missing.code(), "MISSING_OPTION");
        assert!(!family.path().join(".bullet-family").exists());
    }

    let mut unknown = recovery_args(family.path());
    unknown.extend(["--result".to_owned(), "published".to_owned()]);
    let rejected = run_cli(family.path(), &unknown).unwrap_err();
    assert_eq!(rejected.code(), "UNKNOWN_OPTION");
    assert!(!family.path().join(".bullet-family").exists());
}

#[cfg(not(target_os = "linux"))]
#[test]
fn unsupported_platform_refuses_before_subject_io_or_coord_creation() {
    let family = family();
    let error = run_cli(family.path(), &recovery_args(family.path())).unwrap_err();
    assert_eq!(error.code(), "COORD_RECOVERY_PLATFORM_UNSUPPORTED");
    assert!(!family.path().join(".bullet-family").exists());
}
