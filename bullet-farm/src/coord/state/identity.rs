use serde::Serialize;

use crate::coord::CoordError;

const CLAIM_DOMAIN: &str = "bullet-family.coord.claim.v2";

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct ClaimIdentity<'a> {
    generation_id: &'a str,
    request_id: &'a str,
    agent: &'a str,
    lane: &'a str,
    repo: &'a str,
    normalized_paths: &'a [String],
    ttl_seconds: u64,
}

pub(in crate::coord) fn claim_id(
    generation_id: &str,
    request_id: &str,
    agent: &str,
    lane: &str,
    repo: &str,
    paths: &[String],
    ttl_seconds: u64,
) -> Result<String, CoordError> {
    let bytes = bullet_wire::canonical_json(&ClaimIdentity {
        generation_id,
        request_id,
        agent,
        lane,
        repo,
        normalized_paths: paths,
        ttl_seconds,
    })
    .map_err(wire)?;
    let digest = bullet_wire::hash_framed_bytes(CLAIM_DOMAIN, &bytes).map_err(wire)?;
    Ok(format!("clm_{}", digest.to_hex()))
}

pub(in crate::coord) fn validate_claim_id(value: &str) -> Result<(), CoordError> {
    if value.len() == 68
        && value.starts_with("clm_")
        && value[4..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(CoordError::new(
            "INVALID_CLAIM_ID",
            "claim ID must be clm_ plus 64 lowercase hexadecimal digits",
        ))
    }
}

fn wire(error: bullet_wire::WireError) -> CoordError {
    CoordError::new(
        "INVALID_COORD_COMMAND",
        format!("cannot derive canonical claim identity: {error}"),
    )
}

#[cfg(test)]
mod tests {
    use super::{claim_id, validate_claim_id};

    #[test]
    fn identity_is_process_time_and_order_independent() {
        let paths = vec!["README.md".to_owned(), "src".to_owned()];
        let first = claim_id(
            &format!("gen_{}", "a".repeat(64)),
            &format!("req_{}", "b".repeat(64)),
            "agent-a",
            "lane-a",
            "bullet-farm",
            &paths,
            600,
        )
        .unwrap();
        let second = claim_id(
            &format!("gen_{}", "a".repeat(64)),
            &format!("req_{}", "b".repeat(64)),
            "agent-a",
            "lane-a",
            "bullet-farm",
            &paths,
            600,
        )
        .unwrap();
        assert_eq!(first, second);
        assert_eq!(first.len(), 68);
        validate_claim_id(&first).unwrap();
        assert_ne!(
            first,
            claim_id(
                &format!("gen_{}", "a".repeat(64)),
                &format!("req_{}", "c".repeat(64)),
                "agent-a",
                "lane-a",
                "bullet-farm",
                &paths,
                600,
            )
            .unwrap()
        );
    }
}
