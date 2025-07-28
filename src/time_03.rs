use std::{convert::TryFrom, ops::Add};

use crate::Duration;

use time::{Date, OffsetDateTime, PrimitiveDateTime};

impl Add<Duration> for OffsetDateTime {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        // Date component arithmetic

        let (year, month, mut day) = self.date().to_calendar_date();
        let month_u8 = month as u8;

        // Add years and months
        // We do this manually to handle month-end clamping correctly.
        // Month is 1-based, so convert to 0-based for calculation
        let month_0_based = month_u8 as u32 - 1;
        let total_months_0_based = month_0_based + rhs.month as u32;

        let new_year = year + rhs.year as i32 + (total_months_0_based / 12) as i32;
        let new_month_u8 = (total_months_0_based % 12 + 1) as u8;

        let new_month = match time::Month::try_from(new_month_u8) {
            Ok(m) => m,
            // This should not happen with the modulo arithmetic above, but as a safeguard:
            Err(_) => return self,
        };

        // Clamp day to the valid range for the new month and year.
        let max_day_in_month = new_month.length(new_year);
        if day > max_day_in_month {
            day = max_day_in_month;
        }

        let date_with_ym_added = match Date::from_calendar_date(new_year, new_month, day) {
            Ok(d) => d,
            // This should not happen due to the clamping logic, but as a safeguard:
            Err(_) => return self,
        };

        // Add days. `saturating_add` with `time::Duration::days` handles calendar days.
        let final_date = date_with_ym_added.saturating_add(time::Duration::days(rhs.day as i64));

        // Time component arithmetic
        let time_duration = time::Duration::hours(rhs.hour as i64)
            + time::Duration::minutes(rhs.minute as i64)
            + time::Duration::seconds_f32(rhs.second);

        // Reconstruct the datetime and add the time duration
        let primitive_dt = PrimitiveDateTime::new(final_date, self.time());
        let offset_dt = primitive_dt.assume_offset(self.offset());

        offset_dt.saturating_add(time_duration)
    }
}

#[cfg(all(test, feature = "time_03"))]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn add_one_month_to_end_of_january() {
        let start = datetime!(2023-01-31 10:00:00 UTC);
        let duration = Duration {
            year: 0.0,
            month: 1.0,
            day: 0.0,
            hour: 0.0,
            minute: 0.0,
            second: 0.0,
        };
        let end = start + duration;
        // Adding one month to Jan 31 should result in Feb 28 in a non-leap year.
        assert_eq!(end, datetime!(2023-02-28 10:00:00 UTC));
    }

    #[test]
    fn add_one_year_to_leap_day() {
        let start = datetime!(2024-02-29 10:00:00 UTC); // Leap year
        let duration = Duration {
            year: 1.0,
            month: 0.0,
            day: 0.0,
            hour: 0.0,
            minute: 0.0,
            second: 0.0,
        };
        let end = start + duration;
        // Adding one year to Feb 29 should result in Feb 28 of the next year.
        assert_eq!(end, datetime!(2025-02-28 10:00:00 UTC));
    }

    #[test]
    fn add_one_day() {
        let start = datetime!(2023-03-15 10:00:00 UTC);
        let duration = Duration {
            year: 0.0,
            month: 0.0,
            day: 1.0,
            hour: 0.0,
            minute: 0.0,
            second: 0.0,
        };
        let end = start + duration;
        assert_eq!(end, datetime!(2023-03-16 10:00:00 UTC));
    }

    #[test]
    fn add_one_hour() {
        let start = datetime!(2023-03-15 10:00:00 UTC);
        let duration = Duration {
            year: 0.0,
            month: 0.0,
            day: 0.0,
            hour: 1.0,
            minute: 0.0,
            second: 0.0,
        };
        let end = start + duration;
        assert_eq!(end, datetime!(2023-03-15 11:00:00 UTC));
    }

    #[test]
    fn add_mixed_duration() {
        let start = datetime!(2023-01-15 10:30:00 UTC);
        let duration = Duration {
            year: 1.0,
            month: 1.0,
            day: 1.0,
            hour: 1.0,
            minute: 1.0,
            second: 1.0,
        };
        let end = start + duration;
        assert_eq!(end, datetime!(2024-02-16 11:31:01 UTC));
    }

    #[test]
    fn add_duration_crossing_year_boundary_with_month() {
        let start = datetime!(2023-12-15 10:00:00 UTC);
        let duration = Duration {
            year: 0.0,
            month: 1.0,
            day: 0.0,
            hour: 0.0,
            minute: 0.0,
            second: 0.0,
        };
        let end = start + duration;
        assert_eq!(end, datetime!(2024-01-15 10:00:00 UTC));
    }
}
