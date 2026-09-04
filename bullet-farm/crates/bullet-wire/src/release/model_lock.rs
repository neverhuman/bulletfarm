//! Formal model-lock binding for `GateReceiptV1` evidence subjects.
//!
//! `formal/model-lock.json` and `formal/toolchain.lock.json` pin exactly two
//! bounded TLA+ models (`EffectCheck`, `LeaseFence`): module and config
//! SHA-256 digests, the TLC jar digest, and the deterministic distinct-state
//! count and depth that `formal/model-check.sh` re-verifies. This module
//! projects those committed lock files into wire records and binds them,
//! together with the executable trace fixtures under `formal/traces/`, into one
//! receipt evidence subject so a receipt cannot claim "model checked" without
//! naming exactly which model, configuration, tool, and trace were checked.
//!
//! Subject convention (needs no schema change): the receipt carries exactly one
//! `proof-bundle` evidence subject whose `native_subject_id` is
//! `proof-bundle:formal-model-lock_<binding hex>` and whose `subject_digest`
//! is `blake3:<binding hex>`. Field promotion: the catalog lane must add the
//! optional field [`FORMAL_MODEL_LOCK_BINDING_FIELD`] (a `blake3:`-tagged
//! digest string) to `GateReceiptV1` in `contracts/v1alpha1/schema-bundle.json`
//! and regenerate the bundle; [`ModelLockBinding::binding_digest`] is the value
//! that field carries. Nothing here verifies signatures or clears a gate.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{
    WireError, decode_unique_value, hash_canonical, hash_framed_bytes,
    v1alpha1::{ReleaseEvidenceKindV1, ReleaseEvidenceSubjectV1},
};

use super::{fields::MAX_SAFE_INTEGER, validate::ReleaseWireRecord};

/// The family's pinned model inventory, byte-sorted. `formal/model-check.sh`
/// refuses any other inventory; the wire type refuses any other lock set.
pub const FORMAL_MODEL_IDS: [&str; 2] = ["EffectCheck", "LeaseFence"];
pub const FORMAL_MODEL_LOCK_SEED: u64 = 20_260_824;
pub const FORMAL_MODEL_LOCK_WORKERS: u64 = 1;
pub const FORMAL_MODEL_LOCK_DIGEST_DOMAIN: &str = "formal.model-lock.v1alpha1";
pub const FORMAL_TRACE_DIGEST_DOMAIN: &str = "formal.trace.v1alpha1";
pub const FORMAL_MODEL_LOCK_BINDING_DIGEST_DOMAIN: &str = "formal.model-lock-binding.v1alpha1";
pub const FORMAL_MODEL_LOCK_NATIVE_SUBJECT_PREFIX: &str = "proof-bundle:formal-model-lock_";
/// Documented receipt field name for the promoted binding digest.
pub const FORMAL_MODEL_LOCK_BINDING_FIELD: &str = "formal_model_lock_binding";

pub const MODEL_LOCK_MISSING: &str = "MODEL_LOCK_MISSING";
pub const MODEL_LOCK_MISMATCH: &str = "MODEL_LOCK_MISMATCH";
pub const FORMAL_TRACE_UNBOUND: &str = "FORMAL_TRACE_UNBOUND";
pub const MODEL_LOCK_MALFORMED: &str = "MODEL_LOCK_MALFORMED";
pub const MODEL_SET_INCOMPLETE: &str = "MODEL_SET_INCOMPLETE";

const LOCK_SCHEMA_VERSION: &str = "v1alpha1";
const SHA256_TAG: &str = "sha256:";
const BLAKE3_TAG: &str = "blake3:";

/// One pinned bounded model as recorded by the committed lock files.
/// `states` is the deterministic distinct-state count; `lock_digest` is the
/// domain-separated digest of every other field and must recompute exactly.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelLockV1 {
    pub model_id: String,
    pub source_digest: String,
    pub config_digest: String,
    pub tool_digest: String,
    pub states: u64,
    pub depth: u64,
    pub lock_digest: String,
}

/// One executable trace fixture bound to the exact lock of the model it
/// replays.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormalTraceBindingV1 {
    pub model_id: String,
    pub trace_digest: String,
    pub lock_digest: String,
}

/// The accepted binding: the complete pinned lock set, every bound trace, and
/// the digest a receipt subject must carry.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ModelLockBinding {
    pub binding_digest: String,
    pub locks: Vec<ModelLockV1>,
    pub traces: Vec<FormalTraceBindingV1>,
}

#[derive(Serialize)]
struct ModelLockBody<'a> {
    model_id: &'a str,
    source_digest: &'a str,
    config_digest: &'a str,
    tool_digest: &'a str,
    states: u64,
    depth: u64,
}

#[derive(Serialize)]
struct ModelLockBindingBody<'a> {
    locks: &'a [ModelLockV1],
    traces: &'a [FormalTraceBindingV1],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelLockFileV1 {
    models: Vec<ModelLockEntryV1>,
    schema_version: String,
    seed: u64,
    workers: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelLockEntryV1 {
    config: String,
    config_sha256: String,
    depth: u64,
    distinct_states: u64,
    generated_states: u64,
    module: String,
    module_sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ToolchainLockFileV1 {
    java_major: u64,
    schema_version: String,
    tlc_jar_sha1: String,
    tlc_jar_sha256: String,
    tlc_release: String,
    tlc_revision: String,
    tlc_version: String,
    url: String,
}

/// Parses the committed `formal/model-lock.json` and
/// `formal/toolchain.lock.json` bytes into the pinned lock set. The shape is
/// exactly the one `formal/model-check.sh` validates: unknown or missing
/// members, duplicate keys, unsafe or non-integer numbers, a seed or worker
/// count other than the pinned values, and any inventory other than the two
/// pinned modules are refused.
pub fn parse_model_locks(
    model_lock: &[u8],
    toolchain_lock: &[u8],
) -> Result<Vec<ModelLockV1>, WireError> {
    let file: ModelLockFileV1 = strict_document(model_lock, "model lock")?;
    let tool: ToolchainLockFileV1 = strict_document(toolchain_lock, "toolchain lock")?;
    if file.schema_version != LOCK_SCHEMA_VERSION || tool.schema_version != LOCK_SCHEMA_VERSION {
        return Err(refuse(MODEL_LOCK_MALFORMED, "lock schema version is unsupported"));
    }
    if file.seed != FORMAL_MODEL_LOCK_SEED || file.workers != FORMAL_MODEL_LOCK_WORKERS {
        return Err(refuse(
            MODEL_LOCK_MALFORMED,
            "model lock seed or worker count differs from the pinned values",
        ));
    }
    lower_hex(&tool.tlc_jar_sha256, 64, "TLC jar SHA-256")?;
    lower_hex(&tool.tlc_jar_sha1, 40, "TLC jar SHA-1")?;
    if tool.java_major == 0
        || tool.tlc_release.is_empty()
        || tool.tlc_revision.is_empty()
        || tool.tlc_version.is_empty()
        || !tool.url.starts_with("https://")
    {
        return Err(refuse(MODEL_LOCK_MALFORMED, "toolchain lock pins are empty"));
    }
    let tool_digest = format!("{SHA256_TAG}{}", tool.tlc_jar_sha256);

    let modules = file
        .models
        .iter()
        .map(|entry| entry.module.as_str())
        .collect::<Vec<_>>();
    let expected = FORMAL_MODEL_IDS.map(|id| format!("{id}.tla"));
    if modules != expected.iter().map(String::as_str).collect::<Vec<_>>() {
        return Err(refuse(
            MODEL_SET_INCOMPLETE,
            "model lock inventory is not exactly the pinned EffectCheck and LeaseFence modules",
        ));
    }
    file.models
        .iter()
        .zip(FORMAL_MODEL_IDS)
        .map(|(entry, model_id)| {
            if entry.config != format!("{model_id}.cfg") {
                return Err(refuse(
                    MODEL_LOCK_MALFORMED,
                    "model config name does not pair with its module",
                ));
            }
            if entry.generated_states < entry.distinct_states {
                return Err(refuse(
                    MODEL_LOCK_MALFORMED,
                    "generated states are fewer than distinct states",
                ));
            }
            ModelLockV1::new(
                model_id,
                &format!("{SHA256_TAG}{}", entry.module_sha256),
                &format!("{SHA256_TAG}{}", entry.config_sha256),
                &tool_digest,
                entry.distinct_states,
                entry.depth,
            )
        })
        .collect()
}

impl ModelLockV1 {
    /// Builds a lock and computes its `lock_digest`; every field is validated.
    pub fn new(
        model_id: &str,
        source_digest: &str,
        config_digest: &str,
        tool_digest: &str,
        states: u64,
        depth: u64,
    ) -> Result<Self, WireError> {
        let body = ModelLockBody {
            model_id,
            source_digest,
            config_digest,
            tool_digest,
            states,
            depth,
        };
        let lock_digest = body.digest()?;
        let lock = Self {
            model_id: model_id.to_owned(),
            source_digest: source_digest.to_owned(),
            config_digest: config_digest.to_owned(),
            tool_digest: tool_digest.to_owned(),
            states,
            depth,
            lock_digest,
        };
        lock.validate()?;
        Ok(lock)
    }

    /// Refuses malformed fields (`MODEL_LOCK_MALFORMED`) and a `lock_digest`
    /// that does not recompute from the other fields (`MODEL_LOCK_MISMATCH`).
    pub fn validate(&self) -> Result<(), WireError> {
        model_id(&self.model_id)?;
        for (label, digest) in [
            ("source", &self.source_digest),
            ("config", &self.config_digest),
            ("tool", &self.tool_digest),
        ] {
            tagged(digest, SHA256_TAG, label)?;
        }
        tagged(&self.lock_digest, BLAKE3_TAG, "lock")?;
        positive(self.states, "distinct state count")?;
        positive(self.depth, "search depth")?;
        let expected = ModelLockBody {
            model_id: &self.model_id,
            source_digest: &self.source_digest,
            config_digest: &self.config_digest,
            tool_digest: &self.tool_digest,
            states: self.states,
            depth: self.depth,
        }
        .digest()?;
        if expected != self.lock_digest {
            return Err(refuse(
                MODEL_LOCK_MISMATCH,
                "lock digest does not recompute from the lock fields",
            ));
        }
        Ok(())
    }

    /// Proves the exact module and config bytes are the ones this lock pins.
    pub fn verify_sources(&self, module: &[u8], config: &[u8]) -> Result<(), WireError> {
        for (label, digest, bytes) in [
            ("module", &self.source_digest, module),
            ("config", &self.config_digest, config),
        ] {
            if *digest != format!("{SHA256_TAG}{:x}", Sha256::digest(bytes)) {
                return Err(refuse(
                    MODEL_LOCK_MISMATCH,
                    format!("{label} bytes differ from the {} lock", self.model_id),
                ));
            }
        }
        Ok(())
    }
}

impl<'a> ModelLockBody<'a> {
    fn digest(&self) -> Result<String, WireError> {
        Ok(format!(
            "{BLAKE3_TAG}{}",
            hash_canonical(FORMAL_MODEL_LOCK_DIGEST_DOMAIN, self)?.to_hex()
        ))
    }
}

impl FormalTraceBindingV1 {
    /// Digests the exact trace bytes and binds them to `lock`. The trace must
    /// be a strict `v1alpha1` fixture whose `model` names the lock's model.
    pub fn bind(trace: &[u8], lock: &ModelLockV1) -> Result<Self, WireError> {
        lock.validate()?;
        let model = trace_model(trace)?;
        if model != lock.model_id {
            return Err(refuse(
                FORMAL_TRACE_UNBOUND,
                format!("trace replays {model} but binds the {} lock", lock.model_id),
            ));
        }
        Ok(Self {
            model_id: model,
            trace_digest: format!(
                "{BLAKE3_TAG}{}",
                hash_framed_bytes(FORMAL_TRACE_DIGEST_DOMAIN, trace)?.to_hex()
            ),
            lock_digest: lock.lock_digest.clone(),
        })
    }

    pub fn validate(&self) -> Result<(), WireError> {
        model_id(&self.model_id)?;
        tagged(&self.trace_digest, BLAKE3_TAG, "trace")?;
        tagged(&self.lock_digest, BLAKE3_TAG, "trace lock")
    }
}

impl ModelLockBinding {
    /// Accepts the lock set and traces without any receipt subject: every
    /// lock recomputes, the lock set is exactly the pinned models in order,
    /// every trace names a present lock by its exact digest, and every model
    /// has at least one bound trace.
    pub fn compute(
        locks: &[ModelLockV1],
        traces: &[FormalTraceBindingV1],
    ) -> Result<Self, WireError> {
        for lock in locks {
            lock.validate()?;
        }
        model_set(locks)?;
        let mut previous = None;
        for trace in traces {
            trace.validate()?;
            let identity = (trace.model_id.as_str(), trace.trace_digest.as_str());
            if previous.is_some_and(|prior| prior >= identity) {
                return Err(refuse(
                    MODEL_LOCK_MALFORMED,
                    "trace bindings must be byte-sorted and unique",
                ));
            }
            previous = Some(identity);
            let lock = locks
                .iter()
                .find(|lock| lock.model_id == trace.model_id)
                .ok_or_else(|| {
                    refuse(
                        MODEL_LOCK_MISSING,
                        format!("trace names {} which has no lock", trace.model_id),
                    )
                })?;
            if trace.lock_digest != lock.lock_digest {
                return Err(refuse(
                    MODEL_LOCK_MISMATCH,
                    format!("trace for {} names a different lock digest", trace.model_id),
                ));
            }
        }
        for lock in locks {
            if !traces.iter().any(|trace| trace.model_id == lock.model_id) {
                return Err(refuse(
                    FORMAL_TRACE_UNBOUND,
                    format!("{} has no bound executable trace", lock.model_id),
                ));
            }
        }
        let body = ModelLockBindingBody { locks, traces };
        Ok(Self {
            binding_digest: format!(
                "{BLAKE3_TAG}{}",
                hash_canonical(FORMAL_MODEL_LOCK_BINDING_DIGEST_DOMAIN, &body)?.to_hex()
            ),
            locks: locks.to_vec(),
            traces: traces.to_vec(),
        })
    }

    /// The exact `native_subject_id` a bound receipt subject must carry.
    pub fn native_subject_id(&self) -> String {
        format!(
            "{FORMAL_MODEL_LOCK_NATIVE_SUBJECT_PREFIX}{}",
            &self.binding_digest[BLAKE3_TAG.len()..]
        )
    }

    /// Refuses a subject that carries no binding (`MODEL_LOCK_MISSING`) or a
    /// binding other than this one (`MODEL_LOCK_MISMATCH`).
    pub fn verify_subject(&self, subject: &ReleaseEvidenceSubjectV1) -> Result<(), WireError> {
        subject.validate_release()?;
        if subject.subject_kind != ReleaseEvidenceKindV1::ProofBundle
            || !subject
                .native_subject_id
                .starts_with(FORMAL_MODEL_LOCK_NATIVE_SUBJECT_PREFIX)
        {
            return Err(refuse(
                MODEL_LOCK_MISSING,
                "receipt subject does not carry the formal model-lock binding",
            ));
        }
        if subject.native_subject_id != self.native_subject_id()
            || subject.subject_digest != self.binding_digest
        {
            return Err(refuse(
                MODEL_LOCK_MISMATCH,
                "receipt subject names a different model-lock binding digest",
            ));
        }
        Ok(())
    }
}

/// Binds `locks` and `traces` and requires `receipt_subject` to name exactly
/// that binding. Any refusal carries one of the five typed codes.
pub fn bind_model_locks(
    receipt_subject: &ReleaseEvidenceSubjectV1,
    locks: &[ModelLockV1],
    traces: &[FormalTraceBindingV1],
) -> Result<ModelLockBinding, WireError> {
    let binding = ModelLockBinding::compute(locks, traces)?;
    binding.verify_subject(receipt_subject)?;
    Ok(binding)
}

/// Selects the single model-lock binding subject among a receipt's evidence
/// subjects. A receipt without one cannot claim a model check.
pub fn find_model_lock_subject(
    evidence_subjects: &[ReleaseEvidenceSubjectV1],
) -> Result<&ReleaseEvidenceSubjectV1, WireError> {
    let mut candidates = evidence_subjects.iter().filter(|subject| {
        subject.subject_kind == ReleaseEvidenceKindV1::ProofBundle
            && subject
                .native_subject_id
                .starts_with(FORMAL_MODEL_LOCK_NATIVE_SUBJECT_PREFIX)
    });
    let subject = candidates.next().ok_or_else(|| {
        refuse(
            MODEL_LOCK_MISSING,
            "receipt names no formal model-lock binding subject",
        )
    })?;
    if candidates.next().is_some() {
        return Err(refuse(
            MODEL_LOCK_MALFORMED,
            "receipt names more than one formal model-lock binding subject",
        ));
    }
    Ok(subject)
}

fn model_set(locks: &[ModelLockV1]) -> Result<(), WireError> {
    let ids = locks
        .iter()
        .map(|lock| lock.model_id.as_str())
        .collect::<Vec<_>>();
    if let Some(unknown) = ids.iter().find(|id| !FORMAL_MODEL_IDS.contains(id)) {
        return Err(refuse(
            MODEL_SET_INCOMPLETE,
            format!("{unknown} is not one of the pinned formal models"),
        ));
    }
    if let Some(missing) = FORMAL_MODEL_IDS.iter().find(|id| !ids.contains(id)) {
        return Err(refuse(
            MODEL_LOCK_MISSING,
            format!("lock for pinned model {missing} is missing"),
        ));
    }
    if ids.len() != FORMAL_MODEL_IDS.len() {
        return Err(refuse(
            MODEL_SET_INCOMPLETE,
            "lock set repeats a pinned model",
        ));
    }
    if ids != FORMAL_MODEL_IDS {
        return Err(refuse(
            MODEL_LOCK_MALFORMED,
            "locks must be byte-sorted by model id",
        ));
    }
    Ok(())
}

fn strict_document<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    label: &str,
) -> Result<T, WireError> {
    let value = decode_unique_value(bytes)
        .map_err(|error| refuse(MODEL_LOCK_MALFORMED, format!("{label}: {error}")))?;
    serde_json::from_value(value).map_err(|error| {
        refuse(
            MODEL_LOCK_MALFORMED,
            format!("{label} does not match its exact shape: {error}"),
        )
    })
}

fn trace_model(bytes: &[u8]) -> Result<String, WireError> {
    let value = decode_unique_value(bytes)
        .map_err(|error| refuse(MODEL_LOCK_MALFORMED, format!("formal trace: {error}")))?;
    let malformed = || refuse(MODEL_LOCK_MALFORMED, "formal trace is not a strict v1alpha1 fixture");
    let Value::Object(members) = &value else {
        return Err(malformed());
    };
    let keys = members.keys().map(String::as_str).collect::<Vec<_>>();
    if keys != ["model", "schema_version", "steps"]
        || members.get("schema_version").and_then(Value::as_str) != Some(LOCK_SCHEMA_VERSION)
    {
        return Err(malformed());
    }
    let steps = members.get("steps").and_then(Value::as_array).ok_or_else(malformed)?;
    if steps.is_empty()
        || !steps.iter().all(|step| {
            step.get("action").is_some_and(Value::is_string)
                && step.get("expected").is_some_and(Value::is_string)
        })
    {
        return Err(malformed());
    }
    let model = members.get("model").and_then(Value::as_str).ok_or_else(malformed)?;
    model_id(model)?;
    Ok(model.to_owned())
}

fn model_id(value: &str) -> Result<(), WireError> {
    let bytes = value.as_bytes();
    if bytes.is_empty()
        || bytes.len() > 64
        || !bytes[0].is_ascii_alphabetic()
        || !bytes.iter().all(u8::is_ascii_alphanumeric)
    {
        return Err(refuse(MODEL_LOCK_MALFORMED, "model id is malformed"));
    }
    Ok(())
}

fn positive(value: u64, label: &str) -> Result<(), WireError> {
    if value == 0 || value > MAX_SAFE_INTEGER {
        return Err(refuse(
            MODEL_LOCK_MALFORMED,
            format!("{label} is zero or outside the exact integer range"),
        ));
    }
    Ok(())
}

fn tagged(value: &str, tag: &str, label: &str) -> Result<(), WireError> {
    let hex = value
        .strip_prefix(tag)
        .ok_or_else(|| refuse(MODEL_LOCK_MALFORMED, format!("{label} digest lacks its {tag} tag")))?;
    lower_hex(hex, 64, label)
}

fn lower_hex(value: &str, length: usize, label: &str) -> Result<(), WireError> {
    if value.len() != length
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(refuse(
            MODEL_LOCK_MALFORMED,
            format!("{label} digest is not {length} lowercase hexadecimal characters"),
        ));
    }
    Ok(())
}

fn refuse(code: &'static str, reason: impl Into<String>) -> WireError {
    WireError::new(code, reason)
}
