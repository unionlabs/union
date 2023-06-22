#[derive(Debug, Clone)]
pub struct Fork {
    pub version: Vec<u8>,
    pub epoch: u64,
}

impl From<Fork> for protos::union::ibc::lightclients::ethereum::v1::Fork {
    fn from(value: Fork) -> Self {
        Self {
            version: value.version,
            epoch: value.epoch,
        }
    }
}

impl From<protos::union::ibc::lightclients::ethereum::v1::Fork> for Fork {
    fn from(proto: protos::union::ibc::lightclients::ethereum::v1::Fork) -> Self {
        Self {
            version: proto.version,
            epoch: proto.epoch,
        }
    }
}
