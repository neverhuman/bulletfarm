#[test]
fn unique_decode_admits_formatting_but_never_ambiguous_members_or_numbers() {
    let value = decode_unique_value(b"{\n  \"b\": 2,\n  \"a\": 1\n}\n").unwrap();
    assert_eq!(value, serde_json::json!({"a": 1, "b": 2}));
    for hostile in [
        br#"{"status":"FAIL","status":"PASS"}"#.as_slice(),
        br#"{"number":NaN}"#.as_slice(),
        br#"{"number":Infinity}"#.as_slice(),
        br#"{"number":-Infinity}"#.as_slice(),
        br#"{"number":1e9999}"#.as_slice(),
        br#"{"number":1e-9999}"#.as_slice(),
        br#"{"number":18446744073709551616}"#.as_slice(),
        br#"{"number":9007199254740992.0}"#.as_slice(),
        br#"{"number":0.10000000000000001}"#.as_slice(),
    ] {
        assert!(decode_unique_value(hostile).is_err(), "{hostile:?}");
    }

    for ordinary in [
        br#"{"number":0.1}"#.as_slice(),
        br#"{"number":1.50}"#.as_slice(),
        br#"{"number":1e2}"#.as_slice(),
        br#"{"number":9007199254740991.0}"#.as_slice(),
    ] {
        assert!(decode_unique_value(ordinary).is_ok(), "{ordinary:?}");
    }

    let duplicate_member = "credential_ghp_1234567890abcdef";
    let hostile = format!(r#"{{"{duplicate_member}":"first","{duplicate_member}":"second"}}"#);
    let error = decode_unique_value(hostile.as_bytes()).unwrap_err();
    assert_eq!(error.code(), "DUPLICATE_JSON_KEY");
    assert!(!error.to_string().contains(duplicate_member));

    let (family, sources, production, test_module_sites) = include!("metadata.rs");
    assert_eq!(
        test_module_sites,
        BTreeMap::from([
            (
                PathBuf::from("crates/bullet-wire/src/authority/request.rs"),
                1
            ),
            (PathBuf::from("crates/bullet-wire/src/catalog/schema.rs"), 1),
            (
                PathBuf::from("crates/bullet-wire/src/catalog/validation.rs"),
                1,
            ),
            (PathBuf::from("crates/bullet-wire/src/contract_bindings.rs"), 1),
            (PathBuf::from("crates/bullet-wire/src/dogfood.rs"), 1),
            (
                PathBuf::from("crates/bullet-wire/src/dogfood/enrollment_signing.rs"),
                1,
            ),
            (
                PathBuf::from("crates/bullet-wire/src/dogfood/grant_signing.rs"),
                1,
            ),
            (
                PathBuf::from("crates/bullet-wire/src/dogfood/run_signing.rs"),
                1,
            ),
            (
                PathBuf::from("crates/bullet-wire/src/runtime_passport.rs"),
                1,
            ),
            (PathBuf::from("src/check/model.rs"), 1),
            (PathBuf::from("src/check/release_evidence.rs"), 1),
            (PathBuf::from("src/checkout/git.rs"), 1),
            (PathBuf::from("src/coord/fresh_genesis.rs"), 1),
            (PathBuf::from("src/coord/generation/manifest.rs"), 1),
            (
                PathBuf::from("src/coord/generation/recovery/authority/metadata.rs"),
                1,
            ),
            (PathBuf::from("src/coord/generation/recovery.rs"), 1),
            (PathBuf::from("src/coord/generation/segment.rs"), 1),
            (PathBuf::from("src/coord/git/wave0.rs"), 1),
            (PathBuf::from("src/coord/model/fresh_genesis.rs"), 1),
            (
                PathBuf::from("src/coord/model/recovery_manifest/bootstrap_build.rs"),
                1,
            ),
            (
                PathBuf::from("src/coord/model/recovery_manifest/bootstrap_contract.rs"),
                1,
            ),
            (PathBuf::from("src/coord/recovered_wave0.rs"), 1),
            (PathBuf::from("src/coord/recovery.rs"), 1),
            (PathBuf::from("src/coord/recovery_manifest/authoring.rs"), 1),
            (
                PathBuf::from("src/coord/recovery_manifest/bootstrap_build.rs"),
                1,
            ),
            (PathBuf::from("src/coord/recovery_manifest/trust.rs"), 1),
            (PathBuf::from("src/coord/sealed.rs"), 1),
            (PathBuf::from("src/coord/model/recovery_adoption.rs"), 1),
            (PathBuf::from("src/coord/model/recovery_production.rs"), 1,),
            (
                PathBuf::from("src/coord/model/recovery_adoption/evidence.rs"),
                1,
            ),
            (
                PathBuf::from("src/coord/recovery_adoption_verify/forensic.rs"),
                1,
            ),
            (
                PathBuf::from("src/coord/recovery_adoption_verify/generation.rs"),
                1,
            ),
            (
                PathBuf::from("src/coord/recovery_adoption_verify/git.rs"),
                1,
            ),
            (PathBuf::from("src/coord/state/recovery_adoption.rs"), 1),
            (PathBuf::from("src/coord/state/recovery_evidence.rs"), 1),
            (PathBuf::from("src/coord/store/ledger.rs"), 1),
            (PathBuf::from("src/coord/store/ledger/adoption.rs"), 1),
            (
                PathBuf::from("src/coord/store/ledger/recovery_production.rs"),
                1,
            ),
            (PathBuf::from("src/family_lock.rs"), 1),
            (PathBuf::from("src/family_lock/git/command.rs"), 1),
            (PathBuf::from("src/family_lock/schema.rs"), 1),
            (PathBuf::from("src/fuse.rs"), 1),
            (PathBuf::from("src/process.rs"), 1),
            (PathBuf::from("src/release/archive.rs"), 1),
            (PathBuf::from("src/release/build/mod.rs"), 1),
            (PathBuf::from("src/release/receipt.rs"), 1),
            (PathBuf::from("src/release/verify.rs"), 1),
            (PathBuf::from("src/setup.rs"), 1),
            (PathBuf::from("src/setup/command.rs"), 1),
            (PathBuf::from("src/setup/transaction.rs"), 1),
        ])
    );
    include!("module_graph.rs");
}
