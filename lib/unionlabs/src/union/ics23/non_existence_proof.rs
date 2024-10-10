use macros::model;

use crate::{
    errors::MissingField,
    union::ics23::existence_proof::{ExistenceProof, TryFromExistenceProofError},
};

#[model(
    proto(raw(protos::cosmos::ics23::v1::NonExistenceProof), into, from),
    ethabi(raw(ibc_solidity::ics23::NonExistenceProof), into, from)
)]
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

#[cfg(feature = "ethabi")]
impl From<NonExistenceProof> for ibc_solidity::ics23::NonExistenceProof {
    fn from(value: NonExistenceProof) -> Self {
        let exist_default = || ExistenceProof {
            key: vec![],
            value: vec![],
            leaf_prefix: vec![],
            path: vec![],
        };

        ibc_solidity::ics23::NonExistenceProof {
            key: value.key.into(),
            left: value.left.unwrap_or_else(exist_default).into(),
            right: value.right.unwrap_or_else(exist_default).into(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<ibc_solidity::ics23::NonExistenceProof> for NonExistenceProof {
    fn from(value: ibc_solidity::ics23::NonExistenceProof) -> Self {
        let is_default = |e: &ibc_solidity::ics23::ExistenceProof| {
            e.key.is_empty() && e.value.is_empty() && e.leafPrefix.is_empty() && e.path.is_empty()
        };

        NonExistenceProof {
            key: value.key.to_vec(),
            left: (is_default(&value.left)).then_some(value.left.into()),
            right: (is_default(&value.right)).then_some(value.right.into()),
        }
    }
}

#[cfg(feature = "proto")]
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

#[cfg(feature = "proto")]
impl From<NonExistenceProof> for protos::cosmos::ics23::v1::NonExistenceProof {
    fn from(value: NonExistenceProof) -> Self {
        Self {
            key: value.key,
            left: value.left.map(Into::into),
            right: value.right.map(Into::into),
        }
    }
}
