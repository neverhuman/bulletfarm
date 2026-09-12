use super::*;
use std::path::PathBuf;

struct Fixture {
    dir: tempfile::TempDir,
    context: Value,
}
fn write(root: &Path, name: &str, value: &Value) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
}
fn raw(root: &Path, name: &str, data: &[u8]) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, data).unwrap();
}
fn hashes(root: &Path, excluded: &str) -> Value {
    json!(inventory(root)
        .unwrap()
        .into_iter()
        .filter(|p| p != excluded)
        .map(|p| json!({"sha256":crate::evidence::hash(&root.join(&p)).unwrap(),"path":p}))
        .collect::<Vec<_>>())
}
fn read(root: &Path, name: &str) -> Value {
    document(root, name).unwrap()
}
fn source_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
impl Fixture {
    // Deliberately synthetic byte fixtures exercise acceptance logic, never PTY qualification.
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let context = json!({"commit":"diagnostic-source","tree":"diagnostic-tree","workflow_sha256":"diagnostic-workflow",
            "workflow_commit":"diagnostic-workflow-source","workflow_ref":"diagnostic-ref","repository":"fixture/component",
            "event":"push","run_id":"123","run_attempt":"1","workspace":"/diagnostic/workspace","job":"operator-tui"});
        let selected = crate::custody::IDENTITIES
            .iter()
            .chain(crate::cases::IDENTITIES.iter())
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let rows = selected
            .iter()
            .map(|id| json!({"id":id,"outcome":"PASS","error":null}))
            .collect::<Vec<_>>();
        let mut receipt = json!({"schema":"bullet.operator-tui.hosted.v1","context":context,"process_exit":0,
            "build_target":"/diagnostic/build","executables":{},"artifacts":[]});
        let sources=crate::evidence::SOURCE_NAMES.iter().map(|name|json!({"path":name,"sha256":crate::evidence::hash(&source_root().join(name)).unwrap()})).collect::<Vec<_>>();
        for alias in ["bullet", "bulletfarm"] {
            let bin = format!("bin/{alias}");
            raw(
                root,
                &bin,
                b"\x7fELF explicitly synthetic executable fixture",
            );
            let exe = json!({"path":bin,"original_path":root.join(&bin),"sha256":crate::evidence::hash(&root.join(&bin)).unwrap()});
            receipt["executables"][alias] = exe.clone();
            let at = root.join(alias);
            raw(&at, "harness", b"synthetic harness identity");
            raw(&at, "exit-status", b"0\n");
            let subjects = json!({"bullet":{"path":exe["original_path"],"sha256":exe["sha256"]},
                "harness":{"path":root.join(alias).join("harness"),"sha256":crate::evidence::hash(&at.join("harness")).unwrap()},
                "runtime_observed_source":sources});
            write(
                &at,
                "inputs.json",
                &json!({"profile":"hosted-component","hosted_subjects":context,"bullet":subjects["bullet"]}),
            );
            write(&at, "selected.json", &json!(selected));
            write(&at, "run/subjects.json", &subjects);
            let mut profile = context.clone();
            profile["profile"] = json!("hosted-component");
            profile["profile_source_sha256"] =
                json!(crate::evidence::hash(&source_root().join("src/profile.rs")).unwrap());
            let mut events = vec![
                json!({"sequence":1,"case":"profile","kind":"admitted_component_context","data":profile}),
            ];
            for row in &rows {
                let id = row["id"].as_str().unwrap();
                let mut observations = Vec::new();
                let count = match id {
                    "six_withheld_http" | "six_locked_credentials" | "six_missing_credentials" => 6,
                    "revoked_owner_recovery" => 2,
                    "custody_after_page_failure" | "custody_parent_death" => 0,
                    _ => 1,
                };
                for _ in 0..count {
                    let mut data = json!({"kind":"process_settlement","reaped":true,"code":0,"signal":null,"terminal_restored":true,"termination_requested":false,"terminal_read_error":null});
                    match id {
                        "custody_nonce_rejection" => {
                            data["code"] = json!(1);
                            data["terminal_restored"] = json!(false);
                            data["termination_requested"] = json!(true);
                        }
                        "custody_exec_failure" => data["code"] = json!(1),
                        "custody_nonzero_exit" => data["code"] = json!(19),
                        "custody_detach_timeout" => {
                            data["code"] = Value::Null;
                            data["signal"] = json!(9);
                            data["terminal_restored"] = json!(false);
                            data["termination_requested"] = json!(true);
                        }
                        "custody_partial_terminal_restore" => {
                            data["terminal_restored"] = json!(false)
                        }
                        _ => {}
                    }
                    observations.push(data);
                }
                match id {
                    "custody_nonce_rejection"=>observations.push(json!({"kind":"failure_screen","cols":120,"rows":30,"text":"diagnostic failure screen"})),
                    "custody_after_page_failure"=>observations.push(json!({"kind":"preconstruction_cleanup","reaped":true,"code":null,"signal":9})),
                    "custody_parent_death"=>observations.push(json!({"kind":"parent_fixture_exit","diagnostic_error":null,"status":"Ok(ExitStatus(unix_wait_status(9)))","stderr":""})),_=>{}
                }
                let fixtures = match id {
                    "six_withheld_http" | "six_locked_credentials" | "six_missing_credentials" => 1,
                    "revoked_owner_recovery" => 2,
                    _ => 0,
                };
                for _ in 0..fixtures {
                    observations.push(json!({"kind":"http_fixture_settled","threads_joined":true,"errors":[],"reads":1}));
                }
                for data in observations {
                    events.push(json!({"sequence":events.len()+1,"case":id,"kind":"process_custody","data":data}));
                }
                events.push(
                    json!({"sequence":events.len()+1,"case":id,"kind":"completed","data":row}),
                );
            }
            raw(
                &at,
                "run/events.jsonl",
                events
                    .iter()
                    .map(|e| format!("{}\n", serde_json::to_string(e).unwrap()))
                    .collect::<String>()
                    .as_bytes(),
            );
            raw(
                &at,
                "run/junit.xml",
                crate::junit::render(&selected, &rows, &[])
                    .unwrap()
                    .as_bytes(),
            );
            write(
                &at,
                "run/manifest.json",
                &json!({"schema":1,"evidence_class":"COMPONENT_PROOF","outcome":"PASS",
                "installed_authenticated":false,"release_eligible":false,"failures":[],"selected":selected,"completed":rows,
                "subjects":subjects,"artifacts":hashes(&at.join("run"),"manifest.json")}),
            );
            Self::build(
                &at,
                "build.jsonl",
                &[(
                    "bullet-tuiwright-qualification",
                    "/diagnostic/build/release/bullet-tuiwright-qualification",
                )],
            );
        }
        Self::build(
            root,
            "cli-build.jsonl",
            &[
                ("bullet", "/diagnostic/build/debug/bullet"),
                ("bulletfarm", "/diagnostic/build/debug/bulletfarm"),
            ],
        );
        write(root, "receipt.json", &receipt);
        let f = Self { dir, context };
        f.reseal();
        f
    }
    fn build(root: &Path, name: &str, targets: &[(&str, &str)]) {
        let mut rows=targets.iter().map(|(n,e)|json!({"reason":"compiler-artifact","target":{"name":n,"kind":["bin"]},"executable":e})).collect::<Vec<_>>();
        rows.push(json!({"reason":"build-finished","success":true}));
        raw(
            root,
            name,
            rows.iter()
                .map(|r| format!("{r}\n"))
                .collect::<String>()
                .as_bytes(),
        );
    }
    fn reseal(&self) {
        let root = self.dir.path();
        for alias in ["bullet", "bulletfarm"] {
            let at = root.join(alias);
            let mut manifest = read(&at, "run/manifest.json");
            manifest["artifacts"] = hashes(&at.join("run"), "manifest.json");
            write(&at, "run/manifest.json", &manifest);
        }
        let mut receipt = read(root, "receipt.json");
        receipt["artifacts"] = hashes(root, "receipt.json");
        write(root, "receipt.json", &receipt);
    }
    fn verify(&self) -> Result<()> {
        verify(self.dir.path(), &self.context, &source_root())
    }
    fn mutate(&self, name: &str, f: impl FnOnce(&mut Value)) {
        let mut v = read(self.dir.path(), name);
        f(&mut v);
        write(self.dir.path(), name, &v);
    }
}
#[test]
fn dual_alias_synthetic_consumer_control_passes() {
    Fixture::new().verify().unwrap();
}
#[test]
fn stale_run_source_workflow_and_job_refuse_even_after_rehash() {
    for key in [
        "commit",
        "tree",
        "workflow_sha256",
        "workflow_commit",
        "workflow_ref",
        "repository",
        "event",
        "run_id",
        "run_attempt",
        "workspace",
        "job",
    ] {
        let f = Fixture::new();
        f.mutate("receipt.json", |v| v["context"][key] = json!("changed"));
        f.reseal();
        assert!(f.verify().is_err(), "{key}");
    }
}
#[test]
fn altered_or_missing_binaries_artifacts_and_symlinks_refuse() {
    for name in [
        "bin/bullet",
        "bin/bulletfarm",
        "bullet/harness",
        "bullet/run/junit.xml",
    ] {
        let f = Fixture::new();
        raw(f.dir.path(), name, b"changed");
        assert!(f.verify().is_err(), "{name}");
        if name != "bullet/run/junit.xml" {
            f.reseal();
            assert!(f.verify().is_err(), "rehash {name}");
        }
    }
    let f = Fixture::new();
    std::fs::remove_file(f.dir.path().join("bullet/run/junit.xml")).unwrap();
    assert!(f.verify().is_err());
    let f = Fixture::new();
    std::os::unix::fs::symlink("run/junit.xml", f.dir.path().join("bullet/link")).unwrap();
    assert!(f.verify().is_err());
    let f = Fixture::new();
    raw(f.dir.path(), "unlisted", b"extra");
    assert!(f.verify().is_err());
}
#[test]
fn semantic_report_and_process_disagreements_refuse_after_rehash() {
    for mode in 0..12 {
        let f = Fixture::new();
        match mode {
            0 => f.mutate("bullet/selected.json", |v| {
                v.as_array_mut().unwrap().reverse()
            }),
            1 => f.mutate("bullet/run/manifest.json", |v| {
                v["completed"].as_array_mut().unwrap().pop();
            }),
            2 => f.mutate("bullet/run/manifest.json", |v| {
                v["completed"][0]["outcome"] = json!("FAIL")
            }),
            3 => f.mutate("bullet/run/manifest.json", |v| {
                v["failures"] = json!(["cleanup unknown"])
            }),
            4 => raw(f.dir.path(), "bullet/exit-status", b"1\n"),
            5 => raw(
                f.dir.path(),
                "bullet/run/junit.xml",
                b"<testsuites tests=\"0\"/>",
            ),
            6 => f.mutate("bullet/inputs.json", |v| {
                v["hosted_subjects"]["run_attempt"] = json!("2")
            }),
            7 => f.mutate("receipt.json", |v| v["process_exit"] = json!(1)),
            8 => raw(f.dir.path(), "bullet/run/events.jsonl", b"{}\n"),
            9 => raw(
                f.dir.path(),
                "cli-build.jsonl",
                b"{\"reason\":\"build-finished\",\"success\":true}\n",
            ),
            _ => {
                let p = f.dir.path().join("bullet/run/events.jsonl");
                let mut rows = std::fs::read_to_string(&p)
                    .unwrap()
                    .lines()
                    .map(|l| parse(l.as_bytes()).unwrap())
                    .collect::<Vec<_>>();
                if mode == 10 {
                    rows.retain(|r| r["kind"] != "process_custody");
                } else {
                    for r in &mut rows {
                        if r["kind"] == "process_custody" {
                            r["data"]["reaped"] = json!(false);
                        }
                    }
                }
                for (i, r) in rows.iter_mut().enumerate() {
                    r["sequence"] = json!(i + 1);
                }
                std::fs::write(p, rows.iter().map(|r| format!("{r}\n")).collect::<String>())
                    .unwrap();
            }
        }
        f.reseal();
        assert!(f.verify().is_err(), "mode {mode}");
    }
}
#[test]
fn duplicate_json_and_path_traversal_refuse() {
    assert!(parse(br#"{"context":{"id":1,"id":2}}"#).is_err());
    for path in ["", "/absolute", "../up", "a/../b", "a\\b"] {
        assert!(relative(path).is_err());
    }
    let f = Fixture::new();
    f.mutate("receipt.json", |v| {
        v["artifacts"][0]["path"] = json!("../outside")
    });
    assert!(f.verify().is_err());
}

#[test]
fn deep_empty_directory_and_unclean_product_exit_refuse() {
    let f = Fixture::new();
    let mut p = f.dir.path().to_path_buf();
    for _ in 0..18 {
        p.push("nested");
    }
    std::fs::create_dir_all(p).unwrap();
    assert!(f.verify().is_err());
    for key in [
        "code",
        "signal",
        "terminal_restored",
        "termination_requested",
        "terminal_read_error",
    ] {
        let f = Fixture::new();
        let p = f.dir.path().join("bullet/run/events.jsonl");
        let mut rows = std::fs::read_to_string(&p)
            .unwrap()
            .lines()
            .map(|l| parse(l.as_bytes()).unwrap())
            .collect::<Vec<_>>();
        let row = rows
            .iter_mut()
            .find(|r| r["case"] == "six_withheld_http" && r["data"]["kind"] == "process_settlement")
            .unwrap();
        row["data"][key] = match key {
            "code" | "signal" => json!(9),
            "terminal_restored" => json!(false),
            "termination_requested" => json!(true),
            _ => json!("I/O error"),
        };
        std::fs::write(p, rows.iter().map(|r| format!("{r}\n")).collect::<String>()).unwrap();
        f.reseal();
        assert!(f.verify().is_err(), "{key}");
    }
}

#[test]
fn nonce_rejection_accepts_only_eof_or_owned_kill_settlement() {
    for (code, signal) in [(json!(1), Value::Null), (Value::Null, json!(9))] {
        let mut pending = cleanup::Cleanup::default();
        pending.observe("custody_after_page_failure",&json!({"kind":"preconstruction_cleanup","reaped":true,"code":code,"signal":signal})).unwrap();
        let mut c = cleanup::Cleanup::default();
        c.observe("custody_nonce_rejection",&json!({"kind":"process_settlement","reaped":true,"code":code,"signal":signal,"terminal_restored":false,"termination_requested":true,"terminal_read_error":null})).unwrap();
    }
    let mut c = cleanup::Cleanup::default();
    assert!(c.observe("custody_nonce_rejection",&json!({"kind":"process_settlement","reaped":true,"code":0,"signal":null,"terminal_restored":false,"termination_requested":true,"terminal_read_error":null})).is_err());
}
