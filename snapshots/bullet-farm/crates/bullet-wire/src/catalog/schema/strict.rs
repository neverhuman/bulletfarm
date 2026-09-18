use serde_json::{Map, Value, json};

use crate::WireError;

use super::super::{
    CodeClassV1, CollectionBoundsV1, ContractFieldV1, FieldTypeV1, LegacyReferenceShapeV1,
    ResolvedSymbolV1, ResolvedTaggedUnionV1, ScalarDefinitionV1, ScalarTypeV1,
};
use super::{field_schema, mismatch};

pub(super) fn legacy_reference_schema(
    field: &ContractFieldV1,
    shape: LegacyReferenceShapeV1,
    target: ResolvedSymbolV1<'_>,
) -> Result<Value, WireError> {
    let shape_matches = match shape {
        LegacyReferenceShapeV1::Direct => matches!(
            field.field_type,
            FieldTypeV1::RiskPolicy
                | FieldTypeV1::EvidencePolicy
                | FieldTypeV1::SandboxPolicy
                | FieldTypeV1::BudgetPolicy
                | FieldTypeV1::RoutePolicy
                | FieldTypeV1::SignedAuthorityEnvelope
                | FieldTypeV1::SignedMutationPermit
                | FieldTypeV1::MutationReplayResult
                | FieldTypeV1::ScopeGrant
                | FieldTypeV1::PatchProposal
                | FieldTypeV1::CleanupAuthorization
                | FieldTypeV1::ReleaseFamilySubject
        ),
        LegacyReferenceShapeV1::Optional => matches!(
            field.field_type,
            FieldTypeV1::OptionalSignedMutationPermit | FieldTypeV1::OptionalMutationReplayResult
        ),
        LegacyReferenceShapeV1::Array => matches!(
            field.field_type,
            FieldTypeV1::IssuerKeyArray
                | FieldTypeV1::PatchOperationArray
                | FieldTypeV1::ReleaseRepositorySubjectArray
                | FieldTypeV1::ReleaseEvidenceSubjectArray
                | FieldTypeV1::ReleaseProfileNodeArray
                | FieldTypeV1::ReleaseSignerKeyArray
                | FieldTypeV1::ReleaseRegistryEntryArray
                | FieldTypeV1::ReleaseRegistryObjectArray
                | FieldTypeV1::ReleaseReplayBindingArray
                | FieldTypeV1::ExecutionToolArray
        ),
    };
    if !shape_matches {
        return Err(mismatch(field));
    }
    if field.field_type == FieldTypeV1::ExecutionToolArray {
        return Ok(json!({
            "type": "array", "minItems": 1, "maxItems": 64, "uniqueItems": true,
            "items": {"$ref": format!("#/schemas/{}", target.name())}
        }));
    }
    Ok(match shape {
        LegacyReferenceShapeV1::Direct => named_reference(target),
        LegacyReferenceShapeV1::Optional => optional_named_reference(target),
        LegacyReferenceShapeV1::Array => json!({
            "type": "array", "items": named_reference(target)
        }),
    })
}

pub(super) fn named_reference(target: ResolvedSymbolV1<'_>) -> Value {
    json!({"$ref": format!("#/schemas/{}", target.name())})
}

pub(super) fn optional_named_reference(target: ResolvedSymbolV1<'_>) -> Value {
    json!({"anyOf": [named_reference(target), {"type": "null"}]})
}

pub(super) fn bounded_array(target: ResolvedSymbolV1<'_>, bounds: CollectionBoundsV1) -> Value {
    json!({
        "items": named_reference(target),
        "maxItems": bounds.max_items,
        "minItems": bounds.min_items,
        "type": "array",
    })
}

pub(super) fn bounded_set(target: ResolvedSymbolV1<'_>, bounds: CollectionBoundsV1) -> Value {
    json!({
        "items": named_reference(target),
        "maxItems": bounds.max_items,
        "minItems": bounds.min_items,
        "type": "array",
        "uniqueItems": true,
        "x-bullet-order": "rfc8785",
    })
}

pub(super) fn scalar_schema(scalar: &ScalarTypeV1) -> Value {
    match &scalar.definition {
        ScalarDefinitionV1::SafeInteger { minimum, maximum } if minimum == maximum => {
            json!({"const": minimum, "type": "integer"})
        }
        ScalarDefinitionV1::SafeInteger { minimum, maximum } => {
            json!({"maximum": maximum, "minimum": minimum, "type": "integer"})
        }
        ScalarDefinitionV1::Text {
            minimum_utf8_bytes,
            maximum_utf8_bytes,
        } => json!({
            "maxLength": maximum_utf8_bytes,
            "minLength": 1,
            "type": "string",
            "x-bullet-max-utf8-bytes": maximum_utf8_bytes,
            "x-bullet-min-utf8-bytes": minimum_utf8_bytes,
        }),
        ScalarDefinitionV1::Code {
            minimum_ascii_bytes,
            maximum_ascii_bytes,
            class,
        } => json!({
            "maxLength": maximum_ascii_bytes,
            "minLength": minimum_ascii_bytes,
            "pattern": code_pattern(*class),
            "type": "string",
        }),
        ScalarDefinitionV1::Enum { values } => {
            json!({"enum": values, "type": "string"})
        }
        ScalarDefinitionV1::TypedId { prefix } => json!({
            "pattern": format!("^{prefix}_[0-9a-f]{{64}}$"),
            "type": "string",
        }),
    }
}

pub(super) fn tagged_union_schema(
    tagged_union: &ResolvedTaggedUnionV1<'_>,
) -> Result<Value, WireError> {
    let discriminator = &tagged_union.definition().discriminator;
    let mut branches = Vec::with_capacity(tagged_union.variants().len());
    for variant in tagged_union.variants() {
        let mut properties = Map::new();
        properties.insert(
            discriminator.clone(),
            json!({"const": variant.definition().tag, "type": "string"}),
        );
        let mut required = Vec::with_capacity(variant.record().fields.len() + 1);
        required.push(Value::String(discriminator.clone()));
        for field in variant.fields() {
            if properties
                .insert(field.definition().name.clone(), field_schema(field)?)
                .is_some()
            {
                return Err(mismatch(field.definition()));
            }
            required.push(Value::String(field.definition().name.clone()));
        }
        branches.push(json!({
            "additionalProperties": false,
            "properties": properties,
            "required": required,
            "type": "object",
        }));
    }
    Ok(json!({"oneOf": branches}))
}

const fn code_pattern(class: CodeClassV1) -> &'static str {
    match class {
        CodeClassV1::LowerKebab => "^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$",
        CodeClassV1::UpperHyphen => "^[A-Z][A-Z0-9]*(?:-[A-Z0-9]+)*$",
        CodeClassV1::AsciiToken => "^[A-Za-z0-9][A-Za-z0-9._:/+-]*$",
        CodeClassV1::InvariantId => "^BF-[A-Z0-9]+(?:-[A-Z0-9]+)*$",
    }
}
