//! Prior-or-complete-next setup transaction with a final family commit marker.

#[cfg(unix)]
mod root;

#[cfg(not(unix))]
mod root {
    use std::path::{Path, PathBuf};

    use crate::coord::CoordError;

    #[derive(Clone, Debug)]
    pub(in crate::setup) struct AdmittedRoot {
        path: PathBuf,
    }

    #[derive(Debug)]
    pub(in crate::setup) struct Staging;

    impl AdmittedRoot {
        pub(in crate::setup) fn open(path: &Path) -> Result<Self, CoordError> {
            let _ = path;
            Err(unsupported())
        }

        pub(in crate::setup) fn path(&self) -> &Path {
            &self.path
        }

        pub(in crate::setup) fn ensure_path_identity(&self) -> Result<(), CoordError> {
            Err(unsupported())
        }

        pub(in crate::setup) fn create_staging(&self, _class: &str) -> Result<Staging, CoordError> {
            Err(unsupported())
        }

        pub(super) fn read_optional_file(
            &self,
            _name: &str,
        ) -> Result<Option<Vec<u8>>, CoordError> {
            Err(unsupported())
        }

        pub(super) fn read_hub_manifest(&self) -> Result<Vec<u8>, CoordError> {
            Err(unsupported())
        }
    }

    impl Staging {
        pub(in crate::setup) fn path(&self) -> &Path {
            Path::new("")
        }

        pub(in crate::setup) fn ensure_path_identity(&self) -> Result<(), CoordError> {
            Err(unsupported())
        }

        pub(in crate::setup) fn create_private_dir(&self, _name: &str) -> Result<(), CoordError> {
            Err(unsupported())
        }

        pub(super) fn publish_member(&self, _member: &str) -> Result<(), CoordError> {
            Err(unsupported())
        }

        pub(super) fn write_manifest(&self, _bytes: &[u8]) -> Result<(), CoordError> {
            Err(unsupported())
        }

        pub(super) fn link_manifest(&self) -> Result<(), CoordError> {
            Err(unsupported())
        }

        pub(super) fn sync_root(&self) -> Result<(), CoordError> {
            Err(unsupported())
        }

        pub(super) fn remove_manifest_temporary(&self) -> Result<(), CoordError> {
            Err(unsupported())
        }

        pub(in crate::setup) fn finish(self) -> Result<(), CoordError> {
            Err(unsupported())
        }
    }

    fn unsupported() -> CoordError {
        CoordError::new(
            "UNSUPPORTED_PLATFORM_CONTAINMENT",
            "descriptor-relative setup publication requires the admitted Unix containment backend",
        )
    }
}

use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use super::{CloneTransport, checkout_locked_commit};
use crate::{
    checkout::{ensure_regular_file, required_members, verify_family, verify_hub, verify_member},
    coord::CoordError,
    family_lock::{FamilyLock, LockedMember},
};

pub(in crate::setup) use root::{AdmittedRoot, Staging};

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Boundary {
    CloneComplete,
    CheckoutComplete,
    CandidateValidated,
    BeforeMemberPublish,
    AfterMemberPublish,
    ManifestFileSynced,
    ManifestLinked,
    ManifestDirectorySynced,
}

pub(super) trait FaultInjector {
    fn reach(&self, boundary: Boundary, member: Option<&str>) -> Result<(), CoordError>;
}

pub(super) struct NoFault;

impl FaultInjector for NoFault {
    fn reach(&self, _boundary: Boundary, _member: Option<&str>) -> Result<(), CoordError> {
        Ok(())
    }
}

#[cfg(test)]
pub(super) struct ExactOnlyValidator;

#[cfg(test)]
impl CandidateValidator for ExactOnlyValidator {
    fn validate(&self, _candidate: &CandidateFamily<'_>) -> Result<(), CoordError> {
        Ok(())
    }
}

struct CandidateMember<'a> {
    lock: &'a LockedMember,
    path: PathBuf,
    parent: PathBuf,
    staged: bool,
}

pub(super) struct CandidateFamily<'a> {
    hub: &'a Path,
    family_root: &'a Path,
    allowed_signers: &'a Path,
    lock: &'a FamilyLock,
    members: Vec<CandidateMember<'a>>,
}

impl CandidateFamily<'_> {
    pub(super) fn path(&self, name: &str) -> Result<&Path, CoordError> {
        if name == "bullet-farm" {
            return Ok(self.hub);
        }
        self.members
            .iter()
            .find(|entry| entry.lock.name == name)
            .map(|entry| entry.path.as_path())
            .ok_or_else(|| {
                CoordError::new(
                    "FAMILY_MEMBER_MISSING",
                    format!("candidate family has no {name}"),
                )
            })
    }

    fn verify_exact(&self, verifier: &dyn FamilyVerifier) -> Result<(), CoordError> {
        verifier.verify_hub(self.lock, self.hub, self.family_root, self.allowed_signers)?;
        for entry in &self.members {
            verifier.verify_member(entry.lock, &entry.path, &entry.parent, self.allowed_signers)?;
        }
        Ok(())
    }
}

pub(super) trait CandidateValidator {
    fn validate(&self, candidate: &CandidateFamily<'_>) -> Result<(), CoordError>;
}

trait FamilyVerifier {
    fn verify_hub(
        &self,
        lock: &FamilyLock,
        hub: &Path,
        family_root: &Path,
        allowed_signers: &Path,
    ) -> Result<(), CoordError>;

    fn verify_member(
        &self,
        member: &LockedMember,
        path: &Path,
        parent: &Path,
        allowed_signers: &Path,
    ) -> Result<(), CoordError>;

    fn verify_family(
        &self,
        family_root: &Path,
        hub: &Path,
        lock: &FamilyLock,
    ) -> Result<(), CoordError>;
}

struct ExactVerifier;

impl FamilyVerifier for ExactVerifier {
    fn verify_hub(
        &self,
        lock: &FamilyLock,
        hub: &Path,
        family_root: &Path,
        allowed_signers: &Path,
    ) -> Result<(), CoordError> {
        verify_hub(lock, hub, family_root, allowed_signers)
    }

    fn verify_member(
        &self,
        member: &LockedMember,
        path: &Path,
        parent: &Path,
        allowed_signers: &Path,
    ) -> Result<(), CoordError> {
        verify_member(member, path, parent, allowed_signers)
    }

    fn verify_family(
        &self,
        family_root: &Path,
        hub: &Path,
        lock: &FamilyLock,
    ) -> Result<(), CoordError> {
        verify_family(family_root, hub, lock)
    }
}

struct Controls<'a> {
    faults: &'a dyn FaultInjector,
    verifier: &'a dyn FamilyVerifier,
}

pub(super) fn install_transaction(
    root: &AdmittedRoot,
    hub_root: &Path,
    lock: &FamilyLock,
    offline: bool,
    transport: &dyn CloneTransport,
    validator: &dyn CandidateValidator,
    faults: &dyn FaultInjector,
) -> Result<(), CoordError> {
    install_with_verifier(
        root,
        hub_root,
        lock,
        offline,
        transport,
        validator,
        Controls {
            faults,
            verifier: &ExactVerifier,
        },
    )
}

fn install_with_verifier(
    root: &AdmittedRoot,
    hub_root: &Path,
    lock: &FamilyLock,
    offline: bool,
    transport: &dyn CloneTransport,
    validator: &dyn CandidateValidator,
    controls: Controls<'_>,
) -> Result<(), CoordError> {
    let Controls { faults, verifier } = controls;
    let family_root = root.path();
    root.ensure_path_identity()?;
    let required = required_members(hub_root)?;
    lock.validate_required_members(&required)?;
    let allowed_signers = hub_root.join("release/allowed_signers");
    ensure_regular_file(&allowed_signers, "allowed signers")?;
    verifier.verify_hub(lock, hub_root, family_root, &allowed_signers)?;
    root.ensure_path_identity()?;
    let (committed, manifest_bytes) = inspect_manifest(root)?;
    let staging = (!committed)
        .then(|| root.create_staging("checkout"))
        .transpose()?;

    let mut members = Vec::with_capacity(lock.member.len());
    for member in &lock.member {
        root.ensure_path_identity()?;
        let target = family_root.join(&member.name);
        match fs::symlink_metadata(&target) {
            Ok(_) => {
                verifier.verify_member(member, &target, family_root, &allowed_signers)?;
                root.ensure_path_identity()?;
                members.push(CandidateMember {
                    lock: member,
                    path: target,
                    parent: family_root.to_path_buf(),
                    staged: false,
                });
            }
            Err(error) if error.kind() == ErrorKind::NotFound && committed => {
                return Err(CoordError::new(
                    "FAMILY_COMMIT_INCOMPLETE",
                    format!(
                        "the durable family manifest covers missing {}; preserve the root and repair from the signed release",
                        member.name
                    ),
                ));
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                let stage = staging.as_ref().expect("uncommitted setup has staging");
                let path = stage.path().join(&member.name);
                transport.clone_member(member, &path, offline)?;
                reach(faults, root, Boundary::CloneComplete, Some(&member.name))?;
                checkout_locked_commit(member, &path)?;
                reach(faults, root, Boundary::CheckoutComplete, Some(&member.name))?;
                verifier.verify_member(member, &path, stage.path(), &allowed_signers)?;
                stage.ensure_path_identity()?;
                members.push(CandidateMember {
                    lock: member,
                    path,
                    parent: stage.path().to_path_buf(),
                    staged: true,
                });
            }
            Err(error) => return Err(CoordError::io(error)),
        }
    }

    let candidate = CandidateFamily {
        hub: hub_root,
        family_root,
        allowed_signers: &allowed_signers,
        lock,
        members,
    };
    validator.validate(&candidate)?;
    root.ensure_path_identity()?;
    candidate.verify_exact(verifier)?;
    reach(faults, root, Boundary::CandidateValidated, None)?;

    if committed {
        let result = verifier.verify_family(family_root, hub_root, lock);
        root.ensure_path_identity()?;
        return result;
    }
    let staging = staging.expect("uncommitted setup has staging");
    for entry in candidate.members.iter().filter(|entry| entry.staged) {
        reach(
            faults,
            root,
            Boundary::BeforeMemberPublish,
            Some(&entry.lock.name),
        )?;
        staging.publish_member(&entry.lock.name)?;
        reach(
            faults,
            root,
            Boundary::AfterMemberPublish,
            Some(&entry.lock.name),
        )?;
    }
    verifier.verify_family(family_root, hub_root, lock)?;
    root.ensure_path_identity()?;
    publish_manifest(root, &manifest_bytes, &staging, faults)?;
    verifier.verify_family(family_root, hub_root, lock)?;
    root.ensure_path_identity()?;
    staging.finish()
}

fn inspect_manifest(root: &AdmittedRoot) -> Result<(bool, Vec<u8>), CoordError> {
    root.ensure_path_identity()?;
    let source = root.read_hub_manifest()?;
    match root.read_optional_file("repos.manifest.toml")? {
        Some(destination) => {
            if destination != source {
                return Err(manifest_conflict(
                    "family-root repos.manifest.toml differs from the signed hub manifest",
                ));
            }
            Ok((true, source))
        }
        None => Ok((false, source)),
    }
}

fn publish_manifest(
    root: &AdmittedRoot,
    expected: &[u8],
    staging: &root::Staging,
    faults: &dyn FaultInjector,
) -> Result<(), CoordError> {
    let (exists, current) = inspect_manifest(root)?;
    if exists || current != expected {
        return Err(manifest_conflict(
            "family manifest appeared or changed before final publication",
        ));
    }
    staging.write_manifest(expected)?;
    reach(faults, root, Boundary::ManifestFileSynced, None)?;
    staging.link_manifest()?;
    reach(faults, root, Boundary::ManifestLinked, None)?;
    staging.sync_root()?;
    reach(faults, root, Boundary::ManifestDirectorySynced, None)?;
    staging.remove_manifest_temporary()
}

fn manifest_conflict(reason: impl Into<String>) -> CoordError {
    CoordError::new("FAMILY_MANIFEST_CONFLICT", reason)
}

fn reach(
    faults: &dyn FaultInjector,
    root: &AdmittedRoot,
    boundary: Boundary,
    member: Option<&str>,
) -> Result<(), CoordError> {
    faults.reach(boundary, member)?;
    root.ensure_path_identity()
}

#[cfg(test)]
pub(super) fn publish_staged_for_test(staging: &Staging, member: &str) -> Result<(), CoordError> {
    staging.publish_member(member)
}

#[cfg(test)]
pub(super) fn admitted_root_for_test(path: &Path) -> Result<AdmittedRoot, CoordError> {
    AdmittedRoot::open(path)
}
