use macros::model;
use ssz::{Decode, Encode, TreeHash};

use crate::{errors::InvalidLength, hash::H256};

#[derive(Encode, Decode, TreeHash)]
#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::BeaconBlockHeader),
    into,
    from
))]
pub struct BeaconBlockHeader {
    #[serde(with = "serde_utils::string")]
    pub slot: u64,
    #[serde(with = "serde_utils::string")]
    pub proposer_index: u64,
    pub parent_root: H256,
    pub state_root: H256,
    pub body_root: H256,
}

impl From<BeaconBlockHeader> for protos::union::ibc::lightclients::ethereum::v1::BeaconBlockHeader {
    fn from(value: BeaconBlockHeader) -> Self {
        Self {
            slot: value.slot,
            proposer_index: value.proposer_index,
            parent_root: value.parent_root.0.into(),
            state_root: value.state_root.0.into(),
            body_root: value.body_root.0.into(),
        }
    }
}

#[derive(Debug)]
pub enum TryFromBeaconBlockHeaderError {
    ParentRoot(InvalidLength),
    StateRoot(InvalidLength),
    BodyRoot(InvalidLength),
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
