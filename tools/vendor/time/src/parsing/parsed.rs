//! Information parsed from an input and format description.

use core::num::{NonZeroU16, NonZeroU8};

use deranged::{
    OptionRangedI128, OptionRangedI32, OptionRangedI8, OptionRangedU16, OptionRangedU32,
    OptionRangedU8, RangedI128, RangedI32, RangedI8, RangedU16, RangedU32, RangedU8,
};

use crate::convert::{Day, Hour, Minute, Nanosecond, Second};
use crate::date::{MAX_YEAR, MIN_YEAR};
use crate::date_time::{maybe_offset_from_offset, offset_kind, DateTime, MaybeOffset};
use crate::error::TryFromParsed::InsufficientInformation;
#[cfg(feature = "alloc")]
use crate::format_description::OwnedFormatItem;
use crate::format_description::{modifier, Component, FormatItem};
use crate::parsing::component::{
    parse_day, parse_end, parse_hour, parse_ignore, parse_minute, parse_month, parse_offset_hour,
    parse_offset_minute, parse_offset_second, parse_ordinal, parse_period, parse_second,
    parse_subsecond, parse_unix_timestamp, parse_week_number, parse_weekday, parse_year, Period,
};
use crate::parsing::ParsedItem;
use crate::{error, Date, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset, Weekday};

/// Sealed to prevent downstream implementations.
mod sealed {
    use super::*;

    /// A trait to allow `parse_item` to be generic.
    pub trait AnyFormatItem {
        /// Parse a single item, returning the remaining input on success.
        fn parse_item<'a>(
            &self,
            parsed: &mut Parsed,
            input: &'a [u8],
        ) -> Result<&'a [u8], error::ParseFromDescription>;
    }
}

impl sealed::AnyFormatItem for FormatItem<'_> {
    fn parse_item<'a>(
        &self,
        parsed: &mut Parsed,
        input: &'a [u8],
    ) -> Result<&'a [u8], error::ParseFromDescription> {
        match self {
            Self::Literal(literal) => Parsed::parse_literal(input, literal),
            Self::Component(component) => parsed.parse_component(input, *component),
            Self::Compound(compound) => parsed.parse_items(input, compound),
            Self::Optional(item) => parsed.parse_item(input, *item).or(Ok(input)),
            Self::First(items) => {
                let mut first_err = None;

                for item in items.iter() {
                    match parsed.parse_item(input, item) {
                        Ok(remaining_input) => return Ok(remaining_input),
                        Err(err) if first_err.is_none() => first_err = Some(err),
                        Err(_) => {}
                    }
                }

                match first_err {
                    Some(err) => Err(err),
                    // This location will be reached if the slice is empty, skipping the `for` loop.
                    // As this case is expected to be uncommon, there's no need to check up front.
                    None => Ok(input),
                }
            }
        }
    }
}

#[cfg(feature = "alloc")]
impl sealed::AnyFormatItem for OwnedFormatItem {
    fn parse_item<'a>(
        &self,
        parsed: &mut Parsed,
        input: &'a [u8],
    ) -> Result<&'a [u8], error::ParseFromDescription> {
        match self {
            Self::Literal(literal) => Parsed::parse_literal(input, literal),
            Self::Component(component) => parsed.parse_component(input, *component),
            Self::Compound(compound) => parsed.parse_items(input, compound),
            Self::Optional(item) => parsed.parse_item(input, item.as_ref()).or(Ok(input)),
            Self::First(items) => {
                let mut first_err = None;

                for item in items.iter() {
                    match parsed.parse_item(input, item) {
                        Ok(remaining_input) => return Ok(remaining_input),
                        Err(err) if first_err.is_none() => first_err = Some(err),
                        Err(_) => {}
                    }
                }

                match first_err {
                    Some(err) => Err(err),
                    // This location will be reached if the slice is empty, skipping the `for` loop.
                    // As this case is expected to be uncommon, there's no need to check up front.
                    None => Ok(input),
                }
            }
        }
    }
}

/// All information parsed.
///
/// This information is directly used to construct the final values.
///
/// Most users will not need think about this struct in any way. It is public to allow for manual
/// control over values, in the instance that the default parser is insufficient.
#[derive(Debug, Clone, Copy)]
pub struct Parsed {
    /// Calendar year.
    year: OptionRangedI32<{ MIN_YEAR }, { MAX_YEAR }>,
    /// The last two digits of the calendar year.
    year_last_two: OptionRangedU8<0, 99>,
    /// Year of the [ISO week date](https://en.wikipedia.org/wiki/ISO_week_date).
    iso_year: OptionRangedI32<{ MIN_YEAR }, { MAX_YEAR }>,
    /// The last two digits of the ISO week year.
    iso_year_last_two: OptionRangedU8<0, 99>,
    /// Month of the year.
    month: Option<Month>,
    /// Week of the year, where week one begins on the first Sunday of the calendar year.
    sunday_week_number: OptionRangedU8<0, 53>,
    /// Week of the year, where week one begins on the first Monday of the calendar year.
    monday_week_number: OptionRangedU8<0, 53>,
    /// Week of the year, where week one is the Monday-to-Sunday period containing January 4.
    iso_week_number: OptionRangedU8<1, 53>,
    /// Day of the week.
    weekday: Option<Weekday>,
    /// Day of the year.
    ordinal: OptionRangedU16<1, 366>,
    /// Day of the month.
    day: OptionRangedU8<1, 31>,
    /// Hour within the day.
    hour_24: OptionRangedU8<0, { Hour.per(Day) - 1 }>,
    /// Hour within the 12-hour period (midnight to noon or vice versa). This is typically used in
    /// conjunction with AM/PM, which is indicated by the `hour_12_is_pm` field.
    hour_12: OptionRangedU8<1, 12>,
    /// Whether the `hour_12` field indicates a time that "PM".
    hour_12_is_pm: Option<bool>,
    /// Minute within the hour.
    // minute: MaybeUninit<u8>,
    minute: OptionRangedU8<0, { Minute.per(Hour) - 1 }>,
    /// Second within the minute.
    // do not subtract one, as leap seconds may be allowed
    second: OptionRangedU8<0, { Second.per(Minute) }>,
    /// Nanosecond within the second.
    subsecond: OptionRangedU32<0, { Nanosecond.per(Second) - 1 }>,
    /// Whole hours of the UTC offset.
    offset_hour: OptionRangedI8<-23, 23>,
    /// Minutes within the hour of the UTC offset.
    offset_minute:
        OptionRangedI8<{ -((Minute.per(Hour) - 1) as i8) }, { (Minute.per(Hour) - 1) as _ }>,
    /// Seconds within the minute of the UTC offset.
    offset_second:
        OptionRangedI8<{ -((Second.per(Minute) - 1) as i8) }, { (Second.per(Minute) - 1) as _ }>,
    /// The Unix timestamp in nanoseconds.
    // unix_timestamp_nanos: MaybeUninit<i128>,
    unix_timestamp_nanos: OptionRangedI128<
        { Date::MIN.midnight().assume_utc().unix_timestamp_nanos() },
        {
            Date::MAX
                .with_time(Time::MAX)
                .assume_utc()
                .unix_timestamp_nanos()
        },
    >,
    /// Indicates whether the [`UtcOffset`] is negative. This information is obtained when parsing
    /// the offset hour, but may not otherwise be stored due to "-0" being equivalent to "0".
    offset_is_negative: Option<bool>,
    /// Indicates whether a leap second is permitted to be parsed. This is required by some
    /// well-known formats.
    pub(super) leap_second_allowed: bool,
}

impl Parsed {
    /// Create a new instance of `Parsed` with no information known.
    pub const fn new() -> Self {
        Self {
            year: OptionRangedI32::None,
            year_last_two: OptionRangedU8::None,
            iso_year: OptionRangedI32::None,
            iso_year_last_two: OptionRangedU8::None,
            month: None,
            sunday_week_number: OptionRangedU8::None,
            monday_week_number: OptionRangedU8::None,
            iso_week_number: OptionRangedU8::None,
            weekday: None,
            ordinal: OptionRangedU16::None,
            day: OptionRangedU8::None,
            hour_24: OptionRangedU8::None,
            hour_12: OptionRangedU8::None,
            hour_12_is_pm: None,
            minute: OptionRangedU8::None,
            second: OptionRangedU8::None,
            subsecond: OptionRangedU32::None,
            offset_hour: OptionRangedI8::None,
            offset_minute: OptionRangedI8::None,
            offset_second: OptionRangedI8::None,
            unix_timestamp_nanos: OptionRangedI128::None,
            offset_is_negative: None,
            leap_second_allowed: false,
        }
    }

    /// Parse a single [`FormatItem`] or [`OwnedFormatItem`], mutating the struct. The remaining
    /// input is returned as the `Ok` value.
    ///
    /// If a [`FormatItem::Optional`] or [`OwnedFormatItem::Optional`] is passed, parsing will not
    /// fail; the input will be returned as-is if the expected format is not present.
    pub fn parse_item<'a>(
        &mut self,
        input: &'a [u8],
        item: &impl sealed::AnyFormatItem,
    ) -> Result<&'a [u8], error::ParseFromDescription> {
        item.parse_item(self, input)
    }

    /// Parse a sequence of [`FormatItem`]s or [`OwnedFormatItem`]s, mutating the struct. The
    /// remaining input is returned as the `Ok` value.
    ///
    /// This method will fail if any of the contained [`FormatItem`]s or [`OwnedFormatItem`]s fail
    /// to parse. `self` will not be mutated in this instance.
    pub fn parse_items<'a>(
        &mut self,
        mut input: &'a [u8],
        items: &[impl sealed::AnyFormatItem],
    ) -> Result<&'a [u8], error::ParseFromDescription> {
        // Make a copy that we can mutate. It will only be set to the user's copy if everything
        // succeeds.
        let mut this = *self;
        for item in items {
            input = this.parse_item(input, item)?;
        }
        *self = this;
        Ok(input)
    }

    /// Parse a literal byte sequence. The remaining input is returned as the `Ok` value.
    pub fn parse_literal<'a>(
        input: &'a [u8],
        literal: &[u8],
    ) -> Result<&'a [u8], error::ParseFromDescription> {
        input
            .strip_prefix(literal)
            .ok_or(error::ParseFromDescription::InvalidLiteral)
    }

    /// Parse a single component, mutating the struct. The remaining input is returned as the `Ok`
    /// value.
    pub fn parse_component<'a>(
        &mut self,
        input: &'a [u8],
        component: Component,
    ) -> Result<&'a [u8], error::ParseFromDescription> {
        use error::ParseFromDescription::InvalidComponent;

        match component {
            Component::Day(modifiers) => parse_day(input, modifiers)
                .and_then(|parsed| parsed.consume_value(|value| self.set_day(value)))
                .ok_or(InvalidComponent("day")),
            Component::Month(modifiers) => parse_month(input, modifiers)
                .and_then(|parsed| parsed.consume_value(|value| self.set_month(value)))
                .ok_or(InvalidComponent("month")),
            Component::Ordinal(modifiers) => parse_ordinal(input, modifiers)
                .and_then(|parsed| parsed.consume_value(|value| self.set_ordinal(value)))
                .ok_or(InvalidComponent("ordinal")),
            Component::Weekday(modifiers) => parse_weekday(input, modifiers)
                .and_then(|parsed| parsed.consume_value(|value| self.set_weekday(value)))
                .ok_or(InvalidComponent("weekday")),
            Component::WeekNumber(modifiers) => {
                let ParsedItem(remaining, value) =
                    parse_week_number(input, modifiers).ok_or(InvalidComponent("week number"))?;
                match modifiers.repr {
                    modifier::WeekNumberRepr::Iso => {
                        NonZeroU8::new(value).and_then(|value| self.set_iso_week_number(value))
                    }
                    modifier::WeekNumberRepr::Sunday => self.set_sunday_week_number(value),
                    modifier::WeekNumberRepr::Monday => self.set_monday_week_number(value),
                }
                .ok_or(InvalidComponent("week number"))?;
                Ok(remaining)
            }
            Component::Year(modifiers) => {
                let ParsedItem(remaining, value) =
                    parse_year(input, modifiers).ok_or(InvalidComponent("year"))?;
                match (modifiers.iso_week_based, modifiers.repr) {
                    (false, modifier::YearRepr::Full) => self.set_year(value),
                    (false, modifier::YearRepr::LastTwo) => self.set_year_last_two(value as _),
                    (true, modifier::YearRepr::Full) => self.set_iso_year(value),
                    (true, modifier::YearRepr::LastTwo) => self.set_iso_year_last_two(value as _),
                }
                .ok_or(InvalidComponent("year"))?;
                Ok(remaining)
            }
            Component::Hour(modifiers) => {
                let ParsedItem(remaining, value) =
                    parse_hour(input, modifiers).ok_or(InvalidComponent("hour"))?;
                if modifiers.is_12_hour_clock {
                    NonZeroU8::new(value).and_then(|value| self.set_hour_12(value))
                } else {
                    self.set_hour_24(value)
                }
                .ok_or(InvalidComponent("hour"))?;
                Ok(remaining)
            }
            Component::Minute(modifiers) => parse_minute(input, modifiers)
                .and_then(|parsed| parsed.consume_value(|value| self.set_minute(value)))
                .ok_or(InvalidComponent("minute")),
            Component::Period(modifiers) => parse_period(input, modifiers)
                .and_then(|parsed| {
                    parsed.consume_value(|value| self.set_hour_12_is_pm(value == Period::Pm))
                })
                .ok_or(InvalidComponent("period")),
            Component::Second(modifiers) => parse_second(input, modifiers)
                .and_then(|parsed| parsed.consume_value(|value| self.set_second(value)))
                .ok_or(InvalidComponent("second")),
            Component::Subsecond(modifiers) => parse_subsecond(input, modifiers)
                .and_then(|parsed| parsed.consume_value(|value| self.set_subsecond(value)))
                .ok_or(InvalidComponent("subsecond")),
            Component::OffsetHour(modifiers) => parse_offset_hour(input, modifiers)
                .and_then(|parsed| {
                    parsed.consume_value(|(value, is_negative)| {
                        self.set_offset_hour(value)?;
                        self.offset_is_negative = Some(is_negative);
                        Some(())
                    })
                })
                .ok_or(InvalidComponent("offset hour")),
            Component::OffsetMinute(modifiers) => parse_offset_minute(input, modifiers)
                .and_then(|parsed| {
                    parsed.consume_value(|value| self.set_offset_minute_signed(value))
                })
                .ok_or(InvalidComponent("offset minute")),
            Component::OffsetSecond(modifiers) => parse_offset_second(input, modifiers)
                .and_then(|parsed| {
                    parsed.consume_value(|value| self.set_offset_second_signed(value))
                })
                .ok_or(InvalidComponent("offset second")),
            Component::Ignore(modifiers) => parse_ignore(input, modifiers)
                .map(ParsedItem::<()>::into_inner)
                .ok_or(InvalidComponent("ignore")),
            Component::UnixTimestamp(modifiers) => parse_unix_timestamp(input, modifiers)
                .and_then(|parsed| {
                    parsed.consume_value(|value| self.set_unix_timestamp_nanos(value))
                })
                .ok_or(InvalidComponent("unix_timestamp")),
            Component::End(modifiers) => parse_end(input, modifiers)
                .map(ParsedItem::<()>::into_inner)
                .ok_or(error::ParseFromDescription::UnexpectedTrailingCharacters),
        }
    }
}

/// Getter methods
impl Parsed {
    /// Obtain the `year` component.
    pub const fn year(&self) -> Option<i32> {
        self.year.get_primitive()
    }

    /// Obtain the `year_last_two` component.
    pub const fn year_last_two(&self) -> Option<u8> {
        self.year_last_two.get_primitive()
    }

    /// Obtain the `iso_year` component.
    pub const fn iso_year(&self) -> Option<i32> {
        self.iso_year.get_primitive()
    }

    /// Obtain the `iso_year_last_two` component.
    pub const fn iso_year_last_two(&self) -> Option<u8> {
        self.iso_year_last_two.get_primitive()
    }

    /// Obtain the `month` component.
    pub const fn month(&self) -> Option<Month> {
        self.month
    }

    /// Obtain the `sunday_week_number` component.
    pub const fn sunday_week_number(&self) -> Option<u8> {
        self.sunday_week_number.get_primitive()
    }

    /// Obtain the `monday_week_number` component.
    pub const fn monday_week_number(&self) -> Option<u8> {
        self.monday_week_number.get_primitive()
    }

    /// Obtain the `iso_week_number` component.
    pub const fn iso_week_number(&self) -> Option<NonZeroU8> {
        NonZeroU8::new(const_try_opt!(self.iso_week_number.get_primitive()))
    }

    /// Obtain the `weekday` component.
    pub const fn weekday(&self) -> Option<Weekday> {
        self.weekday
    }

    /// Obtain the `ordinal` component.
    pub const fn ordinal(&self) -> Option<NonZeroU16> {
        NonZeroU16::new(const_try_opt!(self.ordinal.get_primitive()))
    }

    /// Obtain the `day` component.
    pub const fn day(&self) -> Option<NonZeroU8> {
        NonZeroU8::new(const_try_opt!(self.day.get_primitive()))
    }

    /// Obtain the `hour_24` component.
    pub const fn hour_24(&self) -> Option<u8> {
        self.hour_24.get_primitive()
    }

    /// Obtain the `hour_12` component.
    pub const fn hour_12(&self) -> Option<NonZeroU8> {
        NonZeroU8::new(const_try_opt!(self.hour_12.get_primitive()))
    }

    /// Obtain the `hour_12_is_pm` component.
    pub const fn hour_12_is_pm(&self) -> Option<bool> {
        self.hour_12_is_pm
    }

    /// Obtain the `minute` component.
    pub const fn minute(&self) -> Option<u8> {
        self.minute.get_primitive()
    }

    /// Obtain the `second` component.
    pub const fn second(&self) -> Option<u8> {
        self.second.get_primitive()
    }

    /// Obtain the `subsecond` component.
    pub const fn subsecond(&self) -> Option<u32> {
        self.subsecond.get_primitive()
    }

    /// Obtain the `offset_hour` component.
    pub const fn offset_hour(&self) -> Option<i8> {
        self.offset_hour.get_primitive()
    }

    /// Obtain the absolute value of the `offset_minute` component.
    #[doc(hidden)]
    #[deprecated(since = "0.3.8", note = "use `parsed.offset_minute_signed()` instead")]
    pub const fn offset_minute(&self) -> Option<u8> {
        Some(const_try_opt!(self.offset_minute_signed()).unsigned_abs())
    }

    /// Obtain the `offset_minute` component.
    pub const fn offset_minute_signed(&self) -> Option<i8> {
        match (self.offset_minute.get_primitive(), self.offset_is_negative) {
            (Some(offset_minute), Some(true)) => Some(-offset_minute),
            (Some(offset_minute), _) => Some(offset_minute),
            (None, _) => None,
        }
    }

    /// Obtain the absolute value of the `offset_second` component.
    #[doc(hidden)]
    #[deprecated(since = "0.3.8", note = "use `parsed.offset_second_signed()` instead")]
    pub const fn offset_second(&self) -> Option<u8> {
        Some(const_try_opt!(self.offset_second_signed()).unsigned_abs())
    }

    /// Obtain the `offset_second` component.
    pub const fn offset_second_signed(&self) -> Option<i8> {
        match (self.offset_second.get_primitive(), self.offset_is_negative) {
            (Some(offset_second), Some(true)) => Some(-offset_second),
            (Some(offset_second), _) => Some(offset_second),
            (None, _) => None,
        }
    }

    /// Obtain the `unix_timestamp_nanos` component.
    pub const fn unix_timestamp_nanos(&self) -> Option<i128> {
        self.unix_timestamp_nanos.get_primitive()
    }
}

/// Generate setters based on the builders.
macro_rules! setters {
    ($($name:ident $setter:ident $builder:ident $type:ty;)*) => {$(
        #[doc = concat!("Set the `", stringify!($setter), "` component.")]
        pub fn $setter(&mut self, value: $type) -> Option<()> {
            *self = self.$builder(value)?;
            Some(())
        }
    )*};
}

/// Setter methods
///
/// All setters return `Option<()>`, which is `Some` if the value was set, and `None` if not. The
/// setters _may_ fail if the value is invalid, though behavior is not guaranteed.
impl Parsed {
    setters! {
        year set_year with_year i32;
        year_last_two set_year_last_two with_year_last_two u8;
        iso_year set_iso_year with_iso_year i32;
        iso_year_last_two set_iso_year_last_two with_iso_year_last_two u8;
        month set_month with_month Month;
        sunday_week_number set_sunday_week_number with_sunday_week_number u8;
        monday_week_number set_monday_week_number with_monday_week_number u8;
        iso_week_number set_iso_week_number with_iso_week_number NonZeroU8;
        weekday set_weekday with_weekday Weekday;
        ordinal set_ordinal with_ordinal NonZeroU16;
        day set_day with_day NonZeroU8;
        hour_24 set_hour_24 with_hour_24 u8;
        hour_12 set_hour_12 with_hour_12 NonZeroU8;
        hour_12_is_pm set_hour_12_is_pm with_hour_12_is_pm bool;
        minute set_minute with_minute u8;
        second set_second with_second u8;
        subsecond set_subsecond with_subsecond u32;
        offset_hour set_offset_hour with_offset_hour i8;
        offset_minute set_offset_minute_signed with_offset_minute_signed i8;
        offset_second set_offset_second_signed with_offset_second_signed i8;
        unix_timestamp_nanos set_unix_timestamp_nanos with_unix_timestamp_nanos i128;
    }

    /// Set the `offset_minute` component.
    #[doc(hidden)]
    #[deprecated(
        since = "0.3.8",
        note = "use `parsed.set_offset_minute_signed()` instead"
    )]
    pub fn set_offset_minute(&mut self, value: u8) -> Option<()> {
        if value > i8::MAX as u8 {
            None
        } else {
            self.set_offset_minute_signed(value as _)
        }
    }

    /// Set the `offset_minute` component.
    #[doc(hidden)]
    #[deprecated(
        since = "0.3.8",
        note = "use `parsed.set_offset_second_signed()` instead"
    )]
    pub fn set_offset_second(&mut self, value: u8) -> Option<()> {
        if value > i8::MAX as u8 {
            None
        } else {
            self.set_offset_second_signed(value as _)
        }
    }
}

/// Builder methods
///
/// All builder methods return `Option<Self>`, which is `Some` if the value was set, and `None` if
/// not. The builder methods _may_ fail if the value is invalid, though behavior is not guaranteed.
impl Parsed {
    /// Set the `year` component and return `self`.
    pub const fn with_year(mut self, value: i32) -> Option<Self> {
        self.year = OptionRangedI32::Some(const_try_opt!(RangedI32::new(value)));
        Some(self)
    }

    /// Set the `year_last_two` component and return `self`.
    pub const fn with_year_last_two(mut self, value: u8) -> Option<Self> {
        self.year_last_two = OptionRangedU8::Some(const_try_opt!(RangedU8::new(value)));
        Some(self)
    }

    /// Set the `iso_year` component and return `self`.
    pub const fn with_iso_year(mut self, value: i32) -> Option<Self> {
        self.iso_year = OptionRangedI32::Some(const_try_opt!(RangedI32::new(value)));
        Some(self)
    }

    /// Set the `iso_year_last_two` component and return `self`.
    pub const fn with_iso_year_last_two(mut self, value: u8) -> Option<Self> {
        self.iso_year_last_two = OptionRangedU8::Some(const_try_opt!(RangedU8::new(value)));
        Some(self)
    }

    /// Set the `month` component and return `self`.
    pub const fn with_month(mut self, value: Month) -> Option<Self> {
        self.month = Some(value);
        Some(self)
    }

    /// Set the `sunday_week_number` component and return `self`.
    pub const fn with_sunday_week_number(mut self, value: u8) -> Option<Self> {
        self.sunday_week_number = OptionRangedU8::Some(const_try_opt!(RangedU8::new(value)));
        Some(self)
    }

    /// Set the `monday_week_number` component and return `self`.
    pub const fn with_monday_week_number(mut self, value: u8) -> Option<Self> {
        self.monday_week_number = OptionRangedU8::Some(const_try_opt!(RangedU8::new(value)));
        Some(self)
    }

    /// Set the `iso_week_number` component and return `self`.
    pub const fn with_iso_week_number(mut self, value: NonZeroU8) -> Option<Self> {
        self.iso_week_number = OptionRangedU8::Some(const_try_opt!(RangedU8::new(value.get())));
        Some(self)
    }

    /// Set the `weekday` component and return `self`.
    pub const fn with_weekday(mut self, value: Weekday) -> Option<Self> {
        self.weekday = Some(value);
        Some(self)
    }

    /// Set the `ordinal` component and return `self`.
    pub const fn with_ordinal(mut self, value: NonZeroU16) -> Option<Self> {
        self.ordinal = OptionRangedU16::Some(const_try_opt!(RangedU16::new(value.get())));
        Some(self)
    }

    /// Set the `day` component and return `self`.
    pub const fn with_day(mut self, value: NonZeroU8) -> Option<Self> {
        self.day = OptionRangedU8::Some(const_try_opt!(RangedU8::new(value.get())));
        Some(self)
    }

    /// Set the `hour_24` component and return `self`.
    pub const fn with_hour_24(mut self, value: u8) -> Option<Self> {
        self.hour_24 = OptionRangedU8::Some(const_try_opt!(RangedU8::new(value)));
        Some(self)
    }

    /// Set the `hour_12` component and return `self`.
    pub const fn with_hour_12(mut self, value: NonZeroU8) -> Option<Self> {
        self.hour_12 = OptionRangedU8::Some(const_try_opt!(RangedU8::new(value.get())));
        Some(self)
    }

    /// Set the `hour_12_is_pm` component and return `self`.
    pub const fn with_hour_12_is_pm(mut self, value: bool) -> Option<Self> {
        self.hour_12_is_pm = Some(value);
        Some(self)
    }

    /// Set the `minute` component and return `self`.
    pub const fn with_minute(mut self, value: u8) -> Option<Self> {
        self.minute = OptionRangedU8::Some(const_try_opt!(RangedU8::new(value)));
        Some(self)
    }

    /// Set the `second` component and return `self`.
    pub const fn with_second(mut self, value: u8) -> Option<Self> {
        self.second = OptionRangedU8::Some(const_try_opt!(RangedU8::new(value)));
        Some(self)
    }

    /// Set the `subsecond` component and return `self`.
    pub const fn with_subsecond(mut self, value: u32) -> Option<Self> {
        self.subsecond = OptionRangedU32::Some(const_try_opt!(RangedU32::new(value)));
        Some(self)
    }

    /// Set the `offset_hour` component and return `self`.
    pub const fn with_offset_hour(mut self, value: i8) -> Option<Self> {
        self.offset_hour = OptionRangedI8::Some(const_try_opt!(RangedI8::new(value)));
        Some(self)
    }

    /// Set the `offset_minute` component and return `self`.
    #[doc(hidden)]
    #[deprecated(
        since = "0.3.8",
        note = "use `parsed.with_offset_minute_signed()` instead"
    )]
    pub const fn with_offset_minute(self, value: u8) -> Option<Self> {
        if value > i8::MAX as u8 {
            None
        } else {
            self.with_offset_minute_signed(value as _)
        }
    }

    /// Set the `offset_minute` component and return `self`.
    pub const fn with_offset_minute_signed(mut self, value: i8) -> Option<Self> {
        self.offset_minute = OptionRangedI8::Some(const_try_opt!(RangedI8::new(value)));
        Some(self)
    }

    /// Set the `offset_minute` component and return `self`.
    #[doc(hidden)]
    #[deprecated(
        since = "0.3.8",
        note = "use `parsed.with_offset_second_signed()` instead"
    )]
    pub const fn with_offset_second(self, value: u8) -> Option<Self> {
        if value > i8::MAX as u8 {
            None
        } else {
            self.with_offset_second_signed(value as _)
        }
    }

    /// Set the `offset_second` component and return `self`.
    pub const fn with_offset_second_signed(mut self, value: i8) -> Option<Self> {
        self.offset_second = OptionRangedI8::Some(const_try_opt!(RangedI8::new(value)));
        Some(self)
    }

    /// Set the `unix_timestamp_nanos` component and return `self`.
    pub const fn with_unix_timestamp_nanos(mut self, value: i128) -> Option<Self> {
        self.unix_timestamp_nanos = OptionRangedI128::Some(const_try_opt!(RangedI128::new(value)));
        Some(self)
    }
}

impl TryFrom<Parsed> for Date {
    type Error = error::TryFromParsed;

    fn try_from(parsed: Parsed) -> Result<Self, Self::Error> {
        /// Match on the components that need to be present.
        macro_rules! match_ {
            (_ => $catch_all:expr $(,)?) => {
                $catch_all
            };
            (($($name:ident),* $(,)?) => $arm:expr, $($rest:tt)*) => {
                if let ($(Some($name)),*) = ($(parsed.$name()),*) {
                    $arm
                } else {
                    match_!($($rest)*)
                }
            };
        }

        /// Get the value needed to adjust the ordinal day for Sunday and Monday-based week
        /// numbering.
        const fn adjustment(year: i32) -> i16 {
            match Date::__from_ordinal_date_unchecked(year, 1).weekday() {
                Weekday::Monday => 7,
                Weekday::Tuesday => 1,
                Weekday::Wednesday => 2,
                Weekday::Thursday => 3,
                Weekday::Friday => 4,
                Weekday::Saturday => 5,
                Weekday::Sunday => 6,
            }
        }

        // TODO Only the basics have been covered. There are many other valid values that are not
        // currently constructed from the information known.

        match_! {
            (year, ordinal) => Ok(Self::from_ordinal_date(year, ordinal.get())?),
            (year, month, day) => Ok(Self::from_calendar_date(year, month, day.get())?),
            (iso_year, iso_week_number, weekday) => Ok(Self::from_iso_week_date(
                iso_year,
                iso_week_number.get(),
                weekday,
            )?),
            (year, sunday_week_number, weekday) => Ok(Self::from_ordinal_date(
                year,
                (sunday_week_number as i16 * 7 + weekday.number_days_from_sunday() as i16
                    - adjustment(year)
                    + 1) as u16,
            )?),
            (year, monday_week_number, weekday) => Ok(Self::from_ordinal_date(
                year,
                (monday_week_number as i16 * 7 + weekday.number_days_from_monday() as i16
                    - adjustment(year)
                    + 1) as u16,
            )?),
            _ => Err(InsufficientInformation),
        }
    }
}

impl TryFrom<Parsed> for Time {
    type Error = error::TryFromParsed;

    fn try_from(parsed: Parsed) -> Result<Self, Self::Error> {
        let hour = match (parsed.hour_24(), parsed.hour_12(), parsed.hour_12_is_pm()) {
            (Some(hour), _, _) => hour,
            (_, Some(hour), Some(false)) if hour.get() == 12 => 0,
            (_, Some(hour), Some(true)) if hour.get() == 12 => 12,
            (_, Some(hour), Some(false)) => hour.get(),
            (_, Some(hour), Some(true)) => hour.get() + 12,
            _ => return Err(InsufficientInformation),
        };

        if parsed.hour_24().is_none()
            && parsed.hour_12().is_some()
            && parsed.hour_12_is_pm().is_some()
            && parsed.minute().is_none()
            && parsed.second().is_none()
            && parsed.subsecond().is_none()
        {
            return Ok(Self::from_hms_nano(hour, 0, 0, 0)?);
        }

        // Reject combinations such as hour-second with minute omitted.
        match (parsed.minute(), parsed.second(), parsed.subsecond()) {
            (None, None, None) => Ok(Self::from_hms_nano(hour, 0, 0, 0)?),
            (Some(minute), None, None) => Ok(Self::from_hms_nano(hour, minute, 0, 0)?),
            (Some(minute), Some(second), None) => Ok(Self::from_hms_nano(hour, minute, second, 0)?),
            (Some(minute), Some(second), Some(subsecond)) => {
                Ok(Self::from_hms_nano(hour, minute, second, subsecond)?)
            }
            _ => Err(InsufficientInformation),
        }
    }
}

impl TryFrom<Parsed> for UtcOffset {
    type Error = error::TryFromParsed;

    fn try_from(parsed: Parsed) -> Result<Self, Self::Error> {
        let hour = parsed.offset_hour().ok_or(InsufficientInformation)?;
        let minute = parsed.offset_minute_signed().unwrap_or(0);
        let second = parsed.offset_second_signed().unwrap_or(0);

        Self::from_hms(hour, minute, second).map_err(|mut err| {
            // Provide the user a more accurate error.
            if err.name == "hours" {
                err.name = "offset hour";
            } else if err.name == "minutes" {
                err.name = "offset minute";
            } else if err.name == "seconds" {
                err.name = "offset second";
            }
            err.into()
        })
    }
}

impl TryFrom<Parsed> for PrimitiveDateTime {
    type Error = <DateTime<offset_kind::None> as TryFrom<Parsed>>::Error;

    fn try_from(parsed: Parsed) -> Result<Self, Self::Error> {
        parsed.try_into().map(Self)
    }
}

impl TryFrom<Parsed> for OffsetDateTime {
    type Error = <DateTime<offset_kind::Fixed> as TryFrom<Parsed>>::Error;

    fn try_from(parsed: Parsed) -> Result<Self, Self::Error> {
        parsed.try_into().map(Self)
    }
}

impl<O: MaybeOffset> TryFrom<Parsed> for DateTime<O> {
    type Error = error::TryFromParsed;

    #[allow(clippy::unwrap_in_result)] // We know the values are valid.
    fn try_from(mut parsed: Parsed) -> Result<Self, Self::Error> {
        if O::HAS_LOGICAL_OFFSET {
            if let Some(timestamp) = parsed.unix_timestamp_nanos() {
                let DateTime { date, time, offset } =
                    DateTime::<offset_kind::Fixed>::from_unix_timestamp_nanos(timestamp)?;

                let mut value = Self {
                    date,
                    time,
                    offset: maybe_offset_from_offset::<O>(offset),
                };
                if let Some(subsecond) = parsed.subsecond() {
                    value = value.replace_nanosecond(subsecond)?;
                }
                return Ok(value);
            }
        }

        // Some well-known formats explicitly allow leap seconds. We don't currently support them,
        // so treat it as the nearest preceding moment that can be represented. Because leap seconds
        // always fall at the end of a month UTC, reject any that are at other times.
        let leap_second_input = if parsed.leap_second_allowed && parsed.second() == Some(60) {
            parsed.set_second(59).expect("59 is a valid second");
            parsed
                .set_subsecond(999_999_999)
                .expect("999_999_999 is a valid subsecond");
            true
        } else {
            false
        };

        let dt = Self {
            date: Date::try_from(parsed)?,
            time: Time::try_from(parsed)?,
            offset: O::try_from_parsed(parsed)?,
        };

        if leap_second_input && !dt.is_valid_leap_second_stand_in() {
            return Err(error::TryFromParsed::ComponentRange(
                error::ComponentRange {
                    name: "second",
                    minimum: 0,
                    maximum: 59,
                    value: 60,
                    conditional_range: true,
                },
            ));
        }
        Ok(dt)
    }
}
