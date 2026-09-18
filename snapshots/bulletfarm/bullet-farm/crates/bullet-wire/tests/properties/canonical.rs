use bullet_wire::{
    Blake3Digest, canonical_json, decode_canonical, decode_canonical_value, decode_unique_value,
    hash_canonical, hash_framed_bytes,
};
use serde_json::Value;

use super::support::{CASES, DOMAIN_ALPHABET, MAX_SAFE, Rng, code, count_objects, ctx, render};

#[test]
fn canonical_encoding_is_deterministic_for_random_values() {
    for index in 0..CASES {
        let (mut rng, context) = (Rng::for_case(index), ctx(index));
        let value = rng.object(3);
        let expected = canonical_json(&value).expect("generated value");
        let mut texts = Vec::new();
        for _ in 0..3 {
            let mut text = String::new();
            render(&value, &mut rng, &mut (usize::MAX, 0), &mut text);
            let decoded = decode_unique_value(text.as_bytes())
                .unwrap_or_else(|error| panic!("{context}: {error}"));
            assert_eq!(decoded, value, "{context}: loose text changed the value");
            assert_eq!(
                canonical_json(&decoded).expect("decoded value"),
                expected,
                "{context}: canonical bytes differ"
            );
            texts.push(text);
        }
        assert!(
            texts
                .iter()
                .any(|text| text.as_bytes() != expected.as_slice()),
            "{context}: no rendering differed from canonical form"
        );
        assert_eq!(
            decode_canonical_value(&expected).expect("canonical value"),
            value,
            "{context}"
        );
    }
    assert_eq!(
        code(decode_canonical_value(br#"{"b":1,"a":2}"#)),
        Err("NON_CANONICAL_JSON")
    );
    assert_eq!(
        canonical_json(&decode_unique_value(br#"{"b":1,"a":2}"#).expect("loose JSON"))
            .expect("canonical JSON"),
        br#"{"a":2,"b":1}"#
    );
}

#[test]
fn canonical_roundtrip_preserves_value() {
    for index in 0..CASES {
        let (mut rng, context) = (Rng::for_case(index), ctx(index));
        let value = rng.value(3);
        let bytes = canonical_json(&value).expect("generated value");
        let typed: Value =
            decode_canonical(&bytes).unwrap_or_else(|error| panic!("{context}: {error}"));
        assert_eq!(typed, value, "{context}: decode changed the value");
        assert_eq!(
            canonical_json(&typed).expect("typed value"),
            bytes,
            "{context}: re-encoding drifted"
        );
        let encoded = canonical_json(&typed).expect("typed value");
        assert_eq!(
            decode_canonical_value(&encoded).expect("second decode"),
            value,
            "{context}: second round trip drifted"
        );
    }
    assert_eq!(code(decode_canonical_value(b"")), Err("EMPTY_DOCUMENT"));
    assert_eq!(
        code(decode_canonical_value(b"{\"a\":1 }")),
        Err("NON_CANONICAL_JSON")
    );
    assert_eq!(
        code(decode_canonical::<u8>(b"300")),
        Err("DOCUMENT_SCHEMA_INVALID")
    );
}

#[test]
fn duplicate_keys_are_always_refused() {
    for index in 0..CASES {
        let (mut rng, context) = (Rng::for_case(index), ctx(index));
        let value = rng.object(3);
        let target = rng.below(count_objects(&value));
        let mut text = String::new();
        render(&value, &mut rng, &mut (target, 0), &mut text);
        assert_eq!(
            code(decode_unique_value(text.as_bytes())),
            Err("DUPLICATE_JSON_KEY"),
            "{context}: {text}"
        );
        assert_eq!(
            code(decode_canonical_value(text.as_bytes())),
            Err("DUPLICATE_JSON_KEY"),
            "{context}"
        );
        let mut clean = String::new();
        render(&value, &mut rng, &mut (usize::MAX, 0), &mut clean);
        assert_eq!(
            decode_unique_value(clean.as_bytes()).expect("clean loose JSON"),
            value,
            "{context}: clean text refused"
        );
    }
    assert_eq!(
        code(decode_unique_value(br#"{"a":1,"a":1}"#)),
        Err("DUPLICATE_JSON_KEY")
    );
    assert_eq!(
        code(decode_unique_value(br#"{"a":1,"b":{"a":1}}"#)).map(|_| ()),
        Ok(())
    );
}

#[test]
fn unsafe_integers_are_always_refused() {
    let wrap = |rng: &mut Rng, digits: &str| match rng.below(4) {
        0 => digits.to_owned(),
        1 => format!("[{digits}]"),
        2 => format!("{{\"k\":{digits}}}"),
        _ => format!("{{\"a\":{{\"b\":[0,{digits}]}}}}"),
    };
    for index in 0..CASES {
        let (mut rng, context) = (Rng::for_case(index), ctx(index));
        let mut magnitude = rng.next_u64();
        if magnitude <= MAX_SAFE {
            magnitude = MAX_SAFE + 1 + magnitude % 4096;
        }
        let sign = if rng.coin() { "-" } else { "" };
        let unsafe_text = wrap(&mut rng, &format!("{sign}{magnitude}"));
        assert_eq!(
            code(decode_unique_value(unsafe_text.as_bytes())),
            Err("UNSAFE_JSON_INTEGER"),
            "{context}: {unsafe_text}"
        );
        assert_eq!(
            code(decode_canonical_value(unsafe_text.as_bytes())),
            Err("UNSAFE_JSON_INTEGER"),
            "{context}"
        );
        let safe = (rng.next_u64() % (MAX_SAFE + 1)) as i64;
        let signed = if rng.coin() { -safe } else { safe };
        let safe_text = wrap(&mut rng, &signed.to_string());
        let decoded = decode_unique_value(safe_text.as_bytes())
            .unwrap_or_else(|error| panic!("{context}: {error}"));
        assert_eq!(
            canonical_json(&decoded).expect("safe integer"),
            safe_text.as_bytes(),
            "{context}: safe integer drifted"
        );
        assert!(
            decoded.to_string().contains(&signed.to_string()),
            "{context}: safe integer lost"
        );
    }
    assert!(decode_unique_value(b"9007199254740991").is_ok());
    assert!(decode_unique_value(b"-9007199254740991").is_ok());
    assert_eq!(
        code(decode_unique_value(b"9007199254740992")),
        Err("UNSAFE_JSON_INTEGER")
    );
    assert_eq!(
        code(decode_unique_value(b"-9007199254740992")),
        Err("UNSAFE_JSON_INTEGER")
    );
    assert_eq!(
        code(decode_unique_value(b"1e16")),
        Err("UNSAFE_JSON_INTEGER")
    );
    assert!(decode_unique_value(b"1e15").is_ok());
}

fn independent_digest(domain: &[u8], bytes: &[u8]) -> Blake3Digest {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"bullet-wire.v1\0");
    for subject in [domain, bytes] {
        hasher.update(&(subject.len() as u64).to_le_bytes());
        hasher.update(subject);
    }
    Blake3Digest::from_bytes(*hasher.finalize().as_bytes())
}

#[test]
fn digest_is_domain_separated() {
    for index in 0..CASES {
        let (mut rng, context) = (Rng::for_case(index), ctx(index));
        let (left, mut right) = (rng.domain(), rng.domain());
        if left == right {
            right.push('x');
        }
        let byte_count = rng.below(128);
        let mut bytes = rng.bytes(byte_count);
        let alphabet_index = rng.below(DOMAIN_ALPHABET.len());
        bytes.insert(0, DOMAIN_ALPHABET[alphabet_index]);
        let a =
            hash_framed_bytes(&left, &bytes).unwrap_or_else(|error| panic!("{context}: {error}"));
        let b = hash_framed_bytes(&right, &bytes).expect("right domain");
        assert_ne!(a, b, "{context}: domains {left:?}/{right:?} collided");
        assert_eq!(
            a,
            independent_digest(left.as_bytes(), &bytes),
            "{context}: framing drifted"
        );
        assert_eq!(
            a,
            hash_framed_bytes(&left, &bytes).expect("repeat digest"),
            "{context}: digest unstable"
        );
        let shifted = format!("{left}{}", bytes[0] as char);
        assert_ne!(
            a,
            hash_framed_bytes(&shifted, &bytes[1..]).expect("shifted digest"),
            "{context}: frame boundary ambiguous"
        );
        let value = rng.value(2);
        assert_ne!(
            hash_canonical(&left, &value).expect("left canonical digest"),
            hash_canonical(&right, &value).expect("right canonical digest"),
            "{context}: canonical domains collided"
        );
    }
    assert_eq!(
        code(hash_framed_bytes("Bad", b"x")),
        Err("INVALID_HASH_DOMAIN")
    );
    assert_eq!(
        code(hash_framed_bytes("", b"x")),
        Err("INVALID_HASH_DOMAIN")
    );
    assert_ne!(
        hash_framed_bytes("a", b"").expect("domain a"),
        hash_framed_bytes("b", b"").expect("domain b")
    );
}

#[test]
fn digest_changes_for_any_single_byte_flip() {
    for index in 0..CASES {
        let (mut rng, context) = (Rng::for_case(index), ctx(index));
        let domain = rng.domain();
        let byte_count = 1 + rng.below(256);
        let mut bytes = rng.bytes(byte_count);
        let original = hash_framed_bytes(&domain, &bytes).expect("original digest");
        let (position, bit) = (rng.below(bytes.len()), 1_u8 << rng.below(8));
        bytes[position] ^= bit;
        assert_ne!(
            original,
            hash_framed_bytes(&domain, &bytes).expect("changed digest"),
            "{context}: flip at {position} unseen"
        );
        bytes[position] ^= bit;
        assert_eq!(
            original,
            hash_framed_bytes(&domain, &bytes).expect("restored digest"),
            "{context}: digest not restored"
        );
        let value = rng.object(2);
        let mut canonical = canonical_json(&value).expect("generated object");
        assert_eq!(
            hash_canonical(&domain, &value).expect("canonical digest"),
            hash_framed_bytes(&domain, &canonical).expect("framed canonical bytes"),
            "{context}"
        );
        let position = rng.below(canonical.len());
        canonical[position] ^= 1 << rng.below(8);
        assert_ne!(
            hash_canonical(&domain, &value).expect("canonical digest"),
            hash_framed_bytes(&domain, &canonical).expect("mutated bytes digest"),
            "{context}"
        );
    }
    assert_ne!(
        hash_framed_bytes("d", b"\x00").expect("zero byte"),
        hash_framed_bytes("d", b"\x01").expect("one byte")
    );
    assert_ne!(
        hash_framed_bytes("d", b"").expect("empty"),
        hash_framed_bytes("d", b"\x00").expect("zero byte")
    );
}

/// Legal JSON whitespace boundaries and ASCII letters inside string literals.
fn positions(canonical: &[u8]) -> (Vec<usize>, Vec<usize>) {
    let (mut boundaries, mut letters) = (vec![0, canonical.len()], Vec::new());
    let (mut in_string, mut escaped) = (false, false);
    for (offset, byte) in canonical.iter().copied().enumerate() {
        match (in_string, escaped, byte) {
            (true, true, _) => escaped = false,
            (true, false, b'\\') => escaped = true,
            (true, false, b'"') => in_string = false,
            (true, false, value) if value.is_ascii_alphabetic() => letters.push(offset),
            (true, false, _) => {}
            (false, _, b'"') => in_string = true,
            (false, _, b'{' | b'}' | b'[' | b']' | b',' | b':') => {
                boundaries.extend([offset, offset + 1]);
            }
            (false, _, _) => {}
        }
    }
    boundaries.sort_unstable();
    boundaries.dedup();
    (boundaries, letters)
}

#[test]
fn non_canonical_whitespace_or_escapes_are_refused_by_decode_canonical() {
    for index in 0..CASES {
        let (mut rng, context) = (Rng::for_case(index), ctx(index));
        let value = rng.object(3);
        let canonical = canonical_json(&value).expect("generated object");
        let (boundaries, letters) = positions(&canonical);
        let mut hostile = String::from_utf8(canonical.clone()).expect("canonical UTF-8");
        match rng.below(if letters.is_empty() { 2 } else { 4 }) {
            0 => {
                let boundary = boundaries[rng.below(boundaries.len())];
                hostile.insert_str(boundary, [" ", "\n", "\t"][rng.below(3)]);
            }
            1 => hostile.push_str(["\n", " ", "\r\n"][rng.below(3)]),
            2 => {
                let offset = letters[rng.below(letters.len())];
                hostile.replace_range(
                    offset..offset + 1,
                    &format!("\\u00{:02x}", canonical[offset]),
                );
            }
            _ => {
                let offset = letters[rng.below(letters.len())];
                hostile.replace_range(
                    offset..offset + 1,
                    &format!("\\u00{:02X}", canonical[offset]),
                );
            }
        }
        assert_eq!(
            code(decode_canonical_value(hostile.as_bytes())),
            Err("NON_CANONICAL_JSON"),
            "{context}: {hostile}"
        );
        assert_eq!(
            decode_unique_value(hostile.as_bytes())
                .unwrap_or_else(|error| panic!("{context}: {error}")),
            value,
            "{context}: hostile text was not merely non-canonical"
        );
        assert_eq!(
            decode_canonical_value(&canonical).expect("canonical control"),
            value,
            "{context}"
        );
    }
    assert_eq!(
        code(decode_canonical_value(b"{\"a\":1}\n")),
        Err("NON_CANONICAL_JSON")
    );
    assert_eq!(
        code(decode_canonical_value(b"{\"a\":\"\\u0041\"}")),
        Err("NON_CANONICAL_JSON")
    );
    assert_eq!(
        code(decode_canonical_value(b"{\"a\":\"\\/\"}")),
        Err("NON_CANONICAL_JSON")
    );
    assert!(decode_canonical_value(b"{\"a\":\"A/\"}").is_ok());
}
