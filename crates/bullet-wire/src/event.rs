use serde::{Deserialize, Serialize};

use crate::{Blake3Digest, CommandId, EventId};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventEnvelope<T> {
    pub schema_version: u32,
    pub id: EventId,
    pub seq: u64,
    pub at: String,
    pub kind: String,
    pub body: T,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot<T> {
    pub data: T,
    pub as_of_sequence: u64,
    pub observed_at: String,
    pub source: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CommandState {
    Pending,
    Applied,
    Verified,
    Failed,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandEnvelope<T> {
    pub schema_version: u32,
    pub command_id: CommandId,
    pub idempotency_key: String,
    pub request_digest: Blake3Digest,
    pub submitted_at: String,
    pub body: T,
}
