#[cfg(target_os = "linux")]
fn snapshot_archive(bundle: &Path, expected: &ReleaseFile) -> Result<File, CoordError> {
    use nix::{
        fcntl::{FcntlArg, SealFlag, fcntl},
        sys::memfd::{MemFdCreateFlag, memfd_create},
    };

    if expected.size > MAX_ARCHIVE_BYTES {
        return Err(limit("signed archive exceeds the extraction byte limit"));
    }
    let path = bundle.join(&expected.path);
    let mut source = OpenOptions::new()
        .read(true)
        .custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_NONBLOCK)
        .open(&path)
        .map_err(CoordError::io)?;
    let metadata = source.metadata().map_err(CoordError::io)?;
    if !metadata.file_type().is_file() || metadata.len() != expected.size {
        return Err(invalid_archive(
            "signed archive is not the exact bounded regular file",
        ));
    }
    let descriptor = memfd_create(
        c"bullet-release-archive",
        MemFdCreateFlag::MFD_ALLOW_SEALING | MemFdCreateFlag::MFD_CLOEXEC,
    )
    .map_err(|error| {
        CoordError::new(
            "RELEASE_ARCHIVE_PIN_FAILED",
            format!("could not create archive snapshot: {error}"),
        )
    })?;
    let mut snapshot = File::from(descriptor);
    let mut hasher = blake3::Hasher::new();
    let mut copied = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    while copied <= expected.size {
        let count = source.read(&mut buffer).map_err(CoordError::io)?;
        if count == 0 {
            break;
        }
        copied = copied
            .checked_add(count as u64)
            .ok_or_else(|| limit("archive snapshot size overflowed"))?;
        if copied > expected.size {
            return Err(invalid_archive(
                "signed archive grew while it was being pinned",
            ));
        }
        hasher.update(&buffer[..count]);
        snapshot
            .write_all(&buffer[..count])
            .map_err(CoordError::io)?;
    }
    let digest = format!("blake3:{}", hasher.finalize().to_hex());
    if copied != expected.size || digest != expected.digest {
        return Err(invalid_archive(
            "pinned archive bytes differ from the signed manifest",
        ));
    }
    snapshot.flush().map_err(CoordError::io)?;
    fcntl(
        snapshot.as_raw_fd(),
        FcntlArg::F_ADD_SEALS(
            SealFlag::F_SEAL_WRITE
                | SealFlag::F_SEAL_GROW
                | SealFlag::F_SEAL_SHRINK
                | SealFlag::F_SEAL_SEAL,
        ),
    )
    .map_err(|error| {
        CoordError::new(
            "RELEASE_ARCHIVE_PIN_FAILED",
            format!("could not seal archive snapshot: {error}"),
        )
    })?;
    snapshot.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    Ok(snapshot)
}

#[cfg(not(target_os = "linux"))]
fn snapshot_archive(_bundle: &Path, _expected: &ReleaseFile) -> Result<File, CoordError> {
    Err(CoordError::new(
        "RELEASE_EXTRACTION_PLATFORM_UNSUPPORTED",
        "exact archive snapshots are currently supported only on Linux",
    ))
}
