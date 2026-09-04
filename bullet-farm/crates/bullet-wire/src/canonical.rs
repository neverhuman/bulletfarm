use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, Deserializer, Serialize, de, de::DeserializeOwned};
use serde_json::{Number, Value};
use unicode_normalization::UnicodeNormalization;

use crate::{WireError, canonical_json};

pub const MAX_CANONICAL_DOCUMENT_BYTES: usize = 1_048_576;
pub const MAX_UNIQUE_DOCUMENT_BYTES: usize = 32 * 1_048_576;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const MAX_SAFE_INTEGER_DECIMAL: &str = "9007199254740991";

#[derive(Debug, Eq, PartialEq)]
struct NormalizedDecimal {
    negative: bool,
    digits: String,
    exponent: i64,
}

pub fn decode_canonical<T>(bytes: &[u8]) -> Result<T, WireError>
where
    T: DeserializeOwned + Serialize,
{
    let value = decode_canonical_value(bytes)?;
    serde_json::from_value(value).map_err(|error| {
        WireError::new(
            "DOCUMENT_SCHEMA_INVALID",
            format!("canonical document does not match its strict type: {error}"),
        )
    })
}

pub fn decode_canonical_value(bytes: &[u8]) -> Result<Value, WireError> {
    let value = decode_unique_value(bytes)?;
    let canonical = canonical_json(&value)?;
    if bytes != canonical {
        return Err(WireError::new(
            "NON_CANONICAL_JSON",
            "input bytes differ from their RFC 8785 encoding",
        ));
    }
    Ok(value)
}

/// Decode bounded UTF-8 JSON without silently collapsing duplicate object
/// members. Unlike [`decode_canonical_value`], this admits ordinary whitespace
/// and member ordering for diagnostic and repository-owned configuration files.
pub fn decode_unique_value(bytes: &[u8]) -> Result<Value, WireError> {
    decode_unique_value_bounded(bytes, MAX_CANONICAL_DOCUMENT_BYTES)
}

/// Decode strict unique-key JSON with a caller-selected size bound. The caller
/// may narrow or widen the canonical one-MiB default, but never beyond the
/// process-wide hard ceiling.
pub fn decode_unique_value_bounded(bytes: &[u8], max_bytes: usize) -> Result<Value, WireError> {
    validate_input_size(bytes, max_bytes)?;
    let text = std::str::from_utf8(bytes).map_err(|error| {
        WireError::new(
            "INVALID_UTF8",
            format!("document is not strict UTF-8: {error}"),
        )
    })?;
    if text.starts_with('\u{feff}') {
        return Err(WireError::new(
            "UTF8_BOM_FORBIDDEN",
            "canonical documents do not carry a UTF-8 byte-order mark",
        ));
    }

    decode_reviewed_text(text)
}

fn decode_reviewed_text(text: &str) -> Result<Value, WireError> {
    let bytes = text.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if bytes[cursor] == b'"' {
            cursor += 1;
            while cursor < bytes.len() {
                match bytes[cursor] {
                    b'\\' => cursor = (cursor + 2).min(bytes.len()),
                    b'"' => {
                        cursor += 1;
                        break;
                    }
                    _ => cursor += 1,
                }
            }
            continue;
        }
        if bytes[cursor] != b'-' && !bytes[cursor].is_ascii_digit() {
            cursor += 1;
            continue;
        }

        let start = cursor;
        cursor += 1;
        while cursor < bytes.len() && !is_json_delimiter(bytes[cursor]) {
            cursor += 1;
        }
        let token = &text[start..cursor];
        let Some(normalized) = normalize_decimal(token)? else {
            continue;
        };
        if normalized.is_unsafe_integer() {
            return Err(WireError::new(
                "UNSAFE_JSON_INTEGER",
                "integer is outside the interoperable IEEE-754 safe range",
            ));
        }

        #[expect(
            clippy::disallowed_methods,
            reason = "the reviewed number boundary verifies an exact finite round trip before admission"
        )]
        let parsed = token.parse::<f64>().map_err(|_| number_out_of_range())?;
        if !parsed.is_finite() {
            return Err(number_out_of_range());
        }
        let round_trip = serde_json::to_string(&parsed)
            .map_err(|_| number_out_of_range())
            .and_then(|value| normalize_decimal(&value)?.ok_or_else(number_out_of_range))?;
        if normalized != round_trip {
            return Err(WireError::new(
                "JSON_NUMBER_PRECISION_LOSS",
                "JSON number cannot be represented without precision loss",
            ));
        }
    }
    #[expect(
        clippy::disallowed_methods,
        reason = "the reviewed document boundary decodes into the duplicate-rejecting UniqueValue visitor"
    )]
    let unique = serde_json::from_str::<UniqueValue>(text).map_err(parse_error)?;
    validate_value(&unique.0)?;
    Ok(unique.0)
}

fn is_json_delimiter(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n' | b',' | b']' | b'}')
}

fn normalize_decimal(token: &str) -> Result<Option<NormalizedDecimal>, WireError> {
    let bytes = token.as_bytes();
    let mut cursor = 0;
    let negative = bytes.first() == Some(&b'-');
    if negative {
        cursor += 1;
    }

    let integer_start = cursor;
    match bytes.get(cursor) {
        Some(b'0') => cursor += 1,
        Some(b'1'..=b'9') => {
            cursor += 1;
            while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
                cursor += 1;
            }
        }
        _ => return Ok(None),
    }
    let integer_end = cursor;
    if integer_end - integer_start > 1 && bytes[integer_start] == b'0' {
        return Ok(None);
    }

    let mut fraction_start = cursor;
    let mut fraction_end = cursor;
    if bytes.get(cursor) == Some(&b'.') {
        cursor += 1;
        fraction_start = cursor;
        while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
            cursor += 1;
        }
        fraction_end = cursor;
        if fraction_start == fraction_end {
            return Ok(None);
        }
    }

    let mut explicit_exponent = 0_i64;
    if matches!(bytes.get(cursor), Some(b'e' | b'E')) {
        cursor += 1;
        let exponent_negative = bytes.get(cursor) == Some(&b'-');
        if exponent_negative || bytes.get(cursor) == Some(&b'+') {
            cursor += 1;
        }
        let exponent_start = cursor;
        while let Some(digit @ b'0'..=b'9') = bytes.get(cursor) {
            explicit_exponent = explicit_exponent
                .checked_mul(10)
                .and_then(|value| value.checked_add(i64::from(*digit - b'0')))
                .ok_or_else(number_out_of_range)?;
            cursor += 1;
        }
        if exponent_start == cursor {
            return Ok(None);
        }
        if exponent_negative {
            explicit_exponent = explicit_exponent
                .checked_neg()
                .ok_or_else(number_out_of_range)?;
        }
    }
    if cursor != bytes.len() {
        return Ok(None);
    }

    let mut digits =
        String::with_capacity((integer_end - integer_start) + (fraction_end - fraction_start));
    digits.push_str(&token[integer_start..integer_end]);
    digits.push_str(&token[fraction_start..fraction_end]);
    let fraction_digits =
        i64::try_from(fraction_end - fraction_start).map_err(|_| number_out_of_range())?;
    let mut exponent = explicit_exponent
        .checked_sub(fraction_digits)
        .ok_or_else(number_out_of_range)?;

    let Some(first_nonzero) = digits.bytes().position(|digit| digit != b'0') else {
        return Ok(Some(NormalizedDecimal {
            negative: false,
            digits: "0".to_owned(),
            exponent: 0,
        }));
    };
    digits.drain(..first_nonzero);
    while digits.ends_with('0') {
        digits.pop();
        exponent = exponent.checked_add(1).ok_or_else(number_out_of_range)?;
    }

    Ok(Some(NormalizedDecimal {
        negative,
        digits,
        exponent,
    }))
}

impl NormalizedDecimal {
    fn is_unsafe_integer(&self) -> bool {
        if self.digits == "0" || self.exponent < 0 {
            return false;
        }
        let Ok(exponent) = usize::try_from(self.exponent) else {
            return true;
        };
        let Some(width) = self.digits.len().checked_add(exponent) else {
            return true;
        };
        match width.cmp(&MAX_SAFE_INTEGER_DECIMAL.len()) {
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => {
                let mut integer = self.digits.clone();
                integer.extend(std::iter::repeat_n('0', exponent));
                integer.as_str() > MAX_SAFE_INTEGER_DECIMAL
            }
        }
    }
}

fn number_out_of_range() -> WireError {
    WireError::new(
        "JSON_NUMBER_OUT_OF_RANGE",
        "JSON number is outside the finite interoperable range",
    )
}

fn validate_input_size(bytes: &[u8], max_bytes: usize) -> Result<(), WireError> {
    if max_bytes == 0 || max_bytes > MAX_UNIQUE_DOCUMENT_BYTES {
        return Err(WireError::new(
            "DOCUMENT_LIMIT_INVALID",
            format!("document limit must be between 1 and {MAX_UNIQUE_DOCUMENT_BYTES} bytes"),
        ));
    }
    if bytes.is_empty() {
        return Err(WireError::new("EMPTY_DOCUMENT", "document is empty"));
    }
    if bytes.len() > max_bytes {
        return Err(WireError::new(
            "DOCUMENT_TOO_LARGE",
            format!("document is {} bytes; maximum is {max_bytes}", bytes.len()),
        ));
    }
    Ok(())
}

fn parse_error(error: serde_json::Error) -> WireError {
    let reason = error.to_string();
    let code = if reason.contains("duplicate object key") {
        "DUPLICATE_JSON_KEY"
    } else {
        "INVALID_JSON"
    };
    let public_reason = if code == "DUPLICATE_JSON_KEY" {
        "document contains a duplicate JSON object member".to_owned()
    } else {
        reason
    };
    WireError::new(code, public_reason)
}

fn validate_value(value: &Value) -> Result<(), WireError> {
    match value {
        Value::Array(values) => values.iter().try_for_each(validate_value),
        Value::Object(values) => values.iter().try_for_each(|(key, value)| {
            validate_string(key)?;
            validate_value(value)
        }),
        Value::String(value) => validate_string(value),
        Value::Number(number) => validate_number(number),
        Value::Null | Value::Bool(_) => Ok(()),
    }
}

fn validate_number(number: &Number) -> Result<(), WireError> {
    let outside_safe_range = number
        .as_u64()
        .is_some_and(|value| value > MAX_SAFE_INTEGER)
        || number
            .as_i64()
            .is_some_and(|value| value.unsigned_abs() > MAX_SAFE_INTEGER);
    if outside_safe_range {
        return Err(WireError::new(
            "UNSAFE_JSON_INTEGER",
            "integer is outside the interoperable IEEE-754 safe range",
        ));
    }
    Ok(())
}

fn validate_string(value: &str) -> Result<(), WireError> {
    for character in value.chars() {
        let codepoint = character as u32;
        if character.is_control() && !matches!(character, '\t' | '\n' | '\r') {
            return Err(WireError::new(
                "CONTROL_CHARACTER_FORBIDDEN",
                format!("string contains control character U+{codepoint:04X}"),
            ));
        }
        if is_directional_control(character) {
            return Err(WireError::new(
                "DIRECTIONAL_CONTROL_FORBIDDEN",
                format!("string contains directional control U+{codepoint:04X}"),
            ));
        }
        if is_unicode_15_1_default_ignorable(codepoint) {
            return Err(WireError::new(
                "ZERO_WIDTH_CHARACTER_FORBIDDEN",
                format!("string contains zero-width character U+{codepoint:04X}"),
            ));
        }
        if is_noncharacter(codepoint) {
            return Err(WireError::new(
                "UNICODE_NONCHARACTER_FORBIDDEN",
                format!("string contains Unicode noncharacter U+{codepoint:04X}"),
            ));
        }
    }
    if !value.nfc().eq(value.chars()) {
        return Err(WireError::new(
            "NON_NFC_STRING",
            "string is not Unicode NFC",
        ));
    }
    Ok(())
}

fn is_directional_control(character: char) -> bool {
    matches!(
        character,
        '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'
    )
}

// Frozen from Unicode 15.1 DerivedCoreProperties.txt. That property has no
// cross-version stability guarantee, so changing the Unicode version is an
// explicit compatibility decision rather than an ambient dependency update.
fn is_unicode_15_1_default_ignorable(codepoint: u32) -> bool {
    matches!(
        codepoint,
        0x00ad
            | 0x034f
            | 0x061c
            | 0x115f..=0x1160
            | 0x17b4..=0x17b5
            | 0x180b..=0x180f
            | 0x200b..=0x200f
            | 0x202a..=0x202e
            | 0x2060..=0x206f
            | 0x3164
            | 0xfe00..=0xfe0f
            | 0xfeff
            | 0xffa0
            | 0xfff0..=0xfff8
            | 0x1bca0..=0x1bca3
            | 0x1d173..=0x1d17a
            | 0xe0000..=0xe0fff
    )
}

fn is_noncharacter(codepoint: u32) -> bool {
    (0xfdd0..=0xfdef).contains(&codepoint) || codepoint & 0xffff >= 0xfffe
}

struct UniqueValue(Value);

impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueValueVisitor)
    }
}

struct UniqueValueVisitor;

impl<'de> de::Visitor<'de> for UniqueValueVisitor {
    type Value = UniqueValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Number(value.into())))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Number(value.into())))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Number::from_f64(value)
            .map(Value::Number)
            .map(UniqueValue)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::String(value.to_owned())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::String(value)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Null))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Null))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<UniqueValue>()? {
            values.push(value.0);
        }
        Ok(UniqueValue(Value::Array(values)))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut values = BTreeMap::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom("duplicate object key"));
            }
            values.insert(key, map.next_value::<UniqueValue>()?.0);
        }
        Ok(UniqueValue(Value::Object(values.into_iter().collect())))
    }
}
