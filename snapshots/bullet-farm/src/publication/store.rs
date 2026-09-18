use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use fs2::FileExt;
use serde::{Deserialize, Serialize};

use super::{MANIFEST, Manifest, Result, capture, decode, encode, git, oid, require};
use crate::coord::CoordError;

#[derive(Clone, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Request {
    pub schema_version: String,
    pub request_id: String,
    pub expected_main: String,
    pub manifest: Manifest,
}

impl Request {
    pub(super) fn bootstrap(&self) -> Result<bool> {
        oid(&self.expected_main)?;
        let bootstrap = self.expected_main == super::EMPTY_MAIN;
        require(
            !bootstrap || self.manifest.tool_config.destination == super::JERYU_DESTINATION,
            "PUBLICATION_BOOTSTRAP_DESTINATION_INVALID",
        )?;
        Ok(bootstrap)
    }
}

#[derive(Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Prepared {
    pub schema_version: String,
    pub aggregate_commit: String,
    pub request_sha256: String,
    pub review_ref: String,
}

pub(super) struct Store {
    pub root: PathBuf,
    pub objects: PathBuf,
    _lock: File,
}

pub(super) fn digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn request_id(id: &str) -> Result<()> {
    require(
        !id.is_empty()
            && id.len() <= 64
            && id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-'),
        "PUBLICATION_REQUEST_ID_INVALID",
    )
}

pub(super) fn persist(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| CoordError::new("PUBLICATION_PATH_INVALID", "parent required"))?;
    let mut stage = tempfile::NamedTempFile::new_in(parent).map_err(CoordError::io)?;
    stage.write_all(bytes).map_err(CoordError::io)?;
    stage.as_file().sync_all().map_err(CoordError::io)?;
    match stage.persist_noclobber(path) {
        Ok(_) => File::open(parent)
            .and_then(|file| file.sync_all())
            .map_err(CoordError::io),
        Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
            require(
                fs::symlink_metadata(path)
                    .map_err(CoordError::io)?
                    .is_file(),
                "PUBLICATION_STORE_SUBSTITUTED",
            )?;
            require(
                fs::read(path).map_err(CoordError::io)? == bytes,
                "PUBLICATION_REQUEST_CONFLICT",
            )
        }
        Err(error) => Err(CoordError::io(error.error)),
    }
}

fn read_record(path: &Path) -> Result<Vec<u8>> {
    use std::{io::Read, os::unix::fs::OpenOptionsExt};
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_NONBLOCK)
        .open(path)
        .map_err(CoordError::io)?;
    let metadata = file.metadata().map_err(CoordError::io)?;
    require(
        metadata.is_file() && metadata.len() <= 1024 * 1024,
        "PUBLICATION_RECORD_INVALID",
    )?;
    let mut bytes = Vec::new();
    (&mut file)
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    require(bytes.len() <= 1024 * 1024, "PUBLICATION_RECORD_INVALID")?;
    Ok(bytes)
}

impl Store {
    pub fn open(root: &Path) -> Result<Self> {
        use std::os::unix::fs::OpenOptionsExt;
        let parent = root
            .parent()
            .ok_or_else(|| CoordError::new("PUBLICATION_STORE_INVALID", "parent required"))?;
        git::canonical_directory(parent)?;
        if !root.exists() {
            fs::create_dir(root).map_err(CoordError::io)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(root, fs::Permissions::from_mode(0o700))
                    .map_err(CoordError::io)?;
            }
            File::open(parent)
                .and_then(|file| file.sync_all())
                .map_err(CoordError::io)?;
        }
        git::canonical_directory(root)?;
        let lock_path = root.join("lock");
        if let Ok(metadata) = fs::symlink_metadata(&lock_path) {
            require(metadata.is_file(), "PUBLICATION_STORE_SUBSTITUTED")?;
        }
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_NONBLOCK)
            .open(lock_path)
            .map_err(CoordError::io)?;
        lock.try_lock_exclusive().map_err(|_| {
            CoordError::new(
                "PUBLICATION_STORE_BUSY",
                "another request owns publication custody",
            )
        })?;
        let objects = root.join("objects.git");
        if !objects.exists() {
            git::bytes(root, &["init", "--bare", "objects.git"])?;
        }
        git::canonical_directory(&objects)?;
        require(
            git::text(&objects, &["rev-parse", "--is-bare-repository"])? == "true",
            "PUBLICATION_STORE_NOT_BARE",
        )?;
        git::metadata_admission(&objects, &objects)?;
        Ok(Self {
            root: root.into(),
            objects,
            _lock: lock,
        })
    }

    pub fn path(&self, id: &str, kind: &str) -> PathBuf {
        self.root.join(format!("{id}.{kind}.json"))
    }

    pub fn load(&self, id: &str) -> Result<(Request, Prepared)> {
        request_id(id)?;
        let request_bytes = read_record(&self.path(id, "request"))?;
        let request: Request = decode(&request_bytes)?;
        let prepared: Prepared = decode(&read_record(&self.path(id, "prepared"))?)?;
        request.manifest.validate()?;
        request.bootstrap()?;
        oid(&prepared.aggregate_commit)?;
        require(
            request.schema_version == "bullet.publication-request.v1"
                && request.request_id == id
                && prepared.schema_version == "bullet.publication-prepared.v1"
                && prepared.review_ref == format!("refs/heads/publication/{id}")
                && prepared.request_sha256 == digest(&request_bytes),
            "PUBLICATION_STORE_DRIFT",
        )?;
        require(
            build_commit(&self.objects, &request)? == prepared.aggregate_commit,
            "PUBLICATION_STORE_DRIFT",
        )?;
        Ok((request, prepared))
    }
}

fn write_blob(repo: &Path, bytes: &[u8]) -> Result<String> {
    String::from_utf8(git::execute(
        git::command(repo).args(["hash-object", "-w", "--stdin"]),
        Some(bytes),
    )?)
    .map(|value| value.trim_end().into())
    .map_err(|_| CoordError::new("PUBLICATION_OBJECT_INVALID", "invalid blob identity"))
}

pub(super) fn build_tree(repo: &Path, manifest: &Manifest) -> Result<String> {
    let temporary = tempfile::tempdir().map_err(CoordError::io)?;
    let index = temporary.path().join("index");
    let mut entries = Vec::new();
    for (name, subject) in &manifest.members {
        require(
            git::text(
                repo,
                &["rev-parse", &format!("{}^{{tree}}", subject.commit)],
            )? == subject.tree,
            "PUBLICATION_SOURCE_TREE_MISMATCH",
        )?;
        // read-tree imports trees without filtering or checking out member files.
        git::execute(
            git::command(repo).env("GIT_INDEX_FILE", &index).args([
                "read-tree",
                "-i",
                &format!("--prefix={name}/"),
                &subject.tree,
            ]),
            None,
        )?;
    }
    entries.push((MANIFEST.to_owned(), write_blob(repo, &encode(manifest)?)?));
    for (target, bytes) in super::ci_render::root_files(repo, manifest)? {
        entries.push((target, write_blob(repo, &bytes)?));
    }
    for (path, object) in entries {
        git::execute(
            git::command(repo).env("GIT_INDEX_FILE", &index).args([
                "update-index",
                "--add",
                "--cacheinfo",
                &format!("100644,{object},{path}"),
            ]),
            None,
        )?;
    }
    String::from_utf8(git::execute(
        git::command(repo)
            .env("GIT_INDEX_FILE", &index)
            .arg("write-tree"),
        None,
    )?)
    .map(|value| value.trim_end().into())
    .map_err(|_| CoordError::new("PUBLICATION_OBJECT_INVALID", "invalid tree identity"))
}

pub(super) fn build_commit(repo: &Path, request: &Request) -> Result<String> {
    let tree = build_tree(repo, &request.manifest)?;
    let mut command = git::command(repo);
    // Fixed identity/date plus bound parent, tree, and request make retries deterministic.
    // This technical author is not an independent reviewer or release signer.
    for role in ["AUTHOR", "COMMITTER"] {
        command.env(format!("GIT_{role}_NAME"), "Bullet publication");
        command.env(format!("GIT_{role}_EMAIL"), "publication@bullet.invalid");
        command.env(format!("GIT_{role}_DATE"), "2000-01-01T00:00:00Z");
    }
    command.args(["-c", "commit.gpgSign=false", "commit-tree", &tree]);
    if !request.bootstrap()? {
        command.args(["-p", &request.expected_main]);
    }
    let message = format!(
        "Publish exact Bullet family: {}\n\nRequest-SHA256: {}\n",
        request.request_id,
        digest(&encode(request)?)
    );
    String::from_utf8(git::execute(&mut command, Some(message.as_bytes()))?)
        .map(|value| value.trim_end().into())
        .map_err(|_| CoordError::new("PUBLICATION_OBJECT_INVALID", "invalid commit identity"))
}

pub(super) fn prepare(root: &Path, store: &Path, id: &str, base: &str) -> Result<String> {
    let destination = capture(root)?.tool_config.destination;
    prepare_from(root, store, id, base, &destination)
}

pub(super) fn prepare_from(
    root: &Path,
    store: &Path,
    id: &str,
    base: &str,
    destination: &str,
) -> Result<String> {
    request_id(id)?;
    oid(base)?;
    let manifest = capture(root)?;
    // Publication storage belongs to the Hub metadata, not a second checkout.
    require(
        store == root.join("bullet-farm/.git/bullet-publication"),
        "PUBLICATION_STORE_LOCATION_INVALID",
    )?;
    let store = Store::open(store)?;
    let request = Request {
        schema_version: "bullet.publication-request.v1".into(),
        request_id: id.into(),
        expected_main: base.into(),
        manifest,
    };
    request.bootstrap()?;
    require(
        destination == request.manifest.tool_config.destination
            || Path::new(destination).is_absolute(),
        "PUBLICATION_DESTINATION_MISMATCH",
    )?;
    persist(&store.path(id, "request"), &encode(&request)?)?;
    if store.path(id, "prepared").exists() {
        let (_, prepared) = store.load(id)?;
        return String::from_utf8(encode(&prepared)?)
            .map_err(|_| CoordError::new("PUBLICATION_ENCODING", "UTF-8 required"));
    }
    for (name, subject) in &request.manifest.members {
        let source = root.join(name);
        git::execute(
            git::command(&store.objects)
                .args(["fetch", "--no-tags", "--no-write-fetch-head"])
                .arg(&source)
                .arg(&subject.commit),
            None,
        )?;
    }
    if !request.bootstrap()? {
        let mut command = git::command(&store.objects);
        if destination == super::JERYU_DESTINATION {
            super::transport::authenticate_git(
                &mut command,
                destination,
                &super::transport::jeryu_token()?,
            )?;
        }
        git::execute(
            command.args([
                "fetch",
                "--no-tags",
                "--no-write-fetch-head",
                destination,
                base,
            ]),
            None,
        )?;
    }
    git::bytes(
        &store.objects,
        &["fsck", "--full", "--strict", "--no-dangling"],
    )?;
    let aggregate_commit = build_commit(&store.objects, &request)?;
    require(
        capture(root)? == request.manifest,
        "PUBLICATION_SOURCE_CHANGED",
    )?;
    let prepared = Prepared {
        schema_version: "bullet.publication-prepared.v1".into(),
        aggregate_commit,
        request_sha256: digest(&encode(&request)?),
        review_ref: format!("refs/heads/publication/{id}"),
    };
    // Retain objects even when an interrupted publication has no remote refs yet.
    let retained_ref = format!("refs/publication/{id}");
    let retained = git::text(
        &store.objects,
        &["for-each-ref", "--format=%(objectname)", &retained_ref],
    )?;
    require(
        retained.is_empty() || retained == prepared.aggregate_commit,
        "PUBLICATION_REQUEST_CONFLICT",
    )?;
    if retained.is_empty() {
        git::bytes(
            &store.objects,
            &[
                "update-ref",
                &retained_ref,
                &prepared.aggregate_commit,
                &"0".repeat(40),
            ],
        )?;
    }
    for (name, subject) in &request.manifest.members {
        git::bytes(
            &store.objects,
            &[
                "update-ref",
                &format!("refs/publication-sources/{id}/{name}"),
                &subject.commit,
            ],
        )?;
    }
    persist(&store.path(id, "prepared"), &encode(&prepared)?)?;
    String::from_utf8(encode(&prepared)?)
        .map_err(|_| CoordError::new("PUBLICATION_ENCODING", "UTF-8 required"))
}
