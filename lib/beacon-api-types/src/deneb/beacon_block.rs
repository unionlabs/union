use unionlabs::primitives::H256;

#[cfg(feature = "ssz")]
use crate::{chain_spec::ChainSpec, deneb::BeaconBlockBodySsz};
use crate::{
    custom_types::{Slot, ValidatorIndex},
    deneb::BeaconBlockBody,
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
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
    serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)
)]
pub struct BeaconBlockSsz<C: ChainSpec> {
    pub slot: Slot,
    pub proposer_index: ValidatorIndex,
    pub parent_root: H256,
    pub state_root: H256,
    pub body: BeaconBlockBodySsz<C>,
}
