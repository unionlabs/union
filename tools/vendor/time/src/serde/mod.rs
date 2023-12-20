//! Differential formats for serde.
// This also includes the serde implementations for all types. This doesn't need to be externally
// documented, though.

// Types with guaranteed stable serde representations. Strings are avoided to allow for optimal
// representations in various binary forms.

/// Consume the next item in a sequence.
macro_rules! item {
    ($seq:expr, $name:literal) => {
        $seq.next_element()?
            .ok_or_else(|| <A::Error as serde::de::Error>::custom(concat!("expected ", $name)))
    };
}

#[cfg(any(feature = "formatting", feature = "parsing"))]
pub mod iso8601;
#[cfg(any(feature = "formatting", feature = "parsing"))]
pub mod rfc2822;
#[cfg(any(feature = "formatting", feature = "parsing"))]
pub mod rfc3339;
pub mod timestamp;
mod visitor;

use core::marker::PhantomData;

#[cfg(feature = "serde-human-readable")]
use serde::ser::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
/// Generate a custom serializer and deserializer from a format string or an existing format.
///
/// The syntax accepted by this macro is the same as [`format_description::parse()`], which can
/// be found in [the book](https://time-rs.github.io/book/api/format-description.html).
///
/// # Usage
///
/// Invoked as `serde::format_description!(mod_name, Date, FORMAT)` where `FORMAT` is either a
/// `"<format string>"` or something that implements
#[cfg_attr(
    all(feature = "formatting", feature = "parsing"),
    doc = "[`Formattable`](crate::formatting::Formattable) and \
           [`Parsable`](crate::parsing::Parsable)."
)]
#[cfg_attr(
    all(feature = "formatting", not(feature = "parsing")),
    doc = "[`Formattable`](crate::formatting::Formattable)."
)]
#[cfg_attr(
    all(not(feature = "formatting"), feature = "parsing"),
    doc = "[`Parsable`](crate::parsing::Parsable)."
)]
/// This puts a module named `mod_name` in the current scope that can be used to format `Date`
/// structs. A submodule (`mod_name::option`) is also generated for `Option<Date>`. Both
/// modules are only visible in the current scope.
///
/// The returned `Option` will contain a deserialized value if present and `None` if the field
/// is present but the value is `null` (or the equivalent in other formats). To return `None`
/// when the field is not present, you should use `#[serde(default)]` on the field.
///
/// # Examples
///
/// Using a format string:
///
/// ```rust,no_run
/// # use time::OffsetDateTime;
#[cfg_attr(
    all(feature = "formatting", feature = "parsing"),
    doc = "use ::serde::{Serialize, Deserialize};"
)]
#[cfg_attr(
    all(feature = "formatting", not(feature = "parsing")),
    doc = "use ::serde::Serialize;"
)]
#[cfg_attr(
    all(not(feature = "formatting"), feature = "parsing"),
    doc = "use ::serde::Deserialize;"
)]
/// use time::serde;
///
/// // Makes a module `mod my_format { ... }`.
/// serde::format_description!(my_format, OffsetDateTime, "hour=[hour], minute=[minute]");
///
/// # #[allow(dead_code)]
#[cfg_attr(
    all(feature = "formatting", feature = "parsing"),
    doc = "#[derive(Serialize, Deserialize)]"
)]
#[cfg_attr(
    all(feature = "formatting", not(feature = "parsing")),
    doc = "#[derive(Serialize)]"
)]
#[cfg_attr(
    all(not(feature = "formatting"), feature = "parsing"),
    doc = "#[derive(Deserialize)]"
)]
/// struct SerializesWithCustom {
///     #[serde(with = "my_format")]
///     dt: OffsetDateTime,
///     #[serde(with = "my_format::option")]
///     maybe_dt: Option<OffsetDateTime>,
/// }
/// ```
/// 
/// Define the format separately to be used in multiple places:
/// ```rust,no_run
/// # use time::OffsetDateTime;
#[cfg_attr(
    all(feature = "formatting", feature = "parsing"),
    doc = "use ::serde::{Serialize, Deserialize};"
)]
#[cfg_attr(
    all(feature = "formatting", not(feature = "parsing")),
    doc = "use ::serde::Serialize;"
)]
#[cfg_attr(
    all(not(feature = "formatting"), feature = "parsing"),
    doc = "use ::serde::Deserialize;"
)]
/// use time::serde;
/// use time::format_description::FormatItem;
///
/// const DATE_TIME_FORMAT: &[FormatItem<'_>] = time::macros::format_description!(
///     "hour=[hour], minute=[minute]"
/// );
///
/// // Makes a module `mod my_format { ... }`.
/// serde::format_description!(my_format, OffsetDateTime, DATE_TIME_FORMAT);
///
/// # #[allow(dead_code)]
#[cfg_attr(
    all(feature = "formatting", feature = "parsing"),
    doc = "#[derive(Serialize, Deserialize)]"
)]
#[cfg_attr(
    all(feature = "formatting", not(feature = "parsing")),
    doc = "#[derive(Serialize)]"
)]
#[cfg_attr(
    all(not(feature = "formatting"), feature = "parsing"),
    doc = "#[derive(Deserialize)]"
)]
/// struct SerializesWithCustom {
///     #[serde(with = "my_format")]
///     dt: OffsetDateTime,
///     #[serde(with = "my_format::option")]
///     maybe_dt: Option<OffsetDateTime>,
/// }
///
/// fn main() {
///     # #[allow(unused_variables)]
///     let str_ts = OffsetDateTime::now_utc().format(DATE_TIME_FORMAT).unwrap();
/// }
/// ```
/// 
/// Customize the configuration of ISO 8601 formatting/parsing:
/// ```rust,no_run
/// # use time::OffsetDateTime;
#[cfg_attr(
    all(feature = "formatting", feature = "parsing"),
    doc = "use ::serde::{Serialize, Deserialize};"
)]
#[cfg_attr(
    all(feature = "formatting", not(feature = "parsing")),
    doc = "use ::serde::Serialize;"
)]
#[cfg_attr(
    all(not(feature = "formatting"), feature = "parsing"),
    doc = "use ::serde::Deserialize;"
)]
/// use time::serde;
/// use time::format_description::well_known::{iso8601, Iso8601};
///
/// const CONFIG: iso8601::EncodedConfig = iso8601::Config::DEFAULT
///     .set_year_is_six_digits(false)
///     .encode();
/// const FORMAT: Iso8601<CONFIG> = Iso8601::<CONFIG>;
///
/// // Makes a module `mod my_format { ... }`.
/// serde::format_description!(my_format, OffsetDateTime, FORMAT);
///
/// # #[allow(dead_code)]
#[cfg_attr(
    all(feature = "formatting", feature = "parsing"),
    doc = "#[derive(Serialize, Deserialize)]"
)]
#[cfg_attr(
    all(feature = "formatting", not(feature = "parsing")),
    doc = "#[derive(Serialize)]"
)]
#[cfg_attr(
    all(not(feature = "formatting"), feature = "parsing"),
    doc = "#[derive(Deserialize)]"
)]
/// struct SerializesWithCustom {
///     #[serde(with = "my_format")]
///     dt: OffsetDateTime,
///     #[serde(with = "my_format::option")]
///     maybe_dt: Option<OffsetDateTime>,
/// }
/// # fn main() {}
/// ```
/// 
/// [`format_description::parse()`]: crate::format_description::parse()
#[cfg(all(feature = "macros", any(feature = "formatting", feature = "parsing"),))]
pub use time_macros::serde_format_description as format_description;

use self::visitor::Visitor;
#[cfg(feature = "parsing")]
use crate::format_description::{modifier, Component, FormatItem};
use crate::{Date, Duration, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset, Weekday};

// region: Date
/// The format used when serializing and deserializing a human-readable `Date`.
#[cfg(feature = "parsing")]
const DATE_FORMAT: &[FormatItem<'_>] = &[
    FormatItem::Component(Component::Year(modifier::Year::default())),
    FormatItem::Literal(b"-"),
    FormatItem::Component(Component::Month(modifier::Month::default())),
    FormatItem::Literal(b"-"),
    FormatItem::Component(Component::Day(modifier::Day::default())),
];

impl Serialize for Date {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[cfg(feature = "serde-human-readable")]
        if serializer.is_human_readable() {
            let Ok(s) = self.format(&DATE_FORMAT) else {
                return Err(S::Error::custom("failed formatting `Date`"));
            };
            return serializer.serialize_str(&s);
        }

        (self.year(), self.ordinal()).serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for Date {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        if cfg!(feature = "serde-human-readable") && deserializer.is_human_readable() {
            deserializer.deserialize_any(Visitor::<Self>(PhantomData))
        } else {
            deserializer.deserialize_tuple(2, Visitor::<Self>(PhantomData))
        }
    }
}
// endregion date

// region: Duration
impl Serialize for Duration {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[cfg(feature = "serde-human-readable")]
        if serializer.is_human_readable() {
            return serializer.collect_str(&format_args!(
                "{}.{:>09}",
                self.whole_seconds(),
                self.subsec_nanoseconds().abs()
            ));
        }

        (self.whole_seconds(), self.subsec_nanoseconds()).serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for Duration {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        if cfg!(feature = "serde-human-readable") && deserializer.is_human_readable() {
            deserializer.deserialize_any(Visitor::<Self>(PhantomData))
        } else {
            deserializer.deserialize_tuple(2, Visitor::<Self>(PhantomData))
        }
    }
}
// endregion Duration

// region: OffsetDateTime
/// The format used when serializing and deserializing a human-readable `OffsetDateTime`.
#[cfg(feature = "parsing")]
const OFFSET_DATE_TIME_FORMAT: &[FormatItem<'_>] = &[
    FormatItem::Compound(DATE_FORMAT),
    FormatItem::Literal(b" "),
    FormatItem::Compound(TIME_FORMAT),
    FormatItem::Literal(b" "),
    FormatItem::Compound(UTC_OFFSET_FORMAT),
];

impl Serialize for OffsetDateTime {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[cfg(feature = "serde-human-readable")]
        if serializer.is_human_readable() {
            let Ok(s) = self.format(&OFFSET_DATE_TIME_FORMAT) else {
                return Err(S::Error::custom("failed formatting `OffsetDateTime`"));
            };
            return serializer.serialize_str(&s);
        }

        (
            self.year(),
            self.ordinal(),
            self.hour(),
            self.minute(),
            self.second(),
            self.nanosecond(),
            self.offset().whole_hours(),
            self.offset().minutes_past_hour(),
            self.offset().seconds_past_minute(),
        )
            .serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for OffsetDateTime {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        if cfg!(feature = "serde-human-readable") && deserializer.is_human_readable() {
            deserializer.deserialize_any(Visitor::<Self>(PhantomData))
        } else {
            deserializer.deserialize_tuple(9, Visitor::<Self>(PhantomData))
        }
    }
}
// endregion OffsetDateTime

// region: PrimitiveDateTime
/// The format used when serializing and deserializing a human-readable `PrimitiveDateTime`.
#[cfg(feature = "parsing")]
const PRIMITIVE_DATE_TIME_FORMAT: &[FormatItem<'_>] = &[
    FormatItem::Compound(DATE_FORMAT),
    FormatItem::Literal(b" "),
    FormatItem::Compound(TIME_FORMAT),
];

impl Serialize for PrimitiveDateTime {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[cfg(feature = "serde-human-readable")]
        if serializer.is_human_readable() {
            let Ok(s) = self.format(&PRIMITIVE_DATE_TIME_FORMAT) else {
                return Err(S::Error::custom("failed formatting `PrimitiveDateTime`"));
            };
            return serializer.serialize_str(&s);
        }

        (
            self.year(),
            self.ordinal(),
            self.hour(),
            self.minute(),
            self.second(),
            self.nanosecond(),
        )
            .serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for PrimitiveDateTime {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        if cfg!(feature = "serde-human-readable") && deserializer.is_human_readable() {
            deserializer.deserialize_any(Visitor::<Self>(PhantomData))
        } else {
            deserializer.deserialize_tuple(6, Visitor::<Self>(PhantomData))
        }
    }
}
// endregion PrimitiveDateTime

// region: Time
/// The format used when serializing and deserializing a human-readable `Time`.
#[cfg(feature = "parsing")]
const TIME_FORMAT: &[FormatItem<'_>] = &[
    FormatItem::Component(Component::Hour(<modifier::Hour>::default())),
    FormatItem::Literal(b":"),
    FormatItem::Component(Component::Minute(<modifier::Minute>::default())),
    FormatItem::Literal(b":"),
    FormatItem::Component(Component::Second(<modifier::Second>::default())),
    FormatItem::Literal(b"."),
    FormatItem::Component(Component::Subsecond(<modifier::Subsecond>::default())),
];

impl Serialize for Time {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[cfg(feature = "serde-human-readable")]
        if serializer.is_human_readable() {
            let Ok(s) = self.format(&TIME_FORMAT) else {
                return Err(S::Error::custom("failed formatting `Time`"));
            };
            return serializer.serialize_str(&s);
        }

        (self.hour(), self.minute(), self.second(), self.nanosecond()).serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for Time {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        if cfg!(feature = "serde-human-readable") && deserializer.is_human_readable() {
            deserializer.deserialize_any(Visitor::<Self>(PhantomData))
        } else {
            deserializer.deserialize_tuple(4, Visitor::<Self>(PhantomData))
        }
    }
}
// endregion Time

// region: UtcOffset
/// The format used when serializing and deserializing a human-readable `UtcOffset`.
#[cfg(feature = "parsing")]
const UTC_OFFSET_FORMAT: &[FormatItem<'_>] = &[
    FormatItem::Component(Component::OffsetHour(modifier::OffsetHour::default())),
    FormatItem::Optional(&FormatItem::Compound(&[
        FormatItem::Literal(b":"),
        FormatItem::Component(Component::OffsetMinute(modifier::OffsetMinute::default())),
        FormatItem::Optional(&FormatItem::Compound(&[
            FormatItem::Literal(b":"),
            FormatItem::Component(Component::OffsetSecond(modifier::OffsetSecond::default())),
        ])),
    ])),
];

impl Serialize for UtcOffset {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[cfg(feature = "serde-human-readable")]
        if serializer.is_human_readable() {
            let Ok(s) = self.format(&UTC_OFFSET_FORMAT) else {
                return Err(S::Error::custom("failed formatting `UtcOffset`"));
            };
            return serializer.serialize_str(&s);
        }

        (
            self.whole_hours(),
            self.minutes_past_hour(),
            self.seconds_past_minute(),
        )
            .serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for UtcOffset {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        if cfg!(feature = "serde-human-readable") && deserializer.is_human_readable() {
            deserializer.deserialize_any(Visitor::<Self>(PhantomData))
        } else {
            deserializer.deserialize_tuple(3, Visitor::<Self>(PhantomData))
        }
    }
}
// endregion UtcOffset

// region: Weekday
impl Serialize for Weekday {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[cfg(feature = "serde-human-readable")]
        if serializer.is_human_readable() {
            #[cfg(not(feature = "std"))]
            use alloc::string::ToString;
            return self.to_string().serialize(serializer);
        }

        self.number_from_monday().serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for Weekday {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        if cfg!(feature = "serde-human-readable") && deserializer.is_human_readable() {
            deserializer.deserialize_any(Visitor::<Self>(PhantomData))
        } else {
            deserializer.deserialize_u8(Visitor::<Self>(PhantomData))
        }
    }
}
// endregion Weekday

// region: Month
impl Serialize for Month {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[cfg(feature = "serde-human-readable")]
        if serializer.is_human_readable() {
            #[cfg(not(feature = "std"))]
            use alloc::string::String;
            return self.to_string().serialize(serializer);
        }

        (*self as u8).serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for Month {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        if cfg!(feature = "serde-human-readable") && deserializer.is_human_readable() {
            deserializer.deserialize_any(Visitor::<Self>(PhantomData))
        } else {
            deserializer.deserialize_u8(Visitor::<Self>(PhantomData))
        }
    }
}
// endregion Month
