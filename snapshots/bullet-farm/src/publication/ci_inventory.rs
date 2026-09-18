//! Reviewed nested job topology. Any workflow byte change requires renewed admission.
use std::collections::BTreeSet;

use serde::Serialize;

use super::{MEMBERS, Result, require};

pub(super) const CI: &str = ".github/workflows/ci.yml";
pub(super) const SCHEDULED: &str = ".github/workflows/scheduled.yml";
const UBUNTU: &str = "ubuntu-24.04";
const MATRIX: &str = "${{ matrix.os }}";
const PORTABLE: [&str; 2] = ["macos-15", "windows-2025"];

#[derive(Clone, Eq, PartialEq, Serialize)]
pub(super) struct Job {
    pub id: &'static str,
    pub needs: Vec<&'static str>,
    pub runner: &'static str,
    pub os: Vec<&'static str>,
    pub always: bool,
    pub timeout_minutes: u16,
}

#[derive(Clone, Serialize)]
pub(super) struct Workflow {
    pub member: &'static str,
    pub path: &'static str,
    pub scope: &'static str,
    pub sha256: String,
    pub jobs: Vec<Job>,
}

// Exact job budgets from the eight workflow digests admitted below.
fn reviewed_timeout(member: &str, path: &str, id: &str) -> Option<u16> {
    match (member, path, id) {
        ("bullet-farm", CI, "contract") => Some(25),
        ("bullet-farm", CI, "docs") => Some(35),
        ("bullet-farm", CI, "fast") => Some(20),
        ("bullet-farm", CI, "lint") => Some(25),
        ("bullet-farm", CI, "required") => Some(8),
        ("bullet-farm", CI, "security") => Some(25),
        ("bullet-farm", CI, "source_scan") => Some(10),
        ("bullet-farm", SCHEDULED, "advisory") => Some(20),
        ("bullet-farm", SCHEDULED, "audit") => Some(20),
        ("bullet-farm", SCHEDULED, "coverage") => Some(30),
        ("bullet-farm", SCHEDULED, "history") => Some(20),
        ("bullet-farm", SCHEDULED, "links") => Some(15),
        ("bullet-farm", SCHEDULED, "macos") => Some(25),
        ("bullet-farm", SCHEDULED, "source_scan") => Some(10),
        ("bullet-farm", SCHEDULED, "windows") => Some(30),
        ("bullet-git", CI, "contract") => Some(20),
        ("bullet-git", CI, "docs") => Some(15),
        ("bullet-git", CI, "fast") => Some(15),
        ("bullet-git", CI, "lint") => Some(15),
        ("bullet-git", CI, "required") => Some(5),
        ("bullet-git", CI, "security") => Some(15),
        ("bullet-git", CI, "source_scan") => Some(10),
        ("bullet-git", SCHEDULED, "advisory") => Some(15),
        ("bullet-git", SCHEDULED, "audit") => Some(15),
        ("bullet-git", SCHEDULED, "coverage") => Some(20),
        ("bullet-git", SCHEDULED, "history") => Some(15),
        ("bullet-git", SCHEDULED, "links") => Some(10),
        ("bullet-git", SCHEDULED, "macos") => Some(20),
        ("bullet-git", SCHEDULED, "source_scan") => Some(10),
        ("bullet-git", SCHEDULED, "windows") => Some(25),
        ("bullet-kernel", CI, "contract") => Some(20),
        ("bullet-kernel", CI, "docs") => Some(20),
        ("bullet-kernel", CI, "fast") => Some(20),
        ("bullet-kernel", CI, "lint") => Some(25),
        ("bullet-kernel", CI, "preflight") => Some(10),
        ("bullet-kernel", CI, "required") => Some(5),
        ("bullet-kernel", CI, "security") => Some(20),
        ("bullet-kernel", SCHEDULED, "advisories") => Some(20),
        ("bullet-kernel", SCHEDULED, "audit") => Some(15),
        ("bullet-kernel", SCHEDULED, "coverage") => Some(30),
        ("bullet-kernel", SCHEDULED, "history-secrets") => Some(15),
        ("bullet-kernel", SCHEDULED, "links") => Some(15),
        ("bullet-kernel", SCHEDULED, "portable-refusal") => Some(35),
        ("bullet-kernel", SCHEDULED, "source-admission") => Some(10),
        ("bullet-portal", CI, "contract") => Some(20),
        ("bullet-portal", CI, "docs") => Some(10),
        ("bullet-portal", CI, "fast") => Some(15),
        ("bullet-portal", CI, "lint") => Some(10),
        ("bullet-portal", CI, "required") => Some(5),
        ("bullet-portal", CI, "security") => Some(15),
        ("bullet-portal", SCHEDULED, "coverage") => Some(20),
        ("bullet-portal", SCHEDULED, "hygiene") => Some(20),
        ("bullet-portal", SCHEDULED, "portable") => Some(25),
        _ => None,
    }
}

fn job(member: &str, path: &str, id: &'static str, needs: &[&'static str]) -> Job {
    Job {
        id,
        needs: needs.to_vec(),
        runner: UBUNTU,
        os: Vec::new(),
        always: false,
        timeout_minutes: reviewed_timeout(member, path, id)
            .expect("compiled workflow job must have a reviewed timeout"),
    }
}

fn required(member: &str, preflight: Option<&'static str>) -> Vec<Job> {
    let gates = ["fast", "lint", "contract", "security", "docs"];
    let dependencies = preflight.into_iter().collect::<Vec<_>>();
    let mut jobs = preflight
        .into_iter()
        .map(|id| job(member, CI, id, &[]))
        .collect::<Vec<_>>();
    jobs.extend(gates.map(|id| job(member, CI, id, &dependencies)));
    let mut final_needs = dependencies;
    final_needs.extend(gates);
    let mut final_job = job(member, CI, "required", &final_needs);
    final_job.always = true;
    jobs.push(final_job);
    jobs
}

fn scheduled(member: &str) -> Vec<Job> {
    let mut jobs = match member {
        "bullet-farm" | "bullet-git" => {
            let mut jobs = vec![job(member, SCHEDULED, "source_scan", &[])];
            for id in [
                "history", "links", "advisory", "coverage", "macos", "windows", "audit",
            ] {
                let mut item = job(member, SCHEDULED, id, &["source_scan"]);
                item.always = member == "bullet-farm";
                item.runner = match id {
                    "macos" => PORTABLE[0],
                    "windows" => PORTABLE[1],
                    _ => UBUNTU,
                };
                jobs.push(item);
            }
            jobs
        }
        "bullet-kernel" => {
            let mut jobs = vec![job(member, SCHEDULED, "source-admission", &[])];
            jobs.extend(
                [
                    "links",
                    "advisories",
                    "coverage",
                    "history-secrets",
                    "portable-refusal",
                    "audit",
                ]
                .map(|id| job(member, SCHEDULED, id, &["source-admission"])),
            );
            jobs
        }
        _ => ["hygiene", "coverage", "portable"]
            .map(|id| job(member, SCHEDULED, id, &[]))
            .into(),
    };
    for item in &mut jobs {
        if matches!(item.id, "portable" | "portable-refusal") {
            item.runner = MATRIX;
            item.os = PORTABLE.into();
        }
    }
    jobs
}

pub(super) fn reviewed() -> Vec<Workflow> {
    // Pins cover entire YAML bytes, including steps, permissions, conditions and matrices.
    // The catalog is source authority for expected identities, never observed result data.
    let pins = [
        (
            "bullet-farm",
            "a6a4e78bc68635a5dff3f896b95a4940adecd9858505c5bb1781ecf73e41ab3e",
            "700c2b023ef5279c6e99f990575e3e7232b150b32a8ab8b1ef0004948bbc0cfc",
        ),
        (
            "bullet-git",
            "7e7f03bfc74e9fb87f0f46aae726a6a455bac0ae9b7208989653ea46325bf1ce",
            "144d2d040fd151c32027eb88a24238f6ca294122c4e75d1d767bc26cb2b8b6d7",
        ),
        (
            "bullet-kernel",
            "5ac1b2c114587970e152c0ea8f73a273fec4c2c1512bc2c82aa8acb32ebf48d8",
            "b703dc10751eb777347511bafae7b9d125752c5b75d590135fcf2156f698f0ff",
        ),
        (
            "bullet-portal",
            "6637ec9e3d1ba4d971c229ca0aa04dd022d8ec86b67188680a355ff81b595dc1",
            "83e80d3f981ef39d4f7a84fbf06b6ffb9d05254c491525eed6c8941bb697da22",
        ),
    ];
    let mut workflows = Vec::new();
    for (member, ci, scheduled_digest) in pins {
        let first = match member {
            "bullet-kernel" => Some("preflight"),
            "bullet-portal" => None,
            _ => Some("source_scan"),
        };
        workflows.push(Workflow {
            member,
            path: CI,
            scope: "REQUIRED",
            sha256: ci.into(),
            jobs: required(member, first),
        });
        workflows.push(Workflow {
            member,
            path: SCHEDULED,
            scope: "SCHEDULED",
            sha256: scheduled_digest.into(),
            jobs: scheduled(member),
        });
    }
    workflows
}

pub(super) fn canonical_catalog(workflows: &[Workflow]) -> Result<Vec<Workflow>> {
    let expected = MEMBERS
        .into_iter()
        .flat_map(|member| [(member, CI), (member, SCHEDULED)])
        .collect::<BTreeSet<_>>();
    let actual = workflows
        .iter()
        .map(|w| (w.member, w.path))
        .collect::<BTreeSet<_>>();
    require(
        workflows.len() == expected.len() && actual == expected,
        "PUBLICATION_CI_WORKFLOW_INVENTORY_INVALID",
    )?;
    let mut catalog = workflows.to_vec();
    catalog.sort_by_key(|w| (w.member, w.path));
    let reviewed = reviewed();
    for workflow in &mut catalog {
        require(
            workflow.scope
                == if workflow.path == CI {
                    "REQUIRED"
                } else {
                    "SCHEDULED"
                }
                && workflow.sha256.len() == 64
                && workflow
                    .sha256
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
            "PUBLICATION_CI_CATALOG_INVALID",
        )?;
        workflow.jobs.sort_by_key(|j| j.id);
        let ids = workflow.jobs.iter().map(|j| j.id).collect::<BTreeSet<_>>();
        require(
            ids.len() == workflow.jobs.len() && !ids.is_empty(),
            "PUBLICATION_CI_JOB_INVENTORY_INVALID",
        )?;
        for item in &mut workflow.jobs {
            require(
                !item.id.is_empty()
                    && item
                        .id
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b)),
                "PUBLICATION_CI_JOB_ID_INVALID",
            )?;
            require(
                Some(item.timeout_minutes)
                    == reviewed_timeout(workflow.member, workflow.path, item.id),
                "PUBLICATION_CI_TIMEOUT_INVALID",
            )?;
            item.needs.sort();
            require(
                item.needs.iter().collect::<BTreeSet<_>>().len() == item.needs.len()
                    && item
                        .needs
                        .iter()
                        .all(|id| ids.contains(id) && *id != item.id),
                "PUBLICATION_CI_DEPENDENCY_INVALID",
            )?;
            item.os.sort();
            require(
                (item.os.is_empty() && [UBUNTU, PORTABLE[0], PORTABLE[1]].contains(&item.runner))
                    || (item.runner == MATRIX && item.os == PORTABLE),
                "PUBLICATION_CI_MATRIX_INVALID",
            )?;
        }
        let mut reached = BTreeSet::new();
        loop {
            let prior = reached.len();
            for item in &workflow.jobs {
                if item.needs.iter().all(|id| reached.contains(id)) {
                    reached.insert(item.id);
                }
            }
            if reached.len() == prior {
                break;
            }
        }
        require(reached == ids, "PUBLICATION_CI_DEPENDENCY_CYCLE")?;
        let mut expected_jobs = reviewed
            .iter()
            .find(|w| w.member == workflow.member && w.path == workflow.path)
            .expect("workflow inventory validated above")
            .jobs
            .clone();
        expected_jobs.sort_by_key(|j| j.id);
        for item in &mut expected_jobs {
            item.needs.sort();
            item.os.sort();
        }
        require(
            ids == expected_jobs.iter().map(|j| j.id).collect(),
            "PUBLICATION_CI_JOB_INVENTORY_INVALID",
        )?;
        require(
            workflow.jobs == expected_jobs,
            "PUBLICATION_CI_JOB_TOPOLOGY_INVALID",
        )?;
    }
    Ok(catalog)
}
