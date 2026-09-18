//! Deterministic RFC 3339 UTC rendering without a date dependency.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::coord::CoordError;

/// Seconds since the Unix epoch, refusing a pre-epoch or unrepresentable clock.
pub(super) fn now_unix_seconds() -> Result<i64, CoordError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| CoordError::new("CLOCK_BEFORE_EPOCH", format!("system clock: {error}")))?;
    i64::try_from(duration.as_secs())
        .map_err(|_| CoordError::new("CLOCK_OVERFLOW", "system time does not fit i64 seconds"))
}

/// `YYYY-MM-DDThh:mm:ssZ` for a Unix timestamp, using the proleptic Gregorian
/// civil-from-days algorithm. UTC only; this build records no local time zone.
#[must_use]
pub(super) fn rfc3339_utc(unix_seconds: i64) -> String {
    let days = unix_seconds.div_euclid(86_400);
    let seconds = unix_seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        seconds / 3600,
        (seconds % 3600) / 60,
        seconds % 60,
    )
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = u32::try_from(day_of_year - (153 * shifted_month + 2) / 5 + 1)
        .expect("day of month is bounded");
    let month = u32::try_from(if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    })
    .expect("month is bounded");
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
mod tests {
    use super::rfc3339_utc;

    #[test]
    fn known_instants_render_exactly() {
        assert_eq!(rfc3339_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(rfc3339_utc(1), "1970-01-01T00:00:01Z");
        assert_eq!(rfc3339_utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(rfc3339_utc(1_787_654_321), "2026-08-25T10:38:41Z");
        assert_eq!(rfc3339_utc(4_102_444_799), "2099-12-31T23:59:59Z");
    }
}
