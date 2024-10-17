use macros::model;

use crate::{
    bounded::{BoundedI64, BoundedIntError},
    errors::InvalidLength,
    hash::H256,
};

#[model(proto(raw(protos::cometbft::crypto::v1::Proof), into, from))]
pub struct Proof {
    pub total: BoundedI64<0, { i64::MAX }>,
    pub index: BoundedI64<0, { i64::MAX }>,
    pub leaf_hash: H256,
    #[serde(with = "::serde_utils::hex_string_list")]
    #[debug(wrap = ::serde_utils::fmt::DebugListAsHex)]
    pub aunts: Vec<Vec<u8>>,
}

impl From<Proof> for protos::cometbft::crypto::v1::Proof {
    fn from(value: Proof) -> Self {
        Self {
            total: value.total.into(),
            index: value.index.into(),
            leaf_hash: value.leaf_hash.into(),
            aunts: value.aunts,
        }
    }
}

impl TryFrom<protos::cometbft::crypto::v1::Proof> for Proof {
    type Error = TryFromProofError;

    fn try_from(value: protos::cometbft::crypto::v1::Proof) -> Result<Self, Self::Error> {
        Ok(Self {
            total: value.total.try_into().map_err(TryFromProofError::Total)?,
            index: value.index.try_into().map_err(TryFromProofError::Index)?,
            leaf_hash: value.leaf_hash.try_into()?,
            aunts: value.aunts,
        })
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromProofError {
    #[error("invalid total")]
    Total(#[source] BoundedIntError<i64>),
    #[error("invalid index")]
    Index(#[source] BoundedIntError<i64>),
    #[error("invalid leaf hash")]
    LeafHash(#[from] InvalidLength),
}
