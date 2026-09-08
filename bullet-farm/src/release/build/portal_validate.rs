use std::cmp::Ordering;
use std::collections::BTreeSet;

use serde_json::Value;

use super::{
    BuildPlan, MAX_FILE_BYTES, MAX_FILES, MAX_PACKAGE_LOCK_BYTES, MAX_TOOL_BYTES,
    MAX_TOOL_TREE_BYTES, MAX_TOOL_TREE_FILES, MAX_TOTAL_BYTES, PORTAL_ROOT_DOMAIN, PortalFile,
    PortalManifest, PortalTool,
};
use super::super::invalid;
use crate::coord::CoordError;

pub(super) fn validate_raw_tool_shapes(value: &Value) -> Result<(), CoordError> {
    let tools = value
        .get("tools")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                "Portal tool subjects must be a JSON array",
            )
        })?;
    let expected: [(&str, &[&str]); 3] = [
        ("git", &["blake3", "name", "size", "version"]),
        (
            "node",
            &[
                "architecture",
                "blake3",
                "name",
                "platform",
                "size",
                "version",
            ],
        ),
        ("npm", &["blake3", "file_count", "name", "size", "version"]),
    ];
    if tools.len() != expected.len() {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal tool subjects must contain exactly git, node, and npm",
        ));
    }
    for (index, tool) in tools.iter().enumerate() {
        let object = tool.as_object().ok_or_else(|| {
            CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                "Portal tool subject must be a JSON object",
            )
        })?;
        let (name, keys) = expected[index];
        let expected_keys = keys.iter().copied().collect::<BTreeSet<_>>();
        let actual_keys = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
        if object.get("name").and_then(Value::as_str) != Some(name) || actual_keys != expected_keys
        {
            return Err(CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                format!("Portal {name} tool subject has producer-impossible keys"),
            ));
        }
    }
    Ok(())
}

pub(super) fn manifest_root(mut value: Value) -> Result<String, CoordError> {
    let object = value.as_object_mut().ok_or_else(|| {
        CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal bundle manifest must be a JSON object",
        )
    })?;
    if object.remove("root").is_none() {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal bundle manifest root is missing",
        ));
    }
    let body = bullet_wire::canonical_json(&value).map_err(|error| {
        CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            format!("Portal bundle manifest body cannot be canonicalized: {error}"),
        )
    })?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(PORTAL_ROOT_DOMAIN);
    hasher.update(&body);
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

pub(super) fn validate_manifest_semantics(manifest: &PortalManifest) -> Result<(), CoordError> {
    if manifest.schema_version != "bullet.portal.bundle.v1" {
        return Err(CoordError::new(
            "UNSUPPORTED_SCHEMA",
            format!(
                "Portal bundle manifest schema {} is unsupported",
                manifest.schema_version
            ),
        ));
    }
    if manifest.source.repository != "bullet-portal" {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal bundle manifest names the wrong source repository",
        ));
    }
    crate::release::schema::validate_oid("Portal source commit", &manifest.source.commit_oid)?;
    crate::release::schema::validate_oid("Portal source tree", &manifest.source.tree_oid)?;
    if manifest.package_lock.path != "package-lock.json"
        || manifest.package_lock.size > MAX_PACKAGE_LOCK_BYTES
    {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal package-lock subject is outside its producer contract",
        ));
    }
    for digest in std::iter::once(&manifest.root)
        .chain(std::iter::once(&manifest.package_lock.blake3))
        .chain(manifest.tools.iter().map(|tool| &tool.blake3))
        .chain(manifest.files.iter().map(|file| &file.blake3))
    {
        crate::release::schema::validate_digest(digest)?;
    }
    validate_tools(&manifest.tools)?;
    validate_file_inventory(&manifest.files, manifest.total_size)
}

pub(super) fn validate_tools(tools: &[PortalTool]) -> Result<(), CoordError> {
    if tools.len() != 3
        || tools
            .iter()
            .map(|tool| tool.name.as_str())
            .ne(["git", "node", "npm"])
    {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal tool subjects must be ordered exactly as git, node, npm",
        ));
    }
    let git = &tools[0];
    let node = &tools[1];
    let npm = &tools[2];
    let git_valid = exact_numeric_version(&git.version, "git version ", 3, 4)
        && (1..=MAX_TOOL_BYTES).contains(&git.size)
        && git.platform.is_none()
        && git.architecture.is_none()
        && git.file_count.is_none();
    let node_valid = exact_numeric_version(&node.version, "v", 3, 3)
        && (1..=MAX_TOOL_BYTES).contains(&node.size)
        && node.platform.as_deref() == Some("linux")
        && node.architecture.as_deref() == Some("x64")
        && node.file_count.is_none();
    let npm_valid = exact_numeric_version(&npm.version, "", 3, 3)
        && (1..=MAX_TOOL_TREE_BYTES).contains(&npm.size)
        && npm.platform.is_none()
        && npm.architecture.is_none()
        && npm
            .file_count
            .is_some_and(|count| (1..=MAX_TOOL_TREE_FILES).contains(&count));
    if !git_valid || !node_valid || !npm_valid {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal tool subjects do not match the exact git/node/npm producer contracts",
        ));
    }
    Ok(())
}

pub(super) fn exact_numeric_version(
    version: &str,
    prefix: &str,
    minimum_parts: usize,
    maximum_parts: usize,
) -> bool {
    if !(1..=160).contains(&version.len()) {
        return false;
    }
    let Some(number) = version.strip_prefix(prefix) else {
        return false;
    };
    let parts = number.split('.').collect::<Vec<_>>();
    (minimum_parts..=maximum_parts).contains(&parts.len())
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

pub(super) fn validate_file_inventory(files: &[PortalFile], declared_total: u64) -> Result<(), CoordError> {
    if files.is_empty() || files.len() > MAX_FILES {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            format!("Portal bundle manifest must name 1..={MAX_FILES} files"),
        ));
    }
    let mut exact = BTreeSet::new();
    let mut portable = BTreeSet::new();
    let mut previous: Option<&str> = None;
    let mut index_count = 0_usize;
    let mut total = 0_u64;
    for file in files {
        let expected = admit_bundle_path(&file.path)?;
        if file.mime != expected {
            return Err(CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                format!("{} has MIME {}, expected {expected}", file.path, file.mime),
            ));
        }
        if previous.is_some_and(|path| js_string_cmp(path, &file.path) != Ordering::Less) {
            return Err(CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                "Portal bundle file records are not in strict producer path order",
            ));
        }
        previous = Some(&file.path);
        if !exact.insert(file.path.clone()) {
            return Err(CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                format!("Portal bundle repeats exact path {}", file.path),
            ));
        }
        if !portable.insert(file.path.to_ascii_lowercase()) {
            return Err(CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                format!(
                    "Portal bundle has a portable path collision at {}",
                    file.path
                ),
            ));
        }
        if file.path == "index.html" {
            index_count += 1;
        }
        if file.size > MAX_FILE_BYTES {
            return Err(CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                format!("{} exceeds its producer file-size bound", file.path),
            ));
        }
        total = total
            .checked_add(file.size)
            .ok_or_else(|| invalid("Portal bundle byte total overflowed"))?;
        if total > MAX_TOTAL_BYTES {
            return Err(CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                "Portal bundle exceeds its producer byte bound",
            ));
        }
    }
    if index_count != 1 {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal bundle must contain exactly one index.html",
        ));
    }
    if total != declared_total {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal bundle manifest total size differs from its file records",
        ));
    }
    Ok(())
}

pub(super) fn admit_bundle_path(relative: &str) -> Result<&'static str, CoordError> {
    if relative.is_empty()
        || relative.len() > 240
        || relative.starts_with('/')
        || relative.contains(['\\', ':'])
        || relative
            .chars()
            .any(|character| character <= '\u{1f}' || character == '\u{7f}')
    {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            format!("unsafe Portal bundle path {relative:?}"),
        ));
    }
    let components = relative.split('/').collect::<Vec<_>>();
    if components.iter().any(|component| {
        component.is_empty()
            || matches!(*component, "." | "..")
            || component.starts_with('.')
            || component.ends_with(['.', ' '])
            || component.eq_ignore_ascii_case(".git")
    }) || (relative != "index.html" && !(components.len() == 2 && components[0] == "assets"))
    {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            format!("unsafe or unexpected Portal bundle path {relative:?}"),
        ));
    }
    expected_mime(relative).ok_or_else(|| {
        CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            format!("{relative} has an unsupported bundle media type"),
        )
    })
}

pub(super) fn js_string_cmp(left: &str, right: &str) -> Ordering {
    left.encode_utf16().cmp(right.encode_utf16())
}

pub(super) fn expected_mime(path: &str) -> Option<&'static str> {
    let extension = path.rsplit_once('.')?.1;
    match extension {
        "css" => Some("text/css; charset=utf-8"),
        "gif" => Some("image/gif"),
        "html" => Some("text/html; charset=utf-8"),
        "ico" => Some("image/x-icon"),
        "jpeg" | "jpg" => Some("image/jpeg"),
        "js" => Some("text/javascript; charset=utf-8"),
        "json" => Some("application/json"),
        "png" => Some("image/png"),
        "svg" => Some("image/svg+xml"),
        "txt" => Some("text/plain; charset=utf-8"),
        "webp" => Some("image/webp"),
        "woff" => Some("font/woff"),
        "woff2" => Some("font/woff2"),
        _ => None,
    }
}

pub(super) fn npm_install_args(plan: &BuildPlan) -> Vec<String> {
    let mut args = vec![
        "ci".to_owned(),
        "--ignore-scripts".to_owned(),
        "--no-audit".to_owned(),
        "--no-fund".to_owned(),
    ];
    args.push(if plan.offline {
        "--offline".to_owned()
    } else {
        "--prefer-offline".to_owned()
    });
    args
}
