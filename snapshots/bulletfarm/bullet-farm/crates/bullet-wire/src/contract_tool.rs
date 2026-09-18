use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{
    AuthorityAudience, ContractCatalogV1, EnforcementTier, InvariantLifecycle, InvariantRegistryV1,
    IssuerKeyV1, KeyAlgorithmV1, KeyPurposeV1, LIVE_ADMISSION_MIN_GENERATION,
    POLICY_SCHEMA_VERSION, POLICY_SCHEMA_VERSION_V1ALPHA2, PolicySnapshotV1, PolicyTemplateV1,
    WireError, canonical_json,
    contract_bindings::{BindingInputs, rust_constants, typescript_constants},
    decode_canonical, hash_canonical, hash_framed_bytes,
};

mod authority;
mod launch;
use authority::authority_golden;
use launch::launch_grant_golden;

const CATALOG: &str = "contracts/v1alpha1/contract-catalog.json";
const REGISTRY: &str = "policy/v1alpha1/invariant-registry.json";
const POLICY_TEMPLATE: &str = "policy/v1alpha1/policy-template.json";
const HOSTILE_TEAM: &str = "fixtures/hostile/team-original.bin";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractMode {
    Generate,
    Check,
}

pub fn execute(root: &Path, mode: ContractMode) -> Result<(), WireError> {
    let outputs = render(root)?;
    match mode {
        ContractMode::Generate => write_outputs(root, &outputs),
        ContractMode::Check => check_outputs(root, &outputs),
    }
}

fn render(root: &Path) -> Result<BTreeMap<PathBuf, Vec<u8>>, WireError> {
    let catalog_bytes = read(root, CATALOG)?;
    let catalog = decode_canonical::<ContractCatalogV1>(&catalog_bytes)?;
    let resolved = catalog.resolve()?;
    let schema_bundle = resolved.json_schema_bundle()?;
    let schema_bundle_bytes = canonical_json(&schema_bundle)?;
    let schema_bundle_hash = hash_canonical("schema.bundle", &schema_bundle)?;

    let registry_bytes = read(root, REGISTRY)?;
    let registry = decode_canonical::<InvariantRegistryV1>(&registry_bytes)?;
    registry.validate()?;
    let registry_hash = hash_canonical("invariant.registry", &registry)?;

    let template_bytes = read(root, POLICY_TEMPLATE)?;
    let template = decode_canonical::<PolicyTemplateV1>(&template_bytes)?;
    let policy = policy_snapshot(template, schema_bundle_hash, registry_hash);
    policy.validate()?;
    let policy_bytes = canonical_json(&policy)?;
    let policy_hash = hash_canonical("policy.snapshot", &policy)?;
    let live_policy = live_policy_snapshot(policy.clone())?;
    let live_policy_bytes = canonical_json(&live_policy)?;

    let team = read(root, HOSTILE_TEAM)?;
    let fixture_manifest = fixture_manifest(&team)?;
    let (golden, golden_json, golden_hash) = canonical_golden()?;
    let (authority_golden, authority_golden_hash) = authority_golden()?;
    let (launch_grant_golden, launch_grant_golden_hash) = launch_grant_golden()?;
    let binding_inputs = BindingInputs {
        schema: schema_bundle_hash,
        registry: registry_hash,
        policy: policy_hash,
        golden_json: &golden_json,
        golden_hash,
        authority_golden_hash,
        launch_grant_golden_hash,
        resolved: &resolved,
    };
    let rust_binding = rust_constants(&binding_inputs)?;
    let typescript_binding = typescript_constants(&binding_inputs)?;
    let generated_clients = json!({
        "rust": hash_framed_bytes("generated.client.rust", rust_binding.as_bytes())?,
        "typescript": hash_framed_bytes(
            "generated.client.typescript",
            typescript_binding.as_bytes()
        )?
    });
    let bundle_manifest = json!({
        "bundle_hash": schema_bundle_hash,
        "authority_golden_hash": authority_golden_hash,
        "catalog_hash": hash_framed_bytes("contract.catalog", &catalog_bytes)?,
        "generated_client_hash": hash_canonical("generated.clients", &generated_clients)?,
        "generated_clients": generated_clients,
        "generator": "bullet-wire-contract-tool-v1alpha1",
        "invariant_registry_hash": registry_hash,
        "launch_grant_golden_hash": launch_grant_golden_hash,
        "policy_snapshot_hash": policy_hash,
        "record_count": resolved.records().len(),
        "schema_version": "v1alpha1"
    });

    let mut outputs = BTreeMap::new();
    outputs.insert(
        "contracts/v1alpha1/schema-bundle.json".into(),
        schema_bundle_bytes,
    );
    outputs.insert(
        "contracts/v1alpha1/bundle-manifest.json".into(),
        canonical_json(&bundle_manifest)?,
    );
    outputs.insert(
        "contracts/generated/rust/schema_bundle.rs".into(),
        rust_binding.into_bytes(),
    );
    outputs.insert(
        "contracts/generated/typescript/schemaBundle.ts".into(),
        typescript_binding.into_bytes(),
    );
    outputs.insert("policy/v1alpha1/policy.json".into(), policy_bytes);
    outputs.insert(
        "crates/bullet-wire/tests/fixtures/policy-v1alpha2-live-enabled.json".into(),
        live_policy_bytes,
    );
    outputs.insert(
        "fixtures/hostile/fixture-manifest.json".into(),
        canonical_json(&fixture_manifest)?,
    );
    outputs.insert(
        "fixtures/canonical/canonical-golden.json".into(),
        canonical_json(&golden)?,
    );
    outputs.insert(
        "fixtures/canonical/authority-golden.json".into(),
        canonical_json(&authority_golden)?,
    );
    outputs.insert(
        "fixtures/canonical/launch-grant-golden.json".into(),
        canonical_json(&launch_grant_golden)?,
    );
    outputs.insert(
        "docs/assurance/invariant-crosswalk.generated.md".into(),
        invariant_crosswalk(&registry).into_bytes(),
    );
    outputs.extend(hostile_cases());
    Ok(outputs)
}

fn invariant_crosswalk(registry: &InvariantRegistryV1) -> String {
    let mut markdown = String::from(
        "<!-- Generated by: bullet-wire-contract-tool v1alpha1 -->\n<!-- Source: policy/v1alpha1/invariant-registry.json -->\n<!-- Command: just contract-generate -->\n<!-- DO NOT EDIT BY HAND. -->\n\n# Invariant crosswalk\n\n| Stable ID | Control | Tier | Lifecycle | First wave | Owner | Enforcement target | Gate |\n| --- | --- | --- | --- | ---: | --- | --- | --- |\n",
    );
    for entry in &registry.entries {
        let controls = if entry.control_ids.is_empty() {
            "—".to_owned()
        } else {
            entry.control_ids.join(", ")
        };
        markdown.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            entry.id,
            controls,
            tier_label(entry.tier),
            lifecycle_label(entry.lifecycle),
            entry.first_applicable_wave,
            entry.owner,
            if entry.enforcement_target.is_empty() {
                "—"
            } else {
                &entry.enforcement_target
            },
            entry.gate,
        ));
    }
    markdown
}

const fn tier_label(tier: EnforcementTier) -> &'static str {
    match tier {
        EnforcementTier::T1Schema => "T1 schema",
        EnforcementTier::T2Gateway => "T2 gateway",
        EnforcementTier::T3Test => "T3 test",
    }
}

const fn lifecycle_label(lifecycle: InvariantLifecycle) -> &'static str {
    match lifecycle {
        InvariantLifecycle::Planned => "planned",
        InvariantLifecycle::Enforced => "enforced",
        InvariantLifecycle::Retired => "retired",
    }
}

fn hostile_cases() -> BTreeMap<PathBuf, Vec<u8>> {
    let cases: [(&str, &[u8]); 11] = [
        ("bom.json", b"\xef\xbb\xbf{\"x\":1}"),
        ("bidi.json", br#"{"x":"a\u202eb"}"#),
        ("crlf.json", b"{\r\n\"x\":1\r\n}"),
        ("duplicate-key.json", br#"{"x":1,"x":2}"#),
        ("escaped-control.json", br#"{"x":"\u001b"}"#),
        ("invalid-utf8.json", b"{\"x\":\"\xff\"}"),
        ("lf.json", b"{\n\"x\":1\n}"),
        ("non-nfc.json", br#"{"x":"e\u0301"}"#),
        ("nul.json", br#"{"x":"\u0000"}"#),
        ("raw-control.json", b"{\"x\":\"\x1b\"}"),
        ("zero-width.json", br#"{"x":"a\u200bb"}"#),
    ];
    cases
        .into_iter()
        .map(|(name, bytes)| {
            (
                PathBuf::from("fixtures/hostile/cases").join(name),
                bytes.to_vec(),
            )
        })
        .collect()
}

fn policy_snapshot(
    template: PolicyTemplateV1,
    schema_bundle_hash: crate::Blake3Digest,
    invariant_registry_hash: crate::Blake3Digest,
) -> PolicySnapshotV1 {
    PolicySnapshotV1 {
        schema_version: template.schema_version,
        policy_generation: template.policy_generation,
        schema_bundle_hash,
        invariant_registry_hash,
        activation_at_unix_ms: template.activation_at_unix_ms,
        expires_at_unix_ms: template.expires_at_unix_ms,
        issuer_keys: template.issuer_keys,
        risk_policy: template.risk_policy,
        evidence_policy: template.evidence_policy,
        sandbox_policy: template.sandbox_policy,
        budget_policy: template.budget_policy,
        route_policy: template.route_policy,
    }
}

fn live_policy_snapshot(mut policy: PolicySnapshotV1) -> Result<PolicySnapshotV1, WireError> {
    let release_key = policy.issuer_keys.first().ok_or_else(|| {
        WireError::new(
            "INVALID_POLICY_WINDOW",
            "generated live-policy fixture requires the release-signing key lifecycle",
        )
    })?;
    let activates_at_unix_ms = release_key.activates_at_unix_ms;
    let expires_at_unix_ms = release_key.expires_at_unix_ms;
    let retain_until_unix_ms = release_key.retain_until_unix_ms;

    policy.schema_version = POLICY_SCHEMA_VERSION_V1ALPHA2.to_owned();
    policy.policy_generation = LIVE_ADMISSION_MIN_GENERATION;
    policy.sandbox_policy.live_admission_enabled = true;
    policy.issuer_keys.push(IssuerKeyV1 {
        schema_version: POLICY_SCHEMA_VERSION.to_owned(),
        issuer: "bullet-kernel-local".to_owned(),
        key_id: "authority-test-1".to_owned(),
        key_purpose: KeyPurposeV1::AuthoritySigning,
        algorithm: KeyAlgorithmV1::PasetoV4Public,
        public_key: "1eb9dbbbbc047c03fd70604e0071f0987e16b28b757225c11f00415d0e20b1a2".to_owned(),
        audiences: vec![AuthorityAudience::ProviderRunner],
        activates_at_unix_ms,
        expires_at_unix_ms,
        revoked_at_unix_ms: None,
        retain_until_unix_ms,
    });
    policy.validate()?;
    Ok(policy)
}

fn fixture_manifest(team: &[u8]) -> Result<Value, WireError> {
    let sha256 = Sha256::digest(team);
    Ok(json!({
        "fixtures": [{
            "blake3": hash_framed_bytes("hostile.fixture", team)?,
            "bytes": team.len(),
            "classification": "hostile_forensic_input",
            "path": HOSTILE_TEAM,
            "runtime_authority": false,
            "sha256": format!("{sha256:x}")
        }],
        "schema_version": "v1alpha1"
    }))
}

fn canonical_golden() -> Result<(Value, String, crate::Blake3Digest), WireError> {
    let value = json!({"a": "é", "array": [true, null, 17], "z": "last"});
    let bytes = canonical_json(&value)?;
    let text = String::from_utf8(bytes.clone())
        .map_err(|error| WireError::new("GOLDEN_ENCODING_FAILED", error.to_string()))?;
    let hash = hash_framed_bytes("golden.cross-language", &bytes)?;
    Ok((
        json!({
            "canonical_json_utf8": text,
            "domain": "golden.cross-language",
            "framed_blake3": hash,
            "schema_version": "v1alpha1"
        }),
        text,
        hash,
    ))
}

fn read(root: &Path, relative: &str) -> Result<Vec<u8>, WireError> {
    fs::read(root.join(relative)).map_err(|error| {
        WireError::new(
            "CONTRACT_INPUT_READ_FAILED",
            format!("cannot read {relative}: {error}"),
        )
    })
}

fn write_outputs(root: &Path, outputs: &BTreeMap<PathBuf, Vec<u8>>) -> Result<(), WireError> {
    for (relative, bytes) in outputs {
        let destination = root.join(relative);
        let parent = destination.parent().ok_or_else(|| {
            WireError::new("CONTRACT_OUTPUT_PATH", "generated output has no parent")
        })?;
        fs::create_dir_all(parent).map_err(io_error("create output directory", relative))?;
        let staged_output = destination.with_extension("bullet-contract.tmp");
        let mut file = File::create(&staged_output).map_err(io_error("create output", relative))?;
        file.write_all(bytes)
            .map_err(io_error("write output", relative))?;
        file.sync_all().map_err(io_error("sync output", relative))?;
        fs::rename(&staged_output, &destination).map_err(io_error("replace output", relative))?;
    }
    Ok(())
}

fn check_outputs(root: &Path, outputs: &BTreeMap<PathBuf, Vec<u8>>) -> Result<(), WireError> {
    for (relative, expected) in outputs {
        let actual = fs::read(root.join(relative)).map_err(io_error("read output", relative))?;
        if &actual != expected {
            return Err(WireError::new(
                "CONTRACT_DRIFT",
                format!("{} differs from generated bytes", relative.display()),
            ));
        }
    }
    Ok(())
}

fn io_error<'a>(action: &'a str, path: &'a Path) -> impl Fn(std::io::Error) -> WireError + 'a {
    move |error| {
        WireError::new(
            "CONTRACT_IO_FAILED",
            format!("{action} {}: {error}", path.display()),
        )
    }
}
