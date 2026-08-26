//! Immutable local command catalog. Model or repository content cannot add commands.

use std::time::Duration;

use super::model::{CheckTier, GateClass};

pub(super) const BASH_BIN: &str = "/bin/bash";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SubjectScope {
    Repository(&'static str),
    Family,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CommandGate {
    pub id: &'static str,
    pub class: GateClass,
    pub scope: SubjectScope,
    pub repository: &'static str,
    pub script: &'static str,
    pub arguments: &'static [&'static str],
    pub timeout: Duration,
}

const FAST_TIMEOUT: Duration = Duration::from_secs(60);
const REQUIRED_TIMEOUT: Duration = Duration::from_secs(1_800);

const FAST: &[CommandGate] = &[
    gate(
        "fast.hub",
        GateClass::Component,
        SubjectScope::Repository("bullet-farm"),
        "bullet-farm",
        "scripts/ci-local.sh",
        &["fast"],
        FAST_TIMEOUT,
    ),
    gate(
        "fast.kernel",
        GateClass::Component,
        SubjectScope::Repository("bullet-kernel"),
        "bullet-kernel",
        "scripts/ci-local.sh",
        &["fast"],
        FAST_TIMEOUT,
    ),
    gate(
        "fast.bullet-git",
        GateClass::Component,
        SubjectScope::Repository("bullet-git"),
        "bullet-git",
        "scripts/ci-local.sh",
        &["fast"],
        FAST_TIMEOUT,
    ),
    gate(
        "fast.portal",
        GateClass::Component,
        SubjectScope::Repository("bullet-portal"),
        "bullet-portal",
        "scripts/ci-local.sh",
        &["fast"],
        FAST_TIMEOUT,
    ),
    gate(
        "fast.generated-drift",
        GateClass::Component,
        SubjectScope::Family,
        "bullet-farm",
        "scripts/sync-family-contracts.sh",
        &["check"],
        FAST_TIMEOUT,
    ),
];

const REQUIRED: &[CommandGate] = &[
    gate(
        "required.family-contract",
        GateClass::Component,
        SubjectScope::Family,
        "bullet-farm",
        "ops/ci/family-contract.sh",
        &[],
        REQUIRED_TIMEOUT,
    ),
    gate(
        "required.demo-component",
        GateClass::Component,
        SubjectScope::Family,
        "bullet-farm",
        "scripts/demo.sh",
        &[],
        REQUIRED_TIMEOUT,
    ),
];

const fn gate(
    id: &'static str,
    class: GateClass,
    scope: SubjectScope,
    repository: &'static str,
    script: &'static str,
    arguments: &'static [&'static str],
    timeout: Duration,
) -> CommandGate {
    CommandGate {
        id,
        class,
        scope,
        repository,
        script,
        arguments,
        timeout,
    }
}

pub(super) const fn commands(tier: CheckTier) -> &'static [CommandGate] {
    match tier {
        CheckTier::Fast => FAST,
        CheckTier::Required => REQUIRED,
        CheckTier::Release => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogs_are_fixed_nonempty_and_release_executes_nothing() {
        assert_eq!(commands(CheckTier::Fast).len(), 5);
        assert_eq!(commands(CheckTier::Required).len(), 2);
        assert!(commands(CheckTier::Release).is_empty());
        for gate in FAST.iter().chain(REQUIRED) {
            assert!(gate.script.ends_with(".sh"));
            assert!(!gate.script.starts_with('/'));
            assert!(!gate.script.contains(".."));
        }
        assert!(FAST.iter().all(|gate| gate.class == GateClass::Component));
        assert_eq!(REQUIRED[0].class, GateClass::Component);
        assert_eq!(REQUIRED[1].class, GateClass::Component);
    }
}
