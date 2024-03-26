use ethers::abi::Tokenize;
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

#[cfg(feature = "ethabi")]
impl crate::encoding::Encode<crate::encoding::EthAbi> for NonExistenceProof {
    fn encode(self) -> Vec<u8> {
        use ethers::abi::ethabi::{self, Token};

        let empty_existence_proof_tokens =
            crate::union::ics23::existence_proof::ExistenceProofEthAbi {
                key: vec![].into(),
                value: vec![].into(),
                leaf_prefix: vec![].into(),
                path: vec![],
            }
            .into_tokens();

        let ethabi =
            |e: ExistenceProof| crate::union::ics23::existence_proof::ExistenceProofEthAbi {
                key: e.key.into(),
                value: e.value.into(),
                leaf_prefix: e.leaf_prefix.into(),
                path: e
                    .path
                    .into_iter()
                    .map(|io| crate::union::ics23::inner_op::InnerOpEthAbi {
                        prefix: io.prefix.into(),
                        suffix: io.suffix.into(),
                    })
                    .collect(),
            };

        let left = self.left.map_or_else(
            || empty_existence_proof_tokens.clone(),
            |e| ethabi(e).into_tokens(),
        );
        let right = self
            .right
            .map_or(empty_existence_proof_tokens, |e| ethabi(e).into_tokens());

        ethabi::encode(&[
            Token::Bytes(self.key),
            Token::Tuple(left),
            Token::Tuple(right),
        ])
    }
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

// #[cfg(feature = "ethabi")]
// impl From<NonExistenceProof> for contracts::glue::CosmosIcs23V1NonExistenceProofData {
//     fn from(value: NonExistenceProof) -> Self {
//         Self {
//             key: value.key.into(),
//             left: value.left.map(Into::into).unwrap_or_default(),
//             right: value.right.map(Into::into).unwrap_or_default(),
//         }
//     }
// }

impl From<NonExistenceProof> for protos::cosmos::ics23::v1::NonExistenceProof {
    fn from(value: NonExistenceProof) -> Self {
        Self {
            key: value.key,
            left: value.left.map(Into::into),
            right: value.right.map(Into::into),
        }
    }
}
