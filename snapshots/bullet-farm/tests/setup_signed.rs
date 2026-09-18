#[test]
fn install_authority_rejects_a_circular_hub_member() {
    let lock = concat!(
        "schema_version = \"3\"\n",
        "family = \"bullet-farm\"\n",
        "tag = \"v1.0.0\"\n",
        "schema_bundle_hash = \"blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"\n",
        "[[member]]\n",
        "name = \"bullet-farm\"\n",
        "jeryu_url = \"https://jeryu.example/git/root/bullet-farm.git\"\n",
        "jeryu_slug = \"root/bullet-farm\"\n",
        "tag = \"v1.0.0\"\n",
        "commit_oid = \"sha1:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\"\n",
        "tree_oid = \"sha1:cccccccccccccccccccccccccccccccccccccccc\"\n",
        "release_signing_identity = \"release@bullet.farm|ed25519|SHA256:abc+123=\"\n",
        "[[member.lockfile]]\n",
        "path = \"Cargo.lock\"\n",
        "digest = \"blake3:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd\"\n",
    );
    let error = bullet_family::family_lock::parse(lock.as_bytes())
        .expect_err("the hub tag is the top-level signed subject, never a member self-entry");
    assert_eq!(error.code(), "INVALID_FAMILY_LOCK");
}
