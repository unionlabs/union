use serde::{Deserialize, Serialize};

use crate::types::{commit::Commit, header::Header};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct SignedHeader {
    pub header: Header,
    pub commit: Commit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct SignedHeader2 {
    pub header: Header,
    pub commit: super::commit::TmpCommit,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{errors::MissingField, required};

    use crate::types::{commit, header, signed_header::SignedHeader};

    impl From<SignedHeader> for protos::cometbft::types::v1::SignedHeader {
        fn from(value: SignedHeader) -> Self {
            Self {
                header: Some(value.header.into()),
                commit: Some(value.commit.into()),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid header")]
        Header(#[source] header::proto::Error),
        #[error("invalid commit")]
        Commit(#[source] commit::proto::Error),
    }

    impl TryFrom<protos::cometbft::types::v1::SignedHeader> for SignedHeader {
        type Error = Error;

        fn try_from(value: protos::cometbft::types::v1::SignedHeader) -> Result<Self, Self::Error> {
            Ok(Self {
                header: required!(value.header)?.try_into().map_err(Error::Header)?,
                commit: required!(value.commit)?.try_into().map_err(Error::Commit)?,
            })
        }
    }

    impl From<SignedHeader> for protos::tendermint::types::SignedHeader {
        fn from(value: SignedHeader) -> Self {
            Self {
                header: Some(value.header.into()),
                commit: Some(value.commit.into()),
            }
        }
    }

    impl TryFrom<protos::tendermint::types::SignedHeader> for SignedHeader {
        type Error = Error;

        fn try_from(value: protos::tendermint::types::SignedHeader) -> Result<Self, Self::Error> {
            Ok(Self {
                header: required!(value.header)?.try_into().map_err(Error::Header)?,
                commit: required!(value.commit)?.try_into().map_err(Error::Commit)?,
            })
        }
    }
}
