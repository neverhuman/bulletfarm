//! Exact local family subject anchoring for a present semantic registry.

use std::{
    fs::{File, Metadata, OpenOptions},
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};

use bullet_wire::v1alpha1::ReleaseRegistryManifestV1;

use super::{Reject, reject};

const MAX_FAMILY_LOCK_BYTES: u64 = 1024 * 1024;

pub(super) fn validate(hub: &Path, manifest: &ReleaseRegistryManifestV1) -> Result<(), Reject> {
    let path = hub.join("family.lock");
    let bytes = read_stable(&path, || {})?;
    let lock = crate::family_lock::parse(&bytes).map_err(|error| {
        reject(format!(
            "family.lock is not an installable schema-3 lock: {error}"
        ))
    })?;
    let digest = format!("blake3:{}", blake3::hash(&bytes).to_hex());
    if digest != manifest.family_lock_digest {
        return Err(reject(
            "registry family_lock_digest differs from the exact admitted family.lock bytes",
        ));
    }
    if lock.external.release_signing.policy_digest != manifest.signer_policy_digest {
        return Err(reject(
            "registry signer_policy_digest differs from the policy locked by family.lock",
        ));
    }
    Ok(())
}

fn open_input(path: &Path) -> Result<File, Reject> {
    OpenOptions::new()
        .read(true)
        .custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_NONBLOCK | nix::libc::O_CLOEXEC)
        .open(path)
        .map_err(|error| reject(format!("family.lock admission failed: {error}")))
}

fn read_stable(path: &Path, after_first_read: impl FnOnce()) -> Result<Vec<u8>, Reject> {
    read_stable_with_hooks(path, after_first_read, || {})
}

fn read_stable_with_hooks(
    path: &Path,
    after_first_read: impl FnOnce(),
    before_reopen: impl FnOnce(),
) -> Result<Vec<u8>, Reject> {
    let mut input = open_input(path)?;
    let before = input
        .metadata()
        .map_err(|error| reject(format!("family.lock metadata is unavailable: {error}")))?;
    if !before.file_type().is_file() || before.nlink() != 1 {
        return Err(reject(
            "family.lock must be a regular, non-symlink, single-link file",
        ));
    }
    if before.len() > MAX_FAMILY_LOCK_BYTES {
        return Err(reject("family.lock exceeds the 1 MiB admission limit"));
    }
    let identity = Identity::from(&before);

    let mut bytes = Vec::with_capacity(before.len() as usize);
    input
        .by_ref()
        .take(MAX_FAMILY_LOCK_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| reject(format!("family.lock read failed: {error}")))?;
    if bytes.len() as u64 > MAX_FAMILY_LOCK_BYTES {
        return Err(reject("family.lock exceeds the 1 MiB admission limit"));
    }
    after_first_read();
    let after = input
        .metadata()
        .map_err(|error| reject(format!("family.lock metadata is unavailable: {error}")))?;
    if bytes.len() as u64 != before.len() || Identity::from(&after) != identity {
        return Err(reject("family.lock changed while it was being admitted"));
    }

    before_reopen();
    let mut reopened = open_input(path)?;
    let reopened_before = reopened
        .metadata()
        .map_err(|error| reject(format!("family.lock metadata is unavailable: {error}")))?;
    if Identity::from(&reopened_before) != identity {
        return Err(reject("family.lock was substituted during admission"));
    }
    let mut confirmed = Vec::with_capacity(bytes.len());
    reopened
        .by_ref()
        .take(MAX_FAMILY_LOCK_BYTES + 1)
        .read_to_end(&mut confirmed)
        .map_err(|error| reject(format!("family.lock confirmation read failed: {error}")))?;
    let reopened_after = reopened
        .metadata()
        .map_err(|error| reject(format!("family.lock metadata is unavailable: {error}")))?;
    if confirmed != bytes || Identity::from(&reopened_after) != identity {
        return Err(reject("family.lock changed during confirmation read"));
    }
    Ok(bytes)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Identity {
    device: u64,
    inode: u64,
    length: u64,
    links: u64,
    modified_seconds: i64,
    modified_nanoseconds: i64,
    changed_seconds: i64,
    changed_nanoseconds: i64,
}

impl Identity {
    fn from(metadata: &Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            length: metadata.len(),
            links: metadata.nlink(),
            modified_seconds: metadata.mtime(),
            modified_nanoseconds: metadata.mtime_nsec(),
            changed_seconds: metadata.ctime(),
            changed_nanoseconds: metadata.ctime_nsec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{read_stable, read_stable_with_hooks};

    #[test]
    fn family_lock_admission_refuses_hardlinks_and_same_length_rewrites() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("family.lock");
        fs::write(&path, b"aa").expect("write subject");
        fs::hard_link(&path, root.path().join("alias")).expect("hardlink");
        let hardlink = read_stable(&path, || {}).expect_err("hardlink refused");
        assert!(hardlink.detail.contains("single-link"));

        fs::remove_file(root.path().join("alias")).expect("remove alias");
        let rewrite = read_stable(&path, || fs::write(&path, b"bb").expect("rewrite"))
            .expect_err("same-length rewrite refused");
        assert!(
            rewrite.detail.contains("changed"),
            "unexpected refusal: {}",
            rewrite.detail
        );
    }

    #[test]
    fn family_lock_admission_refuses_rename_substitution() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("family.lock");
        let original = root.path().join("family.original");
        fs::write(&path, b"aa").expect("write subject");
        let substitution = read_stable_with_hooks(
            &path,
            || {},
            || {
                fs::rename(&path, &original).expect("retain original");
                fs::write(&path, b"aa").expect("replace path");
            },
        )
        .expect_err("rename substitution refused");
        assert!(
            substitution.detail.contains("substituted"),
            "unexpected refusal: {}",
            substitution.detail
        );
    }
}
