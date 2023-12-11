use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::math::Uint64;

/// A point in time in nanosecond precision.
///
/// This type can represent times from 1970-01-01T00:00:00Z to 2554-07-21T23:34:33Z.
///
/// ## Examples
///
/// ```
/// # use cosmwasm_std::Timestamp;
/// let ts = Timestamp::from_nanos(1_000_000_202);
/// assert_eq!(ts.nanos(), 1_000_000_202);
/// assert_eq!(ts.seconds(), 1);
/// assert_eq!(ts.subsec_nanos(), 202);
///
/// let ts = ts.plus_seconds(2);
/// assert_eq!(ts.nanos(), 3_000_000_202);
/// assert_eq!(ts.seconds(), 3);
/// assert_eq!(ts.subsec_nanos(), 202);
/// ```
#[derive(
    Serialize, Deserialize, Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, JsonSchema,
)]
pub struct Timestamp(Uint64);

impl Timestamp {
    /// Creates a timestamp from nanoseconds since epoch
    pub const fn from_nanos(nanos_since_epoch: u64) -> Self {
        Timestamp(Uint64::new(nanos_since_epoch))
    }

    /// Creates a timestamp from seconds since epoch
    pub const fn from_seconds(seconds_since_epoch: u64) -> Self {
        Timestamp(Uint64::new(seconds_since_epoch * 1_000_000_000))
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub const fn plus_days(&self, addition: u64) -> Timestamp {
        self.plus_hours(addition * 24)
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub const fn plus_hours(&self, addition: u64) -> Timestamp {
        self.plus_minutes(addition * 60)
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub const fn plus_minutes(&self, addition: u64) -> Timestamp {
        self.plus_seconds(addition * 60)
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn plus_seconds(&self, addition: u64) -> Timestamp {
        self.plus_nanos(addition * 1_000_000_000)
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn plus_nanos(&self, addition: u64) -> Timestamp {
        let nanos = Uint64::new(self.0.u64() + addition);
        Timestamp(nanos)
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub const fn minus_days(&self, subtrahend: u64) -> Timestamp {
        self.minus_hours(subtrahend * 24)
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub const fn minus_hours(&self, subtrahend: u64) -> Timestamp {
        self.minus_minutes(subtrahend * 60)
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub const fn minus_minutes(&self, subtrahend: u64) -> Timestamp {
        self.minus_seconds(subtrahend * 60)
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn minus_seconds(&self, subtrahend: u64) -> Timestamp {
        self.minus_nanos(subtrahend * 1_000_000_000)
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn minus_nanos(&self, subtrahend: u64) -> Timestamp {
        let nanos = Uint64::new(self.0.u64() - subtrahend);
        Timestamp(nanos)
    }

    /// Returns nanoseconds since epoch
    #[inline]
    pub fn nanos(&self) -> u64 {
        self.0.u64()
    }

    /// Returns seconds since epoch (truncate nanoseconds)
    #[inline]
    pub fn seconds(&self) -> u64 {
        self.0.u64() / 1_000_000_000
    }

    /// Returns nanoseconds since the last whole second (the remainder truncated
    /// by `seconds()`)
    #[inline]
    pub fn subsec_nanos(&self) -> u64 {
        self.0.u64() % 1_000_000_000
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let whole = self.seconds();
        let fractional = self.subsec_nanos();
        write!(f, "{}.{:09}", whole, fractional)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_from_nanos() {
        let t = Timestamp::from_nanos(123);
        assert_eq!(t.0.u64(), 123);
        let t = Timestamp::from_nanos(0);
        assert_eq!(t.0.u64(), 0);
    }

    #[test]
    fn timestamp_from_seconds() {
        let t = Timestamp::from_seconds(123);
        assert_eq!(t.0.u64(), 123_000_000_000);
        let t = Timestamp::from_seconds(0);
        assert_eq!(t.0.u64(), 0);
    }

    #[test]
    fn timestamp_plus_seconds() {
        let sum = Timestamp::from_nanos(123).plus_seconds(42);
        assert_eq!(sum.0.u64(), 42_000_000_123);
        let sum = Timestamp::from_nanos(123).plus_seconds(0);
        assert_eq!(sum.0.u64(), 123);
    }

    #[test]
    fn timestamp_plus_nanos() {
        let sum = Timestamp::from_nanos(123).plus_nanos(3);
        assert_eq!(sum.0.u64(), 126);
        let sum = Timestamp::from_nanos(123).plus_nanos(0);
        assert_eq!(sum.0.u64(), 123);
    }

    #[test]
    fn timestamp_minus_seconds() {
        let earlier = Timestamp::from_seconds(123).minus_seconds(0);
        assert_eq!(earlier.0.u64(), 123_000_000_000);
        let earlier = Timestamp::from_seconds(123).minus_seconds(3);
        assert_eq!(earlier.0.u64(), 120_000_000_000);
        let earlier = Timestamp::from_seconds(123).minus_seconds(123);
        assert_eq!(earlier.0.u64(), 0);
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn timestamp_minus_seconds_panics_on_overflow() {
        let _earlier = Timestamp::from_seconds(100).minus_seconds(101);
    }

    #[test]
    fn timestamp_minus_nanos() {
        let earlier = Timestamp::from_seconds(123).minus_nanos(0);
        assert_eq!(earlier.0.u64(), 123_000_000_000);
        let earlier = Timestamp::from_seconds(123).minus_nanos(3);
        assert_eq!(earlier.0.u64(), 122_999_999_997);
        let earlier = Timestamp::from_seconds(123).minus_nanos(123_000_000_000);
        assert_eq!(earlier.0.u64(), 0);
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn timestamp_minus_nanos_panics_on_overflow() {
        let _earlier = Timestamp::from_nanos(100).minus_nanos(101);
    }

    #[test]
    fn timestamp_plus_days() {
        let ts = Timestamp::from_seconds(123).plus_days(0);
        assert_eq!(ts.0.u64(), 123_000_000_000);
        let ts = Timestamp::from_seconds(123).plus_days(10);
        assert_eq!(ts.0.u64(), 864_123_000_000_000);
    }

    #[test]
    fn timestamp_minus_days() {
        let ts = Timestamp::from_seconds(123).minus_days(0);
        assert_eq!(ts.0.u64(), 123_000_000_000);
        let ts = Timestamp::from_seconds(2 * 86400 + 123).minus_days(1);
        assert_eq!(ts.0.u64(), 86_523_000_000_000);
        let ts = Timestamp::from_seconds(86400).minus_days(1);
        assert_eq!(ts.0.u64(), 0);
    }

    #[test]
    fn timestamp_plus_hours() {
        let ts = Timestamp::from_seconds(123).plus_hours(0);
        assert_eq!(ts.0.u64(), 123_000_000_000);
        let ts = Timestamp::from_seconds(123).plus_hours(2);
        assert_eq!(ts.0.u64(), 123_000_000_000 + 60 * 60 * 2 * 1_000_000_000);
    }

    #[test]
    fn timestamp_minus_hours() {
        let ts = Timestamp::from_seconds(2 * 60 * 60).minus_hours(0);
        assert_eq!(ts.0.u64(), 2 * 60 * 60 * 1_000_000_000);
        let ts = Timestamp::from_seconds(2 * 60 * 60 + 123).minus_hours(1);
        assert_eq!(ts.0.u64(), 60 * 60 * 1_000_000_000 + 123_000_000_000);
    }

    #[test]
    fn timestamp_plus_minutes() {
        let ts = Timestamp::from_seconds(123).plus_minutes(0);
        assert_eq!(ts.0.u64(), 123_000_000_000);
        let ts = Timestamp::from_seconds(123).plus_minutes(2);
        assert_eq!(ts.0.u64(), 123_000_000_000 + 60 * 2 * 1_000_000_000);
    }

    #[test]
    fn timestamp_minus_minutes() {
        let ts = Timestamp::from_seconds(5 * 60).minus_minutes(0);
        assert_eq!(ts.0.u64(), 5 * 60 * 1_000_000_000);
        let ts = Timestamp::from_seconds(5 * 60 + 123).minus_minutes(1);
        assert_eq!(ts.0.u64(), 4 * 60 * 1_000_000_000 + 123_000_000_000);
    }

    #[test]
    fn timestamp_nanos() {
        let sum = Timestamp::from_nanos(123);
        assert_eq!(sum.nanos(), 123);
        let sum = Timestamp::from_nanos(0);
        assert_eq!(sum.nanos(), 0);
        let sum = Timestamp::from_nanos(987654321000);
        assert_eq!(sum.nanos(), 987654321000);
    }

    #[test]
    fn timestamp_seconds() {
        let sum = Timestamp::from_nanos(987654321000);
        assert_eq!(sum.seconds(), 987);
        let sum = Timestamp::from_seconds(1234567).plus_nanos(8765436);
        assert_eq!(sum.seconds(), 1234567);
    }

    #[test]
    fn timestamp_subsec_nanos() {
        let sum = Timestamp::from_nanos(987654321000);
        assert_eq!(sum.subsec_nanos(), 654321000);
        let sum = Timestamp::from_seconds(1234567).plus_nanos(8765436);
        assert_eq!(sum.subsec_nanos(), 8765436);
    }

    #[test]
    fn timestamp_implements_display() {
        let embedded = format!("Time: {}", Timestamp::from_nanos(0));
        assert_eq!(embedded, "Time: 0.000000000");
        let embedded = format!("Time: {}", Timestamp::from_nanos(1));
        assert_eq!(embedded, "Time: 0.000000001");
        let embedded = format!("Time: {}", Timestamp::from_nanos(10));
        assert_eq!(embedded, "Time: 0.000000010");
        let embedded = format!("Time: {}", Timestamp::from_nanos(100));
        assert_eq!(embedded, "Time: 0.000000100");
        let embedded = format!("Time: {}", Timestamp::from_nanos(1000));
        assert_eq!(embedded, "Time: 0.000001000");
        let embedded = format!("Time: {}", Timestamp::from_nanos(10000));
        assert_eq!(embedded, "Time: 0.000010000");
        let embedded = format!("Time: {}", Timestamp::from_nanos(100000));
        assert_eq!(embedded, "Time: 0.000100000");
        let embedded = format!("Time: {}", Timestamp::from_nanos(1000000));
        assert_eq!(embedded, "Time: 0.001000000");
        let embedded = format!("Time: {}", Timestamp::from_nanos(1000000));
        assert_eq!(embedded, "Time: 0.001000000");
        let embedded = format!("Time: {}", Timestamp::from_nanos(10000000));
        assert_eq!(embedded, "Time: 0.010000000");
        let embedded = format!("Time: {}", Timestamp::from_nanos(100000000));
        assert_eq!(embedded, "Time: 0.100000000");
        let embedded = format!("Time: {}", Timestamp::from_nanos(1000000000));
        assert_eq!(embedded, "Time: 1.000000000");
        let embedded = format!("Time: {}", Timestamp::from_nanos(10000000000));
        assert_eq!(embedded, "Time: 10.000000000");
        let embedded = format!("Time: {}", Timestamp::from_nanos(100000000000));
        assert_eq!(embedded, "Time: 100.000000000");
    }
}
