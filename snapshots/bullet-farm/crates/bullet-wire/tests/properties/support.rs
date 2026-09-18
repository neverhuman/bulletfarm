use std::fmt::Debug;

use bullet_wire::{Blake3Digest, GitOid, WireError};
use serde_json::{Map, Value};

/// Documented seed; change it only together with the property inventory.
const SEED: u64 = 0x5EED_B011_E7F4_2026;
const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;
pub(super) const CASES: usize = 512;
pub(super) const MAX_SAFE: u64 = 9_007_199_254_740_991;
pub(super) const DOMAIN_ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789.-_";
const TEXT_ALPHABET: &[char] = &[
    'a', 'b', 'z', 'A', 'Z', '0', '9', ' ', '"', '\\', '/', '\t', '\n', 'é', 'ß', '日', '😀', '<',
    '&',
];

pub(super) struct Rng(u64);

impl Rng {
    pub(super) fn for_case(index: usize) -> Self {
        Self(SEED ^ (index as u64 + 1).wrapping_mul(GOLDEN))
    }

    pub(super) fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(GOLDEN);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub(super) fn below(&mut self, bound: usize) -> usize {
        (self.next_u64() % bound as u64) as usize
    }

    pub(super) fn coin(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }

    pub(super) fn bytes(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| self.next_u64() as u8).collect()
    }

    pub(super) fn digest(&mut self) -> Blake3Digest {
        let mut raw = [0; 32];
        raw.copy_from_slice(&self.bytes(32));
        Blake3Digest::from_bytes(raw)
    }

    pub(super) fn label(&mut self) -> String {
        let len = 1 + self.below(8);
        (0..len)
            .map(|_| DOMAIN_ALPHABET[self.below(36)] as char)
            .collect()
    }

    pub(super) fn domain(&mut self) -> String {
        let len = 1 + self.below(12);
        (0..len)
            .map(|_| DOMAIN_ALPHABET[self.below(DOMAIN_ALPHABET.len())] as char)
            .collect()
    }

    fn text(&mut self) -> String {
        let len = self.below(6);
        (0..len)
            .map(|_| TEXT_ALPHABET[self.below(TEXT_ALPHABET.len())])
            .collect()
    }

    pub(super) fn oid(&mut self) -> GitOid {
        let hex = self.digest().to_hex();
        if self.coin() {
            GitOid::Sha1(hex[..40].to_owned())
        } else {
            GitOid::Sha256(hex)
        }
    }

    pub(super) fn value(&mut self, depth: usize) -> Value {
        match self.below(if depth == 0 { 5 } else { 7 }) {
            0 => Value::Null,
            1 => Value::Bool(self.coin()),
            2 => {
                let magnitude = (self.next_u64() % (MAX_SAFE + 1)) as i64;
                Value::from(if self.coin() { magnitude } else { -magnitude })
            }
            // An odd numerator over a power of two is exact and non-integral.
            3 => {
                let numerator = 2.0 * self.below(1 << 20) as f64 + 1.0;
                let shift = self.below(10) + 1;
                Value::from(numerator / f64::from(1_u32 << shift))
            }
            4 => Value::String(self.text()),
            5 => Value::Array((0..self.below(4)).map(|_| self.value(depth - 1)).collect()),
            _ => self.object(depth),
        }
    }

    pub(super) fn object(&mut self, depth: usize) -> Value {
        let mut members = Map::new();
        for _ in 0..1 + self.below(4) {
            let key = self.text();
            let value = self.value(depth.saturating_sub(1));
            members.insert(key, value);
        }
        Value::Object(members)
    }

    fn shuffle<T>(&mut self, items: &mut [T]) {
        for index in (1..items.len()).rev() {
            items.swap(index, self.below(index + 1));
        }
    }
}

pub(super) fn ctx(index: usize) -> String {
    format!("seed=0x{SEED:016x} case={index}")
}

pub(super) fn code<T: Debug>(result: Result<T, WireError>) -> Result<T, &'static str> {
    result.map_err(|error| error.code())
}

/// Render valid loose JSON with shuffled object order and grammar-boundary
/// whitespace. `dup` counts objects; object `dup.0` repeats one member.
pub(super) fn render(value: &Value, rng: &mut Rng, dup: &mut (usize, usize), out: &mut String) {
    let ws = |rng: &mut Rng, out: &mut String| {
        if rng.coin() {
            out.push_str([" ", "\n", "\t", "\r\n"][rng.below(4)]);
        }
    };
    match value {
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                ws(rng, out);
                render(item, rng, dup, out);
                ws(rng, out);
            }
            out.push(']');
        }
        Value::Object(members) => {
            let this = dup.1;
            dup.1 += 1;
            let mut order: Vec<_> = members.iter().collect();
            rng.shuffle(&mut order);
            if this == dup.0 {
                order.push(order[0]);
            }
            out.push('{');
            for (index, (key, item)) in order.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                ws(rng, out);
                out.push_str(&serde_json::to_string(key).expect("JSON object key"));
                ws(rng, out);
                out.push(':');
                ws(rng, out);
                if this == dup.0 && index + 1 == order.len() && rng.coin() {
                    out.push_str("null");
                } else {
                    render(item, rng, dup, out);
                }
                ws(rng, out);
            }
            out.push('}');
        }
        scalar => out.push_str(&serde_json::to_string(scalar).expect("JSON scalar")),
    }
}

pub(super) fn count_objects(value: &Value) -> usize {
    match value {
        Value::Array(items) => items.iter().map(count_objects).sum(),
        Value::Object(members) => 1 + members.values().map(count_objects).sum::<usize>(),
        _ => 0,
    }
}
