#[path = "semantic_registry/fixture.rs"]
mod fixture;
#[cfg(target_os = "linux")]
#[path = "semantic_registry/hostile.rs"]
mod hostile;

use std::fs;

use super::command;
use fixture::{
    assert_registry_rejected, cleanup_registry_fixture, mutate_registry_manifest,
    profiled_registry_output, release_id, write_canonical, write_structural_registry,
};

#[test]
fn release_profiles_are_named_independent_and_fail_closed() {
    let registry_path =
        std::env::temp_dir().join(format!("bullet-profile-registry-{}", std::process::id()));
    if registry_path.exists() {
        fs::remove_dir_all(&registry_path).unwrap();
    }
    fs::create_dir(&registry_path).unwrap();
    let registry = registry_path.to_str().unwrap();

    let linux_preview = command(&[
        "check",
        "release",
        "--profile",
        "linux-preview",
        "--receipts",
        registry,
        "--json",
    ]);
    assert_eq!(linux_preview.status.code(), Some(3));
    assert!(linux_preview.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&linux_preview.stdout).unwrap();
    assert_eq!(report["schema_version"], 3);
    assert_eq!(report["profile"], "linux-preview");
    assert_eq!(report["status"], "BLOCKED");
    let ids = report["gates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|gate| gate["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 23);
    for excluded in [
        "release.forge.github-app",
        "release.provider.codex",
        "release.provider.cursor",
        "release.provider.antigravity",
        "release.package-matrix",
        "release.operations-v1",
        "release.evolution-v1",
    ] {
        assert!(!ids.contains(&excluded));
    }
    for required in [
        "release.forge.jeryu",
        "release.provider.claude",
        "release.package-linux-x86_64",
        "release.systemd-v1",
    ] {
        assert!(ids.contains(&required));
    }

    for (profile, gate) in [
        ("provider-codex", "release.provider.codex"),
        ("provider-cursor", "release.provider.cursor"),
        ("provider-antigravity", "release.provider.antigravity"),
        ("github-adapter-v1", "release.forge.github-app"),
    ] {
        let output = command(&[
            "check",
            "release",
            "--profile",
            profile,
            "--receipts",
            registry,
            "--json",
        ]);
        assert_eq!(output.status.code(), Some(3), "profile={profile}");
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["profile"], profile);
        let gates = report["gates"].as_array().unwrap();
        assert_eq!(gates.len(), 3);
        assert!(gates.iter().any(|item| item["id"] == gate));
        assert!(
            gates
                .iter()
                .any(|item| item["id"] == format!("release.profile.{profile}"))
        );
        assert!(
            gates
                .iter()
                .any(|item| item["id"] == "release.receipt-contracts")
        );
    }

    for profile in [
        "self-hosted-v1",
        "evolution-v1",
        "provider-claude",
        "jeryu-forge-v1",
        "gitlab-adapter-v1",
        "gitlab-self-managed-v1",
        "platform-linux-x86_64",
        "platform-linux-aarch64",
        "platform-macos-x86_64",
        "platform-macos-aarch64",
        "platform-windows-x86_64",
        "universal-v1",
        "team-v1",
        "saga-v1",
    ] {
        let output = command(&[
            "check",
            "release",
            "--profile",
            profile,
            "--receipts",
            registry,
            "--json",
        ]);
        assert_eq!(output.status.code(), Some(3), "profile={profile}");
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["profile"], profile);
        assert_eq!(report["status"], "BLOCKED");
    }

    write_structural_registry(&registry_path);
    let structurally_valid = profiled_registry_output(&registry_path, "provider-codex");
    #[cfg(target_os = "linux")]
    {
        assert_eq!(structurally_valid.status.code(), Some(3));
        let report: serde_json::Value = serde_json::from_slice(&structurally_valid.stdout).unwrap();
        let receipt_gate = report["gates"]
            .as_array()
            .unwrap()
            .iter()
            .find(|gate| gate["id"] == "release.receipt-contracts")
            .unwrap();
        assert_eq!(receipt_gate["status"], "BLOCKED");
        assert!(
            receipt_gate["detail"]
                .as_str()
                .unwrap()
                .contains("external trust-root")
        );
        assert!(
            report["gates"]
                .as_array()
                .unwrap()
                .iter()
                .all(|gate| gate["status"] != "PASS")
        );
    }
    #[cfg(not(target_os = "linux"))]
    assert_eq!(structurally_valid.status.code(), Some(1));

    #[cfg(target_os = "linux")]
    hostile::assert_hostile_registries(&registry_path);

    assert_registry_rejected(&registry_path, "provider-cursor");

    write_structural_registry(&registry_path);
    write_canonical(
        &registry_path.join("registry-manifest.json"),
        &serde_json::json!({"payload":"generic","signature":"self-selected"}),
    );
    assert_registry_rejected(&registry_path, "provider-codex");

    write_structural_registry(&registry_path);
    fs::write(registry_path.join("registry-manifest.json"), b"{malformed").unwrap();
    assert_registry_rejected(&registry_path, "provider-codex");

    write_structural_registry(&registry_path);
    mutate_registry_manifest(&registry_path, |manifest| {
        manifest["objects"][0]["object_path"] = "../receipt.json".into();
    });
    assert_registry_rejected(&registry_path, "provider-codex");

    write_structural_registry(&registry_path);
    mutate_registry_manifest(&registry_path, |manifest| {
        manifest["objects"][1]["object_path"] = manifest["objects"][0]["object_path"].clone();
    });
    assert_registry_rejected(&registry_path, "provider-codex");

    write_structural_registry(&registry_path);
    mutate_registry_manifest(&registry_path, |manifest| {
        manifest["entries"][1]["gate_receipt_id"] = release_id("grc", '3').into();
    });
    assert_registry_rejected(&registry_path, "provider-codex");

    write_structural_registry(&registry_path);
    let receipt_path = registry_path.join("receipts/provider-codex.json");
    let mut receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
    receipt["gate_receipt_id"] = release_id("grc", '2').into();
    write_canonical(&receipt_path, &receipt);
    assert_registry_rejected(&registry_path, "provider-codex");

    write_structural_registry(&registry_path);
    fs::remove_file(registry_path.join("requests/provider-codex.json")).unwrap();
    assert_registry_rejected(&registry_path, "provider-codex");

    write_structural_registry(&registry_path);
    fs::remove_file(&receipt_path).unwrap();
    fs::create_dir(&receipt_path).unwrap();
    assert_registry_rejected(&registry_path, "provider-codex");

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        write_structural_registry(&registry_path);
        fs::rename(
            &receipt_path,
            registry_path.join("receipts/provider-codex.real"),
        )
        .unwrap();
        symlink("provider-codex.real", &receipt_path).unwrap();
        assert_registry_rejected(&registry_path, "provider-codex");

        write_structural_registry(&registry_path);
        fs::rename(
            registry_path.join("receipts"),
            registry_path.join("receipts-real"),
        )
        .unwrap();
        symlink("receipts-real", registry_path.join("receipts")).unwrap();
        assert_registry_rejected(&registry_path, "provider-codex");

        write_structural_registry(&registry_path);
        let manifest = registry_path.join("registry-manifest.json");
        fs::rename(&manifest, registry_path.join("registry-manifest.real")).unwrap();
        symlink("registry-manifest.real", &manifest).unwrap();
        assert_registry_rejected(&registry_path, "provider-codex");

        write_structural_registry(&registry_path);
        let receipt_signature = registry_path.join("signatures/provider-codex.sig");
        let time_signature = registry_path.join("time/provider-codex.sig");
        fs::remove_file(&time_signature).unwrap();
        fs::hard_link(&receipt_signature, &time_signature).unwrap();
        assert_registry_rejected(&registry_path, "provider-codex");

        write_structural_registry(&registry_path);
        let admitted = registry_path.with_extension("admitted");
        if admitted.exists() {
            fs::remove_dir_all(&admitted).unwrap();
        }
        fs::rename(&registry_path, &admitted).unwrap();
        symlink(&admitted, &registry_path).unwrap();
        assert_registry_rejected(&registry_path, "provider-codex");
        fs::remove_file(&registry_path).unwrap();
        fs::rename(&admitted, &registry_path).unwrap();
    }
    cleanup_registry_fixture(&registry_path);
}
