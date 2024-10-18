use macros::model;

use crate::cosmos::ics23::existence_proof::ExistenceProof;

#[model(proto(raw(protos::cosmos::ics23::v1::NonExistenceProof), into, from))]
pub struct NonExistenceProof {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub key: Vec<u8>,
    pub left: Option<ExistenceProof>,
    pub right: Option<ExistenceProof>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        cosmos::ics23::{
            existence_proof::proto::TryFromExistenceProofError,
            non_existence_proof::NonExistenceProof,
        },
        errors::MissingField,
    };

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum TryFromNonExistenceProofError {
        #[error(transparent)]
        MissingField(MissingField),
        #[error("left proof invalid")]
        Left(TryFromExistenceProofError),
        #[error("right proof invalid")]
        Right(TryFromExistenceProofError),
    }

    impl TryFrom<protos::cosmos::ics23::v1::NonExistenceProof> for NonExistenceProof {
        type Error = TryFromNonExistenceProofError;

        fn try_from(
            value: protos::cosmos::ics23::v1::NonExistenceProof,
        ) -> Result<Self, Self::Error> {
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
}
