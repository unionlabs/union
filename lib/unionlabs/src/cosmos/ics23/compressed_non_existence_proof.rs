use macros::model;

use crate::{
    cosmos::ics23::compressed_existence_proof::{
        CompressedExistenceProof, TryFromCompressedExistenceProofError,
    },
    errors::MissingField,
};

#[model(proto(
    raw(protos::cosmos::ics23::v1::CompressedNonExistenceProof),
    into,
    from
))]
pub struct CompressedNonExistenceProof {
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub key: Vec<u8>,
    pub left: Option<CompressedExistenceProof>,
    pub right: Option<CompressedExistenceProof>,
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromCompressedNonExistenceProofError {
    #[error(transparent)]
    MissingField(MissingField),
    #[error("left proof invalid")]
    Left(#[source] TryFromCompressedExistenceProofError),
    #[error("right proof invalid")]
    Right(#[source] TryFromCompressedExistenceProofError),
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
            left: value
                .left
                .map(TryInto::try_into)
                .transpose()
                .map_err(TryFromCompressedNonExistenceProofError::Left)?,
            right: value
                .right
                .map(TryInto::try_into)
                .transpose()
                .map_err(TryFromCompressedNonExistenceProofError::Right)?,
        })
    }
}

impl From<CompressedNonExistenceProof> for protos::cosmos::ics23::v1::CompressedNonExistenceProof {
    fn from(value: CompressedNonExistenceProof) -> Self {
        Self {
            key: value.key,
            left: value.left.map(Into::into),
            right: value.right.map(Into::into),
        }
    }
}
