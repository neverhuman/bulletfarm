use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};

use crate::{WireError, canonical_json, policy::POLICY_SCHEMA_VERSION};

use super::{
    ContractCatalogV1, ContractFieldV1, ContractRecordV1, FieldTypeV1, LegacyReferenceShapeV1,
    RecordShapeV1, ResolvedCatalogV1, ResolvedFieldKindV1, ResolvedFieldV1, ResolvedRecordV1,
    ResolvedSymbolKindV1, ResolvedSymbolV1, ResolvedTaggedUnionV1, ResolvedUnionVariantV1,
    ScalarDefinitionV1, ScalarTypeV1, TaggedUnionV1, records::required_records,
    rust_variant_identifier,
};

#[cfg(test)]
mod tests;

#[rustfmt::skip]
#[derive(Eq, Ord, PartialEq, PartialOrd)]
struct Failure { class: u8, owner: String, field: String, detail: String, code: &'static str }

#[derive(Default)]
struct Failures(Vec<Failure>);

#[rustfmt::skip]
impl Failures {
    fn add(&mut self, class: u8, code: &'static str, owner: impl Into<String>,
        field: impl Into<String>, detail: impl Into<String>) {
        self.0.push(Failure { class, owner: owner.into(), field: field.into(),
            detail: detail.into(), code });
    }

    fn into_error(mut self) -> Option<WireError> {
        self.0.sort();
        self.0.first().map(|failure| {
            WireError::new(
                failure.code,
                format!("{}/{}/{}", failure.owner, failure.field, failure.detail),
            )
        })
    }
}

#[rustfmt::skip]
#[derive(Clone, Copy)]
enum ResolutionMode { Production, #[cfg(test)] TestStrict }

pub(super) fn resolve(catalog: &ContractCatalogV1) -> Result<ResolvedCatalogV1<'_>, WireError> {
    resolve_mode(catalog, ResolutionMode::Production)
}

#[cfg(test)]
#[rustfmt::skip]
pub(super) fn resolve_test_strict(catalog: &ContractCatalogV1) -> Result<ResolvedCatalogV1<'_>, WireError> {
    resolve_mode(catalog, ResolutionMode::TestStrict)
}

#[rustfmt::skip]
fn resolve_mode(catalog: &ContractCatalogV1, mode: ResolutionMode)
    -> Result<ResolvedCatalogV1<'_>, WireError> {
    let mut failures = Failures::default();
    let (expected_version, test_strict) = match mode {
        ResolutionMode::Production => ("v1alpha1.0", false),
        #[cfg(test)] ResolutionMode::TestStrict => ("test.strict.v1", true),
    };
    if catalog.schema_version != POLICY_SCHEMA_VERSION
        || catalog.catalog_version != expected_version
        || catalog.scalar_types.len() > 256
        || catalog.tagged_unions.len() > 128
        || catalog.records.is_empty()
        || catalog.records.len() > 512
    {
        return Err(WireError::new("INVALID_CONTRACT_CATALOG", "root or container"));
    }
    let mut record_failures = Failures::default();
    for record in &catalog.records {
        if !valid_type_name(&record.name) || record.unknown_fields != "reject" || record.fields.is_empty() || record.fields.len() > 256 { record_failures.add(2, "INVALID_CONTRACT_RECORD", &record.name, "", "record metadata"); }
    }
    if let Some(error) = record_failures.into_error() { return Err(error); }

    let mut symbols = BTreeMap::new();
    for scalar in &catalog.scalar_types {
        declare_symbol(
            &mut symbols,
            ResolvedSymbolV1::Scalar(scalar),
            7,
            &mut failures,
        );
        validate_scalar(scalar, &mut failures);
    }
    let mut record_names = BTreeSet::new();
    for record in &catalog.records {
        if !record_names.insert(record.name.as_str()) { failures.add(3, "DUPLICATE_CONTRACT_RECORD", &record.name, "", "duplicate record"); }
        declare_symbol(
            &mut symbols,
            ResolvedSymbolV1::Record(record),
            3,
            &mut failures,
        );
    }
    for union in &catalog.tagged_unions {
        declare_symbol(
            &mut symbols,
            ResolvedSymbolV1::TaggedUnion(union),
            7,
            &mut failures,
        );
    }
    validate_declaration_order(catalog, test_strict, &mut failures);

    let mut adjacency = symbols
        .keys()
        .map(|name| (*name, Vec::new()))
        .collect::<BTreeMap<_, _>>();
    let mut resolved_records = Vec::with_capacity(catalog.records.len());
    for record in &catalog.records {
        validate_record(record, test_strict, &mut failures);
        let mut fields = Vec::with_capacity(record.fields.len());
        for field in &record.fields {
            if test_strict && matches!(field.field_type, FieldTypeV1::Object | FieldTypeV1::ObjectArray)
            {
                failures.add(11, "OPEN_CONTRACT_FIELD", &record.name, &field.name, "open type");
            }
            if let Some(kind) = resolve_field(record, field, &symbols, &mut failures) {
                if let Some(target) = field_target(kind)
                    && !matches!(target.kind(), ResolvedSymbolKindV1::Scalar)
                {
                    adjacency.entry(&record.name).or_default().push(target.name());
                }
                fields.push(ResolvedFieldV1 {
                    definition: field,
                    kind,
                });
            }
        }
        resolved_records.push(ResolvedRecordV1 {
            definition: record,
            fields,
        });
    }

    let mut resolved_unions = Vec::with_capacity(catalog.tagged_unions.len());
    for union in &catalog.tagged_unions {
        let variants = resolve_union(
            union,
            &symbols,
            &resolved_records,
            &mut adjacency,
            &mut failures,
        );
        resolved_unions.push(ResolvedTaggedUnionV1 {
            definition: union,
            variants,
        });
    }
    for edges in adjacency.values_mut() {
        edges.sort_unstable();
        edges.dedup();
    }
    collect_cycles(&adjacency, &mut failures);

    if !test_strict {
        let names = catalog
            .records
            .iter()
            .map(|record| record.name.as_str())
            .collect::<BTreeSet<_>>();
        if names != required_records() {
            failures.add(12, "CONTRACT_CATALOG_COVERAGE", "", "", "record set");
        } else if !legacy_schema_fields_match(catalog) {
            failures.add(6, "INVALID_CONTRACT_RECORD_SHAPE", "", "schema_version", "legacy assignment");
        }
    }
    if let Some(error) = failures.into_error() {
        return Err(error);
    }
    Ok(ResolvedCatalogV1 {
        definition: catalog,
        scalar_types: catalog.scalar_types.iter().collect(),
        records: resolved_records,
        tagged_unions: resolved_unions,
        #[cfg(test)]
        symbols,
        #[cfg(test)]
        adjacency,
    })
}

#[rustfmt::skip]
fn declare_symbol<'a>(symbols: &mut BTreeMap<&'a str, ResolvedSymbolV1<'a>>,
    symbol: ResolvedSymbolV1<'a>, same_class: u8, failures: &mut Failures) {
    let name = symbol.name();
    let code = if same_class == 3 { "DUPLICATE_CONTRACT_RECORD" } else { "INVALID_CONTRACT_FIELD_REFERENCE" };
    if !valid_type_name(name) {
        failures.add(same_class.min(7), code, name, "", "invalid type name");
    } else if generator_reserved(name) {
        failures.add(7, "INVALID_CONTRACT_FIELD_REFERENCE", name, "", "reserved type");
    }
    if let Some(previous) = symbols.get(name) {
        let class = if previous.kind() == ResolvedSymbolKindV1::Record && symbol.kind() == ResolvedSymbolKindV1::Record { 3 } else { 7 };
        let code = if class == 3 { "DUPLICATE_CONTRACT_RECORD" } else { "INVALID_CONTRACT_FIELD_REFERENCE" };
        failures.add(class, code, name, "", "duplicate symbol");
    } else if valid_type_name(name) && !generator_reserved(name) {
        symbols.insert(name, symbol);
    }
}

#[rustfmt::skip]
fn validate_declaration_order(catalog: &ContractCatalogV1, strict: bool, failures: &mut Failures) {
    let mut groups = vec![
        catalog.scalar_types.iter().map(|item| item.name.as_str()).collect::<Vec<_>>(),
        catalog.tagged_unions.iter().map(|item| item.name.as_str()).collect(),
    ];
    groups.push(catalog.records.iter().map(|item| item.name.as_str()).collect());
    for names in groups {
        for pair in names.windows(2) {
            let reviewed_legacy_inversion = !strict && pair == ["ReviewerAssignment", "ReviewReceipt"];
            if pair[0] >= pair[1] && !reviewed_legacy_inversion {
                failures.add(7, "INVALID_CONTRACT_FIELD_REFERENCE", pair[1], "", "declaration order");
            }
        }
    }
}

#[rustfmt::skip]
fn validate_record(record: &ContractRecordV1, strict: bool, failures: &mut Failures) {
    let mut names = BTreeSet::new();
    for field in &record.fields {
        if !valid_field_name(&field.name) || !names.insert(field.name.as_str()) {
            failures.add(4, "INVALID_CONTRACT_FIELD", &record.name, &field.name, "name or duplicate");
        }
    }
    let schemas = record
        .fields
        .iter()
        .filter(|field| field.name == "schema_version")
        .collect::<Vec<_>>();
    let schema = schemas.first().copied();
    match record.shape {
        RecordShapeV1::Versioned if schema.is_none() => {
            failures.add(5, "MISSING_SCHEMA_VERSION", &record.name, "schema_version", "missing");
        }
        RecordShapeV1::Versioned
            if schemas.len() != 1
                || schema.is_some_and(|field| {
                    !matches!((strict, field.field_type), (true, FieldTypeV1::SchemaVersion) | (false, FieldTypeV1::String | FieldTypeV1::SchemaVersion | FieldTypeV1::PolicySchemaVersion))
                        || field.target.is_some()
                        || field.bounds.is_some()
                }) =>
        {
            failures.add(6, "INVALID_CONTRACT_RECORD_SHAPE", &record.name, "schema_version", "versioned");
        }
        RecordShapeV1::Embedded if !schemas.is_empty() => {
            failures.add(6, "INVALID_CONTRACT_RECORD_SHAPE", &record.name, "schema_version", "embedded");
        }
        _ => {}
    }
}

#[rustfmt::skip]
fn legacy_schema_fields_match(catalog: &ContractCatalogV1) -> bool {
    let mut rows = catalog.records.iter().filter_map(|record| record.fields.iter().find(|field| field.name == "schema_version").map(|field| (record.name.as_str(), field.field_type))).collect::<Vec<_>>();
    rows.sort_unstable_by(|left, right| left.0.cmp(right.0));
    canonical_json(&rows).is_ok_and(|bytes| format!("{:x}", Sha256::digest(bytes)) == LEGACY_SCHEMA_FIELDS_SHA256)
}

#[rustfmt::skip]
fn validate_scalar(scalar: &ScalarTypeV1, failures: &mut Failures) {
    if !valid_type_name(&scalar.name) || generator_reserved(&scalar.name) {
        failures.add(7, "INVALID_CONTRACT_FIELD_REFERENCE", &scalar.name, "", "scalar name");
    }
    match &scalar.definition {
        ScalarDefinitionV1::SafeInteger { minimum, maximum }
            if minimum > maximum
                || *minimum < -9_007_199_254_740_991
                || *maximum > 9_007_199_254_740_991 =>
        {
            failures.add(8, "INVALID_CONTRACT_FIELD_BOUNDS", &scalar.name, "", "integer bounds");
        }
        ScalarDefinitionV1::Text { minimum_utf8_bytes, maximum_utf8_bytes }
            if *minimum_utf8_bytes == 0
                || minimum_utf8_bytes > maximum_utf8_bytes
                || *maximum_utf8_bytes > 8_388_608 =>
        {
            failures.add(8, "INVALID_CONTRACT_FIELD_BOUNDS", &scalar.name, "", "text bounds");
        }
        ScalarDefinitionV1::Code { minimum_ascii_bytes, maximum_ascii_bytes, .. }
            if *minimum_ascii_bytes == 0
                || minimum_ascii_bytes > maximum_ascii_bytes
                || *maximum_ascii_bytes > 256 =>
        {
            failures.add(8, "INVALID_CONTRACT_FIELD_BOUNDS", &scalar.name, "", "code bounds");
        }
        ScalarDefinitionV1::Enum { values } => validate_literals(&scalar.name, values, 7, failures),
        ScalarDefinitionV1::TypedId { prefix } if !valid_id_prefix(prefix) => {
            failures.add(7, "INVALID_CONTRACT_FIELD_REFERENCE", &scalar.name, "", "typed-id prefix");
        }
        _ => {}
    }
}

#[rustfmt::skip]
fn resolve_field<'a>(
    record: &ContractRecordV1,
    field: &'a ContractFieldV1,
    symbols: &BTreeMap<&'a str, ResolvedSymbolV1<'a>>,
    failures: &mut Failures,
) -> Option<ResolvedFieldKindV1<'a>> {
    let owner = &record.name;
    let legacy = legacy_reference(field.field_type);
    if !matches!(field.field_type, FieldTypeV1::NamedRef | FieldTypeV1::OptionalNamedRef | FieldTypeV1::BoundedArray | FieldTypeV1::BoundedSet) {
        if field.target.is_some() || field.bounds.is_some() {
            failures.add(7, "INVALID_CONTRACT_FIELD_REFERENCE", owner, &field.name, "legacy metadata");
        }
        return match legacy {
            Some((name, shape)) => symbol(name, owner, field, symbols, failures).map(|target| {
                ResolvedFieldKindV1::LegacyReference { shape, target }
            }),
            None => Some(ResolvedFieldKindV1::LegacyValue),
        };
    }
    let needs_bounds = matches!(field.field_type, FieldTypeV1::BoundedArray | FieldTypeV1::BoundedSet);
    if field.target.is_none() || field.bounds.is_some() != needs_bounds {
        failures.add(7, "INVALID_CONTRACT_FIELD_REFERENCE", owner, &field.name, "target or bounds metadata");
    }
    let target_name = field.target.as_deref().map_or("", |name| name);
    let target = symbol(target_name, owner, field, symbols, failures)?;
    match field.field_type {
        FieldTypeV1::NamedRef => Some(ResolvedFieldKindV1::Named(target)),
        FieldTypeV1::OptionalNamedRef => Some(ResolvedFieldKindV1::OptionalNamed(target)),
        FieldTypeV1::BoundedArray | FieldTypeV1::BoundedSet => field.bounds.map(|bounds| {
            if bounds.max_items == 0 || bounds.max_items > 4096 || bounds.min_items > bounds.max_items {
                failures.add(8, "INVALID_CONTRACT_FIELD_BOUNDS", owner, &field.name, "collection bounds");
            }
            if field.field_type == FieldTypeV1::BoundedArray {
                ResolvedFieldKindV1::BoundedArray { target, bounds }
            } else {
                ResolvedFieldKindV1::BoundedSet { target, bounds }
            }
        }),
        _ => None,
    }
}

#[rustfmt::skip]
fn symbol<'a>(
    name: &str,
    owner: &str,
    field: &ContractFieldV1,
    symbols: &BTreeMap<&'a str, ResolvedSymbolV1<'a>>,
    failures: &mut Failures,
) -> Option<ResolvedSymbolV1<'a>> {
    let value = symbols.get(name).copied();
    if value.is_none() {
        failures.add(7, "INVALID_CONTRACT_FIELD_REFERENCE", owner, &field.name, "unknown target");
    }
    value
}

#[rustfmt::skip]
fn field_target(kind: ResolvedFieldKindV1<'_>) -> Option<ResolvedSymbolV1<'_>> {
    match kind {
        ResolvedFieldKindV1::LegacyValue => None,
        ResolvedFieldKindV1::LegacyReference { target, .. }
        | ResolvedFieldKindV1::Named(target)
        | ResolvedFieldKindV1::OptionalNamed(target)
        | ResolvedFieldKindV1::BoundedArray { target, .. }
        | ResolvedFieldKindV1::BoundedSet { target, .. } => Some(target),
    }
}

#[rustfmt::skip]
fn resolve_union<'a>(
    union: &'a TaggedUnionV1,
    symbols: &BTreeMap<&'a str, ResolvedSymbolV1<'a>>,
    records: &[ResolvedRecordV1<'a>],
    adjacency: &mut BTreeMap<&'a str, Vec<&'a str>>,
    failures: &mut Failures,
) -> Vec<ResolvedUnionVariantV1<'a>> {
    let oversized = union.variants.len() > 32;
    if !valid_type_name(&union.name) || generator_reserved(&union.name) || !(2..=32).contains(&union.variants.len()) || !valid_field_name(&union.discriminator) {
        failures.add(9, "INVALID_CONTRACT_TAGGED_UNION", &union.name, "", "union metadata");
    }
    if oversized { return Vec::new(); }
    let tags = union.variants.iter().map(|item| item.tag.clone()).collect::<Vec<_>>();
    validate_literals(&union.name, &tags, 9, failures);
    let mut variant_records = BTreeSet::new();
    let mut resolved = Vec::new();
    for variant in &union.variants {
        let value = symbols.get(variant.record.as_str()).copied();
        if !variant_records.insert(variant.record.as_str()) {
            failures.add(9, "INVALID_CONTRACT_TAGGED_UNION", &union.name, &variant.tag, "duplicate record");
        }
        match value {
            Some(ResolvedSymbolV1::Record(record)) if record.shape == RecordShapeV1::Embedded && !record.fields.iter().any(|field| field.name == union.discriminator) => {
                adjacency.entry(&union.name).or_default().push(&record.name);
                if let Some(item) = records.iter().find(|item| item.definition.name == record.name) {
                    resolved.push(ResolvedUnionVariantV1 {
                        definition: variant,
                        record,
                        fields: item.fields.clone(),
                    });
                }
            }
            _ => failures.add(9, "INVALID_CONTRACT_TAGGED_UNION", &union.name, &variant.tag, "variant record"),
        }
    }
    resolved
}

#[rustfmt::skip]
fn validate_literals(owner: &str, values: &[String], class: u8, failures: &mut Failures) {
    if values.is_empty() || values.len() > 256 {
        failures.add(if class == 7 { 8 } else { class }, if class == 7 { "INVALID_CONTRACT_FIELD_BOUNDS" } else { "INVALID_CONTRACT_TAGGED_UNION" }, owner, "", "literal count");
    }
    let mut variants = BTreeSet::new();
    for (index, value) in values.iter().enumerate() {
        let transformed = rust_variant_identifier(value);
        if !valid_literal(value) || (index > 0 && values[index - 1] >= *value) || transformed == "Self" || !variants.insert(transformed) {
            failures.add(class, if class == 9 { "INVALID_CONTRACT_TAGGED_UNION" } else { "INVALID_CONTRACT_FIELD_REFERENCE" }, owner, index.to_string(), "literal or transform");
        }
    }
}

#[rustfmt::skip]
fn collect_cycles<'a>(graph: &BTreeMap<&'a str, Vec<&'a str>>, failures: &mut Failures) {
    fn visit<'a>(node: &'a str, graph: &BTreeMap<&'a str, Vec<&'a str>>, colors: &mut BTreeMap<&'a str, u8>, failures: &mut Failures) {
        colors.insert(node, 1);
        if let Some(edges) = graph.get(node) {
            for edge in edges {
                match colors.get(edge).copied().map_or(0, |color| color) {
                    0 => visit(edge, graph, colors, failures),
                    1 => failures.add(10, "CONTRACT_TYPE_CYCLE", node, *edge, "cycle_edge"),
                    _ => {}
                }
            }
        }
        colors.insert(node, 2);
    }
    let mut colors = BTreeMap::new();
    for node in graph.keys().copied() {
        if colors.get(node).copied().map_or(0, |color| color) == 0 {
            visit(node, graph, &mut colors, failures);
        }
    }
}

#[rustfmt::skip]
fn valid_type_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    !value.is_empty() && value.len() <= 80 && bytes.next().is_some_and(|byte| byte.is_ascii_uppercase()) && bytes.all(|byte| byte.is_ascii_alphanumeric())
}

#[rustfmt::skip]
fn valid_field_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    !value.is_empty() && value.len() <= 80 && bytes.next().is_some_and(|byte| byte.is_ascii_lowercase()) && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_') && !RESERVED_FIELDS.split_ascii_whitespace().any(|word| word == value)
}

#[rustfmt::skip]
fn valid_literal(value: &str) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty() && bytes.len() <= 64 && bytes[0].is_ascii_alphabetic() && bytes[bytes.len() - 1].is_ascii_alphanumeric() && bytes.iter().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')) && !bytes.windows(2).any(|pair| !pair[0].is_ascii_alphanumeric() && !pair[1].is_ascii_alphanumeric())
}

#[rustfmt::skip]
fn valid_id_prefix(value: &str) -> bool {
    let mut bytes = value.bytes();
    (2..=16).contains(&value.len()) && bytes.next().is_some_and(|byte| byte.is_ascii_lowercase()) && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

#[rustfmt::skip]
fn generator_reserved(value: &str) -> bool {
    value.starts_with("BulletGenerated") || GENERATOR_RESERVED.split_ascii_whitespace().any(|word| word == value)
}

#[rustfmt::skip]
fn legacy_reference(field_type: FieldTypeV1) -> Option<(&'static str, LegacyReferenceShapeV1)> {
    use FieldTypeV1::*;
    use LegacyReferenceShapeV1::{Array, Direct, Optional};
    match field_type {
        IssuerKeyArray => Some(("IssuerKeyV1", Array)), RiskPolicy => Some(("RiskPolicyV1", Direct)), EvidencePolicy => Some(("EvidencePolicyV1", Direct)), SandboxPolicy => Some(("SandboxPolicyV1", Direct)), BudgetPolicy => Some(("BudgetPolicyV1", Direct)), RoutePolicy => Some(("RoutePolicyV1", Direct)), SignedAuthorityEnvelope => Some(("SignedAuthorityEnvelopeV1", Direct)), SignedMutationPermit => Some(("SignedMutationPermitV1", Direct)), OptionalSignedMutationPermit => Some(("SignedMutationPermitV1", Optional)), MutationReplayResult => Some(("MutationReplayResultV1", Direct)), OptionalMutationReplayResult => Some(("MutationReplayResultV1", Optional)), ScopeGrant => Some(("ScopeGrantV1", Direct)), PatchProposal => Some(("PatchProposalV1", Direct)), PatchOperationArray => Some(("PatchOperationV1", Array)), CleanupAuthorization => Some(("CleanupAuthorizationV1", Direct)), ReleaseFamilySubject => Some(("ReleaseFamilySubjectV1", Direct)), ReleaseRepositorySubjectArray => Some(("ReleaseRepositorySubjectV1", Array)), ReleaseEvidenceSubjectArray => Some(("ReleaseEvidenceSubjectV1", Array)), ReleaseProfileNodeArray => Some(("ReleaseProfileNodeV1", Array)), ReleaseSignerKeyArray => Some(("ReleaseSignerKeyV1", Array)), ReleaseRegistryEntryArray => Some(("ReleaseRegistryEntryV1", Array)), ReleaseRegistryObjectArray => Some(("ReleaseRegistryObjectV1", Array)), ReleaseReplayBindingArray => Some(("ReleaseReplayBindingV1", Array)), ExecutionToolArray => Some(("ExecutionToolV1", Array)),
        String | SchemaVersion | PolicySchemaVersion | Identifier | Digest | OrganizationId | RepositoryId | MissionId | AcceptanceContractId | PlanRevisionId | GraphRevisionId | WorkPackageId | SelectionGroupId | VariantId | AttemptId | RunnerId | WorkspaceId | PrincipalId | ProviderProfileId | ContentId | MutationId | MutationReservationId | ScopeGrantId | SourceDescriptorId | ChangeId | CheckpointId | CandidateId | GateReceiptId | ReleaseRegistryId | GateId | EffectIntentId | CandidateProofRoot | IntegrationProofRoot | GitOid | TaggedBlake3Digest | ReleaseGateId | ReleaseNativeSubjectId | ReleaseProfileId | ReleaseTag | SigningIdentity | SshEd25519PublicKey | RepoPath | AuthorityAudience | MutationOperation | AuthorityDecision | ReplayDisposition | MutationResultState | MutationOutcome | SettlementStatus | PatchPreimageKind | PatchMutationKind | ReleaseReceiptKind | ReleaseEvidenceKind | ReleaseRegistryObjectKind | ReleaseSignerRole | ReleaseRepositoryName | KeyId | KeyPurpose | KeyAlgorithm | PasetoV4Public | SafeU64 | U64 | Timestamp | OptionalTimestamp | OptionalDigest | OptionalString | OptionalMutationReservationId | Boolean | Object | StringArray | ObjectArray | AuthorityAudienceArray | CandidateIdArray | OrderedCandidateIdArray | GateIdArray | ReleaseGateIdArray | ReleaseProfileIdArray | ReleaseEvidenceKindArray | RepoPathArray | NamedRef | OptionalNamedRef | BoundedArray | BoundedSet => None,
    }
}

const GENERATOR_RESERVED: &str = "Box Option Result Self String Vec Array ArrayBuffer BigInt Boolean Date Error Function Map Number Object Promise Readonly ReadonlyArray Record RegExp Set Symbol Uint8Array WeakMap WeakSet AuthorityAudienceV1 MutationOperationV1 AuthorityDecisionV1 ReplayDispositionV1 MutationResultStateV1 MutationOutcomeV1 SettlementStatusV1 PatchPreimageKindV1 PatchMutationKindV1 ReleaseReceiptKindV1 ReleaseEvidenceKindV1 ReleaseRegistryObjectKindV1 ReleaseSignerRoleV1 ReleaseRepositoryNameV1 KeyPurposeV1 KeyAlgorithmV1 PinnedContract ContractPinError ContractValidationErrorV1 ContractDecodeResultV1 SchemaVersionLiteralV1 BoundedArrayV1 BoundedSetV1";
const RESERVED_FIELDS: &str = "abstract any as async await become boolean box break case catch class const constructor continue crate debugger declare default delete do dyn else enum export extends extern false final finally fn for from function gen get if impl implements import in instanceof interface let loop macro macro_rules match mod module move mut namespace never new null number object of override package priv private protected pub public raw readonly ref require return safe self set static string struct super switch symbol this throw trait true try type typeof undefined union unique unknown unsafe unsized use var virtual void where while with yield";
const LEGACY_SCHEMA_FIELDS_SHA256: &str =
    "e6205e51b34aecd4956bb1f30f3907ebbb5eedeab5dfd1b531e291723f51cf07";
