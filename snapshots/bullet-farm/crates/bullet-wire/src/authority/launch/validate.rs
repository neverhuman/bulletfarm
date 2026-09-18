use std::collections::BTreeSet;

use super::{LaunchGrantClaims, MAX_LAUNCH_GRANT_GATE_IDS, MAX_LAUNCH_GRANT_TTL_MS, launch_error};
use crate::{
    AuthorityAudience, GateId, WireError,
    authority::{AUTHORITY_SCHEMA_VERSION, validate_label},
};

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const MAX_EXECUTABLE_PATH_BYTES: usize = 4_096;
const MAX_PROTOCOL_BYTES: usize = 64;

impl LaunchGrantClaims {
    /// Structural validation of every bound from the shared spec. Shape failures
    /// are `LAUNCH_GRANT_INVALID`, the audience bound is
    /// `LAUNCH_GRANT_AUDIENCE_MISMATCH`, and a window wider than
    /// [`MAX_LAUNCH_GRANT_TTL_MS`] is `LAUNCH_GRANT_TTL_EXCEEDED`.
    pub fn validate_shape(&self) -> Result<(), WireError> {
        if self.schema_version != AUTHORITY_SCHEMA_VERSION {
            return Err(invalid("launch grant claims require schema v1alpha1"));
        }
        if self.audience != AuthorityAudience::ProviderRunner {
            return Err(launch_error(
                "LAUNCH_GRANT_AUDIENCE_MISMATCH",
                "launch grants are only valid for the provider-runner audience",
            ));
        }
        for (name, value) in [
            ("issuer", self.issuer.as_str()),
            ("key_id", self.key_id.as_str()),
            ("adapter", self.adapter.as_str()),
            ("model", self.model.as_str()),
        ] {
            validate_label(name, value).map_err(|error| invalid(error.reason().to_owned()))?;
        }
        validate_protocol(&self.protocol)?;
        validate_executable_path(&self.executable_path)?;
        for (name, value) in [
            ("attempt_fence", self.attempt_fence),
            ("max_invocations", self.max_invocations),
            ("max_wall_clock_ms", self.max_wall_clock_ms),
        ] {
            if value == 0 {
                return Err(invalid(format!("{name} must be positive")));
            }
        }
        for (name, value) in [
            ("attempt_fence", self.attempt_fence),
            ("runner_epoch", self.runner_epoch),
            ("authority_epoch", self.authority_epoch),
            ("freeze_generation", self.freeze_generation),
            ("credential_generation", self.credential_generation),
            ("policy_generation", self.policy_generation),
            ("max_invocations", self.max_invocations),
            ("max_wall_clock_ms", self.max_wall_clock_ms),
            ("max_cost_micro_usd", self.max_cost_micro_usd),
            ("issued_at_unix_ms", self.issued_at_unix_ms),
            ("not_before_unix_ms", self.not_before_unix_ms),
            ("expires_at_unix_ms", self.expires_at_unix_ms),
        ] {
            if value > MAX_SAFE_INTEGER {
                return Err(invalid(format!(
                    "{name} exceeds the interoperable integer range"
                )));
            }
        }
        if self.issued_at_unix_ms > self.not_before_unix_ms
            || self.not_before_unix_ms >= self.expires_at_unix_ms
        {
            return Err(invalid(
                "launch grant requires issued_at <= not_before < expires_at",
            ));
        }
        if self.expires_at_unix_ms - self.not_before_unix_ms > MAX_LAUNCH_GRANT_TTL_MS {
            return Err(launch_error(
                "LAUNCH_GRANT_TTL_EXCEEDED",
                "launch grant window exceeds the 15s maximum lifetime",
            ));
        }
        validate_gate_ids(&self.gate_ids)
    }
}

fn validate_protocol(value: &str) -> Result<(), WireError> {
    let mut bytes = value.bytes();
    let well_formed = value.len() <= MAX_PROTOCOL_BYTES
        && bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
    if !well_formed {
        return Err(invalid(
            "protocol must be a bounded lowercase snake_case provider protocol label",
        ));
    }
    Ok(())
}

fn validate_executable_path(value: &str) -> Result<(), WireError> {
    if value.is_empty()
        || value.len() > MAX_EXECUTABLE_PATH_BYTES
        || !value.starts_with('/')
        || value.chars().any(char::is_control)
        || value
            .split('/')
            .skip(1)
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(invalid(
            "executable_path must be a bounded, normalized, absolute, control-free path",
        ));
    }
    Ok(())
}

fn validate_gate_ids(gate_ids: &[GateId]) -> Result<(), WireError> {
    if gate_ids.is_empty() || gate_ids.len() > MAX_LAUNCH_GRANT_GATE_IDS {
        return Err(invalid(format!(
            "gate_ids must contain between 1 and {MAX_LAUNCH_GRANT_GATE_IDS} gates"
        )));
    }
    let unique = gate_ids.iter().collect::<BTreeSet<_>>();
    if unique.len() != gate_ids.len() {
        return Err(invalid("gate_ids must be unique"));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> WireError {
    launch_error("LAUNCH_GRANT_INVALID", message)
}
