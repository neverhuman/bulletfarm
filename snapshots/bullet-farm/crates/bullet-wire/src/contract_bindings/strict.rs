use crate::{
    FieldTypeV1, WireError,
    catalog::{
        CodeClassV1, LegacyReferenceShapeV1, ResolvedCatalogV1, ResolvedFieldKindV1,
        ResolvedFieldV1, ResolvedRecordV1, ResolvedSymbolKindV1, ResolvedSymbolV1,
        ResolvedTaggedUnionV1, ScalarDefinitionV1, rust_variant_identifier,
    },
};

use super::{
    StrictLegacyValue, impossible_field,
    strict_template::{
        RUST_ARRAYS, RUST_BOOLEAN, RUST_BOUNDED_ARRAY, RUST_BOUNDED_SET, RUST_BOUNDS, RUST_CHOOSE,
        RUST_COLLECT_ARRAY, RUST_CORE, RUST_INTEGER, RUST_LEGACY_STRING, RUST_REBASE, RUST_SET,
        RUST_STRING, RUST_TEXT, RUST_UNIQUE, TYPESCRIPT_ARRAYS, TYPESCRIPT_BOOLEAN,
        TYPESCRIPT_BOUNDED_ARRAY, TYPESCRIPT_BOUNDED_SET, TYPESCRIPT_BOUNDS, TYPESCRIPT_CANONICAL,
        TYPESCRIPT_COLLECT_ARRAY, TYPESCRIPT_CORE, TYPESCRIPT_INTEGER, TYPESCRIPT_LEGACY_STRING,
        TYPESCRIPT_PATTERN, TYPESCRIPT_SET, TYPESCRIPT_STRING, TYPESCRIPT_TEXT, TYPESCRIPT_UNIQUE,
    },
};

pub(super) fn is_required(catalog: &ResolvedCatalogV1<'_>) -> bool {
    catalog.catalog_version() != "v1alpha1.0"
        || !catalog.scalar_types().is_empty()
        || !catalog.tagged_unions().is_empty()
        || catalog
            .records()
            .iter()
            .flat_map(|record| record.fields())
            .any(|field| {
                matches!(
                    field.kind(),
                    ResolvedFieldKindV1::Named(_)
                        | ResolvedFieldKindV1::OptionalNamed(_)
                        | ResolvedFieldKindV1::BoundedArray { .. }
                        | ResolvedFieldKindV1::BoundedSet { .. }
                )
            })
}

#[rustfmt::skip]
pub(super) fn render_rust(output: &mut String, catalog: &ResolvedCatalogV1<'_>) -> Result<(), WireError> {
    let uses = super::strict_features(catalog);
    output.push_str(RUST_CORE);
    for (needed, fragment) in [(uses.string, RUST_STRING), (uses.arrays, RUST_ARRAYS), (uses.collect_array, RUST_COLLECT_ARRAY), (uses.integer, RUST_INTEGER), (uses.boolean, RUST_BOOLEAN), (uses.bounds, RUST_BOUNDS), (uses.rebase, RUST_REBASE), (uses.set, RUST_SET), (uses.set || uses.unique, RUST_CHOOSE), (uses.unique, RUST_UNIQUE), (uses.text, RUST_TEXT), (uses.legacy_string, RUST_LEGACY_STRING), (uses.bounded_array, RUST_BOUNDED_ARRAY), (uses.bounded_set, RUST_BOUNDED_SET)] { if needed { output.push_str(fragment); } }
    for scalar in catalog.scalar_types() { rust_scalar(output, scalar)?; }
    for union in catalog.tagged_unions() { rust_union(output, union)?; }
    for record in catalog.records() { rust_record(output, record)?; }
    Ok(())
}

#[rustfmt::skip]
pub(super) fn render_typescript(output: &mut String, catalog: &ResolvedCatalogV1<'_>) -> Result<(), WireError> {
    let uses = super::strict_features(catalog);
    output.push_str(TYPESCRIPT_CORE);
    for (needed, fragment) in [(uses.string, TYPESCRIPT_STRING), (uses.pattern, TYPESCRIPT_PATTERN), (uses.arrays, TYPESCRIPT_ARRAYS), (uses.collect_array, TYPESCRIPT_COLLECT_ARRAY), (uses.integer, TYPESCRIPT_INTEGER), (uses.boolean, TYPESCRIPT_BOOLEAN), (uses.bounds, TYPESCRIPT_BOUNDS), (uses.set || uses.unique, TYPESCRIPT_CANONICAL), (uses.set, TYPESCRIPT_SET), (uses.unique, TYPESCRIPT_UNIQUE), (uses.text, TYPESCRIPT_TEXT), (uses.legacy_string, TYPESCRIPT_LEGACY_STRING), (uses.bounded_array, TYPESCRIPT_BOUNDED_ARRAY), (uses.bounded_set, TYPESCRIPT_BOUNDED_SET)] { if needed { output.push_str(fragment); } }
    for scalar in catalog.scalar_types() { typescript_scalar(output, scalar)?; }
    for union in catalog.tagged_unions() { typescript_union(output, union)?; }
    for record in catalog.records() { typescript_record(output, record)?; }
    Ok(())
}

#[rustfmt::skip]
fn rust_scalar(output: &mut String, scalar: &crate::catalog::ScalarTypeV1) -> Result<(), WireError> {
    let name = &scalar.name;
    match &scalar.definition {
        ScalarDefinitionV1::SafeInteger { minimum, maximum } => {
            output.push_str(&format!("\n#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]\n#[serde(transparent)]\npub struct {name}(i64);\n"));
            rust_collector(output, name, &format!("let value = bullet_generated_integer(value, path)?;\n    if !({minimum}..={maximum}).contains(&value) {{ return Err(bullet_generated_error(path)); }}\n    Ok({name}(value))"));
        }
        ScalarDefinitionV1::Text { minimum_utf8_bytes, maximum_utf8_bytes } => {
            output.push_str(&format!("\n#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]\n#[serde(transparent)]\npub struct {name}(String);\n"));
            rust_collector(output, name, &format!("let value = bullet_generated_string(value, path)?;\n    if !bullet_generated_valid_text(&value, {minimum_utf8_bytes}, {maximum_utf8_bytes}) {{ return Err(bullet_generated_error(path)); }}\n    Ok({name}(value))"));
        }
        ScalarDefinitionV1::Code { minimum_ascii_bytes, maximum_ascii_bytes, class } => {
            output.push_str(&format!("\n#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]\n#[serde(transparent)]\npub struct {name}(String);\n"));
            let check = rust_code_check(*class, "value");
            rust_collector(output, name, &format!("let value = bullet_generated_string(value, path)?;\n    if !(({minimum_ascii_bytes}..={maximum_ascii_bytes}).contains(&value.len()) && {check}) {{ return Err(bullet_generated_error(path)); }}\n    Ok({name}(value))"));
        }
        ScalarDefinitionV1::Enum { values } => {
            output.push_str(&format!("\n#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]\npub enum {name} {{\n"));
            for value in values {
                output.push_str(&format!("    #[serde(rename = \"{value}\")]\n    {},\n", rust_variant_identifier(value)));
            }
            output.push_str("}\n");
            rust_collector(output, name, &rust_enum_body(name, values.iter().map(String::as_str)));
        }
        ScalarDefinitionV1::TypedId { prefix } => {
            output.push_str(&format!("\n#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]\n#[serde(transparent)]\npub struct {name}(String);\n"));
            rust_collector(output, name, &format!("let value = bullet_generated_string(value, path)?;\n    let suffix = value.strip_prefix(\"{prefix}_\");\n    if !suffix.is_some_and(|suffix| suffix.len() == 64 && suffix.bytes().all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))) {{ return Err(bullet_generated_error(path)); }}\n    Ok({name}(value))"));
        }
    }
    rust_decode(output, name);
    Ok(())
}

#[rustfmt::skip]
fn typescript_scalar(output: &mut String, scalar: &crate::catalog::ScalarTypeV1) -> Result<(), WireError> {
    let name = &scalar.name;
    match &scalar.definition {
        ScalarDefinitionV1::Enum { values } => {
            output.push_str(&format!("\nexport type {name} = {};\n", values.iter().map(|value| format!("\"{value}\"")).collect::<Vec<_>>().join(" | ")));
            let cases = values.iter().map(|value| format!("case \"{value}\":" )).collect::<Vec<_>>().join(" ");
            typescript_collector(output, name, &format!("const item = bulletGeneratedString(value, path); switch (item) {{ {cases} return item; default: return bulletGeneratedFail(path); }}"));
        }
        definition => {
            let primitive = if matches!(definition, ScalarDefinitionV1::SafeInteger { .. }) { "number" } else { "string" };
            output.push_str(&format!("\ndeclare const BulletGenerated{name}Brand: unique symbol;\nexport type {name} = {primitive} & Readonly<{{ readonly [BulletGenerated{name}Brand]: true }}>;\n"));
            let body = match definition {
                ScalarDefinitionV1::SafeInteger { minimum, maximum } => format!("const item = bulletGeneratedInteger(value, path); if (item < {minimum} || item > {maximum}) bulletGeneratedFail(path); return item as {name};"),
                ScalarDefinitionV1::Text { minimum_utf8_bytes, maximum_utf8_bytes } => format!("const item = bulletGeneratedString(value, path); if (!bulletGeneratedValidText(item, {minimum_utf8_bytes}, {maximum_utf8_bytes})) bulletGeneratedFail(path); return item as {name};"),
                ScalarDefinitionV1::Code { minimum_ascii_bytes, maximum_ascii_bytes, class } => format!("const item = bulletGeneratedString(value, path); if (item.length < {minimum_ascii_bytes} || item.length > {maximum_ascii_bytes} || !bulletGeneratedExact({}, item)) bulletGeneratedFail(path); return item as {name};", typescript_code_pattern(*class)),
                ScalarDefinitionV1::TypedId { prefix } => format!("const item = bulletGeneratedString(value, path); if (!bulletGeneratedExact(/^{}_\x5b0-9a-f\x5d{{64}}$/, item)) bulletGeneratedFail(path); return item as {name};", prefix),
                ScalarDefinitionV1::Enum { .. } => return Err(impossible_field()),
            };
            typescript_collector(output, name, &body);
        }
    }
    typescript_decode(output, name);
    Ok(())
}

#[rustfmt::skip]
fn rust_record(output: &mut String, record: &ResolvedRecordV1<'_>) -> Result<(), WireError> {
    let name = &record.definition().name;
    output.push_str(&format!("\n#[derive(Clone, Debug, PartialEq, serde::Serialize)]\npub struct {name} {{\n"));
    for field in record.fields() {
        output.push_str(&format!("    pub {}: {},\n", field.definition().name, rust_field_type(field)?));
    }
    output.push_str("}\n");
    rust_record_collector(output, name, record.fields())?;
    rust_decode(output, name);
    Ok(())
}

#[rustfmt::skip]
fn typescript_record(output: &mut String, record: &ResolvedRecordV1<'_>) -> Result<(), WireError> {
    let name = &record.definition().name;
    output.push_str(&format!("\nexport type {name} = Readonly<{{\n"));
    for field in record.fields() {
        output.push_str(&format!("  readonly {}: {};\n", field.definition().name, typescript_field_type(field)?));
    }
    output.push_str("}>;\n");
    typescript_record_collector(output, name, record.fields())?;
    typescript_decode(output, name);
    Ok(())
}

#[rustfmt::skip]
fn rust_union(output: &mut String, union: &ResolvedTaggedUnionV1<'_>) -> Result<(), WireError> {
    let definition = union.definition();
    output.push_str(&format!("\n#[derive(Clone, Debug, PartialEq, serde::Serialize)]\n#[serde(tag = \"{}\")]\npub enum {} {{\n", definition.discriminator, definition.name));
    for variant in union.variants() {
        output.push_str(&format!("    #[serde(rename = \"{}\")]\n    {} {{\n", variant.definition().tag, rust_variant_identifier(&variant.definition().tag)));
        for field in variant.fields() {
            output.push_str(&format!("        {}: {},\n", field.definition().name, rust_field_type(field)?));
        }
        output.push_str("    },\n");
    }
    output.push_str("}\n");
    rust_union_collector(output, union)?;
    rust_decode(output, &definition.name);
    Ok(())
}

#[rustfmt::skip]
fn typescript_union(output: &mut String, union: &ResolvedTaggedUnionV1<'_>) -> Result<(), WireError> {
    let definition = union.definition();
    output.push_str(&format!("\nexport type {} =\n", definition.name));
    for (index, variant) in union.variants().iter().enumerate() {
        output.push_str(if index == 0 { "  " } else { "  | " });
        output.push_str(&format!("Readonly<{{ readonly {}: \"{}\";", definition.discriminator, variant.definition().tag));
        for field in variant.fields() {
            output.push_str(&format!(" readonly {}: {};", field.definition().name, typescript_field_type(field)?));
        }
        output.push_str(" }>;\n");
    }
    typescript_union_collector(output, union)?;
    typescript_decode(output, &definition.name);
    Ok(())
}

#[rustfmt::skip]
fn rust_record_collector(output: &mut String, name: &str, fields: &[ResolvedFieldV1<'_>]) -> Result<(), WireError> {
    output.push_str(&format!("\n#[allow(non_snake_case)]\nfn bullet_generated_collect_{name}(value: BulletGeneratedUniqueJsonValueV1, path: &str) -> Result<{name}, ContractValidationErrorV1> {{\n    let mut fields = bullet_generated_object(value, path)?;\n"));
    for field in sorted_fields(fields) {
        let field_name = &field.definition().name;
        output.push_str(&format!("    bullet_generated_before(&fields, \"{field_name}\", path)?;\n    let {field_name}_path = bullet_generated_path(path, \"{field_name}\");\n    let {field_name} = {{ let value = bullet_generated_take(&mut fields, \"{field_name}\", path)?; let path = &{field_name}_path; {} }};\n", rust_collect(field)?));
    }
    output.push_str("    bullet_generated_closed(fields, path)?;\n    Ok(");
    output.push_str(name);
    output.push_str(" { ");
    for field in fields {
        output.push_str(&field.definition().name);
        output.push_str(", ");
    }
    output.push_str("})\n}\n");
    Ok(())
}

#[rustfmt::skip]
fn typescript_record_collector(output: &mut String, name: &str, fields: &[ResolvedFieldV1<'_>]) -> Result<(), WireError> {
    output.push_str(&format!("\nfunction bulletGeneratedCollect{name}(value: BulletGeneratedNode, path: string): {name} {{\n  const fields = bulletGeneratedObject(value, path);\n"));
    for field in sorted_fields(fields) {
        let field_name = &field.definition().name;
        output.push_str(&format!("  bulletGeneratedBefore(fields, \"{field_name}\", path);\n  const {field_name}Path = bulletGeneratedPath(path, \"{field_name}\");\n  const {field_name} = {};\n", typescript_collect(field, &format!("bulletGeneratedTake(fields, \"{field_name}\", path)"), &format!("{field_name}Path"))?));
    }
    output.push_str("  bulletGeneratedClosed(fields, path);\n  return Object.freeze({ ");
    for field in fields {
        output.push_str(&field.definition().name);
        output.push_str(", ");
    }
    output.push_str("});\n}\n");
    Ok(())
}

#[rustfmt::skip]
fn rust_union_collector(output: &mut String, union: &ResolvedTaggedUnionV1<'_>) -> Result<(), WireError> {
    let definition = union.definition();
    let branches = union.variants().iter().enumerate().map(|(index, variant)| format!("\"{}\" => {index},", variant.definition().tag)).collect::<String>();
    output.push_str(&format!("\n#[allow(non_snake_case)]\nfn bullet_generated_collect_{}(value: BulletGeneratedUniqueJsonValueV1, path: &str) -> Result<{}, ContractValidationErrorV1> {{\n    let mut fields = bullet_generated_object(value, path)?;\n    let tag_path = bullet_generated_path(path, \"{}\");\n    let branch = match fields.get(\"{}\") {{ Some(BulletGeneratedUniqueJsonValueV1::String(tag)) => match tag.as_str() {{ {branches} _ => usize::MAX }}, _ => usize::MAX }};\n    match branch {{\n", definition.name, definition.name, definition.discriminator, definition.discriminator));
    for (index, variant) in union.variants().iter().enumerate() {
        output.push_str(&format!("        {index} => {{\n"));
        for (name, field) in union_members(&definition.discriminator, variant.fields()) {
            output.push_str(&format!("            bullet_generated_before(&fields, \"{name}\", path)?;\n"));
            if let Some(field) = field {
                output.push_str(&format!("            let {name}_path = bullet_generated_path(path, \"{name}\");\n            let {name} = {{ let value = bullet_generated_take(&mut fields, \"{name}\", path)?; let path = &{name}_path; {} }};\n", rust_collect(field)?));
            } else {
                output.push_str(&format!("            let tag = bullet_generated_string(bullet_generated_take(&mut fields, \"{name}\", path)?, &tag_path)?;\n            if tag != \"{}\" {{ return Err(bullet_generated_error(tag_path)); }}\n", variant.definition().tag));
            }
        }
        output.push_str("            bullet_generated_closed(fields, path)?;\n            Ok(");
        output.push_str(&definition.name);
        output.push_str("::");
        output.push_str(&rust_variant_identifier(&variant.definition().tag));
        output.push_str(" { ");
        for field in variant.fields() { output.push_str(&format!("{}, ", field.definition().name)); }
        output.push_str("})\n        }\n");
    }
    output.push_str(&format!("        _ => {{ bullet_generated_before(&fields, \"{}\", path)?; Err(bullet_generated_error(tag_path)) }},\n    }}\n}}\n", definition.discriminator));
    Ok(())
}

#[rustfmt::skip]
fn typescript_union_collector(output: &mut String, union: &ResolvedTaggedUnionV1<'_>) -> Result<(), WireError> {
    let definition = union.definition();
    output.push_str(&format!("\nfunction bulletGeneratedCollect{}(value: BulletGeneratedNode, path: string): {} {{\n  const fields = bulletGeneratedObject(value, path);\n  const tagPath = bulletGeneratedPath(path, \"{}\");\n  const rawTag = fields.get(\"{}\");\n  const tag = rawTag !== undefined && typeof rawTag.value === \"string\" ? rawTag.value : \"\";\n  switch (tag) {{\n", definition.name, definition.name, definition.discriminator, definition.discriminator));
    for variant in union.variants() {
        output.push_str(&format!("    case \"{}\": {{\n", variant.definition().tag));
        for (name, field) in union_members(&definition.discriminator, variant.fields()) {
            output.push_str(&format!("      bulletGeneratedBefore(fields, \"{name}\", path);\n"));
            if let Some(field) = field {
                output.push_str(&format!("      const {name}Path = bulletGeneratedPath(path, \"{name}\");\n      const {name} = {};\n", typescript_collect(field, &format!("bulletGeneratedTake(fields, \"{name}\", path)"), &format!("{name}Path"))?));
            } else {
                output.push_str(&format!("      if (bulletGeneratedString(bulletGeneratedTake(fields, \"{name}\", path), tagPath) !== \"{}\") bulletGeneratedFail(tagPath);\n", variant.definition().tag));
            }
        }
        output.push_str(&format!("      bulletGeneratedClosed(fields, path);\n      return Object.freeze({{ {}: \"{}\", ", definition.discriminator, variant.definition().tag));
        for field in variant.fields() { output.push_str(&format!("{}, ", field.definition().name)); }
        output.push_str("});\n    }\n");
    }
    output.push_str(&format!("    default: bulletGeneratedBefore(fields, \"{}\", path); return bulletGeneratedFail(tagPath);\n  }}\n}}\n", definition.discriminator));
    Ok(())
}

fn rust_collector(output: &mut String, name: &str, body: &str) {
    output.push_str(&format!("\n#[allow(non_snake_case)]\nfn bullet_generated_collect_{name}(value: BulletGeneratedUniqueJsonValueV1, path: &str) -> Result<{name}, ContractValidationErrorV1> {{\n    {body}\n}}\n"));
}

fn typescript_collector(output: &mut String, name: &str, body: &str) {
    output.push_str(&format!("\nfunction bulletGeneratedCollect{name}(value: BulletGeneratedNode, path: string): {name} {{ {body} }}\n"));
}

fn rust_decode(output: &mut String, name: &str) {
    output.push_str(&format!("\nimpl {name} {{\n    pub fn decode_bytes(bytes: &[u8]) -> Result<Self, ContractValidationErrorV1> {{ bullet_generated_collect_{name}(bullet_generated_decode_unique(bytes)?, \"\") }}\n    pub fn decode_str(text: &str) -> Result<Self, ContractValidationErrorV1> {{ Self::decode_bytes(text.as_bytes()) }}\n}}\n"));
}

fn typescript_decode(output: &mut String, name: &str) {
    output.push_str(&format!("\nexport function decode{name}Bytes(bytes: Uint8Array): ContractDecodeResultV1<{name}> {{ return bulletGeneratedResult(() => bulletGeneratedCollect{name}(BulletGeneratedParser.bytes(bytes), \"\")); }}\nexport function decode{name}Text(text: string): ContractDecodeResultV1<{name}> {{ return bulletGeneratedResult(() => {{ const result = decode{name}Bytes(bulletGeneratedEncodeUtf8(text)); if (!result.ok) bulletGeneratedFail(result.error.path); return result.value; }}); }}\n"));
}

#[rustfmt::skip]
fn rust_field_type(field: &ResolvedFieldV1<'_>) -> Result<String, WireError> {
    Ok(match field.kind() {
        ResolvedFieldKindV1::LegacyValue if field.definition().field_type == FieldTypeV1::SchemaVersion => "SchemaVersionLiteralV1".into(),
        ResolvedFieldKindV1::LegacyValue | ResolvedFieldKindV1::LegacyReference { .. } => super::rust_type(field)?,
        ResolvedFieldKindV1::Named(target) => { ensure_new(field, FieldTypeV1::NamedRef)?; target_name(target).into() }
        ResolvedFieldKindV1::OptionalNamed(target) => { ensure_new(field, FieldTypeV1::OptionalNamedRef)?; format!("Option<{}>", target_name(target)) }
        ResolvedFieldKindV1::BoundedArray { target, bounds } => { ensure_new(field, FieldTypeV1::BoundedArray)?; format!("BoundedArrayV1<{}, {}, {}>", target_name(target), bounds.min_items, bounds.max_items) }
        ResolvedFieldKindV1::BoundedSet { target, bounds } => { ensure_new(field, FieldTypeV1::BoundedSet)?; format!("BoundedSetV1<{}, {}, {}>", target_name(target), bounds.min_items, bounds.max_items) }
    })
}

#[rustfmt::skip]
fn typescript_field_type(field: &ResolvedFieldV1<'_>) -> Result<String, WireError> {
    Ok(match field.kind() {
        ResolvedFieldKindV1::LegacyValue if field.definition().field_type == FieldTypeV1::SchemaVersion => "SchemaVersionLiteralV1".into(),
        ResolvedFieldKindV1::LegacyValue => match super::strict_legacy_value(field.definition().field_type) {
            StrictLegacyValue::StringArray(..) => "ReadonlyArray<string>".into(),
            StrictLegacyValue::EnumArray(name, _) => format!("ReadonlyArray<{name}>") ,
            _ => super::typescript_type(field)?,
        },
        ResolvedFieldKindV1::LegacyReference { shape, target } => {
            super::ensure_reference_shape(field.definition().field_type, shape)?;
            match shape { LegacyReferenceShapeV1::Direct => target_name(target).into(), LegacyReferenceShapeV1::Optional => format!("{} | null", target_name(target)), LegacyReferenceShapeV1::Array => format!("ReadonlyArray<{}>", target_name(target)) }
        }
        ResolvedFieldKindV1::Named(target) => { ensure_new(field, FieldTypeV1::NamedRef)?; target_name(target).into() }
        ResolvedFieldKindV1::OptionalNamed(target) => { ensure_new(field, FieldTypeV1::OptionalNamedRef)?; format!("{} | null", target_name(target)) }
        ResolvedFieldKindV1::BoundedArray { target, bounds } => { ensure_new(field, FieldTypeV1::BoundedArray)?; format!("BoundedArrayV1<{}, {}, {}>", target_name(target), bounds.min_items, bounds.max_items) }
        ResolvedFieldKindV1::BoundedSet { target, bounds } => { ensure_new(field, FieldTypeV1::BoundedSet)?; format!("BoundedSetV1<{}, {}, {}>", target_name(target), bounds.min_items, bounds.max_items) }
    })
}

#[rustfmt::skip]
fn rust_collect(field: &ResolvedFieldV1<'_>) -> Result<String, WireError> {
    Ok(match field.kind() {
        ResolvedFieldKindV1::LegacyValue => super::rust_strict_legacy_collect(field.definition().field_type)?,
        ResolvedFieldKindV1::LegacyReference { shape, target } => {
            super::ensure_reference_shape(field.definition().field_type, shape)?;
            let item = format!("bullet_generated_collect_{}(value, path)?", target_name(target));
            match shape { LegacyReferenceShapeV1::Direct => item, LegacyReferenceShapeV1::Optional => format!("match value {{ BulletGeneratedUniqueJsonValueV1::Null => None, value => Some({item}) }}"), LegacyReferenceShapeV1::Array if field.definition().field_type == FieldTypeV1::ExecutionToolArray => format!("{{ let nodes = bullet_generated_array(value, path)?; bullet_generated_cardinality(nodes.len(), 1, 64, path)?; let duplicate = bullet_generated_duplicate_failure(&nodes, path)?; let values = match bullet_generated_collect_nodes(nodes, path, |value, path| bullet_generated_collect_{}(value, path)) {{ Ok(values) => values, Err(error) => return Err(bullet_generated_choose(error, duplicate)) }}; if let Some(error) = duplicate {{ return Err(error); }} values }}", target_name(target)), LegacyReferenceShapeV1::Array => format!("bullet_generated_collect_array(value, path, |value, path| bullet_generated_collect_{}(value, path))?", target_name(target)) }
        }
        ResolvedFieldKindV1::Named(target) => { ensure_new(field, FieldTypeV1::NamedRef)?; format!("bullet_generated_collect_{}(value, path)?", target_name(target)) }
        ResolvedFieldKindV1::OptionalNamed(target) => { ensure_new(field, FieldTypeV1::OptionalNamedRef)?; format!("match value {{ BulletGeneratedUniqueJsonValueV1::Null => None, value => Some(bullet_generated_collect_{}(value, path)?) }}", target_name(target)) }
        ResolvedFieldKindV1::BoundedArray { target, bounds } => { ensure_new(field, FieldTypeV1::BoundedArray)?; format!("{{ let nodes = bullet_generated_array(value, path)?; bullet_generated_cardinality(nodes.len(), {}, {}, path)?; let values = bullet_generated_collect_nodes(nodes, path, |value, path| bullet_generated_collect_{}(value, path))?; BoundedArrayV1::<{}, {}, {}>::try_new(values).map_err(|error| bullet_generated_rebase(error, path))? }}", bounds.min_items, bounds.max_items, target_name(target), target_name(target), bounds.min_items, bounds.max_items) }
        ResolvedFieldKindV1::BoundedSet { target, bounds } => { ensure_new(field, FieldTypeV1::BoundedSet)?; format!("{{ let nodes = bullet_generated_array(value, path)?; bullet_generated_cardinality(nodes.len(), {}, {}, path)?; let order = bullet_generated_set_failure(&nodes, path)?; let values = match bullet_generated_collect_nodes(nodes, path, |value, path| bullet_generated_collect_{}(value, path)) {{ Ok(values) => values, Err(error) => return Err(bullet_generated_choose(error, order)) }}; if let Some(error) = order {{ return Err(error); }} BoundedSetV1::<{}, {}, {}>::try_new(values).map_err(|error| bullet_generated_rebase(error, path))? }}", bounds.min_items, bounds.max_items, target_name(target), target_name(target), bounds.min_items, bounds.max_items) }
    })
}

#[rustfmt::skip]
fn typescript_collect(field: &ResolvedFieldV1<'_>, value: &str, path: &str) -> Result<String, WireError> {
    Ok(match field.kind() {
        ResolvedFieldKindV1::LegacyValue => super::typescript_strict_legacy_collect(field.definition().field_type, value, path)?,
        ResolvedFieldKindV1::LegacyReference { shape, target } => {
            super::ensure_reference_shape(field.definition().field_type, shape)?;
            let collect = format!("bulletGeneratedCollect{}(item, itemPath)", target_name(target));
            match shape { LegacyReferenceShapeV1::Direct => format!("bulletGeneratedCollect{}({value}, {path})", target_name(target)), LegacyReferenceShapeV1::Optional => format!("{value}.value === null ? null : bulletGeneratedCollect{}({value}, {path})", target_name(target)), LegacyReferenceShapeV1::Array if field.definition().field_type == FieldTypeV1::ExecutionToolArray => format!("(() => {{ const nodes = bulletGeneratedArrayNode({value}, {path}); bulletGeneratedCardinality(nodes.length, 1, 64, {path}); const duplicate = bulletGeneratedDuplicateFailure(nodes, {path}); try {{ const values = bulletGeneratedCollectNodes(nodes, {path}, (item, itemPath) => {collect}); if (duplicate !== null) bulletGeneratedFail(duplicate); return values; }} catch (error) {{ if (error instanceof BulletGeneratedFailure && duplicate !== null && bulletGeneratedCompareUtf8(duplicate, error.path) < 0) bulletGeneratedFail(duplicate); throw error; }} }})()"), LegacyReferenceShapeV1::Array => format!("bulletGeneratedCollectArray({value}, {path}, (item, itemPath) => {collect})") }
        }
        ResolvedFieldKindV1::Named(target) => { ensure_new(field, FieldTypeV1::NamedRef)?; format!("bulletGeneratedCollect{}({value}, {path})", target_name(target)) }
        ResolvedFieldKindV1::OptionalNamed(target) => { ensure_new(field, FieldTypeV1::OptionalNamedRef)?; format!("{value}.value === null ? null : bulletGeneratedCollect{}({value}, {path})", target_name(target)) }
        ResolvedFieldKindV1::BoundedArray { target, bounds } => { ensure_new(field, FieldTypeV1::BoundedArray)?; format!("(() => {{ const nodes = bulletGeneratedArrayNode({value}, {path}); bulletGeneratedCardinality(nodes.length, {}, {}, {path}); return bulletGeneratedArray(bulletGeneratedCollectNodes(nodes, {path}, (item, itemPath) => bulletGeneratedCollect{}(item, itemPath)).slice(), {}, {}, {path}); }})()", bounds.min_items, bounds.max_items, target_name(target), bounds.min_items, bounds.max_items) }
        ResolvedFieldKindV1::BoundedSet { target, bounds } => { ensure_new(field, FieldTypeV1::BoundedSet)?; format!("(() => {{ const nodes = bulletGeneratedArrayNode({value}, {path}); bulletGeneratedCardinality(nodes.length, {}, {}, {path}); const keys = nodes.map(bulletGeneratedCanonical); const order = bulletGeneratedSetFailure(keys, {path}); try {{ const values = bulletGeneratedCollectNodes(nodes, {path}, (item, itemPath) => bulletGeneratedCollect{}(item, itemPath)).slice(); if (order !== null) bulletGeneratedFail(order); return bulletGeneratedSet(values, keys, {}, {}, {path}); }} catch (error) {{ if (error instanceof BulletGeneratedFailure && order !== null && bulletGeneratedCompareUtf8(order, error.path) < 0) bulletGeneratedFail(order); throw error; }} }})()", bounds.min_items, bounds.max_items, target_name(target), bounds.min_items, bounds.max_items) }
    })
}

#[rustfmt::skip]
fn sorted_fields<'scope, 'catalog>(fields: &'scope [ResolvedFieldV1<'catalog>]) -> Vec<&'scope ResolvedFieldV1<'catalog>> {
    let mut fields = fields.iter().collect::<Vec<_>>(); fields.sort_by_key(|field| field.definition().name.as_str());
    fields
}

#[rustfmt::skip]
fn union_members<'scope, 'catalog>(discriminator: &'scope str, fields: &'scope [ResolvedFieldV1<'catalog>]) -> Vec<(&'scope str, Option<&'scope ResolvedFieldV1<'catalog>>)> {
    let mut members = fields.iter().map(|field| (field.definition().name.as_str(), Some(field))).collect::<Vec<_>>();
    members.push((discriminator, None));
    members.sort_by_key(|(name, _)| *name);
    members
}
#[rustfmt::skip]
fn ensure_new(field: &ResolvedFieldV1<'_>, expected: FieldTypeV1) -> Result<(), WireError> {
    if field.definition().field_type == expected { Ok(()) } else { Err(impossible_field()) }
}

#[rustfmt::skip]
fn target_name(target: ResolvedSymbolV1<'_>) -> &str {
    match target.kind() {
        ResolvedSymbolKindV1::Scalar | ResolvedSymbolKindV1::Record | ResolvedSymbolKindV1::TaggedUnion => target.name(),
    }
}

#[rustfmt::skip]
fn rust_enum_body<'a>(name: &str, values: impl Iterator<Item = &'a str>) -> String {
    format!("let value = bullet_generated_string(value, path)?;\n    match value.as_str() {{ {} _ => Err(bullet_generated_error(path)), }}", values.map(|value| format!("\"{value}\" => Ok({name}::{}),", rust_variant_identifier(value))).collect::<String>())
}

#[rustfmt::skip]
fn rust_code_check(class: CodeClassV1, value: &str) -> String {
    match class {
        CodeClassV1::LowerKebab => format!("{value}.as_bytes().first().is_some_and(u8::is_ascii_lowercase) && {value}.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-') && !{value}.ends_with('-') && !{value}.contains(\"--\")"),
        CodeClassV1::UpperHyphen => format!("{value}.as_bytes().first().is_some_and(u8::is_ascii_uppercase) && {value}.bytes().all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-') && !{value}.ends_with('-') && !{value}.contains(\"--\")"),
        CodeClassV1::AsciiToken => format!("{value}.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric) && {value}.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'/' | b'+' | b'-'))"),
        CodeClassV1::InvariantId => format!("{value}.strip_prefix(\"BF-\").is_some_and(|tail| !tail.is_empty() && tail.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric) && tail.bytes().all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-') && !tail.ends_with('-') && !tail.contains(\"--\"))"),
    }
}

fn typescript_code_pattern(class: CodeClassV1) -> &'static str {
    match class {
        CodeClassV1::LowerKebab => "/^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$/",
        CodeClassV1::UpperHyphen => "/^[A-Z][A-Z0-9]*(?:-[A-Z0-9]+)*$/",
        CodeClassV1::AsciiToken => "/^[A-Za-z0-9][A-Za-z0-9._:/+\\-]*$/",
        CodeClassV1::InvariantId => "/^BF-[A-Z0-9]+(?:-[A-Z0-9]+)*$/",
    }
}
