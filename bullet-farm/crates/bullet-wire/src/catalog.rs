#[cfg(test)]
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::WireError;

mod constraints;
mod launch;
mod records;
mod schema;
mod validation;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldTypeV1 {
    String,
    SchemaVersion,
    PolicySchemaVersion,
    Identifier,
    Digest,
    OrganizationId,
    RepositoryId,
    MissionId,
    AcceptanceContractId,
    PlanRevisionId,
    GraphRevisionId,
    WorkPackageId,
    SelectionGroupId,
    VariantId,
    AttemptId,
    RunnerId,
    WorkspaceId,
    PrincipalId,
    ProviderProfileId,
    ContentId,
    MutationId,
    MutationReservationId,
    ScopeGrantId,
    SourceDescriptorId,
    ChangeId,
    CheckpointId,
    CandidateId,
    GateReceiptId,
    ReleaseRegistryId,
    GateId,
    EffectIntentId,
    CandidateProofRoot,
    IntegrationProofRoot,
    GitOid,
    TaggedBlake3Digest,
    ReleaseGateId,
    ReleaseNativeSubjectId,
    ReleaseProfileId,
    ReleaseTag,
    SigningIdentity,
    SshEd25519PublicKey,
    RepoPath,
    AuthorityAudience,
    MutationOperation,
    AuthorityDecision,
    ReplayDisposition,
    MutationResultState,
    MutationOutcome,
    SettlementStatus,
    PatchPreimageKind,
    PatchMutationKind,
    ReleaseReceiptKind,
    ReleaseEvidenceKind,
    ReleaseRegistryObjectKind,
    ReleaseSignerRole,
    ReleaseRepositoryName,
    KeyId,
    KeyPurpose,
    KeyAlgorithm,
    PasetoV4Public,
    SafeU64,
    U64,
    Timestamp,
    OptionalTimestamp,
    OptionalDigest,
    OptionalString,
    OptionalMutationReservationId,
    Boolean,
    Object,
    StringArray,
    ObjectArray,
    AuthorityAudienceArray,
    IssuerKeyArray,
    RiskPolicy,
    EvidencePolicy,
    SandboxPolicy,
    BudgetPolicy,
    RoutePolicy,
    SignedAuthorityEnvelope,
    SignedMutationPermit,
    OptionalSignedMutationPermit,
    MutationReplayResult,
    OptionalMutationReplayResult,
    ScopeGrant,
    PatchProposal,
    PatchOperationArray,
    CandidateIdArray,
    OrderedCandidateIdArray,
    GateIdArray,
    ReleaseGateIdArray,
    ReleaseProfileIdArray,
    ReleaseEvidenceKindArray,
    RepoPathArray,
    CleanupAuthorization,
    ReleaseFamilySubject,
    ReleaseRepositorySubjectArray,
    ReleaseEvidenceSubjectArray,
    ReleaseProfileNodeArray,
    ReleaseSignerKeyArray,
    ReleaseRegistryEntryArray,
    ReleaseRegistryObjectArray,
    ReleaseReplayBindingArray,
    ExecutionToolArray,
    NamedRef,
    OptionalNamedRef,
    BoundedArray,
    BoundedSet,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionBoundsV1 {
    pub min_items: u16,
    pub max_items: u16,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordShapeV1 {
    #[default]
    Versioned,
    Embedded,
}

#[rustfmt::skip]
impl RecordShapeV1 { fn is_versioned(value: &Self) -> bool { *value == Self::Versioned } }

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityClassV1 {
    Attestation,
    Audit,
    CommandAuthority,
    EffectAuthority,
    Holdout,
    Integration,
    Policy,
    Projection,
    Release,
    Research,
    Review,
    Verification,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodeClassV1 {
    LowerKebab,
    UpperHyphen,
    AsciiToken,
    InvariantId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case", tag = "kind")]
pub enum ScalarDefinitionV1 {
    SafeInteger {
        minimum: i64,
        maximum: i64,
    },
    Text {
        minimum_utf8_bytes: u32,
        maximum_utf8_bytes: u32,
    },
    Code {
        minimum_ascii_bytes: u16,
        maximum_ascii_bytes: u16,
        class: CodeClassV1,
    },
    Enum {
        values: Vec<String>,
    },
    TypedId {
        prefix: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScalarTypeV1 {
    pub name: String,
    pub definition: ScalarDefinitionV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnionVariantV1 {
    pub tag: String,
    pub record: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaggedUnionV1 {
    pub name: String,
    pub discriminator: String,
    pub variants: Vec<UnionVariantV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LegacyReferenceShapeV1 {
    Direct,
    Optional,
    Array,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResolvedSymbolKindV1 {
    Scalar,
    Record,
    TaggedUnion,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ResolvedSymbolV1<'a> {
    Scalar(&'a ScalarTypeV1),
    Record(&'a ContractRecordV1),
    TaggedUnion(&'a TaggedUnionV1),
}

impl<'a> ResolvedSymbolV1<'a> {
    pub(crate) fn name(self) -> &'a str {
        match self {
            Self::Scalar(value) => &value.name,
            Self::Record(value) => &value.name,
            Self::TaggedUnion(value) => &value.name,
        }
    }

    pub(crate) fn kind(self) -> ResolvedSymbolKindV1 {
        match self {
            Self::Scalar(_) => ResolvedSymbolKindV1::Scalar,
            Self::Record(_) => ResolvedSymbolKindV1::Record,
            Self::TaggedUnion(_) => ResolvedSymbolKindV1::TaggedUnion,
        }
    }
}

pub(crate) fn rust_variant_identifier(value: &str) -> String {
    let mut output = String::new();
    for part in value.split(['-', '_']) {
        let mut bytes = part.bytes();
        if let Some(first) = bytes.next() {
            output.push(first.to_ascii_uppercase() as char);
            output.extend(bytes.map(|byte| byte.to_ascii_lowercase() as char));
        }
    }
    output
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ResolvedFieldKindV1<'a> {
    LegacyValue,
    LegacyReference {
        shape: LegacyReferenceShapeV1,
        target: ResolvedSymbolV1<'a>,
    },
    Named(ResolvedSymbolV1<'a>),
    OptionalNamed(ResolvedSymbolV1<'a>),
    BoundedArray {
        target: ResolvedSymbolV1<'a>,
        bounds: CollectionBoundsV1,
    },
    BoundedSet {
        target: ResolvedSymbolV1<'a>,
        bounds: CollectionBoundsV1,
    },
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ResolvedFieldV1<'a> {
    definition: &'a ContractFieldV1,
    kind: ResolvedFieldKindV1<'a>,
}

#[rustfmt::skip]
impl<'a> ResolvedFieldV1<'a> {
    pub(crate) fn definition(&self) -> &'a ContractFieldV1 { self.definition }
    pub(crate) fn kind(&self) -> ResolvedFieldKindV1<'a> { self.kind }
}

#[derive(Debug)]
pub(crate) struct ResolvedRecordV1<'a> {
    definition: &'a ContractRecordV1,
    fields: Vec<ResolvedFieldV1<'a>>,
}

#[rustfmt::skip]
impl<'a> ResolvedRecordV1<'a> {
    pub(crate) fn definition(&self) -> &'a ContractRecordV1 { self.definition }
    pub(crate) fn fields(&self) -> &[ResolvedFieldV1<'a>] { &self.fields }
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedUnionVariantV1<'a> {
    definition: &'a UnionVariantV1,
    record: &'a ContractRecordV1,
    fields: Vec<ResolvedFieldV1<'a>>,
}

#[rustfmt::skip]
impl<'a> ResolvedUnionVariantV1<'a> {
    pub(crate) fn definition(&self) -> &'a UnionVariantV1 { self.definition }
    pub(crate) fn record(&self) -> &'a ContractRecordV1 { self.record }
    pub(crate) fn fields(&self) -> &[ResolvedFieldV1<'a>] { &self.fields }
}

#[derive(Debug)]
pub(crate) struct ResolvedTaggedUnionV1<'a> {
    definition: &'a TaggedUnionV1,
    variants: Vec<ResolvedUnionVariantV1<'a>>,
}

#[rustfmt::skip]
impl<'a> ResolvedTaggedUnionV1<'a> {
    pub(crate) fn definition(&self) -> &'a TaggedUnionV1 { self.definition }
    pub(crate) fn variants(&self) -> &[ResolvedUnionVariantV1<'a>] { &self.variants }
}

#[derive(Debug)]
pub(crate) struct ResolvedCatalogV1<'a> {
    definition: &'a ContractCatalogV1,
    scalar_types: Vec<&'a ScalarTypeV1>,
    records: Vec<ResolvedRecordV1<'a>>,
    tagged_unions: Vec<ResolvedTaggedUnionV1<'a>>,
    #[cfg(test)]
    symbols: BTreeMap<&'a str, ResolvedSymbolV1<'a>>,
    #[cfg(test)]
    adjacency: BTreeMap<&'a str, Vec<&'a str>>,
}

impl<'a> ResolvedCatalogV1<'a> {
    pub(crate) fn schema_version(&self) -> &str {
        &self.definition.schema_version
    }
    pub(crate) fn catalog_version(&self) -> &str {
        &self.definition.catalog_version
    }
    pub(crate) fn scalar_types(&self) -> &[&'a ScalarTypeV1] {
        &self.scalar_types
    }
    pub(crate) fn records(&self) -> &[ResolvedRecordV1<'a>] {
        &self.records
    }
    pub(crate) fn tagged_unions(&self) -> &[ResolvedTaggedUnionV1<'a>] {
        &self.tagged_unions
    }
    #[cfg(test)]
    pub(crate) fn symbols(&self) -> &BTreeMap<&'a str, ResolvedSymbolV1<'a>> {
        &self.symbols
    }
    #[cfg(test)]
    pub(crate) fn adjacency(&self) -> &BTreeMap<&'a str, Vec<&'a str>> {
        &self.adjacency
    }
    #[cfg(test)]
    pub(crate) fn record(&self, name: &str) -> Option<&ResolvedRecordV1<'a>> {
        self.records
            .iter()
            .find(|item| item.definition.name == name)
    }
    pub(crate) fn json_schema_bundle(&self) -> Result<Value, WireError> {
        schema::json_schema_bundle(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractFieldV1 {
    pub name: String,
    pub field_type: FieldTypeV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounds: Option<CollectionBoundsV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractRecordV1 {
    pub name: String,
    pub security_class: SecurityClassV1,
    pub unknown_fields: String,
    #[serde(default, skip_serializing_if = "RecordShapeV1::is_versioned")]
    pub shape: RecordShapeV1,
    pub fields: Vec<ContractFieldV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractCatalogV1 {
    pub schema_version: String,
    pub catalog_version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scalar_types: Vec<ScalarTypeV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tagged_unions: Vec<TaggedUnionV1>,
    pub records: Vec<ContractRecordV1>,
}

impl ContractCatalogV1 {
    pub(crate) fn resolve(&self) -> Result<ResolvedCatalogV1<'_>, WireError> {
        validation::resolve(self)
    }

    #[cfg(test)]
    pub(crate) fn resolve_test_strict(&self) -> Result<ResolvedCatalogV1<'_>, WireError> {
        validation::resolve_test_strict(self)
    }

    pub fn validate(&self) -> Result<(), WireError> {
        self.resolve().map(|_| ())
    }

    pub fn json_schema_bundle(&self) -> Result<Value, WireError> {
        self.resolve()?.json_schema_bundle()
    }
}
