use super::*;

#[test]
fn every_role_and_each_dirty_state_refuse() {
    for (name, _) in WAVE0_REPOSITORIES {
        for state in ["untracked", "unstaged", "staged"] {
            let fixture = Fixture::new(None);
            let repo = fixture.repo(name);
            let tracked = repo.join("subject.txt");
            match state {
                "untracked" => fs::write(repo.join("untracked"), b"x").unwrap(),
                "unstaged" => fs::write(tracked, b"unstaged\n").unwrap(),
                "staged" => {
                    fs::write(&tracked, b"staged\n").unwrap();
                    git(&repo, &["add", "subject.txt"]);
                }
                _ => unreachable!(),
            }
            assert_fixture_shape(&repo, state);
            fixture.assert_code("DIRTY_CHECKOUT");
        }
    }
}

fn assert_fixture_shape(repo: &Path, state: &str) {
    let cached = git_output(repo, &["diff", "--cached", "--name-only", "--"]);
    let worktree = git_output(repo, &["diff", "--name-only", "--"]);
    let status = git_output(
        repo,
        &["status", "--porcelain=v2", "--untracked-files=all", "--"],
    );
    assert_eq!(status.lines().count(), 1);
    let shape = if status.starts_with("? ") {
        "?"
    } else {
        status.split_whitespace().nth(1).unwrap()
    };
    let path = status.split_whitespace().last().unwrap();
    let expected = match state {
        "untracked" => ("", "", "?", "untracked"),
        "unstaged" => ("", "subject.txt\n", ".M", "subject.txt"),
        "staged" => ("subject.txt\n", "", "M.", "subject.txt"),
        _ => unreachable!(),
    };
    assert_eq!((cached.as_str(), worktree.as_str(), shape, path), expected);
}
