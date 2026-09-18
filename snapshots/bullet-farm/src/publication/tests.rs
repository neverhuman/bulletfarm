use std::{
    fs,
    path::{Path, PathBuf},
};

use super::*;

mod ci_contract;

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
    ci_contract::contract();
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
    ci_contract::workflow_refusals();
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
