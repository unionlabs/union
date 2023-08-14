use serde::{Deserialize, Serialize};

use crate::{Proto, TypeUrl};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Timestamp {
    pub seconds: i64,
    pub nanos: i32,
}

impl Proto for Timestamp {
    type Proto = protos::google::protobuf::Timestamp;
}

impl TypeUrl for protos::google::protobuf::Timestamp {
    const TYPE_URL: &'static str = "/google.protobuf.Timestamp";
}

impl From<Timestamp> for protos::google::protobuf::Timestamp {
    fn from(value: Timestamp) -> Self {
        Self {
            seconds: value.seconds,
            nanos: value.nanos,
        }
    }
}

impl From<protos::google::protobuf::Timestamp> for Timestamp {
    fn from(value: protos::google::protobuf::Timestamp) -> Self {
        Self {
            seconds: value.seconds,
            nanos: value.nanos,
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<Timestamp> for contracts::glue::GoogleProtobufTimestampData {
    fn from(value: Timestamp) -> Self {
        Self {
            secs: value.seconds,
            nanos: value.nanos.into(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<contracts::glue::GoogleProtobufTimestampData> for Timestamp {
    fn from(value: contracts::glue::GoogleProtobufTimestampData) -> Self {
        Self {
            seconds: value.secs,
            // REVIEW(benluelo): Is this conversion *actually* fallible?
            // As per the proto docs: "Must be from 0 to 999,999,999 inclusive."
            nanos: value.nanos.try_into().unwrap(),
        }
    }
}
