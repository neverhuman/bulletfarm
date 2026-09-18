use serde::Serialize;

use crate::family_lock::FamilyLock;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(super) enum CheckStatus {
    Pass,
    Blocked,
}

#[derive(Debug, Serialize)]
pub(super) struct DoctorCheck {
    pub(super) id: &'static str,
    pub(super) status: CheckStatus,
    pub(super) detail: String,
    pub(super) repair: Option<String>,
}

impl DoctorCheck {
    pub(super) fn pass(id: &'static str, detail: impl Into<String>) -> Self {
        Self {
            id,
            status: CheckStatus::Pass,
            detail: detail.into(),
            repair: None,
        }
    }

    pub(super) fn blocked(
        id: &'static str,
        detail: impl Into<String>,
        repair: impl Into<String>,
    ) -> Self {
        Self {
            id,
            status: CheckStatus::Blocked,
            detail: detail.into(),
            repair: Some(repair.into()),
        }
    }
}

/// Aggregate readiness of one doctor run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DoctorStatus {
    Ready,
    Blocked,
}

impl DoctorStatus {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "READY",
            Self::Blocked => "BLOCKED",
        }
    }

    /// The process exit status that carries the same verdict to a shell.
    /// `BLOCKED` is 3 — the family's "diagnosed, not usable" code, shared with
    /// `check`'s blocked gates (`check/model.rs`) and the coordinator's claim
    /// refusals (`coord/mod.rs`) — so scripting on exit status can never read a
    /// blocked hub as success. Every other failure keeps its typed
    /// `CoordError` code.
    pub(super) const fn exit_code(self) -> u8 {
        match self {
            Self::Ready => 0,
            Self::Blocked => 3,
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct DoctorReport {
    pub(super) schema_version: u32,
    pub(super) command: &'static str,
    pub(super) status: &'static str,
    pub(super) hub_root: String,
    pub(super) family_root: Option<String>,
    pub(super) checks: Vec<DoctorCheck>,
}

#[derive(Debug)]
pub(super) struct DoctorFamilyLock {
    pub(super) schema_version: String,
    pub(super) tag: String,
    pub(super) installable_schema: bool,
    pub(super) current: Option<FamilyLock>,
    pub(super) member: Vec<DoctorLockedMember>,
}

#[derive(Debug)]
pub(super) struct DoctorLockedMember {
    pub(super) name: String,
    pub(super) commit_oid: String,
    pub(super) jeryu_url: Option<String>,
    pub(super) jeryu_slug: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::DoctorStatus;

    /// Both exit paths of the verdict. `READY` is the only one an integration
    /// test cannot reach on this host: it needs a signed schema-3 lock and
    /// clean clones at exact OIDs, which is operator input (ADR 0013 OD-D).
    #[test]
    fn each_status_carries_its_exit_code_and_wire_name() {
        assert_eq!(DoctorStatus::Ready.as_str(), "READY");
        assert_eq!(DoctorStatus::Ready.exit_code(), 0);
        assert_eq!(DoctorStatus::Blocked.as_str(), "BLOCKED");
        assert_eq!(
            DoctorStatus::Blocked.exit_code(),
            3,
            "BLOCKED must use the family's diagnosed-not-usable code"
        );
    }
}
