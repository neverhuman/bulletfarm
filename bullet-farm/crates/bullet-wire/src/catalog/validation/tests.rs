use std::collections::BTreeSet;

use sha2::{Digest, Sha256};

use super::super::CollectionBoundsV1;
use super::*;
use crate::{canonical_json, decode_canonical};

fn field(name: &str, field_type: FieldTypeV1) -> ContractFieldV1 {
    ContractFieldV1 {
        name: name.into(),
        field_type,
        target: None,
        bounds: None,
    }
}

fn reference(
    name: &str,
    field_type: FieldTypeV1,
    target: &str,
    bounds: Option<CollectionBoundsV1>,
) -> ContractFieldV1 {
    ContractFieldV1 {
        name: name.into(),
        field_type,
        target: Some(target.into()),
        bounds,
    }
}

fn embedded(name: &str) -> ContractRecordV1 {
    ContractRecordV1 {
        name: name.into(),
        security_class: super::super::SecurityClassV1::Verification,
        unknown_fields: "reject".into(),
        shape: RecordShapeV1::Embedded,
        fields: vec![field("value", FieldTypeV1::String)],
    }
}

fn strict_catalog() -> ContractCatalogV1 {
    let scalars = vec![
        ScalarTypeV1 {
            name: "CodeV1".into(),
            definition: ScalarDefinitionV1::Code {
                minimum_ascii_bytes: 1,
                maximum_ascii_bytes: 16,
                class: super::super::CodeClassV1::LowerKebab,
            },
        },
        ScalarTypeV1 {
            name: "EnumV1".into(),
            definition: ScalarDefinitionV1::Enum {
                values: vec!["alpha".into(), "beta".into()],
            },
        },
        ScalarTypeV1 {
            name: "IntegerV1".into(),
            definition: ScalarDefinitionV1::SafeInteger {
                minimum: -1,
                maximum: 1,
            },
        },
        ScalarTypeV1 {
            name: "TextV1".into(),
            definition: ScalarDefinitionV1::Text {
                minimum_utf8_bytes: 1,
                maximum_utf8_bytes: 32,
            },
        },
        ScalarTypeV1 {
            name: "TypedIdV1".into(),
            definition: ScalarDefinitionV1::TypedId {
                prefix: "tid".into(),
            },
        },
    ];
    let root = ContractRecordV1 {
        name: "RootV1".into(),
        security_class: super::super::SecurityClassV1::Verification,
        unknown_fields: "reject".into(),
        shape: RecordShapeV1::Versioned,
        fields: vec![
            field("schema_version", FieldTypeV1::SchemaVersion),
            reference("alpha", FieldTypeV1::NamedRef, "AlphaV1", None),
            reference("choice", FieldTypeV1::NamedRef, "ChoiceV1", None),
            reference("code", FieldTypeV1::OptionalNamedRef, "CodeV1", None),
            reference(
                "items",
                FieldTypeV1::BoundedArray,
                "TextV1",
                Some(CollectionBoundsV1 {
                    min_items: 0,
                    max_items: 4,
                }),
            ),
            reference(
                "ids",
                FieldTypeV1::BoundedSet,
                "TypedIdV1",
                Some(CollectionBoundsV1 {
                    min_items: 1,
                    max_items: 2,
                }),
            ),
        ],
    };
    ContractCatalogV1 {
        schema_version: "v1alpha1".into(),
        catalog_version: "test.strict.v1".into(),
        scalar_types: scalars,
        tagged_unions: vec![TaggedUnionV1 {
            name: "ChoiceV1".into(),
            discriminator: "kind".into(),
            variants: vec![
                super::super::UnionVariantV1 {
                    tag: "alpha".into(),
                    record: "AlphaV1".into(),
                },
                super::super::UnionVariantV1 {
                    tag: "zulu".into(),
                    record: "ZuluV1".into(),
                },
            ],
        }],
        records: vec![embedded("AlphaV1"), root, embedded("ZuluV1")],
    }
}

fn strict_code(catalog: &ContractCatalogV1) -> Option<&'static str> {
    catalog
        .resolve_test_strict()
        .err()
        .map(|error| error.code())
}

fn root_mut(catalog: &mut ContractCatalogV1) -> &mut ContractRecordV1 {
    catalog
        .records
        .iter_mut()
        .find(|record| record.name == "RootV1")
        .expect("strict fixture has RootV1")
}

fn root_field_mut<'a>(catalog: &'a mut ContractCatalogV1, name: &str) -> &'a mut ContractFieldV1 {
    root_mut(catalog)
        .fields
        .iter_mut()
        .find(|field| field.name == name)
        .expect("strict fixture field exists")
}

#[rustfmt::skip]
fn apply_fault(catalog: &mut ContractCatalogV1, class: usize) {
    match class {
        1 => catalog.schema_version = "bad".into(),
        2 => catalog.records[0].name = "bad".into(),
        3 => { let duplicate = root_mut(catalog).clone(); catalog.records.push(duplicate); }
        4 => root_field_mut(catalog, "alpha").name = "type".into(),
        5 => { root_mut(catalog).fields.remove(0); }
        6 => catalog.records[0].fields.push(field("schema_version", FieldTypeV1::SchemaVersion)),
        7 => root_field_mut(catalog, "alpha").target = Some("MissingV1".into()),
        8 => root_field_mut(catalog, "items").bounds = Some(CollectionBoundsV1 { min_items: 0, max_items: 0 }),
        9 => { catalog.tagged_unions[0].variants.pop(); }
        10 => root_field_mut(catalog, "alpha").target = Some("RootV1".into()),
        11 => { let item = root_field_mut(catalog, "code"); item.field_type = FieldTypeV1::Object; item.target = None; }
        12 => catalog.catalog_version = "v1alpha1.0".into(),
        _ => unreachable!(),
    }
}

#[test]
#[rustfmt::skip]
fn legacy_defaults_round_trip_and_four_sentinels_hold() {
    let catalog_bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../contracts/v1alpha1/contract-catalog.json"
    ));
    let catalog = decode_canonical::<ContractCatalogV1>(catalog_bytes).expect("catalog decodes");
    assert!(catalog.validate().is_ok());
    let rendered = canonical_json(&catalog.json_schema_bundle().expect("legacy schema resolves")).expect("schema encodes");
    assert_eq!(rendered, include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../contracts/v1alpha1/schema-bundle.json")));
    let mut reordered = catalog.clone(); reordered.records.swap(0, 1);
    assert_eq!(reordered.validate().unwrap_err().code(), "INVALID_CONTRACT_FIELD_REFERENCE");
    let mut malformed = catalog.clone(); let schema = malformed.records[0].fields.iter_mut().find(|field| field.name == "schema_version").unwrap(); schema.field_type = FieldTypeV1::Boolean; schema.target = Some("IssuerKeyV1".into());
    assert_eq!(malformed.validate().unwrap_err().code(), "INVALID_CONTRACT_RECORD_SHAPE");
    let mut metadata = catalog.clone(); metadata.records[0].fields.iter_mut().find(|field| field.name == "schema_version").unwrap().target = Some("IssuerKeyV1".into());
    assert_eq!(metadata.validate().unwrap_err().code(), "INVALID_CONTRACT_RECORD_SHAPE");
    let mut remapped = catalog.clone(); let schema = remapped.records.iter_mut().flat_map(|record| &mut record.fields).find(|field| field.name == "schema_version" && field.field_type == FieldTypeV1::String).unwrap(); schema.field_type = FieldTypeV1::SchemaVersion;
    assert_eq!(remapped.validate().unwrap_err().code(), "INVALID_CONTRACT_RECORD_SHAPE");
    let mut schema_rows = catalog.records.iter().map(|record| (record.name.as_str(), record.fields.iter().find(|field| field.name == "schema_version").unwrap().field_type)).collect::<Vec<_>>(); schema_rows.sort_unstable_by(|left, right| left.0.cmp(right.0));
    assert_eq!(format!("{:x}", Sha256::digest(canonical_json(&schema_rows).unwrap())), "e6205e51b34aecd4956bb1f30f3907ebbb5eedeab5dfd1b531e291723f51cf07");
    assert!(catalog.scalar_types.is_empty() && catalog.tagged_unions.is_empty());
    assert!(catalog.records.iter().all(|record| {
        record.shape == RecordShapeV1::Versioned
            && record
                .fields
                .iter()
                .all(|field| field.target.is_none() && field.bounds.is_none())
    }));
    assert_eq!(
        canonical_json(&catalog).expect("catalog encodes"),
        catalog_bytes
    );
    let subjects: &[(&[u8], &str)] = &[
        (catalog_bytes, "0b8319f4527673c5879b5afcf6d9ba15f5b824ec2488c6ca18b0f50b9fc2ac14"),
        (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../contracts/v1alpha1/schema-bundle.json")), "5b47756bcab8bc88aa24c42a5bcf535e6cbcf95241151b5ebfc50055e7d0b167"),
        (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../contracts/generated/rust/schema_bundle.rs")), "53d84a74f1ef9482811718c7e3df1744daea0f7c098b1740de7d3e4760e531a9"),
        (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../contracts/generated/typescript/schemaBundle.ts")), "ff1e5266fa0b74069bf53cd0c8f7722c653b38b98e7679ccb351d2c342ff2ef0"),
    ];
    for (bytes, expected) in subjects {
        assert_eq!(format!("{:x}", Sha256::digest(bytes)), *expected);
    }
}

#[test]
fn strict_fixture_resolves_new_types_and_sorted_graph() {
    let catalog = strict_catalog();
    let resolved = catalog
        .resolve_test_strict()
        .expect("strict fixture resolves");
    assert_eq!(resolved.scalar_types().len(), 5);
    assert_eq!(resolved.records().len(), 3);
    assert_eq!(resolved.tagged_unions().len(), 1);
    assert_eq!(resolved.symbols().len(), 9);
    assert_eq!(resolved.adjacency()["RootV1"], vec!["AlphaV1", "ChoiceV1"]);
    assert_eq!(resolved.adjacency()["ChoiceV1"], vec!["AlphaV1", "ZuluV1"]);
    let union = &resolved.tagged_unions()[0];
    assert_eq!(union.variants().len(), 2);
    assert!(
        union
            .variants()
            .iter()
            .all(|variant| !variant.fields().is_empty())
    );
    assert!(resolved.record("RootV1").is_some());
}

#[test]
fn public_paths_enforce_coverage_while_test_helper_skips_only_coverage() {
    let fixture = strict_catalog();
    assert!(fixture.resolve_test_strict().is_ok());
    let mut production = fixture.clone();
    production.catalog_version = "v1alpha1.0".into();
    assert_eq!(
        production.validate().unwrap_err().code(),
        "CONTRACT_CATALOG_COVERAGE"
    );
    assert_eq!(
        production.json_schema_bundle().unwrap_err().code(),
        "CONTRACT_CATALOG_COVERAGE"
    );
    let mut open = fixture;
    let field = root_field_mut(&mut open, "alpha");
    field.field_type = FieldTypeV1::Object;
    field.target = None;
    assert_eq!(strict_code(&open), Some("OPEN_CONTRACT_FIELD"));
}

#[test]
#[rustfmt::skip]
fn validation_precedence_and_lexical_selection_are_stable() {
    let expected = [
        "INVALID_CONTRACT_CATALOG", "INVALID_CONTRACT_RECORD", "DUPLICATE_CONTRACT_RECORD",
        "INVALID_CONTRACT_FIELD", "MISSING_SCHEMA_VERSION", "INVALID_CONTRACT_RECORD_SHAPE",
        "INVALID_CONTRACT_FIELD_REFERENCE", "INVALID_CONTRACT_FIELD_BOUNDS",
        "INVALID_CONTRACT_TAGGED_UNION", "CONTRACT_TYPE_CYCLE", "OPEN_CONTRACT_FIELD",
        "CONTRACT_CATALOG_COVERAGE",
    ];
    for lower in 1..=12 {
        for higher in lower + 1..=12 {
            let mut failures = Failures::default();
            failures.add(higher, expected[higher as usize - 1], "a", "", "higher");
            failures.add(lower, expected[lower as usize - 1], "z", "", "lower");
            assert_eq!(failures.into_error().unwrap().code(), expected[lower as usize - 1]);
        }
        let mut failures = Failures::default();
        failures.add(lower, expected[lower as usize - 1], "z", "", "same");
        failures.add(lower, expected[lower as usize - 1], "a", "", "same");
        assert!(failures.into_error().unwrap().reason().starts_with("a/"));
    }
    for (index, code) in expected.iter().enumerate() {
        let mut catalog = strict_catalog();
        apply_fault(&mut catalog, index + 1);
        if index < 10 { apply_fault(&mut catalog, index + 2); }
        let actual = if index == 11 {
            catalog.validate().err().map(|error| error.code())
        } else {
            strict_code(&catalog)
        };
        assert_eq!(actual, Some(*code), "precedence class {}", index + 1);
    }
    let mut lexical = strict_catalog();
    lexical.records[0].fields[0].name = "type".into();
    lexical.records[2].fields[0].name = "type".into();
    let error = lexical.resolve_test_strict().unwrap_err();
    assert!(error.reason().starts_with("AlphaV1/type/"));
    let mut references = strict_catalog();
    references.records[0].fields[0] = reference("value", FieldTypeV1::NamedRef, "ZuluMissingV1", None);
    references.records[2].fields[0] = reference("value", FieldTypeV1::NamedRef, "AlphaMissingV1", None);
    assert!(references.resolve_test_strict().unwrap_err().reason().starts_with("AlphaV1/value/"));
}

#[test]
#[rustfmt::skip]
fn names_metadata_and_generator_reservations_refuse() {
    for case in 0..13 {
        let mut catalog = strict_catalog();
        match case {
            0 => catalog.scalar_types[0].name = "String".into(),
            1 => catalog.records[0].name = "CodeV1".into(),
            2 => catalog.scalar_types[0].name = "BulletGeneratedThing".into(),
            3 => root_field_mut(&mut catalog, "alpha").target = None,
            4 => root_field_mut(&mut catalog, "schema_version").target = Some("AlphaV1".into()),
            5 => if let ScalarDefinitionV1::TypedId { prefix } = &mut catalog.scalar_types[4].definition { *prefix = "Bad".into(); },
            6 => if let ScalarDefinitionV1::Enum { values } = &mut catalog.scalar_types[1].definition { *values = vec!["A".into(), "a".into()]; },
            7 => catalog.scalar_types.swap(0, 1),
            8 => root_field_mut(&mut catalog, "alpha").name = "await".into(),
            9 => catalog.records.swap(0, 1),
            10 => root_field_mut(&mut catalog, "alpha").name = "any".into(),
            11 => root_field_mut(&mut catalog, "alpha").name = "extern".into(),
            12 => { catalog.records[0].name = "CodeV1".into(); catalog.records[1].name = "CodeV1".into(); }
            _ => unreachable!(),
        }
        let expected = match case { 4 => "INVALID_CONTRACT_RECORD_SHAPE", 8 | 10 | 11 => "INVALID_CONTRACT_FIELD", 12 => "DUPLICATE_CONTRACT_RECORD", _ => "INVALID_CONTRACT_FIELD_REFERENCE" };
        assert_eq!(strict_code(&catalog), Some(expected), "case {case}");
    }
}

#[test]
#[rustfmt::skip]
fn scalar_and_collection_bounds_refuse() {
    for case in 0..14 {
        let mut catalog = strict_catalog();
        match case {
            0 => catalog.scalar_types[2].definition = ScalarDefinitionV1::SafeInteger { minimum: 2, maximum: 1 },
            1 => catalog.scalar_types[2].definition = ScalarDefinitionV1::SafeInteger { minimum: -9_007_199_254_740_992, maximum: 1 },
            2 => catalog.scalar_types[2].definition = ScalarDefinitionV1::SafeInteger { minimum: 0, maximum: 9_007_199_254_740_992 },
            3 => catalog.scalar_types[3].definition = ScalarDefinitionV1::Text { minimum_utf8_bytes: 0, maximum_utf8_bytes: 1 },
            4 => catalog.scalar_types[3].definition = ScalarDefinitionV1::Text { minimum_utf8_bytes: 2, maximum_utf8_bytes: 1 },
            5 => catalog.scalar_types[3].definition = ScalarDefinitionV1::Text { minimum_utf8_bytes: 1, maximum_utf8_bytes: 8_388_609 },
            6 => catalog.scalar_types[0].definition = ScalarDefinitionV1::Code { minimum_ascii_bytes: 0, maximum_ascii_bytes: 1, class: super::super::CodeClassV1::AsciiToken },
            7 => catalog.scalar_types[0].definition = ScalarDefinitionV1::Code { minimum_ascii_bytes: 2, maximum_ascii_bytes: 1, class: super::super::CodeClassV1::AsciiToken },
            8 => catalog.scalar_types[0].definition = ScalarDefinitionV1::Code { minimum_ascii_bytes: 1, maximum_ascii_bytes: 257, class: super::super::CodeClassV1::AsciiToken },
            9 => catalog.scalar_types[1].definition = ScalarDefinitionV1::Enum { values: vec![] },
            10 => catalog.scalar_types[1].definition = ScalarDefinitionV1::Enum { values: (0..257).map(|n| format!("A{n:03}")).collect() },
            11 => root_field_mut(&mut catalog, "items").bounds = Some(CollectionBoundsV1 { min_items: 0, max_items: 4097 }),
            12 => root_field_mut(&mut catalog, "items").bounds = Some(CollectionBoundsV1 { min_items: 3, max_items: 2 }),
            13 => catalog.scalar_types[1].definition = ScalarDefinitionV1::Enum { values: std::iter::once(String::new()).chain((0..256).map(|n| format!("A{n:03}"))).collect() },
            _ => unreachable!(),
        }
        let expected = if case == 13 { "INVALID_CONTRACT_FIELD_REFERENCE" } else { "INVALID_CONTRACT_FIELD_BOUNDS" };
        assert_eq!(strict_code(&catalog), Some(expected), "case {case}");
    }
}

#[test]
#[rustfmt::skip]
fn record_shapes_and_tagged_unions_refuse() {
    let expected = ["MISSING_SCHEMA_VERSION", "INVALID_CONTRACT_RECORD_SHAPE", "INVALID_CONTRACT_RECORD_SHAPE", "INVALID_CONTRACT_TAGGED_UNION", "INVALID_CONTRACT_TAGGED_UNION", "INVALID_CONTRACT_TAGGED_UNION", "INVALID_CONTRACT_TAGGED_UNION", "INVALID_CONTRACT_TAGGED_UNION", "INVALID_CONTRACT_TAGGED_UNION", "INVALID_CONTRACT_TAGGED_UNION", "INVALID_CONTRACT_TAGGED_UNION"];
    for (case, code) in expected.iter().enumerate() {
        let mut catalog = strict_catalog();
        match case {
            0 => { root_mut(&mut catalog).fields.remove(0); }
            1 => root_mut(&mut catalog).shape = RecordShapeV1::Embedded,
            2 => catalog.records[0].fields.push(field("schema_version", FieldTypeV1::SchemaVersion)),
            3 => { catalog.tagged_unions[0].variants.pop(); }
            4 => catalog.tagged_unions[0].variants.swap(0, 1),
            5 => catalog.tagged_unions[0].variants[0].record = "RootV1".into(),
            6 => catalog.tagged_unions[0].discriminator = "value".into(),
            7 => catalog.tagged_unions[0].variants[1].record = "AlphaV1".into(),
            8 => { catalog.tagged_unions[0].variants[0].tag = "A".into(); catalog.tagged_unions[0].variants[1].tag = "a".into(); }
            9 => catalog.tagged_unions[0].discriminator = "type".into(),
            10 => catalog.tagged_unions[0].variants = (0..33).map(|n| super::super::UnionVariantV1 { tag: format!("A{n:02}"), record: "AlphaV1".into() }).collect(),
            _ => unreachable!(),
        }
        assert_eq!(strict_code(&catalog), Some(*code), "case {case}");
    }
}

#[test]
#[rustfmt::skip]
fn direct_indirect_and_mixed_cycles_refuse() {
    for case in 0..4 {
        let mut catalog = strict_catalog();
        match case {
            0 => root_field_mut(&mut catalog, "alpha").target = Some("RootV1".into()),
            1 => { catalog.records[0].fields[0] = reference("root", FieldTypeV1::NamedRef, "RootV1", None); root_field_mut(&mut catalog, "choice").target = Some("CodeV1".into()); }
            2 => { catalog.records[0].fields[0] = reference("root", FieldTypeV1::NamedRef, "RootV1", None); root_field_mut(&mut catalog, "alpha").target = Some("TextV1".into()); }
            3 => {
                let mut risk = embedded("RiskPolicyV1");
                risk.fields[0] = reference("root", FieldTypeV1::NamedRef, "RootV1", None);
                catalog.records.insert(1, risk);
                let root = root_field_mut(&mut catalog, "alpha");
                root.field_type = FieldTypeV1::RiskPolicy;
                root.target = None;
            }
            _ => unreachable!(),
        }
        assert_eq!(strict_code(&catalog), Some("CONTRACT_TYPE_CYCLE"), "case {case}");
    }
    let mut mixed = strict_catalog();
    mixed.records[0].fields[0] = reference("root", FieldTypeV1::NamedRef, "RootV1", None);
    root_field_mut(&mut mixed, "alpha").target = Some("TextV1".into());
    let reason = mixed.resolve_test_strict().unwrap_err().reason().to_owned();
    root_mut(&mut mixed).fields.reverse();
    assert_eq!(reason, "ChoiceV1/AlphaV1/cycle_edge");
    assert_eq!(mixed.resolve_test_strict().unwrap_err().reason(), reason);
}

#[test]
fn legacy_target_oracle_is_exact_and_exhaustive() {
    assert_eq!(ALL_FIELD_TYPES.len(), 107);
    let mut mapped = 0;
    let mut targets = BTreeSet::new();
    for field_type in ALL_FIELD_TYPES {
        let expected = oracle(*field_type);
        assert_eq!(legacy_reference(*field_type), expected, "{field_type:?}");
        if let Some((target, _)) = expected {
            mapped += 1;
            targets.insert(target);
        }
    }
    assert_eq!(mapped, 24);
    assert_eq!(targets.len(), 22);
}

#[test]
#[rustfmt::skip]
fn unknown_fields_and_meta_container_caps_refuse() {
    let mut value = serde_json::to_value(strict_catalog()).expect("fixture serializes");
    value["extra"] = serde_json::json!(true);
    assert!(serde_json::from_value::<ContractCatalogV1>(value).is_err());
    for path in 0..7 {
        let mut value = serde_json::to_value(strict_catalog()).expect("fixture serializes");
        match path {
            0 => value["scalar_types"][0]["definition"]["extra"] = serde_json::json!(true),
            1 => value["records"][1]["fields"][4]["bounds"]["extra"] = serde_json::json!(true),
            2 => value["tagged_unions"][0]["variants"][0]["extra"] = serde_json::json!(true),
            3 => value["records"][0]["security_class"] = serde_json::json!("unknown"),
            4 => value["records"][0]["shape"] = serde_json::json!("unknown"),
            5 => value["records"][0]["fields"][0]["field_type"] = serde_json::json!("unknown"),
            6 => value["scalar_types"][0]["definition"]["class"] = serde_json::json!("unknown"),
            _ => unreachable!(),
        }
        assert!(serde_json::from_value::<ContractCatalogV1>(value).is_err(), "path {path}");
    }
    let mut unknown_version = strict_catalog();
    unknown_version.catalog_version = "unknown".into();
    assert_eq!(strict_code(&unknown_version), Some("INVALID_CONTRACT_CATALOG"));
    for case in 0..4 {
        let mut catalog = strict_catalog();
        let expected = match case {
            0 => { catalog.scalar_types = vec![catalog.scalar_types[0].clone(); 257]; "INVALID_CONTRACT_CATALOG" }
            1 => { catalog.tagged_unions = vec![catalog.tagged_unions[0].clone(); 129]; "INVALID_CONTRACT_CATALOG" }
            2 => { catalog.records = vec![catalog.records[0].clone(); 513]; "INVALID_CONTRACT_CATALOG" }
            3 => { root_mut(&mut catalog).fields = vec![field("value", FieldTypeV1::String); 257]; "INVALID_CONTRACT_RECORD" }
            _ => unreachable!(),
        };
        assert_eq!(strict_code(&catalog), Some(expected), "case {case}");
    }
}

#[rustfmt::skip]
const ALL_FIELD_TYPES: &[FieldTypeV1] = &[
    FieldTypeV1::String, FieldTypeV1::SchemaVersion, FieldTypeV1::PolicySchemaVersion, FieldTypeV1::Identifier, FieldTypeV1::Digest, FieldTypeV1::OrganizationId, FieldTypeV1::RepositoryId, FieldTypeV1::MissionId, FieldTypeV1::AcceptanceContractId, FieldTypeV1::PlanRevisionId, FieldTypeV1::GraphRevisionId, FieldTypeV1::WorkPackageId, FieldTypeV1::SelectionGroupId, FieldTypeV1::VariantId, FieldTypeV1::AttemptId, FieldTypeV1::RunnerId, FieldTypeV1::WorkspaceId, FieldTypeV1::PrincipalId, FieldTypeV1::ProviderProfileId, FieldTypeV1::ContentId, FieldTypeV1::MutationId, FieldTypeV1::MutationReservationId, FieldTypeV1::ScopeGrantId, FieldTypeV1::SourceDescriptorId, FieldTypeV1::ChangeId, FieldTypeV1::CheckpointId, FieldTypeV1::CandidateId, FieldTypeV1::GateReceiptId, FieldTypeV1::ReleaseRegistryId, FieldTypeV1::GateId, FieldTypeV1::EffectIntentId, FieldTypeV1::CandidateProofRoot, FieldTypeV1::IntegrationProofRoot, FieldTypeV1::GitOid, FieldTypeV1::TaggedBlake3Digest, FieldTypeV1::ReleaseGateId, FieldTypeV1::ReleaseNativeSubjectId, FieldTypeV1::ReleaseProfileId, FieldTypeV1::ReleaseTag, FieldTypeV1::SigningIdentity, FieldTypeV1::SshEd25519PublicKey, FieldTypeV1::RepoPath, FieldTypeV1::AuthorityAudience, FieldTypeV1::MutationOperation, FieldTypeV1::AuthorityDecision, FieldTypeV1::ReplayDisposition, FieldTypeV1::MutationResultState, FieldTypeV1::MutationOutcome, FieldTypeV1::SettlementStatus, FieldTypeV1::PatchPreimageKind, FieldTypeV1::PatchMutationKind, FieldTypeV1::ReleaseReceiptKind, FieldTypeV1::ReleaseEvidenceKind, FieldTypeV1::ReleaseRegistryObjectKind, FieldTypeV1::ReleaseSignerRole, FieldTypeV1::ReleaseRepositoryName, FieldTypeV1::KeyId, FieldTypeV1::KeyPurpose, FieldTypeV1::KeyAlgorithm, FieldTypeV1::PasetoV4Public, FieldTypeV1::SafeU64, FieldTypeV1::U64, FieldTypeV1::Timestamp, FieldTypeV1::OptionalTimestamp, FieldTypeV1::OptionalDigest, FieldTypeV1::OptionalString, FieldTypeV1::OptionalMutationReservationId, FieldTypeV1::Boolean, FieldTypeV1::Object, FieldTypeV1::StringArray, FieldTypeV1::ObjectArray, FieldTypeV1::AuthorityAudienceArray, FieldTypeV1::IssuerKeyArray, FieldTypeV1::RiskPolicy, FieldTypeV1::EvidencePolicy, FieldTypeV1::SandboxPolicy, FieldTypeV1::BudgetPolicy, FieldTypeV1::RoutePolicy, FieldTypeV1::SignedAuthorityEnvelope, FieldTypeV1::SignedMutationPermit, FieldTypeV1::OptionalSignedMutationPermit, FieldTypeV1::MutationReplayResult, FieldTypeV1::OptionalMutationReplayResult, FieldTypeV1::ScopeGrant, FieldTypeV1::PatchProposal, FieldTypeV1::PatchOperationArray, FieldTypeV1::CandidateIdArray, FieldTypeV1::OrderedCandidateIdArray, FieldTypeV1::GateIdArray, FieldTypeV1::ReleaseGateIdArray, FieldTypeV1::ReleaseProfileIdArray, FieldTypeV1::ReleaseEvidenceKindArray, FieldTypeV1::RepoPathArray, FieldTypeV1::CleanupAuthorization, FieldTypeV1::ReleaseFamilySubject, FieldTypeV1::ReleaseRepositorySubjectArray, FieldTypeV1::ReleaseEvidenceSubjectArray, FieldTypeV1::ReleaseProfileNodeArray, FieldTypeV1::ReleaseSignerKeyArray, FieldTypeV1::ReleaseRegistryEntryArray, FieldTypeV1::ReleaseRegistryObjectArray, FieldTypeV1::ReleaseReplayBindingArray, FieldTypeV1::ExecutionToolArray, FieldTypeV1::NamedRef, FieldTypeV1::OptionalNamedRef, FieldTypeV1::BoundedArray, FieldTypeV1::BoundedSet,
];

#[rustfmt::skip]
fn oracle(field_type: FieldTypeV1) -> Option<(&'static str, LegacyReferenceShapeV1)> {
    use FieldTypeV1::*; use LegacyReferenceShapeV1::{Array, Direct, Optional};
    match field_type {
        IssuerKeyArray => Some(("IssuerKeyV1", Array)), RiskPolicy => Some(("RiskPolicyV1", Direct)), EvidencePolicy => Some(("EvidencePolicyV1", Direct)), SandboxPolicy => Some(("SandboxPolicyV1", Direct)), BudgetPolicy => Some(("BudgetPolicyV1", Direct)), RoutePolicy => Some(("RoutePolicyV1", Direct)), SignedAuthorityEnvelope => Some(("SignedAuthorityEnvelopeV1", Direct)), SignedMutationPermit => Some(("SignedMutationPermitV1", Direct)), OptionalSignedMutationPermit => Some(("SignedMutationPermitV1", Optional)), MutationReplayResult => Some(("MutationReplayResultV1", Direct)), OptionalMutationReplayResult => Some(("MutationReplayResultV1", Optional)), ScopeGrant => Some(("ScopeGrantV1", Direct)), PatchProposal => Some(("PatchProposalV1", Direct)), PatchOperationArray => Some(("PatchOperationV1", Array)), CleanupAuthorization => Some(("CleanupAuthorizationV1", Direct)), ReleaseFamilySubject => Some(("ReleaseFamilySubjectV1", Direct)), ReleaseRepositorySubjectArray => Some(("ReleaseRepositorySubjectV1", Array)), ReleaseEvidenceSubjectArray => Some(("ReleaseEvidenceSubjectV1", Array)), ReleaseProfileNodeArray => Some(("ReleaseProfileNodeV1", Array)), ReleaseSignerKeyArray => Some(("ReleaseSignerKeyV1", Array)), ReleaseRegistryEntryArray => Some(("ReleaseRegistryEntryV1", Array)), ReleaseRegistryObjectArray => Some(("ReleaseRegistryObjectV1", Array)), ReleaseReplayBindingArray => Some(("ReleaseReplayBindingV1", Array)), ExecutionToolArray => Some(("ExecutionToolV1", Array)),
        String | SchemaVersion | PolicySchemaVersion | Identifier | Digest | OrganizationId | RepositoryId | MissionId | AcceptanceContractId | PlanRevisionId | GraphRevisionId | WorkPackageId | SelectionGroupId | VariantId | AttemptId | RunnerId | WorkspaceId | PrincipalId | ProviderProfileId | ContentId | MutationId | MutationReservationId | ScopeGrantId | SourceDescriptorId | ChangeId | CheckpointId | CandidateId | GateReceiptId | ReleaseRegistryId | GateId | EffectIntentId | CandidateProofRoot | IntegrationProofRoot | GitOid | TaggedBlake3Digest | ReleaseGateId | ReleaseNativeSubjectId | ReleaseProfileId | ReleaseTag | SigningIdentity | SshEd25519PublicKey | RepoPath | AuthorityAudience | MutationOperation | AuthorityDecision | ReplayDisposition | MutationResultState | MutationOutcome | SettlementStatus | PatchPreimageKind | PatchMutationKind | ReleaseReceiptKind | ReleaseEvidenceKind | ReleaseRegistryObjectKind | ReleaseSignerRole | ReleaseRepositoryName | KeyId | KeyPurpose | KeyAlgorithm | PasetoV4Public | SafeU64 | U64 | Timestamp | OptionalTimestamp | OptionalDigest | OptionalString | OptionalMutationReservationId | Boolean | Object | StringArray | ObjectArray | AuthorityAudienceArray | CandidateIdArray | OrderedCandidateIdArray | GateIdArray | ReleaseGateIdArray | ReleaseProfileIdArray | ReleaseEvidenceKindArray | RepoPathArray | NamedRef | OptionalNamedRef | BoundedArray | BoundedSet => None,
    }
}
