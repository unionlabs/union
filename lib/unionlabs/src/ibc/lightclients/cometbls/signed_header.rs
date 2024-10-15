use macros::model;

use crate::{bounded::BoundedI64, google::protobuf::timestamp::Timestamp, hash::H256};

#[model(
    proto(
        raw(protos::union::ibc::lightclients::cometbls::v1::SignedHeader),
        into,
        from
    ),
    ethabi(raw(ibc_solidity::cometbls::SignedHeader), into, from)
)]
pub struct SignedHeader {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub height: BoundedI64<0, { i64::MAX }>,
    pub time: Timestamp,
    pub validators_hash: H256,
    pub next_validators_hash: H256,
    pub app_hash: H256,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        bounded::BoundedIntError,
        errors::{required, InvalidLength, MissingField},
        google::protobuf::timestamp::proto::TryFromTimestampError,
        ibc::lightclients::cometbls::signed_header::SignedHeader,
    };

    impl From<SignedHeader> for protos::union::ibc::lightclients::cometbls::v1::SignedHeader {
        fn from(value: SignedHeader) -> Self {
            Self {
                height: value.height.into(),
                time: Some(value.time.into()),
                validators_hash: value.validators_hash.into(),
                next_validators_hash: value.next_validators_hash.into(),
                app_hash: value.app_hash.into(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum TryFromSignedHeaderError {
        #[error(transparent)]
        MissingField(MissingField),
        #[error("invalid height")]
        Height(#[source] BoundedIntError<i64>),
        #[error("invalid timestamp")]
        Timestamp(#[from] TryFromTimestampError),
        #[error("invalid validators hash")]
        ValidatorsHash(#[source] InvalidLength),
        #[error("invalid next validators hash")]
        NextValidatorsHash(#[source] InvalidLength),
        #[error("invalid app hash")]
        AppHash(#[source] InvalidLength),
    }

    impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::SignedHeader> for SignedHeader {
        type Error = TryFromSignedHeaderError;

        fn try_from(
            value: protos::union::ibc::lightclients::cometbls::v1::SignedHeader,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                height: value
                    .height
                    .try_into()
                    .map_err(TryFromSignedHeaderError::Height)?,
                time: required!(value.time)?.try_into()?,
                validators_hash: value
                    .validators_hash
                    .try_into()
                    .map_err(TryFromSignedHeaderError::ValidatorsHash)?,
                next_validators_hash: value
                    .next_validators_hash
                    .try_into()
                    .map_err(TryFromSignedHeaderError::NextValidatorsHash)?,
                app_hash: value
                    .app_hash
                    .try_into()
                    .map_err(TryFromSignedHeaderError::AppHash)?,
            })
        }
    }
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use crate::{
        bounded::{BoundedI32, BoundedI64, BoundedIntError},
        google::protobuf::timestamp::Timestamp,
        ibc::lightclients::cometbls::signed_header::SignedHeader,
    };

    impl From<SignedHeader> for ibc_solidity::cometbls::SignedHeader {
        fn from(value: SignedHeader) -> Self {
            Self {
                height: value
                    .height
                    .inner()
                    .try_into()
                    .expect("value is >= 0; qed;"),
                secs: value
                    .time
                    .seconds
                    .inner()
                    .try_into()
                    .expect("value is >= 0; qed;"),
                nanos: value
                    .time
                    .nanos
                    .inner()
                    .try_into()
                    .expect("value is >= 0; qed;"),
                validatorsHash: value.validators_hash.into(),
                nextValidatorsHash: value.next_validators_hash.into(),
                appHash: value.app_hash.into(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum TryFromEthAbiSignedHeaderError {
        Height(BoundedIntError<i64, u64>),
        Secs(BoundedIntError<i64, u64>),
        Nanos(BoundedIntError<i32, u64>),
    }

    impl TryFrom<ibc_solidity::cometbls::SignedHeader> for SignedHeader {
        type Error = TryFromEthAbiSignedHeaderError;

        fn try_from(value: ibc_solidity::cometbls::SignedHeader) -> Result<Self, Self::Error> {
            Ok(Self {
                height: BoundedI64::new(value.height)
                    .map_err(TryFromEthAbiSignedHeaderError::Height)?,
                time: Timestamp {
                    seconds: BoundedI64::new(value.secs)
                        .map_err(TryFromEthAbiSignedHeaderError::Secs)?,
                    nanos: BoundedI32::new(value.nanos)
                        .map_err(TryFromEthAbiSignedHeaderError::Nanos)?,
                },
                validators_hash: value.validatorsHash.into(),
                next_validators_hash: value.nextValidatorsHash.into(),
                app_hash: value.appHash.into(),
            })
        }
    }
}
