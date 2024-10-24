use macros::model;

use crate::{
    bounded::{BoundedI32, BoundedI64, BoundedIntError},
    errors::{required, InvalidLength, MissingField, UnknownEnumVariant},
    google::protobuf::timestamp::{Timestamp, TryFromTimestampError},
    hash::H160,
    tendermint::types::{
        block_id::{BlockId, TryFromBlockIdError},
        signed_msg_type::SignedMsgType,
    },
};

#[model(proto(raw(protos::tendermint::types::Vote), from, into))]
pub struct Vote {
    pub ty: SignedMsgType,
    pub height: BoundedI64<0, { i64::MAX }>,
    pub round: BoundedI32<0, { i32::MAX }>,
    pub block_id: BlockId,
    pub timestamp: Timestamp,
    pub validator_address: H160,
    pub validator_index: i32,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub signature: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub extension: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub extension_signature: Vec<u8>,
}

impl From<Vote> for protos::tendermint::types::Vote {
    fn from(value: Vote) -> Self {
        Self {
            r#type: value.ty.into(),
            height: value.height.inner(),
            round: value.round.inner(),
            block_id: Some(value.block_id.into()),
            timestamp: Some(value.timestamp.into()),
            validator_address: value.validator_address.into(),
            validator_index: value.validator_index,
            signature: value.signature,
            extension: value.extension,
            extension_signature: value.extension_signature,
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromVoteError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid type")]
    Type(#[from] UnknownEnumVariant<i32>),
    #[error("invalid height")]
    Height(#[from] BoundedIntError<i64>),
    #[error("invalid round")]
    Round(#[from] BoundedIntError<i32>),
    #[error("invalid block id")]
    BlockId(#[from] TryFromBlockIdError),
    #[error("invalid timestamp")]
    Timestamp(#[from] TryFromTimestampError),
    #[error("invalid validator address")]
    ValidatorAddress(#[from] InvalidLength),
}

impl TryFrom<protos::tendermint::types::Vote> for Vote {
    type Error = TryFromVoteError;

    fn try_from(value: protos::tendermint::types::Vote) -> Result<Self, Self::Error> {
        Ok(Self {
            ty: value.r#type.try_into()?,
            height: value.height.try_into()?,
            round: value.round.try_into()?,
            block_id: required!(value.block_id)?.try_into()?,
            timestamp: required!(value.timestamp)?.try_into()?,
            validator_address: value.validator_address.try_into()?,
            validator_index: value.validator_index,
            signature: value.signature,
            extension: value.extension,
            extension_signature: value.extension_signature,
        })
    }
}
