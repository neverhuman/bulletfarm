use std::{
    collections::BTreeSet,
    io::{Cursor, Read},
    path::{Path, PathBuf},
};

use serde::Serialize;
use sha2::{Digest, Sha256};
use tar::{Archive, EntryType};

use super::{provenance::archive, require_normalized_absolute};
use crate::coord::{
    CoordError, RecoveryBootstrapProvenanceV1,
    model::{
        CargoOfflineCacheManifestV1, RecoveryBootstrapBuildObservationV1,
        RecoveryBootstrapBuilderContractV1, RecoveryBootstrapCommandContractV1,
        RecoveryBootstrapToolchainContractV1,
    },
};

const CACHE_MAX_BYTES: u64 = 512 * 1024 * 1024;
const CRATE_MAX_BYTES: u64 = 64 * 1024 * 1024;
const EXECUTABLE_MAX_BYTES: u64 = 512 * 1024 * 1024;
const CACHE_TREE_DOMAIN: &[u8] = b"bullet-family.recovery-cargo-cache-tree.v1\0";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::coord) struct RecoveryBootstrapBuildVerifyCommand {
    pub(in crate::coord) provenance: PathBuf,
    pub(in crate::coord) source_archive: PathBuf,
    pub(in crate::coord) builder_contract: PathBuf,
    pub(in crate::coord) toolchain_contract: PathBuf,
    pub(in crate::coord) command_contract: PathBuf,
    pub(in crate::coord) cache_manifest: PathBuf,
    pub(in crate::coord) cache_archive: PathBuf,
    pub(in crate::coord) executable_run_1: PathBuf,
    pub(in crate::coord) executable_run_2: PathBuf,
    pub(in crate::coord) output: PathBuf,
}

/// Verify and create-once seal the component observation, returning only its
/// derived identity to the path-only CLI. No caller-selected fact enters the
/// observation.
pub(crate) fn seal_bootstrap_build_observation(paths: [PathBuf; 10]) -> Result<String, CoordError> {
    let [
        provenance,
        source_archive,
        builder_contract,
        toolchain_contract,
        command_contract,
        cache_manifest,
        cache_archive,
        executable_run_1,
        executable_run_2,
        output,
    ] = paths;
    let command = RecoveryBootstrapBuildVerifyCommand {
        provenance,
        source_archive,
        builder_contract,
        toolchain_contract,
        command_contract,
        cache_manifest,
        cache_archive,
        executable_run_1,
        executable_run_2,
        output,
    };
    let (_, observation_id) = verify_and_seal_with_identity(&command)?;
    Ok(observation_id)
}

#[cfg(test)]
fn verify_and_seal_bootstrap_build_observation(
    command: &RecoveryBootstrapBuildVerifyCommand,
) -> Result<RecoveryBootstrapBuildObservationV1, CoordError> {
    let (observation, _) = verify_and_seal_with_identity(command)?;
    Ok(observation)
}

fn observation_id(observation: &RecoveryBootstrapBuildObservationV1) -> Result<String, CoordError> {
    let value = serde_json::to_value(observation).map_err(|error| {
        invalid(format!(
            "cannot render bootstrap build observation: {error}"
        ))
    })?;
    value
        .get("observation_id")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| invalid("bootstrap build observation has no derived identity"))
}

fn verify_and_seal_with_identity(
    command: &RecoveryBootstrapBuildVerifyCommand,
) -> Result<(RecoveryBootstrapBuildObservationV1, String), CoordError> {
    validate_paths(command)?;
    let provenance: RecoveryBootstrapProvenanceV1 =
        crate::coord::sealed::read(&command.provenance)?;
    let builder: RecoveryBootstrapBuilderContractV1 =
        crate::coord::sealed::read(&command.builder_contract)?;
    let toolchain: RecoveryBootstrapToolchainContractV1 =
        crate::coord::sealed::read(&command.toolchain_contract)?;
    let build_command: RecoveryBootstrapCommandContractV1 =
        crate::coord::sealed::read(&command.command_contract)?;
    let cache: CargoOfflineCacheManifestV1 = crate::coord::sealed::read(&command.cache_manifest)?;
    provenance.validate()?;
    builder.validate()?;
    toolchain.validate()?;
    build_command.validate()?;
    cache.validate()?;

    let source =
        crate::coord::sealed::read_raw(&command.source_archive, archive::MAX_ARCHIVE_BYTES as u64)?;
    let source_facts = raw_facts(&source)?;
    if source_facts.0 != provenance.archive_sha256 {
        return Err(invalid(
            "retained source TAR SHA-256 differs from provenance",
        ));
    }
    archive::verify_retained_source(&source, &provenance)?;
    drop(source);

    let cache_bytes = crate::coord::sealed::read_raw(&command.cache_archive, CACHE_MAX_BYTES)?;
    verify_cache_archive(&cache_bytes, &cache, &provenance)?;
    let cache_facts = raw_facts(&cache_bytes)?;
    drop(cache_bytes);

    let run_1 = crate::coord::sealed::read_raw(&command.executable_run_1, EXECUTABLE_MAX_BYTES)?;
    let run_2 = crate::coord::sealed::read_raw(&command.executable_run_2, EXECUTABLE_MAX_BYTES)?;
    if run_1 != run_2 {
        return Err(invalid(
            "bootstrap executable outputs are not byte-for-byte equal",
        ));
    }
    let executable_facts = raw_facts(&run_1)?;
    if executable_facts.0 != provenance.executable_sha256
        || executable_facts.1 != provenance.executable_byte_length
    {
        return Err(invalid(
            "bootstrap executable bytes differ from sealed provenance",
        ));
    }

    let observation = RecoveryBootstrapBuildObservationV1::from_contracts(
        sealed_sha256(&provenance)?,
        (source_facts.0, source_facts.1),
        (
            cache.sealed_sha256()?.as_str().to_owned(),
            cache_facts.0,
            cache_facts.1,
        ),
        (&builder, &toolchain, &build_command),
        [
            (executable_facts.1, executable_facts.0.clone()),
            (executable_facts.1, executable_facts.0),
        ],
    )?;
    let observation_id = observation_id(&observation)?;
    crate::coord::sealed::write(&command.output, &observation)?;
    let persisted: RecoveryBootstrapBuildObservationV1 =
        crate::coord::sealed::read(&command.output)?;
    persisted.validate()?;
    if persisted != observation {
        return Err(changed(
            "sealed bootstrap build observation changed during read-back",
        ));
    }
    Ok((observation, observation_id))
}

fn validate_paths(command: &RecoveryBootstrapBuildVerifyCommand) -> Result<(), CoordError> {
    let paths: [(&Path, &'static str); 10] = [
        (&command.provenance, "bootstrap provenance"),
        (&command.source_archive, "source archive"),
        (&command.builder_contract, "builder contract"),
        (&command.toolchain_contract, "toolchain contract"),
        (&command.command_contract, "command contract"),
        (&command.cache_manifest, "Cargo cache manifest"),
        (&command.cache_archive, "Cargo cache archive"),
        (&command.executable_run_1, "first executable"),
        (&command.executable_run_2, "second executable"),
        (&command.output, "build observation output"),
    ];
    let mut seen = BTreeSet::new();
    for (path, label) in paths {
        require_normalized_absolute(path, label)?;
        if !seen.insert(path) {
            return Err(invalid("bootstrap build paths must be pairwise distinct"));
        }
    }
    Ok(())
}

fn verify_cache_archive(
    bytes: &[u8],
    cache: &CargoOfflineCacheManifestV1,
    provenance: &RecoveryBootstrapProvenanceV1,
) -> Result<(), CoordError> {
    let byte_length = u64::try_from(bytes.len())
        .map_err(|_| invalid("Cargo cache archive length cannot be represented"))?;
    if byte_length == 0
        || byte_length > CACHE_MAX_BYTES
        || !bytes.len().is_multiple_of(512)
        || cache.archive_byte_length != byte_length
        || cache.archive_sha256 != digest(bytes)
    {
        return Err(invalid(
            "Cargo cache archive length or SHA-256 differs from manifest",
        ));
    }
    if cache.cargo_lock_sha256 != provenance.cargo_lock_sha256 {
        return Err(invalid(
            "Cargo cache manifest does not bind the provenance Cargo.lock",
        ));
    }
    let tree_sha256 = cache_tree_sha256(&cache.registry_cache_id, &cache.package_archives)?;
    if cache.tree_sha256 != tree_sha256 {
        return Err(invalid(
            "Cargo cache tree digest does not bind its inventory",
        ));
    }
    archive::verify_raw_archive(bytes, &tree_sha256)?;
    let mut tar = Archive::new(Cursor::new(bytes));
    let mut package_index = 0_usize;
    let mut paths = BTreeSet::new();
    for item in tar.entries().map_err(tar_error)? {
        let mut entry = item.map_err(tar_error)?;
        if entry.header().entry_type() == EntryType::XGlobalHeader {
            continue;
        }
        if entry.header().entry_type() != EntryType::Regular || entry.link_name_bytes().is_some() {
            return Err(invalid(
                "Cargo cache TAR contains a directory, link, extension, or special member",
            ));
        }
        let (name, version, sha256, expected_length) = cache
            .package_archives
            .get(package_index)
            .ok_or_else(|| invalid("Cargo cache TAR contains an unlisted package"))?;
        let expected_path = format!(
            "registry/cache/{}/{}-{}.crate",
            cache.registry_cache_id, name, version
        );
        let path = std::str::from_utf8(&entry.path_bytes())
            .map_err(|_| invalid("Cargo cache TAR path is not UTF-8"))?
            .to_owned();
        if crate::coord::validate_path(&path).ok().as_deref() != Some(path.as_str())
            || path != expected_path
            || !paths.insert(path)
            || entry.header().mode().map_err(tar_error)? & 0o7777 != 0o444
            || entry.header().uid().map_err(tar_error)? != 0
            || entry.header().gid().map_err(tar_error)? != 0
            || entry.size() != *expected_length
        {
            return Err(invalid(
                "Cargo cache TAR path, order, custody, mode, or length differs from inventory",
            ));
        }
        let mut content = Vec::new();
        (&mut entry)
            .take(CRATE_MAX_BYTES + 1)
            .read_to_end(&mut content)
            .map_err(tar_error)?;
        if content.len() as u64 != *expected_length || digest(&content) != *sha256 {
            return Err(invalid(
                "Cargo cache package bytes differ from the manifest inventory",
            ));
        }
        package_index += 1;
    }
    if package_index != cache.package_archives.len() {
        return Err(invalid("Cargo cache TAR omits manifest packages"));
    }
    Ok(())
}

fn cache_tree_sha256(
    registry_cache_id: &str,
    packages: &[(String, String, String, u64)],
) -> Result<String, CoordError> {
    let canonical = bullet_wire::canonical_json(&(registry_cache_id, packages))
        .map_err(|error| invalid(format!("cannot bind Cargo cache tree: {error}")))?;
    let mut hasher = Sha256::new();
    hasher.update(CACHE_TREE_DOMAIN);
    hasher.update(canonical);
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

fn raw_facts(bytes: &[u8]) -> Result<(String, u64), CoordError> {
    let length = u64::try_from(bytes.len())
        .map_err(|_| invalid("recovery artifact length cannot be represented"))?;
    Ok((digest(bytes), length))
}

fn sealed_sha256(value: &impl Serialize) -> Result<String, CoordError> {
    let mut bytes = bullet_wire::canonical_json(value)
        .map_err(|error| invalid(format!("cannot bind sealed recovery record: {error}")))?;
    bytes.push(b'\n');
    Ok(digest(&bytes))
}

fn digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn tar_error(error: impl std::fmt::Display) -> CoordError {
    invalid(format!("invalid Cargo cache TAR: {error}"))
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RECOVERY_PRODUCTION", reason)
}

fn changed(reason: impl Into<String>) -> CoordError {
    CoordError::new("COORD_SUBJECT_CHANGED", reason)
}

#[cfg(test)]
#[path = "bootstrap_build/tests.rs"]
mod tests;
