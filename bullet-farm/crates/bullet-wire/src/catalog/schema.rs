use serde_json::{Map, Value, json};

use crate::{WireError, policy::PolicySchemaVersion};

use super::{
    ContractFieldV1, FieldTypeV1, ResolvedCatalogV1, ResolvedFieldKindV1, ResolvedFieldV1,
    ResolvedRecordV1, constraints::conditional_constraints,
};

mod strict;

#[cfg(test)]
mod tests;

pub(super) fn json_schema_bundle(resolved: &ResolvedCatalogV1<'_>) -> Result<Value, WireError> {
    let mut schemas = Map::new();
    for scalar in resolved.scalar_types() {
        schemas.insert(scalar.name.clone(), strict::scalar_schema(scalar));
    }
    for record in resolved.records() {
        schemas.insert(record.definition().name.clone(), record_schema(record)?);
    }
    for tagged_union in resolved.tagged_unions() {
        schemas.insert(
            tagged_union.definition().name.clone(),
            strict::tagged_union_schema(tagged_union)?,
        );
    }
    Ok(json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "bundle_version": resolved.catalog_version(),
        "schema_version": resolved.schema_version(),
        "schemas": schemas,
    }))
}

fn record_schema(record: &ResolvedRecordV1<'_>) -> Result<Value, WireError> {
    let definition = record.definition();
    let mut properties = Map::new();
    for field in record.fields() {
        properties.insert(field.definition().name.clone(), field_schema(field)?);
    }
    let required = record
        .fields()
        .iter()
        .map(|field| Value::String(field.definition().name.clone()))
        .collect::<Vec<_>>();
    let mut schema = json!({
        "$id": format!("https://schemas.bullet.farm/v1alpha1/{}.json", definition.name),
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "additionalProperties": false,
        "properties": properties,
        "required": required,
        "title": definition.name,
        "type": "object",
        "x-bullet-security-class": definition.security_class,
        "x-bullet-unknown-fields": definition.unknown_fields,
    });
    if let Some(constraints) = conditional_constraints(&definition.name) {
        schema["allOf"] = constraints;
    }
    Ok(schema)
}

fn field_schema(field: &ResolvedFieldV1<'_>) -> Result<Value, WireError> {
    let definition = field.definition();
    match field.kind() {
        ResolvedFieldKindV1::LegacyValue => legacy_value_schema(definition),
        ResolvedFieldKindV1::LegacyReference { shape, target } => {
            strict::legacy_reference_schema(definition, shape, target)
        }
        ResolvedFieldKindV1::Named(target) if definition.field_type == FieldTypeV1::NamedRef => {
            Ok(strict::named_reference(target))
        }
        ResolvedFieldKindV1::OptionalNamed(target)
            if definition.field_type == FieldTypeV1::OptionalNamedRef =>
        {
            Ok(strict::optional_named_reference(target))
        }
        ResolvedFieldKindV1::BoundedArray { target, bounds }
            if definition.field_type == FieldTypeV1::BoundedArray =>
        {
            Ok(strict::bounded_array(target, bounds))
        }
        ResolvedFieldKindV1::BoundedSet { target, bounds }
            if definition.field_type == FieldTypeV1::BoundedSet =>
        {
            Ok(strict::bounded_set(target, bounds))
        }
        ResolvedFieldKindV1::Named(_)
        | ResolvedFieldKindV1::OptionalNamed(_)
        | ResolvedFieldKindV1::BoundedArray { .. }
        | ResolvedFieldKindV1::BoundedSet { .. } => Err(mismatch(definition)),
    }
}

fn legacy_value_schema(field: &ContractFieldV1) -> Result<Value, WireError> {
    let schema = match field.field_type {
        FieldTypeV1::String => json!({"type": "string", "minLength": 1}),
        FieldTypeV1::SchemaVersion => json!({"type": "string", "const": "v1alpha1"}),
        FieldTypeV1::PolicySchemaVersion => json!({
            "type": "string", "enum": PolicySchemaVersion::ACCEPTED
        }),
        FieldTypeV1::Identifier => json!({
            "type": "string",
            "pattern": "^[a-z][a-z0-9-]{1,15}_[0-9a-f]{64}$"
        }),
        FieldTypeV1::Digest => json!({"type": "string", "pattern": "^[0-9a-f]{64}$"}),
        FieldTypeV1::OrganizationId => digest_id_schema("org"),
        FieldTypeV1::RepositoryId => digest_id_schema("rep"),
        FieldTypeV1::MissionId => digest_id_schema("mis"),
        FieldTypeV1::AcceptanceContractId => digest_id_schema("acc"),
        FieldTypeV1::PlanRevisionId => digest_id_schema("pln"),
        FieldTypeV1::GraphRevisionId => digest_id_schema("grf"),
        FieldTypeV1::WorkPackageId => digest_id_schema("wpk"),
        FieldTypeV1::SelectionGroupId => digest_id_schema("sel"),
        FieldTypeV1::VariantId => digest_id_schema("var"),
        FieldTypeV1::AttemptId => digest_id_schema("atm"),
        FieldTypeV1::RunnerId => digest_id_schema("run"),
        FieldTypeV1::WorkspaceId => digest_id_schema("wsp"),
        FieldTypeV1::PrincipalId => digest_id_schema("pri"),
        FieldTypeV1::ProviderProfileId => digest_id_schema("prf"),
        FieldTypeV1::ContentId => digest_id_schema("cnt"),
        FieldTypeV1::MutationId => digest_id_schema("mut"),
        FieldTypeV1::MutationReservationId => digest_id_schema("rsv"),
        FieldTypeV1::ScopeGrantId => digest_id_schema("sgr"),
        FieldTypeV1::SourceDescriptorId => digest_id_schema("src"),
        FieldTypeV1::ChangeId => digest_id_schema("chg"),
        FieldTypeV1::CheckpointId => digest_id_schema("ckp"),
        FieldTypeV1::CandidateId => digest_id_schema("can"),
        FieldTypeV1::GateReceiptId => digest_id_schema("grc"),
        FieldTypeV1::ReleaseRegistryId => digest_id_schema("rrg"),
        FieldTypeV1::GateId => digest_id_schema("gat"),
        FieldTypeV1::EffectIntentId => digest_id_schema("efi"),
        FieldTypeV1::CandidateProofRoot => digest_id_schema("cpr"),
        FieldTypeV1::IntegrationProofRoot => digest_id_schema("ipr"),
        FieldTypeV1::GitOid => json!({
            "type": "string", "pattern": "^(sha1:[0-9a-f]{40}|sha256:[0-9a-f]{64})$"
        }),
        FieldTypeV1::TaggedBlake3Digest => json!({
            "type": "string", "pattern": "^blake3:[0-9a-f]{64}$"
        }),
        FieldTypeV1::ReleaseGateId => json!({
            "type": "string", "pattern": "^release\\.[a-z0-9][a-z0-9._-]{0,119}$"
        }),
        FieldTypeV1::ReleaseNativeSubjectId => json!({
            "type": "string",
            "pattern": "^[a-z][a-z0-9-]{0,63}:[a-z][a-z0-9-]{1,31}_[0-9a-f]{64}$"
        }),
        FieldTypeV1::ReleaseProfileId => json!({
            "type": "string", "pattern": "^[a-z][a-z0-9-]{0,62}[a-z0-9]$"
        }),
        FieldTypeV1::ReleaseTag => json!({
            "type": "string", "maxLength": 128,
            "pattern": "^v[0-9](?:[A-Za-z0-9.-]{0,126}[A-Za-z0-9])?$"
        }),
        FieldTypeV1::SigningIdentity => json!({
            "type": "string", "maxLength": 256,
            "pattern": "^[A-Za-z0-9._@+-]{1,128}\\|ed25519\\|SHA256:[A-Za-z0-9+/=]{16,96}$"
        }),
        FieldTypeV1::SshEd25519PublicKey => json!({
            "type": "string", "maxLength": 384,
            "pattern": "^ssh-ed25519 [A-Za-z0-9+/=]{40,256}$"
        }),
        FieldTypeV1::RepoPath => json!({
            "type": "string", "minLength": 1, "maxLength": 4096,
            "pattern": "^(?!/)(?!.*\\\\)(?!.*(?:^|/)\\.{1,2}(?:/|$))(?!.*(?:^|/)\\.git(?:/|$)).+$"
        }),
        FieldTypeV1::AuthorityAudience => json!({
            "type": "string", "enum": ["bullet-gitd", "effect-broker", "provider-runner"]
        }),
        FieldTypeV1::MutationOperation => json!({
            "type": "string",
            "enum": [
                "clone-workspace", "read-workspace", "apply-patch", "checkpoint",
                "prepare-candidate", "preserve-workspace", "cleanup-workspace",
                "dispatch-effect", "reconcile-effect"
            ]
        }),
        FieldTypeV1::AuthorityDecision => json!({
            "type": "string", "enum": ["authorized", "settled", "refused"]
        }),
        FieldTypeV1::ReplayDisposition => json!({
            "type": "string", "enum": ["fresh", "exact-replay", "conflict"]
        }),
        FieldTypeV1::MutationResultState => json!({
            "type": "string", "enum": ["in-flight", "committed", "aborted", "unknown"]
        }),
        FieldTypeV1::MutationOutcome => json!({
            "type": "string", "enum": ["committed", "aborted", "unknown"]
        }),
        FieldTypeV1::SettlementStatus => json!({
            "type": "string", "enum": ["accepted", "exact-replay", "conflict", "refused"]
        }),
        FieldTypeV1::PatchPreimageKind => json!({
            "type": "string", "enum": ["absent", "digest"]
        }),
        FieldTypeV1::PatchMutationKind => json!({
            "type": "string", "enum": ["write", "delete"]
        }),
        FieldTypeV1::ReleaseReceiptKind => json!({
            "type": "string",
            "enum": [
                "artifact", "containment", "forge", "operations", "profile-closure",
                "provider", "rust-toolchain", "scanner", "transaction"
            ]
        }),
        FieldTypeV1::ReleaseEvidenceKind => json!({
            "type": "string",
            "enum": [
                "artifact", "audit-anchor", "candidate", "check", "configuration",
                "effect", "environment", "evidence", "integration", "jeryu",
                "observation", "platform", "policy", "profile-graph", "proof-bundle",
                "provider", "provenance", "sandbox", "sbom", "scanner", "schema",
                "toolchain", "transaction"
            ]
        }),
        FieldTypeV1::ReleaseRegistryObjectKind => json!({
            "type": "string",
            "enum": [
                "gate-receipt", "gate-receipt-signature", "gate-spec", "profile-graph",
                "release-bundle-manifest-v2", "signer-policy", "trusted-time-observation",
                "trusted-time-signature", "verification-request"
            ]
        }),
        FieldTypeV1::ReleaseSignerRole => json!({
            "type": "string",
            "enum": [
                "artifact-release", "gate-attestor", "registry-curator", "source-tag",
                "trusted-time"
            ]
        }),
        FieldTypeV1::ReleaseRepositoryName => json!({
            "type": "string",
            "enum": ["bullet-farm", "bullet-git", "bullet-kernel", "bullet-portal"]
        }),
        FieldTypeV1::KeyId => json!({
            "type": "string", "pattern": "^[A-Za-z0-9][A-Za-z0-9._:/-]{0,127}$"
        }),
        FieldTypeV1::KeyPurpose => json!({
            "type": "string", "enum": ["authority-signing", "release-signing"]
        }),
        FieldTypeV1::KeyAlgorithm => json!({
            "type": "string", "enum": ["paseto-v4.public", "ssh-ed25519"]
        }),
        FieldTypeV1::PasetoV4Public => json!({
            "type": "string", "pattern": "^v4\\.public\\.", "maxLength": 32768
        }),
        FieldTypeV1::SafeU64 => {
            json!({"type": "integer", "minimum": 0, "maximum": 9007199254740991_u64})
        }
        FieldTypeV1::U64 => {
            json!({"type": "integer", "minimum": 0, "maximum": 9007199254740991_u64})
        }
        FieldTypeV1::Timestamp => {
            json!({"type": "integer", "minimum": 0, "maximum": 9007199254740991_u64})
        }
        FieldTypeV1::OptionalTimestamp => json!({
            "type": ["integer", "null"], "minimum": 0, "maximum": 9007199254740991_u64
        }),
        FieldTypeV1::OptionalDigest => json!({
            "type": ["string", "null"], "pattern": "^[0-9a-f]{64}$"
        }),
        FieldTypeV1::OptionalString => json!({"type": ["string", "null"]}),
        FieldTypeV1::OptionalMutationReservationId => optional_digest_id_schema("rsv"),
        FieldTypeV1::Boolean => json!({"type": "boolean"}),
        FieldTypeV1::Object => json!({"type": "object"}),
        FieldTypeV1::StringArray => json!({"type": "array", "items": {"type": "string"}}),
        FieldTypeV1::ObjectArray => json!({"type": "array", "items": {"type": "object"}}),
        FieldTypeV1::AuthorityAudienceArray => json!({
            "type": "array", "items": {
                "type": "string", "enum": ["bullet-gitd", "effect-broker", "provider-runner"]
            }
        }),
        FieldTypeV1::CandidateIdArray => typed_string_array("^can_[0-9a-f]{64}$"),
        FieldTypeV1::OrderedCandidateIdArray => json!({
            "type": "array",
            "maxItems": 128,
            "uniqueItems": true,
            "items": {"type": "string", "pattern": "^can_[0-9a-f]{64}$"}
        }),
        FieldTypeV1::GateIdArray => typed_string_array("^gat_[0-9a-f]{64}$"),
        FieldTypeV1::ReleaseGateIdArray => {
            typed_string_array("^release\\.[a-z0-9][a-z0-9._-]{0,119}$")
        }
        FieldTypeV1::ReleaseProfileIdArray => typed_string_array("^[a-z][a-z0-9-]{0,62}[a-z0-9]$"),
        FieldTypeV1::ReleaseEvidenceKindArray => json!({
            "type": "array",
            "items": legacy_value_schema(&ContractFieldV1 {
                name: "evidence_kind".to_owned(),
                field_type: FieldTypeV1::ReleaseEvidenceKind,
                target: None,
                bounds: None,
            })?
        }),
        FieldTypeV1::RepoPathArray => json!({
            "type": "array", "items": legacy_value_schema(&ContractFieldV1 {
                name: "path".to_owned(), field_type: FieldTypeV1::RepoPath,
                target: None,
                bounds: None,
            })?
        }),
        FieldTypeV1::IssuerKeyArray
        | FieldTypeV1::RiskPolicy
        | FieldTypeV1::EvidencePolicy
        | FieldTypeV1::SandboxPolicy
        | FieldTypeV1::BudgetPolicy
        | FieldTypeV1::RoutePolicy
        | FieldTypeV1::SignedAuthorityEnvelope
        | FieldTypeV1::SignedMutationPermit
        | FieldTypeV1::OptionalSignedMutationPermit
        | FieldTypeV1::MutationReplayResult
        | FieldTypeV1::OptionalMutationReplayResult
        | FieldTypeV1::ScopeGrant
        | FieldTypeV1::PatchProposal
        | FieldTypeV1::PatchOperationArray
        | FieldTypeV1::CleanupAuthorization
        | FieldTypeV1::ReleaseFamilySubject
        | FieldTypeV1::ReleaseRepositorySubjectArray
        | FieldTypeV1::ReleaseEvidenceSubjectArray
        | FieldTypeV1::ReleaseProfileNodeArray
        | FieldTypeV1::ReleaseSignerKeyArray
        | FieldTypeV1::ReleaseRegistryEntryArray
        | FieldTypeV1::ReleaseRegistryObjectArray
        | FieldTypeV1::ReleaseReplayBindingArray
        | FieldTypeV1::ExecutionToolArray
        | FieldTypeV1::NamedRef
        | FieldTypeV1::OptionalNamedRef
        | FieldTypeV1::BoundedArray
        | FieldTypeV1::BoundedSet => return Err(mismatch(field)),
    };
    Ok(schema)
}

pub(super) fn mismatch(field: &ContractFieldV1) -> WireError {
    WireError::new(
        "INVALID_CONTRACT_FIELD_REFERENCE",
        format!("resolved field kind disagrees with {}", field.name),
    )
}

fn digest_id_schema(prefix: &str) -> Value {
    json!({"type": "string", "pattern": format!("^{prefix}_[0-9a-f]{{64}}$")})
}

fn optional_digest_id_schema(prefix: &str) -> Value {
    json!({"type": ["string", "null"], "pattern": format!("^{prefix}_[0-9a-f]{{64}}$")})
}

fn typed_string_array(pattern: &str) -> Value {
    json!({"type": "array", "items": {"type": "string", "pattern": pattern}})
}
