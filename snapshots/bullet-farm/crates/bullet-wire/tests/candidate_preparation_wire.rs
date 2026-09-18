use std::{fs, path::PathBuf};

use bullet_wire::{
    CANDIDATE_PREPARATION_CLAIMS_DOMAIN, CANDIDATE_PREPARATION_ENVELOPE_DOMAIN,
    CANDIDATE_PREPARATION_SIGNING_PURPOSE, EXECUTION_ENVELOPE_CLAIMS_DOMAIN,
    EXECUTION_ENVELOPE_SIGNING_PURPOSE, MAX_SAFE_INTEGER, candidate_preparation_digest,
    canonical_json, decode_candidate_preparation_grant, decode_canonical_value,
    decode_execution_envelope, decode_signed_candidate_preparation_grant,
    execution_toolchain_digest,
    v1alpha1::{
        CandidatePreparationGrantV1, ExecutionEnvelopeV1, ExecutionToolV1,
        SignedCandidatePreparationGrantV1,
    },
    validate_candidate_preparation_binding,
};

fn id(prefix: &str, byte: char) -> String {
    format!("{prefix}_{}", byte.to_string().repeat(64))
}

fn digest(byte: char) -> String {
    byte.to_string().repeat(64)
}

fn tool(role: &str, byte: char) -> ExecutionToolV1 {
    ExecutionToolV1 {
        schema_version: "v1alpha1".into(),
        tool_id: id("etl", byte),
        role: role.into(),
        executable_path: format!("/usr/lib/bullet/{role}"),
        executable_digest: digest(byte),
        descriptor_digest: digest(if byte == 'f' { 'e' } else { 'f' }),
        version: "v1.0.0".into(),
    }
}

fn envelope() -> ExecutionEnvelopeV1 {
    let tools = vec![tool("runner", 'a'), tool("bullet-gitd", 'b')];
    ExecutionEnvelopeV1 {
        schema_version: "v1alpha1".into(),
        execution_envelope_id: id("exe", '1'),
        issuer: "bullet-kernel-local".into(),
        key_id: "authority-alpha".into(),
        signing_purpose: EXECUTION_ENVELOPE_SIGNING_PURPOSE.into(),
        claims_domain: EXECUTION_ENVELOPE_CLAIMS_DOMAIN.into(),
        runner_id: id("run", '2'),
        runner_epoch: 7,
        provider: "sim".into(),
        model: "deterministic".into(),
        adapter: "sim-v1".into(),
        provider_profile_id: id("prf", '3'),
        platform: "ubuntu-24.04-x86_64".into(),
        containment_profile_id: id("ctp", '4'),
        environment_digest: digest('5'),
        toolchain_digest: execution_toolchain_digest(&tools).unwrap().to_string(),
        sandbox_image_digest: digest('6'),
        tools,
        authority_epoch: 11,
        freeze_generation: 0,
        issued_at_unix_ms: 1_800_000_000_000,
        expires_at_unix_ms: 1_800_000_010_000,
    }
}

fn grant(parents: Vec<String>) -> CandidatePreparationGrantV1 {
    let envelope = envelope();
    CandidatePreparationGrantV1 {
        schema_version: "v1alpha1".into(),
        candidate_preparation_grant_id: id("cpg", '7'),
        issuer: envelope.issuer.clone(),
        key_id: envelope.key_id.clone(),
        signing_purpose: CANDIDATE_PREPARATION_SIGNING_PURPOSE.into(),
        claims_domain: CANDIDATE_PREPARATION_CLAIMS_DOMAIN.into(),
        envelope_domain: CANDIDATE_PREPARATION_ENVELOPE_DOMAIN.into(),
        request_digest: digest('8'),
        authority_token_digest: digest('9'),
        grant_nonce: digest('a'),
        repository_id: id("rep", 'b'),
        mission_id: id("mis", 'c'),
        plan_revision_id: id("pln", 'd'),
        work_package_id: id("wpk", 'e'),
        variant_id: id("var", 'f'),
        attempt_id: id("atm", '1'),
        attempt_fence: 17,
        runner_id: envelope.runner_id.clone(),
        runner_epoch: envelope.runner_epoch,
        workspace_id: id("wsp", '2'),
        scope_grant_digest: digest('3'),
        scope_revision: 1,
        context_revision: 1,
        change_id: id("chg", '4'),
        graph_revision_id: id("grf", '5'),
        parent_candidate_ids: parents,
        context_capsule_id: id("cnt", '6'),
        execution_envelope_id: envelope.execution_envelope_id.clone(),
        environment_digest: envelope.environment_digest.clone(),
        toolchain_digest: envelope.toolchain_digest.clone(),
        authority_epoch: envelope.authority_epoch,
        freeze_generation: envelope.freeze_generation,
        issued_at_unix_ms: envelope.issued_at_unix_ms,
        not_before_unix_ms: envelope.issued_at_unix_ms,
        expires_at_unix_ms: envelope.expires_at_unix_ms,
    }
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap()
        .to_path_buf()
}

#[test]
fn canonical_grant_and_execution_digests_are_stable_and_domain_separated() {
    let grant = grant(vec![id("can", '7'), id("can", '8')]);
    let envelope = envelope();
    validate_candidate_preparation_binding(&grant, &envelope).unwrap();
    let bytes = canonical_json(&grant).unwrap();
    assert_eq!(decode_candidate_preparation_grant(&bytes).unwrap(), grant);
    assert_eq!(
        decode_execution_envelope(&canonical_json(&envelope).unwrap()).unwrap(),
        envelope
    );
    assert_ne!(
        candidate_preparation_digest(&grant).unwrap(),
        bullet_wire::execution_envelope_digest(&envelope).unwrap()
    );
}

#[test]
fn ordered_parents_bind_identity_and_duplicates_refuse() {
    let first = grant(vec![id("can", '7'), id("can", '8')]);
    let reordered = grant(vec![id("can", '8'), id("can", '7')]);
    assert_ne!(
        candidate_preparation_digest(&first).unwrap(),
        candidate_preparation_digest(&reordered).unwrap()
    );
    let duplicate = grant(vec![id("can", '7'), id("can", '7')]);
    assert!(candidate_preparation_digest(&duplicate).is_err());
}

#[test]
fn recursive_unknowns_domain_substitution_and_unsafe_numbers_refuse() {
    let mut grant_value = serde_json::to_value(grant(vec![])).unwrap();
    grant_value["claims_domain"] = serde_json::json!(EXECUTION_ENVELOPE_CLAIMS_DOMAIN);
    let bytes = canonical_json(&grant_value).unwrap();
    assert!(decode_candidate_preparation_grant(&bytes).is_err());

    let mut unsafe_value = serde_json::to_value(grant(vec![])).unwrap();
    unsafe_value["authority_epoch"] = serde_json::json!(MAX_SAFE_INTEGER + 1);
    assert!(decode_candidate_preparation_grant(&canonical_json(&unsafe_value).unwrap()).is_err());

    let mut envelope_value = serde_json::to_value(envelope()).unwrap();
    envelope_value["tools"][0]["caller_selected"] = serde_json::json!(true);
    assert!(decode_execution_envelope(&canonical_json(&envelope_value).unwrap()).is_err());
}

#[test]
fn signed_grant_wrapper_is_closed_and_requires_a_bounded_v4_public_envelope() {
    let signed = SignedCandidatePreparationGrantV1 {
        schema_version: "v1alpha1".into(),
        issuer: "bullet-kernel-local".into(),
        key_id: "authority-alpha".into(),
        paseto: "v4.public.fixture".into(),
    };
    assert_eq!(
        decode_signed_candidate_preparation_grant(&canonical_json(&signed).unwrap()).unwrap(),
        signed
    );

    let mut wrong_protocol = serde_json::to_value(&signed).unwrap();
    wrong_protocol["paseto"] = serde_json::json!("v4.local.fixture");
    assert!(
        decode_signed_candidate_preparation_grant(&canonical_json(&wrong_protocol).unwrap())
            .is_err()
    );

    let mut unknown = serde_json::to_value(&signed).unwrap();
    unknown["signature"] = serde_json::json!("caller-supplied");
    assert!(decode_signed_candidate_preparation_grant(&canonical_json(&unknown).unwrap()).is_err());
}

#[test]
fn tool_manifest_and_grant_must_bind_the_exact_execution_envelope() {
    let envelope = envelope();
    let mut changed_tool = envelope.clone();
    changed_tool.tools[0].executable_digest = digest('0');
    assert!(bullet_wire::execution_envelope_digest(&changed_tool).is_err());

    let mut changed_grant = grant(vec![]);
    changed_grant.environment_digest = digest('0');
    assert!(validate_candidate_preparation_binding(&changed_grant, &envelope).is_err());
}

#[test]
fn generated_schema_and_both_bindings_close_the_new_nested_records() {
    let schema = decode_canonical_value(
        &fs::read(root().join("contracts/v1alpha1/schema-bundle.json")).unwrap(),
    )
    .unwrap();
    let grant = &schema["schemas"]["CandidatePreparationGrantV1"];
    assert_eq!(grant["additionalProperties"], false);
    assert_eq!(
        grant["properties"]["attempt_fence"]["maximum"],
        MAX_SAFE_INTEGER
    );
    assert_eq!(
        grant["properties"]["parent_candidate_ids"]["uniqueItems"],
        true
    );
    let tools = &schema["schemas"]["ExecutionEnvelopeV1"]["properties"]["tools"];
    assert_eq!(tools["items"]["$ref"], "#/schemas/ExecutionToolV1");
    assert_eq!(
        schema["schemas"]["ExecutionToolV1"]["additionalProperties"],
        false
    );

    let rust =
        fs::read_to_string(root().join("contracts/generated/rust/schema_bundle.rs")).unwrap();
    let typescript =
        fs::read_to_string(root().join("contracts/generated/typescript/schemaBundle.ts")).unwrap();
    for name in [
        "CandidatePreparationGrantV1",
        "SignedCandidatePreparationGrantV1",
        "ExecutionEnvelopeV1",
        "ExecutionToolV1",
    ] {
        assert!(rust.contains(&format!("pub struct {name}")));
        assert!(typescript.contains(&format!("export interface {name}")));
    }
    assert!(rust.contains("pub tools: Vec<ExecutionToolV1>"));
    assert!(typescript.contains("tools: ExecutionToolV1[]"));
}
