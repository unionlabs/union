use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use super::{hash_op::HashOp, length_op::LengthOp};
use crate::{errors::UnknownEnumVariant, Proto, TypeUrl};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct LeafOp {
    pub hash: HashOp,
    pub prehash_key: HashOp,
    pub prehash_value: HashOp,
    pub length: LengthOp,
    #[serde(with = "::serde_utils::hex_string")]
    #[cfg_attr(feature = "arbitrary", arbitrary(with = crate::arbitrary_cow_static))]
    pub prefix: Cow<'static, [u8]>,
}

impl TypeUrl for protos::cosmos::ics23::v1::LeafOp {
    const TYPE_URL: &'static str = "/cosmos.ics23.v1.LeafOp";
}

impl Proto for LeafOp {
    type Proto = protos::cosmos::ics23::v1::LeafOp;
}

impl From<LeafOp> for protos::cosmos::ics23::v1::LeafOp {
    fn from(value: LeafOp) -> Self {
        Self {
            hash: value.hash.into(),
            prehash_key: value.prehash_key.into(),
            prehash_value: value.prehash_value.into(),
            length: value.length.into(),
            prefix: value.prefix.into(),
        }
    }
}

#[derive(Debug)]
pub enum TryFromLeafOpError {
    Hash(UnknownEnumVariant<i32>),
    PrehashKey(UnknownEnumVariant<i32>),
    PrehashValue(UnknownEnumVariant<i32>),
    Length(UnknownEnumVariant<i32>),
}

impl TryFrom<protos::cosmos::ics23::v1::LeafOp> for LeafOp {
    type Error = TryFromLeafOpError;

    fn try_from(value: protos::cosmos::ics23::v1::LeafOp) -> Result<Self, Self::Error> {
        Ok(Self {
            hash: value.hash.try_into().map_err(TryFromLeafOpError::Hash)?,
            prehash_key: value
                .prehash_key
                .try_into()
                .map_err(TryFromLeafOpError::PrehashKey)?,
            prehash_value: value
                .prehash_value
                .try_into()
                .map_err(TryFromLeafOpError::PrehashValue)?,
            length: value
                .length
                .try_into()
                .map_err(TryFromLeafOpError::Length)?,
            prefix: value.prefix.into(),
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<LeafOp> for contracts::glue::CosmosIcs23V1LeafOpData {
    fn from(value: LeafOp) -> Self {
        Self {
            hash: value.hash.into(),
            prehash_key: value.prehash_key.into(),
            prehash_value: value.prehash_value.into(),
            length: value.length.into(),
            prefix: value.prefix.into_owned().into(),
        }
    }
}
