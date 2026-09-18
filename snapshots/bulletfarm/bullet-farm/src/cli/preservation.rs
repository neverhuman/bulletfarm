use std::path::{Path, PathBuf};

use crate::coord::{CoordError, publish_preservation_record, recovery_manifest, sealed};

use super::coord::Options;

pub(super) fn run(root: &Path, args: &[String]) -> Result<String, CoordError> {
    if args.first().is_some_and(|value| value == "replay-verify") {
        let options = Options::parse(&args[1..])?;
        options.reject_flags()?;
        options.reject_unknown_values(&["record"])?;
        let verified =
            crate::coord::fresh_replay::verify(root, Path::new(&options.one("record")?))?;
        return serde_json::to_string(&verified).map_err(CoordError::json);
    }
    if args.first().is_some_and(|value| value == "replay") {
        let options = Options::parse(&args[1..])?;
        options.reject_flags()?;
        options.reject_unknown_values(&["request", "out"])?;
        let publication = crate::coord::fresh_replay::publish(
            root,
            Path::new(&options.one("request")?),
            Path::new(&options.one("out")?),
        )?;
        return serde_json::to_string(&publication).map_err(CoordError::json);
    }
    let options = Options::parse(args)?;
    options.reject_flags()?;
    options.reject_unknown_values(&["outer-inventory", "hub-inventory", "out"])?;
    let outer_path = PathBuf::from(options.one("outer-inventory")?);
    let hub_path = PathBuf::from(options.one("hub-inventory")?);
    let output = PathBuf::from(options.one("out")?);
    for path in [&outer_path, &hub_path, &output] {
        recovery_manifest::require_normalized_absolute(path, "preservation record")?;
        if path.starts_with(root) {
            return Err(CoordError::new(
                "INVALID_FRESH_GENESIS_PRODUCTION",
                "preservation input and output records must remain outside the family root",
            ));
        }
    }
    if outer_path == hub_path || outer_path == output || hub_path == output {
        return Err(CoordError::new(
            "INVALID_FRESH_GENESIS_PRODUCTION",
            "preservation input and output paths must be distinct",
        ));
    }
    let publication = publish_preservation_record(
        root,
        &output,
        &sealed::read_observation(&outer_path)?,
        &sealed::read_observation(&hub_path)?,
    )?;
    serde_json::to_string(&serde_json::json!({
        "preservation_id": publication.subject.preservation_id,
        "sealed_sha256": publication.sealed_sha256,
        "byte_length": publication.byte_length,
        "outcome": publication.outcome,
        "path": output,
    }))
    .map_err(CoordError::json)
}

#[cfg(test)]
mod tests;
