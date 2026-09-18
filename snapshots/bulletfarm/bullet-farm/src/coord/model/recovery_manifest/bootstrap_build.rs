#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "COMPONENT_ONLY model awaits separate producer and verifier custody"
    )
)]

use serde::{Deserialize, Serialize};

use crate::coord::{CoordError, generation::manifest::Sha256Digest};

#[path = "bootstrap_contract.rs"]
pub(in crate::coord) mod bootstrap_contract;
use super::{invalid, validate_prefixed};
use bootstrap_contract::{BootstrapContractRefsV1, contract_digests};

const CACHE_KIND: &str = "bullet.coord.recovery-cargo-offline-cache.v1";
const BUILD_KIND: &str = "bullet.coord.recovery-bootstrap-build-observation.v1";
const BUILD_ID_DOMAIN: &str = "bullet-family.coord.recovery-bootstrap-build-observation.v1";
const REGISTRY_SOURCE: &str = "registry+https://github.com/rust-lang/crates.io-index";
const TARGET_TRIPLE: &str = "x86_64-unknown-linux-gnu";
const BUILD_PROFILE: &str = "release";
const NETWORK_MODE: &str = "OFFLINE";
const MAX_PACKAGES: usize = 4_096;
const MAX_PACKAGE_NAME_BYTES: usize = 64;
const MAX_PACKAGE_VERSION_BYTES: usize = 64;
const MAX_REGISTRY_CACHE_ID_BYTES: usize = 128;
const MAX_CRATE_ARCHIVE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_SOURCE_ARCHIVE_BYTES: u64 = 256 * 1024 * 1024;
const MAX_CACHE_ARCHIVE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_CACHE_AGGREGATE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_EXECUTABLE_BYTES: u64 = 512 * 1024 * 1024;

pub(in crate::coord) type CargoPackageArchiveV1 = (String, String, String, u64);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum ComponentEvidenceClassV1 {
    ComponentOnly,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum BuildComparisonV1 {
    ExactBytesEqual,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::coord) struct CargoOfflineCacheManifestV1 {
    kind: String,
    schema_version: u32,
    authority: ComponentEvidenceClassV1,
    pub(in crate::coord) cargo_lock_sha256: String,
    pub(in crate::coord) registry_source: String,
    pub(in crate::coord) registry_cache_id: String,
    pub(in crate::coord) archive_sha256: String,
    pub(in crate::coord) archive_byte_length: u64,
    pub(in crate::coord) tree_sha256: String,
    pub(in crate::coord) package_count: u64,
    pub(in crate::coord) aggregate_byte_length: u64,
    pub(in crate::coord) package_archives: Vec<CargoPackageArchiveV1>,
}

impl CargoOfflineCacheManifestV1 {
    pub(in crate::coord) fn from_observations(
        cargo_lock_sha256: String,
        registry_cache_id: String,
        archive: (String, u64),
        tree_sha256: String,
        package_archives: Vec<CargoPackageArchiveV1>,
    ) -> Result<Self, CoordError> {
        let package_count = u64::try_from(package_archives.len())
            .map_err(|_| invalid("offline Cargo cache package count does not fit u64"))?;
        let aggregate_byte_length = package_archives
            .iter()
            .try_fold(0_u64, |aggregate, (_, _, _, byte_length)| {
                aggregate.checked_add(*byte_length)
            })
            .ok_or_else(|| invalid("offline Cargo cache aggregate byte length overflowed"))?;
        let value = Self {
            kind: CACHE_KIND.to_owned(),
            schema_version: 1,
            authority: ComponentEvidenceClassV1::ComponentOnly,
            cargo_lock_sha256,
            registry_source: REGISTRY_SOURCE.to_owned(),
            registry_cache_id,
            archive_sha256: archive.0,
            archive_byte_length: archive.1,
            tree_sha256,
            package_count,
            aggregate_byte_length,
            package_archives,
        };
        value.validate()?;
        Ok(value)
    }

    pub(in crate::coord) fn validate(&self) -> Result<(), CoordError> {
        if self.kind != CACHE_KIND || self.schema_version != 1 {
            return Err(invalid(
                "offline Cargo cache kind or schema version is unsupported",
            ));
        }
        if self.registry_source != REGISTRY_SOURCE {
            return Err(invalid(
                "offline Cargo cache registry source must be the exact crates.io registry",
            ));
        }
        validate_registry_cache_id(&self.registry_cache_id)?;
        validate_sha256(&self.cargo_lock_sha256, "Cargo.lock SHA-256")?;
        validate_sha256(&self.archive_sha256, "Cargo cache archive SHA-256")?;
        validate_sha256(&self.tree_sha256, "Cargo cache tree SHA-256")?;
        validate_positive_bound(
            self.archive_byte_length,
            MAX_CACHE_ARCHIVE_BYTES,
            "Cargo cache archive byte length",
        )?;
        if self.package_archives.is_empty() || self.package_archives.len() > MAX_PACKAGES {
            return Err(invalid(
                "offline Cargo cache inventory must contain 1..=4,096 packages",
            ));
        }
        if self.package_count != self.package_archives.len() as u64 {
            return Err(invalid(
                "offline Cargo cache package count does not match its inventory",
            ));
        }

        let mut previous: Option<(&str, &str)> = None;
        let mut aggregate = 0_u64;
        for (name, version, sha256, byte_length) in &self.package_archives {
            validate_package_name(name)?;
            validate_package_version(version)?;
            validate_sha256(sha256, "Cargo package archive SHA-256")?;
            validate_positive_bound(
                *byte_length,
                MAX_CRATE_ARCHIVE_BYTES,
                "Cargo package archive byte length",
            )?;
            if previous.is_some_and(|item| item >= (name.as_str(), version.as_str())) {
                return Err(invalid(
                    "offline Cargo cache package identities must be sorted and unique",
                ));
            }
            aggregate = aggregate
                .checked_add(*byte_length)
                .filter(|value| *value <= MAX_CACHE_AGGREGATE_BYTES)
                .ok_or_else(|| {
                    invalid("offline Cargo cache packages exceed 512 MiB in aggregate")
                })?;
            previous = Some((name, version));
        }
        if self.aggregate_byte_length != aggregate {
            return Err(invalid(
                "offline Cargo cache aggregate byte length does not match its inventory",
            ));
        }
        validate_document_size(self)
    }

    pub(in crate::coord) fn sealed_sha256(&self) -> Result<Sha256Digest, CoordError> {
        self.validate()?;
        let mut bytes = canonical_bytes(self)?;
        bytes.push(b'\n');
        Ok(Sha256Digest::for_bytes(&bytes))
    }
}

/// Component-only observation of two contained, locked, offline build outputs.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::coord) struct RecoveryBootstrapBuildObservationV1 {
    kind: String,
    schema_version: u32,
    authority: ComponentEvidenceClassV1,
    observation_id: String,
    bootstrap_provenance_sha256: String,
    source_archive_sha256: String,
    source_archive_byte_length: u64,
    cargo_cache_manifest_sha256: String,
    cargo_cache_archive_sha256: String,
    cargo_cache_archive_byte_length: u64,
    builder_descriptor_sha256: String,
    toolchain_manifest_sha256: String,
    command_contract_sha256: String,
    target_triple: String,
    build_profile: String,
    network_mode: String,
    run_count: u32,
    executable_byte_lengths: [u64; 2],
    executable_sha256: [String; 2],
    comparison: BuildComparisonV1,
}

impl RecoveryBootstrapBuildObservationV1 {
    pub(in crate::coord) fn from_contracts(
        bootstrap_provenance_sha256: String,
        source_archive: (String, u64),
        cargo_cache: (String, String, u64),
        contracts: BootstrapContractRefsV1<'_>,
        executable_runs: [(u64, String); 2],
    ) -> Result<Self, CoordError> {
        let digests = contract_digests(contracts.0, contracts.1, contracts.2)?;
        Self::from_bound_observations(
            bootstrap_provenance_sha256,
            source_archive,
            cargo_cache,
            digests,
            executable_runs,
        )
    }

    #[cfg(test)]
    pub(super) fn from_observations(
        bootstrap_provenance_sha256: String,
        source_archive: (String, u64),
        cargo_cache: (String, String, u64),
        build_contracts: (String, String, String),
        executable_runs: [(u64, String); 2],
    ) -> Result<Self, CoordError> {
        Self::from_bound_observations(
            bootstrap_provenance_sha256,
            source_archive,
            cargo_cache,
            build_contracts,
            executable_runs,
        )
    }

    fn from_bound_observations(
        bootstrap_provenance_sha256: String,
        source_archive: (String, u64),
        cargo_cache: (String, String, u64),
        build_contracts: (String, String, String),
        executable_runs: [(u64, String); 2],
    ) -> Result<Self, CoordError> {
        let mut value = Self {
            kind: BUILD_KIND.to_owned(),
            schema_version: 1,
            authority: ComponentEvidenceClassV1::ComponentOnly,
            observation_id: String::new(),
            bootstrap_provenance_sha256,
            source_archive_sha256: source_archive.0,
            source_archive_byte_length: source_archive.1,
            cargo_cache_manifest_sha256: cargo_cache.0,
            cargo_cache_archive_sha256: cargo_cache.1,
            cargo_cache_archive_byte_length: cargo_cache.2,
            builder_descriptor_sha256: build_contracts.0,
            toolchain_manifest_sha256: build_contracts.1,
            command_contract_sha256: build_contracts.2,
            target_triple: TARGET_TRIPLE.to_owned(),
            build_profile: BUILD_PROFILE.to_owned(),
            network_mode: NETWORK_MODE.to_owned(),
            run_count: 2,
            executable_byte_lengths: [executable_runs[0].0, executable_runs[1].0],
            executable_sha256: [executable_runs[0].1.clone(), executable_runs[1].1.clone()],
            comparison: BuildComparisonV1::ExactBytesEqual,
        };
        value.observation_id = value.expected_id()?;
        value.validate()?;
        Ok(value)
    }

    pub(in crate::coord) fn validate(&self) -> Result<(), CoordError> {
        if self.kind != BUILD_KIND || self.schema_version != 1 {
            return Err(invalid(
                "bootstrap build observation kind or schema version is unsupported",
            ));
        }
        validate_prefixed(&self.observation_id, "rbo_", 64, "build observation ID")?;
        for (value, label) in [
            (
                &self.bootstrap_provenance_sha256,
                "bootstrap provenance SHA-256",
            ),
            (&self.source_archive_sha256, "source archive SHA-256"),
            (
                &self.cargo_cache_manifest_sha256,
                "Cargo cache manifest SHA-256",
            ),
            (
                &self.cargo_cache_archive_sha256,
                "Cargo cache archive SHA-256",
            ),
            (
                &self.builder_descriptor_sha256,
                "builder descriptor SHA-256",
            ),
            (
                &self.toolchain_manifest_sha256,
                "toolchain manifest SHA-256",
            ),
            (
                &self.command_contract_sha256,
                "build command contract SHA-256",
            ),
        ] {
            validate_sha256(value, label)?;
        }
        validate_positive_bound(
            self.source_archive_byte_length,
            MAX_SOURCE_ARCHIVE_BYTES,
            "source archive byte length",
        )?;
        validate_positive_bound(
            self.cargo_cache_archive_byte_length,
            MAX_CACHE_ARCHIVE_BYTES,
            "Cargo cache archive byte length",
        )?;
        if self.target_triple != TARGET_TRIPLE
            || self.build_profile != BUILD_PROFILE
            || self.network_mode != NETWORK_MODE
            || self.run_count != 2
        {
            return Err(invalid(
                "bootstrap build must be exactly two offline release builds for x86_64-unknown-linux-gnu",
            ));
        }
        for (byte_length, sha256) in self
            .executable_byte_lengths
            .iter()
            .zip(&self.executable_sha256)
        {
            validate_positive_bound(
                *byte_length,
                MAX_EXECUTABLE_BYTES,
                "bootstrap executable byte length",
            )?;
            validate_sha256(sha256, "bootstrap executable SHA-256")?;
        }
        if self.executable_byte_lengths[0] != self.executable_byte_lengths[1]
            || self.executable_sha256[0] != self.executable_sha256[1]
        {
            return Err(invalid(
                "bootstrap build outputs must be exactly byte-equal across both runs",
            ));
        }
        if self.observation_id != self.expected_id()? {
            return Err(invalid(
                "bootstrap build observation ID does not bind its exact subject",
            ));
        }
        validate_document_size(self)
    }

    fn expected_id(&self) -> Result<String, CoordError> {
        let identity = BuildObservationIdentityV1 {
            kind: &self.kind,
            schema_version: self.schema_version,
            authority: self.authority,
            bootstrap_provenance_sha256: &self.bootstrap_provenance_sha256,
            source_archive_sha256: &self.source_archive_sha256,
            source_archive_byte_length: self.source_archive_byte_length,
            cargo_cache_manifest_sha256: &self.cargo_cache_manifest_sha256,
            cargo_cache_archive_sha256: &self.cargo_cache_archive_sha256,
            cargo_cache_archive_byte_length: self.cargo_cache_archive_byte_length,
            builder_descriptor_sha256: &self.builder_descriptor_sha256,
            toolchain_manifest_sha256: &self.toolchain_manifest_sha256,
            command_contract_sha256: &self.command_contract_sha256,
            target_triple: &self.target_triple,
            build_profile: &self.build_profile,
            network_mode: &self.network_mode,
            run_count: self.run_count,
            executable_byte_lengths: &self.executable_byte_lengths,
            executable_sha256: &self.executable_sha256,
            comparison: self.comparison,
        };
        let digest = bullet_wire::hash_canonical(BUILD_ID_DOMAIN, &identity).map_err(wire)?;
        Ok(format!("rbo_{}", digest.to_hex()))
    }
}

#[derive(Serialize)]
struct BuildObservationIdentityV1<'a> {
    kind: &'a str,
    schema_version: u32,
    authority: ComponentEvidenceClassV1,
    bootstrap_provenance_sha256: &'a str,
    source_archive_sha256: &'a str,
    source_archive_byte_length: u64,
    cargo_cache_manifest_sha256: &'a str,
    cargo_cache_archive_sha256: &'a str,
    cargo_cache_archive_byte_length: u64,
    builder_descriptor_sha256: &'a str,
    toolchain_manifest_sha256: &'a str,
    command_contract_sha256: &'a str,
    target_triple: &'a str,
    build_profile: &'a str,
    network_mode: &'a str,
    run_count: u32,
    executable_byte_lengths: &'a [u64; 2],
    executable_sha256: &'a [String; 2],
    comparison: BuildComparisonV1,
}

fn validate_sha256(value: &str, label: &str) -> Result<(), CoordError> {
    validate_prefixed(value, "sha256:", 64, label)
}

fn validate_positive_bound(value: u64, maximum: u64, label: &str) -> Result<(), CoordError> {
    if value == 0 || value > maximum {
        return Err(invalid(format!(
            "{label} must be within 1..={maximum} bytes"
        )));
    }
    Ok(())
}

fn validate_registry_cache_id(value: &str) -> Result<(), CoordError> {
    let valid = !value.is_empty()
        && value.len() <= MAX_REGISTRY_CACHE_ID_BYTES
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && value
            .bytes()
            .last()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
        && !value.contains("..");
    if !valid {
        return Err(invalid(
            "Cargo registry cache ID must be a bounded path-safe ASCII atom",
        ));
    }
    Ok(())
}

fn validate_package_name(value: &str) -> Result<(), CoordError> {
    let valid = !value.is_empty()
        && value.len() <= MAX_PACKAGE_NAME_BYTES
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
    if !valid {
        return Err(invalid(
            "Cargo package name must be a bounded ASCII crate name",
        ));
    }
    Ok(())
}

fn validate_package_version(value: &str) -> Result<(), CoordError> {
    let valid = !value.is_empty()
        && value.len() <= MAX_PACKAGE_VERSION_BYTES
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && value
            .bytes()
            .last()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+' | b'_'))
        && !value.contains("..");
    if !valid {
        return Err(invalid(
            "Cargo package version must be a bounded path-safe ASCII atom",
        ));
    }
    Ok(())
}

fn validate_document_size(value: &impl Serialize) -> Result<(), CoordError> {
    let bytes = canonical_bytes(value)?;
    if bytes.len() > bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES {
        return Err(invalid(
            "recovery build component document exceeds the canonical one-MiB bound",
        ));
    }
    Ok(())
}

fn canonical_bytes(value: &impl Serialize) -> Result<Vec<u8>, CoordError> {
    bullet_wire::canonical_json(value).map_err(wire)
}

fn wire(error: impl std::fmt::Display) -> CoordError {
    invalid(format!("cannot bind recovery build component: {error}"))
}

#[cfg(test)]
#[path = "build/tests.rs"]
mod tests;
