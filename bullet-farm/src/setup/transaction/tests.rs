use std::{cell::Cell, fs, path::Path};

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};

use super::*;
use crate::setup::tests::{
    LocalTransport, MEMBERS, TAG, assert_no_staging, create_signing_key, create_source_family,
    fixture_root, installed_heads, test_git,
};

struct CrashAt {
    boundary: Boundary,
    member: Option<&'static str>,
    fired: Cell<bool>,
}

impl CrashAt {
    fn new(boundary: Boundary, member: Option<&'static str>) -> Self {
        Self {
            boundary,
            member,
            fired: Cell::new(false),
        }
    }
}

impl FaultInjector for CrashAt {
    fn reach(&self, boundary: Boundary, member: Option<&str>) -> Result<(), CoordError> {
        if !self.fired.get() && boundary == self.boundary && member == self.member {
            self.fired.set(true);
            return Err(CoordError::new(
                "INJECTED_SETUP_CRASH",
                format!("{boundary:?} at {}", member.unwrap_or("family")),
            ));
        }
        Ok(())
    }
}

struct ReplaceRootAt {
    boundary: Boundary,
    root: PathBuf,
    moved: PathBuf,
    fired: Cell<bool>,
}

impl FaultInjector for ReplaceRootAt {
    fn reach(&self, boundary: Boundary, _member: Option<&str>) -> Result<(), CoordError> {
        if !self.fired.get() && boundary == self.boundary {
            fs::rename(&self.root, &self.moved).map_err(CoordError::io)?;
            fs::create_dir(&self.root).map_err(CoordError::io)?;
            fs::write(
                self.root.join("replacement-sentinel"),
                "preserve replacement\n",
            )
            .map_err(CoordError::io)?;
            self.fired.set(true);
        }
        Ok(())
    }
}

struct SideEffectValidator {
    corrupt_tracked: bool,
    calls: Cell<usize>,
}

struct FastVerifier;

impl FamilyVerifier for FastVerifier {
    fn verify_hub(
        &self,
        _lock: &FamilyLock,
        hub: &Path,
        family_root: &Path,
        _allowed_signers: &Path,
    ) -> Result<(), CoordError> {
        crate::checkout::ensure_ordinary_checkout(hub, family_root, "bullet-farm")?;
        crate::checkout::verify_exact_worktree(hub)
    }

    fn verify_member(
        &self,
        member: &LockedMember,
        path: &Path,
        parent: &Path,
        _allowed_signers: &Path,
    ) -> Result<(), CoordError> {
        crate::checkout::ensure_ordinary_checkout(path, parent, &member.name)?;
        let (commit, tree) = crate::family_lock::checkout_subject(path)?;
        if commit != member.commit_oid || tree != member.tree_oid {
            return Err(CoordError::new(
                "LOCKED_SUBJECT_MISMATCH",
                format!("{} differs from the fixture lock", member.name),
            ));
        }
        crate::checkout::verify_exact_worktree(path)
    }

    fn verify_family(
        &self,
        family_root: &Path,
        hub: &Path,
        lock: &FamilyLock,
    ) -> Result<(), CoordError> {
        self.verify_hub(lock, hub, family_root, &hub.join("release/allowed_signers"))?;
        for member in &lock.member {
            self.verify_member(
                member,
                &family_root.join(&member.name),
                family_root,
                &hub.join("release/allowed_signers"),
            )?;
        }
        Ok(())
    }
}

impl SideEffectValidator {
    fn clean() -> Self {
        Self {
            corrupt_tracked: false,
            calls: Cell::new(0),
        }
    }
}

impl CandidateValidator for SideEffectValidator {
    fn validate(&self, candidate: &CandidateFamily<'_>) -> Result<(), CoordError> {
        self.calls.set(self.calls.get() + 1);
        let cargo_output = candidate.path("bullet-kernel")?.join("target/setup/ready");
        fs::create_dir_all(cargo_output.parent().unwrap()).map_err(CoordError::io)?;
        fs::write(cargo_output, "ignored cargo output\n").map_err(CoordError::io)?;
        let npm_output = candidate
            .path("bullet-portal")?
            .join("node_modules/.setup-ready");
        fs::create_dir_all(npm_output.parent().unwrap()).map_err(CoordError::io)?;
        fs::write(npm_output, "ignored npm output\n").map_err(CoordError::io)?;
        if self.corrupt_tracked {
            fs::write(
                candidate.path("bullet-kernel")?.join("Cargo.lock"),
                "tracked tool mutation\n",
            )
            .map_err(CoordError::io)?;
        }
        Ok(())
    }
}

#[test]
fn every_transaction_boundary_recovers_to_one_complete_exact_family() {
    let fixture = fixture_root("transaction-faults");
    let sources = fixture.join("sources");
    let home = fixture.join("home");
    fs::create_dir(&sources).unwrap();
    fs::create_dir(&home).unwrap();
    let key = create_signing_key(&fixture);
    create_source_family(&sources, &home, &key);
    let points = [
        (Boundary::CloneComplete, Some("bullet-kernel")),
        (Boundary::CheckoutComplete, Some("bullet-git")),
        (Boundary::CandidateValidated, None),
        (Boundary::BeforeMemberPublish, Some("bullet-portal")),
        (Boundary::AfterMemberPublish, Some("bullet-kernel")),
        (Boundary::ManifestFileSynced, None),
        (Boundary::ManifestLinked, None),
        (Boundary::ManifestDirectorySynced, None),
    ];

    for (index, (boundary, member)) in points.into_iter().enumerate() {
        let root = fixture.join(format!("install-{index}"));
        fs::create_dir(&root).unwrap();
        let hub = install_hub(&sources, &root, &home);
        let admitted = AdmittedRoot::open(&root).unwrap();
        let lock = crate::family_lock::load(&hub.join("family.lock")).unwrap();
        let transport = LocalTransport {
            sources: sources.clone(),
            clone_count: Cell::new(0),
        };
        let validator = SideEffectValidator::clean();
        let crash = CrashAt::new(boundary, member);
        let error = install_with_verifier(
            &admitted,
            &hub,
            &lock,
            true,
            &transport,
            &validator,
            Controls {
                faults: &crash,
                verifier: &FastVerifier,
            },
        )
        .expect_err("selected transaction boundary must stop setup");
        assert_eq!(error.code(), "INJECTED_SETUP_CRASH");
        assert!(
            crash.fired.get(),
            "fault point was not reached: {boundary:?}"
        );
        assert_authority_invariant(&root, &hub, &lock, &FastVerifier);
        assert_no_staging(&root);
        let existing = existing_identities(&root);

        install_with_verifier(
            &admitted,
            &hub,
            &lock,
            true,
            &transport,
            &validator,
            Controls {
                faults: &NoFault,
                verifier: &FastVerifier,
            },
        )
        .expect("rerun completes exact family");
        assert_existing_not_replaced(&root, &existing);
        FastVerifier.verify_family(&root, &hub, &lock).unwrap();
        assert_eq!(
            fs::read(root.join("repos.manifest.toml")).unwrap(),
            fs::read(hub.join("repos.manifest.toml")).unwrap()
        );
        assert!(root.join("bullet-kernel/target/setup/ready").is_file());
        assert!(
            root.join("bullet-portal/node_modules/.setup-ready")
                .is_file()
        );
        let _exact_clean_heads = installed_heads(&root, &home);
        assert_no_staging(&root);
    }
    fs::remove_dir_all(fixture).unwrap();
}

#[cfg(unix)]
#[test]
fn root_replacement_cannot_redirect_member_or_manifest_publication() {
    let fixture = fixture_root("transaction-root-replacement");
    let sources = fixture.join("sources");
    let home = fixture.join("home");
    fs::create_dir(&sources).unwrap();
    fs::create_dir(&home).unwrap();
    let key = create_signing_key(&fixture);
    create_source_family(&sources, &home, &key);

    for (index, boundary) in [Boundary::BeforeMemberPublish, Boundary::ManifestFileSynced]
        .into_iter()
        .enumerate()
    {
        let root = fixture.join(format!("replace-{index}"));
        let moved = fixture.join(format!("original-{index}"));
        fs::create_dir(&root).unwrap();
        let hub = install_hub(&sources, &root, &home);
        let lock = crate::family_lock::load(&hub.join("family.lock")).unwrap();
        let admitted = AdmittedRoot::open(&root).unwrap();
        let transport = LocalTransport {
            sources: sources.clone(),
            clone_count: Cell::new(0),
        };
        let replacement = ReplaceRootAt {
            boundary,
            root: root.clone(),
            moved: moved.clone(),
            fired: Cell::new(false),
        };
        let error = install_with_verifier(
            &admitted,
            &hub,
            &lock,
            true,
            &transport,
            &SideEffectValidator::clean(),
            Controls {
                faults: &replacement,
                verifier: &FastVerifier,
            },
        )
        .expect_err("a replaced admitted root must fail closed");
        assert_eq!(error.code(), "SETUP_ROOT_REPLACED");
        assert!(replacement.fired.get());
        assert_eq!(
            fs::read_to_string(root.join("replacement-sentinel")).unwrap(),
            "preserve replacement\n"
        );
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        assert!(!moved.join("repos.manifest.toml").exists());
        assert_no_staging(&moved);
        assert_authority_invariant(&moved, &moved.join("bullet-farm"), &lock, &FastVerifier);
    }
    fs::remove_dir_all(fixture).unwrap();
}

#[cfg(unix)]
#[test]
fn staging_is_private_and_cleanup_never_follows_symlinks() {
    let fixture = fixture_root("private-staging");
    let outside = fixture_root("private-staging-outside");
    fs::write(outside.join("sentinel"), "preserve outside\n").unwrap();
    let root = AdmittedRoot::open(&fixture).unwrap();
    let staging = root.create_staging("permission-test").unwrap();
    let mode = fs::metadata(staging.path()).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o700);
    fs::create_dir(staging.path().join("nested")).unwrap();
    fs::write(staging.path().join("nested/data"), "cleanup me\n").unwrap();
    symlink(&outside, staging.path().join("outside-link")).unwrap();
    staging.finish().unwrap();
    assert_no_staging(&fixture);
    assert_eq!(
        fs::read_to_string(outside.join("sentinel")).unwrap(),
        "preserve outside\n"
    );
    fs::remove_dir_all(fixture).unwrap();
    fs::remove_dir_all(outside).unwrap();
}

#[test]
fn tracked_tool_mutation_and_hostile_partial_state_never_publish_authority() {
    let fixture = fixture_root("transaction-negative");
    let sources = fixture.join("sources");
    let home = fixture.join("home");
    fs::create_dir(&sources).unwrap();
    fs::create_dir(&home).unwrap();
    let key = create_signing_key(&fixture);
    create_source_family(&sources, &home, &key);

    let corrupt_root = fixture.join("corrupt-install");
    fs::create_dir(&corrupt_root).unwrap();
    let corrupt_hub = install_hub(&sources, &corrupt_root, &home);
    let corrupt_admitted = AdmittedRoot::open(&corrupt_root).unwrap();
    let lock = crate::family_lock::load(&corrupt_hub.join("family.lock")).unwrap();
    let transport = LocalTransport {
        sources: sources.clone(),
        clone_count: Cell::new(0),
    };
    let corrupt = SideEffectValidator {
        corrupt_tracked: true,
        calls: Cell::new(0),
    };
    let error = install_transaction(
        &corrupt_admitted,
        &corrupt_hub,
        &lock,
        true,
        &transport,
        &corrupt,
        &NoFault,
    )
    .unwrap_err();
    assert_eq!(error.code(), "DIRTY_CHECKOUT");
    assert!(!corrupt_root.join("repos.manifest.toml").exists());
    for member in &lock.member {
        assert!(!corrupt_root.join(&member.name).exists());
    }
    assert_no_staging(&corrupt_root);

    let hostile_root = fixture.join("hostile-install");
    fs::create_dir(&hostile_root).unwrap();
    let hostile_hub = install_hub(&sources, &hostile_root, &home);
    let hostile_admitted = AdmittedRoot::open(&hostile_root).unwrap();
    fs::write(
        hostile_root.join("repos.manifest.toml"),
        fs::read(hostile_hub.join("repos.manifest.toml")).unwrap(),
    )
    .unwrap();
    let lock = crate::family_lock::load(&hostile_hub.join("family.lock")).unwrap();
    let transport = LocalTransport {
        sources,
        clone_count: Cell::new(0),
    };
    let error = install_transaction(
        &hostile_admitted,
        &hostile_hub,
        &lock,
        true,
        &transport,
        &SideEffectValidator::clean(),
        &NoFault,
    )
    .unwrap_err();
    assert_eq!(error.code(), "FAMILY_COMMIT_INCOMPLETE");
    assert_eq!(transport.clone_count.get(), 0);
    fs::remove_dir_all(fixture).unwrap();
}

fn install_hub(sources: &Path, root: &Path, home: &Path) -> PathBuf {
    let hub = root.join("bullet-farm");
    super::super::clone_repository(sources.join("bullet-farm").as_os_str(), &hub, true).unwrap();
    test_git(&hub, home, &["checkout", "--detach", "--force", TAG]);
    hub
}

fn assert_authority_invariant(
    root: &Path,
    hub: &Path,
    lock: &FamilyLock,
    verifier: &dyn FamilyVerifier,
) {
    if root.join("repos.manifest.toml").exists() {
        verifier
            .verify_family(root, hub, lock)
            .expect("a durable marker covers only a complete family");
        return;
    }
    let allowed = hub.join("release/allowed_signers");
    for member in &lock.member {
        let path = root.join(&member.name);
        if path.exists() {
            verifier
                .verify_member(member, &path, root, &allowed)
                .expect("partial member remains exact");
        }
    }
}

#[cfg(unix)]
fn existing_identities(root: &Path) -> Vec<(String, u64, u64)> {
    MEMBERS
        .iter()
        .filter_map(|name| {
            fs::metadata(root.join(name))
                .ok()
                .map(|metadata| ((*name).to_owned(), metadata.dev(), metadata.ino()))
        })
        .collect()
}

#[cfg(not(unix))]
fn existing_identities(_root: &Path) -> Vec<(String, u64, u64)> {
    Vec::new()
}

#[cfg(unix)]
fn assert_existing_not_replaced(root: &Path, expected: &[(String, u64, u64)]) {
    for (name, device, inode) in expected {
        let actual = fs::metadata(root.join(name)).unwrap();
        assert_eq!((actual.dev(), actual.ino()), (*device, *inode));
    }
}

#[cfg(not(unix))]
fn assert_existing_not_replaced(_root: &Path, _expected: &[(String, u64, u64)]) {}
