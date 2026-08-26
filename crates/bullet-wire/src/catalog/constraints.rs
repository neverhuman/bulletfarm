use serde_json::{Value, json};

pub(super) fn conditional_constraints(record_name: &str) -> Option<Value> {
    match record_name {
        "AuthorityClaimsV1" | "MutationPermitClaimsV1" => Some(operation_audience_constraints()),
        "LaunchGrantClaimsV1" => Some(super::launch::launch_grant_claims_constraints()),
        "PatchOperationV1" => Some(json!([
            {
                "if": {"properties": {"preimage_kind": {"const": "absent"}}},
                "then": {"properties": {"preimage_digest": {"type": "null"}}}
            },
            {
                "if": {"properties": {"preimage_kind": {"const": "digest"}}},
                "then": {"properties": {"preimage_digest": digest_schema()}}
            },
            {
                "if": {"properties": {"mutation_kind": {"const": "write"}}},
                "then": {"properties": {"content_utf8": {"type": "string"}}}
            },
            {
                "if": {"properties": {"mutation_kind": {"const": "delete"}}},
                "then": {
                    "properties": {
                        "content_utf8": {"type": "null"},
                        "preimage_kind": {"const": "digest"}
                    }
                }
            }
        ])),
        "FinalAuthorityDecisionV1" => Some(json!([
            {
                "if": {"properties": {"decision": {"const": "authorized"}}},
                "then": {
                    "properties": {
                        "replay": {"enum": ["fresh", "exact-replay"]},
                        "reservation_id": digest_id_schema("rsv"),
                        "permit": schema_ref("SignedMutationPermitV1"),
                        "replay_result": {"type": "null"},
                        "reason_code": {"type": "null"}
                    }
                }
            },
            {
                "if": {"properties": {"decision": {"const": "settled"}}},
                "then": {
                    "properties": {
                        "replay": {"const": "exact-replay"},
                        "reservation_id": digest_id_schema("rsv"),
                        "permit": {"type": "null"},
                        "replay_result": schema_ref("MutationReplayResultV1"),
                        "reason_code": {"type": "null"}
                    }
                }
            },
            {
                "if": {"properties": {"decision": {"const": "refused"}}},
                "then": {
                    "properties": {
                        "reservation_id": {"type": "null"},
                        "permit": {"type": "null"},
                        "replay_result": {"type": "null"},
                        "reason_code": reason_code_schema()
                    }
                }
            }
        ])),
        "MutationReplayResultV1" => Some(json!([
            {
                "if": {"properties": {"state": {"const": "in-flight"}}},
                "then": {
                    "properties": {
                        "result_digest": {"type": "null"},
                        "completed_at_unix_ms": {"type": "null"}
                    }
                }
            },
            {
                "if": {
                    "properties": {
                        "state": {"enum": ["committed", "aborted", "unknown"]}
                    }
                },
                "then": {
                    "properties": {
                        "result_digest": digest_schema(),
                        "completed_at_unix_ms": timestamp_schema()
                    }
                }
            }
        ])),
        "MutationSettlementResultV1" => Some(json!([
            {
                "if": {"properties": {"status": {"const": "accepted"}}},
                "then": {
                    "properties": {
                        "replay": {"const": "fresh"},
                        "result_digest": digest_schema(),
                        "reason_code": {"type": "null"}
                    }
                }
            },
            {
                "if": {"properties": {"status": {"const": "exact-replay"}}},
                "then": {
                    "properties": {
                        "replay": {"const": "exact-replay"},
                        "result_digest": digest_schema(),
                        "reason_code": {"type": "null"}
                    }
                }
            },
            {
                "if": {"properties": {"status": {"const": "conflict"}}},
                "then": {
                    "properties": {
                        "replay": {"const": "conflict"},
                        "result_digest": {"type": "null"},
                        "reason_code": reason_code_schema()
                    }
                }
            },
            {
                "if": {"properties": {"status": {"const": "refused"}}},
                "then": {
                    "properties": {
                        "result_digest": {"type": "null"},
                        "reason_code": reason_code_schema()
                    }
                }
            }
        ])),
        _ => None,
    }
}

fn operation_audience_constraints() -> Value {
    json!([
        {
            "if": {
                "properties": {
                    "operation": {
                        "enum": [
                            "clone-workspace", "read-workspace", "apply-patch", "checkpoint",
                            "prepare-candidate", "preserve-workspace", "cleanup-workspace"
                        ]
                    }
                }
            },
            "then": {"properties": {"audience": {"const": "bullet-gitd"}}}
        },
        {
            "if": {
                "properties": {
                    "operation": {"enum": ["dispatch-effect", "reconcile-effect"]}
                }
            },
            "then": {"properties": {"audience": {"const": "effect-broker"}}}
        }
    ])
}

fn digest_schema() -> Value {
    json!({"type": "string", "pattern": "^[0-9a-f]{64}$"})
}

fn digest_id_schema(prefix: &str) -> Value {
    json!({"type": "string", "pattern": format!("^{prefix}_[0-9a-f]{{64}}$")})
}

fn schema_ref(name: &str) -> Value {
    json!({"$ref": format!("#/schemas/{name}")})
}

fn reason_code_schema() -> Value {
    json!({
        "type": "string",
        "minLength": 1,
        "maxLength": 128,
        "pattern": "^[A-Za-z0-9._:/-]+$"
    })
}

fn timestamp_schema() -> Value {
    json!({"type": "integer", "minimum": 0, "maximum": 9007199254740991_u64})
}
