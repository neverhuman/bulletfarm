use crate::{
    DogfoodLaunchGrantClaimsV1, DogfoodReadOnlyIntentV1, ProviderEnrollmentClaimsV2,
    ProviderRuntimePassportV1, RuntimePassportError, WireError,
};

use super::verify_dogfood_subjects;

/// Close the unsigned enrollment edge over the exact runtime body.
///
/// Success is structural component evidence only. It does not authenticate an
/// enrollment or grant, bind unrelated enrollment observations, inspect a
/// filesystem, or authorize a process launch.
pub fn verify_dogfood_runtime_binding(
    grant: &DogfoodLaunchGrantClaimsV1,
    intent: &DogfoodReadOnlyIntentV1,
    enrollment: &ProviderEnrollmentClaimsV2,
    passport: &ProviderRuntimePassportV1,
) -> Result<(), WireError> {
    verify_dogfood_subjects(grant, intent, enrollment)?;
    passport.validate().map_err(runtime_error)?;

    let actual_id = passport.passport_id().map_err(runtime_error)?;
    let bound = &intent.subject.provider;
    if actual_id != bound.runtime_passport_id || actual_id != enrollment.runtime_passport_id {
        return Err(WireError::new(
            "RUNTIME_PASSPORT_ID_MISMATCH",
            "runtime passport body does not match the intent and enrollment lock",
        ));
    }

    if passport.provider != enrollment.provider
        || passport.protocol != enrollment.protocol
        || passport.version.as_bytes() != enrollment.runtime_version.as_bytes()
    {
        return Err(WireError::new(
            "PROVIDER_ENROLLMENT_RUNTIME_MISMATCH",
            "enrollment provider, protocol, or version does not match the runtime passport body",
        ));
    }
    Ok(())
}

fn runtime_error(error: RuntimePassportError) -> WireError {
    WireError::new(error.reason_code(), error.to_string())
}
