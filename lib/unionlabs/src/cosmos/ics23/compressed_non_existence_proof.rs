use serde::{Deserialize, Serialize};

use crate::{
    cosmos::ics23::compressed_existence_proof::CompressedExistenceProof,
    errors::{required, MissingField},
    TryFromProtoErrorOf,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompressedNonExistenceProof {
    #[serde(with = "::serde_utils::hex_string")]
    pub key: Vec<u8>,
    pub left: CompressedExistenceProof,
    pub right: CompressedExistenceProof,
}

impl crate::Proto for CompressedNonExistenceProof {
    type Proto = protos::cosmos::ics23::v1::CompressedNonExistenceProof;
}

#[derive(Debug)]
pub enum TryFromCompressedNonExistenceProofError {
    MissingField(MissingField),
    Left(TryFromProtoErrorOf<CompressedExistenceProof>),
    Right(TryFromProtoErrorOf<CompressedExistenceProof>),
}

impl TryFrom<protos::cosmos::ics23::v1::CompressedNonExistenceProof>
    for CompressedNonExistenceProof
{
    type Error = TryFromCompressedNonExistenceProofError;

    fn try_from(
        value: protos::cosmos::ics23::v1::CompressedNonExistenceProof,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            key: value.key,
            left: required!(value.left)?
                .try_into()
                .map_err(TryFromCompressedNonExistenceProofError::Left)?,
            right: required!(value.right)?
                .try_into()
                .map_err(TryFromCompressedNonExistenceProofError::Right)?,
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<CompressedNonExistenceProof>
    for contracts::glue::CosmosIcs23V1CompressedNonExistenceProofData
{
    fn from(value: CompressedNonExistenceProof) -> Self {
        Self {
            key: value.key.into(),
            left: value.left.into(),
            right: value.right.into(),
        }
    }
}

impl From<CompressedNonExistenceProof> for protos::cosmos::ics23::v1::CompressedNonExistenceProof {
    fn from(value: CompressedNonExistenceProof) -> Self {
        Self {
            key: value.key,
            left: Some(value.left.into()),
            right: Some(value.right.into()),
        }
    }
}
