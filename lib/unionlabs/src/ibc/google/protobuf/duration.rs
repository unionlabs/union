use serde::{Deserialize, Serialize};

use super::timestamp::Timestamp;
use crate::{
    bounded_int::{BoundedI32, BoundedI64, BoundedIntError},
    Proto, TypeUrl,
};

const NANOS_PER_SEC: i32 = 1_000_000_000;
const SECONDS_LIMIT: i64 = 315_576_000_000;
const NANOS_LIMIT: i32 = 999_999_999;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Duration {
    pub seconds: BoundedI64<{ -SECONDS_LIMIT }, SECONDS_LIMIT>,
    pub nanos: BoundedI32<{ -NANOS_LIMIT }, NANOS_LIMIT>,
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
        // No need to do checked_add here since 2 * SECONDS_LIMIT doesn't overflow
        let mut seconds = self.seconds.inner() + rhs.seconds.inner();
        let mut nanos = self.nanos.inner() + rhs.nanos.inner();

        if nanos >= NANOS_PER_SEC {
            nanos -= NANOS_PER_SEC;
            seconds += 1;
        }

        if seconds > SECONDS_LIMIT {
            return None;
        }

        Some(Duration {
            seconds: BoundedI64::new(seconds).expect("impossible since the bounds are checked"),
            nanos: BoundedI32::new(nanos).expect("impossible since the bounds are checked"),
        })
    }
}

impl From<Timestamp> for Duration {
    fn from(value: Timestamp) -> Self {
        Self {
            seconds: BoundedI64::new(value.seconds.inner()).expect(
                "impossible since timestamp.seconds is always within the range of duration.seconds",
            ),
            nanos: BoundedI32::new(value.nanos.inner()).expect(
                "impossible since timestamp.nanos is always within the range of duration.nanos",
            ),
        }
    }
}

impl From<Duration> for protos::google::protobuf::Duration {
    fn from(value: Duration) -> Self {
        Self {
            seconds: value.seconds.inner(),
            nanos: value.nanos.inner(),
        }
    }
}

#[derive(Debug)]
pub enum TryFromDurationError {
    Seconds(BoundedIntError<i64>),
    Nanos(BoundedIntError<i32>),
}

impl TryFrom<protos::google::protobuf::Duration> for Duration {
    type Error = TryFromDurationError;

    fn try_from(value: protos::google::protobuf::Duration) -> Result<Self, Self::Error> {
        Ok(Self {
            seconds: value
                .seconds
                .try_into()
                .map_err(TryFromDurationError::Seconds)?,
            nanos: value
                .nanos
                .try_into()
                .map_err(TryFromDurationError::Nanos)?,
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<Duration> for contracts::glue::GoogleProtobufDurationData {
    fn from(value: Duration) -> Self {
        Self {
            seconds: value.seconds.inner(),
            nanos: value.nanos.inner(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::glue::GoogleProtobufDurationData> for Duration {
    type Error = TryFromDurationError;

    fn try_from(value: contracts::glue::GoogleProtobufDurationData) -> Result<Self, Self::Error> {
        Ok(Self {
            seconds: value
                .seconds
                .try_into()
                .map_err(TryFromDurationError::Seconds)?,
            nanos: value
                .nanos
                .try_into()
                .map_err(TryFromDurationError::Nanos)?,
        })
    }
}
