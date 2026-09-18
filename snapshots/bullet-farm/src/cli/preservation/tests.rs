use std::{
    ffi::OsStr,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use crate::{cli, coord::wave0_producer};

struct Fixture {
    family: tempfile::TempDir,
    records: tempfile::TempDir,
    incident_paths: Vec<PathBuf>,
}

impl Fixture {
    fn new() -> Self {
        let family = tempfile::tempdir().unwrap();
        let records = tempfile::tempdir().unwrap();
        fs::set_permissions(records.path(), fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(
            family.path().join("repos.manifest.toml"),
            "fixture = true\n",
        )
        .unwrap();
        let mut incident_paths = Vec::new();
        for (location, name) in [
            (".bullet-family/coord", "outer.json"),
            ("bullet-farm/.bullet-family/coord", "hub.json"),
        ] {
            let directory = family.path().join(location);
            fs::create_dir_all(&directory).unwrap();
            fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).unwrap();
            let incident = directory.join("events.jsonl");
            fs::write(&incident, b"retained incident bytes\n").unwrap();
            fs::set_permissions(&incident, fs::Permissions::from_mode(0o400)).unwrap();
            wave0_producer::produce_incident_inventory(
                &directory,
                OsStr::new("coord-preserved"),
                &records.path().join(name),
            )
            .unwrap();
            incident_paths.push(incident);
        }
        Self {
            family,
            records,
            incident_paths,
        }
    }

    fn options(&self) -> Vec<String> {
        [
            "--outer-inventory".into(),
            self.record("outer.json").display().to_string(),
            "--hub-inventory".into(),
            self.record("hub.json").display().to_string(),
            "--out".into(),
            self.record("pair.json").display().to_string(),
        ]
        .into()
    }

    fn record(&self, name: &str) -> PathBuf {
        self.records.path().join(name)
    }

    fn run(&self, options: Vec<String>) -> Result<String, crate::coord::CoordError> {
        let mut args = vec![
            "bullet-family".to_owned(),
            "--root".to_owned(),
            self.family.path().display().to_string(),
            "preservation-bind".to_owned(),
        ];
        args.extend(options);
        cli::run(
            args.into_iter().map(Into::into),
            Ok(self.family.path().to_path_buf()),
        )
    }

    fn assert_incidents_preserved(&self) {
        for path in &self.incident_paths {
            assert_eq!(fs::read(path).unwrap(), b"retained incident bytes\n");
            assert!(
                !path
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join("coord-preserved")
                    .exists()
            );
            assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
        }
    }
}

#[test]
fn cli_binds_real_producer_records_and_retries_without_incident_effects() {
    let fixture = Fixture::new();
    let original = fs::read(fixture.record("outer.json")).unwrap();
    let first = fixture.run(fixture.options()).unwrap();
    let first: serde_json::Value = bullet_wire::decode_unique_value(first.as_bytes()).unwrap();
    assert_eq!(first["outcome"], "CREATED");
    assert!(
        first["preservation_id"]
            .as_str()
            .unwrap()
            .starts_with("fgp_")
    );
    let bytes = fs::read(fixture.record("pair.json")).unwrap();
    assert_eq!(bytes.last(), Some(&b'\n'));
    assert_eq!(first["byte_length"], bytes.len());
    assert_eq!(
        fs::metadata(fixture.record("pair.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o7777,
        0o400
    );
    let retry = fixture.run(fixture.options()).unwrap();
    let retry: serde_json::Value = bullet_wire::decode_unique_value(retry.as_bytes()).unwrap();
    assert_eq!(retry["outcome"], "ADOPTED_EXACT_EXISTING");
    for key in ["preservation_id", "sealed_sha256", "byte_length", "path"] {
        assert_eq!(retry[key], first[key]);
    }
    assert_eq!(fs::read(fixture.record("pair.json")).unwrap(), bytes);
    assert_eq!(fs::read(fixture.record("outer.json")).unwrap(), original);
    fixture.assert_incidents_preserved();
}

#[test]
fn cli_refuses_missing_duplicate_swapped_and_unsafe_inputs_before_output() {
    let fixture = Fixture::new();
    for variant in [
        "missing",
        "duplicate",
        "unknown",
        "flag",
        "roles",
        "alias",
        "inside",
        "relative",
    ] {
        let mut options = fixture.options();
        match variant {
            "missing" => {
                options.drain(2..4);
            }
            "duplicate" => {
                options.extend([
                    "--out".into(),
                    fixture.record("other.json").display().to_string(),
                ]);
            }
            "unknown" => {
                options.extend(["--operator-approved".into(), "true".into()]);
            }
            "flag" => options.push("--json".into()),
            "roles" => options.swap(1, 3),
            "alias" => options[3] = options[1].clone(),
            "inside" => {
                options[1] = fixture
                    .family
                    .path()
                    .join(".bullet-family/coord/events.jsonl")
                    .display()
                    .to_string()
            }
            "relative" => options[5] = Path::new("pair.json").display().to_string(),
            _ => unreachable!(),
        }
        assert!(fixture.run(options).is_err(), "accepted {variant}");
        assert!(!fixture.record("pair.json").exists());
        fixture.assert_incidents_preserved();
    }
    let input = fixture.record("outer.json");
    fs::write(&input, b"{}\n").unwrap();
    assert!(fixture.run(fixture.options()).is_err());
    assert!(!fixture.record("pair.json").exists());
    fixture.assert_incidents_preserved();
}
