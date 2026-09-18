//! Reviewed nested job topology. Any workflow byte change requires renewed admission.
use std::collections::BTreeSet;

use serde::Serialize;

use super::{MEMBERS, Result, require};

pub(super) const CI: &str = ".github/workflows/ci.yml";
pub(super) const SCHEDULED: &str = ".github/workflows/scheduled.yml";
const UBUNTU: &str = "ubuntu-24.04";
const MATRIX: &str = "${{ matrix.os }}";
const PORTABLE: [&str; 2] = ["macos-15", "windows-2025"];

#[derive(Clone, Serialize)]
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
            "52c7841a5a4c6ab4a2321fce6b18755552da49d5b0e065c7be4925ad0880a1fb",
            "e284e768a81d1ff83a5e453c3b81fbd3df0762e172367378af845e87502b422f",
        ),
        (
            "bullet-git",
            "fd628b973138e9a27a7c2e7c1a6d700fc6675647b0049fd0ba14e84dfcb64f82",
            "7a9a8db8573a90467c5c62783cfb016f8e7ba31e3449fe81c4780d429a03a9fc",
        ),
        (
            "bullet-kernel",
            "5e45f68e8a682b8f474ff73bfa0b35545af3a77f3a13534fa4f462c6e1e1451d",
            "b703dc10751eb777347511bafae7b9d125752c5b75d590135fcf2156f698f0ff",
        ),
        (
            "bullet-portal",
            "41d796d45036e41f4f8999935c9cacde2c50c3f051dca49cb4ec6221c5fd6aeb",
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
        workflows.len() == 8 && actual == expected,
        "PUBLICATION_CI_WORKFLOW_INVENTORY_INVALID",
    )?;
    let mut catalog = workflows.to_vec();
    catalog.sort_by_key(|w| (w.member, w.path));
    let mut counts = (0, 0, 0);
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
            counts.0 += usize::from(workflow.scope == "REQUIRED");
            counts.1 += usize::from(workflow.scope == "SCHEDULED");
            counts.2 += item.os.len().max(1);
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
    }
    require(
        counts == (27, 26, 55),
        "PUBLICATION_CI_JOB_INVENTORY_INVALID",
    )?;
    Ok(catalog)
}
