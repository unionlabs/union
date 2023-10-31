// This is a part of Chrono.
// See README.md and LICENSE.txt for details.

//! A collection of parsed date and time items.
//! They can be constructed incrementally while being checked for consistency.

use num_integer::div_rem;
use num_traits::ToPrimitive;

use super::{ParseResult, IMPOSSIBLE, NOT_ENOUGH, OUT_OF_RANGE};
use crate::naive::{NaiveDate, NaiveDateTime, NaiveTime};
use crate::offset::{FixedOffset, LocalResult, Offset, TimeZone};
use crate::oldtime::Duration as OldDuration;
use crate::DateTime;
use crate::Weekday;
use crate::{Datelike, Timelike};

/// Parsed parts of date and time. There are two classes of methods:
///
/// - `set_*` methods try to set given field(s) while checking for the consistency.
///   It may or may not check for the range constraint immediately (for efficiency reasons).
///
/// - `to_*` methods try to make a concrete date and time value out of set fields.
///   It fully checks any remaining out-of-range conditions and inconsistent/impossible fields.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Parsed {
    /// Year.
    ///
    /// This can be negative unlike [`year_div_100`](#structfield.year_div_100)
    /// and [`year_mod_100`](#structfield.year_mod_100) fields.
    pub year: Option<i32>,

    /// Year divided by 100. Implies that the year is >= 1 BCE when set.
    ///
    /// Due to the common usage, if this field is missing but
    /// [`year_mod_100`](#structfield.year_mod_100) is present,
    /// it is inferred to 19 when `year_mod_100 >= 70` and 20 otherwise.
    pub year_div_100: Option<i32>,

    /// Year modulo 100. Implies that the year is >= 1 BCE when set.
    pub year_mod_100: Option<i32>,

    /// Year in the [ISO week date](../naive/struct.NaiveDate.html#week-date).
    ///
    /// This can be negative unlike [`isoyear_div_100`](#structfield.isoyear_div_100) and
    /// [`isoyear_mod_100`](#structfield.isoyear_mod_100) fields.
    pub isoyear: Option<i32>,

    /// Year in the [ISO week date](../naive/struct.NaiveDate.html#week-date), divided by 100.
    /// Implies that the year is >= 1 BCE when set.
    ///
    /// Due to the common usage, if this field is missing but
    /// [`isoyear_mod_100`](#structfield.isoyear_mod_100) is present,
    /// it is inferred to 19 when `isoyear_mod_100 >= 70` and 20 otherwise.
    pub isoyear_div_100: Option<i32>,

    /// Year in the [ISO week date](../naive/struct.NaiveDate.html#week-date), modulo 100.
    /// Implies that the year is >= 1 BCE when set.
    pub isoyear_mod_100: Option<i32>,

    /// Month (1--12).
    pub month: Option<u32>,

    /// Week number, where the week 1 starts at the first Sunday of January
    /// (0--53, 1--53 or 1--52 depending on the year).
    pub week_from_sun: Option<u32>,

    /// Week number, where the week 1 starts at the first Monday of January
    /// (0--53, 1--53 or 1--52 depending on the year).
    pub week_from_mon: Option<u32>,

    /// [ISO week number](../naive/struct.NaiveDate.html#week-date)
    /// (1--52 or 1--53 depending on the year).
    pub isoweek: Option<u32>,

    /// Day of the week.
    pub weekday: Option<Weekday>,

    /// Day of the year (1--365 or 1--366 depending on the year).
    pub ordinal: Option<u32>,

    /// Day of the month (1--28, 1--29, 1--30 or 1--31 depending on the month).
    pub day: Option<u32>,

    /// Hour number divided by 12 (0--1). 0 indicates AM and 1 indicates PM.
    pub hour_div_12: Option<u32>,

    /// Hour number modulo 12 (0--11).
    pub hour_mod_12: Option<u32>,

    /// Minute number (0--59).
    pub minute: Option<u32>,

    /// Second number (0--60, accounting for leap seconds).
    pub second: Option<u32>,

    /// The number of nanoseconds since the whole second (0--999,999,999).
    pub nanosecond: Option<u32>,

    /// The number of non-leap seconds since the midnight UTC on January 1, 1970.
    ///
    /// This can be off by one if [`second`](#structfield.second) is 60 (a leap second).
    pub timestamp: Option<i64>,

    /// Offset from the local time to UTC, in seconds.
    pub offset: Option<i32>,

    /// A dummy field to make this type not fully destructible (required for API stability).
    _dummy: (),
}

/// Checks if `old` is either empty or has the same value as `new` (i.e. "consistent"),
/// and if it is empty, set `old` to `new` as well.
#[inline]
fn set_if_consistent<T: PartialEq>(old: &mut Option<T>, new: T) -> ParseResult<()> {
    if let Some(ref old) = *old {
        if *old == new {
            Ok(())
        } else {
            Err(IMPOSSIBLE)
        }
    } else {
        *old = Some(new);
        Ok(())
    }
}

impl Parsed {
    /// Returns the initial value of parsed parts.
    pub fn new() -> Parsed {
        Parsed::default()
    }

    /// Tries to set the [`year`](#structfield.year) field from given value.
    #[inline]
    pub fn set_year(&mut self, value: i64) -> ParseResult<()> {
        set_if_consistent(&mut self.year, value.to_i32().ok_or(OUT_OF_RANGE)?)
    }

    /// Tries to set the [`year_div_100`](#structfield.year_div_100) field from given value.
    #[inline]
    pub fn set_year_div_100(&mut self, value: i64) -> ParseResult<()> {
        if value < 0 {
            return Err(OUT_OF_RANGE);
        }
        set_if_consistent(&mut self.year_div_100, value.to_i32().ok_or(OUT_OF_RANGE)?)
    }

    /// Tries to set the [`year_mod_100`](#structfield.year_mod_100) field from given value.
    #[inline]
    pub fn set_year_mod_100(&mut self, value: i64) -> ParseResult<()> {
        if value < 0 {
            return Err(OUT_OF_RANGE);
        }
        set_if_consistent(&mut self.year_mod_100, value.to_i32().ok_or(OUT_OF_RANGE)?)
    }

    /// Tries to set the [`isoyear`](#structfield.isoyear) field from given value.
    #[inline]
    pub fn set_isoyear(&mut self, value: i64) -> ParseResult<()> {
        set_if_consistent(&mut self.isoyear, value.to_i32().ok_or(OUT_OF_RANGE)?)
    }

    /// Tries to set the [`isoyear_div_100`](#structfield.isoyear_div_100) field from given value.
    #[inline]
    pub fn set_isoyear_div_100(&mut self, value: i64) -> ParseResult<()> {
        if value < 0 {
            return Err(OUT_OF_RANGE);
        }
        set_if_consistent(&mut self.isoyear_div_100, value.to_i32().ok_or(OUT_OF_RANGE)?)
    }

    /// Tries to set the [`isoyear_mod_100`](#structfield.isoyear_mod_100) field from given value.
    #[inline]
    pub fn set_isoyear_mod_100(&mut self, value: i64) -> ParseResult<()> {
        if value < 0 {
            return Err(OUT_OF_RANGE);
        }
        set_if_consistent(&mut self.isoyear_mod_100, value.to_i32().ok_or(OUT_OF_RANGE)?)
    }

    /// Tries to set the [`month`](#structfield.month) field from given value.
    #[inline]
    pub fn set_month(&mut self, value: i64) -> ParseResult<()> {
        set_if_consistent(&mut self.month, value.to_u32().ok_or(OUT_OF_RANGE)?)
    }

    /// Tries to set the [`week_from_sun`](#structfield.week_from_sun) field from given value.
    #[inline]
    pub fn set_week_from_sun(&mut self, value: i64) -> ParseResult<()> {
        set_if_consistent(&mut self.week_from_sun, value.to_u32().ok_or(OUT_OF_RANGE)?)
    }

    /// Tries to set the [`week_from_mon`](#structfield.week_from_mon) field from given value.
    #[inline]
    pub fn set_week_from_mon(&mut self, value: i64) -> ParseResult<()> {
        set_if_consistent(&mut self.week_from_mon, value.to_u32().ok_or(OUT_OF_RANGE)?)
    }

    /// Tries to set the [`isoweek`](#structfield.isoweek) field from given value.
    #[inline]
    pub fn set_isoweek(&mut self, value: i64) -> ParseResult<()> {
        set_if_consistent(&mut self.isoweek, value.to_u32().ok_or(OUT_OF_RANGE)?)
    }

    /// Tries to set the [`weekday`](#structfield.weekday) field from given value.
    #[inline]
    pub fn set_weekday(&mut self, value: Weekday) -> ParseResult<()> {
        set_if_consistent(&mut self.weekday, value)
    }

    /// Tries to set the [`ordinal`](#structfield.ordinal) field from given value.
    #[inline]
    pub fn set_ordinal(&mut self, value: i64) -> ParseResult<()> {
        set_if_consistent(&mut self.ordinal, value.to_u32().ok_or(OUT_OF_RANGE)?)
    }

    /// Tries to set the [`day`](#structfield.day) field from given value.
    #[inline]
    pub fn set_day(&mut self, value: i64) -> ParseResult<()> {
        set_if_consistent(&mut self.day, value.to_u32().ok_or(OUT_OF_RANGE)?)
    }

    /// Tries to set the [`hour_div_12`](#structfield.hour_div_12) field from given value.
    /// (`false` for AM, `true` for PM)
    #[inline]
    pub fn set_ampm(&mut self, value: bool) -> ParseResult<()> {
        set_if_consistent(&mut self.hour_div_12, u32::from(value))
    }

    /// Tries to set the [`hour_mod_12`](#structfield.hour_mod_12) field from
    /// given hour number in 12-hour clocks.
    #[inline]
    pub fn set_hour12(&mut self, value: i64) -> ParseResult<()> {
        if !(1..=12).contains(&value) {
            return Err(OUT_OF_RANGE);
        }
        set_if_consistent(&mut self.hour_mod_12, value as u32 % 12)
    }

    /// Tries to set both [`hour_div_12`](#structfield.hour_div_12) and
    /// [`hour_mod_12`](#structfield.hour_mod_12) fields from given value.
    #[inline]
    pub fn set_hour(&mut self, value: i64) -> ParseResult<()> {
        let v = value.to_u32().ok_or(OUT_OF_RANGE)?;
        set_if_consistent(&mut self.hour_div_12, v / 12)?;
        set_if_consistent(&mut self.hour_mod_12, v % 12)?;
        Ok(())
    }

    /// Tries to set the [`minute`](#structfield.minute) field from given value.
    #[inline]
    pub fn set_minute(&mut self, value: i64) -> ParseResult<()> {
        set_if_consistent(&mut self.minute, value.to_u32().ok_or(OUT_OF_RANGE)?)
    }

    /// Tries to set the [`second`](#structfield.second) field from given value.
    #[inline]
    pub fn set_second(&mut self, value: i64) -> ParseResult<()> {
        set_if_consistent(&mut self.second, value.to_u32().ok_or(OUT_OF_RANGE)?)
    }

    /// Tries to set the [`nanosecond`](#structfield.nanosecond) field from given value.
    #[inline]
    pub fn set_nanosecond(&mut self, value: i64) -> ParseResult<()> {
        set_if_consistent(&mut self.nanosecond, value.to_u32().ok_or(OUT_OF_RANGE)?)
    }

    /// Tries to set the [`timestamp`](#structfield.timestamp) field from given value.
    #[inline]
    pub fn set_timestamp(&mut self, value: i64) -> ParseResult<()> {
        set_if_consistent(&mut self.timestamp, value)
    }

    /// Tries to set the [`offset`](#structfield.offset) field from given value.
    #[inline]
    pub fn set_offset(&mut self, value: i64) -> ParseResult<()> {
        set_if_consistent(&mut self.offset, value.to_i32().ok_or(OUT_OF_RANGE)?)
    }

    /// Returns a parsed naive date out of given fields.
    ///
    /// This method is able to determine the date from given subset of fields:
    ///
    /// - Year, month, day.
    /// - Year, day of the year (ordinal).
    /// - Year, week number counted from Sunday or Monday, day of the week.
    /// - ISO week date.
    ///
    /// Gregorian year and ISO week date year can have their century number (`*_div_100`) omitted,
    /// the two-digit year is used to guess the century number then.
    pub fn to_naive_date(&self) -> ParseResult<NaiveDate> {
        fn resolve_year(
            y: Option<i32>,
            q: Option<i32>,
            r: Option<i32>,
        ) -> ParseResult<Option<i32>> {
            match (y, q, r) {
                // if there is no further information, simply return the given full year.
                // this is a common case, so let's avoid division here.
                (y, None, None) => Ok(y),

                // if there is a full year *and* also quotient and/or modulo,
                // check if present quotient and/or modulo is consistent to the full year.
                // since the presence of those fields means a positive full year,
                // we should filter a negative full year first.
                (Some(y), q, r @ Some(0..=99)) | (Some(y), q, r @ None) => {
                    if y < 0 {
                        return Err(OUT_OF_RANGE);
                    }
                    let (q_, r_) = div_rem(y, 100);
                    if q.unwrap_or(q_) == q_ && r.unwrap_or(r_) == r_ {
                        Ok(Some(y))
                    } else {
                        Err(IMPOSSIBLE)
                    }
                }

                // the full year is missing but we have quotient and modulo.
                // reconstruct the full year. make sure that the result is always positive.
                (None, Some(q), Some(r @ 0..=99)) => {
                    if q < 0 {
                        return Err(OUT_OF_RANGE);
                    }
                    let y = q.checked_mul(100).and_then(|v| v.checked_add(r));
                    Ok(Some(y.ok_or(OUT_OF_RANGE)?))
                }

                // we only have modulo. try to interpret a modulo as a conventional two-digit year.
                // note: we are affected by Rust issue #18060. avoid multiple range patterns.
                (None, None, Some(r @ 0..=99)) => Ok(Some(r + if r < 70 { 2000 } else { 1900 })),

                // otherwise it is an out-of-bound or insufficient condition.
                (None, Some(_), None) => Err(NOT_ENOUGH),
                (_, _, Some(_)) => Err(OUT_OF_RANGE),
            }
        }

        let given_year = resolve_year(self.year, self.year_div_100, self.year_mod_100)?;
        let given_isoyear = resolve_year(self.isoyear, self.isoyear_div_100, self.isoyear_mod_100)?;

        // verify the normal year-month-day date.
        let verify_ymd = |date: NaiveDate| {
            let year = date.year();
            let (year_div_100, year_mod_100) = if year >= 0 {
                let (q, r) = div_rem(year, 100);
                (Some(q), Some(r))
            } else {
                (None, None) // they should be empty to be consistent
            };
            let month = date.month();
            let day = date.day();
            self.year.unwrap_or(year) == year
                && self.year_div_100.or(year_div_100) == year_div_100
                && self.year_mod_100.or(year_mod_100) == year_mod_100
                && self.month.unwrap_or(month) == month
                && self.day.unwrap_or(day) == day
        };

        // verify the ISO week date.
        let verify_isoweekdate = |date: NaiveDate| {
            let week = date.iso_week();
            let isoyear = week.year();
            let isoweek = week.week();
            let weekday = date.weekday();
            let (isoyear_div_100, isoyear_mod_100) = if isoyear >= 0 {
                let (q, r) = div_rem(isoyear, 100);
                (Some(q), Some(r))
            } else {
                (None, None) // they should be empty to be consistent
            };
            self.isoyear.unwrap_or(isoyear) == isoyear
                && self.isoyear_div_100.or(isoyear_div_100) == isoyear_div_100
                && self.isoyear_mod_100.or(isoyear_mod_100) == isoyear_mod_100
                && self.isoweek.unwrap_or(isoweek) == isoweek
                && self.weekday.unwrap_or(weekday) == weekday
        };

        // verify the ordinal and other (non-ISO) week dates.
        let verify_ordinal = |date: NaiveDate| {
            let ordinal = date.ordinal();
            let weekday = date.weekday();
            let week_from_sun = (ordinal as i32 - weekday.num_days_from_sunday() as i32 + 7) / 7;
            let week_from_mon = (ordinal as i32 - weekday.num_days_from_monday() as i32 + 7) / 7;
            self.ordinal.unwrap_or(ordinal) == ordinal
                && self.week_from_sun.map_or(week_from_sun, |v| v as i32) == week_from_sun
                && self.week_from_mon.map_or(week_from_mon, |v| v as i32) == week_from_mon
        };

        // test several possibilities.
        // tries to construct a full `NaiveDate` as much as possible, then verifies that
        // it is consistent with other given fields.
        let (verified, parsed_date) = match (given_year, given_isoyear, self) {
            (Some(year), _, &Parsed { month: Some(month), day: Some(day), .. }) => {
                // year, month, day
                let date = NaiveDate::from_ymd_opt(year, month, day).ok_or(OUT_OF_RANGE)?;
                (verify_isoweekdate(date) && verify_ordinal(date), date)
            }

            (Some(year), _, &Parsed { ordinal: Some(ordinal), .. }) => {
                // year, day of the year
                let date = NaiveDate::from_yo_opt(year, ordinal).ok_or(OUT_OF_RANGE)?;
                (verify_ymd(date) && verify_isoweekdate(date) && verify_ordinal(date), date)
            }

            (
                Some(year),
                _,
                &Parsed { week_from_sun: Some(week_from_sun), weekday: Some(weekday), .. },
            ) => {
                // year, week (starting at 1st Sunday), day of the week
                let newyear = NaiveDate::from_yo_opt(year, 1).ok_or(OUT_OF_RANGE)?;
                let firstweek = match newyear.weekday() {
                    Weekday::Sun => 0,
                    Weekday::Mon => 6,
                    Weekday::Tue => 5,
                    Weekday::Wed => 4,
                    Weekday::Thu => 3,
                    Weekday::Fri => 2,
                    Weekday::Sat => 1,
                };

                // `firstweek+1`-th day of January is the beginning of the week 1.
                if week_from_sun > 53 {
                    return Err(OUT_OF_RANGE);
                } // can it overflow?
                let ndays = firstweek
                    + (week_from_sun as i32 - 1) * 7
                    + weekday.num_days_from_sunday() as i32;
                let date = newyear
                    .checked_add_signed(OldDuration::days(i64::from(ndays)))
                    .ok_or(OUT_OF_RANGE)?;
                if date.year() != year {
                    return Err(OUT_OF_RANGE);
                } // early exit for correct error

                (verify_ymd(date) && verify_isoweekdate(date) && verify_ordinal(date), date)
            }

            (
                Some(year),
                _,
                &Parsed { week_from_mon: Some(week_from_mon), weekday: Some(weekday), .. },
            ) => {
                // year, week (starting at 1st Monday), day of the week
                let newyear = NaiveDate::from_yo_opt(year, 1).ok_or(OUT_OF_RANGE)?;
                let firstweek = match newyear.weekday() {
                    Weekday::Sun => 1,
                    Weekday::Mon => 0,
                    Weekday::Tue => 6,
                    Weekday::Wed => 5,
                    Weekday::Thu => 4,
                    Weekday::Fri => 3,
                    Weekday::Sat => 2,
                };

                // `firstweek+1`-th day of January is the beginning of the week 1.
                if week_from_mon > 53 {
                    return Err(OUT_OF_RANGE);
                } // can it overflow?
                let ndays = firstweek
                    + (week_from_mon as i32 - 1) * 7
                    + weekday.num_days_from_monday() as i32;
                let date = newyear
                    .checked_add_signed(OldDuration::days(i64::from(ndays)))
                    .ok_or(OUT_OF_RANGE)?;
                if date.year() != year {
                    return Err(OUT_OF_RANGE);
                } // early exit for correct error

                (verify_ymd(date) && verify_isoweekdate(date) && verify_ordinal(date), date)
            }

            (_, Some(isoyear), &Parsed { isoweek: Some(isoweek), weekday: Some(weekday), .. }) => {
                // ISO year, week, day of the week
                let date = NaiveDate::from_isoywd_opt(isoyear, isoweek, weekday);
                let date = date.ok_or(OUT_OF_RANGE)?;
                (verify_ymd(date) && verify_ordinal(date), date)
            }

            (_, _, _) => return Err(NOT_ENOUGH),
        };

        if verified {
            Ok(parsed_date)
        } else {
            Err(IMPOSSIBLE)
        }
    }

    /// Returns a parsed naive time out of given fields.
    ///
    /// This method is able to determine the time from given subset of fields:
    ///
    /// - Hour, minute. (second and nanosecond assumed to be 0)
    /// - Hour, minute, second. (nanosecond assumed to be 0)
    /// - Hour, minute, second, nanosecond.
    ///
    /// It is able to handle leap seconds when given second is 60.
    pub fn to_naive_time(&self) -> ParseResult<NaiveTime> {
        let hour_div_12 = match self.hour_div_12 {
            Some(v @ 0..=1) => v,
            Some(_) => return Err(OUT_OF_RANGE),
            None => return Err(NOT_ENOUGH),
        };
        let hour_mod_12 = match self.hour_mod_12 {
            Some(v @ 0..=11) => v,
            Some(_) => return Err(OUT_OF_RANGE),
            None => return Err(NOT_ENOUGH),
        };
        let hour = hour_div_12 * 12 + hour_mod_12;

        let minute = match self.minute {
            Some(v @ 0..=59) => v,
            Some(_) => return Err(OUT_OF_RANGE),
            None => return Err(NOT_ENOUGH),
        };

        // we allow omitting seconds or nanoseconds, but they should be in the range.
        let (second, mut nano) = match self.second.unwrap_or(0) {
            v @ 0..=59 => (v, 0),
            60 => (59, 1_000_000_000),
            _ => return Err(OUT_OF_RANGE),
        };
        nano += match self.nanosecond {
            Some(v @ 0..=999_999_999) if self.second.is_some() => v,
            Some(0..=999_999_999) => return Err(NOT_ENOUGH), // second is missing
            Some(_) => return Err(OUT_OF_RANGE),
            None => 0,
        };

        NaiveTime::from_hms_nano_opt(hour, minute, second, nano).ok_or(OUT_OF_RANGE)
    }

    /// Returns a parsed naive date and time out of given fields,
    /// except for the [`offset`](#structfield.offset) field (assumed to have a given value).
    /// This is required for parsing a local time or other known-timezone inputs.
    ///
    /// This method is able to determine the combined date and time
    /// from date and time fields or a single [`timestamp`](#structfield.timestamp) field.
    /// Either way those fields have to be consistent to each other.
    pub fn to_naive_datetime_with_offset(&self, offset: i32) -> ParseResult<NaiveDateTime> {
        let date = self.to_naive_date();
        let time = self.to_naive_time();
        if let (Ok(date), Ok(time)) = (date, time) {
            let datetime = date.and_time(time);

            // verify the timestamp field if any
            // the following is safe, `timestamp` is very limited in range
            let timestamp = datetime.timestamp() - i64::from(offset);
            if let Some(given_timestamp) = self.timestamp {
                // if `datetime` represents a leap second, it might be off by one second.
                if given_timestamp != timestamp
                    && !(datetime.nanosecond() >= 1_000_000_000 && given_timestamp == timestamp + 1)
                {
                    return Err(IMPOSSIBLE);
                }
            }

            Ok(datetime)
        } else if let Some(timestamp) = self.timestamp {
            use super::ParseError as PE;
            use super::ParseErrorKind::{Impossible, OutOfRange};

            // if date and time is problematic already, there is no point proceeding.
            // we at least try to give a correct error though.
            match (date, time) {
                (Err(PE(OutOfRange)), _) | (_, Err(PE(OutOfRange))) => return Err(OUT_OF_RANGE),
                (Err(PE(Impossible)), _) | (_, Err(PE(Impossible))) => return Err(IMPOSSIBLE),
                (_, _) => {} // one of them is insufficient
            }

            // reconstruct date and time fields from timestamp
            let ts = timestamp.checked_add(i64::from(offset)).ok_or(OUT_OF_RANGE)?;
            let datetime = NaiveDateTime::from_timestamp_opt(ts, 0);
            let mut datetime = datetime.ok_or(OUT_OF_RANGE)?;

            // fill year, ordinal, hour, minute and second fields from timestamp.
            // if existing fields are consistent, this will allow the full date/time reconstruction.
            let mut parsed = self.clone();
            if parsed.second == Some(60) {
                // `datetime.second()` cannot be 60, so this is the only case for a leap second.
                match datetime.second() {
                    // it's okay, just do not try to overwrite the existing field.
                    59 => {}
                    // `datetime` is known to be off by one second.
                    0 => {
                        datetime -= OldDuration::seconds(1);
                    }
                    // otherwise it is impossible.
                    _ => return Err(IMPOSSIBLE),
                }
            // ...and we have the correct candidates for other fields.
            } else {
                parsed.set_second(i64::from(datetime.second()))?;
            }
            parsed.set_year(i64::from(datetime.year()))?;
            parsed.set_ordinal(i64::from(datetime.ordinal()))?; // more efficient than ymd
            parsed.set_hour(i64::from(datetime.hour()))?;
            parsed.set_minute(i64::from(datetime.minute()))?;

            // validate other fields (e.g. week) and return
            let date = parsed.to_naive_date()?;
            let time = parsed.to_naive_time()?;
            Ok(date.and_time(time))
        } else {
            // reproduce the previous error(s)
            date?;
            time?;
            unreachable!()
        }
    }

    /// Returns a parsed fixed time zone offset out of given fields.
    pub fn to_fixed_offset(&self) -> ParseResult<FixedOffset> {
        self.offset.and_then(FixedOffset::east_opt).ok_or(OUT_OF_RANGE)
    }

    /// Returns a parsed timezone-aware date and time out of given fields.
    ///
    /// This method is able to determine the combined date and time
    /// from date and time fields or a single [`timestamp`](#structfield.timestamp) field,
    /// plus a time zone offset.
    /// Either way those fields have to be consistent to each other.
    pub fn to_datetime(&self) -> ParseResult<DateTime<FixedOffset>> {
        let offset = self.offset.ok_or(NOT_ENOUGH)?;
        let datetime = self.to_naive_datetime_with_offset(offset)?;
        let offset = FixedOffset::east_opt(offset).ok_or(OUT_OF_RANGE)?;

        // this is used to prevent an overflow when calling FixedOffset::from_local_datetime
        datetime
            .checked_sub_signed(OldDuration::seconds(i64::from(offset.local_minus_utc())))
            .ok_or(OUT_OF_RANGE)?;

        match offset.from_local_datetime(&datetime) {
            LocalResult::None => Err(IMPOSSIBLE),
            LocalResult::Single(t) => Ok(t),
            LocalResult::Ambiguous(..) => Err(NOT_ENOUGH),
        }
    }

    /// Returns a parsed timezone-aware date and time out of given fields,
    /// with an additional `TimeZone` used to interpret and validate the local date.
    ///
    /// This method is able to determine the combined date and time
    /// from date and time fields or a single [`timestamp`](#structfield.timestamp) field,
    /// plus a time zone offset.
    /// Either way those fields have to be consistent to each other.
    /// If parsed fields include an UTC offset, it also has to be consistent to
    /// [`offset`](#structfield.offset).
    pub fn to_datetime_with_timezone<Tz: TimeZone>(&self, tz: &Tz) -> ParseResult<DateTime<Tz>> {
        // if we have `timestamp` specified, guess an offset from that.
        let mut guessed_offset = 0;
        if let Some(timestamp) = self.timestamp {
            // make a naive `DateTime` from given timestamp and (if any) nanosecond.
            // an empty `nanosecond` is always equal to zero, so missing nanosecond is fine.
            let nanosecond = self.nanosecond.unwrap_or(0);
            let dt = NaiveDateTime::from_timestamp_opt(timestamp, nanosecond);
            let dt = dt.ok_or(OUT_OF_RANGE)?;
            guessed_offset = tz.offset_from_utc_datetime(&dt).fix().local_minus_utc();
        }

        // checks if the given `DateTime` has a consistent `Offset` with given `self.offset`.
        let check_offset = |dt: &DateTime<Tz>| {
            if let Some(offset) = self.offset {
                dt.offset().fix().local_minus_utc() == offset
            } else {
                true
            }
        };

        // `guessed_offset` should be correct when `self.timestamp` is given.
        // it will be 0 otherwise, but this is fine as the algorithm ignores offset for that case.
        let datetime = self.to_naive_datetime_with_offset(guessed_offset)?;
        match tz.from_local_datetime(&datetime) {
            LocalResult::None => Err(IMPOSSIBLE),
            LocalResult::Single(t) => {
                if check_offset(&t) {
                    Ok(t)
                } else {
                    Err(IMPOSSIBLE)
                }
            }
            LocalResult::Ambiguous(min, max) => {
                // try to disambiguate two possible local dates by offset.
                match (check_offset(&min), check_offset(&max)) {
                    (false, false) => Err(IMPOSSIBLE),
                    (false, true) => Ok(max),
                    (true, false) => Ok(min),
                    (true, true) => Err(NOT_ENOUGH),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{IMPOSSIBLE, NOT_ENOUGH, OUT_OF_RANGE};
    use super::Parsed;
    use crate::naive::{NaiveDate, NaiveTime};
    use crate::offset::{FixedOffset, TimeZone, Utc};
    use crate::Datelike;
    use crate::Weekday::*;

    #[test]
    fn test_parsed_set_fields() {
        // year*, isoyear*
        let mut p = Parsed::new();
        assert_eq!(p.set_year(1987), Ok(()));
        assert_eq!(p.set_year(1986), Err(IMPOSSIBLE));
        assert_eq!(p.set_year(1988), Err(IMPOSSIBLE));
        assert_eq!(p.set_year(1987), Ok(()));
        assert_eq!(p.set_year_div_100(20), Ok(())); // independent to `year`
        assert_eq!(p.set_year_div_100(21), Err(IMPOSSIBLE));
        assert_eq!(p.set_year_div_100(19), Err(IMPOSSIBLE));
        assert_eq!(p.set_year_mod_100(37), Ok(())); // ditto
        assert_eq!(p.set_year_mod_100(38), Err(IMPOSSIBLE));
        assert_eq!(p.set_year_mod_100(36), Err(IMPOSSIBLE));

        let mut p = Parsed::new();
        assert_eq!(p.set_year(0), Ok(()));
        assert_eq!(p.set_year_div_100(0), Ok(()));
        assert_eq!(p.set_year_mod_100(0), Ok(()));

        let mut p = Parsed::new();
        assert_eq!(p.set_year_div_100(-1), Err(OUT_OF_RANGE));
        assert_eq!(p.set_year_mod_100(-1), Err(OUT_OF_RANGE));
        assert_eq!(p.set_year(-1), Ok(()));
        assert_eq!(p.set_year(-2), Err(IMPOSSIBLE));
        assert_eq!(p.set_year(0), Err(IMPOSSIBLE));

        let mut p = Parsed::new();
        assert_eq!(p.set_year_div_100(0x1_0000_0008), Err(OUT_OF_RANGE));
        assert_eq!(p.set_year_div_100(8), Ok(()));
        assert_eq!(p.set_year_div_100(0x1_0000_0008), Err(OUT_OF_RANGE));

        // month, week*, isoweek, ordinal, day, minute, second, nanosecond, offset
        let mut p = Parsed::new();
        assert_eq!(p.set_month(7), Ok(()));
        assert_eq!(p.set_month(1), Err(IMPOSSIBLE));
        assert_eq!(p.set_month(6), Err(IMPOSSIBLE));
        assert_eq!(p.set_month(8), Err(IMPOSSIBLE));
        assert_eq!(p.set_month(12), Err(IMPOSSIBLE));

        let mut p = Parsed::new();
        assert_eq!(p.set_month(8), Ok(()));
        assert_eq!(p.set_month(0x1_0000_0008), Err(OUT_OF_RANGE));

        // hour
        let mut p = Parsed::new();
        assert_eq!(p.set_hour(12), Ok(()));
        assert_eq!(p.set_hour(11), Err(IMPOSSIBLE));
        assert_eq!(p.set_hour(13), Err(IMPOSSIBLE));
        assert_eq!(p.set_hour(12), Ok(()));
        assert_eq!(p.set_ampm(false), Err(IMPOSSIBLE));
        assert_eq!(p.set_ampm(true), Ok(()));
        assert_eq!(p.set_hour12(12), Ok(()));
        assert_eq!(p.set_hour12(0), Err(OUT_OF_RANGE)); // requires canonical representation
        assert_eq!(p.set_hour12(1), Err(IMPOSSIBLE));
        assert_eq!(p.set_hour12(11), Err(IMPOSSIBLE));

        let mut p = Parsed::new();
        assert_eq!(p.set_ampm(true), Ok(()));
        assert_eq!(p.set_hour12(7), Ok(()));
        assert_eq!(p.set_hour(7), Err(IMPOSSIBLE));
        assert_eq!(p.set_hour(18), Err(IMPOSSIBLE));
        assert_eq!(p.set_hour(19), Ok(()));

        // timestamp
        let mut p = Parsed::new();
        assert_eq!(p.set_timestamp(1_234_567_890), Ok(()));
        assert_eq!(p.set_timestamp(1_234_567_889), Err(IMPOSSIBLE));
        assert_eq!(p.set_timestamp(1_234_567_891), Err(IMPOSSIBLE));
    }

    #[test]
    fn test_parsed_to_naive_date() {
        macro_rules! parse {
            ($($k:ident: $v:expr),*) => (
                Parsed { $($k: Some($v),)* ..Parsed::new() }.to_naive_date()
            )
        }

        let ymd = |y, m, d| Ok(NaiveDate::from_ymd_opt(y, m, d).unwrap());

        // ymd: omission of fields
        assert_eq!(parse!(), Err(NOT_ENOUGH));
        assert_eq!(parse!(year: 1984), Err(NOT_ENOUGH));
        assert_eq!(parse!(year: 1984, month: 1), Err(NOT_ENOUGH));
        assert_eq!(parse!(year: 1984, month: 1, day: 2), ymd(1984, 1, 2));
        assert_eq!(parse!(year: 1984, day: 2), Err(NOT_ENOUGH));
        assert_eq!(parse!(year_div_100: 19), Err(NOT_ENOUGH));
        assert_eq!(parse!(year_div_100: 19, year_mod_100: 84), Err(NOT_ENOUGH));
        assert_eq!(parse!(year_div_100: 19, year_mod_100: 84, month: 1), Err(NOT_ENOUGH));
        assert_eq!(parse!(year_div_100: 19, year_mod_100: 84, month: 1, day: 2), ymd(1984, 1, 2));
        assert_eq!(parse!(year_div_100: 19, year_mod_100: 84, day: 2), Err(NOT_ENOUGH));
        assert_eq!(parse!(year_div_100: 19, month: 1, day: 2), Err(NOT_ENOUGH));
        assert_eq!(parse!(year_mod_100: 70, month: 1, day: 2), ymd(1970, 1, 2));
        assert_eq!(parse!(year_mod_100: 69, month: 1, day: 2), ymd(2069, 1, 2));

        // ymd: out-of-range conditions
        assert_eq!(parse!(year_div_100: 19, year_mod_100: 84, month: 2, day: 29), ymd(1984, 2, 29));
        assert_eq!(
            parse!(year_div_100: 19, year_mod_100: 83, month: 2, day: 29),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(year_div_100: 19, year_mod_100: 83, month: 13, day: 1),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(year_div_100: 19, year_mod_100: 83, month: 12, day: 31),
            ymd(1983, 12, 31)
        );
        assert_eq!(
            parse!(year_div_100: 19, year_mod_100: 83, month: 12, day: 32),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(year_div_100: 19, year_mod_100: 83, month: 12, day: 0),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(year_div_100: 19, year_mod_100: 100, month: 1, day: 1),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(parse!(year_div_100: 19, year_mod_100: -1, month: 1, day: 1), Err(OUT_OF_RANGE));
        assert_eq!(parse!(year_div_100: 0, year_mod_100: 0, month: 1, day: 1), ymd(0, 1, 1));
        assert_eq!(parse!(year_div_100: -1, year_mod_100: 42, month: 1, day: 1), Err(OUT_OF_RANGE));
        let max_year = NaiveDate::MAX.year();
        assert_eq!(
            parse!(year_div_100: max_year / 100,
                          year_mod_100: max_year % 100, month: 1, day: 1),
            ymd(max_year, 1, 1)
        );
        assert_eq!(
            parse!(year_div_100: (max_year + 1) / 100,
                          year_mod_100: (max_year + 1) % 100, month: 1, day: 1),
            Err(OUT_OF_RANGE)
        );

        // ymd: conflicting inputs
        assert_eq!(parse!(year: 1984, year_div_100: 19, month: 1, day: 1), ymd(1984, 1, 1));
        assert_eq!(parse!(year: 1984, year_div_100: 20, month: 1, day: 1), Err(IMPOSSIBLE));
        assert_eq!(parse!(year: 1984, year_mod_100: 84, month: 1, day: 1), ymd(1984, 1, 1));
        assert_eq!(parse!(year: 1984, year_mod_100: 83, month: 1, day: 1), Err(IMPOSSIBLE));
        assert_eq!(
            parse!(year: 1984, year_div_100: 19, year_mod_100: 84, month: 1, day: 1),
            ymd(1984, 1, 1)
        );
        assert_eq!(
            parse!(year: 1984, year_div_100: 18, year_mod_100: 94, month: 1, day: 1),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(year: 1984, year_div_100: 18, year_mod_100: 184, month: 1, day: 1),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(year: -1, year_div_100: 0, year_mod_100: -1, month: 1, day: 1),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(year: -1, year_div_100: -1, year_mod_100: 99, month: 1, day: 1),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(parse!(year: -1, year_div_100: 0, month: 1, day: 1), Err(OUT_OF_RANGE));
        assert_eq!(parse!(year: -1, year_mod_100: 99, month: 1, day: 1), Err(OUT_OF_RANGE));

        // weekdates
        assert_eq!(parse!(year: 2000, week_from_mon: 0), Err(NOT_ENOUGH));
        assert_eq!(parse!(year: 2000, week_from_sun: 0), Err(NOT_ENOUGH));
        assert_eq!(parse!(year: 2000, weekday: Sun), Err(NOT_ENOUGH));
        assert_eq!(parse!(year: 2000, week_from_mon: 0, weekday: Fri), Err(OUT_OF_RANGE));
        assert_eq!(parse!(year: 2000, week_from_sun: 0, weekday: Fri), Err(OUT_OF_RANGE));
        assert_eq!(parse!(year: 2000, week_from_mon: 0, weekday: Sat), ymd(2000, 1, 1));
        assert_eq!(parse!(year: 2000, week_from_sun: 0, weekday: Sat), ymd(2000, 1, 1));
        assert_eq!(parse!(year: 2000, week_from_mon: 0, weekday: Sun), ymd(2000, 1, 2));
        assert_eq!(parse!(year: 2000, week_from_sun: 1, weekday: Sun), ymd(2000, 1, 2));
        assert_eq!(parse!(year: 2000, week_from_mon: 1, weekday: Mon), ymd(2000, 1, 3));
        assert_eq!(parse!(year: 2000, week_from_sun: 1, weekday: Mon), ymd(2000, 1, 3));
        assert_eq!(parse!(year: 2000, week_from_mon: 1, weekday: Sat), ymd(2000, 1, 8));
        assert_eq!(parse!(year: 2000, week_from_sun: 1, weekday: Sat), ymd(2000, 1, 8));
        assert_eq!(parse!(year: 2000, week_from_mon: 1, weekday: Sun), ymd(2000, 1, 9));
        assert_eq!(parse!(year: 2000, week_from_sun: 2, weekday: Sun), ymd(2000, 1, 9));
        assert_eq!(parse!(year: 2000, week_from_mon: 2, weekday: Mon), ymd(2000, 1, 10));
        assert_eq!(parse!(year: 2000, week_from_sun: 52, weekday: Sat), ymd(2000, 12, 30));
        assert_eq!(parse!(year: 2000, week_from_sun: 53, weekday: Sun), ymd(2000, 12, 31));
        assert_eq!(parse!(year: 2000, week_from_sun: 53, weekday: Mon), Err(OUT_OF_RANGE));
        assert_eq!(parse!(year: 2000, week_from_sun: 0xffffffff, weekday: Mon), Err(OUT_OF_RANGE));
        assert_eq!(parse!(year: 2006, week_from_sun: 0, weekday: Sat), Err(OUT_OF_RANGE));
        assert_eq!(parse!(year: 2006, week_from_sun: 1, weekday: Sun), ymd(2006, 1, 1));

        // weekdates: conflicting inputs
        assert_eq!(
            parse!(year: 2000, week_from_mon: 1, week_from_sun: 1, weekday: Sat),
            ymd(2000, 1, 8)
        );
        assert_eq!(
            parse!(year: 2000, week_from_mon: 1, week_from_sun: 2, weekday: Sun),
            ymd(2000, 1, 9)
        );
        assert_eq!(
            parse!(year: 2000, week_from_mon: 1, week_from_sun: 1, weekday: Sun),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(year: 2000, week_from_mon: 2, week_from_sun: 2, weekday: Sun),
            Err(IMPOSSIBLE)
        );

        // ISO weekdates
        assert_eq!(parse!(isoyear: 2004, isoweek: 53), Err(NOT_ENOUGH));
        assert_eq!(parse!(isoyear: 2004, isoweek: 53, weekday: Fri), ymd(2004, 12, 31));
        assert_eq!(parse!(isoyear: 2004, isoweek: 53, weekday: Sat), ymd(2005, 1, 1));
        assert_eq!(parse!(isoyear: 2004, isoweek: 0xffffffff, weekday: Sat), Err(OUT_OF_RANGE));
        assert_eq!(parse!(isoyear: 2005, isoweek: 0, weekday: Thu), Err(OUT_OF_RANGE));
        assert_eq!(parse!(isoyear: 2005, isoweek: 5, weekday: Thu), ymd(2005, 2, 3));
        assert_eq!(parse!(isoyear: 2005, weekday: Thu), Err(NOT_ENOUGH));

        // year and ordinal
        assert_eq!(parse!(ordinal: 123), Err(NOT_ENOUGH));
        assert_eq!(parse!(year: 2000, ordinal: 0), Err(OUT_OF_RANGE));
        assert_eq!(parse!(year: 2000, ordinal: 1), ymd(2000, 1, 1));
        assert_eq!(parse!(year: 2000, ordinal: 60), ymd(2000, 2, 29));
        assert_eq!(parse!(year: 2000, ordinal: 61), ymd(2000, 3, 1));
        assert_eq!(parse!(year: 2000, ordinal: 366), ymd(2000, 12, 31));
        assert_eq!(parse!(year: 2000, ordinal: 367), Err(OUT_OF_RANGE));
        assert_eq!(parse!(year: 2000, ordinal: 0xffffffff), Err(OUT_OF_RANGE));
        assert_eq!(parse!(year: 2100, ordinal: 0), Err(OUT_OF_RANGE));
        assert_eq!(parse!(year: 2100, ordinal: 1), ymd(2100, 1, 1));
        assert_eq!(parse!(year: 2100, ordinal: 59), ymd(2100, 2, 28));
        assert_eq!(parse!(year: 2100, ordinal: 60), ymd(2100, 3, 1));
        assert_eq!(parse!(year: 2100, ordinal: 365), ymd(2100, 12, 31));
        assert_eq!(parse!(year: 2100, ordinal: 366), Err(OUT_OF_RANGE));
        assert_eq!(parse!(year: 2100, ordinal: 0xffffffff), Err(OUT_OF_RANGE));

        // more complex cases
        assert_eq!(
            parse!(year: 2014, month: 12, day: 31, ordinal: 365, isoyear: 2015, isoweek: 1,
                          week_from_sun: 52, week_from_mon: 52, weekday: Wed),
            ymd(2014, 12, 31)
        );
        assert_eq!(
            parse!(year: 2014, month: 12, ordinal: 365, isoyear: 2015, isoweek: 1,
                          week_from_sun: 52, week_from_mon: 52),
            ymd(2014, 12, 31)
        );
        assert_eq!(
            parse!(year: 2014, month: 12, day: 31, ordinal: 365, isoyear: 2014, isoweek: 53,
                          week_from_sun: 52, week_from_mon: 52, weekday: Wed),
            Err(IMPOSSIBLE)
        ); // no ISO week date 2014-W53-3
        assert_eq!(
            parse!(year: 2012, isoyear: 2015, isoweek: 1,
                          week_from_sun: 52, week_from_mon: 52),
            Err(NOT_ENOUGH)
        ); // ambiguous (2014-12-29, 2014-12-30, 2014-12-31)
        assert_eq!(parse!(year_div_100: 20, isoyear_mod_100: 15, ordinal: 366), Err(NOT_ENOUGH));
        // technically unique (2014-12-31) but Chrono gives up
    }

    #[test]
    fn test_parsed_to_naive_time() {
        macro_rules! parse {
            ($($k:ident: $v:expr),*) => (
                Parsed { $($k: Some($v),)* ..Parsed::new() }.to_naive_time()
            )
        }

        let hms = |h, m, s| Ok(NaiveTime::from_hms_opt(h, m, s).unwrap());
        let hmsn = |h, m, s, n| Ok(NaiveTime::from_hms_nano_opt(h, m, s, n).unwrap());

        // omission of fields
        assert_eq!(parse!(), Err(NOT_ENOUGH));
        assert_eq!(parse!(hour_div_12: 0), Err(NOT_ENOUGH));
        assert_eq!(parse!(hour_div_12: 0, hour_mod_12: 1), Err(NOT_ENOUGH));
        assert_eq!(parse!(hour_div_12: 0, hour_mod_12: 1, minute: 23), hms(1, 23, 0));
        assert_eq!(parse!(hour_div_12: 0, hour_mod_12: 1, minute: 23, second: 45), hms(1, 23, 45));
        assert_eq!(
            parse!(hour_div_12: 0, hour_mod_12: 1, minute: 23, second: 45,
                          nanosecond: 678_901_234),
            hmsn(1, 23, 45, 678_901_234)
        );
        assert_eq!(parse!(hour_div_12: 1, hour_mod_12: 11, minute: 45, second: 6), hms(23, 45, 6));
        assert_eq!(parse!(hour_mod_12: 1, minute: 23), Err(NOT_ENOUGH));
        assert_eq!(
            parse!(hour_div_12: 0, hour_mod_12: 1, minute: 23, nanosecond: 456_789_012),
            Err(NOT_ENOUGH)
        );

        // out-of-range conditions
        assert_eq!(parse!(hour_div_12: 2, hour_mod_12: 0, minute: 0), Err(OUT_OF_RANGE));
        assert_eq!(parse!(hour_div_12: 1, hour_mod_12: 12, minute: 0), Err(OUT_OF_RANGE));
        assert_eq!(parse!(hour_div_12: 0, hour_mod_12: 1, minute: 60), Err(OUT_OF_RANGE));
        assert_eq!(
            parse!(hour_div_12: 0, hour_mod_12: 1, minute: 23, second: 61),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(hour_div_12: 0, hour_mod_12: 1, minute: 23, second: 34,
                          nanosecond: 1_000_000_000),
            Err(OUT_OF_RANGE)
        );

        // leap seconds
        assert_eq!(
            parse!(hour_div_12: 0, hour_mod_12: 1, minute: 23, second: 60),
            hmsn(1, 23, 59, 1_000_000_000)
        );
        assert_eq!(
            parse!(hour_div_12: 0, hour_mod_12: 1, minute: 23, second: 60,
                          nanosecond: 999_999_999),
            hmsn(1, 23, 59, 1_999_999_999)
        );
    }

    #[test]
    fn test_parsed_to_naive_datetime_with_offset() {
        macro_rules! parse {
            (offset = $offset:expr; $($k:ident: $v:expr),*) => (
                Parsed { $($k: Some($v),)* ..Parsed::new() }.to_naive_datetime_with_offset($offset)
            );
            ($($k:ident: $v:expr),*) => (parse!(offset = 0; $($k: $v),*))
        }

        let ymdhms = |y, m, d, h, n, s| {
            Ok(NaiveDate::from_ymd_opt(y, m, d).unwrap().and_hms_opt(h, n, s).unwrap())
        };
        let ymdhmsn = |y, m, d, h, n, s, nano| {
            Ok(NaiveDate::from_ymd_opt(y, m, d).unwrap().and_hms_nano_opt(h, n, s, nano).unwrap())
        };

        // omission of fields
        assert_eq!(parse!(), Err(NOT_ENOUGH));
        assert_eq!(
            parse!(year: 2015, month: 1, day: 30,
                          hour_div_12: 1, hour_mod_12: 2, minute: 38),
            ymdhms(2015, 1, 30, 14, 38, 0)
        );
        assert_eq!(
            parse!(year: 1997, month: 1, day: 30,
                          hour_div_12: 1, hour_mod_12: 2, minute: 38, second: 5),
            ymdhms(1997, 1, 30, 14, 38, 5)
        );
        assert_eq!(
            parse!(year: 2012, ordinal: 34, hour_div_12: 0, hour_mod_12: 5,
                          minute: 6, second: 7, nanosecond: 890_123_456),
            ymdhmsn(2012, 2, 3, 5, 6, 7, 890_123_456)
        );
        assert_eq!(parse!(timestamp: 0), ymdhms(1970, 1, 1, 0, 0, 0));
        assert_eq!(parse!(timestamp: 1, nanosecond: 0), ymdhms(1970, 1, 1, 0, 0, 1));
        assert_eq!(parse!(timestamp: 1, nanosecond: 1), ymdhmsn(1970, 1, 1, 0, 0, 1, 1));
        assert_eq!(parse!(timestamp: 1_420_000_000), ymdhms(2014, 12, 31, 4, 26, 40));
        assert_eq!(parse!(timestamp: -0x1_0000_0000), ymdhms(1833, 11, 24, 17, 31, 44));

        // full fields
        assert_eq!(
            parse!(year: 2014, year_div_100: 20, year_mod_100: 14, month: 12, day: 31,
                          ordinal: 365, isoyear: 2015, isoyear_div_100: 20, isoyear_mod_100: 15,
                          isoweek: 1, week_from_sun: 52, week_from_mon: 52, weekday: Wed,
                          hour_div_12: 0, hour_mod_12: 4, minute: 26, second: 40,
                          nanosecond: 12_345_678, timestamp: 1_420_000_000),
            ymdhmsn(2014, 12, 31, 4, 26, 40, 12_345_678)
        );
        assert_eq!(
            parse!(year: 2014, year_div_100: 20, year_mod_100: 14, month: 12, day: 31,
                          ordinal: 365, isoyear: 2015, isoyear_div_100: 20, isoyear_mod_100: 15,
                          isoweek: 1, week_from_sun: 52, week_from_mon: 52, weekday: Wed,
                          hour_div_12: 0, hour_mod_12: 4, minute: 26, second: 40,
                          nanosecond: 12_345_678, timestamp: 1_419_999_999),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(offset = 32400;
                          year: 2014, year_div_100: 20, year_mod_100: 14, month: 12, day: 31,
                          ordinal: 365, isoyear: 2015, isoyear_div_100: 20, isoyear_mod_100: 15,
                          isoweek: 1, week_from_sun: 52, week_from_mon: 52, weekday: Wed,
                          hour_div_12: 0, hour_mod_12: 4, minute: 26, second: 40,
                          nanosecond: 12_345_678, timestamp: 1_419_967_600),
            ymdhmsn(2014, 12, 31, 4, 26, 40, 12_345_678)
        );

        // more timestamps
        let max_days_from_year_1970 =
            NaiveDate::MAX.signed_duration_since(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        let year_0_from_year_1970 = NaiveDate::from_ymd_opt(0, 1, 1)
            .unwrap()
            .signed_duration_since(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        let min_days_from_year_1970 =
            NaiveDate::MIN.signed_duration_since(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        assert_eq!(
            parse!(timestamp: min_days_from_year_1970.num_seconds()),
            ymdhms(NaiveDate::MIN.year(), 1, 1, 0, 0, 0)
        );
        assert_eq!(
            parse!(timestamp: year_0_from_year_1970.num_seconds()),
            ymdhms(0, 1, 1, 0, 0, 0)
        );
        assert_eq!(
            parse!(timestamp: max_days_from_year_1970.num_seconds() + 86399),
            ymdhms(NaiveDate::MAX.year(), 12, 31, 23, 59, 59)
        );

        // leap seconds #1: partial fields
        assert_eq!(parse!(second: 59, timestamp: 1_341_100_798), Err(IMPOSSIBLE));
        assert_eq!(parse!(second: 59, timestamp: 1_341_100_799), ymdhms(2012, 6, 30, 23, 59, 59));
        assert_eq!(parse!(second: 59, timestamp: 1_341_100_800), Err(IMPOSSIBLE));
        assert_eq!(
            parse!(second: 60, timestamp: 1_341_100_799),
            ymdhmsn(2012, 6, 30, 23, 59, 59, 1_000_000_000)
        );
        assert_eq!(
            parse!(second: 60, timestamp: 1_341_100_800),
            ymdhmsn(2012, 6, 30, 23, 59, 59, 1_000_000_000)
        );
        assert_eq!(parse!(second: 0, timestamp: 1_341_100_800), ymdhms(2012, 7, 1, 0, 0, 0));
        assert_eq!(parse!(second: 1, timestamp: 1_341_100_800), Err(IMPOSSIBLE));
        assert_eq!(parse!(second: 60, timestamp: 1_341_100_801), Err(IMPOSSIBLE));

        // leap seconds #2: full fields
        // we need to have separate tests for them since it uses another control flow.
        assert_eq!(
            parse!(year: 2012, ordinal: 182, hour_div_12: 1, hour_mod_12: 11,
                          minute: 59, second: 59, timestamp: 1_341_100_798),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(year: 2012, ordinal: 182, hour_div_12: 1, hour_mod_12: 11,
                          minute: 59, second: 59, timestamp: 1_341_100_799),
            ymdhms(2012, 6, 30, 23, 59, 59)
        );
        assert_eq!(
            parse!(year: 2012, ordinal: 182, hour_div_12: 1, hour_mod_12: 11,
                          minute: 59, second: 59, timestamp: 1_341_100_800),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(year: 2012, ordinal: 182, hour_div_12: 1, hour_mod_12: 11,
                          minute: 59, second: 60, timestamp: 1_341_100_799),
            ymdhmsn(2012, 6, 30, 23, 59, 59, 1_000_000_000)
        );
        assert_eq!(
            parse!(year: 2012, ordinal: 182, hour_div_12: 1, hour_mod_12: 11,
                          minute: 59, second: 60, timestamp: 1_341_100_800),
            ymdhmsn(2012, 6, 30, 23, 59, 59, 1_000_000_000)
        );
        assert_eq!(
            parse!(year: 2012, ordinal: 183, hour_div_12: 0, hour_mod_12: 0,
                          minute: 0, second: 0, timestamp: 1_341_100_800),
            ymdhms(2012, 7, 1, 0, 0, 0)
        );
        assert_eq!(
            parse!(year: 2012, ordinal: 183, hour_div_12: 0, hour_mod_12: 0,
                          minute: 0, second: 1, timestamp: 1_341_100_800),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(year: 2012, ordinal: 182, hour_div_12: 1, hour_mod_12: 11,
                          minute: 59, second: 60, timestamp: 1_341_100_801),
            Err(IMPOSSIBLE)
        );

        // error codes
        assert_eq!(
            parse!(year: 2015, month: 1, day: 20, weekday: Tue,
                          hour_div_12: 2, hour_mod_12: 1, minute: 35, second: 20),
            Err(OUT_OF_RANGE)
        ); // `hour_div_12` is out of range
    }

    #[test]
    fn test_parsed_to_datetime() {
        macro_rules! parse {
            ($($k:ident: $v:expr),*) => (
                Parsed { $($k: Some($v),)* ..Parsed::new() }.to_datetime()
            )
        }

        let ymdhmsn = |y, m, d, h, n, s, nano, off| {
            Ok(FixedOffset::east_opt(off)
                .unwrap()
                .from_local_datetime(
                    &NaiveDate::from_ymd_opt(y, m, d)
                        .unwrap()
                        .and_hms_nano_opt(h, n, s, nano)
                        .unwrap(),
                )
                .unwrap())
        };

        assert_eq!(parse!(offset: 0), Err(NOT_ENOUGH));
        assert_eq!(
            parse!(year: 2014, ordinal: 365, hour_div_12: 0, hour_mod_12: 4,
                          minute: 26, second: 40, nanosecond: 12_345_678),
            Err(NOT_ENOUGH)
        );
        assert_eq!(
            parse!(year: 2014, ordinal: 365, hour_div_12: 0, hour_mod_12: 4,
                          minute: 26, second: 40, nanosecond: 12_345_678, offset: 0),
            ymdhmsn(2014, 12, 31, 4, 26, 40, 12_345_678, 0)
        );
        assert_eq!(
            parse!(year: 2014, ordinal: 365, hour_div_12: 1, hour_mod_12: 1,
                          minute: 26, second: 40, nanosecond: 12_345_678, offset: 32400),
            ymdhmsn(2014, 12, 31, 13, 26, 40, 12_345_678, 32400)
        );
        assert_eq!(
            parse!(year: 2014, ordinal: 365, hour_div_12: 0, hour_mod_12: 1,
                          minute: 42, second: 4, nanosecond: 12_345_678, offset: -9876),
            ymdhmsn(2014, 12, 31, 1, 42, 4, 12_345_678, -9876)
        );
        assert_eq!(
            parse!(year: 2015, ordinal: 1, hour_div_12: 0, hour_mod_12: 4,
                          minute: 26, second: 40, nanosecond: 12_345_678, offset: 86_400),
            Err(OUT_OF_RANGE)
        ); // `FixedOffset` does not support such huge offset
    }

    #[test]
    fn test_parsed_to_datetime_with_timezone() {
        macro_rules! parse {
            ($tz:expr; $($k:ident: $v:expr),*) => (
                Parsed { $($k: Some($v),)* ..Parsed::new() }.to_datetime_with_timezone(&$tz)
            )
        }

        // single result from ymdhms
        assert_eq!(
            parse!(Utc;
                          year: 2014, ordinal: 365, hour_div_12: 0, hour_mod_12: 4,
                          minute: 26, second: 40, nanosecond: 12_345_678, offset: 0),
            Ok(Utc
                .from_local_datetime(
                    &NaiveDate::from_ymd_opt(2014, 12, 31)
                        .unwrap()
                        .and_hms_nano_opt(4, 26, 40, 12_345_678)
                        .unwrap()
                )
                .unwrap())
        );
        assert_eq!(
            parse!(Utc;
                          year: 2014, ordinal: 365, hour_div_12: 1, hour_mod_12: 1,
                          minute: 26, second: 40, nanosecond: 12_345_678, offset: 32400),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(FixedOffset::east_opt(32400).unwrap();
                          year: 2014, ordinal: 365, hour_div_12: 0, hour_mod_12: 4,
                          minute: 26, second: 40, nanosecond: 12_345_678, offset: 0),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(FixedOffset::east_opt(32400).unwrap();
                          year: 2014, ordinal: 365, hour_div_12: 1, hour_mod_12: 1,
                          minute: 26, second: 40, nanosecond: 12_345_678, offset: 32400),
            Ok(FixedOffset::east_opt(32400)
                .unwrap()
                .from_local_datetime(
                    &NaiveDate::from_ymd_opt(2014, 12, 31)
                        .unwrap()
                        .and_hms_nano_opt(13, 26, 40, 12_345_678)
                        .unwrap()
                )
                .unwrap())
        );

        // single result from timestamp
        assert_eq!(
            parse!(Utc; timestamp: 1_420_000_000, offset: 0),
            Ok(Utc.with_ymd_and_hms(2014, 12, 31, 4, 26, 40).unwrap())
        );
        assert_eq!(parse!(Utc; timestamp: 1_420_000_000, offset: 32400), Err(IMPOSSIBLE));
        assert_eq!(
            parse!(FixedOffset::east_opt(32400).unwrap(); timestamp: 1_420_000_000, offset: 0),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(FixedOffset::east_opt(32400).unwrap(); timestamp: 1_420_000_000, offset: 32400),
            Ok(FixedOffset::east_opt(32400)
                .unwrap()
                .with_ymd_and_hms(2014, 12, 31, 13, 26, 40)
                .unwrap())
        );

        // TODO test with a variable time zone (for None and Ambiguous cases)
    }
}
