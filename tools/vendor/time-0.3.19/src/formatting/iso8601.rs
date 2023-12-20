//! Helpers for implementing formatting for ISO 8601.

use std::io;

use crate::format_description::well_known::iso8601::{
    DateKind, EncodedConfig, OffsetPrecision, TimePrecision,
};
use crate::format_description::well_known::Iso8601;
use crate::formatting::{format_float, format_number_pad_zero, write, write_if, write_if_else};
use crate::{error, Date, Time, UtcOffset};

/// Format the date portion of ISO 8601.
pub(super) fn format_date<W: io::Write, const CONFIG: EncodedConfig>(
    output: &mut W,
    date: Date,
) -> Result<usize, error::Format> {
    let mut bytes = 0;

    match Iso8601::<CONFIG>::DATE_KIND {
        DateKind::Calendar => {
            let (year, month, day) = date.to_calendar_date();
            if Iso8601::<CONFIG>::YEAR_IS_SIX_DIGITS {
                bytes += write_if_else(output, year < 0, b"-", b"+")?;
                bytes += format_number_pad_zero::<6, _, _>(output, year.unsigned_abs())?;
            } else if !(0..=9999).contains(&year) {
                return Err(error::Format::InvalidComponent("year"));
            } else {
                bytes += format_number_pad_zero::<4, _, _>(output, year as u32)?;
            }
            bytes += write_if(output, Iso8601::<CONFIG>::USE_SEPARATORS, b"-")?;
            bytes += format_number_pad_zero::<2, _, _>(output, month as u8)?;
            bytes += write_if(output, Iso8601::<CONFIG>::USE_SEPARATORS, b"-")?;
            bytes += format_number_pad_zero::<2, _, _>(output, day)?;
        }
        DateKind::Week => {
            let (year, week, day) = date.to_iso_week_date();
            if Iso8601::<CONFIG>::YEAR_IS_SIX_DIGITS {
                bytes += write_if_else(output, year < 0, b"-", b"+")?;
                bytes += format_number_pad_zero::<6, _, _>(output, year.unsigned_abs())?;
            } else if !(0..=9999).contains(&year) {
                return Err(error::Format::InvalidComponent("year"));
            } else {
                bytes += format_number_pad_zero::<4, _, _>(output, year as u32)?;
            }
            bytes += write_if_else(output, Iso8601::<CONFIG>::USE_SEPARATORS, b"-W", b"W")?;
            bytes += format_number_pad_zero::<2, _, _>(output, week)?;
            bytes += write_if(output, Iso8601::<CONFIG>::USE_SEPARATORS, b"-")?;
            bytes += format_number_pad_zero::<1, _, _>(output, day.number_from_monday())?;
        }
        DateKind::Ordinal => {
            let (year, day) = date.to_ordinal_date();
            if Iso8601::<CONFIG>::YEAR_IS_SIX_DIGITS {
                bytes += write_if_else(output, year < 0, b"-", b"+")?;
                bytes += format_number_pad_zero::<6, _, _>(output, year.unsigned_abs())?;
            } else if !(0..=9999).contains(&year) {
                return Err(error::Format::InvalidComponent("year"));
            } else {
                bytes += format_number_pad_zero::<4, _, _>(output, year as u32)?;
            }
            bytes += write_if(output, Iso8601::<CONFIG>::USE_SEPARATORS, b"-")?;
            bytes += format_number_pad_zero::<3, _, _>(output, day)?;
        }
    }

    Ok(bytes)
}

/// Format the time portion of ISO 8601.
pub(super) fn format_time<W: io::Write, const CONFIG: EncodedConfig>(
    output: &mut W,
    time: Time,
) -> Result<usize, error::Format> {
    let mut bytes = 0;

    // The "T" can only be omitted in extended format where there is no date being formatted.
    bytes += write_if(
        output,
        Iso8601::<CONFIG>::USE_SEPARATORS || Iso8601::<CONFIG>::FORMAT_DATE,
        b"T",
    )?;

    let (hours, minutes, seconds, nanoseconds) = time.as_hms_nano();

    match Iso8601::<CONFIG>::TIME_PRECISION {
        TimePrecision::Hour { decimal_digits } => {
            let hours = (hours as f64)
                + (minutes as f64) / 60.
                + (seconds as f64) / 3_600.
                + (nanoseconds as f64) / 3_600. / 1_000_000_000.;
            format_float(output, hours, 2, decimal_digits)?;
        }
        TimePrecision::Minute { decimal_digits } => {
            bytes += format_number_pad_zero::<2, _, _>(output, hours)?;
            bytes += write_if(output, Iso8601::<CONFIG>::USE_SEPARATORS, b":")?;
            let minutes = (minutes as f64)
                + (seconds as f64) / 60.
                + (nanoseconds as f64) / 60. / 1_000_000_000.;
            bytes += format_float(output, minutes, 2, decimal_digits)?;
        }
        TimePrecision::Second { decimal_digits } => {
            bytes += format_number_pad_zero::<2, _, _>(output, hours)?;
            bytes += write_if(output, Iso8601::<CONFIG>::USE_SEPARATORS, b":")?;
            bytes += format_number_pad_zero::<2, _, _>(output, minutes)?;
            bytes += write_if(output, Iso8601::<CONFIG>::USE_SEPARATORS, b":")?;
            let seconds = (seconds as f64) + (nanoseconds as f64) / 1_000_000_000.;
            bytes += format_float(output, seconds, 2, decimal_digits)?;
        }
    }

    Ok(bytes)
}

/// Format the UTC offset portion of ISO 8601.
pub(super) fn format_offset<W: io::Write, const CONFIG: EncodedConfig>(
    output: &mut W,
    offset: UtcOffset,
) -> Result<usize, error::Format> {
    if Iso8601::<CONFIG>::FORMAT_TIME && offset.is_utc() {
        return Ok(write(output, b"Z")?);
    }

    let mut bytes = 0;

    let (hours, minutes, seconds) = offset.as_hms();
    if seconds != 0 {
        return Err(error::Format::InvalidComponent("offset_second"));
    }
    bytes += write_if_else(output, offset.is_negative(), b"-", b"+")?;
    bytes += format_number_pad_zero::<2, _, _>(output, hours.unsigned_abs())?;

    if Iso8601::<CONFIG>::OFFSET_PRECISION == OffsetPrecision::Hour && minutes != 0 {
        return Err(error::Format::InvalidComponent("offset_minute"));
    } else if Iso8601::<CONFIG>::OFFSET_PRECISION == OffsetPrecision::Minute {
        bytes += write_if(output, Iso8601::<CONFIG>::USE_SEPARATORS, b":")?;
        bytes += format_number_pad_zero::<2, _, _>(output, minutes.unsigned_abs())?;
    }

    Ok(bytes)
}
