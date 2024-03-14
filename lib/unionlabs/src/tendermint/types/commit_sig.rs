use macros::model;

#[cfg(feature = "ethabi")]
use crate::google::protobuf::timestamp::TryFromEthAbiTimestampError;
use crate::{
    errors::{required, InvalidLength, MissingField, UnknownEnumVariant},
    google::protobuf::timestamp::{Timestamp, TryFromTimestampError},
    hash::{H160, H512},
    tendermint::types::block_id_flag::BlockIdFlag,
};

#[model(proto(raw(protos::tendermint::types::CommitSig), into, from))]
pub enum CommitSig {
    Absent,
    Commit {
        validator_address: H160,
        timestamp: Timestamp,
        signature: H512,
    },
    Nil {
        validator_address: H160,
        timestamp: Timestamp,
        signature: H512,
    },
}

impl From<CommitSig> for protos::tendermint::types::CommitSig {
    fn from(value: CommitSig) -> Self {
        match value {
            CommitSig::Absent => Self {
                block_id_flag: BlockIdFlag::Absent.into(),
                validator_address: vec![],
                timestamp: None,
                signature: vec![],
            },
            CommitSig::Commit {
                validator_address,
                timestamp,
                signature,
            } => Self {
                block_id_flag: BlockIdFlag::Commit.into(),
                validator_address: validator_address.into(),
                timestamp: Some(timestamp.into()),
                signature: signature.into(),
            },
            CommitSig::Nil {
                validator_address,
                timestamp,
                signature,
            } => Self {
                block_id_flag: BlockIdFlag::Nil.into(),
                validator_address: validator_address.into(),
                timestamp: Some(timestamp.into()),
                signature: signature.into(),
            },
        }
    }
}

#[cfg(feature = "ethabi")]
#[derive(Debug, Clone, PartialEq)]
pub enum TryFromEthAbiCommitSigError {
    BlockIdFlag(UnknownEnumVariant<u8>),
    ValidatorAddress(crate::errors::InvalidLength),
    Timestamp(TryFromEthAbiTimestampError),
    Signature(InvalidLength),
    UnknownBlockIdFlag,
    AbsentWithValidatorAddress,
    AbsentWithTimestamp,
    AbsentWithSignature,
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::glue::TendermintTypesCommitSigData> for CommitSig {
    type Error = TryFromEthAbiCommitSigError;

    fn try_from(value: contracts::glue::TendermintTypesCommitSigData) -> Result<Self, Self::Error> {
        let block_id_flag = BlockIdFlag::try_from(value.block_id_flag)
            .map_err(TryFromEthAbiCommitSigError::BlockIdFlag)?;

        match block_id_flag {
            BlockIdFlag::Unknown => Err(TryFromEthAbiCommitSigError::UnknownBlockIdFlag),
            BlockIdFlag::Absent => {
                if !value.validator_address.is_empty() {
                    Err(TryFromEthAbiCommitSigError::AbsentWithValidatorAddress)
                } else if value.timestamp != contracts::glue::GoogleProtobufTimestampData::default()
                {
                    Err(TryFromEthAbiCommitSigError::AbsentWithTimestamp)
                } else if !value.signature.is_empty() {
                    Err(TryFromEthAbiCommitSigError::AbsentWithSignature)
                } else {
                    Ok(Self::Absent)
                }
            }
            BlockIdFlag::Commit => Ok(Self::Commit {
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
            }),
            BlockIdFlag::Nil => Ok(Self::Nil {
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
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TryFromCommitSigError {
    MissingField(MissingField),
    ValidatorAddress(InvalidLength),
    BlockIdFlag(UnknownEnumVariant<i32>),
    Timestamp(TryFromTimestampError),
    Signature(InvalidLength),
    UnknownBlockIdFlag,
    AbsentWithValidatorAddress,
    AbsentWithTimestamp,
    AbsentWithSignature,
}

impl TryFrom<protos::tendermint::types::CommitSig> for CommitSig {
    type Error = TryFromCommitSigError;

    fn try_from(value: protos::tendermint::types::CommitSig) -> Result<Self, Self::Error> {
        let block_id_flag = BlockIdFlag::try_from(value.block_id_flag)
            .map_err(TryFromCommitSigError::BlockIdFlag)?;

        match block_id_flag {
            BlockIdFlag::Unknown => Err(TryFromCommitSigError::UnknownBlockIdFlag),
            BlockIdFlag::Absent => {
                if !value.validator_address.is_empty() {
                    Err(TryFromCommitSigError::AbsentWithValidatorAddress)
                } else if value.timestamp.is_some() {
                    Err(TryFromCommitSigError::AbsentWithTimestamp)
                } else if !value.signature.is_empty() {
                    Err(TryFromCommitSigError::AbsentWithSignature)
                } else {
                    Ok(Self::Absent)
                }
            }
            BlockIdFlag::Commit => Ok(Self::Commit {
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
            }),
            BlockIdFlag::Nil => Ok(Self::Nil {
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
            }),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<CommitSig> for contracts::glue::TendermintTypesCommitSigData {
    fn from(value: CommitSig) -> Self {
        match value {
            CommitSig::Absent => Self {
                block_id_flag: BlockIdFlag::Absent.into(),
                validator_address: vec![].into(),
                timestamp: contracts::glue::GoogleProtobufTimestampData::default(),
                signature: vec![].into(),
            },
            CommitSig::Commit {
                validator_address,
                timestamp,
                signature,
            } => Self {
                block_id_flag: BlockIdFlag::Commit.into(),
                validator_address: validator_address.into(),
                timestamp: timestamp.into(),
                signature: signature.into(),
            },
            CommitSig::Nil {
                validator_address,
                timestamp,
                signature,
            } => Self {
                block_id_flag: BlockIdFlag::Nil.into(),
                validator_address: validator_address.into(),
                timestamp: timestamp.into(),
                signature: signature.into(),
            },
        }
    }
}
