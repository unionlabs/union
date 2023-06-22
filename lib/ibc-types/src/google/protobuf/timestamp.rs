use contracts::glue::GoogleProtobufTimestampData;

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

impl From<Timestamp> for GoogleProtobufTimestampData {
    fn from(value: Timestamp) -> Self {
        Self {
            secs: value.seconds,
            // REVIEW(benluelo): Is this conversion *actually* fallible?
            nanos: value.nanos.try_into().unwrap(),
        }
    }
}
