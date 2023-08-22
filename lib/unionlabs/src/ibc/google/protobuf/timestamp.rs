use core::num::TryFromIntError;
use std::{cmp::Ordering, fmt::Display, ops::Neg};

use chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    bounded_int::{BoundedI32, BoundedI64, BoundedIntError},
    ibc::google::protobuf::duration::{Duration, NANOS_PER_SECOND},
    Proto, TypeUrl,
};

/// See <https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=c27d92ace805175896bb68664bb492b6>
pub const TIMESTAMP_SECONDS_MAX: i64 = 253_402_300_799;
pub const TIMESTAMP_SECONDS_MIN: i64 = -62_135_596_800;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timestamp {
    /// As per the proto docs: "Must be from 0001-01-01T00:00:00Z to
    /// 9999-12-31T23:59:59Z inclusive."
    pub seconds: BoundedI64<TIMESTAMP_SECONDS_MIN, TIMESTAMP_SECONDS_MAX>,
    // As per the proto docs: "Must be from 0 to 999,999,999 inclusive."
    pub nanos: BoundedI32<0, 999_999_999>,
}

impl Ord for Timestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        self.seconds
            .cmp(&other.seconds)
            .then_with(|| self.nanos.cmp(&other.nanos))
    }
}

impl PartialOrd for Timestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Timestamp {
    #[must_use]
    /// Returns the duration between `self` and `other`. If `self` > `other`, the
    /// resulting [`Duration`] will be positive, and if `other` > `self` then the
    /// resulting [`Duration`] will be negative.
    pub fn duration_since(&self, other: &Self) -> Option<Duration> {
        match self.cmp(other) {
            Ordering::Greater => {
                let mut seconds = self.seconds.inner().checked_sub(other.seconds.inner())?;

                let nanos = if self.nanos < other.nanos {
                    seconds -= 1;

                    NANOS_PER_SECOND - (other.nanos.inner() - self.nanos.inner())
                } else {
                    self.nanos.inner() - other.nanos.inner()
                };

                Duration::new(seconds, nanos).ok()
            }
            Ordering::Equal => Duration::new(0, 0).ok(),
            Ordering::Less => other.duration_since(self).map(Neg::neg),
        }
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&DateTime::<Utc>::from(*self).to_rfc3339_opts(
            SecondsFormat::Nanos,
            // use_z
            true,
        ))
    }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(value: Timestamp) -> Self {
        DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp_opt(
                value.seconds.inner(),
                value
                    .nanos
                    .inner()
                    .try_into()
                    .expect("nanos bounds are within the bounds of u32; qed;"),
            )
            .expect("values are within bounds; qed;"),
            Utc,
        )
    }
}

#[derive(Debug)]
pub enum TryFromDateTimeError {
    Seconds(BoundedIntError<i64>),
}

impl TryFrom<DateTime<Utc>> for Timestamp {
    type Error = TryFromDateTimeError;

    fn try_from(value: DateTime<Utc>) -> Result<Self, Self::Error> {
        let mut seconds = value.timestamp();
        let mut nanos: i32 = value.timestamp_subsec_nanos().try_into().expect(
            "timestamp_subsec_nanos returns a value in 0..=1_999_999_999, which is in range of i32; qed;",
        );

        if nanos >= NANOS_PER_SECOND {
            nanos -= NANOS_PER_SECOND;

            debug_assert!(NaiveDateTime::MAX.timestamp() < i64::MAX);

            seconds += 1;
        }

        Ok(Self {
            seconds: seconds.try_into().map_err(TryFromDateTimeError::Seconds)?,
            nanos: nanos
                .try_into()
                .expect("nanos is within 0..=999_999_999; qed;"),
        })
    }
}

// String::deserialize(deserializer).and_then(|str| {
//     chrono::DateTime::parse_from_rfc3339(&str).map_err(de::Error::invalid_value(
//         de::Unexpected::Str(&str),
//         &"an RFC 3339 datetime string",
//     ))
// }).map(|dt| dt.naive_utc())

impl Proto for Timestamp {
    type Proto = protos::google::protobuf::Timestamp;
}

impl TypeUrl for protos::google::protobuf::Timestamp {
    const TYPE_URL: &'static str = "/google.protobuf.Timestamp";
}

#[derive(Debug)]
pub enum TryFromCosmwasmTimestampError {
    Seconds(BoundedIntError<i64>),
    Nanos(BoundedIntError<i32>),
    IntCast(TryFromIntError),
}

impl TryFrom<cosmwasm_std::Timestamp> for Timestamp {
    type Error = TryFromCosmwasmTimestampError;

    fn try_from(value: cosmwasm_std::Timestamp) -> Result<Self, Self::Error> {
        Ok(Self {
            seconds: TryInto::<i64>::try_into(value.seconds())
                .map_err(TryFromCosmwasmTimestampError::IntCast)?
                .try_into()
                .map_err(TryFromCosmwasmTimestampError::Seconds)?,
            nanos: TryInto::<i32>::try_into(value.nanos())
                .map_err(TryFromCosmwasmTimestampError::IntCast)?
                .try_into()
                .map_err(TryFromCosmwasmTimestampError::Nanos)?,
        })
    }
}

impl From<Timestamp> for cosmwasm_std::Timestamp {
    fn from(value: Timestamp) -> Self {
        // REVIEW(aeryz): I always expect timestamp to be non-negative integer, that's
        // why `unwrap`ping seems like the right way to go, please give me a heads up
        // if there is an exception and we should convert this implementation to
        // `TryFrom` instead.
        cosmwasm_std::Timestamp::from_seconds(
            value
                .seconds
                .inner()
                .try_into()
                .expect("impossible since this is always inbounds"),
        )
        .plus_nanos(
            value
                .nanos
                .inner()
                .try_into()
                .expect("impossible since this is always inbounds"),
        )
    }
}

impl From<Timestamp> for protos::google::protobuf::Timestamp {
    fn from(value: Timestamp) -> Self {
        Self {
            seconds: value.seconds.into(),
            nanos: value.nanos.into(),
        }
    }
}

#[derive(Debug)]
pub enum TryFromTimestampError {
    Seconds(BoundedIntError<i64>),
    Nanos(BoundedIntError<i32>),
}

impl TryFrom<protos::google::protobuf::Timestamp> for Timestamp {
    type Error = TryFromTimestampError;

    fn try_from(value: protos::google::protobuf::Timestamp) -> Result<Self, Self::Error> {
        Ok(Self {
            seconds: value
                .seconds
                .try_into()
                .map_err(TryFromTimestampError::Seconds)?,
            nanos: value
                .nanos
                .try_into()
                .map_err(TryFromTimestampError::Nanos)?,
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<Timestamp> for contracts::glue::GoogleProtobufTimestampData {
    fn from(value: Timestamp) -> Self {
        Self {
            secs: value.seconds.into(),
            nanos: value.nanos.inner().into(),
        }
    }
}

#[cfg(feature = "ethabi")]
#[derive(Debug)]
pub enum TryFromEthAbiTimestampError {
    Seconds(BoundedIntError<i64>),
    Nanos(BoundedIntError<i32>),
    NanosTryFromI64(std::num::TryFromIntError),
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::glue::GoogleProtobufTimestampData> for Timestamp {
    type Error = TryFromEthAbiTimestampError;

    fn try_from(value: contracts::glue::GoogleProtobufTimestampData) -> Result<Self, Self::Error> {
        Ok(Self {
            seconds: value
                .secs
                .try_into()
                .map_err(TryFromEthAbiTimestampError::Seconds)?,
            nanos: i32::try_from(value.nanos)
                .map_err(TryFromEthAbiTimestampError::NanosTryFromI64)?
                .try_into()
                .map_err(TryFromEthAbiTimestampError::Nanos)?,
        })
    }
}

#[cfg(feature = "ethabi")]
impl crate::EthAbi for Timestamp {
    type EthAbi = contracts::glue::GoogleProtobufTimestampData;
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! ts {
        ($s:literal, $n:literal) => {
            Timestamp {
                seconds: BoundedI64::new($s).unwrap(),
                nanos: BoundedI32::new($n).unwrap(),
            }
        };
    }

    macro_rules! dur {
        ($s:literal, $n:literal) => {
            Duration::new($s, $n).unwrap()
        };
    }

    #[test]
    fn duration_since() {
        assert_eq!(
            ts!(1, 100_000_000).duration_since(&ts!(1, 100_000_000)),
            Some(dur!(0, 000_000_000))
        );
        assert_eq!(
            ts!(2, 100_000_000).duration_since(&ts!(1, 100_000_000)),
            Some(dur!(1, 000_000_000))
        );
        assert_eq!(
            ts!(1, 100_000_000).duration_since(&ts!(2, 100_000_000)),
            Some(dur!(-1, 000_000_000))
        );
        assert_eq!(
            ts!(1, 000_000_000).duration_since(&ts!(2, 100_000_000)),
            Some(dur!(-1, -100_000_000))
        );
        assert_eq!(
            ts!(2, 100_000_000).duration_since(&ts!(1, 000_000_000)),
            Some(dur!(1, 100_000_000))
        );
    }
}
