//! Byte oracle for Hub canonical JSON.
//!
//! This crate is not a workspace member. A panic is a defect; a typed
//! refusal is a successful closed run.

use bullet_wire::{WireError, decode_canonical_value};

/// Run `decode_canonical_value` on one seed. Admission is `Ok`; every
/// typed `WireError` is a closed refusal.
pub fn fuzz_canonical(data: &[u8]) -> Result<(), WireError> {
    decode_canonical_value(data).map(|_| ())
}
