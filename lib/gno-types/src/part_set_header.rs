use serde::{Deserialize, Serialize};
use unionlabs::primitives::{H256, encoding::Base64};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct PartSetHeader {
    #[serde(with = "::serde_utils::string")]
    pub total: u32,
    /// Hash of the previous block. This is only None on block 1, as the genesis block does not have a hash.
    pub hash: Option<H256<Base64>>,
}
