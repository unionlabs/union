use macros::model;

use crate::{
    bounded::{BoundedI32, BoundedIntError},
    cosmos::ics23::leaf_op::{LeafOp, TryFromLeafOpError},
    errors::{MissingField, required},
};

#[model(proto(raw(protos::cosmos::ics23::v1::CompressedExistenceProof), into, from))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct CompressedExistenceProof {
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub key: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub value: Vec<u8>,
    pub leaf: LeafOp,
    // these are indexes into an array, hence non-negative
    pub path: Vec<BoundedI32<0, { i32::MAX }>>,
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromCompressedExistenceProofError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid leaf")]
    Leaf(#[from] TryFromLeafOpError),
    #[error("invalid path")]
    Path(#[source] BoundedIntError<i32>),
}

impl TryFrom<protos::cosmos::ics23::v1::CompressedExistenceProof> for CompressedExistenceProof {
    type Error = TryFromCompressedExistenceProofError;

    fn try_from(
        value: protos::cosmos::ics23::v1::CompressedExistenceProof,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            key: value.key,
            value: value.value,
            leaf: required!(value.leaf)?
                .try_into()
                .map_err(TryFromCompressedExistenceProofError::Leaf)?,
            path: value
                .path
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()
                .map_err(TryFromCompressedExistenceProofError::Path)?,
        })
    }
}

impl From<CompressedExistenceProof> for protos::cosmos::ics23::v1::CompressedExistenceProof {
    fn from(value: CompressedExistenceProof) -> Self {
        Self {
            key: value.key,
            value: value.value,
            leaf: Some(value.leaf.into()),
            path: value.path.into_iter().map(Into::into).collect(),
        }
    }
}
