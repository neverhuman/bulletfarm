use super::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Object {
    pub(super) oid: String,
    pub(super) kind: String,
    pub(super) size: u64,
}

pub(super) fn inventory(repo: &Path, heads: &[String]) -> Result<Vec<Object>> {
    let mut command = git::command(repo);
    command.args([
        "rev-list",
        "--objects",
        "--no-object-names",
        "--missing=error",
    ]);
    command.args(heads).arg("--");
    let raw = String::from_utf8(git::execute(&mut command, None)?).map_err(|_| {
        CoordError::new(
            "PUBLICATION_SCAN_INVENTORY_INVALID",
            "non-ASCII object list",
        )
    })?;
    let ids = raw.lines().map(str::to_owned).collect::<BTreeSet<_>>();
    require(
        !ids.is_empty() && ids.len() <= MAX_OBJECTS,
        "PUBLICATION_SCAN_OBJECT_LIMIT",
    )?;
    for id in &ids {
        oid(id)?;
    }
    let input = ids.iter().map(|id| format!("{id}\n")).collect::<String>();
    let raw = git::execute(
        git::command(repo)
            .arg("cat-file")
            .arg("--batch-check=%(objectname) %(objecttype) %(objectsize)"),
        Some(input.as_bytes()),
    )?;
    let text = String::from_utf8(raw).map_err(|_| {
        CoordError::new(
            "PUBLICATION_SCAN_INVENTORY_INVALID",
            "invalid object metadata",
        )
    })?;
    let mut objects = Vec::new();
    for (line, id) in text.lines().zip(&ids) {
        let fields = line.split(' ').collect::<Vec<_>>();
        require(
            fields.len() == 3
                && fields[0] == id
                && ["blob", "commit", "tree", "tag"].contains(&fields[1]),
            "PUBLICATION_SCAN_INVENTORY_INVALID",
        )?;
        let size = fields[2]
            .bytes()
            .try_fold(0_u64, |value, byte| {
                if !byte.is_ascii_digit() {
                    return None;
                }
                value.checked_mul(10)?.checked_add(u64::from(byte - b'0'))
            })
            .filter(|_| !fields[2].is_empty())
            .ok_or_else(|| CoordError::new("PUBLICATION_SCAN_INVENTORY_INVALID", "invalid size"))?;
        objects.push(Object {
            oid: id.clone(),
            kind: fields[1].into(),
            size,
        });
    }
    require(
        text.lines().count() == ids.len() && objects.len() == ids.len(),
        "PUBLICATION_SCAN_INVENTORY_INVALID",
    )?;
    validate_limits(&objects)?;
    Ok(objects)
}

pub(super) fn validate_limits(objects: &[Object]) -> Result<u64> {
    require(
        !objects.is_empty() && objects.len() <= MAX_OBJECTS,
        "PUBLICATION_SCAN_OBJECT_LIMIT",
    )?;
    let mut total = 0_u64;
    for object in objects {
        require(
            object.size <= MAX_OBJECT_BYTES,
            "PUBLICATION_SCAN_OBJECT_LIMIT",
        )?;
        total = total
            .checked_add(object.size)
            .ok_or_else(|| CoordError::new("PUBLICATION_SCAN_TOTAL_LIMIT", "size overflow"))?;
        require(total <= MAX_TOTAL_BYTES, "PUBLICATION_SCAN_TOTAL_LIMIT")?;
    }
    Ok(total)
}

pub(super) fn batch<'a>(bytes: &'a [u8], objects: &[Object]) -> Result<Vec<&'a [u8]>> {
    let mut remaining = bytes;
    let mut contents = Vec::new();
    for object in objects {
        let header = format!("{} {} {}\n", object.oid, object.kind, object.size);
        require(
            remaining.starts_with(header.as_bytes()),
            "PUBLICATION_SCAN_OBJECT_DRIFT",
        )?;
        remaining = &remaining[header.len()..];
        let size = usize::try_from(object.size)
            .map_err(|_| CoordError::new("PUBLICATION_SCAN_OBJECT_LIMIT", "size overflow"))?;
        require(
            remaining.len() > size && remaining[size] == b'\n',
            "PUBLICATION_SCAN_OBJECT_TRUNCATED",
        )?;
        contents.push(&remaining[..size]);
        remaining = &remaining[size + 1..];
    }
    require(
        remaining.is_empty(),
        "PUBLICATION_SCAN_OBJECT_TRAILING_BYTES",
    )?;
    Ok(contents)
}

pub(super) fn hash_blob(repo: &Path, content: &[u8]) -> Result<String> {
    let value = git::execute(
        git::command(repo).args(["hash-object", "-w", "--stdin"]),
        Some(content),
    )?;
    let value = String::from_utf8(value)
        .map_err(|_| CoordError::new("PUBLICATION_SCAN_OBJECT_INVALID", "invalid blob identity"))?;
    let value = value.trim_end().to_owned();
    oid(&value)?;
    Ok(value)
}

pub(super) fn synthetic(
    repo: &Path,
    objects: &[Object],
    deadline: Instant,
) -> Result<(String, String)> {
    validate_limits(objects)?;
    let mut hasher = Sha256::new();
    let mut entries = Vec::new();
    let mut start = 0;
    while start < objects.len() {
        require(Instant::now() < deadline, "PUBLICATION_SCAN_DEADLINE")?;
        let mut end = start + 1;
        let mut size = objects[start].size;
        while end < objects.len() && size + objects[end].size <= BATCH_BYTES && end - start < 1024 {
            size += objects[end].size;
            end += 1;
        }
        let selected = &objects[start..end];
        let input = selected
            .iter()
            .map(|object| format!("{}\n", object.oid))
            .collect::<String>();
        let output = git::execute(
            git::command(repo).args(["cat-file", "--batch"]),
            Some(input.as_bytes()),
        )?;
        for (object, raw) in selected.iter().zip(batch(&output, selected)?) {
            require(Instant::now() < deadline, "PUBLICATION_SCAN_DEADLINE")?;
            hasher.update(format!("{} {} {}\n", object.oid, object.kind, object.size).as_bytes());
            hasher.update(raw);
            hasher.update(b"\n");
            // Reuse blobs; metadata becomes ordinary blobs so commit messages and tree names are scanned too.
            let blob = if object.kind == "blob" {
                object.oid.clone()
            } else {
                hash_blob(repo, raw)?
            };
            entries.extend_from_slice(format!("100644 blob {blob}\t{}\0", object.oid).as_bytes());
        }
        start = end;
    }
    let tree = git::execute(git::command(repo).args(["mktree", "-z"]), Some(&entries))?;
    let tree = String::from_utf8(tree)
        .map_err(|_| CoordError::new("PUBLICATION_SCAN_TREE_INVALID", "invalid tree"))?;
    let mut command = git::command(repo);
    for role in ["AUTHOR", "COMMITTER"] {
        command.env(format!("GIT_{role}_NAME"), "Bullet secret scan");
        command.env(format!("GIT_{role}_EMAIL"), "scan@bullet.invalid");
        command.env(format!("GIT_{role}_DATE"), "2000-01-01T00:00:00Z");
    }
    let commit = git::execute(
        command.args(["-c", "commit.gpgSign=false", "commit-tree", tree.trim_end()]),
        Some(b"Local raw-object secret scan; never publish\n"),
    )?;
    let commit = String::from_utf8(commit)
        .map_err(|_| CoordError::new("PUBLICATION_SCAN_TREE_INVALID", "invalid commit"))?;
    let commit = commit.trim_end().to_owned();
    oid(&commit)?;
    Ok((commit, format!("{:x}", hasher.finalize())))
}
