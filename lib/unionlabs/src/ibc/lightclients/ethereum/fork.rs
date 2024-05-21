use macros::model;
use ssz::Ssz;

use crate::{errors::InvalidLength, ethereum::Version};

#[cfg_attr(
    feature = "ethabi",
    derive(
        ethers_contract_derive::EthAbiType,
        ethers_contract_derive::EthAbiCodec
    )
)]
// REVIEW: Are these derives used?
#[derive(Ssz)]
#[model(proto(raw(protos::union::ibc::lightclients::ethereum::v1::Fork), into, from))]
pub struct Fork {
    pub version: Version,
    pub epoch: u64,
}

impl From<Fork> for protos::union::ibc::lightclients::ethereum::v1::Fork {
    fn from(value: Fork) -> Self {
        Self {
            version: value.version.into(),
            epoch: value.epoch,
        }
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum TryFromForkError {
    #[error("invalid version")]
    Version(#[source] InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::Fork> for Fork {
    type Error = TryFromForkError;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::Fork,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            version: value
                .version
                .try_into()
                .map_err(TryFromForkError::Version)?,
            epoch: value.epoch,
        })
    }
}
