//! Re-open portable hosted artifacts with this checkout's trusted validator.
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Deserializer};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

mod cleanup;

const MAX_FILE: u64 = 512 * 1024 * 1024;
const MAX_JSON: u64 = 32 * 1024 * 1024;

struct Strict(Value);
impl<'de> Deserialize<'de> for Strict {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Strict;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("strict JSON")
            }
            fn visit_bool<E: serde::de::Error>(self, v: bool) -> std::result::Result<Strict, E> {
                Ok(Strict(v.into()))
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> std::result::Result<Strict, E> {
                Ok(Strict(v.into()))
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> std::result::Result<Strict, E> {
                Ok(Strict(v.into()))
            }
            fn visit_f64<E: serde::de::Error>(self, v: f64) -> std::result::Result<Strict, E> {
                serde_json::Number::from_f64(v)
                    .map(|n| Strict(n.into()))
                    .ok_or_else(|| E::custom("nonfinite number"))
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> std::result::Result<Strict, E> {
                Ok(Strict(v.into()))
            }
            fn visit_none<E: serde::de::Error>(self) -> std::result::Result<Strict, E> {
                Ok(Strict(Value::Null))
            }
            fn visit_unit<E: serde::de::Error>(self) -> std::result::Result<Strict, E> {
                Ok(Strict(Value::Null))
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Strict, A::Error> {
                let mut out = Vec::new();
                while let Some(Strict(v)) = a.next_element()? {
                    out.push(v);
                }
                Ok(Strict(out.into()))
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Strict, A::Error> {
                let mut out = serde_json::Map::new();
                while let Some((k, Strict(v))) = a.next_entry::<String, Strict>()? {
                    if out.insert(k, v).is_some() {
                        return Err(serde::de::Error::custom("duplicate key"));
                    }
                }
                Ok(Strict(out.into()))
            }
        }
        d.deserialize_any(Visitor)
    }
}
fn parse(bytes: &[u8]) -> Result<Value> {
    Ok(serde_json::from_slice::<Strict>(bytes)?.0)
}
fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v[key]
        .as_str()
        .with_context(|| format!("EVIDENCE_FIELD_MISSING: {key}"))
}
fn relative(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty()
            && Path::new(value)
                .components()
                .all(|c| matches!(c, Component::Normal(_)))
            && !value.contains('\\'),
        "EVIDENCE_PATH_INVALID"
    );
    Ok(())
}
fn bytes(root: &Path, name: &str, limit: u64) -> Result<Vec<u8>> {
    relative(name)?;
    let mut p = root.to_path_buf();
    for component in Path::new(name).components() {
        p.push(component);
        ensure!(
            !p.symlink_metadata()?.file_type().is_symlink(),
            "EVIDENCE_SYMLINK"
        );
    }
    let meta = p.metadata()?;
    ensure!(meta.is_file() && meta.len() <= limit, "EVIDENCE_FILE_BOUND");
    let result = std::fs::read(p)?;
    ensure!(result.len() as u64 <= limit, "EVIDENCE_FILE_BOUND");
    Ok(result)
}
fn document(root: &Path, name: &str) -> Result<Value> {
    parse(&bytes(root, name, MAX_JSON)?)
}
fn inventory(root: &Path) -> Result<BTreeSet<String>> {
    fn walk(
        root: &Path,
        at: &Path,
        files: &mut BTreeSet<String>,
        total: &mut u64,
        directories: &mut usize,
        depth: usize,
    ) -> Result<()> {
        *directories += 1;
        ensure!(
            *directories <= 1024 && depth <= 16,
            "EVIDENCE_DIRECTORY_BOUND"
        );
        for entry in std::fs::read_dir(at)? {
            let entry = entry?;
            let meta = entry.file_type()?;
            let p = entry.path();
            ensure!(!meta.is_symlink(), "EVIDENCE_SYMLINK");
            if meta.is_dir() {
                walk(root, &p, files, total, directories, depth + 1)?;
            } else {
                ensure!(meta.is_file(), "EVIDENCE_SPECIAL_FILE");
                let size = entry.metadata()?.len();
                *total = total.checked_add(size).context("EVIDENCE_TOTAL_OVERFLOW")?;
                ensure!(
                    size <= MAX_FILE && *total <= 2 * 1024 * 1024 * 1024,
                    "EVIDENCE_FILE_BOUND"
                );
                files.insert(
                    p.strip_prefix(root)?
                        .to_str()
                        .context("EVIDENCE_PATH_UTF8")?
                        .into(),
                );
                ensure!(files.len() <= 1024, "EVIDENCE_INVENTORY_BOUND");
            }
        }
        Ok(())
    }
    let mut out = BTreeSet::new();
    walk(root, root, &mut out, &mut 0, &mut 0, 0)?;
    Ok(out)
}
fn artifacts(root: &Path, rows: &Value, exclude: &str) -> Result<()> {
    let rows = rows.as_array().context("EVIDENCE_ARTIFACTS_REQUIRED")?;
    ensure!(
        !rows.is_empty() && rows.len() <= 1024,
        "EVIDENCE_ARTIFACTS_BOUND"
    );
    let mut declared = BTreeSet::new();
    for row in rows {
        let name = text(row, "path")?;
        relative(name)?;
        ensure!(
            name != exclude && declared.insert(name.to_owned()),
            "EVIDENCE_ARTIFACT_DUPLICATE"
        );
        bytes(root, name, MAX_FILE)?;
        ensure!(
            crate::evidence::hash(&root.join(name))? == text(row, "sha256")?,
            "EVIDENCE_ARTIFACT_HASH"
        );
    }
    declared.insert(exclude.into());
    ensure!(inventory(root)? == declared, "EVIDENCE_ARTIFACT_INVENTORY");
    Ok(())
}
fn compiler(root: &Path, name: &str, targets: &[(&str, String)]) -> Result<()> {
    let data = bytes(root, name, MAX_JSON)?;
    let string = std::str::from_utf8(&data)?;
    let mut observed = BTreeMap::<String, usize>::new();
    let mut finished = 0;
    for line in string.lines().filter(|line| !line.is_empty()) {
        let row = parse(line.as_bytes())?;
        if row["reason"] == "build-finished" {
            ensure!(row["success"] == true, "EVIDENCE_BUILD_FAILED");
            finished += 1;
        }
        if row["reason"] == "compiler-artifact" {
            for (target, exe) in targets {
                if row["target"]["name"] == *target
                    && row["target"]["kind"]
                        .as_array()
                        .is_some_and(|k| k.contains(&json!("bin")))
                {
                    ensure!(row["executable"] == *exe, "EVIDENCE_BUILD_EXECUTABLE");
                    *observed.entry(target.to_string()).or_default() += 1;
                }
            }
        }
    }
    ensure!(
        finished == 1 && targets.iter().all(|(n, _)| observed.get(*n) == Some(&1)),
        "EVIDENCE_BUILD_INCOMPLETE"
    );
    Ok(())
}
fn source(root: &Path, subjects: &Value, source_root: &Path) -> Result<()> {
    let rows = subjects["runtime_observed_source"]
        .as_array()
        .context("EVIDENCE_SOURCE_INVENTORY")?;
    let mut names = BTreeSet::new();
    for row in rows {
        let name = text(row, "path")?;
        relative(name)?;
        ensure!(names.insert(name.to_owned()), "EVIDENCE_SOURCE_DUPLICATE");
        ensure!(
            crate::evidence::hash(&source_root.join(name))? == text(row, "sha256")?,
            "EVIDENCE_SOURCE_CHANGED"
        );
    }
    // The producer and consumer use the exact source inventory emitted by this version.
    let expected = crate::evidence::SOURCE_NAMES
        .iter()
        .map(|s| s.to_string())
        .collect::<BTreeSet<_>>();
    ensure!(names == expected, "EVIDENCE_SOURCE_INVENTORY");
    let _ = root;
    Ok(())
}
fn run(
    root: &Path,
    alias: &str,
    receipt: &Value,
    expected: &Value,
    source_root: &Path,
) -> Result<()> {
    let at = root.join(alias);
    let manifest = document(&at, "run/manifest.json")?;
    ensure!(
        bytes(&at, "exit-status", 32)? == b"0\n",
        "EVIDENCE_PROCESS_FAILED"
    );
    let selected: Vec<String> = serde_json::from_value(document(&at, "selected.json")?)?;
    let current = crate::custody::IDENTITIES
        .iter()
        .chain(crate::cases::IDENTITIES.iter())
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    ensure!(selected == current, "EVIDENCE_SELECTION_CHANGED");
    ensure!(
        manifest["schema"] == 1
            && manifest["evidence_class"] == "COMPONENT_PROOF"
            && manifest["outcome"] == "PASS"
            && manifest["installed_authenticated"] == false
            && manifest["release_eligible"] == false
            && manifest["failures"] == json!([])
            && manifest["selected"] == json!(selected),
        "EVIDENCE_MANIFEST_FAILED"
    );
    let rows = manifest["completed"]
        .as_array()
        .context("EVIDENCE_COMPLETION_REQUIRED")?;
    ensure!(
        rows.iter().map(|r| r["id"].as_str()).collect::<Vec<_>>()
            == selected
                .iter()
                .map(|s| Some(s.as_str()))
                .collect::<Vec<_>>()
            && rows
                .iter()
                .all(|r| r["outcome"] == "PASS" && r["error"].is_null()),
        "EVIDENCE_COMPLETION_FAILED"
    );
    let junit = crate::junit::render(&selected, rows, &[])?;
    ensure!(
        bytes(&at, "run/junit.xml", MAX_JSON)? == junit.as_bytes(),
        "EVIDENCE_JUNIT_DISAGREEMENT"
    );
    artifacts(&at.join("run"), &manifest["artifacts"], "manifest.json")?;
    let subjects = &manifest["subjects"];
    let executable = &receipt["executables"][alias];
    ensure!(
        subjects["bullet"]
            == json!({"path":executable["original_path"],"sha256":executable["sha256"]}),
        "EVIDENCE_ALIAS_SUBJECT"
    );
    ensure!(
        document(&at, "run/subjects.json")? == *subjects,
        "EVIDENCE_SUBJECT_DISAGREEMENT"
    );
    source(&at, subjects, source_root)?;
    let harness_hash = crate::evidence::hash(&at.join("harness"))?;
    let original = text(executable, "original_path")?;
    let stage = Path::new(original)
        .parent()
        .and_then(Path::parent)
        .context("EVIDENCE_STAGE_PATH")?;
    ensure!(
        subjects["harness"]
            == json!({"path":stage.join(alias).join("harness"),"sha256":harness_hash}),
        "EVIDENCE_HARNESS_SUBJECT"
    );
    let input = document(&at, "inputs.json")?;
    ensure!(
        input["profile"] == "hosted-component"
            && input["hosted_subjects"] == *expected
            && input["bullet"] == subjects["bullet"],
        "EVIDENCE_INPUT_DISAGREEMENT"
    );
    compiler(
        &at,
        "build.jsonl",
        &[(
            "bullet-tuiwright-qualification",
            format!(
                "{}/release/bullet-tuiwright-qualification",
                text(receipt, "build_target")?
            ),
        )],
    )?;
    let events = bytes(&at, "run/events.jsonl", MAX_JSON)?;
    let mut profiles = 0;
    let mut cleanup = cleanup::Cleanup::default();
    let mut completed = Vec::new();
    let mut sequence = 0;
    for line in std::str::from_utf8(&events)?.lines() {
        let e = parse(line.as_bytes())?;
        sequence += 1;
        ensure!(
            e["sequence"].as_u64() == Some(sequence),
            "EVIDENCE_EVENT_SEQUENCE"
        );
        if e["kind"] == "admitted_component_context" {
            profiles += 1;
            let mut profile = expected.clone();
            profile["profile"] = json!("hosted-component");
            profile["profile_source_sha256"] =
                json!(crate::evidence::hash(&source_root.join("src/profile.rs"))?);
            ensure!(
                e["case"] == "profile" && e["data"] == profile,
                "EVIDENCE_PROFILE_EVENT"
            );
        }
        if e["kind"] == "completed" {
            ensure!(e["case"] == e["data"]["id"], "EVIDENCE_EVENT_SUBJECT");
            completed.push(e["data"].clone());
        }
        if e["kind"] == "process_custody" {
            cleanup.observe(text(&e, "case")?, &e["data"])?;
        }
    }
    cleanup.finish(&selected)?;
    ensure!(
        profiles == 1 && completed == *rows,
        "EVIDENCE_EVENT_COMPLETION"
    );
    Ok(())
}
fn verify(root: &Path, expected: &Value, source_root: &Path) -> Result<()> {
    ensure!(
        root.is_dir() && !root.symlink_metadata()?.file_type().is_symlink(),
        "EVIDENCE_ROOT_INVALID"
    );
    let r = document(root, "receipt.json")?;
    let keys = r
        .as_object()
        .context("EVIDENCE_RECEIPT_OBJECT")?
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    ensure!(
        keys == [
            "schema",
            "context",
            "process_exit",
            "executables",
            "build_target",
            "artifacts"
        ]
        .into_iter()
        .collect(),
        "EVIDENCE_RECEIPT_FIELDS"
    );
    ensure!(
        r["schema"] == "bullet.operator-tui.hosted.v1"
            && r["context"] == *expected
            && r["process_exit"] == 0,
        "EVIDENCE_RECEIPT_SUBJECT"
    );
    artifacts(root, &r["artifacts"], "receipt.json")?;
    let target = text(&r, "build_target")?;
    ensure!(Path::new(target).is_absolute(), "EVIDENCE_BUILD_TARGET");
    for alias in ["bullet", "bulletfarm"] {
        let exe = &r["executables"][alias];
        let name = format!("bin/{alias}");
        ensure!(
            exe["path"] == name
                && Path::new(text(exe, "original_path")?).is_absolute()
                && text(exe, "original_path")?.ends_with(&format!("/bin/{alias}")),
            "EVIDENCE_ALIAS_PATH"
        );
        ensure!(
            crate::evidence::hash(&root.join(&name))? == text(exe, "sha256")?,
            "EVIDENCE_ALIAS_HASH"
        );
        ensure!(
            bytes(root, &name, MAX_FILE)?.starts_with(b"\x7fELF"),
            "EVIDENCE_ALIAS_NOT_NATIVE"
        );
        run(root, alias, &r, expected, source_root)?;
    }
    compiler(
        root,
        "cli-build.jsonl",
        &[
            ("bullet", format!("{target}/debug/bullet")),
            ("bulletfarm", format!("{target}/debug/bulletfarm")),
        ],
    )?;
    Ok(())
}
pub fn cli(args: &[String]) -> Result<()> {
    ensure!(
        args.len() == 1,
        "USAGE: --validate-hosted /artifact/directory"
    );
    let source = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = source.join("../../..").canonicalize()?;
    let env = |name: &str| {
        std::env::var(name).with_context(|| format!("EVIDENCE_CONTEXT_MISSING: {name}"))
    };
    let git = |args: &[&str]| -> Result<Vec<u8>> {
        let o = std::process::Command::new("git")
            .current_dir(&root)
            .args(args)
            .output()?;
        ensure!(o.status.success(), "EVIDENCE_GIT_FAILED");
        Ok(o.stdout)
    };
    let commit = env("GITHUB_SHA")?;
    let workflow_commit = env("GITHUB_WORKFLOW_SHA")?;
    ensure!(
        String::from_utf8(git(&["rev-parse", "HEAD"])?)?.trim() == commit,
        "EVIDENCE_CHECKOUT_CHANGED"
    );
    let workflow = std::fs::read(root.join(".github/workflows/ci.yml"))?;
    ensure!(
        git(&[
            "show",
            &format!("{workflow_commit}:.github/workflows/ci.yml")
        ])? == workflow,
        "EVIDENCE_WORKFLOW_CHANGED"
    );
    let expected = json!({"commit":commit,"tree":String::from_utf8(git(&["rev-parse","HEAD^{tree}"])?)?.trim(),
        "workflow_sha256":crate::evidence::hash(&root.join(".github/workflows/ci.yml"))?,"workflow_commit":workflow_commit,
        "workflow_ref":env("GITHUB_WORKFLOW_REF")?,"repository":env("GITHUB_REPOSITORY")?,"event":env("GITHUB_EVENT_NAME")?,
        "run_id":env("GITHUB_RUN_ID")?,"run_attempt":env("GITHUB_RUN_ATTEMPT")?,"workspace":env("GITHUB_WORKSPACE")?,"job":"operator-tui"});
    ensure!(
        bytes(Path::new(&args[0]), "source-tree.txt", MAX_JSON)?
            == git(&["ls-tree", "-r", "HEAD"])?,
        "EVIDENCE_RAW_SOURCE_TREE"
    );
    verify(Path::new(&args[0]), &expected, source)?;
    println!("OPERATOR_TUI_EVIDENCE_PASS: exact dual-alias component reports; no raw transcript/live credit");
    Ok(())
}
#[cfg(test)]
mod tests;
