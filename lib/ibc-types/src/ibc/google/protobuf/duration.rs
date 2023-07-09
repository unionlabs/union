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
