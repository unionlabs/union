use std::{
    fmt::{self, Debug, Display},
    num::ParseIntError,
    ops::Neg,
    str::FromStr,
};

use serde::{
    de::{self, Unexpected},
    Deserialize, Serialize,
};

use crate::{
    bounded_int::{BoundedI32, BoundedI64, BoundedIntError},
    Proto, TypeUrl,
};

pub const NANOS_PER_SECOND: i32 = 1_000_000_000;
pub const DURATION_MAX_SECONDS: i64 = 315_576_000_000;
pub const DURATION_MAX_NANOS: i32 = 999_999_999;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Duration(DurationImpl);

impl Debug for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Duration({self})"))
    }
}

impl Neg for Duration {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.seconds(), -self.nanos()).expect("lower bound == neg(upper bound); qed;")
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).and_then(|str| {
            str.parse().map_err(|_| {
                de::Error::invalid_value(
                    Unexpected::Str(&str),
                    &"a valid protobuf duration string (`<seconds>[.<nanos>]s`)",
                )
            })
        })
    }
}

impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

pub enum DurationFromStrError {
    /// The `s` suffix was missing.
    MissingSecondsSuffix,
    /// Either the seconds or nanos values couldn't be parsed as numbers.
    ParseInt(ParseIntError),
    /// The string was in the expected format, but the values were out of bounds.
    OutOfBounds(DurationError),
}

impl FromStr for Duration {
    type Err = DurationFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // seconds, nanos
        s.strip_suffix('s')
            .ok_or(DurationFromStrError::MissingSecondsSuffix)
            .and_then(|s| {
                match s.split_once('.') {
                    Some((seconds, nanos)) => Self::new(
                        seconds.parse().map_err(DurationFromStrError::ParseInt)?,
                        nanos.parse().map_err(DurationFromStrError::ParseInt)?,
                    ),
                    // no nanos
                    None => Self::new(s.parse().map_err(DurationFromStrError::ParseInt)?, 0),
                }
                .map_err(DurationFromStrError::OutOfBounds)
            })
    }
}

impl Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seconds = self.seconds();
        let nanos = self.nanos();
        let nanos_str = if nanos == 0 {
            String::new()
        } else {
            // SAFETY: .abs() is safe to call here since self.nanos()
            // will never be i32::MIN
            format!(".{:09}", self.nanos().abs())
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
    // NOTE: This unfortunately can't be const yet without becoming incredibly complicated
    pub fn new(seconds: i64, nanos: i32) -> Result<Self, DurationError> {
        use std::cmp::Ordering::{Equal, Greater, Less};

        Ok(Self(match (seconds.cmp(&0), nanos.cmp(&0)) {
            (Equal, _) => DurationImpl::ZeroSeconds {
                nanos: nanos.try_into().map_err(DurationError::Nanos)?,
            },
            // negative seconds, negative nanos
            (Less, Less | Equal) => DurationImpl::Negative {
                seconds: seconds.try_into().map_err(DurationError::Seconds)?,
                nanos: nanos.try_into().map_err(DurationError::Nanos)?,
            },
            // negative seconds, positive nanos
            (Less, Greater) | (Greater, Less) => return Err(DurationError::Sign),
            (Greater, Greater | Equal) => DurationImpl::Positive {
                seconds: seconds.try_into().map_err(DurationError::Seconds)?,
                nanos: nanos.try_into().map_err(DurationError::Nanos)?,
            },
        }))
    }

    #[must_use]
    pub fn seconds(&self) -> i64 {
        match self.0 {
            DurationImpl::ZeroSeconds { nanos: _ } => 0,
            DurationImpl::Positive { seconds, nanos: _ } => seconds.inner(),
            DurationImpl::Negative { seconds, nanos: _ } => seconds.inner(),
        }
    }

    #[must_use]
    pub fn nanos(&self) -> i32 {
        match self.0 {
            DurationImpl::ZeroSeconds { nanos } => nanos.inner(),
            DurationImpl::Positive { seconds: _, nanos } => nanos.inner(),
            DurationImpl::Negative { seconds: _, nanos } => nanos.inner(),
        }
    }
}

#[derive(Debug)]
pub enum DurationError {
    Seconds(BoundedIntError<i64>),
    Nanos(BoundedIntError<i32>),
    /// The nanos field was the incorrect sign.
    Sign,
}

/// # Seconds:
///
/// > Signed seconds of the span of time. Must be from -315,576,000,000
///   to +315,576,000,000 inclusive. Note: these bounds are computed from:
///   60 sec/min * 60 min/hr * 24 hr/day * 365.25 days/year * 10000 years
///
/// # Nanos:
///
/// > Signed fractions of a second at nanosecond resolution of the span
///   of time. Durations less than one second are represented with a 0
///   `seconds` field and a positive or negative `nanos` field. For durations
///   of one second or more, a non-zero value for the `nanos` field must be
///   of the same sign as the `seconds` field. Must be from -999,999,999
///   to +999,999,999 inclusive.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum DurationImpl {
    ZeroSeconds {
        // NOTE: `seconds` is zero here
        nanos: BoundedI32<{ -DURATION_MAX_NANOS }, DURATION_MAX_NANOS>,
    },
    Positive {
        seconds: BoundedI64<1, DURATION_MAX_SECONDS>,
        nanos: BoundedI32<0, DURATION_MAX_NANOS>,
    },
    Negative {
        seconds: BoundedI64<{ -DURATION_MAX_SECONDS }, -1>,
        nanos: BoundedI32<{ -DURATION_MAX_NANOS }, 0>,
    },
}

impl Ord for Duration {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.seconds()
            .cmp(&other.seconds())
            .then_with(|| self.nanos().cmp(&other.nanos()))
    }
}

impl PartialOrd for Duration {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Proto for Duration {
    type Proto = protos::google::protobuf::Duration;
}

impl TypeUrl for protos::google::protobuf::Duration {
    const TYPE_URL: &'static str = "/google.protobuf.Duration";
}

impl Duration {
    #[must_use]
    pub fn checked_add(self, rhs: Duration) -> Option<Duration> {
        const _: () = assert!(DURATION_MAX_SECONDS
            .checked_add(DURATION_MAX_SECONDS)
            .is_some());

        // No need to do checked_add here since 2 * DURATION_MAX_SECONDS doesn't overflow, see above
        let mut seconds = self.seconds() + rhs.seconds();
        let mut nanos = self.nanos() + rhs.nanos();

        if nanos >= NANOS_PER_SECOND {
            nanos -= NANOS_PER_SECOND;
            seconds += 1;
        }

        if seconds > DURATION_MAX_SECONDS {
            return None;
        }

        Some(Duration::new(seconds, nanos).expect("values are within bounds; qed;"))
    }
}

impl From<Duration> for protos::google::protobuf::Duration {
    fn from(value: Duration) -> Self {
        match value.0 {
            DurationImpl::ZeroSeconds { nanos } => Self {
                seconds: 0,
                nanos: nanos.inner(),
            },
            DurationImpl::Positive { seconds, nanos } => Self {
                seconds: seconds.inner(),
                nanos: nanos.inner(),
            },
            DurationImpl::Negative { seconds, nanos } => Self {
                seconds: seconds.inner(),
                nanos: nanos.inner(),
            },
        }
    }
}

impl TryFrom<protos::google::protobuf::Duration> for Duration {
    type Error = DurationError;

    fn try_from(value: protos::google::protobuf::Duration) -> Result<Self, Self::Error> {
        Self::new(value.seconds, value.nanos)
    }
}

#[cfg(feature = "ethabi")]
impl From<Duration> for contracts::glue::GoogleProtobufDurationData {
    fn from(value: Duration) -> Self {
        Self {
            seconds: value.seconds(),
            nanos: value.nanos(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::glue::GoogleProtobufDurationData> for Duration {
    type Error = DurationError;

    fn try_from(value: contracts::glue::GoogleProtobufDurationData) -> Result<Self, Self::Error> {
        Self::new(value.seconds, value.nanos)
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::*;

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
}
