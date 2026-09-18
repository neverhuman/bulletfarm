const DOCUMENT_ALLOWANCE: &str = "#[expect(clippy::disallowed_methods,reason=\"therevieweddocumentboundarydecodesintotheduplicate-rejectingUniqueValuevisitor\")]letunique=serde_json::from_str::<UniqueValue>(text).map_err(parse_error)?;";

const BOUNDED_ENTRYPOINT: &str = "fndecode_unique_value_bounded(bytes:&[u8],max_bytes:usize)->Result<Value,WireError>{validate_input_size(bytes,max_bytes)?;lettext=std::str::from_utf8(bytes).map_err(|error|{WireError::new(\"INVALID_UTF8\",format!(\"documentisnotstrictUTF-8:{error}\"),)})?;iftext.starts_with('\\u{feff}'){returnErr(WireError::new(\"UTF8_BOM_FORBIDDEN\",\"canonicaldocumentsdonotcarryaUTF-8byte-ordermark\",));}decode_reviewed_text(text)}";

const BOUNDED_ENTRYPOINT_DIGEST: &str =
    "e1a9a21cf4361d343c08c84ab432dae90eedd7b85729da84e997868028fa01b4";

const REVIEWED_ENTRYPOINT_DIGEST: &str =
    "b5dc3cd32a988d760294aa55560db470c5fccb49c6b01cfd0e1bf5c1a121ddfd";

const CANONICAL_SOURCE_DIGEST: &str =
    "ddb16e6df4e2480a189b7d542e60da0783528a6a0e76463bf02ab2565e982481";

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
