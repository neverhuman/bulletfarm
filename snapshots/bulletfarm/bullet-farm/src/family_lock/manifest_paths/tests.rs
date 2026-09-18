use std::os::unix::fs::symlink;

use super::*;

struct Fixture(tempfile::TempDir);

impl Fixture {
    fn new() -> Self {
        let fixture = Self(tempfile::tempdir().unwrap());
        for name in MEMBERS {
            fs::create_dir(fixture.root().join(name)).unwrap();
            fs::create_dir(fixture.root().join(name).join(".git")).unwrap();
        }
        fixture.write(&fixture.manifest(true));
        fixture
    }

    fn root(&self) -> &Path {
        self.0.path()
    }

    fn write(&self, text: &str) {
        fs::write(self.root().join("repos.manifest.toml"), text).unwrap();
    }

    fn manifest(&self, portable: bool) -> String {
        let version = if portable { "1.3.0" } else { "1.2.0" };
        let mut text = format!("schema_version = {version:?}\nfamily = \"bullet-farm\"\n");
        if portable {
            text.push_str("split_root = \".\"\n");
        }
        text.push_str(&format!("required_repos = {MEMBERS:?}\n"));
        for name in MEMBERS {
            let path = if portable {
                PathBuf::from(name)
            } else {
                self.root().join(name)
            };
            text.push_str(&format!(
                "[[repo]]\nname = {name:?}\npath = {:?}\njeryu_slug = \"root/{name}\"\n",
                path.to_str().unwrap()
            ));
        }
        text
    }
}

#[test]
fn portable_and_absolute_manifests_bind_the_same_four_checkouts() {
    let fixture = Fixture::new();
    for portable in [true, false] {
        let text = fixture.manifest(portable);
        fixture.write(&text);
        validate_publication_family_root(fixture.root()).unwrap();
        let manifest: super::super::Manifest = toml::from_str(&text).unwrap();
        let indexed = super::super::indexed_repos(fixture.root(), &manifest).unwrap();
        for name in MEMBERS {
            assert_eq!(indexed[name], fixture.root().join(name));
        }
    }
    let legacy: PathMode = toml::from_str("").unwrap();
    assert_eq!(
        legacy
            .resolve(
                fixture.root(),
                "bullet-git",
                &fixture.root().join("bullet-git")
            )
            .unwrap(),
        fixture.root().join("bullet-git")
    );
    assert!(
        legacy
            .resolve(fixture.root(), "bullet-git", Path::new("bullet-git"))
            .is_err()
    );
}

#[test]
fn portable_paths_versions_and_root_markers_refuse_aliases() {
    let fixture = Fixture::new();
    let text = fixture.manifest(true);
    for path in [
        "../bullet-git",
        "./bullet-git",
        "bullet-git/",
        "x/../bullet-git",
        "/bullet-git",
        "bullet-kernel",
        "",
        "bullet-git//",
    ] {
        let hostile = text.replace("path = \"bullet-git\"", &format!("path = {path:?}"));
        assert!(
            validate_exact_manifest(hostile.as_bytes(), fixture.root()).is_err(),
            "{path}"
        );
    }
    for marker in [
        "",
        "/",
        "..",
        "./",
        "bullet-farm",
        fixture.root().to_str().unwrap(),
    ] {
        let hostile = text.replace("split_root = \".\"", &format!("split_root = {marker:?}"));
        assert!(
            validate_exact_manifest(hostile.as_bytes(), fixture.root()).is_err(),
            "{marker}"
        );
    }
    for version in ["1.2.0", "1.1.0", "1.3.1", "2.0.0", ""] {
        let hostile = text.replace("1.3.0", version);
        assert!(validate_exact_manifest(hostile.as_bytes(), fixture.root()).is_err());
        let mode: PathMode = toml::from_str(&format!("schema_version = {version:?}")).unwrap();
        assert_eq!(
            mode.validate_if_portable(b"", fixture.root()).is_ok(),
            version == "1.2.0"
        );
    }
    for removed in ["split_root = \".\"\n", "schema_version = \"1.3.0\"\n"] {
        assert!(
            validate_exact_manifest(text.replace(removed, "").as_bytes(), fixture.root()).is_err()
        );
    }
    assert!(validate_exact_manifest(text.as_bytes(), Path::new(".")).is_err());
    assert!(validate_exact_manifest(text.as_bytes(), &fixture.root().join(".")).is_err());
    assert!(
        validate_exact_manifest(text.as_bytes(), &fixture.root().join("bullet-farm/..")).is_err()
    );
    let absolute = fixture.manifest(false).replace(
        &format!(
            "path = {:?}",
            fixture.root().join("bullet-git").to_str().unwrap()
        ),
        "path = \"/unadmitted/bullet-git\"",
    );
    assert!(validate_exact_manifest(absolute.as_bytes(), fixture.root()).is_err());
}

#[test]
fn publication_manifest_requires_exact_identity_inventory_and_framing() {
    let fixture = Fixture::new();
    let text = fixture.manifest(true);
    for (old, new) in [
        ("family = \"bullet-farm\"", "family = \"other\""),
        ("root/bullet-git", "root/not-git"),
        ("name = \"bullet-git\"", "name = \"bullet-kernel\""),
        (
            "\"bullet-kernel\", \"bullet-git\"",
            "\"bullet-git\", \"bullet-kernel\"",
        ),
        ("\"bullet-kernel\", ", ""),
    ] {
        assert!(
            validate_exact_manifest(text.replace(old, new).as_bytes(), fixture.root()).is_err()
        );
    }
    assert!(validate_exact_manifest(text.trim_end().as_bytes(), fixture.root()).is_err());
    assert!(validate_exact_manifest(&[0xff, b'\n'], fixture.root()).is_err());
    assert!(validate_exact_manifest(&vec![b'\n'; MAX_BYTES as usize + 1], fixture.root()).is_err());
    let extra = format!(
        "{text}[[repo]]\nname = \"extra\"\npath = \"extra\"\njeryu_slug = \"root/extra\"\n"
    );
    assert!(validate_exact_manifest(extra.as_bytes(), fixture.root()).is_err());
}

#[test]
fn publication_root_refuses_symbolic_missing_and_nonordinary_checkouts() {
    for path in ["repos.manifest.toml", "bullet-git", "bullet-git/.git"] {
        let fixture = Fixture::new();
        let original = fixture.root().join(path);
        let displaced = fixture.root().join("retained");
        fs::rename(&original, &displaced).unwrap();
        assert!(
            validate_publication_family_root(fixture.root()).is_err(),
            "missing {path}"
        );
        symlink(&displaced, &original).unwrap();
        assert!(
            validate_publication_family_root(fixture.root()).is_err(),
            "symlink {path}"
        );
    }
    let fixture = Fixture::new();
    let git = fixture.root().join("bullet-git/.git");
    fs::remove_dir(&git).unwrap();
    fs::write(git, "gitdir: elsewhere\n").unwrap();
    assert!(validate_publication_family_root(fixture.root()).is_err());

    let fixture = Fixture::new();
    let parent = tempfile::tempdir().unwrap();
    let linked = parent.path().join("linked");
    symlink(fixture.root(), &linked).unwrap();
    assert!(validate_publication_family_root(&linked).is_err());
    assert!(validate_publication_family_root(&linked.join("bullet-farm/..")).is_err());
    fs::hard_link(
        fixture.root().join("repos.manifest.toml"),
        parent.path().join("copy"),
    )
    .unwrap();
    assert!(validate_publication_family_root(fixture.root()).is_err());
}

#[test]
fn publication_admission_refuses_manifest_and_checkout_substitution_during_read() {
    let fixture = Fixture::new();
    assert!(
        validate_root_with(fixture.root(), || {
            fixture.write(
                &fixture
                    .manifest(true)
                    .replace("root/bullet-git", "root/other-git"),
            );
        })
        .is_err()
    );

    let fixture = Fixture::new();
    assert!(
        validate_root_with(fixture.root(), || {
            fs::rename(
                fixture.root().join("repos.manifest.toml"),
                fixture.root().join("old.toml"),
            )
            .unwrap();
            fixture.write(&fixture.manifest(true));
        })
        .is_err()
    );

    let fixture = Fixture::new();
    assert!(
        validate_root_with(fixture.root(), || {
            fs::rename(
                fixture.root().join("bullet-git"),
                fixture.root().join("old-git"),
            )
            .unwrap();
            fs::create_dir_all(fixture.root().join("bullet-git/.git")).unwrap();
        })
        .is_err()
    );
}
