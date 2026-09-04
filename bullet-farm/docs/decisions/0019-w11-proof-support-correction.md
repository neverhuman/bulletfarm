# ADR 0019: W11 proof support correction

Status: Accepted (DESIGNED; no implementation bytes)
Owner: Bullet Farm maintainers
Related: [0017 vocabulary](0017-catalog-type-expression-vocabulary.md),
[0017 proof annex](0017-catalog-type-expression-proof-annex.md)

## Context

The first W11 A/B/C/D/I compile and whole-suite hostile run found two contradictions that the
accepted ADR 0017 static packet did not expose.

First, ADR 0017 says only W11-ABC-i may update inventory literals, but its exact path list names
only `ops/ci/lib.sh` and `crates/bullet-wire/tests/canonical_hostile/inventory_tests.rs`. The frozen
qualified-attribute inventory literal is actually in
`crates/bullet-wire/tests/canonical_hostile/metadata.rs`. The new cap-safe A/C sources require exact
`rustfmt::skip` sites; removing all of them in memory and formatting would put at least
`catalog/validation.rs`, `contract_bindings.rs`, and `contract_bindings/strict.rs` over their
accepted physical-line caps. The unchanged detector correctly refuses those unregistered sites.

Second, the generated Rust visitor places Bullet's depth check at the correct container entry, but
`serde_json` 1.0.151 applies its own recursion limit first. With the default feature set it rejects
the 128th nested container before Bullet can enforce the promised boundary. The TypeScript parser
can admit 128 and refuse 129, so the frozen Rust proof manifest would produce cross-language drift.

These are proof-support defects, not permission to weaken a bound, detector, or evidence ceiling.

## Decision

This decision narrowly supersedes ADR 0017's conflicting W11-ABC-i path list and standalone Rust
manifest literal. Every other ADR 0017 rule remains unchanged.

### Exact inventory support path

W11-ABC-i owns exactly these three support paths and still co-lands atomically with A/B/C/D:

1. `ops/ci/lib.sh`;
2. `crates/bullet-wire/tests/canonical_hostile/inventory_tests.rs`;
3. `crates/bullet-wire/tests/canonical_hostile/metadata.rs`.

The correction binds these final A/C subjects; any byte or LOC drift reopens this decision before
the metadata map may change:

| Path | SHA-256 | LOC |
| --- | --- | ---: |
| `crates/bullet-wire/src/catalog.rs` | `528d9d979742c030a5752ff60c71cfa970fc0ac2aeaa4b23446a781444fceb33` | 435 |
| `crates/bullet-wire/src/catalog/validation.rs` | `35cb9dda70ce27555b5c5735b79366f12497eb0ef71d47ceade23d450d24cb68` | 490 |
| `crates/bullet-wire/src/catalog/validation/tests.rs` | `33705393b2d550ae850f690a26f392f127994a90a672c5075da07f2c4933432e` | 477 |
| `crates/bullet-wire/src/contract_bindings.rs` | `6a184e4a017b76fc9f5bd768e2301a31a43cfaad912431efbe3d51401265cd24` | 248 |
| `crates/bullet-wire/src/contract_bindings/strict.rs` | `133aa59c5d82c77003e1084a85b08b55c7eaf1856de6e63ef042970ae1f8f92b` | 393 |
| `crates/bullet-wire/src/contract_bindings/strict_template.rs` | `47dc589c4954dad2767900ce41816add1bda650bfcc2a32e8e3c0e6529efa125` | 469 |
| `crates/bullet-wire/src/contract_bindings/tests.rs` | `0de2a7003fdb18b306432f62103d6d9ec33ba094ac248c0f321dc01251f9dd79` | 341 |

The metadata edit adds exactly these raw-path-sorted rows to the existing map and no others:

| Path | Qualified attribute | Exact count |
| --- | --- | ---: |
| `crates/bullet-wire/src/catalog.rs` | `rustfmt::skip` | 5 |
| `crates/bullet-wire/src/catalog/validation.rs` | `rustfmt::skip` | 22 |
| `crates/bullet-wire/src/contract_bindings.rs` | `rustfmt::skip` | 17 |
| `crates/bullet-wire/src/contract_bindings/strict.rs` | `rustfmt::skip` | 22 |

It may not delete an old row, wildcard a path or attribute, exclude a source, change the scanner,
or tolerate count drift. Inventory tests must still prove exact module sites, exact qualified
paths/counts, exact test identities, disjoint/complete partitions, and count-neutral substitution
refusal. Every support file remains below 500 physical lines.

### Exact 128-container Rust boundary

JSON depth is the number of enclosing array/object containers. A scalar or empty container at depth
128 is admitted when every other rule matches. Entry into a 129th container refuses with
`DOCUMENT_SCHEMA_INVALID` at path `""`. Rust and TypeScript must agree for nested arrays, nested
objects, and alternating containers.

The generated Rust decoder must perform the existing byte-length, UTF-8, BOM, negative-zero, and
root-structure preflight; create `serde_json::Deserializer`; call
`disable_recursion_limit()`; and then rely on Bullet's visitor-owned container-entry cap. No public
unbounded decoder exists. The 33,554,432-byte cap, recursively closed private value graph, exact
integer grammar, duplicate refusal, and no-echo error contract remain mandatory.

The temporary standalone Rust manifest is exactly the following LF-final 468 bytes, SHA-256
`0063e111dd50915d63105044dfc93ec03faa6b8f72a4c44f01666091c0b031a0`:

```toml
[package]
name = "bullet-w11-generated-proof"
version = "0.0.0"
edition = "2024"
rust-version = "1.95"
publish = false

[dependencies]
blake3 = { version = "=1.8.7", default-features = false, features = ["pure", "std"] }
serde = { version = "=1.0.229", features = ["derive"] }
serde_jcs = "=0.2.0"
serde_json = { version = "=1.0.151", features = ["unbounded_depth"] }
unicode-normalization = "=0.1.25"

[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
all = "deny"
```

`unbounded_depth` adds no dependency. W11-D must nevertheless generate the lock twice offline from
this exact manifest and re-prove the annex's exact package count, bytes, lines, sources, and digest;
an assumption that the old lock remains identical is not evidence. Any difference requires another
reviewed correction rather than an implementation-time repin.

## Mandatory correction proof and sequencing

Before the atomic W11-ABC commit:

- source/generation tests pin the one `disable_recursion_limit()` call and forbid another unbounded
  entrypoint;
- a synthetic valid unused scalar and unused tagged union each render exactly once while
  unreferenced helper fragments remain absent; exact legacy-version routing may not omit either;
- the TypeScript parser node has a module-private nominal brand, a private constructor, tokenizer-
  only factory calls, and no exported constructor, cast, structural object validator, or caller path;
- source/generation tests cover all three carried legacy-reference shapes, ExecutionToolArray's
  1..64 cardinality and uniqueness, every legacy descriptor family, absolute TypeScript pattern
  matching, and immutable wrapper construction;
- the canonical hostile suite observes exactly the four additive rows above and otherwise remains
  byte-for-byte strict; and
- library Clippy runs with `-D warnings` after production-only warning cleanup.

The atomic ABC commit may establish only generator/schema COMPONENT machinery; it is not executable
Rust/TypeScript decoder proof. W11-D retains its accepted position after P0 and R and is the mandatory
owner of executable proof. Before W11-D may release, its exact authenticated subjects must:

- admit exactly 128 containers and refuse 129 at root in generated Rust and TypeScript for nested
  arrays, nested objects, and alternating shapes;
- prove unused-declaration routing and refuse a structural parser-node lookalike or any parser/source
  bypass; and
- cover final LF/CR/U+2028/U+2029 anchoring, malformed UTF and surrogate input,
  floats/exponents/unsafe integers, global negative-zero versus duplicate order, escaped duplicate
  paths containing ` at line `, Unicode denylist edges, UTF-8 path ordering, RFC 8785 set ordering,
  immutable success graphs, and every legacy/reference constraint family.

No ambient host compiler, Node installation, ephemeral manual vector, or caller-selected transcript
can satisfy that exit. Host-local observations clear no D custody requirement, and D may not borrow
them. Until D passes, strict decoder execution remains explicitly UNPROVED even if ABC is committed.

## Consequences and evidence ceiling

This correction creates no catalog declaration, generated contract, key, authority, provider run,
Candidate, Evidence, receipt, transaction, or release eligibility. It does not authorize W11-P0 or
manufacture any missing custody value. Its maximum evidence is DESIGNED until independently
accepted; corrected W11 remains at most COMPONENT_ONLY after its full proof succeeds.
