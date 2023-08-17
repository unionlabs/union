use serde::{Deserialize, Serialize};

use super::timestamp::Timestamp;

const NANOS_PER_SEC: i32 = 1_000_000_000;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Duration {
    pub seconds: i64,
    pub nanos: i32,
}

impl Duration {
    #[must_use]
    pub const fn checked_add(self, rhs: Duration) -> Option<Duration> {
        if let Some(mut seconds) = self.seconds.checked_add(rhs.seconds) {
            let mut nanos = self.nanos + rhs.nanos;
            if nanos >= NANOS_PER_SEC {
                nanos -= NANOS_PER_SEC;
                if let Some(new_secs) = seconds.checked_add(1) {
                    seconds = new_secs;
                } else {
                    return None;
                }
            }
            Some(Duration { seconds, nanos })
        } else {
            None
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
