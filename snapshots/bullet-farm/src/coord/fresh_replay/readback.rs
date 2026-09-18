//! Reconstruct sealed replay facts; no review or coordinator admission is granted.
use super::*;

#[derive(Debug, Serialize)]
pub(crate) struct ReplayVerification {
    kind: &'static str,
    schema_version: u32,
    purpose: &'static str,
    replay_id: String,
    record: PathBuf,
    sealed_sha256: String,
    byte_length: u64,
}

pub(crate) fn verify(root: &Path, record: &Path) -> Result<ReplayVerification, CoordError> {
    require_normalized_absolute(root, "replay family root")?;
    outside(root, record)?;
    // The sealed reader rejects duplicate members, unsafe numbers, noncanonical
    // framing, aliases and oversized input before any embedded path is followed.
    let value: serde_json::Value = sealed::read(record)?;
    let observed = canonical_lf(&value)?;
    let request = value
        .get("facts")
        .and_then(|facts| facts.get("request"))
        .ok_or_else(|| invalid("replay record omits its embedded request"))?;
    let request: FreshReplayRequestV1 = serde_json::from_value(request.clone())
        .map_err(|error| invalid(format!("replay request is not closed: {error}")))?;
    let reconstructed = reconstruct(root, request, &[record])?;
    // Comparing the entire canonical reconstruction also closes the outer
    // record and computed projections without adding permissive DTO decoders.
    if reconstructed.bytes != observed {
        return Err(invalid(
            "replay record differs from its complete canonical reconstruction",
        ));
    }
    #[cfg(all(test, target_os = "linux"))]
    if let Some(hook) = AFTER_RECONSTRUCTION.with(|slot| slot.borrow_mut().take()) {
        hook();
    }
    revalidate(&reconstructed.inputs())?;
    revalidate(&[(record, &observed)])?;
    Ok(ReplayVerification {
        kind: "FRESH_REPLAY_VERIFICATION_V1",
        schema_version: 1,
        purpose: "REPLAY_FACTS_READBACK_ONLY",
        replay_id: reconstructed.subject.replay_id,
        record: record.to_owned(),
        sealed_sha256: sha256(&observed),
        byte_length: observed.len() as u64,
    })
}

#[cfg(all(test, target_os = "linux"))]
thread_local! {
    static AFTER_RECONSTRUCTION: std::cell::RefCell<Option<Box<dyn FnOnce()>>> = const { std::cell::RefCell::new(None) };
}

#[cfg(all(test, target_os = "linux"))]
mod tests;
