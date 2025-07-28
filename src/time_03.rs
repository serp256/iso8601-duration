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
