use std::{
    ffi::OsString,
    fs,
    io::{self, Cursor},
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::Path,
};

use tar::{Builder, EntryType, Header};
use tempfile::TempDir;

use super::*;
use crate::coord::model::{ToolchainArtifactKindV1, ToolchainMemberV1, ToolchainRoleV1};

const COMMIT: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TREE: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const CACHE_ID: &str = "index.crates.io-1949cf8c6b5b557f";
const LOCK: &[u8] = b"version = 4\n";
const TOOLCHAIN: &[u8] = b"[toolchain]\nchannel = \"1.95.0\"\n";
const CRATE: &[u8] = b"crate archive";
const EXECUTABLE: &[u8] = b"reproducible executable";

struct Fixture {
    _root: TempDir,
    command: RecoveryBootstrapBuildVerifyCommand,
    source: Vec<u8>,
    cache: Vec<u8>,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let path = |name: &str| root.path().join(name);
        let command = RecoveryBootstrapBuildVerifyCommand {
            provenance: path("provenance.json"),
            source_archive: path("source.tar"),
            builder_contract: path("builder.json"),
            toolchain_contract: path("toolchain.json"),
            command_contract: path("command.json"),
            cache_manifest: path("cache.json"),
            cache_archive: path("cache.tar"),
            executable_run_1: path("run-1"),
            executable_run_2: path("run-2"),
            output: path("observation.json"),
        };
        let source = source_tar(LOCK, TOOLCHAIN);
        let provenance = provenance(&source, LOCK, TOOLCHAIN);
        let cache = cache_tar(cache_path(), EntryType::Regular, 1);
        let manifest = cache_manifest(&cache, digest(LOCK), packages());
        let (builder, toolchain, build_command) = contracts();
        write(&command.provenance, &provenance);
        write_raw(&command.source_archive, &source);
        write(&command.builder_contract, &builder);
        write(&command.toolchain_contract, &toolchain);
        write(&command.command_contract, &build_command);
        write(&command.cache_manifest, &manifest);
        write_raw(&command.cache_archive, &cache);
        write_raw(&command.executable_run_1, EXECUTABLE);
        write_raw(&command.executable_run_2, EXECUTABLE);
        Self {
            _root: root,
            command,
            source,
            cache,
        }
    }

    fn replace_raw(&self, path: &Path, bytes: &[u8]) {
        fs::remove_file(path).unwrap();
        write_raw(path, bytes);
    }

    fn replace_document(&self, path: &Path, value: &impl serde::Serialize) {
        fs::remove_file(path).unwrap();
        write(path, value);
    }

    fn replace_cache(&mut self, bytes: Vec<u8>) {
        self.replace_raw(&self.command.cache_archive, &bytes);
        let manifest = cache_manifest(&bytes, digest(LOCK), packages());
        self.replace_document(&self.command.cache_manifest, &manifest);
        self.cache = bytes;
    }
}

fn contracts() -> (
    RecoveryBootstrapBuilderContractV1,
    RecoveryBootstrapToolchainContractV1,
    RecoveryBootstrapCommandContractV1,
) {
    let builder = RecoveryBootstrapBuilderContractV1::from_rootfs(
        digest(b"rootfs"),
        6,
        digest(b"rootfs tree"),
    )
    .unwrap();
    let specs = [
        (ToolchainRoleV1::Git, "/toolchain/bin/git", 0o555),
        (ToolchainRoleV1::Cargo, "/toolchain/bin/cargo", 0o555),
        (ToolchainRoleV1::Rustc, "/toolchain/bin/rustc", 0o555),
        (ToolchainRoleV1::Linker, "/toolchain/bin/cc", 0o555),
        (ToolchainRoleV1::Sysroot, "/toolchain/rust/sysroot", 0o555),
        (
            ToolchainRoleV1::RuntimeLoader,
            "/toolchain/lib/ld-linux-x86-64.so.2",
            0o555,
        ),
        (
            ToolchainRoleV1::RuntimeLibrary,
            "/toolchain/lib/libc.so.6",
            0o444,
        ),
    ];
    let members = specs
        .into_iter()
        .enumerate()
        .map(|(index, (role, path, mode))| {
            ToolchainMemberV1::observed(
                role,
                path.to_owned(),
                if role == ToolchainRoleV1::Sysroot {
                    ToolchainArtifactKindV1::DirectoryTree
                } else {
                    ToolchainArtifactKindV1::RegularFile
                },
                mode,
                1,
                digest(&[index as u8]),
            )
        })
        .collect();
    let toolchain = RecoveryBootstrapToolchainContractV1::from_members(&builder, members).unwrap();
    let command = RecoveryBootstrapCommandContractV1::exact(&builder, &toolchain).unwrap();
    (builder, toolchain, command)
}

fn provenance(source: &[u8], lock: &[u8], toolchain: &[u8]) -> RecoveryBootstrapProvenanceV1 {
    RecoveryBootstrapProvenanceV1::from_observations(
        COMMIT.to_owned(),
        TREE.to_owned(),
        digest(source),
        digest(lock),
        vec![
            ("Cargo.lock".to_owned(), lock.len() as u64, digest(lock)),
            (
                "rust-toolchain.toml".to_owned(),
                toolchain.len() as u64,
                digest(toolchain),
            ),
        ],
        ("rustc 1.95.0".to_owned(), "cargo 1.95.0".to_owned()),
        (EXECUTABLE.len() as u64, digest(EXECUTABLE)),
    )
    .unwrap()
}

fn packages() -> Vec<(String, String, String, u64)> {
    vec![(
        "demo".to_owned(),
        "1.2.3".to_owned(),
        digest(CRATE),
        CRATE.len() as u64,
    )]
}

fn cache_manifest(
    bytes: &[u8],
    lock: String,
    packages: Vec<(String, String, String, u64)>,
) -> CargoOfflineCacheManifestV1 {
    let tree = cache_tree_sha256(CACHE_ID, &packages).unwrap();
    CargoOfflineCacheManifestV1::from_observations(
        lock,
        CACHE_ID.to_owned(),
        (digest(bytes), bytes.len() as u64),
        tree,
        packages,
    )
    .unwrap()
}

fn source_tar(lock: &[u8], toolchain: &[u8]) -> Vec<u8> {
    build_tar(
        COMMIT,
        &[
            ("Cargo.lock", 0o664, EntryType::Regular, lock),
            ("rust-toolchain.toml", 0o664, EntryType::Regular, toolchain),
        ],
    )
}

fn cache_path() -> &'static str {
    "registry/cache/index.crates.io-1949cf8c6b5b557f/demo-1.2.3.crate"
}

fn cache_tar(path: &str, kind: EntryType, copies: usize) -> Vec<u8> {
    let packages = packages();
    let tree = cache_tree_sha256(CACHE_ID, &packages).unwrap();
    let members = (0..copies)
        .map(|_| (path, 0o444, kind, CRATE))
        .collect::<Vec<_>>();
    build_tar(&tree, &members)
}

fn cache_tar_with_path_override(extension_type: EntryType) -> Vec<u8> {
    let tree = cache_tree_sha256(CACHE_ID, &packages()).unwrap();
    let mut builder = Builder::new(Vec::new());
    let global = pax_record("comment", &tree);
    append(
        &mut builder,
        "pax_global_header",
        0o444,
        EntryType::XGlobalHeader,
        &global,
    );
    let extension = if extension_type == EntryType::XHeader {
        pax_record("path", cache_path())
    } else {
        let mut path = cache_path().as_bytes().to_vec();
        path.push(0);
        path
    };
    append(
        &mut builder,
        "path-extension",
        0o444,
        extension_type,
        &extension,
    );
    append(
        &mut builder,
        "carrier.crate",
        0o444,
        EntryType::Regular,
        CRATE,
    );
    let mut bytes = builder.into_inner().unwrap();
    rewrite_header_path(&mut bytes[2048..2560], b"../hidden.crate");
    bytes
}

fn rewrite_header_path(header: &mut [u8], path: &[u8]) {
    assert!(path.len() <= 100);
    header[..100].fill(0);
    header[..path.len()].copy_from_slice(path);
    header[148..156].fill(b' ');
    let checksum = header.iter().map(|byte| u32::from(*byte)).sum::<u32>();
    let checksum = format!("{checksum:06o}\0 ");
    header[148..156].copy_from_slice(checksum.as_bytes());
}

fn build_tar(marker: &str, members: &[(&str, u32, EntryType, &[u8])]) -> Vec<u8> {
    let mut builder = Builder::new(Vec::new());
    let pax = pax_record("comment", marker);
    append(
        &mut builder,
        "pax_global_header",
        0o444,
        EntryType::XGlobalHeader,
        &pax,
    );
    for (path, mode, kind, content) in members {
        append(&mut builder, path, *mode, *kind, content);
    }
    builder.into_inner().unwrap()
}

fn append(builder: &mut Builder<Vec<u8>>, path: &str, mode: u32, kind: EntryType, bytes: &[u8]) {
    let mut header = Header::new_gnu();
    header.set_uid(0);
    header.set_gid(0);
    header.set_mode(mode);
    header.set_mtime(1);
    header.set_entry_type(kind);
    header.set_size(bytes.len() as u64);
    if kind == EntryType::Symlink {
        header.set_link_name("target").unwrap();
        header.set_size(0);
    }
    header.set_cksum();
    builder
        .append_data(&mut header, path, Cursor::new(bytes))
        .unwrap();
}

fn pax_record(key: &str, value: &str) -> Vec<u8> {
    let body = format!("{key}={value}\n");
    for digits in 1..20 {
        let length = digits + 1 + body.len();
        if length.to_string().len() == digits {
            return format!("{length} {body}").into_bytes();
        }
    }
    unreachable!()
}

fn write(path: &Path, value: &impl serde::Serialize) {
    crate::coord::sealed::write(path, value).unwrap();
}

fn write_raw(path: &Path, bytes: &[u8]) {
    crate::coord::sealed::write_raw(path, bytes, bytes.len() as u64).unwrap();
}

fn assert_refused(fixture: &Fixture) {
    assert!(verify_and_seal_bootstrap_build_observation(&fixture.command).is_err());
    assert!(!fixture.command.output.exists());
}

#[test]
fn seals_only_one_exact_contract_derived_observation() {
    let fixture = Fixture::new();
    let first = verify_and_seal_bootstrap_build_observation(&fixture.command).unwrap();
    let read: RecoveryBootstrapBuildObservationV1 =
        crate::coord::sealed::read(&fixture.command.output).unwrap();
    assert_eq!(first, read);
    let before = fs::read(&fixture.command.output).unwrap();
    assert!(verify_and_seal_bootstrap_build_observation(&fixture.command).is_err());
    assert_eq!(fs::read(&fixture.command.output).unwrap(), before);
}

#[test]
fn real_cli_is_root_independent_create_once_and_returns_the_persisted_id() {
    let fixture = Fixture::new();
    let command = &fixture.command;
    let mut argv = vec![
        OsString::from("bullet-family"),
        OsString::from("coord"),
        OsString::from("recovery-build-observe"),
    ];
    for (name, path) in [
        ("provenance", &command.provenance),
        ("source-archive", &command.source_archive),
        ("builder-contract", &command.builder_contract),
        ("toolchain-contract", &command.toolchain_contract),
        ("command-contract", &command.command_contract),
        ("cache-manifest", &command.cache_manifest),
        ("cache-archive", &command.cache_archive),
        ("executable-run-1", &command.executable_run_1),
        ("executable-run-2", &command.executable_run_2),
        ("output", &command.output),
    ] {
        argv.push(OsString::from(format!("--{name}")));
        argv.push(path.as_os_str().to_owned());
    }
    let absent_cwd = || io::Error::new(io::ErrorKind::NotFound, "test cwd absent");

    let result = crate::cli::execute(argv.clone(), Err(absent_cwd())).unwrap();
    assert_eq!(result.exit_code(), 0);
    let summary = bullet_wire::decode_unique_value(result.output().as_bytes()).unwrap();
    let persisted_bytes = fs::read(&command.output).unwrap();
    let persisted = bullet_wire::decode_unique_value(&persisted_bytes).unwrap();
    assert_eq!(summary["id"], persisted["observation_id"]);
    assert_eq!(summary["path"].as_str(), command.output.to_str());
    let metadata = fs::metadata(&command.output).unwrap();
    assert_eq!(metadata.permissions().mode() & 0o7777, 0o400);
    assert_eq!(metadata.nlink(), 1);
    assert!(!fixture._root.path().join(".bullet-family").exists());

    let error = crate::cli::execute(argv, Err(absent_cwd())).unwrap_err();
    assert_eq!(error.code(), "INVALID_RECOVERY_PRODUCTION");
    assert_eq!(fs::read(&command.output).unwrap(), persisted_bytes);
    assert!(!fixture._root.path().join(".bullet-family").exists());
}

#[test]
fn refuses_source_cache_and_executable_byte_substitution() {
    let fixture = Fixture::new();
    let mut changed = fixture.source.clone();
    changed[600] ^= 1;
    fixture.replace_raw(&fixture.command.source_archive, &changed);
    assert_refused(&fixture);

    let mut fixture = Fixture::new();
    let mut changed = fixture.cache.clone();
    changed[600] ^= 1;
    fixture.replace_cache(changed);
    assert_refused(&fixture);

    let fixture = Fixture::new();
    let changed = vec![b'x'; EXECUTABLE.len()];
    fixture.replace_raw(&fixture.command.executable_run_2, &changed);
    assert_refused(&fixture);
}

#[test]
fn refuses_source_inventory_and_cargo_lock_cross_binding() {
    let fixture = Fixture::new();
    let changed = source_tar(b"version = 3\n", TOOLCHAIN);
    let hostile = provenance(&changed, LOCK, TOOLCHAIN);
    fixture.replace_raw(&fixture.command.source_archive, &changed);
    fixture.replace_document(&fixture.command.provenance, &hostile);
    assert_refused(&fixture);

    let fixture = Fixture::new();
    let hostile = cache_manifest(&fixture.cache, digest(b"other lock"), packages());
    fixture.replace_document(&fixture.command.cache_manifest, &hostile);
    assert_refused(&fixture);
}

#[test]
fn refuses_duplicate_unlisted_and_link_cache_members() {
    for hostile in [
        cache_tar(cache_path(), EntryType::Regular, 2),
        cache_tar(
            "registry/cache/index.crates.io-1949cf8c6b5b557f/other-1.2.3.crate",
            EntryType::Regular,
            1,
        ),
        cache_tar(cache_path(), EntryType::Symlink, 1),
    ] {
        let mut fixture = Fixture::new();
        fixture.replace_cache(hostile);
        assert_refused(&fixture);
    }
}

#[test]
fn refuses_contract_and_path_substitution() {
    let fixture = Fixture::new();
    let hostile = RecoveryBootstrapBuilderContractV1::from_rootfs(
        digest(b"other rootfs"),
        12,
        digest(b"other tree"),
    )
    .unwrap();
    fixture.replace_document(&fixture.command.builder_contract, &hostile);
    assert_refused(&fixture);

    let mut fixture = Fixture::new();
    fixture.command.cache_archive = fixture.command.source_archive.clone();
    assert_refused(&fixture);
}

#[test]
fn refuses_preexisting_output_and_unsafe_output_parent() {
    let fixture = Fixture::new();
    write_raw(&fixture.command.output, b"occupied");
    assert!(verify_and_seal_bootstrap_build_observation(&fixture.command).is_err());
    assert_eq!(fs::read(&fixture.command.output).unwrap(), b"occupied");

    let mut fixture = Fixture::new();
    let unsafe_parent = fixture._root.path().join("unsafe");
    fs::create_dir(&unsafe_parent).unwrap();
    fs::set_permissions(&unsafe_parent, fs::Permissions::from_mode(0o755)).unwrap();
    fixture.command.output = unsafe_parent.join("observation.json");
    assert_refused(&fixture);
}

#[test]
fn refuses_safe_pax_path_over_a_traversal_carrier() {
    let mut fixture = Fixture::new();
    fixture.replace_cache(cache_tar_with_path_override(EntryType::XHeader));
    assert_refused(&fixture);
}

#[test]
fn refuses_safe_gnu_path_over_a_traversal_carrier() {
    let mut fixture = Fixture::new();
    fixture.replace_cache(cache_tar_with_path_override(EntryType::GNULongName));
    assert_refused(&fixture);
}
