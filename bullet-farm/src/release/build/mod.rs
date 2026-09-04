//! Single-target release archive production for `x86_64-unknown-linux-gnu`.
//!
//! This component produces the byte subjects the committed verifier already
//! consumes: one `bullet-farm/`-rooted TAR+Zstandard archive, one CycloneDX
//! SBOM, one unsigned in-toto provenance statement, a re-read BLAKE3 checksum
//! manifest, and a non-circular canonical-JSON build manifest.
//!
//! It is deliberately *not* the five-archive release contract. It builds one of
//! five required targets, it signs nothing, and the checked-in `family.lock` is
//! still schema 2, so `release verify` refuses the result by design. Every
//! release gate stays blocked; the final report line says so.

mod archive;
mod cargo;
mod checksums;
mod license;
mod manifest;
mod portal;
mod sbom;
mod subject;
mod time;

#[cfg(all(test, target_os = "linux"))]
mod tests;

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::coord::CoordError;

pub(super) const BUILD_USAGE: &str = "usage: bullet-family release build --target x86_64-unknown-linux-gnu --out ABSOLUTE_ABSENT_PATH [--family-root ABSOLUTE_PATH] [--cache-dir ABSOLUTE_PATH] [--offline]";

/// The only target this component can honestly produce today.
pub(super) const SUPPORTED_TARGET: &str = "x86_64-unknown-linux-gnu";

/// Rust binaries copied into `bullet-farm/bin/` of the archive, as
/// `(member, cargo package, binary name)`.
const BINARIES: [(&str, &str, &str); 8] = [
    ("bullet-kernel", "bullet", "bullet"),
    ("bullet-kernel", "bullet-effects", "bullet-effects"),
    ("bullet-farm", "bullet-family", "bullet-family"),
    ("bullet-kernel", "bullet-farmd", "bullet-farmd"),
    ("bullet-git", "bullet-gitd", "bullet-gitd"),
    ("bullet-kernel", "bullet-mcpd", "bullet-mcpd"),
    ("bullet-kernel", "bullet-runner", "bullet-runner"),
    ("bullet-kernel", "bullet-verifier", "bullet-verifier"),
];

#[derive(Debug)]
pub(super) struct BuildArgs {
    pub(super) target: String,
    pub(super) out: PathBuf,
    pub(super) family_root: Option<PathBuf>,
    pub(super) cache_dir: Option<PathBuf>,
    pub(super) offline: bool,
}

/// One byte subject published beside the archive in the bundle directory.
#[derive(Clone, Debug, Eq, PartialEq)]
struct BundleFile {
    relative: String,
    size: u64,
    digest: String,
}

/// Everything the build resolved before it wrote a byte.
struct BuildPlan {
    family_root: PathBuf,
    out: PathBuf,
    scratch: PathBuf,
    cache: PathBuf,
    offline: bool,
    tag: String,
    signing_identity: String,
    lock_schema_version: String,
    subjects: Vec<subject::MemberSubject>,
    tools: subject::Toolchain,
    started_at: i64,
    builder_identity: String,
    invocation_id: String,
}

impl BuildPlan {
    fn member(&self, name: &str) -> Result<&subject::MemberSubject, CoordError> {
        self.subjects
            .iter()
            .find(|subject| subject.name == name)
            .ok_or_else(|| {
                invalid(format!(
                    "family member {name} is absent from repos.manifest.toml"
                ))
            })
    }

    fn stem(&self) -> String {
        format!("bullet-farm-{}-{SUPPORTED_TARGET}", self.tag)
    }
}

pub(super) fn run(args: &BuildArgs) -> Result<String, CoordError> {
    let plan = plan(args)?;
    let commands = &mut Vec::new();
    let lock = publish_family_lock(&plan)?;
    let portal = portal::build(&plan, commands)?;
    let binaries = cargo::build_binaries(&plan, &portal, commands)?;
    let archive = archive::write(&plan, &binaries)?;
    let sbom = sbom::write(&plan, &portal, commands)?;
    let provenance = manifest::write_provenance(&plan, &archive, commands)?;
    let checksums = checksums::write(
        &plan,
        &archive,
        &[lock.clone(), sbom_file(&sbom), provenance.clone()],
    )?;
    let checksums_relative = checksums.relative.clone();
    let record = manifest::write_manifest(
        &plan,
        &manifest::ManifestInput {
            lock: &lock,
            archive: &archive,
            sbom: &sbom,
            provenance: &provenance,
            checksums: &checksums,
            portal: &portal,
            commands,
        },
    )?;
    let extracted = archive::reread(&plan, &archive, &checksums)?;
    Ok(report(
        &plan,
        &archive,
        &sbom,
        &checksums_relative,
        &record,
        &extracted,
    ))
}

fn sbom_file(sbom: &sbom::SbomOutput) -> BundleFile {
    BundleFile {
        relative: sbom.relative.clone(),
        size: sbom.size,
        digest: sbom.digest.clone(),
    }
}

/// Copies the hub `family.lock` into the bundle at the exact path the frozen
/// release manifest schema binds it to.
fn publish_family_lock(plan: &BuildPlan) -> Result<BundleFile, CoordError> {
    let source = plan.member("bullet-farm")?.path.join("family.lock");
    let bytes = fs::read(&source).map_err(CoordError::io)?;
    if bytes.is_empty() || bytes.len() > 1024 * 1024 {
        return Err(invalid(
            "family.lock is empty or exceeds its admission limit",
        ));
    }
    write_new(&plan.out.join("family.lock"), &bytes)?;
    Ok(BundleFile {
        relative: "family.lock".to_owned(),
        size: bytes.len() as u64,
        digest: digest_bytes(&bytes),
    })
}

fn plan(args: &BuildArgs) -> Result<BuildPlan, CoordError> {
    if args.target != SUPPORTED_TARGET {
        return Err(CoordError::new(
            "UNSUPPORTED_RELEASE_TARGET",
            format!(
                "{} is not producible here; this component builds only {SUPPORTED_TARGET}. The V1 \
                 five-archive contract additionally requires aarch64-unknown-linux-gnu, \
                 x86_64-apple-darwin, aarch64-apple-darwin, and x86_64-pc-windows-msvc from their \
                 own native hosts",
                args.target
            ),
        ));
    }
    let family_root = match args.family_root.as_ref() {
        Some(root) => admitted_absolute_dir(root, "family root")?,
        None => {
            let current = std::env::current_dir().map_err(CoordError::io)?;
            crate::coord::discover_family_root(&current, None)?
        }
    };
    if !args.out.is_absolute() || args.out.file_name().is_none() {
        return Err(invalid(
            "release build --out must be an absolute path with a file name",
        ));
    }
    let tools = subject::admit_toolchain()?;
    let subjects = subject::admit_family(&family_root, &tools)?;
    let lock = subject::read_lock(&family_root.join("bullet-farm").join("family.lock"))?;
    // Nothing is created until every subject, tool, and lock refusal has passed.
    let out = admitted_absent_output(&args.out)?;
    let scratch = out.join(".scratch");
    let cache = match args.cache_dir.as_ref() {
        Some(cache) => admitted_absolute_dir(cache, "build cache")?,
        None => scratch.join("cache"),
    };
    fs::create_dir_all(&scratch).map_err(CoordError::io)?;
    fs::create_dir_all(&cache).map_err(CoordError::io)?;
    fs::create_dir(out.join(SUPPORTED_TARGET)).map_err(CoordError::io)?;
    let started_at = time::now_unix_seconds()?;
    let builder_identity = builder_identity();
    let invocation_id = digest_bytes(
        format!(
            "{builder_identity}|{started_at}|{}|{}",
            out.display(),
            subjects
                .iter()
                .map(|subject| format!("{}={}", subject.name, subject.commit_oid))
                .collect::<Vec<_>>()
                .join(",")
        )
        .as_bytes(),
    );
    Ok(BuildPlan {
        family_root,
        out,
        scratch,
        cache,
        offline: args.offline,
        tag: lock.tag,
        signing_identity: lock.release_signing_identity,
        lock_schema_version: lock.schema_version,
        subjects,
        tools,
        started_at,
        builder_identity,
        invocation_id,
    })
}

/// Builder identity for the provenance record: this host and this account, read
/// from the kernel rather than from a spoofable environment variable alone.
fn builder_identity() -> String {
    let host = fs::read_to_string("/proc/sys/kernel/hostname")
        .map(|value| value.trim().to_owned())
        .unwrap_or_else(|_| "unknown-host".to_owned());
    let uid = fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status
                .lines()
                .find_map(|line| line.strip_prefix("Uid:").map(str::to_owned))
        })
        .and_then(|line| line.split_whitespace().next().map(str::to_owned))
        .unwrap_or_else(|| "unknown-uid".to_owned());
    let user = std::env::var("USER").unwrap_or_else(|_| "unknown-user".to_owned());
    format!("host={host} uid={uid} user={user}")
}

fn report(
    plan: &BuildPlan,
    archive: &archive::ArchiveOutput,
    sbom: &sbom::SbomOutput,
    checksums: &str,
    record: &manifest::ManifestOutput,
    extracted: &Path,
) -> String {
    format!(
        "release build produced one unsigned {SUPPORTED_TARGET} bundle at {out}\n  \
         archive        {archive_path} ({archive_bytes} bytes, {entries} entries, {archive_digest})\n  \
         sbom           {sbom_path} ({components} components = {cargo_components} cargo + {npm_components} npm, CycloneDX 1.6, every license admitted)\n  \
         checksums      {checksums} ({archive_entries} archive entries, generated and re-read)\n  \
         provenance     {SUPPORTED_TARGET}/{stem}.intoto.jsonl (UNSIGNED; sign with the command in SIGNING.txt)\n  \
         build manifest release-build-manifest.json ({manifest_digest}, non-circular)\n  \
         re-read        archive re-extracted through the committed extractor at {extracted}\n  \
         operator step  {signing}\n\
         BLOCKED: this is 1 of the 5 required archives, it carries no signature, and it binds a \
         schema-{lock} family.lock. release.package-matrix, release.checksums, release.sbom, \
         release.manifest-non-circular, and release.provenance all remain BLOCKED; \
         `bullet-family release verify` refuses this bundle by design.",
        out = plan.out.display(),
        archive_path = archive.relative,
        archive_bytes = archive.size,
        entries = archive.entries.len(),
        archive_digest = archive.digest,
        sbom_path = sbom.relative,
        components = sbom.component_count,
        cargo_components = sbom.cargo_components,
        npm_components = sbom.npm_components,
        stem = plan.stem(),
        archive_entries = archive.entries.len(),
        manifest_digest = record.digest,
        extracted = extracted.display(),
        signing = record.signing_command,
        lock = plan.lock_schema_version,
    )
}

fn admitted_absent_output(path: &Path) -> Result<PathBuf, CoordError> {
    let parent = path
        .parent()
        .ok_or_else(|| invalid("release build --out has no parent directory"))?;
    let parent = admitted_absolute_dir(parent, "release build output parent")?;
    match fs::symlink_metadata(path) {
        Ok(_) => Err(CoordError::new(
            "RELEASE_OUTPUT_EXISTS",
            format!(
                "{} already exists; release build never replaces an existing output",
                path.display()
            ),
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let out = parent.join(path.file_name().expect("checked file name"));
            fs::create_dir(&out).map_err(CoordError::io)?;
            Ok(out)
        }
        Err(error) => Err(CoordError::io(error)),
    }
}

pub(super) fn admitted_absolute_dir(path: &Path, label: &str) -> Result<PathBuf, CoordError> {
    if !path.is_absolute() {
        return Err(invalid(format!("{label} path must be absolute")));
    }
    let metadata = fs::symlink_metadata(path).map_err(CoordError::io)?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
        return Err(invalid(format!(
            "{label} must be a regular non-symlink directory"
        )));
    }
    let canonical = path.canonicalize().map_err(CoordError::io)?;
    if canonical != path {
        return Err(invalid(format!("{label} path must already be canonical")));
    }
    Ok(canonical)
}

pub(super) fn digest_bytes(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

pub(super) fn digest_path(path: &Path) -> Result<(String, u64), CoordError> {
    use std::io::Read;

    let mut input = fs::File::open(path).map_err(CoordError::io)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0_u8; 256 * 1024];
    let mut size = 0_u64;
    loop {
        let count = input.read(&mut buffer).map_err(CoordError::io)?;
        if count == 0 {
            return Ok((format!("blake3:{}", hasher.finalize().to_hex()), size));
        }
        hasher.update(&buffer[..count]);
        size = size
            .checked_add(count as u64)
            .ok_or_else(|| invalid("release input byte count overflowed"))?;
    }
}

/// Writes one bundle byte subject exactly once; an existing path is a refusal.
pub(super) fn write_new(path: &Path, bytes: &[u8]) -> Result<(), CoordError> {
    use std::{fs::OpenOptions, io::Write};

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| match error.kind() {
            std::io::ErrorKind::AlreadyExists => CoordError::new(
                "RELEASE_OUTPUT_EXISTS",
                format!("{} already exists", path.display()),
            ),
            _ => CoordError::io(error),
        })?;
    file.write_all(bytes).map_err(CoordError::io)?;
    file.sync_all().map_err(CoordError::io)
}

pub(super) fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RELEASE_BUILD_INPUT", reason)
}

pub(super) fn failed(reason: impl Into<String>) -> CoordError {
    CoordError::new("RELEASE_BUILD_FAILED", reason)
}
