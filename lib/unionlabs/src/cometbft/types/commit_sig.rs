use macros::model;

#[cfg(feature = "ethabi")]
use crate::google::protobuf::timestamp::TryFromEthAbiTimestampError;
use crate::{
    cometbft::types::block_id_flag::BlockIdFlag,
    errors::{required, InvalidLength, MissingField, UnknownEnumVariant},
    google::protobuf::timestamp::{Timestamp, TryFromTimestampError},
    hash::H160,
};

#[model(proto(raw(protos::cometbft::types::v1::CommitSig), into, from))]
pub enum CommitSig {
    Absent,
    Commit {
        validator_address: H160,
        timestamp: Timestamp,
        #[serde(with = "::serde_utils::hex_string")]
        #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
        signature: Vec<u8>,
    },
    Nil {
        validator_address: H160,
        timestamp: Timestamp,
        #[serde(with = "::serde_utils::hex_string")]
        #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
        signature: Vec<u8>,
    },
}

impl From<CommitSig> for protos::cometbft::types::v1::CommitSig {
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
                signature,
            },
            CommitSig::Nil {
                validator_address,
                timestamp,
                signature,
            } => Self {
                block_id_flag: BlockIdFlag::Nil.into(),
                validator_address: validator_address.into(),
                timestamp: Some(timestamp.into()),
                signature,
            },
        }
    }
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
                signature,
            },
            CommitSig::Nil {
                validator_address,
                timestamp,
                signature,
            } => Self {
                block_id_flag: BlockIdFlag::Nil.into(),
                validator_address: validator_address.into(),
                timestamp: Some(timestamp.into()),
                signature,
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
                    .to_vec()
                    .try_into()
                    .map_err(TryFromEthAbiCommitSigError::ValidatorAddress)?,
                timestamp: value
                    .timestamp
                    .try_into()
                    .map_err(TryFromEthAbiCommitSigError::Timestamp)?,
                signature: value.signature.to_vec(),
            }),
            BlockIdFlag::Nil => Ok(Self::Nil {
                validator_address: value
                    .validator_address
                    .to_vec()
                    .try_into()
                    .map_err(TryFromEthAbiCommitSigError::ValidatorAddress)?,
                timestamp: value
                    .timestamp
                    .try_into()
                    .map_err(TryFromEthAbiCommitSigError::Timestamp)?,
                signature: value.signature.to_vec(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromCommitSigError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid validator address")]
    ValidatorAddress(InvalidLength),
    #[error("invalid block id flag")]
    BlockIdFlag(UnknownEnumVariant<i32>),
    #[error("invalid timestamp")]
    Timestamp(TryFromTimestampError),
    #[error("block id flag was `Unknown`")]
    UnknownBlockIdFlag,
    #[error("an absent commit sig had an address")]
    AbsentWithValidatorAddress,
    #[error("an absent commit sig had a timestamp")]
    AbsentWithTimestamp,
    #[error("an absent commit sig had a signature")]
    AbsentWithSignature,
}

impl TryFrom<protos::cometbft::types::v1::CommitSig> for CommitSig {
    type Error = TryFromCommitSigError;

    fn try_from(value: protos::cometbft::types::v1::CommitSig) -> Result<Self, Self::Error> {
        let block_id_flag = BlockIdFlag::try_from(value.block_id_flag)
            .map_err(TryFromCommitSigError::BlockIdFlag)?;

        match block_id_flag {
            BlockIdFlag::Unknown => Err(TryFromCommitSigError::UnknownBlockIdFlag),
            BlockIdFlag::Absent => {
                if !value.validator_address.is_empty() {
                    Err(TryFromCommitSigError::AbsentWithValidatorAddress)
                } else if value
                    .timestamp
                    .is_some_and(|ts| ts != Timestamp::default().into())
                {
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
                signature: value.signature,
            }),
            BlockIdFlag::Nil => Ok(Self::Nil {
                validator_address: value
                    .validator_address
                    .try_into()
                    .map_err(TryFromCommitSigError::ValidatorAddress)?,
                timestamp: required!(value.timestamp)?
                    .try_into()
                    .map_err(TryFromCommitSigError::Timestamp)?,
                signature: value.signature,
            }),
        }
    }
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
                } else if value
                    .timestamp
                    .is_some_and(|ts| ts != Timestamp::default().into())
                {
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
                signature: value.signature,
            }),
            BlockIdFlag::Nil => Ok(Self::Nil {
                validator_address: value
                    .validator_address
                    .try_into()
                    .map_err(TryFromCommitSigError::ValidatorAddress)?,
                timestamp: required!(value.timestamp)?
                    .try_into()
                    .map_err(TryFromCommitSigError::Timestamp)?,
                signature: value.signature,
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
                validator_address: validator_address.get().into(),
                timestamp: timestamp.into(),
                signature: signature.into(),
            },
            CommitSig::Nil {
                validator_address,
                timestamp,
                signature,
            } => Self {
                block_id_flag: BlockIdFlag::Nil.into(),
                validator_address: validator_address.get().into(),
                timestamp: timestamp.into(),
                signature: signature.into(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cometbft::types::commit_sig::CommitSig;

    #[test]
    fn proto_json() {
        let json = r#"
          {
            "block_id_flag": 1,
            "validator_address": "",
            "timestamp": "0001-01-01T00:00:00Z",
            "signature": null
          }
        "#;

        let proto = serde_json::from_str::<protos::cometbft::types::v1::CommitSig>(json).unwrap();

        assert_eq!(CommitSig::try_from(proto).unwrap(), CommitSig::Absent);
    }
}
