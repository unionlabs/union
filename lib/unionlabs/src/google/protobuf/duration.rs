use core::{
    fmt::{self, Debug, Display},
    num::ParseIntError,
    ops::{Mul, Neg},
    str::FromStr,
};

use macros::model;

use crate::{
    bounded::{BoundedI128, BoundedI32, BoundedI64, BoundedIntError},
    constants::metric::NANOS_PER_SECOND,
    macros::result_try,
};

pub const DURATION_MAX_SECONDS: i64 = 315_576_000_000;
pub const DURATION_MIN_SECONDS: i64 = -DURATION_MAX_SECONDS;

pub const DURATION_MAX_NANOS: i32 = 999_999_999;
pub const DURATION_MIN_NANOS: i32 = -DURATION_MAX_NANOS;

type SubZeroNanos = BoundedI32<DURATION_MIN_NANOS, DURATION_MAX_NANOS>;

type NegativeSeconds = BoundedI64<DURATION_MIN_SECONDS, -1>;
type NegativeNanos = BoundedI32<DURATION_MIN_NANOS, 0>;

type PositiveSeconds = BoundedI64<1, DURATION_MAX_SECONDS>;
type PositiveNanos = BoundedI32<0, DURATION_MAX_NANOS>;

type DurationInner = BoundedI128<
    { (DURATION_MIN_SECONDS as i128 * NANOS_PER_SECOND as i128) + DURATION_MIN_NANOS as i128 },
    { (DURATION_MAX_SECONDS as i128 * NANOS_PER_SECOND as i128) + DURATION_MAX_NANOS as i128 },
>;

/// # Seconds
///
/// > Signed seconds of the span of time. Must be from -315,576,000,000
/// > to +315,576,000,000 inclusive. Note: these bounds are computed from:
/// > 60 sec/min * 60 min/hr * 24 hr/day * 365.25 days/year * 10000 years
///
/// # Nanos
///
/// > Signed fractions of a second at nanosecond resolution of the span
/// > of time. Durations less than one second are represented with a 0
/// > `seconds` field and a positive or negative `nanos` field. For durations
/// > of one second or more, a non-zero value for the `nanos` field must be
/// > of the same sign as the `seconds` field. Must be from -999,999,999
/// > to +999,999,999 inclusive.
#[model(proto(raw(protos::google::protobuf::Duration), into, from), no_serde)]
#[derive(PartialOrd, Ord, Copy)]
#[debug("Duration({})", self)]
pub struct Duration(DurationInner);

impl Neg for Duration {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.0
            .inner()
            .neg()
            .try_into()
            .map(Self)
            .expect("lower bound == neg(upper bound); qed;")
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).and_then(|str| {
            str.parse().map_err(|_| {
                serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(&str),
                    &"a valid protobuf duration string (`<seconds>[.<nanos>]s`)",
                )
            })
        })
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[derive(Debug)]
pub enum DurationFromStrError {
    /// The `s` suffix was missing.
    MissingSecondsSuffix,
    /// Either the seconds or nanos values couldn't be parsed as numbers.
    ParseInt(ParseIntError),
    /// The seconds value was out of bounds.
    Seconds(BoundedIntError<i64>),
    /// Too many nanos were present (max precision is 9 decimal places).
    TooManyNanos,
}

impl FromStr for Duration {
    type Err = DurationFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // seconds, nanos
        let s = s
            .strip_suffix('s')
            .ok_or(DurationFromStrError::MissingSecondsSuffix)?;

        match s.split_once('.') {
            Some((seconds, nanos)) => {
                const NANOS_EXPECT_MSG: &str = "value is within i32 bounds as length is bounded at 9 digits; max is 999_999_999; qed;";

                let seconds = seconds
                    .parse::<i64>()
                    .map_err(DurationFromStrError::ParseInt)?;

                let nanos_power_of_10 = 9_u32
                    .checked_sub(
                        nanos
                            .len()
                            .try_into()
                            .map_err(|_| DurationFromStrError::TooManyNanos)?,
                    )
                    .ok_or(DurationFromStrError::TooManyNanos)?;

                // can't use seconds.is_negative() since "-0" parses as 0
                let is_positive = !s.starts_with('-');

                // only parse "positive" nanos as the sign is determined by the seconds
                let nanos: i32 = nanos
                    .parse::<u32>()
                    .map_err(DurationFromStrError::ParseInt)?
                    .mul(10_u32.pow(nanos_power_of_10))
                    .try_into()
                    .expect(NANOS_EXPECT_MSG);

                if seconds == 0 {
                    let nanos: SubZeroNanos = (if is_positive { nanos } else { -nanos })
                        .try_into()
                        .expect(NANOS_EXPECT_MSG);

                    Ok(Self::new_private(0, nanos.inner()))
                } else {
                    #[allow(clippy::collapsible_else_if)] // makes the semantics more clear
                    if is_positive {
                        let seconds: PositiveSeconds =
                            seconds.try_into().map_err(DurationFromStrError::Seconds)?;
                        let nanos: PositiveNanos = nanos.try_into().expect(NANOS_EXPECT_MSG);

                        Ok(Self::new_private(seconds.inner(), nanos.inner()))
                    } else {
                        let seconds: NegativeSeconds =
                            seconds.try_into().map_err(DurationFromStrError::Seconds)?;
                        let nanos: NegativeNanos = nanos.neg().try_into().expect(NANOS_EXPECT_MSG);

                        Ok(Self::new_private(seconds.inner(), nanos.inner()))
                    }
                }
            }
            // no nanos
            None => s
                .parse::<i64>()
                .map_err(DurationFromStrError::ParseInt)?
                .try_into()
                .map(
                    |seconds: BoundedI64<DURATION_MIN_SECONDS, DURATION_MAX_SECONDS>| {
                        Self::new_private(seconds.inner(), 0)
                    },
                )
                .map_err(DurationFromStrError::Seconds),
        }
    }
}

impl Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seconds = self.seconds().inner();
        let nanos = self.nanos().inner();
        let nanos_str = if nanos == 0 {
            String::new()
        } else {
            // SAFETY: .abs() is safe to call here since self.nanos()
            // will never be i32::MIN
            format!(".{:09}", nanos.abs())
        };

        let sign = if seconds == 0 && nanos.is_negative() {
            "-"
        } else {
            ""
        };

        f.write_fmt(format_args!(
            "{sign}{seconds}{}s",
            nanos_str.trim_end_matches('0')
        ))
    }
}

impl Duration {
    pub const fn new(seconds: i64, nanos: i32) -> Result<Self, DurationError> {
        #[allow(overlapping_range_endpoints)] // false positive, report upstream
        match (seconds, nanos) {
            (0, _) => {
                let nanos = result_try!(SubZeroNanos::new_const(nanos), DurationError::Nanos);

                Ok(Self::new_private(0, nanos.inner()))
            }
            // negative seconds, negative or zero nanos
            (..=-1, ..=0) => {
                let seconds =
                    result_try!(NegativeSeconds::new_const(seconds), DurationError::Seconds);
                let nanos = result_try!(NegativeNanos::new_const(nanos), DurationError::Nanos);

                Ok(Self::new_private(seconds.inner(), nanos.inner()))
            }
            // positive seconds, positive or zero nanos
            (1.., 0..) => {
                let seconds =
                    result_try!(PositiveSeconds::new_const(seconds), DurationError::Seconds);
                let nanos = result_try!(PositiveNanos::new_const(nanos), DurationError::Nanos);

                Ok(Self::new_private(seconds.inner(), nanos.inner()))
            }
            // negative seconds, positive nanos
            (1.., ..=-1) | (..=-1, 1..) => Err(DurationError::Sign),
        }
    }

    /// Expects that the values passed in are a valid Duration representation.
    const fn new_private(seconds: i64, nanos: i32) -> Self {
        let inner = (seconds as i128 * NANOS_PER_SECOND as i128) + nanos as i128;

        // false positive (fixed in newer versions)
        // https://github.com/rust-lang/rust-clippy/pull/10811
        #[allow(clippy::match_wild_err_arm)]
        match BoundedI128::new_const(inner) {
            Ok(ok) => Self(ok),
            Err(_) => {
                unreachable!()
            }
        }
    }

    /// Return the seconds portion of this [`Duration`].
    #[must_use]
    pub const fn seconds(&self) -> BoundedI64<DURATION_MIN_SECONDS, DURATION_MAX_SECONDS> {
        let value = self.0.inner() / NANOS_PER_SECOND as i128;

        debug_assert!(
            value >= DURATION_MIN_SECONDS as i128 && value <= DURATION_MAX_SECONDS as i128
        );

        #[allow(clippy::cast_possible_truncation)] // invariant checked above
        match BoundedI64::new_const(value as i64) {
            Ok(ok) => ok,
            Err(_) => {
                unreachable!()
            }
        }
    }

    /// Return the nanosecond portion of this [`Duration`].
    #[must_use]
    pub const fn nanos(&self) -> BoundedI32<DURATION_MIN_NANOS, DURATION_MAX_NANOS> {
        let value = self.0.inner() % NANOS_PER_SECOND as i128;

        debug_assert!(value >= DURATION_MIN_NANOS as i128 && value <= DURATION_MAX_NANOS as i128);

        // false positive (fixed in newer versions)
        // https://github.com/rust-lang/rust-clippy/pull/10811
        #[allow(clippy::cast_possible_truncation)] // invariant checked above
        match BoundedI32::new_const(value as i32) {
            Ok(ok) => ok,
            Err(_) => {
                unreachable!()
            }
        }
    }

    /// Return this [`Duration`] with full nanosecond precision.
    #[must_use]
    pub const fn as_nanos(&self) -> DurationInner {
        self.0
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum DurationError {
    #[error("invalid seconds")]
    Seconds(BoundedIntError<i64>),
    #[error("invalid nanos")]
    Nanos(BoundedIntError<i32>),
    #[error("incorrect sign for nanos")]
    Sign,
}

impl Duration {
    #[must_use]
    pub fn checked_add(self, rhs: Duration) -> Option<Duration> {
        self.0
            .inner()
            .checked_add(rhs.0.inner())?
            .try_into()
            .map(Self)
            .ok()
    }
}

#[cfg(feature = "cosmwasm")]
pub mod proto {
    use crate::google::protobuf::duration::{Duration, DurationError};

    impl From<Duration> for protos::google::protobuf::Duration {
        fn from(value: Duration) -> Self {
            Self {
                seconds: value.seconds().inner(),
                nanos: value.nanos().inner(),
            }
        }
    }

    impl TryFrom<protos::google::protobuf::Duration> for Duration {
        type Error = DurationError;

        fn try_from(value: protos::google::protobuf::Duration) -> Result<Self, Self::Error> {
            Self::new(value.seconds, value.nanos)
        }
    }
}

#[cfg(test)]
mod tests {
    use core::cmp::Ordering;

    use super::*;
    use crate::test_utils::{assert_json_roundtrip, assert_proto_roundtrip};

    #[test]
    fn ord() {
        let a = Duration::new(1, 0).unwrap();
        let b = Duration::new(1, 0).unwrap();
        assert_eq!(a.cmp(&b), Ordering::Equal);

        let a = Duration::new(-1, 0).unwrap();
        let b = Duration::new(1, 0).unwrap();
        assert_eq!(a.cmp(&b), Ordering::Less);

        let a = Duration::new(1, 0).unwrap();
        let b = Duration::new(-1, 0).unwrap();
        assert_eq!(a.cmp(&b), Ordering::Greater);

        let a = Duration::new(1, 1).unwrap();
        let b = Duration::new(1, 1).unwrap();
        assert_eq!(a.cmp(&b), Ordering::Equal);

        let a = Duration::new(1, 1).unwrap();
        let b = Duration::new(1, 2).unwrap();
        assert_eq!(a.cmp(&b), Ordering::Less);

        let a = Duration::new(1, 1).unwrap();
        let b = Duration::new(2, 2).unwrap();
        assert_eq!(a.cmp(&b), Ordering::Less);

        let a = Duration::new(1, 1).unwrap();
        let b = Duration::new(2, 0).unwrap();
        assert_eq!(a.cmp(&b), Ordering::Less);

        let a = Duration::new(-1, -1).unwrap();
        let b = Duration::new(1, 1).unwrap();
        assert_eq!(a.cmp(&b), Ordering::Less);

        let a = Duration::new(1, 1).unwrap();
        let b = Duration::new(-1, -1).unwrap();
        assert_eq!(a.cmp(&b), Ordering::Greater);
    }

    #[test]
    fn string_roundtrip() {
        "-315576000000.999999999s".parse::<Duration>().unwrap();
        "-315576000000.9999s".parse::<Duration>().unwrap();
        "-0.999999999s".parse::<Duration>().unwrap();
    }

    #[test]
    fn serde_roundtrip() {
        assert_json_roundtrip(&Duration::new(123, 456).unwrap());
    }

    #[test]
    fn proto_roundtrip() {
        assert_proto_roundtrip(&Duration::new(-789, -101_112).unwrap());
    }

    #[test]
    fn checked_add() {
        let ensure_commutative = |a: Duration, b: Duration| {
            let ab = a.checked_add(b).unwrap();
            let ba = b.checked_add(a).unwrap();
            let n_ab = -((-a).checked_add(-b).unwrap());
            let n_ba = -((-b).checked_add(-a).unwrap());

            // ensure commutativity
            assert_eq!(ab, ba);
            assert_eq!(ab, n_ab);
            assert_eq!(n_ab, n_ba);
            assert_eq!(n_ab, ba);
        };

        ensure_commutative(
            Duration::from_str("1s").unwrap(),
            Duration::from_str("-0.999999999s").unwrap(),
        );

        ensure_commutative(
            Duration::from_str("-0.253480778s").unwrap(),
            Duration::from_str("0.25766784s").unwrap(),
        );
    }
}
