//! The [`UtcOffset`] struct and its associated `impl`s.

use core::fmt;
use core::ops::Neg;
#[cfg(feature = "formatting")]
use std::io;

use deranged::RangedI8;

use crate::convert::*;
use crate::error;
#[cfg(feature = "formatting")]
use crate::formatting::Formattable;
#[cfg(feature = "parsing")]
use crate::parsing::Parsable;
#[cfg(feature = "local-offset")]
use crate::sys::local_offset_at;
#[cfg(feature = "local-offset")]
use crate::OffsetDateTime;

/// The type of the `hours` field of `UtcOffset`.
type Hours = RangedI8<{ -(Hour.per(Day) as i8 - 1) }, { Hour.per(Day) as i8 - 1 }>;
/// The type of the `minutes` field of `UtcOffset`.
type Minutes = RangedI8<{ -(Minute.per(Hour) as i8 - 1) }, { Minute.per(Hour) as i8 - 1 }>;
/// The type of the `seconds` field of `UtcOffset`.
type Seconds = RangedI8<{ -(Second.per(Minute) as i8 - 1) }, { Second.per(Minute) as i8 - 1 }>;

/// An offset from UTC.
///
/// This struct can store values up to ±23:59:59. If you need support outside this range, please
/// file an issue with your use case.
// All three components _must_ have the same sign.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UtcOffset {
    #[allow(clippy::missing_docs_in_private_items)]
    hours: Hours,
    #[allow(clippy::missing_docs_in_private_items)]
    minutes: Minutes,
    #[allow(clippy::missing_docs_in_private_items)]
    seconds: Seconds,
}

impl UtcOffset {
    /// A `UtcOffset` that is UTC.
    ///
    /// ```rust
    /// # use time::UtcOffset;
    /// # use time_macros::offset;
    /// assert_eq!(UtcOffset::UTC, offset!(UTC));
    /// ```
    #[allow(clippy::undocumented_unsafe_blocks)] // rust-lang/rust-clippy#11246
    // Safety: All values are in range.
    pub const UTC: Self = unsafe { Self::__from_hms_unchecked(0, 0, 0) };

    // region: constructors
    /// Create a `UtcOffset` representing an offset of the hours, minutes, and seconds provided, the
    /// validity of which must be guaranteed by the caller. All three parameters must have the same
    /// sign.
    ///
    /// # Safety
    ///
    /// - Hours must be in the range `-23..=23`.
    /// - Minutes must be in the range `-59..=59`.
    /// - Seconds must be in the range `-59..=59`.
    ///
    /// While the signs of the parameters are required to match to avoid bugs, this is not a safety
    /// invariant.
    #[doc(hidden)]
    pub const unsafe fn __from_hms_unchecked(hours: i8, minutes: i8, seconds: i8) -> Self {
        // Safety: The caller must uphold the safety invariants.
        unsafe {
            Self::from_hms_ranged_unchecked(
                Hours::new_unchecked(hours),
                Minutes::new_unchecked(minutes),
                Seconds::new_unchecked(seconds),
            )
        }
    }

    /// Create a `UtcOffset` representing an offset by the number of hours, minutes, and seconds
    /// provided.
    ///
    /// The sign of all three components should match. If they do not, all smaller components will
    /// have their signs flipped.
    ///
    /// ```rust
    /// # use time::UtcOffset;
    /// assert_eq!(UtcOffset::from_hms(1, 2, 3)?.as_hms(), (1, 2, 3));
    /// assert_eq!(UtcOffset::from_hms(1, -2, -3)?.as_hms(), (1, 2, 3));
    /// # Ok::<_, time::Error>(())
    /// ```
    pub const fn from_hms(
        hours: i8,
        minutes: i8,
        seconds: i8,
    ) -> Result<Self, error::ComponentRange> {
        Ok(Self::from_hms_ranged(
            ensure_ranged!(Hours: hours),
            ensure_ranged!(Minutes: minutes),
            ensure_ranged!(Seconds: seconds),
        ))
    }

    /// Create a `UtcOffset` representing an offset of the hours, minutes, and seconds provided. All
    /// three parameters must have the same sign.
    ///
    /// While the signs of the parameters are required to match, this is not a safety invariant.
    pub(crate) const fn from_hms_ranged_unchecked(
        hours: Hours,
        minutes: Minutes,
        seconds: Seconds,
    ) -> Self {
        if hours.get() < 0 {
            debug_assert!(minutes.get() <= 0);
            debug_assert!(seconds.get() <= 0);
        } else if hours.get() > 0 {
            debug_assert!(minutes.get() >= 0);
            debug_assert!(seconds.get() >= 0);
        }
        if minutes.get() < 0 {
            debug_assert!(seconds.get() <= 0);
        } else if minutes.get() > 0 {
            debug_assert!(seconds.get() >= 0);
        }

        Self {
            hours,
            minutes,
            seconds,
        }
    }

    /// Create a `UtcOffset` representing an offset by the number of hours, minutes, and seconds
    /// provided.
    ///
    /// The sign of all three components should match. If they do not, all smaller components will
    /// have their signs flipped.
    pub(crate) const fn from_hms_ranged(
        hours: Hours,
        mut minutes: Minutes,
        mut seconds: Seconds,
    ) -> Self {
        if (hours.get() > 0 && minutes.get() < 0) || (hours.get() < 0 && minutes.get() > 0) {
            minutes = minutes.neg();
        }
        if (hours.get() > 0 && seconds.get() < 0)
            || (hours.get() < 0 && seconds.get() > 0)
            || (minutes.get() > 0 && seconds.get() < 0)
            || (minutes.get() < 0 && seconds.get() > 0)
        {
            seconds = seconds.neg();
        }

        Self {
            hours,
            minutes,
            seconds,
        }
    }

    /// Create a `UtcOffset` representing an offset by the number of seconds provided.
    ///
    /// ```rust
    /// # use time::UtcOffset;
    /// assert_eq!(UtcOffset::from_whole_seconds(3_723)?.as_hms(), (1, 2, 3));
    /// # Ok::<_, time::Error>(())
    /// ```
    pub const fn from_whole_seconds(seconds: i32) -> Result<Self, error::ComponentRange> {
        ensure_value_in_range!(
            seconds in -24 * Second.per(Hour) as i32 - 1 => 24 * Second.per(Hour) as i32 - 1
        );

        // Safety: The value was checked to be in range.
        Ok(unsafe {
            Self::__from_hms_unchecked(
                (seconds / Second.per(Hour) as i32) as _,
                ((seconds % Second.per(Hour) as i32) / Minute.per(Hour) as i32) as _,
                (seconds % Second.per(Minute) as i32) as _,
            )
        })
    }
    // endregion constructors

    // region: getters
    /// Obtain the UTC offset as its hours, minutes, and seconds. The sign of all three components
    /// will always match. A positive value indicates an offset to the east; a negative to the west.
    ///
    /// ```rust
    /// # use time_macros::offset;
    /// assert_eq!(offset!(+1:02:03).as_hms(), (1, 2, 3));
    /// assert_eq!(offset!(-1:02:03).as_hms(), (-1, -2, -3));
    /// ```
    pub const fn as_hms(self) -> (i8, i8, i8) {
        (self.hours.get(), self.minutes.get(), self.seconds.get())
    }

    /// Obtain the UTC offset as its hours, minutes, and seconds. The sign of all three components
    /// will always match. A positive value indicates an offset to the east; a negative to the west.
    ///
    /// ```rust
    /// # use time_macros::offset;
    /// assert_eq!(offset!(+1:02:03).as_hms(), (1, 2, 3));
    /// assert_eq!(offset!(-1:02:03).as_hms(), (-1, -2, -3));
    /// ```
    #[cfg(feature = "quickcheck")]
    pub(crate) const fn as_hms_ranged(self) -> (Hours, Minutes, Seconds) {
        (self.hours, self.minutes, self.seconds)
    }

    /// Obtain the number of whole hours the offset is from UTC. A positive value indicates an
    /// offset to the east; a negative to the west.
    ///
    /// ```rust
    /// # use time_macros::offset;
    /// assert_eq!(offset!(+1:02:03).whole_hours(), 1);
    /// assert_eq!(offset!(-1:02:03).whole_hours(), -1);
    /// ```
    pub const fn whole_hours(self) -> i8 {
        self.hours.get()
    }

    /// Obtain the number of whole minutes the offset is from UTC. A positive value indicates an
    /// offset to the east; a negative to the west.
    ///
    /// ```rust
    /// # use time_macros::offset;
    /// assert_eq!(offset!(+1:02:03).whole_minutes(), 62);
    /// assert_eq!(offset!(-1:02:03).whole_minutes(), -62);
    /// ```
    pub const fn whole_minutes(self) -> i16 {
        self.hours.get() as i16 * Minute.per(Hour) as i16 + self.minutes.get() as i16
    }

    /// Obtain the number of minutes past the hour the offset is from UTC. A positive value
    /// indicates an offset to the east; a negative to the west.
    ///
    /// ```rust
    /// # use time_macros::offset;
    /// assert_eq!(offset!(+1:02:03).minutes_past_hour(), 2);
    /// assert_eq!(offset!(-1:02:03).minutes_past_hour(), -2);
    /// ```
    pub const fn minutes_past_hour(self) -> i8 {
        self.minutes.get()
    }

    /// Obtain the number of whole seconds the offset is from UTC. A positive value indicates an
    /// offset to the east; a negative to the west.
    ///
    /// ```rust
    /// # use time_macros::offset;
    /// assert_eq!(offset!(+1:02:03).whole_seconds(), 3723);
    /// assert_eq!(offset!(-1:02:03).whole_seconds(), -3723);
    /// ```
    // This may be useful for anyone manually implementing arithmetic, as it
    // would let them construct a `Duration` directly.
    pub const fn whole_seconds(self) -> i32 {
        self.hours.get() as i32 * Second.per(Hour) as i32
            + self.minutes.get() as i32 * Second.per(Minute) as i32
            + self.seconds.get() as i32
    }

    /// Obtain the number of seconds past the minute the offset is from UTC. A positive value
    /// indicates an offset to the east; a negative to the west.
    ///
    /// ```rust
    /// # use time_macros::offset;
    /// assert_eq!(offset!(+1:02:03).seconds_past_minute(), 3);
    /// assert_eq!(offset!(-1:02:03).seconds_past_minute(), -3);
    /// ```
    pub const fn seconds_past_minute(self) -> i8 {
        self.seconds.get()
    }
    // endregion getters

    // region: is_{sign}
    /// Check if the offset is exactly UTC.
    ///
    ///
    /// ```rust
    /// # use time_macros::offset;
    /// assert!(!offset!(+1:02:03).is_utc());
    /// assert!(!offset!(-1:02:03).is_utc());
    /// assert!(offset!(UTC).is_utc());
    /// ```
    pub const fn is_utc(self) -> bool {
        self.hours.get() == 0 && self.minutes.get() == 0 && self.seconds.get() == 0
    }

    /// Check if the offset is positive, or east of UTC.
    ///
    /// ```rust
    /// # use time_macros::offset;
    /// assert!(offset!(+1:02:03).is_positive());
    /// assert!(!offset!(-1:02:03).is_positive());
    /// assert!(!offset!(UTC).is_positive());
    /// ```
    pub const fn is_positive(self) -> bool {
        self.hours.get() > 0 || self.minutes.get() > 0 || self.seconds.get() > 0
    }

    /// Check if the offset is negative, or west of UTC.
    ///
    /// ```rust
    /// # use time_macros::offset;
    /// assert!(!offset!(+1:02:03).is_negative());
    /// assert!(offset!(-1:02:03).is_negative());
    /// assert!(!offset!(UTC).is_negative());
    /// ```
    pub const fn is_negative(self) -> bool {
        self.hours.get() < 0 || self.minutes.get() < 0 || self.seconds.get() < 0
    }
    // endregion is_{sign}

    // region: local offset
    /// Attempt to obtain the system's UTC offset at a known moment in time. If the offset cannot be
    /// determined, an error is returned.
    ///
    /// ```rust
    /// # use time::{UtcOffset, OffsetDateTime};
    /// let local_offset = UtcOffset::local_offset_at(OffsetDateTime::UNIX_EPOCH);
    /// # if false {
    /// assert!(local_offset.is_ok());
    /// # }
    /// ```
    #[cfg(feature = "local-offset")]
    pub fn local_offset_at(datetime: OffsetDateTime) -> Result<Self, error::IndeterminateOffset> {
        local_offset_at(datetime).ok_or(error::IndeterminateOffset)
    }

    /// Attempt to obtain the system's current UTC offset. If the offset cannot be determined, an
    /// error is returned.
    ///
    /// ```rust
    /// # use time::UtcOffset;
    /// let local_offset = UtcOffset::current_local_offset();
    /// # if false {
    /// assert!(local_offset.is_ok());
    /// # }
    /// ```
    #[cfg(feature = "local-offset")]
    pub fn current_local_offset() -> Result<Self, error::IndeterminateOffset> {
        let now = OffsetDateTime::now_utc();
        local_offset_at(now).ok_or(error::IndeterminateOffset)
    }
    // endregion: local offset
}

// region: formatting & parsing
#[cfg(feature = "formatting")]
impl UtcOffset {
    /// Format the `UtcOffset` using the provided [format description](crate::format_description).
    pub fn format_into(
        self,
        output: &mut impl io::Write,
        format: &(impl Formattable + ?Sized),
    ) -> Result<usize, error::Format> {
        format.format_into(output, None, None, Some(self))
    }

    /// Format the `UtcOffset` using the provided [format description](crate::format_description).
    ///
    /// ```rust
    /// # use time::format_description;
    /// # use time_macros::offset;
    /// let format = format_description::parse("[offset_hour sign:mandatory]:[offset_minute]")?;
    /// assert_eq!(offset!(+1).format(&format)?, "+01:00");
    /// # Ok::<_, time::Error>(())
    /// ```
    pub fn format(self, format: &(impl Formattable + ?Sized)) -> Result<String, error::Format> {
        format.format(None, None, Some(self))
    }
}

#[cfg(feature = "parsing")]
impl UtcOffset {
    /// Parse a `UtcOffset` from the input using the provided [format
    /// description](crate::format_description).
    ///
    /// ```rust
    /// # use time::UtcOffset;
    /// # use time_macros::{offset, format_description};
    /// let format = format_description!("[offset_hour]:[offset_minute]");
    /// assert_eq!(UtcOffset::parse("-03:42", &format)?, offset!(-3:42));
    /// # Ok::<_, time::Error>(())
    /// ```
    pub fn parse(
        input: &str,
        description: &(impl Parsable + ?Sized),
    ) -> Result<Self, error::Parse> {
        description.parse_offset(input.as_bytes())
    }
}

impl fmt::Display for UtcOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{:02}:{:02}:{:02}",
            if self.is_negative() { '-' } else { '+' },
            self.hours.abs(),
            self.minutes.abs(),
            self.seconds.abs(),
        )
    }
}

impl fmt::Debug for UtcOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
// endregion formatting & parsing

impl Neg for UtcOffset {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::from_hms_ranged(self.hours.neg(), self.minutes.neg(), self.seconds.neg())
    }
}
