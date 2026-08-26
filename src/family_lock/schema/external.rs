use std::{collections::BTreeSet, fs, path::Path};

use serde::{Deserialize, Serialize};

use super::{
    invalid, validate_atom, validate_digest, validate_git_oid, validate_signing_identity,
    validate_tag,
};
use crate::coord::CoordError;

const SUBJECT_MANIFEST_SCHEMA_VERSION: &str = "2";
const MAX_SUBJECT_MANIFEST_BYTES: u64 = 1024 * 1024;
const MAX_SUBJECTS_PER_CLASS: usize = 64;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalSubjectManifest {
    pub schema_version: String,
    pub external: ExternalSubjects,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalSubjects {
    pub toolchain: Vec<ToolchainSubject>,
    pub provider: Vec<ProviderSubject>,
    pub portal: PortalSubject,
    pub sandbox: Vec<SandboxSubject>,
    pub jeryu: JeryuSubject,
    pub release_signing: ReleaseSigningSubject,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ToolchainSubject {
    pub id: String,
    pub version: String,
    pub install_path: String,
    pub binary_digest: String,
    pub manifest_path: String,
    pub manifest_digest: String,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderSubject {
    pub id: String,
    pub version: String,
    pub profile: String,
    pub install_path: String,
    pub binary_digest: String,
    pub protocol_digest: String,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PortalSubject {
    pub id: String,
    pub version: String,
    pub source_commit_oid: String,
    pub source_tree_oid: String,
    pub install_path: String,
    pub bundle_digest: String,
    pub manifest_digest: String,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SandboxSubject {
    pub id: String,
    pub class: String,
    pub platform: String,
    pub install_path: String,
    pub image_digest: String,
    pub policy_digest: String,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JeryuSubject {
    pub id: String,
    pub version: String,
    pub tag: String,
    pub install_path: String,
    pub manifest_digest: String,
    pub binary_digest: String,
    pub api_schema_digest: String,
    pub capability_digest: String,
    pub sbom_digest: String,
    pub provenance_digest: String,
    pub signature_digest: String,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseSigningSubject {
    pub id: String,
    pub identity: String,
    pub policy_digest: String,
    pub allowed_signers_digest: String,
    pub key_digest: String,
    pub policy_path: String,
    pub not_before_unix_ms: u64,
    pub not_after_unix_ms: u64,
}

impl ExternalSubjectManifest {
    pub fn new(external: ExternalSubjects) -> Self {
        Self {
            schema_version: SUBJECT_MANIFEST_SCHEMA_VERSION.to_owned(),
            external,
        }
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, CoordError> {
        if bytes.len() as u64 > MAX_SUBJECT_MANIFEST_BYTES {
            return Err(subject_error(
                "external subject manifest exceeds the 1 MiB admission limit",
            ));
        }
        let text = std::str::from_utf8(bytes)
            .map_err(|_| subject_error("external subject manifest must contain valid UTF-8"))?;
        if text
            .bytes()
            .any(|byte| byte == 0 || (byte < 0x20 && !matches!(byte, b'\n' | b'\r' | b'\t')))
        {
            return Err(subject_error(
                "external subject manifest contains a forbidden control byte",
            ));
        }
        let manifest: Self = toml::from_str(text)
            .map_err(|error| subject_error(format!("invalid TOML: {error}")))?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn load(path: &Path) -> Result<Self, CoordError> {
        if !path.is_absolute() {
            return Err(subject_error(
                "external subject manifest path must be absolute",
            ));
        }
        let metadata = fs::symlink_metadata(path).map_err(CoordError::io)?;
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return Err(subject_error(
                "external subject manifest must be a regular, non-symlink file",
            ));
        }
        if metadata.len() > MAX_SUBJECT_MANIFEST_BYTES {
            return Err(subject_error(
                "external subject manifest exceeds the 1 MiB admission limit",
            ));
        }
        Self::parse(&fs::read(path).map_err(CoordError::io)?)
    }

    pub fn encode(&self) -> Result<Vec<u8>, CoordError> {
        self.validate()?;
        toml::to_string_pretty(self)
            .map(String::into_bytes)
            .map_err(|error| subject_error(format!("encode failed: {error}")))
    }

    fn validate(&self) -> Result<(), CoordError> {
        if self.schema_version != SUBJECT_MANIFEST_SCHEMA_VERSION {
            return Err(subject_error(
                "external subject manifest schema_version must be 2",
            ));
        }
        self.external.validate().map_err(|error| {
            subject_error(format!("external subject manifest is invalid: {error}"))
        })
    }
}

impl ExternalSubjects {
    pub(super) fn validate(&self) -> Result<(), CoordError> {
        validate_cardinality("toolchain", self.toolchain.len())?;
        validate_cardinality("provider", self.provider.len())?;
        validate_cardinality("sandbox", self.sandbox.len())?;

        let mut ids = BTreeSet::new();
        let mut tool_paths = BTreeSet::new();
        for subject in &self.toolchain {
            validate_subject_id(&mut ids, &subject.id)?;
            validate_version("toolchain version", &subject.version)?;
            validate_absolute_path("toolchain install_path", &subject.install_path)?;
            validate_digest("toolchain binary_digest", &subject.binary_digest)?;
            validate_absolute_path("toolchain manifest_path", &subject.manifest_path)?;
            if !tool_paths.insert(subject.install_path.clone())
                || !tool_paths.insert(subject.manifest_path.clone())
            {
                return Err(invalid(
                    "toolchain executable and manifest paths must be pairwise distinct",
                ));
            }
            validate_required_tool_version(subject)?;
            validate_digest("toolchain manifest_digest", &subject.manifest_digest)?;
            validate_size("toolchain size_bytes", subject.size_bytes)?;
        }
        for required in ["cargo", "node", "npm-cli"] {
            if !self.toolchain.iter().any(|subject| subject.id == required) {
                return Err(invalid(format!(
                    "family.lock external toolchain is missing required {required} subject"
                )));
            }
        }
        for subject in &self.provider {
            validate_subject_id(&mut ids, &subject.id)?;
            validate_version("provider version", &subject.version)?;
            validate_atom("provider profile", &subject.profile, 128)?;
            validate_absolute_path("provider install_path", &subject.install_path)?;
            validate_digest("provider binary_digest", &subject.binary_digest)?;
            validate_digest("provider protocol_digest", &subject.protocol_digest)?;
            validate_size("provider size_bytes", subject.size_bytes)?;
        }

        if self.portal.id != "portal" {
            return Err(invalid("Portal subject id must be portal"));
        }
        validate_subject_id(&mut ids, &self.portal.id)?;
        validate_version("Portal version", &self.portal.version)?;
        validate_git_oid("Portal source_commit_oid", &self.portal.source_commit_oid)?;
        validate_git_oid("Portal source_tree_oid", &self.portal.source_tree_oid)?;
        validate_absolute_path("Portal install_path", &self.portal.install_path)?;
        validate_digest("Portal bundle_digest", &self.portal.bundle_digest)?;
        validate_digest("Portal manifest_digest", &self.portal.manifest_digest)?;
        validate_size("Portal size_bytes", self.portal.size_bytes)?;

        for subject in &self.sandbox {
            validate_subject_id(&mut ids, &subject.id)?;
            if !matches!(subject.class.as_str(), "s1" | "s2") {
                return Err(invalid("sandbox class must be s1 or s2"));
            }
            validate_atom("sandbox platform", &subject.platform, 128)?;
            validate_absolute_path("sandbox install_path", &subject.install_path)?;
            validate_digest("sandbox image_digest", &subject.image_digest)?;
            validate_digest("sandbox policy_digest", &subject.policy_digest)?;
            validate_size("sandbox size_bytes", subject.size_bytes)?;
        }

        if self.jeryu.id != "jeryu" {
            return Err(invalid("Jeryu subject id must be jeryu"));
        }
        validate_subject_id(&mut ids, &self.jeryu.id)?;
        validate_version("Jeryu version", &self.jeryu.version)?;
        validate_tag(&self.jeryu.tag)?;
        validate_absolute_path("Jeryu install_path", &self.jeryu.install_path)?;
        for (label, digest) in [
            ("Jeryu manifest_digest", &self.jeryu.manifest_digest),
            ("Jeryu binary_digest", &self.jeryu.binary_digest),
            ("Jeryu api_schema_digest", &self.jeryu.api_schema_digest),
            ("Jeryu capability_digest", &self.jeryu.capability_digest),
            ("Jeryu sbom_digest", &self.jeryu.sbom_digest),
            ("Jeryu provenance_digest", &self.jeryu.provenance_digest),
            ("Jeryu signature_digest", &self.jeryu.signature_digest),
        ] {
            validate_digest(label, digest)?;
        }
        validate_size("Jeryu size_bytes", self.jeryu.size_bytes)?;

        if self.release_signing.id != "release-signing" {
            return Err(invalid(
                "release-signing subject id must be release-signing",
            ));
        }
        validate_subject_id(&mut ids, &self.release_signing.id)?;
        validate_signing_identity(&self.release_signing.identity)?;
        validate_digest(
            "release-signing policy_digest",
            &self.release_signing.policy_digest,
        )?;
        validate_digest(
            "release-signing allowed_signers_digest",
            &self.release_signing.allowed_signers_digest,
        )?;
        validate_digest(
            "release-signing key_digest",
            &self.release_signing.key_digest,
        )?;
        validate_absolute_path(
            "release-signing policy_path",
            &self.release_signing.policy_path,
        )?;
        validate_safe_integer(
            "release-signing not_before_unix_ms",
            self.release_signing.not_before_unix_ms,
        )?;
        validate_safe_integer(
            "release-signing not_after_unix_ms",
            self.release_signing.not_after_unix_ms,
        )?;
        if self.release_signing.not_before_unix_ms >= self.release_signing.not_after_unix_ms {
            return Err(invalid(
                "release-signing validity window must be non-empty and ordered",
            ));
        }
        Ok(())
    }
}

fn validate_required_tool_version(subject: &ToolchainSubject) -> Result<(), CoordError> {
    match subject.id.as_str() {
        "cargo" if !numeric_triplet(&subject.version) => Err(invalid(
            "cargo toolchain version must be exactly three numeric components",
        )),
        "node" if subject.version != crate::toolchain_pins::node() => Err(invalid(format!(
            "node toolchain version must be exactly {}",
            crate::toolchain_pins::node()
        ))),
        "npm-cli" if subject.version != crate::toolchain_pins::npm() => Err(invalid(format!(
            "npm-cli toolchain version must be exactly {}",
            crate::toolchain_pins::npm()
        ))),
        _ => Ok(()),
    }
}

fn numeric_triplet(value: &str) -> bool {
    let mut parts = value.split('.');
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some(major), Some(minor), Some(patch), None)
            if [major, minor, patch].into_iter().all(|part| {
                !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit())
            })
    )
}

fn validate_cardinality(class: &str, count: usize) -> Result<(), CoordError> {
    if count == 0 || count > MAX_SUBJECTS_PER_CLASS {
        Err(invalid(format!(
            "family.lock must contain 1..={MAX_SUBJECTS_PER_CLASS} {class} subjects"
        )))
    } else {
        Ok(())
    }
}

fn validate_subject_id<'a>(ids: &mut BTreeSet<&'a str>, id: &'a str) -> Result<(), CoordError> {
    validate_atom("external subject id", id, 128)?;
    if !ids.insert(id) {
        return Err(invalid(format!("external subject id {id} is duplicated")));
    }
    Ok(())
}

fn validate_version(label: &str, value: &str) -> Result<(), CoordError> {
    if value.is_empty()
        || value.len() > 256
        || !value.is_ascii()
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace())
    {
        Err(invalid(format!("{label} is not a bounded exact version")))
    } else {
        Ok(())
    }
}

fn validate_absolute_path(label: &str, path: &str) -> Result<(), CoordError> {
    if path.len() < 2
        || path.len() > 4096
        || !path.starts_with('/')
        || path.ends_with('/')
        || path.contains(['\\', '\0'])
        || path.bytes().any(|byte| byte.is_ascii_control())
        || path
            .split('/')
            .skip(1)
            .any(|segment| segment.is_empty() || matches!(segment, "." | ".."))
    {
        Err(invalid(format!(
            "{label} must be an absolute canonical path"
        )))
    } else {
        Ok(())
    }
}

fn validate_size(label: &str, value: u64) -> Result<(), CoordError> {
    if value == 0 {
        return Err(invalid(format!("{label} must be non-zero")));
    }
    validate_safe_integer(label, value)
}

fn validate_safe_integer(label: &str, value: u64) -> Result<(), CoordError> {
    if value > MAX_SAFE_INTEGER {
        Err(invalid(format!(
            "{label} exceeds the JSON safe-integer range"
        )))
    } else {
        Ok(())
    }
}

fn subject_error(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_LOCK_SUBJECTS", reason)
}
