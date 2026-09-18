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
