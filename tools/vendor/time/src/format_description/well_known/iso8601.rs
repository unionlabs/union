//! The format described in ISO 8601.

mod adt_hack;

use core::num::NonZeroU8;

pub use self::adt_hack::{DoNotRelyOnWhatThisIs, EncodedConfig};

/// A configuration for [`Iso8601`] that only parses values.
const PARSING_ONLY: EncodedConfig = Config {
    formatted_components: FormattedComponents::None,
    use_separators: false,
    year_is_six_digits: false,
    date_kind: DateKind::Calendar,
    time_precision: TimePrecision::Hour {
        decimal_digits: None,
    },
    offset_precision: OffsetPrecision::Hour,
}
.encode();

/// The default configuration for [`Iso8601`].
const DEFAULT_CONFIG: EncodedConfig = Config::DEFAULT.encode();

/// The format described in [ISO 8601](https://www.iso.org/iso-8601-date-and-time-format.html).
///
/// This implementation is of ISO 8601-1:2019. It may not be compatible with other versions.
///
/// The const parameter `CONFIG` **must** be a value that was returned by [`Config::encode`].
/// Passing any other value is **unspecified behavior**.
///
/// Example: 1997-11-21T09:55:06.000000000-06:00
///
/// # Examples
#[cfg_attr(feature = "formatting", doc = "```rust")]
#[cfg_attr(not(feature = "formatting"), doc = "```rust,ignore")]
/// # use time::format_description::well_known::Iso8601;
/// # use time_macros::datetime;
/// assert_eq!(
///     datetime!(1997-11-12 9:55:06 -6:00).format(&Iso8601::DEFAULT)?,
///     "1997-11-12T09:55:06.000000000-06:00"
/// );
/// # Ok::<_, time::Error>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Iso8601<const CONFIG: EncodedConfig = DEFAULT_CONFIG>;

impl<const CONFIG: EncodedConfig> core::fmt::Debug for Iso8601<CONFIG> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Iso8601")
            .field("config", &Config::decode(CONFIG))
            .finish()
    }
}

impl Iso8601<DEFAULT_CONFIG> {
    /// An [`Iso8601`] with the default configuration.
    ///
    /// The following is the default behavior:
    ///
    /// - The configuration can be used for both formatting and parsing.
    /// - The date, time, and UTC offset are all formatted.
    /// - Separators (such as `-` and `:`) are included.
    /// - The year contains four digits, such that the year must be between 0 and 9999.
    /// - The date uses the calendar format.
    /// - The time has precision to the second and nine decimal digits.
    /// - The UTC offset has precision to the minute.
    ///
    /// If you need different behavior, use [`Config::DEFAULT`] and [`Config`]'s methods to create
    /// a custom configuration.
    pub const DEFAULT: Self = Self;
}

impl Iso8601<PARSING_ONLY> {
    /// An [`Iso8601`] that can only be used for parsing. Using this to format a value is
    /// unspecified behavior.
    pub const PARSING: Self = Self;
}

/// Which components to format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormattedComponents {
    /// The configuration can only be used for parsing. Using this to format a value is
    /// unspecified behavior.
    None,
    /// Format only the date.
    Date,
    /// Format only the time.
    Time,
    /// Format only the UTC offset.
    Offset,
    /// Format the date and time.
    DateTime,
    /// Format the date, time, and UTC offset.
    DateTimeOffset,
    /// Format the time and UTC offset.
    TimeOffset,
}

/// Which format to use for the date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateKind {
    /// Use the year-month-day format.
    Calendar,
    /// Use the year-week-weekday format.
    Week,
    /// Use the week-ordinal format.
    Ordinal,
}

/// The precision and number of decimal digits present for the time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimePrecision {
    /// Format the hour only. Minutes, seconds, and nanoseconds will be represented with the
    /// specified number of decimal digits, if any.
    Hour {
        #[allow(clippy::missing_docs_in_private_items)]
        decimal_digits: Option<NonZeroU8>,
    },
    /// Format the hour and minute. Seconds and nanoseconds will be represented with the specified
    /// number of decimal digits, if any.
    Minute {
        #[allow(clippy::missing_docs_in_private_items)]
        decimal_digits: Option<NonZeroU8>,
    },
    /// Format the hour, minute, and second. Nanoseconds will be represented with the specified
    /// number of decimal digits, if any.
    Second {
        #[allow(clippy::missing_docs_in_private_items)]
        decimal_digits: Option<NonZeroU8>,
    },
}

/// The precision for the UTC offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffsetPrecision {
    /// Format only the offset hour. Requires the offset minute to be zero.
    Hour,
    /// Format both the offset hour and minute.
    Minute,
}

/// Configuration for [`Iso8601`].
// This is only used as a const generic, so there's no need to have a number of implementations on
// it.
#[allow(missing_copy_implementations)]
#[doc(alias = "EncodedConfig")] // People will likely search for `EncodedConfig`, so show them this.
#[derive(Debug)]
pub struct Config {
    /// Which components, if any, will be formatted.
    pub(crate) formatted_components: FormattedComponents,
    /// Whether the format contains separators (such as `-` or `:`).
    pub(crate) use_separators: bool,
    /// Whether the year is six digits.
    pub(crate) year_is_six_digits: bool,
    /// The format used for the date.
    pub(crate) date_kind: DateKind,
    /// The precision and number of decimal digits present for the time.
    pub(crate) time_precision: TimePrecision,
    /// The precision for the UTC offset.
    pub(crate) offset_precision: OffsetPrecision,
}

impl Config {
    /// A configuration for the [`Iso8601`] format.
    ///
    /// The following is the default behavior:
    ///
    /// - The configuration can be used for both formatting and parsing.
    /// - The date, time, and UTC offset are all formatted.
    /// - Separators (such as `-` and `:`) are included.
    /// - The year contains four digits, such that the year must be between 0 and 9999.
    /// - The date uses the calendar format.
    /// - The time has precision to the second and nine decimal digits.
    /// - The UTC offset has precision to the minute.
    ///
    /// If you need different behavior, use the setter methods on this struct.
    pub const DEFAULT: Self = Self {
        formatted_components: FormattedComponents::DateTimeOffset,
        use_separators: true,
        year_is_six_digits: false,
        date_kind: DateKind::Calendar,
        time_precision: TimePrecision::Second {
            decimal_digits: NonZeroU8::new(9),
        },
        offset_precision: OffsetPrecision::Minute,
    };

    /// Set whether the format the date, time, and/or UTC offset.
    pub const fn set_formatted_components(self, formatted_components: FormattedComponents) -> Self {
        Self {
            formatted_components,
            ..self
        }
    }

    /// Set whether the format contains separators (such as `-` or `:`).
    pub const fn set_use_separators(self, use_separators: bool) -> Self {
        Self {
            use_separators,
            ..self
        }
    }

    /// Set whether the year is six digits.
    pub const fn set_year_is_six_digits(self, year_is_six_digits: bool) -> Self {
        Self {
            year_is_six_digits,
            ..self
        }
    }

    /// Set the format used for the date.
    pub const fn set_date_kind(self, date_kind: DateKind) -> Self {
        Self { date_kind, ..self }
    }

    /// Set the precision and number of decimal digits present for the time.
    pub const fn set_time_precision(self, time_precision: TimePrecision) -> Self {
        Self {
            time_precision,
            ..self
        }
    }

    /// Set the precision for the UTC offset.
    pub const fn set_offset_precision(self, offset_precision: OffsetPrecision) -> Self {
        Self {
            offset_precision,
            ..self
        }
    }
}
