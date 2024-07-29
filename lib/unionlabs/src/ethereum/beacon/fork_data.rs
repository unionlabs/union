use macros::model;
use ssz::Ssz;

use crate::{ethereum::Version, hash::H256};

#[model]
#[derive(Ssz)]
pub struct ForkData {
    pub current_version: Version,
    pub genesis_validators_root: H256,
}
