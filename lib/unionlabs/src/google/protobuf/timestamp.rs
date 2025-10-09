use core::{cmp::Ordering, fmt::Display, num::TryFromIntError, ops::Neg, str::FromStr};

use chrono::{DateTime, NaiveDateTime, SecondsFormat, TimeZone, Utc};
use macros::model;
use serde::{
    Deserialize, Serialize,
    de::{self, Unexpected},
};

use crate::{
    bounded::{BoundedI32, BoundedI64, BoundedI128, BoundedIntError},
    constants::metric::NANOS_PER_SECOND,
    google::protobuf::duration::Duration,
    result_unwrap,
};

/// See <https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=c27d92ace805175896bb68664bb492b6>
pub const TIMESTAMP_SECONDS_MAX: i64 = 253_402_300_799;
pub const TIMESTAMP_SECONDS_MIN: i64 = -62_135_596_800;

const NANOS_MAX: i32 = NANOS_PER_SECOND - 1;

pub type ValidTimestampUnixNanos = BoundedI128<
    { TIMESTAMP_SECONDS_MIN as i128 * NANOS_PER_SECOND as i128 },
    { (TIMESTAMP_SECONDS_MAX as i128 * NANOS_PER_SECOND as i128) + NANOS_MAX as i128 },
>;

#[model(proto(raw(protos::google::protobuf::Timestamp), into, from), no_serde)]
#[debug("Timestamp({})", self)]
#[derive(Copy)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Timestamp {
    /// As per the proto docs: "Must be from 0001-01-01T00:00:00Z to
    /// 9999-12-31T23:59:59Z inclusive."
    pub seconds: BoundedI64<TIMESTAMP_SECONDS_MIN, TIMESTAMP_SECONDS_MAX>,
    // As per the proto docs: "Must be from 0 to 999,999,999 inclusive."
    pub nanos: BoundedI32<0, NANOS_MAX>,
}

impl Default for Timestamp {
    fn default() -> Self {
        MIN_TIMESTAMP
    }
}

pub const MIN_TIMESTAMP: Timestamp = Timestamp {
    seconds: result_unwrap!(
        BoundedI64::<TIMESTAMP_SECONDS_MIN, TIMESTAMP_SECONDS_MAX>::new_const(
            TIMESTAMP_SECONDS_MIN
        )
    ),
    nanos: result_unwrap!(BoundedI32::<0, NANOS_MAX>::new_const(0)),
};

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

#[derive(Serialize, Deserialize)]
#[serde(rename = "Timestamp")]
struct TimestampSerde {
    pub seconds: BoundedI64<TIMESTAMP_SECONDS_MIN, TIMESTAMP_SECONDS_MAX>,
    pub nanos: BoundedI32<0, NANOS_MAX>,
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            TimestampSerde {
                seconds: self.seconds,
                nanos: self.nanos,
            }
            .serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            String::deserialize(deserializer).and_then(|str| {
                str.parse().map_err(|_| {
                    de::Error::invalid_value(Unexpected::Str(&str), &"a valid RFC 3339 string")
                })
            })
        } else {
            let TimestampSerde { seconds, nanos } = TimestampSerde::deserialize(deserializer)?;
            Ok(Self { seconds, nanos })
        }
    }
}

impl Timestamp {
    #[must_use]
    pub fn try_from_unix_nanos(nanos: i128) -> Option<Self> {
        Some(Self {
            seconds: i64::try_from(nanos / i128::from(NANOS_PER_SECOND))
                .ok()?
                .try_into()
                .ok()?,
            nanos: i32::try_from(nanos % i128::from(NANOS_PER_SECOND))
                .ok()?
                .try_into()
                .ok()?,
        })
    }

    #[must_use]
    #[allow(clippy::missing_panics_doc)] // panics are impossible
    pub fn from_unix_nanos(nanos: ValidTimestampUnixNanos) -> Self {
        Self {
            seconds: i64::try_from(nanos.inner() / i128::from(NANOS_PER_SECOND))
                .expect("type is bounded; qed;")
                .try_into()
                .expect("type is bounded; qed;"),
            nanos: i32::try_from(nanos.inner() % i128::from(NANOS_PER_SECOND))
                .expect("type is bounded; qed;")
                .try_into()
                .expect("type is bounded; qed;"),
        }
    }

    /// Returns the timestamp as unix timestamp in nanoseconds.
    #[allow(clippy::missing_panics_doc)] // panics are impossible
    #[must_use]
    pub fn as_unix_nanos(&self) -> u64 {
        u64::try_from(self.seconds.inner()).expect("impossible") * 1_000_000_000
            + u64::try_from(self.nanos.inner()).expect("impossible")
    }

    /// Returns the duration between `self` and `other`. If `self` > `other`, the
    /// resulting [`Duration`] will be positive, and if `other` > `self` then the
    /// resulting [`Duration`] will be negative.
    #[must_use]
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

    #[must_use]
    pub fn checked_add(&self, duration: Duration) -> Option<Timestamp> {
        let mut seconds = self
            .seconds
            .inner()
            .checked_add(duration.seconds().inner())?;

        // No need to do checked_add here since MAX and MIN values of this
        // addition is within the bounds of i32
        let mut nanos = self.nanos.inner() + duration.nanos().inner();

        if nanos < 0 {
            nanos += NANOS_MAX + 1;
            seconds -= 1;
        } else if nanos > NANOS_MAX {
            // Subtract instead of mod since we know that NANOS cannot be greater
            // than 2 * NANOS_MAX;
            nanos -= NANOS_MAX + 1;
            seconds += 1;
        }

        match (seconds.try_into(), nanos.try_into()) {
            (Ok(seconds), Ok(nanos)) => Some(Timestamp { seconds, nanos }),
            _ => None,
        }
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&DateTime::<Utc>::from(*self).to_rfc3339_opts(
            SecondsFormat::Nanos,
            // use_z
            true,
        ))
    }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(value: Timestamp) -> Self {
        <DateTime<Utc>>::from(&value)
    }
}

impl From<&Timestamp> for DateTime<Utc> {
    fn from(value: &Timestamp) -> Self {
        DateTime::from_timestamp(
            value.seconds.inner(),
            value
                .nanos
                .inner()
                .try_into()
                .expect("nanos bounds are within the bounds of u32; qed;"),
        )
        .expect("values are within bounds; qed;")
    }
}

#[derive(Debug)]
pub enum TryFromDateTimeError {
    Seconds(BoundedIntError<i64>),
}

impl<Tz: TimeZone> TryFrom<DateTime<Tz>> for Timestamp {
    type Error = TryFromDateTimeError;

    fn try_from(value: DateTime<Tz>) -> Result<Self, Self::Error> {
        let mut seconds = value.timestamp();
        let mut nanos: i32 = value.timestamp_subsec_nanos().try_into().expect(
            "timestamp_subsec_nanos returns a value in 0..=1_999_999_999, which is in range of i32; qed;",
        );

        if nanos >= NANOS_PER_SECOND {
            nanos -= NANOS_PER_SECOND;

            debug_assert!(NaiveDateTime::MAX.and_utc().timestamp() < i64::MAX);

            // REVIEW: is this expected behaviour for leap seconds? The proto docs
            // mention [smear](https://developers.google.com/time/smear) but I'm
            // not sure what to do with potential leap seconds in this context,
            // especially since chrono doesn't make any guarantees about when or
            // where they will fall (i.e. any value in 0..=1_999_999_999 is a valid
            // nanos value).
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

#[derive(Debug)]
pub enum TimestampFromStrError {
    Parse(chrono::ParseError),
    OutOfRange(TryFromDateTimeError),
}

impl FromStr for Timestamp {
    type Err = TimestampFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        DateTime::parse_from_rfc3339(s)
            .map_err(TimestampFromStrError::Parse)
            .and_then(|dt| dt.try_into().map_err(TimestampFromStrError::OutOfRange))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TryFromCosmwasmTimestampError {
    #[error("invalid seconds")]
    Seconds(#[source] BoundedIntError<i64>),
    #[error("invalid seconds")]
    Nanos(#[source] BoundedIntError<i32>),
    #[error("cosmwasm_std::Timestamp::seconds value could not be converted to an i64")]
    SecondsCast(TryFromIntError),
}

#[cfg(feature = "cosmwasm")]
impl TryFrom<cosmwasm_std::Timestamp> for Timestamp {
    type Error = TryFromCosmwasmTimestampError;

    fn try_from(value: cosmwasm_std::Timestamp) -> Result<Self, Self::Error> {
        const NANOS_EXPECT_MSG: &str = "cosmwasm_std::Timestamp::subsec_nanos returns a value mod 1_000_000_000, this should be infallible; qed;";

        Ok(Self {
            seconds: i64::try_from(value.seconds())
                .map_err(TryFromCosmwasmTimestampError::SecondsCast)?
                .try_into()
                .map_err(TryFromCosmwasmTimestampError::Seconds)?,
            nanos: i32::try_from(value.subsec_nanos())
                .expect(NANOS_EXPECT_MSG)
                .try_into()
                .expect(NANOS_EXPECT_MSG),
        })
    }
}

#[cfg(feature = "cosmwasm")]
impl From<Timestamp> for cosmwasm_std::Timestamp {
    fn from(value: Timestamp) -> Self {
        // TODO(benluelo): Yes this needs to be a fallible conversion, unwrap in the application code if this invariant is upheld elsewhere
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

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromTimestampError {
    #[error("invalid seconds")]
    Seconds(#[source] BoundedIntError<i64>),
    #[error("invalid nanos")]
    Nanos(#[source] BoundedIntError<i32>),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::assert_string_roundtrip;

    macro_rules! ts {
        ($s:expr, $n:expr) => {
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

    #[test]
    fn parse() {
        Timestamp::from_str("2017-01-15T01:30:15.03441Z").unwrap();

        assert_string_roundtrip(&ts!(12345, 6789));

        Timestamp::from_str("0001-01-01T00:00:00Z").unwrap();

        assert_string_roundtrip(&ts!(TIMESTAMP_SECONDS_MIN, 0));
    }

    #[test]
    fn timestamp_duration_arithmetic() {
        // (timestamp.seconds, timestamp.nanos) + (duration) = (timestamp)
        let test_items = [
            // Simple sum
            (
                (100_231_231, 1000),
                (100_000_000, 12),
                Some((200_231_231, 1012)),
            ),
            // Duration contains negative values
            (
                (100_231_111, 2312),
                (-100_000, -12),
                Some((100_131_111, 2300)),
            ),
            // Nanos carry 1 to seconds when the sum > MAX
            (
                (1_234, 100_000_000),
                (1_000, NANOS_MAX - 80_000_000),
                Some((2_235, 19_999_999)),
            ),
            // Nanos carry 1 to seconds when the sum == MAX
            ((1_234, 100_000_000), (1_000, 900_000_000), Some((2_235, 0))),
            // Seconds -1 when nanos < 0
            (
                (1_234, 100_000_000),
                (-1_000, -900_000_000),
                Some((233, 200_000_000)),
            ),
            // None when seconds is not within the bounds
            ((1_234, 0), (TIMESTAMP_SECONDS_MIN - 1_235, 0), None),
            // None when carry from nanos causes seconds to be out of bounds
            (
                (-TIMESTAMP_SECONDS_MIN, 0),
                (2 * TIMESTAMP_SECONDS_MIN, -1),
                None,
            ),
        ];

        for items in test_items {
            assert_eq!(
                ts!(items.0.0, items.0.1).checked_add(Duration::new(items.1.0, items.1.1).unwrap()),
                items.2.map(|(seconds, nanos)| Timestamp {
                    seconds: seconds.try_into().unwrap(),
                    nanos: nanos.try_into().unwrap()
                })
            );
        }
    }

    #[test]
    fn try_from_unix_nanos() {
        assert_eq!(Timestamp::try_from_unix_nanos(1), Some(ts!(0, 1)));

        assert_eq!(
            Timestamp::try_from_unix_nanos(NANOS_PER_SECOND.into()),
            Some(ts!(1, 0))
        );

        assert_eq!(
            Timestamp::try_from_unix_nanos(
                i128::from(TIMESTAMP_SECONDS_MAX) * i128::from(NANOS_PER_SECOND)
            ),
            Some(ts!(TIMESTAMP_SECONDS_MAX, 0))
        );

        assert_eq!(
            Timestamp::try_from_unix_nanos(
                i128::from(TIMESTAMP_SECONDS_MIN) * i128::from(NANOS_PER_SECOND)
            ),
            Some(ts!(TIMESTAMP_SECONDS_MIN, 0))
        );

        assert_eq!(
            Timestamp::try_from_unix_nanos(
                i128::from(TIMESTAMP_SECONDS_MAX + 1) * i128::from(NANOS_PER_SECOND)
            ),
            None
        );

        assert_eq!(
            Timestamp::try_from_unix_nanos(
                i128::from(TIMESTAMP_SECONDS_MIN - 1) * i128::from(NANOS_PER_SECOND)
            ),
            None
        );
    }

    #[test]
    fn from_unix_nanos() {
        assert_eq!(Timestamp::from_unix_nanos(1.try_into().unwrap()), ts!(0, 1));

        assert_eq!(
            Timestamp::from_unix_nanos(i128::from(NANOS_PER_SECOND).try_into().unwrap()),
            ts!(1, 0)
        );

        assert_eq!(
            Timestamp::from_unix_nanos(
                (i128::from(TIMESTAMP_SECONDS_MAX) * i128::from(NANOS_PER_SECOND))
                    .try_into()
                    .unwrap()
            ),
            ts!(TIMESTAMP_SECONDS_MAX, 0)
        );

        assert_eq!(
            Timestamp::from_unix_nanos(
                (i128::from(TIMESTAMP_SECONDS_MIN) * i128::from(NANOS_PER_SECOND))
                    .try_into()
                    .unwrap()
            ),
            ts!(TIMESTAMP_SECONDS_MIN, 0)
        );

        ValidTimestampUnixNanos::try_from(
            (i128::from(TIMESTAMP_SECONDS_MAX) + 1) * i128::from(NANOS_PER_SECOND),
        )
        .unwrap_err();

        ValidTimestampUnixNanos::try_from(
            (i128::from(TIMESTAMP_SECONDS_MIN) - 1) * i128::from(NANOS_PER_SECOND),
        )
        .unwrap_err();
    }

    #[test]
    #[cfg(feature = "cosmwasm")]
    fn cosmwasm_timestamp() {
        use cosmwasm_std::Timestamp as CwTs;

        let cw_ts = CwTs::from_nanos(
            ((123_456 * i64::from(NANOS_PER_SECOND)) + 100)
                .try_into()
                .unwrap(),
        );

        let ts = Timestamp::try_from(cw_ts).unwrap();

        assert_eq!(ts, ts!(123_456, 100));
    }
}
