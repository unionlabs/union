//! Various types used in the public interfaces of consensus verification.

#![warn(clippy::pedantic, missing_docs, clippy::missing_const_for_fn)]
#![cfg_attr(not(feature = "std"), no_std)]

use core::{fmt, ops::Add, str::FromStr};

/// Represents a timestamp, normalized to nanoseconds.
///
/// In order to reduce confusion when dealing with many different consensus mechanisms that all
/// store timestamps differently, this type only exposes explicit constructors and accessors (for
/// both seconds and nanoseconds).
///
/// ```rust
/// # use consensus_primitives::Timestamp;
/// assert_eq!(Timestamp::from_nanos(1_000_000_000), Timestamp::from_secs(1));
/// ```
///
/// This type can represent timestamps between **January 1, 1970 12:00:00 AM** and **July 21, 2554
/// 11:34:33.709 PM** (about 529 years from the time of writing this). If this code is still in use
/// at this time, good luck.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Timestamp(u64);

impl Timestamp {
    /// Zero timestamp.
    pub const ZERO: Self = Timestamp(0);

    /// Construct a [`Timestamp`] from the given ***nanoseconds*** value.
    ///
    /// ```rust
    /// # use consensus_primitives::Timestamp;
    /// assert_eq!(Timestamp::from_nanos(1_000_000_000).as_secs(), 1);
    /// ```
    #[must_use = "constructing a Timestamp has no effect"]
    pub const fn from_nanos(nanos: u64) -> Self {
        Timestamp(nanos)
    }

    /// Construct a [`Timestamp`] from the given ***seconds*** value.
    ///
    /// ```rust
    /// # use consensus_primitives::Timestamp;
    /// assert_eq!(Timestamp::from_secs(1).as_secs(), 1);
    /// ```
    #[must_use = "constructing a Timestamp has no effect"]
    pub const fn from_secs(secs: u64) -> Self {
        Timestamp(secs * 1_000_000_000)
    }

    /// Access the inner timestamp value, as ***nanoseconds***.
    ///
    /// ```rust
    /// # use consensus_primitives::Timestamp;
    /// assert_eq!(Timestamp::from_secs(1).as_nanos(), 1_000_000_000);
    /// ```
    #[must_use = "accessing the inner value has no effect"]
    pub const fn as_nanos(&self) -> u64 {
        self.0
    }

    /// Access the inner timestamp value, as ***seconds***.
    ///
    /// ```rust
    /// # use consensus_primitives::Timestamp;
    /// assert_eq!(Timestamp::from_nanos(1).as_nanos(), 1);
    /// ```
    #[must_use = "accessing the inner value has no effect"]
    pub const fn as_secs(&self) -> u64 {
        self.0 / 1_000_000_000
    }

    /// Check if the timestamp is zero.
    ///
    /// Note that this checks against the ***nanos***. If the value is sub-second, this will return *false*, whereas [`Timestamp::as_secs()`] will return 0:
    ///
    /// ```rust
    /// # use consensus_primitives::Timestamp;
    /// let ts = Timestamp::from_nanos(123);
    /// assert_eq!(ts.as_secs(), 0);
    /// assert!(!ts.is_zero());
    /// ```
    #[must_use = "accessing the inner value has no effect"]
    pub const fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl FromStr for Timestamp {
    type Err = <u64 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add for Timestamp {
    type Output = Self;

    #[track_caller]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
