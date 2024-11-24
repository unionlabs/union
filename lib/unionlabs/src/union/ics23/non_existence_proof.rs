use macros::model;

use crate::{
    errors::MissingField,
    union::ics23::existence_proof::{ExistenceProof, TryFromExistenceProofError},
};

#[model(proto(raw(protos::cosmos::ics23::v1::NonExistenceProof), into, from))]
pub struct NonExistenceProof {
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    // TODO: Remove this, as it appears to be unused and the cosmos protos have a comment mentioning the same
    pub key: Vec<u8>,
    pub left: Option<ExistenceProof>,
    pub right: Option<ExistenceProof>,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromNonExistenceProofError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("unable to decode left existence proof")]
    Left(TryFromExistenceProofError),
    #[error("unable to decode right existence proof")]
    Right(TryFromExistenceProofError),
}

impl TryFrom<protos::cosmos::ics23::v1::NonExistenceProof> for NonExistenceProof {
    type Error = TryFromNonExistenceProofError;

    fn try_from(value: protos::cosmos::ics23::v1::NonExistenceProof) -> Result<Self, Self::Error> {
        Ok(Self {
            key: value.key,
            left: value
                .left
                .map(TryInto::try_into)
                .transpose()
                .map_err(TryFromNonExistenceProofError::Left)?,
            right: value
                .right
                .map(TryInto::try_into)
                .transpose()
                .map_err(TryFromNonExistenceProofError::Right)?,
        })
    }
}

impl From<NonExistenceProof> for protos::cosmos::ics23::v1::NonExistenceProof {
    fn from(value: NonExistenceProof) -> Self {
        Self {
            key: value.key,
            left: value.left.map(Into::into),
            right: value.right.map(Into::into),
        }
    }
}
