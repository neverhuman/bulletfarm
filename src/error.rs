#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("INVALID_CONTRACT: {0}")]
    InvalidContract(String),
    #[error("UNKNOWN_DEPENDENCY: {0}")]
    UnknownDependency(String),
    #[error("DEPENDENCY_INVALID: {0}")]
    DependencyInvalid(String),
    #[error("UNKNOWN_PROFILE: {0}")]
    UnknownProfile(String),
    #[error("CHECK_MISSING: {0}")]
    CheckMissing(String),
    #[error("STALE_VERSION")]
    StaleVersion,
    #[error("STALE_GENERATION")]
    StaleGeneration,
    #[error("STALE_SELECTION")]
    StaleSelection,
    #[error("POLICY_DENIED: {0}")]
    PolicyDenied(String),
    #[error("AUTH_REQUIRED")]
    AuthRequired,
    #[error("BUDGET_UNAVAILABLE: {0}")]
    BudgetUnavailable(String),
    #[error("RESOURCE_CONFLICT: {0}")]
    ResourceConflict(String),
    #[error("OUTCOME_UNKNOWN: {0}")]
    OutcomeUnknown(String),
    #[error("STORAGE_UNAVAILABLE: {0}")]
    StorageUnavailable(String),
    /// The same (actor, command_id) was reused with different bytes. On the wire this is
    /// the spec's `RESOURCE_CONFLICT` (§20.2-d closed code list): the conflicting resource
    /// is the actor's command id. The variant stays distinct so the kernel and CLI can
    /// tell key reuse from other conflicts.
    #[error("RESOURCE_CONFLICT: same command_id with a different body")]
    CommandConflict,
    #[error("{0}")]
    Other(String),
}

impl Error {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidContract(_) => "INVALID_CONTRACT",
            Self::UnknownDependency(_) => "UNKNOWN_DEPENDENCY",
            Self::DependencyInvalid(_) => "DEPENDENCY_INVALID",
            Self::UnknownProfile(_) => "UNKNOWN_PROFILE",
            Self::CheckMissing(_) => "CHECK_MISSING",
            Self::StaleVersion => "STALE_VERSION",
            Self::StaleGeneration => "STALE_GENERATION",
            Self::StaleSelection => "STALE_SELECTION",
            Self::PolicyDenied(_) => "POLICY_DENIED",
            Self::AuthRequired => "AUTH_REQUIRED",
            Self::BudgetUnavailable(_) => "BUDGET_UNAVAILABLE",
            Self::ResourceConflict(_) | Self::CommandConflict => "RESOURCE_CONFLICT",
            Self::OutcomeUnknown(_) => "OUTCOME_UNKNOWN",
            Self::StorageUnavailable(_) => "STORAGE_UNAVAILABLE",
            Self::Other(_) => "ERROR",
        }
    }

    /// HTTP status for the code. Rendering (body, correlation id, log line) lives in `api.rs`.
    pub(crate) fn status(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;
        match self {
            Self::AuthRequired => StatusCode::UNAUTHORIZED,
            Self::PolicyDenied(_) => StatusCode::FORBIDDEN,
            Self::CommandConflict
            | Self::StaleVersion
            | Self::StaleGeneration
            | Self::StaleSelection => StatusCode::CONFLICT,
            Self::BudgetUnavailable(_) | Self::ResourceConflict(_) => StatusCode::CONFLICT,
            Self::StorageUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::OutcomeUnknown(_) => StatusCode::ACCEPTED,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Self {
        Self::StorageUnavailable(value.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Other(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::InvalidContract(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
