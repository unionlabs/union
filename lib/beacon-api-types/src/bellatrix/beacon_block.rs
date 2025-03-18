use unionlabs::primitives::H256;

use crate::{
    bellatrix::BeaconBlockBody,
    custom_types::{Slot, ValidatorIndex},
};
#[cfg(feature = "ssz")]
use crate::{bellatrix::BeaconBlockBodySsz, chain_spec::ChainSpec};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BeaconBlock {
    pub slot: Slot,
    pub proposer_index: ValidatorIndex,
    pub parent_root: H256,
    pub state_root: H256,
    pub body: BeaconBlockBody,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, Eq, ssz::Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""))
)]
pub struct BeaconBlockSsz<C: ChainSpec> {
    pub slot: Slot,
    pub proposer_index: ValidatorIndex,
    pub parent_root: H256,
    pub state_root: H256,
    pub body: BeaconBlockBodySsz<C>,
}
