use serde::{Deserialize, Serialize};
use unionlabs::{
    bytes::Bytes,
    errors::{InvalidLength, UnknownEnumVariant},
    google::protobuf::timestamp::{Timestamp, TryFromTimestampError},
    hash::H160,
};

use crate::types::block_id_flag::BlockIdFlag;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "CommitSigRaw", into = "CommitSigRaw")]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitSigRaw {
    pub block_id_flag: i32,
    #[serde(with = "::serde_utils::hex_upper_unprefixed")]
    pub validator_address: Vec<u8>,
    #[serde(
        default,
        with = "::serde_utils::parse_from_rfc3339_string_but_0001_01_01T00_00_00Z_is_none"
    )]
    pub timestamp: Option<Timestamp>,
    #[serde(with = "::serde_utils::base64_opt_default")]
    pub signature: Vec<u8>,
}

impl From<CommitSig> for CommitSigRaw {
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
                timestamp: Some(timestamp),
                signature: signature.into_vec(),
            },
            CommitSig::Nil {
                validator_address,
                timestamp,
                signature,
            } => Self {
                block_id_flag: BlockIdFlag::Nil.into(),
                validator_address: validator_address.into(),
                timestamp: Some(timestamp),
                signature: signature.into_vec(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("invalid validator_address")]
    ValidatorAddress(#[from] InvalidLength),
    #[error("invalid block_id_flag")]
    BlockIdFlag(#[from] UnknownEnumVariant<i32>),
    #[error("invalid timestamp")]
    Timestamp(#[from] TryFromTimestampError),
    #[error("block id flag was `Unknown`")]
    UnknownBlockIdFlag,
    #[error("an absent commit sig had an address")]
    AbsentWithValidatorAddress,
    #[error("an absent commit sig had a timestamp")]
    AbsentWithTimestamp,
    #[error("an absent commit sig had a signature")]
    AbsentWithSignature,
    #[error("a commit commit sig requires timestamp to be set")]
    CommitMissingTimestamp,
    #[error("a nil commit sig requires timestamp to be set")]
    NilMissingTimestamp,
}

impl TryFrom<CommitSigRaw> for CommitSig {
    type Error = Error;

    fn try_from(value: CommitSigRaw) -> Result<Self, Self::Error> {
        let block_id_flag = BlockIdFlag::try_from(value.block_id_flag)?;

        match block_id_flag {
            BlockIdFlag::Unknown => Err(Error::UnknownBlockIdFlag),
            BlockIdFlag::Absent => {
                if !value.validator_address.is_empty() {
                    Err(Error::AbsentWithValidatorAddress)
                } else if value.timestamp.is_some_and(|ts| ts != Timestamp::default()) {
                    Err(Error::AbsentWithTimestamp)
                } else if !value.signature.is_empty() {
                    Err(Error::AbsentWithSignature)
                } else {
                    Ok(Self::Absent)
                }
            }
            BlockIdFlag::Commit => Ok(Self::Commit {
                validator_address: value.validator_address.try_into()?,
                timestamp: value.timestamp.ok_or(Error::CommitMissingTimestamp)?,
                signature: value.signature.into(),
            }),
            BlockIdFlag::Nil => Ok(Self::Nil {
                validator_address: value.validator_address.try_into()?,
                timestamp: value.timestamp.ok_or(Error::NilMissingTimestamp)?,
                signature: value.signature.into(),
            }),
        }
    }
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::types::{
        block_id_flag::BlockIdFlag,
        commit_sig::{CommitSig, CommitSigRaw},
    };

    // COMETBFT <-> CANONICAL

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

    impl TryFrom<protos::cometbft::types::v1::CommitSig> for CommitSig {
        type Error = super::Error;

        fn try_from(value: protos::cometbft::types::v1::CommitSig) -> Result<Self, Self::Error> {
            CommitSigRaw {
                block_id_flag: value.block_id_flag,
                validator_address: value.validator_address,
                timestamp: value.timestamp.map(TryInto::try_into).transpose()?,
                signature: value.signature,
            }
            .try_into()
        }
    }

    // TENDERMINT <-> CANONICAL

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
        type Error = super::Error;

        fn try_from(value: protos::tendermint::types::CommitSig) -> Result<Self, Self::Error> {
            CommitSigRaw {
                block_id_flag: value.block_id_flag,
                validator_address: value.validator_address,
                timestamp: value.timestamp.map(TryInto::try_into).transpose()?,
                signature: value.signature,
            }
            .try_into()
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
