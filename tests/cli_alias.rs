//! `bulletfarm` and `bf` are the same program: same version, same verbs, same
//! `BF_DATA_DIR` / `~/.bf`. Neither name creates a second database.
use std::process::{Command, Output};

fn bin(name: &str) -> Command {
    let path = if name == "bulletfarm" {
        env!("CARGO_BIN_EXE_bulletfarm")
    } else {
        env!("CARGO_BIN_EXE_bf")
    };
    let mut c = Command::new(path);
    c.env_remove("BF_AGENT");
    c.env_remove("BF_DATA_DIR");
    c
}

fn run(name: &str, data: &str, args: &[&str]) -> Output {
    bin(name)
        .env("HOME", data)
        .env("BF_DATA_DIR", data)
        .args(["--data-dir", data])
        .args(args)
        .output()
        .unwrap()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn both_bins_print_the_same_version() {
    let a = bin("bulletfarm").arg("--version").output().unwrap();
    let b = bin("bf").arg("--version").output().unwrap();
    assert!(a.status.success() && b.status.success());
    let va = stdout(&a);
    let vb = stdout(&b);
    assert!(va.contains("0.1.0"), "bulletfarm version:\n{va}");
    assert!(vb.contains("0.1.0"), "bf version:\n{vb}");
    let na = va.split_whitespace().last().unwrap_or(va.as_str());
    let nb = vb.split_whitespace().last().unwrap_or(vb.as_str());
    assert_eq!(na, nb, "version numbers differ:\n{va}\n{vb}");
}

#[test]
fn help_names_the_invoked_binary_and_lists_the_alias() {
    let farm = stdout(&bin("bulletfarm").arg("--help").output().unwrap());
    let alias = stdout(&bin("bf").arg("--help").output().unwrap());
    assert!(
        farm.contains("Usage: bulletfarm"),
        "bulletfarm help must name itself:\n{farm}"
    );
    assert!(
        alias.contains("Usage: bf"),
        "bf help must name itself:\n{alias}"
    );
    assert!(
        farm.contains("`bulletfarm` or `bf`"),
        "about must mention both names:\n{farm}"
    );
}

#[test]
fn claim_with_one_name_is_visible_to_the_other_in_the_same_data_dir() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path().to_str().unwrap();
    // Isolate from the live checkout's board: a private git repo as --repo.
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/lib.rs"), "//\n").unwrap();
    for args in [
        ["init", "-q"].as_slice(),
        ["add", "-A"].as_slice(),
        [
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-q",
            "-m",
            "base",
        ]
        .as_slice(),
    ] {
        let st = Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .status()
            .unwrap();
        assert!(st.success(), "git {args:?}");
    }
    let claimed = run(
        "bulletfarm",
        d,
        &[
            "claim",
            "src/lib.rs",
            "-m",
            "alias",
            "--as",
            "t",
            "--repo",
            d,
        ],
    );
    assert_eq!(
        claimed.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&claimed.stderr)
    );
    let board = run("bf", d, &["board"]);
    assert_eq!(
        board.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&board.stderr)
    );
    let text = stdout(&board);
    assert!(
        text.contains("src/lib.rs"),
        "bf board must see bulletfarm's claim:\n{text}"
    );
    let sqlite = dir.path().join("bf.sqlite");
    assert!(
        sqlite.is_file(),
        "alias must use the existing bf.sqlite name"
    );
    assert_eq!(
        std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .is_some_and(|x| x == "sqlite" || x == "db")
            })
            .count(),
        1,
        "rename must not create a second database"
    );
}
