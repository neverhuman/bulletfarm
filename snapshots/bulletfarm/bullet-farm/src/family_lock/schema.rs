//! Strict, reusable schema for an installable Bullet Farm family lock.

mod external;
#[cfg(test)]
mod tests;
mod validate;

use std::{collections::BTreeSet, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::coord::CoordError;

use self::validate::{
    invalid, validate_atom, validate_digest, validate_git_oid, validate_signing_identity,
};
pub(crate) use self::validate::{validate_jeryu_source, validate_repository_path, validate_tag};

pub use self::external::{
    ExternalSubjectManifest, ExternalSubjects, JeryuSubject, PortalSubject, ProviderSubject,
    ReleaseSigningSubject, SandboxSubject, ToolchainSubject,
};

pub const LOCK_SCHEMA_VERSION: &str = "3";
const MAX_LOCK_BYTES: u64 = 1024 * 1024;
const MAX_FILES_PER_MEMBER: usize = 4096;
const CANONICAL_MEMBERS: [&str; 3] = ["bullet-git", "bullet-kernel", "bullet-portal"];

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FamilyLock {
    pub schema_version: String,
    pub family: String,
    pub tag: String,
    pub schema_bundle_hash: String,
    pub hub: LockedHub,
    pub member: Vec<LockedMember>,
    pub external: ExternalSubjects,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedHub {
    pub name: String,
    pub tag: String,
    pub release_signing_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedMember {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jeryu_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jeryu_slug: Option<String>,
    pub tag: String,
    pub commit_oid: String,
    pub tree_oid: String,
    pub release_signing_identity: String,
    pub lockfile: Vec<LockedFile>,
    pub artifact: Vec<LockedFile>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedFile {
    pub path: String,
    pub digest: String,
}

impl FamilyLock {
    pub fn member(&self, name: &str) -> Option<&LockedMember> {
        self.member.iter().find(|member| member.name == name)
    }

    pub fn validate_required_members(&self, required: &[String]) -> Result<(), CoordError> {
        let all = required.iter().map(String::as_str).collect::<BTreeSet<_>>();
        if all.len() != required.len() {
            return Err(invalid("required family member names contain duplicates"));
        }
        if !all.contains("bullet-farm") {
            return Err(invalid("required family members omit bullet-farm"));
        }
        let expected = all
            .into_iter()
            .filter(|name| *name != "bullet-farm")
            .collect::<BTreeSet<_>>();
        let actual = self
            .member
            .iter()
            .map(|member| member.name.as_str())
            .collect::<BTreeSet<_>>();
        if actual != expected {
            return Err(invalid(
                "locked members do not exactly match the family manifest",
            ));
        }
        Ok(())
    }
}

pub fn load(path: &Path) -> Result<FamilyLock, CoordError> {
    let metadata = fs::symlink_metadata(path).map_err(CoordError::io)?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(invalid("family.lock must be a regular, non-symlink file"));
    }
    if metadata.len() > MAX_LOCK_BYTES {
        return Err(invalid("family.lock exceeds the 1 MiB admission limit"));
    }
    parse(&fs::read(path).map_err(CoordError::io)?)
}

pub fn parse(bytes: &[u8]) -> Result<FamilyLock, CoordError> {
    if bytes.len() as u64 > MAX_LOCK_BYTES {
        return Err(invalid("family.lock exceeds the 1 MiB admission limit"));
    }
    let text =
        std::str::from_utf8(bytes).map_err(|_| invalid("family.lock must contain valid UTF-8"))?;
    if text
        .bytes()
        .any(|byte| byte == 0 || (byte < 0x20 && !matches!(byte, b'\n' | b'\r' | b'\t')))
    {
        return Err(invalid("family.lock contains a forbidden control byte"));
    }
    let table: toml::Table =
        toml::from_str(text).map_err(|error| invalid(format!("invalid TOML: {error}")))?;
    let version = table
        .get("schema_version")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| invalid("family.lock has no string schema_version"))?;
    if version != LOCK_SCHEMA_VERSION {
        return Err(CoordError::new(
            "UNSUPPORTED_SCHEMA",
            format!(
                "family.lock schema {version} is not installable; retain it for diagnosis, regenerate schema {LOCK_SCHEMA_VERSION} from authenticated signed tags, and replace it atomically"
            ),
        ));
    }
    let lock: FamilyLock =
        toml::from_str(text).map_err(|error| invalid(format!("invalid TOML: {error}")))?;
    validate(&lock)?;
    Ok(lock)
}

pub fn encode(lock: &FamilyLock) -> Result<Vec<u8>, CoordError> {
    validate(lock)?;
    toml::to_string_pretty(lock)
        .map(String::into_bytes)
        .map_err(|error| CoordError::new("FAMILY_LOCK_ENCODE_FAILED", error.to_string()))
}

pub(crate) fn validate(lock: &FamilyLock) -> Result<(), CoordError> {
    if lock.schema_version != LOCK_SCHEMA_VERSION {
        return Err(CoordError::new(
            "UNSUPPORTED_SCHEMA",
            format!(
                "family.lock schema {} is not installable; retain it for diagnosis, regenerate schema {LOCK_SCHEMA_VERSION} from authenticated signed tags, and replace it atomically",
                lock.schema_version
            ),
        ));
    }
    if lock.family != "bullet-farm" {
        return Err(invalid("family must be bullet-farm"));
    }
    validate_tag(&lock.tag)?;
    validate_digest("schema_bundle_hash", &lock.schema_bundle_hash)?;
    if lock.hub.name != "bullet-farm" {
        return Err(invalid("hub subject name must be bullet-farm"));
    }
    if lock.hub.tag != lock.tag {
        return Err(invalid("hub subject tag must match the family tag"));
    }
    validate_signing_identity(&lock.hub.release_signing_identity)?;
    if lock.member.len() != CANONICAL_MEMBERS.len() {
        return Err(invalid(
            "family.lock must bind exactly the three non-Hub canonical repositories",
        ));
    }
    let mut names = BTreeSet::new();
    for member in &lock.member {
        if member.name == "bullet-farm" {
            return Err(invalid(
                "bullet-farm is the signed top-level subject and must not be a member entry",
            ));
        }
        validate_member(member, &lock.tag)?;
        if !names.insert(member.name.as_str()) {
            return Err(invalid(format!(
                "family.lock repeats member {}",
                member.name
            )));
        }
    }
    let ordered = lock
        .member
        .iter()
        .map(|member| member.name.as_str())
        .collect::<Vec<_>>();
    if ordered != CANONICAL_MEMBERS {
        return Err(invalid(
            "family.lock members must be the canonical repositories in byte order",
        ));
    }
    lock.external.validate()?;
    if lock.external.release_signing.identity != lock.hub.release_signing_identity {
        return Err(invalid(
            "release-signing subject identity does not bind the repository signer",
        ));
    }
    let portal = lock
        .member("bullet-portal")
        .ok_or_else(|| invalid("family.lock has no bullet-portal repository subject"))?;
    if lock.external.portal.source_commit_oid != portal.commit_oid
        || lock.external.portal.source_tree_oid != portal.tree_oid
    {
        return Err(invalid(
            "Portal artifact subject does not bind the locked bullet-portal commit and tree",
        ));
    }
    Ok(())
}

fn validate_member(member: &LockedMember, family_tag: &str) -> Result<(), CoordError> {
    validate_atom("member name", &member.name, 64)?;
    if member.tag != family_tag {
        return Err(invalid(format!(
            "{} tag does not match the family tag",
            member.name
        )));
    }
    validate_git_oid("commit_oid", &member.commit_oid)?;
    validate_git_oid("tree_oid", &member.tree_oid)?;
    validate_signing_identity(&member.release_signing_identity)?;
    match (&member.jeryu_url, &member.jeryu_slug) {
        (Some(url), Some(slug)) => {
            validate_jeryu_source(url, slug)?;
            if slug.rsplit('/').next() != Some(member.name.as_str()) {
                return Err(invalid(format!(
                    "{} Jeryu slug does not bind the member name",
                    member.name
                )));
            }
        }
        (None, None) => {
            return Err(invalid(format!(
                "{} lacks authenticated Jeryu URL/slug metadata",
                member.name
            )));
        }
        _ => {
            return Err(invalid(format!(
                "{} must provide both jeryu_url and jeryu_slug",
                member.name
            )));
        }
    }
    validate_files(&member.name, "lockfile", &member.lockfile, false)?;
    validate_files(&member.name, "artifact", &member.artifact, true)?;
    let lockfile_paths = member
        .lockfile
        .iter()
        .map(|file| file.path.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    if member
        .artifact
        .iter()
        .any(|file| lockfile_paths.contains(&file.path.to_ascii_lowercase()))
    {
        return Err(invalid(format!(
            "{} repeats a path across lockfile and artifact manifests",
            member.name
        )));
    }
    let expected_lockfile = if member.name == "bullet-portal" {
        "package-lock.json"
    } else {
        "Cargo.lock"
    };
    if member.lockfile.len() != 1 || member.lockfile[0].path != expected_lockfile {
        return Err(invalid(format!(
            "{} must bind exactly {expected_lockfile}",
            member.name
        )));
    }
    Ok(())
}

fn validate_files(
    member: &str,
    class: &str,
    files: &[LockedFile],
    permit_empty: bool,
) -> Result<(), CoordError> {
    if (!permit_empty && files.is_empty()) || files.len() > MAX_FILES_PER_MEMBER {
        return Err(invalid(format!(
            "{member} has an invalid number of {class} checksums"
        )));
    }
    let mut prior = None;
    let mut casefolded = BTreeSet::new();
    for file in files {
        validate_repository_path(&file.path)?;
        validate_digest("file digest", &file.digest)?;
        if prior.is_some_and(|path: &str| path >= file.path.as_str()) {
            return Err(invalid(format!(
                "{member} {class} paths must be unique and byte-sorted"
            )));
        }
        if !casefolded.insert(file.path.to_ascii_lowercase()) {
            return Err(invalid(format!(
                "{member} {class} paths collide under ASCII case folding"
            )));
        }
        prior = Some(file.path.as_str());
    }
    Ok(())
}
