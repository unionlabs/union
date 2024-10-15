use macros::model;

use crate::{ethereum::Domain, hash::H256};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signingdata>
#[model]
#[cfg_attr(feature = "ssz", derive(::ssz::Ssz))]
pub struct SigningData {
    pub object_root: H256,
    pub domain: Domain,
}
