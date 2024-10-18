use macros::model;

use crate::{ethereum::Version, hash::H256};

#[model]
#[cfg_attr(feature = "ssz", derive(::ssz::Ssz))]
pub struct ForkData {
    pub current_version: Version,
    pub genesis_validators_root: H256,
}
