use serde::{Deserialize, Serialize};
use unionlabs::{
    errors::UnknownEnumVariant,
    google::protobuf::timestamp::{Timestamp, TryFromTimestampError},
    primitives::{
        Bytes, FixedBytesError, H160,
        encoding::{Base64, HexUnprefixed},
    },
};

use crate::types::block_id_flag::BlockIdFlag;

/// Specialized CommitSig structure for CometBFT
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "CommitSigRaw", into = "CommitSigRaw")]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
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

/// Raw CommitSig struct, able to represent all possible CometBFT-compatible commit sigs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct CommitSigRaw {
    pub block_id_flag: i32,
    #[serde(with = "commit_sig_validator_address")]
    pub validator_address: Option<H160<HexUnprefixed>>,
    #[serde(
        default,
        with = "::serde_utils::parse_from_rfc3339_string_but_0001_01_01T00_00_00Z_is_none"
    )]
    pub timestamp: Option<Timestamp>,
    pub signature: Option<Bytes<Base64>>,
}

mod commit_sig_validator_address {
    use serde::{Deserialize, Deserializer, Serializer, de};
    use unionlabs::primitives::{H160, encoding::HexUnprefixed};

    pub fn serialize<S>(
        data: &Option<H160<HexUnprefixed>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match data {
            Some(validator_address) => serializer.collect_str(validator_address),
            None => serializer.serialize_str(""),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<H160<HexUnprefixed>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            Ok(None)
        } else {
            s.parse().map_err(de::Error::custom).map(Some)
        }
    }
}

impl From<CommitSig> for CommitSigRaw {
    fn from(value: CommitSig) -> Self {
        match value {
            CommitSig::Absent => Self {
                block_id_flag: BlockIdFlag::Absent.into(),
                validator_address: Default::default(),
                timestamp: None,
                signature: None,
            },
            CommitSig::Commit {
                validator_address,
                timestamp,
                signature,
            } => Self {
                block_id_flag: BlockIdFlag::Commit.into(),
                validator_address: Some(validator_address.into_encoding()),
                timestamp: Some(timestamp),
                signature: Some(signature.into_encoding()),
            },
            CommitSig::Nil {
                validator_address,
                timestamp,
                signature,
            } => Self {
                block_id_flag: BlockIdFlag::Nil.into(),
                validator_address: Some(validator_address.into_encoding()),
                timestamp: Some(timestamp),
                signature: Some(signature.into_encoding()),
            },
        }
    }
}

impl TryFrom<CommitSigRaw> for CommitSig {
    type Error = Error;

    fn try_from(value: CommitSigRaw) -> Result<Self, Self::Error> {
        let block_id_flag = BlockIdFlag::try_from(value.block_id_flag)?;

        match block_id_flag {
            BlockIdFlag::Unknown => Err(Error::UnknownBlockIdFlag),
            BlockIdFlag::Absent => {
                if value.validator_address.is_some() {
                    Err(Error::AbsentWithValidatorAddress)
                } else if value.timestamp.is_some_and(|ts| ts != Timestamp::default()) {
                    Err(Error::AbsentWithTimestamp)
                } else if !value.signature.unwrap_or_default().is_empty() {
                    Err(Error::AbsentWithSignature)
                } else {
                    Ok(Self::Absent)
                }
            }
            BlockIdFlag::Commit => Ok(Self::Commit {
                validator_address: value
                    .validator_address
                    .ok_or(Error::CommitMissingValidatorAddress)?
                    .into_encoding(),
                timestamp: value.timestamp.ok_or(Error::CommitMissingTimestamp)?,
                signature: value
                    .signature
                    .ok_or(Error::CommitMissingSignature)?
                    .into_encoding(),
            }),
            BlockIdFlag::Nil => Ok(Self::Nil {
                validator_address: value
                    .validator_address
                    .ok_or(Error::NilMissingValidatorAddress)?
                    .into_encoding(),
                timestamp: value.timestamp.ok_or(Error::NilMissingTimestamp)?,
                signature: value
                    .signature
                    .ok_or(Error::NilMissingSignature)?
                    .into_encoding(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("invalid validator_address")]
    ValidatorAddress(#[from] FixedBytesError),
    #[error("invalid block_id_flag")]
    BlockIdFlag(#[from] UnknownEnumVariant<i32>),
    #[error("invalid timestamp")]
    Timestamp(#[from] TryFromTimestampError),

    #[error("block id flag was `Unknown`")]
    UnknownBlockIdFlag,

    #[error("an absent commit sig had an address")]
    AbsentWithValidatorAddress,
    #[error("an absent/agg absent commit sig had a timestamp")]
    AbsentWithTimestamp,
    #[error("an absent/agg absent/agg nill absent  commit sig had a signature")]
    AbsentWithSignature,

    #[error("a commit commit sig requires validator address to be set")]
    CommitMissingValidatorAddress,
    #[error("a commit commit sig requires timestamp to be set")]
    CommitMissingTimestamp,
    #[error("a commit commit sig requires signature to be set")]
    CommitMissingSignature,

    #[error("a nill commit sig requires validator address to be set")]
    NilMissingValidatorAddress,
    #[error("a nil commit sig requires timestamp to be set")]
    NilMissingTimestamp,
    #[error("a nil commit sig requires signature to be set")]
    NilMissingSignature,
}

/// NOTE: We provide conversions to/from both the cometbft protos and the tendermint protos. If we ever remove support for the old tendermint protos, it can be removed here as well.
#[cfg(feature = "proto")]
pub mod proto {
    use crate::types::commit_sig::{CommitSig, CommitSigRaw};

    // COMETBFT <-> CANONICAL

    impl From<CommitSigRaw> for protos::cometbft::types::v1::CommitSig {
        fn from(value: CommitSigRaw) -> Self {
            Self {
                block_id_flag: value.block_id_flag,
                validator_address: value
                    .validator_address
                    .map_or_else(Vec::new, |validator_address| validator_address.into()),
                timestamp: value.timestamp.map(Into::into),
                signature: value.signature.unwrap_or_default().into(),
            }
        }
    }

    impl From<CommitSig> for protos::cometbft::types::v1::CommitSig {
        fn from(value: CommitSig) -> Self {
            CommitSigRaw::from(value).into()
        }
    }

    impl TryFrom<protos::cometbft::types::v1::CommitSig> for CommitSig {
        type Error = super::Error;

        fn try_from(value: protos::cometbft::types::v1::CommitSig) -> Result<Self, Self::Error> {
            CommitSigRaw {
                block_id_flag: value.block_id_flag,
                validator_address: if value.validator_address.is_empty() {
                    None
                } else {
                    Some(value.validator_address.try_into()?)
                },
                timestamp: value.timestamp.map(TryInto::try_into).transpose()?,
                signature: Some(value.signature.into()),
            }
            .try_into()
        }
    }

    // TENDERMINT <-> CANONICAL

    impl From<CommitSigRaw> for protos::tendermint::types::CommitSig {
        fn from(value: CommitSigRaw) -> Self {
            Self {
                block_id_flag: value.block_id_flag,
                validator_address: value
                    .validator_address
                    .map_or_else(Vec::new, |validator_address| validator_address.into()),
                timestamp: value.timestamp.map(Into::into),
                signature: value.signature.unwrap_or_default().into(),
            }
        }
    }

    impl From<CommitSig> for protos::tendermint::types::CommitSig {
        fn from(value: CommitSig) -> Self {
            CommitSigRaw::from(value).into()
        }
    }

    impl TryFrom<protos::tendermint::types::CommitSig> for CommitSig {
        type Error = super::Error;

        fn try_from(value: protos::tendermint::types::CommitSig) -> Result<Self, Self::Error> {
            CommitSigRaw {
                block_id_flag: value.block_id_flag,
                validator_address: if value.validator_address.is_empty() {
                    None
                } else {
                    Some(value.validator_address.try_into()?)
                },
                timestamp: value.timestamp.map(TryInto::try_into).transpose()?,
                signature: Some(value.signature.into()),
            }
            .try_into()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::commit_sig::CommitSig;

    // #[test]
    // fn proto_json() {
    //     let json = r#"
    //       {
    //         "block_id_flag": 1,
    //         "validator_address": "",
    //         "timestamp": "0001-01-01T00:00:00Z",
    //         "signature": null
    //       }
    //     "#;

    //     let proto = serde_json::from_str::<protos::cometbft::types::v1::CommitSig>(json).unwrap();

    //     assert_eq!(CommitSig::try_from(proto).unwrap(), CommitSig::Absent);
    // }

    #[test]
    fn json_absent() {
        let json = r#"
          {
            "block_id_flag": 1,
            "validator_address": "",
            "timestamp": "0001-01-01T00:00:00Z",
            "signature": null
          }
        "#;

        let sig = serde_json::from_str::<CommitSig>(json).unwrap();

        assert_eq!(sig, CommitSig::Absent);
    }

    #[test]
    fn json_commit() {
        let json = r#"
            {
                "block_id_flag": 2,
                "validator_address": "3F9FEED6073BAF751CDCC6F9FE355435FC14B0F0",
                "timestamp": "2024-11-19T15:24:34.213395758Z",
                "signature": "7qr+0z46EhPK+6jYyL4JJpVrGRkxX+JS1kSqYdGkw4onyiqclriLkLDA7DQcdlT4v6Ky+2iG2VKRg2lO9cXKUg=="
            }
        "#;

        let sig = serde_json::from_str::<CommitSig>(json).unwrap();

        assert!(matches!(sig, CommitSig::Commit { .. }));
    }
}
