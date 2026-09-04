use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{canonical_json, decode_canonical};

use super::super::{
    CodeClassV1, CollectionBoundsV1, ContractCatalogV1, ContractFieldV1, ContractRecordV1,
    FieldTypeV1, RecordShapeV1, ScalarDefinitionV1, ScalarTypeV1, SecurityClassV1, TaggedUnionV1,
    UnionVariantV1,
};

const CATALOG: &[u8] = include_bytes!("../../../../../contracts/v1alpha1/contract-catalog.json");
const SCHEMA_BUNDLE: &[u8] = include_bytes!("../../../../../contracts/v1alpha1/schema-bundle.json");
const GENERATED_RUST: &[u8] =
    include_bytes!("../../../../../contracts/generated/rust/schema_bundle.rs");
const GENERATED_TYPESCRIPT: &[u8] =
    include_bytes!("../../../../../contracts/generated/typescript/schemaBundle.ts");

#[test]
fn legacy_schema_bundle_and_literal_fields_are_byte_identical() {
    let catalog = decode_canonical::<ContractCatalogV1>(CATALOG).unwrap();
    let bundle = catalog.json_schema_bundle().unwrap();
    assert_eq!(canonical_json(&bundle).unwrap(), SCHEMA_BUNDLE);
    assert_eq!(
        [
            sha256(CATALOG),
            sha256(SCHEMA_BUNDLE),
            sha256(GENERATED_RUST),
            sha256(GENERATED_TYPESCRIPT),
        ],
        [
            "0b8319f4527673c5879b5afcf6d9ba15f5b824ec2488c6ca18b0f50b9fc2ac14",
            "5b47756bcab8bc88aa24c42a5bcf535e6cbcf95241151b5ebfc50055e7d0b167",
            "53d84a74f1ef9482811718c7e3df1744daea0f7c098b1740de7d3e4760e531a9",
            "ff1e5266fa0b74069bf53cd0c8f7722c653b38b98e7679ccb351d2c342ff2ef0",
        ]
    );
}

#[test]
fn strict_scalar_and_reference_templates_are_exact() {
    let bundle = strict_bundle();
    let schemas = bundle["schemas"].as_object().unwrap();
    let cases = [
        (
            "AsciiTokenV1",
            json!({"maxLength":8,"minLength":1,"pattern":"^[A-Za-z0-9][A-Za-z0-9._:/+-]*$","type":"string"}),
        ),
        (
            "EnumScalarV1",
            json!({"enum":["alpha","beta-value"],"type":"string"}),
        ),
        ("FixedIntegerV1", json!({"const":1,"type":"integer"})),
        (
            "IntegerScalarV1",
            json!({"maximum":7,"minimum":-7,"type":"integer"}),
        ),
        (
            "InvariantCodeV1",
            json!({"maxLength":16,"minLength":4,"pattern":"^BF-[A-Z0-9]+(?:-[A-Z0-9]+)*$","type":"string"}),
        ),
        (
            "LowerCodeV1",
            json!({"maxLength":12,"minLength":1,"pattern":"^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$","type":"string"}),
        ),
        (
            "TextScalarV1",
            json!({"maxLength":12,"minLength":1,"type":"string","x-bullet-max-utf8-bytes":12,"x-bullet-min-utf8-bytes":2}),
        ),
        (
            "TypedIdScalarV1",
            json!({"pattern":"^thing_[0-9a-f]{64}$","type":"string"}),
        ),
        (
            "UpperCodeV1",
            json!({"maxLength":12,"minLength":1,"pattern":"^[A-Z][A-Z0-9]*(?:-[A-Z0-9]+)*$","type":"string"}),
        ),
    ];
    for (name, expected) in cases {
        assert_eq!(schemas[name], expected, "{name}");
    }

    let properties = schemas["StrictRecordV1"]["properties"].as_object().unwrap();
    assert_eq!(properties["choice"], json!({"$ref":"#/schemas/ChoiceV1"}));
    assert_eq!(
        properties["branch"],
        json!({"$ref":"#/schemas/AlphaBranchV1"})
    );
    assert_eq!(
        properties["scalar_value"],
        json!({"$ref":"#/schemas/TextScalarV1"})
    );
    assert_eq!(
        properties["maybe_choice"],
        json!({"anyOf":[{"$ref":"#/schemas/ChoiceV1"},{"type":"null"}]})
    );
    assert_eq!(
        properties["ordered_items"],
        json!({"items":{"$ref":"#/schemas/TextScalarV1"},"maxItems":4,"minItems":1,"type":"array"})
    );
    assert_eq!(
        properties["unique_items"],
        json!({"items":{"$ref":"#/schemas/TypedIdScalarV1"},"maxItems":5,"minItems":0,"type":"array","uniqueItems":true,"x-bullet-order":"rfc8785"})
    );
}

#[test]
fn tagged_union_branches_are_inline_closed_and_ordered() {
    let bundle = strict_bundle();
    assert_eq!(
        bundle["schemas"]["ChoiceV1"],
        json!({"oneOf":[
            {"additionalProperties":false,"properties":{"alpha_value":{"minLength":1,"type":"string"},"kind":{"const":"alpha","type":"string"}},"required":["kind","alpha_value"],"type":"object"},
            {"additionalProperties":false,"properties":{"beta_value":{"minLength":1,"type":"string"},"kind":{"const":"beta","type":"string"}},"required":["kind","beta_value"],"type":"object"}
        ]})
    );
    for branch in bundle["schemas"]["ChoiceV1"]["oneOf"].as_array().unwrap() {
        for forbidden in ["$id", "$ref", "$schema", "allOf", "title"] {
            assert!(branch.get(forbidden).is_none(), "{forbidden}");
        }
        assert!(
            branch
                .as_object()
                .unwrap()
                .keys()
                .all(|key| !key.starts_with("x-bullet-"))
        );
    }
}

#[test]
fn resolved_schema_emits_each_declaration_once() {
    let catalog = strict_catalog();
    let first = super::json_schema_bundle(&catalog.resolve_test_strict().unwrap()).unwrap();
    let second = super::json_schema_bundle(&catalog.resolve_test_strict().unwrap()).unwrap();
    assert_eq!(first, second);
    let names = first["schemas"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        [
            "AlphaBranchV1",
            "AsciiTokenV1",
            "BetaBranchV1",
            "ChoiceV1",
            "EnumScalarV1",
            "FixedIntegerV1",
            "IntegerScalarV1",
            "InvariantCodeV1",
            "LowerCodeV1",
            "StrictRecordV1",
            "TextScalarV1",
            "TypedIdScalarV1",
            "UnusedScalarV1",
            "UpperCodeV1",
        ]
    );
    assert_eq!(
        first["schemas"]["UnusedScalarV1"],
        json!({"maxLength":3,"minLength":1,"type":"string","x-bullet-max-utf8-bytes":3,"x-bullet-min-utf8-bytes":1})
    );
}

fn strict_bundle() -> Value {
    let catalog = strict_catalog();
    super::json_schema_bundle(&catalog.resolve_test_strict().unwrap()).unwrap()
}

fn strict_catalog() -> ContractCatalogV1 {
    ContractCatalogV1 {
        schema_version: "v1alpha1".into(),
        catalog_version: "test.strict.v1".into(),
        scalar_types: scalar_types(),
        tagged_unions: vec![TaggedUnionV1 {
            name: "ChoiceV1".into(),
            discriminator: "kind".into(),
            variants: vec![
                UnionVariantV1 {
                    tag: "alpha".into(),
                    record: "AlphaBranchV1".into(),
                },
                UnionVariantV1 {
                    tag: "beta".into(),
                    record: "BetaBranchV1".into(),
                },
            ],
        }],
        records: vec![
            embedded_record("AlphaBranchV1", "alpha_value"),
            embedded_record("BetaBranchV1", "beta_value"),
            strict_record(),
        ],
    }
}

fn scalar_types() -> Vec<ScalarTypeV1> {
    use CodeClassV1::{AsciiToken, InvariantId, LowerKebab, UpperHyphen};
    vec![
        scalar(
            "AsciiTokenV1",
            ScalarDefinitionV1::Code {
                minimum_ascii_bytes: 1,
                maximum_ascii_bytes: 8,
                class: AsciiToken,
            },
        ),
        scalar(
            "EnumScalarV1",
            ScalarDefinitionV1::Enum {
                values: vec!["alpha".into(), "beta-value".into()],
            },
        ),
        scalar(
            "FixedIntegerV1",
            ScalarDefinitionV1::SafeInteger {
                minimum: 1,
                maximum: 1,
            },
        ),
        scalar(
            "IntegerScalarV1",
            ScalarDefinitionV1::SafeInteger {
                minimum: -7,
                maximum: 7,
            },
        ),
        scalar(
            "InvariantCodeV1",
            ScalarDefinitionV1::Code {
                minimum_ascii_bytes: 4,
                maximum_ascii_bytes: 16,
                class: InvariantId,
            },
        ),
        scalar(
            "LowerCodeV1",
            ScalarDefinitionV1::Code {
                minimum_ascii_bytes: 1,
                maximum_ascii_bytes: 12,
                class: LowerKebab,
            },
        ),
        scalar(
            "TextScalarV1",
            ScalarDefinitionV1::Text {
                minimum_utf8_bytes: 2,
                maximum_utf8_bytes: 12,
            },
        ),
        scalar(
            "TypedIdScalarV1",
            ScalarDefinitionV1::TypedId {
                prefix: "thing".into(),
            },
        ),
        scalar(
            "UnusedScalarV1",
            ScalarDefinitionV1::Text {
                minimum_utf8_bytes: 1,
                maximum_utf8_bytes: 3,
            },
        ),
        scalar(
            "UpperCodeV1",
            ScalarDefinitionV1::Code {
                minimum_ascii_bytes: 1,
                maximum_ascii_bytes: 12,
                class: UpperHyphen,
            },
        ),
    ]
}

fn scalar(name: &str, definition: ScalarDefinitionV1) -> ScalarTypeV1 {
    ScalarTypeV1 {
        name: name.into(),
        definition,
    }
}

fn embedded_record(name: &str, field_name: &str) -> ContractRecordV1 {
    ContractRecordV1 {
        name: name.into(),
        security_class: SecurityClassV1::Policy,
        unknown_fields: "reject".into(),
        shape: RecordShapeV1::Embedded,
        fields: vec![field(field_name, FieldTypeV1::String, None, None)],
    }
}

fn strict_record() -> ContractRecordV1 {
    ContractRecordV1 {
        name: "StrictRecordV1".into(),
        security_class: SecurityClassV1::Policy,
        unknown_fields: "reject".into(),
        shape: RecordShapeV1::Versioned,
        fields: vec![
            field("schema_version", FieldTypeV1::SchemaVersion, None, None),
            field("branch", FieldTypeV1::NamedRef, Some("AlphaBranchV1"), None),
            field("choice", FieldTypeV1::NamedRef, Some("ChoiceV1"), None),
            field(
                "maybe_choice",
                FieldTypeV1::OptionalNamedRef,
                Some("ChoiceV1"),
                None,
            ),
            field(
                "ordered_items",
                FieldTypeV1::BoundedArray,
                Some("TextScalarV1"),
                Some((1, 4)),
            ),
            field(
                "scalar_value",
                FieldTypeV1::NamedRef,
                Some("TextScalarV1"),
                None,
            ),
            field(
                "unique_items",
                FieldTypeV1::BoundedSet,
                Some("TypedIdScalarV1"),
                Some((0, 5)),
            ),
        ],
    }
}

fn field(
    name: &str,
    field_type: FieldTypeV1,
    target: Option<&str>,
    bounds: Option<(u16, u16)>,
) -> ContractFieldV1 {
    ContractFieldV1 {
        name: name.into(),
        field_type,
        target: target.map(str::to_owned),
        bounds: bounds.map(|(min_items, max_items)| CollectionBoundsV1 {
            min_items,
            max_items,
        }),
    }
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
