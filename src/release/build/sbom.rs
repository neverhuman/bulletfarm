//! CycloneDX 1.6 SBOM production and semantic admission.
//!
//! CycloneDX JSON is the format, not SPDX, because it is the format the
//! committed verifier already requires: `src/release/schema.rs` refuses any
//! package whose SBOM path does not end in `.cdx.json`. ADR 0010 names neither
//! format, so the frozen wire contract decides. `docs/release.md` still calls
//! for "both SBOM formats"; the frozen manifest schema has exactly one SBOM
//! slot per package, so a second document could not be bound or signed and is
//! deliberately not emitted here.

use std::collections::BTreeMap;

use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};

use super::{
    BuildPlan, SUPPORTED_TARGET,
    cargo::{self, RecordedCommand},
    invalid,
    license::{self, AllowList},
    portal::PortalOutput,
};
use crate::coord::CoordError;

const RUST_MEMBERS: [&str; 3] = ["bullet-farm", "bullet-git", "bullet-kernel"];
const MAX_COMPONENTS: usize = 8192;
const MAX_CARGO_METADATA_BYTES: usize = bullet_wire::MAX_UNIQUE_DOCUMENT_BYTES;
const MAX_PACKAGE_LOCK_BYTES: usize = 16 * 1024 * 1024;

pub(super) struct SbomOutput {
    pub(super) relative: String,
    pub(super) size: u64,
    pub(super) digest: String,
    pub(super) component_count: usize,
    pub(super) cargo_components: usize,
    pub(super) npm_components: usize,
}

#[derive(Clone, Debug)]
struct Component {
    purl: String,
    name: String,
    version: String,
    ecosystem: &'static str,
    declared: String,
    expression: String,
    shipped: bool,
    members: Vec<String>,
}

#[derive(Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
}

#[derive(Deserialize)]
struct CargoPackage {
    name: String,
    version: String,
    license: Option<String>,
    source: Option<String>,
    manifest_path: String,
}

#[derive(Deserialize)]
struct NpmLock {
    #[serde(rename = "lockfileVersion")]
    lockfile_version: u32,
    packages: BTreeMap<String, NpmPackage>,
}

#[derive(Deserialize)]
struct NpmPackage {
    version: Option<String>,
    license: Option<String>,
    #[serde(default)]
    dev: bool,
}

pub(super) fn write(
    plan: &BuildPlan,
    portal: &PortalOutput,
    commands: &mut Vec<RecordedCommand>,
) -> Result<SbomOutput, CoordError> {
    let mut lists = Vec::with_capacity(RUST_MEMBERS.len());
    for member in RUST_MEMBERS {
        lists.push(AllowList::read(member, &plan.member(member)?.path)?);
    }
    let policy = AllowList::union(&lists);
    let mut components: BTreeMap<String, Component> = BTreeMap::new();
    for member in RUST_MEMBERS {
        collect_cargo(plan, member, commands, &mut components)?;
    }
    let cargo_components = components.len();
    collect_npm(portal, &mut components)?;
    let npm_components = components.len() - cargo_components;
    if components.is_empty() || components.len() > MAX_COMPONENTS {
        return Err(invalid(
            "the release SBOM component count is outside its admitted bound",
        ));
    }
    for component in components.values() {
        admit(component, &policy)?;
    }
    let document = document(plan, &components);
    let bytes = super::manifest::canonical_bytes(&document)?;
    let relative = format!("{SUPPORTED_TARGET}/{}.cdx.json", plan.stem());
    super::write_new(&plan.out.join(&relative), &bytes)?;
    Ok(SbomOutput {
        relative,
        size: bytes.len() as u64,
        digest: super::digest_bytes(&bytes),
        component_count: components.len(),
        cargo_components,
        npm_components,
    })
}

/// Every component must carry a name, a version, a purl, and a license that the
/// governing committed allow-list admits. Cargo components are governed by the
/// `deny.toml` of every workspace whose locked graph contains them. npm
/// components that are actually shipped are governed by the union of those
/// reviewed lists; the family has no committed npm license policy file yet, so
/// build-only npm components are inventoried and must declare a license, but
/// they are not gated by a policy that does not exist.
fn admit(component: &Component, policy: &AllowList) -> Result<(), CoordError> {
    let subject = &component.purl;
    if component.name.is_empty() || component.name.len() > 256 {
        return Err(invalid(format!("{subject} has no admissible name")));
    }
    if component.version.is_empty() || component.version.len() > 128 {
        return Err(invalid(format!("{subject} has no admissible version")));
    }
    if !component.purl.starts_with("pkg:") {
        return Err(invalid(format!("{subject} has no package URL")));
    }
    if component.ecosystem == "cargo" || component.shipped {
        return license::admit(subject, &component.expression, &[policy]);
    }
    // Build-only npm components: the family has no committed npm license policy
    // to gate them with, so they are inventoried and must declare a license.
    if component.expression.trim().is_empty() {
        return Err(CoordError::new(
            "RELEASE_SBOM_LICENSE_REFUSED",
            format!("{subject} declares no license"),
        ));
    }
    Ok(())
}

fn collect_cargo(
    plan: &BuildPlan,
    member: &str,
    commands: &mut Vec<RecordedCommand>,
    components: &mut BTreeMap<String, Component>,
) -> Result<(), CoordError> {
    let bytes = cargo::metadata(plan, member, commands)?;
    let metadata: CargoMetadata =
        decode_projection(&bytes, "cargo metadata", MAX_CARGO_METADATA_BYTES)?;
    let family_root = plan.family_root.to_str().unwrap_or_default();
    for package in metadata.packages {
        if package.source.is_none() && !package.manifest_path.starts_with(family_root) {
            return Err(invalid(format!(
                "{} has no registry source and is not inside the admitted family",
                package.name
            )));
        }
        let purl = format!("pkg:cargo/{}@{}", package.name, package.version);
        let declared = package.license.unwrap_or_default();
        let entry = components.entry(purl.clone()).or_insert_with(|| Component {
            purl,
            name: package.name,
            version: package.version,
            ecosystem: "cargo",
            expression: license::normalize(&declared),
            declared,
            shipped: true,
            members: Vec::new(),
        });
        if !entry.members.iter().any(|name| name == member) {
            entry.members.push(member.to_owned());
        }
    }
    Ok(())
}

fn collect_npm(
    portal: &PortalOutput,
    components: &mut BTreeMap<String, Component>,
) -> Result<(), CoordError> {
    let lock: NpmLock = decode_projection(
        &portal.package_lock,
        "package-lock.json",
        MAX_PACKAGE_LOCK_BYTES,
    )?;
    if lock.lockfile_version != 3 {
        return Err(CoordError::new(
            "UNSUPPORTED_SCHEMA",
            format!(
                "package-lock.json version {} is unsupported",
                lock.lockfile_version
            ),
        ));
    }
    for (key, package) in lock.packages {
        let Some(name) = key.rsplit_once("node_modules/").map(|(_, name)| name) else {
            continue;
        };
        let version = package.version.unwrap_or_default();
        let purl = format!("pkg:npm/{}@{version}", name.replace('@', "%40"));
        let declared = package.license.unwrap_or_default();
        components.entry(purl.clone()).or_insert_with(|| Component {
            purl,
            name: name.to_owned(),
            version,
            ecosystem: "npm",
            expression: license::normalize(&declared),
            declared,
            shipped: !package.dev,
            members: vec!["bullet-portal".to_owned()],
        });
    }
    Ok(())
}

fn decode_projection<T: DeserializeOwned>(
    bytes: &[u8],
    label: &str,
    max_bytes: usize,
) -> Result<T, CoordError> {
    let value = bullet_wire::decode_unique_value_bounded(bytes, max_bytes)
        .map_err(|error| invalid(format!("{label} is not strict JSON: {error}")))?;
    serde_json::from_value(value)
        .map_err(|error| invalid(format!("{label} does not match its typed schema: {error}")))
}

fn document(plan: &BuildPlan, components: &BTreeMap<String, Component>) -> Value {
    let tools = &plan.tools;
    let mut metadata = Map::new();
    metadata.insert(
        "component".to_owned(),
        json!({
            "bom-ref": format!("bullet-farm@{}", plan.tag),
            "type": "application",
            "name": "bullet-farm",
            "version": plan.tag,
            "description": format!("Bullet Farm {SUPPORTED_TARGET} release archive (unsigned component build)"),
        }),
    );
    metadata.insert(
        "tools".to_owned(),
        json!({
            "components": [
                tool("cargo", &tools.cargo_version),
                tool("git", &tools.git_version),
                tool("node", &tools.node_version),
                tool("npm", &tools.npm_version),
                tool("rustc", &tools.rustc_version),
            ]
        }),
    );
    metadata.insert(
        "properties".to_owned(),
        Value::Array(
            plan.subjects
                .iter()
                .map(|subject| {
                    json!({
                        "name": format!("bullet-farm:subject:{}", subject.name),
                        "value": format!("{} {}", subject.commit_oid, subject.tree_oid),
                    })
                })
                .collect(),
        ),
    );
    json!({
        "bomFormat": "CycloneDX",
        "specVersion": "1.6",
        "version": 1,
        "metadata": Value::Object(metadata),
        "components": components.values().map(component_value).collect::<Vec<_>>(),
    })
}

fn component_value(component: &Component) -> Value {
    json!({
        "bom-ref": component.purl,
        "type": "library",
        "name": component.name,
        "version": component.version,
        "purl": component.purl,
        "scope": if component.shipped { "required" } else { "excluded" },
        "licenses": [{ "expression": component.expression }],
        "properties": [
            { "name": "bullet-farm:ecosystem", "value": component.ecosystem },
            { "name": "bullet-farm:declared-license", "value": component.declared },
            { "name": "bullet-farm:graph", "value": component.members.join(",") },
        ],
    })
}

fn tool(name: &str, version: &str) -> Value {
    json!({ "type": "application", "name": name, "version": version })
}

#[cfg(test)]
mod tests {
    use super::{
        CargoMetadata, Component, MAX_CARGO_METADATA_BYTES, MAX_PACKAGE_LOCK_BYTES, NpmLock, admit,
        decode_projection,
    };
    use crate::release::build::license::AllowList;

    fn policy() -> AllowList {
        AllowList {
            member: "test".to_owned(),
            allowed: ["MIT", "Apache-2.0"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
        }
    }

    fn component(ecosystem: &'static str, license: &str, shipped: bool) -> Component {
        Component {
            purl: format!("pkg:{ecosystem}/thing@1.0.0"),
            name: "thing".to_owned(),
            version: "1.0.0".to_owned(),
            ecosystem,
            declared: license.to_owned(),
            expression: super::license::normalize(license),
            shipped,
            members: vec!["bullet-farm".to_owned()],
        }
    }

    #[test]
    fn a_component_without_a_license_is_a_build_failure() {
        let policy = policy();
        for candidate in [
            component("cargo", "", true),
            component("npm", "", true),
            component("npm", "   ", false),
        ] {
            assert_eq!(
                admit(&candidate, &policy).unwrap_err().code(),
                "RELEASE_SBOM_LICENSE_REFUSED",
                "{}",
                candidate.declared
            );
        }
    }

    #[test]
    fn identity_fields_and_the_allow_list_are_both_enforced() {
        let policy = policy();
        admit(&component("cargo", "MIT/Apache-2.0", true), &policy).expect("legacy slash form");
        assert_eq!(
            admit(&component("cargo", "GPL-3.0-only", true), &policy)
                .unwrap_err()
                .code(),
            "RELEASE_SBOM_LICENSE_REFUSED"
        );
        // A build-only npm component is inventoried, not gated: the family has
        // no committed npm license policy to gate it with.
        admit(&component("npm", "ISC", false), &policy).expect("build-only npm is inventoried");
        assert_eq!(
            admit(&component("npm", "ISC", true), &policy)
                .unwrap_err()
                .code(),
            "RELEASE_SBOM_LICENSE_REFUSED"
        );
        let mut nameless = component("cargo", "MIT", true);
        nameless.name = String::new();
        assert_eq!(
            admit(&nameless, &policy).unwrap_err().code(),
            "INVALID_RELEASE_BUILD_INPUT"
        );
        let mut unversioned = component("cargo", "MIT", true);
        unversioned.version = String::new();
        assert_eq!(
            admit(&unversioned, &policy).unwrap_err().code(),
            "INVALID_RELEASE_BUILD_INPUT"
        );
    }

    #[test]
    fn release_inventory_json_refuses_ambiguous_or_unsafe_projections() {
        let duplicate_metadata = br#"{
            "packages": [{
                "name": "thing",
                "name": "other",
                "version": "1.0.0",
                "license": "MIT",
                "source": null,
                "manifest_path": "/source/Cargo.toml"
            }]
        }"#;
        let error = decode_projection::<CargoMetadata>(
            duplicate_metadata,
            "cargo metadata",
            MAX_CARGO_METADATA_BYTES,
        )
        .err()
        .expect("duplicate cargo metadata members must fail closed");
        assert!(error.to_string().contains("DUPLICATE_JSON_KEY"));

        let mut large_metadata = vec![b' '; bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES + 1];
        large_metadata.extend_from_slice(br#"{"packages":[]}"#);
        assert!(
            decode_projection::<CargoMetadata>(
                &large_metadata,
                "cargo metadata",
                MAX_CARGO_METADATA_BYTES,
            )
            .is_ok(),
            "Cargo inventory preserves its intentional 32 MiB producer allowance"
        );

        let unsafe_lock = br#"{
            "lockfileVersion": 9007199254740992,
            "packages": {}
        }"#;
        let error =
            decode_projection::<NpmLock>(unsafe_lock, "package-lock.json", MAX_PACKAGE_LOCK_BYTES)
                .err()
                .expect("an unsafe lockfile integer must fail closed");
        assert!(error.to_string().contains("UNSAFE_JSON_INTEGER"));
    }
}
