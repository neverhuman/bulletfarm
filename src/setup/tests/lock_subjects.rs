use std::{fs, path::Path, process::Command};

use crate::family_lock;

use super::{TAG, test_git_output};

pub(super) fn fixture_external_subjects(root: &Path, home: &Path) -> family_lock::ExternalSubjects {
    let hub = root.join("bullet-farm");
    let portal = root.join("bullet-portal");
    let allowed = fs::read(hub.join("release/allowed_signers")).expect("allowed signers");
    let public_key = root.parent().unwrap().join("signing.pub");
    let public_key_bytes = fs::read(&public_key).expect("fixture public key");
    let output = Command::new("/usr/bin/ssh-keygen")
        .args(["-lf", public_key.to_str().unwrap(), "-E", "sha256"])
        .output()
        .expect("fingerprint fixture key");
    assert!(output.status.success());
    let fingerprint = String::from_utf8(output.stdout)
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap()
        .to_owned();
    family_lock::ExternalSubjects {
        toolchain: fixture_toolchains(),
        provider: vec![family_lock::ProviderSubject {
            id: "claude".into(),
            version: "1.0.0".into(),
            profile: "service".into(),
            install_path: "/usr/lib/bullet/providers/claude/1.0.0/claude".into(),
            binary_digest: fixture_digest(b'p'),
            protocol_digest: fixture_digest(b'r'),
            size_bytes: 1,
        }],
        portal: family_lock::PortalSubject {
            id: "portal".into(),
            version: "1.0.0".into(),
            source_commit_oid: fixture_tagged_subject(&portal, home, "commit"),
            source_tree_oid: fixture_tagged_subject(&portal, home, "tree"),
            install_path: "/usr/lib/bullet/portal/1.0.0/index.html".into(),
            bundle_digest: fixture_digest(b'b'),
            manifest_digest: fixture_digest(b'a'),
            size_bytes: 1,
        },
        sandbox: vec![family_lock::SandboxSubject {
            id: "sandbox-s1".into(),
            class: "s1".into(),
            platform: "ubuntu-24-04-x86-64".into(),
            install_path: "/usr/lib/bullet/sandboxes/s1/rootfs.img".into(),
            image_digest: fixture_digest(b'i'),
            policy_digest: fixture_digest(b's'),
            size_bytes: 1,
        }],
        jeryu: family_lock::JeryuSubject {
            id: "jeryu".into(),
            version: "1.0.0".into(),
            tag: "v1.0.0".into(),
            install_path: "/usr/lib/bullet/jeryu/1.0.0/jeryu".into(),
            manifest_digest: fixture_digest(b'1'),
            binary_digest: fixture_digest(b'2'),
            api_schema_digest: fixture_digest(b'3'),
            capability_digest: fixture_digest(b'4'),
            sbom_digest: fixture_digest(b'5'),
            provenance_digest: fixture_digest(b'6'),
            signature_digest: fixture_digest(b'7'),
            size_bytes: 1,
        },
        release_signing: family_lock::ReleaseSigningSubject {
            id: "release-signing".into(),
            identity: format!("release@bullet.farm|ed25519|{fingerprint}"),
            policy_digest: fixture_digest(b'8'),
            allowed_signers_digest: format!("blake3:{}", blake3::hash(&allowed).to_hex()),
            key_digest: format!("blake3:{}", blake3::hash(&public_key_bytes).to_hex()),
            policy_path: "/etc/bullet/release/allowed_signers".into(),
            not_before_unix_ms: 1,
            not_after_unix_ms: 2,
        },
    }
}

fn fixture_toolchains() -> Vec<family_lock::ToolchainSubject> {
    [
        ("cargo", "1.95.0", "rust/1.95.0/cargo", b't', b'm'),
        ("node", "22.23.2", "node/22.23.2/node", b'n', b'o'),
        ("npm-cli", "10.9.8", "npm/10.9.8/npm-cli.js", b'q', b's'),
    ]
    .into_iter()
    .map(
        |(id, version, relative, binary, manifest)| family_lock::ToolchainSubject {
            id: id.into(),
            version: version.into(),
            install_path: format!("/usr/lib/bullet/toolchains/{relative}"),
            binary_digest: fixture_digest(binary),
            manifest_path: format!("/usr/lib/bullet/toolchains/{relative}.manifest"),
            manifest_digest: fixture_digest(manifest),
            size_bytes: 1,
        },
    )
    .collect()
}

fn fixture_digest(seed: u8) -> String {
    format!("blake3:{}", blake3::hash(&[seed]).to_hex())
}

fn fixture_tagged_subject(repo: &Path, home: &Path, class: &str) -> String {
    let suffix = if class == "commit" {
        "^{commit}"
    } else {
        "^{tree}"
    };
    format!(
        "sha1:{}",
        test_git_output(repo, home, &["rev-parse", &format!("{TAG}{suffix}")])
    )
}
