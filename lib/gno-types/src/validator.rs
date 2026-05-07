use serde::{Deserialize, Serialize};
use unionlabs::{
    bounded::BoundedI64,
    primitives::{Bech32, H160},
};

use crate::PublicKey;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Validator {
    pub address: Bech32<H160>,
    pub pub_key: PublicKey,
    #[serde(with = "::serde_utils::string")]
    pub voting_power: BoundedI64<0, { i64::MAX }>,
    #[serde(with = "::serde_utils::string")]
    pub proposer_priority: i64,
}
