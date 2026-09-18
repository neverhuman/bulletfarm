//! Strict schema for a signed five-platform release bundle.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::coord::CoordError;

pub const RELEASE_MANIFEST_SCHEMA_VERSION: &str = "2";
const FAMILY_LOCK_SCHEMA_VERSION: &str = "3";
const MAX_RELEASE_FILE_BYTES: u64 = 4 * 1024 * 1024 * 1024;
pub(super) const REQUIRED_TARGETS: [&str; 5] = [
    "aarch64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc",
    "x86_64-unknown-linux-gnu",
];

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseManifest {
    pub release_manifest_schema_version: String,
    pub family_lock_schema_version: String,
    pub family: String,
    pub tag: String,
    pub hub_commit_oid: String,
    pub hub_tree_oid: String,
    pub release_signing_identity: String,
    pub family_lock: ReleaseFile,
    pub package: Vec<ReleasePackage>,
}

#[derive(Deserialize)]
struct ReleaseManifestVersion {
    release_manifest_schema_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReleasePackage {
    pub target: String,
    pub archive: SignedReleaseFile,
    pub checksums: SignedReleaseFile,
    pub cyclonedx_sbom: SignedReleaseFile,
    pub spdx_sbom: SignedReleaseFile,
    pub provenance: SignedReleaseFile,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SignedReleaseFile {
    pub file: ReleaseFile,
    pub signature: ReleaseFile,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseFile {
    pub path: String,
    pub size: u64,
    pub digest: String,
}

impl ReleaseManifest {
    pub fn parse(bytes: &[u8]) -> Result<Self, CoordError> {
        if bytes.len() > 1024 * 1024 {
            return Err(invalid("release manifest exceeds 1 MiB"));
        }
        let text = std::str::from_utf8(bytes)
            .map_err(|_| invalid("release manifest must contain valid UTF-8"))?;
        if text
            .bytes()
            .any(|byte| byte == 0 || (byte < 0x20 && !matches!(byte, b'\n' | b'\r' | b'\t')))
        {
            return Err(invalid(
                "release manifest contains a forbidden control byte",
            ));
        }
        let version: ReleaseManifestVersion = toml::from_str(text)
            .map_err(|error| invalid(format!("invalid release manifest TOML: {error}")))?;
        if version.release_manifest_schema_version != RELEASE_MANIFEST_SCHEMA_VERSION {
            return Err(CoordError::new(
                "UNSUPPORTED_RELEASE_MANIFEST_SCHEMA",
                format!(
                    "release manifest schema {} is unsupported",
                    version.release_manifest_schema_version
                ),
            ));
        }
        let manifest: Self = toml::from_str(text)
            .map_err(|error| invalid(format!("invalid release manifest TOML: {error}")))?;
        manifest.validate()?;
        Ok(manifest)
    }

    fn validate(&self) -> Result<(), CoordError> {
        if self.release_manifest_schema_version != RELEASE_MANIFEST_SCHEMA_VERSION {
            return Err(CoordError::new(
                "UNSUPPORTED_RELEASE_MANIFEST_SCHEMA",
                format!(
                    "release manifest schema {} is unsupported",
                    self.release_manifest_schema_version
                ),
            ));
        }
        if self.family_lock_schema_version != FAMILY_LOCK_SCHEMA_VERSION {
            return Err(CoordError::new(
                "UNSUPPORTED_SCHEMA",
                "release manifest must bind family.lock schema 3",
            ));
        }
        if self.family != "bullet-farm" {
            return Err(invalid("release family must be bullet-farm"));
        }
        validate_tag(&self.tag)?;
        validate_oid("hub_commit_oid", &self.hub_commit_oid)?;
        validate_oid("hub_tree_oid", &self.hub_tree_oid)?;
        validate_signing_identity(&self.release_signing_identity)?;
        validate_file(&self.family_lock, false)?;
        if self.family_lock.path != "family.lock" {
            return Err(invalid(
                "release manifest must bind family.lock at family.lock",
            ));
        }
        if self.package.len() != REQUIRED_TARGETS.len()
            || self
                .package
                .iter()
                .map(|package| package.target.as_str())
                .ne(REQUIRED_TARGETS)
        {
            return Err(invalid(
                "release manifest must contain the five byte-sorted required targets exactly once",
            ));
        }
        let mut paths = BTreeSet::from([self.family_lock.path.to_ascii_lowercase()]);
        for package in &self.package {
            validate_package(package, &mut paths)?;
        }
        Ok(())
    }

    pub(super) fn signer_parts(&self) -> (&str, &str) {
        let (principal, rest) = self
            .release_signing_identity
            .split_once('|')
            .expect("validated signing identity has principal");
        let fingerprint = rest
            .strip_prefix("ed25519|")
            .expect("validated signing identity is Ed25519");
        (principal, fingerprint)
    }
}

fn validate_package(
    package: &ReleasePackage,
    paths: &mut BTreeSet<String>,
) -> Result<(), CoordError> {
    for signed in [
        &package.archive,
        &package.checksums,
        &package.cyclonedx_sbom,
        &package.spdx_sbom,
        &package.provenance,
    ] {
        validate_signed_file(signed, paths)?;
    }
    let archive_suffix = if package.target == "x86_64-pc-windows-msvc" {
        ".zip"
    } else {
        ".tar.zst"
    };
    if !package.archive.file.path.ends_with(archive_suffix) {
        return Err(invalid(format!(
            "{} archive must end with {archive_suffix}",
            package.target
        )));
    }
    if !package.checksums.file.path.ends_with(".checksums.json") {
        return Err(invalid(format!(
            "{} checksum manifest must end with .checksums.json",
            package.target
        )));
    }
    if !package.cyclonedx_sbom.file.path.ends_with(".cdx.json") {
        return Err(invalid(format!(
            "{} CycloneDX SBOM must end with .cdx.json",
            package.target
        )));
    }
    if !package.spdx_sbom.file.path.ends_with(".spdx.json") {
        return Err(invalid(format!(
            "{} SPDX SBOM must end with .spdx.json",
            package.target
        )));
    }
    if !package.provenance.file.path.ends_with(".intoto.jsonl") {
        return Err(invalid(format!(
            "{} provenance must be an in-toto JSONL statement",
            package.target
        )));
    }
    Ok(())
}

fn validate_signed_file(
    signed: &SignedReleaseFile,
    paths: &mut BTreeSet<String>,
) -> Result<(), CoordError> {
    validate_file(&signed.file, false)?;
    validate_file(&signed.signature, true)?;
    if signed.signature.path != format!("{}.sig", signed.file.path) {
        return Err(invalid(format!(
            "{} signature path must be the payload path plus .sig",
            signed.file.path
        )));
    }
    for file in [&signed.file, &signed.signature] {
        if !paths.insert(file.path.to_ascii_lowercase()) {
            return Err(invalid(format!(
                "release bundle path collides or repeats: {}",
                file.path
            )));
        }
    }
    Ok(())
}

fn validate_file(file: &ReleaseFile, signature: bool) -> Result<(), CoordError> {
    validate_relative_path(&file.path)?;
    let maximum = if signature {
        64 * 1024
    } else {
        MAX_RELEASE_FILE_BYTES
    };
    if file.size == 0 || file.size > maximum {
        return Err(invalid(format!("{} has an invalid byte size", file.path)));
    }
    validate_digest(&file.digest)
}

fn validate_relative_path(path: &str) -> Result<(), CoordError> {
    if path.is_empty()
        || path.len() > 4096
        || !path.is_ascii()
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains('\\')
        || path.bytes().any(|byte| byte.is_ascii_control())
        || path.split('/').any(|segment| {
            segment.is_empty()
                || matches!(segment, "." | "..")
                || segment.eq_ignore_ascii_case(".git")
        })
    {
        return Err(invalid(format!("unsafe release bundle path: {path:?}")));
    }
    Ok(())
}

pub(super) fn validate_digest(digest: &str) -> Result<(), CoordError> {
    let Some(hex) = digest.strip_prefix("blake3:") else {
        return Err(invalid(
            "release file digest must be algorithm-tagged BLAKE3",
        ));
    };
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(invalid("release file digest must be full lowercase BLAKE3"));
    }
    Ok(())
}

pub(super) fn validate_oid(field: &str, oid: &str) -> Result<(), CoordError> {
    let valid = oid.strip_prefix("sha1:").is_some_and(|hex| hex.len() == 40)
        || oid
            .strip_prefix("sha256:")
            .is_some_and(|hex| hex.len() == 64);
    if !valid
        || oid.split_once(':').is_none_or(|(_, hex)| {
            !hex.bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        })
    {
        return Err(invalid(format!(
            "{field} is not a full lowercase algorithm-tagged Git OID"
        )));
    }
    Ok(())
}

pub(super) fn validate_tag(tag: &str) -> Result<(), CoordError> {
    if tag.len() > 128
        || !tag.starts_with('v')
        || !tag.as_bytes().get(1).is_some_and(u8::is_ascii_digit)
        || tag.ends_with(['.', '-'])
        || tag.contains("..")
        || tag
            .bytes()
            .any(|byte| !(byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-')))
    {
        return Err(invalid("release tag is malformed"));
    }
    Ok(())
}

fn validate_signing_identity(identity: &str) -> Result<(), CoordError> {
    let Some((principal, rest)) = identity.split_once('|') else {
        return Err(invalid("release signing identity is malformed"));
    };
    let Some(fingerprint) = rest.strip_prefix("ed25519|") else {
        return Err(invalid("release signing identity must use Ed25519"));
    };
    if principal.is_empty()
        || principal.len() > 256
        || principal
            .bytes()
            .any(|byte| byte.is_ascii_whitespace() || byte == b'|')
        || !fingerprint.starts_with("SHA256:")
        || fingerprint.len() > 128
        || fingerprint.bytes().any(|byte| {
            !(byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'+' | b'/' | b'='))
        })
    {
        return Err(invalid("release signing identity is malformed"));
    }
    Ok(())
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RELEASE_MANIFEST", reason)
}
