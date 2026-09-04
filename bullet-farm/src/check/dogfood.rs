//! Diagnostic dogfood board. Never authoritative. Never a release receipt.

use serde::Serialize;
use std::path::{Path, PathBuf};

use super::{executor, profiles::ReleaseProfile};
use crate::{
    coord::{ClaimState, ClaimSummary, CoordError, CoordStore, discover_family_root},
    scorecard,
};

const SCHEMA_VERSION: u32 = 1;
const DOGFOOD_BINDING_ENV: &str = "BULLET_DOGFOOD_BINDING";
const DOGFOOD_POLICY_ENV: &str = "BULLET_DOGFOOD_POLICY";
const W0_REPOS: &[&str] = &[
    "bullet-farm",
    "bullet-kernel",
    "bullet-git",
    "bullet-portal",
];

/// Purpose-separated operational observation. Not a release receipt kind.
#[derive(Serialize)]
pub(super) struct DogfoodRunObservationV0 {
    kind: &'static str,
    schema_version: &'static str,
    release_eligible: bool,
    transaction_eligible: bool,
    live_eligible: bool,
    profile_eligible: bool,
    custody: &'static str,
}

impl DogfoodRunObservationV0 {
    pub(super) const KIND: &'static str = "DOGFOOD_RUN";

    fn template() -> Self {
        Self {
            kind: Self::KIND,
            schema_version: "v0",
            release_eligible: false,
            transaction_eligible: false,
            live_eligible: false,
            profile_eligible: false,
            custody: "OPERATOR_LOCAL_KEY_SAME_UID",
        }
    }
}

#[derive(Clone, Copy)]
struct LeftoverLane {
    id: &'static str,
    lane: &'static str,
    repo: &'static str,
    paths: &'static [&'static str],
}

const LEFTOVERS: &[LeftoverLane] = &[
    LeftoverLane {
        id: "L-06",
        lane: "HUB-WIRE-FUZZ",
        repo: "bullet-farm",
        paths: &["crates/bullet-wire/fuzz"],
    },
    LeftoverLane {
        id: "L-15",
        lane: "GIT-GC-RETENTION",
        repo: "bullet-git",
        paths: &[
            "crates/bullet-git-workspace/src/gc.rs",
            "crates/bullet-git-workspace/tests/gc_safety.rs",
        ],
    },
    LeftoverLane {
        id: "L-30",
        lane: "KERNEL-COMMAND-DISPATCH",
        repo: "bullet-kernel",
        paths: &[
            "apps/bullet-farmd/src/dispatch.rs",
            "apps/bullet-farmd/src/commands.rs",
        ],
    },
    LeftoverLane {
        id: "L-59",
        lane: "PORTAL-SHIFT-BRIEF-DEFAULT",
        repo: "bullet-portal",
        paths: &["src/surfaces.ts", "src/App.test.tsx"],
    },
    LeftoverLane {
        id: "L-64",
        lane: "KERNEL-SAGA-QUARANTINE",
        repo: "bullet-kernel",
        paths: &["crates/runner/src/saga"],
    },
    LeftoverLane {
        id: "S-01",
        lane: "INDEPENDENT-SCORER",
        repo: "bullet-farm",
        paths: &["docs/assurance/scorecard.generated.md"],
    },
];

#[derive(Serialize)]
struct Board {
    schema_version: u32,
    kind: &'static str,
    track: &'static str,
    authoritative: bool,
    scorecard: ScorecardView,
    release: ReleaseView,
    coord: CoordView,
    next_free_lanes: Vec<LaneView>,
    leftover_allowlist: Vec<LaneIdView>,
    loop_operable: bool,
    loop_blockers: Vec<&'static str>,
}

#[derive(Serialize)]
struct ScorecardView {
    blended: f64,
    admitted_row_count: usize,
    authoritative: bool,
}

#[derive(Serialize)]
struct ReleaseView {
    profile: &'static str,
    status: String,
}

#[derive(Serialize)]
struct CoordView {
    available: bool,
    active: usize,
    handed_off_uncommitted: usize,
    paths: Vec<PathView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
struct PathView {
    repo: String,
    path: String,
    lane: String,
}

#[derive(Serialize)]
struct LaneView {
    id: &'static str,
    lane: &'static str,
    repo: &'static str,
    paths: &'static [&'static str],
}

#[derive(Serialize)]
struct LaneIdView {
    id: &'static str,
    lane: &'static str,
    repo: &'static str,
}

/// Which loop the board evaluates. The master plan's M0.2 split: the
/// coordination track and the dogfood-policy track fail for their own
/// reasons only, so a governance blocker on one can never be silenced by
/// manufacturing an artifact for the other.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BoardTrack {
    Coord,
    Dogfood,
    All,
}

impl BoardTrack {
    pub(crate) fn parse(value: &str) -> Result<Self, CoordError> {
        match value {
            "coord" => Ok(Self::Coord),
            "dogfood" => Ok(Self::Dogfood),
            "all" => Ok(Self::All),
            other => Err(CoordError::new(
                "USAGE",
                format!("--track must be coord, dogfood, or all (got {other:?})"),
            )),
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Coord => "coord",
            Self::Dogfood => "dogfood",
            Self::All => "all",
        }
    }

    const fn includes_coord(self) -> bool {
        matches!(self, Self::Coord | Self::All)
    }

    const fn includes_dogfood(self) -> bool {
        matches!(self, Self::Dogfood | Self::All)
    }
}

pub(super) fn board_json(hub: &Path, track: BoardTrack) -> Result<(String, u8), CoordError> {
    let scorecard = scorecard::evaluate(hub)?;
    let admitted_row_count = scorecard.rows.iter().filter(|row| row.admitted).count();
    let receipts = tempfile::tempdir().map_err(CoordError::io)?;
    let release = executor::report_profile(hub, ReleaseProfile::SelfHostedV1, receipts.path())?;
    let (coord, next_free_lanes) = coord_view(hub);
    let root = family_root(hub);
    let mut loop_blockers = Vec::new();
    if track.includes_coord() {
        if !coord.available {
            loop_blockers.push("COORD_UNAVAILABLE");
        }
        if wave0_dirty(&root) {
            loop_blockers.push("WAVE0_DIRTY_SUBJECTS");
        }
    }
    if track.includes_dogfood() {
        match dogfood_binding_status() {
            DogfoodBindingStatus::Missing => loop_blockers.push("DOGFOOD_POLICY_MISSING"),
            DogfoodBindingStatus::Invalid => loop_blockers.push("DOGFOOD_BINDING_INVALID"),
            DogfoodBindingStatus::Valid => {}
        }
    }
    let loop_operable = loop_blockers.is_empty();
    let board = Board {
        schema_version: SCHEMA_VERSION,
        kind: "DIAGNOSTIC",
        track: track.name(),
        authoritative: false,
        scorecard: ScorecardView {
            blended: scorecard.blended,
            admitted_row_count,
            authoritative: false,
        },
        release: ReleaseView {
            profile: release.profile().unwrap_or("self-hosted-v1"),
            status: release.status().as_str().to_owned(),
        },
        coord,
        next_free_lanes,
        leftover_allowlist: LEFTOVERS
            .iter()
            .map(|lane| LaneIdView {
                id: lane.id,
                lane: lane.lane,
                repo: lane.repo,
            })
            .collect(),
        loop_operable,
        loop_blockers,
    };
    if board.authoritative || board.scorecard.authoritative {
        return Err(CoordError::new(
            "DOGFOOD_BOARD_AUTHORITATIVE",
            "diagnostic board must never be authoritative",
        ));
    }
    let _ = DogfoodRunObservationV0::template();
    let json = serde_json::to_string_pretty(&board).map_err(CoordError::json)?;
    Ok((json, if loop_operable { 0 } else { 1 }))
}

fn coord_view(hub: &Path) -> (CoordView, Vec<LaneView>) {
    let root = family_root(hub);
    match CoordStore::new(root).status() {
        Ok(status) => project_claims(&status.claims),
        Err(error) => unavailable_coord(&error),
    }
}

fn project_claims(claims: &[ClaimSummary]) -> (CoordView, Vec<LaneView>) {
    let mut active = 0;
    let mut handed = 0;
    let mut paths = Vec::new();
    let mut blocking_paths = Vec::new();
    for claim in claims {
        let is_active = claim.state == ClaimState::Active;
        let is_handed_uncommitted =
            claim.state == ClaimState::HandedOff && claim.commit_oid.is_none();
        if is_active {
            active += 1;
        }
        if is_handed_uncommitted {
            handed += 1;
        }
        if is_active || is_handed_uncommitted {
            for path in &claim.paths {
                blocking_paths.push((claim.repo.as_str(), path.as_str()));
                if is_active {
                    paths.push(PathView {
                        repo: claim.repo.clone(),
                        path: path.clone(),
                        lane: claim.lane.clone(),
                    });
                }
            }
        }
    }
    let next_free = LEFTOVERS
        .iter()
        .filter(|lane| !lane_blocked(lane, &blocking_paths))
        .map(|lane| LaneView {
            id: lane.id,
            lane: lane.lane,
            repo: lane.repo,
            paths: lane.paths,
        })
        .collect();
    (
        CoordView {
            available: true,
            active,
            handed_off_uncommitted: handed,
            paths,
            error: None,
        },
        next_free,
    )
}

fn unavailable_coord(error: &CoordError) -> (CoordView, Vec<LaneView>) {
    (
        CoordView {
            available: false,
            active: 0,
            handed_off_uncommitted: 0,
            paths: Vec::new(),
            error: Some(error.code().to_owned()),
        },
        Vec::new(),
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DogfoodBindingStatus {
    Missing,
    Invalid,
    Valid,
}

fn dogfood_binding_status() -> DogfoodBindingStatus {
    let Some(path) = std::env::var_os(DOGFOOD_BINDING_ENV) else {
        return DogfoodBindingStatus::Missing;
    };
    let path = Path::new(&path);
    if !path.is_file() {
        return DogfoodBindingStatus::Missing;
    }
    match load_and_validate_binding(path) {
        Ok(()) => DogfoodBindingStatus::Valid,
        Err(()) => DogfoodBindingStatus::Invalid,
    }
}

fn load_and_validate_binding(path: &Path) -> Result<(), ()> {
    let bytes = std::fs::read(path).map_err(|_| ())?;
    let value = bullet_wire::decode_unique_value(&bytes).map_err(|_| ())?;
    let object = value.as_object().ok_or(())?;
    let schema = object
        .get("schema_version")
        .and_then(|value| value.as_str())
        .ok_or(())?;
    let audience = object
        .get("audience")
        .and_then(|value| value.as_str())
        .ok_or(())?;
    let operation = object
        .get("operation")
        .and_then(|value| value.as_str())
        .ok_or(())?;
    if schema != bullet_wire::DogfoodBindingV1::SCHEMA_VERSION
        || audience != "dogfood-runner"
        || operation != "read-only-propose"
        || object.len() != 3
    {
        return Err(());
    }
    let binding = bullet_wire::DogfoodBindingV1::read_only_propose();
    let Some(policy_path) = std::env::var_os(DOGFOOD_POLICY_ENV) else {
        return match bullet_wire::refuse_dogfood_binding_as_live(&binding) {
            Err(error) if error.code() == "LIVE_ADMISSION_REFUSES_DOGFOOD_BINDING" => Ok(()),
            _ => Err(()),
        };
    };
    let policy_path = Path::new(&policy_path);
    if !policy_path.is_file() {
        return Err(());
    }
    let policy_bytes = std::fs::read(policy_path).map_err(|_| ())?;
    let policy: bullet_wire::PolicySnapshotV1 =
        bullet_wire::decode_canonical(&policy_bytes).map_err(|_| ())?;
    bullet_wire::validate_dogfood_admission(&policy, &binding).map_err(|_| ())
}

fn wave0_dirty(root: &Path) -> bool {
    W0_REPOS.iter().any(|repo| repo_dirty(&root.join(repo)))
}

fn repo_dirty(path: &Path) -> bool {
    if !path.join(".git").exists() {
        return true;
    }
    std::process::Command::new("git")
        .args(["-C", &path.to_string_lossy(), "status", "--porcelain"])
        .output()
        .map(|output| !output.status.success() || !output.stdout.is_empty())
        .unwrap_or(true)
}

fn family_root(hub: &Path) -> PathBuf {
    if let Ok(root) = discover_family_root(hub, None) {
        return root;
    }
    let parent = hub.parent().unwrap_or(hub);
    if parent.join("repos.manifest.toml").is_file() {
        parent.to_path_buf()
    } else {
        hub.to_path_buf()
    }
}

fn lane_blocked(lane: &LeftoverLane, active: &[(&str, &str)]) -> bool {
    active.iter().any(|(repo, path)| {
        *repo == lane.repo && lane.paths.iter().any(|claimed| path_overlap(path, claimed))
    })
}

fn path_overlap(left: &str, right: &str) -> bool {
    let left = left.trim_matches('/');
    let right = right.trim_matches('/');
    left == right
        || left
            .strip_prefix(right)
            .is_some_and(|rest| rest.starts_with('/'))
        || right
            .strip_prefix(left)
            .is_some_and(|rest| rest.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::{
        LEFTOVERS, load_and_validate_binding, path_overlap, project_claims, unavailable_coord,
    };
    use crate::coord::{ClaimState, ClaimSummary, CoordError};

    fn claim(state: ClaimState, committed: bool, repo: &str, paths: &[&str]) -> ClaimSummary {
        ClaimSummary {
            claim_id: format!("clm_{}", "a".repeat(64)),
            agent: "fixture-agent".to_owned(),
            lane: "fixture-lane".to_owned(),
            repo: repo.to_owned(),
            paths: paths.iter().map(|path| (*path).to_owned()).collect(),
            claimed_at_unix_ms: 1,
            last_event_unix_ms: 2,
            expires_unix_ms: 3,
            state,
            proof_command: None,
            changed_paths: Vec::new(),
            commit_oid: committed.then(|| "b".repeat(40)),
            commit_orchestrator: None,
            commit_recorded_at_unix_ms: None,
            recovery_adoption: None,
        }
    }

    fn lane_ids(lanes: &[super::LaneView]) -> Vec<&'static str> {
        lanes.iter().map(|lane| lane.id).collect()
    }

    #[test]
    fn successful_projection_blocks_owned_unintegrated_paths_only() {
        let claims = vec![
            claim(
                ClaimState::Active,
                false,
                "bullet-farm",
                &["crates/bullet-wire/fuzz/corpus"],
            ),
            claim(
                ClaimState::HandedOff,
                false,
                "bullet-git",
                &["crates/bullet-git-workspace"],
            ),
            claim(
                ClaimState::Active,
                false,
                "bullet-kernel",
                &["apps/bullet-farmd/src/dispatch.rs"],
            ),
            claim(ClaimState::Active, false, "bullet-portal", &["src"]),
            claim(
                ClaimState::HandedOff,
                true,
                "bullet-kernel",
                &["crates/runner/src/saga"],
            ),
            claim(
                ClaimState::Active,
                false,
                "bullet-farm",
                &["crates/runner/src/saga"],
            ),
            claim(
                ClaimState::Active,
                false,
                "bullet-kernel",
                &["crates/runner/src/saga-old"],
            ),
        ];

        let (coord, next_free) = project_claims(&claims);

        assert!(coord.available);
        assert_eq!(coord.active, 5);
        assert_eq!(coord.handed_off_uncommitted, 1);
        assert_eq!(coord.paths.len(), 5);
        assert_eq!(lane_ids(&next_free), ["L-64", "S-01"]);
    }

    #[test]
    fn overlap_is_symmetric_and_segment_aware() {
        assert!(path_overlap("src", "src/App.test.tsx"));
        assert!(path_overlap("src/App.test.tsx", "src"));
        assert!(path_overlap("src/App.test.tsx", "src/App.test.tsx"));
        assert!(!path_overlap("src-old", "src"));
        assert!(!path_overlap("src", "src-old"));
    }

    #[test]
    fn l15_names_the_real_repository_relative_test() {
        let lane = LEFTOVERS.iter().find(|lane| lane.id == "L-15").unwrap();
        assert_eq!(
            lane.paths,
            [
                "crates/bullet-git-workspace/src/gc.rs",
                "crates/bullet-git-workspace/tests/gc_safety.rs",
            ]
        );
    }

    #[test]
    fn unavailable_projection_exposes_only_the_typed_code() {
        let error = CoordError::new(
            "COORD_IO_FAILED",
            "/home/secret-user/private-family/token-value",
        );

        let (coord, next_free) = unavailable_coord(&error);

        assert!(!coord.available);
        assert_eq!(coord.error.as_deref(), Some("COORD_IO_FAILED"));
        assert!(coord.paths.is_empty());
        assert!(next_free.is_empty());
    }

    #[test]
    fn malformed_binding_is_typed_refuse() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("binding.json");
        std::fs::write(&path, b"{\"schema_version\":\"nope\"}\n").unwrap();
        assert!(load_and_validate_binding(&path).is_err());
        let ok = dir.path().join("ok.json");
        std::fs::write(
            &ok,
            br#"{"audience":"dogfood-runner","operation":"read-only-propose","schema_version":"v1alpha1"}"#,
        )
        .unwrap();
        assert!(load_and_validate_binding(&ok).is_ok());
    }
}
