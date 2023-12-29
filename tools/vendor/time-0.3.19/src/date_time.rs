//! The [`DateTime`] struct and its associated `impl`s.

// TODO(jhpratt) Document everything before making public.
#![allow(clippy::missing_docs_in_private_items)]
// This is intentional, as the struct will likely be exposed at some point.
#![allow(unreachable_pub)]

use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::{Add, AddAssign, Sub, SubAssign};
use core::time::Duration as StdDuration;
#[cfg(feature = "formatting")]
use std::io;
#[cfg(feature = "std")]
use std::time::SystemTime;

use crate::date::{MAX_YEAR, MIN_YEAR};
#[cfg(feature = "formatting")]
use crate::formatting::Formattable;
#[cfg(feature = "parsing")]
use crate::parsing::{Parsable, Parsed};
use crate::{error, util, Date, Duration, Month, Time, UtcOffset, Weekday};

#[allow(missing_debug_implementations, missing_copy_implementations)]
pub(crate) mod offset_kind {
    pub enum None {}
    pub enum Fixed {}
}

pub(crate) use sealed::MaybeOffset;
use sealed::{FixedOffset, HasOffset, UnsafeBool, WithoutOffset};
mod sealed {
    use super::*;

    #[allow(unreachable_pub)] // intentional
    pub trait MaybeOffsetType {}
    impl MaybeOffsetType for () {}
    impl MaybeOffsetType for UtcOffset {}

    #[allow(missing_debug_implementations)] // never stored as a field
    #[allow(missing_copy_implementations)] // only used in const contexts
    pub struct UnsafeBool(bool);
    impl UnsafeBool {
        // Skip code coverage because this is only used in const contexts. As such LLVM thinks it is
        // never called.
        #[cfg_attr(coverage_nightly, no_coverage)]
        pub(super) const unsafe fn new(value: bool) -> Self {
            Self(value)
        }

        // TODO When const trait impls are stabilized, change this to `impl const Deref for
        // UnsafeBool`.
        #[allow(clippy::wrong_self_convention)] // only used in const contexts
        pub(crate) const fn as_bool(self) -> bool {
            self.0
        }
    }

    pub trait MaybeOffset {
        // Statically restrict the type to be `()` and `UtcOffset`.
        #[cfg(feature = "quickcheck")]
        type OffsetType: Copy + MaybeOffsetType + quickcheck::Arbitrary;
        #[cfg(not(feature = "quickcheck"))]
        type OffsetType: Copy + MaybeOffsetType;

        /// Required to be `Self`. Used for bound equality.
        type OffsetKind;

        // True if and only if `Offset` is `UtcOffset`.
        // Note that an incorrect value here will cause undefined behavior inside
        // `maybe_offset_as_offset`.
        const HAS_OFFSET: UnsafeBool;

        fn as_offset(offset: Self::OffsetType) -> Option<UtcOffset>;
        #[cfg(feature = "parsing")]
        fn try_from_parsed(parsed: Parsed) -> Result<Self::OffsetType, error::TryFromParsed>;
        #[cfg(feature = "parsing")]
        fn from_offset(offset: UtcOffset) -> Self::OffsetType;
    }

    pub trait HasOffset: MaybeOffset<OffsetType = UtcOffset> {}
    impl<T: MaybeOffset<OffsetType = UtcOffset>> HasOffset for T {}

    pub trait WithoutOffset: MaybeOffset<OffsetType = ()> {}
    impl<T: MaybeOffset<OffsetType = ()>> WithoutOffset for T {}

    pub trait FixedOffset: MaybeOffset<OffsetKind = super::offset_kind::Fixed> {}
    impl<T: MaybeOffset<OffsetKind = super::offset_kind::Fixed>> FixedOffset for T {}
}

impl MaybeOffset for offset_kind::None {
    type OffsetType = ();
    type OffsetKind = Self;

    #[allow(clippy::undocumented_unsafe_blocks)] // false positive
    // SAFETY: `()` is not `UtcOffset`.
    const HAS_OFFSET: UnsafeBool = unsafe { UnsafeBool::new(false) };

    fn as_offset(_: ()) -> Option<UtcOffset> {
        None
    }

    #[cfg(feature = "parsing")]
    fn try_from_parsed(_: Parsed) -> Result<(), error::TryFromParsed> {
        Ok(())
    }

    #[cfg(feature = "parsing")]
    fn from_offset(_: UtcOffset) -> Self::OffsetType {}
}

impl MaybeOffset for offset_kind::Fixed {
    type OffsetType = UtcOffset;
    type OffsetKind = Self;

    #[allow(clippy::undocumented_unsafe_blocks)] // false positive
    // Safety: `UtcOffset` is the offset type.
    const HAS_OFFSET: UnsafeBool = unsafe { UnsafeBool::new(true) };

    fn as_offset(offset: UtcOffset) -> Option<UtcOffset> {
        Some(offset)
    }

    #[cfg(feature = "parsing")]
    fn try_from_parsed(parsed: Parsed) -> Result<UtcOffset, error::TryFromParsed> {
        parsed.try_into()
    }

    #[cfg(feature = "parsing")]
    fn from_offset(offset: UtcOffset) -> Self::OffsetType {
        offset
    }
}

const fn maybe_offset_as_offset<O: MaybeOffset>(offset: O::OffsetType) -> Option<UtcOffset> {
    if O::HAS_OFFSET.as_bool() {
        union Convert<O: MaybeOffset> {
            input: O::OffsetType,
            output: UtcOffset,
        }

        // SAFETY: `O::HAS_OFFSET` indicates that `O::Offset` is `UtcOffset`. This code effectively
        // performs a transmute from `O::Offset` to `UtcOffset`, which we know is the same type.
        Some(unsafe { Convert::<O> { input: offset }.output })
    } else {
        None
    }
}

/// The Julian day of the Unix epoch.
const UNIX_EPOCH_JULIAN_DAY: i32 = Date::__from_ordinal_date_unchecked(1970, 1).to_julian_day();

pub struct DateTime<O: MaybeOffset> {
    pub(crate) date: Date,
    pub(crate) time: Time,
    pub(crate) offset: O::OffsetType,
}

// Manual impl to remove extraneous bounds.
impl<O: MaybeOffset> Clone for DateTime<O> {
    fn clone(&self) -> Self {
        Self {
            date: self.date,
            time: self.time,
            offset: self.offset,
        }
    }
}

// Manual impl to remove extraneous bounds.
impl<O: MaybeOffset> Copy for DateTime<O> {}

// region: constructors
impl DateTime<offset_kind::None> {
    pub const MIN: Self = Self {
        date: Date::MIN,
        time: Time::MIN,
        offset: (),
    };

    pub const MAX: Self = Self {
        date: Date::MAX,
        time: Time::MAX,
        offset: (),
    };
}

impl DateTime<offset_kind::Fixed> {
    pub const UNIX_EPOCH: Self = Self {
        date: Date::__from_ordinal_date_unchecked(1970, 1),
        time: Time::MIDNIGHT,
        offset: UtcOffset::UTC,
    };
}

impl<O: MaybeOffset> DateTime<O> {
    pub const fn new(date: Date, time: Time) -> Self
    where
        O: WithoutOffset,
    {
        Self {
            date,
            time,
            offset: (),
        }
    }

    pub const fn from_unix_timestamp(timestamp: i64) -> Result<Self, error::ComponentRange>
    where
        O: HasOffset,
    {
        #[allow(clippy::missing_docs_in_private_items)]
        const MIN_TIMESTAMP: i64 = Date::MIN.midnight().assume_utc().unix_timestamp();
        #[allow(clippy::missing_docs_in_private_items)]
        const MAX_TIMESTAMP: i64 = Date::MAX
            .with_time(Time::__from_hms_nanos_unchecked(23, 59, 59, 999_999_999))
            .assume_utc()
            .unix_timestamp();

        ensure_value_in_range!(timestamp in MIN_TIMESTAMP => MAX_TIMESTAMP);

        // Use the unchecked method here, as the input validity has already been verified.
        let date = Date::from_julian_day_unchecked(
            UNIX_EPOCH_JULIAN_DAY + div_floor!(timestamp, 86_400) as i32,
        );

        let seconds_within_day = timestamp.rem_euclid(86_400);
        let time = Time::__from_hms_nanos_unchecked(
            (seconds_within_day / 3_600) as _,
            ((seconds_within_day % 3_600) / 60) as _,
            (seconds_within_day % 60) as _,
            0,
        );

        Ok(Self {
            date,
            time,
            offset: UtcOffset::UTC,
        })
    }

    pub const fn from_unix_timestamp_nanos(timestamp: i128) -> Result<Self, error::ComponentRange>
    where
        O: HasOffset,
    {
        let datetime = const_try!(Self::from_unix_timestamp(
            div_floor!(timestamp, 1_000_000_000) as i64
        ));

        Ok(Self {
            date: datetime.date,
            time: Time::__from_hms_nanos_unchecked(
                datetime.hour(),
                datetime.minute(),
                datetime.second(),
                timestamp.rem_euclid(1_000_000_000) as u32,
            ),
            offset: UtcOffset::UTC,
        })
    }
    // endregion constructors

    // region: now
    // The return type will likely be loosened once `ZonedDateTime` is implemented. This is not a
    // breaking change calls are currently limited to only `OffsetDateTime`.
    #[cfg(feature = "std")]
    pub fn now_utc() -> DateTime<offset_kind::Fixed>
    where
        O: FixedOffset,
    {
        #[cfg(all(
            target_family = "wasm",
            not(any(target_os = "emscripten", target_os = "wasi")),
            feature = "wasm-bindgen"
        ))]
        {
            js_sys::Date::new_0().into()
        }

        #[cfg(not(all(
            target_family = "wasm",
            not(any(target_os = "emscripten", target_os = "wasi")),
            feature = "wasm-bindgen"
        )))]
        SystemTime::now().into()
    }

    // The return type will likely be loosened once `ZonedDateTime` is implemented. This is not a
    // breaking change calls are currently limited to only `OffsetDateTime`.
    #[cfg(feature = "local-offset")]
    pub fn now_local() -> Result<DateTime<offset_kind::Fixed>, error::IndeterminateOffset>
    where
        O: FixedOffset,
    {
        let t = DateTime::<offset_kind::Fixed>::now_utc();
        Ok(t.to_offset(UtcOffset::local_offset_at(crate::OffsetDateTime(t))?))
    }
    // endregion now

    // region: getters
    // region: component getters
    pub const fn date(self) -> Date {
        self.date
    }

    pub const fn time(self) -> Time {
        self.time
    }

    pub const fn offset(self) -> UtcOffset
    where
        O: HasOffset,
    {
        self.offset
    }
    // endregion component getters

    // region: date getters
    pub const fn year(self) -> i32 {
        self.date.year()
    }

    pub const fn month(self) -> Month {
        self.date.month()
    }

    pub const fn day(self) -> u8 {
        self.date.day()
    }

    pub const fn ordinal(self) -> u16 {
        self.date.ordinal()
    }

    pub const fn iso_week(self) -> u8 {
        self.date.iso_week()
    }

    pub const fn sunday_based_week(self) -> u8 {
        self.date.sunday_based_week()
    }

    pub const fn monday_based_week(self) -> u8 {
        self.date.monday_based_week()
    }

    pub const fn to_calendar_date(self) -> (i32, Month, u8) {
        self.date.to_calendar_date()
    }

    pub const fn to_ordinal_date(self) -> (i32, u16) {
        self.date.to_ordinal_date()
    }

    pub const fn to_iso_week_date(self) -> (i32, u8, Weekday) {
        self.date.to_iso_week_date()
    }

    pub const fn weekday(self) -> Weekday {
        self.date.weekday()
    }

    pub const fn to_julian_day(self) -> i32 {
        self.date.to_julian_day()
    }
    // endregion date getters

    // region: time getters
    pub const fn as_hms(self) -> (u8, u8, u8) {
        self.time.as_hms()
    }

    pub const fn as_hms_milli(self) -> (u8, u8, u8, u16) {
        self.time.as_hms_milli()
    }

    pub const fn as_hms_micro(self) -> (u8, u8, u8, u32) {
        self.time.as_hms_micro()
    }

    pub const fn as_hms_nano(self) -> (u8, u8, u8, u32) {
        self.time.as_hms_nano()
    }

    pub const fn hour(self) -> u8 {
        self.time.hour()
    }

    pub const fn minute(self) -> u8 {
        self.time.minute()
    }

    pub const fn second(self) -> u8 {
        self.time.second()
    }

    pub const fn millisecond(self) -> u16 {
        self.time.millisecond()
    }

    pub const fn microsecond(self) -> u32 {
        self.time.microsecond()
    }

    pub const fn nanosecond(self) -> u32 {
        self.time.nanosecond()
    }
    // endregion time getters

    // region: unix timestamp getters
    pub const fn unix_timestamp(self) -> i64
    where
        O: HasOffset,
    {
        let offset = self.offset.whole_seconds() as i64;

        let days = (self.to_julian_day() as i64 - UNIX_EPOCH_JULIAN_DAY as i64) * 86_400;
        let hours = self.hour() as i64 * 3_600;
        let minutes = self.minute() as i64 * 60;
        let seconds = self.second() as i64;
        days + hours + minutes + seconds - offset
    }

    pub const fn unix_timestamp_nanos(self) -> i128
    where
        O: HasOffset,
    {
        self.unix_timestamp() as i128 * 1_000_000_000 + self.nanosecond() as i128
    }
    // endregion unix timestamp getters
    // endregion: getters

    // region: attach offset
    pub const fn assume_offset(self, offset: UtcOffset) -> DateTime<offset_kind::Fixed>
    where
        O: WithoutOffset,
    {
        DateTime {
            date: self.date,
            time: self.time,
            offset,
        }
    }

    pub const fn assume_utc(self) -> DateTime<offset_kind::Fixed>
    where
        O: WithoutOffset,
    {
        self.assume_offset(UtcOffset::UTC)
    }
    // endregion attach offset

    // region: to offset
    pub const fn to_offset(self, offset: UtcOffset) -> DateTime<offset_kind::Fixed>
    where
        O: HasOffset,
    {
        if self.offset.whole_hours() == offset.whole_hours()
            && self.offset.minutes_past_hour() == offset.minutes_past_hour()
            && self.offset.seconds_past_minute() == offset.seconds_past_minute()
        {
            return DateTime {
                date: self.date,
                time: self.time,
                offset,
            };
        }

        let (year, ordinal, time) = self.to_offset_raw(offset);

        if year > MAX_YEAR || year < MIN_YEAR {
            panic!("local datetime out of valid range");
        }

        DateTime {
            date: Date::__from_ordinal_date_unchecked(year, ordinal),
            time,
            offset,
        }
    }

    /// Equivalent to `.to_offset(UtcOffset::UTC)`, but returning the year, ordinal, and time. This
    /// avoids constructing an invalid [`Date`] if the new value is out of range.
    pub(crate) const fn to_offset_raw(self, offset: UtcOffset) -> (i32, u16, Time) {
        guard!(let Some(from) = maybe_offset_as_offset::<O>(self.offset) else {
            // No adjustment is needed because there is no offset.
            return (self.year(), self.ordinal(), self.time);
        });
        let to = offset;

        // Fast path for when no conversion is necessary.
        if from.whole_hours() == to.whole_hours()
            && from.minutes_past_hour() == to.minutes_past_hour()
            && from.seconds_past_minute() == to.seconds_past_minute()
        {
            return (self.year(), self.ordinal(), self.time());
        }

        let mut second = self.second() as i16 - from.seconds_past_minute() as i16
            + to.seconds_past_minute() as i16;
        let mut minute =
            self.minute() as i16 - from.minutes_past_hour() as i16 + to.minutes_past_hour() as i16;
        let mut hour = self.hour() as i8 - from.whole_hours() + to.whole_hours();
        let (mut year, ordinal) = self.to_ordinal_date();
        let mut ordinal = ordinal as i16;

        // Cascade the values twice. This is needed because the values are adjusted twice above.
        cascade!(second in 0..60 => minute);
        cascade!(second in 0..60 => minute);
        cascade!(minute in 0..60 => hour);
        cascade!(minute in 0..60 => hour);
        cascade!(hour in 0..24 => ordinal);
        cascade!(hour in 0..24 => ordinal);
        cascade!(ordinal => year);

        debug_assert!(ordinal > 0);
        debug_assert!(ordinal <= crate::util::days_in_year(year) as i16);

        (
            year,
            ordinal as _,
            Time::__from_hms_nanos_unchecked(
                hour as _,
                minute as _,
                second as _,
                self.nanosecond(),
            ),
        )
    }
    // endregion to offset

    // region: checked arithmetic
    pub const fn checked_add(self, duration: Duration) -> Option<Self> {
        let (date_adjustment, time) = self.time.adjusting_add(duration);
        let date = const_try_opt!(self.date.checked_add(duration));

        Some(Self {
            date: match date_adjustment {
                util::DateAdjustment::Previous => const_try_opt!(date.previous_day()),
                util::DateAdjustment::Next => const_try_opt!(date.next_day()),
                util::DateAdjustment::None => date,
            },
            time,
            offset: self.offset,
        })
    }

    pub const fn checked_sub(self, duration: Duration) -> Option<Self> {
        let (date_adjustment, time) = self.time.adjusting_sub(duration);
        let date = const_try_opt!(self.date.checked_sub(duration));

        Some(Self {
            date: match date_adjustment {
                util::DateAdjustment::Previous => const_try_opt!(date.previous_day()),
                util::DateAdjustment::Next => const_try_opt!(date.next_day()),
                util::DateAdjustment::None => date,
            },
            time,
            offset: self.offset,
        })
    }
    // endregion checked arithmetic

    // region: saturating arithmetic
    pub const fn saturating_add(self, duration: Duration) -> Self {
        if let Some(datetime) = self.checked_add(duration) {
            datetime
        } else if duration.is_negative() {
            Self {
                date: Date::MIN,
                time: Time::MIN,
                offset: self.offset,
            }
        } else {
            Self {
                date: Date::MAX,
                time: Time::MAX,
                offset: self.offset,
            }
        }
    }

    pub const fn saturating_sub(self, duration: Duration) -> Self {
        if let Some(datetime) = self.checked_sub(duration) {
            datetime
        } else if duration.is_negative() {
            Self {
                date: Date::MAX,
                time: Time::MAX,
                offset: self.offset,
            }
        } else {
            Self {
                date: Date::MIN,
                time: Time::MIN,
                offset: self.offset,
            }
        }
    }
    // endregion saturating arithmetic

    // region: replacement
    pub const fn replace_time(self, time: Time) -> Self {
        Self {
            date: self.date,
            time,
            offset: self.offset,
        }
    }

    pub const fn replace_date(self, date: Date) -> Self {
        Self {
            date,
            time: self.time,
            offset: self.offset,
        }
    }

    pub const fn replace_date_time(self, date_time: DateTime<offset_kind::None>) -> Self
    where
        O: HasOffset,
    {
        Self {
            date: date_time.date,
            time: date_time.time,
            offset: self.offset,
        }
    }

    pub const fn replace_year(self, year: i32) -> Result<Self, error::ComponentRange> {
        Ok(Self {
            date: const_try!(self.date.replace_year(year)),
            time: self.time,
            offset: self.offset,
        })
    }

    pub const fn replace_month(self, month: Month) -> Result<Self, error::ComponentRange> {
        Ok(Self {
            date: const_try!(self.date.replace_month(month)),
            time: self.time,
            offset: self.offset,
        })
    }

    pub const fn replace_day(self, day: u8) -> Result<Self, error::ComponentRange> {
        Ok(Self {
            date: const_try!(self.date.replace_day(day)),
            time: self.time,
            offset: self.offset,
        })
    }

    pub const fn replace_hour(self, hour: u8) -> Result<Self, error::ComponentRange> {
        Ok(Self {
            date: self.date,
            time: const_try!(self.time.replace_hour(hour)),
            offset: self.offset,
        })
    }

    pub const fn replace_minute(self, minute: u8) -> Result<Self, error::ComponentRange> {
        Ok(Self {
            date: self.date,
            time: const_try!(self.time.replace_minute(minute)),
            offset: self.offset,
        })
    }

    pub const fn replace_second(self, second: u8) -> Result<Self, error::ComponentRange> {
        Ok(Self {
            date: self.date,
            time: const_try!(self.time.replace_second(second)),
            offset: self.offset,
        })
    }

    pub const fn replace_millisecond(
        self,
        millisecond: u16,
    ) -> Result<Self, error::ComponentRange> {
        Ok(Self {
            date: self.date,
            time: const_try!(self.time.replace_millisecond(millisecond)),
            offset: self.offset,
        })
    }

    pub const fn replace_microsecond(
        self,
        microsecond: u32,
    ) -> Result<Self, error::ComponentRange> {
        Ok(Self {
            date: self.date,
            time: const_try!(self.time.replace_microsecond(microsecond)),
            offset: self.offset,
        })
    }

    pub const fn replace_nanosecond(self, nanosecond: u32) -> Result<Self, error::ComponentRange> {
        Ok(Self {
            date: self.date,
            time: const_try!(self.time.replace_nanosecond(nanosecond)),
            offset: self.offset,
        })
    }

    // Don't gate this on just having an offset, as `ZonedDateTime` cannot be set to an arbitrary
    // offset.
    pub const fn replace_offset(self, offset: UtcOffset) -> DateTime<offset_kind::Fixed>
    where
        O: FixedOffset,
    {
        DateTime {
            date: self.date,
            time: self.time,
            offset,
        }
    }

    // endregion replacement

    // region: formatting & parsing
    #[cfg(feature = "formatting")]
    pub fn format_into(
        self,
        output: &mut impl io::Write,
        format: &(impl Formattable + ?Sized),
    ) -> Result<usize, error::Format> {
        format.format_into(
            output,
            Some(self.date),
            Some(self.time),
            O::as_offset(self.offset),
        )
    }

    #[cfg(feature = "formatting")]
    pub fn format(self, format: &(impl Formattable + ?Sized)) -> Result<String, error::Format> {
        format.format(Some(self.date), Some(self.time), O::as_offset(self.offset))
    }

    #[cfg(feature = "parsing")]
    pub fn parse(
        input: &str,
        description: &(impl Parsable + ?Sized),
    ) -> Result<Self, error::Parse> {
        description.parse_date_time(input.as_bytes())
    }

    /// A helper method to check if the `OffsetDateTime` is a valid representation of a leap second.
    /// Leap seconds, when parsed, are represented as the preceding nanosecond. However, leap
    /// seconds can only occur as the last second of a month UTC.
    #[cfg(feature = "parsing")]
    pub(crate) const fn is_valid_leap_second_stand_in(self) -> bool {
        // Leap seconds aren't allowed if there is no offset.
        if !O::HAS_OFFSET.as_bool() {
            return false;
        }

        // This comparison doesn't need to be adjusted for the stored offset, so check it first for
        // speed.
        if self.nanosecond() != 999_999_999 {
            return false;
        }

        let (year, ordinal, time) = self.to_offset_raw(UtcOffset::UTC);
        guard!(let Ok(date) = Date::from_ordinal_date(year, ordinal) else { return false });

        time.hour() == 23
            && time.minute() == 59
            && time.second() == 59
            && date.day() == util::days_in_year_month(year, date.month())
    }

    // endregion formatting & parsing

    // region: deprecated time getters

    // All the way at the bottom as it's low priority. These methods only exist for when
    // `OffsetDateTime` is made an alias of `DateTime<Fixed>`. Consider hiding these methods from
    // documentation in the future.

    #[allow(dead_code)] // while functionally private
    #[deprecated(since = "0.3.18", note = "use `as_hms` instead")]
    pub const fn to_hms(self) -> (u8, u8, u8)
    where
        O: FixedOffset,
    {
        self.time.as_hms()
    }

    #[allow(dead_code)] // while functionally private
    #[deprecated(since = "0.3.18", note = "use `as_hms_milli` instead")]
    pub const fn to_hms_milli(self) -> (u8, u8, u8, u16)
    where
        O: FixedOffset,
    {
        self.time.as_hms_milli()
    }

    #[allow(dead_code)] // while functionally private
    #[deprecated(since = "0.3.18", note = "use `as_hms_micro` instead")]
    pub const fn to_hms_micro(self) -> (u8, u8, u8, u32)
    where
        O: FixedOffset,
    {
        self.time.as_hms_micro()
    }

    #[allow(dead_code)] // while functionally private
    #[deprecated(since = "0.3.18", note = "use `as_hms_nano` instead")]
    pub const fn to_hms_nano(self) -> (u8, u8, u8, u32)
    where
        O: FixedOffset,
    {
        self.time.as_hms_nano()
    }
    // endregion deprecated time getters
}

impl<O: MaybeOffset> fmt::Debug for DateTime<O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

impl<O: MaybeOffset> fmt::Display for DateTime<O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.date, self.time)?;
        if let Some(offset) = O::as_offset(self.offset) {
            write!(f, " {}", offset)?;
        }
        Ok(())
    }
}

// region: trait impls
impl<O: MaybeOffset> PartialEq for DateTime<O> {
    fn eq(&self, rhs: &Self) -> bool {
        if O::HAS_OFFSET.as_bool() {
            self.to_offset_raw(UtcOffset::UTC) == rhs.to_offset_raw(UtcOffset::UTC)
        } else {
            (self.date, self.time) == (rhs.date, rhs.time)
        }
    }
}

impl<O: MaybeOffset> Eq for DateTime<O> {}

impl<O: MaybeOffset> PartialOrd for DateTime<O> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl<O: MaybeOffset> Ord for DateTime<O> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        if O::HAS_OFFSET.as_bool() {
            self.to_offset_raw(UtcOffset::UTC)
                .cmp(&rhs.to_offset_raw(UtcOffset::UTC))
        } else {
            (self.date, self.time).cmp(&(rhs.date, rhs.time))
        }
    }
}

impl<O: MaybeOffset> Hash for DateTime<O> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        if O::HAS_OFFSET.as_bool() {
            self.to_offset_raw(UtcOffset::UTC).hash(hasher);
        } else {
            (self.date, self.time).hash(hasher);
        }
    }
}

impl<O: MaybeOffset> Add<Duration> for DateTime<O> {
    type Output = Self;

    fn add(self, duration: Duration) -> Self {
        self.checked_add(duration)
            .expect("resulting value is out of range")
    }
}

impl<O: MaybeOffset> Add<StdDuration> for DateTime<O> {
    type Output = Self;

    fn add(self, duration: StdDuration) -> Self::Output {
        let (is_next_day, time) = self.time.adjusting_add_std(duration);

        Self {
            date: if is_next_day {
                (self.date + duration)
                    .next_day()
                    .expect("resulting value is out of range")
            } else {
                self.date + duration
            },
            time,
            offset: self.offset,
        }
    }
}

impl<O: MaybeOffset> AddAssign<Duration> for DateTime<O> {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl<O: MaybeOffset> AddAssign<StdDuration> for DateTime<O> {
    fn add_assign(&mut self, rhs: StdDuration) {
        *self = *self + rhs;
    }
}

impl<O: MaybeOffset> Sub<Duration> for DateTime<O> {
    type Output = Self;

    fn sub(self, duration: Duration) -> Self {
        self.checked_sub(duration)
            .expect("resulting value is out of range")
    }
}

impl<O: MaybeOffset> Sub<StdDuration> for DateTime<O> {
    type Output = Self;

    fn sub(self, duration: StdDuration) -> Self::Output {
        let (is_previous_day, time) = self.time.adjusting_sub_std(duration);

        Self {
            date: if is_previous_day {
                (self.date - duration)
                    .previous_day()
                    .expect("resulting value is out of range")
            } else {
                self.date - duration
            },
            time,
            offset: self.offset,
        }
    }
}

impl<O: MaybeOffset> SubAssign<Duration> for DateTime<O> {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

impl<O: MaybeOffset> SubAssign<StdDuration> for DateTime<O> {
    fn sub_assign(&mut self, rhs: StdDuration) {
        *self = *self - rhs;
    }
}

impl<O: MaybeOffset> Sub<Self> for DateTime<O> {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        let base = (self.date - rhs.date) + (self.time - rhs.time);

        match (O::as_offset(self.offset), O::as_offset(rhs.offset)) {
            (Some(self_offset), Some(rhs_offset)) => {
                let adjustment = Duration::seconds(
                    (self_offset.whole_seconds() - rhs_offset.whole_seconds()) as i64,
                );
                base - adjustment
            }
            (left, right) => {
                debug_assert!(
                    left.is_none() && right.is_none(),
                    "offset type should not be different for the same type"
                );
                base
            }
        }
    }
}

#[cfg(feature = "std")]
impl Add<Duration> for SystemTime {
    type Output = Self;

    fn add(self, duration: Duration) -> Self::Output {
        if duration.is_zero() {
            self
        } else if duration.is_positive() {
            self + duration.unsigned_abs()
        } else {
            debug_assert!(duration.is_negative());
            self - duration.unsigned_abs()
        }
    }
}

impl_add_assign!(SystemTime: #[cfg(feature = "std")] Duration);

#[cfg(feature = "std")]
impl Sub<Duration> for SystemTime {
    type Output = Self;

    fn sub(self, duration: Duration) -> Self::Output {
        (DateTime::from(self) - duration).into()
    }
}

impl_sub_assign!(SystemTime: #[cfg(feature = "std")] Duration);

#[cfg(feature = "std")]
impl Sub<SystemTime> for DateTime<offset_kind::Fixed> {
    type Output = Duration;

    fn sub(self, rhs: SystemTime) -> Self::Output {
        self - Self::from(rhs)
    }
}

#[cfg(feature = "std")]
impl Sub<DateTime<offset_kind::Fixed>> for SystemTime {
    type Output = Duration;

    fn sub(self, rhs: DateTime<offset_kind::Fixed>) -> Self::Output {
        DateTime::<offset_kind::Fixed>::from(self) - rhs
    }
}

#[cfg(feature = "std")]
impl PartialEq<SystemTime> for DateTime<offset_kind::Fixed> {
    fn eq(&self, rhs: &SystemTime) -> bool {
        self == &Self::from(*rhs)
    }
}

#[cfg(feature = "std")]
impl PartialEq<DateTime<offset_kind::Fixed>> for SystemTime {
    fn eq(&self, rhs: &DateTime<offset_kind::Fixed>) -> bool {
        &DateTime::<offset_kind::Fixed>::from(*self) == rhs
    }
}

#[cfg(feature = "std")]
impl PartialOrd<SystemTime> for DateTime<offset_kind::Fixed> {
    fn partial_cmp(&self, other: &SystemTime) -> Option<Ordering> {
        self.partial_cmp(&Self::from(*other))
    }
}

#[cfg(feature = "std")]
impl PartialOrd<DateTime<offset_kind::Fixed>> for SystemTime {
    fn partial_cmp(&self, other: &DateTime<offset_kind::Fixed>) -> Option<Ordering> {
        DateTime::<offset_kind::Fixed>::from(*self).partial_cmp(other)
    }
}

#[cfg(feature = "std")]
impl From<SystemTime> for DateTime<offset_kind::Fixed> {
    fn from(system_time: SystemTime) -> Self {
        match system_time.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => Self::UNIX_EPOCH + duration,
            Err(err) => Self::UNIX_EPOCH - err.duration(),
        }
    }
}

#[allow(clippy::fallible_impl_from)] // caused by `debug_assert!`
#[cfg(feature = "std")]
impl From<DateTime<offset_kind::Fixed>> for SystemTime {
    fn from(datetime: DateTime<offset_kind::Fixed>) -> Self {
        let duration = datetime - DateTime::<offset_kind::Fixed>::UNIX_EPOCH;

        if duration.is_zero() {
            Self::UNIX_EPOCH
        } else if duration.is_positive() {
            Self::UNIX_EPOCH + duration.unsigned_abs()
        } else {
            debug_assert!(duration.is_negative());
            Self::UNIX_EPOCH - duration.unsigned_abs()
        }
    }
}

#[allow(clippy::fallible_impl_from)]
#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm-bindgen"
))]
impl From<js_sys::Date> for DateTime<offset_kind::Fixed> {
    fn from(js_date: js_sys::Date) -> Self {
        // get_time() returns milliseconds
        let timestamp_nanos = (js_date.get_time() * 1_000_000.0) as i128;
        Self::from_unix_timestamp_nanos(timestamp_nanos)
            .expect("invalid timestamp: Timestamp cannot fit in range")
    }
}

#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm-bindgen"
))]
impl From<DateTime<offset_kind::Fixed>> for js_sys::Date {
    fn from(datetime: DateTime<offset_kind::Fixed>) -> Self {
        // new Date() takes milliseconds
        let timestamp = (datetime.unix_timestamp_nanos() / 1_000_000) as f64;
        js_sys::Date::new(&timestamp.into())
    }
}
// endregion trait impls
