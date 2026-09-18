use std::path::PathBuf;

use sha2::{Digest, Sha256};

use super::*;
use crate::{
    ContractCatalogV1,
    catalog::{
        CodeClassV1, CollectionBoundsV1, ContractFieldV1, ContractRecordV1, RecordShapeV1,
        ScalarDefinitionV1, ScalarTypeV1, SecurityClassV1, TaggedUnionV1, UnionVariantV1,
    },
    contract_tool::{ContractMode, execute},
};

const CATALOG: &[u8] = include_bytes!("../../../../contracts/v1alpha1/contract-catalog.json");
const SCHEMA: &[u8] = include_bytes!("../../../../contracts/v1alpha1/schema-bundle.json");
const GENERATED_RUST: &[u8] =
    include_bytes!("../../../../contracts/generated/rust/schema_bundle.rs");
const GENERATED_TYPESCRIPT: &[u8] =
    include_bytes!("../../../../contracts/generated/typescript/schemaBundle.ts");
const BASE_TEMPLATE: &[u8] = include_bytes!("template.rs");

#[test]
#[rustfmt::skip]
fn legacy_bindings_and_base_template_are_byte_identical() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    execute(&root, ContractMode::Check).expect("legacy generated subjects stay exact");
    assert_eq!(
        [sha256(CATALOG), sha256(SCHEMA), sha256(GENERATED_RUST), sha256(GENERATED_TYPESCRIPT), sha256(BASE_TEMPLATE)],
        [
            "0b8319f4527673c5879b5afcf6d9ba15f5b824ec2488c6ca18b0f50b9fc2ac14",
            "5b47756bcab8bc88aa24c42a5bcf535e6cbcf95241151b5ebfc50055e7d0b167",
            "53d84a74f1ef9482811718c7e3df1744daea0f7c098b1740de7d3e4760e531a9",
            "ff1e5266fa0b74069bf53cd0c8f7722c653b38b98e7679ccb351d2c342ff2ef0",
            "b1ec160ff9d5a43c3bb756b1b124c1a2b0dc585a85ec0bf8a2567b0ae5d87c9a",
        ]
    );
}

#[test]
#[rustfmt::skip]
fn strict_rust_bindings_are_closed_and_duplicate_aware() {
    let (rust, _) = strict_bindings();
    for required in [
        "serde::de::DeserializeSeed", "BulletGeneratedUniqueJsonValueV1", "BULLET_DUPLICATE:{}",
        "bullet_generated_has_negative_zero", "disable_recursion_limit();", "bullet_generated_before", "self.depth >= 128",
        "sort_by_cached_key", "bullet_generated_cardinality", "impl StrictRecordV1",
        "pub fn decode_bytes(bytes: &[u8])", "pub fn decode_str(text: &str)",
        "#[serde(tag = \"kind\")]", "SchemaVersionLiteralV1", "0x2066..=0x206f",
        "bullet_generated_duplicate_failure", "\"release-gate\" =>",
    ] {
        assert!(rust.contains(required), "missing {required}");
    }
    for forbidden in ["serde::Deserialize)]\npub struct StrictRecordV1", "pub values:"] {
        assert!(!rust.contains(forbidden), "forbidden {forbidden}");
    }
    assert_eq!(rust.matches("disable_recursion_limit();").count(), 1, "sole bounded decoder override");
    let production = rust.split_once("\n#[cfg(test)]\nmod tests {").expect("frozen test boundary").0;
    let strict_runtime = rust.split_once("\n#[derive(Clone, Debug, Eq, PartialEq)]\npub struct ContractValidationErrorV1").expect("strict boundary").1;
    for forbidden in ["unwrap()", "expect("] { assert!(!production.contains(forbidden) && !strict_runtime.contains(forbidden), "production {forbidden}"); }
    assert_eq!(rust.matches(".unwrap();").count(), 1, "sole unwrap stays in frozen legacy test");
    assert!(rust.contains("actual.as_str() < name"));
    assert!(rust.contains("bullet_generated_cardinality(nodes.len(), 1, 12, path)?"));
}

#[test]
#[rustfmt::skip]
fn strict_typescript_bindings_are_readonly_branded_and_duplicate_aware() {
    let (_, typescript) = strict_bindings();
    for required in [
        "new Uint8Array(input)", "class BulletGeneratedNode", "private readonly bulletGeneratedNodeBrand", "private constructor", "const node = new BulletGeneratedNode(value); Object.freeze(node); return node;", "BulletGeneratedParser", "bulletGeneratedHasNegativeZero", "bulletGeneratedExact",
        "declare const BulletGeneratedBoundedArrayBrand: unique symbol;",
        "readonly [BulletGeneratedBoundedArrayBrand]: readonly [MIN, MAX]",
        "readonly [BulletGeneratedBoundedSetBrand]: readonly [MIN, MAX]",
        "[\" \", \"\\t\", \"\\r\", \"\\n\"]",
        "bulletGeneratedBefore", "bulletGeneratedCompareUtf8", "bulletGeneratedCompareBytes",
        "bulletGeneratedCollectNodes", "bulletGeneratedCardinality", "depth >= 128", "code <= 0x206f",
        "decodeStrictRecordV1Bytes", "decodeStrictRecordV1Text",
    ] {
        assert!(typescript.contains(required), "missing {required}");
    }
    for forbidden in ["JSON.parse", "type BulletGeneratedNode =", "export class BulletGeneratedNode", "export declare const BulletGenerated", "localeCompare", "Record<string, unknown>"] {
        assert!(!typescript.contains(forbidden), "forbidden {forbidden}");
    }
    assert_eq!(typescript.matches("BulletGeneratedNode.from(").count(), 7, "tokenizer-only node construction");
}

#[test]
#[rustfmt::skip]
fn bounded_wrapper_apis_are_exact_and_invariant_preserving() {
    let (rust, typescript) = strict_bindings();
    for wrapper in ["BoundedArrayV1", "BoundedSetV1"] {
        assert!(rust.contains(&format!("pub struct {wrapper}<T, const MIN: usize, const MAX: usize> {{ values: Vec<T> }}")));
        for method in ["try_new", "TryFrom<Vec<T>>", "as_slice", "into_vec"] {
            assert!(rust.contains(method));
        }
    }
    for forbidden in ["DerefMut", "> From<Vec<T>> for Bounded", "pub values", "fn new_unchecked"] {
        assert!(!rust.contains(forbidden));
    }
    assert!(rust.contains("failure: Option<(String, u8)>") && rust.contains("&candidate < current"));
    assert!(typescript.contains("Array<readonly [string, number]>") && typescript.contains("bulletGeneratedCompareUtf8(left[0], right[0])"));
    for (field_type, rust_token, typescript_token) in [
        (FieldTypeV1::String, "\"nonempty\"", "\"nonempty\""),
        (FieldTypeV1::PolicySchemaVersion, "\"policy\"", "\"policy\""),
        (FieldTypeV1::CandidateId, "\"id:can\"", "\"id:can\""),
        (FieldTypeV1::KeyAlgorithm, "PasetoV4Public", "paseto-v4.public"),
        (FieldTypeV1::OrderedCandidateIdArray, "duplicate_failure", "DuplicateFailure"),
    ] {
        assert!(rust_strict_legacy_collect(field_type).expect("Rust rule").contains(rust_token));
        assert!(typescript_strict_legacy_collect(field_type, "value", "path").expect("TS rule").contains(typescript_token));
    }
    assert_eq!(rust_strict_legacy_collect(FieldTypeV1::Object).expect_err("open refuses").code(), "INVALID_CONTRACT_FIELD_REFERENCE");
}

#[test]
#[rustfmt::skip]
fn strict_binding_generation_is_deterministic_and_target_complete() {
    let first = strict_bindings();
    let second = strict_bindings();
    assert_eq!(first, second);
    for name in [
        "AlphaBranchV1", "BetaBranchV1", "ChoiceV1", "CodeInvariantV1", "CodeScalarV1",
        "CodeTokenV1", "CodeUpperV1", "EnumScalarV1", "ExecutionToolV1", "IntegerScalarV1",
        "IssuerKeyV1", "ReferenceRootV1", "RiskPolicyV1", "SignedMutationPermitV1",
        "StrictRecordV1", "TextScalarV1", "TypedIdScalarV1",
    ] {
        assert!(first.0.contains(name) && first.1.contains(name), "missing {name}");
    }
    for required in ["pub direct_policy: RiskPolicyV1", "pub execution_tools: Vec<ExecutionToolV1>", "pub issuer_keys: Vec<IssuerKeyV1>", "pub optional_permit: Option<SignedMutationPermitV1>"] { assert!(first.0.contains(required), "missing Rust reference {required}"); }
    for required in ["readonly direct_policy: RiskPolicyV1", "readonly execution_tools: ReadonlyArray<ExecutionToolV1>", "readonly issuer_keys: ReadonlyArray<IssuerKeyV1>", "readonly optional_permit: SignedMutationPermitV1 | null"] { assert!(first.1.contains(required), "missing TS reference {required}"); }
    assert!(first.0.contains("bullet_generated_cardinality(nodes.len(), 1, 64, path)?") && first.1.contains("bulletGeneratedCardinality(nodes.length, 1, 64"));
    let (bool_rust, bool_ts) = bindings_for(bool_catalog());
    for required in ["bullet_generated_bool", "bulletGeneratedBoolean"] { assert!(bool_rust.contains(required) || bool_ts.contains(required)); }
    for absent in ["bullet_generated_string", "bullet_generated_array", "bullet_generated_valid_text", "serde_jcs", "BoundedArrayV1", "BoundedSetV1"] { assert!(!bool_rust.contains(absent), "bool-only leaked {absent}"); }
    for absent in ["bulletGeneratedString", "bulletGeneratedArrayNode", "bulletGeneratedValidText", "bulletGeneratedCanonical", "BoundedArrayV1", "BoundedSetV1"] { assert!(!bool_ts.contains(absent), "bool-only leaked {absent}"); }
    let (text_rust, text_ts) = bindings_for(text_catalog());
    assert!(text_rust.contains("bullet_generated_valid_text") && text_ts.contains("bulletGeneratedValidText"));
    for absent in ["bullet_generated_array", "serde_jcs", "BoundedArrayV1", "BoundedSetV1"] { assert!(!text_rust.contains(absent), "text-only leaked {absent}"); }
    for absent in ["bulletGeneratedArrayNode", "bulletGeneratedCanonical", "BoundedArrayV1", "BoundedSetV1"] { assert!(!text_ts.contains(absent), "text-only leaked {absent}"); }
    let (array_rust, array_ts) = bindings_for(collection_catalog(FieldTypeV1::BoundedArray));
    for required in ["pub struct BoundedArrayV1", "bullet_generated_cardinality", "bullet_generated_rebase"] { assert!(array_rust.contains(required), "bounded-array missing {required}"); }
    for absent in ["pub struct BoundedSetV1", "bullet_generated_collect_array", "bullet_generated_set_failure", "serde_jcs"] { assert!(!array_rust.contains(absent), "bounded-array leaked {absent}"); }
    for absent in ["BoundedSetV1", "bulletGeneratedCollectArray", "bulletGeneratedCanonical"] { assert!(!array_ts.contains(absent), "bounded-array leaked {absent}"); }
    let (set_rust, set_ts) = bindings_for(collection_catalog(FieldTypeV1::BoundedSet));
    for required in ["pub struct BoundedSetV1", "bullet_generated_set_failure", "serde_jcs"] { assert!(set_rust.contains(required), "bounded-set missing {required}"); }
    for required in ["BoundedSetV1", "bulletGeneratedSetFailure", "bulletGeneratedCanonical"] { assert!(set_ts.contains(required), "bounded-set missing {required}"); }
    let (unused_rust, unused_ts) = bindings_for(unused_declaration_catalog());
    for (source, declaration) in [(&unused_rust, "pub struct UnusedCodeV1"), (&unused_rust, "pub enum UnusedChoiceV1"), (&unused_ts, "export type UnusedCodeV1"), (&unused_ts, "export type UnusedChoiceV1")] { assert_eq!(source.matches(declaration).count(), 1, "unused declaration {declaration}"); }
    for absent in ["bullet_generated_array", "bullet_generated_valid_text", "serde_jcs", "BoundedArrayV1", "BoundedSetV1"] { assert!(!unused_rust.contains(absent), "unused declarations leaked {absent}"); }
    for absent in ["bulletGeneratedArrayNode", "bulletGeneratedValidText", "bulletGeneratedCanonical", "BoundedArrayV1", "BoundedSetV1"] { assert!(!unused_ts.contains(absent), "unused declarations leaked {absent}"); }
    assert!(include_str!("strict.rs").contains("catalog.catalog_version() != \"v1alpha1.0\"\n        || !catalog.scalar_types().is_empty()\n        || !catalog.tagged_unions().is_empty()"), "exact-legacy union routing must remain structural");
    let mut legacy: ContractCatalogV1 = serde_json::from_slice(CATALOG).expect("legacy catalog");
    assert!(!strict::is_required(&legacy.resolve().expect("legacy resolves")));
    legacy.scalar_types.push(scalar("UnusedScalarV1", ScalarDefinitionV1::Code { minimum_ascii_bytes: 1, maximum_ascii_bytes: 8, class: CodeClassV1::LowerKebab }));
    let resolved = legacy.resolve().expect("unused scalar remains structurally valid");
    assert!(strict::is_required(&resolved), "declaration cannot be omitted by version-only routing");
    let inputs = BindingInputs { schema: Blake3Digest::from_bytes([1; 32]), registry: Blake3Digest::from_bytes([2; 32]), policy: Blake3Digest::from_bytes([3; 32]), golden_json: "{}", golden_hash: Blake3Digest::from_bytes([4; 32]), authority_golden_hash: Blake3Digest::from_bytes([5; 32]), launch_grant_golden_hash: Blake3Digest::from_bytes([6; 32]), resolved: &resolved };
    assert_eq!(rust_constants(&inputs).expect_err("strict routing refuses retained open legacy fields").code(), "INVALID_CONTRACT_FIELD_REFERENCE");
    let generator = include_str!("../contract_bindings.rs");
    let collector = include_str!("strict.rs");
    for forbidden in ["conditional_constraints", "ExecutionToolV1", "SignedMutationPermitV1"] { assert!(!generator.contains(forbidden) && !collector.contains(forbidden), "second semantic table marker {forbidden}"); }
    assert!(collector.contains("target_name(target)") && collector.contains("FieldTypeV1::ExecutionToolArray"));
    assert_eq!((sha256(first.0.as_bytes()), sha256(first.1.as_bytes())), strict_golden_hashes());
}

#[rustfmt::skip]
fn strict_bindings() -> (String, String) {
    bindings_for(strict_catalog())
}

#[rustfmt::skip]
fn bindings_for(catalog: ContractCatalogV1) -> (String, String) {
    let resolved = catalog.resolve_test_strict().expect("strict fixture resolves");
    let inputs = BindingInputs {
        schema: Blake3Digest::from_bytes([1; 32]),
        registry: Blake3Digest::from_bytes([2; 32]),
        policy: Blake3Digest::from_bytes([3; 32]),
        golden_json: "{}",
        golden_hash: Blake3Digest::from_bytes([4; 32]),
        authority_golden_hash: Blake3Digest::from_bytes([5; 32]),
        launch_grant_golden_hash: Blake3Digest::from_bytes([6; 32]),
        resolved: &resolved,
    };
    (rust_constants(&inputs).expect("Rust renders"), typescript_constants(&inputs).expect("TypeScript renders"))
}

#[rustfmt::skip]
fn strict_golden_hashes() -> (String, String) {
    ("203bd0ddea7261212fe8c532195184ee0c5226ce789868bd0d6bba670d7065df".into(), "4942fbd3982322e5976b8cbe09566c53aad2d3647a374ad3b4eebe306b0c5ce6".into())
}

#[rustfmt::skip]
fn strict_catalog() -> ContractCatalogV1 {
    ContractCatalogV1 {
        schema_version: "v1alpha1".into(),
        catalog_version: "test.strict.v1".into(),
        scalar_types: vec![
            scalar("CodeInvariantV1", ScalarDefinitionV1::Code { minimum_ascii_bytes: 3, maximum_ascii_bytes: 24, class: CodeClassV1::InvariantId }),
            scalar("CodeScalarV1", ScalarDefinitionV1::Code { minimum_ascii_bytes: 1, maximum_ascii_bytes: 16, class: CodeClassV1::LowerKebab }),
            scalar("CodeTokenV1", ScalarDefinitionV1::Code { minimum_ascii_bytes: 1, maximum_ascii_bytes: 24, class: CodeClassV1::AsciiToken }),
            scalar("CodeUpperV1", ScalarDefinitionV1::Code { minimum_ascii_bytes: 1, maximum_ascii_bytes: 24, class: CodeClassV1::UpperHyphen }),
            scalar("EnumScalarV1", ScalarDefinitionV1::Enum { values: vec!["alpha".into(), "beta-value".into()] }),
            scalar("IntegerScalarV1", ScalarDefinitionV1::SafeInteger { minimum: -7, maximum: 7 }),
            scalar("TextScalarV1", ScalarDefinitionV1::Text { minimum_utf8_bytes: 1, maximum_utf8_bytes: 32 }),
            scalar("TypedIdScalarV1", ScalarDefinitionV1::TypedId { prefix: "thing".into() }),
        ],
        tagged_unions: vec![TaggedUnionV1 {
            name: "ChoiceV1".into(), discriminator: "kind".into(),
            variants: vec![
                UnionVariantV1 { tag: "alpha".into(), record: "AlphaBranchV1".into() },
                UnionVariantV1 { tag: "beta".into(), record: "BetaBranchV1".into() },
            ],
        }],
        records: vec![
            embedded("AlphaBranchV1", "alpha_value"),
            embedded("BetaBranchV1", "beta_value"),
            embedded("ExecutionToolV1", "tool_name"),
            embedded("IssuerKeyV1", "issuer_name"),
            ContractRecordV1 {
                name: "ReferenceRootV1".into(), security_class: SecurityClassV1::Verification,
                unknown_fields: "reject".into(), shape: RecordShapeV1::Embedded,
                fields: vec![
                    field("direct_policy", FieldTypeV1::RiskPolicy, None, None),
                    field("execution_tools", FieldTypeV1::ExecutionToolArray, None, None),
                    field("issuer_keys", FieldTypeV1::IssuerKeyArray, None, None),
                    field("optional_permit", FieldTypeV1::OptionalSignedMutationPermit, None, None),
                ],
            },
            embedded("RiskPolicyV1", "risk_name"),
            embedded("SignedMutationPermitV1", "permit_name"),
            ContractRecordV1 {
                name: "StrictRecordV1".into(), security_class: SecurityClassV1::Verification,
                unknown_fields: "reject".into(), shape: RecordShapeV1::Versioned,
                fields: vec![
                    field("schema_version", FieldTypeV1::SchemaVersion, None, None),
                    field("choice", FieldTypeV1::NamedRef, Some("ChoiceV1"), None),
                    field("code", FieldTypeV1::NamedRef, Some("CodeScalarV1"), None),
                    field("digest", FieldTypeV1::Digest, None, None),
                    field("legacy_ordered", FieldTypeV1::OrderedCandidateIdArray, None, None),
                    field("maybe_branch", FieldTypeV1::OptionalNamedRef, Some("AlphaBranchV1"), None),
                    field("optional_text", FieldTypeV1::OptionalString, None, None),
                    field("ordered", FieldTypeV1::BoundedArray, Some("TextScalarV1"), Some((1, 12))),
                    field("unique_items", FieldTypeV1::BoundedSet, Some("TypedIdScalarV1"), Some((0, 12))),
                    field("count", FieldTypeV1::NamedRef, Some("IntegerScalarV1"), None),
                    field("mode", FieldTypeV1::NamedRef, Some("EnumScalarV1"), None),
                    field("policy_schema_version", FieldTypeV1::PolicySchemaVersion, None, None),
                    field("repo_path", FieldTypeV1::RepoPath, None, None),
                ],
            },
        ],
    }
}

#[rustfmt::skip]
fn bool_catalog() -> ContractCatalogV1 {
    ContractCatalogV1 {
        schema_version: "v1alpha1".into(), catalog_version: "test.strict.v1".into(),
        scalar_types: vec![], tagged_unions: vec![],
        records: vec![ContractRecordV1 {
            name: "BoolRecordV1".into(), security_class: SecurityClassV1::Verification,
            unknown_fields: "reject".into(), shape: RecordShapeV1::Embedded,
            fields: vec![field("enabled", FieldTypeV1::Boolean, None, None)],
        }],
    }
}

#[rustfmt::skip]
fn text_catalog() -> ContractCatalogV1 {
    ContractCatalogV1 {
        schema_version: "v1alpha1".into(), catalog_version: "test.strict.v1".into(),
        scalar_types: vec![scalar("TextOnlyV1", ScalarDefinitionV1::Text { minimum_utf8_bytes: 1, maximum_utf8_bytes: 32 })], tagged_unions: vec![],
        records: vec![ContractRecordV1 {
            name: "TextRecordV1".into(), security_class: SecurityClassV1::Verification,
            unknown_fields: "reject".into(), shape: RecordShapeV1::Embedded,
            fields: vec![field("text_value", FieldTypeV1::NamedRef, Some("TextOnlyV1"), None)],
        }],
    }
}

#[rustfmt::skip]
fn collection_catalog(field_type: FieldTypeV1) -> ContractCatalogV1 {
    ContractCatalogV1 {
        schema_version: "v1alpha1".into(), catalog_version: "test.strict.v1".into(), scalar_types: vec![], tagged_unions: vec![],
        records: vec![
            ContractRecordV1 {
                name: "CollectionRootV1".into(), security_class: SecurityClassV1::Verification,
                unknown_fields: "reject".into(), shape: RecordShapeV1::Embedded,
                fields: vec![field("items", field_type, Some("ItemV1"), Some((1, 4)))],
            },
            ContractRecordV1 {
                name: "ItemV1".into(), security_class: SecurityClassV1::Verification,
                unknown_fields: "reject".into(), shape: RecordShapeV1::Embedded,
                fields: vec![field("enabled", FieldTypeV1::Boolean, None, None)],
            },
        ],
    }
}

#[rustfmt::skip]
fn unused_declaration_catalog() -> ContractCatalogV1 {
    ContractCatalogV1 {
        schema_version: "v1alpha1".into(), catalog_version: "test.strict.v1".into(),
        scalar_types: vec![scalar("UnusedCodeV1", ScalarDefinitionV1::Code { minimum_ascii_bytes: 1, maximum_ascii_bytes: 8, class: CodeClassV1::LowerKebab })],
        tagged_unions: vec![TaggedUnionV1 {
            name: "UnusedChoiceV1".into(), discriminator: "kind".into(),
            variants: vec![UnionVariantV1 { tag: "alpha".into(), record: "BranchAlphaV1".into() }, UnionVariantV1 { tag: "beta".into(), record: "BranchBetaV1".into() }],
        }],
        records: vec![
            ContractRecordV1 { name: "BranchAlphaV1".into(), security_class: SecurityClassV1::Verification, unknown_fields: "reject".into(), shape: RecordShapeV1::Embedded, fields: vec![field("enabled", FieldTypeV1::Boolean, None, None)] },
            ContractRecordV1 { name: "BranchBetaV1".into(), security_class: SecurityClassV1::Verification, unknown_fields: "reject".into(), shape: RecordShapeV1::Embedded, fields: vec![field("enabled", FieldTypeV1::Boolean, None, None)] },
            ContractRecordV1 { name: "RootV1".into(), security_class: SecurityClassV1::Verification, unknown_fields: "reject".into(), shape: RecordShapeV1::Embedded, fields: vec![field("enabled", FieldTypeV1::Boolean, None, None)] },
        ],
    }
}

#[rustfmt::skip]
fn embedded(name: &str, field_name: &str) -> ContractRecordV1 {
    ContractRecordV1 {
        name: name.into(), security_class: SecurityClassV1::Verification,
        unknown_fields: "reject".into(), shape: RecordShapeV1::Embedded,
        fields: vec![field(field_name, FieldTypeV1::String, None, None)],
    }
}

#[rustfmt::skip]
fn scalar(name: &str, definition: ScalarDefinitionV1) -> ScalarTypeV1 {
    ScalarTypeV1 { name: name.into(), definition }
}

#[rustfmt::skip]
fn field(name: &str, field_type: FieldTypeV1, target: Option<&str>, bounds: Option<(u16, u16)>) -> ContractFieldV1 {
    ContractFieldV1 {
        name: name.into(), field_type, target: target.map(str::to_owned),
        bounds: bounds.map(|(min_items, max_items)| CollectionBoundsV1 { min_items, max_items }),
    }
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
