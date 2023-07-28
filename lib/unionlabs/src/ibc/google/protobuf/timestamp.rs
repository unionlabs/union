#[derive(Clone, PartialEq)]
pub struct Timestamp {
    pub seconds: i64,
    pub nanos: i32,
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
            // REVIEW(benluelo): Is this conversion *actually* fallible?
            nanos: value.nanos.try_into().unwrap(),
        }
    }
}
