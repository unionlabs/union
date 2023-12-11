use serde::{Deserialize, Serialize};

use crate::{
    cosmos::ics23::existence_proof::ExistenceProof,
    errors::{required, MissingField},
    TryFromProtoErrorOf,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NonExistenceProof {
    #[serde(with = "::serde_utils::hex_string")]
    pub key: Vec<u8>,
    pub left: ExistenceProof,
    pub right: ExistenceProof,
}

impl crate::Proto for NonExistenceProof {
    type Proto = protos::cosmos::ics23::v1::NonExistenceProof;
}

#[derive(Debug)]
pub enum TryFromNonExistenceProofError {
    MissingField(MissingField),
    Left(TryFromProtoErrorOf<ExistenceProof>),
    Right(TryFromProtoErrorOf<ExistenceProof>),
}

impl TryFrom<protos::cosmos::ics23::v1::NonExistenceProof> for NonExistenceProof {
    type Error = TryFromNonExistenceProofError;

    fn try_from(value: protos::cosmos::ics23::v1::NonExistenceProof) -> Result<Self, Self::Error> {
        Ok(Self {
            key: value.key,
            left: required!(value.left)?
                .try_into()
                .map_err(TryFromNonExistenceProofError::Left)?,
            right: required!(value.right)?
                .try_into()
                .map_err(TryFromNonExistenceProofError::Right)?,
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<NonExistenceProof> for contracts::glue::CosmosIcs23V1NonExistenceProofData {
    fn from(value: NonExistenceProof) -> Self {
        Self {
            key: value.key.into(),
            left: value.left.into(),
            right: value.right.into(),
        }
    }
}

impl From<NonExistenceProof> for protos::cosmos::ics23::v1::NonExistenceProof {
    fn from(value: NonExistenceProof) -> Self {
        Self {
            key: value.key,
            left: Some(value.left.into()),
            right: Some(value.right.into()),
        }
    }
}
