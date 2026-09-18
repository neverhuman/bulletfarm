//! Fail-closed validation of the public hub's onboarding surface.

use std::{fs, path::Path};

use bullet_wire::decode_unique_value;
use serde_json::Value;

use crate::{coord::CoordError, doctor::discover_hub};

const USAGE: &str = "usage: bullet-family [--root PATH] hub check";
const REQUIRED_README: &[&str] = &[
    "Many minds. One verified line to main.",
    "Current alpha:",
    "just preview",
    "just dev",
    "contract-tested / live blocked",
    "TRANSACTION_PROOF",
    "What we will not claim",
    "BulletGit",
    "family.lock",
];
const REQUIRED_FILES: &[&str] = &[
    "README.md",
    "AGENTS.md",
    "SPLIT.md",
    "Justfile",
    "Cargo.toml",
    "Cargo.lock",
    "repos.manifest.toml",
    "family.lock",
    "scripts/fuse.sh",
    "scripts/demo.sh",
    "ops/ci/family.sh",
    "docs/architecture/overview.md",
    "src/main.rs",
    "src/coord/store.rs",
    "src/deps_check.rs",
    "src/family_lock.rs",
    "src/hub_check.rs",
    "release/allowed_signers",
];

pub fn run(
    current_dir: &Path,
    explicit_root: Option<&str>,
    args: &[String],
) -> Result<String, CoordError> {
    if args != ["check"] {
        return Err(CoordError::new("USAGE", USAGE));
    }
    let hub = discover_hub(current_dir, explicit_root)?;
    validate(&hub)?;
    Ok("hub-check: ok".to_owned())
}

fn validate(hub: &Path) -> Result<(), CoordError> {
    for relative in REQUIRED_FILES {
        if !hub.join(relative).is_file() {
            return Err(failed(format!("missing {relative}")));
        }
    }
    let readme = fs::read_to_string(hub.join("README.md")).map_err(CoordError::io)?;
    for phrase in REQUIRED_README {
        if !readme.contains(phrase) {
            return Err(failed(format!(
                "README.md missing required phrase: {phrase}"
            )));
        }
    }
    let Some((claims, _)) = readme.split_once("## What we will not claim") else {
        return Err(failed("README.md missing 'What we will not claim'"));
    };
    let words = claims
        .to_ascii_lowercase()
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if words
        .windows(2)
        .any(|pair| matches!(pair, [left, right] if (left == "100%" && right == "autonomy") || (left == "zero" && right == "regressions")))
    {
        return Err(failed(
            "README.md must not claim 100% autonomy or zero regressions",
        ));
    }
    let owners: Value =
        decode_unique_value(&fs::read(hub.join("agent/owner-map.json")).map_err(CoordError::io)?)
            .map_err(|error| CoordError::new("INVALID_OWNER_MAP", error.to_string()))?;
    if !owners
        .get("owners")
        .and_then(Value::as_object)
        .is_some_and(|entries| entries.contains_key("README.md"))
    {
        return Err(failed("owner-map.json missing README.md"));
    }
    Ok(())
}

fn failed(reason: impl Into<String>) -> CoordError {
    CoordError::new("HUB_CHECK_FAILED", reason)
}
