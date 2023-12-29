//! Implementations of the [`quickcheck::Arbitrary`](quickcheck::Arbitrary) trait.
//!
//! This enables users to write tests such as this, and have test values provided automatically:
//!
//! ```
//! # #![allow(dead_code)]
//! use quickcheck::quickcheck;
//! use time::Date;
//!
//! struct DateRange {
//!     from: Date,
//!     to: Date,
//! }
//!
//! impl DateRange {
//!     fn new(from: Date, to: Date) -> Result<Self, ()> {
//!         Ok(DateRange { from, to })
//!     }
//! }
//!
//! quickcheck! {
//!     fn date_range_is_well_defined(from: Date, to: Date) -> bool {
//!         let r = DateRange::new(from, to);
//!         if from <= to {
//!             r.is_ok()
//!         } else {
//!             r.is_err()
//!         }
//!     }
//! }
//! ```
//!
//! An implementation for `Instant` is intentionally omitted since its values are only meaningful in
//! relation to a [`Duration`], and obtaining an `Instant` from a [`Duration`] is very simple
//! anyway.

use alloc::boxed::Box;

use quickcheck::{empty_shrinker, single_shrinker, Arbitrary, Gen};

use crate::date_time::{DateTime, MaybeOffset};
use crate::{Date, Duration, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset, Weekday};

/// Obtain an arbitrary value between the minimum and maximum inclusive.
macro_rules! arbitrary_between {
    ($type:ty; $gen:expr, $min:expr, $max:expr) => {{
        let min = $min;
        let max = $max;
        let range = max - min;
        <$type>::arbitrary($gen).rem_euclid(range + 1) + min
    }};
}

impl Arbitrary for Date {
    fn arbitrary(g: &mut Gen) -> Self {
        Self::from_julian_day_unchecked(arbitrary_between!(
            i32;
            g,
            Self::MIN.to_julian_day(),
            Self::MAX.to_julian_day()
        ))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            self.to_ordinal_date()
                .shrink()
                .flat_map(|(year, ordinal)| Self::from_ordinal_date(year, ordinal)),
        )
    }
}

impl Arbitrary for Duration {
    fn arbitrary(g: &mut Gen) -> Self {
        Self::new_ranged(<_>::arbitrary(g), <_>::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.subsec_nanoseconds_ranged(), self.whole_seconds())
                .shrink()
                .map(|(mut nanoseconds, seconds)| {
                    // Coerce the sign if necessary.
                    if (seconds > 0 && nanoseconds.get() < 0)
                        || (seconds < 0 && nanoseconds.get() > 0)
                    {
                        nanoseconds = nanoseconds.neg();
                    }

                    Self::new_ranged_unchecked(seconds, nanoseconds)
                }),
        )
    }
}

impl Arbitrary for Time {
    fn arbitrary(g: &mut Gen) -> Self {
        Self::from_hms_nanos_ranged(
            <_>::arbitrary(g),
            <_>::arbitrary(g),
            <_>::arbitrary(g),
            <_>::arbitrary(g),
        )
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            self.as_hms_nano_ranged()
                .shrink()
                .map(|(hour, minute, second, nanosecond)| {
                    Self::from_hms_nanos_ranged(hour, minute, second, nanosecond)
                }),
        )
    }
}

impl Arbitrary for PrimitiveDateTime {
    fn arbitrary(g: &mut Gen) -> Self {
        Self(<_>::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.0.shrink().map(Self))
    }
}

impl Arbitrary for UtcOffset {
    fn arbitrary(g: &mut Gen) -> Self {
        Self::from_hms_ranged(<_>::arbitrary(g), <_>::arbitrary(g), <_>::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            self.as_hms_ranged()
                .shrink()
                .map(|(hours, minutes, seconds)| Self::from_hms_ranged(hours, minutes, seconds)),
        )
    }
}

impl Arbitrary for OffsetDateTime {
    fn arbitrary(g: &mut Gen) -> Self {
        Self(<_>::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.0.shrink().map(Self))
    }
}

impl<O: MaybeOffset + 'static> Arbitrary for DateTime<O> {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            date: <_>::arbitrary(g),
            time: <_>::arbitrary(g),
            offset: <_>::arbitrary(g),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.date, self.time, self.offset)
                .shrink()
                .map(|(date, time, offset)| Self { date, time, offset }),
        )
    }
}

impl Arbitrary for Weekday {
    fn arbitrary(g: &mut Gen) -> Self {
        use Weekday::*;
        match arbitrary_between!(u8; g, 0, 6) {
            0 => Monday,
            1 => Tuesday,
            2 => Wednesday,
            3 => Thursday,
            4 => Friday,
            5 => Saturday,
            val => {
                debug_assert!(val == 6);
                Sunday
            }
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            Self::Monday => empty_shrinker(),
            _ => single_shrinker(self.previous()),
        }
    }
}

impl Arbitrary for Month {
    fn arbitrary(g: &mut Gen) -> Self {
        use Month::*;
        match arbitrary_between!(u8; g, 1, 12) {
            1 => January,
            2 => February,
            3 => March,
            4 => April,
            5 => May,
            6 => June,
            7 => July,
            8 => August,
            9 => September,
            10 => October,
            11 => November,
            val => {
                debug_assert!(val == 12);
                December
            }
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            Self::January => empty_shrinker(),
            _ => single_shrinker(self.previous()),
        }
    }
}
