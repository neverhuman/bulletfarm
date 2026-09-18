use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use bullet_wire::{
    MAX_CANONICAL_DOCUMENT_BYTES, MAX_UNIQUE_DOCUMENT_BYTES, PolicyTemplateV1, canonical_json,
    decode_canonical, decode_canonical_value, decode_unique_value, decode_unique_value_bounded,
    hash_framed_bytes,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap()
        .to_path_buf()
}

fn normalized_lf(source: &str) -> Result<Cow<'_, str>, &'static str> {
    if !source.contains('\r') {
        return Ok(Cow::Borrowed(source));
    }
    let normalized = source.replace("\r\n", "\n");
    if normalized.contains('\r') {
        return Err("source contains a carriage return outside CRLF");
    }
    Ok(Cow::Owned(normalized))
}

fn independent_framed_digest(domain: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"bullet-wire.v1\0");
    for subject in [domain.as_bytes(), bytes] {
        hasher.update(&(subject.len() as u64).to_le_bytes());
        hasher.update(subject);
    }
    hasher.finalize().to_hex().to_string()
}

fn portable_relative_path(path: &Path) -> Result<String, &'static str> {
    path.components()
        .map(|component| match component {
            std::path::Component::Normal(value) => {
                value.to_str().ok_or("relative path component is not UTF-8")
            }
            _ => Err("relative path contains a non-normal component"),
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|components| components.join("/"))
}

fn cargo_metadata(manifest: &Path, locked: bool) -> serde_json::Value {
    let mut command = Command::new(env!("CARGO"));
    command
        .args([
            "metadata",
            "--offline",
            "--format-version",
            "1",
            "--no-deps",
        ])
        .arg("--manifest-path")
        .arg(manifest);
    if locked {
        command.arg("--locked");
    }
    let output = command.output().expect("run cargo metadata");
    assert!(
        output.status.success(),
        "cargo metadata failed for {}: {}",
        manifest.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("cargo metadata emits JSON")
}

fn cargo_metadata_with_dependencies(manifest: &Path) -> serde_json::Value {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--locked", "--offline", "--format-version", "1"])
        .arg("--manifest-path")
        .arg(manifest)
        .output()
        .expect("run full cargo metadata");
    assert!(
        output.status.success(),
        "full cargo metadata failed for {}: {}",
        manifest.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("full cargo metadata emits JSON")
}

fn serde_json_dependency_inventory(metadata: &serde_json::Value) -> Vec<(String, Option<String>)> {
    let mut inventory = Vec::new();
    for package in metadata["packages"]
        .as_array()
        .expect("cargo metadata packages array")
    {
        let package_name = package["name"].as_str().expect("package name");
        for dependency in package["dependencies"]
            .as_array()
            .expect("package dependency array")
        {
            if dependency["name"].as_str() != Some("serde_json") {
                continue;
            }
            let rename = match dependency.get("rename") {
                Some(serde_json::Value::Null) => None,
                Some(serde_json::Value::String(rename)) => Some(rename.clone()),
                _ => Some("<invalid-rename-field>".to_owned()),
            };
            inventory.push((package_name.to_owned(), rename));
        }
    }
    inventory.sort();
    inventory
}

type CargoTargetInventory = Vec<(String, String, Vec<String>, String)>;

fn non_test_target_inventory(
    metadata: &serde_json::Value,
    family: &Path,
) -> Result<CargoTargetInventory, &'static str> {
    let family = fs::canonicalize(family).map_err(|_| "family root is not canonicalizable")?;
    let mut inventory = Vec::new();
    for package in metadata["packages"]
        .as_array()
        .ok_or("cargo metadata packages array missing")?
    {
        let package_name = package["name"].as_str().ok_or("package name missing")?;
        for target in package["targets"]
            .as_array()
            .ok_or("package targets array missing")?
        {
            let mut kinds = target["kind"]
                .as_array()
                .ok_or("target kind array missing")?
                .iter()
                .map(|kind| {
                    kind.as_str()
                        .ok_or("target kind is not text")
                        .map(str::to_owned)
                })
                .collect::<Result<Vec<_>, _>>()?;
            if kinds
                .iter()
                .all(|kind| matches!(kind.as_str(), "test" | "example" | "bench"))
            {
                continue;
            }
            kinds.sort();
            let source = fs::canonicalize(Path::new(
                target["src_path"]
                    .as_str()
                    .ok_or("target source path missing")?,
            ))
            .map_err(|_| "non-test Cargo target source is missing")?;
            let relative = source
                .strip_prefix(&family)
                .map_err(|_| "non-test Cargo target escapes the family root")?;
            inventory.push((
                package_name.to_owned(),
                target["name"]
                    .as_str()
                    .ok_or("target name missing")?
                    .to_owned(),
                kinds,
                portable_relative_path(relative)?,
            ));
        }
    }
    inventory.sort();
    Ok(inventory)
}

fn proc_macro_inventory(metadata: &serde_json::Value) -> Vec<(String, String, Option<String>)> {
    let mut inventory = metadata["packages"]
        .as_array()
        .expect("cargo metadata packages array")
        .iter()
        .filter(|package| {
            package["targets"].as_array().is_some_and(|targets| {
                targets.iter().any(|target| {
                    target["kind"]
                        .as_array()
                        .is_some_and(|kinds| kinds.iter().any(|kind| kind == "proc-macro"))
                })
            })
        })
        .map(|package| {
            (
                package["name"].as_str().expect("package name").to_owned(),
                package["version"]
                    .as_str()
                    .expect("package version")
                    .to_owned(),
                package["source"].as_str().map(str::to_owned),
            )
        })
        .collect::<Vec<_>>();
    inventory.sort();
    inventory
}

type DirectDependencyInventory = Vec<(
    String,
    String,
    Option<String>,
    String,
    Option<String>,
    bool,
    bool,
    Vec<String>,
    Option<String>,
    String,
)>;

fn workspace_direct_dependency_inventory(
    metadata: &serde_json::Value,
    family: &Path,
) -> Result<(usize, String), &'static str> {
    let family = fs::canonicalize(family).map_err(|_| "family root is not canonicalizable")?;
    let members = metadata["workspace_members"]
        .as_array()
        .ok_or("cargo metadata workspace members missing")?
        .iter()
        .map(|member| member.as_str().ok_or("workspace member ID is not text"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let mut inventory: DirectDependencyInventory = Vec::new();
    for package in metadata["packages"]
        .as_array()
        .ok_or("cargo metadata packages array missing")?
    {
        let package_id = package["id"].as_str().ok_or("package ID missing")?;
        if !members.contains(package_id) {
            continue;
        }
        let package_name = package["name"].as_str().ok_or("package name missing")?;
        for dependency in package["dependencies"]
            .as_array()
            .ok_or("package dependency array missing")?
        {
            let mut features = dependency["features"]
                .as_array()
                .ok_or("dependency feature array missing")?
                .iter()
                .map(|feature| {
                    feature
                        .as_str()
                        .ok_or("dependency feature is not text")
                        .map(str::to_owned)
                })
                .collect::<Result<Vec<_>, _>>()?;
            features.sort();
            let subject = match (dependency["source"].as_str(), dependency["path"].as_str()) {
                (Some(source), None) => format!("source:{source}"),
                (None, Some(path)) => {
                    let dependency_path = fs::canonicalize(path)
                        .map_err(|_| "workspace dependency path is not canonicalizable")?;
                    let relative = dependency_path
                        .strip_prefix(&family)
                        .map_err(|_| "workspace dependency path escapes the family root")?;
                    format!("path:{}", portable_relative_path(relative)?)
                }
                _ => return Err("dependency must have exactly one source or path subject"),
            };
            inventory.push((
                package_name.to_owned(),
                dependency["name"]
                    .as_str()
                    .ok_or("dependency name missing")?
                    .to_owned(),
                dependency["rename"].as_str().map(str::to_owned),
                dependency["req"]
                    .as_str()
                    .ok_or("dependency requirement missing")?
                    .to_owned(),
                dependency["kind"].as_str().map(str::to_owned),
                dependency["optional"]
                    .as_bool()
                    .ok_or("dependency optional flag missing")?,
                dependency["uses_default_features"]
                    .as_bool()
                    .ok_or("dependency default-feature flag missing")?,
                features,
                dependency["target"].as_str().map(str::to_owned),
                subject,
            ));
        }
    }
    inventory.sort();
    let encoded = serde_jcs::to_vec(&inventory)
        .map_err(|_| "workspace dependency inventory is not canonicalizable")?;
    let digest = independent_framed_digest("hostile.workspace-direct-dependencies-v1", &encoded);
    Ok((inventory.len(), digest))
}

fn hostile_renamed_dependency_metadata() -> serde_json::Value {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after epoch")
        .as_nanos();
    let fixture = std::env::temp_dir().join(format!(
        "bullet-serde-rename-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(fixture.join("src")).expect("create metadata fixture");
    fs::write(
        fixture.join("Cargo.toml"),
        r#"[package]
name = "hostile-metadata"
version = "0.0.0"
edition = "2024"

[dependencies.json]
package = "serde_\u006ason"
version = "1"

[workspace]
"#,
    )
    .expect("write metadata fixture manifest");
    fs::write(fixture.join("src/lib.rs"), "pub fn fixture() {}\n")
        .expect("write metadata fixture source");
    let metadata = cargo_metadata(&fixture.join("Cargo.toml"), false);
    fs::remove_dir_all(&fixture).expect("remove metadata fixture");
    metadata
}

fn excluded_root_tree(root: &Path, path: &Path) -> bool {
    path.parent() == Some(root)
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                matches!(
                    name,
                    ".git" | "target" | ".ci-artifacts" | ".ci-tools" | ".jankurai" | ".fusion"
                )
            })
}

fn collect_rust_sources(root: &Path, path: &Path, sources: &mut Vec<PathBuf>) {
    if excluded_root_tree(root, path) {
        return;
    }
    let mut entries = fs::read_dir(path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
        .map(|entry| entry.expect("source directory entry").path())
        .collect::<Vec<_>>();
    entries.sort();
    for entry in entries {
        let metadata = fs::symlink_metadata(&entry)
            .unwrap_or_else(|error| panic!("inspect {}: {error}", entry.display()));
        assert!(
            !metadata.file_type().is_symlink(),
            "source inventory refuses symlink {}",
            entry.display()
        );
        if metadata.is_dir() {
            collect_rust_sources(root, &entry, sources);
        } else if metadata.is_file()
            && entry.extension().and_then(|value| value.to_str()) == Some("rs")
        {
            sources.push(entry);
        }
    }
}

fn is_production_rust(relative: &std::path::Path) -> bool {
    if relative == std::path::Path::new("contracts/generated/rust/schema_bundle.rs") {
        return true;
    }
    let components = relative
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>();
    components.first() == Some(&"src")
        || (components.first() == Some(&"crates") && components.get(2) == Some(&"src"))
}

const UNICODE_15_1_DEFAULT_IGNORABLE_INTERVALS: &[(u32, u32)] = &[
    (0x00ad, 0x00ad),
    (0x034f, 0x034f),
    (0x061c, 0x061c),
    (0x115f, 0x1160),
    (0x17b4, 0x17b5),
    (0x180b, 0x180f),
    (0x200b, 0x200f),
    (0x202a, 0x202e),
    (0x2060, 0x206f),
    (0x3164, 0x3164),
    (0xfe00, 0xfe0f),
    (0xfeff, 0xfeff),
    (0xffa0, 0xffa0),
    (0xfff0, 0xfff8),
    (0x1bca0, 0x1bca3),
    (0x1d173, 0x1d17a),
    (0xe0000, 0xe0fff),
];
