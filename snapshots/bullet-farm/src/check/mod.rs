//! Rust-owned, exact-subject local check catalogs.

mod catalog;
pub mod corpus;
mod dogfood;
mod executor;
mod prerequisites;
mod profiles;
mod release_evidence;
mod semantic_registry;
mod subject;
mod truth;

pub mod model;

use crate::coord::CoordError;
use model::{CheckReport, CheckTier};
use std::path::{Path, PathBuf};

const USAGE: &str = "usage: bullet-family [--root PATH] check <fast|required> [--json] | check release --profile PROFILE --receipts ABSOLUTE_PATH [--json | --report [--portable]]";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputMode {
    Human,
    Json,
    Report(truth::Variant),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckExecution {
    report: CheckReport,
    mode: OutputMode,
    /// Rendered release truth page; present only for `--report`.
    page: Option<String>,
}

impl CheckExecution {
    pub const fn exit_code(&self) -> u8 {
        self.report.exit_code()
    }

    pub fn output(&self) -> Result<String, CoordError> {
        match (self.mode, &self.page) {
            (OutputMode::Json, _) => self.report.stable_json().map_err(CoordError::json),
            (OutputMode::Report(_), Some(page)) => Ok(page.clone()),
            (OutputMode::Report(_), None) => Err(CoordError::new(
                "RELEASE_TRUTH_UNAVAILABLE",
                "the release truth page was not rendered",
            )),
            (OutputMode::Human, _) => Ok(self.report.human()),
        }
    }

    pub const fn report(&self) -> &CheckReport {
        &self.report
    }
}

pub fn run(hub: &Path, args: &[String]) -> Result<CheckExecution, CoordError> {
    let parsed = parse(args)?;
    if parsed.tier == CheckTier::Release && parsed.profile.is_none() {
        return Err(CoordError::new(
            "PROFILE_REQUIRED",
            format!(
                "release admission requires --profile; choose one of {} (legacy-v1-26 is diagnostic only)",
                profiles::ReleaseProfile::NAMES.join(", ")
            ),
        ));
    }
    let report = match (parsed.profile, parsed.receipts.as_deref()) {
        (Some(profile), Some(receipts)) => executor::report_profile(hub, profile, receipts)?,
        (None, None) => executor::report(hub, parsed.tier)?,
        _ => return Err(CoordError::new("USAGE", USAGE)),
    };
    let page = match parsed.mode {
        OutputMode::Report(variant) => Some(truth::render(hub, &report, variant)?),
        OutputMode::Human | OutputMode::Json => None,
    };
    Ok(CheckExecution {
        report,
        mode: parsed.mode,
        page,
    })
}

pub(crate) use dogfood::BoardTrack;

pub(crate) fn dogfood_board(
    hub: &Path,
    track: dogfood::BoardTrack,
) -> Result<(String, u8), CoordError> {
    dogfood::board_json(hub, track)
}

#[derive(Debug)]
struct Parsed {
    tier: CheckTier,
    mode: OutputMode,
    profile: Option<profiles::ReleaseProfile>,
    receipts: Option<PathBuf>,
}

fn parse(args: &[String]) -> Result<Parsed, CoordError> {
    let (tier, rest) = args
        .split_first()
        .ok_or_else(|| CoordError::new("USAGE", USAGE))?;
    let tier = match tier.as_str() {
        "fast" => CheckTier::Fast,
        "required" => CheckTier::Required,
        "release" => CheckTier::Release,
        _ => return Err(CoordError::new("USAGE", USAGE)),
    };
    let mode = match rest {
        [] => OutputMode::Human,
        [flag] if flag == "--json" => OutputMode::Json,
        [flag] if flag == "--report" && tier == CheckTier::Release => {
            OutputMode::Report(truth::Variant::Live)
        }
        [flag, portable]
            if flag == "--report" && portable == "--portable" && tier == CheckTier::Release =>
        {
            OutputMode::Report(truth::Variant::Portable)
        }
        _ if tier == CheckTier::Release => return parse_profile(rest),
        _ => return Err(CoordError::new("USAGE", USAGE)),
    };
    Ok(Parsed {
        tier,
        mode,
        profile: None,
        receipts: None,
    })
}

fn parse_profile(args: &[String]) -> Result<Parsed, CoordError> {
    let mut profile = None;
    let mut receipts = None;
    let mut json = false;
    let mut report = false;
    let mut portable = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--profile" if profile.is_none() => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| CoordError::new("USAGE", USAGE))?;
                profile = Some(profiles::ReleaseProfile::parse(value)?);
                index += 2;
            }
            "--receipts" if receipts.is_none() => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| CoordError::new("USAGE", USAGE))?;
                let path = PathBuf::from(value);
                if !path.is_absolute() {
                    return Err(CoordError::new(
                        "INVALID_RECEIPT_REGISTRY",
                        "--receipts must be an absolute path",
                    ));
                }
                receipts = Some(path);
                index += 2;
            }
            "--json" if !json => {
                json = true;
                index += 1;
            }
            "--report" if !report => {
                report = true;
                index += 1;
            }
            "--portable" if !portable => {
                portable = true;
                index += 1;
            }
            _ => return Err(CoordError::new("USAGE", USAGE)),
        }
    }
    if receipts.is_none() || (json && report) || (portable && !report) {
        return Err(CoordError::new("USAGE", USAGE));
    }
    Ok(Parsed {
        tier: CheckTier::Release,
        mode: if report {
            OutputMode::Report(if portable {
                truth::Variant::Portable
            } else {
                truth::Variant::Live
            })
        } else if json {
            OutputMode::Json
        } else {
            OutputMode::Human
        },
        profile,
        receipts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_is_exact() {
        for tier in ["fast", "required", "release"] {
            assert!(parse(&[tier.into()]).is_ok());
            assert!(parse(&[tier.into(), "--json".into()]).is_ok());
        }
        assert_eq!(
            parse(&["release".into(), "--report".into()]).unwrap().mode,
            OutputMode::Report(truth::Variant::Live)
        );
        assert_eq!(
            parse(&["release".into(), "--report".into(), "--portable".into()])
                .unwrap()
                .mode,
            OutputMode::Report(truth::Variant::Portable)
        );
        for invalid in [
            vec![],
            vec!["unknown".into()],
            vec!["fast".into(), "--yaml".into()],
            vec!["fast".into(), "--json".into(), "--json".into()],
            vec!["--json".into(), "fast".into()],
            vec!["fast".into(), "--report".into()],
            vec!["required".into(), "--report".into(), "--portable".into()],
            vec!["release".into(), "--portable".into()],
            vec!["release".into(), "--portable".into(), "--report".into()],
            vec!["release".into(), "--json".into(), "--report".into()],
            vec!["release".into(), "--profile".into(), "linux-preview".into()],
        ] {
            assert_eq!(parse(&invalid).unwrap_err().code(), "USAGE");
        }
        let unprofiled = parse(&[
            "release".into(),
            "--receipts".into(),
            "/tmp/receipts".into(),
            "--json".into(),
        ])
        .unwrap();
        assert_eq!(unprofiled.profile, None);
        assert_eq!(unprofiled.mode, OutputMode::Json);
        let profiled = parse(&[
            "release".into(),
            "--profile".into(),
            "linux-preview".into(),
            "--receipts".into(),
            "/tmp/receipts".into(),
            "--json".into(),
        ])
        .unwrap();
        assert_eq!(profiled.profile.unwrap().as_str(), "linux-preview");
        assert_eq!(profiled.mode, OutputMode::Json);
        let profiled_report = parse(&[
            "release".into(),
            "--profile".into(),
            "legacy-v1-26".into(),
            "--receipts".into(),
            "/tmp/receipts".into(),
            "--report".into(),
            "--portable".into(),
        ])
        .unwrap();
        assert_eq!(
            profiled_report.mode,
            OutputMode::Report(truth::Variant::Portable)
        );
        assert_eq!(
            parse(&[
                "release".into(),
                "--profile".into(),
                "linux-preview".into(),
                "--receipts".into(),
                "relative".into(),
            ])
            .unwrap_err()
            .code(),
            "INVALID_RECEIPT_REGISTRY"
        );
        assert_eq!(
            parse(&[
                "release".into(),
                "--profile".into(),
                "dogfood-local-v0".into(),
                "--receipts".into(),
                "/tmp/receipts".into(),
            ])
            .unwrap_err()
            .code(),
            "NOT_A_RELEASE_PROFILE"
        );
    }
}
