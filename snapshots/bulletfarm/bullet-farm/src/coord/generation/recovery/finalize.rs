use std::fs::File;

use super::{
    RecoveryInput,
    authority::{Authority, BaselineSubject},
    exchange, platform_fs as io,
    tree::Layout,
    verifier::LegacyReadLease,
};
use crate::coord::{CoordError, generation::manifest::GenerationManifest, model::Record};

#[allow(clippy::too_many_arguments)]
pub(super) fn revalidate(
    authority: &Authority,
    layout: &Layout,
    input: &RecoveryInput,
    manifest: &GenerationManifest,
    baseline_record: &Record,
    baseline: &BaselineSubject,
    sibling_name: &str,
    tombstone: &File,
    retired_source: &mut File,
    legacy_lease: &LegacyReadLease,
) -> Result<(), CoordError> {
    legacy_lease.revalidate()?;
    authority.revalidate_final(&input.coord_dir)?;
    layout.verify_generation(manifest, baseline_record, baseline)?;
    exchange::revalidate_final_topology(
        authority.root(),
        layout.recovery(),
        sibling_name,
        tombstone,
        retired_source,
        authority.owner(),
    )?;
    io::verify_open_file(retired_source, &input.frozen_live_source.content)?;
    legacy_lease.revalidate()
}
