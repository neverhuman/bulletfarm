//! Every claims field is either subject-checked against the verifier's durable
//! expectation or, when it is not part of that expectation, still identity-bound
//! by the signature: splicing a changed payload under the original signature is
//! refused as `LAUNCH_GRANT_INVALID`.

use bullet_wire::{AuthorityAudience, LaunchGrantClaims, LaunchProvider, canonical_json};
use serde_json::{Value, json};

use super::{
    NOT_BEFORE, claims, digest, expectation, hostile::splice_payload, id, signer, verifier,
};

type Mutation = fn(&mut LaunchGrantClaims);

const SUBJECT_MUTATIONS: &[(&str, Mutation)] = &[
    ("mission_id", |claims| claims.mission_id = id("mis_", '0')),
    ("repository_id", |claims| {
        claims.repository_id = id("rep_", '0');
    }),
    ("graph_revision_id", |claims| {
        claims.graph_revision_id = id("grf_", '0');
    }),
    ("work_package_id", |claims| {
        claims.work_package_id = id("wpk_", '0');
    }),
    ("variant_id", |claims| claims.variant_id = id("var_", '0')),
    ("attempt_id", |claims| claims.attempt_id = id("atm_", '0')),
    ("attempt_fence", |claims| claims.attempt_fence += 1),
    ("runner_id", |claims| claims.runner_id = id("run_", '0')),
    ("runner_epoch", |claims| claims.runner_epoch += 1),
    ("workspace_id", |claims| {
        claims.workspace_id = id("wsp_", '0');
    }),
    ("workspace_nonce_digest", |claims| {
        claims.workspace_nonce_digest = digest(99);
    }),
    ("authority_epoch", |claims| claims.authority_epoch += 1),
    ("freeze_generation", |claims| claims.freeze_generation += 1),
    ("provider", |claims| claims.provider = LaunchProvider::Codex),
    ("adapter", |claims| {
        claims.adapter = "codex-app-server-v1".to_owned()
    }),
    ("provider_profile_id", |claims| {
        claims.provider_profile_id = id("prf_", '0');
    }),
    ("model", |claims| claims.model = "claude-other".to_owned()),
    ("credential_generation", |claims| {
        claims.credential_generation += 1;
    }),
    ("protocol", |claims| {
        claims.protocol = "codex_exec_json".to_owned()
    }),
    ("executable_path", |claims| {
        claims.executable_path = "/usr/bin/claude".to_owned();
    }),
    ("executable_digest", |claims| {
        claims.executable_digest = digest(99)
    }),
    ("descriptor_digest", |claims| {
        claims.descriptor_digest = digest(99)
    }),
    ("capability_digest", |claims| {
        claims.capability_digest = digest(99)
    }),
    ("policy_snapshot_digest", |claims| {
        claims.policy_snapshot_digest = digest(99);
    }),
];

const SHAPE_MUTATIONS: &[(&str, Mutation, &str)] = &[
    (
        "schema_version",
        |claims| claims.schema_version = "v2".to_owned(),
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "audience",
        |claims| claims.audience = AuthorityAudience::BulletGitd,
        "LAUNCH_GRANT_AUDIENCE_MISMATCH",
    ),
    (
        "issuer empty",
        |claims| claims.issuer = String::new(),
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "key_id space",
        |claims| claims.key_id = "key id".to_owned(),
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "adapter control",
        |claims| claims.adapter = "a\u{1b}b".to_owned(),
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "model overlong",
        |claims| claims.model = "m".repeat(129),
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "protocol empty",
        |claims| claims.protocol = String::new(),
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "protocol case",
        |claims| claims.protocol = "Claude_Stream".to_owned(),
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "path relative",
        |claims| claims.executable_path = "bin/claude".to_owned(),
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "path dotdot",
        |claims| claims.executable_path = "/usr/../bin/claude".to_owned(),
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "path dot",
        |claims| claims.executable_path = "/usr/./claude".to_owned(),
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "path trailing",
        |claims| claims.executable_path = "/usr/bin/".to_owned(),
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "path double slash",
        |claims| claims.executable_path = "/usr//claude".to_owned(),
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "path control",
        |claims| claims.executable_path = "/usr/bin/cl\u{7f}aude".to_owned(),
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "path root",
        |claims| claims.executable_path = "/".to_owned(),
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "attempt_fence zero",
        |claims| claims.attempt_fence = 0,
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "max_invocations zero",
        |claims| claims.max_invocations = 0,
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "max_wall_clock_ms zero",
        |claims| claims.max_wall_clock_ms = 0,
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "runner_epoch unsafe",
        |claims| claims.runner_epoch = 1 << 53,
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "max_cost unsafe",
        |claims| claims.max_cost_micro_usd = u64::MAX,
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "expires unsafe",
        |claims| claims.expires_at_unix_ms = 1 << 53,
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "issued after not_before",
        |claims| claims.issued_at_unix_ms = NOT_BEFORE + 1,
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "expires equals not_before",
        |claims| claims.expires_at_unix_ms = NOT_BEFORE,
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "expires before not_before",
        |claims| claims.expires_at_unix_ms = NOT_BEFORE - 1,
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "ttl 15001",
        |claims| claims.expires_at_unix_ms = NOT_BEFORE + 15_001,
        "LAUNCH_GRANT_TTL_EXCEEDED",
    ),
    (
        "gate_ids empty",
        |claims| claims.gate_ids.clear(),
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "gate_ids seventeen",
        |claims| {
            claims.gate_ids = "0123456789abcdef0"
                .chars()
                .map(|fill| id("gat_", fill))
                .collect();
        },
        "LAUNCH_GRANT_INVALID",
    ),
    (
        "gate_ids duplicate",
        |claims| claims.gate_ids.push(claims.gate_ids[0].clone()),
        "LAUNCH_GRANT_INVALID",
    ),
];

#[test]
fn every_lease_provider_and_policy_field_is_subject_checked() {
    let original = claims();
    let expected = expectation(&original);
    for (field, mutate) in SUBJECT_MUTATIONS {
        let mut changed = original.clone();
        mutate(&mut changed);
        assert_ne!(changed, original, "{field} mutation was a no-op");
        let grant = signer().sign_launch_grant(&changed).unwrap();
        assert_eq!(
            grant
                .verify(&verifier(), &expected, NOT_BEFORE)
                .unwrap_err()
                .code(),
            "LAUNCH_GRANT_SUBJECT_MISMATCH",
            "field {field}"
        );
        grant
            .verify(&verifier(), &expectation(&changed), NOT_BEFORE)
            .unwrap();
    }
}

#[test]
fn shape_bounds_refuse_before_signing_and_at_verification() {
    for (case, mutate, code) in SHAPE_MUTATIONS {
        let mut changed = claims();
        mutate(&mut changed);
        assert_eq!(
            changed.validate_shape().unwrap_err().code(),
            *code,
            "case {case}"
        );
        assert_eq!(
            signer().sign_launch_grant(&changed).unwrap_err().code(),
            *code,
            "case {case}"
        );
        assert!(changed.digest().is_err(), "case {case}");
    }
    let mut sixteen = claims();
    sixteen.gate_ids = "0123456789abcdef"
        .chars()
        .map(|fill| id("gat_", fill))
        .collect();
    signer().sign_launch_grant(&sixteen).unwrap();
}

#[test]
fn issuer_and_key_id_must_name_the_signing_key() {
    let mut other_issuer = claims();
    other_issuer.issuer = "another-kernel".to_owned();
    assert_eq!(
        signer()
            .sign_launch_grant(&other_issuer)
            .unwrap_err()
            .code(),
        "LAUNCH_GRANT_KEY_UNKNOWN"
    );
    let mut other_key = claims();
    other_key.key_id = "authority-test-2".to_owned();
    assert_eq!(
        signer().sign_launch_grant(&other_key).unwrap_err().code(),
        "LAUNCH_GRANT_KEY_UNKNOWN"
    );
}

#[test]
fn every_claims_field_is_identity_bound_by_the_signature() {
    let original = claims();
    let expected = expectation(&original);
    let grant = signer().sign_launch_grant(&original).unwrap();
    let value = serde_json::to_value(&original).unwrap();
    let fields = value.as_object().unwrap();
    assert_eq!(fields.len(), 42);
    let original_digest = original.digest().unwrap();
    for (field, field_value) in fields {
        let mut changed = value.clone();
        changed[field] = mutate_json(field_value);
        let payload = canonical_json(&changed).unwrap();
        let spliced = bullet_wire::SignedLaunchGrant {
            paseto: splice_payload(&grant.paseto, &payload),
            ..grant.clone()
        };
        assert_eq!(
            spliced
                .verify(&verifier(), &expected, NOT_BEFORE)
                .unwrap_err()
                .code(),
            "LAUNCH_GRANT_INVALID",
            "field {field}"
        );
        let mut removed = value.clone();
        removed.as_object_mut().unwrap().remove(field);
        let digest = bullet_wire::hash_framed_bytes(
            bullet_wire::LAUNCH_GRANT_CLAIMS_DOMAIN,
            &canonical_json(&removed).unwrap(),
        )
        .unwrap();
        assert_ne!(
            digest, original_digest,
            "field {field} was not identity-bound"
        );
    }
}

fn mutate_json(value: &Value) -> Value {
    match value {
        Value::String(text) => {
            let mut changed = text.clone();
            let last = changed.pop().unwrap_or('x');
            changed.push(if last == 'a' { 'b' } else { 'a' });
            Value::String(changed)
        }
        Value::Number(number) => json!(number.as_u64().unwrap() + 1),
        Value::Array(items) => Value::Array(items[..1].to_vec()),
        other => panic!("unexpected claims value {other}"),
    }
}
