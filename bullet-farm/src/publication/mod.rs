//! Exact split-source identities for the public aggregate. No release authority.
mod ci_inventory;
mod ci_job;
mod ci_plan;
mod ci_render;
mod git;
mod observation;
mod pull_request;
mod scan;
mod store;
#[cfg(test)]
mod tests;
mod transport;
#[cfg(test)]
mod transport_tests;

use std::{collections::BTreeMap, path::Path};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::coord::CoordError;

type Result<T> = std::result::Result<T, CoordError>;
const MEMBERS: [&str; 4] = [
    "bullet-farm",
    "bullet-git",
    "bullet-kernel",
    "bullet-portal",
];
const DESTINATION: &str = "https://github.com/neverhuman/bulletfarm.git";
const JERYU_DESTINATION: &str = "https://git.neverhuman.org/git/root/bulletfarm.git";
const EMPTY_MAIN: &str = "0000000000000000000000000000000000000000";
const MANIFEST: &str = "publication.json";
const CONFIG: &str = "publication/config.json";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Config {
    schema_version: String,
    destination: String,
    root_files: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Subject {
    commit: String,
    tree: String,
    object_format: String,
    clean: bool,
    source_ref: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: String,
    tool_config: Config,
    members: BTreeMap<String, Subject>,
}

fn require(condition: bool, code: &'static str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(CoordError::new(code, "publication admission refused"))
    }
}

fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    let value = bullet_wire::decode_unique_value_bounded(bytes, 1024 * 1024)
        .map_err(|error| CoordError::new("PUBLICATION_JSON_INVALID", error.to_string()))?;
    serde_json::from_value(value).map_err(CoordError::json)
}

fn encode(value: &impl Serialize) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(CoordError::json)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn oid(value: &str) -> Result<()> {
    require(
        value.len() == 40
            && value
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)),
        "PUBLICATION_OID_INVALID",
    )
}

fn safe_path(value: &str) -> bool {
    !value.is_empty()
        && value.split('/').all(|part| {
            !part.is_empty()
                && !part.ends_with('.')
                && !part.eq_ignore_ascii_case(".git")
                && !part.to_ascii_lowercase().starts_with("git~")
                && part
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
        })
}

fn source_ref(name: &str, commit: &str) -> String {
    format!("refs/tags/bullet-source/v1/{name}/{commit}")
}

impl Config {
    fn validate(&self) -> Result<()> {
        require(
            matches!(
                self.schema_version.as_str(),
                "bullet.publication-config.v1" | "bullet.publication-config.v2"
            ) && (self.destination == DESTINATION || self.destination == JERYU_DESTINATION),
            "PUBLICATION_CONFIG_INVALID",
        )?;
        require(
            !self.root_files.is_empty() && self.root_files.len() <= 64,
            "PUBLICATION_ROOT_INVENTORY_INVALID",
        )?;
        for (target, source) in &self.root_files {
            require(
                safe_path(target)
                    && safe_path(source)
                    && source.starts_with("publication/root/")
                    && target != MANIFEST
                    && !MEMBERS.contains(&target.split('/').next().unwrap_or_default()),
                "PUBLICATION_ROOT_PATH_INVALID",
            )?;
        }
        for target in self.root_files.keys() {
            require(
                !self.root_files.keys().any(|other| {
                    other != target
                        && (other.starts_with(&format!("{target}/"))
                            || other.eq_ignore_ascii_case(target))
                }),
                "PUBLICATION_ROOT_PATH_COLLISION",
            )?;
        }
        Ok(())
    }
}

impl Manifest {
    fn validate(&self) -> Result<()> {
        self.tool_config.validate()?;
        require(
            self.schema_version == "bullet.publication.v1"
                && self.members.keys().map(String::as_str).eq(MEMBERS),
            "PUBLICATION_MEMBER_INVENTORY_INVALID",
        )?;
        for (name, subject) in &self.members {
            oid(&subject.commit)?;
            oid(&subject.tree)?;
            require(
                subject.clean
                    && subject.object_format == "sha1"
                    && subject.source_ref == source_ref(name, &subject.commit),
                "PUBLICATION_SOURCE_INVALID",
            )?;
        }
        Ok(())
    }
}

fn capture(root: &Path) -> Result<Manifest> {
    git::canonical_directory(root)?;
    crate::family_lock::validate_publication_family_root(root)?;
    let mut members = BTreeMap::new();
    for name in MEMBERS {
        let repo = root.join(name);
        git::checkout(&repo)?;
        let commit = git::text(&repo, &["rev-parse", "HEAD"])?;
        let subject = Subject {
            tree: git::text(&repo, &["rev-parse", "HEAD^{tree}"])?,
            object_format: git::text(&repo, &["rev-parse", "--show-object-format"])?,
            clean: true,
            source_ref: source_ref(name, &commit),
            commit,
        };
        members.insert(name.to_owned(), subject);
    }
    let hub = root.join("bullet-farm");
    let config = decode(&git::blob(&hub, &members["bullet-farm"].commit, CONFIG)?)?;
    let manifest = Manifest {
        schema_version: "bullet.publication.v1".into(),
        tool_config: config,
        members,
    };
    manifest.validate()?;
    Ok(manifest)
}

fn read_manifest(aggregate: &Path) -> Result<Manifest> {
    git::checkout(aggregate)?;
    read_manifest_at(aggregate, "HEAD")
}

fn read_source_manifest_at(aggregate: &Path, revision: &str) -> Result<Manifest> {
    let bytes = git::blob(aggregate, revision, MANIFEST)?;
    let manifest: Manifest = decode(&bytes)?;
    manifest.validate()?;
    require(
        encode(&manifest)? == bytes,
        "PUBLICATION_MANIFEST_NOT_CANONICAL",
    )?;
    let hub_revision = git::text(
        aggregate,
        &["rev-parse", &format!("{revision}:bullet-farm")],
    )?;
    let config: Config = decode(&git::blob(aggregate, &hub_revision, CONFIG)?)?;
    require(config == manifest.tool_config, "PUBLICATION_CONFIG_DRIFT")?;
    // Tree objects are present in a shallow aggregate before source commits are fetched.
    for (name, subject) in &manifest.members {
        require(
            git::text(aggregate, &["rev-parse", &format!("{revision}:{name}")])? == subject.tree,
            "PUBLICATION_TREE_MISMATCH",
        )?;
    }
    Ok(manifest)
}

fn read_manifest_at(aggregate: &Path, revision: &str) -> Result<Manifest> {
    let manifest = read_source_manifest_at(aggregate, revision)?;
    let root_files = ci_render::root_files(aggregate, &manifest)?;
    let mut expected = MEMBERS.iter().map(|n| (*n).to_owned()).collect::<Vec<_>>();
    expected.push(MANIFEST.into());
    for (target, rendered) in &root_files {
        if manifest.tool_config.schema_version == "bullet.publication-config.v2" {
            require(
                git::text(aggregate, &["ls-tree", revision, "--", target])?
                    .starts_with("100644 blob "),
                "PUBLICATION_ROOT_MODE_DRIFT",
            )?;
        }
        require(
            git::blob(aggregate, revision, target)? == *rendered,
            "PUBLICATION_TEMPLATE_DRIFT",
        )?;
        expected.push(target.split('/').next().unwrap_or_default().into());
    }
    expected.sort();
    expected.dedup();
    require(
        git::text(aggregate, &["ls-tree", "--name-only", revision])?
            .lines()
            .eq(expected.iter().map(String::as_str)),
        "PUBLICATION_ROOT_INVENTORY_DRIFT",
    )?;
    // Each root directory must contain exactly its selected leaves.
    let mut leaves = vec![MANIFEST.to_owned()];
    leaves.extend(root_files.keys().cloned());
    let actual = git::text(aggregate, &["ls-tree", "-r", "--name-only", revision])?;
    let mut actual = actual
        .lines()
        .filter(|p| !MEMBERS.iter().any(|m| p.starts_with(&format!("{m}/"))))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    actual.sort();
    leaves.sort();
    require(actual == leaves, "PUBLICATION_ROOT_INVENTORY_DRIFT")?;
    Ok(manifest)
}

pub fn run(args: Vec<String>) -> Result<String> {
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["inspect", root] => String::from_utf8(encode(&capture(Path::new(root))?)?)
            .map_err(|_| CoordError::new("PUBLICATION_ENCODING", "UTF-8 required")),
        ["prepare", root, store, request, base] => {
            store::prepare(Path::new(root), Path::new(store), request, base)
        }
        ["verify", aggregate] => {
            read_manifest(Path::new(aggregate))?;
            Ok("publication integrity passed".into())
        }
        ["reconstruct", aggregate, root] => {
            transport::reconstruct(Path::new(aggregate), Path::new(root))
        }
        ["ci-observe", aggregate, root, artifacts] => {
            observation::write(Path::new(aggregate), Path::new(root), Path::new(artifacts))
        }
        ["ci-root-files", aggregate] => ci_render::preview(Path::new(aggregate)),
        ["ci-plan", aggregate] => ci_plan::run(Path::new(aggregate)),
        ["ci-plan", store, request] => ci_plan::stored(Path::new(store), request),
        ["ci-job-context", aggregate, root, key] => {
            ci_job::run(Path::new(aggregate), Path::new(root), key, None)
        }
        ["ci-job-observe", aggregate, root, key, artifacts] => ci_job::run(
            Path::new(aggregate),
            Path::new(root),
            key,
            Some(Path::new(artifacts)),
        ),
        ["push", store, request] => transport::push(Path::new(store), request),
        ["pr", path, id] => {
            {
                let store = store::Store::open(Path::new(path))?;
                let (request, _) = store.load(id)?;
                require(
                    request.manifest.tool_config.destination == DESTINATION,
                    "PUBLICATION_JERYU_PR_TRANSPORT_UNAVAILABLE",
                )?;
            }
            pull_request::run(Path::new(path), id)
        }
        ["scan", path, id] => {
            let store = store::Store::open(Path::new(path))?;
            let (request, prepared) = store.load(id)?;
            scan::scan(&store, &request, &prepared)?;
            Ok("complete reachable-object history scan passed".into())
        }
        _ => Err(CoordError::new(
            "USAGE",
            "bullet-publish inspect ROOT | prepare ROOT STORE REQUEST EXPECTED_MAIN | verify AGGREGATE | reconstruct AGGREGATE NEW_ROOT | ci-root-files AGGREGATE | ci-plan AGGREGATE | ci-plan STORE REQUEST | ci-job-context AGGREGATE ROOT KEY | ci-job-observe AGGREGATE ROOT KEY ARTIFACTS | ci-observe AGGREGATE ROOT ARTIFACTS | scan STORE REQUEST | push STORE REQUEST | pr STORE REQUEST",
        )),
    }
}
