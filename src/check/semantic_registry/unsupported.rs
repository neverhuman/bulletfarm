use std::{fs, io::ErrorKind, path::Path};

use super::Evaluation;

pub(super) fn evaluate(registry: &Path, manifest_name: &str) -> Evaluation {
    let root = match fs::symlink_metadata(registry) {
        Err(error) if error.kind() == ErrorKind::NotFound => return Evaluation::Absent,
        Err(error) => return Evaluation::Rejected(error.to_string()),
        Ok(root) => root,
    };
    if root.file_type().is_symlink() || !root.is_dir() {
        return Evaluation::Rejected(
            "registry root is not a real non-symlink directory".to_owned(),
        );
    }
    match fs::symlink_metadata(registry.join(manifest_name)) {
        Err(error) if error.kind() == ErrorKind::NotFound => Evaluation::Absent,
        Err(error) => Evaluation::Rejected(error.to_string()),
        Ok(_) => Evaluation::Rejected(
            "descriptor-relative release-registry admission is unavailable on this platform"
                .to_owned(),
        ),
    }
}
