use super::jobs::{final_step, job_id};
use super::*;
use crate::publication::{
    read_manifest_at,
    tests::{Fixture, commit},
};
use std::fs;

fn fixture() -> (Fixture, Vec<Workflow>) {
    let fixture = Fixture::new();
    let mut catalog = ci_inventory::reviewed();
    for workflow in &mut catalog {
        let bytes = format!(
            "# trusted synthetic workflow {} {}\n",
            workflow.member, workflow.path
        );
        workflow.sha256 = store::digest(bytes.as_bytes());
        let repo = fixture.root.join(workflow.member);
        fs::create_dir_all(repo.join(".github/workflows")).unwrap();
        fs::write(repo.join(workflow.path), bytes).unwrap();
    }
    fs::write(
        fixture.root.join("bullet-farm").join(TEMPLATE_PATH),
        TEMPLATE,
    )
    .unwrap();
    for member in MEMBERS {
        commit(&fixture.root.join(member), "synthetic pinned workflows");
    }
    (fixture, catalog)
}

fn assert_reviewed_timeouts() {
    let catalog = ci_inventory::reviewed();
    assert!(ci_inventory::canonical_catalog(&catalog).is_ok());
    for (member, path, id, minutes) in [
        ("bullet-farm", ci_inventory::CI, "source_scan", 10),
        ("bullet-farm", ci_inventory::CI, "docs", 35),
        ("bullet-git", ci_inventory::CI, "required", 5),
        (
            "bullet-kernel",
            ci_inventory::SCHEDULED,
            "portable-refusal",
            35,
        ),
        ("bullet-portal", ci_inventory::CI, "lint", 10),
    ] {
        let workflow = catalog
            .iter()
            .find(|w| w.member == member && w.path == path)
            .unwrap();
        let job = workflow.jobs.iter().find(|j| j.id == id).unwrap();
        assert_eq!(job.timeout_minutes, minutes, "{member}:{path}:{id}");
    }
    for minutes in [0, 1, 5, u16::MAX] {
        let mut invalid = catalog.clone();
        invalid[0].jobs[0].timeout_minutes = minutes;
        assert_eq!(
            ci_inventory::canonical_catalog(&invalid)
                .err()
                .unwrap()
                .code(),
            "PUBLICATION_CI_TIMEOUT_INVALID"
        );
    }
    let mut unknown = catalog;
    unknown[0].jobs.last_mut().unwrap().id = "unknown_job";
    assert_eq!(
        ci_inventory::canonical_catalog(&unknown)
            .err()
            .unwrap()
            .code(),
        "PUBLICATION_CI_TIMEOUT_INVALID"
    );
}

#[test]
fn generated_topology_preserves_dependencies_matrices_events_and_real_producer() {
    assert_reviewed_timeouts();
    let (fixture, mut catalog) = fixture();
    let (store, request, prepared) = fixture.prepared();
    let request_bytes = fs::read(store.path("fixture-1", "request")).unwrap();
    let mut manifest = request.manifest.clone();
    manifest.tool_config.schema_version = V2.into();
    let files = render(&store.objects, &manifest, &catalog).unwrap();
    assert_eq!(files.len(), manifest.tool_config.root_files.len() + 4);
    let primary = String::from_utf8(files[PRIMARY].clone()).unwrap();
    assert!(primary.contains("  push:\n  merge_group:\n    types: [checks_requested]"));
    assert!(primary.contains("PUBLICATION_CI_BOOTSTRAP_EVENT_UNSUPPORTED"));
    assert_eq!(
        primary
            .matches("run: bash bullet-farm/publication/ci.sh member-source-scan")
            .count(),
        1
    );
    assert!(primary.contains("run: bash bullet-farm/publication/ci-required.sh required"));
    assert!(!primary.contains("branches: [main]"));
    let bootstrap = primary
        .split("\n  publication_integrity:\n")
        .nth(1)
        .unwrap()
        .split("\n  bullet_farm_")
        .next()
        .unwrap();
    assert!(bootstrap.contains("    timeout-minutes: 30\n"));
    let mut counts = (1, 1);
    for workflow in &catalog {
        let path = if workflow.scope == "REQUIRED" {
            PRIMARY.into()
        } else {
            format!(".github/workflows/{}-scheduled.yml", workflow.member)
        };
        let text = String::from_utf8(files[&path].clone()).unwrap();
        for job in &workflow.jobs {
            let id = job_id(workflow, job.id);
            if id == "publication_integrity" {
                continue;
            }
            let block = text
                .split(&format!("\n  {id}:\n"))
                .nth(1)
                .unwrap()
                .lines()
                .take_while(|line| !line.starts_with("  ") || line.starts_with("    "))
                .collect::<Vec<_>>()
                .join("\n");
            assert!(block.contains(&format!(
                "name: '{}:{}:{}'",
                workflow.member, workflow.scope, job.id
            )));
            assert_eq!(block.contains("    if: ${{ always() }}"), job.always);
            assert_eq!(
                block
                    .lines()
                    .find(|line| line.starts_with("    timeout-minutes:")),
                Some(format!("    timeout-minutes: {}", job.timeout_minutes).as_str())
            );
            let mut needs = job.needs.clone();
            needs.sort();
            let mut needs = needs
                .iter()
                .map(|id| job_id(workflow, id))
                .collect::<Vec<_>>();
            let active = workflow.scope == "REQUIRED" && id == "bullet_git_source_scan";
            if active {
                assert!(needs.is_empty(), "source dependencies are unchanged");
                needs.push("publication_integrity".into()); // Infrastructure-only tool transfer.
            }
            assert_eq!(
                block
                    .lines()
                    .find(|l| l.starts_with("    needs:"))
                    .map(str::to_owned),
                if needs.is_empty() {
                    None
                } else {
                    Some(format!("    needs: [{}]", needs.join(", ")))
                }
            );
            assert!(block.contains(&format!(
                "CI_MEMBER_COMMIT: '{}'",
                manifest.members[workflow.member].commit
            )));
            assert!(block.contains("CI_AGGREGATE_EVENT_SHA: ${{ github.sha }}"));
            assert!(!block.contains("      GITHUB_SHA:"));
            assert_eq!(block.contains("fail-fast: false"), !job.os.is_empty());
            if !job.os.is_empty() {
                assert!(block.contains("os: [macos-15, windows-2025]"));
            }
            if active {
                assert!(!block.contains("PUBLICATION_CI_RUNTIME_ADMISSION_UNAVAILABLE"));
                for required in [
                    "ci-transfer.sh receive",
                    "ci-required.sh git-member-run",
                    "needs.publication_integrity.outputs.transfer_sha256",
                    "needs.publication_integrity.outputs.bootstrap_completion_sha256",
                    "needs.publication_integrity.outputs.member_completion_sha256",
                    "needs.publication_integrity.result",
                    "steps.member.outputs.git_member_completion_sha256",
                    "actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683",
                    "actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02",
                    "actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093",
                    "name: publication-tools-${{ github.run_id }}-${{ github.run_attempt }}",
                    "name: publication-git-source-scan-${{ github.run_id }}-${{ github.run_attempt }}",
                    "if-no-files-found: error",
                    "include-hidden-files: true",
                    "bullet-publication-transfer-download/",
                    "retention-days: 14",
                    "reconstruct \"$GITHUB_WORKSPACE\"",
                    "persist-credentials: false",
                ] {
                    assert!(block.contains(required), "{required}");
                }
                assert!(!block.contains("cargo build"));
            } else {
                assert!(block.trim_end().ends_with("          exit 1"));
            }
            counts.0 += 1;
            counts.1 += job.os.len().max(1);
        }
        assert!(text.contains("PUBLICATION_CI_FULL_EXECUTION_ARTIFACT_ADMISSION_UNAVAILABLE"));
        assert!(text.contains("if: ${{ always() }}"));
        if workflow.scope == "SCHEDULED" {
            let cron = if matches!(workflow.member, "bullet-farm" | "bullet-portal") {
                "17 4 * * 1"
            } else {
                "23 4 * * 1"
            };
            assert!(text.contains(&format!("cron: '{cron}'")));
        }
    }
    assert_eq!(counts, (53, 55));
    assert_eq!(
        files
            .values()
            .map(|bytes| String::from_utf8_lossy(bytes)
                .matches("PUBLICATION_CI_RUNTIME_ADMISSION_UNAVAILABLE")
                .count())
            .sum::<usize>(),
        51
    );
    assert!(bootstrap.contains("ci-transfer.sh prepare"));
    assert!(bootstrap.contains("steps.transfer.outputs.transfer_sha256"));
    assert!(bootstrap.contains("steps.transfer.outputs.verifier_sha256"));
    assert!(bootstrap.contains("actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02"));
    assert!(primary.contains("ci-transfer.sh git-required"));

    let rendered_jobs = files
        .values()
        .flat_map(|bytes| std::str::from_utf8(bytes).unwrap().lines())
        .filter(|line| {
            line.starts_with("    name: '")
                && (line.contains(":REQUIRED:") || line.contains(":SCHEDULED:"))
        })
        .count();
    assert_eq!(rendered_jobs + 1, 53);
    let expected_ids = catalog
        .iter()
        .filter(|w| w.scope == "REQUIRED")
        .flat_map(|w| w.jobs.iter().map(move |j| job_id(w, j.id)))
        .collect::<std::collections::BTreeSet<_>>();
    let final_job = primary.split("\n  publication_required:\n").nth(1).unwrap();
    assert!(final_job.contains("    timeout-minutes: 5\n"));
    for binding in [
        "needs.bullet_git_source_scan.result",
        "needs.bullet_git_source_scan.outputs.git_member_completion_sha256",
        "needs.publication_integrity.outputs.verifier_sha256",
        "bullet-publication-git-downloaded/publication-git-source-scan-",
    ] {
        assert!(final_job.contains(binding), "{binding}");
    }
    let dependencies = final_job
        .lines()
        .find_map(|line| line.strip_prefix("    needs: ["))
        .unwrap();
    assert_eq!(
        dependencies
            .strip_suffix(']')
            .unwrap()
            .split(", ")
            .map(str::to_owned)
            .collect::<std::collections::BTreeSet<_>>(),
        expected_ids
    );
    let gate = final_step(&["one".into(), "two".into()]);
    let script = gate
        .split("        run: |\n")
        .nth(1)
        .unwrap()
        .lines()
        .map(|line| line.strip_prefix("          ").unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    for result in [
        "success",
        "failure",
        "skipped",
        "cancelled",
        "neutral",
        "",
        "malformed",
    ] {
        let output = std::process::Command::new("/bin/bash")
            .args(["-c", &script])
            .env_clear()
            .env("RESULT_0", "success")
            .env("RESULT_1", result)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{result}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains(if result == "success" {
            "ARTIFACT_ADMISSION_UNAVAILABLE"
        } else {
            "RESULT_INVALID"
        }));
    }
    catalog.reverse();
    for workflow in &mut catalog {
        workflow.jobs.reverse();
        for job in &mut workflow.jobs {
            job.needs.reverse();
            job.os.reverse();
        }
    }
    assert_eq!(render(&store.objects, &manifest, &catalog).unwrap(), files);
    assert!(root_files(&store.objects, &manifest).is_err()); // No synthetic CLI admission.
    assert_eq!(
        store.load("fixture-1").unwrap().1.aggregate_commit,
        prepared.aggregate_commit
    );
    assert_eq!(
        fs::read(store.path("fixture-1", "request")).unwrap(),
        request_bytes
    );
}

#[test]
fn generated_roots_refuse_workflow_template_and_path_drift() {
    for variant in [
        "missing",
        "extra",
        "digest",
        "template",
        "collision",
        "case-collision",
    ] {
        let (fixture, catalog) = fixture();
        let hub = fixture.root.join("bullet-farm");
        let portal = fixture.root.join("bullet-portal");
        match variant {
            "missing" => fs::remove_file(portal.join(ci_inventory::SCHEDULED)).unwrap(),
            "extra" => fs::write(portal.join(".github/workflows/extra.yml"), "extra").unwrap(),
            "digest" => fs::write(portal.join(ci_inventory::SCHEDULED), "changed matrix").unwrap(),
            "template" => fs::write(hub.join(TEMPLATE_PATH), "changed producer").unwrap(),
            _ => (),
        }
        if matches!(variant, "template") {
            commit(&hub, "changed template");
        }
        if matches!(variant, "missing" | "extra" | "digest") {
            commit(&portal, "changed workflow");
        }
        let (store, request, _) = fixture.prepared();
        let mut manifest = request.manifest;
        manifest.tool_config.schema_version = V2.into();
        if variant.ends_with("collision") {
            let target = if variant == "collision" {
                ".github/workflows/bullet-farm-scheduled.yml"
            } else {
                ".GITHUB/workflows/BULLET-FARM-scheduled.yml"
            };
            manifest
                .tool_config
                .root_files
                .insert(target.into(), "publication/root/README.md".into());
        }
        assert!(
            render(&store.objects, &manifest, &catalog).is_err(),
            "{variant}"
        );
    }
}

#[test]
fn tree_rendering_works_before_shallow_source_commits_are_available() {
    let (fixture, catalog) = fixture();
    let (store, request, prepared) = fixture.prepared();
    git::bytes(fixture.temp.path(), &["init", "--bare", "shallow.git"]).unwrap();
    let shallow = fixture.temp.path().join("shallow.git");
    git::execute(
        git::command(&shallow)
            .args(["fetch", "--depth=1", "--no-tags"])
            .arg(&store.objects)
            .arg(&prepared.aggregate_commit),
        None,
    )
    .unwrap();
    assert_eq!(
        read_manifest_at(&shallow, &prepared.aggregate_commit).unwrap(),
        request.manifest
    );
    for subject in request.manifest.members.values() {
        assert!(git::bytes(&shallow, &["cat-file", "-e", &subject.commit]).is_err());
    }
    let mut manifest = request.manifest;
    manifest.tool_config.schema_version = V2.into();
    assert_eq!(
        render(&shallow, &manifest, &catalog).unwrap(),
        render(&store.objects, &manifest, &catalog).unwrap()
    );
    // Construction and execution still need source commits; early tree inspection does not.
    assert!(store::build_tree(&shallow, &manifest).is_err());
}

#[test]
fn authentic_v1_roots_keep_original_request_and_commit_bytes() {
    let (fixture, _) = fixture();
    let (store, request, prepared) = fixture.prepared();
    let original_request = fs::read(store.path("fixture-1", "request")).unwrap();
    let original_commit = git::bytes(
        &store.objects,
        &["cat-file", "commit", &prepared.aggregate_commit],
    )
    .unwrap();
    let aggregate = fixture.aggregate(&store, &prepared);
    assert_eq!(
        git::blob(&aggregate, "HEAD", PRIMARY).unwrap(),
        TEMPLATE.as_bytes()
    );
    let preview: serde_json::Value =
        super::super::decode(preview(&aggregate).unwrap().as_bytes()).unwrap();
    assert_eq!(preview["purpose"], "EXPECTED_ROOT_FILES_ONLY");
    assert_eq!(preview["execution_evidence"], false);
    assert_eq!(preview["files"][PRIMARY], TEMPLATE);
    assert_eq!(
        preview["files"].as_object().unwrap().len(),
        request.manifest.tool_config.root_files.len()
    );
    assert_eq!(
        read_manifest_at(&aggregate, "HEAD").unwrap(),
        request.manifest
    );
    assert_eq!(
        store::build_commit(&store.objects, &request).unwrap(),
        prepared.aggregate_commit
    );
    assert_eq!(
        store.load("fixture-1").unwrap().1.aggregate_commit,
        prepared.aggregate_commit
    );
    assert_eq!(
        fs::read(store.path("fixture-1", "request")).unwrap(),
        original_request
    );
    assert_eq!(
        git::bytes(
            &store.objects,
            &["cat-file", "commit", &prepared.aggregate_commit]
        )
        .unwrap(),
        original_commit
    );
}

#[test]
fn preview_admits_exact_sources_without_claiming_existing_root_integrity() {
    for variant in ["root", "manifest", "member", "config", "dirty"] {
        let fixture = Fixture::new();
        let (store, _, prepared) = fixture.prepared();
        let aggregate = fixture.aggregate(&store, &prepared);
        let original: serde_json::Value =
            super::super::decode(preview(&aggregate).unwrap().as_bytes()).unwrap();
        let path = match variant {
            "manifest" => super::super::MANIFEST,
            "member" => "bullet-git/source.txt",
            "config" => "bullet-farm/publication/config.json",
            _ => "README.md",
        };
        let bytes = if variant == "manifest" {
            let mut bytes = fs::read(aggregate.join(path)).unwrap();
            bytes.push(b'\n');
            bytes
        } else {
            b"changed\n".to_vec()
        };
        fs::write(aggregate.join(path), bytes).unwrap();
        if variant != "dirty" {
            commit(&aggregate, "source admission negative");
        }
        let result = preview(&aggregate);
        assert!(read_manifest_at(&aggregate, "HEAD").is_err() || variant == "dirty");
        if variant == "root" {
            let changed: serde_json::Value =
                super::super::decode(result.unwrap().as_bytes()).unwrap();
            assert_eq!(changed["files"], original["files"]);
            assert_eq!(changed["execution_evidence"], false);
            assert_eq!(changed["purpose"], "EXPECTED_ROOT_FILES_ONLY");
        } else {
            assert!(result.is_err(), "{variant}");
        }
    }
}
