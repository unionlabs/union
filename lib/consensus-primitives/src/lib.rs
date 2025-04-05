//! Various types used in the public interfaces of consensus verification.

#![warn(clippy::pedantic, missing_docs, clippy::missing_const_for_fn)]
#![cfg_attr(not(feature = "std"), no_std)]

use core::fmt;

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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Timestamp(u64);

impl Timestamp {
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
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
