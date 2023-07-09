use serde::{Deserialize, Serialize};
use ssz::{Decode, Encode};

use crate::{
    errors::{InvalidLength, MissingField},
    ethereum::Version,
    ibc::lightclients::ethereum::fork::Fork,
    IntoProto, TryFromProto, TryFromProtoErrorOf, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
pub struct ForkParameters {
    pub genesis_fork_version: Version,
    pub genesis_slot: u64,
    pub altair: Fork,
    pub bellatrix: Fork,
    pub capella: Fork,
    pub eip4844: Fork,
}

impl From<ForkParameters> for protos::union::ibc::lightclients::ethereum::v1::ForkParameters {
    fn from(value: ForkParameters) -> Self {
        Self {
            genesis_fork_version: value.genesis_fork_version.into(),
            genesis_slot: value.genesis_slot,
            altair: Some(value.altair.into()),
            bellatrix: Some(value.bellatrix.into()),
            capella: Some(value.capella.into()),
            eip4844: Some(value.eip4844.into()),
        }
    }
}

#[derive(Debug)]
pub enum TryFromForkParametersError {
    MissingField(MissingField),
    InvalidLength(InvalidLength),
    Fork(TryFromProtoErrorOf<Fork>),
}

impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::ForkParameters> for ForkParameters {
    type Error = TryFromForkParametersError;

    fn try_from(
        proto: protos::union::ibc::lightclients::ethereum::v1::ForkParameters,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            genesis_fork_version: proto
                .genesis_fork_version
                .try_into()
                .map_err(TryFromForkParametersError::InvalidLength)?,
            genesis_slot: proto.genesis_slot,
            altair: proto
                .altair
                .ok_or(TryFromForkParametersError::MissingField(MissingField(
                    "altair",
                )))?
                .try_into()
                .map_err(TryFromForkParametersError::Fork)?,
            bellatrix: proto
                .bellatrix
                .ok_or(TryFromForkParametersError::MissingField(MissingField(
                    "bellatrix",
                )))?
                .try_into()
                .map_err(TryFromForkParametersError::Fork)?,
            capella: proto
                .capella
                .ok_or(TryFromForkParametersError::MissingField(MissingField(
                    "capella",
                )))?
                .try_into()
                .map_err(TryFromForkParametersError::Fork)?,
            eip4844: proto
                .eip4844
                .ok_or(TryFromForkParametersError::MissingField(MissingField(
                    "eip4844",
                )))?
                .try_into()
                .map_err(TryFromForkParametersError::Fork)?,
        })
    }
}

impl TypeUrl for protos::union::ibc::lightclients::ethereum::v1::ForkParameters {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.ethereum.v1.ForkParameters";
}

impl IntoProto for ForkParameters {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::ForkParameters;
}

impl TryFromProto for ForkParameters {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::ForkParameters;
}
