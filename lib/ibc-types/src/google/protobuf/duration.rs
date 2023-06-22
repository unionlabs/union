use contracts::glue::GoogleProtobufDurationData;

#[derive(Debug, Clone, PartialEq)]
pub struct Duration {
    pub seconds: i64,
    pub nanos: i32,
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

impl From<Duration> for GoogleProtobufDurationData {
    fn from(value: Duration) -> Self {
        Self {
            seconds: value.seconds,
            nanos: value.nanos,
        }
    }
}

impl From<GoogleProtobufDurationData> for Duration {
    fn from(value: GoogleProtobufDurationData) -> Self {
        Self {
            seconds: value.seconds,
            nanos: value.nanos,
        }
    }
}
