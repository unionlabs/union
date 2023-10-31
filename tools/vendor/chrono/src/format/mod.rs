// This is a part of Chrono.
// See README.md and LICENSE.txt for details.

//! Formatting (and parsing) utilities for date and time.
//!
//! This module provides the common types and routines to implement,
//! for example, [`DateTime::format`](../struct.DateTime.html#method.format) or
//! [`DateTime::parse_from_str`](../struct.DateTime.html#method.parse_from_str) methods.
//! For most cases you should use these high-level interfaces.
//!
//! Internally the formatting and parsing shares the same abstract **formatting items**,
//! which are just an [`Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html) of
//! the [`Item`](./enum.Item.html) type.
//! They are generated from more readable **format strings**;
//! currently Chrono supports a built-in syntax closely resembling
//! C's `strftime` format. The available options can be found [here](./strftime/index.html).
//!
//! # Example
//! ```rust
//! # use std::error::Error;
//! use chrono::prelude::*;
//!
//! let date_time = Utc.with_ymd_and_hms(2020, 11, 10, 0, 1, 32).unwrap();
//!
//! let formatted = format!("{}", date_time.format("%Y-%m-%d %H:%M:%S"));
//! assert_eq!(formatted, "2020-11-10 00:01:32");
//!
//! let parsed = Utc.datetime_from_str(&formatted, "%Y-%m-%d %H:%M:%S")?;
//! assert_eq!(parsed, date_time);
//! # Ok::<(), chrono::ParseError>(())
//! ```

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::string::{String, ToString};
#[cfg(any(feature = "alloc", feature = "std", test))]
use core::borrow::Borrow;
use core::fmt;
use core::fmt::Write;
use core::str::FromStr;
#[cfg(any(feature = "std", test))]
use std::error::Error;

#[cfg(any(feature = "alloc", feature = "std", test))]
use crate::naive::{NaiveDate, NaiveTime};
#[cfg(any(feature = "alloc", feature = "std", test))]
use crate::offset::{FixedOffset, Offset};
#[cfg(any(feature = "alloc", feature = "std", test))]
use crate::{Datelike, Timelike};
use crate::{Month, ParseMonthError, ParseWeekdayError, Weekday};

#[cfg(feature = "unstable-locales")]
pub(crate) mod locales;

pub use parse::parse;
pub use parsed::Parsed;
/// L10n locales.
#[cfg(feature = "unstable-locales")]
pub use pure_rust_locales::Locale;
pub use strftime::StrftimeItems;

#[cfg(not(feature = "unstable-locales"))]
#[allow(dead_code)]
#[derive(Debug)]
struct Locale;

/// An uninhabited type used for `InternalNumeric` and `InternalFixed` below.
#[derive(Clone, PartialEq, Eq)]
enum Void {}

/// Padding characters for numeric items.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Pad {
    /// No padding.
    None,
    /// Zero (`0`) padding.
    Zero,
    /// Space padding.
    Space,
}

/// Numeric item types.
/// They have associated formatting width (FW) and parsing width (PW).
///
/// The **formatting width** is the minimal width to be formatted.
/// If the number is too short, and the padding is not [`Pad::None`](./enum.Pad.html#variant.None),
/// then it is left-padded.
/// If the number is too long or (in some cases) negative, it is printed as is.
///
/// The **parsing width** is the maximal width to be scanned.
/// The parser only tries to consume from one to given number of digits (greedily).
/// It also trims the preceding whitespace if any.
/// It cannot parse the negative number, so some date and time cannot be formatted then
/// parsed with the same formatting items.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Numeric {
    /// Full Gregorian year (FW=4, PW=∞).
    /// May accept years before 1 BCE or after 9999 CE, given an initial sign.
    Year,
    /// Gregorian year divided by 100 (century number; FW=PW=2). Implies the non-negative year.
    YearDiv100,
    /// Gregorian year modulo 100 (FW=PW=2). Cannot be negative.
    YearMod100,
    /// Year in the ISO week date (FW=4, PW=∞).
    /// May accept years before 1 BCE or after 9999 CE, given an initial sign.
    IsoYear,
    /// Year in the ISO week date, divided by 100 (FW=PW=2). Implies the non-negative year.
    IsoYearDiv100,
    /// Year in the ISO week date, modulo 100 (FW=PW=2). Cannot be negative.
    IsoYearMod100,
    /// Month (FW=PW=2).
    Month,
    /// Day of the month (FW=PW=2).
    Day,
    /// Week number, where the week 1 starts at the first Sunday of January (FW=PW=2).
    WeekFromSun,
    /// Week number, where the week 1 starts at the first Monday of January (FW=PW=2).
    WeekFromMon,
    /// Week number in the ISO week date (FW=PW=2).
    IsoWeek,
    /// Day of the week, where Sunday = 0 and Saturday = 6 (FW=PW=1).
    NumDaysFromSun,
    /// Day of the week, where Monday = 1 and Sunday = 7 (FW=PW=1).
    WeekdayFromMon,
    /// Day of the year (FW=PW=3).
    Ordinal,
    /// Hour number in the 24-hour clocks (FW=PW=2).
    Hour,
    /// Hour number in the 12-hour clocks (FW=PW=2).
    Hour12,
    /// The number of minutes since the last whole hour (FW=PW=2).
    Minute,
    /// The number of seconds since the last whole minute (FW=PW=2).
    Second,
    /// The number of nanoseconds since the last whole second (FW=PW=9).
    /// Note that this is *not* left-aligned;
    /// see also [`Fixed::Nanosecond`](./enum.Fixed.html#variant.Nanosecond).
    Nanosecond,
    /// The number of non-leap seconds since the midnight UTC on January 1, 1970 (FW=1, PW=∞).
    /// For formatting, it assumes UTC upon the absence of time zone offset.
    Timestamp,

    /// Internal uses only.
    ///
    /// This item exists so that one can add additional internal-only formatting
    /// without breaking major compatibility (as enum variants cannot be selectively private).
    Internal(InternalNumeric),
}

/// An opaque type representing numeric item types for internal uses only.
pub struct InternalNumeric {
    _dummy: Void,
}

impl Clone for InternalNumeric {
    fn clone(&self) -> Self {
        match self._dummy {}
    }
}

impl PartialEq for InternalNumeric {
    fn eq(&self, _other: &InternalNumeric) -> bool {
        match self._dummy {}
    }
}

impl Eq for InternalNumeric {}

impl fmt::Debug for InternalNumeric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<InternalNumeric>")
    }
}

/// Fixed-format item types.
///
/// They have their own rules of formatting and parsing.
/// Otherwise noted, they print in the specified cases but parse case-insensitively.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Fixed {
    /// Abbreviated month names.
    ///
    /// Prints a three-letter-long name in the title case, reads the same name in any case.
    ShortMonthName,
    /// Full month names.
    ///
    /// Prints a full name in the title case, reads either a short or full name in any case.
    LongMonthName,
    /// Abbreviated day of the week names.
    ///
    /// Prints a three-letter-long name in the title case, reads the same name in any case.
    ShortWeekdayName,
    /// Full day of the week names.
    ///
    /// Prints a full name in the title case, reads either a short or full name in any case.
    LongWeekdayName,
    /// AM/PM.
    ///
    /// Prints in lower case, reads in any case.
    LowerAmPm,
    /// AM/PM.
    ///
    /// Prints in upper case, reads in any case.
    UpperAmPm,
    /// An optional dot plus one or more digits for left-aligned nanoseconds.
    /// May print nothing, 3, 6 or 9 digits according to the available accuracy.
    /// See also [`Numeric::Nanosecond`](./enum.Numeric.html#variant.Nanosecond).
    Nanosecond,
    /// Same as [`Nanosecond`](#variant.Nanosecond) but the accuracy is fixed to 3.
    Nanosecond3,
    /// Same as [`Nanosecond`](#variant.Nanosecond) but the accuracy is fixed to 6.
    Nanosecond6,
    /// Same as [`Nanosecond`](#variant.Nanosecond) but the accuracy is fixed to 9.
    Nanosecond9,
    /// Timezone name.
    ///
    /// It does not support parsing, its use in the parser is an immediate failure.
    TimezoneName,
    /// Offset from the local time to UTC (`+09:00` or `-04:00` or `+00:00`).
    ///
    /// In the parser, the colon can be omitted and/or surrounded with any amount of whitespace.
    /// The offset is limited from `-24:00` to `+24:00`,
    /// which is the same as [`FixedOffset`](../offset/struct.FixedOffset.html)'s range.
    TimezoneOffsetColon,
    /// Offset from the local time to UTC with seconds (`+09:00:00` or `-04:00:00` or `+00:00:00`).
    ///
    /// In the parser, the colon can be omitted and/or surrounded with any amount of whitespace.
    /// The offset is limited from `-24:00:00` to `+24:00:00`,
    /// which is the same as [`FixedOffset`](../offset/struct.FixedOffset.html)'s range.
    TimezoneOffsetDoubleColon,
    /// Offset from the local time to UTC without minutes (`+09` or `-04` or `+00`).
    ///
    /// In the parser, the colon can be omitted and/or surrounded with any amount of whitespace.
    /// The offset is limited from `-24` to `+24`,
    /// which is the same as [`FixedOffset`](../offset/struct.FixedOffset.html)'s range.
    TimezoneOffsetTripleColon,
    /// Offset from the local time to UTC (`+09:00` or `-04:00` or `Z`).
    ///
    /// In the parser, the colon can be omitted and/or surrounded with any amount of whitespace,
    /// and `Z` can be either in upper case or in lower case.
    /// The offset is limited from `-24:00` to `+24:00`,
    /// which is the same as [`FixedOffset`](../offset/struct.FixedOffset.html)'s range.
    TimezoneOffsetColonZ,
    /// Same as [`TimezoneOffsetColon`](#variant.TimezoneOffsetColon) but prints no colon.
    /// Parsing allows an optional colon.
    TimezoneOffset,
    /// Same as [`TimezoneOffsetColonZ`](#variant.TimezoneOffsetColonZ) but prints no colon.
    /// Parsing allows an optional colon.
    TimezoneOffsetZ,
    /// RFC 2822 date and time syntax. Commonly used for email and MIME date and time.
    RFC2822,
    /// RFC 3339 & ISO 8601 date and time syntax.
    RFC3339,

    /// Internal uses only.
    ///
    /// This item exists so that one can add additional internal-only formatting
    /// without breaking major compatibility (as enum variants cannot be selectively private).
    Internal(InternalFixed),
}

/// An opaque type representing fixed-format item types for internal uses only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternalFixed {
    val: InternalInternal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum InternalInternal {
    /// Same as [`TimezoneOffsetColonZ`](#variant.TimezoneOffsetColonZ), but
    /// allows missing minutes (per [ISO 8601][iso8601]).
    ///
    /// # Panics
    ///
    /// If you try to use this for printing.
    ///
    /// [iso8601]: https://en.wikipedia.org/wiki/ISO_8601#Time_offsets_from_UTC
    TimezoneOffsetPermissive,
    /// Same as [`Nanosecond`](#variant.Nanosecond) but the accuracy is fixed to 3 and there is no leading dot.
    Nanosecond3NoDot,
    /// Same as [`Nanosecond`](#variant.Nanosecond) but the accuracy is fixed to 6 and there is no leading dot.
    Nanosecond6NoDot,
    /// Same as [`Nanosecond`](#variant.Nanosecond) but the accuracy is fixed to 9 and there is no leading dot.
    Nanosecond9NoDot,
}

#[cfg(any(feature = "alloc", feature = "std", test))]
#[derive(Debug, Clone, PartialEq, Eq)]
enum Colons {
    None,
    Single,
    Double,
    Triple,
}

/// A single formatting item. This is used for both formatting and parsing.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Item<'a> {
    /// A literally printed and parsed text.
    Literal(&'a str),
    /// Same as `Literal` but with the string owned by the item.
    #[cfg(any(feature = "alloc", feature = "std", test))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
    OwnedLiteral(Box<str>),
    /// Whitespace. Prints literally but reads zero or more whitespace.
    Space(&'a str),
    /// Same as `Space` but with the string owned by the item.
    #[cfg(any(feature = "alloc", feature = "std", test))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
    OwnedSpace(Box<str>),
    /// Numeric item. Can be optionally padded to the maximal length (if any) when formatting;
    /// the parser simply ignores any padded whitespace and zeroes.
    Numeric(Numeric, Pad),
    /// Fixed-format item.
    Fixed(Fixed),
    /// Issues a formatting error. Used to signal an invalid format string.
    Error,
}

macro_rules! lit {
    ($x:expr) => {
        Item::Literal($x)
    };
}
macro_rules! sp {
    ($x:expr) => {
        Item::Space($x)
    };
}
macro_rules! num {
    ($x:ident) => {
        Item::Numeric(Numeric::$x, Pad::None)
    };
}
macro_rules! num0 {
    ($x:ident) => {
        Item::Numeric(Numeric::$x, Pad::Zero)
    };
}
macro_rules! nums {
    ($x:ident) => {
        Item::Numeric(Numeric::$x, Pad::Space)
    };
}
macro_rules! fix {
    ($x:ident) => {
        Item::Fixed(Fixed::$x)
    };
}
macro_rules! internal_fix {
    ($x:ident) => {
        Item::Fixed(Fixed::Internal(InternalFixed { val: InternalInternal::$x }))
    };
}

/// An error from the `parse` function.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct ParseError(ParseErrorKind);

impl ParseError {
    /// The category of parse error
    pub fn kind(&self) -> ParseErrorKind {
        self.0
    }
}

/// The category of parse error
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ParseErrorKind {
    /// Given field is out of permitted range.
    OutOfRange,

    /// There is no possible date and time value with given set of fields.
    ///
    /// This does not include the out-of-range conditions, which are trivially invalid.
    /// It includes the case that there are one or more fields that are inconsistent to each other.
    Impossible,

    /// Given set of fields is not enough to make a requested date and time value.
    ///
    /// Note that there *may* be a case that given fields constrain the possible values so much
    /// that there is a unique possible value. Chrono only tries to be correct for
    /// most useful sets of fields however, as such constraint solving can be expensive.
    NotEnough,

    /// The input string has some invalid character sequence for given formatting items.
    Invalid,

    /// The input string has been prematurely ended.
    TooShort,

    /// All formatting items have been read but there is a remaining input.
    TooLong,

    /// There was an error on the formatting string, or there were non-supported formating items.
    BadFormat,

    // TODO: Change this to `#[non_exhaustive]` (on the enum) when MSRV is increased
    #[doc(hidden)]
    __Nonexhaustive,
}

/// Same as `Result<T, ParseError>`.
pub type ParseResult<T> = Result<T, ParseError>;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            ParseErrorKind::OutOfRange => write!(f, "input is out of range"),
            ParseErrorKind::Impossible => write!(f, "no possible date and time matching input"),
            ParseErrorKind::NotEnough => write!(f, "input is not enough for unique date and time"),
            ParseErrorKind::Invalid => write!(f, "input contains invalid characters"),
            ParseErrorKind::TooShort => write!(f, "premature end of input"),
            ParseErrorKind::TooLong => write!(f, "trailing input"),
            ParseErrorKind::BadFormat => write!(f, "bad or unsupported format string"),
            _ => unreachable!(),
        }
    }
}

#[cfg(any(feature = "std", test))]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl Error for ParseError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        "parser error, see to_string() for details"
    }
}

// to be used in this module and submodules
const OUT_OF_RANGE: ParseError = ParseError(ParseErrorKind::OutOfRange);
const IMPOSSIBLE: ParseError = ParseError(ParseErrorKind::Impossible);
const NOT_ENOUGH: ParseError = ParseError(ParseErrorKind::NotEnough);
const INVALID: ParseError = ParseError(ParseErrorKind::Invalid);
const TOO_SHORT: ParseError = ParseError(ParseErrorKind::TooShort);
const TOO_LONG: ParseError = ParseError(ParseErrorKind::TooLong);
const BAD_FORMAT: ParseError = ParseError(ParseErrorKind::BadFormat);

#[cfg(any(feature = "alloc", feature = "std", test))]
struct Locales {
    short_months: &'static [&'static str],
    long_months: &'static [&'static str],
    short_weekdays: &'static [&'static str],
    long_weekdays: &'static [&'static str],
    am_pm: &'static [&'static str],
}

#[cfg(any(feature = "alloc", feature = "std", test))]
impl Locales {
    fn new(_locale: Option<Locale>) -> Self {
        #[cfg(feature = "unstable-locales")]
        {
            let locale = _locale.unwrap_or(Locale::POSIX);
            Self {
                short_months: locales::short_months(locale),
                long_months: locales::long_months(locale),
                short_weekdays: locales::short_weekdays(locale),
                long_weekdays: locales::long_weekdays(locale),
                am_pm: locales::am_pm(locale),
            }
        }
        #[cfg(not(feature = "unstable-locales"))]
        Self {
            short_months: &[
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ],
            long_months: &[
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ],
            short_weekdays: &["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
            long_weekdays: &[
                "Sunday",
                "Monday",
                "Tuesday",
                "Wednesday",
                "Thursday",
                "Friday",
                "Saturday",
            ],
            am_pm: &["AM", "PM"],
        }
    }
}

/// Formats single formatting item
#[cfg(any(feature = "alloc", feature = "std", test))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
pub fn format_item<'a>(
    w: &mut fmt::Formatter,
    date: Option<&NaiveDate>,
    time: Option<&NaiveTime>,
    off: Option<&(String, FixedOffset)>,
    item: &Item<'a>,
) -> fmt::Result {
    let mut result = String::new();
    format_inner(&mut result, date, time, off, item, None)?;
    w.pad(&result)
}

#[cfg(any(feature = "alloc", feature = "std", test))]
fn format_inner<'a>(
    result: &mut String,
    date: Option<&NaiveDate>,
    time: Option<&NaiveTime>,
    off: Option<&(String, FixedOffset)>,
    item: &Item<'a>,
    locale: Option<Locale>,
) -> fmt::Result {
    let locale = Locales::new(locale);

    use num_integer::{div_floor, mod_floor};

    match *item {
        Item::Literal(s) | Item::Space(s) => result.push_str(s),
        #[cfg(any(feature = "alloc", feature = "std", test))]
        Item::OwnedLiteral(ref s) | Item::OwnedSpace(ref s) => result.push_str(s),

        Item::Numeric(ref spec, ref pad) => {
            use self::Numeric::*;

            let week_from_sun = |d: &NaiveDate| {
                (d.ordinal() as i32 - d.weekday().num_days_from_sunday() as i32 + 7) / 7
            };
            let week_from_mon = |d: &NaiveDate| {
                (d.ordinal() as i32 - d.weekday().num_days_from_monday() as i32 + 7) / 7
            };

            let (width, v) = match *spec {
                Year => (4, date.map(|d| i64::from(d.year()))),
                YearDiv100 => (2, date.map(|d| div_floor(i64::from(d.year()), 100))),
                YearMod100 => (2, date.map(|d| mod_floor(i64::from(d.year()), 100))),
                IsoYear => (4, date.map(|d| i64::from(d.iso_week().year()))),
                IsoYearDiv100 => (2, date.map(|d| div_floor(i64::from(d.iso_week().year()), 100))),
                IsoYearMod100 => (2, date.map(|d| mod_floor(i64::from(d.iso_week().year()), 100))),
                Month => (2, date.map(|d| i64::from(d.month()))),
                Day => (2, date.map(|d| i64::from(d.day()))),
                WeekFromSun => (2, date.map(|d| i64::from(week_from_sun(d)))),
                WeekFromMon => (2, date.map(|d| i64::from(week_from_mon(d)))),
                IsoWeek => (2, date.map(|d| i64::from(d.iso_week().week()))),
                NumDaysFromSun => (1, date.map(|d| i64::from(d.weekday().num_days_from_sunday()))),
                WeekdayFromMon => (1, date.map(|d| i64::from(d.weekday().number_from_monday()))),
                Ordinal => (3, date.map(|d| i64::from(d.ordinal()))),
                Hour => (2, time.map(|t| i64::from(t.hour()))),
                Hour12 => (2, time.map(|t| i64::from(t.hour12().1))),
                Minute => (2, time.map(|t| i64::from(t.minute()))),
                Second => (2, time.map(|t| i64::from(t.second() + t.nanosecond() / 1_000_000_000))),
                Nanosecond => (9, time.map(|t| i64::from(t.nanosecond() % 1_000_000_000))),
                Timestamp => (
                    1,
                    match (date, time, off) {
                        (Some(d), Some(t), None) => Some(d.and_time(*t).timestamp()),
                        (Some(d), Some(t), Some(&(_, off))) => {
                            Some((d.and_time(*t) - off).timestamp())
                        }
                        (_, _, _) => None,
                    },
                ),

                // for the future expansion
                Internal(ref int) => match int._dummy {},
            };

            if let Some(v) = v {
                if (spec == &Year || spec == &IsoYear) && !(0..10_000).contains(&v) {
                    // non-four-digit years require an explicit sign as per ISO 8601
                    match *pad {
                        Pad::None => write!(result, "{:+}", v),
                        Pad::Zero => write!(result, "{:+01$}", v, width + 1),
                        Pad::Space => write!(result, "{:+1$}", v, width + 1),
                    }
                } else {
                    match *pad {
                        Pad::None => write!(result, "{}", v),
                        Pad::Zero => write!(result, "{:01$}", v, width),
                        Pad::Space => write!(result, "{:1$}", v, width),
                    }
                }?
            } else {
                return Err(fmt::Error); // insufficient arguments for given format
            }
        }

        Item::Fixed(ref spec) => {
            use self::Fixed::*;

            let ret =
                match *spec {
                    ShortMonthName => date.map(|d| {
                        result.push_str(locale.short_months[d.month0() as usize]);
                        Ok(())
                    }),
                    LongMonthName => date.map(|d| {
                        result.push_str(locale.long_months[d.month0() as usize]);
                        Ok(())
                    }),
                    ShortWeekdayName => date.map(|d| {
                        result.push_str(
                            locale.short_weekdays[d.weekday().num_days_from_sunday() as usize],
                        );
                        Ok(())
                    }),
                    LongWeekdayName => date.map(|d| {
                        result.push_str(
                            locale.long_weekdays[d.weekday().num_days_from_sunday() as usize],
                        );
                        Ok(())
                    }),
                    LowerAmPm => time.map(|t| {
                        let ampm = if t.hour12().0 { locale.am_pm[1] } else { locale.am_pm[0] };
                        for char in ampm.chars() {
                            result.extend(char.to_lowercase())
                        }
                        Ok(())
                    }),
                    UpperAmPm => time.map(|t| {
                        result.push_str(if t.hour12().0 {
                            locale.am_pm[1]
                        } else {
                            locale.am_pm[0]
                        });
                        Ok(())
                    }),
                    Nanosecond => time.map(|t| {
                        let nano = t.nanosecond() % 1_000_000_000;
                        if nano == 0 {
                            Ok(())
                        } else if nano % 1_000_000 == 0 {
                            write!(result, ".{:03}", nano / 1_000_000)
                        } else if nano % 1_000 == 0 {
                            write!(result, ".{:06}", nano / 1_000)
                        } else {
                            write!(result, ".{:09}", nano)
                        }
                    }),
                    Nanosecond3 => time.map(|t| {
                        let nano = t.nanosecond() % 1_000_000_000;
                        write!(result, ".{:03}", nano / 1_000_000)
                    }),
                    Nanosecond6 => time.map(|t| {
                        let nano = t.nanosecond() % 1_000_000_000;
                        write!(result, ".{:06}", nano / 1_000)
                    }),
                    Nanosecond9 => time.map(|t| {
                        let nano = t.nanosecond() % 1_000_000_000;
                        write!(result, ".{:09}", nano)
                    }),
                    Internal(InternalFixed { val: InternalInternal::Nanosecond3NoDot }) => time
                        .map(|t| {
                            let nano = t.nanosecond() % 1_000_000_000;
                            write!(result, "{:03}", nano / 1_000_000)
                        }),
                    Internal(InternalFixed { val: InternalInternal::Nanosecond6NoDot }) => time
                        .map(|t| {
                            let nano = t.nanosecond() % 1_000_000_000;
                            write!(result, "{:06}", nano / 1_000)
                        }),
                    Internal(InternalFixed { val: InternalInternal::Nanosecond9NoDot }) => time
                        .map(|t| {
                            let nano = t.nanosecond() % 1_000_000_000;
                            write!(result, "{:09}", nano)
                        }),
                    TimezoneName => off.map(|&(ref name, _)| {
                        result.push_str(name);
                        Ok(())
                    }),
                    TimezoneOffsetColon => off
                        .map(|&(_, off)| write_local_minus_utc(result, off, false, Colons::Single)),
                    TimezoneOffsetDoubleColon => off
                        .map(|&(_, off)| write_local_minus_utc(result, off, false, Colons::Double)),
                    TimezoneOffsetTripleColon => off
                        .map(|&(_, off)| write_local_minus_utc(result, off, false, Colons::Triple)),
                    TimezoneOffsetColonZ => off
                        .map(|&(_, off)| write_local_minus_utc(result, off, true, Colons::Single)),
                    TimezoneOffset => {
                        off.map(|&(_, off)| write_local_minus_utc(result, off, false, Colons::None))
                    }
                    TimezoneOffsetZ => {
                        off.map(|&(_, off)| write_local_minus_utc(result, off, true, Colons::None))
                    }
                    Internal(InternalFixed { val: InternalInternal::TimezoneOffsetPermissive }) => {
                        panic!("Do not try to write %#z it is undefined")
                    }
                    RFC2822 =>
                    // same as `%a, %d %b %Y %H:%M:%S %z`
                    {
                        if let (Some(d), Some(t), Some(&(_, off))) = (date, time, off) {
                            Some(write_rfc2822_inner(result, d, t, off, locale))
                        } else {
                            None
                        }
                    }
                    RFC3339 =>
                    // same as `%Y-%m-%dT%H:%M:%S%.f%:z`
                    {
                        if let (Some(d), Some(t), Some(&(_, off))) = (date, time, off) {
                            Some(write_rfc3339(result, crate::NaiveDateTime::new(*d, *t), off))
                        } else {
                            None
                        }
                    }
                };

            match ret {
                Some(ret) => ret?,
                None => return Err(fmt::Error), // insufficient arguments for given format
            }
        }

        Item::Error => return Err(fmt::Error),
    }
    Ok(())
}

/// Prints an offset from UTC in the format of `+HHMM` or `+HH:MM`.
/// `Z` instead of `+00[:]00` is allowed when `allow_zulu` is true.
#[cfg(any(feature = "alloc", feature = "std", test))]
fn write_local_minus_utc(
    result: &mut String,
    off: FixedOffset,
    allow_zulu: bool,
    colon_type: Colons,
) -> fmt::Result {
    let off = off.local_minus_utc();
    if allow_zulu && off == 0 {
        result.push('Z');
        return Ok(());
    }
    let (sign, off) = if off < 0 { ('-', -off) } else { ('+', off) };
    result.push(sign);

    write_hundreds(result, (off / 3600) as u8)?;

    match colon_type {
        Colons::None => write_hundreds(result, (off / 60 % 60) as u8),
        Colons::Single => {
            result.push(':');
            write_hundreds(result, (off / 60 % 60) as u8)
        }
        Colons::Double => {
            result.push(':');
            write_hundreds(result, (off / 60 % 60) as u8)?;
            result.push(':');
            write_hundreds(result, (off % 60) as u8)
        }
        Colons::Triple => Ok(()),
    }
}

/// Writes the date, time and offset to the string. same as `%Y-%m-%dT%H:%M:%S%.f%:z`
#[cfg(any(feature = "alloc", feature = "std", test))]
pub(crate) fn write_rfc3339(
    result: &mut String,
    dt: crate::NaiveDateTime,
    off: FixedOffset,
) -> fmt::Result {
    // reuse `Debug` impls which already print ISO 8601 format.
    // this is faster in this way.
    write!(result, "{:?}", dt)?;
    write_local_minus_utc(result, off, false, Colons::Single)
}

#[cfg(any(feature = "alloc", feature = "std", test))]
/// write datetimes like `Tue, 1 Jul 2003 10:52:37 +0200`, same as `%a, %d %b %Y %H:%M:%S %z`
pub(crate) fn write_rfc2822(
    result: &mut String,
    dt: crate::NaiveDateTime,
    off: FixedOffset,
) -> fmt::Result {
    write_rfc2822_inner(result, &dt.date(), &dt.time(), off, Locales::new(None))
}

#[cfg(any(feature = "alloc", feature = "std", test))]
/// write datetimes like `Tue, 1 Jul 2003 10:52:37 +0200`, same as `%a, %d %b %Y %H:%M:%S %z`
fn write_rfc2822_inner(
    result: &mut String,
    d: &NaiveDate,
    t: &NaiveTime,
    off: FixedOffset,
    locale: Locales,
) -> fmt::Result {
    let year = d.year();
    // RFC2822 is only defined on years 0 through 9999
    if !(0..=9999).contains(&year) {
        return Err(fmt::Error);
    }

    result.push_str(locale.short_weekdays[d.weekday().num_days_from_sunday() as usize]);
    result.push_str(", ");
    write_hundreds(result, d.day() as u8)?;
    result.push(' ');
    result.push_str(locale.short_months[d.month0() as usize]);
    result.push(' ');
    write_hundreds(result, (year / 100) as u8)?;
    write_hundreds(result, (year % 100) as u8)?;
    result.push(' ');
    write_hundreds(result, t.hour() as u8)?;
    result.push(':');
    write_hundreds(result, t.minute() as u8)?;
    result.push(':');
    let sec = t.second() + t.nanosecond() / 1_000_000_000;
    write_hundreds(result, sec as u8)?;
    result.push(' ');
    write_local_minus_utc(result, off, false, Colons::None)
}

/// Equivalent to `{:02}` formatting for n < 100.
pub(crate) fn write_hundreds(w: &mut impl Write, n: u8) -> fmt::Result {
    if n >= 100 {
        return Err(fmt::Error);
    }

    let tens = b'0' + n / 10;
    let ones = b'0' + n % 10;
    w.write_char(tens as char)?;
    w.write_char(ones as char)
}

/// Tries to format given arguments with given formatting items.
/// Internally used by `DelayedFormat`.
#[cfg(any(feature = "alloc", feature = "std", test))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
pub fn format<'a, I, B>(
    w: &mut fmt::Formatter,
    date: Option<&NaiveDate>,
    time: Option<&NaiveTime>,
    off: Option<&(String, FixedOffset)>,
    items: I,
) -> fmt::Result
where
    I: Iterator<Item = B> + Clone,
    B: Borrow<Item<'a>>,
{
    let mut result = String::new();
    for item in items {
        format_inner(&mut result, date, time, off, item.borrow(), None)?;
    }
    w.pad(&result)
}

mod parsed;

// due to the size of parsing routines, they are in separate modules.
mod parse;
mod scan;

pub mod strftime;

/// A *temporary* object which can be used as an argument to `format!` or others.
/// This is normally constructed via `format` methods of each date and time type.
#[cfg(any(feature = "alloc", feature = "std", test))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
#[derive(Debug)]
pub struct DelayedFormat<I> {
    /// The date view, if any.
    date: Option<NaiveDate>,
    /// The time view, if any.
    time: Option<NaiveTime>,
    /// The name and local-to-UTC difference for the offset (timezone), if any.
    off: Option<(String, FixedOffset)>,
    /// An iterator returning formatting items.
    items: I,
    /// Locale used for text.
    // TODO: Only used with the locale feature. We should make this property
    // only present when the feature is enabled.
    #[cfg(feature = "unstable-locales")]
    locale: Option<Locale>,
}

#[cfg(any(feature = "alloc", feature = "std", test))]
impl<'a, I: Iterator<Item = B> + Clone, B: Borrow<Item<'a>>> DelayedFormat<I> {
    /// Makes a new `DelayedFormat` value out of local date and time.
    pub fn new(date: Option<NaiveDate>, time: Option<NaiveTime>, items: I) -> DelayedFormat<I> {
        DelayedFormat {
            date,
            time,
            off: None,
            items,
            #[cfg(feature = "unstable-locales")]
            locale: None,
        }
    }

    /// Makes a new `DelayedFormat` value out of local date and time and UTC offset.
    pub fn new_with_offset<Off>(
        date: Option<NaiveDate>,
        time: Option<NaiveTime>,
        offset: &Off,
        items: I,
    ) -> DelayedFormat<I>
    where
        Off: Offset + fmt::Display,
    {
        let name_and_diff = (offset.to_string(), offset.fix());
        DelayedFormat {
            date,
            time,
            off: Some(name_and_diff),
            items,
            #[cfg(feature = "unstable-locales")]
            locale: None,
        }
    }

    /// Makes a new `DelayedFormat` value out of local date and time and locale.
    #[cfg(feature = "unstable-locales")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable-locales")))]
    pub fn new_with_locale(
        date: Option<NaiveDate>,
        time: Option<NaiveTime>,
        items: I,
        locale: Locale,
    ) -> DelayedFormat<I> {
        DelayedFormat { date, time, off: None, items, locale: Some(locale) }
    }

    /// Makes a new `DelayedFormat` value out of local date and time, UTC offset and locale.
    #[cfg(feature = "unstable-locales")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable-locales")))]
    pub fn new_with_offset_and_locale<Off>(
        date: Option<NaiveDate>,
        time: Option<NaiveTime>,
        offset: &Off,
        items: I,
        locale: Locale,
    ) -> DelayedFormat<I>
    where
        Off: Offset + fmt::Display,
    {
        let name_and_diff = (offset.to_string(), offset.fix());
        DelayedFormat { date, time, off: Some(name_and_diff), items, locale: Some(locale) }
    }
}

#[cfg(any(feature = "alloc", feature = "std", test))]
impl<'a, I: Iterator<Item = B> + Clone, B: Borrow<Item<'a>>> fmt::Display for DelayedFormat<I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[cfg(feature = "unstable-locales")]
        {
            if let Some(locale) = self.locale {
                return format_localized(
                    f,
                    self.date.as_ref(),
                    self.time.as_ref(),
                    self.off.as_ref(),
                    self.items.clone(),
                    locale,
                );
            }
        }

        format(f, self.date.as_ref(), self.time.as_ref(), self.off.as_ref(), self.items.clone())
    }
}

// this implementation is here only because we need some private code from `scan`

/// Parsing a `str` into a `Weekday` uses the format [`%W`](./format/strftime/index.html).
///
/// # Example
///
/// ```
/// use chrono::Weekday;
///
/// assert_eq!("Sunday".parse::<Weekday>(), Ok(Weekday::Sun));
/// assert!("any day".parse::<Weekday>().is_err());
/// ```
///
/// The parsing is case-insensitive.
///
/// ```
/// # use chrono::Weekday;
/// assert_eq!("mON".parse::<Weekday>(), Ok(Weekday::Mon));
/// ```
///
/// Only the shortest form (e.g. `sun`) and the longest form (e.g. `sunday`) is accepted.
///
/// ```
/// # use chrono::Weekday;
/// assert!("thurs".parse::<Weekday>().is_err());
/// ```
impl FromStr for Weekday {
    type Err = ParseWeekdayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(("", w)) = scan::short_or_long_weekday(s) {
            Ok(w)
        } else {
            Err(ParseWeekdayError { _dummy: () })
        }
    }
}

/// Formats single formatting item
#[cfg(feature = "unstable-locales")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-locales")))]
pub fn format_item_localized<'a>(
    w: &mut fmt::Formatter,
    date: Option<&NaiveDate>,
    time: Option<&NaiveTime>,
    off: Option<&(String, FixedOffset)>,
    item: &Item<'a>,
    locale: Locale,
) -> fmt::Result {
    let mut result = String::new();
    format_inner(&mut result, date, time, off, item, Some(locale))?;
    w.pad(&result)
}

/// Tries to format given arguments with given formatting items.
/// Internally used by `DelayedFormat`.
#[cfg(feature = "unstable-locales")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-locales")))]
pub fn format_localized<'a, I, B>(
    w: &mut fmt::Formatter,
    date: Option<&NaiveDate>,
    time: Option<&NaiveTime>,
    off: Option<&(String, FixedOffset)>,
    items: I,
    locale: Locale,
) -> fmt::Result
where
    I: Iterator<Item = B> + Clone,
    B: Borrow<Item<'a>>,
{
    let mut result = String::new();
    for item in items {
        format_inner(&mut result, date, time, off, item.borrow(), Some(locale))?;
    }
    w.pad(&result)
}

/// Parsing a `str` into a `Month` uses the format [`%W`](./format/strftime/index.html).
///
/// # Example
///
/// ```
/// use chrono::Month;
///
/// assert_eq!("January".parse::<Month>(), Ok(Month::January));
/// assert!("any day".parse::<Month>().is_err());
/// ```
///
/// The parsing is case-insensitive.
///
/// ```
/// # use chrono::Month;
/// assert_eq!("fEbruARy".parse::<Month>(), Ok(Month::February));
/// ```
///
/// Only the shortest form (e.g. `jan`) and the longest form (e.g. `january`) is accepted.
///
/// ```
/// # use chrono::Month;
/// assert!("septem".parse::<Month>().is_err());
/// assert!("Augustin".parse::<Month>().is_err());
/// ```
impl FromStr for Month {
    type Err = ParseMonthError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(("", w)) = scan::short_or_long_month0(s) {
            match w {
                0 => Ok(Month::January),
                1 => Ok(Month::February),
                2 => Ok(Month::March),
                3 => Ok(Month::April),
                4 => Ok(Month::May),
                5 => Ok(Month::June),
                6 => Ok(Month::July),
                7 => Ok(Month::August),
                8 => Ok(Month::September),
                9 => Ok(Month::October),
                10 => Ok(Month::November),
                11 => Ok(Month::December),
                _ => Err(ParseMonthError { _dummy: () }),
            }
        } else {
            Err(ParseMonthError { _dummy: () })
        }
    }
}
