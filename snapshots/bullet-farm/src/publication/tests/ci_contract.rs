use super::*;

fn ci_fixture() -> (Fixture, Vec<ci_inventory::Workflow>) {
    let fixture = Fixture::new();
    let mut catalog = ci_inventory::reviewed();
    for workflow in &mut catalog {
        let jobs = workflow
            .jobs
            .iter()
            .map(|job| {
                (
                    job.id,
                    serde_json::json!({"runs-on": job.runner, "needs": job.needs,
                "if": if job.always { "always()" } else { "success()" },
                "strategy": {"matrix": {"os": job.os}}}),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let bytes =
            bullet_wire::canonical_json(&serde_json::json!({"name": "fixture", "jobs": jobs}))
                .unwrap();
        workflow.sha256 = store::digest(&bytes);
        let repo = fixture.root.join(workflow.member);
        fs::create_dir_all(repo.join(".github/workflows")).unwrap();
        fs::write(repo.join(workflow.path), bytes).unwrap();
    }
    for member in MEMBERS {
        commit(&fixture.root.join(member), "fixture workflow catalog");
    }
    (fixture, catalog)
}

pub(super) fn contract() {
    let reviewed = ci_inventory::reviewed();
    for variant in [
        "duplicate-workflow",
        "duplicate-job",
        "duplicate-cell",
        "dependency",
        "cycle",
        "missing-workflow",
        "missing-job",
        "extra-job",
        "runner-swap",
        "removed-needs",
        "added-needs",
        "runner",
        "always",
        "missing-cell",
        "extra-cell",
        "matrix-relocated",
        "known-job-reassigned",
    ] {
        let mut bad = reviewed.clone();
        match variant {
            "duplicate-workflow" => bad[1] = bad[0].clone(),
            "duplicate-job" => bad[0].jobs.extend_from_within(..1),
            "duplicate-cell" => bad[5].jobs[5].os.push("macos-15"),
            "dependency" => bad[0].jobs[1].needs.push("invented"),
            "cycle" => bad[0].jobs[0].needs.push("fast"),
            "missing-workflow" => {
                bad.pop();
            }
            "missing-job" => {
                bad[0].jobs.pop();
            }
            "extra-job" => {
                let mut job = bad[0].jobs[0].clone();
                job.id = "extra";
                bad[0].jobs.push(job);
            }
            "runner-swap" => {
                // Both jobs, timeouts and totals stay valid; their reviewed runners differ.
                let last = bad[1].jobs.len() - 1;
                let runner = bad[1].jobs[last].runner;
                bad[1].jobs[last].runner = bad[1].jobs[last - 1].runner;
                bad[1].jobs[last - 1].runner = runner;
            }
            "removed-needs" => bad[0].jobs[1].needs.clear(),
            "added-needs" => bad[0].jobs[2].needs.push("fast"),
            "runner" => bad[0].jobs[0].runner = "macos-15",
            "always" => bad[0].jobs[0].always = true,
            "missing-cell" => {
                bad[5].jobs[5].os.pop();
            }
            "extra-cell" => bad[5].jobs[5].os.push("ubuntu-24.04"),
            "matrix-relocated" => {
                let matrix = bad[5].jobs[5].clone();
                bad[5].jobs[1].runner = matrix.runner;
                bad[5].jobs[1].os = matrix.os;
                bad[5].jobs[5].runner = "ubuntu-24.04";
                bad[5].jobs[5].os.clear();
            }
            "known-job-reassigned" => bad[7].jobs[1] = bad[1].jobs[4].clone(),
            _ => unreachable!(),
        }
        let error = ci_inventory::canonical_catalog(&bad).err().unwrap();
        if matches!(
            variant,
            "runner-swap"
                | "removed-needs"
                | "added-needs"
                | "runner"
                | "always"
                | "matrix-relocated"
        ) {
            assert_eq!(
                error.code(),
                "PUBLICATION_CI_JOB_TOPOLOGY_INVALID",
                "{variant}"
            );
        }
    }
    let (fixture, catalog) = ci_fixture();
    let (store, request, prepared) = fixture.prepared();
    let aggregate = fixture.aggregate(&store, &prepared);
    for subject in request.manifest.members.values() {
        git::execute(
            git::command(&aggregate)
                .arg("fetch")
                .arg(&store.objects)
                .arg(&subject.commit),
            None,
        )
        .unwrap();
    }
    let environment = std::env::var_os("GITHUB_SHA");
    let bytes = ci_plan::build(&aggregate, &catalog).unwrap();
    let value = bullet_wire::decode_canonical_value(bytes.as_bytes()).unwrap();
    assert_eq!(
        value["plan_sha256"],
        store::digest(&bullet_wire::canonical_json(&value["plan"]).unwrap())
    );
    assert_eq!(value["plan"]["aggregate_commit"], prepared.aggregate_commit);
    assert_eq!(value["plan"]["purpose"], "EXPECTED_INVOCATIONS_ONLY");
    assert_eq!(value["plan"]["execution_evidence"], false);
    let invocations = value["plan"]["invocations"].as_object().unwrap();
    let jobs = catalog.iter().flat_map(|w| &w.jobs).collect::<Vec<_>>();
    assert_eq!(
        invocations.len(),
        jobs.iter().map(|j| j.os.len().max(1)).sum::<usize>()
    );
    assert_eq!(value["plan"]["workflow_count"], catalog.len());
    assert_eq!(value["plan"]["job_definition_count"], jobs.len());
    assert_eq!(value["plan"]["invocation_count"], invocations.len());
    let (mut required, mut matrix) = (0, 0);
    for row in invocations.values() {
        required += usize::from(row["scope"] == "REQUIRED");
        matrix += usize::from(!row["matrix"].as_object().unwrap().is_empty());
        let member = &request.manifest.members[row["member"].as_str().unwrap()];
        assert_eq!(row["member_commit"], member.commit);
        assert_eq!(row["member_tree"], member.tree);
        for dependency in row["needs"].as_array().unwrap() {
            assert!(invocations.contains_key(dependency.as_str().unwrap()));
        }
    }
    assert_eq!(
        required,
        catalog
            .iter()
            .filter(|w| w.scope == "REQUIRED")
            .flat_map(|w| &w.jobs)
            .map(|j| j.os.len().max(1))
            .sum::<usize>()
    );
    assert_eq!(matrix, jobs.iter().map(|j| j.os.len()).sum::<usize>());
    let mut reordered = catalog.clone();
    reordered.reverse();
    for workflow in &mut reordered {
        workflow.jobs.reverse();
        for job in &mut workflow.jobs {
            job.needs.reverse();
            job.os.reverse();
        }
    }
    assert_eq!(ci_plan::build(&aggregate, &reordered).unwrap(), bytes);
    assert!(run(vec!["ci-plan".into(), aggregate.display().to_string()]).is_err());
    let store_root = store.root.clone();
    assert!(ci_plan::stored_with(&store_root, "fixture-1", &catalog).is_err());
    drop(store);
    assert_eq!(
        ci_plan::stored_with(&store_root, "fixture-1", &catalog).unwrap(),
        bytes
    );
    assert_eq!(std::env::var_os("GITHUB_SHA"), environment);
    let lock = store_root.join("lock");
    fs::remove_file(&lock).unwrap();
    assert!(ci_plan::stored_with(&store_root, "fixture-1", &catalog).is_err());
    assert!(!lock.exists());
    let missing = fixture.temp.path().join("missing-store");
    assert!(ci_plan::stored(&missing, "fixture-1").is_err());
    assert!(!missing.exists());
    let mut substituted = request.manifest;
    let wrong = substituted.members["bullet-farm"].commit.clone();
    let subject = substituted.members.get_mut("bullet-git").unwrap();
    subject.commit = wrong.clone();
    subject.source_ref = source_ref("bullet-git", &wrong);
    fs::write(aggregate.join(MANIFEST), encode(&substituted).unwrap()).unwrap();
    commit(&aggregate, "wrong member commit with unchanged member tree");
    assert_eq!(
        ci_plan::build(&aggregate, &catalog).unwrap_err().code(),
        "PUBLICATION_CI_SOURCE_TREE_MISMATCH"
    );
}

pub(super) fn workflow_refusals() {
    for variant in ["missing", "extra", "job", "matrix"] {
        let (fixture, catalog) = ci_fixture();
        let member = fixture.root.join("bullet-portal");
        let path = member.join(ci_inventory::SCHEDULED);
        match variant {
            "missing" => fs::remove_file(&path).unwrap(),
            "extra" => fs::write(member.join(".github/workflows/extra.yml"), "jobs: {}\n").unwrap(),
            _ => {
                let text = fs::read_to_string(&path).unwrap();
                let changed = if variant == "job" {
                    text.replace("hygiene", "invented")
                } else {
                    text.replace("macos-15", "macos-14")
                };
                assert_ne!(changed, text);
                fs::write(path, changed).unwrap();
            }
        }
        commit(&member, "workflow drift with valid aggregate manifest");
        let (store, _, _) = fixture.prepared();
        let root = store.root.clone();
        drop(store);
        let expected = if matches!(variant, "missing" | "extra") {
            "PUBLICATION_CI_WORKFLOW_INVENTORY_DRIFT"
        } else {
            "PUBLICATION_CI_WORKFLOW_DIGEST_DRIFT"
        };
        assert_eq!(
            ci_plan::stored_with(&root, "fixture-1", &catalog)
                .unwrap_err()
                .code(),
            expected,
            "{variant}"
        );
    }
}
