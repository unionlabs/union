use serde::{Deserialize, Serialize};

use crate::{
    cosmos::ics23::{inner_op::InnerOp, leaf_op::LeafOp},
    errors::{required, MissingField},
    TryFromProtoErrorOf,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExistenceProof {
    #[serde(with = "::serde_utils::hex_string")]
    pub key: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    pub value: Vec<u8>,
    pub leaf: LeafOp,
    pub path: Vec<InnerOp>,
}

impl crate::Proto for ExistenceProof {
    type Proto = protos::cosmos::ics23::v1::ExistenceProof;
}

#[derive(Debug)]
pub enum TryFromExistenceProofError {
    MissingField(MissingField),
    Leaf(TryFromProtoErrorOf<LeafOp>),
    Path(TryFromProtoErrorOf<InnerOp>),
}

impl TryFrom<protos::cosmos::ics23::v1::ExistenceProof> for ExistenceProof {
    type Error = TryFromExistenceProofError;

    fn try_from(value: protos::cosmos::ics23::v1::ExistenceProof) -> Result<Self, Self::Error> {
        Ok(Self {
            key: value.key,
            value: value.value,
            leaf: required!(value.leaf)?
                .try_into()
                .map_err(TryFromExistenceProofError::Leaf)?,
            path: value
                .path
                .into_iter()
                .map(TryFrom::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map_err(TryFromExistenceProofError::Path)?,
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<ExistenceProof> for contracts::glue::CosmosIcs23V1ExistenceProofData {
    fn from(value: ExistenceProof) -> Self {
        Self {
            key: value.key.into(),
            value: value.value.into(),
            leaf: value.leaf.into(),
            path: value.path.into_iter().map(Into::into).collect::<Vec<_>>(),
        }
    }
}

impl From<ExistenceProof> for protos::cosmos::ics23::v1::ExistenceProof {
    fn from(value: ExistenceProof) -> Self {
        Self {
            key: value.key,
            value: value.value,
            leaf: Some(value.leaf.into()),
            path: value.path.into_iter().map(Into::into).collect::<Vec<_>>(),
        }
    }
}
