use std::{
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

use bullet_family::coord::{
    Applied, ClaimInput, ClaimSummary, CoordStore, GenerationId, GenesisInput, HandoffInput,
    MutationEnvelope, RequestId,
};
use serde::de::DeserializeOwned;

const FAMILY_REPOSITORIES: [(&str, &str); 4] = [
    ("bullet-farm", "root/bullet-farm"),
    ("bullet-kernel", "root/bullet-kernel"),
    ("bullet-git", "root/bullet-git"),
    ("bullet-portal", "root/bullet-portal"),
];

pub struct Harness {
    root: tempfile::TempDir,
    store: CoordStore,
    generation: GenerationId,
    next_request: AtomicU64,
}

impl Harness {
    pub fn new(name: &str) -> Self {
        let root = tempfile::Builder::new()
            .prefix(&format!("bullet-coord-v2-{name}-"))
            .tempdir()
            .unwrap();
        let mut manifest =
            "schema_version = \"1.2.0\"\nfamily = \"bullet-farm\"\nrequired_repos = [\n".to_owned();
        for (repo_name, _) in FAMILY_REPOSITORIES {
            writeln!(&mut manifest, "  \"{repo_name}\",").unwrap();
        }
        manifest.push_str("]\n");
        for (repo_name, jeryu_slug) in FAMILY_REPOSITORIES {
            let checkout = root.path().join(repo_name);
            fs::create_dir(&checkout).unwrap();
            writeln!(
                &mut manifest,
                "\n[[repo]]\nname = \"{repo_name}\"\npath = {}\njeryu_slug = \"{jeryu_slug}\"",
                serde_json::to_string(checkout.to_str().unwrap()).unwrap()
            )
            .unwrap();
        }
        fs::write(root.path().join("repos.manifest.toml"), manifest).unwrap();
        let repo = root.path().join("bullet-farm");
        git(&repo, &["init", "--quiet"]);
        git(&repo, &["config", "user.name", "Coord Test"]);
        git(&repo, &["config", "user.email", "coord@example.invalid"]);

        let store = CoordStore::new(root.path().to_path_buf());
        let status = store.initialize(&genesis()).unwrap();
        let generation = GenerationId::parse(status.generation_id).unwrap();
        Self {
            root,
            store,
            generation,
            next_request: AtomicU64::new(1),
        }
    }

    pub fn store(&self) -> &CoordStore {
        &self.store
    }

    pub fn reopen(&self) -> CoordStore {
        CoordStore::new(self.root.path().to_path_buf())
    }

    pub fn root(&self) -> &Path {
        self.root.path()
    }

    pub fn generation(&self) -> GenerationId {
        self.generation.clone()
    }

    pub fn mutation<T>(&self, command: T) -> MutationEnvelope<T> {
        let index = self.next_request.fetch_add(1, Ordering::Relaxed);
        self.mutation_with(index, command)
    }

    pub fn mutation_with<T>(&self, index: u64, command: T) -> MutationEnvelope<T> {
        MutationEnvelope {
            request_id: request_id(index),
            expected_generation_id: self.generation(),
            command,
        }
    }

    pub fn claim(&self, agent: &str, paths: &[&str]) -> Applied<ClaimSummary> {
        self.store
            .claim(&self.mutation(claim_input(agent, paths)))
            .unwrap()
    }

    pub fn handoff(&self, claim_id: &str, agent: &str, paths: &[&str]) -> Applied<ClaimSummary> {
        self.store
            .handoff(&self.mutation(HandoffInput {
                claim_id: claim_id.to_owned(),
                agent: agent.to_owned(),
                proof_command: "cargo test --locked".to_owned(),
                proof_exit_code: 0,
                changed_paths: strings(paths),
                commit_oid: None,
            }))
            .unwrap()
    }

    pub fn claim_and_handoff(&self, agent: &str, paths: &[&str]) -> String {
        let claim = self.claim(agent, paths);
        let claim_id = claim.projection.claim_id;
        self.handoff(&claim_id, agent, paths);
        claim_id
    }

    pub fn commit(&self, path: &str, contents: &str) -> String {
        self.commit_many(&[(path, contents)])
    }

    pub fn commit_many(&self, files: &[(&str, &str)]) -> String {
        let repo = self.root.path().join("bullet-farm");
        for (path, contents) in files {
            let target = repo.join(path);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::write(&target, contents).unwrap();
            git(&repo, &["add", path]);
        }
        git(&repo, &["commit", "--quiet", "-m", "fixture"]);
        git(&repo, &["rev-parse", "HEAD"])
    }

    pub fn empty_commit(&self) -> String {
        let repo = self.root.path().join("bullet-farm");
        git(
            &repo,
            &["commit", "--quiet", "--allow-empty", "-m", "empty"],
        );
        git(&repo, &["rev-parse", "HEAD"])
    }

    pub fn segment_len(&self) -> u64 {
        fs::metadata(self.segment_path()).unwrap().len()
    }

    pub fn last_record(&self) -> serde_json::Value {
        let text = fs::read_to_string(self.segment_path()).unwrap();
        let envelope: serde_json::Value = strict_json(text.lines().last().unwrap());
        envelope.get("record").unwrap().clone()
    }

    fn segment_path(&self) -> PathBuf {
        PathBuf::from(self.store.status().unwrap().source)
    }
}

pub fn claim_input(agent: &str, paths: &[&str]) -> ClaimInput {
    ClaimInput {
        agent: agent.to_owned(),
        lane: format!("lane-{agent}"),
        repo: "bullet-farm".to_owned(),
        paths: strings(paths),
        ttl_seconds: 600,
    }
}

pub fn request_id(index: u64) -> RequestId {
    RequestId::parse(format!("req_{index:064x}")).unwrap()
}

pub fn strict_json<T: DeserializeOwned>(text: &str) -> T {
    let value = bullet_wire::decode_unique_value(text.as_bytes()).unwrap();
    serde_json::from_value(value).unwrap()
}

fn genesis() -> GenesisInput {
    GenesisInput {
        operator: "coord-test-operator".to_owned(),
        policy_sha256: format!("sha256:{}", "a".repeat(64)),
        replay_contract_version: 1,
        replay_contract_sha256: format!("sha256:{}", "b".repeat(64)),
        bootstrap_commit_oid: "c".repeat(40),
        bootstrap_paths: vec!["Cargo.toml".to_owned(), "src".to_owned()],
    }
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("/usr/bin/git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}
