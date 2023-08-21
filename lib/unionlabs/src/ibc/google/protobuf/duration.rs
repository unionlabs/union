use core::ops::Add;

use serde::{Deserialize, Serialize};

use super::timestamp::Timestamp;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Duration {
    pub seconds: i64,
    pub nanos: i32,
}

impl Add for Duration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            seconds: self.seconds + rhs.seconds,
            nanos: self.nanos + rhs.nanos,
        }
    }
}

impl From<Timestamp> for Duration {
    fn from(value: Timestamp) -> Self {
        Self {
            seconds: value.seconds.inner(),
            nanos: value.nanos.inner(),
        }
    }
}

impl From<Duration> for protos::google::protobuf::Duration {
    fn from(value: Duration) -> Self {
        Self {
            seconds: value.seconds,
            nanos: value.nanos,
        }
    }
}

impl From<protos::google::protobuf::Duration> for Duration {
    fn from(value: protos::google::protobuf::Duration) -> Self {
        Self {
            seconds: value.seconds,
            nanos: value.nanos,
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<Duration> for contracts::glue::GoogleProtobufDurationData {
    fn from(value: Duration) -> Self {
        Self {
            seconds: value.seconds,
            nanos: value.nanos,
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<contracts::glue::GoogleProtobufDurationData> for Duration {
    fn from(value: contracts::glue::GoogleProtobufDurationData) -> Self {
        Self {
            seconds: value.seconds,
            nanos: value.nanos,
        }
    }
}
