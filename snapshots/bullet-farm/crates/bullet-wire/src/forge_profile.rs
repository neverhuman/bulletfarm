//! Primary forge profile and replication intent records (closure roadmap,
//! Wave 4). Exactly one `PrimaryForgeProfileV1` generation is active per
//! repository; `ReplicationIntentV1` mirrors refs between two profiles and is
//! structurally incapable of carrying an integration subject.

use std::{cmp::Ordering, collections::BTreeMap, fmt};

use serde::{Deserialize, Serialize};

use crate::{
    Blake3Digest, RepositoryId, WireError, decode_canonical, digest::validate_lower_hex,
    hash_canonical,
};

pub const FORGE_PROFILE_SCHEMA_VERSION: &str = "v1alpha1";
pub const PRIMARY_FORGE_PROFILE_DIGEST_DOMAIN: &str = "forge.primary-profile.v1alpha1";
pub const REPLICATION_INTENT_DIGEST_DOMAIN: &str = "forge.replication-intent.v1alpha1";
pub const MAX_FORGE_BASE_URL_BYTES: usize = 2048;
pub const MAX_REPLICATION_REFS: usize = 1024;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

/// Content address of a sealed primary forge profile (`fpf_<blake3 hex>`).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub struct ForgeProfileId(String);

impl ForgeProfileId {
    pub const PREFIX: &'static str = "fpf_";

    pub fn from_digest(digest: Blake3Digest) -> Self {
        Self(format!("{}{}", Self::PREFIX, digest.to_hex()))
    }
}

impl TryFrom<String> for ForgeProfileId {
    type Error = WireError;

    fn try_from(raw: String) -> Result<Self, WireError> {
        let hex = raw
            .strip_prefix(Self::PREFIX)
            .ok_or_else(|| refuse("INVALID_FORGE_PROFILE_ID"))?;
        validate_lower_hex(hex, 64, "INVALID_FORGE_PROFILE_ID")?;
        Ok(Self(raw))
    }
}

impl From<ForgeProfileId> for String {
    fn from(id: ForgeProfileId) -> Self {
        id.0
    }
}

impl fmt::Display for ForgeProfileId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ForgeKind {
    Jeryu,
    Github,
    Gitlab,
    LocalBare,
}

/// Semantic-port capabilities. Variants are declared in byte order of their
/// wire names, so the derived `Ord` is the canonical wire ordering.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForgeCapability {
    ExactShaChecks,
    ExpectedOldOid,
    IntegrationSubject,
    MergeGroups,
    ProtectedRefs,
    PullRequests,
    ReadBack,
}

/// Only a validated primary profile carrying `integration_subject` yields one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrationSubjectBinding {
    pub repository_id: RepositoryId,
    pub profile_id: ForgeProfileId,
    pub generation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrimaryForgeProfileV1 {
    pub schema_version: String,
    pub repository_id: RepositoryId,
    pub forge_kind: ForgeKind,
    pub base_url: String,
    pub capabilities: Vec<ForgeCapability>,
    pub generation: u64,
    pub activated_by: String,
    pub activated_at_unix_ms: u64,
    pub digest: Blake3Digest,
}

impl PrimaryForgeProfileV1 {
    /// Bind the content address to caller-supplied fields and validate. The
    /// incoming `digest` value is ignored and replaced.
    pub fn seal(mut self) -> Result<Self, WireError> {
        self.digest = self.expected_digest()?;
        self.validate()?;
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), WireError> {
        let generation = self.generation;
        check(
            self.schema_version == FORGE_PROFILE_SCHEMA_VERSION,
            "UNSUPPORTED_FORGE_PROFILE_SCHEMA",
        )?;
        validate_base_url(self.forge_kind, &self.base_url)?;
        validate_capabilities(&self.capabilities)?;
        check(
            generation != 0 && generation <= MAX_SAFE_INTEGER,
            "INVALID_FORGE_PROFILE_GENERATION",
        )?;
        check(
            is_label(&self.activated_by),
            "INVALID_FORGE_PROFILE_ACTIVATOR",
        )?;
        check(
            self.activated_at_unix_ms <= MAX_SAFE_INTEGER,
            "INVALID_FORGE_PROFILE_TIME",
        )?;
        check(
            self.digest == self.expected_digest()?,
            "FORGE_PROFILE_DIGEST_MISMATCH",
        )
    }

    /// Identity is the canonical record without its `digest` member.
    pub fn expected_digest(&self) -> Result<Blake3Digest, WireError> {
        identity_digest(PRIMARY_FORGE_PROFILE_DIGEST_DOMAIN, self)
    }

    pub fn profile_id(&self) -> ForgeProfileId {
        ForgeProfileId::from_digest(self.digest)
    }

    pub fn has_capability(&self, capability: ForgeCapability) -> bool {
        self.capabilities.contains(&capability)
    }

    pub fn integration_subject(&self) -> Result<IntegrationSubjectBinding, WireError> {
        self.validate()?;
        check(
            self.has_capability(ForgeCapability::IntegrationSubject),
            "FORGE_PROFILE_LACKS_INTEGRATION_SUBJECT",
        )?;
        Ok(IntegrationSubjectBinding {
            repository_id: self.repository_id.clone(),
            profile_id: self.profile_id(),
            generation: self.generation,
        })
    }
}

pub fn decode_primary_forge_profile(bytes: &[u8]) -> Result<PrimaryForgeProfileV1, WireError> {
    let profile: PrimaryForgeProfileV1 = decode_canonical(bytes)?;
    profile.validate()?;
    Ok(profile)
}

fn identity_digest<T: Serialize>(domain: &str, record: &T) -> Result<Blake3Digest, WireError> {
    let mut value = serde_json::to_value(record).map_err(|_| refuse("CANONICAL_JSON_FAILED"))?;
    value
        .as_object_mut()
        .and_then(|members| members.remove("digest"))
        .ok_or_else(|| refuse("CANONICAL_JSON_FAILED"))?;
    hash_canonical(domain, &value)
}

fn validate_base_url(kind: ForgeKind, url: &str) -> Result<(), WireError> {
    let printable = !url.is_empty()
        && url.len() <= MAX_FORGE_BASE_URL_BYTES
        && url.is_ascii()
        && !url
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
        && !url.contains(['?', '#', '\\']);
    check(printable, "INVALID_FORGE_PROFILE_URL")?;
    if kind == ForgeKind::LocalBare {
        let path = url.strip_prefix("file:///");
        return check(
            path.is_some_and(safe_path_segments),
            "INVALID_FORGE_PROFILE_URL",
        );
    }
    let (https, remainder) = if let Some(rest) = url.strip_prefix("https://") {
        (true, rest)
    } else if let Some(rest) = url.strip_prefix("http://") {
        (false, rest)
    } else {
        return Err(refuse("INVALID_FORGE_PROFILE_URL"));
    };
    let (authority, path) = remainder.split_once('/').unwrap_or((remainder, ""));
    let well_formed = !authority.is_empty() && !authority.contains('@') && safe_path_segments(path);
    check(well_formed, "INVALID_FORGE_PROFILE_URL")?;
    let loopback = valid_loopback_authority(authority);
    check(
        kind != ForgeKind::Jeryu || loopback,
        "FORGE_PROFILE_URL_NOT_LOOPBACK",
    )?;
    check(https || loopback, "FORGE_PROFILE_URL_NOT_HTTPS")
}

fn safe_path_segments(path: &str) -> bool {
    path.is_empty()
        || path.split('/').all(|segment| {
            !segment.is_empty()
                && !matches!(segment, "." | "..")
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        })
}

/// Only the three exact loopback authorities, with an optional non-zero port.
fn valid_loopback_authority(authority: &str) -> bool {
    let Some(rest) = ["[::1]", "127.0.0.1", "localhost"]
        .iter()
        .find_map(|host| authority.strip_prefix(host))
    else {
        return false;
    };
    rest.is_empty()
        || rest.strip_prefix(':').is_some_and(|port| {
            !port.is_empty()
                && port.bytes().all(|byte| byte.is_ascii_digit())
                && port
                    .bytes()
                    .try_fold(0_u32, |number, byte| {
                        number.checked_mul(10)?.checked_add(u32::from(byte - b'0'))
                    })
                    .is_some_and(|port| port != 0 && port <= u32::from(u16::MAX))
        })
}

fn validate_capabilities(capabilities: &[ForgeCapability]) -> Result<(), WireError> {
    for pair in capabilities.windows(2) {
        match pair[0].cmp(&pair[1]) {
            Ordering::Less => {}
            Ordering::Equal => return Err(refuse("FORGE_PROFILE_CAPABILITY_DUPLICATE")),
            Ordering::Greater => return Err(refuse("FORGE_PROFILE_CAPABILITY_UNSORTED")),
        }
    }
    Ok(())
}

fn is_label(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'/' | b'-')
        })
}

/// In-memory registry of active primary profiles. The map is keyed by
/// repository, so at most one profile can ever be active per repository;
/// `activate` replaces the previous generation in a single insert.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ForgeProfileRegistry {
    active: BTreeMap<RepositoryId, PrimaryForgeProfileV1>,
}

impl ForgeProfileRegistry {
    /// Activate `profile`, returning the generation it superseded. Re-activating
    /// the active generation or any lower generation is refused and leaves the
    /// registry unchanged.
    pub fn activate(
        &mut self,
        profile: PrimaryForgeProfileV1,
    ) -> Result<Option<PrimaryForgeProfileV1>, WireError> {
        profile.validate()?;
        if let Some(current) = self.active.get(&profile.repository_id) {
            match profile.generation.cmp(&current.generation) {
                Ordering::Greater => {}
                Ordering::Equal => return Err(refuse("FORGE_PROFILE_GENERATION_REPLAY")),
                Ordering::Less => return Err(refuse("FORGE_PROFILE_GENERATION_REGRESSION")),
            }
        }
        Ok(self.active.insert(profile.repository_id.clone(), profile))
    }

    pub fn active(&self, repository_id: &RepositoryId) -> Option<&PrimaryForgeProfileV1> {
        self.active.get(repository_id)
    }

    pub fn active_profiles(&self) -> impl Iterator<Item = &PrimaryForgeProfileV1> {
        self.active.values()
    }

    /// Resolve a replication intent's source: it must be the currently active
    /// primary of some repository. This yields replication provenance only,
    /// never an integration subject.
    pub fn replication_source(
        &self,
        intent: &ReplicationIntentV1,
    ) -> Result<&PrimaryForgeProfileV1, WireError> {
        intent.validate()?;
        self.active_profiles()
            .find(|profile| profile.profile_id() == intent.source_profile_id)
            .ok_or_else(|| refuse("REPLICATION_SOURCE_NOT_ACTIVE"))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReplicationIntentKind {
    Mirror,
}

/// Mirror refs from one profile to another. There is no field for a target
/// ref, candidate, proof root, or integration subject, and the decoder denies
/// unknown fields, so no document of this type can name one.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplicationIntentV1 {
    pub schema_version: String,
    pub intent_kind: ReplicationIntentKind,
    pub source_profile_id: ForgeProfileId,
    pub destination_profile_id: ForgeProfileId,
    pub refs: Vec<String>,
    pub digest: Blake3Digest,
}

impl ReplicationIntentV1 {
    /// Bind the content address and validate; the incoming `digest` is replaced.
    pub fn seal(mut self) -> Result<Self, WireError> {
        self.digest = self.expected_digest()?;
        self.validate()?;
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), WireError> {
        check(
            self.schema_version == FORGE_PROFILE_SCHEMA_VERSION,
            "UNSUPPORTED_REPLICATION_INTENT_SCHEMA",
        )?;
        check(
            self.source_profile_id != self.destination_profile_id,
            "REPLICATION_INTENT_SELF_TARGET",
        )?;
        let count = self.refs.len();
        check(
            count != 0 && count <= MAX_REPLICATION_REFS,
            "INVALID_REPLICATION_REF",
        )?;
        for (index, name) in self.refs.iter().enumerate() {
            check(is_safe_ref(name), "INVALID_REPLICATION_REF")?;
            let prior = index.checked_sub(1).map(|prior| self.refs[prior].as_str());
            check(prior != Some(name.as_str()), "REPLICATION_REF_DUPLICATE")?;
            check(
                prior.is_none_or(|prior| prior < name.as_str()),
                "REPLICATION_REFS_UNSORTED",
            )?;
        }
        check(
            self.digest == self.expected_digest()?,
            "REPLICATION_INTENT_DIGEST_MISMATCH",
        )
    }

    pub fn expected_digest(&self) -> Result<Blake3Digest, WireError> {
        identity_digest(REPLICATION_INTENT_DIGEST_DOMAIN, self)
    }

    /// Replication never proves integration. A consumer that presents an
    /// intent where an integration subject is required receives this typed
    /// refusal regardless of the intent's content.
    pub fn integration_subject(&self) -> Result<IntegrationSubjectBinding, WireError> {
        Err(refuse("REPLICATION_INTENT_NOT_INTEGRATION"))
    }
}

pub fn decode_replication_intent(bytes: &[u8]) -> Result<ReplicationIntentV1, WireError> {
    let intent: ReplicationIntentV1 = decode_canonical(bytes)?;
    intent.validate()?;
    Ok(intent)
}

fn is_safe_ref(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 1024
        && name.starts_with("refs/")
        && !name.contains("..")
        && !name.contains("@{")
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'.' | b'_' | b'-'))
        && name.split('/').all(|segment| {
            !segment.is_empty()
                && !segment.starts_with('.')
                && !segment.ends_with('.')
                && !segment.ends_with(".lock")
        })
}

fn check(condition: bool, code: &'static str) -> Result<(), WireError> {
    if condition { Ok(()) } else { Err(refuse(code)) }
}

/// Every refusal this module emits carries one of these stable codes.
fn refuse(code: &'static str) -> WireError {
    let reason = match code {
        "INVALID_FORGE_PROFILE_ID" => "forge profile id must be fpf_ plus 64 lowercase hex",
        "UNSUPPORTED_FORGE_PROFILE_SCHEMA" => "primary forge profile requires schema v1alpha1",
        "INVALID_FORGE_PROFILE_URL" => "base_url is not a safe https, loopback-http, or file URL",
        "FORGE_PROFILE_URL_NOT_LOOPBACK" => "jeryu base_url must be a loopback host (ADR 0002)",
        "FORGE_PROFILE_URL_NOT_HTTPS" => "plain http is allowed only on a loopback host",
        "FORGE_PROFILE_CAPABILITY_DUPLICATE" => "a capability is declared twice",
        "FORGE_PROFILE_CAPABILITY_UNSORTED" => "capabilities must be in canonical wire order",
        "INVALID_FORGE_PROFILE_GENERATION" => "generation must be a positive interoperable integer",
        "INVALID_FORGE_PROFILE_ACTIVATOR" => "activated_by must be a bounded printable label",
        "INVALID_FORGE_PROFILE_TIME" => "activated_at_unix_ms exceeds the interoperable range",
        "FORGE_PROFILE_DIGEST_MISMATCH" => "profile digest does not bind the profile content",
        "FORGE_PROFILE_LACKS_INTEGRATION_SUBJECT" => "profile lacks integration_subject capability",
        "FORGE_PROFILE_GENERATION_REPLAY" => "this generation is already the active generation",
        "FORGE_PROFILE_GENERATION_REGRESSION" => "a newer generation is already active",
        "REPLICATION_SOURCE_NOT_ACTIVE" => "replication source is not an active primary profile",
        "UNSUPPORTED_REPLICATION_INTENT_SCHEMA" => "replication intent requires schema v1alpha1",
        "REPLICATION_INTENT_SELF_TARGET" => "source and destination must be different profiles",
        "INVALID_REPLICATION_REF" => "refs require 1..=1024 safe fully-qualified names",
        "REPLICATION_REF_DUPLICATE" => "a ref is listed twice",
        "REPLICATION_REFS_UNSORTED" => "refs must be byte-sorted",
        "REPLICATION_INTENT_DIGEST_MISMATCH" => "intent digest does not bind the intent content",
        "REPLICATION_INTENT_NOT_INTEGRATION" => "a replication intent never proves integration",
        _ => "record could not be encoded as canonical JSON",
    };
    WireError::new(code, reason)
}
