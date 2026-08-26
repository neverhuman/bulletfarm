use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_TTL_SECONDS: u64 = 600;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GroupReceipt {
    pub claim_id: String,
    pub committed_paths: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Record {
    Claim {
        schema_version: u32,
        at_unix_ms: u64,
        claim_id: String,
        agent: String,
        lane: String,
        repo: String,
        paths: Vec<String>,
        expires_unix_ms: u64,
    },
    Heartbeat {
        schema_version: u32,
        at_unix_ms: u64,
        claim_id: String,
        agent: String,
        expires_unix_ms: u64,
        note: Option<String>,
    },
    Handoff {
        schema_version: u32,
        at_unix_ms: u64,
        claim_id: String,
        agent: String,
        proof_command: String,
        proof_exit_code: i32,
        changed_paths: Vec<String>,
        commit_oid: Option<String>,
    },
    CommitReceipt {
        schema_version: u32,
        at_unix_ms: u64,
        claim_id: String,
        orchestrator: String,
        commit_oid: String,
        committed_paths: Vec<String>,
    },
    CommitReceiptCorrection {
        schema_version: u32,
        at_unix_ms: u64,
        claim_id: String,
        orchestrator: String,
        previous_commit_oid: String,
        commit_oid: String,
        committed_paths: Vec<String>,
        reason: String,
    },
    CommitReceiptGroup {
        schema_version: u32,
        at_unix_ms: u64,
        orchestrator: String,
        commit_oid: String,
        receipts: Vec<GroupReceipt>,
    },
    CommitReceiptGroupCorrection {
        schema_version: u32,
        at_unix_ms: u64,
        orchestrator: String,
        previous_commit_oid: String,
        commit_oid: String,
        receipts: Vec<GroupReceipt>,
        reason: String,
    },
}

impl Record {
    pub const fn schema_version(&self) -> u32 {
        match self {
            Self::Claim { schema_version, .. }
            | Self::Heartbeat { schema_version, .. }
            | Self::Handoff { schema_version, .. }
            | Self::CommitReceipt { schema_version, .. }
            | Self::CommitReceiptCorrection { schema_version, .. }
            | Self::CommitReceiptGroup { schema_version, .. }
            | Self::CommitReceiptGroupCorrection { schema_version, .. } => *schema_version,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ClaimSummary {
    pub claim_id: String,
    pub agent: String,
    pub lane: String,
    pub repo: String,
    pub paths: Vec<String>,
    pub claimed_at_unix_ms: u64,
    pub last_event_unix_ms: u64,
    pub expires_unix_ms: u64,
    pub state: ClaimState,
    pub proof_command: Option<String>,
    pub changed_paths: Vec<String>,
    pub commit_oid: Option<String>,
    pub commit_orchestrator: Option<String>,
    pub commit_recorded_at_unix_ms: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimState {
    Active,
    Expired,
    HandedOff,
}

impl ClaimSummary {
    pub fn refresh_state(&mut self, now_unix_ms: u64) {
        if self.state != ClaimState::HandedOff {
            self.state = if self.expires_unix_ms > now_unix_ms {
                ClaimState::Active
            } else {
                ClaimState::Expired
            };
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Status {
    pub schema_version: u32,
    pub source: String,
    pub as_of_unix_ms: u64,
    pub claims: Vec<ClaimSummary>,
}
