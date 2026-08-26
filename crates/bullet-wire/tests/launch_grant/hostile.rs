//! Hostile tokens: wrong key, wrong footer purpose, wrong implicit assertion,
//! tampered bytes, unknown fields, duplicate keys, and non-canonical payloads.
//! Nonce replay is deliberately not refused here: the Kernel persists consumed
//! nonces and refuses `LAUNCH_GRANT_REPLAYED` itself.

use bullet_wire::{AuthorityAudience, AuthorityVerificationKey, SignedLaunchGrant, canonical_json};
use pasetors::{
    keys::AsymmetricSecretKey,
    version4::{PublicToken, V4},
};

use super::{ISSUER, KEY_ID, NOT_BEFORE, SECRET_KEY, claims, expectation, signer, verifier};

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
const LAUNCH_ASSERTION: &[u8] = b"bullet-farm.launch-grant.v1alpha1";
/// RFC 8032 section 7.1 test 1 keypair (seed || public): a valid but untrusted signer.
const RFC8032_TEST_1_SECRET_KEY: &str = "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";

pub(crate) fn b64url_encode(bytes: &[u8]) -> String {
    let mut output = String::new();
    for chunk in bytes.chunks(3) {
        let mut buffer = [0_u8; 3];
        buffer[..chunk.len()].copy_from_slice(chunk);
        let bits = u32::from(buffer[0]) << 16 | u32::from(buffer[1]) << 8 | u32::from(buffer[2]);
        for index in 0..=chunk.len() {
            let shift = 18 - 6 * index;
            output.push(ALPHABET[((bits >> shift) & 63) as usize] as char);
        }
    }
    output
}

pub(crate) fn b64url_decode(text: &str) -> Vec<u8> {
    let mut output = Vec::new();
    let mut accumulator = 0_u32;
    let mut bits = 0;
    for byte in text.bytes() {
        let value = ALPHABET.iter().position(|entry| *entry == byte).unwrap() as u32;
        accumulator = (accumulator << 6) | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push(((accumulator >> bits) & 0xff) as u8);
        }
    }
    output
}

/// Replace the signed payload while keeping the original signature and footer.
pub(crate) fn splice_payload(paseto: &str, payload: &[u8]) -> String {
    let segments = paseto.split('.').collect::<Vec<_>>();
    let signed = b64url_decode(segments[2]);
    let signature = &signed[signed.len() - 64..];
    let mut message = payload.to_vec();
    message.extend_from_slice(signature);
    format!(
        "{}.{}.{}.{}",
        segments[0],
        segments[1],
        b64url_encode(&message),
        segments[3]
    )
}

pub(crate) fn sign_raw(
    payload: &[u8],
    purpose: &str,
    assertion: &[u8],
    secret: &[u8; 64],
) -> SignedLaunchGrant {
    let secret = AsymmetricSecretKey::<V4>::from(secret).unwrap();
    let footer = canonical_json(&serde_json::json!({
        "issuer": ISSUER,
        "key_id": KEY_ID,
        "purpose": purpose,
        "schema_version": "v1alpha1",
    }))
    .unwrap();
    SignedLaunchGrant {
        schema_version: "v1alpha1".to_owned(),
        issuer: ISSUER.to_owned(),
        key_id: KEY_ID.to_owned(),
        paseto: PublicToken::sign(&secret, payload, Some(&footer), Some(assertion)).unwrap(),
    }
}

fn code(grant: &SignedLaunchGrant) -> &'static str {
    grant
        .verify(&verifier(), &expectation(&claims()), NOT_BEFORE)
        .unwrap_err()
        .code()
}

#[test]
fn base64url_codec_round_trips() {
    for length in 0..8 {
        let bytes = (0..length)
            .map(|index| index as u8 * 37 + 1)
            .collect::<Vec<_>>();
        assert_eq!(b64url_decode(&b64url_encode(&bytes)), bytes);
    }
}

#[test]
fn wrong_key_material_or_identity_is_refused() {
    let grant = signer().sign_launch_grant(&claims()).unwrap();
    let rogue_secret: [u8; 64] = hex::decode(RFC8032_TEST_1_SECRET_KEY)
        .unwrap()
        .try_into()
        .unwrap();
    let rogue = sign_raw(
        &canonical_json(&claims()).unwrap(),
        "launch-grant-signing",
        LAUNCH_ASSERTION,
        &rogue_secret,
    );
    assert_eq!(code(&rogue), "LAUNCH_GRANT_INVALID");

    let other_key =
        AuthorityVerificationKey::from_bytes(ISSUER, "authority-test-2", &super::PUBLIC_KEY)
            .unwrap();
    assert_eq!(
        grant
            .verify(&other_key, &expectation(&claims()), NOT_BEFORE)
            .unwrap_err()
            .code(),
        "LAUNCH_GRANT_KEY_UNKNOWN"
    );
    let other_issuer =
        AuthorityVerificationKey::from_bytes("another-kernel", KEY_ID, &super::PUBLIC_KEY).unwrap();
    assert_eq!(
        grant
            .verify(&other_issuer, &expectation(&claims()), NOT_BEFORE)
            .unwrap_err()
            .code(),
        "LAUNCH_GRANT_KEY_UNKNOWN"
    );
    let relabelled = SignedLaunchGrant {
        key_id: "authority-test-2".to_owned(),
        ..grant.clone()
    };
    assert_eq!(code(&relabelled), "LAUNCH_GRANT_KEY_UNKNOWN");
}

#[test]
fn wrong_footer_purpose_or_implicit_assertion_is_refused() {
    let payload = canonical_json(&claims()).unwrap();
    for purpose in [
        "authority-signing",
        "mutation-permit-signing",
        "release-signing",
    ] {
        let grant = sign_raw(&payload, purpose, LAUNCH_ASSERTION, &SECRET_KEY);
        assert_eq!(code(&grant), "LAUNCH_GRANT_INVALID", "purpose {purpose}");
    }
    for assertion in [
        b"bullet-farm.authority.v1alpha1".as_slice(),
        b"bullet-farm.mutation-permit.v1alpha1".as_slice(),
        b"".as_slice(),
    ] {
        let grant = sign_raw(&payload, "launch-grant-signing", assertion, &SECRET_KEY);
        assert_eq!(code(&grant), "LAUNCH_GRANT_INVALID");
    }
    let exact = sign_raw(
        &payload,
        "launch-grant-signing",
        LAUNCH_ASSERTION,
        &SECRET_KEY,
    );
    assert_eq!(exact, signer().sign_launch_grant(&claims()).unwrap());
}

#[test]
fn tampered_or_malformed_token_bytes_are_refused() {
    let grant = signer().sign_launch_grant(&claims()).unwrap();
    let mut flipped = grant.clone();
    let last = flipped.paseto.pop().unwrap();
    flipped.paseto.push(if last == 'A' { 'B' } else { 'A' });
    assert_eq!(code(&flipped), "LAUNCH_GRANT_INVALID");

    let truncated = SignedLaunchGrant {
        paseto: grant.paseto[..grant.paseto.len() - 8].to_owned(),
        ..grant.clone()
    };
    assert_eq!(code(&truncated), "LAUNCH_GRANT_INVALID");
    let local = SignedLaunchGrant {
        paseto: grant.paseto.replacen("v4.public.", "v4.local.", 1),
        ..grant.clone()
    };
    assert_eq!(code(&local), "LAUNCH_GRANT_INVALID");
    let oversize = SignedLaunchGrant {
        paseto: format!("v4.public.{}", "A".repeat(32_768)),
        ..grant.clone()
    };
    assert_eq!(code(&oversize), "LAUNCH_GRANT_INVALID");
    assert!(oversize.digest().is_err());
    let wrong_schema = SignedLaunchGrant {
        schema_version: "v2".to_owned(),
        ..grant.clone()
    };
    assert_eq!(code(&wrong_schema), "LAUNCH_GRANT_INVALID");
    let spliced = SignedLaunchGrant {
        paseto: splice_payload(&grant.paseto, &canonical_json(&claims()).unwrap()),
        ..grant.clone()
    };
    assert_eq!(
        spliced
            .verify(&verifier(), &expectation(&claims()), NOT_BEFORE)
            .unwrap(),
        claims()
    );
}

#[test]
fn unknown_field_duplicate_key_and_noncanonical_payloads_are_refused() {
    let mut unknown = serde_json::to_value(claims()).unwrap();
    unknown["surprise"] = serde_json::json!(true);
    let grant = sign_raw(
        &canonical_json(&unknown).unwrap(),
        "launch-grant-signing",
        LAUNCH_ASSERTION,
        &SECRET_KEY,
    );
    let error = grant
        .verify(&verifier(), &expectation(&claims()), NOT_BEFORE)
        .unwrap_err();
    assert_eq!(error.code(), "LAUNCH_GRANT_INVALID");
    assert!(error.reason().contains("DOCUMENT_SCHEMA_INVALID"));

    let canonical = String::from_utf8(canonical_json(&claims()).unwrap()).unwrap();
    let duplicated = canonical.replacen(
        "{\"adapter\":\"claude-stream-json-v1\",",
        "{\"adapter\":\"claude-stream-json-v1\",\"adapter\":\"claude-stream-json-v1\",",
        1,
    );
    assert_ne!(duplicated, canonical);
    let grant = sign_raw(
        duplicated.as_bytes(),
        "launch-grant-signing",
        LAUNCH_ASSERTION,
        &SECRET_KEY,
    );
    let error = grant
        .verify(&verifier(), &expectation(&claims()), NOT_BEFORE)
        .unwrap_err();
    assert_eq!(error.code(), "LAUNCH_GRANT_INVALID");
    assert!(error.reason().contains("DUPLICATE_JSON_KEY"));

    let pretty = serde_json::to_vec_pretty(&claims()).unwrap();
    let grant = sign_raw(
        &pretty,
        "launch-grant-signing",
        LAUNCH_ASSERTION,
        &SECRET_KEY,
    );
    let error = grant
        .verify(&verifier(), &expectation(&claims()), NOT_BEFORE)
        .unwrap_err();
    assert_eq!(error.code(), "LAUNCH_GRANT_INVALID");
    assert!(error.reason().contains("NON_CANONICAL_JSON"));
}

#[test]
fn audience_expectation_mismatch_is_typed_and_nonce_replay_is_delegated() {
    let grant = signer().sign_launch_grant(&claims()).unwrap();
    let mut expected = expectation(&claims());
    expected.audience = AuthorityAudience::BulletGitd;
    assert_eq!(
        grant
            .verify(&verifier(), &expected, NOT_BEFORE)
            .unwrap_err()
            .code(),
        "LAUNCH_GRANT_AUDIENCE_MISMATCH"
    );
    let expected = expectation(&claims());
    let first = grant.verify(&verifier(), &expected, NOT_BEFORE).unwrap();
    let second = grant.verify(&verifier(), &expected, NOT_BEFORE).unwrap();
    assert_eq!(first.grant_nonce, second.grant_nonce);
}
