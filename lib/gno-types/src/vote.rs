use serde::{Deserialize, Serialize};
use unionlabs::{
    bounded::{BoundedI32, BoundedI64},
    google::protobuf::timestamp::Timestamp,
    primitives::{Bech32, Bytes, H160, encoding::Base64},
};

use crate::{BlockId, SignedMsgType};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Vote {
    #[serde(rename = "type")]
    pub ty: SignedMsgType,
    #[serde(with = "::serde_utils::string")]
    pub height: BoundedI64<0, { i64::MAX }>,
    #[serde(with = "::serde_utils::string")]
    pub round: BoundedI32<0, { i32::MAX }>,
    pub block_id: BlockId,
    pub timestamp: Timestamp,
    pub validator_address: Bech32<H160>,
    #[serde(with = "::serde_utils::string")]
    pub validator_index: i32,
    pub signature: Bytes<Base64>,
}

impl Vote {
    fn SignBytes(&self, chainID: string) -> Bytes {
        amino.MarshalSized(CanonicalizeVote(chainID, vote))
    }
}
