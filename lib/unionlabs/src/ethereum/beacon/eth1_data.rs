use macros::model;
use ssz::Ssz;

use crate::hash::H256;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#eth1data>
#[model]
#[derive(Ssz)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct Eth1Data {
    pub deposit_root: H256,
    #[serde(with = "::serde_utils::string")]
    pub deposit_count: u64,
    pub block_hash: H256,
}
