use crate::{errors::MissingField, lightclients::ethereum::fork::Fork};

#[derive(Debug, Clone)]
pub struct ForkParameters {
    // REVIEW(benluelo): Are these versions a fixed-length array? (in Fork as well)
    pub genesis_fork_version: Vec<u8>,
    pub genesis_slot: u64,
    pub altair: Fork,
    pub bellatrix: Fork,
    pub capella: Fork,
    pub eip4844: Fork,
}

impl From<ForkParameters> for protos::union::ibc::lightclients::ethereum::v1::ForkParameters {
    fn from(value: ForkParameters) -> Self {
        Self {
            genesis_fork_version: value.genesis_fork_version,
            genesis_slot: value.genesis_slot,
            altair: Some(value.altair.into()),
            bellatrix: Some(value.bellatrix.into()),
            capella: Some(value.capella.into()),
            eip4844: Some(value.eip4844.into()),
        }
    }
}

impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::ForkParameters> for ForkParameters {
    type Error = MissingField;

    fn try_from(
        proto: protos::union::ibc::lightclients::ethereum::v1::ForkParameters,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            genesis_fork_version: proto.genesis_fork_version,
            genesis_slot: proto.genesis_slot,
            altair: proto.altair.ok_or(MissingField("altair"))?.into(),
            bellatrix: proto.bellatrix.ok_or(MissingField("bellatrix"))?.into(),
            capella: proto.capella.ok_or(MissingField("capella"))?.into(),
            eip4844: proto.eip4844.ok_or(MissingField("eip4844"))?.into(),
        })
    }
}
