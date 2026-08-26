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

fn contains_rust_identifier(source: &str, identifier: &str) -> bool {
    rust_identifier_count(source, identifier) > 0
}

fn rust_identifier_count(source: &str, identifier: &str) -> usize {
    source
        .match_indices(identifier)
        .filter(|(start, _)| {
            let before = source[..*start].chars().next_back();
            let end = start + identifier.len();
            let after = source[end..].chars().next();
            let is_identifier = |character: char| character.is_alphanumeric() || character == '_';
            !before.is_some_and(is_identifier) && !after.is_some_and(is_identifier)
        })
        .count()
}

fn rust_code_skeleton(source: &str) -> String {
    #[derive(Clone, Copy)]
    enum State {
        Code,
        LineComment,
        BlockComment(usize),
        String(bool),
        RawString(usize),
        Char(bool),
    }

    let bytes = source.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut state = State::Code;
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        let next = bytes.get(index + 1).copied();
        match state {
            State::Code if byte == b'/' && next == Some(b'/') => {
                output.extend_from_slice(b"  ");
                index += 2;
                state = State::LineComment;
            }
            State::Code if byte == b'/' && next == Some(b'*') => {
                output.extend_from_slice(b"  ");
                index += 2;
                state = State::BlockComment(1);
            }
            State::Code => {
                let raw_start = if byte == b'r' {
                    Some(index)
                } else if byte == b'b' && next == Some(b'r') {
                    Some(index + 1)
                } else {
                    None
                };
                if let Some(raw_index) = raw_start {
                    let mut quote = raw_index + 1;
                    while bytes.get(quote) == Some(&b'#') {
                        quote += 1;
                    }
                    if bytes.get(quote) == Some(&b'"') {
                        let hash_count = quote - raw_index - 1;
                        while index <= quote {
                            output.push(b' ');
                            index += 1;
                        }
                        state = State::RawString(hash_count);
                        continue;
                    }
                }
                if byte == b'"' {
                    output.push(b' ');
                    index += 1;
                    state = State::String(false);
                } else if byte == b'\''
                    && (bytes.get(index + 2) == Some(&b'\'')
                        || (next == Some(b'\\') && bytes.get(index + 3) == Some(&b'\'')))
                {
                    output.push(b' ');
                    index += 1;
                    state = State::Char(false);
                } else {
                    output.push(byte);
                    index += 1;
                }
            }
            State::LineComment => {
                output.push(if byte == b'\n' { b'\n' } else { b' ' });
                index += 1;
                if byte == b'\n' {
                    state = State::Code;
                }
            }
            State::BlockComment(depth) if byte == b'/' && next == Some(b'*') => {
                output.extend_from_slice(b"  ");
                index += 2;
                state = State::BlockComment(depth + 1);
            }
            State::BlockComment(depth) if byte == b'*' && next == Some(b'/') => {
                output.extend_from_slice(b"  ");
                index += 2;
                state = if depth == 1 {
                    State::Code
                } else {
                    State::BlockComment(depth - 1)
                };
            }
            State::BlockComment(depth) => {
                output.push(if byte == b'\n' { b'\n' } else { b' ' });
                index += 1;
                state = State::BlockComment(depth);
            }
            State::String(escaped) => {
                output.push(if byte == b'\n' { b'\n' } else { b' ' });
                index += 1;
                state = if escaped {
                    State::String(false)
                } else if byte == b'\\' {
                    State::String(true)
                } else if byte == b'"' {
                    State::Code
                } else {
                    State::String(false)
                };
            }
            State::RawString(hash_count) => {
                let closes = byte == b'"'
                    && (0..hash_count).all(|offset| bytes.get(index + 1 + offset) == Some(&b'#'));
                output.push(if byte == b'\n' { b'\n' } else { b' ' });
                index += 1;
                if closes {
                    output.extend(std::iter::repeat_n(b' ', hash_count));
                    index += hash_count;
                    state = State::Code;
                }
            }
            State::Char(escaped) => {
                output.push(b' ');
                index += 1;
                state = if escaped {
                    State::Char(false)
                } else if byte == b'\\' {
                    State::Char(true)
                } else if byte == b'\'' {
                    State::Code
                } else {
                    State::Char(false)
                };
            }
        }
    }
    String::from_utf8(output).expect("source skeleton stays UTF-8")
}

fn raw_serde_json_decoder(source: &str) -> Option<&'static str> {
    let code = rust_code_skeleton(source);
    let source = code.as_str();
    if !contains_rust_identifier(source, "serde_json") {
        return None;
    }
    let compact = source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    if compact.contains("pubuseserde_json")
        || compact.contains("useserde_jsonas")
        || compact.contains("useserde_json::{selfas")
        || compact.contains("externcrateserde_jsonas")
    {
        return Some("serde_json alias or re-export");
    }

    for statement in source.split(';') {
        let compact_statement = statement
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        let serde_import = contains_rust_identifier(statement, "use")
            && contains_rust_identifier(statement, "serde_json");
        let local_reexport = contains_rust_identifier(statement, "pub")
            && contains_rust_identifier(statement, "use");
        if (serde_import && local_reexport)
            || (serde_import
                && contains_rust_identifier(statement, "as")
                && contains_rust_identifier(statement, "serde_json"))
            || (contains_rust_identifier(statement, "extern")
                && contains_rust_identifier(statement, "crate")
                && contains_rust_identifier(statement, "serde_json"))
        {
            return Some("serde_json alias or re-export");
        }
        for json_type in ["Value", "Map", "Number", "RawValue"] {
            if (serde_import && compact_statement.contains(&format!("{json_type}as")))
                || (local_reexport
                    && contains_rust_identifier(statement, json_type)
                    && compact_statement.contains("as"))
            {
                return Some("serde_json type alias or re-export");
            }
            let alias = statement.split_once('=');
            let is_type_alias = alias.is_some_and(|(lhs, _)| type_alias_lhs(lhs));
            if is_type_alias
                && alias.is_some_and(|(_, rhs)| contains_rust_identifier(rhs, json_type))
                && (contains_rust_identifier(source, "serde_json")
                    || statement.contains("serde_json"))
            {
                return Some("serde_json document type alias");
            }
        }
        if serde_import
            && [
                "from_str",
                "from_slice",
                "from_reader",
                "Deserializer",
                "StreamDeserializer",
                "SliceRead",
            ]
            .into_iter()
            .any(|identifier| contains_rust_identifier(statement, identifier))
        {
            return Some("imported serde_json decoder");
        }
        if serde_import
            && (compact_statement.contains("useserde_json::*")
                || compact_statement == "useserde_json::de"
                || compact_statement.contains("useserde_json::{de"))
        {
            return Some("imported serde_json decoder namespace");
        }
        if contains_rust_identifier(statement, "serde_json")
            && [
                "from_str",
                "from_slice",
                "from_reader",
                "Deserializer",
                "StreamDeserializer",
                "SliceRead",
                "RawValue",
            ]
            .into_iter()
            .any(|identifier| contains_rust_identifier(statement, identifier))
        {
            return Some("qualified serde_json decoder");
        }
    }

    let has_json_document_type = ["Value", "Map", "Number", "RawValue"]
        .into_iter()
        .any(|identifier| contains_rust_identifier(source, identifier));
    if has_json_document_type && method_call_count(source, "parse") > 0 {
        return Some("serde_json document FromStr parse");
    }
    if [
        "serde_json::from_str",
        "serde_json::from_slice",
        "serde_json::from_reader",
        "serde_json::Deserializer",
        "serde_json::de::Deserializer",
        "StreamDeserializer",
        "SliceRead",
        "RawValue",
    ]
    .into_iter()
    .any(|decoder| source.contains(decoder))
    {
        return Some("qualified serde_json decoder");
    }
    None
}

fn type_alias_lhs(lhs: &str) -> bool {
    lhs.match_indices("type").any(|(start, _)| {
        let before = lhs[..start].chars().next_back();
        let after = lhs[start + "type".len()..].chars().next();
        let is_identifier = |character: char| character.is_alphanumeric() || character == '_';
        if before.is_some_and(is_identifier) || after.is_some_and(is_identifier) {
            return false;
        }
        lhs[start + "type".len()..]
            .trim_start()
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
    })
}

fn lint_policy_override(source: &str) -> bool {
    let code = rust_code_skeleton(source);
    let compact = code
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    let lowers = ["allow", "warn", "expect"]
        .into_iter()
        .any(|level| contains_rust_identifier(&code, level));
    let dangerous_clippy_name = [
        "all",
        "cargo",
        "complexity",
        "correctness",
        "disallowed_methods",
        "nursery",
        "pedantic",
        "perf",
        "restriction",
        "style",
        "suspicious",
    ]
    .into_iter()
    .any(|name| compact.contains(&format!("clippy::{name}")));
    (lowers && dangerous_clippy_name)
        || ["allow(warnings)", "warn(warnings)", "expect(warnings)"]
            .into_iter()
            .any(|override_| compact.contains(override_))
}

fn method_call_count(source: &str, method: &str) -> usize {
    source
        .match_indices(method)
        .filter(|(start, _)| {
            let before = source[..*start]
                .chars()
                .rev()
                .find(|character| !character.is_whitespace());
            let end = start + method.len();
            let after = source[end..]
                .chars()
                .find(|character| !character.is_whitespace());
            before == Some('.') && matches!(after, Some('(' | ':'))
        })
        .count()
}

fn associated_call_count(source: &str, function: &str) -> usize {
    source
        .match_indices(function)
        .filter(|(start, _)| {
            let before = source[..*start]
                .chars()
                .rev()
                .find(|character| !character.is_whitespace());
            let end = start + function.len();
            let after = source[end..]
                .chars()
                .find(|character| !character.is_whitespace());
            before == Some(':') && matches!(after, Some('(' | ':' | '<'))
        })
        .count()
}

fn include_macro_ranges(source: &str) -> Result<Vec<std::ops::Range<usize>>, &'static str> {
    let code = rust_code_skeleton(source);
    let bytes = code.as_bytes();
    let mut ranges = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        let Some(offset) = code[cursor..].find("include") else {
            break;
        };
        let start = cursor + offset;
        let end = start + "include".len();
        let is_identifier = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_';
        if start
            .checked_sub(1)
            .and_then(|index| bytes.get(index))
            .copied()
            .is_some_and(is_identifier)
            || bytes.get(end).copied().is_some_and(is_identifier)
        {
            cursor = end;
            continue;
        }
        let mut next = end;
        while bytes.get(next).is_some_and(u8::is_ascii_whitespace) {
            next += 1;
        }
        if bytes.get(next) != Some(&b'!') {
            cursor = end;
            continue;
        }
        next += 1;
        while bytes.get(next).is_some_and(u8::is_ascii_whitespace) {
            next += 1;
        }
        if bytes.get(next) != Some(&b'(') {
            return Err("include macro lacks a parenthesized token tree");
        }
        let mut depth = 0_u32;
        let mut close = None;
        for (relative, byte) in bytes[next..].iter().enumerate() {
            match byte {
                b'(' => depth += 1,
                b')' => {
                    depth = depth
                        .checked_sub(1)
                        .ok_or("include macro delimiters are unbalanced")?;
                    if depth == 0 {
                        close = Some(next + relative + 1);
                        break;
                    }
                }
                _ => {}
            }
        }
        let close = close.ok_or("include macro token tree is unbalanced")?;
        ranges.push(start..close);
        cursor = close;
    }
    Ok(ranges)
}

fn path_attribute_lines(source: &str) -> Result<Vec<String>, &'static str> {
    let code = rust_code_skeleton(source);
    let compact_code = code
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    let token_count = compact_code.matches("#[path=").count();
    let lines = source
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.chars()
                .filter(|character| !character.is_whitespace())
                .collect::<String>()
                .starts_with("#[path=")
        })
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let exact_literals = lines.iter().all(|line| {
        let Some(first_quote) = line.find('"') else {
            return false;
        };
        let Some(last_quote) = line.rfind('"') else {
            return false;
        };
        let prefix = line[..first_quote]
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        let suffix = line[last_quote + 1..]
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        let target = &line[first_quote + 1..last_quote];
        prefix == "#[path=" && suffix == "]" && !target.is_empty() && !target.contains(['\\', '\0'])
    });
    (lines.len() == token_count && exact_literals)
        .then_some(lines)
        .ok_or("path attributes must be exact one-line string literals")
}

fn path_attribute_target(line: &str) -> Option<&str> {
    let first_quote = line.find('"')?;
    let last_quote = line.rfind('"')?;
    (first_quote < last_quote).then_some(&line[first_quote + 1..last_quote])
}

fn attribute_bodies(source: &str) -> Result<Vec<String>, &'static str> {
    let code = rust_code_skeleton(source);
    let bytes = code.as_bytes();
    let mut attributes = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        let Some(offset) = code[cursor..].find('#') else {
            break;
        };
        let start = cursor + offset;
        let mut open = start + 1;
        while bytes.get(open).is_some_and(u8::is_ascii_whitespace) {
            open += 1;
        }
        if bytes.get(open) == Some(&b'!') {
            open += 1;
            while bytes.get(open).is_some_and(u8::is_ascii_whitespace) {
                open += 1;
            }
        }
        if bytes.get(open) != Some(&b'[') {
            cursor = start + 1;
            continue;
        }
        let mut depth = 0_u32;
        let mut close = None;
        for (offset, byte) in bytes[open..].iter().enumerate() {
            match byte {
                b'[' => depth += 1,
                b']' => {
                    depth = depth
                        .checked_sub(1)
                        .ok_or("attribute brackets are unbalanced")?;
                    if depth == 0 {
                        close = Some(open + offset);
                        break;
                    }
                }
                _ => {}
            }
        }
        let close = close.ok_or("attribute brackets are unbalanced")?;
        attributes.push(code[open + 1..close].to_owned());
        cursor = close + 1;
    }
    Ok(attributes)
}

fn attribute_assigns_path(attribute: &str) -> bool {
    let bytes = attribute.as_bytes();
    let identifier = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_';
    let mut cursor = 0;
    while cursor < bytes.len() {
        if !bytes[cursor].is_ascii_alphabetic() && bytes[cursor] != b'_' {
            cursor += 1;
            continue;
        }
        let start = cursor;
        cursor += 1;
        while bytes.get(cursor).is_some_and(|byte| identifier(*byte)) {
            cursor += 1;
        }
        if &attribute[start..cursor] != "path" {
            continue;
        }
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if bytes.get(cursor) == Some(&b'=') {
            return true;
        }
    }
    false
}

fn direct_path_attribute(attribute: &str) -> bool {
    let trimmed = attribute.trim_start();
    let Some(remainder) = trimmed.strip_prefix("path") else {
        return false;
    };
    remainder.trim_start().starts_with('=')
}

fn indirect_attribute_redirects_path(source: &str) -> Result<bool, &'static str> {
    Ok(attribute_bodies(source)?.iter().any(|attribute| {
        attribute.contains('$')
            || (attribute_assigns_path(attribute) && !direct_path_attribute(attribute))
    }))
}

fn macro_tt_fragment(source: &str) -> bool {
    let compact = rust_code_skeleton(source)
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    compact.match_indices(":tt").any(|(start, _)| {
        compact[start + 3..]
            .chars()
            .next()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_')
    })
}

fn macro_arguments_assign_path(source: &str) -> Result<bool, &'static str> {
    let code = rust_code_skeleton(source);
    let bytes = code.as_bytes();
    for (bang, _) in code.match_indices('!') {
        let macro_name_ends = code[..bang]
            .chars()
            .next_back()
            .is_some_and(|character| character.is_alphanumeric() || character == '_');
        if !macro_name_ends {
            continue;
        }
        let mut open = bang + 1;
        while bytes.get(open).is_some_and(u8::is_ascii_whitespace) {
            open += 1;
        }
        let Some(first_close) = bytes.get(open).and_then(|byte| match byte {
            b'(' => Some(b')'),
            b'[' => Some(b']'),
            b'{' => Some(b'}'),
            _ => None,
        }) else {
            continue;
        };
        let mut stack = vec![first_close];
        let mut cursor = open + 1;
        while let Some(byte) = bytes.get(cursor).copied() {
            match byte {
                b'(' => stack.push(b')'),
                b'[' => stack.push(b']'),
                b'{' => stack.push(b'}'),
                b')' | b']' | b'}' => {
                    if stack.pop() != Some(byte) {
                        return Err("macro invocation delimiters are unbalanced");
                    }
                    if stack.is_empty() {
                        if attribute_assigns_path(&code[open + 1..cursor]) {
                            return Ok(true);
                        }
                        break;
                    }
                }
                _ => {}
            }
            cursor += 1;
        }
        if !stack.is_empty() {
            return Err("macro invocation delimiters are unbalanced");
        }
    }
    Ok(false)
}

fn qualified_attribute_paths(source: &str) -> Result<Vec<String>, &'static str> {
    let attributes = attribute_bodies(source)?;

    let identifier = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_';
    let mut paths = Vec::new();
    for attribute in &attributes {
        let bytes = attribute.as_bytes();
        let mut cursor = 0;
        while cursor < bytes.len() {
            if !bytes[cursor].is_ascii_alphabetic() && bytes[cursor] != b'_' {
                cursor += 1;
                continue;
            }
            let mut end = cursor + 1;
            while bytes.get(end).is_some_and(|byte| identifier(*byte)) {
                end += 1;
            }
            let mut normalized = attribute[cursor..end].to_owned();
            let mut segments = 1_usize;
            loop {
                let mut separator = end;
                while bytes.get(separator).is_some_and(u8::is_ascii_whitespace) {
                    separator += 1;
                }
                if bytes.get(separator..separator + 2) != Some(b"::") {
                    break;
                }
                let mut next = separator + 2;
                while bytes.get(next).is_some_and(u8::is_ascii_whitespace) {
                    next += 1;
                }
                if !bytes
                    .get(next)
                    .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
                {
                    break;
                }
                let mut next_end = next + 1;
                while bytes.get(next_end).is_some_and(|byte| identifier(*byte)) {
                    next_end += 1;
                }
                normalized.push_str("::");
                normalized.push_str(&attribute[next..next_end]);
                segments += 1;
                end = next_end;
            }
            if segments > 1 {
                paths.push(normalized);
            }
            cursor = end.max(cursor + 1);
        }
    }
    paths.sort();
    Ok(paths)
}

fn external_test_module_lines(source: &str) -> Result<Vec<usize>, &'static str> {
    let code = rust_code_skeleton(source);
    let mut compact = String::with_capacity(code.len());
    let mut original_offsets = Vec::with_capacity(code.len());
    for (offset, character) in code.char_indices() {
        if character.is_whitespace() {
            continue;
        }
        compact.push(character);
        original_offsets.extend(std::iter::repeat_n(offset, character.len_utf8()));
    }
    let mut modules = Vec::new();
    for (raw, declaration) in [(false, "modtests;"), (true, "modr#tests;")] {
        for (start, _) in compact.match_indices(declaration) {
            let identifier = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_';
            if start > 0 && identifier(compact.as_bytes()[start - 1]) {
                let previous = *original_offsets
                    .get(start - 1)
                    .ok_or("external tests module predecessor offset is missing")?;
                let current = *original_offsets
                    .get(start)
                    .ok_or("external tests module offset is missing")?;
                if current == previous + 1 {
                    continue;
                }
            }
            if raw {
                return Err("external tests module must not use a raw identifier");
            }
            let boundary = compact[..start]
                .rfind([';', '{', '}', ']'])
                .map_or(0, |index| index + 1);
            if boundary != start {
                return Err("external tests module must use the exact private declaration");
            }
            let original = *original_offsets
                .get(start)
                .ok_or("external tests module offset is missing")?;
            modules.push(
                code.as_bytes()[..original]
                    .iter()
                    .filter(|byte| **byte == b'\n')
                    .count(),
            );
        }
    }
    modules.sort_unstable();
    modules.dedup();
    Ok(modules)
}

fn test_modules_are_cfg_gated(source: &str) -> bool {
    let Ok(module_lines) = external_test_module_lines(source) else {
        return false;
    };
    let lines = source.lines().collect::<Vec<_>>();
    let skeleton = rust_code_skeleton(source);
    let skeleton_lines = skeleton.lines().collect::<Vec<_>>();
    for index in module_lines {
        let mut predecessors = lines[..index]
            .iter()
            .zip(&skeleton_lines[..index])
            .rev()
            .filter(|(_, skeleton)| !skeleton.trim().is_empty())
            .map(|(line, _)| line.trim());
        let mut cfg = predecessors.next().unwrap_or_default();
        if cfg
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>()
            .starts_with("#[path=")
        {
            cfg = predecessors.next().unwrap_or_default();
        }
        let cfg = cfg
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        if !matches!(
            cfg.as_str(),
            "#[cfg(test)]" | "#[cfg(all(test,unix))]" | "#[cfg(all(test,target_os=\"linux\"))]"
        ) {
            return false;
        }
    }
    true
}

fn external_test_module_targets(
    source_path: &Path,
    source: &str,
) -> Result<Vec<PathBuf>, &'static str> {
    if !test_modules_are_cfg_gated(source) {
        return Err("external tests module is not cfg-gated");
    }
    let lines = source.lines().collect::<Vec<_>>();
    let skeleton = rust_code_skeleton(source);
    let skeleton_lines = skeleton.lines().collect::<Vec<_>>();
    let mut targets = Vec::new();
    for index in external_test_module_lines(source)? {
        let predecessor = lines[..index]
            .iter()
            .zip(&skeleton_lines[..index])
            .rev()
            .find(|(_, skeleton)| !skeleton.trim().is_empty())
            .map(|(line, _)| line.trim())
            .ok_or("external tests module lacks a preceding cfg")?;
        let parent = source_path.parent().ok_or("source path lacks a parent")?;
        let target = if predecessor
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>()
            .starts_with("#[path=")
        {
            parent.join(
                path_attribute_target(predecessor)
                    .ok_or("external tests path is not an exact literal")?,
            )
        } else {
            let module_dir =
                if source_path.file_name().and_then(|name| name.to_str()) == Some("mod.rs") {
                    parent.to_path_buf()
                } else {
                    parent.join(
                        source_path
                            .file_stem()
                            .and_then(|stem| stem.to_str())
                            .ok_or("source path lacks a UTF-8 stem")?,
                    )
                };
            let candidates = [module_dir.join("tests.rs"), module_dir.join("tests/mod.rs")];
            let existing = candidates
                .into_iter()
                .filter(|candidate| candidate.is_file())
                .collect::<Vec<_>>();
            if existing.len() != 1 {
                return Err("external tests module must resolve to exactly one file");
            }
            existing[0].clone()
        };
        targets.push(fs::canonicalize(target).map_err(|_| "external tests target is missing")?);
    }
    Ok(targets)
}

const NUMBER_ALLOWANCE: &str = "#[expect(clippy::disallowed_methods,reason=\"thereviewednumberboundaryverifiesanexactfiniteroundtripbeforeadmission\")]letparsed=token.parse::<f64>().map_err(|_|number_out_of_range())?;";

const DOCUMENT_ALLOWANCE: &str = "#[expect(clippy::disallowed_methods,reason=\"therevieweddocumentboundarydecodesintotheduplicate-rejectingUniqueValuevisitor\")]letunique=serde_json::from_str::<UniqueValue>(text).map_err(parse_error)?;";

const BOUNDED_ENTRYPOINT: &str = "fndecode_unique_value_bounded(bytes:&[u8],max_bytes:usize)->Result<Value,WireError>{validate_input_size(bytes,max_bytes)?;lettext=std::str::from_utf8(bytes).map_err(|error|{WireError::new(\"INVALID_UTF8\",format!(\"documentisnotstrictUTF-8:{error}\"),)})?;iftext.starts_with('\\u{feff}'){returnErr(WireError::new(\"UTF8_BOM_FORBIDDEN\",\"canonicaldocumentsdonotcarryaUTF-8byte-ordermark\",));}decode_reviewed_text(text)}";

const BOUNDED_ENTRYPOINT_DIGEST: &str =
    "e1a9a21cf4361d343c08c84ab432dae90eedd7b85729da84e997868028fa01b4";

const REVIEWED_ENTRYPOINT_DIGEST: &str =
    "b5dc3cd32a988d760294aa55560db470c5fccb49c6b01cfd0e1bf5c1a121ddfd";

const CANONICAL_SOURCE_DIGEST: &str =
    "ddb16e6df4e2480a189b7d542e60da0783528a6a0e76463bf02ab2565e982481";

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

const CANONICAL_INCLUDE: &str = "include!(concat!(env!(\"CARGO_MANIFEST_DIR\"),\"/../../contracts/generated/rust/schema_bundle.rs\"))";

fn has_only_canonical_include(source: &str) -> bool {
    let Ok(ranges) = include_macro_ranges(source) else {
        return false;
    };
    ranges.len() == 1
        && source[ranges[0].clone()]
            .chars()
            .filter(|character| !character.is_whitespace())
            .eq(CANONICAL_INCLUDE.chars())
}

fn function_range(source: &str, name: &str) -> Result<std::ops::Range<usize>, &'static str> {
    let marker = format!("fn {name}");
    let starts = source
        .match_indices(&marker)
        .filter_map(|(start, _)| {
            let after = source[start + marker.len()..]
                .chars()
                .find(|character| !character.is_whitespace());
            (after == Some('(')).then_some(start)
        })
        .collect::<Vec<_>>();
    let [start] = starts.as_slice() else {
        return Err("reviewed function must have one exact identifier");
    };
    let start = *start;
    let brace = source[start..]
        .find('{')
        .map(|offset| start + offset)
        .ok_or("reviewed function body missing")?;
    let mut depth = 0_u32;
    for (offset, byte) in source.as_bytes()[brace..].iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth = depth
                    .checked_sub(1)
                    .ok_or("reviewed function braces are unbalanced")?;
                if depth == 0 {
                    return Ok(start..brace + offset + 1);
                }
            }
            _ => {}
        }
    }
    Err("reviewed function braces are unbalanced")
}

fn macro_invocation_count(source: &str) -> usize {
    let code = rust_code_skeleton(source);
    code.match_indices('!')
        .filter(|(index, _)| {
            let before = code[..*index]
                .chars()
                .rev()
                .find(|character| !character.is_whitespace());
            let after = code[index + 1..]
                .chars()
                .find(|character| !character.is_whitespace());
            before.is_some_and(|character| character.is_alphanumeric() || character == '_')
                && matches!(after, Some('(' | '[' | '{'))
        })
        .count()
}

fn canonical_entrypoint_shape(source: &str) -> Result<(), &'static str> {
    let normalized = normalized_lf(source)?;
    let source = normalized.as_ref();
    if independent_framed_digest("hostile.canonical.production-source-v1", source.as_bytes())
        != CANONICAL_SOURCE_DIGEST
    {
        return Err("canonical production source digest changed");
    }
    let code = rust_code_skeleton(source);
    let compact_source = source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    let raw = "serde_json::from_str::<UniqueValue>";
    if code.matches(raw).count() != 1 || code.matches("serde_json::from_str").count() != 1 {
        return Err("canonical source must contain one exact UniqueValue decoder");
    }
    if code.matches(".parse::<f64>()").count() != 1 {
        return Err("canonical source must contain one reviewed numeric parser");
    }
    if source.matches("clippy::disallowed_methods").count() != 2
        || compact_source.matches(NUMBER_ALLOWANCE).count() != 1
        || compact_source.matches(DOCUMENT_ALLOWANCE).count() != 1
        || source.contains("#![allow(clippy::disallowed_methods)]")
        || source.contains("#![expect(clippy::disallowed_methods)]")
    {
        return Err("canonical decoder must have two exact statement-attached lint expectations");
    }
    let remainder = code.replacen(raw, "strict_unique_decoder", 1).replacen(
        ".parse::<f64>()",
        ".strict_numeric_parse::<f64>()",
        1,
    );
    if raw_serde_json_decoder(&remainder).is_some() {
        return Err("canonical source contains a second raw decoder surface");
    }

    let bounded_range = function_range(&code, "decode_unique_value_bounded")?;
    let bounded_item_start = bounded_range
        .start
        .checked_sub("pub ".len())
        .ok_or("bounded entrypoint visibility missing")?;
    if &source[bounded_item_start..bounded_range.start] != "pub " {
        return Err("bounded entrypoint must retain exact public visibility");
    }
    let predecessor_end = code[..bounded_range.start]
        .rfind('}')
        .ok_or("bounded entrypoint predecessor missing")?;
    let bounded_prefix = code[predecessor_end + 1..bounded_range.start]
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    if bounded_prefix != "pub" {
        return Err("bounded entrypoint must not carry an outer attribute or modifier");
    }
    let function = &code[bounded_range.clone()];
    let bounded_source = &source[bounded_item_start..bounded_range.end];
    let compact_function = source[bounded_range.clone()]
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    if compact_function != BOUNDED_ENTRYPOINT {
        return Err("bounded entrypoint must retain its exact fail-closed control flow");
    }
    if independent_framed_digest(
        "hostile.canonical.bounded-raw-shape",
        bounded_source.as_bytes(),
    ) != BOUNDED_ENTRYPOINT_DIGEST
    {
        return Err("bounded entrypoint shape digest changed");
    }
    for (stage, count) in [
        ("validate_input_size(", 1),
        ("std::str::from_utf8(", 1),
        ("text.starts_with(", 1),
        ("decode_reviewed_text(", 1),
    ] {
        if compact_function.matches(stage).count() != count {
            return Err("bounded entrypoint must contain each validation stage exactly once");
        }
    }
    if compact_function.contains("#[cfg(")
        || compact_function.contains("#[cfg_attr(")
        || compact_function.contains("returnOk(")
    {
        return Err("bounded entrypoint must not conditionally bypass validation");
    }
    let mut cursor = 0;
    for marker in [
        "validate_input_size",
        "std::str::from_utf8",
        "text.starts_with",
        "decode_reviewed_text",
    ] {
        let position = function[cursor..]
            .find(marker)
            .ok_or("bounded entrypoint validation stage missing")?;
        cursor += position + marker.len();
    }

    let reviewed_range = function_range(&code, "decode_reviewed_text")?;
    let reviewed = &code[reviewed_range.clone()];
    let reviewed_source = &source[reviewed_range.clone()];
    if macro_invocation_count(reviewed_source) != 0 {
        return Err("reviewed decoder must not contain macro invocations");
    }
    let predecessor_end = source[..reviewed_range.start]
        .rfind('}')
        .ok_or("reviewed decoder predecessor missing")?;
    if !source[predecessor_end + 1..reviewed_range.start]
        .trim()
        .is_empty()
    {
        return Err("reviewed decoder must not carry an outer attribute");
    }
    let compact_reviewed = reviewed_source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    if independent_framed_digest(
        "hostile.canonical.reviewed-raw-shape",
        reviewed_source.as_bytes(),
    ) != REVIEWED_ENTRYPOINT_DIGEST
    {
        return Err("reviewed entrypoint shape digest changed");
    }
    if compact_reviewed.contains("#[cfg(")
        || compact_reviewed.contains("#[cfg_attr(")
        || compact_reviewed.contains("returnOk(")
        || rust_identifier_count(reviewed, "return") != 3
        || compact_reviewed.matches("returnErr(").count() != 3
        || reviewed.matches("validate_value").count() != 1
        || reviewed.matches(raw).count() != 1
        || reviewed.matches(".parse::<f64>()").count() != 1
    {
        return Err("reviewed decoder must not conditionally bypass validation");
    }
    let exact_tail = format!("{DOCUMENT_ALLOWANCE}validate_value(&unique.0)?;Ok(unique.0)}}");
    if !compact_reviewed.ends_with(&exact_tail) {
        return Err("reviewed decoder must end in the exact validation-and-return tail");
    }
    let mut cursor = 0;
    for marker in [".parse::<f64>()", raw, "validate_value"] {
        let position = reviewed[cursor..]
            .find(marker)
            .ok_or("reviewed decoder stage missing")?;
        cursor += position + marker.len();
    }
    Ok(())
}

fn padded_document(size: usize) -> Vec<u8> {
    assert!(size >= 2);
    let mut bytes = vec![b' '; size];
    bytes[size - 2..].copy_from_slice(b"{}");
    bytes
}

#[test]
fn hostile_fixture_files_fail_with_stable_reason_codes() {
    let expected = BTreeMap::from([
        ("bidi.json", "DIRECTIONAL_CONTROL_FORBIDDEN"),
        ("bom.json", "UTF8_BOM_FORBIDDEN"),
        ("crlf.json", "NON_CANONICAL_JSON"),
        ("duplicate-key.json", "DUPLICATE_JSON_KEY"),
        ("escaped-control.json", "CONTROL_CHARACTER_FORBIDDEN"),
        ("invalid-utf8.json", "INVALID_UTF8"),
        ("lf.json", "NON_CANONICAL_JSON"),
        ("non-nfc.json", "NON_NFC_STRING"),
        ("nul.json", "CONTROL_CHARACTER_FORBIDDEN"),
        ("raw-control.json", "INVALID_JSON"),
        ("zero-width.json", "ZERO_WIDTH_CHARACTER_FORBIDDEN"),
    ]);
    for (name, code) in expected {
        let bytes = fs::read(root().join("fixtures/hostile/cases").join(name)).unwrap();
        let error = decode_canonical_value(&bytes).unwrap_err();
        assert_eq!(error.code(), code, "fixture {name}");
    }
    for noncharacter in [
        br#"{"value":"\ufdd0"}"#.as_slice(),
        br#"{"value":"\uffff"}"#.as_slice(),
    ] {
        assert_eq!(
            decode_unique_value(noncharacter).unwrap_err().code(),
            "UNICODE_NONCHARACTER_FORBIDDEN"
        );
    }
}

fn escaped_json_character(codepoint: u32) -> Vec<u8> {
    if codepoint <= 0xffff {
        return format!(r#"{{"value":"\u{codepoint:04x}"}}"#).into_bytes();
    }
    let scalar = codepoint - 0x1_0000;
    let high = 0xd800 + (scalar >> 10);
    let low = 0xdc00 + (scalar & 0x3ff);
    format!(r#"{{"value":"\u{high:04x}\u{low:04x}"}}"#).into_bytes()
}

fn expected_default_ignorable_code(codepoint: u32) -> &'static str {
    if matches!(
        codepoint,
        0x061c | 0x200e | 0x200f | 0x202a..=0x202e | 0x2066..=0x2069
    ) {
        "DIRECTIONAL_CONTROL_FORBIDDEN"
    } else {
        "ZERO_WIDTH_CHARACTER_FORBIDDEN"
    }
}

#[test]
fn unicode_15_1_default_ignorables_are_exact_and_typed() {
    let count = UNICODE_15_1_DEFAULT_IGNORABLE_INTERVALS
        .iter()
        .map(|(start, end)| end - start + 1)
        .sum::<u32>();
    assert_eq!(count, 4_174);
    assert!(
        UNICODE_15_1_DEFAULT_IGNORABLE_INTERVALS
            .windows(2)
            .all(|pair| pair[0].1 < pair[1].0)
    );

    let mut members = BTreeSet::new();
    let mut outside_neighbors = BTreeSet::new();
    for &(start, end) in UNICODE_15_1_DEFAULT_IGNORABLE_INTERVALS {
        members.extend(start..=end);
        if start > 0 {
            outside_neighbors.insert(start - 1);
        }
        if end < 0x10ffff {
            outside_neighbors.insert(end + 1);
        }
    }
    assert_eq!(members.len(), 4_174);

    let mut observed_members = 0_usize;
    for codepoint in members {
        let character = char::from_u32(codepoint).expect("DICP interval contains Unicode scalar");
        let expected = expected_default_ignorable_code(codepoint);
        let literal = format!(r#"{{"value":"{character}"}}"#);
        assert_eq!(
            decode_unique_value(literal.as_bytes()).unwrap_err().code(),
            expected,
            "literal U+{codepoint:04X}"
        );
        assert_eq!(
            decode_unique_value(&escaped_json_character(codepoint))
                .unwrap_err()
                .code(),
            expected,
            "escaped U+{codepoint:04X}"
        );
        observed_members += 1;
    }
    assert_eq!(observed_members, 4_174);

    for codepoint in outside_neighbors {
        if UNICODE_15_1_DEFAULT_IGNORABLE_INTERVALS
            .iter()
            .any(|(start, end)| (*start..=*end).contains(&codepoint))
        {
            continue;
        }
        let escaped = escaped_json_character(codepoint);
        let code = decode_unique_value(&escaped)
            .err()
            .map(|error| error.code().to_owned());
        assert!(
            !matches!(
                code.as_deref(),
                Some("ZERO_WIDTH_CHARACTER_FORBIDDEN" | "DIRECTIONAL_CONTROL_FORBIDDEN")
            ),
            "outside neighbor U+{codepoint:04X} was classified as default ignorable"
        );
    }

    assert_eq!(
        decode_unique_value(b"\xef\xbb\xbf{}").unwrap_err().code(),
        "UTF8_BOM_FORBIDDEN"
    );
    assert_eq!(
        decode_unique_value(br#"{"value":"\ufeff"}"#)
            .unwrap_err()
            .code(),
        "ZERO_WIDTH_CHARACTER_FORBIDDEN"
    );
    assert!(decode_unique_value("{\"value\":\"😀\"}".as_bytes()).is_ok());
    assert_eq!(
        decode_unique_value(br#"{"value":"\u0001"}"#)
            .unwrap_err()
            .code(),
        "CONTROL_CHARACTER_FORBIDDEN"
    );
}

#[test]
fn overlong_and_unsafe_numeric_inputs_fail_before_use() {
    let default_limit = padded_document(MAX_CANONICAL_DOCUMENT_BYTES);
    assert!(decode_unique_value(&default_limit).is_ok());
    let mut overlong = default_limit;
    overlong.push(b' ');
    assert_eq!(
        decode_canonical_value(&overlong).unwrap_err().code(),
        "DOCUMENT_TOO_LARGE"
    );
    assert_eq!(
        decode_unique_value_bounded(b"{}", 0).unwrap_err().code(),
        "DOCUMENT_LIMIT_INVALID"
    );
    assert_eq!(
        decode_unique_value_bounded(b"{}", MAX_UNIQUE_DOCUMENT_BYTES + 1)
            .unwrap_err()
            .code(),
        "DOCUMENT_LIMIT_INVALID"
    );
    let caller_limit = MAX_CANONICAL_DOCUMENT_BYTES + 4096;
    let mut caller_bounded = padded_document(caller_limit);
    assert_eq!(
        decode_unique_value(&caller_bounded).unwrap_err().code(),
        "DOCUMENT_TOO_LARGE"
    );
    assert!(decode_unique_value_bounded(&caller_bounded, caller_limit).is_ok());
    caller_bounded.push(b' ');
    assert_eq!(
        decode_unique_value_bounded(&caller_bounded, caller_limit)
            .unwrap_err()
            .code(),
        "DOCUMENT_TOO_LARGE"
    );

    let mut global_bounded = padded_document(MAX_UNIQUE_DOCUMENT_BYTES);
    assert!(
        decode_unique_value_bounded(&global_bounded, MAX_UNIQUE_DOCUMENT_BYTES).is_ok(),
        "the exact global boundary must be admissible"
    );
    global_bounded.push(b' ');
    assert_eq!(
        decode_unique_value_bounded(&global_bounded, MAX_UNIQUE_DOCUMENT_BYTES)
            .unwrap_err()
            .code(),
        "DOCUMENT_TOO_LARGE"
    );
    assert_eq!(
        decode_canonical_value(b"9007199254740992")
            .unwrap_err()
            .code(),
        "UNSAFE_JSON_INTEGER"
    );
}

#[test]
fn unique_decode_admits_formatting_but_never_ambiguous_members_or_numbers() {
    let value = decode_unique_value(b"{\n  \"b\": 2,\n  \"a\": 1\n}\n").unwrap();
    assert_eq!(value, serde_json::json!({"a": 1, "b": 2}));
    for hostile in [
        br#"{"status":"FAIL","status":"PASS"}"#.as_slice(),
        br#"{"number":NaN}"#.as_slice(),
        br#"{"number":Infinity}"#.as_slice(),
        br#"{"number":-Infinity}"#.as_slice(),
        br#"{"number":1e9999}"#.as_slice(),
        br#"{"number":1e-9999}"#.as_slice(),
        br#"{"number":18446744073709551616}"#.as_slice(),
        br#"{"number":9007199254740992.0}"#.as_slice(),
        br#"{"number":0.10000000000000001}"#.as_slice(),
    ] {
        assert!(decode_unique_value(hostile).is_err(), "{hostile:?}");
    }

    for ordinary in [
        br#"{"number":0.1}"#.as_slice(),
        br#"{"number":1.50}"#.as_slice(),
        br#"{"number":1e2}"#.as_slice(),
        br#"{"number":9007199254740991.0}"#.as_slice(),
    ] {
        assert!(decode_unique_value(ordinary).is_ok(), "{ordinary:?}");
    }

    let secret = "credential_ghp_1234567890abcdef";
    let hostile = format!(r#"{{"{secret}":"first","{secret}":"second"}}"#);
    let error = decode_unique_value(hostile.as_bytes()).unwrap_err();
    assert_eq!(error.code(), "DUPLICATE_JSON_KEY");
    assert!(!error.to_string().contains(secret));

    let family = root();
    let mut sources = Vec::new();
    assert!(excluded_root_tree(&family, &family.join("target")));
    assert!(!excluded_root_tree(&family, &family.join("src/target")));
    collect_rust_sources(&family, &family, &mut sources);
    let mut test_only_sources = BTreeSet::new();
    for path in &sources {
        let relative = path.strip_prefix(&family).expect("family-relative source");
        if !is_production_rust(relative) {
            continue;
        }
        let text = fs::read_to_string(path).expect("UTF-8 Rust source");
        test_only_sources.extend(
            external_test_module_targets(path, &text)
                .unwrap_or_else(|error| panic!("{}: {error}", relative.display())),
        );
    }
    let mut production = Vec::new();
    let mut include_identifier_sites = BTreeMap::new();
    let mut parse_sites = BTreeMap::new();
    let mut parse_id_sites = BTreeMap::new();
    let mut path_attribute_sites = BTreeMap::new();
    let mut qualified_attribute_sites = BTreeMap::new();
    let mut test_module_sites = BTreeMap::new();
    for path in &sources {
        let relative = path.strip_prefix(&family).expect("family-relative source");
        if !is_production_rust(relative)
            || test_only_sources
                .contains(&fs::canonicalize(path).expect("canonical inventoried Rust source"))
        {
            continue;
        }
        production.push(relative.to_path_buf());
        let text = fs::read_to_string(path).expect("UTF-8 Rust source");
        assert!(
            !indirect_attribute_redirects_path(&text).unwrap_or_else(|error| {
                panic!("{} has an invalid attribute: {error}", relative.display())
            }),
            "{} indirectly redirects a module path",
            relative.display()
        );
        assert!(
            !macro_tt_fragment(&text),
            "{} defines a `tt` macro fragment that can synthesize unreviewed source",
            relative.display()
        );
        assert!(
            !macro_arguments_assign_path(&text).unwrap_or_else(|error| {
                panic!(
                    "{} has an invalid macro invocation: {error}",
                    relative.display()
                )
            }),
            "{} supplies a module path through a macro argument",
            relative.display()
        );
        let code = rust_code_skeleton(&text);
        let qualified_attributes = qualified_attribute_paths(&text).unwrap_or_else(|error| {
            panic!("{} has an invalid attribute: {error}", relative.display())
        });
        if !qualified_attributes.is_empty() {
            let mut counts = BTreeMap::new();
            for attribute in qualified_attributes {
                *counts.entry(attribute).or_insert(0_usize) += 1;
            }
            qualified_attribute_sites.insert(relative.to_path_buf(), counts);
        }
        let include_identifier_count = rust_identifier_count(&code, "include");
        if include_identifier_count > 0 {
            include_identifier_sites.insert(relative.to_path_buf(), include_identifier_count);
        }
        let parse_count = method_call_count(&code, "parse");
        if parse_count > 0 {
            parse_sites.insert(relative.to_path_buf(), parse_count);
        }
        let parse_id_count = code.matches("parse_id(").count();
        if parse_id_count > 0 {
            parse_id_sites.insert(relative.to_path_buf(), parse_id_count);
        }
        let associated_from_str = associated_call_count(&code, "from_str");
        let compact_code = code
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        let path_attribute_count = compact_code.matches("#[path=").count();
        if path_attribute_count > 0 {
            let attributes = path_attribute_lines(&text).unwrap_or_else(|error| {
                panic!(
                    "{} has an invalid path attribute: {error}",
                    relative.display()
                )
            });
            assert_eq!(attributes.len(), path_attribute_count);
            path_attribute_sites.insert(relative.to_path_buf(), path_attribute_count);
        }
        let test_module_count = compact_code.matches("modtests;").count();
        if test_module_count > 0 {
            assert!(
                test_modules_are_cfg_gated(&text),
                "{} exposes an external tests module outside cfg(test)",
                relative.display()
            );
            test_module_sites.insert(relative.to_path_buf(), test_module_count);
        }
        let toml_from_str = compact_code.matches("toml::from_str(").count();
        let reviewed_raw_from_str =
            usize::from(relative == std::path::Path::new("crates/bullet-wire/src/canonical.rs"));
        assert_eq!(
            associated_from_str,
            toml_from_str + reviewed_raw_from_str,
            "{} adds an unreviewed associated FromStr call",
            relative.display()
        );
        if relative == std::path::Path::new("crates/bullet-wire/src/canonical.rs") {
            assert_eq!(canonical_entrypoint_shape(&text), Ok(()));
            continue;
        }
        assert!(
            raw_serde_json_decoder(&text).is_none(),
            "{} contains a direct raw serde_json decoder or document-type hiding surface {:?}; production JSON input must route through bullet-wire's bounded unique/canonical decoder",
            relative.display(),
            raw_serde_json_decoder(&text)
        );
    }
    assert_eq!(
        parse_sites,
        BTreeMap::from([(PathBuf::from("crates/bullet-wire/src/canonical.rs"), 1)])
    );
    assert_eq!(
        include_identifier_sites,
        BTreeMap::from([(PathBuf::from("crates/bullet-wire/src/lib.rs"), 1)])
    );
    assert_eq!(
        parse_id_sites,
        BTreeMap::from([
            (
                PathBuf::from("crates/bullet-wire/src/contract_tool/authority.rs"),
                19,
            ),
            (
                PathBuf::from("crates/bullet-wire/src/contract_tool/launch.rs"),
                11,
            ),
        ])
    );
    assert_eq!(
        path_attribute_sites,
        BTreeMap::from([
            (PathBuf::from("src/check/model.rs"), 1),
            (PathBuf::from("src/check/profiles.rs"), 1),
            (PathBuf::from("src/check/release_evidence.rs"), 1),
            (PathBuf::from("src/check/semantic_registry.rs"), 3),
            (PathBuf::from("src/fuse.rs"), 1),
            (PathBuf::from("src/process.rs"), 1),
            (PathBuf::from("src/release/receipt.rs"), 1),
        ])
    );
    assert_eq!(
        qualified_attribute_sites,
        BTreeMap::from([
            (
                PathBuf::from("contracts/generated/rust/schema_bundle.rs"),
                BTreeMap::from([
                    ("serde::Deserialize".to_owned(), 114),
                    ("serde::Serialize".to_owned(), 114),
                ]),
            ),
            (
                PathBuf::from("crates/bullet-wire/src/canonical.rs"),
                BTreeMap::from([("clippy::disallowed_methods".to_owned(), 2)]),
            ),
            (
                PathBuf::from("src/release/receipt/verify.rs"),
                BTreeMap::from([("clippy::too_many_arguments".to_owned(), 1)]),
            ),
            (
                PathBuf::from("src/release/signature.rs"),
                BTreeMap::from([("clippy::too_many_arguments".to_owned(), 1)]),
            ),
        ])
    );
    assert_eq!(
        test_module_sites,
        BTreeMap::from([
            (
                PathBuf::from("crates/bullet-wire/src/authority/request.rs"),
                1
            ),
            (PathBuf::from("src/check/model.rs"), 1),
            (PathBuf::from("src/check/release_evidence.rs"), 1),
            (PathBuf::from("src/checkout/git.rs"), 1),
            (PathBuf::from("src/family_lock.rs"), 1),
            (PathBuf::from("src/family_lock/git/command.rs"), 1),
            (PathBuf::from("src/family_lock/schema.rs"), 1),
            (PathBuf::from("src/fuse.rs"), 1),
            (PathBuf::from("src/process.rs"), 1),
            (PathBuf::from("src/release/archive.rs"), 1),
            (PathBuf::from("src/release/build/mod.rs"), 1),
            (PathBuf::from("src/release/receipt.rs"), 1),
            (PathBuf::from("src/release/verify.rs"), 1),
            (PathBuf::from("src/setup.rs"), 1),
            (PathBuf::from("src/setup/command.rs"), 1),
            (PathBuf::from("src/setup/transaction.rs"), 1),
        ])
    );
    for (relative, attributes) in [
        ("src/check/model.rs", &["#[path = \"model_tests.rs\"]"][..]),
        (
            "src/check/profiles.rs",
            &["#[path = \"profiles/graph.rs\"]"][..],
        ),
        (
            "src/check/release_evidence.rs",
            &["#[path = \"release_evidence/tests.rs\"]"][..],
        ),
        (
            "src/check/semantic_registry.rs",
            &[
                "#[path = \"semantic_registry/admission.rs\"]",
                "#[path = \"semantic_registry/unsupported.rs\"]",
                "#[path = \"semantic_registry/validation.rs\"]",
            ][..],
        ),
        ("src/fuse.rs", &["#[path = \"fuse/tests.rs\"]"][..]),
        (
            "src/process.rs",
            &["#[path = \"../tests/support/process_unit.rs\"]"][..],
        ),
        (
            "src/release/receipt.rs",
            &["#[path = \"receipt/tests.rs\"]"][..],
        ),
    ] {
        let text = fs::read_to_string(family.join(relative)).expect("UTF-8 Rust source");
        let actual = path_attribute_lines(&text).expect("exact literal path attributes");
        assert_eq!(
            actual, attributes,
            "{relative} changed its path reachability"
        );
        for attribute in attributes {
            let target = path_attribute_target(attribute).expect("literal path target");
            let source_dir = family
                .join(relative)
                .parent()
                .expect("production source parent")
                .to_path_buf();
            let resolved = fs::canonicalize(source_dir.join(target))
                .unwrap_or_else(|error| panic!("resolve {relative} {target}: {error}"));
            assert!(
                resolved.starts_with(fs::canonicalize(&family).expect("canonical family root")),
                "{relative} path target escapes the family: {target}"
            );
            assert!(
                resolved.is_file(),
                "{relative} path target is not a file: {target}"
            );
        }
    }
    assert!(production.contains(&PathBuf::from("contracts/generated/rust/schema_bundle.rs")));
    let include_sites = production
        .iter()
        .filter_map(|relative| {
            let source = fs::read_to_string(family.join(relative)).expect("UTF-8 Rust source");
            (!include_macro_ranges(&source)
                .expect("well-formed include macro inventory")
                .is_empty())
            .then(|| relative.clone())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        include_sites,
        [PathBuf::from("crates/bullet-wire/src/lib.rs")]
    );
    let include_source = fs::read_to_string(family.join(&include_sites[0])).unwrap();
    assert!(has_only_canonical_include(&include_source));

    let overrides = sources
        .iter()
        .filter(|path| lint_policy_override(&fs::read_to_string(path).expect("UTF-8 Rust source")))
        .map(|path| {
            path.strip_prefix(&family)
                .expect("family-relative source")
                .to_path_buf()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        overrides,
        [PathBuf::from("crates/bullet-wire/src/canonical.rs")]
    );
    let metadata = cargo_metadata(&family.join("Cargo.toml"), true);
    let expected_targets = vec![
        (
            "bullet-family".to_owned(),
            "bullet-family".to_owned(),
            vec!["bin".to_owned()],
            "src/main.rs".to_owned(),
        ),
        (
            "bullet-family".to_owned(),
            "bullet_family".to_owned(),
            vec!["lib".to_owned()],
            "src/lib.rs".to_owned(),
        ),
        (
            "bullet-wire".to_owned(),
            "bullet-contract".to_owned(),
            vec!["bin".to_owned()],
            "crates/bullet-wire/src/bin/bullet-contract.rs".to_owned(),
        ),
        (
            "bullet-wire".to_owned(),
            "bullet_wire".to_owned(),
            vec!["lib".to_owned()],
            "crates/bullet-wire/src/lib.rs".to_owned(),
        ),
    ];
    assert_eq!(
        non_test_target_inventory(&metadata, &family),
        Ok(expected_targets.clone())
    );
    let mut hostile_targets = metadata.clone();
    hostile_targets["packages"]
        .as_array_mut()
        .expect("cargo metadata packages array")
        .iter_mut()
        .find(|package| package["name"] == "bullet-wire")
        .expect("bullet-wire package")["targets"]
        .as_array_mut()
        .expect("package targets array")
        .push(serde_json::json!({
            "kind": ["bin"],
            "name": "hidden-decoder",
            "src_path": family.join("tools/hidden-decoder.rs"),
        }));
    assert_ne!(
        non_test_target_inventory(&hostile_targets, &family),
        Ok(expected_targets)
    );
    assert_eq!(
        serde_json_dependency_inventory(&metadata),
        [
            ("bullet-family".to_owned(), None),
            ("bullet-wire".to_owned(), None),
        ]
    );
    assert_eq!(
        serde_json_dependency_inventory(&hostile_renamed_dependency_metadata()),
        [("hostile-metadata".to_owned(), Some("json".to_owned()))]
    );
    let registry = Some("registry+https://github.com/rust-lang/crates.io-index".to_owned());
    let full_metadata = cargo_metadata_with_dependencies(&family.join("Cargo.toml"));
    assert_eq!(
        proc_macro_inventory(&full_metadata),
        [
            (
                "derive_arbitrary".to_owned(),
                "1.4.2".to_owned(),
                registry.clone()
            ),
            (
                "displaydoc".to_owned(),
                "0.2.7".to_owned(),
                registry.clone()
            ),
            (
                "rustversion".to_owned(),
                "1.0.23".to_owned(),
                registry.clone()
            ),
            (
                "serde_derive".to_owned(),
                "1.0.229".to_owned(),
                registry.clone()
            ),
            (
                "thiserror-impl".to_owned(),
                "2.0.20".to_owned(),
                registry.clone()
            ),
            (
                "wasm-bindgen-macro".to_owned(),
                "0.2.127".to_owned(),
                registry
            ),
        ]
    );
    let dependency_inventory = workspace_direct_dependency_inventory(&full_metadata, &family)
        .expect("workspace dependency inventory");
    assert_eq!(
        dependency_inventory,
        (
            20,
            "46afbafef6e6b1275183a974164f57f3b217b725f26edc04834fba94b348f171".to_owned()
        )
    );
    let mut hostile_direct_edge = full_metadata.clone();
    hostile_direct_edge["packages"]
        .as_array_mut()
        .expect("cargo metadata packages array")
        .iter_mut()
        .find(|package| package["name"] == "bullet-wire")
        .expect("bullet-wire package")["dependencies"]
        .as_array_mut()
        .expect("package dependency array")
        .push(serde_json::json!({
            "name": "rustversion",
            "source": "registry+https://github.com/rust-lang/crates.io-index",
            "req": "^1.0",
            "kind": null,
            "rename": null,
            "optional": false,
            "uses_default_features": true,
            "features": [],
            "target": null,
            "registry": null,
            "path": null
        }));
    assert_eq!(
        proc_macro_inventory(&hostile_direct_edge),
        proc_macro_inventory(&full_metadata),
        "the package-only inventory must demonstrate the same-version edge bypass"
    );
    assert_ne!(
        workspace_direct_dependency_inventory(&hostile_direct_edge, &family)
            .expect("hostile workspace dependency inventory"),
        dependency_inventory,
        "the direct-dependency inventory must bind an edge to an already-locked proc macro"
    );
}

#[test]
fn source_guard_rejects_qualified_imported_aliased_and_streaming_decoders() {
    for hostile in [
        "let value = serde_json::from_slice::<Value>(bytes)?;",
        "use serde_json::from_str; let value = from_str(text)?;",
        "use serde_json::{from_slice, Value}; let value = from_slice(bytes)?;",
        "use serde_json::{from_reader as parse, Value}; let value = parse(reader)?;",
        "use serde_json as json; let value = json::from_slice(bytes)?;",
        "pub use serde_json as json;",
        "pub use ::serde_json as json;",
        "pub(crate) extern crate serde_json /* gap */ as json;",
        "pub(crate) use serde_json::Value;",
        "pub(in crate) use serde_json::Value;",
        "pub use serde_json::Value as JsonValue;",
        "extern crate serde_json as json;",
        "type JsonValue = serde_json::Value; let value = text.parse::<JsonValue>()?;",
        "fn marker() {} type JsonValue = serde_json::Value;",
        "#[cfg(unix)] pub type JsonValue = serde_json::Value;",
        "use serde_json::Value; pub type JsonValue = Value;",
        "use serde_json::{Value as JsonValue}; let value: JsonValue = text.parse()?;",
        "let value = text.parse::<serde_json::Value>()?;",
        "use serde_json::Value; let value = text.parse::<Value>()?;",
        "fn parse(text: &str) -> serde_json::Value { text.parse().unwrap() }",
        "use serde_json::Value; fn f(text: &str) -> Value { text.parse /* comment */ ().unwrap() }",
        "use serde_json::Value; fn f(text: &str) -> Value { text.parse /* comment */ ::<Value>().unwrap() }",
        "use serde_json::Value; let value: Value = text.parse()?;",
        "use serde_json::Map; let value: Map<String, serde_json::Value> = text.parse()?;",
        "let value = text.parse::<serde_json::Map<String, serde_json::Value>>()?;",
        "use serde_json::Number; let value: Number = text.parse()?;",
        "let value = text.parse::<serde_json::Number>()?;",
        "use serde_json::Deserializer; let value = Deserializer::from_reader(reader);",
        "use serde_json::*; let value = from_slice::<Value>(bytes)?;",
        "use serde_json::de; let value = de::Deserializer::from_slice(bytes);",
        "use serde_json::{de::Deserializer as JsonDecoder, Value}; let value = JsonDecoder::from_slice(bytes);",
        "use serde_json::{de::{Deserializer, SliceRead}, Value}; let value = Deserializer::new(SliceRead::new(bytes));",
        "use serde_json::{de::{SliceRead, StreamDeserializer}, Value}; let value = StreamDeserializer::<_, Value>::new(SliceRead::new(bytes));",
        "#[allow(clippy::disallowed_methods)] fn bypass() { let _ = serde_json::from_slice::<serde_json::Value>(b\"{}\"); }",
        "#[allow(clippy /* gap */ :: disallowed_methods)] fn lower() {} use serde_json::Value;",
        "#[warn(clippy::disallowed_methods)] fn lower() {} use serde_json::Value;",
        "#[warn(clippy /* gap */ :: disallowed_methods)] fn lower() {} use serde_json::Value;",
        "#[expect(clippy::disallowed_methods)] fn lower() {} use serde_json::Value;",
        "#[cfg_attr(unix, allow(clippy::disallowed_methods))] fn lower() {} use serde_json::Value;",
        "#[allow(clippy::all)] fn lower() {} use serde_json::Value;",
        "#[expect(clippy::all)] fn lower() {} use serde_json::Value;",
        "#[allow(clippy::style)] fn lower() {} use serde_json::Value;",
        "#[expect(clippy::style)] fn lower() {} use serde_json::Value;",
        "#[cfg_attr(all(), allow(clippy::all))] fn lower() {} use serde_json::Value;",
    ] {
        assert!(
            raw_serde_json_decoder(hostile).is_some() || lint_policy_override(hostile),
            "source guard admitted raw decoder fixture: {hostile}"
        );
    }
    for safe in [
        "use serde_json::{Map, Value, json}; let value = json!({});",
        "use serde_json::Value; let value = serde_json::from_value::<Value>(owned)?;",
        "use serde_json::Value; let value = serde_json::to_value(subject)?;",
        "use serde_json::Value; buffer.extend_from_slice(bytes);",
        "use serde_json::Value; output.copy_from_slice(bytes);",
    ] {
        assert_eq!(
            raw_serde_json_decoder(safe),
            None,
            "source guard rejected a non-decoder identifier: {safe}"
        );
    }

    let canonical = fs::read_to_string(root().join("crates/bullet-wire/src/canonical.rs")).unwrap();
    for added in [
        "let _bypass = serde_json::from_slice::<serde_json::Value>(bytes);",
        "let _bypass = serde_json::from_str::<serde_json::Value>(text);",
        "use serde_json::{from_slice as decode}; let _bypass = decode::<serde_json::Value>(bytes);",
        "let _bypass = serde_json /* gap */ :: from_slice::<serde_json::Value>(bytes);",
        "/*\nfn hidden\n*/ let _bypass = serde_json::from_slice::<serde_json::Value>(bytes);",
    ] {
        let hostile = canonical.replacen(
            "let unique = serde_json::from_str::<UniqueValue>(text)",
            &format!("{added}\n    let unique = serde_json::from_str::<UniqueValue>(text)"),
            1,
        );
        assert!(canonical_entrypoint_shape(&hostile).is_err(), "{added}");
    }
    let outside = format!(
        "{canonical}\nfn bypass(text: &str) -> serde_json::Value {{ text.parse().unwrap() }}"
    );
    assert!(canonical_entrypoint_shape(&outside).is_err());
    let module_override = format!(
        "#![allow(clippy::disallowed_methods)]\n{canonical}\nfn bypass(bytes: &[u8]) {{ let _ = serde_json::from_slice::<serde_json::Value>(bytes); }}"
    );
    assert!(canonical_entrypoint_shape(&module_override).is_err());
    let macro_composed = canonical.replacen(
        "let unique = serde_json::from_str::<UniqueValue>(text)",
        "decode_more!(text);\n    let unique = serde_json::from_str::<UniqueValue>(text)",
        1,
    );
    assert!(canonical_entrypoint_shape(&macro_composed).is_err());
    let widened_statement = canonical.replacen(
        "let parsed = token.parse::<f64>().map_err(|_| number_out_of_range())?;",
        "let _also = serde_json::from_slice::<serde_json::Value>(token.as_bytes());\n        let parsed = token.parse::<f64>().map_err(|_| number_out_of_range())?;",
        1,
    );
    assert!(canonical_entrypoint_shape(&widened_statement).is_err());
    let early_success = canonical.replacen(
        "let unique = serde_json::from_str::<UniqueValue>(text).map_err(parse_error)?;",
        "let unique = serde_json::from_str::<UniqueValue>(text).map_err(parse_error)?;\n    if std::env::consts::OS == \"windows\" { return Ok(unique.0.clone()); }",
        1,
    );
    assert!(canonical_entrypoint_shape(&early_success).is_err());
    let qualified_success = canonical.replacen(
        "if !parsed.is_finite() {",
        "if parsed == 0.0 { return Result::Ok(serde_json::Value::Null); }\n        if !parsed.is_finite() {",
        1,
    );
    assert!(canonical_entrypoint_shape(&qualified_success).is_err());
    let aliased_decoder = canonical.replacen(
        "let unique = serde_json::from_str::<UniqueValue>(text).map_err(parse_error)?;",
        "use serde_json::from_str as decode_alias;\n    let unique = decode_alias::<UniqueValue>(text).map_err(parse_error)?;",
        1,
    );
    assert!(canonical_entrypoint_shape(&aliased_decoder).is_err());
    let early_decode = canonical.replacen(
        "pub fn decode_unique_value_bounded(bytes: &[u8], max_bytes: usize) -> Result<Value, WireError> {",
        "pub fn decode_unique_value_bounded(bytes: &[u8], max_bytes: usize) -> Result<Value, WireError> {\n    #[cfg(target_os = \"linux\")]\n    return decode_reviewed_text(std::str::from_utf8(bytes).unwrap());",
        1,
    );
    assert!(canonical_entrypoint_shape(&early_decode).is_err());
    let merged_return = canonical.replacen(
        "return Err(WireError::new(\n            \"UTF8_BOM_FORBIDDEN\"",
        "returnErr(WireError::new(\n            \"UTF8_BOM_FORBIDDEN\"",
        1,
    );
    let compact = |source: &str| {
        source
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>()
    };
    assert_eq!(compact(&merged_return), compact(&canonical));
    assert!(canonical_entrypoint_shape(&merged_return).is_err());
    let bounded_outer_attribute = canonical.replacen(
        "pub fn decode_unique_value_bounded",
        "#[cfg(target_os = \"linux\")]\npub fn decode_unique_value_bounded",
        1,
    );
    assert!(canonical_entrypoint_shape(&bounded_outer_attribute).is_err());
    let prefixed_decoy = canonical.replacen(
        "fn decode_reviewed_text(text: &str)",
        "fn decode_reviewed_text_decoy() {}\n\nfn decode_reviewed_text(text: &str)",
        1,
    );
    assert!(canonical_entrypoint_shape(&prefixed_decoy).is_err());
    let duplicate_item = format!(
        "{canonical}\nfn decode_reviewed_text(text: &str) -> Result<Value, WireError> {{ decode_reviewed_text(text) }}"
    );
    assert!(canonical_entrypoint_shape(&duplicate_item).is_err());
    let outer_attribute = canonical.replacen(
        "fn decode_reviewed_text(text: &str)",
        "#[inline]\nfn decode_reviewed_text(text: &str)",
        1,
    );
    assert!(canonical_entrypoint_shape(&outer_attribute).is_err());
    let marker_outside = canonical.replacen(
        "validate_value(&unique.0)?;",
        "let _validation_was_removed = &unique.0;",
        1,
    ) + "\nfn marker_decoy(value: &serde_json::Value) { let _ = validate_value(value); }\n";
    assert!(canonical_entrypoint_shape(&marker_outside).is_err());
    let helper_bypass = canonical.replacen(
        "(0xfdd0..=0xfdef).contains(&codepoint) || codepoint & 0xffff >= 0xfffe",
        "false",
        1,
    );
    assert!(canonical_entrypoint_shape(&helper_bypass).is_err());
    let canonical_crlf = canonical.replace('\n', "\r\n");
    assert_eq!(canonical_entrypoint_shape(&canonical_crlf), Ok(()));
    let canonical_lone_cr = format!("{canonical}\r");
    assert!(canonical_entrypoint_shape(&canonical_lone_cr).is_err());

    for associated in [
        "use serde_json::Value; use std::str::FromStr; fn f(s: &str) -> Value { Value::from_str(s).unwrap() }",
        "use serde_json::{Map, Value}; use std::str::FromStr; fn f(s: &str) { let _ = <Map<String, Value> as FromStr>::from_str(s); }",
        "use serde_json::Number; use std::str::FromStr; fn f(s: &str) { let _ = Number::from_str(s); }",
    ] {
        assert_eq!(
            associated_call_count(&rust_code_skeleton(associated), "from_str"),
            1
        );
    }
    assert!(
        path_attribute_lines("#[path = concat!(env!(\"OUT_DIR\"), \"/raw.rs\")]\nmod raw;")
            .is_err()
    );
    assert!(!test_modules_are_cfg_gated(
        "#[path = \"tests.rs\"]\nmod tests;"
    ));
    assert!(!test_modules_are_cfg_gated("#[cfg(not(test))]\nmod tests;"));
    assert!(!test_modules_are_cfg_gated(
        "#[cfg(any(test, not(test)))]\nmod tests;"
    ));
    assert!(!test_modules_are_cfg_gated("mod/* gap */tests;"));
    assert!(!test_modules_are_cfg_gated(
        "#[cfg(test)]\npub(crate) mod tests;"
    ));
    assert!(!test_modules_are_cfg_gated(
        "#[cfg(test)]\npub\nmod\n tests;"
    ));
    assert!(!test_modules_are_cfg_gated("#[cfg(test)]\nmod r#tests;"));
    assert!(test_modules_are_cfg_gated("#[cfg(test)]\nmod tests;"));
    assert!(test_modules_are_cfg_gated(
        "#[cfg(test)]\n// retained comment\nmod\n/* gap */ tests\n;"
    ));
    assert!(!test_modules_are_cfg_gated(
        "// no cfg\nmod\n/* gap */ tests\n;"
    ));
    assert_eq!(
        indirect_attribute_redirects_path(
            "#[cfg_attr(all(), path = \"../../../tests/hidden.rs\")]\nmod hidden;"
        ),
        Ok(true)
    );
    for hostile_attribute in [
        "#[rustversion::attr(since(1.95), path = \"../../tests/hidden.rs\")]\nmod hidden;",
        "#[cfg_attr(all(), rustversion::attr(since(1.95), path = \"../../tests/hidden.rs\"))]\nmod hidden;",
    ] {
        assert_eq!(
            indirect_attribute_redirects_path(hostile_attribute),
            Ok(true)
        );
        assert_eq!(
            qualified_attribute_paths(hostile_attribute),
            Ok(vec!["rustversion::attr".to_owned()])
        );
    }
    assert_eq!(
        indirect_attribute_redirects_path(
            "#[rv(since(1.95), nested = [allow(dead_code)], path = \"../../tests/hidden.rs\")]\nmod hidden;"
        ),
        Ok(true)
    );
    assert_eq!(
        indirect_attribute_redirects_path(
            "use rustversion::attr as path;\n#[path (since(1.95), path = \"../../tests/hidden.rs\")]\nmod hidden;"
        ),
        Ok(true)
    );
    for hostile_attribute in [
        "# [cfg_attr(all(), path = \"../../tests/hidden.rs\")]\nmod hidden;",
        "#/* gap */[cfg_attr(all(), path = \"../../tests/hidden.rs\")]\nmod hidden;",
        "#\n[cfg_attr(all(), path = \"../../tests/hidden.rs\")]\nmod hidden;",
        "# ! [cfg_attr(all(), path = \"../../tests/hidden.rs\")]\nmod hidden;",
        "#!/* gap */[cfg_attr(all(), path = \"../../tests/hidden.rs\")]\nmod hidden;",
    ] {
        assert_eq!(
            indirect_attribute_redirects_path(hostile_attribute),
            Ok(true),
            "attribute trivia hid an indirect path: {hostile_attribute}"
        );
    }
    let macro_loader = r#"
        macro_rules! load { ($p:meta) => { #[$p] mod hidden; } }
        load!(path = "../../../tests/hidden.rs");
    "#;
    assert_eq!(indirect_attribute_redirects_path(macro_loader), Ok(true));
    assert_eq!(macro_arguments_assign_path(macro_loader), Ok(true));
    let split_lint_macro = r#"
        macro_rules! lower {
            ($lvl:ident,$tool:ident,$lint:ident,$item:item) => {
                #[$lvl($tool::$lint)] $item
            }
        }
        lower!(expect, clippy, disallowed_methods,
            pub fn bypass(bytes: &[u8]) {
                let _ = serde_json::from_slice::<serde_json::Value>(bytes);
            }
        );
    "#;
    assert_eq!(
        indirect_attribute_redirects_path(split_lint_macro),
        Ok(true)
    );
    let token_synthesizer = r#"
        macro_rules! load {
            ($hash:tt, $open:tt, $name:ident, $equal:tt, $value:literal, $close:tt) => {
                $hash $open $name $equal $value $close mod hidden;
            }
        }
        load!(#, [, path, =, "../../../tests/hidden.rs", ]);
    "#;
    assert!(macro_tt_fragment(token_synthesizer));

    let include_source = fs::read_to_string(root().join("crates/bullet-wire/src/lib.rs")).unwrap();
    assert!(has_only_canonical_include(&include_source));
    let include_alias = format!(
        "{include_source}\nuse std::include as inc;\ninc!(concat!(env!(\"OUT_DIR\"), \"/hidden.rs\"));"
    );
    assert_eq!(
        rust_identifier_count(&rust_code_skeleton(&include_alias), "include"),
        2
    );
    for hostile_include in [
        format!("{include_source}\ninclude ! (concat!(env!(\"OUT_DIR\"), \"/generated.rs\"));"),
        format!("{include_source}\ninclude ! (generated_path());"),
        format!("{include_source}\ninclude ! (\"../../../tests/raw.rs\");"),
    ] {
        assert_eq!(include_macro_ranges(&hostile_include).unwrap().len(), 2);
        assert!(!has_only_canonical_include(&hostile_include));
    }
    let decoys_and_canonical = r#"
        const DECOY: &str = "include ! (concat!(env!(\"OUT_DIR\"), \"/x.rs\"))";
        /* include ! ("ignored.rs") */
        include ! ( concat ! ( env ! ( "CARGO_MANIFEST_DIR" ) , "/../../contracts/generated/rust/schema_bundle.rs" ) );
    "#;
    assert_eq!(include_macro_ranges(decoys_and_canonical).unwrap().len(), 1);
    assert!(has_only_canonical_include(decoys_and_canonical));
}

#[test]
fn strict_type_decode_rejects_unknown_fields() {
    let mut value = serde_json::from_slice::<serde_json::Value>(
        &fs::read(root().join("policy/v1alpha1/policy-template.json")).unwrap(),
    )
    .unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("surprise".to_owned(), serde_json::json!(true));
    let bytes = canonical_json(&value).unwrap();
    assert_eq!(
        decode_canonical::<PolicyTemplateV1>(&bytes)
            .unwrap_err()
            .code(),
        "DOCUMENT_SCHEMA_INVALID"
    );
}

#[test]
fn framing_and_domains_disambiguate_hostile_preimages() {
    let joined_left = hash_framed_bytes("golden.left", b"ab\0c").unwrap();
    let joined_right = hash_framed_bytes("golden.left", b"a\0bc").unwrap();
    let other_domain = hash_framed_bytes("golden.right", b"ab\0c").unwrap();
    assert_ne!(joined_left, joined_right);
    assert_ne!(joined_left, other_domain);
    assert_eq!(
        independent_framed_digest("golden.left", b"ab\0c"),
        joined_left.to_hex(),
        "test-local framing drifted from the production wire format"
    );
    assert_eq!(
        hash_framed_bytes("UPPER", b"x").unwrap_err().code(),
        "INVALID_HASH_DOMAIN"
    );
}

#[test]
fn generated_cross_language_golden_is_itself_canonical() {
    let bytes = fs::read(root().join("fixtures/canonical/canonical-golden.json")).unwrap();
    let value = decode_canonical_value(&bytes).unwrap();
    assert_eq!(
        value["canonical_json_utf8"],
        r#"{"a":"é","array":[true,null,17],"z":"last"}"#
    );
    assert_eq!(value["domain"], "golden.cross-language");
}
