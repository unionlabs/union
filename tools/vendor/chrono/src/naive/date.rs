// This is a part of Chrono.
// See README.md and LICENSE.txt for details.

//! ISO 8601 calendar date without timezone.

#[cfg(any(feature = "alloc", feature = "std", test))]
use core::borrow::Borrow;
use core::convert::TryFrom;
use core::ops::{Add, AddAssign, RangeInclusive, Sub, SubAssign};
use core::{fmt, str};

use num_integer::div_mod_floor;
use num_traits::ToPrimitive;
#[cfg(feature = "rkyv")]
use rkyv::{Archive, Deserialize, Serialize};

#[cfg(any(feature = "alloc", feature = "std", test))]
use crate::format::DelayedFormat;
use crate::format::{parse, write_hundreds, ParseError, ParseResult, Parsed, StrftimeItems};
use crate::format::{Item, Numeric, Pad};
use crate::month::Months;
use crate::naive::{IsoWeek, NaiveDateTime, NaiveTime};
use crate::oldtime::Duration as OldDuration;
use crate::{Datelike, Duration, Weekday};

use super::internals::{self, DateImpl, Mdf, Of, YearFlags};
use super::isoweek;

const MAX_YEAR: i32 = internals::MAX_YEAR;
const MIN_YEAR: i32 = internals::MIN_YEAR;

//   MAX_YEAR-12-31 minus 0000-01-01
// = ((MAX_YEAR+1)-01-01 minus 0001-01-01) + (0001-01-01 minus 0000-01-01) - 1 day
// = ((MAX_YEAR+1)-01-01 minus 0001-01-01) + 365 days
// = MAX_YEAR * 365 + (# of leap years from 0001 to MAX_YEAR) + 365 days
#[cfg(test)] // only used for testing
const MAX_DAYS_FROM_YEAR_0: i32 =
    MAX_YEAR * 365 + MAX_YEAR / 4 - MAX_YEAR / 100 + MAX_YEAR / 400 + 365;

//   MIN_YEAR-01-01 minus 0000-01-01
// = (MIN_YEAR+400n+1)-01-01 minus (400n+1)-01-01
// = ((MIN_YEAR+400n+1)-01-01 minus 0001-01-01) - ((400n+1)-01-01 minus 0001-01-01)
// = ((MIN_YEAR+400n+1)-01-01 minus 0001-01-01) - 146097n days
//
// n is set to 1000 for convenience.
#[cfg(test)] // only used for testing
const MIN_DAYS_FROM_YEAR_0: i32 = (MIN_YEAR + 400_000) * 365 + (MIN_YEAR + 400_000) / 4
    - (MIN_YEAR + 400_000) / 100
    + (MIN_YEAR + 400_000) / 400
    - 146_097_000;

#[cfg(test)] // only used for testing, but duplicated in naive::datetime
const MAX_BITS: usize = 44;

/// A week represented by a [`NaiveDate`] and a [`Weekday`] which is the first
/// day of the week.
#[derive(Debug)]
pub struct NaiveWeek {
    date: NaiveDate,
    start: Weekday,
}

impl NaiveWeek {
    /// Returns a date representing the first day of the week.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::{NaiveDate, Weekday};
    ///
    /// let date = NaiveDate::from_ymd_opt(2022, 4, 18).unwrap();
    /// let week = date.week(Weekday::Mon);
    /// assert!(week.first_day() <= date);
    /// ```
    #[inline]
    pub fn first_day(&self) -> NaiveDate {
        let start = self.start.num_days_from_monday();
        let end = self.date.weekday().num_days_from_monday();
        let days = if start > end { 7 - start + end } else { end - start };
        self.date - Duration::days(days.into())
    }

    /// Returns a date representing the last day of the week.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::{NaiveDate, Weekday};
    ///
    /// let date = NaiveDate::from_ymd_opt(2022, 4, 18).unwrap();
    /// let week = date.week(Weekday::Mon);
    /// assert!(week.last_day() >= date);
    /// ```
    #[inline]
    pub fn last_day(&self) -> NaiveDate {
        self.first_day() + Duration::days(6)
    }

    /// Returns a [`RangeInclusive<T>`] representing the whole week bounded by
    /// [first_day](./struct.NaiveWeek.html#method.first_day) and
    /// [last_day](./struct.NaiveWeek.html#method.last_day) functions.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::{NaiveDate, Weekday};
    ///
    /// let date = NaiveDate::from_ymd_opt(2022, 4, 18).unwrap();
    /// let week = date.week(Weekday::Mon);
    /// let days = week.days();
    /// assert!(days.contains(&date));
    /// ```
    #[inline]
    pub fn days(&self) -> RangeInclusive<NaiveDate> {
        self.first_day()..=self.last_day()
    }
}

/// A duration in calendar days.
///
/// This is useful becuase when using `Duration` it is possible
/// that adding `Duration::days(1)` doesn't increment the day value as expected due to it being a
/// fixed number of seconds. This difference applies only when dealing with `DateTime<TimeZone>` data types
/// and in other cases `Duration::days(n)` and `Days::new(n)` are equivalent.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct Days(pub(crate) u64);

impl Days {
    /// Construct a new `Days` from a number of months
    pub fn new(num: u64) -> Self {
        Self(num)
    }
}

/// ISO 8601 calendar date without timezone.
/// Allows for every [proleptic Gregorian date](#calendar-date)
/// from Jan 1, 262145 BCE to Dec 31, 262143 CE.
/// Also supports the conversion from ISO 8601 ordinal and week date.
///
/// # Calendar Date
///
/// The ISO 8601 **calendar date** follows the proleptic Gregorian calendar.
/// It is like a normal civil calendar but note some slight differences:
///
/// * Dates before the Gregorian calendar's inception in 1582 are defined via the extrapolation.
///   Be careful, as historical dates are often noted in the Julian calendar and others
///   and the transition to Gregorian may differ across countries (as late as early 20C).
///
///   (Some example: Both Shakespeare from Britain and Cervantes from Spain seemingly died
///   on the same calendar date---April 23, 1616---but in the different calendar.
///   Britain used the Julian calendar at that time, so Shakespeare's death is later.)
///
/// * ISO 8601 calendars has the year 0, which is 1 BCE (a year before 1 CE).
///   If you need a typical BCE/BC and CE/AD notation for year numbers,
///   use the [`Datelike::year_ce`](../trait.Datelike.html#method.year_ce) method.
///
/// # Week Date
///
/// The ISO 8601 **week date** is a triple of year number, week number
/// and [day of the week](../enum.Weekday.html) with the following rules:
///
/// * A week consists of Monday through Sunday, and is always numbered within some year.
///   The week number ranges from 1 to 52 or 53 depending on the year.
///
/// * The week 1 of given year is defined as the first week containing January 4 of that year,
///   or equivalently, the first week containing four or more days in that year.
///
/// * The year number in the week date may *not* correspond to the actual Gregorian year.
///   For example, January 3, 2016 (Sunday) was on the last (53rd) week of 2015.
///
/// Chrono's date types default to the ISO 8601 [calendar date](#calendar-date),
/// but [`Datelike::iso_week`](../trait.Datelike.html#tymethod.iso_week) and
/// [`Datelike::weekday`](../trait.Datelike.html#tymethod.weekday) methods
/// can be used to get the corresponding week date.
///
/// # Ordinal Date
///
/// The ISO 8601 **ordinal date** is a pair of year number and day of the year ("ordinal").
/// The ordinal number ranges from 1 to 365 or 366 depending on the year.
/// The year number is the same as that of the [calendar date](#calendar-date).
///
/// This is currently the internal format of Chrono's date types.
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, Deserialize, Serialize))]
pub struct NaiveDate {
    ymdf: DateImpl, // (year << 13) | of
}

/// The minimum possible `NaiveDate` (January 1, 262145 BCE).
#[deprecated(since = "0.4.20", note = "Use NaiveDate::MIN instead")]
pub const MIN_DATE: NaiveDate = NaiveDate::MIN;
/// The maximum possible `NaiveDate` (December 31, 262143 CE).
#[deprecated(since = "0.4.20", note = "Use NaiveDate::MAX instead")]
pub const MAX_DATE: NaiveDate = NaiveDate::MAX;

#[cfg(feature = "arbitrary")]
impl arbitrary::Arbitrary<'_> for NaiveDate {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<NaiveDate> {
        let year = u.int_in_range(MIN_YEAR..=MAX_YEAR)?;
        let max_days = YearFlags::from_year(year).ndays();
        let ord = u.int_in_range(1..=max_days)?;
        NaiveDate::from_yo_opt(year, ord).ok_or(arbitrary::Error::IncorrectFormat)
    }
}

// as it is hard to verify year flags in `NaiveDate::MIN` and `NaiveDate::MAX`,
// we use a separate run-time test.
#[test]
fn test_date_bounds() {
    let calculated_min = NaiveDate::from_ymd_opt(MIN_YEAR, 1, 1).unwrap();
    let calculated_max = NaiveDate::from_ymd_opt(MAX_YEAR, 12, 31).unwrap();
    assert!(
        NaiveDate::MIN == calculated_min,
        "`NaiveDate::MIN` should have a year flag {:?}",
        calculated_min.of().flags()
    );
    assert!(
        NaiveDate::MAX == calculated_max,
        "`NaiveDate::MAX` should have a year flag {:?}",
        calculated_max.of().flags()
    );

    // let's also check that the entire range do not exceed 2^44 seconds
    // (sometimes used for bounding `Duration` against overflow)
    let maxsecs = NaiveDate::MAX.signed_duration_since(NaiveDate::MIN).num_seconds();
    let maxsecs = maxsecs + 86401; // also take care of DateTime
    assert!(
        maxsecs < (1 << MAX_BITS),
        "The entire `NaiveDate` range somehow exceeds 2^{} seconds",
        MAX_BITS
    );
}

impl NaiveDate {
    /// Makes a new `NaiveDate` from year and packed ordinal-flags, with a verification.
    fn from_of(year: i32, of: Of) -> Option<NaiveDate> {
        if (MIN_YEAR..=MAX_YEAR).contains(&year) && of.valid() {
            let Of(of) = of;
            Some(NaiveDate { ymdf: (year << 13) | (of as DateImpl) })
        } else {
            None
        }
    }

    /// Makes a new `NaiveDate` from year and packed month-day-flags, with a verification.
    fn from_mdf(year: i32, mdf: Mdf) -> Option<NaiveDate> {
        NaiveDate::from_of(year, mdf.to_of())
    }

    /// Makes a new `NaiveDate` from the [calendar date](#calendar-date)
    /// (year, month and day).
    ///
    /// Panics on the out-of-range date, invalid month and/or day.
    #[deprecated(since = "0.4.23", note = "use `from_ymd_opt()` instead")]
    pub fn from_ymd(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("invalid or out-of-range date")
    }

    /// Makes a new `NaiveDate` from the [calendar date](#calendar-date)
    /// (year, month and day).
    ///
    /// Returns `None` on the out-of-range date, invalid month and/or day.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::NaiveDate;
    ///
    /// let from_ymd_opt = NaiveDate::from_ymd_opt;
    ///
    /// assert!(from_ymd_opt(2015, 3, 14).is_some());
    /// assert!(from_ymd_opt(2015, 0, 14).is_none());
    /// assert!(from_ymd_opt(2015, 2, 29).is_none());
    /// assert!(from_ymd_opt(-4, 2, 29).is_some()); // 5 BCE is a leap year
    /// assert!(from_ymd_opt(400000, 1, 1).is_none());
    /// assert!(from_ymd_opt(-400000, 1, 1).is_none());
    /// ```
    pub fn from_ymd_opt(year: i32, month: u32, day: u32) -> Option<NaiveDate> {
        let flags = YearFlags::from_year(year);
        NaiveDate::from_mdf(year, Mdf::new(month, day, flags)?)
    }

    /// Makes a new `NaiveDate` from the [ordinal date](#ordinal-date)
    /// (year and day of the year).
    ///
    /// Panics on the out-of-range date and/or invalid day of year.
    #[deprecated(since = "0.4.23", note = "use `from_yo_opt()` instead")]
    pub fn from_yo(year: i32, ordinal: u32) -> NaiveDate {
        NaiveDate::from_yo_opt(year, ordinal).expect("invalid or out-of-range date")
    }

    /// Makes a new `NaiveDate` from the [ordinal date](#ordinal-date)
    /// (year and day of the year).
    ///
    /// Returns `None` on the out-of-range date and/or invalid day of year.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::NaiveDate;
    ///
    /// let from_yo_opt = NaiveDate::from_yo_opt;
    ///
    /// assert!(from_yo_opt(2015, 100).is_some());
    /// assert!(from_yo_opt(2015, 0).is_none());
    /// assert!(from_yo_opt(2015, 365).is_some());
    /// assert!(from_yo_opt(2015, 366).is_none());
    /// assert!(from_yo_opt(-4, 366).is_some()); // 5 BCE is a leap year
    /// assert!(from_yo_opt(400000, 1).is_none());
    /// assert!(from_yo_opt(-400000, 1).is_none());
    /// ```
    pub fn from_yo_opt(year: i32, ordinal: u32) -> Option<NaiveDate> {
        let flags = YearFlags::from_year(year);
        NaiveDate::from_of(year, Of::new(ordinal, flags)?)
    }

    /// Makes a new `NaiveDate` from the [ISO week date](#week-date)
    /// (year, week number and day of the week).
    /// The resulting `NaiveDate` may have a different year from the input year.
    ///
    /// Panics on the out-of-range date and/or invalid week number.
    #[deprecated(since = "0.4.23", note = "use `from_isoywd_opt()` instead")]
    pub fn from_isoywd(year: i32, week: u32, weekday: Weekday) -> NaiveDate {
        NaiveDate::from_isoywd_opt(year, week, weekday).expect("invalid or out-of-range date")
    }

    /// Makes a new `NaiveDate` from the [ISO week date](#week-date)
    /// (year, week number and day of the week).
    /// The resulting `NaiveDate` may have a different year from the input year.
    ///
    /// Returns `None` on the out-of-range date and/or invalid week number.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{NaiveDate, Weekday};
    ///
    /// let from_ymd = NaiveDate::from_ymd;
    /// let from_isoywd_opt = NaiveDate::from_isoywd_opt;
    ///
    /// assert_eq!(from_isoywd_opt(2015, 0, Weekday::Sun), None);
    /// assert_eq!(from_isoywd_opt(2015, 10, Weekday::Sun), Some(from_ymd(2015, 3, 8)));
    /// assert_eq!(from_isoywd_opt(2015, 30, Weekday::Mon), Some(from_ymd(2015, 7, 20)));
    /// assert_eq!(from_isoywd_opt(2015, 60, Weekday::Mon), None);
    ///
    /// assert_eq!(from_isoywd_opt(400000, 10, Weekday::Fri), None);
    /// assert_eq!(from_isoywd_opt(-400000, 10, Weekday::Sat), None);
    /// ```
    ///
    /// The year number of ISO week date may differ from that of the calendar date.
    ///
    /// ```
    /// # use chrono::{NaiveDate, Weekday};
    /// # let from_ymd = NaiveDate::from_ymd;
    /// # let from_isoywd_opt = NaiveDate::from_isoywd_opt;
    /// //           Mo Tu We Th Fr Sa Su
    /// // 2014-W52  22 23 24 25 26 27 28    has 4+ days of new year,
    /// // 2015-W01  29 30 31  1  2  3  4 <- so this is the first week
    /// assert_eq!(from_isoywd_opt(2014, 52, Weekday::Sun), Some(from_ymd(2014, 12, 28)));
    /// assert_eq!(from_isoywd_opt(2014, 53, Weekday::Mon), None);
    /// assert_eq!(from_isoywd_opt(2015, 1, Weekday::Mon), Some(from_ymd(2014, 12, 29)));
    ///
    /// // 2015-W52  21 22 23 24 25 26 27    has 4+ days of old year,
    /// // 2015-W53  28 29 30 31  1  2  3 <- so this is the last week
    /// // 2016-W01   4  5  6  7  8  9 10
    /// assert_eq!(from_isoywd_opt(2015, 52, Weekday::Sun), Some(from_ymd(2015, 12, 27)));
    /// assert_eq!(from_isoywd_opt(2015, 53, Weekday::Sun), Some(from_ymd(2016, 1, 3)));
    /// assert_eq!(from_isoywd_opt(2015, 54, Weekday::Mon), None);
    /// assert_eq!(from_isoywd_opt(2016, 1, Weekday::Mon), Some(from_ymd(2016, 1, 4)));
    /// ```
    pub fn from_isoywd_opt(year: i32, week: u32, weekday: Weekday) -> Option<NaiveDate> {
        let flags = YearFlags::from_year(year);
        let nweeks = flags.nisoweeks();
        if 1 <= week && week <= nweeks {
            // ordinal = week ordinal - delta
            let weekord = week * 7 + weekday as u32;
            let delta = flags.isoweek_delta();
            if weekord <= delta {
                // ordinal < 1, previous year
                let prevflags = YearFlags::from_year(year - 1);
                NaiveDate::from_of(
                    year - 1,
                    Of::new(weekord + prevflags.ndays() - delta, prevflags)?,
                )
            } else {
                let ordinal = weekord - delta;
                let ndays = flags.ndays();
                if ordinal <= ndays {
                    // this year
                    NaiveDate::from_of(year, Of::new(ordinal, flags)?)
                } else {
                    // ordinal > ndays, next year
                    let nextflags = YearFlags::from_year(year + 1);
                    NaiveDate::from_of(year + 1, Of::new(ordinal - ndays, nextflags)?)
                }
            }
        } else {
            None
        }
    }

    /// Makes a new `NaiveDate` from a day's number in the proleptic Gregorian calendar, with
    /// January 1, 1 being day 1.
    ///
    /// Panics if the date is out of range.
    #[deprecated(since = "0.4.23", note = "use `from_num_days_from_ce_opt()` instead")]
    #[inline]
    pub fn from_num_days_from_ce(days: i32) -> NaiveDate {
        NaiveDate::from_num_days_from_ce_opt(days).expect("out-of-range date")
    }

    /// Makes a new `NaiveDate` from a day's number in the proleptic Gregorian calendar, with
    /// January 1, 1 being day 1.
    ///
    /// Returns `None` if the date is out of range.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::NaiveDate;
    ///
    /// let from_ndays_opt = NaiveDate::from_num_days_from_ce_opt;
    /// let from_ymd = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
    ///
    /// assert_eq!(from_ndays_opt(730_000),      Some(from_ymd(1999, 9, 3)));
    /// assert_eq!(from_ndays_opt(1),            Some(from_ymd(1, 1, 1)));
    /// assert_eq!(from_ndays_opt(0),            Some(from_ymd(0, 12, 31)));
    /// assert_eq!(from_ndays_opt(-1),           Some(from_ymd(0, 12, 30)));
    /// assert_eq!(from_ndays_opt(100_000_000),  None);
    /// assert_eq!(from_ndays_opt(-100_000_000), None);
    /// ```
    pub fn from_num_days_from_ce_opt(days: i32) -> Option<NaiveDate> {
        let days = days + 365; // make December 31, 1 BCE equal to day 0
        let (year_div_400, cycle) = div_mod_floor(days, 146_097);
        let (year_mod_400, ordinal) = internals::cycle_to_yo(cycle as u32);
        let flags = YearFlags::from_year_mod_400(year_mod_400 as i32);
        NaiveDate::from_of(year_div_400 * 400 + year_mod_400 as i32, Of::new(ordinal, flags)?)
    }

    /// Makes a new `NaiveDate` by counting the number of occurrences of a particular day-of-week
    /// since the beginning of the given month.  For instance, if you want the 2nd Friday of March
    /// 2017, you would use `NaiveDate::from_weekday_of_month(2017, 3, Weekday::Fri, 2)`.
    ///
    /// # Panics
    ///
    /// The resulting `NaiveDate` is guaranteed to be in `month`.  If `n` is larger than the number
    /// of `weekday` in `month` (eg. the 6th Friday of March 2017) then this function will panic.
    ///
    /// `n` is 1-indexed.  Passing `n=0` will cause a panic.
    #[deprecated(since = "0.4.23", note = "use `from_weekday_of_month_opt()` instead")]
    pub fn from_weekday_of_month(year: i32, month: u32, weekday: Weekday, n: u8) -> NaiveDate {
        NaiveDate::from_weekday_of_month_opt(year, month, weekday, n).expect("out-of-range date")
    }

    /// Makes a new `NaiveDate` by counting the number of occurrences of a particular day-of-week
    /// since the beginning of the given month.  For instance, if you want the 2nd Friday of March
    /// 2017, you would use `NaiveDate::from_weekday_of_month(2017, 3, Weekday::Fri, 2)`.  `n` is 1-indexed.
    ///
    /// ```
    /// use chrono::{NaiveDate, Weekday};
    /// assert_eq!(NaiveDate::from_weekday_of_month_opt(2017, 3, Weekday::Fri, 2),
    ///            NaiveDate::from_ymd_opt(2017, 3, 10))
    /// ```
    ///
    /// Returns `None` if `n` out-of-range; ie. if `n` is larger than the number of `weekday` in
    /// `month` (eg. the 6th Friday of March 2017), or if `n == 0`.
    pub fn from_weekday_of_month_opt(
        year: i32,
        month: u32,
        weekday: Weekday,
        n: u8,
    ) -> Option<NaiveDate> {
        if n == 0 {
            return None;
        }
        let first = NaiveDate::from_ymd_opt(year, month, 1)?.weekday();
        let first_to_dow = (7 + weekday.number_from_monday() - first.number_from_monday()) % 7;
        let day = (u32::from(n) - 1) * 7 + first_to_dow + 1;
        NaiveDate::from_ymd_opt(year, month, day)
    }

    /// Parses a string with the specified format string and returns a new `NaiveDate`.
    /// See the [`format::strftime` module](../format/strftime/index.html)
    /// on the supported escape sequences.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::NaiveDate;
    ///
    /// let parse_from_str = NaiveDate::parse_from_str;
    ///
    /// assert_eq!(parse_from_str("2015-09-05", "%Y-%m-%d"),
    ///            Ok(NaiveDate::from_ymd_opt(2015, 9, 5).unwrap()));
    /// assert_eq!(parse_from_str("5sep2015", "%d%b%Y"),
    ///            Ok(NaiveDate::from_ymd_opt(2015, 9, 5).unwrap()));
    /// ```
    ///
    /// Time and offset is ignored for the purpose of parsing.
    ///
    /// ```
    /// # use chrono::NaiveDate;
    /// # let parse_from_str = NaiveDate::parse_from_str;
    /// assert_eq!(parse_from_str("2014-5-17T12:34:56+09:30", "%Y-%m-%dT%H:%M:%S%z"),
    ///            Ok(NaiveDate::from_ymd_opt(2014, 5, 17).unwrap()));
    /// ```
    ///
    /// Out-of-bound dates or insufficient fields are errors.
    ///
    /// ```
    /// # use chrono::NaiveDate;
    /// # let parse_from_str = NaiveDate::parse_from_str;
    /// assert!(parse_from_str("2015/9", "%Y/%m").is_err());
    /// assert!(parse_from_str("2015/9/31", "%Y/%m/%d").is_err());
    /// ```
    ///
    /// All parsed fields should be consistent to each other, otherwise it's an error.
    ///
    /// ```
    /// # use chrono::NaiveDate;
    /// # let parse_from_str = NaiveDate::parse_from_str;
    /// assert!(parse_from_str("Sat, 09 Aug 2013", "%a, %d %b %Y").is_err());
    /// ```
    pub fn parse_from_str(s: &str, fmt: &str) -> ParseResult<NaiveDate> {
        let mut parsed = Parsed::new();
        parse(&mut parsed, s, StrftimeItems::new(fmt))?;
        parsed.to_naive_date()
    }

    /// Add a duration in [`Months`] to the date
    ///
    /// If the day would be out of range for the resulting month, use the last day for that month.
    ///
    /// Returns `None` if the resulting date would be out of range.
    ///
    /// ```
    /// # use chrono::{NaiveDate, Months};
    /// assert_eq!(
    ///     NaiveDate::from_ymd_opt(2022, 2, 20).unwrap().checked_add_months(Months::new(6)),
    ///     Some(NaiveDate::from_ymd_opt(2022, 8, 20).unwrap())
    /// );
    /// assert_eq!(
    ///     NaiveDate::from_ymd_opt(2022, 7, 31).unwrap().checked_add_months(Months::new(2)),
    ///     Some(NaiveDate::from_ymd_opt(2022, 9, 30).unwrap())
    /// );
    /// ```
    pub fn checked_add_months(self, months: Months) -> Option<Self> {
        if months.0 == 0 {
            return Some(self);
        }

        match months.0 <= core::i32::MAX as u32 {
            true => self.diff_months(months.0 as i32),
            false => None,
        }
    }

    /// Subtract a duration in [`Months`] from the date
    ///
    /// If the day would be out of range for the resulting month, use the last day for that month.
    ///
    /// Returns `None` if the resulting date would be out of range.
    ///
    /// ```
    /// # use chrono::{NaiveDate, Months};
    /// assert_eq!(
    ///     NaiveDate::from_ymd_opt(2022, 2, 20).unwrap().checked_sub_months(Months::new(6)),
    ///     Some(NaiveDate::from_ymd_opt(2021, 8, 20).unwrap())
    /// );
    ///
    /// assert_eq!(
    ///     NaiveDate::from_ymd_opt(2014, 1, 1).unwrap()
    ///         .checked_sub_months(Months::new(core::i32::MAX as u32 + 1)),
    ///     None
    /// );
    /// ```
    pub fn checked_sub_months(self, months: Months) -> Option<Self> {
        if months.0 == 0 {
            return Some(self);
        }

        // Copy `i32::MAX` here so we don't have to do a complicated cast
        match months.0 <= 2_147_483_647 {
            true => self.diff_months(-(months.0 as i32)),
            false => None,
        }
    }

    fn diff_months(self, months: i32) -> Option<Self> {
        let (years, left) = ((months / 12), (months % 12));

        // Determine new year (without taking months into account for now

        let year = if (years > 0 && years > (MAX_YEAR - self.year()))
            || (years < 0 && years < (MIN_YEAR - self.year()))
        {
            return None;
        } else {
            self.year() + years
        };

        // Determine new month

        let month = self.month() as i32 + left;
        let (year, month) = if month <= 0 {
            if year == MIN_YEAR {
                return None;
            }

            (year - 1, month + 12)
        } else if month > 12 {
            if year == MAX_YEAR {
                return None;
            }

            (year + 1, month - 12)
        } else {
            (year, month)
        };

        // Clamp original day in case new month is shorter

        let flags = YearFlags::from_year(year);
        let feb_days = if flags.ndays() == 366 { 29 } else { 28 };
        let days = [31, feb_days, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let day = Ord::min(self.day(), days[(month - 1) as usize]);

        NaiveDate::from_mdf(year, Mdf::new(month as u32, day, flags)?)
    }

    /// Add a duration in [`Days`] to the date
    ///
    /// Returns `None` if the resulting date would be out of range.
    ///
    /// ```
    /// # use chrono::{NaiveDate, Days};
    /// assert_eq!(
    ///     NaiveDate::from_ymd_opt(2022, 2, 20).unwrap().checked_add_days(Days::new(9)),
    ///     Some(NaiveDate::from_ymd_opt(2022, 3, 1).unwrap())
    /// );
    /// assert_eq!(
    ///     NaiveDate::from_ymd_opt(2022, 7, 31).unwrap().checked_add_days(Days::new(2)),
    ///     Some(NaiveDate::from_ymd_opt(2022, 8, 2).unwrap())
    /// );
    /// ```
    pub fn checked_add_days(self, days: Days) -> Option<Self> {
        if days.0 == 0 {
            return Some(self);
        }

        i64::try_from(days.0).ok().and_then(|d| self.diff_days(d))
    }

    /// Subtract a duration in [`Days`] from the date
    ///
    /// Returns `None` if the resulting date would be out of range.
    ///
    /// ```
    /// # use chrono::{NaiveDate, Days};
    /// assert_eq!(
    ///     NaiveDate::from_ymd_opt(2022, 2, 20).unwrap().checked_sub_days(Days::new(6)),
    ///     Some(NaiveDate::from_ymd_opt(2022, 2, 14).unwrap())
    /// );
    /// ```
    pub fn checked_sub_days(self, days: Days) -> Option<Self> {
        if days.0 == 0 {
            return Some(self);
        }

        i64::try_from(days.0).ok().and_then(|d| self.diff_days(-d))
    }

    fn diff_days(self, days: i64) -> Option<Self> {
        self.checked_add_signed(Duration::days(days))
    }

    /// Makes a new `NaiveDateTime` from the current date and given `NaiveTime`.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{NaiveDate, NaiveTime, NaiveDateTime};
    ///
    /// let d = NaiveDate::from_ymd_opt(2015, 6, 3).unwrap();
    /// let t = NaiveTime::from_hms_milli_opt(12, 34, 56, 789).unwrap();
    ///
    /// let dt: NaiveDateTime = d.and_time(t);
    /// assert_eq!(dt.date(), d);
    /// assert_eq!(dt.time(), t);
    /// ```
    #[inline]
    pub fn and_time(&self, time: NaiveTime) -> NaiveDateTime {
        NaiveDateTime::new(*self, time)
    }

    /// Makes a new `NaiveDateTime` from the current date, hour, minute and second.
    ///
    /// No [leap second](./struct.NaiveTime.html#leap-second-handling) is allowed here;
    /// use `NaiveDate::and_hms_*` methods with a subsecond parameter instead.
    ///
    /// Panics on invalid hour, minute and/or second.
    #[deprecated(since = "0.4.23", note = "use `and_hms_opt()` instead")]
    #[inline]
    pub fn and_hms(&self, hour: u32, min: u32, sec: u32) -> NaiveDateTime {
        self.and_hms_opt(hour, min, sec).expect("invalid time")
    }

    /// Makes a new `NaiveDateTime` from the current date, hour, minute and second.
    ///
    /// No [leap second](./struct.NaiveTime.html#leap-second-handling) is allowed here;
    /// use `NaiveDate::and_hms_*_opt` methods with a subsecond parameter instead.
    ///
    /// Returns `None` on invalid hour, minute and/or second.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::NaiveDate;
    ///
    /// let d = NaiveDate::from_ymd_opt(2015, 6, 3).unwrap();
    /// assert!(d.and_hms_opt(12, 34, 56).is_some());
    /// assert!(d.and_hms_opt(12, 34, 60).is_none()); // use `and_hms_milli_opt` instead
    /// assert!(d.and_hms_opt(12, 60, 56).is_none());
    /// assert!(d.and_hms_opt(24, 34, 56).is_none());
    /// ```
    #[inline]
    pub fn and_hms_opt(&self, hour: u32, min: u32, sec: u32) -> Option<NaiveDateTime> {
        NaiveTime::from_hms_opt(hour, min, sec).map(|time| self.and_time(time))
    }

    /// Makes a new `NaiveDateTime` from the current date, hour, minute, second and millisecond.
    ///
    /// The millisecond part can exceed 1,000
    /// in order to represent the [leap second](./struct.NaiveTime.html#leap-second-handling).
    ///
    /// Panics on invalid hour, minute, second and/or millisecond.
    #[deprecated(since = "0.4.23", note = "use `and_hms_milli_opt()` instead")]
    #[inline]
    pub fn and_hms_milli(&self, hour: u32, min: u32, sec: u32, milli: u32) -> NaiveDateTime {
        self.and_hms_milli_opt(hour, min, sec, milli).expect("invalid time")
    }

    /// Makes a new `NaiveDateTime` from the current date, hour, minute, second and millisecond.
    ///
    /// The millisecond part can exceed 1,000
    /// in order to represent the [leap second](./struct.NaiveTime.html#leap-second-handling).
    ///
    /// Returns `None` on invalid hour, minute, second and/or millisecond.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::NaiveDate;
    ///
    /// let d = NaiveDate::from_ymd_opt(2015, 6, 3).unwrap();
    /// assert!(d.and_hms_milli_opt(12, 34, 56,   789).is_some());
    /// assert!(d.and_hms_milli_opt(12, 34, 59, 1_789).is_some()); // leap second
    /// assert!(d.and_hms_milli_opt(12, 34, 59, 2_789).is_none());
    /// assert!(d.and_hms_milli_opt(12, 34, 60,   789).is_none());
    /// assert!(d.and_hms_milli_opt(12, 60, 56,   789).is_none());
    /// assert!(d.and_hms_milli_opt(24, 34, 56,   789).is_none());
    /// ```
    #[inline]
    pub fn and_hms_milli_opt(
        &self,
        hour: u32,
        min: u32,
        sec: u32,
        milli: u32,
    ) -> Option<NaiveDateTime> {
        NaiveTime::from_hms_milli_opt(hour, min, sec, milli).map(|time| self.and_time(time))
    }

    /// Makes a new `NaiveDateTime` from the current date, hour, minute, second and microsecond.
    ///
    /// The microsecond part can exceed 1,000,000
    /// in order to represent the [leap second](./struct.NaiveTime.html#leap-second-handling).
    ///
    /// Panics on invalid hour, minute, second and/or microsecond.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{NaiveDate, NaiveDateTime, Datelike, Timelike, Weekday};
    ///
    /// let d = NaiveDate::from_ymd_opt(2015, 6, 3).unwrap();
    ///
    /// let dt: NaiveDateTime = d.and_hms_micro(12, 34, 56, 789_012);
    /// assert_eq!(dt.year(), 2015);
    /// assert_eq!(dt.weekday(), Weekday::Wed);
    /// assert_eq!(dt.second(), 56);
    /// assert_eq!(dt.nanosecond(), 789_012_000);
    /// ```
    #[deprecated(since = "0.4.23", note = "use `and_hms_micro_opt()` instead")]
    #[inline]
    pub fn and_hms_micro(&self, hour: u32, min: u32, sec: u32, micro: u32) -> NaiveDateTime {
        self.and_hms_micro_opt(hour, min, sec, micro).expect("invalid time")
    }

    /// Makes a new `NaiveDateTime` from the current date, hour, minute, second and microsecond.
    ///
    /// The microsecond part can exceed 1,000,000
    /// in order to represent the [leap second](./struct.NaiveTime.html#leap-second-handling).
    ///
    /// Returns `None` on invalid hour, minute, second and/or microsecond.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::NaiveDate;
    ///
    /// let d = NaiveDate::from_ymd_opt(2015, 6, 3).unwrap();
    /// assert!(d.and_hms_micro_opt(12, 34, 56,   789_012).is_some());
    /// assert!(d.and_hms_micro_opt(12, 34, 59, 1_789_012).is_some()); // leap second
    /// assert!(d.and_hms_micro_opt(12, 34, 59, 2_789_012).is_none());
    /// assert!(d.and_hms_micro_opt(12, 34, 60,   789_012).is_none());
    /// assert!(d.and_hms_micro_opt(12, 60, 56,   789_012).is_none());
    /// assert!(d.and_hms_micro_opt(24, 34, 56,   789_012).is_none());
    /// ```
    #[inline]
    pub fn and_hms_micro_opt(
        &self,
        hour: u32,
        min: u32,
        sec: u32,
        micro: u32,
    ) -> Option<NaiveDateTime> {
        NaiveTime::from_hms_micro_opt(hour, min, sec, micro).map(|time| self.and_time(time))
    }

    /// Makes a new `NaiveDateTime` from the current date, hour, minute, second and nanosecond.
    ///
    /// The nanosecond part can exceed 1,000,000,000
    /// in order to represent the [leap second](./struct.NaiveTime.html#leap-second-handling).
    ///
    /// Panics on invalid hour, minute, second and/or nanosecond.
    #[deprecated(since = "0.4.23", note = "use `and_hms_nano_opt()` instead")]
    #[inline]
    pub fn and_hms_nano(&self, hour: u32, min: u32, sec: u32, nano: u32) -> NaiveDateTime {
        self.and_hms_nano_opt(hour, min, sec, nano).expect("invalid time")
    }

    /// Makes a new `NaiveDateTime` from the current date, hour, minute, second and nanosecond.
    ///
    /// The nanosecond part can exceed 1,000,000,000
    /// in order to represent the [leap second](./struct.NaiveTime.html#leap-second-handling).
    ///
    /// Returns `None` on invalid hour, minute, second and/or nanosecond.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::NaiveDate;
    ///
    /// let d = NaiveDate::from_ymd_opt(2015, 6, 3).unwrap();
    /// assert!(d.and_hms_nano_opt(12, 34, 56,   789_012_345).is_some());
    /// assert!(d.and_hms_nano_opt(12, 34, 59, 1_789_012_345).is_some()); // leap second
    /// assert!(d.and_hms_nano_opt(12, 34, 59, 2_789_012_345).is_none());
    /// assert!(d.and_hms_nano_opt(12, 34, 60,   789_012_345).is_none());
    /// assert!(d.and_hms_nano_opt(12, 60, 56,   789_012_345).is_none());
    /// assert!(d.and_hms_nano_opt(24, 34, 56,   789_012_345).is_none());
    /// ```
    #[inline]
    pub fn and_hms_nano_opt(
        &self,
        hour: u32,
        min: u32,
        sec: u32,
        nano: u32,
    ) -> Option<NaiveDateTime> {
        NaiveTime::from_hms_nano_opt(hour, min, sec, nano).map(|time| self.and_time(time))
    }

    /// Returns the packed month-day-flags.
    #[inline]
    fn mdf(&self) -> Mdf {
        self.of().to_mdf()
    }

    /// Returns the packed ordinal-flags.
    #[inline]
    fn of(&self) -> Of {
        Of((self.ymdf & 0b1_1111_1111_1111) as u32)
    }

    /// Makes a new `NaiveDate` with the packed month-day-flags changed.
    ///
    /// Returns `None` when the resulting `NaiveDate` would be invalid.
    #[inline]
    fn with_mdf(&self, mdf: Mdf) -> Option<NaiveDate> {
        self.with_of(mdf.to_of())
    }

    /// Makes a new `NaiveDate` with the packed ordinal-flags changed.
    ///
    /// Returns `None` when the resulting `NaiveDate` would be invalid.
    #[inline]
    fn with_of(&self, of: Of) -> Option<NaiveDate> {
        if of.valid() {
            let Of(of) = of;
            Some(NaiveDate { ymdf: (self.ymdf & !0b1_1111_1111_1111) | of as DateImpl })
        } else {
            None
        }
    }

    /// Makes a new `NaiveDate` for the next calendar date.
    ///
    /// Panics when `self` is the last representable date.
    #[deprecated(since = "0.4.23", note = "use `succ_opt()` instead")]
    #[inline]
    pub fn succ(&self) -> NaiveDate {
        self.succ_opt().expect("out of bound")
    }

    /// Makes a new `NaiveDate` for the next calendar date.
    ///
    /// Returns `None` when `self` is the last representable date.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::NaiveDate;
    ///
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 6, 3).unwrap().succ_opt(),
    ///            Some(NaiveDate::from_ymd_opt(2015, 6, 4).unwrap()));
    /// assert_eq!(NaiveDate::MAX.succ_opt(), None);
    /// ```
    #[inline]
    pub fn succ_opt(&self) -> Option<NaiveDate> {
        self.with_of(self.of().succ()).or_else(|| NaiveDate::from_ymd_opt(self.year() + 1, 1, 1))
    }

    /// Makes a new `NaiveDate` for the previous calendar date.
    ///
    /// Panics when `self` is the first representable date.
    #[deprecated(since = "0.4.23", note = "use `pred_opt()` instead")]
    #[inline]
    pub fn pred(&self) -> NaiveDate {
        self.pred_opt().expect("out of bound")
    }

    /// Makes a new `NaiveDate` for the previous calendar date.
    ///
    /// Returns `None` when `self` is the first representable date.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::NaiveDate;
    ///
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 6, 3).unwrap().pred_opt(),
    ///            Some(NaiveDate::from_ymd_opt(2015, 6, 2).unwrap()));
    /// assert_eq!(NaiveDate::MIN.pred_opt(), None);
    /// ```
    #[inline]
    pub fn pred_opt(&self) -> Option<NaiveDate> {
        self.with_of(self.of().pred()).or_else(|| NaiveDate::from_ymd_opt(self.year() - 1, 12, 31))
    }

    /// Adds the `days` part of given `Duration` to the current date.
    ///
    /// Returns `None` when it will result in overflow.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{Duration, NaiveDate};
    ///
    /// let d = NaiveDate::from_ymd_opt(2015, 9, 5).unwrap();
    /// assert_eq!(d.checked_add_signed(Duration::days(40)),
    ///            Some(NaiveDate::from_ymd_opt(2015, 10, 15).unwrap()));
    /// assert_eq!(d.checked_add_signed(Duration::days(-40)),
    ///            Some(NaiveDate::from_ymd_opt(2015, 7, 27).unwrap()));
    /// assert_eq!(d.checked_add_signed(Duration::days(1_000_000_000)), None);
    /// assert_eq!(d.checked_add_signed(Duration::days(-1_000_000_000)), None);
    /// assert_eq!(NaiveDate::MAX.checked_add_signed(Duration::days(1)), None);
    /// ```
    pub fn checked_add_signed(self, rhs: OldDuration) -> Option<NaiveDate> {
        let year = self.year();
        let (mut year_div_400, year_mod_400) = div_mod_floor(year, 400);
        let cycle = internals::yo_to_cycle(year_mod_400 as u32, self.of().ordinal());
        let cycle = (cycle as i32).checked_add(rhs.num_days().to_i32()?)?;
        let (cycle_div_400y, cycle) = div_mod_floor(cycle, 146_097);
        year_div_400 += cycle_div_400y;

        let (year_mod_400, ordinal) = internals::cycle_to_yo(cycle as u32);
        let flags = YearFlags::from_year_mod_400(year_mod_400 as i32);
        NaiveDate::from_of(year_div_400 * 400 + year_mod_400 as i32, Of::new(ordinal, flags)?)
    }

    /// Subtracts the `days` part of given `Duration` from the current date.
    ///
    /// Returns `None` when it will result in overflow.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{Duration, NaiveDate};
    ///
    /// let d = NaiveDate::from_ymd_opt(2015, 9, 5).unwrap();
    /// assert_eq!(d.checked_sub_signed(Duration::days(40)),
    ///            Some(NaiveDate::from_ymd_opt(2015, 7, 27).unwrap()));
    /// assert_eq!(d.checked_sub_signed(Duration::days(-40)),
    ///            Some(NaiveDate::from_ymd_opt(2015, 10, 15).unwrap()));
    /// assert_eq!(d.checked_sub_signed(Duration::days(1_000_000_000)), None);
    /// assert_eq!(d.checked_sub_signed(Duration::days(-1_000_000_000)), None);
    /// assert_eq!(NaiveDate::MIN.checked_sub_signed(Duration::days(1)), None);
    /// ```
    pub fn checked_sub_signed(self, rhs: OldDuration) -> Option<NaiveDate> {
        let year = self.year();
        let (mut year_div_400, year_mod_400) = div_mod_floor(year, 400);
        let cycle = internals::yo_to_cycle(year_mod_400 as u32, self.of().ordinal());
        let cycle = (cycle as i32).checked_sub(rhs.num_days().to_i32()?)?;
        let (cycle_div_400y, cycle) = div_mod_floor(cycle, 146_097);
        year_div_400 += cycle_div_400y;

        let (year_mod_400, ordinal) = internals::cycle_to_yo(cycle as u32);
        let flags = YearFlags::from_year_mod_400(year_mod_400 as i32);
        NaiveDate::from_of(year_div_400 * 400 + year_mod_400 as i32, Of::new(ordinal, flags)?)
    }

    /// Subtracts another `NaiveDate` from the current date.
    /// Returns a `Duration` of integral numbers.
    ///
    /// This does not overflow or underflow at all,
    /// as all possible output fits in the range of `Duration`.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{Duration, NaiveDate};
    ///
    /// let from_ymd = NaiveDate::from_ymd;
    /// let since = NaiveDate::signed_duration_since;
    ///
    /// assert_eq!(since(from_ymd(2014, 1, 1), from_ymd(2014, 1, 1)), Duration::zero());
    /// assert_eq!(since(from_ymd(2014, 1, 1), from_ymd(2013, 12, 31)), Duration::days(1));
    /// assert_eq!(since(from_ymd(2014, 1, 1), from_ymd(2014, 1, 2)), Duration::days(-1));
    /// assert_eq!(since(from_ymd(2014, 1, 1), from_ymd(2013, 9, 23)), Duration::days(100));
    /// assert_eq!(since(from_ymd(2014, 1, 1), from_ymd(2013, 1, 1)), Duration::days(365));
    /// assert_eq!(since(from_ymd(2014, 1, 1), from_ymd(2010, 1, 1)), Duration::days(365*4 + 1));
    /// assert_eq!(since(from_ymd(2014, 1, 1), from_ymd(1614, 1, 1)), Duration::days(365*400 + 97));
    /// ```
    pub fn signed_duration_since(self, rhs: NaiveDate) -> OldDuration {
        let year1 = self.year();
        let year2 = rhs.year();
        let (year1_div_400, year1_mod_400) = div_mod_floor(year1, 400);
        let (year2_div_400, year2_mod_400) = div_mod_floor(year2, 400);
        let cycle1 = i64::from(internals::yo_to_cycle(year1_mod_400 as u32, self.of().ordinal()));
        let cycle2 = i64::from(internals::yo_to_cycle(year2_mod_400 as u32, rhs.of().ordinal()));
        OldDuration::days(
            (i64::from(year1_div_400) - i64::from(year2_div_400)) * 146_097 + (cycle1 - cycle2),
        )
    }

    /// Returns the number of whole years from the given `base` until `self`.
    pub fn years_since(&self, base: Self) -> Option<u32> {
        let mut years = self.year() - base.year();
        if (self.month(), self.day()) < (base.month(), base.day()) {
            years -= 1;
        }

        match years >= 0 {
            true => Some(years as u32),
            false => None,
        }
    }

    /// Formats the date with the specified formatting items.
    /// Otherwise it is the same as the ordinary `format` method.
    ///
    /// The `Iterator` of items should be `Clone`able,
    /// since the resulting `DelayedFormat` value may be formatted multiple times.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::NaiveDate;
    /// use chrono::format::strftime::StrftimeItems;
    ///
    /// let fmt = StrftimeItems::new("%Y-%m-%d");
    /// let d = NaiveDate::from_ymd_opt(2015, 9, 5).unwrap();
    /// assert_eq!(d.format_with_items(fmt.clone()).to_string(), "2015-09-05");
    /// assert_eq!(d.format("%Y-%m-%d").to_string(),             "2015-09-05");
    /// ```
    ///
    /// The resulting `DelayedFormat` can be formatted directly via the `Display` trait.
    ///
    /// ```
    /// # use chrono::NaiveDate;
    /// # use chrono::format::strftime::StrftimeItems;
    /// # let fmt = StrftimeItems::new("%Y-%m-%d").clone();
    /// # let d = NaiveDate::from_ymd_opt(2015, 9, 5).unwrap();
    /// assert_eq!(format!("{}", d.format_with_items(fmt)), "2015-09-05");
    /// ```
    #[cfg(any(feature = "alloc", feature = "std", test))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
    #[inline]
    pub fn format_with_items<'a, I, B>(&self, items: I) -> DelayedFormat<I>
    where
        I: Iterator<Item = B> + Clone,
        B: Borrow<Item<'a>>,
    {
        DelayedFormat::new(Some(*self), None, items)
    }

    /// Formats the date with the specified format string.
    /// See the [`format::strftime` module](../format/strftime/index.html)
    /// on the supported escape sequences.
    ///
    /// This returns a `DelayedFormat`,
    /// which gets converted to a string only when actual formatting happens.
    /// You may use the `to_string` method to get a `String`,
    /// or just feed it into `print!` and other formatting macros.
    /// (In this way it avoids the redundant memory allocation.)
    ///
    /// A wrong format string does *not* issue an error immediately.
    /// Rather, converting or formatting the `DelayedFormat` fails.
    /// You are recommended to immediately use `DelayedFormat` for this reason.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::NaiveDate;
    ///
    /// let d = NaiveDate::from_ymd_opt(2015, 9, 5).unwrap();
    /// assert_eq!(d.format("%Y-%m-%d").to_string(), "2015-09-05");
    /// assert_eq!(d.format("%A, %-d %B, %C%y").to_string(), "Saturday, 5 September, 2015");
    /// ```
    ///
    /// The resulting `DelayedFormat` can be formatted directly via the `Display` trait.
    ///
    /// ```
    /// # use chrono::NaiveDate;
    /// # let d = NaiveDate::from_ymd_opt(2015, 9, 5).unwrap();
    /// assert_eq!(format!("{}", d.format("%Y-%m-%d")), "2015-09-05");
    /// assert_eq!(format!("{}", d.format("%A, %-d %B, %C%y")), "Saturday, 5 September, 2015");
    /// ```
    #[cfg(any(feature = "alloc", feature = "std", test))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
    #[inline]
    pub fn format<'a>(&self, fmt: &'a str) -> DelayedFormat<StrftimeItems<'a>> {
        self.format_with_items(StrftimeItems::new(fmt))
    }

    /// Returns an iterator that steps by days across all representable dates.
    ///
    /// # Example
    ///
    /// ```
    /// # use chrono::NaiveDate;
    ///
    /// let expected = [
    ///     NaiveDate::from_ymd_opt(2016, 2, 27).unwrap(),
    ///     NaiveDate::from_ymd_opt(2016, 2, 28).unwrap(),
    ///     NaiveDate::from_ymd_opt(2016, 2, 29).unwrap(),
    ///     NaiveDate::from_ymd_opt(2016, 3, 1).unwrap(),
    /// ];
    ///
    /// let mut count = 0;
    /// for (idx, d) in NaiveDate::from_ymd_opt(2016, 2, 27).unwrap().iter_days().take(4).enumerate() {
    ///    assert_eq!(d, expected[idx]);
    ///    count += 1;
    /// }
    /// assert_eq!(count, 4);
    ///
    /// for d in NaiveDate::from_ymd_opt(2016, 3, 1).unwrap().iter_days().rev().take(4) {
    ///     count -= 1;
    ///     assert_eq!(d, expected[count]);
    /// }
    /// ```
    #[inline]
    pub fn iter_days(&self) -> NaiveDateDaysIterator {
        NaiveDateDaysIterator { value: *self }
    }

    /// Returns an iterator that steps by weeks across all representable dates.
    ///
    /// # Example
    ///
    /// ```
    /// # use chrono::NaiveDate;
    ///
    /// let expected = [
    ///     NaiveDate::from_ymd_opt(2016, 2, 27).unwrap(),
    ///     NaiveDate::from_ymd_opt(2016, 3, 5).unwrap(),
    ///     NaiveDate::from_ymd_opt(2016, 3, 12).unwrap(),
    ///     NaiveDate::from_ymd_opt(2016, 3, 19).unwrap(),
    /// ];
    ///
    /// let mut count = 0;
    /// for (idx, d) in NaiveDate::from_ymd_opt(2016, 2, 27).unwrap().iter_weeks().take(4).enumerate() {
    ///    assert_eq!(d, expected[idx]);
    ///    count += 1;
    /// }
    /// assert_eq!(count, 4);
    ///
    /// for d in NaiveDate::from_ymd_opt(2016, 3, 19).unwrap().iter_weeks().rev().take(4) {
    ///     count -= 1;
    ///     assert_eq!(d, expected[count]);
    /// }
    /// ```
    #[inline]
    pub fn iter_weeks(&self) -> NaiveDateWeeksIterator {
        NaiveDateWeeksIterator { value: *self }
    }

    /// Returns the [`NaiveWeek`] that the date belongs to, starting with the [`Weekday`]
    /// specified.
    #[inline]
    pub fn week(&self, start: Weekday) -> NaiveWeek {
        NaiveWeek { date: *self, start }
    }

    /// The minimum possible `NaiveDate` (January 1, 262145 BCE).
    pub const MIN: NaiveDate = NaiveDate { ymdf: (MIN_YEAR << 13) | (1 << 4) | 0o07 /*FE*/ };
    /// The maximum possible `NaiveDate` (December 31, 262143 CE).
    pub const MAX: NaiveDate = NaiveDate { ymdf: (MAX_YEAR << 13) | (365 << 4) | 0o17 /*F*/ };
}

impl Datelike for NaiveDate {
    /// Returns the year number in the [calendar date](#calendar-date).
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{NaiveDate, Datelike};
    ///
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 9, 8).unwrap().year(), 2015);
    /// assert_eq!(NaiveDate::from_ymd_opt(-308, 3, 14).unwrap().year(), -308); // 309 BCE
    /// ```
    #[inline]
    fn year(&self) -> i32 {
        self.ymdf >> 13
    }

    /// Returns the month number starting from 1.
    ///
    /// The return value ranges from 1 to 12.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{NaiveDate, Datelike};
    ///
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 9, 8).unwrap().month(), 9);
    /// assert_eq!(NaiveDate::from_ymd_opt(-308, 3, 14).unwrap().month(), 3);
    /// ```
    #[inline]
    fn month(&self) -> u32 {
        self.mdf().month()
    }

    /// Returns the month number starting from 0.
    ///
    /// The return value ranges from 0 to 11.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{NaiveDate, Datelike};
    ///
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 9, 8).unwrap().month0(), 8);
    /// assert_eq!(NaiveDate::from_ymd_opt(-308, 3, 14).unwrap().month0(), 2);
    /// ```
    #[inline]
    fn month0(&self) -> u32 {
        self.mdf().month() - 1
    }

    /// Returns the day of month starting from 1.
    ///
    /// The return value ranges from 1 to 31. (The last day of month differs by months.)
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{NaiveDate, Datelike};
    ///
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 9, 8).unwrap().day(), 8);
    /// assert_eq!(NaiveDate::from_ymd_opt(-308, 3, 14).unwrap().day(), 14);
    /// ```
    ///
    /// Combined with [`NaiveDate::pred`](#method.pred),
    /// one can determine the number of days in a particular month.
    /// (Note that this panics when `year` is out of range.)
    ///
    /// ```
    /// use chrono::{NaiveDate, Datelike};
    ///
    /// fn ndays_in_month(year: i32, month: u32) -> u32 {
    ///     // the first day of the next month...
    ///     let (y, m) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    ///     let d = NaiveDate::from_ymd_opt(y, m, 1).unwrap();
    ///
    ///     // ...is preceded by the last day of the original month
    ///     d.pred().day()
    /// }
    ///
    /// assert_eq!(ndays_in_month(2015, 8), 31);
    /// assert_eq!(ndays_in_month(2015, 9), 30);
    /// assert_eq!(ndays_in_month(2015, 12), 31);
    /// assert_eq!(ndays_in_month(2016, 2), 29);
    /// assert_eq!(ndays_in_month(2017, 2), 28);
    /// ```
    #[inline]
    fn day(&self) -> u32 {
        self.mdf().day()
    }

    /// Returns the day of month starting from 0.
    ///
    /// The return value ranges from 0 to 30. (The last day of month differs by months.)
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{NaiveDate, Datelike};
    ///
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 9, 8).unwrap().day0(), 7);
    /// assert_eq!(NaiveDate::from_ymd_opt(-308, 3, 14).unwrap().day0(), 13);
    /// ```
    #[inline]
    fn day0(&self) -> u32 {
        self.mdf().day() - 1
    }

    /// Returns the day of year starting from 1.
    ///
    /// The return value ranges from 1 to 366. (The last day of year differs by years.)
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{NaiveDate, Datelike};
    ///
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 9, 8).unwrap().ordinal(), 251);
    /// assert_eq!(NaiveDate::from_ymd_opt(-308, 3, 14).unwrap().ordinal(), 74);
    /// ```
    ///
    /// Combined with [`NaiveDate::pred`](#method.pred),
    /// one can determine the number of days in a particular year.
    /// (Note that this panics when `year` is out of range.)
    ///
    /// ```
    /// use chrono::{NaiveDate, Datelike};
    ///
    /// fn ndays_in_year(year: i32) -> u32 {
    ///     // the first day of the next year...
    ///     let d = NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap();
    ///
    ///     // ...is preceded by the last day of the original year
    ///     d.pred().ordinal()
    /// }
    ///
    /// assert_eq!(ndays_in_year(2015), 365);
    /// assert_eq!(ndays_in_year(2016), 366);
    /// assert_eq!(ndays_in_year(2017), 365);
    /// assert_eq!(ndays_in_year(2000), 366);
    /// assert_eq!(ndays_in_year(2100), 365);
    /// ```
    #[inline]
    fn ordinal(&self) -> u32 {
        self.of().ordinal()
    }

    /// Returns the day of year starting from 0.
    ///
    /// The return value ranges from 0 to 365. (The last day of year differs by years.)
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{NaiveDate, Datelike};
    ///
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 9, 8).unwrap().ordinal0(), 250);
    /// assert_eq!(NaiveDate::from_ymd_opt(-308, 3, 14).unwrap().ordinal0(), 73);
    /// ```
    #[inline]
    fn ordinal0(&self) -> u32 {
        self.of().ordinal() - 1
    }

    /// Returns the day of week.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{NaiveDate, Datelike, Weekday};
    ///
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 9, 8).unwrap().weekday(), Weekday::Tue);
    /// assert_eq!(NaiveDate::from_ymd_opt(-308, 3, 14).unwrap().weekday(), Weekday::Fri);
    /// ```
    #[inline]
    fn weekday(&self) -> Weekday {
        self.of().weekday()
    }

    #[inline]
    fn iso_week(&self) -> IsoWeek {
        isoweek::iso_week_from_yof(self.year(), self.of())
    }

    /// Makes a new `NaiveDate` with the year number changed.
    ///
    /// Returns `None` when the resulting `NaiveDate` would be invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{NaiveDate, Datelike};
    ///
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 9, 8).unwrap().with_year(2016),
    ///            Some(NaiveDate::from_ymd_opt(2016, 9, 8).unwrap()));
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 9, 8).unwrap().with_year(-308),
    ///            Some(NaiveDate::from_ymd_opt(-308, 9, 8).unwrap()));
    /// ```
    ///
    /// A leap day (February 29) is a good example that this method can return `None`.
    ///
    /// ```
    /// # use chrono::{NaiveDate, Datelike};
    /// assert!(NaiveDate::from_ymd_opt(2016, 2, 29).unwrap().with_year(2015).is_none());
    /// assert!(NaiveDate::from_ymd_opt(2016, 2, 29).unwrap().with_year(2020).is_some());
    /// ```
    #[inline]
    fn with_year(&self, year: i32) -> Option<NaiveDate> {
        // we need to operate with `mdf` since we should keep the month and day number as is
        let mdf = self.mdf();

        // adjust the flags as needed
        let flags = YearFlags::from_year(year);
        let mdf = mdf.with_flags(flags);

        NaiveDate::from_mdf(year, mdf)
    }

    /// Makes a new `NaiveDate` with the month number (starting from 1) changed.
    ///
    /// Returns `None` when the resulting `NaiveDate` would be invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{NaiveDate, Datelike};
    ///
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 9, 8).unwrap().with_month(10),
    ///            Some(NaiveDate::from_ymd_opt(2015, 10, 8).unwrap()));
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 9, 8).unwrap().with_month(13), None); // no month 13
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 9, 30).unwrap().with_month(2), None); // no February 30
    /// ```
    #[inline]
    fn with_month(&self, month: u32) -> Option<NaiveDate> {
        self.with_mdf(self.mdf().with_month(month)?)
    }

    /// Makes a new `NaiveDate` with the month number (starting from 0) changed.
    ///
    /// Returns `None` when the resulting `NaiveDate` would be invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{NaiveDate, Datelike};
    ///
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 9, 8).unwrap().with_month0(9),
    ///            Some(NaiveDate::from_ymd_opt(2015, 10, 8).unwrap()));
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 9, 8).unwrap().with_month0(12), None); // no month 13
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 9, 30).unwrap().with_month0(1), None); // no February 30
    /// ```
    #[inline]
    fn with_month0(&self, month0: u32) -> Option<NaiveDate> {
        self.with_mdf(self.mdf().with_month(month0 + 1)?)
    }

    /// Makes a new `NaiveDate` with the day of month (starting from 1) changed.
    ///
    /// Returns `None` when the resulting `NaiveDate` would be invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{NaiveDate, Datelike};
    ///
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 9, 8).unwrap().with_day(30),
    ///            Some(NaiveDate::from_ymd_opt(2015, 9, 30).unwrap()));
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 9, 8).unwrap().with_day(31),
    ///            None); // no September 31
    /// ```
    #[inline]
    fn with_day(&self, day: u32) -> Option<NaiveDate> {
        self.with_mdf(self.mdf().with_day(day)?)
    }

    /// Makes a new `NaiveDate` with the day of month (starting from 0) changed.
    ///
    /// Returns `None` when the resulting `NaiveDate` would be invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{NaiveDate, Datelike};
    ///
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 9, 8).unwrap().with_day0(29),
    ///            Some(NaiveDate::from_ymd_opt(2015, 9, 30).unwrap()));
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 9, 8).unwrap().with_day0(30),
    ///            None); // no September 31
    /// ```
    #[inline]
    fn with_day0(&self, day0: u32) -> Option<NaiveDate> {
        self.with_mdf(self.mdf().with_day(day0 + 1)?)
    }

    /// Makes a new `NaiveDate` with the day of year (starting from 1) changed.
    ///
    /// Returns `None` when the resulting `NaiveDate` would be invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{NaiveDate, Datelike};
    ///
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 1, 1).unwrap().with_ordinal(60),
    ///            Some(NaiveDate::from_ymd_opt(2015, 3, 1).unwrap()));
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 1, 1).unwrap().with_ordinal(366),
    ///            None); // 2015 had only 365 days
    ///
    /// assert_eq!(NaiveDate::from_ymd_opt(2016, 1, 1).unwrap().with_ordinal(60),
    ///            Some(NaiveDate::from_ymd_opt(2016, 2, 29).unwrap()));
    /// assert_eq!(NaiveDate::from_ymd_opt(2016, 1, 1).unwrap().with_ordinal(366),
    ///            Some(NaiveDate::from_ymd_opt(2016, 12, 31).unwrap()));
    /// ```
    #[inline]
    fn with_ordinal(&self, ordinal: u32) -> Option<NaiveDate> {
        self.with_of(self.of().with_ordinal(ordinal)?)
    }

    /// Makes a new `NaiveDate` with the day of year (starting from 0) changed.
    ///
    /// Returns `None` when the resulting `NaiveDate` would be invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{NaiveDate, Datelike};
    ///
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 1, 1).unwrap().with_ordinal0(59),
    ///            Some(NaiveDate::from_ymd_opt(2015, 3, 1).unwrap()));
    /// assert_eq!(NaiveDate::from_ymd_opt(2015, 1, 1).unwrap().with_ordinal0(365),
    ///            None); // 2015 had only 365 days
    ///
    /// assert_eq!(NaiveDate::from_ymd_opt(2016, 1, 1).unwrap().with_ordinal0(59),
    ///            Some(NaiveDate::from_ymd_opt(2016, 2, 29).unwrap()));
    /// assert_eq!(NaiveDate::from_ymd_opt(2016, 1, 1).unwrap().with_ordinal0(365),
    ///            Some(NaiveDate::from_ymd_opt(2016, 12, 31).unwrap()));
    /// ```
    #[inline]
    fn with_ordinal0(&self, ordinal0: u32) -> Option<NaiveDate> {
        self.with_of(self.of().with_ordinal(ordinal0 + 1)?)
    }
}

/// An addition of `Duration` to `NaiveDate` discards the fractional days,
/// rounding to the closest integral number of days towards `Duration::zero()`.
///
/// Panics on underflow or overflow.
/// Use [`NaiveDate::checked_add_signed`](#method.checked_add_signed) to detect that.
///
/// # Example
///
/// ```
/// use chrono::{Duration, NaiveDate};
///
/// let from_ymd = NaiveDate::from_ymd;
///
/// assert_eq!(from_ymd(2014, 1, 1) + Duration::zero(),             from_ymd(2014, 1, 1));
/// assert_eq!(from_ymd(2014, 1, 1) + Duration::seconds(86399),     from_ymd(2014, 1, 1));
/// assert_eq!(from_ymd(2014, 1, 1) + Duration::seconds(-86399),    from_ymd(2014, 1, 1));
/// assert_eq!(from_ymd(2014, 1, 1) + Duration::days(1),            from_ymd(2014, 1, 2));
/// assert_eq!(from_ymd(2014, 1, 1) + Duration::days(-1),           from_ymd(2013, 12, 31));
/// assert_eq!(from_ymd(2014, 1, 1) + Duration::days(364),          from_ymd(2014, 12, 31));
/// assert_eq!(from_ymd(2014, 1, 1) + Duration::days(365*4 + 1),    from_ymd(2018, 1, 1));
/// assert_eq!(from_ymd(2014, 1, 1) + Duration::days(365*400 + 97), from_ymd(2414, 1, 1));
/// ```
impl Add<OldDuration> for NaiveDate {
    type Output = NaiveDate;

    #[inline]
    fn add(self, rhs: OldDuration) -> NaiveDate {
        self.checked_add_signed(rhs).expect("`NaiveDate + Duration` overflowed")
    }
}

impl AddAssign<OldDuration> for NaiveDate {
    #[inline]
    fn add_assign(&mut self, rhs: OldDuration) {
        *self = self.add(rhs);
    }
}

impl Add<Months> for NaiveDate {
    type Output = NaiveDate;

    /// An addition of months to `NaiveDate` clamped to valid days in resulting month.
    ///
    /// # Panics
    ///
    /// Panics if the resulting date would be out of range.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{Duration, NaiveDate, Months};
    ///
    /// let from_ymd = NaiveDate::from_ymd;
    ///
    /// assert_eq!(from_ymd(2014, 1, 1) + Months::new(1), from_ymd(2014, 2, 1));
    /// assert_eq!(from_ymd(2014, 1, 1) + Months::new(11), from_ymd(2014, 12, 1));
    /// assert_eq!(from_ymd(2014, 1, 1) + Months::new(12), from_ymd(2015, 1, 1));
    /// assert_eq!(from_ymd(2014, 1, 1) + Months::new(13), from_ymd(2015, 2, 1));
    /// assert_eq!(from_ymd(2014, 1, 31) + Months::new(1), from_ymd(2014, 2, 28));
    /// assert_eq!(from_ymd(2020, 1, 31) + Months::new(1), from_ymd(2020, 2, 29));
    /// ```
    fn add(self, months: Months) -> Self::Output {
        self.checked_add_months(months).unwrap()
    }
}

impl Sub<Months> for NaiveDate {
    type Output = NaiveDate;

    /// A subtraction of Months from `NaiveDate` clamped to valid days in resulting month.
    ///
    /// # Panics
    ///
    /// Panics if the resulting date would be out of range.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{Duration, NaiveDate, Months};
    ///
    /// let from_ymd = NaiveDate::from_ymd;
    ///
    /// assert_eq!(from_ymd(2014, 1, 1) - Months::new(11), from_ymd(2013, 2, 1));
    /// assert_eq!(from_ymd(2014, 1, 1) - Months::new(12), from_ymd(2013, 1, 1));
    /// assert_eq!(from_ymd(2014, 1, 1) - Months::new(13), from_ymd(2012, 12, 1));
    /// ```
    fn sub(self, months: Months) -> Self::Output {
        self.checked_sub_months(months).unwrap()
    }
}

impl Add<Days> for NaiveDate {
    type Output = NaiveDate;

    fn add(self, days: Days) -> Self::Output {
        self.checked_add_days(days).unwrap()
    }
}

impl Sub<Days> for NaiveDate {
    type Output = NaiveDate;

    fn sub(self, days: Days) -> Self::Output {
        self.checked_sub_days(days).unwrap()
    }
}

/// A subtraction of `Duration` from `NaiveDate` discards the fractional days,
/// rounding to the closest integral number of days towards `Duration::zero()`.
/// It is the same as the addition with a negated `Duration`.
///
/// Panics on underflow or overflow.
/// Use [`NaiveDate::checked_sub_signed`](#method.checked_sub_signed) to detect that.
///
/// # Example
///
/// ```
/// use chrono::{Duration, NaiveDate};
///
/// let from_ymd = NaiveDate::from_ymd;
///
/// assert_eq!(from_ymd(2014, 1, 1) - Duration::zero(),             from_ymd(2014, 1, 1));
/// assert_eq!(from_ymd(2014, 1, 1) - Duration::seconds(86399),     from_ymd(2014, 1, 1));
/// assert_eq!(from_ymd(2014, 1, 1) - Duration::seconds(-86399),    from_ymd(2014, 1, 1));
/// assert_eq!(from_ymd(2014, 1, 1) - Duration::days(1),            from_ymd(2013, 12, 31));
/// assert_eq!(from_ymd(2014, 1, 1) - Duration::days(-1),           from_ymd(2014, 1, 2));
/// assert_eq!(from_ymd(2014, 1, 1) - Duration::days(364),          from_ymd(2013, 1, 2));
/// assert_eq!(from_ymd(2014, 1, 1) - Duration::days(365*4 + 1),    from_ymd(2010, 1, 1));
/// assert_eq!(from_ymd(2014, 1, 1) - Duration::days(365*400 + 97), from_ymd(1614, 1, 1));
/// ```
impl Sub<OldDuration> for NaiveDate {
    type Output = NaiveDate;

    #[inline]
    fn sub(self, rhs: OldDuration) -> NaiveDate {
        self.checked_sub_signed(rhs).expect("`NaiveDate - Duration` overflowed")
    }
}

impl SubAssign<OldDuration> for NaiveDate {
    #[inline]
    fn sub_assign(&mut self, rhs: OldDuration) {
        *self = self.sub(rhs);
    }
}

/// Subtracts another `NaiveDate` from the current date.
/// Returns a `Duration` of integral numbers.
///
/// This does not overflow or underflow at all,
/// as all possible output fits in the range of `Duration`.
///
/// The implementation is a wrapper around
/// [`NaiveDate::signed_duration_since`](#method.signed_duration_since).
///
/// # Example
///
/// ```
/// use chrono::{Duration, NaiveDate};
///
/// let from_ymd = NaiveDate::from_ymd;
///
/// assert_eq!(from_ymd(2014, 1, 1) - from_ymd(2014, 1, 1), Duration::zero());
/// assert_eq!(from_ymd(2014, 1, 1) - from_ymd(2013, 12, 31), Duration::days(1));
/// assert_eq!(from_ymd(2014, 1, 1) - from_ymd(2014, 1, 2), Duration::days(-1));
/// assert_eq!(from_ymd(2014, 1, 1) - from_ymd(2013, 9, 23), Duration::days(100));
/// assert_eq!(from_ymd(2014, 1, 1) - from_ymd(2013, 1, 1), Duration::days(365));
/// assert_eq!(from_ymd(2014, 1, 1) - from_ymd(2010, 1, 1), Duration::days(365*4 + 1));
/// assert_eq!(from_ymd(2014, 1, 1) - from_ymd(1614, 1, 1), Duration::days(365*400 + 97));
/// ```
impl Sub<NaiveDate> for NaiveDate {
    type Output = OldDuration;

    #[inline]
    fn sub(self, rhs: NaiveDate) -> OldDuration {
        self.signed_duration_since(rhs)
    }
}

/// Iterator over `NaiveDate` with a step size of one day.
#[derive(Debug, Copy, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct NaiveDateDaysIterator {
    value: NaiveDate,
}

impl Iterator for NaiveDateDaysIterator {
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        if self.value == NaiveDate::MAX {
            return None;
        }
        // current < NaiveDate::MAX from here on:
        let current = self.value;
        // This can't panic because current is < NaiveDate::MAX:
        self.value = current.succ_opt().unwrap();
        Some(current)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact_size = NaiveDate::MAX.signed_duration_since(self.value).num_days();
        (exact_size as usize, Some(exact_size as usize))
    }
}

impl ExactSizeIterator for NaiveDateDaysIterator {}

impl DoubleEndedIterator for NaiveDateDaysIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.value == NaiveDate::MIN {
            return None;
        }
        let current = self.value;
        self.value = current.pred_opt().unwrap();
        Some(current)
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct NaiveDateWeeksIterator {
    value: NaiveDate,
}

impl Iterator for NaiveDateWeeksIterator {
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        if NaiveDate::MAX - self.value < OldDuration::weeks(1) {
            return None;
        }
        let current = self.value;
        self.value = current + OldDuration::weeks(1);
        Some(current)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact_size = NaiveDate::MAX.signed_duration_since(self.value).num_weeks();
        (exact_size as usize, Some(exact_size as usize))
    }
}

impl ExactSizeIterator for NaiveDateWeeksIterator {}

impl DoubleEndedIterator for NaiveDateWeeksIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.value - NaiveDate::MIN < OldDuration::weeks(1) {
            return None;
        }
        let current = self.value;
        self.value = current - OldDuration::weeks(1);
        Some(current)
    }
}

// TODO: NaiveDateDaysIterator and NaiveDateWeeksIterator should implement FusedIterator,
// TrustedLen, and Step once they becomes stable.
// See: https://github.com/chronotope/chrono/issues/208

/// The `Debug` output of the naive date `d` is the same as
/// [`d.format("%Y-%m-%d")`](../format/strftime/index.html).
///
/// The string printed can be readily parsed via the `parse` method on `str`.
///
/// # Example
///
/// ```
/// use chrono::NaiveDate;
///
/// assert_eq!(format!("{:?}", NaiveDate::from_ymd_opt(2015,  9,  5).unwrap()), "2015-09-05");
/// assert_eq!(format!("{:?}", NaiveDate::from_ymd_opt(   0,  1,  1).unwrap()), "0000-01-01");
/// assert_eq!(format!("{:?}", NaiveDate::from_ymd_opt(9999, 12, 31).unwrap()), "9999-12-31");
/// ```
///
/// ISO 8601 requires an explicit sign for years before 1 BCE or after 9999 CE.
///
/// ```
/// # use chrono::NaiveDate;
/// assert_eq!(format!("{:?}", NaiveDate::from_ymd_opt(   -1,  1,  1).unwrap()),  "-0001-01-01");
/// assert_eq!(format!("{:?}", NaiveDate::from_ymd_opt(10000, 12, 31).unwrap()), "+10000-12-31");
/// ```
impl fmt::Debug for NaiveDate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use core::fmt::Write;

        let year = self.year();
        let mdf = self.mdf();
        if (0..=9999).contains(&year) {
            write_hundreds(f, (year / 100) as u8)?;
            write_hundreds(f, (year % 100) as u8)?;
        } else {
            // ISO 8601 requires the explicit sign for out-of-range years
            write!(f, "{:+05}", year)?;
        }

        f.write_char('-')?;
        write_hundreds(f, mdf.month() as u8)?;
        f.write_char('-')?;
        write_hundreds(f, mdf.day() as u8)
    }
}

/// The `Display` output of the naive date `d` is the same as
/// [`d.format("%Y-%m-%d")`](../format/strftime/index.html).
///
/// The string printed can be readily parsed via the `parse` method on `str`.
///
/// # Example
///
/// ```
/// use chrono::NaiveDate;
///
/// assert_eq!(format!("{}", NaiveDate::from_ymd_opt(2015,  9,  5).unwrap()), "2015-09-05");
/// assert_eq!(format!("{}", NaiveDate::from_ymd_opt(   0,  1,  1).unwrap()), "0000-01-01");
/// assert_eq!(format!("{}", NaiveDate::from_ymd_opt(9999, 12, 31).unwrap()), "9999-12-31");
/// ```
///
/// ISO 8601 requires an explicit sign for years before 1 BCE or after 9999 CE.
///
/// ```
/// # use chrono::NaiveDate;
/// assert_eq!(format!("{}", NaiveDate::from_ymd_opt(   -1,  1,  1).unwrap()),  "-0001-01-01");
/// assert_eq!(format!("{}", NaiveDate::from_ymd_opt(10000, 12, 31).unwrap()), "+10000-12-31");
/// ```
impl fmt::Display for NaiveDate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// Parsing a `str` into a `NaiveDate` uses the same format,
/// [`%Y-%m-%d`](../format/strftime/index.html), as in `Debug` and `Display`.
///
/// # Example
///
/// ```
/// use chrono::NaiveDate;
///
/// let d = NaiveDate::from_ymd_opt(2015, 9, 18).unwrap();
/// assert_eq!("2015-09-18".parse::<NaiveDate>(), Ok(d));
///
/// let d = NaiveDate::from_ymd_opt(12345, 6, 7).unwrap();
/// assert_eq!("+12345-6-7".parse::<NaiveDate>(), Ok(d));
///
/// assert!("foo".parse::<NaiveDate>().is_err());
/// ```
impl str::FromStr for NaiveDate {
    type Err = ParseError;

    fn from_str(s: &str) -> ParseResult<NaiveDate> {
        const ITEMS: &[Item<'static>] = &[
            Item::Numeric(Numeric::Year, Pad::Zero),
            Item::Space(""),
            Item::Literal("-"),
            Item::Numeric(Numeric::Month, Pad::Zero),
            Item::Space(""),
            Item::Literal("-"),
            Item::Numeric(Numeric::Day, Pad::Zero),
            Item::Space(""),
        ];

        let mut parsed = Parsed::new();
        parse(&mut parsed, s, ITEMS.iter())?;
        parsed.to_naive_date()
    }
}

/// The default value for a NaiveDate is 1st of January 1970.
///
/// # Example
///
/// ```rust
/// use chrono::NaiveDate;
///
/// let default_date = NaiveDate::default();
/// assert_eq!(default_date, NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
/// ```
impl Default for NaiveDate {
    fn default() -> Self {
        NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
    }
}

#[cfg(all(test, any(feature = "rustc-serialize", feature = "serde")))]
fn test_encodable_json<F, E>(to_string: F)
where
    F: Fn(&NaiveDate) -> Result<String, E>,
    E: ::std::fmt::Debug,
{
    assert_eq!(
        to_string(&NaiveDate::from_ymd_opt(2014, 7, 24).unwrap()).ok(),
        Some(r#""2014-07-24""#.into())
    );
    assert_eq!(
        to_string(&NaiveDate::from_ymd_opt(0, 1, 1).unwrap()).ok(),
        Some(r#""0000-01-01""#.into())
    );
    assert_eq!(
        to_string(&NaiveDate::from_ymd_opt(-1, 12, 31).unwrap()).ok(),
        Some(r#""-0001-12-31""#.into())
    );
    assert_eq!(to_string(&NaiveDate::MIN).ok(), Some(r#""-262144-01-01""#.into()));
    assert_eq!(to_string(&NaiveDate::MAX).ok(), Some(r#""+262143-12-31""#.into()));
}

#[cfg(all(test, any(feature = "rustc-serialize", feature = "serde")))]
fn test_decodable_json<F, E>(from_str: F)
where
    F: Fn(&str) -> Result<NaiveDate, E>,
    E: ::std::fmt::Debug,
{
    use std::{i32, i64};

    assert_eq!(
        from_str(r#""2016-07-08""#).ok(),
        Some(NaiveDate::from_ymd_opt(2016, 7, 8).unwrap())
    );
    assert_eq!(from_str(r#""2016-7-8""#).ok(), Some(NaiveDate::from_ymd_opt(2016, 7, 8).unwrap()));
    assert_eq!(from_str(r#""+002016-07-08""#).ok(), NaiveDate::from_ymd_opt(2016, 7, 8));
    assert_eq!(from_str(r#""0000-01-01""#).ok(), Some(NaiveDate::from_ymd_opt(0, 1, 1).unwrap()));
    assert_eq!(from_str(r#""0-1-1""#).ok(), Some(NaiveDate::from_ymd_opt(0, 1, 1).unwrap()));
    assert_eq!(
        from_str(r#""-0001-12-31""#).ok(),
        Some(NaiveDate::from_ymd_opt(-1, 12, 31).unwrap())
    );
    assert_eq!(from_str(r#""-262144-01-01""#).ok(), Some(NaiveDate::MIN));
    assert_eq!(from_str(r#""+262143-12-31""#).ok(), Some(NaiveDate::MAX));

    // bad formats
    assert!(from_str(r#""""#).is_err());
    assert!(from_str(r#""20001231""#).is_err());
    assert!(from_str(r#""2000-00-00""#).is_err());
    assert!(from_str(r#""2000-02-30""#).is_err());
    assert!(from_str(r#""2001-02-29""#).is_err());
    assert!(from_str(r#""2002-002-28""#).is_err());
    assert!(from_str(r#""yyyy-mm-dd""#).is_err());
    assert!(from_str(r#"0"#).is_err());
    assert!(from_str(r#"20.01"#).is_err());
    assert!(from_str(&i32::MIN.to_string()).is_err());
    assert!(from_str(&i32::MAX.to_string()).is_err());
    assert!(from_str(&i64::MIN.to_string()).is_err());
    assert!(from_str(&i64::MAX.to_string()).is_err());
    assert!(from_str(r#"{}"#).is_err());
    // pre-0.3.0 rustc-serialize format is now invalid
    assert!(from_str(r#"{"ymdf":20}"#).is_err());
    assert!(from_str(r#"null"#).is_err());
}

#[cfg(feature = "rustc-serialize")]
#[cfg_attr(docsrs, doc(cfg(feature = "rustc-serialize")))]
mod rustc_serialize {
    use super::NaiveDate;
    use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};

    impl Encodable for NaiveDate {
        fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
            format!("{:?}", self).encode(s)
        }
    }

    impl Decodable for NaiveDate {
        fn decode<D: Decoder>(d: &mut D) -> Result<NaiveDate, D::Error> {
            d.read_str()?.parse().map_err(|_| d.error("invalid date"))
        }
    }

    #[cfg(test)]
    use rustc_serialize::json;

    #[test]
    fn test_encodable() {
        super::test_encodable_json(json::encode);
    }

    #[test]
    fn test_decodable() {
        super::test_decodable_json(json::decode);
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
mod serde {
    use super::NaiveDate;
    use core::fmt;
    use serde::{de, ser};

    // TODO not very optimized for space (binary formats would want something better)

    impl ser::Serialize for NaiveDate {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
        {
            struct FormatWrapped<'a, D: 'a> {
                inner: &'a D,
            }

            impl<'a, D: fmt::Debug> fmt::Display for FormatWrapped<'a, D> {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    self.inner.fmt(f)
                }
            }

            serializer.collect_str(&FormatWrapped { inner: &self })
        }
    }

    struct NaiveDateVisitor;

    impl<'de> de::Visitor<'de> for NaiveDateVisitor {
        type Value = NaiveDate;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a formatted date string")
        }

        #[cfg(any(feature = "std", test))]
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            value.parse().map_err(E::custom)
        }

        #[cfg(not(any(feature = "std", test)))]
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            value.parse().map_err(E::custom)
        }
    }

    impl<'de> de::Deserialize<'de> for NaiveDate {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            deserializer.deserialize_str(NaiveDateVisitor)
        }
    }

    #[test]
    fn test_serde_serialize() {
        super::test_encodable_json(serde_json::to_string);
    }

    #[test]
    fn test_serde_deserialize() {
        super::test_decodable_json(|input| serde_json::from_str(input));
    }

    #[test]
    fn test_serde_bincode() {
        // Bincode is relevant to test separately from JSON because
        // it is not self-describing.
        use bincode::{deserialize, serialize};

        let d = NaiveDate::from_ymd_opt(2014, 7, 24).unwrap();
        let encoded = serialize(&d).unwrap();
        let decoded: NaiveDate = deserialize(&encoded).unwrap();
        assert_eq!(d, decoded);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Days, Months, NaiveDate, MAX_DAYS_FROM_YEAR_0, MAX_YEAR, MIN_DAYS_FROM_YEAR_0, MIN_YEAR,
    };
    use crate::oldtime::Duration;
    use crate::{Datelike, Weekday};
    use std::{
        convert::{TryFrom, TryInto},
        i32, u32,
    };

    #[test]
    fn diff_months() {
        // identity
        assert_eq!(
            NaiveDate::from_ymd_opt(2022, 8, 3).unwrap().checked_add_months(Months::new(0)),
            Some(NaiveDate::from_ymd_opt(2022, 8, 3).unwrap())
        );

        // add with months exceeding `i32::MAX`
        assert_eq!(
            NaiveDate::from_ymd_opt(2022, 8, 3)
                .unwrap()
                .checked_add_months(Months::new(i32::MAX as u32 + 1)),
            None
        );

        // sub with months exceeindg `i32::MIN`
        assert_eq!(
            NaiveDate::from_ymd_opt(2022, 8, 3)
                .unwrap()
                .checked_sub_months(Months::new((i32::MIN as i64).abs() as u32 + 1)),
            None
        );

        // add overflowing year
        assert_eq!(NaiveDate::MAX.checked_add_months(Months::new(1)), None);

        // add underflowing year
        assert_eq!(NaiveDate::MIN.checked_sub_months(Months::new(1)), None);

        // sub crossing year 0 boundary
        assert_eq!(
            NaiveDate::from_ymd_opt(2022, 8, 3).unwrap().checked_sub_months(Months::new(2050 * 12)),
            Some(NaiveDate::from_ymd_opt(-28, 8, 3).unwrap())
        );

        // add crossing year boundary
        assert_eq!(
            NaiveDate::from_ymd_opt(2022, 8, 3).unwrap().checked_add_months(Months::new(6)),
            Some(NaiveDate::from_ymd_opt(2023, 2, 3).unwrap())
        );

        // sub crossing year boundary
        assert_eq!(
            NaiveDate::from_ymd_opt(2022, 8, 3).unwrap().checked_sub_months(Months::new(10)),
            Some(NaiveDate::from_ymd_opt(2021, 10, 3).unwrap())
        );

        // add clamping day, non-leap year
        assert_eq!(
            NaiveDate::from_ymd_opt(2022, 1, 29).unwrap().checked_add_months(Months::new(1)),
            Some(NaiveDate::from_ymd_opt(2022, 2, 28).unwrap())
        );

        // add to leap day
        assert_eq!(
            NaiveDate::from_ymd_opt(2022, 10, 29).unwrap().checked_add_months(Months::new(16)),
            Some(NaiveDate::from_ymd_opt(2024, 2, 29).unwrap())
        );

        // add into december
        assert_eq!(
            NaiveDate::from_ymd_opt(2022, 10, 31).unwrap().checked_add_months(Months::new(2)),
            Some(NaiveDate::from_ymd_opt(2022, 12, 31).unwrap())
        );

        // sub into december
        assert_eq!(
            NaiveDate::from_ymd_opt(2022, 10, 31).unwrap().checked_sub_months(Months::new(10)),
            Some(NaiveDate::from_ymd_opt(2021, 12, 31).unwrap())
        );

        // add into january
        assert_eq!(
            NaiveDate::from_ymd_opt(2022, 8, 3).unwrap().checked_add_months(Months::new(5)),
            Some(NaiveDate::from_ymd_opt(2023, 1, 3).unwrap())
        );

        // sub into january
        assert_eq!(
            NaiveDate::from_ymd_opt(2022, 8, 3).unwrap().checked_sub_months(Months::new(7)),
            Some(NaiveDate::from_ymd_opt(2022, 1, 3).unwrap())
        );
    }

    #[test]
    fn test_readme_doomsday() {
        use num_iter::range_inclusive;

        for y in range_inclusive(NaiveDate::MIN.year(), NaiveDate::MAX.year()) {
            // even months
            let d4 = NaiveDate::from_ymd_opt(y, 4, 4).unwrap();
            let d6 = NaiveDate::from_ymd_opt(y, 6, 6).unwrap();
            let d8 = NaiveDate::from_ymd_opt(y, 8, 8).unwrap();
            let d10 = NaiveDate::from_ymd_opt(y, 10, 10).unwrap();
            let d12 = NaiveDate::from_ymd_opt(y, 12, 12).unwrap();

            // nine to five, seven-eleven
            let d59 = NaiveDate::from_ymd_opt(y, 5, 9).unwrap();
            let d95 = NaiveDate::from_ymd_opt(y, 9, 5).unwrap();
            let d711 = NaiveDate::from_ymd_opt(y, 7, 11).unwrap();
            let d117 = NaiveDate::from_ymd_opt(y, 11, 7).unwrap();

            // "March 0"
            let d30 = NaiveDate::from_ymd_opt(y, 3, 1).unwrap().pred_opt().unwrap();

            let weekday = d30.weekday();
            let other_dates = [d4, d6, d8, d10, d12, d59, d95, d711, d117];
            assert!(other_dates.iter().all(|d| d.weekday() == weekday));
        }
    }

    #[test]
    fn test_date_from_ymd() {
        let ymd_opt = NaiveDate::from_ymd_opt;

        assert!(ymd_opt(2012, 0, 1).is_none());
        assert!(ymd_opt(2012, 1, 1).is_some());
        assert!(ymd_opt(2012, 2, 29).is_some());
        assert!(ymd_opt(2014, 2, 29).is_none());
        assert!(ymd_opt(2014, 3, 0).is_none());
        assert!(ymd_opt(2014, 3, 1).is_some());
        assert!(ymd_opt(2014, 3, 31).is_some());
        assert!(ymd_opt(2014, 3, 32).is_none());
        assert!(ymd_opt(2014, 12, 31).is_some());
        assert!(ymd_opt(2014, 13, 1).is_none());
    }

    #[test]
    fn test_date_from_yo() {
        let yo_opt = NaiveDate::from_yo_opt;
        let ymd = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();

        assert_eq!(yo_opt(2012, 0), None);
        assert_eq!(yo_opt(2012, 1), Some(ymd(2012, 1, 1)));
        assert_eq!(yo_opt(2012, 2), Some(ymd(2012, 1, 2)));
        assert_eq!(yo_opt(2012, 32), Some(ymd(2012, 2, 1)));
        assert_eq!(yo_opt(2012, 60), Some(ymd(2012, 2, 29)));
        assert_eq!(yo_opt(2012, 61), Some(ymd(2012, 3, 1)));
        assert_eq!(yo_opt(2012, 100), Some(ymd(2012, 4, 9)));
        assert_eq!(yo_opt(2012, 200), Some(ymd(2012, 7, 18)));
        assert_eq!(yo_opt(2012, 300), Some(ymd(2012, 10, 26)));
        assert_eq!(yo_opt(2012, 366), Some(ymd(2012, 12, 31)));
        assert_eq!(yo_opt(2012, 367), None);

        assert_eq!(yo_opt(2014, 0), None);
        assert_eq!(yo_opt(2014, 1), Some(ymd(2014, 1, 1)));
        assert_eq!(yo_opt(2014, 2), Some(ymd(2014, 1, 2)));
        assert_eq!(yo_opt(2014, 32), Some(ymd(2014, 2, 1)));
        assert_eq!(yo_opt(2014, 59), Some(ymd(2014, 2, 28)));
        assert_eq!(yo_opt(2014, 60), Some(ymd(2014, 3, 1)));
        assert_eq!(yo_opt(2014, 100), Some(ymd(2014, 4, 10)));
        assert_eq!(yo_opt(2014, 200), Some(ymd(2014, 7, 19)));
        assert_eq!(yo_opt(2014, 300), Some(ymd(2014, 10, 27)));
        assert_eq!(yo_opt(2014, 365), Some(ymd(2014, 12, 31)));
        assert_eq!(yo_opt(2014, 366), None);
    }

    #[test]
    fn test_date_from_isoywd() {
        let isoywd_opt = NaiveDate::from_isoywd_opt;
        let ymd = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();

        assert_eq!(isoywd_opt(2004, 0, Weekday::Sun), None);
        assert_eq!(isoywd_opt(2004, 1, Weekday::Mon), Some(ymd(2003, 12, 29)));
        assert_eq!(isoywd_opt(2004, 1, Weekday::Sun), Some(ymd(2004, 1, 4)));
        assert_eq!(isoywd_opt(2004, 2, Weekday::Mon), Some(ymd(2004, 1, 5)));
        assert_eq!(isoywd_opt(2004, 2, Weekday::Sun), Some(ymd(2004, 1, 11)));
        assert_eq!(isoywd_opt(2004, 52, Weekday::Mon), Some(ymd(2004, 12, 20)));
        assert_eq!(isoywd_opt(2004, 52, Weekday::Sun), Some(ymd(2004, 12, 26)));
        assert_eq!(isoywd_opt(2004, 53, Weekday::Mon), Some(ymd(2004, 12, 27)));
        assert_eq!(isoywd_opt(2004, 53, Weekday::Sun), Some(ymd(2005, 1, 2)));
        assert_eq!(isoywd_opt(2004, 54, Weekday::Mon), None);

        assert_eq!(isoywd_opt(2011, 0, Weekday::Sun), None);
        assert_eq!(isoywd_opt(2011, 1, Weekday::Mon), Some(ymd(2011, 1, 3)));
        assert_eq!(isoywd_opt(2011, 1, Weekday::Sun), Some(ymd(2011, 1, 9)));
        assert_eq!(isoywd_opt(2011, 2, Weekday::Mon), Some(ymd(2011, 1, 10)));
        assert_eq!(isoywd_opt(2011, 2, Weekday::Sun), Some(ymd(2011, 1, 16)));

        assert_eq!(isoywd_opt(2018, 51, Weekday::Mon), Some(ymd(2018, 12, 17)));
        assert_eq!(isoywd_opt(2018, 51, Weekday::Sun), Some(ymd(2018, 12, 23)));
        assert_eq!(isoywd_opt(2018, 52, Weekday::Mon), Some(ymd(2018, 12, 24)));
        assert_eq!(isoywd_opt(2018, 52, Weekday::Sun), Some(ymd(2018, 12, 30)));
        assert_eq!(isoywd_opt(2018, 53, Weekday::Mon), None);
    }

    #[test]
    fn test_date_from_isoywd_and_iso_week() {
        for year in 2000..2401 {
            for week in 1..54 {
                for &weekday in [
                    Weekday::Mon,
                    Weekday::Tue,
                    Weekday::Wed,
                    Weekday::Thu,
                    Weekday::Fri,
                    Weekday::Sat,
                    Weekday::Sun,
                ]
                .iter()
                {
                    let d = NaiveDate::from_isoywd_opt(year, week, weekday);
                    if let Some(d) = d {
                        assert_eq!(d.weekday(), weekday);
                        let w = d.iso_week();
                        assert_eq!(w.year(), year);
                        assert_eq!(w.week(), week);
                    }
                }
            }
        }

        for year in 2000..2401 {
            for month in 1..13 {
                for day in 1..32 {
                    let d = NaiveDate::from_ymd_opt(year, month, day);
                    if let Some(d) = d {
                        let w = d.iso_week();
                        let d_ = NaiveDate::from_isoywd_opt(w.year(), w.week(), d.weekday());
                        assert_eq!(d, d_.unwrap());
                    }
                }
            }
        }
    }

    #[test]
    fn test_date_from_num_days_from_ce() {
        let from_ndays_from_ce = NaiveDate::from_num_days_from_ce_opt;
        assert_eq!(from_ndays_from_ce(1), Some(NaiveDate::from_ymd_opt(1, 1, 1).unwrap()));
        assert_eq!(from_ndays_from_ce(2), Some(NaiveDate::from_ymd_opt(1, 1, 2).unwrap()));
        assert_eq!(from_ndays_from_ce(31), Some(NaiveDate::from_ymd_opt(1, 1, 31).unwrap()));
        assert_eq!(from_ndays_from_ce(32), Some(NaiveDate::from_ymd_opt(1, 2, 1).unwrap()));
        assert_eq!(from_ndays_from_ce(59), Some(NaiveDate::from_ymd_opt(1, 2, 28).unwrap()));
        assert_eq!(from_ndays_from_ce(60), Some(NaiveDate::from_ymd_opt(1, 3, 1).unwrap()));
        assert_eq!(from_ndays_from_ce(365), Some(NaiveDate::from_ymd_opt(1, 12, 31).unwrap()));
        assert_eq!(from_ndays_from_ce(365 + 1), Some(NaiveDate::from_ymd_opt(2, 1, 1).unwrap()));
        assert_eq!(
            from_ndays_from_ce(365 * 2 + 1),
            Some(NaiveDate::from_ymd_opt(3, 1, 1).unwrap())
        );
        assert_eq!(
            from_ndays_from_ce(365 * 3 + 1),
            Some(NaiveDate::from_ymd_opt(4, 1, 1).unwrap())
        );
        assert_eq!(
            from_ndays_from_ce(365 * 4 + 2),
            Some(NaiveDate::from_ymd_opt(5, 1, 1).unwrap())
        );
        assert_eq!(
            from_ndays_from_ce(146097 + 1),
            Some(NaiveDate::from_ymd_opt(401, 1, 1).unwrap())
        );
        assert_eq!(
            from_ndays_from_ce(146097 * 5 + 1),
            Some(NaiveDate::from_ymd_opt(2001, 1, 1).unwrap())
        );
        assert_eq!(from_ndays_from_ce(719163), Some(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()));
        assert_eq!(from_ndays_from_ce(0), Some(NaiveDate::from_ymd_opt(0, 12, 31).unwrap())); // 1 BCE
        assert_eq!(from_ndays_from_ce(-365), Some(NaiveDate::from_ymd_opt(0, 1, 1).unwrap()));
        assert_eq!(from_ndays_from_ce(-366), Some(NaiveDate::from_ymd_opt(-1, 12, 31).unwrap())); // 2 BCE

        for days in (-9999..10001).map(|x| x * 100) {
            assert_eq!(from_ndays_from_ce(days).map(|d| d.num_days_from_ce()), Some(days));
        }

        assert_eq!(from_ndays_from_ce(NaiveDate::MIN.num_days_from_ce()), Some(NaiveDate::MIN));
        assert_eq!(from_ndays_from_ce(NaiveDate::MIN.num_days_from_ce() - 1), None);
        assert_eq!(from_ndays_from_ce(NaiveDate::MAX.num_days_from_ce()), Some(NaiveDate::MAX));
        assert_eq!(from_ndays_from_ce(NaiveDate::MAX.num_days_from_ce() + 1), None);
    }

    #[test]
    fn test_date_from_weekday_of_month_opt() {
        let ymwd = NaiveDate::from_weekday_of_month_opt;
        assert_eq!(ymwd(2018, 8, Weekday::Tue, 0), None);
        assert_eq!(
            ymwd(2018, 8, Weekday::Wed, 1),
            Some(NaiveDate::from_ymd_opt(2018, 8, 1).unwrap())
        );
        assert_eq!(
            ymwd(2018, 8, Weekday::Thu, 1),
            Some(NaiveDate::from_ymd_opt(2018, 8, 2).unwrap())
        );
        assert_eq!(
            ymwd(2018, 8, Weekday::Sun, 1),
            Some(NaiveDate::from_ymd_opt(2018, 8, 5).unwrap())
        );
        assert_eq!(
            ymwd(2018, 8, Weekday::Mon, 1),
            Some(NaiveDate::from_ymd_opt(2018, 8, 6).unwrap())
        );
        assert_eq!(
            ymwd(2018, 8, Weekday::Tue, 1),
            Some(NaiveDate::from_ymd_opt(2018, 8, 7).unwrap())
        );
        assert_eq!(
            ymwd(2018, 8, Weekday::Wed, 2),
            Some(NaiveDate::from_ymd_opt(2018, 8, 8).unwrap())
        );
        assert_eq!(
            ymwd(2018, 8, Weekday::Sun, 2),
            Some(NaiveDate::from_ymd_opt(2018, 8, 12).unwrap())
        );
        assert_eq!(
            ymwd(2018, 8, Weekday::Thu, 3),
            Some(NaiveDate::from_ymd_opt(2018, 8, 16).unwrap())
        );
        assert_eq!(
            ymwd(2018, 8, Weekday::Thu, 4),
            Some(NaiveDate::from_ymd_opt(2018, 8, 23).unwrap())
        );
        assert_eq!(
            ymwd(2018, 8, Weekday::Thu, 5),
            Some(NaiveDate::from_ymd_opt(2018, 8, 30).unwrap())
        );
        assert_eq!(
            ymwd(2018, 8, Weekday::Fri, 5),
            Some(NaiveDate::from_ymd_opt(2018, 8, 31).unwrap())
        );
        assert_eq!(ymwd(2018, 8, Weekday::Sat, 5), None);
    }

    #[test]
    fn test_date_fields() {
        fn check(year: i32, month: u32, day: u32, ordinal: u32) {
            let d1 = NaiveDate::from_ymd_opt(year, month, day).unwrap();
            assert_eq!(d1.year(), year);
            assert_eq!(d1.month(), month);
            assert_eq!(d1.day(), day);
            assert_eq!(d1.ordinal(), ordinal);

            let d2 = NaiveDate::from_yo_opt(year, ordinal).unwrap();
            assert_eq!(d2.year(), year);
            assert_eq!(d2.month(), month);
            assert_eq!(d2.day(), day);
            assert_eq!(d2.ordinal(), ordinal);

            assert_eq!(d1, d2);
        }

        check(2012, 1, 1, 1);
        check(2012, 1, 2, 2);
        check(2012, 2, 1, 32);
        check(2012, 2, 29, 60);
        check(2012, 3, 1, 61);
        check(2012, 4, 9, 100);
        check(2012, 7, 18, 200);
        check(2012, 10, 26, 300);
        check(2012, 12, 31, 366);

        check(2014, 1, 1, 1);
        check(2014, 1, 2, 2);
        check(2014, 2, 1, 32);
        check(2014, 2, 28, 59);
        check(2014, 3, 1, 60);
        check(2014, 4, 10, 100);
        check(2014, 7, 19, 200);
        check(2014, 10, 27, 300);
        check(2014, 12, 31, 365);
    }

    #[test]
    fn test_date_weekday() {
        assert_eq!(NaiveDate::from_ymd_opt(1582, 10, 15).unwrap().weekday(), Weekday::Fri);
        // May 20, 1875 = ISO 8601 reference date
        assert_eq!(NaiveDate::from_ymd_opt(1875, 5, 20).unwrap().weekday(), Weekday::Thu);
        assert_eq!(NaiveDate::from_ymd_opt(2000, 1, 1).unwrap().weekday(), Weekday::Sat);
    }

    #[test]
    fn test_date_with_fields() {
        let d = NaiveDate::from_ymd_opt(2000, 2, 29).unwrap();
        assert_eq!(d.with_year(-400), Some(NaiveDate::from_ymd_opt(-400, 2, 29).unwrap()));
        assert_eq!(d.with_year(-100), None);
        assert_eq!(d.with_year(1600), Some(NaiveDate::from_ymd_opt(1600, 2, 29).unwrap()));
        assert_eq!(d.with_year(1900), None);
        assert_eq!(d.with_year(2000), Some(NaiveDate::from_ymd_opt(2000, 2, 29).unwrap()));
        assert_eq!(d.with_year(2001), None);
        assert_eq!(d.with_year(2004), Some(NaiveDate::from_ymd_opt(2004, 2, 29).unwrap()));
        assert_eq!(d.with_year(i32::MAX), None);

        let d = NaiveDate::from_ymd_opt(2000, 4, 30).unwrap();
        assert_eq!(d.with_month(0), None);
        assert_eq!(d.with_month(1), Some(NaiveDate::from_ymd_opt(2000, 1, 30).unwrap()));
        assert_eq!(d.with_month(2), None);
        assert_eq!(d.with_month(3), Some(NaiveDate::from_ymd_opt(2000, 3, 30).unwrap()));
        assert_eq!(d.with_month(4), Some(NaiveDate::from_ymd_opt(2000, 4, 30).unwrap()));
        assert_eq!(d.with_month(12), Some(NaiveDate::from_ymd_opt(2000, 12, 30).unwrap()));
        assert_eq!(d.with_month(13), None);
        assert_eq!(d.with_month(u32::MAX), None);

        let d = NaiveDate::from_ymd_opt(2000, 2, 8).unwrap();
        assert_eq!(d.with_day(0), None);
        assert_eq!(d.with_day(1), Some(NaiveDate::from_ymd_opt(2000, 2, 1).unwrap()));
        assert_eq!(d.with_day(29), Some(NaiveDate::from_ymd_opt(2000, 2, 29).unwrap()));
        assert_eq!(d.with_day(30), None);
        assert_eq!(d.with_day(u32::MAX), None);

        let d = NaiveDate::from_ymd_opt(2000, 5, 5).unwrap();
        assert_eq!(d.with_ordinal(0), None);
        assert_eq!(d.with_ordinal(1), Some(NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()));
        assert_eq!(d.with_ordinal(60), Some(NaiveDate::from_ymd_opt(2000, 2, 29).unwrap()));
        assert_eq!(d.with_ordinal(61), Some(NaiveDate::from_ymd_opt(2000, 3, 1).unwrap()));
        assert_eq!(d.with_ordinal(366), Some(NaiveDate::from_ymd_opt(2000, 12, 31).unwrap()));
        assert_eq!(d.with_ordinal(367), None);
        assert_eq!(d.with_ordinal(u32::MAX), None);
    }

    #[test]
    fn test_date_num_days_from_ce() {
        assert_eq!(NaiveDate::from_ymd_opt(1, 1, 1).unwrap().num_days_from_ce(), 1);

        for year in -9999..10001 {
            assert_eq!(
                NaiveDate::from_ymd_opt(year, 1, 1).unwrap().num_days_from_ce(),
                NaiveDate::from_ymd_opt(year - 1, 12, 31).unwrap().num_days_from_ce() + 1
            );
        }
    }

    #[test]
    fn test_date_succ() {
        let ymd = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
        assert_eq!(ymd(2014, 5, 6).succ_opt(), Some(ymd(2014, 5, 7)));
        assert_eq!(ymd(2014, 5, 31).succ_opt(), Some(ymd(2014, 6, 1)));
        assert_eq!(ymd(2014, 12, 31).succ_opt(), Some(ymd(2015, 1, 1)));
        assert_eq!(ymd(2016, 2, 28).succ_opt(), Some(ymd(2016, 2, 29)));
        assert_eq!(ymd(NaiveDate::MAX.year(), 12, 31).succ_opt(), None);
    }

    #[test]
    fn test_date_pred() {
        let ymd = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
        assert_eq!(ymd(2016, 3, 1).pred_opt(), Some(ymd(2016, 2, 29)));
        assert_eq!(ymd(2015, 1, 1).pred_opt(), Some(ymd(2014, 12, 31)));
        assert_eq!(ymd(2014, 6, 1).pred_opt(), Some(ymd(2014, 5, 31)));
        assert_eq!(ymd(2014, 5, 7).pred_opt(), Some(ymd(2014, 5, 6)));
        assert_eq!(ymd(NaiveDate::MIN.year(), 1, 1).pred_opt(), None);
    }

    #[test]
    fn test_date_add() {
        fn check((y1, m1, d1): (i32, u32, u32), rhs: Duration, ymd: Option<(i32, u32, u32)>) {
            let lhs = NaiveDate::from_ymd_opt(y1, m1, d1).unwrap();
            let sum = ymd.map(|(y, m, d)| NaiveDate::from_ymd_opt(y, m, d).unwrap());
            assert_eq!(lhs.checked_add_signed(rhs), sum);
            assert_eq!(lhs.checked_sub_signed(-rhs), sum);
        }

        check((2014, 1, 1), Duration::zero(), Some((2014, 1, 1)));
        check((2014, 1, 1), Duration::seconds(86399), Some((2014, 1, 1)));
        // always round towards zero
        check((2014, 1, 1), Duration::seconds(-86399), Some((2014, 1, 1)));
        check((2014, 1, 1), Duration::days(1), Some((2014, 1, 2)));
        check((2014, 1, 1), Duration::days(-1), Some((2013, 12, 31)));
        check((2014, 1, 1), Duration::days(364), Some((2014, 12, 31)));
        check((2014, 1, 1), Duration::days(365 * 4 + 1), Some((2018, 1, 1)));
        check((2014, 1, 1), Duration::days(365 * 400 + 97), Some((2414, 1, 1)));

        check((-7, 1, 1), Duration::days(365 * 12 + 3), Some((5, 1, 1)));

        // overflow check
        check((0, 1, 1), Duration::days(MAX_DAYS_FROM_YEAR_0 as i64), Some((MAX_YEAR, 12, 31)));
        check((0, 1, 1), Duration::days(MAX_DAYS_FROM_YEAR_0 as i64 + 1), None);
        check((0, 1, 1), Duration::max_value(), None);
        check((0, 1, 1), Duration::days(MIN_DAYS_FROM_YEAR_0 as i64), Some((MIN_YEAR, 1, 1)));
        check((0, 1, 1), Duration::days(MIN_DAYS_FROM_YEAR_0 as i64 - 1), None);
        check((0, 1, 1), Duration::min_value(), None);
    }

    #[test]
    fn test_date_sub() {
        fn check((y1, m1, d1): (i32, u32, u32), (y2, m2, d2): (i32, u32, u32), diff: Duration) {
            let lhs = NaiveDate::from_ymd_opt(y1, m1, d1).unwrap();
            let rhs = NaiveDate::from_ymd_opt(y2, m2, d2).unwrap();
            assert_eq!(lhs.signed_duration_since(rhs), diff);
            assert_eq!(rhs.signed_duration_since(lhs), -diff);
        }

        check((2014, 1, 1), (2014, 1, 1), Duration::zero());
        check((2014, 1, 2), (2014, 1, 1), Duration::days(1));
        check((2014, 12, 31), (2014, 1, 1), Duration::days(364));
        check((2015, 1, 3), (2014, 1, 1), Duration::days(365 + 2));
        check((2018, 1, 1), (2014, 1, 1), Duration::days(365 * 4 + 1));
        check((2414, 1, 1), (2014, 1, 1), Duration::days(365 * 400 + 97));

        check((MAX_YEAR, 12, 31), (0, 1, 1), Duration::days(MAX_DAYS_FROM_YEAR_0 as i64));
        check((MIN_YEAR, 1, 1), (0, 1, 1), Duration::days(MIN_DAYS_FROM_YEAR_0 as i64));
    }

    #[test]
    fn test_date_add_days() {
        fn check((y1, m1, d1): (i32, u32, u32), rhs: Days, ymd: Option<(i32, u32, u32)>) {
            let lhs = NaiveDate::from_ymd_opt(y1, m1, d1).unwrap();
            let sum = ymd.map(|(y, m, d)| NaiveDate::from_ymd_opt(y, m, d).unwrap());
            assert_eq!(lhs.checked_add_days(rhs), sum);
        }

        check((2014, 1, 1), Days::new(0), Some((2014, 1, 1)));
        // always round towards zero
        check((2014, 1, 1), Days::new(1), Some((2014, 1, 2)));
        check((2014, 1, 1), Days::new(364), Some((2014, 12, 31)));
        check((2014, 1, 1), Days::new(365 * 4 + 1), Some((2018, 1, 1)));
        check((2014, 1, 1), Days::new(365 * 400 + 97), Some((2414, 1, 1)));

        check((-7, 1, 1), Days::new(365 * 12 + 3), Some((5, 1, 1)));

        // overflow check
        check(
            (0, 1, 1),
            Days::new(MAX_DAYS_FROM_YEAR_0.try_into().unwrap()),
            Some((MAX_YEAR, 12, 31)),
        );
        check((0, 1, 1), Days::new(u64::try_from(MAX_DAYS_FROM_YEAR_0).unwrap() + 1), None);
    }

    #[test]
    fn test_date_sub_days() {
        fn check((y1, m1, d1): (i32, u32, u32), (y2, m2, d2): (i32, u32, u32), diff: Days) {
            let lhs = NaiveDate::from_ymd_opt(y1, m1, d1).unwrap();
            let rhs = NaiveDate::from_ymd_opt(y2, m2, d2).unwrap();
            assert_eq!(lhs - diff, rhs);
        }

        check((2014, 1, 1), (2014, 1, 1), Days::new(0));
        check((2014, 1, 2), (2014, 1, 1), Days::new(1));
        check((2014, 12, 31), (2014, 1, 1), Days::new(364));
        check((2015, 1, 3), (2014, 1, 1), Days::new(365 + 2));
        check((2018, 1, 1), (2014, 1, 1), Days::new(365 * 4 + 1));
        check((2414, 1, 1), (2014, 1, 1), Days::new(365 * 400 + 97));

        check((MAX_YEAR, 12, 31), (0, 1, 1), Days::new(MAX_DAYS_FROM_YEAR_0.try_into().unwrap()));
        check((0, 1, 1), (MIN_YEAR, 1, 1), Days::new((-MIN_DAYS_FROM_YEAR_0).try_into().unwrap()));
    }

    #[test]
    fn test_date_addassignment() {
        let ymd = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
        let mut date = ymd(2016, 10, 1);
        date += Duration::days(10);
        assert_eq!(date, ymd(2016, 10, 11));
        date += Duration::days(30);
        assert_eq!(date, ymd(2016, 11, 10));
    }

    #[test]
    fn test_date_subassignment() {
        let ymd = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
        let mut date = ymd(2016, 10, 11);
        date -= Duration::days(10);
        assert_eq!(date, ymd(2016, 10, 1));
        date -= Duration::days(2);
        assert_eq!(date, ymd(2016, 9, 29));
    }

    #[test]
    fn test_date_fmt() {
        assert_eq!(format!("{:?}", NaiveDate::from_ymd_opt(2012, 3, 4).unwrap()), "2012-03-04");
        assert_eq!(format!("{:?}", NaiveDate::from_ymd_opt(0, 3, 4).unwrap()), "0000-03-04");
        assert_eq!(format!("{:?}", NaiveDate::from_ymd_opt(-307, 3, 4).unwrap()), "-0307-03-04");
        assert_eq!(format!("{:?}", NaiveDate::from_ymd_opt(12345, 3, 4).unwrap()), "+12345-03-04");

        assert_eq!(NaiveDate::from_ymd_opt(2012, 3, 4).unwrap().to_string(), "2012-03-04");
        assert_eq!(NaiveDate::from_ymd_opt(0, 3, 4).unwrap().to_string(), "0000-03-04");
        assert_eq!(NaiveDate::from_ymd_opt(-307, 3, 4).unwrap().to_string(), "-0307-03-04");
        assert_eq!(NaiveDate::from_ymd_opt(12345, 3, 4).unwrap().to_string(), "+12345-03-04");

        // the format specifier should have no effect on `NaiveTime`
        assert_eq!(format!("{:+30?}", NaiveDate::from_ymd_opt(1234, 5, 6).unwrap()), "1234-05-06");
        assert_eq!(
            format!("{:30?}", NaiveDate::from_ymd_opt(12345, 6, 7).unwrap()),
            "+12345-06-07"
        );
    }

    #[test]
    fn test_date_from_str() {
        // valid cases
        let valid = [
            "-0000000123456-1-2",
            "    -123456 - 1 - 2    ",
            "-12345-1-2",
            "-1234-12-31",
            "-7-6-5",
            "350-2-28",
            "360-02-29",
            "0360-02-29",
            "2015-2 -18",
            "+70-2-18",
            "+70000-2-18",
            "+00007-2-18",
        ];
        for &s in &valid {
            let d = match s.parse::<NaiveDate>() {
                Ok(d) => d,
                Err(e) => panic!("parsing `{}` has failed: {}", s, e),
            };
            let s_ = format!("{:?}", d);
            // `s` and `s_` may differ, but `s.parse()` and `s_.parse()` must be same
            let d_ = match s_.parse::<NaiveDate>() {
                Ok(d) => d,
                Err(e) => {
                    panic!("`{}` is parsed into `{:?}`, but reparsing that has failed: {}", s, d, e)
                }
            };
            assert!(
                d == d_,
                "`{}` is parsed into `{:?}`, but reparsed result \
                              `{:?}` does not match",
                s,
                d,
                d_
            );
        }

        // some invalid cases
        // since `ParseErrorKind` is private, all we can do is to check if there was an error
        assert!("".parse::<NaiveDate>().is_err());
        assert!("x".parse::<NaiveDate>().is_err());
        assert!("2014".parse::<NaiveDate>().is_err());
        assert!("2014-01".parse::<NaiveDate>().is_err());
        assert!("2014-01-00".parse::<NaiveDate>().is_err());
        assert!("2014-13-57".parse::<NaiveDate>().is_err());
        assert!("9999999-9-9".parse::<NaiveDate>().is_err()); // out-of-bounds
    }

    #[test]
    fn test_date_parse_from_str() {
        let ymd = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
        assert_eq!(
            NaiveDate::parse_from_str("2014-5-7T12:34:56+09:30", "%Y-%m-%dT%H:%M:%S%z"),
            Ok(ymd(2014, 5, 7))
        ); // ignore time and offset
        assert_eq!(
            NaiveDate::parse_from_str("2015-W06-1=2015-033", "%G-W%V-%u = %Y-%j"),
            Ok(ymd(2015, 2, 2))
        );
        assert_eq!(
            NaiveDate::parse_from_str("Fri, 09 Aug 13", "%a, %d %b %y"),
            Ok(ymd(2013, 8, 9))
        );
        assert!(NaiveDate::parse_from_str("Sat, 09 Aug 2013", "%a, %d %b %Y").is_err());
        assert!(NaiveDate::parse_from_str("2014-57", "%Y-%m-%d").is_err());
        assert!(NaiveDate::parse_from_str("2014", "%Y").is_err()); // insufficient
    }

    #[test]
    fn test_date_format() {
        let d = NaiveDate::from_ymd_opt(2012, 3, 4).unwrap();
        assert_eq!(d.format("%Y,%C,%y,%G,%g").to_string(), "2012,20,12,2012,12");
        assert_eq!(d.format("%m,%b,%h,%B").to_string(), "03,Mar,Mar,March");
        assert_eq!(d.format("%d,%e").to_string(), "04, 4");
        assert_eq!(d.format("%U,%W,%V").to_string(), "10,09,09");
        assert_eq!(d.format("%a,%A,%w,%u").to_string(), "Sun,Sunday,0,7");
        assert_eq!(d.format("%j").to_string(), "064"); // since 2012 is a leap year
        assert_eq!(d.format("%D,%x").to_string(), "03/04/12,03/04/12");
        assert_eq!(d.format("%F").to_string(), "2012-03-04");
        assert_eq!(d.format("%v").to_string(), " 4-Mar-2012");
        assert_eq!(d.format("%t%n%%%n%t").to_string(), "\t\n%\n\t");

        // non-four-digit years
        assert_eq!(
            NaiveDate::from_ymd_opt(12345, 1, 1).unwrap().format("%Y").to_string(),
            "+12345"
        );
        assert_eq!(NaiveDate::from_ymd_opt(1234, 1, 1).unwrap().format("%Y").to_string(), "1234");
        assert_eq!(NaiveDate::from_ymd_opt(123, 1, 1).unwrap().format("%Y").to_string(), "0123");
        assert_eq!(NaiveDate::from_ymd_opt(12, 1, 1).unwrap().format("%Y").to_string(), "0012");
        assert_eq!(NaiveDate::from_ymd_opt(1, 1, 1).unwrap().format("%Y").to_string(), "0001");
        assert_eq!(NaiveDate::from_ymd_opt(0, 1, 1).unwrap().format("%Y").to_string(), "0000");
        assert_eq!(NaiveDate::from_ymd_opt(-1, 1, 1).unwrap().format("%Y").to_string(), "-0001");
        assert_eq!(NaiveDate::from_ymd_opt(-12, 1, 1).unwrap().format("%Y").to_string(), "-0012");
        assert_eq!(NaiveDate::from_ymd_opt(-123, 1, 1).unwrap().format("%Y").to_string(), "-0123");
        assert_eq!(NaiveDate::from_ymd_opt(-1234, 1, 1).unwrap().format("%Y").to_string(), "-1234");
        assert_eq!(
            NaiveDate::from_ymd_opt(-12345, 1, 1).unwrap().format("%Y").to_string(),
            "-12345"
        );

        // corner cases
        assert_eq!(
            NaiveDate::from_ymd_opt(2007, 12, 31).unwrap().format("%G,%g,%U,%W,%V").to_string(),
            "2008,08,53,53,01"
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(2010, 1, 3).unwrap().format("%G,%g,%U,%W,%V").to_string(),
            "2009,09,01,00,53"
        );
    }

    #[test]
    fn test_day_iterator_limit() {
        assert_eq!(NaiveDate::from_ymd_opt(262143, 12, 29).unwrap().iter_days().take(4).count(), 2);
        assert_eq!(
            NaiveDate::from_ymd_opt(-262144, 1, 3).unwrap().iter_days().rev().take(4).count(),
            2
        );
    }

    #[test]
    fn test_week_iterator_limit() {
        assert_eq!(
            NaiveDate::from_ymd_opt(262143, 12, 12).unwrap().iter_weeks().take(4).count(),
            2
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(-262144, 1, 15).unwrap().iter_weeks().rev().take(4).count(),
            2
        );
    }

    #[test]
    fn test_naiveweek() {
        let date = NaiveDate::from_ymd_opt(2022, 5, 18).unwrap();
        let asserts = vec![
            (Weekday::Mon, "2022-05-16", "2022-05-22"),
            (Weekday::Tue, "2022-05-17", "2022-05-23"),
            (Weekday::Wed, "2022-05-18", "2022-05-24"),
            (Weekday::Thu, "2022-05-12", "2022-05-18"),
            (Weekday::Fri, "2022-05-13", "2022-05-19"),
            (Weekday::Sat, "2022-05-14", "2022-05-20"),
            (Weekday::Sun, "2022-05-15", "2022-05-21"),
        ];
        for (start, first_day, last_day) in asserts {
            let week = date.week(start);
            let days = week.days();
            assert_eq!(Ok(week.first_day()), NaiveDate::parse_from_str(first_day, "%Y-%m-%d"));
            assert_eq!(Ok(week.last_day()), NaiveDate::parse_from_str(last_day, "%Y-%m-%d"));
            assert!(days.contains(&date));
        }
    }
}
