use crate::coord::CoordError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::coord::recovery_manifest) struct ClockObservation {
    pub(in crate::coord::recovery_manifest) unix_ms: u64,
    pub(in crate::coord::recovery_manifest) boottime_ms: u64,
    pub(in crate::coord::recovery_manifest) boot_id: String,
    pub(in crate::coord::recovery_manifest) time_namespace_device: u64,
    pub(in crate::coord::recovery_manifest) time_namespace_inode: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AuthorizationWindow {
    Active,
    Expired,
    NotYetValid,
}

pub(in crate::coord::recovery_manifest) struct VerifiedAuthorization {
    pub(in crate::coord::recovery_manifest) recovery_operator: String,
    pub(in crate::coord::recovery_manifest) policy_sha256: String,
    pub(in crate::coord::recovery_manifest) operator_decision_sha256: String,
    pub(in crate::coord::recovery_manifest) replay_contract_version: u32,
    pub(in crate::coord::recovery_manifest) replay_contract_sha256: String,
    pub(in crate::coord::recovery_manifest) bootstrap_commit_oid: String,
    pub(in crate::coord::recovery_manifest) bootstrap_paths: Vec<String>,
    pub(in crate::coord::recovery_manifest) decision_at_unix_ms: u64,
    pub(in crate::coord::recovery_manifest) authority_boot_id: String,
    pub(in crate::coord::recovery_manifest) authority_time_namespace_device: u64,
    pub(in crate::coord::recovery_manifest) authority_time_namespace_inode: u64,
    pub(in crate::coord::recovery_manifest) authorized_at_unix_ms: u64,
    pub(in crate::coord::recovery_manifest) expires_at_unix_ms: u64,
    pub(in crate::coord::recovery_manifest) authorized_at_boottime_ms: u64,
    pub(in crate::coord::recovery_manifest) expires_at_boottime_ms: u64,
}

impl VerifiedAuthorization {
    pub(in crate::coord::recovery_manifest) fn require_active(
        &self,
        clock: ClockObservation,
    ) -> Result<(), CoordError> {
        match self.window(&clock)? {
            AuthorizationWindow::Active => Ok(()),
            AuthorizationWindow::Expired => Err(CoordError::new(
                "RECOVERY_AUTHORIZATION_EXPIRED",
                "recovery authorization expired before the mutation boundary",
            )),
            AuthorizationWindow::NotYetValid => Err(CoordError::new(
                "RECOVERY_AUTHORIZATION_NOT_YET_VALID",
                "recovery authorization is not yet valid",
            )),
        }
    }

    pub(in crate::coord::recovery_manifest) fn require_read_only_replay(
        &self,
        clock: ClockObservation,
    ) -> Result<(), CoordError> {
        match self.window(&clock)? {
            AuthorizationWindow::Active | AuthorizationWindow::Expired => Ok(()),
            AuthorizationWindow::NotYetValid => Err(CoordError::new(
                "RECOVERY_AUTHORIZATION_NOT_YET_VALID",
                "recovery authorization is not yet valid",
            )),
        }
    }

    fn window(&self, clock: &ClockObservation) -> Result<AuthorizationWindow, CoordError> {
        if clock.boot_id != self.authority_boot_id {
            return Err(CoordError::new(
                "RECOVERY_AUTHORIZATION_BOOT_CHANGED",
                "recovery authorization names a different Linux boot epoch",
            ));
        }
        if (clock.time_namespace_device, clock.time_namespace_inode)
            != (
                self.authority_time_namespace_device,
                self.authority_time_namespace_inode,
            )
        {
            return Err(CoordError::new(
                "RECOVERY_TIME_NAMESPACE_CHANGED",
                "recovery authorization names a different Linux time namespace",
            ));
        }
        Ok(
            if clock.unix_ms < self.authorized_at_unix_ms
                || clock.boottime_ms < self.authorized_at_boottime_ms
            {
                AuthorizationWindow::NotYetValid
            } else if clock.unix_ms >= self.expires_at_unix_ms
                || clock.boottime_ms >= self.expires_at_boottime_ms
            {
                AuthorizationWindow::Expired
            } else {
                AuthorizationWindow::Active
            },
        )
    }
}
