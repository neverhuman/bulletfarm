#[cfg(test)]
use std::cell::Cell;

use crate::coord::CoordError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Crash {
    Seal,
    Exchange,
    Retire,
}

#[cfg(test)]
thread_local! {
    static CRASH: Cell<Option<Crash>> = const { Cell::new(None) };
}

#[cfg(test)]
pub(super) fn test_crash_at(point: Crash) {
    CRASH.with(|selected| selected.set(Some(point)));
}

#[cfg(not(test))]
pub(super) fn injected(_point: Crash) -> Result<(), CoordError> {
    Ok(())
}

#[cfg(test)]
pub(super) fn injected(point: Crash) -> Result<(), CoordError> {
    if CRASH.with(|selected| selected.get()) == Some(point) {
        CRASH.with(|selected| selected.set(None));
        Err(CoordError::new(
            "COORD_RECOVERY_TEST_INTERRUPTION",
            format!("injected interruption after {point:?}"),
        ))
    } else {
        Ok(())
    }
}
