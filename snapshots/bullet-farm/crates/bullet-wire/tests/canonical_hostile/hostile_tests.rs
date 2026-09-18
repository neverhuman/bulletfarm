const NUMBER_ALLOWANCE: &str = "#[expect(clippy::disallowed_methods,reason=\"thereviewednumberboundaryverifiesanexactfiniteroundtripbeforeadmission\")]letparsed=token.parse::<f64>().map_err(|_|number_out_of_range())?;";

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
