use std::{
    fs,
    path::{Path, PathBuf},
};

use super::*;

pub(super) struct Fixture {
    pub temp: tempfile::TempDir,
    pub root: PathBuf,
}

pub(super) fn commit(repo: &Path, message: &str) -> String {
    git::bytes(repo, &["add", "."]).unwrap();
    git::execute(
        git::command(repo)
            .env("GIT_AUTHOR_NAME", "fixture")
            .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
            .env("GIT_COMMITTER_NAME", "fixture")
            .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
            .args(["-c", "commit.gpgSign=false", "commit", "-m", message]),
        None,
    )
    .unwrap();
    git::text(repo, &["rev-parse", "HEAD"]).unwrap()
}

impl Fixture {
    pub fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("family");
        fs::create_dir(&root).unwrap();
        let portable = include_bytes!("../../publication/root/repos.manifest.toml");
        fs::write(root.join("repos.manifest.toml"), portable).unwrap();
        for name in MEMBERS {
            git::bytes(&root, &["init", "--template=", name]).unwrap();
            let repo = root.join(name);
            fs::write(repo.join("source.txt"), name).unwrap();
            if name == "bullet-farm" {
                fs::create_dir_all(repo.join("publication/root")).unwrap();
                let config = Config {
                    schema_version: "bullet.publication-config.v1".into(),
                    destination: DESTINATION.into(),
                    root_files: BTreeMap::from([
                        (
                            ".github/workflows/publication.yml".into(),
                            "publication/root/.github/workflows/publication.yml".into(),
                        ),
                        ("README.md".into(), "publication/root/README.md".into()),
                        (
                            "repos.manifest.toml".into(),
                            "publication/root/repos.manifest.toml".into(),
                        ),
                    ]),
                };
                fs::write(repo.join(CONFIG), encode(&config).unwrap()).unwrap();
                fs::write(
                    repo.join("publication/root/README.md"),
                    "fixture publication\n",
                )
                .unwrap();
                fs::create_dir_all(repo.join("publication/root/.github/workflows")).unwrap();
                fs::write(
                    repo.join("publication/root/.github/workflows/publication.yml"),
                    "name: fixture\n",
                )
                .unwrap();
                fs::write(repo.join("publication/root/repos.manifest.toml"), portable).unwrap();
                fs::write(
                    repo.join("publication/gitleaks.toml"),
                    "[extend]\nuseDefault = true\n",
                )
                .unwrap();
            }
            commit(&repo, "fixture source");
        }
        Self { temp, root }
    }

    pub fn prepared(&self) -> (store::Store, store::Request, store::Prepared) {
        let store = store::Store::open(&self.temp.path().join("store")).unwrap();
        let manifest = capture(&self.root).unwrap();
        for (name, subject) in &manifest.members {
            git::execute(
                git::command(&store.objects)
                    .args(["fetch", "--no-tags"])
                    .arg(self.root.join(name))
                    .arg(&subject.commit),
                None,
            )
            .unwrap();
        }
        let request = store::Request {
            schema_version: "bullet.publication-request.v1".into(),
            request_id: "fixture-1".into(),
            expected_main: manifest.members["bullet-farm"].commit.clone(),
            manifest,
        };
        let prepared = store::Prepared {
            schema_version: "bullet.publication-prepared.v1".into(),
            aggregate_commit: store::build_commit(&store.objects, &request).unwrap(),
            request_sha256: store::digest(&encode(&request).unwrap()),
            review_ref: "refs/heads/publication/fixture-1".into(),
        };
        store::persist(
            &store.path("fixture-1", "request"),
            &encode(&request).unwrap(),
        )
        .unwrap();
        store::persist(
            &store.path("fixture-1", "prepared"),
            &encode(&prepared).unwrap(),
        )
        .unwrap();
        (store, request, prepared)
    }

    pub fn aggregate(&self, store: &store::Store, prepared: &store::Prepared) -> PathBuf {
        let aggregate = self.temp.path().join("aggregate");
        git::bytes(self.temp.path(), &["init", "--template=", "aggregate"]).unwrap();
        git::execute(
            git::command(&aggregate)
                .arg("fetch")
                .arg(&store.objects)
                .arg(&prepared.aggregate_commit),
            None,
        )
        .unwrap();
        git::bytes(
            &aggregate,
            &["checkout", "--detach", &prepared.aggregate_commit],
        )
        .unwrap();
        aggregate
    }
}

#[test]
fn deterministic_publication_preserves_all_exact_source_trees_and_templates() {
    ci_plan_contract();
    let fixture = Fixture::new();
    let (store, request, prepared) = fixture.prepared();
    assert_eq!(
        store::build_commit(&store.objects, &request).unwrap(),
        prepared.aggregate_commit
    );
    let aggregate = fixture.aggregate(&store, &prepared);
    assert_eq!(read_manifest(&aggregate).unwrap(), request.manifest);
    assert_eq!(
        git::text(&aggregate, &["rev-parse", "HEAD^"]).unwrap(),
        request.expected_main
    );
    assert!(
        !String::from_utf8(encode(&request.manifest).unwrap())
            .unwrap()
            .contains(&prepared.aggregate_commit)
    );
    assert!(store.load("fixture-1").is_ok());
}

#[test]
fn publication_refuses_dirty_hidden_flags_and_symlinked_checkouts() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    let repo = fixture.root.join("bullet-git");
    fs::write(repo.join("untracked"), "dirty").unwrap();
    assert!(capture(&fixture.root).is_err());
    fs::remove_file(repo.join("untracked")).unwrap();
    for flag in ["--assume-unchanged", "--skip-worktree"] {
        git::bytes(&repo, &["update-index", flag, "source.txt"]).unwrap();
        assert!(capture(&fixture.root).is_err());
        git::bytes(
            &repo,
            &[
                "update-index",
                "--no-assume-unchanged",
                "--no-skip-worktree",
                "source.txt",
            ],
        )
        .unwrap();
    }
    let moved = fixture.root.join("moved");
    fs::rename(&repo, &moved).unwrap();
    symlink(&moved, &repo).unwrap();
    assert!(capture(&fixture.root).is_err());
}

#[test]
fn publication_refuses_history_and_endpoint_substitution() {
    let fixture = Fixture::new();
    let repo = fixture.root.join("bullet-git");
    let original = capture(&fixture.root).unwrap();
    git::bytes(
        &repo,
        &["config", "--local", "protocol.blocked-push.allow", "never"],
    )
    .unwrap();
    assert_eq!(capture(&fixture.root).unwrap(), original);
    git::bytes(
        &repo,
        &[
            "config",
            "--local",
            "--add",
            "protocol.blocked-push.allow",
            "always",
        ],
    )
    .unwrap();
    assert!(capture(&fixture.root).is_err());
    git::bytes(
        &repo,
        &[
            "config",
            "--local",
            "--unset-all",
            "protocol.blocked-push.allow",
        ],
    )
    .unwrap();
    for (key, value) in [
        ("protocol.blocked-push.allow", "always"),
        ("protocol.https.allow", "always"),
        ("url.https://elsewhere.invalid/.insteadOf", DESTINATION),
        ("include.path", "/tmp/config"),
        ("filter.hostile.clean", "false"),
        ("remote.origin.promisor", "true"),
    ] {
        git::bytes(&repo, &["config", "--local", key, value]).unwrap();
        assert!(capture(&fixture.root).is_err(), "{key}");
        git::bytes(&repo, &["config", "--local", "--unset", key]).unwrap();
    }
    let commit = git::text(&repo, &["rev-parse", "HEAD"]).unwrap();
    fs::write(repo.join(".git/shallow"), format!("{commit}\n")).unwrap();
    assert!(capture(&fixture.root).is_err());
    fs::remove_file(repo.join(".git/shallow")).unwrap();
    fs::write(
        repo.join(".git/objects/info/alternates"),
        "/tmp/elsewhere\n",
    )
    .unwrap();
    assert!(capture(&fixture.root).is_err());
}

#[test]
fn publication_manifest_refuses_duplicates_unknowns_paths_and_noncanonical_bytes() {
    assert!(decode::<Manifest>(br#"{"schema_version":"x","schema_version":"y"}"#).is_err());
    let fixture = Fixture::new();
    let (store, request, prepared) = fixture.prepared();
    for target in [
        "../escape",
        ".git/config",
        ".GIT/config",
        "file./x",
        "bullet-git/overwritten",
        "publication.json",
    ] {
        let mut manifest = request.manifest.clone();
        manifest
            .tool_config
            .root_files
            .insert(target.into(), "publication/root/README.md".into());
        assert!(manifest.validate().is_err(), "{target}");
    }
    let mut config = request.manifest.tool_config.clone();
    config.root_files.insert(
        "README.md/child".into(),
        "publication/root/README.md".into(),
    );
    assert!(config.validate().is_err());
    let aggregate = fixture.aggregate(&store, &prepared);
    let mut value = serde_json::to_value(&request.manifest).unwrap();
    value["aggregate_commit"] = serde_json::json!(prepared.aggregate_commit);
    fs::write(aggregate.join(MANIFEST), encode(&value).unwrap()).unwrap();
    commit(&aggregate, "hostile manifest extra field");
    assert!(read_manifest(&aggregate).is_err());
}

#[test]
fn publication_rejects_template_and_member_subtree_drift() {
    ci_plan_workflow_refusals();
    let fixture = Fixture::new();
    let (store, _, prepared) = fixture.prepared();
    let aggregate = fixture.aggregate(&store, &prepared);
    fs::write(aggregate.join("README.md"), "changed root").unwrap();
    commit(&aggregate, "hostile generated root");
    assert!(read_manifest(&aggregate).is_err());
    git::bytes(
        &aggregate,
        &["checkout", "--detach", &prepared.aggregate_commit],
    )
    .unwrap();
    fs::write(aggregate.join("bullet-git/source.txt"), "changed member").unwrap();
    commit(&aggregate, "hostile aggregate member");
    assert!(read_manifest(&aggregate).is_err());
}

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

fn ci_plan_contract() {
    let reviewed = ci_inventory::reviewed();
    for variant in [
        "duplicate-workflow",
        "duplicate-job",
        "duplicate-cell",
        "dependency",
        "cycle",
    ] {
        let mut bad = reviewed.clone();
        match variant {
            "duplicate-workflow" => bad[1] = bad[0].clone(),
            "duplicate-job" => bad[0].jobs.extend_from_within(..1),
            "duplicate-cell" => bad[5].jobs[5].os.push("macos-15"),
            "dependency" => bad[0].jobs[1].needs.push("invented"),
            "cycle" => bad[0].jobs[0].needs.push("fast"),
            _ => unreachable!(),
        }
        assert!(ci_inventory::canonical_catalog(&bad).is_err(), "{variant}");
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
    assert_eq!(invocations.len(), 55);
    assert_eq!(value["plan"]["job_definition_count"], 53);
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
    assert_eq!((required, matrix), (27, 4));
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

fn ci_plan_workflow_refusals() {
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

#[test]
fn durable_requests_refuse_changed_inputs_and_same_tree_wrong_parent() {
    let fixture = Fixture::new();
    let (store, request, mut prepared) = fixture.prepared();
    store::persist(
        &store.path("fixture-1", "request"),
        &encode(&request).unwrap(),
    )
    .unwrap();
    let mut changed = request.clone();
    changed.expected_main = request.manifest.members["bullet-git"].commit.clone();
    assert!(
        store::persist(
            &store.path("fixture-1", "request"),
            &encode(&changed).unwrap()
        )
        .is_err()
    );
    prepared.aggregate_commit = store::build_commit(&store.objects, &changed).unwrap();
    fs::write(
        store.path("fixture-1", "prepared"),
        encode(&prepared).unwrap(),
    )
    .unwrap();
    assert!(store.load("fixture-1").is_err());
    fs::remove_file(store.path("fixture-1", "prepared")).unwrap();
    std::os::unix::fs::symlink(
        store.path("fixture-1", "request"),
        store.path("fixture-1", "prepared"),
    )
    .unwrap();
    assert!(store.load("fixture-1").is_err());
}
