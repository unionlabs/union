use serde::{Deserialize, Serialize};

use crate::{
    bounded_int::BoundedI64,
    tendermint::types::{canonical_block_id::CanonicalBlockId, signed_msg_type::SignedMsgType},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalVote {
    /// type alias for byte
    pub ty: SignedMsgType,
    /// canonicalization requires fixed size encoding here
    pub height: BoundedI64<0, { i64::MAX }>,
    /// canonicalization requires fixed size encoding here
    pub round: BoundedI64<0, { i64::MAX }>,
    pub block_id: CanonicalBlockId,
    pub chain_id: String,
}
