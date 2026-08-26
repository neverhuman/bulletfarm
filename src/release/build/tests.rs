//! Component tests for release build production.

use std::{fs, path::Path};

use serde_json::{Value, json};

use super::{
    archive::{plan_entries, write_archive},
    checksums, manifest,
};
use crate::release::{ReleaseFile, ReleaseManifest, ReleasePackage, SignedReleaseFile};

const TARGET: &str = super::SUPPORTED_TARGET;

fn absent(path: &str) -> SignedReleaseFile {
    SignedReleaseFile {
        file: ReleaseFile {
            path: path.to_owned(),
            size: 0,
            digest: String::new(),
        },
        signature: ReleaseFile {
            path: format!("{path}.sig"),
            size: 0,
            digest: String::new(),
        },
    }
}

fn sources(root: &Path) -> Vec<(String, std::path::PathBuf, u32)> {
    let mut sources = Vec::new();
    for (name, body, mode) in [
        ("LICENSE", "Apache-2.0\n", 0o644_u32),
        ("share/family.lock", "schema_version = \"2\"\n", 0o644),
        ("bin/bullet", "#!/bin/true\n", 0o755),
        ("bin/bullet-family", "#!/bin/true\n", 0o755),
        ("bin/bullet-farmd", "#!/bin/true\n", 0o755),
        ("bin/bullet-gitd", "#!/bin/true\n", 0o755),
        ("bin/bullet-mcpd", "#!/bin/true\n", 0o755),
        ("bin/bullet-effects", "#!/bin/true\n", 0o755),
        ("bin/bullet-runner", "#!/bin/true\n", 0o755),
        ("bin/bullet-verifier", "#!/bin/true\n", 0o755),
    ] {
        let path = root.join(name.replace('/', "_"));
        fs::write(&path, body).expect("stage source");
        sources.push((format!("bullet-farm/{name}"), path, mode));
    }
    sources
}

#[test]
fn archive_entries_are_root_first_and_byte_sorted() {
    let staging = tempfile::tempdir().expect("staging");
    let entries = plan_entries(&sources(staging.path())).expect("planned entries");
    let paths = entries
        .iter()
        .map(|entry| entry.path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        [
            "bullet-farm",
            "bullet-farm/LICENSE",
            "bullet-farm/bin",
            "bullet-farm/bin/bullet",
            "bullet-farm/bin/bullet-effects",
            "bullet-farm/bin/bullet-family",
            "bullet-farm/bin/bullet-farmd",
            "bullet-farm/bin/bullet-gitd",
            "bullet-farm/bin/bullet-mcpd",
            "bullet-farm/bin/bullet-runner",
            "bullet-farm/bin/bullet-verifier",
            "bullet-farm/share",
            "bullet-farm/share/family.lock",
        ]
    );
    assert!(entries[0].directory && entries[0].size == 0);
    assert!(paths.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn a_produced_archive_is_admitted_by_the_committed_extractor() {
    let staging = tempfile::tempdir().expect("staging");
    let bundle = tempfile::tempdir().expect("bundle");
    let bundle_root = bundle.path().canonicalize().expect("canonical bundle");
    let sources = sources(staging.path());
    let entries = plan_entries(&sources).expect("planned entries");
    let relative = format!("{TARGET}/bullet-farm-v0.0.0-test-{TARGET}.tar.zst");
    fs::create_dir(bundle_root.join(TARGET)).expect("payload directory");
    let archive_path = bundle_root.join(&relative);
    write_archive(&archive_path, &entries, &sources).expect("archive written");
    let (digest, size) = super::digest_path(&archive_path).expect("archive digest");

    let readback = ReleaseManifest {
        release_manifest_schema_version: crate::release::RELEASE_MANIFEST_SCHEMA_VERSION.to_owned(),
        family_lock_schema_version: "2".to_owned(),
        family: "bullet-farm".to_owned(),
        tag: "v0.0.0-test".to_owned(),
        hub_commit_oid: String::new(),
        hub_tree_oid: String::new(),
        release_signing_identity: String::new(),
        family_lock: ReleaseFile {
            path: "family.lock".to_owned(),
            size: 0,
            digest: String::new(),
        },
        package: vec![ReleasePackage {
            target: TARGET.to_owned(),
            archive: SignedReleaseFile {
                file: ReleaseFile {
                    path: relative.clone(),
                    size,
                    digest,
                },
                signature: ReleaseFile {
                    path: format!("{relative}.sig"),
                    size: 0,
                    digest: String::new(),
                },
            },
            checksums: absent("checksums.checksums.json"),
            cyclonedx_sbom: absent("cyclonedx.cdx.json"),
            spdx_sbom: absent("spdx.spdx.json"),
            provenance: absent("provenance.intoto.jsonl"),
        }],
    };
    let destination = bundle_root.join("extracted");
    crate::release::archive::extract(&bundle_root, &readback, TARGET, &destination)
        .expect("the committed extractor admits the produced archive");
    for name in crate::release::archive::PACKAGED_BINARY_NAMES {
        use std::os::unix::fs::PermissionsExt;

        let path = format!("bin/{name}");
        let metadata = fs::metadata(destination.join(&path)).expect("materialized executable");
        assert!(metadata.is_file());
        assert_eq!(metadata.permissions().mode() & 0o777, 0o755, "{path}");
    }
    let license = fs::metadata(destination.join("LICENSE")).expect("materialized license");
    assert_eq!(
        {
            use std::os::unix::fs::PermissionsExt;
            license.permissions().mode() & 0o777
        },
        0o644,
        "non-executable package data must remain read-only"
    );
}

#[test]
fn builder_and_extractor_bind_the_same_exact_binary_names() {
    let mut built = super::BINARIES
        .iter()
        .map(|(_, _, name)| *name)
        .collect::<Vec<_>>();
    built.sort_unstable();
    assert_eq!(built, crate::release::archive::PACKAGED_BINARY_NAMES);
}

#[test]
fn a_manifest_that_binds_its_own_digest_is_refused() {
    let honest = br#"{"family":"bullet-farm","tag":"v0.0.0-test"}"#;
    let digest = super::digest_bytes(honest);
    manifest::admit(honest, &digest).expect("a manifest without its own digest is admitted");

    for circular in [
        format!(r#"{{"family":"bullet-farm","manifest_digest":"{digest}"}}"#),
        format!(r#"{{"family":"bullet-farm","self_digest":"{digest}"}}"#),
        format!(r#"{{"artifact":{{"digest_of_this_file":"{digest}"}}}}"#),
        r#"{"artifact":{"release_build_manifest_digest":"x"}}"#.to_owned(),
        format!(r#"{{"note":["{digest}"]}}"#),
        format!(r#"{{"nested":{{"deep":{{"value":"{digest}"}}}}}}"#),
    ] {
        let error = manifest::admit(circular.as_bytes(), &digest)
            .expect_err("a self-binding manifest is refused");
        assert_eq!(error.code(), "INVALID_RELEASE_BUILD_MANIFEST", "{circular}");
    }
}

#[test]
fn a_manifest_with_duplicate_members_is_refused() {
    let error = manifest::admit(br#"{"status":"FAIL","status":"PASS"}"#, "blake3:00")
        .expect_err("duplicate members are ambiguous");
    assert_eq!(error.code(), "INVALID_RELEASE_BUILD_MANIFEST");
}

#[test]
fn canonical_json_is_compact_sorted_and_newline_terminated() {
    let bytes = manifest::canonical_bytes(&json!({ "b": 1, "a": { "d": 2, "c": 3 } }))
        .expect("canonical bytes");
    assert_eq!(
        bytes,
        br#"{"a":{"c":3,"d":2},"b":1}"#.iter().copied().chain([b'\n']).collect::<Vec<_>>()
    );
}

#[test]
fn the_checksum_manifest_never_names_itself() {
    assert_eq!(checksums::CHECKSUM_ALGORITHM, "blake3");
    let document: Value = serde_json::from_slice(
        br#"{"bundle_file":[{"path":"a.tar.zst","size":1,"digest":"blake3:00"}]}"#,
    )
    .expect("document");
    let named = document
        .get("bundle_file")
        .and_then(Value::as_array)
        .expect("bundle files")
        .iter()
        .filter_map(|entry| entry.get("path").and_then(Value::as_str))
        .any(|path| path.ends_with(".checksums.json"));
    assert!(!named, "a checksum manifest is never its own subject");
}
