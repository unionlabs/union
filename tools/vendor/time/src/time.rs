//! The [`Time`] struct and its associated `impl`s.

use core::fmt;
use core::ops::{Add, Sub};
use core::time::Duration as StdDuration;
#[cfg(feature = "formatting")]
use std::io;

use deranged::{RangedU32, RangedU8};

use crate::convert::*;
#[cfg(feature = "formatting")]
use crate::formatting::Formattable;
#[cfg(feature = "parsing")]
use crate::parsing::Parsable;
use crate::util::DateAdjustment;
use crate::{error, Duration};

/// By explicitly inserting this enum where padding is expected, the compiler is able to better
/// perform niche value optimization.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Padding {
    #[allow(clippy::missing_docs_in_private_items)]
    Optimize,
}

/// The type of the `hour` field of `Time`.
type Hours = RangedU8<0, { Hour.per(Day) - 1 }>;
/// The type of the `minute` field of `Time`.
type Minutes = RangedU8<0, { Minute.per(Hour) - 1 }>;
/// The type of the `second` field of `Time`.
type Seconds = RangedU8<0, { Second.per(Minute) - 1 }>;
/// The type of the `nanosecond` field of `Time`.
type Nanoseconds = RangedU32<0, { Nanosecond.per(Second) - 1 }>;

/// The clock time within a given date. Nanosecond precision.
///
/// All minutes are assumed to have exactly 60 seconds; no attempt is made to handle leap seconds
/// (either positive or negative).
///
/// When comparing two `Time`s, they are assumed to be in the same calendar date.
#[derive(Clone, Copy, Eq)]
#[repr(C)]
pub struct Time {
    // The order of this struct's fields matter!
    // Do not change them.

    // Little endian version
    #[cfg(target_endian = "little")]
    #[allow(clippy::missing_docs_in_private_items)]
    nanosecond: Nanoseconds,
    #[cfg(target_endian = "little")]
    #[allow(clippy::missing_docs_in_private_items)]
    second: Seconds,
    #[cfg(target_endian = "little")]
    #[allow(clippy::missing_docs_in_private_items)]
    minute: Minutes,
    #[cfg(target_endian = "little")]
    #[allow(clippy::missing_docs_in_private_items)]
    hour: Hours,
    #[cfg(target_endian = "little")]
    #[allow(clippy::missing_docs_in_private_items)]
    padding: Padding,

    // Big endian version
    #[cfg(target_endian = "big")]
    #[allow(clippy::missing_docs_in_private_items)]
    padding: Padding,
    #[cfg(target_endian = "big")]
    #[allow(clippy::missing_docs_in_private_items)]
    hour: Hours,
    #[cfg(target_endian = "big")]
    #[allow(clippy::missing_docs_in_private_items)]
    minute: Minutes,
    #[cfg(target_endian = "big")]
    #[allow(clippy::missing_docs_in_private_items)]
    second: Seconds,
    #[cfg(target_endian = "big")]
    #[allow(clippy::missing_docs_in_private_items)]
    nanosecond: Nanoseconds,
}

impl core::hash::Hash for Time {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_u64().hash(state)
    }
}

impl PartialEq for Time {
    fn eq(&self, other: &Self) -> bool {
        self.as_u64().eq(&other.as_u64())
    }
}

impl PartialOrd for Time {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.as_u64().partial_cmp(&other.as_u64())
    }
}

impl Ord for Time {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_u64().cmp(&other.as_u64())
    }
}

impl Time {
    /// Provides an u64 based representation **of the correct endianness**
    ///
    /// This representation can be used to do comparisons equality testing or hashing.
    const fn as_u64(self) -> u64 {
        let nano_bytes = self.nanosecond.get().to_ne_bytes();

        #[cfg(target_endian = "big")]
        return u64::from_be_bytes([
            self.padding as u8,
            self.hour.get(),
            self.minute.get(),
            self.second.get(),
            nano_bytes[0],
            nano_bytes[1],
            nano_bytes[2],
            nano_bytes[3],
        ]);

        #[cfg(target_endian = "little")]
        return u64::from_le_bytes([
            nano_bytes[0],
            nano_bytes[1],
            nano_bytes[2],
            nano_bytes[3],
            self.second.get(),
            self.minute.get(),
            self.hour.get(),
            self.padding as u8,
        ]);
    }

    /// Create a `Time` that is exactly midnight.
    ///
    /// ```rust
    /// # use time::Time;
    /// # use time_macros::time;
    /// assert_eq!(Time::MIDNIGHT, time!(0:00));
    /// ```
    pub const MIDNIGHT: Self = Self::MIN;

    /// The smallest value that can be represented by `Time`.
    ///
    /// `00:00:00.0`
    pub(crate) const MIN: Self =
        Self::from_hms_nanos_ranged(Hours::MIN, Minutes::MIN, Seconds::MIN, Nanoseconds::MIN);

    /// The largest value that can be represented by `Time`.
    ///
    /// `23:59:59.999_999_999`
    pub(crate) const MAX: Self =
        Self::from_hms_nanos_ranged(Hours::MAX, Minutes::MAX, Seconds::MAX, Nanoseconds::MAX);

    // region: constructors
    /// Create a `Time` from its components.
    ///
    /// # Safety
    ///
    /// - `hours` must be in the range `0..=23`.
    /// - `minutes` must be in the range `0..=59`.
    /// - `seconds` must be in the range `0..=59`.
    /// - `nanoseconds` must be in the range `0..=999_999_999`.
    #[doc(hidden)]
    pub const unsafe fn __from_hms_nanos_unchecked(
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
    ) -> Self {
        // Safety: The caller must uphold the safety invariants.
        unsafe {
            Self::from_hms_nanos_ranged(
                Hours::new_unchecked(hour),
                Minutes::new_unchecked(minute),
                Seconds::new_unchecked(second),
                Nanoseconds::new_unchecked(nanosecond),
            )
        }
    }

    /// Attempt to create a `Time` from the hour, minute, and second.
    ///
    /// ```rust
    /// # use time::Time;
    /// assert!(Time::from_hms(1, 2, 3).is_ok());
    /// ```
    ///
    /// ```rust
    /// # use time::Time;
    /// assert!(Time::from_hms(24, 0, 0).is_err()); // 24 isn't a valid hour.
    /// assert!(Time::from_hms(0, 60, 0).is_err()); // 60 isn't a valid minute.
    /// assert!(Time::from_hms(0, 0, 60).is_err()); // 60 isn't a valid second.
    /// ```
    pub const fn from_hms(hour: u8, minute: u8, second: u8) -> Result<Self, error::ComponentRange> {
        Ok(Self::from_hms_nanos_ranged(
            ensure_ranged!(Hours: hour),
            ensure_ranged!(Minutes: minute),
            ensure_ranged!(Seconds: second),
            Nanoseconds::MIN,
        ))
    }

    /// Create a `Time` from the hour, minute, second, and nanosecond.
    pub(crate) const fn from_hms_nanos_ranged(
        hour: Hours,
        minute: Minutes,
        second: Seconds,
        nanosecond: Nanoseconds,
    ) -> Self {
        Self {
            hour,
            minute,
            second,
            nanosecond,
            padding: Padding::Optimize,
        }
    }

    /// Attempt to create a `Time` from the hour, minute, second, and millisecond.
    ///
    /// ```rust
    /// # use time::Time;
    /// assert!(Time::from_hms_milli(1, 2, 3, 4).is_ok());
    /// ```
    ///
    /// ```rust
    /// # use time::Time;
    /// assert!(Time::from_hms_milli(24, 0, 0, 0).is_err()); // 24 isn't a valid hour.
    /// assert!(Time::from_hms_milli(0, 60, 0, 0).is_err()); // 60 isn't a valid minute.
    /// assert!(Time::from_hms_milli(0, 0, 60, 0).is_err()); // 60 isn't a valid second.
    /// assert!(Time::from_hms_milli(0, 0, 0, 1_000).is_err()); // 1_000 isn't a valid millisecond.
    /// ```
    pub const fn from_hms_milli(
        hour: u8,
        minute: u8,
        second: u8,
        millisecond: u16,
    ) -> Result<Self, error::ComponentRange> {
        Ok(Self::from_hms_nanos_ranged(
            ensure_ranged!(Hours: hour),
            ensure_ranged!(Minutes: minute),
            ensure_ranged!(Seconds: second),
            ensure_ranged!(Nanoseconds: millisecond as u32 * Nanosecond.per(Millisecond)),
        ))
    }

    /// Attempt to create a `Time` from the hour, minute, second, and microsecond.
    ///
    /// ```rust
    /// # use time::Time;
    /// assert!(Time::from_hms_micro(1, 2, 3, 4).is_ok());
    /// ```
    ///
    /// ```rust
    /// # use time::Time;
    /// assert!(Time::from_hms_micro(24, 0, 0, 0).is_err()); // 24 isn't a valid hour.
    /// assert!(Time::from_hms_micro(0, 60, 0, 0).is_err()); // 60 isn't a valid minute.
    /// assert!(Time::from_hms_micro(0, 0, 60, 0).is_err()); // 60 isn't a valid second.
    /// assert!(Time::from_hms_micro(0, 0, 0, 1_000_000).is_err()); // 1_000_000 isn't a valid microsecond.
    /// ```
    pub const fn from_hms_micro(
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
    ) -> Result<Self, error::ComponentRange> {
        Ok(Self::from_hms_nanos_ranged(
            ensure_ranged!(Hours: hour),
            ensure_ranged!(Minutes: minute),
            ensure_ranged!(Seconds: second),
            ensure_ranged!(Nanoseconds: microsecond * Nanosecond.per(Microsecond) as u32),
        ))
    }

    /// Attempt to create a `Time` from the hour, minute, second, and nanosecond.
    ///
    /// ```rust
    /// # use time::Time;
    /// assert!(Time::from_hms_nano(1, 2, 3, 4).is_ok());
    /// ```
    ///
    /// ```rust
    /// # use time::Time;
    /// assert!(Time::from_hms_nano(24, 0, 0, 0).is_err()); // 24 isn't a valid hour.
    /// assert!(Time::from_hms_nano(0, 60, 0, 0).is_err()); // 60 isn't a valid minute.
    /// assert!(Time::from_hms_nano(0, 0, 60, 0).is_err()); // 60 isn't a valid second.
    /// assert!(Time::from_hms_nano(0, 0, 0, 1_000_000_000).is_err()); // 1_000_000_000 isn't a valid nanosecond.
    /// ```
    pub const fn from_hms_nano(
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
    ) -> Result<Self, error::ComponentRange> {
        Ok(Self::from_hms_nanos_ranged(
            ensure_ranged!(Hours: hour),
            ensure_ranged!(Minutes: minute),
            ensure_ranged!(Seconds: second),
            ensure_ranged!(Nanoseconds: nanosecond),
        ))
    }
    // endregion constructors

    // region: getters
    /// Get the clock hour, minute, and second.
    ///
    /// ```rust
    /// # use time_macros::time;
    /// assert_eq!(time!(0:00:00).as_hms(), (0, 0, 0));
    /// assert_eq!(time!(23:59:59).as_hms(), (23, 59, 59));
    /// ```
    pub const fn as_hms(self) -> (u8, u8, u8) {
        (self.hour.get(), self.minute.get(), self.second.get())
    }

    /// Get the clock hour, minute, second, and millisecond.
    ///
    /// ```rust
    /// # use time_macros::time;
    /// assert_eq!(time!(0:00:00).as_hms_milli(), (0, 0, 0, 0));
    /// assert_eq!(time!(23:59:59.999).as_hms_milli(), (23, 59, 59, 999));
    /// ```
    pub const fn as_hms_milli(self) -> (u8, u8, u8, u16) {
        (
            self.hour.get(),
            self.minute.get(),
            self.second.get(),
            (self.nanosecond.get() / Nanosecond.per(Millisecond)) as u16,
        )
    }

    /// Get the clock hour, minute, second, and microsecond.
    ///
    /// ```rust
    /// # use time_macros::time;
    /// assert_eq!(time!(0:00:00).as_hms_micro(), (0, 0, 0, 0));
    /// assert_eq!(
    ///     time!(23:59:59.999_999).as_hms_micro(),
    ///     (23, 59, 59, 999_999)
    /// );
    /// ```
    pub const fn as_hms_micro(self) -> (u8, u8, u8, u32) {
        (
            self.hour.get(),
            self.minute.get(),
            self.second.get(),
            self.nanosecond.get() / Nanosecond.per(Microsecond) as u32,
        )
    }

    /// Get the clock hour, minute, second, and nanosecond.
    ///
    /// ```rust
    /// # use time_macros::time;
    /// assert_eq!(time!(0:00:00).as_hms_nano(), (0, 0, 0, 0));
    /// assert_eq!(
    ///     time!(23:59:59.999_999_999).as_hms_nano(),
    ///     (23, 59, 59, 999_999_999)
    /// );
    /// ```
    pub const fn as_hms_nano(self) -> (u8, u8, u8, u32) {
        (
            self.hour.get(),
            self.minute.get(),
            self.second.get(),
            self.nanosecond.get(),
        )
    }

    /// Get the clock hour, minute, second, and nanosecond.
    #[cfg(feature = "quickcheck")]
    pub(crate) const fn as_hms_nano_ranged(self) -> (Hours, Minutes, Seconds, Nanoseconds) {
        (self.hour, self.minute, self.second, self.nanosecond)
    }

    /// Get the clock hour.
    ///
    /// The returned value will always be in the range `0..24`.
    ///
    /// ```rust
    /// # use time_macros::time;
    /// assert_eq!(time!(0:00:00).hour(), 0);
    /// assert_eq!(time!(23:59:59).hour(), 23);
    /// ```
    pub const fn hour(self) -> u8 {
        self.hour.get()
    }

    /// Get the minute within the hour.
    ///
    /// The returned value will always be in the range `0..60`.
    ///
    /// ```rust
    /// # use time_macros::time;
    /// assert_eq!(time!(0:00:00).minute(), 0);
    /// assert_eq!(time!(23:59:59).minute(), 59);
    /// ```
    pub const fn minute(self) -> u8 {
        self.minute.get()
    }

    /// Get the second within the minute.
    ///
    /// The returned value will always be in the range `0..60`.
    ///
    /// ```rust
    /// # use time_macros::time;
    /// assert_eq!(time!(0:00:00).second(), 0);
    /// assert_eq!(time!(23:59:59).second(), 59);
    /// ```
    pub const fn second(self) -> u8 {
        self.second.get()
    }

    /// Get the milliseconds within the second.
    ///
    /// The returned value will always be in the range `0..1_000`.
    ///
    /// ```rust
    /// # use time_macros::time;
    /// assert_eq!(time!(0:00).millisecond(), 0);
    /// assert_eq!(time!(23:59:59.999).millisecond(), 999);
    /// ```
    pub const fn millisecond(self) -> u16 {
        (self.nanosecond.get() / Nanosecond.per(Millisecond)) as _
    }

    /// Get the microseconds within the second.
    ///
    /// The returned value will always be in the range `0..1_000_000`.
    ///
    /// ```rust
    /// # use time_macros::time;
    /// assert_eq!(time!(0:00).microsecond(), 0);
    /// assert_eq!(time!(23:59:59.999_999).microsecond(), 999_999);
    /// ```
    pub const fn microsecond(self) -> u32 {
        self.nanosecond.get() / Nanosecond.per(Microsecond) as u32
    }

    /// Get the nanoseconds within the second.
    ///
    /// The returned value will always be in the range `0..1_000_000_000`.
    ///
    /// ```rust
    /// # use time_macros::time;
    /// assert_eq!(time!(0:00).nanosecond(), 0);
    /// assert_eq!(time!(23:59:59.999_999_999).nanosecond(), 999_999_999);
    /// ```
    pub const fn nanosecond(self) -> u32 {
        self.nanosecond.get()
    }
    // endregion getters

    // region: arithmetic helpers
    /// Add the sub-day time of the [`Duration`] to the `Time`. Wraps on overflow, returning whether
    /// the date is different.
    pub(crate) const fn adjusting_add(self, duration: Duration) -> (DateAdjustment, Self) {
        let mut nanoseconds = self.nanosecond.get() as i32 + duration.subsec_nanoseconds();
        let mut seconds =
            self.second.get() as i8 + (duration.whole_seconds() % Second.per(Minute) as i64) as i8;
        let mut minutes =
            self.minute.get() as i8 + (duration.whole_minutes() % Minute.per(Hour) as i64) as i8;
        let mut hours =
            self.hour.get() as i8 + (duration.whole_hours() % Hour.per(Day) as i64) as i8;
        let mut date_adjustment = DateAdjustment::None;

        cascade!(nanoseconds in 0..Nanosecond.per(Second) as _ => seconds);
        cascade!(seconds in 0..Second.per(Minute) as _ => minutes);
        cascade!(minutes in 0..Minute.per(Hour) as _ => hours);
        if hours >= Hour.per(Day) as _ {
            hours -= Hour.per(Day) as i8;
            date_adjustment = DateAdjustment::Next;
        } else if hours < 0 {
            hours += Hour.per(Day) as i8;
            date_adjustment = DateAdjustment::Previous;
        }

        (
            date_adjustment,
            // Safety: The cascades above ensure the values are in range.
            unsafe {
                Self::__from_hms_nanos_unchecked(
                    hours as _,
                    minutes as _,
                    seconds as _,
                    nanoseconds as _,
                )
            },
        )
    }

    /// Subtract the sub-day time of the [`Duration`] to the `Time`. Wraps on overflow, returning
    /// whether the date is different.
    pub(crate) const fn adjusting_sub(self, duration: Duration) -> (DateAdjustment, Self) {
        let mut nanoseconds = self.nanosecond.get() as i32 - duration.subsec_nanoseconds();
        let mut seconds =
            self.second.get() as i8 - (duration.whole_seconds() % Second.per(Minute) as i64) as i8;
        let mut minutes =
            self.minute.get() as i8 - (duration.whole_minutes() % Minute.per(Hour) as i64) as i8;
        let mut hours =
            self.hour.get() as i8 - (duration.whole_hours() % Hour.per(Day) as i64) as i8;
        let mut date_adjustment = DateAdjustment::None;

        cascade!(nanoseconds in 0..Nanosecond.per(Second) as _ => seconds);
        cascade!(seconds in 0..Second.per(Minute) as _ => minutes);
        cascade!(minutes in 0..Minute.per(Hour) as _ => hours);
        if hours >= Hour.per(Day) as _ {
            hours -= Hour.per(Day) as i8;
            date_adjustment = DateAdjustment::Next;
        } else if hours < 0 {
            hours += Hour.per(Day) as i8;
            date_adjustment = DateAdjustment::Previous;
        }

        (
            date_adjustment,
            // Safety: The cascades above ensure the values are in range.
            unsafe {
                Self::__from_hms_nanos_unchecked(
                    hours as _,
                    minutes as _,
                    seconds as _,
                    nanoseconds as _,
                )
            },
        )
    }

    /// Add the sub-day time of the [`std::time::Duration`] to the `Time`. Wraps on overflow,
    /// returning whether the date is the previous date as the first element of the tuple.
    pub(crate) const fn adjusting_add_std(self, duration: StdDuration) -> (bool, Self) {
        let mut nanosecond = self.nanosecond.get() + duration.subsec_nanos();
        let mut second = self.second.get() + (duration.as_secs() % Second.per(Minute) as u64) as u8;
        let mut minute = self.minute.get()
            + ((duration.as_secs() / Second.per(Minute) as u64) % Minute.per(Hour) as u64) as u8;
        let mut hour = self.hour.get()
            + ((duration.as_secs() / Second.per(Hour) as u64) % Hour.per(Day) as u64) as u8;
        let mut is_next_day = false;

        cascade!(nanosecond in 0..Nanosecond.per(Second) => second);
        cascade!(second in 0..Second.per(Minute) => minute);
        cascade!(minute in 0..Minute.per(Hour) => hour);
        if hour >= Hour.per(Day) {
            hour -= Hour.per(Day);
            is_next_day = true;
        }

        (
            is_next_day,
            // Safety: The cascades above ensure the values are in range.
            unsafe { Self::__from_hms_nanos_unchecked(hour, minute, second, nanosecond) },
        )
    }

    /// Subtract the sub-day time of the [`std::time::Duration`] to the `Time`. Wraps on overflow,
    /// returning whether the date is the previous date as the first element of the tuple.
    pub(crate) const fn adjusting_sub_std(self, duration: StdDuration) -> (bool, Self) {
        let mut nanosecond = self.nanosecond.get() as i32 - duration.subsec_nanos() as i32;
        let mut second =
            self.second.get() as i8 - (duration.as_secs() % Second.per(Minute) as u64) as i8;
        let mut minute = self.minute.get() as i8
            - ((duration.as_secs() / Second.per(Minute) as u64) % Minute.per(Hour) as u64) as i8;
        let mut hour = self.hour.get() as i8
            - ((duration.as_secs() / Second.per(Hour) as u64) % Hour.per(Day) as u64) as i8;
        let mut is_previous_day = false;

        cascade!(nanosecond in 0..Nanosecond.per(Second) as _ => second);
        cascade!(second in 0..Second.per(Minute) as _ => minute);
        cascade!(minute in 0..Minute.per(Hour) as _ => hour);
        if hour < 0 {
            hour += Hour.per(Day) as i8;
            is_previous_day = true;
        }

        (
            is_previous_day,
            // Safety: The cascades above ensure the values are in range.
            unsafe {
                Self::__from_hms_nanos_unchecked(
                    hour as _,
                    minute as _,
                    second as _,
                    nanosecond as _,
                )
            },
        )
    }
    // endregion arithmetic helpers

    // region: replacement
    /// Replace the clock hour.
    ///
    /// ```rust
    /// # use time_macros::time;
    /// assert_eq!(
    ///     time!(01:02:03.004_005_006).replace_hour(7),
    ///     Ok(time!(07:02:03.004_005_006))
    /// );
    /// assert!(time!(01:02:03.004_005_006).replace_hour(24).is_err()); // 24 isn't a valid hour
    /// ```
    #[must_use = "This method does not mutate the original `Time`."]
    pub const fn replace_hour(mut self, hour: u8) -> Result<Self, error::ComponentRange> {
        self.hour = ensure_ranged!(Hours: hour);
        Ok(self)
    }

    /// Replace the minutes within the hour.
    ///
    /// ```rust
    /// # use time_macros::time;
    /// assert_eq!(
    ///     time!(01:02:03.004_005_006).replace_minute(7),
    ///     Ok(time!(01:07:03.004_005_006))
    /// );
    /// assert!(time!(01:02:03.004_005_006).replace_minute(60).is_err()); // 60 isn't a valid minute
    /// ```
    #[must_use = "This method does not mutate the original `Time`."]
    pub const fn replace_minute(mut self, minute: u8) -> Result<Self, error::ComponentRange> {
        self.minute = ensure_ranged!(Minutes: minute);
        Ok(self)
    }

    /// Replace the seconds within the minute.
    ///
    /// ```rust
    /// # use time_macros::time;
    /// assert_eq!(
    ///     time!(01:02:03.004_005_006).replace_second(7),
    ///     Ok(time!(01:02:07.004_005_006))
    /// );
    /// assert!(time!(01:02:03.004_005_006).replace_second(60).is_err()); // 60 isn't a valid second
    /// ```
    #[must_use = "This method does not mutate the original `Time`."]
    pub const fn replace_second(mut self, second: u8) -> Result<Self, error::ComponentRange> {
        self.second = ensure_ranged!(Seconds: second);
        Ok(self)
    }

    /// Replace the milliseconds within the second.
    ///
    /// ```rust
    /// # use time_macros::time;
    /// assert_eq!(
    ///     time!(01:02:03.004_005_006).replace_millisecond(7),
    ///     Ok(time!(01:02:03.007))
    /// );
    /// assert!(time!(01:02:03.004_005_006).replace_millisecond(1_000).is_err()); // 1_000 isn't a valid millisecond
    /// ```
    #[must_use = "This method does not mutate the original `Time`."]
    pub const fn replace_millisecond(
        mut self,
        millisecond: u16,
    ) -> Result<Self, error::ComponentRange> {
        self.nanosecond =
            ensure_ranged!(Nanoseconds: millisecond as u32 * Nanosecond.per(Millisecond));
        Ok(self)
    }

    /// Replace the microseconds within the second.
    ///
    /// ```rust
    /// # use time_macros::time;
    /// assert_eq!(
    ///     time!(01:02:03.004_005_006).replace_microsecond(7_008),
    ///     Ok(time!(01:02:03.007_008))
    /// );
    /// assert!(time!(01:02:03.004_005_006).replace_microsecond(1_000_000).is_err()); // 1_000_000 isn't a valid microsecond
    /// ```
    #[must_use = "This method does not mutate the original `Time`."]
    pub const fn replace_microsecond(
        mut self,
        microsecond: u32,
    ) -> Result<Self, error::ComponentRange> {
        self.nanosecond =
            ensure_ranged!(Nanoseconds: microsecond * Nanosecond.per(Microsecond) as u32);
        Ok(self)
    }

    /// Replace the nanoseconds within the second.
    ///
    /// ```rust
    /// # use time_macros::time;
    /// assert_eq!(
    ///     time!(01:02:03.004_005_006).replace_nanosecond(7_008_009),
    ///     Ok(time!(01:02:03.007_008_009))
    /// );
    /// assert!(time!(01:02:03.004_005_006).replace_nanosecond(1_000_000_000).is_err()); // 1_000_000_000 isn't a valid nanosecond
    /// ```
    #[must_use = "This method does not mutate the original `Time`."]
    pub const fn replace_nanosecond(
        mut self,
        nanosecond: u32,
    ) -> Result<Self, error::ComponentRange> {
        self.nanosecond = ensure_ranged!(Nanoseconds: nanosecond);
        Ok(self)
    }
    // endregion replacement
}

// region: formatting & parsing
#[cfg(feature = "formatting")]
impl Time {
    /// Format the `Time` using the provided [format description](crate::format_description).
    pub fn format_into(
        self,
        output: &mut impl io::Write,
        format: &(impl Formattable + ?Sized),
    ) -> Result<usize, crate::error::Format> {
        format.format_into(output, None, Some(self), None)
    }

    /// Format the `Time` using the provided [format description](crate::format_description).
    ///
    /// ```rust
    /// # use time::format_description;
    /// # use time_macros::time;
    /// let format = format_description::parse("[hour]:[minute]:[second]")?;
    /// assert_eq!(time!(12:00).format(&format)?, "12:00:00");
    /// # Ok::<_, time::Error>(())
    /// ```
    pub fn format(
        self,
        format: &(impl Formattable + ?Sized),
    ) -> Result<String, crate::error::Format> {
        format.format(None, Some(self), None)
    }
}

#[cfg(feature = "parsing")]
impl Time {
    /// Parse a `Time` from the input using the provided [format
    /// description](crate::format_description).
    ///
    /// ```rust
    /// # use time::Time;
    /// # use time_macros::{time, format_description};
    /// let format = format_description!("[hour]:[minute]:[second]");
    /// assert_eq!(Time::parse("12:00:00", &format)?, time!(12:00));
    /// # Ok::<_, time::Error>(())
    /// ```
    pub fn parse(
        input: &str,
        description: &(impl Parsable + ?Sized),
    ) -> Result<Self, error::Parse> {
        description.parse_time(input.as_bytes())
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (value, width) = match self.nanosecond() {
            nanos if nanos % 10 != 0 => (nanos, 9),
            nanos if (nanos / 10) % 10 != 0 => (nanos / 10, 8),
            nanos if (nanos / 100) % 10 != 0 => (nanos / 100, 7),
            nanos if (nanos / 1_000) % 10 != 0 => (nanos / 1_000, 6),
            nanos if (nanos / 10_000) % 10 != 0 => (nanos / 10_000, 5),
            nanos if (nanos / 100_000) % 10 != 0 => (nanos / 100_000, 4),
            nanos if (nanos / 1_000_000) % 10 != 0 => (nanos / 1_000_000, 3),
            nanos if (nanos / 10_000_000) % 10 != 0 => (nanos / 10_000_000, 2),
            nanos => (nanos / 100_000_000, 1),
        };
        write!(
            f,
            "{}:{:02}:{:02}.{value:0width$}",
            self.hour, self.minute, self.second,
        )
    }
}

impl fmt::Debug for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
// endregion formatting & parsing

// region: trait impls
impl Add<Duration> for Time {
    type Output = Self;

    /// Add the sub-day time of the [`Duration`] to the `Time`. Wraps on overflow.
    ///
    /// ```rust
    /// # use time::ext::NumericalDuration;
    /// # use time_macros::time;
    /// assert_eq!(time!(12:00) + 2.hours(), time!(14:00));
    /// assert_eq!(time!(0:00:01) + (-2).seconds(), time!(23:59:59));
    /// ```
    fn add(self, duration: Duration) -> Self::Output {
        self.adjusting_add(duration).1
    }
}

impl Add<StdDuration> for Time {
    type Output = Self;

    /// Add the sub-day time of the [`std::time::Duration`] to the `Time`. Wraps on overflow.
    ///
    /// ```rust
    /// # use time::ext::NumericalStdDuration;
    /// # use time_macros::time;
    /// assert_eq!(time!(12:00) + 2.std_hours(), time!(14:00));
    /// assert_eq!(time!(23:59:59) + 2.std_seconds(), time!(0:00:01));
    /// ```
    fn add(self, duration: StdDuration) -> Self::Output {
        self.adjusting_add_std(duration).1
    }
}

impl_add_assign!(Time: Duration, StdDuration);

impl Sub<Duration> for Time {
    type Output = Self;

    /// Subtract the sub-day time of the [`Duration`] from the `Time`. Wraps on overflow.
    ///
    /// ```rust
    /// # use time::ext::NumericalDuration;
    /// # use time_macros::time;
    /// assert_eq!(time!(14:00) - 2.hours(), time!(12:00));
    /// assert_eq!(time!(23:59:59) - (-2).seconds(), time!(0:00:01));
    /// ```
    fn sub(self, duration: Duration) -> Self::Output {
        self.adjusting_sub(duration).1
    }
}

impl Sub<StdDuration> for Time {
    type Output = Self;

    /// Subtract the sub-day time of the [`std::time::Duration`] from the `Time`. Wraps on overflow.
    ///
    /// ```rust
    /// # use time::ext::NumericalStdDuration;
    /// # use time_macros::time;
    /// assert_eq!(time!(14:00) - 2.std_hours(), time!(12:00));
    /// assert_eq!(time!(0:00:01) - 2.std_seconds(), time!(23:59:59));
    /// ```
    fn sub(self, duration: StdDuration) -> Self::Output {
        self.adjusting_sub_std(duration).1
    }
}

impl_sub_assign!(Time: Duration, StdDuration);

impl Sub for Time {
    type Output = Duration;

    /// Subtract two `Time`s, returning the [`Duration`] between. This assumes both `Time`s are in
    /// the same calendar day.
    ///
    /// ```rust
    /// # use time::ext::NumericalDuration;
    /// # use time_macros::time;
    /// assert_eq!(time!(0:00) - time!(0:00), 0.seconds());
    /// assert_eq!(time!(1:00) - time!(0:00), 1.hours());
    /// assert_eq!(time!(0:00) - time!(1:00), (-1).hours());
    /// assert_eq!(time!(0:00) - time!(23:00), (-23).hours());
    /// ```
    fn sub(self, rhs: Self) -> Self::Output {
        let hour_diff = (self.hour.get() as i8) - (rhs.hour.get() as i8);
        let minute_diff = (self.minute.get() as i8) - (rhs.minute.get() as i8);
        let second_diff = (self.second.get() as i8) - (rhs.second.get() as i8);
        let nanosecond_diff = (self.nanosecond.get() as i32) - (rhs.nanosecond.get() as i32);

        let seconds = hour_diff as i64 * Second.per(Hour) as i64
            + minute_diff as i64 * Second.per(Minute) as i64
            + second_diff as i64;

        let (seconds, nanoseconds) = if seconds > 0 && nanosecond_diff < 0 {
            (seconds - 1, nanosecond_diff + Nanosecond.per(Second) as i32)
        } else if seconds < 0 && nanosecond_diff > 0 {
            (seconds + 1, nanosecond_diff - Nanosecond.per(Second) as i32)
        } else {
            (seconds, nanosecond_diff)
        };

        // Safety: `nanoseconds` is in range due to the overflow handling.
        unsafe { Duration::new_unchecked(seconds, nanoseconds) }
    }
}
// endregion trait impls
