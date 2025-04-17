//! Various types used in the public interfaces of consensus verification.

#![warn(clippy::pedantic, missing_docs, clippy::missing_const_for_fn)]
#![cfg_attr(not(feature = "std"), no_std)]

use core::{fmt, str::FromStr};

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

    /// Return the timestamp at self + duration.
    ///
    /// This will return `None` if the resulting value would overflow.
    ///
    /// ```rust
    /// # use consensus_primitives::{Timestamp, Duration};
    /// let ts = Timestamp::from_nanos(123);
    /// let d = Duration::from_secs(1);
    /// assert_eq!(ts.plus_duration(d), Some(Timestamp::from_nanos(1_000_000_123)));
    /// assert_eq!(ts.plus_duration(Duration::from_nanos(u64::MAX)), None);
    /// ```
    #[must_use = "accessing the inner value has no effect"]
    pub const fn plus_duration(&self, duration: Duration) -> Option<Self> {
        match self.0.checked_add(duration.0) {
            Some(ts) => Some(Self(ts)),
            None => None,
        }
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

/// Represents a duration, normalized to nanoseconds.
///
/// A duration is a period between two [`Timestamp`]s. In order to reduce confusion when dealing with many different consensus mechanisms that all
/// store timestamps differently, this type only exposes explicit constructors and accessors (for
/// both seconds and nanoseconds).
///
/// ```rust
/// # use consensus_primitives::Duration;
/// assert_eq!(Duration::from_nanos(1_000_000_000), Duration::from_secs(1));
/// ```
///
/// This type can represent durations with a maximum range of about 529 years. If a longer duration is required, good luck.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Duration(u64);

impl Duration {
    /// Zero timestamp.
    pub const ZERO: Self = Duration(0);

    /// Construct a [`Duration`] from the given ***nanoseconds*** value.
    ///
    /// ```rust
    /// # use consensus_primitives::Duration;
    /// assert_eq!(Duration::from_nanos(1_000_000_000).as_secs(), 1);
    /// ```
    #[must_use = "constructing a Duration has no effect"]
    pub const fn from_nanos(nanos: u64) -> Self {
        Duration(nanos)
    }

    /// Construct a [`Duration`] from the given ***seconds*** value.
    ///
    /// ```rust
    /// # use consensus_primitives::Duration;
    /// assert_eq!(Duration::from_secs(1).as_secs(), 1);
    /// ```
    #[must_use = "constructing a Duration has no effect"]
    pub const fn from_secs(secs: u64) -> Self {
        Duration(secs * 1_000_000_000)
    }

    /// Access the inner timestamp value, as ***nanoseconds***.
    ///
    /// ```rust
    /// # use consensus_primitives::Duration;
    /// assert_eq!(Duration::from_secs(1).as_nanos(), 1_000_000_000);
    /// ```
    #[must_use = "accessing the inner value has no effect"]
    pub const fn as_nanos(&self) -> u64 {
        self.0
    }

    /// Access the inner timestamp value, as ***seconds***.
    ///
    /// ```rust
    /// # use consensus_primitives::Duration;
    /// assert_eq!(Duration::from_nanos(1).as_nanos(), 1);
    /// ```
    #[must_use = "accessing the inner value has no effect"]
    pub const fn as_secs(&self) -> u64 {
        self.0 / 1_000_000_000
    }

    /// Check if the timestamp is zero.
    ///
    /// Note that this checks against the ***nanos***. If the value is sub-second, this will return *false*, whereas [`Duration::as_secs()`] will return 0:
    ///
    /// ```rust
    /// # use consensus_primitives::Duration;
    /// let ts = Duration::from_nanos(123);
    /// assert_eq!(ts.as_secs(), 0);
    /// assert!(!ts.is_zero());
    /// ```
    #[must_use = "accessing the inner value has no effect"]
    pub const fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl FromStr for Duration {
    type Err = <u64 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
