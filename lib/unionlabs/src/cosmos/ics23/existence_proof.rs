use macros::model;
use unionlabs_primitives::Bytes;

use crate::{
    cosmos::ics23::{
        inner_op::{InnerOp, TryFromInnerOpError},
        leaf_op::{LeafOp, TryFromLeafOpError},
    },
    errors::{required, MissingField},
};

#[model(proto(raw(protos::cosmos::ics23::v1::ExistenceProof), into, from))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ExistenceProof {
    pub key: Bytes,
    pub value: Bytes,
    pub leaf: LeafOp,
    pub path: Vec<InnerOp>,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromExistenceProofError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("error decoding leaf")]
    Leaf(#[from] TryFromLeafOpError),
    #[error("error decoding path")]
    Path(#[from] TryFromInnerOpError),
}

impl TryFrom<protos::cosmos::ics23::v1::ExistenceProof> for ExistenceProof {
    type Error = TryFromExistenceProofError;

    fn try_from(value: protos::cosmos::ics23::v1::ExistenceProof) -> Result<Self, Self::Error> {
        Ok(Self {
            key: value.key.into(),
            value: value.value.into(),
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

impl From<ExistenceProof> for protos::cosmos::ics23::v1::ExistenceProof {
    fn from(value: ExistenceProof) -> Self {
        Self {
            key: value.key.into(),
            value: value.value.into(),
            leaf: Some(value.leaf.into()),
            path: value.path.into_iter().map(Into::into).collect::<Vec<_>>(),
        }
    }
}
