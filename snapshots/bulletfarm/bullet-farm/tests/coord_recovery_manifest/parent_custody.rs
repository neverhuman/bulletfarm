use super::*;

fn coord_snapshot(fixture: &Fixture) -> Vec<(OsString, Vec<u8>)> {
    let mut entries = fs::read_dir(fixture.root.path().join(".bullet-family/coord"))
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (entry.file_name(), fs::read(entry.path()).unwrap())
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    entries
}

fn inspect(fixture: &Fixture, interrupted: &Path, output: &Path) -> Output {
    run(
        fixture.root.path(),
        "recovery-inspect",
        &[
            ("interrupted-capture", interrupted),
            ("tainted-generation", &fixture.tainted),
            ("frozen-live-source", &fixture.frozen),
            ("output", output),
        ],
    )
}

fn assert_parent_refusal(result: Output) {
    assert!(!result.status.success());
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(
        stderr.contains("INVALID_RECOVERY_MANIFEST_PRODUCTION")
            && stderr.contains("immediate parent"),
        "unexpected parent-custody refusal: {stderr}"
    );
}

#[test]
fn inspect_refuses_wrong_mode_and_symlinked_source_parents_without_mutation() {
    let fixture = fixture();
    let output_parent = fixture.root.path().join("recovery-output");
    fs::create_dir(&output_parent).unwrap();
    fs::set_permissions(&output_parent, fs::Permissions::from_mode(0o700)).unwrap();
    let output = output_parent.join("inspection.json");
    let before = coord_snapshot(&fixture);

    for mode in [0o750, 0o755, 0o775, 0o777] {
        fs::set_permissions(fixture.root.path(), fs::Permissions::from_mode(mode)).unwrap();
        let refused = inspect(&fixture, &fixture.interrupted, &output);
        assert_parent_refusal(refused);
        assert!(!output.exists());
        assert_eq!(coord_snapshot(&fixture), before);
    }
    fs::set_permissions(fixture.root.path(), fs::Permissions::from_mode(0o700)).unwrap();

    let coord = fixture.root.path().join(".bullet-family/coord");
    fs::set_permissions(&coord, fs::Permissions::from_mode(0o755)).unwrap();
    let refused = inspect(&fixture, &fixture.interrupted, &output);
    assert_parent_refusal(refused);
    assert!(!output.exists());
    assert_eq!(coord_snapshot(&fixture), before);
    fs::set_permissions(&coord, fs::Permissions::from_mode(0o700)).unwrap();

    let private = fixture.root.path().join("private-source");
    fs::create_dir(&private).unwrap();
    fs::set_permissions(&private, fs::Permissions::from_mode(0o700)).unwrap();
    let moved = private.join("interrupted.partial");
    write_private(&moved, &fs::read(&fixture.interrupted).unwrap());
    let alias = fixture.root.path().join("source-alias");
    std::os::unix::fs::symlink(&private, &alias).unwrap();
    let refused = inspect(&fixture, &alias.join("interrupted.partial"), &output);
    assert_parent_refusal(refused);
    assert!(!output.exists());
    assert_eq!(coord_snapshot(&fixture), before);

    fs::set_permissions(&coord, fs::Permissions::from_mode(0o775)).unwrap();
    let legacy_output = output_parent.join("legacy-inspection.json");
    let inspection = success(
        fixture.root.path(),
        "recovery-inspect",
        &[
            ("interrupted-capture", &fixture.interrupted),
            ("tainted-generation", &fixture.tainted),
            ("frozen-live-source", &fixture.frozen),
            ("output", &legacy_output),
        ],
    );
    assert_eq!(inspection["kind"], "bullet.coord.recovery-inspection.v1");
    fs::set_permissions(&coord, fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(coord_snapshot(&fixture), before);
}
