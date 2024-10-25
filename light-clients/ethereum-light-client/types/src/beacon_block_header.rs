use serde::{Deserialize, Serialize};
use ssz::Ssz;

use crate::{errors::InvalidLength, hash::H256};

// TODO: Ssz encoding doesn't need to take ownership, impl for &T as well as T
// TODO: Impl this via #[model]
impl crate::encoding::Decode<crate::encoding::Ssz> for BeaconBlockHeader {
    type Error = ssz::decode::DecodeError;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        <Self as ssz::Ssz>::from_ssz_bytes(bytes)
    }
}

// TODO: Impl this via #[model]
impl crate::encoding::Encode<crate::encoding::Ssz> for BeaconBlockHeader {
    fn encode(self) -> Vec<u8> {
        self.as_ssz_bytes()
    }
}

impl From<BeaconBlockHeader> for protos::union::ibc::lightclients::ethereum::v1::BeaconBlockHeader {
    fn from(value: BeaconBlockHeader) -> Self {
        Self {
            slot: value.slot,
            proposer_index: value.proposer_index,
            parent_root: value.parent_root.get().into(),
            state_root: value.state_root.get().into(),
            body_root: value.body_root.get().into(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromBeaconBlockHeaderError {
    #[error("invalid `parent_root`")]
    ParentRoot(#[source] InvalidLength),
    #[error("invalid `state_root`")]
    StateRoot(#[source] InvalidLength),
    #[error("invalid `body_root`")]
    BodyRoot(#[source] InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::BeaconBlockHeader>
    for BeaconBlockHeader
{
    type Error = TryFromBeaconBlockHeaderError;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::BeaconBlockHeader,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            slot: value.slot,
            proposer_index: value.proposer_index,
            parent_root: value
                .parent_root
                .try_into()
                .map_err(TryFromBeaconBlockHeaderError::ParentRoot)?,
            state_root: value
                .state_root
                .try_into()
                .map_err(TryFromBeaconBlockHeaderError::StateRoot)?,
            body_root: value
                .body_root
                .try_into()
                .map_err(TryFromBeaconBlockHeaderError::BodyRoot)?,
        })
    }
}
