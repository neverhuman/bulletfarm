use crate::coord::CoordError;

pub(crate) fn validate_jeryu_source(url: &str, slug: &str) -> Result<(), CoordError> {
    validate_slug(slug)?;
    if url.is_empty()
        || url.len() > 2048
        || !url.is_ascii()
        || url
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace())
        || url.contains(['\\', '?', '#'])
    {
        return Err(invalid("Jeryu source URL is malformed"));
    }
    let (remainder, loopback_only) = if let Some(rest) = url.strip_prefix("https://") {
        (rest, false)
    } else if let Some(rest) = url.strip_prefix("ssh://") {
        (rest, false)
    } else if let Some(rest) = url.strip_prefix("http://") {
        (rest, true)
    } else {
        return Err(invalid(
            "Jeryu source URL must use HTTPS/SSH, or HTTP on an explicit loopback host",
        ));
    };
    let (authority, path) = remainder
        .split_once('/')
        .ok_or_else(|| invalid("Jeryu source URL must contain a repository path"))?;
    if authority.is_empty() || authority.contains('@') {
        return Err(invalid(
            "Jeryu source URL must have a host and must not embed credentials",
        ));
    }
    if loopback_only && !valid_loopback_authority(authority) {
        return Err(invalid(
            "plain HTTP is permitted only for an explicit loopback host",
        ));
    }
    let expected_path = format!("git/{slug}.git");
    if path != expected_path {
        return Err(invalid(
            "Jeryu source URL path must be exactly /git/<jeryu_slug>.git",
        ));
    }
    Ok(())
}

pub(crate) fn validate_repository_path(path: &str) -> Result<(), CoordError> {
    if path.is_empty()
        || path.len() > 4096
        || !path.is_ascii()
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains('\\')
        || path.bytes().any(|byte| byte.is_ascii_control())
        || path.split('/').any(|segment| {
            segment.is_empty()
                || matches!(segment, "." | "..")
                || segment.eq_ignore_ascii_case(".git")
        })
    {
        return Err(invalid(format!("unsafe repository path: {path:?}")));
    }
    Ok(())
}

pub(crate) fn validate_tag(tag: &str) -> Result<(), CoordError> {
    if tag.len() > 128
        || !tag.starts_with('v')
        || !tag.as_bytes().get(1).is_some_and(u8::is_ascii_digit)
        || tag.ends_with(['.', '-'])
        || tag.contains("..")
        || tag
            .bytes()
            .any(|byte| !(byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-')))
    {
        return Err(invalid(
            "release tag must be a bounded v-prefixed ASCII version",
        ));
    }
    Ok(())
}

pub(super) fn validate_atom(label: &str, value: &str, max: usize) -> Result<(), CoordError> {
    if value.is_empty()
        || value.len() > max
        || !value.is_ascii()
        || value
            .bytes()
            .any(|byte| !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'))
    {
        return Err(invalid(format!("{label} is not canonical lowercase ASCII")));
    }
    Ok(())
}

pub(super) fn validate_git_oid(label: &str, oid: &str) -> Result<(), CoordError> {
    let valid = oid
        .strip_prefix("sha1:")
        .is_some_and(|hex| valid_lower_hex(hex, 40))
        || oid
            .strip_prefix("sha256:")
            .is_some_and(|hex| valid_lower_hex(hex, 64));
    if valid {
        Ok(())
    } else {
        Err(invalid(format!(
            "{label} is not an algorithm-tagged Git OID"
        )))
    }
}

pub(super) fn validate_digest(label: &str, digest: &str) -> Result<(), CoordError> {
    if digest
        .strip_prefix("blake3:")
        .is_some_and(|hex| valid_lower_hex(hex, 64))
    {
        Ok(())
    } else {
        Err(invalid(format!("{label} is not a full BLAKE3 digest")))
    }
}

pub(super) fn validate_signing_identity(identity: &str) -> Result<(), CoordError> {
    let mut parts = identity.split('|');
    let principal = parts.next().unwrap_or_default();
    let algorithm = parts.next().unwrap_or_default();
    let fingerprint = parts.next().unwrap_or_default();
    if parts.next().is_some()
        || principal.is_empty()
        || principal.len() > 256
        || principal
            .bytes()
            .any(|byte| byte.is_ascii_whitespace() || byte.is_ascii_control())
        || algorithm != "ed25519"
        || !fingerprint.starts_with("SHA256:")
        || fingerprint.len() > 128
        || fingerprint.bytes().any(|byte| {
            !(byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'+' | b'/' | b'='))
        })
    {
        return Err(invalid("release signing identity is malformed"));
    }
    Ok(())
}

pub(super) fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_FAMILY_LOCK", reason)
}

fn valid_loopback_authority(authority: &str) -> bool {
    if let Some(rest) = authority.strip_prefix("[::1]") {
        return valid_optional_port(rest);
    }
    for host in ["127.0.0.1", "localhost"] {
        if let Some(rest) = authority.strip_prefix(host) {
            return valid_optional_port(rest);
        }
    }
    false
}

fn valid_optional_port(rest: &str) -> bool {
    if rest.is_empty() {
        return true;
    }
    rest.strip_prefix(':').is_some_and(|port| {
        !port.is_empty()
            && port.bytes().all(|byte| byte.is_ascii_digit())
            && port
                .bytes()
                .try_fold(0_u32, |number, byte| {
                    number.checked_mul(10)?.checked_add(u32::from(byte - b'0'))
                })
                .is_some_and(|port| port != 0 && port <= u32::from(u16::MAX))
    })
}

fn validate_slug(slug: &str) -> Result<(), CoordError> {
    if slug.len() > 256
        || slug.split('/').count() != 2
        || slug.split('/').any(|part| {
            part.is_empty()
                || part.starts_with('.')
                || part.ends_with('.')
                || part.bytes().any(|byte| {
                    !(byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
                })
        })
    {
        return Err(invalid(
            "Jeryu slug must be a canonical owner/repository pair",
        ));
    }
    Ok(())
}

fn valid_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
