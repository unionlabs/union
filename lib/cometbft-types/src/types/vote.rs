use serde::{Deserialize, Serialize};
use unionlabs::{
    bounded::{BoundedI32, BoundedI64},
    google::protobuf::timestamp::Timestamp,
    primitives::{
        Bytes, H160,
        encoding::{Base64, HexUnprefixed},
    },
};

use crate::types::{block_id::BlockId, signed_msg_type::SignedMsgType};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vote {
    #[serde(rename = "type")]
    pub ty: SignedMsgType,
    #[serde(with = "::serde_utils::string")]
    pub height: BoundedI64<0, { i64::MAX }>,
    pub round: BoundedI32<0, { i32::MAX }>,
    pub block_id: BlockId,
    pub timestamp: Timestamp,
    pub validator_address: H160<HexUnprefixed>,
    pub validator_index: i32,
    pub signature: Bytes<Base64>,
    pub extension: Option<Bytes<Base64>>,
    pub extension_signature: Option<Bytes<Base64>>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{
        bounded::BoundedIntError,
        errors::{MissingField, UnknownEnumVariant},
        google::protobuf::timestamp::TryFromTimestampError,
        primitives::FixedBytesError,
        required,
    };

    use crate::types::{block_id, vote::Vote};

    impl From<Vote> for protos::cometbft::types::v1::Vote {
        fn from(value: Vote) -> Self {
            Self {
                r#type: value.ty.into(),
                height: value.height.inner(),
                round: value.round.inner(),
                block_id: Some(value.block_id.into()),
                timestamp: Some(value.timestamp.into()),
                validator_address: value.validator_address.into(),
                validator_index: value.validator_index,
                signature: value.signature.into(),
                extension: value.extension.unwrap_or_default().into(),
                extension_signature: value.extension_signature.unwrap_or_default().into(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid type")]
        Type(#[from] UnknownEnumVariant<i32>),
        #[error("invalid height")]
        Height(#[from] BoundedIntError<i64>),
        #[error("invalid round")]
        Round(#[from] BoundedIntError<i32>),
        #[error("invalid block id")]
        BlockId(#[from] block_id::proto::Error),
        #[error("invalid timestamp")]
        Timestamp(#[from] TryFromTimestampError),
        #[error("invalid validator address")]
        ValidatorAddress(#[from] FixedBytesError),
    }

    impl TryFrom<protos::cometbft::types::v1::Vote> for Vote {
        type Error = Error;

        fn try_from(value: protos::cometbft::types::v1::Vote) -> Result<Self, Self::Error> {
            Ok(Self {
                ty: value.r#type.try_into()?,
                height: value.height.try_into()?,
                round: value.round.try_into()?,
                block_id: required!(value.block_id)?.try_into()?,
                timestamp: required!(value.timestamp)?.try_into()?,
                validator_address: value.validator_address.try_into()?,
                validator_index: value.validator_index,
                signature: value.signature.into(),
                extension: if value.extension.is_empty() {
                    None
                } else {
                    Some(value.extension.into())
                },
                extension_signature: if value.extension_signature.is_empty() {
                    None
                } else {
                    Some(value.extension_signature.into())
                },
            })
        }
    }
}
