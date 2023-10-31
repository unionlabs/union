// This is a part of Chrono.
// See README.md and LICENSE.txt for details.

//! ISO 8601 week.

use core::fmt;

use super::internals::{DateImpl, Of, YearFlags};

#[cfg(feature = "rkyv")]
use rkyv::{Archive, Deserialize, Serialize};

/// ISO 8601 week.
///
/// This type, combined with [`Weekday`](../enum.Weekday.html),
/// constitutes the ISO 8601 [week date](./struct.NaiveDate.html#week-date).
/// One can retrieve this type from the existing [`Datelike`](../trait.Datelike.html) types
/// via the [`Datelike::iso_week`](../trait.Datelike.html#tymethod.iso_week) method.
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, Deserialize, Serialize))]
pub struct IsoWeek {
    // note that this allows for larger year range than `NaiveDate`.
    // this is crucial because we have an edge case for the first and last week supported,
    // which year number might not match the calendar year number.
    ywf: DateImpl, // (year << 10) | (week << 4) | flag
}

/// Returns the corresponding `IsoWeek` from the year and the `Of` internal value.
//
// internal use only. we don't expose the public constructor for `IsoWeek` for now,
// because the year range for the week date and the calendar date do not match and
// it is confusing to have a date that is out of range in one and not in another.
// currently we sidestep this issue by making `IsoWeek` fully dependent of `Datelike`.
pub(super) fn iso_week_from_yof(year: i32, of: Of) -> IsoWeek {
    let (rawweek, _) = of.isoweekdate_raw();
    let (year, week) = if rawweek < 1 {
        // previous year
        let prevlastweek = YearFlags::from_year(year - 1).nisoweeks();
        (year - 1, prevlastweek)
    } else {
        let lastweek = of.flags().nisoweeks();
        if rawweek > lastweek {
            // next year
            (year + 1, 1)
        } else {
            (year, rawweek)
        }
    };
    IsoWeek { ywf: (year << 10) | (week << 4) as DateImpl | DateImpl::from(of.flags().0) }
}

impl IsoWeek {
    /// Returns the year number for this ISO week.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{NaiveDate, Datelike, Weekday};
    ///
    /// let d = NaiveDate::from_isoywd(2015, 1, Weekday::Mon);
    /// assert_eq!(d.iso_week().year(), 2015);
    /// ```
    ///
    /// This year number might not match the calendar year number.
    /// Continuing the example...
    ///
    /// ```
    /// # use chrono::{NaiveDate, Datelike, Weekday};
    /// # let d = NaiveDate::from_isoywd(2015, 1, Weekday::Mon);
    /// assert_eq!(d.year(), 2014);
    /// assert_eq!(d, NaiveDate::from_ymd_opt(2014, 12, 29).unwrap());
    /// ```
    #[inline]
    pub fn year(&self) -> i32 {
        self.ywf >> 10
    }

    /// Returns the ISO week number starting from 1.
    ///
    /// The return value ranges from 1 to 53. (The last week of year differs by years.)
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{NaiveDate, Datelike, Weekday};
    ///
    /// let d = NaiveDate::from_isoywd(2015, 15, Weekday::Mon);
    /// assert_eq!(d.iso_week().week(), 15);
    /// ```
    #[inline]
    pub fn week(&self) -> u32 {
        ((self.ywf >> 4) & 0x3f) as u32
    }

    /// Returns the ISO week number starting from 0.
    ///
    /// The return value ranges from 0 to 52. (The last week of year differs by years.)
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{NaiveDate, Datelike, Weekday};
    ///
    /// let d = NaiveDate::from_isoywd(2015, 15, Weekday::Mon);
    /// assert_eq!(d.iso_week().week0(), 14);
    /// ```
    #[inline]
    pub fn week0(&self) -> u32 {
        ((self.ywf >> 4) & 0x3f) as u32 - 1
    }
}

/// The `Debug` output of the ISO week `w` is the same as
/// [`d.format("%G-W%V")`](../format/strftime/index.html)
/// where `d` is any `NaiveDate` value in that week.
///
/// # Example
///
/// ```
/// use chrono::{NaiveDate, Datelike};
///
/// assert_eq!(format!("{:?}", NaiveDate::from_ymd_opt(2015,  9,  5).unwrap().iso_week()), "2015-W36");
/// assert_eq!(format!("{:?}", NaiveDate::from_ymd_opt(   0,  1,  3).unwrap().iso_week()), "0000-W01");
/// assert_eq!(format!("{:?}", NaiveDate::from_ymd_opt(9999, 12, 31).unwrap().iso_week()), "9999-W52");
/// ```
///
/// ISO 8601 requires an explicit sign for years before 1 BCE or after 9999 CE.
///
/// ```
/// # use chrono::{NaiveDate, Datelike};
/// assert_eq!(format!("{:?}", NaiveDate::from_ymd_opt(    0,  1,  2).unwrap().iso_week()),  "-0001-W52");
/// assert_eq!(format!("{:?}", NaiveDate::from_ymd_opt(10000, 12, 31).unwrap().iso_week()), "+10000-W52");
/// ```
impl fmt::Debug for IsoWeek {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let year = self.year();
        let week = self.week();
        if (0..=9999).contains(&year) {
            write!(f, "{:04}-W{:02}", year, week)
        } else {
            // ISO 8601 requires the explicit sign for out-of-range years
            write!(f, "{:+05}-W{:02}", year, week)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::naive::{internals, NaiveDate};
    use crate::Datelike;

    #[test]
    fn test_iso_week_extremes() {
        let minweek = NaiveDate::MIN.iso_week();
        let maxweek = NaiveDate::MAX.iso_week();

        assert_eq!(minweek.year(), internals::MIN_YEAR);
        assert_eq!(minweek.week(), 1);
        assert_eq!(minweek.week0(), 0);
        assert_eq!(format!("{:?}", minweek), NaiveDate::MIN.format("%G-W%V").to_string());

        assert_eq!(maxweek.year(), internals::MAX_YEAR + 1);
        assert_eq!(maxweek.week(), 1);
        assert_eq!(maxweek.week0(), 0);
        assert_eq!(format!("{:?}", maxweek), NaiveDate::MAX.format("%G-W%V").to_string());
    }
}
