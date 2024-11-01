use macros::model;

use crate::{
    bytes::Bytes,
    cosmos::ics23::{hash_op::HashOp, length_op::LengthOp},
    ensure,
    union::ics23::inner_op::InnerOp,
};

#[model(
    proto(raw(protos::cosmos::ics23::v1::ExistenceProof), into, from),
    ethabi(raw(ExistenceProofEthAbi), into, from)
)]
pub struct ExistenceProof {
    pub key: Bytes,
    pub value: Bytes,
    pub leaf_prefix: Bytes,
    pub path: Vec<InnerOp>,
}

#[cfg(feature = "ethabi")]
#[doc(hidden)]
impl From<ExistenceProof> for ExistenceProofEthAbi {
    fn from(value: ExistenceProof) -> Self {
        ExistenceProofEthAbi {
            key: value.key.into_vec().into(),
            value: value.value.into_vec().into(),
            leaf_prefix: value.leaf_prefix.into_vec().into(),
            path: value.path.into_iter().map(Into::into).collect(),
        }
    }
}

#[cfg(feature = "ethabi")]
#[doc(hidden)]
impl From<ExistenceProofEthAbi> for ExistenceProof {
    fn from(value: ExistenceProofEthAbi) -> Self {
        ExistenceProof {
            key: value.key.to_vec().into(),
            value: value.value.to_vec().into(),
            leaf_prefix: value.leaf_prefix.to_vec().into(),
            path: value.path.into_iter().map(Into::into).collect(),
        }
    }
}

#[cfg(feature = "ethabi")]
#[doc(hidden)]
#[derive(Debug, PartialEq, ::ethers::contract::EthAbiType, ::ethers::contract::EthAbiCodec)]
pub struct ExistenceProofEthAbi {
    pub key: ethers::types::Bytes,
    pub value: ethers::types::Bytes,
    pub leaf_prefix: ethers::types::Bytes,
    pub path: Vec<crate::union::ics23::inner_op::InnerOpEthAbi>,
}

const EXPECTED_PREHASH_KEY: HashOp = HashOp::NoHash;
const EXPECTED_PREHASH_VALUE: HashOp = HashOp::Sha256;
const EXPECTED_HASH: HashOp = HashOp::Sha256;
const EXPECTED_LENGTH: LengthOp = LengthOp::VarProto;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromExistenceProofError {
    #[error("unable to decode cosmos::ics23::ExistenceProof")]
    Cosmos(#[from] crate::cosmos::ics23::existence_proof::TryFromExistenceProofError),
    #[error(
        "invalid leaf.prehash_key, expected {} but found {0}",
        EXPECTED_PREHASH_KEY
    )]
    InvalidPrehashKey(HashOp),
    #[error(
        "invalid leaf.prehash_value, expected {} but found {0}",
        EXPECTED_PREHASH_VALUE
    )]
    InvalidPrehashValue(HashOp),
    #[error("invalid leaf.hash, expected {} but found {0}", EXPECTED_HASH)]
    InvalidHash(HashOp),
    #[error("invalid leaf.length, expected {} but found {0}", EXPECTED_LENGTH)]
    InvalidLength(LengthOp),
}

impl TryFrom<protos::cosmos::ics23::v1::ExistenceProof> for ExistenceProof {
    type Error = TryFromExistenceProofError;

    fn try_from(value: protos::cosmos::ics23::v1::ExistenceProof) -> Result<Self, Self::Error> {
        let value = crate::cosmos::ics23::existence_proof::ExistenceProof::try_from(value)?;

        ensure(
            value.leaf.prehash_key == EXPECTED_PREHASH_KEY,
            TryFromExistenceProofError::InvalidPrehashKey(value.leaf.prehash_key),
        )?;

        ensure(
            value.leaf.prehash_value == EXPECTED_PREHASH_VALUE,
            TryFromExistenceProofError::InvalidPrehashValue(value.leaf.prehash_value),
        )?;

        ensure(
            value.leaf.hash == EXPECTED_HASH,
            TryFromExistenceProofError::InvalidHash(value.leaf.hash),
        )?;

        ensure(
            value.leaf.length == EXPECTED_LENGTH,
            TryFromExistenceProofError::InvalidLength(value.leaf.length),
        )?;

        Ok(Self {
            key: value.key.to_vec().into(),
            value: value.value.to_vec().into(),
            leaf_prefix: value.leaf.prefix.to_vec().into(),
            path: value
                .path
                .into_iter()
                .map(|io| InnerOp {
                    prefix: io.prefix.into(),
                    suffix: io.suffix.into(),
                })
                .collect(),
        })
    }
}

impl From<ExistenceProof> for protos::cosmos::ics23::v1::ExistenceProof {
    fn from(value: ExistenceProof) -> Self {
        Self {
            key: value.key.clone().into(),
            value: value.value.clone().into(),
            leaf: Some(
                crate::cosmos::ics23::leaf_op::LeafOp {
                    hash: EXPECTED_HASH,
                    prehash_key: EXPECTED_PREHASH_KEY,
                    prehash_value: EXPECTED_PREHASH_VALUE,
                    length: EXPECTED_LENGTH,
                    prefix: value.leaf_prefix.into(),
                }
                .into(),
            ),
            path: value.path.into_iter().map(Into::into).collect::<Vec<_>>(),
        }
    }
}
