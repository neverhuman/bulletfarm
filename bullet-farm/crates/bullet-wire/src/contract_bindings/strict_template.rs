pub(super) const RUST_CORE: &str = r###"
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractValidationErrorV1 {
    pub code: &'static str,
    pub path: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum SchemaVersionLiteralV1 {
    #[serde(rename = "v1alpha1")]
    V1alpha1,
}

fn bullet_generated_error(path: impl Into<String>) -> ContractValidationErrorV1 {
    ContractValidationErrorV1 { code: "DOCUMENT_SCHEMA_INVALID", path: path.into() }
}

fn bullet_generated_path(parent: &str, child: &str) -> String {
    format!("{parent}/{}", child.replace('~', "~0").replace('/', "~1"))
}

#[allow(dead_code)]
#[derive(serde::Serialize)]
#[serde(untagged)]
enum BulletGeneratedUniqueJsonValueV1 {
    Null,
    Bool(bool),
    Integer(i64),
    String(String),
    Array(Vec<Self>),
    Object(std::collections::BTreeMap<String, Self>),
}

struct BulletGeneratedUniqueSeedV1 { path: String, depth: usize }

impl<'de> serde::de::DeserializeSeed<'de> for BulletGeneratedUniqueSeedV1 {
    type Value = BulletGeneratedUniqueJsonValueV1;
    fn deserialize<D: serde::Deserializer<'de>>(self, decoder: D) -> Result<Self::Value, D::Error> {
        decoder.deserialize_any(BulletGeneratedUniqueVisitorV1 { path: self.path, depth: self.depth })
    }
}

struct BulletGeneratedUniqueVisitorV1 { path: String, depth: usize }

impl<'de> serde::de::Visitor<'de> for BulletGeneratedUniqueVisitorV1 {
    type Value = BulletGeneratedUniqueJsonValueV1;
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a bounded unambiguous JSON value")
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> { Ok(BulletGeneratedUniqueJsonValueV1::Null) }
    fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> { Ok(BulletGeneratedUniqueJsonValueV1::Null) }
    fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Self::Value, E> { Ok(BulletGeneratedUniqueJsonValueV1::Bool(value)) }
    fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Self::Value, E> {
        if value.unsigned_abs() > 9_007_199_254_740_991 { Err(E::custom("BULLET_ROOT")) } else { Ok(BulletGeneratedUniqueJsonValueV1::Integer(value)) }
    }
    fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Self::Value, E> {
        if value > 9_007_199_254_740_991 { Err(E::custom("BULLET_ROOT")) } else { Ok(BulletGeneratedUniqueJsonValueV1::Integer(value as i64)) }
    }
    fn visit_f64<E: serde::de::Error>(self, _: f64) -> Result<Self::Value, E> { Err(E::custom("BULLET_ROOT")) }
    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> { Ok(BulletGeneratedUniqueJsonValueV1::String(value.to_owned())) }
    fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Self::Value, E> { Ok(BulletGeneratedUniqueJsonValueV1::String(value)) }
    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
        if self.depth >= 128 { return Err(serde::de::Error::custom("BULLET_ROOT")); }
        let mut values = Vec::new();
        loop {
            let seed = BulletGeneratedUniqueSeedV1 { path: bullet_generated_path(&self.path, &values.len().to_string()), depth: self.depth + 1 };
            match serde::de::SeqAccess::next_element_seed(&mut sequence, seed)? {
                Some(value) => values.push(value), None => break,
            }
        }
        Ok(BulletGeneratedUniqueJsonValueV1::Array(values))
    }
    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        if self.depth >= 128 { return Err(serde::de::Error::custom("BULLET_ROOT")); }
        let mut values = std::collections::BTreeMap::new();
        while let Some(key) = serde::de::MapAccess::next_key::<String>(&mut map)? {
            let path = bullet_generated_path(&self.path, &key);
            if values.contains_key(&key) { return Err(serde::de::Error::custom(format!("BULLET_DUPLICATE:{}:{path}", path.len()))); }
            let seed = BulletGeneratedUniqueSeedV1 { path, depth: self.depth + 1 };
            values.insert(key, serde::de::MapAccess::next_value_seed(&mut map, seed)?);
        }
        Ok(BulletGeneratedUniqueJsonValueV1::Object(values))
    }
}

fn bullet_generated_decode_unique(bytes: &[u8]) -> Result<BulletGeneratedUniqueJsonValueV1, ContractValidationErrorV1> {
    if bytes.is_empty() || bytes.len() > 33_554_432 || std::str::from_utf8(bytes).is_err()
        || bytes.starts_with(&[0xef, 0xbb, 0xbf]) || bullet_generated_has_negative_zero(bytes) {
        return Err(bullet_generated_error(""));
    }
    let mut decoder = serde_json::Deserializer::from_slice(bytes);
    decoder.disable_recursion_limit();
    let seed = BulletGeneratedUniqueSeedV1 { path: String::new(), depth: 0 };
    let value = serde::de::DeserializeSeed::deserialize(seed, &mut decoder).map_err(bullet_generated_serde_error)?;
    decoder.end().map_err(|_| bullet_generated_error(""))?;
    Ok(value)
}

fn bullet_generated_has_negative_zero(bytes: &[u8]) -> bool {
    let mut quoted = false;
    let mut escaped = false;
    for (index, byte) in bytes.iter().copied().enumerate() {
        if quoted {
            if byte == b'"' && !escaped { quoted = false; }
            escaped = byte == b'\\' && !escaped;
            if byte != b'\\' { escaped = false; }
        } else if byte == b'"' {
            quoted = true;
        } else if byte == b'-' && bytes.get(index + 1) == Some(&b'0')
            && bytes.get(index + 2).is_none_or(|next| matches!(next, b' ' | b'\t' | b'\r' | b'\n' | b',' | b']' | b'}'))
        {
            return true;
        }
    }
    false
}

fn bullet_generated_serde_error(error: serde_json::Error) -> ContractValidationErrorV1 {
    let text = error.to_string();
    let path = text.strip_prefix("BULLET_DUPLICATE:").and_then(|value| value.split_once(':')).and_then(|(length, value)| length.parse::<usize>().ok().and_then(|length| value.get(..length))).map_or("", |value| value);
    bullet_generated_error(path)
}

fn bullet_generated_object(value: BulletGeneratedUniqueJsonValueV1, path: &str) -> Result<std::collections::BTreeMap<String, BulletGeneratedUniqueJsonValueV1>, ContractValidationErrorV1> {
    match value { BulletGeneratedUniqueJsonValueV1::Object(value) => Ok(value), _ => Err(bullet_generated_error(path)) }
}
fn bullet_generated_take(values: &mut std::collections::BTreeMap<String, BulletGeneratedUniqueJsonValueV1>, name: &str, path: &str) -> Result<BulletGeneratedUniqueJsonValueV1, ContractValidationErrorV1> {
    values.remove(name).ok_or_else(|| bullet_generated_error(bullet_generated_path(path, name)))
}
fn bullet_generated_closed(values: std::collections::BTreeMap<String, BulletGeneratedUniqueJsonValueV1>, path: &str) -> Result<(), ContractValidationErrorV1> {
    match values.into_keys().next() { Some(name) => Err(bullet_generated_error(bullet_generated_path(path, &name))), None => Ok(()) }
}
fn bullet_generated_before(values: &std::collections::BTreeMap<String, BulletGeneratedUniqueJsonValueV1>, name: &str, path: &str) -> Result<(), ContractValidationErrorV1> {
    match values.keys().next().filter(|actual| actual.as_str() < name) {
        Some(actual) => Err(bullet_generated_error(bullet_generated_path(path, actual))), None => Ok(()),
    }
}
"###;

pub(super) const RUST_STRING: &str = r###"
fn bullet_generated_string(value: BulletGeneratedUniqueJsonValueV1, path: &str) -> Result<String, ContractValidationErrorV1> {
    match value { BulletGeneratedUniqueJsonValueV1::String(value) => Ok(value), _ => Err(bullet_generated_error(path)) }
}
"###;

pub(super) const RUST_ARRAYS: &str = r###"
fn bullet_generated_array(value: BulletGeneratedUniqueJsonValueV1, path: &str) -> Result<Vec<BulletGeneratedUniqueJsonValueV1>, ContractValidationErrorV1> {
    match value { BulletGeneratedUniqueJsonValueV1::Array(value) => Ok(value), _ => Err(bullet_generated_error(path)) }
}
fn bullet_generated_collect_nodes<T>(values: Vec<BulletGeneratedUniqueJsonValueV1>, path: &str, mut collect: impl FnMut(BulletGeneratedUniqueJsonValueV1, &str) -> Result<T, ContractValidationErrorV1>) -> Result<Vec<T>, ContractValidationErrorV1> {
    let length = values.len();
    let mut indexed = values.into_iter().enumerate().collect::<Vec<_>>();
    indexed.sort_by_cached_key(|(index, _)| index.to_string());
    let mut collected = std::collections::BTreeMap::new();
    for (index, value) in indexed {
        let child = bullet_generated_path(path, &index.to_string());
        collected.insert(index, collect(value, &child)?);
    }
    (0..length).map(|index| collected.remove(&index).ok_or_else(|| bullet_generated_error(path))).collect()
}
"###;

pub(super) const RUST_COLLECT_ARRAY: &str = r###"
fn bullet_generated_collect_array<T>(value: BulletGeneratedUniqueJsonValueV1, path: &str, mut collect: impl FnMut(BulletGeneratedUniqueJsonValueV1, &str) -> Result<T, ContractValidationErrorV1>) -> Result<Vec<T>, ContractValidationErrorV1> {
    bullet_generated_collect_nodes(bullet_generated_array(value, path)?, path, &mut collect)
}
"###;

pub(super) const RUST_INTEGER: &str = r###"
fn bullet_generated_integer(value: BulletGeneratedUniqueJsonValueV1, path: &str) -> Result<i64, ContractValidationErrorV1> {
    match value { BulletGeneratedUniqueJsonValueV1::Integer(value) => Ok(value), _ => Err(bullet_generated_error(path)) }
}
"###;

pub(super) const RUST_BOOLEAN: &str = r###"
fn bullet_generated_bool(value: BulletGeneratedUniqueJsonValueV1, path: &str) -> Result<bool, ContractValidationErrorV1> {
    match value { BulletGeneratedUniqueJsonValueV1::Bool(value) => Ok(value), _ => Err(bullet_generated_error(path)) }
}
"###;

pub(super) const RUST_BOUNDS: &str = r###"
fn bullet_generated_cardinality(length: usize, minimum: usize, maximum: usize, path: &str) -> Result<(), ContractValidationErrorV1> {
    if (minimum..=maximum).contains(&length) { Ok(()) } else { Err(bullet_generated_error(path)) }
}
"###;

pub(super) const RUST_REBASE: &str = r###"
fn bullet_generated_rebase(mut error: ContractValidationErrorV1, path: &str) -> ContractValidationErrorV1 {
    error.path = format!("{path}{}", error.path);
    error
}
"###;

pub(super) const RUST_SET: &str = r###"
fn bullet_generated_set_failure(values: &[BulletGeneratedUniqueJsonValueV1], path: &str) -> Result<Option<ContractValidationErrorV1>, ContractValidationErrorV1> {
    let mut prior: Option<Vec<u8>> = None; let mut seen = std::collections::BTreeSet::new();
    let mut failure: Option<(String, u8)> = None;
    for (index, value) in values.iter().enumerate() {
        let canonical = serde_jcs::to_vec(value).map_err(|_| bullet_generated_error(bullet_generated_path(path, &index.to_string())))?;
        if let Some(class) = if !seen.insert(canonical.clone()) { Some(0) } else { prior.as_ref().filter(|bytes| *bytes > &canonical).map(|_| 1) } {
            let candidate = (bullet_generated_path(path, &index.to_string()), class);
            if failure.as_ref().is_none_or(|current| &candidate < current) { failure = Some(candidate); }
        }
        prior = Some(canonical);
    }
    Ok(failure.map(|(path, _)| bullet_generated_error(path)))
}
"###;

pub(super) const RUST_CHOOSE: &str = r###"
fn bullet_generated_choose(first: ContractValidationErrorV1, second: Option<ContractValidationErrorV1>) -> ContractValidationErrorV1 {
    match second { Some(error) if error.path < first.path => error, _ => first }
}
"###;

pub(super) const RUST_UNIQUE: &str = r###"
fn bullet_generated_duplicate_failure(values: &[BulletGeneratedUniqueJsonValueV1], path: &str) -> Result<Option<ContractValidationErrorV1>, ContractValidationErrorV1> {
    let mut seen = std::collections::BTreeSet::new(); let mut failure: Option<String> = None;
    for (index, value) in values.iter().enumerate() { let bytes = serde_jcs::to_vec(value).map_err(|_| bullet_generated_error(path))?; if !seen.insert(bytes) { let candidate = bullet_generated_path(path, &index.to_string()); if failure.as_ref().is_none_or(|current| &candidate < current) { failure = Some(candidate); } } }
    Ok(failure.map(bullet_generated_error))
}
"###;

pub(super) const RUST_LEGACY_STRING: &str = r###"
fn bullet_generated_hex(value: &str, length: usize) -> bool { value.len() == length && value.bytes().all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f')) }
fn bullet_generated_ascii_name(value: &str, minimum: usize, maximum: usize) -> bool { (minimum..=maximum).contains(&value.len()) && value.as_bytes().first().is_some_and(u8::is_ascii_lowercase) && value.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-') }
fn bullet_generated_identifier(value: &str, maximum: usize) -> bool { value.rsplit_once('_').is_some_and(|(name, digest)| bullet_generated_ascii_name(name, 2, maximum) && bullet_generated_hex(digest, 64)) }
fn bullet_generated_legacy_string(value: BulletGeneratedUniqueJsonValueV1, path: &str, rule: &str) -> Result<String, ContractValidationErrorV1> {
    let value = bullet_generated_string(value, path)?;
    let valid = match rule {
        "any" => true, "nonempty" => !value.is_empty(), "policy" => matches!(value.as_str(), "v1alpha1" | "v1alpha2"), "identifier" => bullet_generated_identifier(&value, 16), "hex64" => bullet_generated_hex(&value, 64),
        rule if rule.strip_prefix("id:").is_some_and(|prefix| value.strip_prefix(prefix).and_then(|tail| tail.strip_prefix('_')).is_some_and(|digest| bullet_generated_hex(digest, 64))) => true,
        "git-oid" => value.strip_prefix("sha1:").is_some_and(|digest| bullet_generated_hex(digest, 40)) || value.strip_prefix("sha256:").is_some_and(|digest| bullet_generated_hex(digest, 64)), "blake3" => value.strip_prefix("blake3:").is_some_and(|digest| bullet_generated_hex(digest, 64)),
        "release-gate" => value.strip_prefix("release.").is_some_and(|tail| (1..=120).contains(&tail.len()) && tail.as_bytes().first().is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit()) && tail.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-'))),
        "native-subject" => value.split_once(':').is_some_and(|(namespace, subject)| bullet_generated_ascii_name(namespace, 1, 64) && bullet_generated_identifier(subject, 32)),
        "profile" => (2..=64).contains(&value.len()) && value.as_bytes().first().is_some_and(u8::is_ascii_lowercase) && value.as_bytes().last().is_some_and(u8::is_ascii_alphanumeric) && value.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'),
        "release-tag" => (2..=128).contains(&value.len()) && value.as_bytes().first() == Some(&b'v') && value.as_bytes().get(1).is_some_and(u8::is_ascii_digit) && value.as_bytes().get(2..).is_none_or(|tail| tail.is_empty() || tail.last().is_some_and(u8::is_ascii_alphanumeric) && tail.iter().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))),
        "signing" => value.split_once("|ed25519|SHA256:").is_some_and(|(name, digest)| (1..=128).contains(&name.len()) && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'@' | b'+' | b'-')) && (16..=96).contains(&digest.len()) && digest.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'='))),
        "ssh" => value.strip_prefix("ssh-ed25519 ").is_some_and(|key| (40..=256).contains(&key.len()) && key.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'='))),
        "repo-path" => (1..=4096).contains(&value.chars().count()) && !value.starts_with('/') && !value.chars().any(|character| matches!(character, '\\' | '\n' | '\r' | '\u{2028}' | '\u{2029}')) && value.split('/').all(|part| !matches!(part, "." | ".." | ".git")),
        "key-id" => (1..=128).contains(&value.len()) && value.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric) && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'/' | b'-')), "paseto" => value.starts_with("v4.public.") && value.chars().count() <= 32768, _ => false,
    };
    if valid { Ok(value) } else { Err(bullet_generated_error(path)) }
}
"###;

pub(super) const RUST_TEXT: &str = r###"
fn bullet_generated_valid_text(value: &str, minimum: usize, maximum: usize) -> bool {
    use unicode_normalization::UnicodeNormalization as _;
    let valid = value.nfc().eq(value.chars()) && value.chars().all(|character| {
        let code = character as u32;
        !matches!(code, 0..=0x1f | 0x7f..=0x9f | 0x61c | 0x200e..=0x200f | 0x202a..=0x202e | 0x2066..=0x206f | 0xfdd0..=0xfdef)
            && code & 0xffff < 0xfffe && !matches!(code, 0xad | 0x34f | 0x115f..=0x1160 | 0x17b4..=0x17b5 | 0x180b..=0x180f | 0x200b..=0x200d | 0x2060..=0x2065 | 0x3164 | 0xfe00..=0xfe0f | 0xfeff | 0xffa0 | 0xfff0..=0xfff8 | 0x1bca0..=0x1bca3 | 0x1d173..=0x1d17a | 0xe0000..=0xe0fff)
    });
    valid && (minimum..=maximum).contains(&value.len())
}
"###;

pub(super) const RUST_BOUNDED_ARRAY: &str = r###"
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub struct BoundedArrayV1<T, const MIN: usize, const MAX: usize> { values: Vec<T> }
impl<T, const MIN: usize, const MAX: usize> BoundedArrayV1<T, MIN, MAX> {
    pub fn try_new(values: Vec<T>) -> Result<Self, ContractValidationErrorV1> {
        if MIN > MAX || MAX == 0 || MAX > 4096 || !(MIN..=MAX).contains(&values.len()) { return Err(bullet_generated_error("")); }
        Ok(Self { values })
    }
    pub fn as_slice(&self) -> &[T] { &self.values }
    pub fn into_vec(self) -> Vec<T> { self.values }
}
impl<T, const MIN: usize, const MAX: usize> TryFrom<Vec<T>> for BoundedArrayV1<T, MIN, MAX> {
    type Error = ContractValidationErrorV1;
    fn try_from(values: Vec<T>) -> Result<Self, Self::Error> { Self::try_new(values) }
}
"###;

pub(super) const RUST_BOUNDED_SET: &str = r###"
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub struct BoundedSetV1<T, const MIN: usize, const MAX: usize> { values: Vec<T> }
impl<T: serde::Serialize, const MIN: usize, const MAX: usize> BoundedSetV1<T, MIN, MAX> {
    pub fn try_new(values: Vec<T>) -> Result<Self, ContractValidationErrorV1> {
        if MIN > MAX || MAX == 0 || MAX > 4096 || !(MIN..=MAX).contains(&values.len()) { return Err(bullet_generated_error("")); }
        let mut prior: Option<Vec<u8>> = None; let mut seen = std::collections::BTreeSet::new();
        let mut failure: Option<(String, u8)> = None;
        for (index, value) in values.iter().enumerate() {
            let canonical = serde_jcs::to_vec(value).map_err(|_| bullet_generated_error(format!("/{index}")))?;
            if let Some(class) = if !seen.insert(canonical.clone()) { Some(0) } else { prior.as_ref().filter(|bytes| *bytes > &canonical).map(|_| 1) } {
                let candidate = (format!("/{index}"), class);
                if failure.as_ref().is_none_or(|current| &candidate < current) { failure = Some(candidate); }
            }
            prior = Some(canonical);
        }
        match failure { Some((path, _)) => Err(bullet_generated_error(path)), None => Ok(Self { values }) }
    }
    pub fn as_slice(&self) -> &[T] { &self.values }
    pub fn into_vec(self) -> Vec<T> { self.values }
}
impl<T: serde::Serialize, const MIN: usize, const MAX: usize> TryFrom<Vec<T>> for BoundedSetV1<T, MIN, MAX> {
    type Error = ContractValidationErrorV1;
    fn try_from(values: Vec<T>) -> Result<Self, Self::Error> { Self::try_new(values) }
}
"###;

pub(super) const TYPESCRIPT_CORE: &str = r###"
export type ContractValidationErrorV1 = Readonly<{ code: "DOCUMENT_SCHEMA_INVALID"; path: string }>;
export type ContractDecodeResultV1<T> = Readonly<{ ok: true; value: T }> | Readonly<{ error: ContractValidationErrorV1; ok: false }>;
export type SchemaVersionLiteralV1 = "v1alpha1";
type BulletGeneratedNodeValue = null | boolean | number | string | ReadonlyArray<BulletGeneratedNode> | ReadonlyMap<string, BulletGeneratedNode>;
class BulletGeneratedNode { private readonly bulletGeneratedNodeBrand = true; private constructor(readonly value: BulletGeneratedNodeValue) {} static from(value: BulletGeneratedNodeValue): BulletGeneratedNode { const node = new BulletGeneratedNode(value); Object.freeze(node); return node; } }
class BulletGeneratedFailure { constructor(readonly path: string) {} }
const bulletGeneratedFail = (path: string): never => { throw new BulletGeneratedFailure(path); };
const bulletGeneratedPath = (parent: string, child: string): string => `${parent}/${child.replace(/~/g, "~0").replace(/\//g, "~1")}`;

function bulletGeneratedDecodeUtf8(bytes: Uint8Array): string {
  let output = "";
  for (let index = 0; index < bytes.length;) {
    const first = bytes[index++]; let code = first; let count = 0;
    if (first >= 0xc2 && first <= 0xdf) { code = first & 0x1f; count = 1; }
    else if (first >= 0xe0 && first <= 0xef) { code = first & 0x0f; count = 2; }
    else if (first >= 0xf0 && first <= 0xf4) { code = first & 0x07; count = 3; }
    else if (first >= 0x80) bulletGeneratedFail("");
    if (index + count > bytes.length) bulletGeneratedFail("");
    for (let offset = 0; offset < count; offset += 1) { const next = bytes[index++]; if ((next & 0xc0) !== 0x80) bulletGeneratedFail(""); code = (code << 6) | (next & 0x3f); }
    if ((count === 2 && code < 0x800) || (count === 3 && code < 0x10000) || code > 0x10ffff || (code >= 0xd800 && code <= 0xdfff)) bulletGeneratedFail("");
    output += String.fromCodePoint(code);
  }
  return output;
}
function bulletGeneratedEncodeUtf8(text: string): Uint8Array {
  const bytes: number[] = [];
  for (let index = 0; index < text.length; index += 1) {
    const first = text.charCodeAt(index); let code = first;
    if (first >= 0xd800 && first <= 0xdbff) { const second = text.charCodeAt(++index); if (!(second >= 0xdc00 && second <= 0xdfff)) bulletGeneratedFail(""); code = 0x10000 + ((first - 0xd800) << 10) + second - 0xdc00; }
    else if (first >= 0xdc00 && first <= 0xdfff) bulletGeneratedFail("");
    if (code < 0x80) bytes.push(code); else if (code < 0x800) bytes.push(0xc0 | (code >> 6), 0x80 | (code & 63));
    else if (code < 0x10000) bytes.push(0xe0 | (code >> 12), 0x80 | ((code >> 6) & 63), 0x80 | (code & 63));
    else bytes.push(0xf0 | (code >> 18), 0x80 | ((code >> 12) & 63), 0x80 | ((code >> 6) & 63), 0x80 | (code & 63));
  }
  return new Uint8Array(bytes);
}
const bulletGeneratedCompareBytes = (left: Uint8Array, right: Uint8Array): number => { for (let index = 0; index < Math.min(left.length, right.length); index += 1) { if (left[index] !== right[index]) return left[index] - right[index]; } return left.length - right.length; };
const bulletGeneratedCompareUtf8 = (left: string, right: string): number => bulletGeneratedCompareBytes(bulletGeneratedEncodeUtf8(left), bulletGeneratedEncodeUtf8(right));
const bulletGeneratedHasNegativeZero = (text: string): boolean => { let quoted = false; let escaped = false; for (let index = 0; index < text.length; index += 1) { const character = text[index]; if (quoted) { if (character === "\"" && !escaped) quoted = false; escaped = character === "\\" && !escaped; if (character !== "\\") escaped = false; } else if (character === "\"") quoted = true; else if (character === "-" && text[index + 1] === "0" && (text[index + 2] === undefined || [" ", "\t", "\r", "\n", ",", "]", "}"].includes(text[index + 2]))) return true; } return false; };

class BulletGeneratedParser {
  private cursor = 0;
  constructor(private readonly text: string) {}
  static bytes(input: Uint8Array): BulletGeneratedNode {
    const bytes = new Uint8Array(input);
    if (bytes.length === 0 || bytes.length > 33554432 || (bytes[0] === 0xef && bytes[1] === 0xbb && bytes[2] === 0xbf)) bulletGeneratedFail("");
    const text = bulletGeneratedDecodeUtf8(bytes); if (bulletGeneratedHasNegativeZero(text)) bulletGeneratedFail(""); return new BulletGeneratedParser(text).parse();
  }
  parse(): BulletGeneratedNode { this.space(); const value = this.value("", 0); this.space(); if (this.cursor !== this.text.length) bulletGeneratedFail(""); return value; }
  private value(path: string, depth: number): BulletGeneratedNode {
    this.space(); const token = this.text[this.cursor];
    if (token === "n") { this.keyword("null"); return BulletGeneratedNode.from(null); } if (token === "t") { this.keyword("true"); return BulletGeneratedNode.from(true); }
    if (token === "f") { this.keyword("false"); return BulletGeneratedNode.from(false); } if (token === "\"") return BulletGeneratedNode.from(this.string());
    if (token === "[") { if (depth >= 128) bulletGeneratedFail(""); return BulletGeneratedNode.from(this.array(path, depth + 1)); }
    if (token === "{") { if (depth >= 128) bulletGeneratedFail(""); return BulletGeneratedNode.from(this.object(path, depth + 1)); } return BulletGeneratedNode.from(this.integer(path));
  }
  private array(path: string, depth: number): ReadonlyArray<BulletGeneratedNode> {
    this.cursor += 1; const values: BulletGeneratedNode[] = []; this.space(); if (this.take("]")) return Object.freeze(values);
    for (;;) { values.push(this.value(bulletGeneratedPath(path, String(values.length)), depth)); if (this.take("]")) return Object.freeze(values); if (!this.take(",")) bulletGeneratedFail(""); }
  }
  private object(path: string, depth: number): ReadonlyMap<string, BulletGeneratedNode> {
    this.cursor += 1; const values = new Map<string, BulletGeneratedNode>(); this.space(); if (this.take("}")) return values;
    for (;;) { const key = this.string(); const child = bulletGeneratedPath(path, key); if (!this.take(":")) bulletGeneratedFail(""); if (values.has(key)) bulletGeneratedFail(child); values.set(key, this.value(child, depth)); if (this.take("}")) return values; if (!this.take(",")) bulletGeneratedFail(""); }
  }
  private string(): string {
    if (!this.take("\"")) bulletGeneratedFail(""); let output = "";
    while (this.cursor < this.text.length) {
      const character = this.text[this.cursor++]; if (character === "\"") return output; if (character < " ") bulletGeneratedFail("");
      if (character !== "\\") { output += character; continue; } const escape = this.text[this.cursor++]; const simple = new Map([["\"", "\""], ["\\", "\\"], ["/", "/"], ["b", "\b"], ["f", "\f"], ["n", "\n"], ["r", "\r"], ["t", "\t"]]);
      if (simple.has(escape)) { output += simple.get(escape); continue; } if (escape !== "u") bulletGeneratedFail(""); const first = this.hex();
      if (first >= 0xd800 && first <= 0xdbff) { if (this.text.slice(this.cursor, this.cursor + 2) !== "\\u") bulletGeneratedFail(""); this.cursor += 2; const second = this.hex(); if (second < 0xdc00 || second > 0xdfff) bulletGeneratedFail(""); output += String.fromCodePoint(0x10000 + ((first - 0xd800) << 10) + second - 0xdc00); }
      else { if (first >= 0xdc00 && first <= 0xdfff) bulletGeneratedFail(""); output += String.fromCharCode(first); }
    }
    return bulletGeneratedFail("");
  }
  private integer(path: string): number {
    const start = this.cursor; if (this.text[this.cursor] === "-") this.cursor += 1; const first = this.text[this.cursor];
    if (!(first >= "0" && first <= "9") || (first === "0" && /[0-9]/.test(this.text[this.cursor + 1] ?? ""))) bulletGeneratedFail("");
    this.cursor += 1; while (/[0-9]/.test(this.text[this.cursor] ?? "")) this.cursor += 1; const token = this.text.slice(start, this.cursor);
    if (token === "-0" || /[.eE]/.test(this.text[this.cursor] ?? "")) bulletGeneratedFail(""); const value = Number(token); if (!Number.isSafeInteger(value)) bulletGeneratedFail(""); return value;
  }
  private hex(): number { const token = this.text.slice(this.cursor, this.cursor + 4); if (!/^[0-9A-Fa-f]{4}$/.test(token)) bulletGeneratedFail(""); this.cursor += 4; return Number.parseInt(token, 16); }
  private keyword(word: string): void { if (this.text.slice(this.cursor, this.cursor + word.length) !== word) bulletGeneratedFail(""); this.cursor += word.length; }
  private take(token: string): boolean { this.space(); if (this.text[this.cursor] === token) { this.cursor += 1; return true; } return false; }
  private space(): void { while ([" ", "\t", "\r", "\n"].includes(this.text[this.cursor] ?? "")) this.cursor += 1; }
}

const bulletGeneratedObject = (value: BulletGeneratedNode, path: string): Map<string, BulletGeneratedNode> => value.value instanceof Map ? new Map(value.value) : bulletGeneratedFail(path);
const bulletGeneratedTake = (values: Map<string, BulletGeneratedNode>, name: string, path: string): BulletGeneratedNode => { if (!values.has(name)) bulletGeneratedFail(bulletGeneratedPath(path, name)); const value = values.get(name); values.delete(name); return value === undefined ? bulletGeneratedFail(path) : value; };
const bulletGeneratedClosed = (values: Map<string, BulletGeneratedNode>, path: string): void => { const name = [...values.keys()].sort(bulletGeneratedCompareUtf8)[0]; if (name !== undefined) bulletGeneratedFail(bulletGeneratedPath(path, name)); };
const bulletGeneratedBefore = (values: Map<string, BulletGeneratedNode>, name: string, path: string): void => { const actual = [...values.keys()].sort(bulletGeneratedCompareUtf8)[0]; if (actual !== undefined && bulletGeneratedCompareUtf8(actual, name) < 0) bulletGeneratedFail(bulletGeneratedPath(path, actual)); };
const bulletGeneratedError = (path: string): ContractValidationErrorV1 => Object.freeze({ code: "DOCUMENT_SCHEMA_INVALID", path });
const bulletGeneratedResult = <T>(collect: () => T): ContractDecodeResultV1<T> => { try { return Object.freeze({ ok: true, value: collect() }); } catch (error) { return Object.freeze({ error: bulletGeneratedError(error instanceof BulletGeneratedFailure ? error.path : ""), ok: false }); } };
"###;

pub(super) const TYPESCRIPT_STRING: &str = r###"
const bulletGeneratedString = (value: BulletGeneratedNode, path: string): string => typeof value.value === "string" ? value.value : bulletGeneratedFail(path);
"###;

pub(super) const TYPESCRIPT_PATTERN: &str = r###"
const bulletGeneratedExact = (pattern: RegExp, value: string): boolean => { const match = pattern.exec(value); return match !== null && match.index === 0 && match[0].length === value.length; };
"###;

pub(super) const TYPESCRIPT_ARRAYS: &str = r###"
const bulletGeneratedArrayNode = (value: BulletGeneratedNode, path: string): ReadonlyArray<BulletGeneratedNode> => Array.isArray(value.value) ? value.value : bulletGeneratedFail(path);
const bulletGeneratedCollectNodes = <T>(values: ReadonlyArray<BulletGeneratedNode>, path: string, collect: (value: BulletGeneratedNode, path: string) => T): ReadonlyArray<T> => { const output: T[] = []; [...values.keys()].sort((left, right) => bulletGeneratedCompareUtf8(String(left), String(right))).forEach((index) => { output[index] = collect(values[index], bulletGeneratedPath(path, String(index))); }); return Object.freeze(output); };
"###;

pub(super) const TYPESCRIPT_COLLECT_ARRAY: &str = r###"
const bulletGeneratedCollectArray = <T>(value: BulletGeneratedNode, path: string, collect: (value: BulletGeneratedNode, path: string) => T): ReadonlyArray<T> => bulletGeneratedCollectNodes(bulletGeneratedArrayNode(value, path), path, collect);
"###;

pub(super) const TYPESCRIPT_INTEGER: &str = r###"
const bulletGeneratedInteger = (value: BulletGeneratedNode, path: string): number => typeof value.value === "number" ? value.value : bulletGeneratedFail(path);
"###;

pub(super) const TYPESCRIPT_BOOLEAN: &str = r###"
const bulletGeneratedBoolean = (value: BulletGeneratedNode, path: string): boolean => typeof value.value === "boolean" ? value.value : bulletGeneratedFail(path);
"###;

pub(super) const TYPESCRIPT_BOUNDS: &str = r###"
const bulletGeneratedCardinality = (length: number, minimum: number, maximum: number, path: string): void => { if (length < minimum || length > maximum) bulletGeneratedFail(path); };
"###;

pub(super) const TYPESCRIPT_TEXT: &str = r###"
const bulletGeneratedValidText = (value: string, minimum: number, maximum: number): boolean => {
  const bytes = bulletGeneratedEncodeUtf8(value).length;
  return bytes >= minimum && bytes <= maximum && value.normalize("NFC") === value && [...value].every((character) => { const code = character.codePointAt(0) ?? 0; return !((code <= 0x1f) || (code >= 0x7f && code <= 0x9f) || code === 0x61c || (code >= 0x200e && code <= 0x200f) || (code >= 0x202a && code <= 0x202e) || (code >= 0x2066 && code <= 0x206f) || (code >= 0xfdd0 && code <= 0xfdef) || (code & 0xffff) >= 0xfffe || [0xad, 0x34f, 0x3164, 0xfeff, 0xffa0].includes(code) || (code >= 0x115f && code <= 0x1160) || (code >= 0x17b4 && code <= 0x17b5) || (code >= 0x180b && code <= 0x180f) || (code >= 0x200b && code <= 0x200d) || (code >= 0x2060 && code <= 0x2065) || (code >= 0xfe00 && code <= 0xfe0f) || (code >= 0xfff0 && code <= 0xfff8) || (code >= 0x1bca0 && code <= 0x1bca3) || (code >= 0x1d173 && code <= 0x1d17a) || (code >= 0xe0000 && code <= 0xe0fff)); });
};
"###;

pub(super) const TYPESCRIPT_CANONICAL: &str = r###"
const bulletGeneratedQuote = (value: string): string => { let output = "\""; for (const character of value) { const code = character.codePointAt(0) ?? 0; if (character === "\"") output += "\\\""; else if (character === "\\") output += "\\\\"; else if (code < 0x20) { const short = new Map([[8, "\\b"], [9, "\\t"], [10, "\\n"], [12, "\\f"], [13, "\\r"]]).get(code); output += short ?? `\\u${code.toString(16).padStart(4, "0")}`; } else output += character; } return `${output}\"`; };
const bulletGeneratedCanonicalText = (node: BulletGeneratedNode): string => { const value = node.value; if (value === null) return "null"; if (typeof value === "boolean" || typeof value === "number") return String(value); if (typeof value === "string") return bulletGeneratedQuote(value); if (Array.isArray(value)) return `[${value.map(bulletGeneratedCanonicalText).join(",")}]`; if (!(value instanceof Map)) return bulletGeneratedFail(""); return `{${[...value.keys()].sort().map((key) => `${bulletGeneratedQuote(key)}:${bulletGeneratedCanonicalText(value.get(key) ?? bulletGeneratedFail(""))}`).join(",")}}`; };
const bulletGeneratedCanonical = (value: BulletGeneratedNode): Uint8Array => bulletGeneratedEncodeUtf8(bulletGeneratedCanonicalText(value));
"###;

pub(super) const TYPESCRIPT_SET: &str = r###"
const bulletGeneratedSetFailure = (keys: ReadonlyArray<Uint8Array>, path: string): string | null => { const failures: Array<readonly [string, number]> = keys.slice(1).flatMap((key, offset) => keys.slice(0, offset + 1).some((prior) => bulletGeneratedCompareBytes(key, prior) === 0) ? [[bulletGeneratedPath(path, String(offset + 1)), 0] as const] : bulletGeneratedCompareBytes(key, keys[offset]) < 0 ? [[bulletGeneratedPath(path, String(offset + 1)), 1] as const] : []); failures.sort((left, right) => bulletGeneratedCompareUtf8(left[0], right[0]) || left[1] - right[1]); return failures[0]?.[0] ?? null; };
"###;

pub(super) const TYPESCRIPT_UNIQUE: &str = r###"
const bulletGeneratedDuplicateFailure = (nodes: ReadonlyArray<BulletGeneratedNode>, path: string): string | null => { const keys = nodes.map(bulletGeneratedCanonical); const failures = keys.flatMap((key, index) => keys.slice(0, index).some((prior) => bulletGeneratedCompareBytes(key, prior) === 0) ? [bulletGeneratedPath(path, String(index))] : []); failures.sort(bulletGeneratedCompareUtf8); return failures[0] ?? null; };
"###;

pub(super) const TYPESCRIPT_LEGACY_STRING: &str = r###"
const bulletGeneratedLegacyString = (node: BulletGeneratedNode, path: string, rule: string): string => { const value = bulletGeneratedString(node, path); let valid = false; switch (rule) {
  case "any": valid = true; break; case "nonempty": valid = value.length > 0; break; case "policy": valid = value === "v1alpha1" || value === "v1alpha2"; break; case "identifier": valid = bulletGeneratedExact(/^[a-z][a-z0-9-]{1,15}_[0-9a-f]{64}$/, value); break; case "hex64": valid = bulletGeneratedExact(/^[0-9a-f]{64}$/, value); break;
  case "git-oid": valid = bulletGeneratedExact(/^(sha1:[0-9a-f]{40}|sha256:[0-9a-f]{64})$/, value); break; case "blake3": valid = bulletGeneratedExact(/^blake3:[0-9a-f]{64}$/, value); break; case "release-gate": valid = bulletGeneratedExact(/^release\.[a-z0-9][a-z0-9._-]{0,119}$/, value); break; case "native-subject": valid = bulletGeneratedExact(/^[a-z][a-z0-9-]{0,63}:[a-z][a-z0-9-]{1,31}_[0-9a-f]{64}$/, value); break; case "profile": valid = bulletGeneratedExact(/^[a-z][a-z0-9-]{0,62}[a-z0-9]$/, value); break; case "release-tag": valid = value.length <= 128 && bulletGeneratedExact(/^v[0-9](?:[A-Za-z0-9.-]{0,126}[A-Za-z0-9])?$/, value); break;
  case "signing": valid = value.length <= 256 && bulletGeneratedExact(/^[A-Za-z0-9._@+-]{1,128}\|ed25519\|SHA256:[A-Za-z0-9+/=]{16,96}$/, value); break; case "ssh": valid = value.length <= 384 && bulletGeneratedExact(/^ssh-ed25519 [A-Za-z0-9+/=]{40,256}$/, value); break; case "repo-path": valid = [...value].length >= 1 && [...value].length <= 4096 && bulletGeneratedExact(/^(?!\/)(?!.*\\)(?!.*(?:^|\/)\.{1,2}(?:\/|$))(?!.*(?:^|\/)\.git(?:\/|$)).+$/, value); break; case "key-id": valid = bulletGeneratedExact(/^[A-Za-z0-9][A-Za-z0-9._:/-]{0,127}$/, value); break; case "paseto": valid = [...value].length <= 32768 && /^v4\.public\./.test(value); break; default: if (rule.startsWith("id:")) valid = bulletGeneratedExact(new RegExp(`^${rule.slice(3)}_[0-9a-f]{64}$`), value);
} if (!valid) bulletGeneratedFail(path); return value; };
"###;

pub(super) const TYPESCRIPT_BOUNDED_ARRAY: &str = r###"
declare const BulletGeneratedBoundedArrayBrand: unique symbol;
export type BoundedArrayV1<T, MIN extends number, MAX extends number> = ReadonlyArray<T> & Readonly<{ readonly [BulletGeneratedBoundedArrayBrand]: readonly [MIN, MAX] }>;
const bulletGeneratedArray = <T, MIN extends number, MAX extends number>(values: T[], minimum: MIN, maximum: MAX, path: string): BoundedArrayV1<T, MIN, MAX> => { if (values.length < minimum || values.length > maximum) bulletGeneratedFail(path); return Object.freeze(values) as BoundedArrayV1<T, MIN, MAX>; };
"###;

pub(super) const TYPESCRIPT_BOUNDED_SET: &str = r###"
declare const BulletGeneratedBoundedSetBrand: unique symbol;
export type BoundedSetV1<T, MIN extends number, MAX extends number> = ReadonlyArray<T> & Readonly<{ readonly [BulletGeneratedBoundedSetBrand]: readonly [MIN, MAX] }>;
const bulletGeneratedSet = <T, MIN extends number, MAX extends number>(values: T[], keys: Uint8Array[], minimum: MIN, maximum: MAX, path: string): BoundedSetV1<T, MIN, MAX> => { if (values.length < minimum || values.length > maximum) bulletGeneratedFail(path); const failure = bulletGeneratedSetFailure(keys, path); if (failure !== null) bulletGeneratedFail(failure); return Object.freeze(values) as BoundedSetV1<T, MIN, MAX>; };
"###;
