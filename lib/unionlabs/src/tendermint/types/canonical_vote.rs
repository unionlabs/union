use macros::model;

use crate::{
    bounded::BoundedI64,
    google::protobuf::timestamp::Timestamp,
    tendermint::types::{canonical_block_id::CanonicalBlockId, signed_msg_type::SignedMsgType},
};

#[model(proto(raw(protos::tendermint::types::LegacyCanonicalVote), from))]
pub struct CanonicalVote {
    /// type alias for byte
    pub ty: SignedMsgType,
    /// canonicalization requires fixed size encoding here
    pub height: BoundedI64<0, { i64::MAX }>,
    /// canonicalization requires fixed size encoding here
    pub round: BoundedI64<0, { i64::MAX }>,
    pub block_id: CanonicalBlockId,
    pub chain_id: String,
    pub timestamp: Timestamp,
}

#[cfg(feature = "proto")]
impl From<CanonicalVote> for protos::tendermint::types::LegacyCanonicalVote {
    fn from(value: CanonicalVote) -> Self {
        Self {
            r#type: value.ty.into(),
            height: value.height.into(),
            round: value.round.into(),
            block_id: Some(value.block_id.into()),
            chain_id: value.chain_id,
            timestamp: Some(value.timestamp.into()),
        }
    }
}
