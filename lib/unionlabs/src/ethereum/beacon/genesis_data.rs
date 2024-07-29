use macros::model;

use crate::{ethereum::Version, hash::H256};

#[model]
pub struct GenesisData {
    pub genesis_validators_root: H256,
    #[serde(with = "::serde_utils::string")]
    pub genesis_time: u64,
    pub genesis_fork_version: Version,
}
