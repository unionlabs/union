//! Formatting for various types.

pub(crate) mod formattable;
mod iso8601;

use core::num::NonZeroU8;
use std::io;

pub use self::formattable::Formattable;
use crate::convert::*;
use crate::format_description::{modifier, Component};
use crate::{error, Date, OffsetDateTime, Time, UtcOffset};

#[allow(clippy::missing_docs_in_private_items)]
const MONTH_NAMES: [&[u8]; 12] = [
    b"January",
    b"February",
    b"March",
    b"April",
    b"May",
    b"June",
    b"July",
    b"August",
    b"September",
    b"October",
    b"November",
    b"December",
];

#[allow(clippy::missing_docs_in_private_items)]
const WEEKDAY_NAMES: [&[u8]; 7] = [
    b"Monday",
    b"Tuesday",
    b"Wednesday",
    b"Thursday",
    b"Friday",
    b"Saturday",
    b"Sunday",
];

// region: extension trait
/// A trait that indicates the formatted width of the value can be determined.
///
/// Note that this should not be implemented for any signed integers. This forces the caller to
/// write the sign if desired.
pub(crate) trait DigitCount {
    /// The number of digits in the stringified value.
    fn num_digits(self) -> u8;
}

/// A macro to generate implementations of `DigitCount` for unsigned integers.
macro_rules! impl_digit_count {
    ($($t:ty),* $(,)?) => {
        $(impl DigitCount for $t {
            fn num_digits(self) -> u8 {
                match self.checked_ilog10() {
                    Some(n) => (n as u8) + 1,
                    None => 1,
                }
            }
        })*
    };
}

impl_digit_count!(u8, u16, u32);
// endregion extension trait

/// Write all bytes to the output, returning the number of bytes written.
pub(crate) fn write(output: &mut impl io::Write, bytes: &[u8]) -> io::Result<usize> {
    output.write_all(bytes)?;
    Ok(bytes.len())
}

/// If `pred` is true, write all bytes to the output, returning the number of bytes written.
pub(crate) fn write_if(output: &mut impl io::Write, pred: bool, bytes: &[u8]) -> io::Result<usize> {
    if pred { write(output, bytes) } else { Ok(0) }
}

/// If `pred` is true, write `true_bytes` to the output. Otherwise, write `false_bytes`.
pub(crate) fn write_if_else(
    output: &mut impl io::Write,
    pred: bool,
    true_bytes: &[u8],
    false_bytes: &[u8],
) -> io::Result<usize> {
    write(output, if pred { true_bytes } else { false_bytes })
}

/// Write the floating point number to the output, returning the number of bytes written.
///
/// This method accepts the number of digits before and after the decimal. The value will be padded
/// with zeroes to the left if necessary.
pub(crate) fn format_float(
    output: &mut impl io::Write,
    value: f64,
    digits_before_decimal: u8,
    digits_after_decimal: Option<NonZeroU8>,
) -> io::Result<usize> {
    match digits_after_decimal {
        Some(digits_after_decimal) => {
            let digits_after_decimal = digits_after_decimal.get() as usize;
            let width = digits_before_decimal as usize + 1 + digits_after_decimal;
            write!(output, "{value:0>width$.digits_after_decimal$}")?;
            Ok(width)
        }
        None => {
            let value = value.trunc() as u64;
            let width = digits_before_decimal as usize;
            write!(output, "{value:0>width$}")?;
            Ok(width)
        }
    }
}

/// Format a number with the provided padding and width.
///
/// The sign must be written by the caller.
pub(crate) fn format_number<const WIDTH: u8>(
    output: &mut impl io::Write,
    value: impl itoa::Integer + DigitCount + Copy,
    padding: modifier::Padding,
) -> Result<usize, io::Error> {
    match padding {
        modifier::Padding::Space => format_number_pad_space::<WIDTH>(output, value),
        modifier::Padding::Zero => format_number_pad_zero::<WIDTH>(output, value),
        modifier::Padding::None => format_number_pad_none(output, value),
    }
}

/// Format a number with the provided width and spaces as padding.
///
/// The sign must be written by the caller.
pub(crate) fn format_number_pad_space<const WIDTH: u8>(
    output: &mut impl io::Write,
    value: impl itoa::Integer + DigitCount + Copy,
) -> Result<usize, io::Error> {
    let mut bytes = 0;
    for _ in 0..(WIDTH.saturating_sub(value.num_digits())) {
        bytes += write(output, b" ")?;
    }
    bytes += write(output, itoa::Buffer::new().format(value).as_bytes())?;
    Ok(bytes)
}

/// Format a number with the provided width and zeros as padding.
///
/// The sign must be written by the caller.
pub(crate) fn format_number_pad_zero<const WIDTH: u8>(
    output: &mut impl io::Write,
    value: impl itoa::Integer + DigitCount + Copy,
) -> Result<usize, io::Error> {
    let mut bytes = 0;
    for _ in 0..(WIDTH.saturating_sub(value.num_digits())) {
        bytes += write(output, b"0")?;
    }
    bytes += write(output, itoa::Buffer::new().format(value).as_bytes())?;
    Ok(bytes)
}

/// Format a number with no padding.
///
/// If the sign is mandatory, the sign must be written by the caller.
pub(crate) fn format_number_pad_none(
    output: &mut impl io::Write,
    value: impl itoa::Integer + Copy,
) -> Result<usize, io::Error> {
    write(output, itoa::Buffer::new().format(value).as_bytes())
}

/// Format the provided component into the designated output. An `Err` will be returned if the
/// component requires information that it does not provide or if the value cannot be output to the
/// stream.
pub(crate) fn format_component(
    output: &mut impl io::Write,
    component: Component,
    date: Option<Date>,
    time: Option<Time>,
    offset: Option<UtcOffset>,
) -> Result<usize, error::Format> {
    use Component::*;
    Ok(match (component, date, time, offset) {
        (Day(modifier), Some(date), ..) => fmt_day(output, date, modifier)?,
        (Month(modifier), Some(date), ..) => fmt_month(output, date, modifier)?,
        (Ordinal(modifier), Some(date), ..) => fmt_ordinal(output, date, modifier)?,
        (Weekday(modifier), Some(date), ..) => fmt_weekday(output, date, modifier)?,
        (WeekNumber(modifier), Some(date), ..) => fmt_week_number(output, date, modifier)?,
        (Year(modifier), Some(date), ..) => fmt_year(output, date, modifier)?,
        (Hour(modifier), _, Some(time), _) => fmt_hour(output, time, modifier)?,
        (Minute(modifier), _, Some(time), _) => fmt_minute(output, time, modifier)?,
        (Period(modifier), _, Some(time), _) => fmt_period(output, time, modifier)?,
        (Second(modifier), _, Some(time), _) => fmt_second(output, time, modifier)?,
        (Subsecond(modifier), _, Some(time), _) => fmt_subsecond(output, time, modifier)?,
        (OffsetHour(modifier), .., Some(offset)) => fmt_offset_hour(output, offset, modifier)?,
        (OffsetMinute(modifier), .., Some(offset)) => fmt_offset_minute(output, offset, modifier)?,
        (OffsetSecond(modifier), .., Some(offset)) => fmt_offset_second(output, offset, modifier)?,
        (Ignore(_), ..) => 0,
        (UnixTimestamp(modifier), Some(date), Some(time), Some(offset)) => {
            fmt_unix_timestamp(output, date, time, offset, modifier)?
        }
        (End(modifier::End {}), ..) => 0,
        _ => return Err(error::Format::InsufficientTypeInformation),
    })
}

// region: date formatters
/// Format the day into the designated output.
fn fmt_day(
    output: &mut impl io::Write,
    date: Date,
    modifier::Day { padding }: modifier::Day,
) -> Result<usize, io::Error> {
    format_number::<2>(output, date.day(), padding)
}

/// Format the month into the designated output.
fn fmt_month(
    output: &mut impl io::Write,
    date: Date,
    modifier::Month {
        padding,
        repr,
        case_sensitive: _, // no effect on formatting
    }: modifier::Month,
) -> Result<usize, io::Error> {
    match repr {
        modifier::MonthRepr::Numerical => format_number::<2>(output, date.month() as u8, padding),
        modifier::MonthRepr::Long => write(output, MONTH_NAMES[date.month() as usize - 1]),
        modifier::MonthRepr::Short => write(output, &MONTH_NAMES[date.month() as usize - 1][..3]),
    }
}

/// Format the ordinal into the designated output.
fn fmt_ordinal(
    output: &mut impl io::Write,
    date: Date,
    modifier::Ordinal { padding }: modifier::Ordinal,
) -> Result<usize, io::Error> {
    format_number::<3>(output, date.ordinal(), padding)
}

/// Format the weekday into the designated output.
fn fmt_weekday(
    output: &mut impl io::Write,
    date: Date,
    modifier::Weekday {
        repr,
        one_indexed,
        case_sensitive: _, // no effect on formatting
    }: modifier::Weekday,
) -> Result<usize, io::Error> {
    match repr {
        modifier::WeekdayRepr::Short => write(
            output,
            &WEEKDAY_NAMES[date.weekday().number_days_from_monday() as usize][..3],
        ),
        modifier::WeekdayRepr::Long => write(
            output,
            WEEKDAY_NAMES[date.weekday().number_days_from_monday() as usize],
        ),
        modifier::WeekdayRepr::Sunday => format_number::<1>(
            output,
            date.weekday().number_days_from_sunday() + one_indexed as u8,
            modifier::Padding::None,
        ),
        modifier::WeekdayRepr::Monday => format_number::<1>(
            output,
            date.weekday().number_days_from_monday() + one_indexed as u8,
            modifier::Padding::None,
        ),
    }
}

/// Format the week number into the designated output.
fn fmt_week_number(
    output: &mut impl io::Write,
    date: Date,
    modifier::WeekNumber { padding, repr }: modifier::WeekNumber,
) -> Result<usize, io::Error> {
    format_number::<2>(
        output,
        match repr {
            modifier::WeekNumberRepr::Iso => date.iso_week(),
            modifier::WeekNumberRepr::Sunday => date.sunday_based_week(),
            modifier::WeekNumberRepr::Monday => date.monday_based_week(),
        },
        padding,
    )
}

/// Format the year into the designated output.
fn fmt_year(
    output: &mut impl io::Write,
    date: Date,
    modifier::Year {
        padding,
        repr,
        iso_week_based,
        sign_is_mandatory,
    }: modifier::Year,
) -> Result<usize, io::Error> {
    let full_year = if iso_week_based {
        date.iso_year_week().0
    } else {
        date.year()
    };
    let value = match repr {
        modifier::YearRepr::Full => full_year,
        modifier::YearRepr::LastTwo => (full_year % 100).abs(),
    };
    let format_number = match repr {
        #[cfg(feature = "large-dates")]
        modifier::YearRepr::Full if value.abs() >= 100_000 => format_number::<6>,
        #[cfg(feature = "large-dates")]
        modifier::YearRepr::Full if value.abs() >= 10_000 => format_number::<5>,
        modifier::YearRepr::Full => format_number::<4>,
        modifier::YearRepr::LastTwo => format_number::<2>,
    };
    let mut bytes = 0;
    if repr != modifier::YearRepr::LastTwo {
        if full_year < 0 {
            bytes += write(output, b"-")?;
        } else if sign_is_mandatory || cfg!(feature = "large-dates") && full_year >= 10_000 {
            bytes += write(output, b"+")?;
        }
    }
    bytes += format_number(output, value.unsigned_abs(), padding)?;
    Ok(bytes)
}
// endregion date formatters

// region: time formatters
/// Format the hour into the designated output.
fn fmt_hour(
    output: &mut impl io::Write,
    time: Time,
    modifier::Hour {
        padding,
        is_12_hour_clock,
    }: modifier::Hour,
) -> Result<usize, io::Error> {
    let value = match (time.hour(), is_12_hour_clock) {
        (hour, false) => hour,
        (0 | 12, true) => 12,
        (hour, true) if hour < 12 => hour,
        (hour, true) => hour - 12,
    };
    format_number::<2>(output, value, padding)
}

/// Format the minute into the designated output.
fn fmt_minute(
    output: &mut impl io::Write,
    time: Time,
    modifier::Minute { padding }: modifier::Minute,
) -> Result<usize, io::Error> {
    format_number::<2>(output, time.minute(), padding)
}

/// Format the period into the designated output.
fn fmt_period(
    output: &mut impl io::Write,
    time: Time,
    modifier::Period {
        is_uppercase,
        case_sensitive: _, // no effect on formatting
    }: modifier::Period,
) -> Result<usize, io::Error> {
    match (time.hour() >= 12, is_uppercase) {
        (false, false) => write(output, b"am"),
        (false, true) => write(output, b"AM"),
        (true, false) => write(output, b"pm"),
        (true, true) => write(output, b"PM"),
    }
}

/// Format the second into the designated output.
fn fmt_second(
    output: &mut impl io::Write,
    time: Time,
    modifier::Second { padding }: modifier::Second,
) -> Result<usize, io::Error> {
    format_number::<2>(output, time.second(), padding)
}

/// Format the subsecond into the designated output.
fn fmt_subsecond<W: io::Write>(
    output: &mut W,
    time: Time,
    modifier::Subsecond { digits }: modifier::Subsecond,
) -> Result<usize, io::Error> {
    use modifier::SubsecondDigits::*;
    let nanos = time.nanosecond();

    if digits == Nine || (digits == OneOrMore && nanos % 10 != 0) {
        format_number_pad_zero::<9>(output, nanos)
    } else if digits == Eight || (digits == OneOrMore && (nanos / 10) % 10 != 0) {
        format_number_pad_zero::<8>(output, nanos / 10)
    } else if digits == Seven || (digits == OneOrMore && (nanos / 100) % 10 != 0) {
        format_number_pad_zero::<7>(output, nanos / 100)
    } else if digits == Six || (digits == OneOrMore && (nanos / 1_000) % 10 != 0) {
        format_number_pad_zero::<6>(output, nanos / 1_000)
    } else if digits == Five || (digits == OneOrMore && (nanos / 10_000) % 10 != 0) {
        format_number_pad_zero::<5>(output, nanos / 10_000)
    } else if digits == Four || (digits == OneOrMore && (nanos / 100_000) % 10 != 0) {
        format_number_pad_zero::<4>(output, nanos / 100_000)
    } else if digits == Three || (digits == OneOrMore && (nanos / 1_000_000) % 10 != 0) {
        format_number_pad_zero::<3>(output, nanos / 1_000_000)
    } else if digits == Two || (digits == OneOrMore && (nanos / 10_000_000) % 10 != 0) {
        format_number_pad_zero::<2>(output, nanos / 10_000_000)
    } else {
        format_number_pad_zero::<1>(output, nanos / 100_000_000)
    }
}
// endregion time formatters

// region: offset formatters
/// Format the offset hour into the designated output.
fn fmt_offset_hour(
    output: &mut impl io::Write,
    offset: UtcOffset,
    modifier::OffsetHour {
        padding,
        sign_is_mandatory,
    }: modifier::OffsetHour,
) -> Result<usize, io::Error> {
    let mut bytes = 0;
    if offset.is_negative() {
        bytes += write(output, b"-")?;
    } else if sign_is_mandatory {
        bytes += write(output, b"+")?;
    }
    bytes += format_number::<2>(output, offset.whole_hours().unsigned_abs(), padding)?;
    Ok(bytes)
}

/// Format the offset minute into the designated output.
fn fmt_offset_minute(
    output: &mut impl io::Write,
    offset: UtcOffset,
    modifier::OffsetMinute { padding }: modifier::OffsetMinute,
) -> Result<usize, io::Error> {
    format_number::<2>(output, offset.minutes_past_hour().unsigned_abs(), padding)
}

/// Format the offset second into the designated output.
fn fmt_offset_second(
    output: &mut impl io::Write,
    offset: UtcOffset,
    modifier::OffsetSecond { padding }: modifier::OffsetSecond,
) -> Result<usize, io::Error> {
    format_number::<2>(output, offset.seconds_past_minute().unsigned_abs(), padding)
}
// endregion offset formatters

/// Format the Unix timestamp into the designated output.
fn fmt_unix_timestamp(
    output: &mut impl io::Write,
    date: Date,
    time: Time,
    offset: UtcOffset,
    modifier::UnixTimestamp {
        precision,
        sign_is_mandatory,
    }: modifier::UnixTimestamp,
) -> Result<usize, io::Error> {
    let date_time = date
        .with_time(time)
        .assume_offset(offset)
        .to_offset(UtcOffset::UTC);

    if date_time < OffsetDateTime::UNIX_EPOCH {
        write(output, b"-")?;
    } else if sign_is_mandatory {
        write(output, b"+")?;
    }

    match precision {
        modifier::UnixTimestampPrecision::Second => {
            format_number_pad_none(output, date_time.unix_timestamp().unsigned_abs())
        }
        modifier::UnixTimestampPrecision::Millisecond => format_number_pad_none(
            output,
            (date_time.unix_timestamp_nanos() / Nanosecond.per(Millisecond) as i128).unsigned_abs(),
        ),
        modifier::UnixTimestampPrecision::Microsecond => format_number_pad_none(
            output,
            (date_time.unix_timestamp_nanos() / Nanosecond.per(Microsecond) as i128).unsigned_abs(),
        ),
        modifier::UnixTimestampPrecision::Nanosecond => {
            format_number_pad_none(output, date_time.unix_timestamp_nanos().unsigned_abs())
        }
    }
}
