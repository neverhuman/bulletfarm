# Canonical document pipeline

Status: Enforced
Owner: Bullet Farm contracts
Last reviewed: 2026-08-25
Applies to: security-sensitive v1alpha1 documents

The only accepted byte pipeline is:

1. Refuse empty or over-1-MiB input.
2. Decode strict UTF-8 and refuse a BOM.
3. Parse JSON while refusing duplicate object keys.
4. Refuse C0/C1 controls, directional controls, Unicode 15.1 default-ignorable code points,
   Unicode noncharacters, non-NFC strings, and integers outside the interoperable IEEE-754 safe
   range.
5. Deserialize into a `deny_unknown_fields` type and run that record's semantic validator.
6. Encode RFC 8785 JSON and require byte equality with the received bytes.
7. Hash `bullet-wire.v1 || frame(domain) || frame(canonical bytes)` with BLAKE3, where each frame
   is an unsigned 64-bit little-endian length followed by exact bytes.

Semantic validation precedes authorization. Hash identity alone never makes a record admissible.
The default-ignorable set is frozen to all 4,174 code points in the Unicode 15.1
[`DerivedCoreProperties.txt`](https://www.unicode.org/Public/15.1.0/ucd/DerivedCoreProperties.txt),
because that property has no cross-version stability guarantee. Leading U+FEFF has the more specific
`UTF8_BOM_FORBIDDEN` result and directional members have `DIRECTIONAL_CONTROL_FORBIDDEN`; every
other member returns `ZERO_WIDTH_CHARACTER_FORBIDDEN`. This global strict boundary intentionally
refuses variation selectors and Mongolian, Khmer, and Hangul formatting or filler characters. A
future display-string use case must introduce a field-scoped contract instead of silently weakening
canonical identity.

The hostile fixture lane covers escaped and raw controls, bidi, default-ignorables, NUL, invalid
UTF-8, CRLF/LF whitespace, non-NFC text, duplicate keys, overlong input, unsafe integers, and
framed-hash ambiguity. The preserved TEAM bytes are never canonicalized in place.

The generated `bullet_wire::v1alpha1` namespace is the sole normative wire-record namespace. It
contains all 83 catalog records as strict Rust types and is copied byte-for-byte to Kernel and
BulletGit; Portal receives the corresponding generated TypeScript interfaces. Nested records stay
`v1alpha1` even after the policy catalog gained a `v1alpha2` field. Pre-v1 proposal,
checkpoint, preservation, Candidate-proof, and integration-proof shapes remain offline component
primitives and are exported only with explicit `Component` names. They cannot satisfy a v1alpha1
gateway. Replacing their Kernel/BulletGit call sites is a Wave 3 requirement, not Gate-0 evidence.

Proof: `cargo test --locked -p bullet-wire --test canonical_hostile` and `just contract-check`.
