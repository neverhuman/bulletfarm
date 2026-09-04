{
    let family = root();
    let mut sources = Vec::new();
    assert!(excluded_root_tree(&family, &family.join("target")));
    assert!(!excluded_root_tree(&family, &family.join("src/target")));
    collect_rust_sources(&family, &family, &mut sources);
    let mut test_only_sources = BTreeSet::new();
    for path in &sources {
        let relative = path.strip_prefix(&family).expect("family-relative source");
        if !is_production_rust(relative) {
            continue;
        }
        let text = fs::read_to_string(path).expect("UTF-8 Rust source");
        test_only_sources.extend(
            external_test_module_targets(path, &text)
                .unwrap_or_else(|error| panic!("{}: {error}", relative.display())),
        );
    }
    let mut production = Vec::new();
    let mut include_identifier_sites = BTreeMap::new();
    let mut parse_sites = BTreeMap::new();
    let mut parse_id_sites = BTreeMap::new();
    let mut path_attribute_sites = BTreeMap::new();
    let mut qualified_attribute_sites = BTreeMap::new();
    let mut test_module_sites = BTreeMap::new();
    for path in &sources {
        let relative = path.strip_prefix(&family).expect("family-relative source");
        if !is_production_rust(relative)
            || test_only_sources
                .contains(&fs::canonicalize(path).expect("canonical inventoried Rust source"))
        {
            continue;
        }
        production.push(relative.to_path_buf());
        let text = fs::read_to_string(path).expect("UTF-8 Rust source");
        assert!(
            !indirect_attribute_redirects_path(&text).unwrap_or_else(|error| {
                panic!("{} has an invalid attribute: {error}", relative.display())
            }),
            "{} indirectly redirects a module path",
            relative.display()
        );
        assert!(
            !macro_tt_fragment(&text),
            "{} defines a `tt` macro fragment that can synthesize unreviewed source",
            relative.display()
        );
        assert!(
            !macro_arguments_assign_path(&text).unwrap_or_else(|error| {
                panic!(
                    "{} has an invalid macro invocation: {error}",
                    relative.display()
                )
            }),
            "{} supplies a module path through a macro argument",
            relative.display()
        );
        let code = rust_code_skeleton(&text);
        let qualified_attributes = qualified_attribute_paths(&text).unwrap_or_else(|error| {
            panic!("{} has an invalid attribute: {error}", relative.display())
        });
        if !qualified_attributes.is_empty() {
            let mut counts = BTreeMap::new();
            for attribute in qualified_attributes {
                *counts.entry(attribute).or_insert(0_usize) += 1;
            }
            qualified_attribute_sites.insert(relative.to_path_buf(), counts);
        }
        let include_identifier_count = rust_identifier_count(&code, "include");
        if include_identifier_count > 0 {
            include_identifier_sites.insert(relative.to_path_buf(), include_identifier_count);
        }
        let parse_count = method_call_count(&code, "parse");
        if parse_count > 0 {
            parse_sites.insert(relative.to_path_buf(), parse_count);
        }
        let parse_id_count = code.matches("parse_id(").count();
        if parse_id_count > 0 {
            parse_id_sites.insert(relative.to_path_buf(), parse_id_count);
        }
        let associated_from_str = associated_call_count(&code, "from_str");
        let compact_code = code
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        let path_attribute_count = compact_code.matches("#[path=").count();
        if path_attribute_count > 0 {
            let attributes = path_attribute_lines(&text).unwrap_or_else(|error| {
                panic!(
                    "{} has an invalid path attribute: {error}",
                    relative.display()
                )
            });
            assert_eq!(attributes.len(), path_attribute_count);
            path_attribute_sites.insert(relative.to_path_buf(), path_attribute_count);
        }
        let test_module_count = compact_code.matches("modtests;").count();
        if test_module_count > 0 {
            assert!(
                test_modules_are_cfg_gated(&text),
                "{} exposes an external tests module outside cfg(test)",
                relative.display()
            );
            test_module_sites.insert(relative.to_path_buf(), test_module_count);
        }
        let toml_from_str = compact_code.matches("toml::from_str(").count();
        let reviewed_raw_from_str =
            usize::from(relative == std::path::Path::new("crates/bullet-wire/src/canonical.rs"));
        assert_eq!(
            associated_from_str,
            toml_from_str + reviewed_raw_from_str,
            "{} adds an unreviewed associated FromStr call",
            relative.display()
        );
        if relative == std::path::Path::new("crates/bullet-wire/src/canonical.rs") {
            assert_eq!(canonical_entrypoint_shape(&text), Ok(()));
            continue;
        }
        assert!(
            raw_serde_json_decoder(&text).is_none(),
            "{} contains a direct raw serde_json decoder or document-type hiding surface {:?}; production JSON input must route through bullet-wire's bounded unique/canonical decoder",
            relative.display(),
            raw_serde_json_decoder(&text)
        );
    }
    assert_eq!(
        parse_sites,
        BTreeMap::from([(PathBuf::from("crates/bullet-wire/src/canonical.rs"), 1)])
    );
    assert_eq!(
        include_identifier_sites,
        BTreeMap::from([
            (PathBuf::from("crates/bullet-wire/src/lib.rs"), 1),
            (PathBuf::from("src/check/truth/render.rs"), 1),
            (PathBuf::from("src/release/archive.rs"), 1),
        ])
    );
    assert_eq!(
        parse_id_sites,
        BTreeMap::from([
            (
                PathBuf::from("crates/bullet-wire/src/contract_tool/authority.rs"),
                19,
            ),
            (
                PathBuf::from("crates/bullet-wire/src/contract_tool/launch.rs"),
                11,
            ),
        ])
    );
    assert_eq!(
        path_attribute_sites,
        BTreeMap::from([
            (PathBuf::from("src/check/model.rs"), 1),
            (PathBuf::from("src/check/profiles.rs"), 1),
            (PathBuf::from("src/check/release_evidence.rs"), 1),
            (PathBuf::from("src/check/semantic_registry.rs"), 3),
            (PathBuf::from("src/coord/generation/manifest.rs"), 1),
            (
                PathBuf::from("src/coord/generation/recovery/authority/metadata.rs"),
                1,
            ),
            (
                PathBuf::from("src/coord/generation/recovery/authority.rs"),
                1
            ),
            (
                PathBuf::from("src/coord/generation/recovery/exchange.rs"),
                1
            ),
            (PathBuf::from("src/coord/generation/recovery/tree.rs"), 1),
            (PathBuf::from("src/coord/generation/recovery/verify.rs"), 2),
            (PathBuf::from("src/coord/generation/recovery.rs"), 13),
            (PathBuf::from("src/coord/git.rs"), 1),
            (
                PathBuf::from("src/coord/model/recovery_adoption/evidence.rs"),
                1,
            ),
            (PathBuf::from("src/coord/model/recovery_adoption.rs"), 3),
            (
                PathBuf::from("src/coord/model/recovery_manifest/bootstrap_build.rs"),
                2,
            ),
            (
                PathBuf::from("src/coord/model/recovery_manifest/bootstrap_contract.rs"),
                2,
            ),
            (PathBuf::from("src/coord/model/recovery_production.rs"), 1,),
            (PathBuf::from("src/coord/recovered_wave0.rs"), 1),
            (
                PathBuf::from("src/coord/recovery_adoption_verify/forensic.rs"),
                2,
            ),
            (
                PathBuf::from("src/coord/recovery_adoption_verify/generation.rs"),
                1,
            ),
            (
                PathBuf::from("src/coord/recovery_adoption_verify/git.rs"),
                4,
            ),
            (PathBuf::from("src/coord/recovery_manifest/authoring.rs"), 1),
            (
                PathBuf::from("src/coord/recovery_manifest/bootstrap_build.rs"),
                1,
            ),
            (PathBuf::from("src/coord/recovery_manifest/linux.rs"), 1),
            (PathBuf::from("src/coord/recovery_manifest/trust.rs"), 3),
            (PathBuf::from("src/coord/sealed.rs"), 3),
            (PathBuf::from("src/coord/state/recovery_adoption.rs"), 1),
            (PathBuf::from("src/coord/state/recovery_evidence.rs"), 1),
            (PathBuf::from("src/coord/store/ledger/adoption.rs"), 1),
            (
                PathBuf::from("src/coord/store/ledger/recovery_production.rs"),
                1,
            ),
            (PathBuf::from("src/coord/store/ledger.rs"), 2),
            (PathBuf::from("src/fuse.rs"), 1),
            (PathBuf::from("src/process.rs"), 1),
            (PathBuf::from("src/release/receipt.rs"), 1),
        ])
    );
    assert_eq!(
        qualified_attribute_sites,
        BTreeMap::from([
            (
                PathBuf::from("contracts/generated/rust/schema_bundle.rs"),
                BTreeMap::from([
                    ("serde::Deserialize".to_owned(), 118),
                    ("serde::Serialize".to_owned(), 118),
                ]),
            ),
            (
                PathBuf::from("crates/bullet-wire/src/canonical.rs"),
                BTreeMap::from([("clippy::disallowed_methods".to_owned(), 2)]),
            ),
            (
                PathBuf::from("crates/bullet-wire/src/catalog.rs"),
                BTreeMap::from([("rustfmt::skip".to_owned(), 5)]),
            ),
            (
                PathBuf::from("crates/bullet-wire/src/catalog/validation.rs"),
                BTreeMap::from([("rustfmt::skip".to_owned(), 22)]),
            ),
            (
                PathBuf::from("crates/bullet-wire/src/contract_bindings.rs"),
                BTreeMap::from([("rustfmt::skip".to_owned(), 17)]),
            ),
            (
                PathBuf::from("crates/bullet-wire/src/contract_bindings/strict.rs"),
                BTreeMap::from([("rustfmt::skip".to_owned(), 22)]),
            ),
            (
                PathBuf::from("src/coord/generation/manifest/types.rs"),
                BTreeMap::from([
                    ("clippy::large_enum_variant".to_owned(), 1),
                    ("clippy::too_many_arguments".to_owned(), 1),
                ]),
            ),
            (
                PathBuf::from("src/coord/generation/recovery/authority/metadata.rs"),
                BTreeMap::from([("clippy::too_many_arguments".to_owned(), 1)]),
            ),
            (
                PathBuf::from("src/coord/generation/recovery/authority.rs"),
                BTreeMap::from([("clippy::too_many_arguments".to_owned(), 1)]),
            ),
            (
                PathBuf::from("src/coord/generation/recovery/exchange/evidence.rs"),
                BTreeMap::from([("clippy::too_many_arguments".to_owned(), 2)]),
            ),
            (
                PathBuf::from("src/coord/generation/recovery/exchange.rs"),
                BTreeMap::from([("clippy::too_many_arguments".to_owned(), 2)]),
            ),
            (
                PathBuf::from("src/coord/generation/recovery/finalize.rs"),
                BTreeMap::from([("clippy::too_many_arguments".to_owned(), 1)]),
            ),
            (
                PathBuf::from("src/coord/generation/recovery/tests/adoption_fixture.rs"),
                BTreeMap::from([("clippy::too_many_arguments".to_owned(), 1)]),
            ),
            (
                PathBuf::from("src/coord/mod.rs"),
                BTreeMap::from([
                    ("serde::Deserialize".to_owned(), 7),
                    ("serde::Serialize".to_owned(), 7),
                ]),
            ),
            (
                PathBuf::from("src/coord/model/recovery_adoption/validate.rs"),
                BTreeMap::from([("clippy::too_many_arguments".to_owned(), 1)]),
            ),
            (
                PathBuf::from("src/coord/model.rs"),
                BTreeMap::from([("clippy::large_enum_variant".to_owned(), 1)]),
            ),
            (
                PathBuf::from("src/coord/recovery.rs"),
                BTreeMap::from([("clippy::too_many_arguments".to_owned(), 1)]),
            ),
            (
                PathBuf::from("src/release/receipt/verify.rs"),
                BTreeMap::from([("clippy::too_many_arguments".to_owned(), 1)]),
            ),
            (
                PathBuf::from("src/release/signature.rs"),
                BTreeMap::from([("clippy::too_many_arguments".to_owned(), 1)]),
            ),
        ])
    );
    (family, sources, production, test_module_sites)
}
