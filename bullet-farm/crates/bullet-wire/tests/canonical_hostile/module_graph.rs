{
    for (relative, attributes) in [
        ("src/check/model.rs", &["#[path = \"model_tests.rs\"]"][..]),
        (
            "src/check/profiles.rs",
            &["#[path = \"profiles/graph.rs\"]"][..],
        ),
        (
            "src/check/release_evidence.rs",
            &["#[path = \"release_evidence/tests.rs\"]"][..],
        ),
        (
            "src/check/semantic_registry.rs",
            &[
                "#[path = \"semantic_registry/admission.rs\"]",
                "#[path = \"semantic_registry/unsupported.rs\"]",
                "#[path = \"semantic_registry/validation.rs\"]",
            ][..],
        ),
        (
            "src/coord/generation/manifest.rs",
            &["#[path = \"manifest/tests.rs\"]"][..],
        ),
        (
            "src/coord/generation/recovery/authority/metadata.rs",
            &["#[path = \"metadata/tests.rs\"]"][..],
        ),
        (
            "src/coord/generation/recovery/authority.rs",
            &["#[path = \"authority/metadata.rs\"]"][..],
        ),
        (
            "src/coord/generation/recovery/exchange.rs",
            &["#[path = \"exchange/evidence.rs\"]"][..],
        ),
        (
            "src/coord/generation/recovery/tree.rs",
            &["#[path = \"tree/io.rs\"]"][..],
        ),
        (
            "src/coord/generation/recovery/verify.rs",
            &[
                "#[path = \"verify/process.rs\"]",
                "#[path = \"verify/lease.rs\"]",
            ][..],
        ),
        (
            "src/coord/generation/recovery.rs",
            &[
                "#[path = \"recovery/api.rs\"]",
                "#[path = \"recovery/fs.rs\"]",
                "#[path = \"recovery/authority.rs\"]",
                "#[path = \"recovery/verify.rs\"]",
                "#[path = \"recovery/projection.rs\"]",
                "#[path = \"recovery/exchange.rs\"]",
                "#[path = \"recovery/tree.rs\"]",
                "#[path = \"recovery/finalize.rs\"]",
                "#[path = \"recovery/transition.rs\"]",
                "#[path = \"recovery/published.rs\"]",
                "#[path = \"recovery/published_api.rs\"]",
                "#[path = \"recovery/support.rs\"]",
                "#[path = \"recovery/tests.rs\"]",
            ][..],
        ),
        (
            "src/coord/git.rs",
            &["#[path = \"recovery_adoption_verify/git.rs\"]"][..],
        ),
        (
            "src/coord/model/recovery_adoption/evidence.rs",
            &["#[path = \"evidence/tests.rs\"]"][..],
        ),
        (
            "src/coord/model/recovery_adoption.rs",
            &[
                "#[path = \"recovery_adoption/evidence.rs\"]",
                "#[path = \"recovery_adoption/validate.rs\"]",
                "#[path = \"recovery_adoption/tests.rs\"]",
            ][..],
        ),
        (
            "src/coord/model/recovery_manifest/bootstrap_build.rs",
            &[
                "#[path = \"bootstrap_contract.rs\"]",
                "#[path = \"build/tests.rs\"]",
            ][..],
        ),
        (
            "src/coord/model/recovery_manifest/bootstrap_contract.rs",
            &[
                "#[path = \"bootstrap_contract/toolchain.rs\"]",
                "#[path = \"bootstrap_contract/tests.rs\"]",
            ][..],
        ),
        (
            "src/coord/model/recovery_production.rs",
            &["#[path = \"recovery_production/tests.rs\"]"][..],
        ),
        (
            "src/coord/recovery_adoption_verify/forensic.rs",
            &[
                "#[path = \"forensic/derive.rs\"]",
                "#[path = \"forensic/tests.rs\"]",
            ][..],
        ),
        (
            "src/coord/recovery_adoption_verify/generation.rs",
            &["#[path = \"generation/tests.rs\"]"][..],
        ),
        (
            "src/coord/recovery_adoption_verify/git.rs",
            &[
                "#[path = \"git/derive.rs\"]",
                "#[path = \"git/manifest.rs\"]",
                "#[path = \"git/object_store.rs\"]",
                "#[path = \"git/tests.rs\"]",
            ][..],
        ),
        (
            "src/coord/recovery_manifest/authoring.rs",
            &["#[path = \"authoring/tests.rs\"]"][..],
        ),
        (
            "src/coord/recovery_manifest/bootstrap_build.rs",
            &["#[path = \"bootstrap_build/tests.rs\"]"][..],
        ),
        (
            "src/coord/recovery_manifest/linux.rs",
            &["#[path = \"linux/source.rs\"]"][..],
        ),
        (
            "src/coord/recovery_manifest/trust.rs",
            &[
                "#[path = \"trust/policy.rs\"]",
                "#[path = \"trust/window.rs\"]",
                "#[path = \"trust/tests.rs\"]",
            ][..],
        ),
        (
            "src/coord/sealed.rs",
            &[
                "#[path = \"sealed/raw.rs\"]",
                "#[path = \"sealed/runtime.rs\"]",
                "#[path = \"sealed/tests.rs\"]",
            ][..],
        ),
        (
            "src/coord/state/recovery_adoption.rs",
            &["#[path = \"recovery_adoption/tests.rs\"]"][..],
        ),
        (
            "src/coord/state/recovery_evidence.rs",
            &["#[path = \"recovery_evidence/tests.rs\"]"][..],
        ),
        (
            "src/coord/store/ledger/adoption.rs",
            &["#[path = \"adoption/tests.rs\"]"][..],
        ),
        (
            "src/coord/store/ledger/recovery_production.rs",
            &["#[path = \"recovery_production/tests.rs\"]"][..],
        ),
        (
            "src/coord/store/ledger.rs",
            &[
                "#[path = \"ledger/adoption/tests/git_fixture.rs\"]",
                "#[path = \"ledger/tests.rs\"]",
            ][..],
        ),
        ("src/fuse.rs", &["#[path = \"fuse/tests.rs\"]"][..]),
        (
            "src/process.rs",
            &["#[path = \"../tests/support/process_unit.rs\"]"][..],
        ),
        (
            "src/release/receipt.rs",
            &["#[path = \"receipt/tests.rs\"]"][..],
        ),
    ] {
        let text = fs::read_to_string(family.join(relative)).expect("UTF-8 Rust source");
        let actual = path_attribute_lines(&text).expect("exact literal path attributes");
        assert_eq!(
            actual, attributes,
            "{relative} changed its path reachability"
        );
        for attribute in attributes {
            let target = path_attribute_target(attribute).expect("literal path target");
            let source_dir = family
                .join(relative)
                .parent()
                .expect("production source parent")
                .to_path_buf();
            let resolved = fs::canonicalize(source_dir.join(target))
                .unwrap_or_else(|error| panic!("resolve {relative} {target}: {error}"));
            assert!(
                resolved.starts_with(fs::canonicalize(&family).expect("canonical family root")),
                "{relative} path target escapes the family: {target}"
            );
            assert!(
                resolved.is_file(),
                "{relative} path target is not a file: {target}"
            );
        }
    }
    assert!(production.contains(&PathBuf::from("contracts/generated/rust/schema_bundle.rs")));
    let include_shapes = production
        .iter()
        .filter_map(|relative| {
            let source = fs::read_to_string(family.join(relative)).expect("UTF-8 Rust source");
            let ranges =
                include_macro_ranges(&source).expect("well-formed include macro inventory");
            (!ranges.is_empty()).then(|| {
                let shapes = ranges
                    .into_iter()
                    .map(|range| {
                        source[range]
                            .chars()
                            .filter(|character| !character.is_whitespace())
                            .collect::<String>()
                    })
                    .collect::<Vec<_>>();
                (relative.clone(), shapes)
            })
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        include_shapes,
        BTreeMap::from([
            (
                PathBuf::from("crates/bullet-wire/src/lib.rs"),
                vec![CANONICAL_INCLUDE.to_owned()],
            ),
            (
                PathBuf::from("src/check/truth/render.rs"),
                vec![r#"include!("render/closing_sections.rs")"#.to_owned()],
            ),
            (
                PathBuf::from("src/release/archive.rs"),
                vec![r#"include!("archive/snapshot.rs")"#.to_owned()],
            ),
        ])
    );
    let include_source = fs::read_to_string(family.join("crates/bullet-wire/src/lib.rs")).unwrap();
    assert!(has_only_canonical_include(&include_source));

    let overrides = sources
        .iter()
        .filter(|path| lint_policy_override(&fs::read_to_string(path).expect("UTF-8 Rust source")))
        .map(|path| {
            path.strip_prefix(&family)
                .expect("family-relative source")
                .to_path_buf()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        overrides,
        [PathBuf::from("crates/bullet-wire/src/canonical.rs")]
    );
    let metadata = cargo_metadata(&family.join("Cargo.toml"), true);
    let expected_targets = vec![
        (
            "bullet-family".to_owned(),
            "bullet-family".to_owned(),
            vec!["bin".to_owned()],
            "src/main.rs".to_owned(),
        ),
        (
            "bullet-family".to_owned(),
            "bullet_family".to_owned(),
            vec!["lib".to_owned()],
            "src/lib.rs".to_owned(),
        ),
        (
            "bullet-linux-lease".to_owned(),
            "bullet_linux_lease".to_owned(),
            vec!["lib".to_owned()],
            "crates/bullet-linux-lease/src/lib.rs".to_owned(),
        ),
        (
            "bullet-wire".to_owned(),
            "bullet-contract".to_owned(),
            vec!["bin".to_owned()],
            "crates/bullet-wire/src/bin/bullet-contract.rs".to_owned(),
        ),
        (
            "bullet-wire".to_owned(),
            "bullet_wire".to_owned(),
            vec!["lib".to_owned()],
            "crates/bullet-wire/src/lib.rs".to_owned(),
        ),
    ];
    assert_eq!(
        non_test_target_inventory(&metadata, &family),
        Ok(expected_targets.clone())
    );
    let mut hostile_targets = metadata.clone();
    hostile_targets["packages"]
        .as_array_mut()
        .expect("cargo metadata packages array")
        .iter_mut()
        .find(|package| package["name"] == "bullet-wire")
        .expect("bullet-wire package")["targets"]
        .as_array_mut()
        .expect("package targets array")
        .push(serde_json::json!({
            "kind": ["bin"],
            "name": "hidden-decoder",
            "src_path": family.join("tools/hidden-decoder.rs"),
        }));
    assert_ne!(
        non_test_target_inventory(&hostile_targets, &family),
        Ok(expected_targets)
    );
    assert_eq!(
        serde_json_dependency_inventory(&metadata),
        [
            ("bullet-family".to_owned(), None),
            ("bullet-wire".to_owned(), None),
        ]
    );
    assert_eq!(
        serde_json_dependency_inventory(&hostile_renamed_dependency_metadata()),
        [("hostile-metadata".to_owned(), Some("json".to_owned()))]
    );
    let registry = Some("registry+https://github.com/rust-lang/crates.io-index".to_owned());
    let full_metadata = cargo_metadata_with_dependencies(&family.join("Cargo.toml"));
    assert_eq!(
        proc_macro_inventory(&full_metadata),
        [
            (
                "derive_arbitrary".to_owned(),
                "1.4.2".to_owned(),
                registry.clone()
            ),
            (
                "displaydoc".to_owned(),
                "0.2.7".to_owned(),
                registry.clone()
            ),
            (
                "rustversion".to_owned(),
                "1.0.23".to_owned(),
                registry.clone()
            ),
            (
                "serde_derive".to_owned(),
                "1.0.229".to_owned(),
                registry.clone()
            ),
            (
                "thiserror-impl".to_owned(),
                "2.0.20".to_owned(),
                registry.clone()
            ),
            (
                "wasm-bindgen-macro".to_owned(),
                "0.2.127".to_owned(),
                registry
            ),
        ]
    );
    let dependency_inventory = workspace_direct_dependency_inventory(&full_metadata, &family)
        .expect("workspace dependency inventory");
    assert_eq!(
        dependency_inventory,
        (
            25,
            "a179f9a0e543229f7461caa42fb34f9380858806313dfd39a7b99c01ce162838".to_owned()
        )
    );
    let mut hostile_direct_edge = full_metadata.clone();
    hostile_direct_edge["packages"]
        .as_array_mut()
        .expect("cargo metadata packages array")
        .iter_mut()
        .find(|package| package["name"] == "bullet-wire")
        .expect("bullet-wire package")["dependencies"]
        .as_array_mut()
        .expect("package dependency array")
        .push(serde_json::json!({
            "name": "rustversion",
            "source": "registry+https://github.com/rust-lang/crates.io-index",
            "req": "^1.0",
            "kind": null,
            "rename": null,
            "optional": false,
            "uses_default_features": true,
            "features": [],
            "target": null,
            "registry": null,
            "path": null
        }));
    assert_eq!(
        proc_macro_inventory(&hostile_direct_edge),
        proc_macro_inventory(&full_metadata),
        "the package-only inventory must demonstrate the same-version edge bypass"
    );
    assert_ne!(
        workspace_direct_dependency_inventory(&hostile_direct_edge, &family)
            .expect("hostile workspace dependency inventory"),
        dependency_inventory,
        "the direct-dependency inventory must bind an edge to an already-locked proc macro"
    );
}
