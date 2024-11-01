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
#[doc(hidden)]
#[derive(Debug, ::ethers::contract::EthAbiType, ::ethers::contract::EthAbiCodec)]
pub struct NonExistenceProofEthAbi {
    pub key: ethers::types::Bytes,
    pub left: crate::union::ics23::existence_proof::ExistenceProofEthAbi,
    pub right: crate::union::ics23::existence_proof::ExistenceProofEthAbi,
}

#[cfg(feature = "ethabi")]
impl From<NonExistenceProof> for NonExistenceProofEthAbi {
    fn from(value: NonExistenceProof) -> Self {
        let exist_default = || ExistenceProof {
            key: vec![].into(),
            value: vec![].into(),
            leaf_prefix: vec![].into(),
            path: vec![],
        };

        NonExistenceProofEthAbi {
            key: value.key.into(),
            left: value.left.unwrap_or_else(exist_default).into(),
            right: value.right.unwrap_or_else(exist_default).into(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<NonExistenceProofEthAbi> for NonExistenceProof {
    fn from(value: NonExistenceProofEthAbi) -> Self {
        let exist_default = super::existence_proof::ExistenceProofEthAbi {
            key: vec![].into(),
            value: vec![].into(),
            leaf_prefix: vec![].into(),
            path: vec![],
        };

        NonExistenceProof {
            key: value.key.to_vec(),
            left: (value.left != exist_default).then_some(value.left.into()),
            right: (value.right != exist_default).then_some(value.right.into()),
        }
    }
}

#[cfg(feature = "ethabi")]
impl crate::encoding::Encode<crate::encoding::EthAbi> for NonExistenceProof {
    fn encode(self) -> Vec<u8> {
        ethers::abi::AbiEncode::encode(NonExistenceProofEthAbi::from(self))
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

impl From<NonExistenceProof> for protos::cosmos::ics23::v1::NonExistenceProof {
    fn from(value: NonExistenceProof) -> Self {
        Self {
            key: value.key,
            left: value.left.map(Into::into),
            right: value.right.map(Into::into),
        }
    }
}
