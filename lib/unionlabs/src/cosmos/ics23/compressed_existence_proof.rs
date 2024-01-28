use serde::{Deserialize, Serialize};

use crate::{
    bounded::{BoundedI32, BoundedIntError},
    cosmos::ics23::leaf_op::LeafOp,
    errors::{required, MissingField},
    TryFromProtoErrorOf,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct CompressedExistenceProof {
    #[serde(with = "::serde_utils::hex_string")]
    pub key: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    pub value: Vec<u8>,
    pub leaf: LeafOp,
    // these are indexes into an array, hence non-negative
    pub path: Vec<BoundedI32<0, { i32::MAX }>>,
}

impl crate::Proto for CompressedExistenceProof {
    type Proto = protos::cosmos::ics23::v1::CompressedExistenceProof;
}

#[derive(Debug)]
pub enum TryFromCompressedExistenceProofError {
    MissingField(MissingField),
    Leaf(TryFromProtoErrorOf<LeafOp>),
    Path(BoundedIntError<i32>),
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

#[cfg(feature = "ethabi")]
impl From<CompressedExistenceProof> for contracts::glue::CosmosIcs23V1CompressedExistenceProofData {
    fn from(value: CompressedExistenceProof) -> Self {
        Self {
            key: value.key.into(),
            value: value.value.into(),
            leaf: value.leaf.into(),
            path: value.path.into_iter().map(Into::into).collect(),
        }
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
