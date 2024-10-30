use serde::{Deserialize, Serialize};
use unionlabs::{bytes::Bytes, google::protobuf::timestamp::Timestamp, hash::H160};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    try_from = "protos::cometbft::types::v1::CommitSig",
    into = "protos::cometbft::types::v1::CommitSig"
)]
pub enum CommitSig {
    Absent,
    Commit {
        validator_address: H160,
        timestamp: Timestamp,
        signature: Bytes,
    },
    Nil {
        validator_address: H160,
        timestamp: Timestamp,
        signature: Bytes,
    },
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{
        errors::{InvalidLength, MissingField, UnknownEnumVariant},
        google::protobuf::timestamp::{Timestamp, TryFromTimestampError},
        required,
    };

    use crate::types::{block_id_flag::BlockIdFlag, commit_sig::CommitSig};

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
                    signature: signature.into_vec(),
                },
                CommitSig::Nil {
                    validator_address,
                    timestamp,
                    signature,
                } => Self {
                    block_id_flag: BlockIdFlag::Nil.into(),
                    validator_address: validator_address.into(),
                    timestamp: Some(timestamp.into()),
                    signature: signature.into_vec(),
                },
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
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
        type Error = Error;

        fn try_from(value: protos::cometbft::types::v1::CommitSig) -> Result<Self, Self::Error> {
            let block_id_flag =
                BlockIdFlag::try_from(value.block_id_flag).map_err(Error::BlockIdFlag)?;

            match block_id_flag {
                BlockIdFlag::Unknown => Err(Error::UnknownBlockIdFlag),
                BlockIdFlag::Absent => {
                    if !value.validator_address.is_empty() {
                        Err(Error::AbsentWithValidatorAddress)
                    } else if value
                        .timestamp
                        .is_some_and(|ts| ts != Timestamp::default().into())
                    {
                        Err(Error::AbsentWithTimestamp)
                    } else if !value.signature.is_empty() {
                        Err(Error::AbsentWithSignature)
                    } else {
                        Ok(Self::Absent)
                    }
                }
                BlockIdFlag::Commit => Ok(Self::Commit {
                    validator_address: value
                        .validator_address
                        .try_into()
                        .map_err(Error::ValidatorAddress)?,
                    timestamp: required!(value.timestamp)?
                        .try_into()
                        .map_err(Error::Timestamp)?,
                    signature: value.signature.into(),
                }),
                BlockIdFlag::Nil => Ok(Self::Nil {
                    validator_address: value
                        .validator_address
                        .try_into()
                        .map_err(Error::ValidatorAddress)?,
                    timestamp: required!(value.timestamp)?
                        .try_into()
                        .map_err(Error::Timestamp)?,
                    signature: value.signature.into(),
                }),
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

    impl TryFrom<protos::tendermint::types::CommitSig> for CommitSig {
        type Error = Error;

        fn try_from(value: protos::tendermint::types::CommitSig) -> Result<Self, Self::Error> {
            let block_id_flag =
                BlockIdFlag::try_from(value.block_id_flag).map_err(Error::BlockIdFlag)?;

            match block_id_flag {
                BlockIdFlag::Unknown => Err(Error::UnknownBlockIdFlag),
                BlockIdFlag::Absent => {
                    if !value.validator_address.is_empty() {
                        Err(Error::AbsentWithValidatorAddress)
                    } else if value
                        .timestamp
                        .is_some_and(|ts| ts != Timestamp::default().into())
                    {
                        Err(Error::AbsentWithTimestamp)
                    } else if !value.signature.is_empty() {
                        Err(Error::AbsentWithSignature)
                    } else {
                        Ok(Self::Absent)
                    }
                }
                BlockIdFlag::Commit => Ok(Self::Commit {
                    validator_address: value
                        .validator_address
                        .try_into()
                        .map_err(Error::ValidatorAddress)?,
                    timestamp: required!(value.timestamp)?
                        .try_into()
                        .map_err(Error::Timestamp)?,
                    signature: value.signature.into(),
                }),
                BlockIdFlag::Nil => Ok(Self::Nil {
                    validator_address: value
                        .validator_address
                        .try_into()
                        .map_err(Error::ValidatorAddress)?,
                    timestamp: required!(value.timestamp)?
                        .try_into()
                        .map_err(Error::Timestamp)?,
                    signature: value.signature.into(),
                }),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::commit_sig::CommitSig;

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
