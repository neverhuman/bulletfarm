use std::{fs, os::unix::fs::MetadataExt};

use crate::coord::CoordError;

use super::invalid;

pub(in crate::coord::generation::recovery) fn has_other_writable_fd(
    identity: (u64, u64),
) -> Result<bool, CoordError> {
    let own_uid = fs::metadata("/proc/self").map_err(CoordError::io)?.uid();
    let mut processes = fs::read_dir("/proc")
        .map_err(legacy_unknown)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(legacy_unknown)?;
    processes.sort_by_key(|entry| {
        entry.file_name() != std::ffi::OsString::from(std::process::id().to_string())
    });
    for process_entry in processes {
        let Some(pid) = process_entry.file_name().to_str().and_then(parse_pid) else {
            continue;
        };
        if process_entry.metadata().map_err(legacy_unknown)?.uid() != own_uid {
            continue;
        }
        for descriptor in fs::read_dir(process_entry.path().join("fd")).map_err(legacy_unknown)? {
            let descriptor = descriptor.map_err(legacy_unknown)?;
            let metadata = fs::metadata(descriptor.path()).map_err(legacy_unknown)?;
            if (metadata.dev(), metadata.ino()) != identity {
                continue;
            }
            let info = fs::read_to_string(format!(
                "/proc/{pid}/fdinfo/{}",
                descriptor.file_name().to_string_lossy()
            ))
            .map_err(legacy_unknown)?;
            let flags = info
                .lines()
                .find_map(|line| line.strip_prefix("flags:\t"))
                .and_then(|value| u32::from_str_radix(value, 8).ok())
                .ok_or_else(|| invalid("cannot parse matching legacy descriptor flags"))?;
            let after = fs::metadata(descriptor.path()).map_err(legacy_unknown)?;
            if (after.dev(), after.ino()) != identity {
                return Err(CoordError::new(
                    "LEGACY_WRITER_UNKNOWN",
                    "matching legacy descriptor identity changed during inspection",
                ));
            }
            if flags & 0o3 != 0 {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn parse_pid(value: &str) -> Option<u32> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.bytes().try_fold(0_u32, |parsed, byte| {
        parsed.checked_mul(10)?.checked_add(u32::from(byte - b'0'))
    })
}

fn legacy_unknown(error: std::io::Error) -> CoordError {
    CoordError::new("LEGACY_WRITER_UNKNOWN", error.to_string())
}
