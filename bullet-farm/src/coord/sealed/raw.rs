//! Bounded binary artifacts used by credential-free recovery authoring.

use std::path::Path;

use crate::coord::CoordError;

/// Publish exact nonempty bytes once beneath an admitted private parent.
///
/// The shared publisher retains an anonymous descriptor through sync, link,
/// and independent readback, and publishes a single mode-0400 link. The caller
/// supplies the artifact-specific bound; no newline or text transformation is
/// performed here.
pub(crate) fn write_raw(path: &Path, bytes: &[u8], maximum: u64) -> Result<(), CoordError> {
    super::write_bytes(path, bytes, maximum)
}
