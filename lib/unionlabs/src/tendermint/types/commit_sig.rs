use serde::{Deserialize, Serialize};

use crate::{
    errors::{required, InvalidLength, MissingField},
    ethereum::{Address, H512},
    google::protobuf::timestamp::Timestamp,
    tendermint::types::block_id_flag::BlockIdFlag,
    Proto, TryFromProtoErrorOf, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommitSig {
    pub block_id_flag: BlockIdFlag,
    pub validator_address: Address,
    pub timestamp: Timestamp,
    // REVIEW: Is this a fixed hash? Testing concludes that it's a 64-byte hash (for cometbls at least).
    pub signature: H512,
}

impl From<CommitSig> for protos::tendermint::types::CommitSig {
    fn from(value: CommitSig) -> Self {
        Self {
            block_id_flag: value.block_id_flag.into(),
            validator_address: value.validator_address.into(),
            timestamp: Some(value.timestamp.into()),
            signature: value.signature.into(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl crate::EthAbi for CommitSig {
    type EthAbi = contracts::glue::TendermintTypesCommitSigData;
}

#[cfg(feature = "ethabi")]
#[derive(Debug)]
pub enum TryFromEthAbiCommitSigError {
    BlockIdFlag(crate::errors::UnknownEnumVariant<u8>),
    ValidatorAddress(crate::errors::InvalidLength),
    Timestamp(crate::TryFromEthAbiErrorOf<Timestamp>),
    Signature(InvalidLength),
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::glue::TendermintTypesCommitSigData> for CommitSig {
    type Error = TryFromEthAbiCommitSigError;

    fn try_from(value: contracts::glue::TendermintTypesCommitSigData) -> Result<Self, Self::Error> {
        Ok(Self {
            block_id_flag: value
                .block_id_flag
                .try_into()
                .map_err(TryFromEthAbiCommitSigError::BlockIdFlag)?,
            validator_address: value
                .validator_address
                .try_into()
                .map_err(TryFromEthAbiCommitSigError::ValidatorAddress)?,
            timestamp: value
                .timestamp
                .try_into()
                .map_err(TryFromEthAbiCommitSigError::Timestamp)?,
            signature: value
                .signature
                .to_vec()
                .try_into()
                .map_err(TryFromEthAbiCommitSigError::Signature)?,
        })
    }
}

#[derive(Debug)]
pub enum TryFromCommitSigError {
    MissingField(MissingField),
    ValidatorAddress(InvalidLength),
    BlockIdFlag(crate::errors::UnknownEnumVariant<i32>),
    Timestamp(TryFromProtoErrorOf<Timestamp>),
    Signature(InvalidLength),
}

impl TryFrom<protos::tendermint::types::CommitSig> for CommitSig {
    type Error = TryFromCommitSigError;

    fn try_from(value: protos::tendermint::types::CommitSig) -> Result<Self, Self::Error> {
        Ok(Self {
            block_id_flag: value
                .block_id_flag
                .try_into()
                .map_err(TryFromCommitSigError::BlockIdFlag)?,
            validator_address: value
                .validator_address
                .try_into()
                .map_err(TryFromCommitSigError::ValidatorAddress)?,
            timestamp: required!(value.timestamp)?
                .try_into()
                .map_err(TryFromCommitSigError::Timestamp)?,
            signature: value
                .signature
                .try_into()
                .map_err(TryFromCommitSigError::Signature)?,
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<CommitSig> for contracts::glue::TendermintTypesCommitSigData {
    fn from(value: CommitSig) -> Self {
        Self {
            block_id_flag: value.block_id_flag.into(),
            validator_address: value.validator_address.into(),
            timestamp: value.timestamp.into(),
            signature: value.signature.into(),
        }
    }
}

impl TypeUrl for protos::tendermint::types::CommitSig {
    const TYPE_URL: &'static str = "/tendermint.types.CommitSig";
}

impl Proto for CommitSig {
    type Proto = protos::tendermint::types::CommitSig;
}
