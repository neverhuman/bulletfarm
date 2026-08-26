use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::{coord::CoordError, family_lock};

use super::model::{DoctorFamilyLock, DoctorLockedMember};

const MAX_LOCK_BYTES: u64 = 1024 * 1024;

#[derive(Deserialize)]
struct SchemaProbe {
    schema_version: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyFamilyLock {
    schema_version: String,
    family: String,
    tag: String,
    schema_bundle_hash: String,
    member: Vec<LegacyLockedMember>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyLockedMember {
    name: String,
    tag: String,
    commit_oid: String,
    schema_bundle_hash: String,
    release_signing_identity: String,
    generated_client_hash: String,
    #[serde(default)]
    jeryu_url: Option<String>,
    #[serde(default)]
    source_url: Option<String>,
    #[serde(default)]
    jeryu_slug: Option<String>,
}

pub(super) fn discover_hub(
    current_dir: &Path,
    explicit_root: Option<&str>,
) -> Result<PathBuf, CoordError> {
    let start = explicit_root.map_or_else(|| current_dir.to_path_buf(), PathBuf::from);
    let start = start.canonicalize().map_err(CoordError::io)?;
    for ancestor in start.ancestors() {
        if is_hub(ancestor) {
            return Ok(ancestor.to_path_buf());
        }
        let child = ancestor.join("bullet-farm");
        if is_hub(&child) {
            return child.canonicalize().map_err(CoordError::io);
        }
        if explicit_root.is_some() {
            break;
        }
    }
    Err(CoordError::new(
        "HUB_CHECKOUT_NOT_FOUND",
        format!(
            "{} is not inside a Bullet Farm hub checkout",
            start.display()
        ),
    ))
}

pub(super) fn read_lock(hub_root: &Path) -> Result<DoctorFamilyLock, CoordError> {
    let path = hub_root.join("family.lock");
    let metadata = fs::symlink_metadata(&path).map_err(CoordError::io)?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(invalid("family.lock must be a regular, non-symlink file"));
    }
    if metadata.len() > MAX_LOCK_BYTES {
        return Err(invalid("family.lock exceeds the 1 MiB admission limit"));
    }
    let bytes = fs::read(&path).map_err(CoordError::io)?;
    let text =
        std::str::from_utf8(&bytes).map_err(|_| invalid("family.lock must contain valid UTF-8"))?;
    let probe: SchemaProbe =
        toml::from_str(text).map_err(|error| invalid(format!("invalid TOML: {error}")))?;
    let lock = match probe.schema_version.as_str() {
        family_lock::LOCK_SCHEMA_VERSION => read_current(&bytes),
        "2" => read_legacy(text),
        version => Err(CoordError::new(
            "UNSUPPORTED_SCHEMA",
            format!("family.lock schema {version} is not supported"),
        )),
    }?;
    validate_required_members(hub_root, &lock)?;
    Ok(lock)
}

fn read_current(bytes: &[u8]) -> Result<DoctorFamilyLock, CoordError> {
    let lock = family_lock::parse(bytes)?;
    let members = lock
        .member
        .iter()
        .map(|member| DoctorLockedMember {
            name: member.name.clone(),
            commit_oid: member
                .commit_oid
                .split_once(':')
                .expect("validated algorithm-tagged OID")
                .1
                .to_owned(),
            jeryu_url: member.jeryu_url.clone(),
            jeryu_slug: member.jeryu_slug.clone(),
        })
        .collect();
    Ok(DoctorFamilyLock {
        schema_version: lock.schema_version.clone(),
        tag: lock.tag.clone(),
        installable_schema: true,
        current: Some(lock),
        member: members,
    })
}

fn read_legacy(text: &str) -> Result<DoctorFamilyLock, CoordError> {
    let lock: LegacyFamilyLock =
        toml::from_str(text).map_err(|error| invalid(format!("invalid schema 2 lock: {error}")))?;
    if lock.schema_version != "2"
        || lock.family != "bullet-farm"
        || !valid_tag(&lock.tag)
        || !valid_digest(&lock.schema_bundle_hash)
        || lock.member.is_empty()
        || lock.member.len() > 64
    {
        return Err(invalid("schema 2 family metadata is malformed"));
    }
    let mut names = BTreeSet::new();
    let mut members = Vec::with_capacity(lock.member.len());
    for member in lock.member {
        if !valid_name(&member.name)
            || !names.insert(member.name.clone())
            || member.tag != lock.tag
            || !valid_oid(&member.commit_oid)
            || !valid_digest(&member.schema_bundle_hash)
            || !valid_digest(&member.generated_client_hash)
            || member.release_signing_identity.is_empty()
            || member.release_signing_identity.len() > 512
        {
            return Err(invalid(format!(
                "schema 2 member {} is malformed",
                member.name
            )));
        }
        let source = match (member.jeryu_url, member.source_url) {
            (Some(_), Some(_)) => {
                return Err(invalid(format!(
                    "schema 2 member {} has ambiguous source URLs",
                    member.name
                )));
            }
            (url, None) | (None, url) => url,
        };
        members.push(DoctorLockedMember {
            name: member.name,
            commit_oid: member.commit_oid,
            jeryu_url: source,
            jeryu_slug: member.jeryu_slug,
        });
    }
    Ok(DoctorFamilyLock {
        schema_version: lock.schema_version,
        tag: lock.tag,
        installable_schema: false,
        current: None,
        member: members,
    })
}

fn validate_required_members(hub_root: &Path, lock: &DoctorFamilyLock) -> Result<(), CoordError> {
    let required = crate::checkout::required_members(hub_root)?;
    if let Some(current) = &lock.current {
        return current.validate_required_members(&required);
    }
    let expected = required.into_iter().collect::<BTreeSet<_>>();
    let actual = lock
        .member
        .iter()
        .map(|member| member.name.clone())
        .collect::<BTreeSet<_>>();
    if expected != actual || actual.len() != lock.member.len() {
        return Err(invalid(
            "schema 2 members do not exactly match the signed hub manifest",
        ));
    }
    Ok(())
}

fn is_hub(path: &Path) -> bool {
    path.join("Cargo.toml").is_file()
        && path.join("family.lock").is_file()
        && path.join("scripts/setup.sh").is_file()
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_tag(tag: &str) -> bool {
    tag.starts_with('v')
        && tag.len() <= 128
        && tag
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
}

fn valid_digest(digest: &str) -> bool {
    digest
        .strip_prefix("blake3:")
        .is_some_and(|hex| valid_hex(hex, 64))
}

fn valid_oid(oid: &str) -> bool {
    matches!(oid.len(), 40 | 64) && valid_hex(oid, oid.len())
}

fn valid_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_FAMILY_LOCK", reason)
}
