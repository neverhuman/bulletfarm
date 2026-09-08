//! Configuration diagnostics, never launch admission or operational evidence.

use serde::Serialize;
use std::{fs::File, io::Read, path::Path};

const REQUIRED_PROVIDERS: &[&str] = &["codex", "claude", "cursor"];

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(super) enum CheckStatus {
    Missing,
    Invalid,
    Validated,
    Unverified,
}

#[derive(Serialize)]
pub(super) struct ConfigurationView {
    ready: bool,
    binding: CheckStatus,
    policy: CheckStatus,
    providers: Vec<ProviderView>,
    containment: CheckStatus,
    durable_launch: CheckStatus,
    pub(super) blockers: Vec<&'static str>,
}

#[derive(Serialize)]
struct ProviderView {
    provider: &'static str,
    required: bool,
    enrollment: CheckStatus,
    runtime: CheckStatus,
    conformance: CheckStatus,
}

#[derive(Serialize)]
pub(super) struct OperationView {
    status: CheckStatus,
    pub(super) blockers: Vec<&'static str>,
}

pub(super) fn configuration() -> ConfigurationView {
    let binding_path = std::env::var_os("BULLET_DOGFOOD_BINDING");
    let policy_path = std::env::var_os("BULLET_DOGFOOD_POLICY");
    let binding = inspect(binding_path.as_deref().map(Path::new), load_binding);
    let policy = inspect(policy_path.as_deref().map(Path::new), load_policy);
    let mut blockers = Vec::new();
    match binding {
        CheckStatus::Missing => blockers.push("DOGFOOD_BINDING_MISSING"),
        CheckStatus::Invalid => blockers.push("DOGFOOD_BINDING_INVALID"),
        _ => {}
    }
    match policy {
        CheckStatus::Missing => blockers.push("DOGFOOD_POLICY_MISSING"),
        CheckStatus::Invalid => blockers.push("DOGFOOD_POLICY_INVALID"),
        _ => {}
    }
    // These consumers do not exist in the board yet. A binding, file-shaped
    // enrollment, or installed CLI must never substitute for their read-back.
    // Replace each UNVERIFIED only with an exact, current, validated subject.
    blockers.extend([
        "DOGFOOD_ENROLLMENT_CHECK_UNAVAILABLE",
        "DOGFOOD_RUNTIME_CHECK_UNAVAILABLE",
        "DOGFOOD_CONFORMANCE_CHECK_UNAVAILABLE",
        "DOGFOOD_CONTAINMENT_CHECK_UNAVAILABLE",
        "DOGFOOD_DURABLE_LAUNCH_CHECK_UNAVAILABLE",
    ]);
    ConfigurationView {
        ready: blockers.is_empty(),
        binding,
        policy,
        providers: REQUIRED_PROVIDERS
            .iter()
            .map(|provider| ProviderView {
                provider,
                required: true,
                enrollment: CheckStatus::Unverified,
                runtime: CheckStatus::Unverified,
                conformance: CheckStatus::Unverified,
            })
            .collect(),
        containment: CheckStatus::Unverified,
        durable_launch: CheckStatus::Unverified,
        blockers,
    }
}

pub(super) fn operation() -> OperationView {
    OperationView {
        status: CheckStatus::Unverified,
        blockers: vec!["DOGFOOD_OPERATION_CHECK_UNAVAILABLE"],
    }
}

fn inspect(path: Option<&Path>, validate: fn(&Path) -> Result<(), ()>) -> CheckStatus {
    let Some(path) = path else {
        return CheckStatus::Missing;
    };
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => CheckStatus::Missing,
        Ok(metadata) if metadata.is_file() => match validate(path) {
            Ok(()) => CheckStatus::Validated,
            Err(()) => CheckStatus::Invalid,
        },
        _ => CheckStatus::Invalid,
    }
}

pub(super) fn load_binding(path: &Path) -> Result<(), ()> {
    let bytes = read_document(path)?;
    let value = bullet_wire::decode_unique_value(&bytes).map_err(|_| ())?;
    let expected = serde_json::json!({
        "schema_version": bullet_wire::DogfoodBindingV1::SCHEMA_VERSION,
        "audience": "dogfood-runner",
        "operation": "read-only-propose",
    });
    if value != expected {
        return Err(());
    }
    Ok(())
}

fn load_policy(path: &Path) -> Result<(), ()> {
    let bytes = read_document(path)?;
    let policy: bullet_wire::PolicySnapshotV1 =
        bullet_wire::decode_canonical(&bytes).map_err(|_| ())?;
    bullet_wire::validate_dogfood_admission(
        &policy,
        &bullet_wire::DogfoodBindingV1::read_only_propose(),
    )
    .map_err(|_| ())
}

fn read_document(path: &Path) -> Result<Vec<u8>, ()> {
    #[cfg(unix)]
    let file = {
        use rustix::fs::{Mode, OFlags, open};
        File::from(
            open(
                path,
                OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(|_| ())?,
        )
    };
    #[cfg(not(unix))]
    let file = File::open(path).map_err(|_| ())?;
    let limit = bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES as u64;
    let metadata = file.metadata().map_err(|_| ())?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > limit {
        return Err(());
    }
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ())?;
    if bytes.len() as u64 != metadata.len() {
        return Err(());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    #[test]
    fn malformed_binding_is_typed_refuse() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("binding.json");
        std::fs::write(&path, b"{\"schema_version\":\"nope\"}\n").unwrap();
        assert!(super::load_binding(&path).is_err());
        let ok = dir.path().join("ok.json");
        std::fs::write(
            &ok,
            br#"{"audience":"dogfood-runner","operation":"read-only-propose","schema_version":"v1alpha1"}"#,
        )
        .unwrap();
        assert!(super::load_binding(&ok).is_ok());
    }
}
