use std::{
    fmt::Write as _,
    fs,
    io::{Cursor, Write as _},
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

use bullet_family::{
    family_lock::{
        ExternalSubjects, FamilyLock, JeryuSubject, LockedFile, LockedHub, LockedMember,
        PortalSubject, ProviderSubject, ReleaseSigningSubject, SandboxSubject, ToolchainSubject,
    },
    release::{RELEASE_MANIFEST_SCHEMA_VERSION, ReleaseManifest},
};

const PRINCIPAL: &str = "fixture@bullet.invalid";
const TAG: &str = "v0.1.0-fixture.1";
static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const TARGETS: [&str; 5] = [
    "aarch64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc",
    "x86_64-unknown-linux-gnu",
];

struct Fixture {
    root: PathBuf,
    bundle: PathBuf,
    allowed_signers: PathBuf,
    key: PathBuf,
    signing_identity: String,
    allowed_signer_entry: String,
}

struct TestSigner {
    key: PathBuf,
    identity: String,
    allowed_signer_entry: String,
}

impl Fixture {
    fn new() -> Self {
        let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "bullet-release-bundle-{}-{sequence}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("remove stale fixture");
        }
        let bundle = root.join("bundle");
        fs::create_dir_all(bundle.join("packages")).expect("fixture directories");
        let signer = generate_signer(&root, "release-key", PRINCIPAL);
        let allowed_signers = root.join("allowed_signers");
        fs::write(&allowed_signers, &signer.allowed_signer_entry).expect("allowed signers");
        let allowed_signers_digest = digest(signer.allowed_signer_entry.as_bytes());

        let family_lock = bundle.join("family.lock");
        fs::write(
            &family_lock,
            family_lock_text(&signer.identity, &allowed_signers_digest),
        )
        .expect("family lock");
        let mut manifest = format!(
            concat!(
                "release_manifest_schema_version = \"{}\"\n",
                "family_lock_schema_version = \"3\"\n",
                "family = \"bullet-farm\"\n",
                "tag = \"{}\"\n",
                "hub_commit_oid = \"sha1:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"\n",
                "hub_tree_oid = \"sha1:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\"\n",
                "release_signing_identity = \"{}\"\n",
            ),
            RELEASE_MANIFEST_SCHEMA_VERSION, TAG, signer.identity,
        );
        append_file_table(&mut manifest, "family_lock", &bundle, "family.lock");

        for target in TARGETS {
            let archive_extension = if target == "x86_64-pc-windows-msvc" {
                "zip"
            } else {
                "tar.zst"
            };
            let archive = format!("packages/bullet-farm-{target}.{archive_extension}");
            let checksums = format!("packages/bullet-farm-{target}.checksums.json");
            let cyclonedx_sbom = format!("packages/bullet-farm-{target}.cdx.json");
            let spdx_sbom = format!("packages/bullet-farm-{target}.spdx.json");
            let provenance = format!("packages/bullet-farm-{target}.intoto.jsonl");
            for (path, bytes) in [
                (archive.clone(), archive_bytes(target)),
                (
                    checksums.clone(),
                    format!(
                        "{{\"install_file\":[{{\"digest\":\"blake3:{}\",\"path\":\"bin/bullet-family\",\"size\":1}}],\"target\":\"{target}\"}}\n",
                        "0".repeat(64)
                    )
                    .into_bytes(),
                ),
                (
                    cyclonedx_sbom.clone(),
                    format!("{{\"bomFormat\":\"CycloneDX\",\"target\":\"{target}\"}}\n")
                        .into_bytes(),
                ),
                (
                    spdx_sbom.clone(),
                    format!("{{\"spdxVersion\":\"SPDX-2.3\",\"target\":\"{target}\"}}\n")
                        .into_bytes(),
                ),
                (
                    provenance.clone(),
                    format!(
                        "{{\"_type\":\"https://in-toto.io/Statement/v1\",\"target\":\"{target}\"}}\n"
                    )
                    .into_bytes(),
                ),
            ] {
                fs::write(bundle.join(&path), bytes).expect("release payload");
                sign(&root, &signer.key, &bundle.join(&path));
            }
            writeln!(manifest, "[[package]]").unwrap();
            writeln!(manifest, "target = {target:?}").unwrap();
            append_signed_table(&mut manifest, "package.archive", &bundle, &archive);
            append_signed_table(&mut manifest, "package.checksums", &bundle, &checksums);
            append_signed_table(
                &mut manifest,
                "package.cyclonedx_sbom",
                &bundle,
                &cyclonedx_sbom,
            );
            append_signed_table(&mut manifest, "package.spdx_sbom", &bundle, &spdx_sbom);
            append_signed_table(&mut manifest, "package.provenance", &bundle, &provenance);
        }
        let manifest_path = bundle.join("release-manifest.toml");
        fs::write(&manifest_path, manifest).expect("release manifest");
        sign(&root, &signer.key, &manifest_path);
        Self {
            root,
            bundle,
            allowed_signers,
            key: signer.key,
            signing_identity: signer.identity,
            allowed_signer_entry: signer.allowed_signer_entry,
        }
    }

    fn verify(&self) -> Result<String, bullet_family::coord::CoordError> {
        bullet_family::cli::run(
            [
                "bullet-family".into(),
                "release".into(),
                "verify".into(),
                "--bundle".into(),
                self.bundle.to_str().expect("UTF-8 path").into(),
                "--allowed-signers".into(),
                self.allowed_signers.to_str().expect("UTF-8 path").into(),
            ],
            Ok(self.bundle.clone()),
        )
    }

    fn extract(
        &self,
        target: &str,
        destination: &Path,
    ) -> Result<String, bullet_family::coord::CoordError> {
        bullet_family::cli::run(
            [
                "bullet-family".into(),
                "release".into(),
                "extract".into(),
                "--bundle".into(),
                self.bundle.to_str().expect("UTF-8 path").into(),
                "--allowed-signers".into(),
                self.allowed_signers.to_str().expect("UTF-8 path").into(),
                "--target".into(),
                target.into(),
                "--destination".into(),
                destination.to_str().expect("UTF-8 path").into(),
            ],
            Ok(self.bundle.clone()),
        )
    }

    fn resign_manifest(&self) {
        let manifest = self.bundle.join("release-manifest.toml");
        fs::remove_file(self.bundle.join("release-manifest.toml.sig"))
            .expect("remove old signature");
        sign(&self.root, &self.key, &manifest);
    }

    fn replace_family_lock(&self, lock: String) {
        let lock_path = self.bundle.join("family.lock");
        fs::write(&lock_path, lock).expect("replace family lock");
        let lock_bytes = fs::read(&lock_path).expect("replacement family lock");
        let manifest_path = self.bundle.join("release-manifest.toml");
        let manifest_bytes = fs::read(&manifest_path).expect("release manifest");
        let mut manifest = ReleaseManifest::parse(&manifest_bytes).expect("valid release manifest");
        manifest.family_lock.size = lock_bytes.len() as u64;
        manifest.family_lock.digest = digest(&lock_bytes);
        fs::write(
            &manifest_path,
            toml::to_string(&manifest).expect("encode rebound release manifest"),
        )
        .expect("replace rebound release manifest");
        self.resign_manifest();
    }

    fn replace_manifest_signer(&self, signer: &TestSigner) {
        let manifest_path = self.bundle.join("release-manifest.toml");
        let manifest_bytes = fs::read(&manifest_path).expect("release manifest");
        let mut manifest = ReleaseManifest::parse(&manifest_bytes).expect("valid release manifest");
        manifest.release_signing_identity = signer.identity.clone();
        fs::write(
            &manifest_path,
            toml::to_string(&manifest).expect("encode rebound release manifest"),
        )
        .expect("replace rebound release manifest");
        fs::remove_file(self.bundle.join("release-manifest.toml.sig"))
            .expect("remove old signature");
        sign(&self.root, &signer.key, &manifest_path);
    }

    fn lock_text(&self, identity: &str) -> String {
        let allowed_signers = fs::read(&self.allowed_signers).expect("allowed signers bytes");
        family_lock_text(identity, &digest(&allowed_signers))
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).expect("remove fixture");
    }
}

#[test]
fn signed_five_platform_bundle_verifies_twice_without_mutation() {
    let fixture = Fixture::new();
    let before = snapshot(&fixture.root);
    let first = fixture.verify().expect("first verification");
    let second = fixture.verify().expect("second verification");
    assert_eq!(first, second);
    assert!(first.contains("5 packages"));
    assert_eq!(snapshot(&fixture.root), before);

    let lock = bullet_family::family_lock::parse(
        &fs::read(fixture.bundle.join("family.lock")).expect("family lock"),
    )
    .expect("valid family lock");
    assert_eq!(lock.external.release_signing.not_before_unix_ms, 1);
    assert_eq!(lock.external.release_signing.not_after_unix_ms, 2);
    // No trusted-time input exists at this component boundary: the deliberately historical
    // fixture interval is structurally valid but remains UNADJUDICATED, not enforced.
}

#[test]
fn signed_bundle_requires_the_exact_canonical_family_before_extracting() {
    let fixture = Fixture::new();
    let allowed_signers = fs::read(&fixture.allowed_signers).expect("allowed signers bytes");
    fixture.replace_family_lock(family_lock_text_for(
        &["bullet-git", "bullet-kernel"],
        &fixture.signing_identity,
        &digest(&allowed_signers),
    ));

    assert_eq!(fixture.verify().unwrap_err().code(), "INVALID_FAMILY_LOCK");
    let destination = fixture.root.join("incomplete-family-install");
    assert_eq!(
        fixture
            .extract("x86_64-unknown-linux-gnu", &destination)
            .unwrap_err()
            .code(),
        "INVALID_FAMILY_LOCK"
    );
    assert!(!destination.exists());
}

#[test]
fn lock_authorized_signer_cannot_be_substituted_by_another_allowed_key() {
    let fixture = Fixture::new();
    let alternate = generate_signer(
        &fixture.root,
        "alternate-release-key",
        "other@bullet.invalid",
    );
    let allowed_signers = format!(
        "{}{}",
        fixture.allowed_signer_entry, alternate.allowed_signer_entry
    );
    fs::write(&fixture.allowed_signers, &allowed_signers).expect("two-key allowed signers");
    fixture.replace_family_lock(family_lock_text(
        &fixture.signing_identity,
        &digest(allowed_signers.as_bytes()),
    ));
    fixture.replace_manifest_signer(&alternate);

    let error = fixture.verify().expect_err("alternate signer must fail");
    assert_eq!(error.code(), "INVALID_RELEASE_BUNDLE");
    assert_eq!(
        error.to_string(),
        "INVALID_RELEASE_BUNDLE: release manifest signer does not match the locked Hub signing identity"
    );
}

#[test]
fn allowed_signers_bytes_must_match_the_locked_policy_digest() {
    let fixture = Fixture::new();
    let mut altered = fs::read(&fixture.allowed_signers).expect("allowed signers bytes");
    altered.extend_from_slice(b"# semantically inert but different bytes\n");
    fs::write(&fixture.allowed_signers, altered).expect("alter allowed signers bytes");

    let error = fixture
        .verify()
        .expect_err("altered policy bytes must fail");
    assert_eq!(error.code(), "INVALID_RELEASE_BUNDLE");
    assert_eq!(
        error.to_string(),
        "INVALID_RELEASE_BUNDLE: admitted allowed-signers bytes do not match the locked external subject"
    );
}

#[test]
fn manifest_signer_must_match_the_identity_bound_by_the_lock() {
    let fixture = Fixture::new();
    let alternate = generate_signer(&fixture.root, "lock-release-key", "lock@bullet.invalid");
    let allowed_signers = format!(
        "{}{}",
        fixture.allowed_signer_entry, alternate.allowed_signer_entry
    );
    fs::write(&fixture.allowed_signers, allowed_signers).expect("two-key allowed signers");
    fixture.replace_family_lock(fixture.lock_text(&alternate.identity));

    let error = fixture
        .verify()
        .expect_err("manifest and lock signer mismatch must fail");
    assert_eq!(error.code(), "INVALID_RELEASE_BUNDLE");
    assert_eq!(
        error.to_string(),
        "INVALID_RELEASE_BUNDLE: release manifest signer does not match the locked Hub signing identity"
    );
}

#[test]
#[cfg(all(target_os = "linux", target_env = "gnu"))]
fn signed_archive_refuses_publication_without_containment() {
    let fixture = Fixture::new();
    let destination = fixture.root.join("installed");
    let error = fixture
        .extract("x86_64-unknown-linux-gnu", &destination)
        .expect_err("uncontained public extraction must refuse");
    assert_eq!(error.code(), "RELEASE_PUBLICATION_CONTAINMENT_UNAVAILABLE");
    assert!(
        error
            .to_string()
            .contains("without a different-UID or privileged containment backend")
    );
    assert!(
        !destination.exists(),
        "refusal must not create a destination"
    );
}

#[test]
#[cfg(all(target_os = "linux", target_env = "gnu"))]
fn extract_refuses_missing_signature_and_schema_two_before_mutation() {
    let fixture = Fixture::new();
    let destination = fixture.root.join("unsigned-install");
    fs::remove_file(fixture.bundle.join("release-manifest.toml.sig"))
        .expect("remove manifest signature");
    assert!(
        fixture
            .extract("x86_64-unknown-linux-gnu", &destination)
            .is_err()
    );
    assert!(!destination.exists());

    let fixture = Fixture::new();
    let destination = fixture.root.join("schema-two-install");
    let manifest = fixture.bundle.join("release-manifest.toml");
    let text = fs::read_to_string(&manifest).expect("manifest text");
    fs::write(
        &manifest,
        text.replacen(
            "family_lock_schema_version = \"3\"",
            "family_lock_schema_version = \"2\"",
            1,
        ),
    )
    .expect("schema-two manifest");
    fixture.resign_manifest();
    assert_eq!(
        fixture
            .extract("x86_64-unknown-linux-gnu", &destination)
            .unwrap_err()
            .code(),
        "UNSUPPORTED_SCHEMA"
    );
    assert!(!destination.exists());
}

#[test]
fn payload_and_symlink_substitution_fail_closed() {
    let fixture = Fixture::new();
    let payload = fixture
        .bundle
        .join("packages/bullet-farm-aarch64-apple-darwin.tar.zst");
    fs::write(&payload, "mutated\n").expect("mutate payload");
    assert_eq!(
        fixture.verify().unwrap_err().code(),
        "INVALID_RELEASE_BUNDLE"
    );

    for relative in [
        "packages/bullet-farm-aarch64-apple-darwin.checksums.json",
        "packages/bullet-farm-aarch64-apple-darwin.cdx.json",
        "packages/bullet-farm-aarch64-apple-darwin.spdx.json",
        "packages/bullet-farm-aarch64-apple-darwin.intoto.jsonl",
    ] {
        let fixture = Fixture::new();
        fs::write(fixture.bundle.join(relative), "mutated\n").expect("mutate signed subject");
        assert_eq!(
            fixture.verify().unwrap_err().code(),
            "INVALID_RELEASE_BUNDLE",
            "{relative} substitution must fail"
        );
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        fs::remove_file(&payload).expect("remove payload");
        symlink("../family.lock", &payload).expect("replace with symlink");
        assert_eq!(
            fixture.verify().unwrap_err().code(),
            "INVALID_RELEASE_BUNDLE"
        );
    }

    let fixture = Fixture::new();
    fs::write(
        fixture
            .bundle
            .join("packages/bullet-farm-aarch64-apple-darwin.spdx.json.sig"),
        "mutated signature\n",
    )
    .expect("mutate detached signature");
    assert_eq!(
        fixture.verify().unwrap_err().code(),
        "INVALID_RELEASE_BUNDLE"
    );
}

#[test]
fn schema_refuses_unknown_fields_missing_targets_and_wrong_lock_version() {
    let fixture = Fixture::new();
    let bytes = fs::read(fixture.bundle.join("release-manifest.toml")).expect("manifest");
    let text = String::from_utf8(bytes).expect("UTF-8 manifest");
    let legacy = text
        .replacen(
            &format!("release_manifest_schema_version = \"{RELEASE_MANIFEST_SCHEMA_VERSION}\""),
            "release_manifest_schema_version = \"1\"",
            1,
        )
        .replace("package.cyclonedx_sbom", "package.sbom");
    assert_eq!(
        ReleaseManifest::parse(legacy.as_bytes())
            .unwrap_err()
            .code(),
        "UNSUPPORTED_RELEASE_MANIFEST_SCHEMA"
    );

    let unknown = text.replacen(
        "family = \"bullet-farm\"",
        "family = \"bullet-farm\"\nunexpected = true",
        1,
    );
    assert_eq!(
        ReleaseManifest::parse(unknown.as_bytes())
            .unwrap_err()
            .code(),
        "INVALID_RELEASE_MANIFEST"
    );

    let parsed = ReleaseManifest::parse(text.as_bytes()).expect("valid manifest");

    let mut collision = parsed.clone();
    collision.package[0].spdx_sbom = collision.package[0].cyclonedx_sbom.clone();
    assert_eq!(
        ReleaseManifest::parse(
            toml::to_string(&collision)
                .expect("encode colliding manifest")
                .as_bytes()
        )
        .unwrap_err()
        .code(),
        "INVALID_RELEASE_MANIFEST"
    );

    let mut wrong_suffix = parsed.clone();
    wrong_suffix.package[0].checksums.file.path = "packages/wrong.checksums".to_owned();
    wrong_suffix.package[0].checksums.signature.path = "packages/wrong.checksums.sig".to_owned();
    assert_eq!(
        ReleaseManifest::parse(
            toml::to_string(&wrong_suffix)
                .expect("encode wrong-suffix manifest")
                .as_bytes()
        )
        .unwrap_err()
        .code(),
        "INVALID_RELEASE_MANIFEST"
    );

    let mut missing_target = parsed;
    missing_target.package.pop();
    let missing = toml::to_string(&missing_target).expect("encode missing target");
    assert_eq!(
        ReleaseManifest::parse(missing.as_bytes())
            .unwrap_err()
            .code(),
        "INVALID_RELEASE_MANIFEST"
    );

    let wrong_lock = text.replacen(
        "family_lock_schema_version = \"3\"",
        "family_lock_schema_version = \"2\"",
        1,
    );
    assert_eq!(
        ReleaseManifest::parse(wrong_lock.as_bytes())
            .unwrap_err()
            .code(),
        "UNSUPPORTED_SCHEMA"
    );
}

fn family_lock_text(identity: &str, allowed_signers_digest: &str) -> String {
    family_lock_text_for(
        &["bullet-git", "bullet-kernel", "bullet-portal"],
        identity,
        allowed_signers_digest,
    )
}

fn family_lock_text_for(members: &[&str], identity: &str, allowed_signers_digest: &str) -> String {
    let locked_members = members
        .iter()
        .map(|member| LockedMember {
            name: (*member).to_owned(),
            jeryu_url: Some(format!("https://jeryu.example/git/root/{member}.git")),
            jeryu_slug: Some(format!("root/{member}")),
            tag: TAG.to_owned(),
            commit_oid: lock_oid('d'),
            tree_oid: lock_oid('e'),
            release_signing_identity: identity.to_owned(),
            lockfile: vec![LockedFile {
                path: if *member == "bullet-portal" {
                    "package-lock.json".to_owned()
                } else {
                    "Cargo.lock".to_owned()
                },
                digest: lock_digest('f'),
            }],
            artifact: Vec::new(),
        })
        .collect();
    toml::to_string_pretty(&FamilyLock {
        schema_version: "3".to_owned(),
        family: "bullet-farm".to_owned(),
        tag: TAG.to_owned(),
        schema_bundle_hash: lock_digest('c'),
        hub: LockedHub {
            name: "bullet-farm".to_owned(),
            tag: TAG.to_owned(),
            release_signing_identity: identity.to_owned(),
        },
        member: locked_members,
        external: external_subjects(identity, allowed_signers_digest),
    })
    .unwrap()
}

fn external_subjects(identity: &str, allowed_signers_digest: &str) -> ExternalSubjects {
    ExternalSubjects {
        toolchain: fixture_toolchains(),
        provider: vec![ProviderSubject {
            id: "claude".into(),
            version: "1.0.0".into(),
            profile: "service".into(),
            install_path: "/usr/lib/bullet/providers/claude/1.0.0/claude".into(),
            binary_digest: lock_digest('3'),
            protocol_digest: lock_digest('4'),
            size_bytes: 1,
        }],
        portal: PortalSubject {
            id: "portal".into(),
            version: "1.0.0".into(),
            source_commit_oid: lock_oid('d'),
            source_tree_oid: lock_oid('e'),
            install_path: "/usr/lib/bullet/portal/1.0.0/index.html".into(),
            bundle_digest: lock_digest('5'),
            manifest_digest: lock_digest('6'),
            size_bytes: 1,
        },
        sandbox: vec![SandboxSubject {
            id: "sandbox-s1".into(),
            class: "s1".into(),
            platform: "ubuntu-24-04-x86-64".into(),
            install_path: "/usr/lib/bullet/sandboxes/s1/rootfs.img".into(),
            image_digest: lock_digest('7'),
            policy_digest: lock_digest('8'),
            size_bytes: 1,
        }],
        jeryu: JeryuSubject {
            id: "jeryu".into(),
            version: "1.0.0".into(),
            tag: "v1.0.0".into(),
            install_path: "/usr/lib/bullet/jeryu/1.0.0/jeryu".into(),
            manifest_digest: lock_digest('9'),
            binary_digest: lock_digest('a'),
            api_schema_digest: lock_digest('b'),
            capability_digest: lock_digest('c'),
            sbom_digest: lock_digest('d'),
            provenance_digest: lock_digest('e'),
            signature_digest: lock_digest('f'),
            size_bytes: 1,
        },
        release_signing: ReleaseSigningSubject {
            id: "release-signing".into(),
            identity: identity.to_owned(),
            policy_digest: lock_digest('1'),
            allowed_signers_digest: allowed_signers_digest.to_owned(),
            key_digest: lock_digest('3'),
            policy_path: "/etc/bullet/release/allowed_signers".into(),
            not_before_unix_ms: 1,
            not_after_unix_ms: 2,
        },
    }
}

fn fixture_toolchains() -> Vec<ToolchainSubject> {
    [
        ("cargo", "1.95.0", "rust/1.95.0/cargo", '1', '2'),
        ("node", "22.23.2", "node/22.23.2/node", '3', '4'),
        ("npm-cli", "10.9.8", "npm/10.9.8/npm-cli.js", '5', '6'),
    ]
    .into_iter()
    .map(
        |(id, version, relative, binary, manifest)| ToolchainSubject {
            id: id.into(),
            version: version.into(),
            install_path: format!("/usr/lib/bullet/toolchains/{relative}"),
            binary_digest: lock_digest(binary),
            manifest_path: format!("/usr/lib/bullet/toolchains/{relative}.manifest"),
            manifest_digest: lock_digest(manifest),
            size_bytes: 1,
        },
    )
    .collect()
}

fn generate_signer(root: &Path, label: &str, principal: &str) -> TestSigner {
    let key = root.join(label);
    command(
        root,
        "/usr/bin/ssh-keygen",
        &["-q", "-t", "ed25519", "-N", "", "-f", text(&key)],
    );
    let public_key = fs::read_to_string(key.with_extension("pub")).expect("public key");
    let fingerprint = fingerprint(root, &key.with_extension("pub"));
    TestSigner {
        key,
        identity: format!("{principal}|ed25519|{fingerprint}"),
        allowed_signer_entry: format!(
            "{principal} namespaces=\"bullet-farm-release\" {}\n",
            public_key.trim()
        ),
    }
}

fn lock_digest(byte: char) -> String {
    format!("blake3:{}", byte.to_string().repeat(64))
}

fn lock_oid(byte: char) -> String {
    format!("sha1:{}", byte.to_string().repeat(40))
}

fn append_signed_table(output: &mut String, table: &str, bundle: &Path, payload: &str) {
    append_file_table(output, &format!("{table}.file"), bundle, payload);
    append_file_table(
        output,
        &format!("{table}.signature"),
        bundle,
        &format!("{payload}.sig"),
    );
}

fn append_file_table(output: &mut String, table: &str, bundle: &Path, relative: &str) {
    let bytes = fs::read(bundle.join(relative)).expect("fixture file");
    writeln!(output, "[{table}]").unwrap();
    writeln!(output, "path = {relative:?}").unwrap();
    writeln!(output, "size = {}", bytes.len()).unwrap();
    writeln!(output, "digest = {:?}", digest(&bytes)).unwrap();
}

fn sign(cwd: &Path, key: &Path, payload: &Path) {
    command(
        cwd,
        "/usr/bin/ssh-keygen",
        &[
            "-Y",
            "sign",
            "-f",
            text(key),
            "-n",
            "bullet-farm-release",
            text(payload),
        ],
    );
}

fn fingerprint(cwd: &Path, public_key: &Path) -> String {
    let output = command_output(
        cwd,
        "/usr/bin/ssh-keygen",
        &["-lf", text(public_key), "-E", "sha256"],
    );
    String::from_utf8(output.stdout)
        .expect("UTF-8 fingerprint")
        .split_whitespace()
        .nth(1)
        .expect("fingerprint field")
        .to_owned()
}

fn digest(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn archive_bytes(target: &str) -> Vec<u8> {
    let suffix = if target == "x86_64-pc-windows-msvc" {
        ".exe"
    } else {
        ""
    };
    let executables = [
        "bullet",
        "bullet-effects",
        "bullet-family",
        "bullet-farmd",
        "bullet-gitd",
        "bullet-mcpd",
        "bullet-runner",
        "bullet-verifier",
    ];
    let payload = format!("fixture:{target}\n");
    if target == "x86_64-pc-windows-msvc" {
        let mut writer = ::zip::ZipWriter::new(Cursor::new(Vec::new()));
        let directory = ::zip::write::SimpleFileOptions::default()
            .compression_method(::zip::CompressionMethod::Stored)
            .unix_permissions(0o755);
        writer.add_directory("bullet-farm/", directory).unwrap();
        writer.add_directory("bullet-farm/bin/", directory).unwrap();
        for executable in executables {
            writer
                .start_file(format!("bullet-farm/bin/{executable}{suffix}"), directory)
                .unwrap();
            writer
                .write_all(if executable == "bullet-family" {
                    payload.as_bytes()
                } else {
                    b"fixture-tool\n"
                })
                .unwrap();
        }
        writer.finish().unwrap().into_inner()
    } else {
        let encoder = zstd::stream::write::Encoder::new(Vec::new(), 3).unwrap();
        let mut builder = tar::Builder::new(encoder);
        append_tar(&mut builder, "bullet-farm", &[], true);
        append_tar(&mut builder, "bullet-farm/bin", &[], true);
        for executable in executables {
            append_tar(
                &mut builder,
                &format!("bullet-farm/bin/{executable}{suffix}"),
                if executable == "bullet-family" {
                    payload.as_bytes()
                } else {
                    b"fixture-tool\n"
                },
                false,
            );
        }
        builder.finish().unwrap();
        builder.into_inner().unwrap().finish().unwrap()
    }
}

fn append_tar(
    builder: &mut tar::Builder<zstd::Encoder<'static, Vec<u8>>>,
    path: &str,
    bytes: &[u8],
    directory: bool,
) {
    let mut header = tar::Header::new_ustar();
    header.set_entry_type(if directory {
        tar::EntryType::Directory
    } else {
        tar::EntryType::Regular
    });
    header.set_path(path).unwrap();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    builder.append(&header, bytes).unwrap();
}

fn command(cwd: &Path, program: &str, args: &[&str]) {
    let output = command_output(cwd, program, args);
    assert!(
        output.status.success(),
        "{program} {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn command_output(cwd: &Path, program: &str, args: &[&str]) -> std::process::Output {
    Command::new(program)
        .current_dir(cwd)
        .args(args)
        .env_clear()
        .env("LC_ALL", "C")
        .output()
        .expect("run fixture command")
}

fn snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn visit(root: &Path, current: &Path, files: &mut Vec<(PathBuf, Vec<u8>)>) {
        for entry in fs::read_dir(current).expect("read fixture directory") {
            let entry = entry.expect("fixture entry");
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.push((
                    path.strip_prefix(root)
                        .expect("relative path")
                        .to_path_buf(),
                    fs::read(path).expect("fixture bytes"),
                ));
            }
        }
    }
    let mut files = Vec::new();
    visit(root, root, &mut files);
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

fn text(path: &Path) -> &str {
    path.to_str().expect("fixture path is UTF-8")
}
