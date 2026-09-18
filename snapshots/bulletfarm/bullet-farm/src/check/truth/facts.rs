//! Machine facts consumed by the release-truth projection. Every value is read
//! from a file or read-only Git query; none of it is release authority and no
//! wall clock is consulted.

use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use bullet_wire::decode_unique_value;
use serde_json::Value;

use crate::{
    check::{executor, model::CHECK_REPORT_SCHEMA_VERSION},
    doctor::{self, LockSummary},
};

pub(super) const REPOSITORIES: &[&str] = &[
    "bullet-farm",
    "bullet-kernel",
    "bullet-git",
    "bullet-portal",
];
pub(super) const RELEASE_INDEX: &str = "docs/release.md";
pub(super) const PRODUCT_GAP_REGISTER: &str = "docs/assurance/product-gaps.md";
pub(super) const MECHANICAL_TIERS: &[(&str, &str, &str)] = &[
    ("fast", "FAST", ".bullet-family/check-fast.json"),
    ("required", "REQUIRED", ".bullet-family/check-required.json"),
];
const MAX_INPUT_BYTES: u64 = 1024 * 1024;
const PORTABLE_NOTE: &str = "not read (portable variant)";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::check) enum Variant {
    /// Machine-local subjects, absolute paths, and mtimes are included.
    Live,
    /// Reproducible from a fresh hub-only checkout; machine-local inputs are excluded.
    Portable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Facts {
    pub variant: Variant,
    pub hub_root: Option<String>,
    pub subjects: Vec<SubjectFact>,
    pub hub_head_committed_at: Option<String>,
    pub lock: Result<LockSummary, String>,
    pub release_index: ReleaseIndexFact,
    pub register: Register,
    pub mechanical: Vec<MechanicalFact>,
    pub inputs: Vec<InputFact>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SubjectFact {
    pub name: &'static str,
    pub commit: Option<String>,
    pub tree: Option<String>,
    pub checkout: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct ReleaseIndexFact {
    pub status: Option<String>,
    pub last_reviewed: Option<String>,
}

/// The product gap register's crosswalk, read from `docs/assurance/product-gaps.md`.
/// Only the `release.*` crosswalk rows and the G-id rows are bound; prose edits
/// elsewhere in the register never move this page.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Register {
    Absent,
    Unparsed(String),
    Read(RegisterTable),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct RegisterTable {
    /// `(gate id, G-ids)` crosswalk rows, sorted by gate id.
    pub gates: Vec<(String, Vec<String>)>,
    /// G-ids of the gap table, in document order.
    pub gap_ids: Vec<String>,
    /// blake3 over the canonical crosswalk rows and G-id list only.
    pub identity: String,
}

impl Register {
    pub(super) fn identity(&self) -> String {
        match self {
            Self::Absent => "absent".to_owned(),
            Self::Unparsed(reason) => format!("unparsed ({reason})"),
            Self::Read(table) => format!(
                "blake3:{} over the {} `release.*` crosswalk rows + G-id list only",
                table.identity,
                table.gates.len()
            ),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct MechanicalFact {
    pub tier: &'static str,
    pub path: &'static str,
    pub outcome: Mechanical,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Mechanical {
    NotRead,
    Absent,
    Unreadable(String),
    Stale {
        status: String,
        gates: usize,
        mismatch: String,
    },
    Fresh {
        status: String,
        gates: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct InputFact {
    pub label: &'static str,
    pub path: &'static str,
    pub identity: String,
    pub mtime: Option<String>,
}

pub(super) fn gather(hub: &Path, variant: Variant) -> Facts {
    let live = variant == Variant::Live;
    let subjects = REPOSITORIES
        .iter()
        .map(|&name| {
            if live {
                subject(name, &repository_path(hub, name))
            } else {
                SubjectFact {
                    name,
                    commit: None,
                    tree: None,
                    checkout: PORTABLE_NOTE.to_owned(),
                }
            }
        })
        .collect::<Vec<_>>();
    let mechanical = MECHANICAL_TIERS
        .iter()
        .map(|&(tier, expected, path)| MechanicalFact {
            tier,
            path,
            outcome: if live {
                mechanical(&hub.join(path), expected, &subjects)
            } else {
                Mechanical::NotRead
            },
        })
        .collect();
    let release_index = release_index(&hub.join(RELEASE_INDEX));
    let register = register(&hub.join(PRODUCT_GAP_REGISTER));
    let mut inputs = vec![
        input("family lock", "family.lock", hub, live),
        input("hub manifest", "repos.manifest.toml", hub, live),
        // Only the `Last reviewed:` line is consumed, so only that line is bound;
        // prose edits elsewhere in the release index never move this page.
        InputFact {
            identity: release_index.last_reviewed.as_deref().map_or_else(
                || "no `Last reviewed:` line".to_owned(),
                |date| format!("`Last reviewed: {date}` line only"),
            ),
            ..input("release index", RELEASE_INDEX, hub, live)
        },
        InputFact {
            identity: register.identity(),
            ..input("product-gap register", PRODUCT_GAP_REGISTER, hub, live)
        },
    ];
    for &(tier, _, path) in MECHANICAL_TIERS {
        let label = if tier == "fast" {
            "fast check report"
        } else {
            "required check report"
        };
        inputs.push(input(label, path, hub, live));
    }
    Facts {
        variant,
        hub_root: live.then(|| hub.to_string_lossy().into_owned()),
        hub_head_committed_at: live
            .then(|| executor::git(hub, &["log", "-1", "--format=%cI"]).ok())
            .flatten(),
        subjects,
        lock: doctor::lock_summary(hub).map_err(|error| error.to_string()),
        release_index,
        register,
        mechanical,
        inputs,
    }
}

fn repository_path(hub: &Path, name: &str) -> PathBuf {
    if name == "bullet-farm" {
        hub.to_path_buf()
    } else {
        hub.parent()
            .map_or_else(|| hub.join(name), |family| family.join(name))
    }
}

fn subject(name: &'static str, repository: &Path) -> SubjectFact {
    if !repository.join(".git").exists() {
        return SubjectFact {
            name,
            commit: None,
            tree: None,
            checkout: "absent".to_owned(),
        };
    }
    let identity =
        executor::git(repository, &["rev-parse", "--show-object-format"]).and_then(|algorithm| {
            let head = executor::git(repository, &["rev-parse", "--verify", "HEAD"])?;
            let tree = executor::git(repository, &["rev-parse", "--verify", "HEAD^{tree}"])?;
            Ok((format!("{algorithm}:{head}"), format!("{algorithm}:{tree}")))
        });
    let (commit, tree) = match identity {
        Ok((commit, tree)) => (Some(commit), Some(tree)),
        Err(error) => {
            return SubjectFact {
                name,
                commit: None,
                tree: None,
                checkout: format!("UNKNOWN ({error})"),
            };
        }
    };
    let checkout = match executor::git(
        repository,
        &["status", "--porcelain=v2", "--untracked-files=all"],
    ) {
        Ok(status) if status.is_empty() => "clean".to_owned(),
        Ok(status) => format!("dirty ({} entries)", status.lines().count()),
        Err(error) => format!("UNKNOWN ({error})"),
    };
    SubjectFact {
        name,
        commit,
        tree,
        checkout,
    }
}

fn release_index(path: &Path) -> ReleaseIndexFact {
    let Some(text) = read_bounded(path).and_then(|bytes| String::from_utf8(bytes).ok()) else {
        return ReleaseIndexFact::default();
    };
    let field = |prefix: &str| {
        text.lines()
            .find_map(|line| line.strip_prefix(prefix))
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
    };
    ReleaseIndexFact {
        status: field("Status:"),
        last_reviewed: field("Last reviewed:"),
    }
}

fn register(path: &Path) -> Register {
    match read_bounded(path).and_then(|bytes| String::from_utf8(bytes).ok()) {
        Some(text) => parse_register(&text),
        None => Register::Absent,
    }
}

fn is_gap_id(cell: &str) -> bool {
    cell.strip_prefix('G').is_some_and(|digits| {
        !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
    })
}

/// Parse the register's `release.<gate>` crosswalk rows (gate, G-ids, class)
/// and its `G<n>` gap rows. Everything else in the document is ignored.
pub(super) fn parse_register(text: &str) -> Register {
    let mut gates = Vec::new();
    let mut gap_ids = Vec::new();
    for line in text.lines() {
        let cells = line
            .trim()
            .strip_prefix('|')
            .and_then(|rest| rest.strip_suffix('|'))
            .map(|inner| inner.split('|').map(str::trim).collect::<Vec<_>>())
            .unwrap_or_default();
        let Some(&first) = cells.first() else {
            continue;
        };
        if let Some(gate) = first
            .strip_prefix("`release.")
            .and_then(|rest| rest.strip_suffix('`'))
        {
            let Some(&gaps) = cells.get(1) else {
                return Register::Unparsed(format!("crosswalk row without gaps: {line}"));
            };
            let ids = gaps.split(',').map(str::trim).collect::<Vec<_>>();
            if ids.is_empty() || !ids.iter().all(|id| is_gap_id(id)) {
                return Register::Unparsed(format!("crosswalk row with malformed gaps: {line}"));
            }
            gates.push((
                format!("release.{gate}"),
                ids.into_iter().map(str::to_owned).collect::<Vec<_>>(),
            ));
        } else if is_gap_id(first) {
            gap_ids.push(first.to_owned());
        }
    }
    if gates.is_empty() {
        return Register::Unparsed("no `release.*` crosswalk rows".to_owned());
    }
    if gap_ids.is_empty() {
        return Register::Unparsed("no G-id rows".to_owned());
    }
    gates.sort();
    let mut canonical = format!("gaps={}\n", gap_ids.join(","));
    for (gate, ids) in &gates {
        canonical.push_str(&format!("{gate}={}\n", ids.join(",")));
    }
    Register::Read(RegisterTable {
        gates,
        gap_ids,
        identity: blake3::hash(canonical.as_bytes()).to_hex().to_string(),
    })
}

fn mechanical(path: &Path, expected_tier: &str, subjects: &[SubjectFact]) -> Mechanical {
    if fs::symlink_metadata(path).is_err() {
        return Mechanical::Absent;
    }
    let Some(bytes) = read_bounded(path) else {
        return Mechanical::Unreadable("not a regular file within 1 MiB".to_owned());
    };
    let report: Value = match decode_unique_value(&bytes) {
        Ok(report) => report,
        Err(error) => return Mechanical::Unreadable(error.to_string()),
    };
    if report["schema_version"] != CHECK_REPORT_SCHEMA_VERSION
        || report["command"] != "check"
        || report["tier"] != expected_tier
    {
        return Mechanical::Unreadable(format!(
            "not a schema {CHECK_REPORT_SCHEMA_VERSION} {expected_tier} check report"
        ));
    }
    let (Some(status), Some(gates)) = (report["status"].as_str(), report["gates"].as_array())
    else {
        return Mechanical::Unreadable("status or gates are missing".to_owned());
    };
    let mismatch = gates.iter().find_map(|gate| {
        gate["subjects"].as_array()?.iter().find_map(|subject| {
            let name = subject["repository"].as_str()?;
            let recorded = subject["commit_oid"].as_str()?;
            let live = subjects
                .iter()
                .find(|subject| subject.name == name)
                .and_then(|subject| subject.commit.as_deref());
            (live != Some(recorded)).then(|| {
                format!(
                    "{name} recorded {recorded}, HEAD {}",
                    live.unwrap_or("UNKNOWN")
                )
            })
        })
    });
    match mismatch {
        Some(mismatch) => Mechanical::Stale {
            status: status.to_owned(),
            gates: gates.len(),
            mismatch,
        },
        None => Mechanical::Fresh {
            status: status.to_owned(),
            gates: gates.len(),
        },
    }
}

fn input(label: &'static str, relative: &'static str, hub: &Path, live: bool) -> InputFact {
    let path = hub.join(relative);
    let Some(bytes) = read_bounded(&path) else {
        return InputFact {
            label,
            path: relative,
            identity: "absent".to_owned(),
            mtime: None,
        };
    };
    let mtime = live
        .then(|| {
            fs::metadata(&path)
                .ok()?
                .modified()
                .ok()?
                .duration_since(UNIX_EPOCH)
                .ok()
                .map(|since| since.as_secs().to_string())
        })
        .flatten();
    InputFact {
        label,
        path: relative,
        identity: format!("blake3:{}", blake3::hash(&bytes).to_hex()),
        mtime,
    }
}

fn read_bounded(path: &Path) -> Option<Vec<u8>> {
    let metadata = fs::symlink_metadata(path).ok()?;
    if !metadata.file_type().is_file() || metadata.len() > MAX_INPUT_BYTES {
        return None;
    }
    fs::read(path).ok()
}
