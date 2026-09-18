use serde_json::{Value, json};

/// Unconditional JSON-Schema bounds for `LaunchGrantClaimsV1` that the field
/// vocabulary alone cannot express: the fixed audience and operation, the
/// closed provider set, label and path shapes, positive budgets, and the gate
/// list bounds. They mirror `LaunchGrantClaims::validate_shape`.
pub(super) fn launch_grant_claims_constraints() -> Value {
    json!([
        {
            "properties": {
                "audience": {"const": "provider-runner"},
                "operation": {"const": "launch-provider"},
                "provider": {"enum": ["claude", "codex", "cursor", "agy"]},
                "issuer": label_schema(),
                "key_id": label_schema(),
                "adapter": label_schema(),
                "model": label_schema(),
                "protocol": {
                    "type": "string",
                    "minLength": 1,
                    "maxLength": 64,
                    "pattern": "^[a-z][a-z0-9_]{0,63}$"
                },
                "executable_path": {
                    "type": "string",
                    "minLength": 1,
                    "maxLength": 4096,
                    "pattern": "^/(?!.*(?:^|/)\\.{1,2}(?:/|$))(?!.*//)(?!.*/$)[^\\x00-\\x1f\\x7f\\u0080-\\u009f]+$"
                }
            }
        },
        {
            "properties": {
                "attempt_fence": {"minimum": 1},
                "max_invocations": {"minimum": 1},
                "max_wall_clock_ms": {"minimum": 1}
            }
        },
        {
            "properties": {
                "gate_ids": {"minItems": 1, "maxItems": 16, "uniqueItems": true}
            }
        }
    ])
}

fn label_schema() -> Value {
    json!({
        "type": "string",
        "minLength": 1,
        "maxLength": 128,
        "pattern": "^[A-Za-z0-9._:/-]+$"
    })
}
