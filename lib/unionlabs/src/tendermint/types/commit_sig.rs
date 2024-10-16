use macros::model;

use crate::{google::protobuf::timestamp::Timestamp, hash::H160};

#[model(proto(raw(protos::tendermint::types::CommitSig), into, from))]
pub enum CommitSig {
    Absent,
    Commit {
        validator_address: H160,
        timestamp: Timestamp,
        #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
        #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
        signature: Vec<u8>,
    },
    Nil {
        validator_address: H160,
        timestamp: Timestamp,
        #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
        #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
        signature: Vec<u8>,
    },
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        errors::{required, InvalidLength, MissingField, UnknownEnumVariant},
        google::protobuf::timestamp::{proto::TryFromTimestampError, Timestamp},
        tendermint::types::{block_id_flag::BlockIdFlag, commit_sig::CommitSig},
    };

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
}

#[cfg(test)]
mod tests {
    use crate::tendermint::types::commit_sig::CommitSig;

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

        let proto = serde_json::from_str::<protos::tendermint::types::CommitSig>(json).unwrap();

        assert_eq!(CommitSig::try_from(proto).unwrap(), CommitSig::Absent);
    }
}
