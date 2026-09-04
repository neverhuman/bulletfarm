use std::fs::File;

use super::GenerationManifest;
#[cfg(target_os = "linux")]
use super::published;
use crate::coord::CoordError;

pub(crate) struct PublishedRecoveryGuard {
    #[cfg(target_os = "linux")]
    inner: published::Guard,
}

impl PublishedRecoveryGuard {
    pub(crate) fn revalidate(&self) -> Result<(), CoordError> {
        #[cfg(target_os = "linux")]
        {
            self.inner.revalidate()
        }
        #[cfg(not(target_os = "linux"))]
        Err(platform())
    }
}

pub(crate) fn verify(
    root: &File,
    manifest: &GenerationManifest,
) -> Result<PublishedRecoveryGuard, CoordError> {
    #[cfg(target_os = "linux")]
    {
        published::verify(root, manifest).map(|inner| PublishedRecoveryGuard { inner })
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (root, manifest);
        Err(platform())
    }
}

pub(crate) fn reverify(
    root: &File,
    manifest: &GenerationManifest,
    guard: &PublishedRecoveryGuard,
) -> Result<(), CoordError> {
    #[cfg(target_os = "linux")]
    {
        published::reverify(root, manifest, &guard.inner)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (root, manifest, guard);
        Err(platform())
    }
}

#[cfg(not(target_os = "linux"))]
fn platform() -> CoordError {
    CoordError::new(
        "COORD_RECOVERY_PLATFORM_UNSUPPORTED",
        "published recovery verification is implemented only on Linux",
    )
}
