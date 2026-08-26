//! Deterministic family lock generation from locally verified signed tags.

mod git;
mod schema;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Deserialize;

pub use self::schema::{
    ExternalSubjectManifest, ExternalSubjects, FamilyLock, JeryuSubject, LOCK_SCHEMA_VERSION,
    LockedFile, LockedHub, LockedMember, PortalSubject, ProviderSubject, ReleaseSigningSubject,
    SandboxSubject, ToolchainSubject, encode, load, parse,
};
use self::{
    git::{
        digest_dependency_lockfiles, digest_generated_artifacts, digest_tagged_tree, tag_commit,
        tag_tree, verify_tag,
    },
    schema::{validate_jeryu_source, validate_tag},
};
use crate::coord::CoordError;

pub(crate) fn run_admitted_git_after_verify(
    repo: &Path,
    args: &[&str],
    limits: crate::process::Limits,
    label: &str,
    after_verify: impl FnOnce() -> Result<(), CoordError>,
) -> Result<std::process::Output, CoordError> {
    git::run_admitted_git_after_verify(repo, args, limits, label, after_verify)
}

const LOCK_FILE: &str = "family.lock";
const ALLOWED_SIGNERS: &str = "release/allowed_signers";
const SCHEMA_PREFIX: &str = "crates/bullet-wire";
const MAX_ALLOWED_SIGNERS_BYTES: u64 = 1024 * 1024;
const CANONICAL_REPOSITORIES: [&str; 4] = [
    "bullet-farm",
    "bullet-git",
    "bullet-kernel",
    "bullet-portal",
];

enum LockAction {
    Generate { tag: String, subjects: PathBuf },
    Verify { tag: String },
}

#[derive(Deserialize)]
struct Manifest {
    family: String,
    required_repos: Vec<String>,
    #[serde(default)]
    repo: Vec<ManifestRepo>,
}

#[derive(Deserialize)]
struct ManifestRepo {
    name: String,
    path: PathBuf,
    #[serde(default)]
    jeryu_url: Option<String>,
    #[serde(default)]
    jeryu_slug: Option<String>,
}

pub fn run(root: &Path, args: &[String]) -> Result<String, CoordError> {
    let action = parse_args(args)?;
    let family_root = resolve_family_root(root)?;
    let path = family_root.join("bullet-farm").join(LOCK_FILE);
    match action {
        LockAction::Generate { tag, subjects } => {
            let subjects = ExternalSubjectManifest::load(&subjects)?.external;
            let bytes = render(&family_root, &tag, "HEAD", subjects)?;
            atomic_write(&path, &bytes)?;
            Ok(format!("generated {} for {tag}", path.display()))
        }
        LockAction::Verify { tag } => {
            let current = fs::read(&path).map_err(CoordError::io)?;
            let lock = parse(&current)?;
            if lock.tag != tag {
                return Err(CoordError::new(
                    "FAMILY_LOCK_TAG_MISMATCH",
                    format!("family.lock binds {}, not requested {tag}", lock.tag),
                ));
            }
            let repos = verification_repos(&family_root, &lock)?;
            let farm = repos.get("bullet-farm").ok_or_else(|| {
                CoordError::new(
                    "FAMILY_MEMBER_MISSING",
                    "manifest has no bullet-farm member",
                )
            })?;
            let allowed_signers = farm.join(ALLOWED_SIGNERS);
            verify_hub_checkout(&lock, farm, &allowed_signers)?;
            for member in &lock.member {
                let repo = repos.get(&member.name).ok_or_else(|| {
                    CoordError::new(
                        "FAMILY_MEMBER_MISSING",
                        format!("manifest has no {} member", member.name),
                    )
                })?;
                verify_locked_checkout(member, repo, &allowed_signers)?;
            }
            Ok(format!("{} matches {tag}", path.display()))
        }
    }
}

fn resolve_family_root(root: &Path) -> Result<PathBuf, CoordError> {
    if root.file_name().and_then(|name| name.to_str()) == Some("bullet-farm")
        && let Some(parent) = root
            .parent()
            .filter(|path| path.join("repos.manifest.toml").is_file())
    {
        return Ok(parent.to_path_buf());
    }
    if root.join("repos.manifest.toml").is_file() {
        return Ok(root.to_path_buf());
    }
    Err(CoordError::new(
        "FAMILY_MANIFEST_MISSING",
        format!(
            "{} is neither a split-family root nor its bullet-farm checkout",
            root.display()
        ),
    ))
}

fn parse_args(args: &[String]) -> Result<LockAction, CoordError> {
    match args {
        [action, flag, tag] if action == "verify" && flag == "--tag" => {
            validate_cli_tag(tag)?;
            Ok(LockAction::Verify { tag: tag.clone() })
        }
        [action, tag_flag, tag, subjects_flag, subjects]
            if action == "generate" && tag_flag == "--tag" && subjects_flag == "--subjects" =>
        {
            validate_cli_tag(tag)?;
            Ok(LockAction::Generate {
                tag: tag.clone(),
                subjects: PathBuf::from(subjects),
            })
        }
        _ => Err(CoordError::new("USAGE", lock_usage())),
    }
}

fn validate_cli_tag(tag: &str) -> Result<(), CoordError> {
    validate_tag(tag).map_err(|_| CoordError::new("INVALID_RELEASE_TAG", "invalid release tag"))
}

fn lock_usage() -> &'static str {
    "usage: bullet-family [--root PATH] lock generate --tag <version> --subjects <absolute-path> | lock verify --tag <version>"
}

fn render(
    root: &Path,
    tag: &str,
    hub_revision: &str,
    external: ExternalSubjects,
) -> Result<Vec<u8>, CoordError> {
    let manifest_text =
        fs::read_to_string(root.join("repos.manifest.toml")).map_err(CoordError::io)?;
    let manifest: Manifest = toml::from_str(&manifest_text)
        .map_err(|error| CoordError::new("INVALID_FAMILY_MANIFEST", error.to_string()))?;
    let repos = indexed_repos(root, &manifest)?;
    let sources = authenticated_sources(&manifest)?;
    let farm = repos.get("bullet-farm").ok_or_else(|| {
        CoordError::new(
            "FAMILY_MEMBER_MISSING",
            "manifest has no bullet-farm member",
        )
    })?;
    let schema_bundle_hash = digest_tagged_tree(farm, hub_revision, SCHEMA_PREFIX)?;
    let allowed_signers = farm.join(ALLOWED_SIGNERS);
    if !allowed_signers.is_file() {
        return Err(CoordError::new(
            "ALLOWED_SIGNERS_MISSING",
            format!("{} does not exist", allowed_signers.display()),
        ));
    }
    verify_allowed_signers_subject(&allowed_signers, &external)?;
    let mut members = Vec::with_capacity(manifest.required_repos.len().saturating_sub(1));
    let mut member_names = manifest
        .required_repos
        .iter()
        .filter(|name| name.as_str() != "bullet-farm")
        .collect::<Vec<_>>();
    member_names.sort_unstable();
    for name in member_names {
        let repo = repos.get(name).ok_or_else(|| {
            CoordError::new(
                "FAMILY_MEMBER_MISSING",
                format!("manifest has no {name} member"),
            )
        })?;
        let commit_oid = tag_commit(repo, tag)?;
        let tree_oid = tag_tree(repo, tag)?;
        let release_signing_identity = verify_tag(repo, tag, &allowed_signers)?;
        let lockfile = digest_dependency_lockfiles(repo, tag, name)?;
        let artifact = digest_generated_artifacts(repo, tag)?;
        let source = sources.get(name).cloned().flatten();
        members.push(LockedMember {
            name: name.clone(),
            jeryu_url: source.as_ref().map(|source| source.0.clone()),
            jeryu_slug: source.map(|source| source.1),
            tag: tag.to_owned(),
            commit_oid,
            tree_oid,
            release_signing_identity,
            lockfile,
            artifact,
        });
    }
    let lock = FamilyLock {
        schema_version: LOCK_SCHEMA_VERSION.to_owned(),
        family: manifest.family,
        tag: tag.to_owned(),
        schema_bundle_hash,
        hub: LockedHub {
            name: "bullet-farm".to_owned(),
            tag: tag.to_owned(),
            release_signing_identity: external.release_signing.identity.clone(),
        },
        member: members,
        external,
    };
    encode(&lock)
}

fn authenticated_sources(
    manifest: &Manifest,
) -> Result<BTreeMap<String, Option<(String, String)>>, CoordError> {
    let mut sources = BTreeMap::new();
    for repo in &manifest.repo {
        let source = match (&repo.jeryu_url, &repo.jeryu_slug) {
            (Some(url), Some(slug)) => {
                validate_jeryu_source(url, slug).map_err(|error| {
                    CoordError::new("INVALID_SOURCE_METADATA", format!("{}: {error}", repo.name))
                })?;
                if slug.rsplit('/').next() != Some(repo.name.as_str()) {
                    return Err(CoordError::new(
                        "INVALID_SOURCE_METADATA",
                        format!("{} Jeryu slug does not bind its member name", repo.name),
                    ));
                }
                Some((url.clone(), slug.clone()))
            }
            (None, None) if repo.name == "bullet-farm" => None,
            (None, _) => {
                return Err(CoordError::new(
                    "SOURCE_METADATA_UNAVAILABLE",
                    format!(
                        "{} lacks an authenticated jeryu_url; restore source authentication, publish signed tags, and update the manifest before generating a lock",
                        repo.name
                    ),
                ));
            }
            (Some(_), None) => {
                return Err(CoordError::new(
                    "INVALID_SOURCE_METADATA",
                    format!("{} must provide both jeryu_url and jeryu_slug", repo.name),
                ));
            }
        };
        sources.insert(repo.name.clone(), source);
    }
    Ok(sources)
}

pub fn verify_locked_checkout(
    member: &LockedMember,
    repo: &Path,
    allowed_signers: &Path,
) -> Result<(), CoordError> {
    git::verify_locked_checkout(member, repo, allowed_signers)
}

pub fn verify_hub_checkout(
    lock: &FamilyLock,
    repo: &Path,
    allowed_signers: &Path,
) -> Result<String, CoordError> {
    verify_allowed_signers_subject(allowed_signers, &lock.external)?;
    let tagged_commit = tag_commit(repo, &lock.tag)?;
    let tagged_tree = tag_tree(repo, &lock.tag)?;
    if git::head_commit(repo)? != tagged_commit || git::head_tree(repo)? != tagged_tree {
        return Err(CoordError::new(
            "HUB_TAG_SUBJECT_MISMATCH",
            "the signed hub tag does not resolve to the invoking hub HEAD/tree",
        ));
    }
    let schema_bundle_hash = digest_tagged_tree(repo, &lock.tag, SCHEMA_PREFIX)?;
    if schema_bundle_hash != lock.schema_bundle_hash {
        return Err(CoordError::new(
            "HUB_SCHEMA_MISMATCH",
            "the signed hub tag does not contain the locked schema bundle",
        ));
    }
    let tagged_lock = parse(&git::verified_blob(repo, &lock.tag, LOCK_FILE)?)?;
    if &tagged_lock != lock {
        return Err(CoordError::new(
            "HUB_LOCK_MISMATCH",
            "the invoking family.lock is not the exact lock in the signed hub tag",
        ));
    }
    let signer = verify_tag(repo, &lock.tag, allowed_signers)?;
    if signer != lock.hub.release_signing_identity {
        return Err(CoordError::new(
            "HUB_SIGNER_MISMATCH",
            "the signed Hub tag does not match the locked release-signing identity",
        ));
    }
    Ok(signer)
}

pub(crate) fn checkout_subject(repo: &Path) -> Result<(String, String), CoordError> {
    Ok((git::head_commit(repo)?, git::head_tree(repo)?))
}

fn verification_repos(
    root: &Path,
    lock: &FamilyLock,
) -> Result<BTreeMap<String, PathBuf>, CoordError> {
    let manifest_text =
        fs::read_to_string(root.join("repos.manifest.toml")).map_err(CoordError::io)?;
    let manifest: Manifest = toml::from_str(&manifest_text)
        .map_err(|error| CoordError::new("INVALID_FAMILY_MANIFEST", error.to_string()))?;
    if manifest.family != "bullet-farm" {
        return Err(CoordError::new(
            "INVALID_FAMILY_MANIFEST",
            "family manifest does not describe bullet-farm",
        ));
    }
    lock.validate_required_members(&manifest.required_repos)?;
    let mut repos = BTreeMap::new();
    repos.insert("bullet-farm".to_owned(), root.join("bullet-farm"));
    for member in &lock.member {
        repos.insert(member.name.clone(), root.join(&member.name));
    }
    Ok(repos)
}

fn indexed_repos(
    root: &Path,
    manifest: &Manifest,
) -> Result<BTreeMap<String, PathBuf>, CoordError> {
    let required: BTreeSet<_> = manifest.required_repos.iter().cloned().collect();
    if required.len() != manifest.required_repos.len() {
        return Err(CoordError::new(
            "DUPLICATE_FAMILY_MEMBER",
            "required_repos contains duplicates",
        ));
    }
    let canonical = CANONICAL_REPOSITORIES
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if required != canonical {
        return Err(CoordError::new(
            "FAMILY_MEMBER_SET_MISMATCH",
            "schema-3 generation requires exactly the four canonical Bullet repositories",
        ));
    }
    let mut repos = BTreeMap::new();
    for entry in &manifest.repo {
        crate::coord::validate_repo_name(&entry.name)?;
        if !entry.path.is_absolute()
            || entry.path.file_name().and_then(|name| name.to_str()) != Some(entry.name.as_str())
        {
            return Err(CoordError::new(
                "INVALID_MEMBER_PATH",
                format!(
                    "manifest path for {} must be absolute and end with its repository name",
                    entry.name
                ),
            ));
        }
        if repos
            .insert(entry.name.clone(), root.join(&entry.name))
            .is_some()
        {
            return Err(CoordError::new(
                "DUPLICATE_FAMILY_MEMBER",
                format!("manifest repeats {}", entry.name),
            ));
        }
    }
    if repos.keys().cloned().collect::<BTreeSet<_>>() != required {
        return Err(CoordError::new(
            "FAMILY_MEMBER_SET_MISMATCH",
            "repo entries must exactly match required_repos",
        ));
    }
    Ok(repos)
}

fn verify_allowed_signers_subject(
    path: &Path,
    external: &ExternalSubjects,
) -> Result<(), CoordError> {
    let metadata = fs::symlink_metadata(path).map_err(CoordError::io)?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(CoordError::new(
            "INVALID_ALLOWED_SIGNERS",
            "release/allowed_signers must be a regular, non-symlink file",
        ));
    }
    if metadata.len() > MAX_ALLOWED_SIGNERS_BYTES {
        return Err(CoordError::new(
            "INVALID_ALLOWED_SIGNERS",
            "release/allowed_signers exceeds the 1 MiB admission limit",
        ));
    }
    let bytes = fs::read(path).map_err(CoordError::io)?;
    let digest = format!("blake3:{}", blake3::hash(&bytes).to_hex());
    if digest != external.release_signing.allowed_signers_digest {
        return Err(CoordError::new(
            "ALLOWED_SIGNERS_SUBJECT_MISMATCH",
            "release/allowed_signers does not match the locked external subject",
        ));
    }
    Ok(())
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), CoordError> {
    let parent = path
        .parent()
        .ok_or_else(|| CoordError::new("INVALID_LOCK_PATH", "family lock has no parent"))?;
    let temporary = parent.join(format!(".family.lock.{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(CoordError::io)?;
    if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        let _ = fs::remove_file(&temporary);
        return Err(CoordError::io(error));
    }
    fs::rename(&temporary, path).map_err(CoordError::io)?;
    let directory = fs::File::open(parent).map_err(CoordError::io)?;
    directory.sync_all().map_err(CoordError::io)
}
