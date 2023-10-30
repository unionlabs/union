// This is a part of Chrono.
// See README.md and LICENSE.txt for details.

//! ISO 8601 date and time with time zone.

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::string::{String, ToString};
#[cfg(any(feature = "alloc", feature = "std", test))]
use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt::Write;
use core::ops::{Add, AddAssign, Sub, SubAssign};
use core::{fmt, hash, str};
#[cfg(feature = "std")]
use std::string::ToString;
#[cfg(any(feature = "std", test))]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(any(feature = "alloc", feature = "std", test))]
use crate::format::DelayedFormat;
#[cfg(feature = "unstable-locales")]
use crate::format::Locale;
use crate::format::{parse, ParseError, ParseResult, Parsed, StrftimeItems};
use crate::format::{Fixed, Item};
use crate::naive::{Days, IsoWeek, NaiveDate, NaiveDateTime, NaiveTime};
#[cfg(feature = "clock")]
use crate::offset::Local;
use crate::offset::{FixedOffset, Offset, TimeZone, Utc};
use crate::oldtime::Duration as OldDuration;
#[allow(deprecated)]
use crate::Date;
use crate::Months;
use crate::{Datelike, Timelike, Weekday};

#[cfg(feature = "rkyv")]
use rkyv::{Archive, Deserialize, Serialize};

#[cfg(feature = "rustc-serialize")]
pub(super) mod rustc_serialize;

/// documented at re-export site
#[cfg(feature = "serde")]
pub(super) mod serde;

#[cfg(test)]
mod tests;

/// Specific formatting options for seconds. This may be extended in the
/// future, so exhaustive matching in external code is not recommended.
///
/// See the `TimeZone::to_rfc3339_opts` function for usage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecondsFormat {
    /// Format whole seconds only, with no decimal point nor subseconds.
    Secs,

    /// Use fixed 3 subsecond digits. This corresponds to
    /// [Fixed::Nanosecond3](format/enum.Fixed.html#variant.Nanosecond3).
    Millis,

    /// Use fixed 6 subsecond digits. This corresponds to
    /// [Fixed::Nanosecond6](format/enum.Fixed.html#variant.Nanosecond6).
    Micros,

    /// Use fixed 9 subsecond digits. This corresponds to
    /// [Fixed::Nanosecond9](format/enum.Fixed.html#variant.Nanosecond9).
    Nanos,

    /// Automatically select one of `Secs`, `Millis`, `Micros`, or `Nanos` to
    /// display all available non-zero sub-second digits.  This corresponds to
    /// [Fixed::Nanosecond](format/enum.Fixed.html#variant.Nanosecond).
    AutoSi,

    // Do not match against this.
    #[doc(hidden)]
    __NonExhaustive,
}

/// ISO 8601 combined date and time with time zone.
///
/// There are some constructors implemented here (the `from_*` methods), but
/// the general-purpose constructors are all via the methods on the
/// [`TimeZone`](./offset/trait.TimeZone.html) implementations.
#[derive(Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, Deserialize, Serialize))]
pub struct DateTime<Tz: TimeZone> {
    datetime: NaiveDateTime,
    offset: Tz::Offset,
}

/// The minimum possible `DateTime<Utc>`.
#[deprecated(since = "0.4.20", note = "Use DateTime::MIN_UTC instead")]
pub const MIN_DATETIME: DateTime<Utc> = DateTime::<Utc>::MIN_UTC;
/// The maximum possible `DateTime<Utc>`.
#[deprecated(since = "0.4.20", note = "Use DateTime::MAX_UTC instead")]
pub const MAX_DATETIME: DateTime<Utc> = DateTime::<Utc>::MAX_UTC;

impl<Tz: TimeZone> DateTime<Tz> {
    /// Makes a new `DateTime` with given *UTC* datetime and offset.
    /// The local datetime should be constructed via the `TimeZone` trait.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};
    ///
    /// let dt = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc);
    /// assert_eq!(Utc.timestamp(61, 0), dt);
    /// ```
    //
    // note: this constructor is purposely not named to `new` to discourage the direct usage.
    #[inline]
    pub fn from_utc(datetime: NaiveDateTime, offset: Tz::Offset) -> DateTime<Tz> {
        DateTime { datetime, offset }
    }

    /// Makes a new `DateTime` with given **local** datetime and offset that
    /// presents local timezone.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::DateTime;
    /// use chrono::naive::NaiveDate;
    /// use chrono::offset::{Utc, FixedOffset};
    ///
    /// let naivedatetime_utc = NaiveDate::from_ymd_opt(2000, 1, 12).unwrap().and_hms_opt(2, 0, 0).unwrap();
    /// let datetime_utc = DateTime::<Utc>::from_utc(naivedatetime_utc, Utc);
    ///
    /// let timezone_east = FixedOffset::east_opt(8 * 60 * 60).unwrap();
    /// let naivedatetime_east = NaiveDate::from_ymd_opt(2000, 1, 12).unwrap().and_hms_opt(10, 0, 0).unwrap();
    /// let datetime_east = DateTime::<FixedOffset>::from_local(naivedatetime_east, timezone_east);
    ///
    /// let timezone_west = FixedOffset::west_opt(7 * 60 * 60).unwrap();
    /// let naivedatetime_west = NaiveDate::from_ymd_opt(2000, 1, 11).unwrap().and_hms_opt(19, 0, 0).unwrap();
    /// let datetime_west = DateTime::<FixedOffset>::from_local(naivedatetime_west, timezone_west);

    /// assert_eq!(datetime_east, datetime_utc.with_timezone(&timezone_east));
    /// assert_eq!(datetime_west, datetime_utc.with_timezone(&timezone_west));
    /// ```
    #[inline]
    pub fn from_local(datetime: NaiveDateTime, offset: Tz::Offset) -> DateTime<Tz> {
        let datetime_utc = datetime - offset.fix();

        DateTime { datetime: datetime_utc, offset }
    }

    /// Retrieves a date component
    ///
    /// Unless you are immediately planning on turning this into a `DateTime`
    /// with the same Timezone you should use the
    /// [`date_naive`](DateTime::date_naive) method.
    #[inline]
    #[deprecated(since = "0.4.23", note = "Use `date_naive()` instead")]
    #[allow(deprecated)]
    pub fn date(&self) -> Date<Tz> {
        Date::from_utc(self.naive_local().date(), self.offset.clone())
    }

    /// Retrieves the Date without an associated timezone
    ///
    /// [`NaiveDate`] is a more well-defined type, and has more traits implemented on it,
    /// so should be preferred to [`Date`] any time you truly want to operate on Dates.
    ///
    /// ```
    /// use chrono::prelude::*;
    ///
    /// let date: DateTime<Utc> = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
    /// let other: DateTime<FixedOffset> = FixedOffset::east_opt(23).unwrap().with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
    /// assert_eq!(date.date_naive(), other.date_naive());
    /// ```
    #[inline]
    pub fn date_naive(&self) -> NaiveDate {
        let local = self.naive_local();
        NaiveDate::from_ymd_opt(local.year(), local.month(), local.day()).unwrap()
    }

    /// Retrieves a time component.
    /// Unlike `date`, this is not associated to the time zone.
    #[inline]
    pub fn time(&self) -> NaiveTime {
        self.datetime.time() + self.offset.fix()
    }

    /// Returns the number of non-leap seconds since January 1, 1970 0:00:00 UTC
    /// (aka "UNIX timestamp").
    #[inline]
    pub fn timestamp(&self) -> i64 {
        self.datetime.timestamp()
    }

    /// Returns the number of non-leap-milliseconds since January 1, 1970 UTC
    ///
    /// Note that this does reduce the number of years that can be represented
    /// from ~584 Billion to ~584 Million. (If this is a problem, please file
    /// an issue to let me know what domain needs millisecond precision over
    /// billions of years, I'm curious.)
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{Utc, TimeZone, NaiveDate};
    ///
    /// let dt = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap().and_hms_milli_opt(0, 0, 1, 444).unwrap().and_local_timezone(Utc).unwrap();
    /// assert_eq!(dt.timestamp_millis(), 1_444);
    ///
    /// let dt = NaiveDate::from_ymd_opt(2001, 9, 9).unwrap().and_hms_milli_opt(1, 46, 40, 555).unwrap().and_local_timezone(Utc).unwrap();
    /// assert_eq!(dt.timestamp_millis(), 1_000_000_000_555);
    /// ```
    #[inline]
    pub fn timestamp_millis(&self) -> i64 {
        self.datetime.timestamp_millis()
    }

    /// Returns the number of non-leap-microseconds since January 1, 1970 UTC
    ///
    /// Note that this does reduce the number of years that can be represented
    /// from ~584 Billion to ~584 Thousand. (If this is a problem, please file
    /// an issue to let me know what domain needs microsecond precision over
    /// millennia, I'm curious.)
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{Utc, TimeZone, NaiveDate};
    ///
    /// let dt = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap().and_hms_micro_opt(0, 0, 1, 444).unwrap().and_local_timezone(Utc).unwrap();
    /// assert_eq!(dt.timestamp_micros(), 1_000_444);
    ///
    /// let dt = NaiveDate::from_ymd_opt(2001, 9, 9).unwrap().and_hms_micro_opt(1, 46, 40, 555).unwrap().and_local_timezone(Utc).unwrap();
    /// assert_eq!(dt.timestamp_micros(), 1_000_000_000_000_555);
    /// ```
    #[inline]
    pub fn timestamp_micros(&self) -> i64 {
        self.datetime.timestamp_micros()
    }

    /// Returns the number of non-leap-nanoseconds since January 1, 1970 UTC
    ///
    /// Note that this does reduce the number of years that can be represented
    /// from ~584 Billion to ~584. (If this is a problem, please file
    /// an issue to let me know what domain needs nanosecond precision over
    /// millennia, I'm curious.)
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{Utc, TimeZone, NaiveDate};
    ///
    /// let dt = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap().and_hms_nano_opt(0, 0, 1, 444).unwrap().and_local_timezone(Utc).unwrap();
    /// assert_eq!(dt.timestamp_nanos(), 1_000_000_444);
    ///
    /// let dt = NaiveDate::from_ymd_opt(2001, 9, 9).unwrap().and_hms_nano_opt(1, 46, 40, 555).unwrap().and_local_timezone(Utc).unwrap();
    /// assert_eq!(dt.timestamp_nanos(), 1_000_000_000_000_000_555);
    /// ```
    #[inline]
    pub fn timestamp_nanos(&self) -> i64 {
        self.datetime.timestamp_nanos()
    }

    /// Returns the number of milliseconds since the last second boundary
    ///
    /// warning: in event of a leap second, this may exceed 999
    ///
    /// note: this is not the number of milliseconds since January 1, 1970 0:00:00 UTC
    #[inline]
    pub fn timestamp_subsec_millis(&self) -> u32 {
        self.datetime.timestamp_subsec_millis()
    }

    /// Returns the number of microseconds since the last second boundary
    ///
    /// warning: in event of a leap second, this may exceed 999_999
    ///
    /// note: this is not the number of microseconds since January 1, 1970 0:00:00 UTC
    #[inline]
    pub fn timestamp_subsec_micros(&self) -> u32 {
        self.datetime.timestamp_subsec_micros()
    }

    /// Returns the number of nanoseconds since the last second boundary
    ///
    /// warning: in event of a leap second, this may exceed 999_999_999
    ///
    /// note: this is not the number of nanoseconds since January 1, 1970 0:00:00 UTC
    #[inline]
    pub fn timestamp_subsec_nanos(&self) -> u32 {
        self.datetime.timestamp_subsec_nanos()
    }

    /// Retrieves an associated offset from UTC.
    #[inline]
    pub fn offset(&self) -> &Tz::Offset {
        &self.offset
    }

    /// Retrieves an associated time zone.
    #[inline]
    pub fn timezone(&self) -> Tz {
        TimeZone::from_offset(&self.offset)
    }

    /// Changes the associated time zone.
    /// The returned `DateTime` references the same instant of time from the perspective of the provided time zone.
    #[inline]
    pub fn with_timezone<Tz2: TimeZone>(&self, tz: &Tz2) -> DateTime<Tz2> {
        tz.from_utc_datetime(&self.datetime)
    }

    /// Adds given `Duration` to the current date and time.
    ///
    /// Returns `None` when it will result in overflow.
    #[inline]
    pub fn checked_add_signed(self, rhs: OldDuration) -> Option<DateTime<Tz>> {
        let datetime = self.datetime.checked_add_signed(rhs)?;
        let tz = self.timezone();
        Some(tz.from_utc_datetime(&datetime))
    }

    /// Adds given `Months` to the current date and time.
    ///
    /// Returns `None` when it will result in overflow, or if the
    /// local time is not valid on the newly calculated date.
    ///
    /// See [`NaiveDate::checked_add_months`] for more details on behavior
    pub fn checked_add_months(self, rhs: Months) -> Option<DateTime<Tz>> {
        self.naive_local()
            .checked_add_months(rhs)?
            .and_local_timezone(Tz::from_offset(&self.offset))
            .single()
    }

    /// Subtracts given `Duration` from the current date and time.
    ///
    /// Returns `None` when it will result in overflow.
    #[inline]
    pub fn checked_sub_signed(self, rhs: OldDuration) -> Option<DateTime<Tz>> {
        let datetime = self.datetime.checked_sub_signed(rhs)?;
        let tz = self.timezone();
        Some(tz.from_utc_datetime(&datetime))
    }

    /// Subtracts given `Months` from the current date and time.
    ///
    /// Returns `None` when it will result in overflow, or if the
    /// local time is not valid on the newly calculated date.
    ///
    /// See [`NaiveDate::checked_sub_months`] for more details on behavior
    pub fn checked_sub_months(self, rhs: Months) -> Option<DateTime<Tz>> {
        self.naive_local()
            .checked_sub_months(rhs)?
            .and_local_timezone(Tz::from_offset(&self.offset))
            .single()
    }

    /// Add a duration in [`Days`] to the date part of the `DateTime`
    ///
    /// Returns `None` if the resulting date would be out of range.
    pub fn checked_add_days(self, days: Days) -> Option<Self> {
        self.datetime
            .checked_add_days(days)?
            .and_local_timezone(TimeZone::from_offset(&self.offset))
            .single()
    }

    /// Subtract a duration in [`Days`] from the date part of the `DateTime`
    ///
    /// Returns `None` if the resulting date would be out of range.
    pub fn checked_sub_days(self, days: Days) -> Option<Self> {
        self.datetime
            .checked_sub_days(days)?
            .and_local_timezone(TimeZone::from_offset(&self.offset))
            .single()
    }

    /// Subtracts another `DateTime` from the current date and time.
    /// This does not overflow or underflow at all.
    #[inline]
    pub fn signed_duration_since<Tz2: TimeZone>(self, rhs: DateTime<Tz2>) -> OldDuration {
        self.datetime.signed_duration_since(rhs.datetime)
    }

    /// Returns a view to the naive UTC datetime.
    #[inline]
    pub fn naive_utc(&self) -> NaiveDateTime {
        self.datetime
    }

    /// Returns a view to the naive local datetime.
    #[inline]
    pub fn naive_local(&self) -> NaiveDateTime {
        self.datetime + self.offset.fix()
    }

    /// Retrieve the elapsed years from now to the given [`DateTime`].
    pub fn years_since(&self, base: Self) -> Option<u32> {
        let mut years = self.year() - base.year();
        let earlier_time =
            (self.month(), self.day(), self.time()) < (base.month(), base.day(), base.time());

        years -= match earlier_time {
            true => 1,
            false => 0,
        };

        match years >= 0 {
            true => Some(years as u32),
            false => None,
        }
    }

    /// The minimum possible `DateTime<Utc>`.
    pub const MIN_UTC: DateTime<Utc> = DateTime { datetime: NaiveDateTime::MIN, offset: Utc };
    /// The maximum possible `DateTime<Utc>`.
    pub const MAX_UTC: DateTime<Utc> = DateTime { datetime: NaiveDateTime::MAX, offset: Utc };
}

impl Default for DateTime<Utc> {
    fn default() -> Self {
        Utc.from_utc_datetime(&NaiveDateTime::default())
    }
}

#[cfg(feature = "clock")]
#[cfg_attr(docsrs, doc(cfg(feature = "clock")))]
impl Default for DateTime<Local> {
    fn default() -> Self {
        Local.from_utc_datetime(&NaiveDateTime::default())
    }
}

impl Default for DateTime<FixedOffset> {
    fn default() -> Self {
        FixedOffset::west_opt(0).unwrap().from_utc_datetime(&NaiveDateTime::default())
    }
}

/// Convert a `DateTime<Utc>` instance into a `DateTime<FixedOffset>` instance.
impl From<DateTime<Utc>> for DateTime<FixedOffset> {
    /// Convert this `DateTime<Utc>` instance into a `DateTime<FixedOffset>` instance.
    ///
    /// Conversion is done via [`DateTime::with_timezone`]. Note that the converted value returned by
    /// this will be created with a fixed timezone offset of 0.
    fn from(src: DateTime<Utc>) -> Self {
        src.with_timezone(&FixedOffset::east_opt(0).unwrap())
    }
}

/// Convert a `DateTime<Utc>` instance into a `DateTime<Local>` instance.
#[cfg(feature = "clock")]
#[cfg_attr(docsrs, doc(cfg(feature = "clock")))]
impl From<DateTime<Utc>> for DateTime<Local> {
    /// Convert this `DateTime<Utc>` instance into a `DateTime<Local>` instance.
    ///
    /// Conversion is performed via [`DateTime::with_timezone`], accounting for the difference in timezones.
    fn from(src: DateTime<Utc>) -> Self {
        src.with_timezone(&Local)
    }
}

/// Convert a `DateTime<FixedOffset>` instance into a `DateTime<Utc>` instance.
impl From<DateTime<FixedOffset>> for DateTime<Utc> {
    /// Convert this `DateTime<FixedOffset>` instance into a `DateTime<Utc>` instance.
    ///
    /// Conversion is performed via [`DateTime::with_timezone`], accounting for the timezone
    /// difference.
    fn from(src: DateTime<FixedOffset>) -> Self {
        src.with_timezone(&Utc)
    }
}

/// Convert a `DateTime<FixedOffset>` instance into a `DateTime<Local>` instance.
#[cfg(feature = "clock")]
#[cfg_attr(docsrs, doc(cfg(feature = "clock")))]
impl From<DateTime<FixedOffset>> for DateTime<Local> {
    /// Convert this `DateTime<FixedOffset>` instance into a `DateTime<Local>` instance.
    ///
    /// Conversion is performed via [`DateTime::with_timezone`]. Returns the equivalent value in local
    /// time.
    fn from(src: DateTime<FixedOffset>) -> Self {
        src.with_timezone(&Local)
    }
}

/// Convert a `DateTime<Local>` instance into a `DateTime<Utc>` instance.
#[cfg(feature = "clock")]
#[cfg_attr(docsrs, doc(cfg(feature = "clock")))]
impl From<DateTime<Local>> for DateTime<Utc> {
    /// Convert this `DateTime<Local>` instance into a `DateTime<Utc>` instance.
    ///
    /// Conversion is performed via [`DateTime::with_timezone`], accounting for the difference in
    /// timezones.
    fn from(src: DateTime<Local>) -> Self {
        src.with_timezone(&Utc)
    }
}

/// Convert a `DateTime<Local>` instance into a `DateTime<FixedOffset>` instance.
#[cfg(feature = "clock")]
#[cfg_attr(docsrs, doc(cfg(feature = "clock")))]
impl From<DateTime<Local>> for DateTime<FixedOffset> {
    /// Convert this `DateTime<Local>` instance into a `DateTime<FixedOffset>` instance.
    ///
    /// Conversion is performed via [`DateTime::with_timezone`]. Note that the converted value returned
    /// by this will be created with a fixed timezone offset of 0.
    fn from(src: DateTime<Local>) -> Self {
        src.with_timezone(&FixedOffset::east_opt(0).unwrap())
    }
}

/// Maps the local datetime to other datetime with given conversion function.
fn map_local<Tz: TimeZone, F>(dt: &DateTime<Tz>, mut f: F) -> Option<DateTime<Tz>>
where
    F: FnMut(NaiveDateTime) -> Option<NaiveDateTime>,
{
    f(dt.naive_local()).and_then(|datetime| dt.timezone().from_local_datetime(&datetime).single())
}

impl DateTime<FixedOffset> {
    /// Parses an RFC 2822 date and time string such as `Tue, 1 Jul 2003 10:52:37 +0200`,
    /// then returns a new [`DateTime`] with a parsed [`FixedOffset`].
    ///
    /// RFC 2822 is the internet message standard that specifies the
    /// representation of times in HTTP and email headers.
    ///
    /// ```
    /// # use chrono::{DateTime, FixedOffset, TimeZone, NaiveDate};
    /// assert_eq!(
    ///     DateTime::parse_from_rfc2822("Wed, 18 Feb 2015 23:16:09 GMT").unwrap(),
    ///     FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(2015, 2, 18, 23, 16, 9).unwrap()
    /// );
    /// ```
    pub fn parse_from_rfc2822(s: &str) -> ParseResult<DateTime<FixedOffset>> {
        const ITEMS: &[Item<'static>] = &[Item::Fixed(Fixed::RFC2822)];
        let mut parsed = Parsed::new();
        parse(&mut parsed, s, ITEMS.iter())?;
        parsed.to_datetime()
    }

    /// Parses an RFC 3339 and ISO 8601 date and time string such as `1996-12-19T16:39:57-08:00`,
    /// then returns a new [`DateTime`] with a parsed [`FixedOffset`].
    ///
    /// Why isn't this named `parse_from_iso8601`? That's because ISO 8601 allows some freedom
    /// over the syntax and RFC 3339 exercises that freedom to rigidly define a fixed format.
    pub fn parse_from_rfc3339(s: &str) -> ParseResult<DateTime<FixedOffset>> {
        const ITEMS: &[Item<'static>] = &[Item::Fixed(Fixed::RFC3339)];
        let mut parsed = Parsed::new();
        parse(&mut parsed, s, ITEMS.iter())?;
        parsed.to_datetime()
    }

    /// Parses a string with the specified format string and returns a new
    /// [`DateTime`] with a parsed [`FixedOffset`].
    ///
    /// See the [`crate::format::strftime`] module on the supported escape
    /// sequences.
    ///
    /// See also [`TimeZone::datetime_from_str`] which gives a local
    /// [`DateTime`] on specific time zone.
    ///
    /// Note that this method *requires a timezone* in the string. See
    /// [`NaiveDateTime::parse_from_str`]
    /// for a version that does not require a timezone in the to-be-parsed str.
    ///
    /// # Example
    ///
    /// ```rust
    /// use chrono::{DateTime, FixedOffset, TimeZone, NaiveDate};
    ///
    /// let dt = DateTime::parse_from_str(
    ///     "1983 Apr 13 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z");
    /// assert_eq!(dt, Ok(FixedOffset::east_opt(0).unwrap().from_local_datetime(&NaiveDate::from_ymd_opt(1983, 4, 13).unwrap().and_hms_milli_opt(12, 9, 14, 274).unwrap()).unwrap()));
    /// ```
    pub fn parse_from_str(s: &str, fmt: &str) -> ParseResult<DateTime<FixedOffset>> {
        let mut parsed = Parsed::new();
        parse(&mut parsed, s, StrftimeItems::new(fmt))?;
        parsed.to_datetime()
    }
}

impl<Tz: TimeZone> DateTime<Tz>
where
    Tz::Offset: fmt::Display,
{
    /// Returns an RFC 2822 date and time string such as `Tue, 1 Jul 2003 10:52:37 +0200`.
    #[cfg(any(feature = "alloc", feature = "std", test))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
    pub fn to_rfc2822(&self) -> String {
        let mut result = String::with_capacity(32);
        crate::format::write_rfc2822(&mut result, self.naive_local(), self.offset.fix())
            .expect("writing rfc2822 datetime to string should never fail");
        result
    }

    /// Returns an RFC 3339 and ISO 8601 date and time string such as `1996-12-19T16:39:57-08:00`.
    #[cfg(any(feature = "alloc", feature = "std", test))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
    pub fn to_rfc3339(&self) -> String {
        let mut result = String::with_capacity(32);
        crate::format::write_rfc3339(&mut result, self.naive_local(), self.offset.fix())
            .expect("writing rfc3339 datetime to string should never fail");
        result
    }

    /// Return an RFC 3339 and ISO 8601 date and time string with subseconds
    /// formatted as per a `SecondsFormat`.
    ///
    /// If passed `use_z` true and the timezone is UTC (offset 0), use 'Z', as
    /// per [`Fixed::TimezoneOffsetColonZ`] If passed `use_z` false, use
    /// [`Fixed::TimezoneOffsetColon`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use chrono::{DateTime, FixedOffset, SecondsFormat, TimeZone, Utc, NaiveDate};
    /// let dt = NaiveDate::from_ymd_opt(2018, 1, 26).unwrap().and_hms_micro_opt(18, 30, 9, 453_829).unwrap().and_local_timezone(Utc).unwrap();
    /// assert_eq!(dt.to_rfc3339_opts(SecondsFormat::Millis, false),
    ///            "2018-01-26T18:30:09.453+00:00");
    /// assert_eq!(dt.to_rfc3339_opts(SecondsFormat::Millis, true),
    ///            "2018-01-26T18:30:09.453Z");
    /// assert_eq!(dt.to_rfc3339_opts(SecondsFormat::Secs, true),
    ///            "2018-01-26T18:30:09Z");
    ///
    /// let pst = FixedOffset::east_opt(8 * 60 * 60).unwrap();
    /// let dt = pst.from_local_datetime(&NaiveDate::from_ymd_opt(2018, 1, 26).unwrap().and_hms_micro_opt(10, 30, 9, 453_829).unwrap()).unwrap();
    /// assert_eq!(dt.to_rfc3339_opts(SecondsFormat::Secs, true),
    ///            "2018-01-26T10:30:09+08:00");
    /// ```
    #[cfg(any(feature = "alloc", feature = "std", test))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
    pub fn to_rfc3339_opts(&self, secform: SecondsFormat, use_z: bool) -> String {
        use crate::format::Numeric::*;
        use crate::format::Pad::Zero;
        use crate::SecondsFormat::*;

        debug_assert!(secform != __NonExhaustive, "Do not use __NonExhaustive!");

        const PREFIX: &[Item<'static>] = &[
            Item::Numeric(Year, Zero),
            Item::Literal("-"),
            Item::Numeric(Month, Zero),
            Item::Literal("-"),
            Item::Numeric(Day, Zero),
            Item::Literal("T"),
            Item::Numeric(Hour, Zero),
            Item::Literal(":"),
            Item::Numeric(Minute, Zero),
            Item::Literal(":"),
            Item::Numeric(Second, Zero),
        ];

        let ssitem = match secform {
            Secs => None,
            Millis => Some(Item::Fixed(Fixed::Nanosecond3)),
            Micros => Some(Item::Fixed(Fixed::Nanosecond6)),
            Nanos => Some(Item::Fixed(Fixed::Nanosecond9)),
            AutoSi => Some(Item::Fixed(Fixed::Nanosecond)),
            __NonExhaustive => unreachable!(),
        };

        let tzitem = Item::Fixed(if use_z {
            Fixed::TimezoneOffsetColonZ
        } else {
            Fixed::TimezoneOffsetColon
        });

        match ssitem {
            None => self.format_with_items(PREFIX.iter().chain([tzitem].iter())).to_string(),
            Some(s) => self.format_with_items(PREFIX.iter().chain([s, tzitem].iter())).to_string(),
        }
    }

    /// Formats the combined date and time with the specified formatting items.
    #[cfg(any(feature = "alloc", feature = "std", test))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
    #[inline]
    pub fn format_with_items<'a, I, B>(&self, items: I) -> DelayedFormat<I>
    where
        I: Iterator<Item = B> + Clone,
        B: Borrow<Item<'a>>,
    {
        let local = self.naive_local();
        DelayedFormat::new_with_offset(Some(local.date()), Some(local.time()), &self.offset, items)
    }

    /// Formats the combined date and time with the specified format string.
    /// See the [`crate::format::strftime`] module
    /// on the supported escape sequences.
    ///
    /// # Example
    /// ```rust
    /// use chrono::prelude::*;
    ///
    /// let date_time: DateTime<Utc> = Utc.with_ymd_and_hms(2017, 04, 02, 12, 50, 32).unwrap();
    /// let formatted = format!("{}", date_time.format("%d/%m/%Y %H:%M"));
    /// assert_eq!(formatted, "02/04/2017 12:50");
    /// ```
    #[cfg(any(feature = "alloc", feature = "std", test))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
    #[inline]
    pub fn format<'a>(&self, fmt: &'a str) -> DelayedFormat<StrftimeItems<'a>> {
        self.format_with_items(StrftimeItems::new(fmt))
    }

    /// Formats the combined date and time with the specified formatting items and locale.
    #[cfg(feature = "unstable-locales")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable-locales")))]
    #[inline]
    pub fn format_localized_with_items<'a, I, B>(
        &self,
        items: I,
        locale: Locale,
    ) -> DelayedFormat<I>
    where
        I: Iterator<Item = B> + Clone,
        B: Borrow<Item<'a>>,
    {
        let local = self.naive_local();
        DelayedFormat::new_with_offset_and_locale(
            Some(local.date()),
            Some(local.time()),
            &self.offset,
            items,
            locale,
        )
    }

    /// Formats the combined date and time with the specified format string and
    /// locale.
    ///
    /// See the [`crate::format::strftime`] module on the supported escape
    /// sequences.
    #[cfg(feature = "unstable-locales")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable-locales")))]
    #[inline]
    pub fn format_localized<'a>(
        &self,
        fmt: &'a str,
        locale: Locale,
    ) -> DelayedFormat<StrftimeItems<'a>> {
        self.format_localized_with_items(StrftimeItems::new_with_locale(fmt, locale), locale)
    }
}

impl<Tz: TimeZone> Datelike for DateTime<Tz> {
    #[inline]
    fn year(&self) -> i32 {
        self.naive_local().year()
    }
    #[inline]
    fn month(&self) -> u32 {
        self.naive_local().month()
    }
    #[inline]
    fn month0(&self) -> u32 {
        self.naive_local().month0()
    }
    #[inline]
    fn day(&self) -> u32 {
        self.naive_local().day()
    }
    #[inline]
    fn day0(&self) -> u32 {
        self.naive_local().day0()
    }
    #[inline]
    fn ordinal(&self) -> u32 {
        self.naive_local().ordinal()
    }
    #[inline]
    fn ordinal0(&self) -> u32 {
        self.naive_local().ordinal0()
    }
    #[inline]
    fn weekday(&self) -> Weekday {
        self.naive_local().weekday()
    }
    #[inline]
    fn iso_week(&self) -> IsoWeek {
        self.naive_local().iso_week()
    }

    #[inline]
    fn with_year(&self, year: i32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_year(year))
    }

    #[inline]
    fn with_month(&self, month: u32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_month(month))
    }

    #[inline]
    fn with_month0(&self, month0: u32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_month0(month0))
    }

    #[inline]
    fn with_day(&self, day: u32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_day(day))
    }

    #[inline]
    fn with_day0(&self, day0: u32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_day0(day0))
    }

    #[inline]
    fn with_ordinal(&self, ordinal: u32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_ordinal(ordinal))
    }

    #[inline]
    fn with_ordinal0(&self, ordinal0: u32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_ordinal0(ordinal0))
    }
}

impl<Tz: TimeZone> Timelike for DateTime<Tz> {
    #[inline]
    fn hour(&self) -> u32 {
        self.naive_local().hour()
    }
    #[inline]
    fn minute(&self) -> u32 {
        self.naive_local().minute()
    }
    #[inline]
    fn second(&self) -> u32 {
        self.naive_local().second()
    }
    #[inline]
    fn nanosecond(&self) -> u32 {
        self.naive_local().nanosecond()
    }

    #[inline]
    fn with_hour(&self, hour: u32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_hour(hour))
    }

    #[inline]
    fn with_minute(&self, min: u32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_minute(min))
    }

    #[inline]
    fn with_second(&self, sec: u32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_second(sec))
    }

    #[inline]
    fn with_nanosecond(&self, nano: u32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_nanosecond(nano))
    }
}

// we need them as automatic impls cannot handle associated types
impl<Tz: TimeZone> Copy for DateTime<Tz> where <Tz as TimeZone>::Offset: Copy {}
unsafe impl<Tz: TimeZone> Send for DateTime<Tz> where <Tz as TimeZone>::Offset: Send {}

impl<Tz: TimeZone, Tz2: TimeZone> PartialEq<DateTime<Tz2>> for DateTime<Tz> {
    fn eq(&self, other: &DateTime<Tz2>) -> bool {
        self.datetime == other.datetime
    }
}

impl<Tz: TimeZone> Eq for DateTime<Tz> {}

impl<Tz: TimeZone, Tz2: TimeZone> PartialOrd<DateTime<Tz2>> for DateTime<Tz> {
    /// Compare two DateTimes based on their true time, ignoring time zones
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::prelude::*;
    ///
    /// let earlier = Utc.with_ymd_and_hms(2015, 5, 15, 2, 0, 0).unwrap().with_timezone(&FixedOffset::west_opt(1 * 3600).unwrap());
    /// let later   = Utc.with_ymd_and_hms(2015, 5, 15, 3, 0, 0).unwrap().with_timezone(&FixedOffset::west_opt(5 * 3600).unwrap());
    ///
    /// assert_eq!(earlier.to_string(), "2015-05-15 01:00:00 -01:00");
    /// assert_eq!(later.to_string(), "2015-05-14 22:00:00 -05:00");
    ///
    /// assert!(later > earlier);
    /// ```
    fn partial_cmp(&self, other: &DateTime<Tz2>) -> Option<Ordering> {
        self.datetime.partial_cmp(&other.datetime)
    }
}

impl<Tz: TimeZone> Ord for DateTime<Tz> {
    fn cmp(&self, other: &DateTime<Tz>) -> Ordering {
        self.datetime.cmp(&other.datetime)
    }
}

impl<Tz: TimeZone> hash::Hash for DateTime<Tz> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.datetime.hash(state)
    }
}

impl<Tz: TimeZone> Add<OldDuration> for DateTime<Tz> {
    type Output = DateTime<Tz>;

    #[inline]
    fn add(self, rhs: OldDuration) -> DateTime<Tz> {
        self.checked_add_signed(rhs).expect("`DateTime + Duration` overflowed")
    }
}

impl<Tz: TimeZone> AddAssign<OldDuration> for DateTime<Tz> {
    #[inline]
    fn add_assign(&mut self, rhs: OldDuration) {
        let datetime =
            self.datetime.checked_add_signed(rhs).expect("`DateTime + Duration` overflowed");
        let tz = self.timezone();
        *self = tz.from_utc_datetime(&datetime);
    }
}

impl<Tz: TimeZone> Add<Months> for DateTime<Tz> {
    type Output = DateTime<Tz>;

    fn add(self, rhs: Months) -> Self::Output {
        self.checked_add_months(rhs).unwrap()
    }
}

impl<Tz: TimeZone> Sub<OldDuration> for DateTime<Tz> {
    type Output = DateTime<Tz>;

    #[inline]
    fn sub(self, rhs: OldDuration) -> DateTime<Tz> {
        self.checked_sub_signed(rhs).expect("`DateTime - Duration` overflowed")
    }
}

impl<Tz: TimeZone> SubAssign<OldDuration> for DateTime<Tz> {
    #[inline]
    fn sub_assign(&mut self, rhs: OldDuration) {
        let datetime =
            self.datetime.checked_sub_signed(rhs).expect("`DateTime - Duration` overflowed");
        let tz = self.timezone();
        *self = tz.from_utc_datetime(&datetime)
    }
}

impl<Tz: TimeZone> Sub<Months> for DateTime<Tz> {
    type Output = DateTime<Tz>;

    fn sub(self, rhs: Months) -> Self::Output {
        self.checked_sub_months(rhs).unwrap()
    }
}

impl<Tz: TimeZone> Sub<DateTime<Tz>> for DateTime<Tz> {
    type Output = OldDuration;

    #[inline]
    fn sub(self, rhs: DateTime<Tz>) -> OldDuration {
        self.signed_duration_since(rhs)
    }
}

impl<Tz: TimeZone> Add<Days> for DateTime<Tz> {
    type Output = DateTime<Tz>;

    fn add(self, days: Days) -> Self::Output {
        self.checked_add_days(days).unwrap()
    }
}

impl<Tz: TimeZone> Sub<Days> for DateTime<Tz> {
    type Output = DateTime<Tz>;

    fn sub(self, days: Days) -> Self::Output {
        self.checked_sub_days(days).unwrap()
    }
}

impl<Tz: TimeZone> fmt::Debug for DateTime<Tz> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.naive_local().fmt(f)?;
        self.offset.fmt(f)
    }
}

impl<Tz: TimeZone> fmt::Display for DateTime<Tz>
where
    Tz::Offset: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.naive_local().fmt(f)?;
        f.write_char(' ')?;
        self.offset.fmt(f)
    }
}

/// Accepts a relaxed form of RFC3339.
/// A space or a 'T' are acepted as the separator between the date and time
/// parts. Additional spaces are allowed between each component.
///
/// All of these examples are equivalent:
/// ```
/// # use chrono::{DateTime, Utc};
/// "2012-12-12T12:12:12Z".parse::<DateTime<Utc>>();
/// "2012-12-12 12:12:12Z".parse::<DateTime<Utc>>();
/// "2012-  12-12T12:  12:12Z".parse::<DateTime<Utc>>();
/// ```
impl str::FromStr for DateTime<Utc> {
    type Err = ParseError;

    fn from_str(s: &str) -> ParseResult<DateTime<Utc>> {
        s.parse::<DateTime<FixedOffset>>().map(|dt| dt.with_timezone(&Utc))
    }
}

/// Accepts a relaxed form of RFC3339.
/// A space or a 'T' are acepted as the separator between the date and time
/// parts. Additional spaces are allowed between each component.
///
/// All of these examples are equivalent:
/// ```
/// # use chrono::{DateTime, Local};
/// "2012-12-12T12:12:12Z".parse::<DateTime<Local>>();
/// "2012-12-12 12:12:12Z".parse::<DateTime<Local>>();
/// "2012-  12-12T12:  12:12Z".parse::<DateTime<Local>>();
/// ```
#[cfg(feature = "clock")]
#[cfg_attr(docsrs, doc(cfg(feature = "clock")))]
impl str::FromStr for DateTime<Local> {
    type Err = ParseError;

    fn from_str(s: &str) -> ParseResult<DateTime<Local>> {
        s.parse::<DateTime<FixedOffset>>().map(|dt| dt.with_timezone(&Local))
    }
}

#[cfg(any(feature = "std", test))]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl From<SystemTime> for DateTime<Utc> {
    fn from(t: SystemTime) -> DateTime<Utc> {
        let (sec, nsec) = match t.duration_since(UNIX_EPOCH) {
            Ok(dur) => (dur.as_secs() as i64, dur.subsec_nanos()),
            Err(e) => {
                // unlikely but should be handled
                let dur = e.duration();
                let (sec, nsec) = (dur.as_secs() as i64, dur.subsec_nanos());
                if nsec == 0 {
                    (-sec, 0)
                } else {
                    (-sec - 1, 1_000_000_000 - nsec)
                }
            }
        };
        Utc.timestamp_opt(sec, nsec).unwrap()
    }
}

#[cfg(feature = "clock")]
#[cfg_attr(docsrs, doc(cfg(feature = "clock")))]
impl From<SystemTime> for DateTime<Local> {
    fn from(t: SystemTime) -> DateTime<Local> {
        DateTime::<Utc>::from(t).with_timezone(&Local)
    }
}

#[cfg(any(feature = "std", test))]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<Tz: TimeZone> From<DateTime<Tz>> for SystemTime {
    fn from(dt: DateTime<Tz>) -> SystemTime {
        use std::time::Duration;

        let sec = dt.timestamp();
        let nsec = dt.timestamp_subsec_nanos();
        if sec < 0 {
            // unlikely but should be handled
            UNIX_EPOCH - Duration::new(-sec as u64, 0) + Duration::new(0, nsec)
        } else {
            UNIX_EPOCH + Duration::new(sec as u64, nsec)
        }
    }
}

#[cfg(all(
    target_arch = "wasm32",
    feature = "wasmbind",
    not(any(target_os = "emscripten", target_os = "wasi"))
))]
#[cfg_attr(
    docsrs,
    doc(cfg(all(
        target_arch = "wasm32",
        feature = "wasmbind",
        not(any(target_os = "emscripten", target_os = "wasi"))
    )))
)]
impl From<js_sys::Date> for DateTime<Utc> {
    fn from(date: js_sys::Date) -> DateTime<Utc> {
        DateTime::<Utc>::from(&date)
    }
}

#[cfg(all(
    target_arch = "wasm32",
    feature = "wasmbind",
    not(any(target_os = "emscripten", target_os = "wasi"))
))]
#[cfg_attr(
    docsrs,
    doc(cfg(all(
        target_arch = "wasm32",
        feature = "wasmbind",
        not(any(target_os = "emscripten", target_os = "wasi"))
    )))
)]
impl From<&js_sys::Date> for DateTime<Utc> {
    fn from(date: &js_sys::Date) -> DateTime<Utc> {
        Utc.timestamp_millis_opt(date.get_time() as i64).unwrap()
    }
}

#[cfg(all(
    target_arch = "wasm32",
    feature = "wasmbind",
    not(any(target_os = "emscripten", target_os = "wasi"))
))]
#[cfg_attr(
    docsrs,
    doc(cfg(all(
        target_arch = "wasm32",
        feature = "wasmbind",
        not(any(target_os = "emscripten", target_os = "wasi"))
    )))
)]
impl From<DateTime<Utc>> for js_sys::Date {
    /// Converts a `DateTime<Utc>` to a JS `Date`. The resulting value may be lossy,
    /// any values that have a millisecond timestamp value greater/less than ±8,640,000,000,000,000
    /// (April 20, 271821 BCE ~ September 13, 275760 CE) will become invalid dates in JS.
    fn from(date: DateTime<Utc>) -> js_sys::Date {
        let js_millis = wasm_bindgen::JsValue::from_f64(date.timestamp_millis() as f64);
        js_sys::Date::new(&js_millis)
    }
}

// Note that implementation of Arbitrary cannot be simply derived for DateTime<Tz>, due to
// the nontrivial bound <Tz as TimeZone>::Offset: Arbitrary.
#[cfg(feature = "arbitrary")]
impl<'a, Tz> arbitrary::Arbitrary<'a> for DateTime<Tz>
where
    Tz: TimeZone,
    <Tz as TimeZone>::Offset: arbitrary::Arbitrary<'a>,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<DateTime<Tz>> {
        let datetime = NaiveDateTime::arbitrary(u)?;
        let offset = <Tz as TimeZone>::Offset::arbitrary(u)?;
        Ok(DateTime::from_utc(datetime, offset))
    }
}

#[test]
fn test_add_sub_months() {
    let utc_dt = Utc.with_ymd_and_hms(2018, 9, 5, 23, 58, 0).unwrap();
    assert_eq!(utc_dt + Months::new(15), Utc.with_ymd_and_hms(2019, 12, 5, 23, 58, 0).unwrap());

    let utc_dt = Utc.with_ymd_and_hms(2020, 1, 31, 23, 58, 0).unwrap();
    assert_eq!(utc_dt + Months::new(1), Utc.with_ymd_and_hms(2020, 2, 29, 23, 58, 0).unwrap());
    assert_eq!(utc_dt + Months::new(2), Utc.with_ymd_and_hms(2020, 3, 31, 23, 58, 0).unwrap());

    let utc_dt = Utc.with_ymd_and_hms(2018, 9, 5, 23, 58, 0).unwrap();
    assert_eq!(utc_dt - Months::new(15), Utc.with_ymd_and_hms(2017, 6, 5, 23, 58, 0).unwrap());

    let utc_dt = Utc.with_ymd_and_hms(2020, 3, 31, 23, 58, 0).unwrap();
    assert_eq!(utc_dt - Months::new(1), Utc.with_ymd_and_hms(2020, 2, 29, 23, 58, 0).unwrap());
    assert_eq!(utc_dt - Months::new(2), Utc.with_ymd_and_hms(2020, 1, 31, 23, 58, 0).unwrap());
}

#[test]
fn test_auto_conversion() {
    let utc_dt = Utc.with_ymd_and_hms(2018, 9, 5, 23, 58, 0).unwrap();
    let cdt_dt = FixedOffset::west_opt(5 * 60 * 60)
        .unwrap()
        .with_ymd_and_hms(2018, 9, 5, 18, 58, 0)
        .unwrap();
    let utc_dt2: DateTime<Utc> = cdt_dt.into();
    assert_eq!(utc_dt, utc_dt2);
}

#[cfg(all(test, any(feature = "rustc-serialize", feature = "serde")))]
fn test_encodable_json<FUtc, FFixed, E>(to_string_utc: FUtc, to_string_fixed: FFixed)
where
    FUtc: Fn(&DateTime<Utc>) -> Result<String, E>,
    FFixed: Fn(&DateTime<FixedOffset>) -> Result<String, E>,
    E: ::core::fmt::Debug,
{
    assert_eq!(
        to_string_utc(&Utc.with_ymd_and_hms(2014, 7, 24, 12, 34, 6).unwrap()).ok(),
        Some(r#""2014-07-24T12:34:06Z""#.into())
    );

    assert_eq!(
        to_string_fixed(
            &FixedOffset::east_opt(3660).unwrap().with_ymd_and_hms(2014, 7, 24, 12, 34, 6).unwrap()
        )
        .ok(),
        Some(r#""2014-07-24T12:34:06+01:01""#.into())
    );
    assert_eq!(
        to_string_fixed(
            &FixedOffset::east_opt(3650).unwrap().with_ymd_and_hms(2014, 7, 24, 12, 34, 6).unwrap()
        )
        .ok(),
        Some(r#""2014-07-24T12:34:06+01:00:50""#.into())
    );
}

#[cfg(all(test, feature = "clock", any(feature = "rustc-serialize", feature = "serde")))]
fn test_decodable_json<FUtc, FFixed, FLocal, E>(
    utc_from_str: FUtc,
    fixed_from_str: FFixed,
    local_from_str: FLocal,
) where
    FUtc: Fn(&str) -> Result<DateTime<Utc>, E>,
    FFixed: Fn(&str) -> Result<DateTime<FixedOffset>, E>,
    FLocal: Fn(&str) -> Result<DateTime<Local>, E>,
    E: ::core::fmt::Debug,
{
    // should check against the offset as well (the normal DateTime comparison will ignore them)
    fn norm<Tz: TimeZone>(dt: &Option<DateTime<Tz>>) -> Option<(&DateTime<Tz>, &Tz::Offset)> {
        dt.as_ref().map(|dt| (dt, dt.offset()))
    }

    assert_eq!(
        norm(&utc_from_str(r#""2014-07-24T12:34:06Z""#).ok()),
        norm(&Some(Utc.with_ymd_and_hms(2014, 7, 24, 12, 34, 6).unwrap()))
    );
    assert_eq!(
        norm(&utc_from_str(r#""2014-07-24T13:57:06+01:23""#).ok()),
        norm(&Some(Utc.with_ymd_and_hms(2014, 7, 24, 12, 34, 6).unwrap()))
    );

    assert_eq!(
        norm(&fixed_from_str(r#""2014-07-24T12:34:06Z""#).ok()),
        norm(&Some(
            FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(2014, 7, 24, 12, 34, 6).unwrap()
        ))
    );
    assert_eq!(
        norm(&fixed_from_str(r#""2014-07-24T13:57:06+01:23""#).ok()),
        norm(&Some(
            FixedOffset::east_opt(60 * 60 + 23 * 60)
                .unwrap()
                .with_ymd_and_hms(2014, 7, 24, 13, 57, 6)
                .unwrap()
        ))
    );

    // we don't know the exact local offset but we can check that
    // the conversion didn't change the instant itself
    assert_eq!(
        local_from_str(r#""2014-07-24T12:34:06Z""#).expect("local shouuld parse"),
        Utc.with_ymd_and_hms(2014, 7, 24, 12, 34, 6).unwrap()
    );
    assert_eq!(
        local_from_str(r#""2014-07-24T13:57:06+01:23""#).expect("local should parse with offset"),
        Utc.with_ymd_and_hms(2014, 7, 24, 12, 34, 6).unwrap()
    );

    assert!(utc_from_str(r#""2014-07-32T12:34:06Z""#).is_err());
    assert!(fixed_from_str(r#""2014-07-32T12:34:06Z""#).is_err());
}

#[cfg(all(test, feature = "clock", feature = "rustc-serialize"))]
fn test_decodable_json_timestamps<FUtc, FFixed, FLocal, E>(
    utc_from_str: FUtc,
    fixed_from_str: FFixed,
    local_from_str: FLocal,
) where
    FUtc: Fn(&str) -> Result<rustc_serialize::TsSeconds<Utc>, E>,
    FFixed: Fn(&str) -> Result<rustc_serialize::TsSeconds<FixedOffset>, E>,
    FLocal: Fn(&str) -> Result<rustc_serialize::TsSeconds<Local>, E>,
    E: ::core::fmt::Debug,
{
    fn norm<Tz: TimeZone>(dt: &Option<DateTime<Tz>>) -> Option<(&DateTime<Tz>, &Tz::Offset)> {
        dt.as_ref().map(|dt| (dt, dt.offset()))
    }

    assert_eq!(
        norm(&utc_from_str("0").ok().map(DateTime::from)),
        norm(&Some(Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap()))
    );
    assert_eq!(
        norm(&utc_from_str("-1").ok().map(DateTime::from)),
        norm(&Some(Utc.with_ymd_and_hms(1969, 12, 31, 23, 59, 59).unwrap()))
    );

    assert_eq!(
        norm(&fixed_from_str("0").ok().map(DateTime::from)),
        norm(&Some(
            FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap()
        ))
    );
    assert_eq!(
        norm(&fixed_from_str("-1").ok().map(DateTime::from)),
        norm(&Some(
            FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(1969, 12, 31, 23, 59, 59).unwrap()
        ))
    );

    assert_eq!(
        *fixed_from_str("0").expect("0 timestamp should parse"),
        Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap()
    );
    assert_eq!(
        *local_from_str("-1").expect("-1 timestamp should parse"),
        Utc.with_ymd_and_hms(1969, 12, 31, 23, 59, 59).unwrap()
    );
}
