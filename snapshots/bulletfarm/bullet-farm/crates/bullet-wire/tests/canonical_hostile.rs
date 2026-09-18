include!("canonical_hostile/support.rs");
include!("canonical_hostile/lexical.rs");

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

include!("canonical_hostile/canonical_tests.rs");
include!("canonical_hostile/inventory_tests.rs");
include!("canonical_hostile/hostile_tests.rs");
