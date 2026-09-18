use std::{collections::VecDeque, fs, os::unix::fs::PermissionsExt, time::Duration};

use super::super::tests::Fixture;
use super::*;

fn pull(intent: &Intent) -> Value {
    serde_json::json!({
        "number":7, "html_url":"https://github.com/neverhuman/bulletfarm/pull/7",
        "user":{"id":42,"login":"bullet-publisher[bot]","type":"Bot"},
        "head":{"ref":intent.head,"sha":intent.aggregate_commit,
            "repo":{"id":123,"full_name":REPOSITORY}},
        "base":{"ref":"main","repo":{"id":123,"full_name":REPOSITORY}},
        "title":intent.title,"body":intent.body,"state":"open",
        "draft":false,"maintainer_can_modify":false
    })
}

struct Script {
    replies: VecDeque<(&'static str, Result<Value>)>,
    posts: usize,
}

impl Script {
    fn new(replies: impl IntoIterator<Item = (&'static str, Result<Value>)>) -> Self {
        Self {
            replies: replies.into_iter().collect(),
            posts: 0,
        }
    }

    fn call(
        &mut self,
        store: &Store,
        intent: &Intent,
        method: &str,
        endpoint: &str,
        body: Option<&Value>,
    ) -> Result<Value> {
        let (expected, reply) = self
            .replies
            .pop_front()
            .expect("unexpected additional API call");
        match expected {
            "list" => {
                assert_eq!(method, "GET");
                assert_eq!(
                    endpoint,
                    format!(
                        "{PULLS}?state=all&head=neverhuman%3Apublication%2Ffixture-1&base=main&per_page=100&page=1"
                    )
                );
                assert!(body.is_none());
            }
            "read" => {
                assert_eq!(
                    (method, endpoint),
                    ("GET", "/repos/neverhuman/bulletfarm/pulls/7")
                );
                assert!(body.is_none());
            }
            "create" => {
                assert_eq!((method, endpoint), ("POST", PULLS));
                self.posts += 1;
                let attempted: Value = decode(
                    &read_record(&store.path("fixture-1", "pr-attempt"))
                        .unwrap()
                        .unwrap(),
                )
                .unwrap();
                assert_eq!(attempted["intent_sha256"], digest(&encode(intent).unwrap()));
                assert_eq!(
                    body.unwrap(),
                    &serde_json::json!({"title":intent.title,"body":intent.body,
                    "head":intent.head,"base":intent.base,"draft":false,"maintainer_can_modify":false})
                );
            }
            _ => panic!("unknown fixture step"),
        }
        reply
    }
}

fn lost() -> Result<Value> {
    Err(CoordError::new(
        "FIXTURE_RESPONSE_LOST",
        "no response available",
    ))
}

#[test]
fn creation_receipt_reopens_same_pr_after_restart_closed_and_merged() {
    let fixture = Fixture::new();
    let (store, request, prepared) = fixture.prepared();
    let intent = intent(&request, &prepared);
    let value = pull(&intent);
    let mut script = Script::new([
        ("list", Ok(serde_json::json!([]))),
        ("create", Ok(value.clone())),
        ("read", Ok(value.clone())),
    ]);
    let bytes = perform(
        &store,
        &request,
        &prepared,
        |m, e, b| script.call(&store, &intent, m, e, b),
        || Ok(()),
    )
    .unwrap();
    assert_eq!(script.posts, 1);
    assert!(script.replies.is_empty());
    assert_eq!(
        fs::read(store.path("fixture-1", "pr-receipt")).unwrap(),
        bytes
    );
    drop(store);
    let store = Store::open(&fixture.temp.path().join("store")).unwrap();
    let (request, prepared) = store.load("fixture-1").unwrap();
    for state in ["open", "closed", "merged"] {
        let mut current = value.clone();
        current["state"] = serde_json::json!(if state == "open" { "open" } else { "closed" });
        current["merged_at"] = serde_json::json!(if state == "merged" {
            Some("2026-09-07T12:00:00Z")
        } else {
            None
        });
        current["draft"] = serde_json::json!(true);
        let mut script = Script::new([("read", Ok(current))]);
        assert_eq!(
            perform(
                &store,
                &request,
                &prepared,
                |m, e, b| script.call(&store, &intent, m, e, b),
                || Ok(())
            )
            .unwrap(),
            bytes
        );
        assert_eq!(script.posts, 0);
        assert!(script.replies.is_empty());
    }
}

#[test]
fn response_loss_reconciles_one_attempt_and_never_reposts_unknown_outcome() {
    for found in [false, true] {
        let fixture = Fixture::new();
        let (store, request, prepared) = fixture.prepared();
        let intent = intent(&request, &prepared);
        let value = pull(&intent);
        let mut replies = vec![
            ("list", Ok(serde_json::json!([]))),
            ("create", lost()),
            (
                "list",
                Ok(if found {
                    serde_json::json!([value.clone()])
                } else {
                    serde_json::json!([])
                }),
            ),
        ];
        if found {
            replies.push(("read", Ok(value)));
        }
        let mut script = Script::new(replies);
        let result = perform(
            &store,
            &request,
            &prepared,
            |m, e, b| script.call(&store, &intent, m, e, b),
            || Ok(()),
        );
        assert_eq!(script.posts, 1);
        assert!(script.replies.is_empty());
        if found {
            assert!(result.is_ok());
        } else {
            assert_eq!(result.unwrap_err().code(), "PUBLICATION_PR_OUTCOME_UNKNOWN");
            assert!(
                read_record(&store.path("fixture-1", "pr-attempt"))
                    .unwrap()
                    .is_some()
            );
            drop(store);
            let store = Store::open(&fixture.temp.path().join("store")).unwrap();
            let (request, prepared) = store.load("fixture-1").unwrap();
            let mut script = Script::new([("list", Ok(serde_json::json!([])))]);
            assert_eq!(
                perform(
                    &store,
                    &request,
                    &prepared,
                    |m, e, b| script.call(&store, &intent, m, e, b),
                    || Ok(())
                )
                .unwrap_err()
                .code(),
                "PUBLICATION_PR_OUTCOME_UNKNOWN"
            );
            assert_eq!(script.posts, 0);
            assert!(script.replies.is_empty());
        }
    }
}

#[test]
fn existing_human_changed_head_marker_and_duplicate_prs_refuse_creation() {
    for kind in ["human", "head", "marker", "repository", "duplicate"] {
        let fixture = Fixture::new();
        let (store, request, prepared) = fixture.prepared();
        let intent = intent(&request, &prepared);
        let mut value = pull(&intent);
        let code = match kind {
            "human" => {
                value["user"]["type"] = serde_json::json!("User");
                "PUBLICATION_PR_AUTHOR_INVALID"
            }
            "head" => {
                value["head"]["sha"] = serde_json::json!("0".repeat(40));
                "PUBLICATION_PR_SUBJECT_DRIFT"
            }
            "marker" => {
                value["body"] = serde_json::json!("different request");
                "PUBLICATION_PR_REQUEST_CONFLICT"
            }
            "repository" => {
                value["head"]["repo"]["id"] = serde_json::json!(456);
                "PUBLICATION_PR_REPOSITORY_INVALID"
            }
            _ => "PUBLICATION_PR_MULTIPLE_MATCHES",
        };
        let list = if kind == "duplicate" {
            serde_json::json!([value.clone(), value])
        } else {
            serde_json::json!([value])
        };
        let mut script = Script::new([("list", Ok(list))]);
        assert_eq!(
            perform(
                &store,
                &request,
                &prepared,
                |m, e, b| script.call(&store, &intent, m, e, b),
                || Ok(())
            )
            .unwrap_err()
            .code(),
            code
        );
        assert_eq!(script.posts, 0);
        assert!(script.replies.is_empty());
        assert!(
            read_record(&store.path("fixture-1", "pr-receipt"))
                .unwrap()
                .is_none()
        );
        assert!(
            read_record(&store.path("fixture-1", "pr-attempt"))
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn preexisting_closed_bot_pr_is_adopted_but_author_readback_drift_refuses() {
    for drift in [false, true] {
        let fixture = Fixture::new();
        let (store, request, prepared) = fixture.prepared();
        let intent = intent(&request, &prepared);
        let mut listed = pull(&intent);
        listed["state"] = serde_json::json!("closed");
        let mut current = listed.clone();
        if drift {
            current["user"]["id"] = serde_json::json!(43);
        }
        let mut script = Script::new([
            ("list", Ok(serde_json::json!([listed]))),
            ("read", Ok(current)),
        ]);
        let result = perform(
            &store,
            &request,
            &prepared,
            |m, e, b| script.call(&store, &intent, m, e, b),
            || Ok(()),
        );
        if drift {
            assert_eq!(result.unwrap_err().code(), "PUBLICATION_PR_READBACK_DRIFT");
        } else {
            assert!(result.is_ok());
        }
        assert_eq!(script.posts, 0);
        assert!(script.replies.is_empty());
        assert!(
            read_record(&store.path("fixture-1", "pr-attempt"))
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn remote_drift_and_changed_intent_refuse_before_creation() {
    let fixture = Fixture::new();
    let (store, request, prepared) = fixture.prepared();
    let intent = intent(&request, &prepared);
    let mut script = Script::new([("list", Ok(serde_json::json!([])))]);
    let mut checks = 0;
    assert_eq!(
        perform(
            &store,
            &request,
            &prepared,
            |m, e, b| script.call(&store, &intent, m, e, b),
            || {
                checks += 1;
                require(checks == 1, "PUBLICATION_PR_REMOTE_DRIFT")
            }
        )
        .unwrap_err()
        .code(),
        "PUBLICATION_PR_REMOTE_DRIFT"
    );
    assert_eq!(script.posts, 0);
    assert!(script.replies.is_empty());
    assert!(
        read_record(&store.path("fixture-1", "pr-attempt"))
            .unwrap()
            .is_none()
    );
    let mut changed: Prepared = decode(&encode(&prepared).unwrap()).unwrap();
    changed.aggregate_commit = "0".repeat(40);
    assert!(
        perform(
            &store,
            &request,
            &changed,
            |_, _, _| panic!("API called for conflicting intent"),
            || Ok(())
        )
        .is_err()
    );
}

#[test]
fn receipt_substitution_and_deleted_review_ref_do_not_create_another_pr() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    let (store, request, prepared) = fixture.prepared();
    let intent = intent(&request, &prepared);
    let value = pull(&intent);
    let receipt = encode(&validate_pull(&value, &intent).unwrap()).unwrap();
    let path = store.path("fixture-1", "pr-receipt");
    persist(&path, &receipt).unwrap();
    assert_eq!(
        perform(
            &store,
            &request,
            &prepared,
            |_, _, _| panic!("API called despite missing ref"),
            || Err(CoordError::new(
                "PUBLICATION_PR_REMOTE_DRIFT",
                "review ref deleted"
            ))
        )
        .unwrap_err()
        .code(),
        "PUBLICATION_PR_REMOTE_DRIFT"
    );
    fs::remove_file(&path).unwrap();
    let target = fixture.temp.path().join("substituted-receipt");
    fs::write(&target, receipt).unwrap();
    symlink(&target, &path).unwrap();
    assert!(
        perform(
            &store,
            &request,
            &prepared,
            |_, _, _| panic!("API called for substituted receipt"),
            || Ok(())
        )
        .is_err()
    );
}

#[test]
fn github_executable_refuses_relative_symlink_and_wrong_digest_subjects() {
    use std::os::unix::fs::symlink;
    assert_eq!(
        transport::gh_executable(Path::new("gh"))
            .unwrap_err()
            .code(),
        "PUBLICATION_GH_PATH_INVALID"
    );
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("gh");
    fs::write(&path, "unadmitted executable").unwrap();
    assert_eq!(
        transport::gh_executable(&path).unwrap_err().code(),
        "PUBLICATION_GH_SUBSTITUTED"
    );
    let link = temp.path().join("link");
    symlink(&path, &link).unwrap();
    assert!(transport::gh_executable(&link).is_err());
}

#[test]
fn github_api_child_has_private_configuration_and_no_inherited_credentials() {
    let temp = tempfile::tempdir().unwrap();
    let executable = temp.path().join("print-environment");
    fs::write(&executable, "#!/bin/sh\nexec /usr/bin/env -0\n").unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
    let mut command = transport::api_command(
        &executable,
        temp.path(),
        "fixture-only-app-token",
        "GET",
        PULLS,
    );
    assert!(
        !format!("{:?}", command.get_args().collect::<Vec<_>>()).contains("fixture-only-app-token")
    );
    let output = crate::process::run_bounded(
        &mut command,
        "offline environment fixture",
        crate::process::Limits {
            timeout: Duration::from_secs(5),
            stdout_bytes: 4096,
            stderr_bytes: 4096,
        },
    )
    .unwrap();
    assert!(output.status.success());
    let environment = String::from_utf8(output.stdout).unwrap();
    let entries = environment
        .split('\0')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    for entry in &entries {
        let name = entry.split_once('=').unwrap().0;
        assert!(
            [
                "GH_CONFIG_DIR",
                "GH_PROMPT_DISABLED",
                "GH_TOKEN",
                "PATH",
                "PWD",
                "SHLVL",
                "_"
            ]
            .contains(&name),
            "unexpected child variable {name}"
        );
    }
    assert!(entries.contains(&"GH_TOKEN=fixture-only-app-token"));
    assert!(entries.contains(&format!("GH_CONFIG_DIR={}", temp.path().display()).as_str()));
    assert!(!entries.iter().any(|v| v.starts_with("HOME=")));
}
