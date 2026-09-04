# ADR 0017: Catalog type-expression vocabulary

Status: Accepted (DESIGNED; no implementation bytes)
Owner: Bullet Farm maintainers
Related: [0016 (legacy contract semantic closure)](0016-legacy-contract-semantic-closure.md)
Normative annex: [0017 proof and admission annex](0017-catalog-type-expression-proof-annex.md),
SHA-256 `c4a0b77a3841dc92a01c0839d57b5c377e311881afc64777f7f143c5fbe45470`

The decision is incomplete if that exact annex path or digest does not match. The annex names this
decision/version but omits this parent's digest, so the binding is intentionally acyclic.

## Context and boundary

The `v1alpha1.0` contract catalog has a closed top-level Rust representation, but its field-type
enum contains legacy open `object` and `object_array` spellings and hard-coded names for many
nested records. The current schema and binding generators consequently emit 40
`serde_json::Value` leaves and 40 `Record<string, unknown>` leaves in public DTOs. ADR 0016 needs
a closed vocabulary before it can publish its retained legacy contracts, tagged unions, bounded
collections, and typed scalar constraints.

This ADR freezes that vocabulary and its backward-compatibility proof. It deliberately does not
define a second `CatalogTypeExprV1` registry. The single source of type expressions remains the
defaulted `FieldTypeV1` plus catalog-level scalar and union declarations described below.

This document is **DESIGNED only**. It changes no catalog, generated contract, wire, authority,
dogfood, transaction, live, evolution, gate, or release bytes. Acceptance would authorize only
the dormant W11 machinery and its tests. It would not authorize any LC record, edit
`required_records()`, publish W12 bytes, or make any runtime or release claim.

## Decision

### 1. Backward-compatible meta-shape

Preserve every existing serialized `FieldTypeV1` spelling and append exactly four dormant
spellings:

```text
named_ref
optional_named_ref
bounded_array
bounded_set
```

There is no field-level union spelling. A named reference can resolve to one named scalar, record,
or tagged union. Extend the recursively closed catalog meta-records in this exact field order:

```text
ContractFieldV1 = { name, field_type, target?:TypeName, bounds?:CollectionBoundsV1 }
CollectionBoundsV1 = { min_items:u16, max_items:u16 }
ContractRecordV1 = { name, security_class:SecurityClassV1, unknown_fields, shape?:RecordShapeV1, fields }
RecordShapeV1 = versioned | embedded
SecurityClassV1 = attestation | audit | command_authority | effect_authority | holdout |
  integration | policy | projection | release | research | review | verification
ContractCatalogV1 = { schema_version, catalog_version, scalar_types?:ScalarTypeV1[], tagged_unions?:TaggedUnionV1[], records }
ScalarTypeV1 = { name:TypeName, definition:ScalarDefinitionV1 }
ScalarDefinitionV1 =
  {kind:"safe_integer",minimum:i64,maximum:i64} |
  {kind:"text",minimum_utf8_bytes:u32,maximum_utf8_bytes:u32} |
  {kind:"code",minimum_ascii_bytes:u16,maximum_ascii_bytes:u16,class:CodeClassV1} |
  {kind:"enum",values:CatalogLiteralV1[]} | {kind:"typed_id",prefix:string}
CodeClassV1 = lower_kebab | upper_hyphen | ascii_token | invariant_id
TaggedUnionV1 = { name:TypeName, discriminator:FieldName, variants:UnionVariantV1[] }
UnionVariantV1 = { tag:CatalogLiteralV1, record:TypeName }
```

Every added field uses `#[serde(default, skip_serializing_if = ...)]`. `shape` defaults to
`versioned` and is omitted exactly when versioned. Both catalog declaration arrays default empty
and are omitted when empty. `target` and `bounds` default to `None` and are omitted when absent.
Every added struct and enum recursively rejects unknown fields. `SecurityClassV1` is a closed enum
whose exact spellings above preserve all 12 observed legacy values; an unknown class fails strict
decode with `DOCUMENT_SCHEMA_INVALID`. `unknown_fields` remains exactly `"reject"`.

Meta-container bounds are validated before symbol resolution: `scalar_types 0..=256`,
`tagged_unions 0..=128`, `records 1..=512`, and each record's `fields 1..=256`. Existing enum and
union limits remain separate. The canonical document byte cap is an independent earlier bound.
Production W11 admits exactly `catalog_version == "v1alpha1.0"`; one exact synthetic version is
private to `cfg(test)` and is exactly `test.strict.v1`. W12 must explicitly add a reviewed
successor. Every other version returns root-level `INVALID_CONTRACT_CATALOG`.

Those defaults are the compatibility mechanism. Decoding and canonical re-encoding the exact
`v1alpha1.0` catalog must reproduce the same bytes. When the two declaration arrays are empty and
no field uses a new spelling, schema and binding generation must take the unchanged legacy
renderer branches and unchanged `RUST_TEMPLATE` branch.

### 2. Names, references, and collection bounds

`TypeName` reuses the existing 1..80-byte ASCII Pascal-alphanumeric record-name grammar: the
first byte is uppercase ASCII and every byte is ASCII alphanumeric. Records, scalar types, and
tagged unions share one namespace. Invalid names and same-kind or cross-kind collisions refuse.

`FieldName` reuses the current reserved-word-aware lower-snake grammar: 1..80 ASCII bytes, an
initial lowercase letter, then lowercase letters, digits, or underscore. `CatalogLiteralV1` is
1..64 ASCII bytes matching
`^[A-Za-z][A-Za-z0-9]*(?:[-_][A-Za-z0-9]+)*$`. Literal arrays are unique and strictly sorted by
raw ASCII wire bytes; neither validation nor generation case-folds a wire value.

Validation owns one generator-reserved namespace. It contains Rust names `Box`, `Option`,
`Result`, `Self`, `String`, and `Vec`; TypeScript/global names `Array`, `ArrayBuffer`, `BigInt`,
`Boolean`, `Date`, `Error`, `Function`, `Map`, `Number`, `Object`, `Promise`, `Readonly`,
`ReadonlyArray`, `Record`, `RegExp`, `Set`, `String`, `Symbol`, `Uint8Array`, `WeakMap`, and
`WeakSet`; and fixed template exports `AuthorityAudienceV1`, `MutationOperationV1`,
`AuthorityDecisionV1`, `ReplayDispositionV1`, `MutationResultStateV1`, `MutationOutcomeV1`,
`SettlementStatusV1`, `PatchPreimageKindV1`, `PatchMutationKindV1`, `ReleaseReceiptKindV1`,
`ReleaseEvidenceKindV1`, `ReleaseRegistryObjectKindV1`, `ReleaseSignerRoleV1`,
`ReleaseRepositoryNameV1`, `KeyPurposeV1`, `KeyAlgorithmV1`, `PinnedContract`, `ContractPinError`,
`ContractValidationErrorV1`, `ContractDecodeResultV1`, `SchemaVersionLiteralV1`, `BoundedArrayV1`,
and `BoundedSetV1`. Prefix
`BulletGenerated` is reserved for private helpers. A template-symbol
inventory test forces later additions through explicit review.

Field names refuse the complete Rust-2024 strict/reserved/weak table: `abstract, as, async, await,
become, box, break, const, continue, crate, do, dyn, else, enum, extern, false, final, fn, for,
gen, if, impl, in, let, loop, macro, macro_rules, match, mod, move, mut, override, priv, pub, raw,
ref, return, safe, self, static, struct, super, trait, true, try, type, typeof, union, unsafe,
unsized, use, virtual, where, while, yield`. They also refuse the TypeScript binding-keyword table:
`any, as, async, await, boolean, break, case, catch, class, const, constructor, continue, debugger,
declare, default, delete, do, else, enum, export, extends, false, finally, for, from, function, get,
if, implements, import, in, instanceof, interface, let, module, namespace, never, new, null, number,
object, of, package, private, protected, public, readonly, require, return, set, static, string,
super, switch, symbol, this, throw, true, try, type, typeof, undefined, unique, unknown, var, void,
while, with, yield`. Type/helper/variant collisions refuse rather than escape.

Field metadata is exact:

- A legacy `FieldTypeV1` requires both `target` and `bounds` to be absent.
- `named_ref` and `optional_named_ref` require `target` and forbid `bounds`.
- `bounded_array` and `bounded_set` require both `target` and `bounds`.
- A target resolves to exactly one scalar type, record, or tagged union.
- Collection `max_items` is mandatory and lies in `1..=4096`.
- Collection `min_items` lies in `0..=max_items`.

`optional_named_ref` is a required property whose value is exact-type-or-null: Rust `Option<T>`
and TypeScript `T | null`, never an omitted property. `bounded_array` preserves order and permits
duplicates. `bounded_set` is an array unique and strictly increasing by full RFC 8785 canonical
element bytes. Its schema carries `uniqueItems:true` and `x-bullet-order:"rfc8785"`; runtime uses
that same comparison.

A semantic-keyed record set is a `bounded_array` plus exactly one named lifecycle/admission
uniqueness-and-key-order validator from its owning ADR. Generic structural generation does not
implement that comparator. `bounded_set` is legal only when semantic order is whole-element RFC
8785 order; no collection satisfies two comparators.

### 3. Scalar declarations

Safe-integer bounds are inclusive, ordered, and wholly inside
`[-9007199254740991, 9007199254740991]`. Equal bounds render a numeric `const`; therefore a scalar
fixed to one accepts integer `1` and refuses string `"1"`, `0`, `2`, `-1`, and unsafe integers.

Text bounds satisfy `1 <= minimum_utf8_bytes <= maximum_utf8_bytes <= 8388608`. Input must already
equal Unicode NFC; validation never normalizes it. Text refuses every C0/C1 control
(`U+0000..U+001F`, `U+007F..U+009F`), bidi control (`U+061C`, `U+200E..U+200F`,
`U+202A..U+202E`, `U+2066..U+2069`), Unicode noncharacter (`U+FDD0..U+FDEF` and every scalar whose
low 16 bits are `FFFE` or `FFFF`), and the frozen Unicode-15.1 default-ignorable set already
refused by canonical JSON. Bounds count UTF-8 bytes of that unchanged input.

Text schema emits exact `x-bullet-min-utf8-bytes` and `x-bullet-max-utf8-bytes`,
`minLength:1`, and at most `maxLength:maximum_utf8_bytes` as a non-rejecting code-point ceiling; it
never maps a byte minimum greater than one to `minLength`. Rust and TypeScript enforce exact bytes.

Code bounds satisfy `1 <= minimum_ascii_bytes <= maximum_ascii_bytes <= 256`; ASCII schema lengths
therefore equal bytes. Exact languages are
`lower_kebab = ^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$`,
`upper_hyphen = ^[A-Z][A-Z0-9]*(?:-[A-Z0-9]+)*$`,
`ascii_token = ^[A-Za-z0-9][A-Za-z0-9._:/+-]*$`, and
`invariant_id = ^BF-[A-Z0-9]+(?:-[A-Z0-9]+)*$`. Implementations use equivalent ASCII byte
predicates without a new regex dependency.

Enum declarations contain `1..=256` unique `CatalogLiteralV1` values. Rust splits a literal on
`-`/`_`, uppercases each segment's first ASCII letter, lowercases remaining ASCII letters,
preserves digits, concatenates, and emits `#[serde(rename = "<exact-wire>")]`. Two values mapping
to one identifier, or one mapping to `Self`, return `INVALID_CONTRACT_FIELD_REFERENCE`. TypeScript
emits exact literals. Typed-ID prefixes match `^[a-z][a-z0-9-]{1,15}$`; values are exactly
`<prefix>_[0-9a-f]{64}`. W11 supplies this generic mechanism only. W12 is the sole catalog change
that may declare concrete Requirement, Service, evolution, or other accepted scalar names.

### 4. Record shapes and tagged unions

A `versioned` record has exactly one `schema_version` field. Exact legacy
`catalog_version == "v1alpha1.0"` grandfathers its present field type byte-for-byte. Every reviewed
successor requires exactly `FieldTypeV1::SchemaVersion` with no target/bounds; absence remains
`MISSING_SCHEMA_VERSION`. An `embedded` record has no `schema_version` field.

A tagged union contains 2..=32 variants. Tags are unique `CatalogLiteralV1` values strictly sorted
by raw wire bytes. Variant record targets are distinct and each resolves to an `embedded` record.
The discriminator is valid/non-reserved and absent from every variant record. Rust applies the
enum transform and exact Serde rename; a transform collision returns
`INVALID_CONTRACT_TAGGED_UNION`.

Schema emission uses the following exact templates, with JSON numbers/strings substituted for
capitals and no unshown key. Object members are RFC 8785 sorted; arrays keep the shown order:

```text
named_ref = {"$ref":"#/schemas/T"}
optional_named_ref = {"anyOf":[{"$ref":"#/schemas/T"},{"type":"null"}]}
bounded_array = {"items":{"$ref":"#/schemas/T"},"maxItems":MAX,"minItems":MIN,"type":"array"}
bounded_set = {"items":{"$ref":"#/schemas/T"},"maxItems":MAX,"minItems":MIN,"type":"array","uniqueItems":true,"x-bullet-order":"rfc8785"}
safe_integer = {"maximum":MAX,"minimum":MIN,"type":"integer"}; equal bounds = {"const":MIN,"type":"integer"}
text = {"maxLength":MAX,"minLength":1,"type":"string","x-bullet-max-utf8-bytes":MAX,"x-bullet-min-utf8-bytes":MIN}
code = {"maxLength":MAX,"minLength":MIN,"pattern":CLASS_PATTERN,"type":"string"}
enum = {"enum":[VALUES_IN_WIRE_ORDER],"type":"string"}
typed_id = {"pattern":"^PREFIX_[0-9a-f]{64}$","type":"string"}
union = {"oneOf":[BRANCHES_IN_TAG_ORDER]}
branch = {"additionalProperties":false,"properties":{DISCRIMINATOR:{"const":TAG,"type":"string"},INLINE_EMBEDDED_PROPERTIES},"required":[DISCRIMINATOR,EMBEDDED_FIELDS_IN_DECLARATION_ORDER],"type":"object"}
record = {"$id":"https://schemas.bullet.farm/v1alpha1/N.json","$schema":"https://json-schema.org/draft/2020-12/schema","additionalProperties":false,"properties":{FIELDS},"required":[FIELDS_IN_DECLARATION_ORDER],"title":N,"type":"object","x-bullet-security-class":CLASS,"x-bullet-unknown-fields":"reject"}
```

Union branches are inline only: no `$ref`, `allOf`, branch `$id`, title, or extension key. Generated
Rust uses closed internally tagged Serde unions; TypeScript uses discriminated unions. Declaration
arrays are strictly sorted/unique by raw TypeName bytes. Every scalar/union, including unused ones,
emits once in that order at `#/schemas/<TypeName>`; references have the exact form above. Legacy
record schema locations and bytes remain unchanged.

### 5. Symbol graph and cycle refusal

One private resolver produces immutable `ResolvedCatalogV1<'a>` with the unified namespace, symbol
kinds, resolved legacy/new targets, and sorted adjacency. A record points to every record/union
reached through any fixed legacy direct, optional, or array FieldType and every new direct,
optional, array, set, or union-valued reference. A union points to every variant record; scalars
add no edge. Legacy `object`/`object_array` have no target and remain version-gated.

The sole production legacy-target map has exactly 22 target rows and 24 mapped enum variants;
`A|B->T` means both `A` and `B` target `T`:

```text
IssuerKeyArray->IssuerKeyV1; RiskPolicy->RiskPolicyV1;
EvidencePolicy->EvidencePolicyV1; SandboxPolicy->SandboxPolicyV1;
BudgetPolicy->BudgetPolicyV1; RoutePolicy->RoutePolicyV1;
SignedAuthorityEnvelope->SignedAuthorityEnvelopeV1;
SignedMutationPermit|OptionalSignedMutationPermit->SignedMutationPermitV1;
MutationReplayResult|OptionalMutationReplayResult->MutationReplayResultV1;
ScopeGrant->ScopeGrantV1; PatchProposal->PatchProposalV1;
PatchOperationArray->PatchOperationV1; CleanupAuthorization->CleanupAuthorizationV1;
ReleaseFamilySubject->ReleaseFamilySubjectV1;
ReleaseRepositorySubjectArray->ReleaseRepositorySubjectV1;
ReleaseEvidenceSubjectArray->ReleaseEvidenceSubjectV1;
ReleaseProfileNodeArray->ReleaseProfileNodeV1;
ReleaseSignerKeyArray->ReleaseSignerKeyV1;
ReleaseRegistryEntryArray->ReleaseRegistryEntryV1;
ReleaseRegistryObjectArray->ReleaseRegistryObjectV1;
ReleaseReplayBindingArray->ReleaseReplayBindingV1;
ExecutionToolArray->ExecutionToolV1
```

Every other legacy variant, including `Object` and `ObjectArray`, has no target. Hostile tests carry
an independent literal copy, assert all 24 presences and every remaining absence, and compute expected
raw-name-sorted edges from that copy—never from the production resolver or an enum-derived oracle.

A tri-color DFS visits nodes and adjacency lexically and refuses every direct or indirect cycle,
including self, legacy-to-new-to-legacy, and new-to-legacy-to-new, with
`CONTRACT_TYPE_CYCLE`. Schema and bindings accept only that resolved view and maintain no second
symbol, target, legacy-target, union, or edge table. Resolved fields carry nonoptional targets and
bounds; both renderers return `Result` and defensively map an impossible private raw-kind mismatch
to `INVALID_CONTRACT_FIELD_REFERENCE`, without panic or unwrap. `contract_tool` resolves once,
passes the same view to both, and its D extension propagates `?`. Public `validate()` wraps
production resolution; the public schema wrapper is fallible and also production-resolves.

Production resolution always enforces exact `required_records()` coverage. Only one
`#[cfg(test)] pub(crate)` helper for an exact synthetic-test version may skip coverage; it skips
nothing else and refuses legacy open types. There is no public, environment, CLI, fixture, tool,
or release bypass. A hostile proves one fixture passes the private helper while public validation
returns `CONTRACT_CATALOG_COVERAGE`.

### 6. Stable validation precedence

Validation collects failures before selecting this exact class precedence:

1. `INVALID_CONTRACT_CATALOG` for root/schema/catalog version or top-level cardinality.
2. `INVALID_CONTRACT_RECORD` for record metadata/name or field-array cardinality.
3. `DUPLICATE_CONTRACT_RECORD`.
4. `INVALID_CONTRACT_FIELD` for field name, duplicate, or reserved word.
5. `MISSING_SCHEMA_VERSION` for a versioned record with none.
6. `INVALID_CONTRACT_RECORD_SHAPE` for every other versioned/embedded schema-field violation.
7. `INVALID_CONTRACT_FIELD_REFERENCE` for declaration name/order, cross-kind or generator
   namespace collision, typed-ID prefix, enum literal/transform collision, or missing, extra,
   mismatched, or unknown target metadata.
8. `INVALID_CONTRACT_FIELD_BOUNDS` for scalar/collection numeric or byte bounds.
9. `INVALID_CONTRACT_TAGGED_UNION` for union size, discriminator, tag/order/duplicate/transform,
   distinctness, or non-embedded target.
10. `CONTRACT_TYPE_CYCLE`.
11. `OPEN_CONTRACT_FIELD` for an open legacy type in an explicitly admitted successor version.
12. `CONTRACT_CATALOG_COVERAGE`.

Within one class select the lexically least `(owner_type, field_or_index, detail)`, independent of
input traversal; declaration/variant arrays and graph adjacency use their frozen sorted orders.
Exact `v1alpha1.0` alone grandfathers legacy open types in W11. Unknown versions fail at step 1;
an explicitly admitted W12 successor refuses open types at step 11. Existing codes retain their
meanings, and no failure is PASS.

### 7. Generator boundary and zero-open admission

Legacy field rendering remains byte-identical. New numeric/text/code/ID scalars are opaque Rust
newtypes; scalar enums are closed; unions are closed. Rust declares exactly
`pub struct BoundedArrayV1<T,const MIN:usize,const MAX:usize>` and
`pub struct BoundedSetV1<T,const MIN:usize,const MAX:usize>`. A field uses exactly
`BoundedArrayV1<TARGET, MIN_DECIMAL, MAX_DECIMAL>` or
`BoundedSetV1<TARGET, MIN_DECIMAL, MAX_DECIMAL>`; `const`/`usize` never occur in an application.
Each public generic wrapper owns a private `Vec<T>`, serializes transparently, and exposes only
`try_new(Vec<T>) -> Result<Self, ContractValidationErrorV1>`, identical `TryFrom<Vec<T>>`,
`as_slice(&self) -> &[T]`, and `into_vec(self) -> Vec<T>`; there is no public field,
mutable/unchecked route, or strict-type `Deserialize`. Malformed const bounds/cardinality fail at
root with `DOCUMENT_SCHEMA_INVALID`. Set construction requires `T: serde::Serialize` and rejects
the first non-increasing RFC-8785 canonical element at its decimal index.

TypeScript emits these exact aliases; both `declare const` brands are non-exported:

```typescript
declare const BulletGeneratedBoundedArrayBrand: unique symbol;
declare const BulletGeneratedBoundedSetBrand: unique symbol;
export type BoundedArrayV1<T, MIN extends number, MAX extends number> =
  ReadonlyArray<T> & Readonly<{ readonly [BulletGeneratedBoundedArrayBrand]: readonly [MIN, MAX] }>;
export type BoundedSetV1<T, MIN extends number, MAX extends number> =
  ReadonlyArray<T> & Readonly<{ readonly [BulletGeneratedBoundedSetBrand]: readonly [MIN, MAX] }>;
```

Fields use exactly `BoundedArrayV1<TARGET, MIN_DECIMAL, MAX_DECIMAL>` or the `BoundedSetV1` form.
Only the private post-validation collector may assert that intersection for a fresh, recursively
frozen array; no public constructor, brand, cast, or object validator exists. No per-field wrapper
name transform is permitted. A strict versioned record renders `SchemaVersionLiteralV1`, whose
only Rust value and TypeScript literal is `"v1alpha1"`; legacy records retain their old `String` bytes.

The digest-bound annex freezes the only public duplicate-aware Rust and TypeScript byte/text
admission APIs, exact result shape, recursive ownership/freeze rules, and private post-unique
collector. No public `serde_json::Value`, `unknown`, object, cast, `JSON.parse`, or direct strict
`Deserialize` route is admission. `ContractValidationErrorV1` has public Rust fields
`code: &'static str` and `path: String`; every DTO rejection uses `DOCUMENT_SCHEMA_INVALID`.

A missing required nullable field fails at its escaped `/<field>` path; present `null` becomes
`None`/`null`. Child paths prepend escaped `/<field>` or decimal `/<index>`; unknown-field paths
name that field; root is `""`. Bounded failure collection selects the lexically least RFC-6901
path, then class order: closure/type, literal/text/ID, numeric/byte bound, cardinality, uniqueness,
order. The annex's byte parser runs before this collector, so duplicate members cannot collapse.

Strict helper fragments are emitted only when referenced. The base `RUST_TEMPLATE` remains byte
identical; separately named dormant template fragments contain only generic algorithms and stable
errors, never concrete catalog symbols, enum literals, unions, or record fields.

"Zero open type" applies to every emitted public DTO field and union branch. It does not prohibit
private `serde_json::Value` inside the Farm JSON-Schema generator or the annex's private branded
TypeScript parser node. Tests inspect exported DTO declarations and branches structurally instead
of banning generator-internal JSON state. W11 preserves the 40 existing Rust and 40 existing
TypeScript open leaves; W12 must remove all of them in one publication.

## Dormant implementation boundary

The source baseline reviewed for this decision is:

| Path | SHA-256 | Physical LOC |
| --- | --- | ---: |
| `crates/bullet-wire/src/catalog.rs` | `887c0c83c1e204ad97f4972d353bce6eaf931b401cacb363b636f5aa2528c7cd` | 233 |
| `crates/bullet-wire/src/catalog/schema.rs` | `9b04c56c8e41647dbc7a8978ddb296807c3791ab7348a38f5fc11aa4e2262502` | 309 |
| `crates/bullet-wire/src/contract_bindings.rs` | `ff40cf28d562d065bc1ed5a6f94bda29e0f715148426d7b8bbe675028d62972b` | 335 |
| `crates/bullet-wire/src/contract_bindings/template.rs` | `b1ec160ff9d5a43c3bb756b1b124c1a2b0dc585a85ec0bf8a2567b0ae5d87c9a` | 282 |
| `crates/bullet-wire/tests/canonical_hostile.rs` | `a4d6eccad4f528810867ecb66c96874594ea516f8991f37db1e879effc74da42` | 2,843 |

The digest-bound annex normatively freezes the pre-W11 structural split, its exceptional claim and
identity guards, all cap-safe W11 atomic path packets, cleanup order, and
proof custody. It supersedes no semantic rule here. An owner must verify the annex digest before
claiming a packet; no lane may borrow a path or weaken an intermediate compiling sentinel.

Every lane re-pins all four legacy sentinels. W11 adds no LC record and does not edit
`required_records()`, `contract-catalog.json`, generated contracts, policy, release, or constraints.
The private fixture/harness exists only under `cfg(test)`; production stays hard-wired to coverage.
The annex is the sole normative W11 OCI custody, signed runtime-profile, container state-machine,
Rust/TypeScript proof-subject, argv/environment, stable-error, and hostile-matrix contract. Its
source-pinned trust values must be real before W11-P0 lands; W11 cannot mint or self-admit them.

After this ADR is accepted, later bounded packets may place semantic constraints only at:

- `crates/bullet-wire/src/catalog/constraints/legacy_core.rs` for LC-A1;
- `crates/bullet-wire/src/catalog/constraints/legacy_evaluation.rs` for LC-A2;
- `crates/bullet-wire/src/catalog/constraints/legacy_archive_experiment.rs` for LC-B;
- `crates/bullet-wire/src/catalog/constraints/legacy_routing_corpus_team.rs` for LC-C;
- `crates/bullet-wire/src/catalog/constraints/dogfood.rs` for dogfood constraints;
- `crates/bullet-wire/src/catalog/constraints.rs` as their bounded dispatcher.

Those files contain constraints, not shadow DTO definitions. They resolve only catalog-declared
symbols and remain outside published `required_records()` until W12. W12 alone may add accepted
scalar, embedded, union, and versioned records to `contract-catalog.json` and
`required_records()`, retire accepted legacy names, and regenerate public bytes.

## Compatibility sentinels

W11 must preserve all four exact generated-byte identities:

| Subject | SHA-256 |
| --- | --- |
| Canonical catalog | `0b8319f4527673c5879b5afcf6d9ba15f5b824ec2488c6ca18b0f50b9fc2ac14` |
| JSON Schema bundle | `5b47756bcab8bc88aa24c42a5bcf535e6cbcf95241151b5ebfc50055e7d0b167` |
| Generated Rust | `53d84a74f1ef9482811718c7e3df1744daea0f7c098b1740de7d3e4760e531a9` |
| Generated TypeScript | `ff1e5266fa0b74069bf53cd0c8f7722c653b38b98e7679ccb351d2c342ff2ef0` |

Any sentinel change, undeclared touched path, Rust file at or above 500 physical lines after
formatting, or newly emitted open public DTO field is a hard stop.

## Required W11 test and proof packet

The annex's complete hostile matrix and proof packet are mandatory and incorporated here by its
digest. They include the synthetic strict catalog, duplicate-aware cross-language admission,
compatibility sentinels, split/inventory/LOC guards, purpose-fixed runtime-profile hostiles,
standalone offline Rust lock proof, twice-clean OCI TypeScript proof, and full zero-drift suites.
Missing tools/cases, zero or skipped tests, warnings, unknown outcomes, or any digest/path mismatch
is a hard failure; generic signed JSON or caller-selected text cannot satisfy the packet.

## Consequences and evidence ceiling

This decision makes the generic catalog vocabulary deterministic, recursively closed, generator
neutral, and backward compatible. It chooses explicit named declarations over catch-all objects
or a duplicate expression registry. The cost is a strict no-cycle rule and generated runtime
validation beyond what JSON Schema can express for UTF-8 byte counts and canonical set ordering.

The maximum honest W11 result is **COMPONENT_ONLY** dormant catalog/schema/generator machinery.
Even after implementation, W11 proves no accepted ADR, published wire bundle, family
sync/signature, authority, custody, persistence, provider execution, operational dogfood,
Candidate, Evidence, effect, integration, transaction, live operation, evolution activation,
gate receipt, or release eligibility. Acceptance leaves this decision **DESIGNED**; it remains so
until W11 separately lands and is reviewed, with no implementation bytes implied here.
