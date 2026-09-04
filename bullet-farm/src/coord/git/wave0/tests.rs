use std::{
    fs::{self, File, OpenOptions, hard_link},
    io::Write,
    os::unix::{ffi::OsStringExt, fs::symlink},
    path::{Path, PathBuf},
    process::Command,
};

use super::*;

mod dirty;

struct Fixture {
    root: tempfile::TempDir,
}

impl Fixture {
    fn new(sha256_member: Option<&str>) -> Self {
        let root = tempfile::tempdir().unwrap();
        for (name, _) in WAVE0_REPOSITORIES {
            init_repo(root.path(), name, sha256_member == Some(name));
        }
        write_manifest(root.path(), &WAVE0_REPOSITORIES);
        fs::write(
            root.path().join("AGENT_CHAT.md"),
            b"stable collaboration prefix\n",
        )
        .unwrap();
        Self { root }
    }

    fn path(&self) -> &Path {
        self.root.path()
    }

    fn repo(&self, name: &str) -> PathBuf {
        self.path().join(name)
    }

    fn assert_code(&self, expected: &str) {
        assert_eq!(
            observe_wave0_mechanical(self.path()).unwrap_err().code(),
            expected
        );
    }
}

#[test]
fn clean_family_observation_is_exact_and_repeatable() {
    let fixture = Fixture::new(None);
    let first = observe_wave0_mechanical(fixture.path()).unwrap();
    let second = observe_wave0_mechanical(fixture.path()).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.members.len(), 4);
    assert_eq!(first.members[0].repository_identity, "root/bullet-farm");
    assert_eq!(first.members[1].role, Wave0MemberRoleV1::Kernel);
    assert!(first.members.iter().all(|member| {
        member.commit_oid.starts_with("sha1:")
            && member.tree_oid.starts_with("sha1:")
            && member.index_state == Wave0CleanStateV1::Clean
            && member.worktree_state == Wave0CleanStateV1::Clean
            && member.untracked_state == Wave0CleanStateV1::Clean
    }));
    assert_eq!(first.collaboration_log_byte_length, 28);
    assert!(first.collaboration_log_sha256.starts_with("sha256:"));
    assert!(
        first
            .collaboration_log_path_hex
            .ends_with("4147454e545f434841542e6d64")
    );
}

#[test]
fn sha256_repository_and_malformed_subjects_refuse() {
    let fixture = Fixture::new(Some("bullet-portal"));
    fixture.assert_code("UNSUPPORTED_GIT_OBJECT_FORMAT");
    let oid = "a".repeat(40);
    for malformed in [
        format!("sha256\n{oid}\n{oid}\n"),
        format!("sha1\n{oid}\n"),
        format!("sha1\n{oid}\n{oid}\ntail\n"),
        format!("sha1\n{}\n{oid}\n", "A".repeat(40)),
    ] {
        assert!(parse_subject(malformed.as_bytes()).is_err());
    }
}

#[test]
fn manifest_order_path_identity_and_races_refuse() {
    let fixture = Fixture::new(None);
    let guard = Wave0FamilyGuard::open(fixture.path()).unwrap();
    let manifest = fixture.path().join("repos.manifest.toml");
    OpenOptions::new()
        .append(true)
        .open(&manifest)
        .unwrap()
        .write_all(b"\n")
        .unwrap();
    assert!(guard.revalidate().is_err());

    let fixture = Fixture::new(None);
    let mut wrong = WAVE0_REPOSITORIES;
    wrong.swap(1, 2);
    write_manifest(fixture.path(), &wrong);
    assert!(Wave0FamilyGuard::open(fixture.path()).is_err());

    let fixture = Fixture::new(None);
    let text = render_manifest(fixture.path(), &WAVE0_REPOSITORIES)
        .replace("root/bullet-git", "root/not-bullet-git");
    fs::write(fixture.path().join("repos.manifest.toml"), text).unwrap();
    assert!(Wave0FamilyGuard::open(fixture.path()).is_err());

    let fixture = Fixture::new(None);
    let text = render_manifest(fixture.path(), &WAVE0_REPOSITORIES)
        .replace("/bullet-portal\"", "/not-portal\"");
    fs::write(fixture.path().join("repos.manifest.toml"), text).unwrap();
    assert!(Wave0FamilyGuard::open(fixture.path()).is_err());

    for transform in [
        |text: String| text.replace("1.2.0", "1.1.0"),
        |text: String| text.replace("family = \"bullet-farm\"", "family = \"other\""),
    ] {
        let fixture = Fixture::new(None);
        let text = transform(render_manifest(fixture.path(), &WAVE0_REPOSITORIES));
        fs::write(fixture.path().join("repos.manifest.toml"), text).unwrap();
        assert!(Wave0FamilyGuard::open(fixture.path()).is_err());
    }

    let fixture = Fixture::new(None);
    write_manifest(fixture.path(), &WAVE0_REPOSITORIES[..3]);
    assert!(Wave0FamilyGuard::open(fixture.path()).is_err());

    let fixture = Fixture::new(None);
    let mut extra = WAVE0_REPOSITORIES.to_vec();
    extra.push(("bullet-extra", "root/bullet-extra"));
    write_manifest(fixture.path(), &extra);
    assert!(Wave0FamilyGuard::open(fixture.path()).is_err());
}

#[test]
fn git_redirection_and_dangerous_configuration_refuse() {
    let fixture = Fixture::new(None);
    let repo = fixture.repo("bullet-git");
    OpenOptions::new()
        .append(true)
        .open(repo.join(".git/config"))
        .unwrap()
        .write_all(b"[extensions]\n\tworktreeConfig = true\n")
        .unwrap();
    fixture.assert_code("UNSAFE_GIT_METADATA");

    let fixture = Fixture::new(None);
    install_replacement(&fixture.repo("bullet-farm"));
    fixture.assert_code("UNSAFE_GIT_METADATA");

    let fixture = Fixture::new(None);
    fs::write(
        fixture
            .repo("bullet-kernel")
            .join(".git/objects/info/alternates"),
        b"/unadmitted/object/store\n",
    )
    .unwrap();
    fixture.assert_code("REPOSITORY_IDENTITY_MISMATCH");

    let fixture = Fixture::new(None);
    let repo = fixture.repo("bullet-portal");
    fs::rename(repo.join(".git"), repo.join("git-real")).unwrap();
    symlink("git-real", repo.join(".git")).unwrap();
    assert!(observe_wave0_mechanical(fixture.path()).is_err());

    let fixture = Fixture::new(None);
    let repo = fixture.repo("bullet-portal");
    fs::rename(repo.join(".git"), repo.join("git-real")).unwrap();
    fs::write(repo.join(".git"), b"gitdir: git-real\n").unwrap();
    assert!(observe_wave0_mechanical(fixture.path()).is_err());

    let fixture = Fixture::new(None);
    fs::write(
        fixture
            .repo("bullet-git")
            .join(".git/objects/info/http-alternates"),
        b"https://example.invalid/objects\n",
    )
    .unwrap();
    assert!(observe_wave0_mechanical(fixture.path()).is_err());

    let fixture = Fixture::new(None);
    let marker = fixture
        .repo("bullet-git")
        .join(".git/objects/pack/hostile.promisor");
    fs::write(&marker, b"promisor\n").unwrap();
    assert!(observe_wave0_mechanical(fixture.path()).is_err());
    assert_eq!(fs::read(&marker).unwrap(), b"promisor\n");

    let fixture = Fixture::new(None);
    fs::write(
        fixture.repo("bullet-git").join(".git/info/exclude"),
        b"hidden-*\n",
    )
    .unwrap();
    fixture.assert_code("UNSAFE_GIT_METADATA");
}

#[test]
fn packed_replacement_refs_refuse_without_mutation() {
    let fixture = Fixture::new(None);
    let repo = fixture.repo("bullet-farm");
    install_replacement(&repo);
    git(&repo, &["pack-refs", "--all", "--prune"]);
    let loose = repo.join(".git/refs/replace");
    if loose.exists() {
        fs::remove_dir_all(loose).unwrap();
    }
    let before = fs::read(repo.join(".git/packed-refs")).unwrap();
    fixture.assert_code("UNSAFE_GIT_METADATA");
    assert_eq!(fs::read(repo.join(".git/packed-refs")).unwrap(), before);
}

#[test]
fn head_index_tree_and_worktree_changes_inside_brackets_refuse() {
    let fixture = Fixture::new(None);
    let repo = fixture.repo("bullet-kernel");
    let original = git_output(&repo, &["rev-parse", "HEAD"]);
    commit(&repo, &["--allow-empty", "-m", "same tree"]);
    let same_tree = git_output(&repo, &["rev-parse", "HEAD"]);
    git(&repo, &["reset", "--hard", "--quiet", original.trim()]);
    let mut changed = false;
    assert!(
        observe_wave0_mechanical_with(fixture.path(), |name, seam| {
            if name == "bullet-kernel" && seam == ObservationSeam::AfterSubject && !changed {
                changed = true;
                git(&repo, &["reset", "--soft", "--quiet", same_tree.trim()]);
            }
        })
        .is_err()
    );

    let fixture = Fixture::new(None);
    let repo = fixture.repo("bullet-kernel");
    let original = git_output(&repo, &["rev-parse", "HEAD"]);
    fs::write(repo.join("subject.txt"), b"different tree\n").unwrap();
    git(&repo, &["add", "subject.txt"]);
    commit(&repo, &["-m", "different tree"]);
    let different_tree = git_output(&repo, &["rev-parse", "HEAD"]);
    git(&repo, &["reset", "--hard", "--quiet", original.trim()]);
    let mut changed = false;
    assert!(
        observe_wave0_mechanical_with(fixture.path(), |name, seam| {
            if name == "bullet-kernel" && seam == ObservationSeam::AfterSubject && !changed {
                changed = true;
                git(
                    &repo,
                    &["reset", "--hard", "--quiet", different_tree.trim()],
                );
            }
        })
        .is_err()
    );

    let fixture = Fixture::new(None);
    let repo = fixture.repo("bullet-git");
    let mut changed = false;
    assert!(
        observe_wave0_mechanical_with(fixture.path(), |name, seam| {
            if name == "bullet-git" && seam == ObservationSeam::AfterStatus && !changed {
                changed = true;
                fs::write(repo.join("subject.txt"), b"staged between brackets\n").unwrap();
                git(&repo, &["add", "subject.txt"]);
                fs::write(repo.join("subject.txt"), b"subject\n").unwrap();
            }
        })
        .is_err()
    );

    let fixture = Fixture::new(None);
    let repo = fixture.repo("bullet-portal");
    let mut changed = false;
    assert!(
        observe_wave0_mechanical_with(fixture.path(), |name, seam| {
            if name == "bullet-portal" && seam == ObservationSeam::AfterStatus && !changed {
                changed = true;
                fs::write(repo.join("subject.txt"), b"worktree between brackets\n").unwrap();
            }
        })
        .is_err()
    );
}

#[test]
fn descriptor_object_walk_is_bounded_and_path_rebound() {
    let root = tempfile::tempdir().unwrap();
    let directory = root.path().join("directory");
    fs::create_dir(&directory).unwrap();
    fs::write(directory.join("a"), b"a").unwrap();
    fs::write(directory.join("b"), b"b").unwrap();
    let retained = File::open(&directory).unwrap();
    assert_eq!(
        super::super::object_snapshot_for_test(&retained, 2).unwrap(),
        2
    );
    assert!(super::super::object_snapshot_for_test(&retained, 1).is_err());
    let non_utf8 = directory.join(std::ffi::OsString::from_vec(vec![0xff]));
    fs::write(&non_utf8, b"x").unwrap();
    assert!(super::super::object_snapshot_for_test(&retained, 3).is_err());
    fs::remove_file(non_utf8).unwrap();
    symlink("a", directory.join("symbolic")).unwrap();
    assert!(super::super::object_snapshot_for_test(&retained, 3).is_err());

    let fixture = Fixture::new(None);
    let repo = fixture.repo("bullet-farm");
    let objects = repo.join(".git/objects");
    let moved = repo.join(".git/objects.moved");
    assert!(
        super::super::descriptor_object_inventory_after_open(&repo, || {
            fs::rename(&objects, &moved).unwrap();
            symlink(&moved, &objects).unwrap();
        })
        .is_err()
    );
}

#[test]
fn ledger_prefix_allows_append_but_refuses_truncation_swap_and_hardlink() {
    let fixture = Fixture::new(None);
    let family = Wave0FamilyGuard::open(fixture.path()).unwrap();
    let ledger = LedgerPrefix::open(&family, fixture.path()).unwrap();
    OpenOptions::new()
        .append(true)
        .open(fixture.path().join("AGENT_CHAT.md"))
        .unwrap()
        .write_all(b"later append\n")
        .unwrap();
    ledger.revalidate(&family).unwrap();
    OpenOptions::new()
        .write(true)
        .open(fixture.path().join("AGENT_CHAT.md"))
        .unwrap()
        .set_len(4)
        .unwrap();
    assert!(ledger.revalidate(&family).is_err());

    let fixture = Fixture::new(None);
    let family = Wave0FamilyGuard::open(fixture.path()).unwrap();
    let ledger = LedgerPrefix::open(&family, fixture.path()).unwrap();
    fs::rename(
        fixture.path().join("AGENT_CHAT.md"),
        fixture.path().join("AGENT_CHAT.old"),
    )
    .unwrap();
    fs::write(
        fixture.path().join("AGENT_CHAT.md"),
        b"stable collaboration prefix\n",
    )
    .unwrap();
    assert!(ledger.revalidate(&family).is_err());

    let fixture = Fixture::new(None);
    hard_link(
        fixture.path().join("AGENT_CHAT.md"),
        fixture.path().join("ledger-link"),
    )
    .unwrap();
    let family = Wave0FamilyGuard::open(fixture.path()).unwrap();
    assert!(LedgerPrefix::open(&family, fixture.path()).is_err());
}

#[test]
fn ledger_bounds_and_terminal_newline_are_exact() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("ledger");
    fs::write(&path, b"x").unwrap();
    let metadata = fs::metadata(&path).unwrap();
    assert!(validate_ledger_metadata(&metadata, 1).is_ok());
    assert!(validate_ledger_metadata(&metadata, 2).is_err());
    assert_eq!(MAX_LEDGER_BYTES, 67_108_864);
    OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(MAX_LEDGER_BYTES)
        .unwrap();
    assert!(validate_ledger_metadata(&fs::metadata(&path).unwrap(), 1).is_ok());
    OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(MAX_LEDGER_BYTES + 1)
        .unwrap();
    assert!(validate_ledger_metadata(&fs::metadata(&path).unwrap(), 1).is_err());

    let fixture = Fixture::new(None);
    fs::write(fixture.path().join("AGENT_CHAT.md"), b"unterminated").unwrap();
    let family = Wave0FamilyGuard::open(fixture.path()).unwrap();
    assert!(LedgerPrefix::open(&family, fixture.path()).is_err());
}

fn init_repo(root: &Path, name: &str, sha256: bool) {
    let repo = root.join(name);
    fs::create_dir(&repo).unwrap();
    if sha256 {
        git(&repo, &["init", "--quiet", "--object-format=sha256"]);
    } else {
        git(&repo, &["init", "--quiet"]);
    }
    fs::write(repo.join("subject.txt"), b"subject\n").unwrap();
    git(&repo, &["add", "subject.txt"]);
    commit(&repo, &["-m", "subject"]);
}

fn git(repo: &Path, args: &[&str]) {
    let _ = git_run(repo, args);
}

fn git_run(repo: &Path, args: &[&str]) -> std::process::Output {
    let output = Command::new("/usr/bin/git")
        .current_dir(repo)
        .args(args)
        .env_clear()
        .env("LC_ALL", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "Git fixture command failed: {args:?}"
    );
    output
}

fn write_manifest(root: &Path, repositories: &[(&str, &str)]) {
    fs::write(
        root.join("repos.manifest.toml"),
        render_manifest(root, repositories),
    )
    .unwrap();
}

fn render_manifest(root: &Path, repositories: &[(&str, &str)]) -> String {
    let mut text =
        String::from("schema_version = \"1.2.0\"\nfamily = \"bullet-farm\"\nrequired_repos = [\n");
    for (name, _) in repositories {
        text.push_str(&format!("  \"{name}\",\n"));
    }
    text.push_str("]\n");
    for (name, identity) in repositories {
        text.push_str(&format!(
            "\n[[repo]]\nname = \"{name}\"\npath = \"{}/{name}\"\njeryu_slug = \"{identity}\"\n",
            root.display()
        ));
    }
    text
}

fn git_output(repo: &Path, args: &[&str]) -> String {
    String::from_utf8(git_run(repo, args).stdout).unwrap()
}

fn install_replacement(repo: &Path) {
    let replaced = git_output(repo, &["rev-parse", "HEAD"]);
    commit(repo, &["--allow-empty", "-m", "replacement"]);
    let replacement = git_output(repo, &["rev-parse", "HEAD"]);
    git(repo, &["replace", replaced.trim(), replacement.trim()]);
}

fn commit(repo: &Path, args: &[&str]) {
    let mut command = vec![
        "-c",
        "user.name=Wave0 Test",
        "-c",
        "user.email=wave0@example.invalid",
        "commit",
        "--quiet",
    ];
    command.extend_from_slice(args);
    git(repo, &command);
}
