use super::*;

#[test]
fn release_tag_parser_is_fail_closed() {
    assert!(
        parse_args(&[
            "generate".into(),
            "--tag".into(),
            "v0.1.0-alpha.1".into(),
            "--subjects".into(),
            "/tmp/subjects.toml".into(),
        ])
        .is_ok()
    );
    for tag in ["alpha", "v1/escape", "v1..\n"] {
        assert!(
            parse_args(&[
                "generate".into(),
                "--tag".into(),
                tag.into(),
                "--subjects".into(),
                "/tmp/subjects.toml".into(),
            ])
            .is_err()
        );
    }
}

#[test]
fn framed_digest_distinguishes_field_boundaries() {
    let digest = |parts: &[&[u8]]| {
        let mut hasher = blake3::Hasher::new();
        for part in parts {
            git::frame(&mut hasher, part);
        }
        hasher.finalize()
    };
    assert_ne!(digest(&[b"ab", b"c"]), digest(&[b"a", b"bc"]));
}

#[test]
fn ssh_status_requires_one_ed25519_identity() {
    let good = "Good \"git\" signature for bot@jekko.ai with ED25519 key SHA256:abc+123\n";
    assert_eq!(
        git::signer_identity(good, "v1").unwrap(),
        "bot@jekko.ai|ed25519|SHA256:abc+123"
    );
    for denied in [
        "",
        "Good \"file\" signature for bot@jekko.ai with ED25519 key SHA256:abc\n",
        "Good \"git\" signature for bot@jekko.ai with RSA key SHA256:abc\n",
        "Good \"git\" signature for bad principal with ED25519 key SHA256:abc\n",
    ] {
        assert!(git::signer_identity(denied, "v1").is_err());
    }
}

#[test]
fn family_root_resolves_from_split_root_and_hub_checkout() {
    let outer =
        std::env::temp_dir().join(format!("bullet-family-lock-root-{}", std::process::id()));
    if outer.exists() {
        fs::remove_dir_all(&outer).unwrap();
    }
    let hub = outer.join("bullet-farm");
    fs::create_dir_all(&hub).unwrap();
    fs::write(
        outer.join("repos.manifest.toml"),
        "family = \"bullet-farm\"\n",
    )
    .unwrap();
    assert_eq!(resolve_family_root(&hub).unwrap(), outer);
    assert_eq!(resolve_family_root(&outer).unwrap(), outer);
    fs::remove_dir_all(outer).unwrap();
}
