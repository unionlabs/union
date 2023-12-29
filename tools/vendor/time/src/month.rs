//! The `Month` enum and its associated `impl`s.

use core::fmt;
use core::num::NonZeroU8;
use core::str::FromStr;

use self::Month::*;
use crate::error;

/// Months of the year.
#[allow(clippy::missing_docs_in_private_items)] // variants
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Month {
    January = 1,
    February = 2,
    March = 3,
    April = 4,
    May = 5,
    June = 6,
    July = 7,
    August = 8,
    September = 9,
    October = 10,
    November = 11,
    December = 12,
}

impl Month {
    /// Create a `Month` from its numerical value.
    pub(crate) const fn from_number(n: NonZeroU8) -> Result<Self, error::ComponentRange> {
        match n.get() {
            1 => Ok(January),
            2 => Ok(February),
            3 => Ok(March),
            4 => Ok(April),
            5 => Ok(May),
            6 => Ok(June),
            7 => Ok(July),
            8 => Ok(August),
            9 => Ok(September),
            10 => Ok(October),
            11 => Ok(November),
            12 => Ok(December),
            n => Err(error::ComponentRange {
                name: "month",
                minimum: 1,
                maximum: 12,
                value: n as _,
                conditional_range: false,
            }),
        }
    }

    /// Get the previous month.
    ///
    /// ```rust
    /// # use time::Month;
    /// assert_eq!(Month::January.previous(), Month::December);
    /// ```
    pub const fn previous(self) -> Self {
        match self {
            January => December,
            February => January,
            March => February,
            April => March,
            May => April,
            June => May,
            July => June,
            August => July,
            September => August,
            October => September,
            November => October,
            December => November,
        }
    }

    /// Get the next month.
    ///
    /// ```rust
    /// # use time::Month;
    /// assert_eq!(Month::January.next(), Month::February);
    /// ```
    pub const fn next(self) -> Self {
        match self {
            January => February,
            February => March,
            March => April,
            April => May,
            May => June,
            June => July,
            July => August,
            August => September,
            September => October,
            October => November,
            November => December,
            December => January,
        }
    }

    /// Get n-th next month.
    ///
    /// ```rust
    /// # use time::Month;
    /// assert_eq!(Month::January.nth_next(4), Month::May);
    /// assert_eq!(Month::July.nth_next(9), Month::April);
    /// ```
    pub const fn nth_next(self, n: u8) -> Self {
        match (self as u8 - 1 + n % 12) % 12 {
            0 => January,
            1 => February,
            2 => March,
            3 => April,
            4 => May,
            5 => June,
            6 => July,
            7 => August,
            8 => September,
            9 => October,
            10 => November,
            val => {
                debug_assert!(val == 11);
                December
            }
        }
    }

    /// Get n-th previous month.
    ///
    /// ```rust
    /// # use time::Month;
    /// assert_eq!(Month::January.nth_prev(4), Month::September);
    /// assert_eq!(Month::July.nth_prev(9), Month::October);
    /// ```
    pub const fn nth_prev(self, n: u8) -> Self {
        match self as i8 - 1 - (n % 12) as i8 {
            1 | -11 => February,
            2 | -10 => March,
            3 | -9 => April,
            4 | -8 => May,
            5 | -7 => June,
            6 | -6 => July,
            7 | -5 => August,
            8 | -4 => September,
            9 | -3 => October,
            10 | -2 => November,
            11 | -1 => December,
            val => {
                debug_assert!(val == 0);
                January
            }
        }
    }
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            January => "January",
            February => "February",
            March => "March",
            April => "April",
            May => "May",
            June => "June",
            July => "July",
            August => "August",
            September => "September",
            October => "October",
            November => "November",
            December => "December",
        })
    }
}

impl FromStr for Month {
    type Err = error::InvalidVariant;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "January" => Ok(January),
            "February" => Ok(February),
            "March" => Ok(March),
            "April" => Ok(April),
            "May" => Ok(May),
            "June" => Ok(June),
            "July" => Ok(July),
            "August" => Ok(August),
            "September" => Ok(September),
            "October" => Ok(October),
            "November" => Ok(November),
            "December" => Ok(December),
            _ => Err(error::InvalidVariant),
        }
    }
}

impl From<Month> for u8 {
    fn from(month: Month) -> Self {
        month as _
    }
}

impl TryFrom<u8> for Month {
    type Error = error::ComponentRange;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match NonZeroU8::new(value) {
            Some(value) => Self::from_number(value),
            None => Err(error::ComponentRange {
                name: "month",
                minimum: 1,
                maximum: 12,
                value: 0,
                conditional_range: false,
            }),
        }
    }
}
