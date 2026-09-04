use bullet_family::family_lock::{
    self, ExternalSubjects, FamilyLock, JeryuSubject, LockedFile, LockedHub, LockedMember,
    PortalSubject, ProviderSubject, ReleaseSigningSubject, SandboxSubject, ToolchainSubject,
};

use super::*;

pub(crate) fn fixture_hub() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "bullet-semantic-registry-hub-{}",
        std::process::id()
    ))
}

pub(crate) fn fixture_family_lock_path() -> std::path::PathBuf {
    fixture_hub().join("family.lock")
}

fn locked_member(name: &str, oid: char, lockfile: &str) -> LockedMember {
    LockedMember {
        name: name.to_owned(),
        jeryu_url: Some(format!("https://jeryu.example.invalid/git/root/{name}.git")),
        jeryu_slug: Some(format!("root/{name}")),
        tag: "v1.0.0".to_owned(),
        commit_oid: format!("sha1:{}", release_hex(oid, 40)),
        tree_oid: format!("sha1:{}", release_hex(oid, 40)),
        release_signing_identity: format!(
            "release-{name}@bullet.invalid|ed25519|SHA256:{}",
            release_hex(oid.to_ascii_uppercase(), 24)
        ),
        lockfile: vec![LockedFile {
            path: lockfile.to_owned(),
            digest: tagged_digest(oid),
        }],
        artifact: Vec::new(),
    }
}

fn matching_family_lock(policy_digest: &str) -> FamilyLock {
    let portal_oid = '4';
    let release_identity =
        "release@bullet.invalid|ed25519|SHA256:ABCDEFGHIJKLMNOPQRSTUVWX".to_owned();
    FamilyLock {
        schema_version: "3".to_owned(),
        family: "bullet-farm".to_owned(),
        tag: "v1.0.0".to_owned(),
        schema_bundle_hash: tagged_digest('a'),
        hub: LockedHub {
            name: "bullet-farm".to_owned(),
            tag: "v1.0.0".to_owned(),
            release_signing_identity: release_identity.clone(),
        },
        member: vec![
            locked_member("bullet-git", '2', "Cargo.lock"),
            locked_member("bullet-kernel", '3', "Cargo.lock"),
            locked_member("bullet-portal", portal_oid, "package-lock.json"),
        ],
        external: ExternalSubjects {
            toolchain: vec![
                ToolchainSubject {
                    id: "cargo".to_owned(),
                    version: "1.95.0".to_owned(),
                    install_path: "/usr/lib/bullet/toolchains/rust/1.95.0/cargo".to_owned(),
                    binary_digest: tagged_digest('5'),
                    manifest_path: "/usr/lib/bullet/toolchains/rust/1.95.0/cargo.manifest"
                        .to_owned(),
                    manifest_digest: tagged_digest('6'),
                    size_bytes: 1,
                },
                ToolchainSubject {
                    id: "node".to_owned(),
                    version: "22.23.2".to_owned(),
                    install_path: "/usr/lib/bullet/toolchains/node/22.23.2/node".to_owned(),
                    binary_digest: tagged_digest('7'),
                    manifest_path: "/usr/lib/bullet/toolchains/node/22.23.2/node.manifest"
                        .to_owned(),
                    manifest_digest: tagged_digest('8'),
                    size_bytes: 1,
                },
                ToolchainSubject {
                    id: "npm-cli".to_owned(),
                    version: "10.9.8".to_owned(),
                    install_path: "/usr/lib/bullet/toolchains/npm/10.9.8/npm-cli.js".to_owned(),
                    binary_digest: tagged_digest('9'),
                    manifest_path: "/usr/lib/bullet/toolchains/npm/10.9.8/npm-cli.manifest"
                        .to_owned(),
                    manifest_digest: tagged_digest('a'),
                    size_bytes: 1,
                },
            ],
            provider: vec![ProviderSubject {
                id: "codex".to_owned(),
                version: "1.0.0".to_owned(),
                profile: "service".to_owned(),
                install_path: "/usr/lib/bullet/providers/codex/1.0.0/codex".to_owned(),
                binary_digest: tagged_digest('b'),
                protocol_digest: tagged_digest('c'),
                size_bytes: 1,
            }],
            portal: PortalSubject {
                id: "portal".to_owned(),
                version: "1.0.0".to_owned(),
                source_commit_oid: format!("sha1:{}", release_hex(portal_oid, 40)),
                source_tree_oid: format!("sha1:{}", release_hex(portal_oid, 40)),
                install_path: "/usr/lib/bullet/portal/1.0.0/index.html".to_owned(),
                bundle_digest: tagged_digest('d'),
                manifest_digest: tagged_digest('e'),
                size_bytes: 1,
            },
            sandbox: vec![SandboxSubject {
                id: "sandbox-s1".to_owned(),
                class: "s1".to_owned(),
                platform: "ubuntu-24-04-x86-64".to_owned(),
                install_path: "/usr/lib/bullet/sandboxes/s1/rootfs.img".to_owned(),
                image_digest: tagged_digest('f'),
                policy_digest: tagged_digest('1'),
                size_bytes: 1,
            }],
            jeryu: JeryuSubject {
                id: "jeryu".to_owned(),
                version: "1.0.0".to_owned(),
                tag: "v1.0.0".to_owned(),
                install_path: "/usr/lib/bullet/jeryu/1.0.0/jeryu".to_owned(),
                manifest_digest: tagged_digest('2'),
                binary_digest: tagged_digest('3'),
                api_schema_digest: tagged_digest('4'),
                capability_digest: tagged_digest('5'),
                sbom_digest: tagged_digest('6'),
                provenance_digest: tagged_digest('7'),
                signature_digest: tagged_digest('8'),
                size_bytes: 1,
            },
            release_signing: ReleaseSigningSubject {
                id: "release-signing".to_owned(),
                identity: release_identity,
                policy_digest: policy_digest.to_owned(),
                allowed_signers_digest: tagged_digest('9'),
                key_digest: tagged_digest('a'),
                policy_path: "/etc/bullet/release/allowed_signers".to_owned(),
                not_before_unix_ms: 1,
                not_after_unix_ms: 900,
            },
        },
    }
}

pub(crate) fn write_matching_family_anchor(policy_digest: &str) -> String {
    let hub = fixture_hub();
    if hub.exists() {
        fs::remove_dir_all(&hub).unwrap();
    }
    fs::create_dir_all(hub.join("scripts")).unwrap();
    fs::write(
        hub.join("Cargo.toml"),
        "[package]\nname='semantic-registry-fixture'\nversion='0.0.0'\n",
    )
    .unwrap();
    fs::write(hub.join("scripts/setup.sh"), "#!/bin/sh\nexit 1\n").unwrap();
    let bytes = family_lock::encode(&matching_family_lock(policy_digest)).unwrap();
    fs::write(hub.join("family.lock"), &bytes).unwrap();
    format!("blake3:{}", blake3::hash(&bytes).to_hex())
}

pub(crate) fn rewrite_family_anchor_policy(root: &Path, policy_digest: &str) {
    let bytes = family_lock::encode(&matching_family_lock(policy_digest)).unwrap();
    fs::write(fixture_family_lock_path(), &bytes).unwrap();
    let lock_digest = format!("blake3:{}", blake3::hash(&bytes).to_hex());
    rewrite_registry_inner(
        root,
        |records| {
            records
                .request
                .family_subject
                .family_lock_digest
                .clone_from(&lock_digest);
            records
                .closure_request
                .family_subject
                .family_lock_digest
                .clone_from(&lock_digest);
            records
                .receipt
                .family_subject
                .family_lock_digest
                .clone_from(&lock_digest);
            records
                .closure_receipt
                .family_subject
                .family_lock_digest
                .clone_from(&lock_digest);
            records.manifest.family_lock_digest.clone_from(&lock_digest);
        },
        false,
    );
}
