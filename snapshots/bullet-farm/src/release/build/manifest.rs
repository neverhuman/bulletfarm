//! Non-circular build manifest and unsigned in-toto provenance.

use serde_json::{Map, Value, json};

use super::{
    BuildPlan, BundleFile, SUPPORTED_TARGET, archive::ArchiveOutput, cargo::RecordedCommand,
    checksums::ChecksumOutput, invalid, portal::PortalOutput, sbom::SbomOutput, time,
};
use crate::coord::CoordError;

pub(super) const BUILD_MANIFEST_SCHEMA_VERSION: &str = "1";
pub(super) const BUILD_MANIFEST_NAME: &str = "release-build-manifest.json";
const STATEMENT_TYPE: &str = "https://in-toto.io/Statement/v1";
const PREDICATE_TYPE: &str = "https://slsa.dev/provenance/v1";
const BUILD_TYPE: &str = "https://bullet.farm/release-build/x86_64-unknown-linux-gnu/v1";
/// Every key a release build manifest may never contain: a manifest that binds
/// its own digest is circular and cannot be verified from its own bytes.
pub(super) const FORBIDDEN_KEYS: [&str; 4] = [
    "release_build_manifest_digest",
    "manifest_digest",
    "self_digest",
    "digest_of_this_file",
];

/// Everything the build manifest binds, gathered before it is rendered.
pub(super) struct ManifestInput<'a> {
    pub(super) lock: &'a BundleFile,
    pub(super) archive: &'a ArchiveOutput,
    pub(super) sbom: &'a SbomOutput,
    pub(super) provenance: &'a BundleFile,
    pub(super) checksums: &'a ChecksumOutput,
    pub(super) portal: &'a PortalOutput,
    pub(super) commands: &'a [RecordedCommand],
}

pub(super) struct ManifestOutput {
    pub(super) digest: String,
    pub(super) signing_command: String,
}

/// Writes the unsigned in-toto provenance statement. Signing is the operator's
/// OD-E step and this build never fabricates a signature.
pub(super) fn write_provenance(
    plan: &BuildPlan,
    archive: &ArchiveOutput,
    commands: &[RecordedCommand],
) -> Result<BundleFile, CoordError> {
    let statement = json!({
        "_type": STATEMENT_TYPE,
        "subject": [{
            "name": archive.relative,
            "digest": { "blake3": strip_algorithm(&archive.digest)? },
        }],
        "predicateType": PREDICATE_TYPE,
        "predicate": predicate(plan, commands),
    });
    let mut bytes = serde_json::to_vec(&statement).map_err(CoordError::json)?;
    bytes.push(b'\n');
    let relative = format!("{SUPPORTED_TARGET}/{}.intoto.jsonl", plan.stem());
    super::write_new(&plan.out.join(&relative), &bytes)?;
    Ok(BundleFile {
        relative,
        size: bytes.len() as u64,
        digest: super::digest_bytes(&bytes),
    })
}

pub(super) fn write_manifest(
    plan: &BuildPlan,
    input: &ManifestInput<'_>,
) -> Result<ManifestOutput, CoordError> {
    let ManifestInput {
        lock,
        archive,
        sbom,
        provenance,
        checksums,
        portal,
        commands,
    } = *input;
    let mut document = Map::new();
    document.insert(
        "release_build_manifest_schema_version".to_owned(),
        json!(BUILD_MANIFEST_SCHEMA_VERSION),
    );
    document.insert("family".to_owned(), json!("bullet-farm"));
    document.insert("tag".to_owned(), json!(plan.tag));
    document.insert("target".to_owned(), json!(SUPPORTED_TARGET));
    document.insert("signed".to_owned(), json!(false));
    document.insert(
        "expected_release_signing_identity".to_owned(),
        json!(plan.signing_identity),
    );
    document.insert(
        "family_lock".to_owned(),
        json!({
            "schema_version": plan.lock_schema_version,
            "path": lock.relative,
            "size": lock.size,
            "digest": lock.digest,
        }),
    );
    document.insert(
        "source".to_owned(),
        Value::Array(
            plan.subjects
                .iter()
                .map(|subject| {
                    json!({
                        "repository": subject.name,
                        "commit_oid": subject.commit_oid,
                        "tree_oid": subject.tree_oid,
                    })
                })
                .collect(),
        ),
    );
    document.insert(
        "artifact".to_owned(),
        json!({
            "archive": { "path": archive.relative, "size": archive.size, "digest": archive.digest },
            "sbom": { "path": sbom.relative, "size": sbom.size, "digest": sbom.digest },
            "provenance": {
                "path": provenance.relative,
                "size": provenance.size,
                "digest": provenance.digest,
            },
            "checksums": { "path": checksums.relative, "digest": checksums.digest },
        }),
    );
    document.insert(
        "portal_bundle".to_owned(),
        json!({
            "root": portal.root,
            "commit_oid": portal.manifest.source.commit_oid,
            "tree_oid": portal.manifest.source.tree_oid,
            "file_count": portal.manifest.files.len(),
            "total_size": portal.manifest.total_size,
            "package_lock_digest": portal.manifest.package_lock.blake3,
        }),
    );
    document.insert("toolchain".to_owned(), toolchain(plan));
    document.insert("provenance".to_owned(), predicate(plan, commands));
    document.insert(
        "unsatisfied_release_contract".to_owned(),
        json!({
            "required_targets": crate::release::schema::REQUIRED_TARGETS,
            "produced_targets": [SUPPORTED_TARGET],
            "signatures": "absent",
            "blocked_gates": [
                "release.checksums",
                "release.manifest-non-circular",
                "release.package-matrix",
                "release.provenance",
                "release.sbom",
            ],
        }),
    );
    let document = Value::Object(document);
    let bytes = canonical_bytes(&document)?;
    let path = plan.out.join(BUILD_MANIFEST_NAME);
    super::write_new(&path, &bytes)?;
    let reread = std::fs::read(&path).map_err(CoordError::io)?;
    if reread != bytes {
        return Err(CoordError::new(
            "RELEASE_CHECKSUM_MISMATCH",
            "the release build manifest changed between writing and re-reading it",
        ));
    }
    let digest = super::digest_bytes(&reread);
    admit(&reread, &digest)?;
    let signing_command = signing_instructions(plan, archive, sbom, provenance)?;
    Ok(ManifestOutput {
        digest,
        signing_command,
    })
}

/// Refuses any release build manifest that binds its own digest. The manifest
/// is verifiable only from bytes that exist before it does.
pub(super) fn admit(bytes: &[u8], digest: &str) -> Result<(), CoordError> {
    let document: Value = bullet_wire::decode_unique_value(bytes)
        .map_err(|error| circular(format!("release build manifest JSON is ambiguous: {error}")))?;
    let mut stack = vec![&document];
    while let Some(value) = stack.pop() {
        match value {
            Value::Object(map) => {
                for (key, child) in map {
                    if FORBIDDEN_KEYS.contains(&key.as_str()) {
                        return Err(circular(format!(
                            "release build manifest binds its own digest through {key}"
                        )));
                    }
                    stack.push(child);
                }
            }
            Value::Array(items) => stack.extend(items.iter()),
            Value::String(text) if text == digest => {
                return Err(circular(
                    "release build manifest contains its own digest as a value",
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn signing_instructions(
    plan: &BuildPlan,
    archive: &ArchiveOutput,
    sbom: &SbomOutput,
    provenance: &BundleFile,
) -> Result<String, CoordError> {
    let mut lines = String::from(
        "# Operator signing (OD-E). This build signs nothing and holds no key.\n\
         # Namespace is fixed at bullet-farm-release; sign each payload separately.\n",
    );
    for relative in [
        archive.relative.as_str(),
        sbom.relative.as_str(),
        provenance.relative.as_str(),
    ] {
        lines.push_str(&format!(
            "ssh-keygen -Y sign -f /absolute/path/to/release-signing-key -n bullet-farm-release {}\n",
            plan.out.join(relative).display()
        ));
    }
    lines.push_str(
        "# release-manifest.toml cannot be assembled from this bundle: the frozen schema in \
         src/release/schema.rs requires all five byte-sorted targets and a schema-3 family.lock. \
         This build produced one target and the checked-in lock is schema 2.\n",
    );
    let path = plan.out.join("SIGNING.txt");
    super::write_new(&path, lines.as_bytes())?;
    Ok(path.display().to_string())
}

fn predicate(plan: &BuildPlan, commands: &[RecordedCommand]) -> Value {
    json!({
        "buildDefinition": {
            "buildType": BUILD_TYPE,
            "externalParameters": {
                "target": SUPPORTED_TARGET,
                "tag": plan.tag,
                "offline": plan.offline,
                "out": plan.out.display().to_string(),
            },
            "resolvedDependencies": plan
                .subjects
                .iter()
                .map(|subject| json!({
                    "name": subject.name,
                    "uri": format!("git+file://{}", subject.path.display()),
                    "digest": { "gitCommit": subject.commit_oid, "gitTree": subject.tree_oid },
                }))
                .collect::<Vec<_>>(),
            "internalParameters": {
                "toolchain": toolchain(plan),
                "command": commands
                    .iter()
                    .map(|command| json!({
                        "program": command.program,
                        "args": command.args,
                        "cwd": command.cwd,
                        "env": command.env.iter().map(|(name, value)| json!([name, value])).collect::<Vec<_>>(),
                    }))
                    .collect::<Vec<_>>(),
            },
        },
        "runDetails": {
            "builder": {
                "id": plan.builder_identity,
                "version": { "bullet-family": env!("CARGO_PKG_VERSION") },
            },
            "metadata": {
                "invocationId": plan.invocation_id,
                "startedOn": time::rfc3339_utc(plan.started_at),
                "signed": false,
            },
        },
    })
}

fn toolchain(plan: &BuildPlan) -> Value {
    let tools = &plan.tools;
    json!({
        "cargo": tools.cargo_version,
        "cargo_path": tools.cargo.display().to_string(),
        "git": tools.git_version,
        "node": tools.node_version,
        "npm": tools.npm_version,
        "rustc": tools.rustc_version,
        "rustc_path": tools.rustc.display().to_string(),
    })
}

/// Canonical JSON for this family: UTF-8, compact, byte-sorted object keys
/// (`serde_json::Map` is a `BTreeMap` here), one trailing newline.
pub(super) fn canonical_bytes(document: &Value) -> Result<Vec<u8>, CoordError> {
    let mut bytes = serde_json::to_vec(document).map_err(CoordError::json)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn strip_algorithm(digest: &str) -> Result<String, CoordError> {
    digest
        .strip_prefix("blake3:")
        .map(str::to_owned)
        .ok_or_else(|| invalid("release digests must be algorithm-tagged BLAKE3"))
}

fn circular(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RELEASE_BUILD_MANIFEST", reason)
}
