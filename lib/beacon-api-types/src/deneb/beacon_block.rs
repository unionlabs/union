use unionlabs::primitives::H256;

#[cfg(feature = "ssz")]
use crate::{chain_spec::ChainSpec, deneb::BeaconBlockBodySsz};
use crate::{deneb::BeaconBlockBody, slot::Slot};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BeaconBlock {
    pub slot: Slot,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub proposer_index: u64,
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
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub proposer_index: u64,
    pub parent_root: H256,
    pub state_root: H256,
    pub body: BeaconBlockBodySsz<C>,
}
