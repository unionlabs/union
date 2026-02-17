use serde::{Deserialize, Serialize};
use unionlabs::primitives::{H256, encoding::Base64};

use crate::CometbftHeight;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoResponse {
    pub data: String,
    pub version: String,
    #[serde(
        with = "::serde_utils::string",
        default,
        skip_serializing_if = "is_zero"
    )]
    pub app_version: u64,
    pub last_block_height: CometbftHeight,
    pub last_block_app_hash: H256<Base64>,
}

const fn is_zero(n: &u64) -> bool {
    *n == 0
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{bounded::BoundedIntError, primitives::FixedBytesError};

    use crate::abci::info_response::InfoResponse;

    impl From<InfoResponse> for protos::cometbft::abci::v1::InfoResponse {
        fn from(value: InfoResponse) -> Self {
            Self {
                data: value.data,
                version: value.version,
                app_version: value.app_version,
                last_block_height: value.last_block_height.into(),
                last_block_app_hash: value.last_block_app_hash.into(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid last block height")]
        LastBlockHeight(#[from] BoundedIntError<i64>),
        #[error("invalid last block app hash")]
        LastBlockAppHash(#[from] FixedBytesError),
    }

    impl TryFrom<protos::cometbft::abci::v1::InfoResponse> for InfoResponse {
        type Error = Error;

        fn try_from(value: protos::cometbft::abci::v1::InfoResponse) -> Result<Self, Self::Error> {
            Ok(Self {
                data: value.data,
                version: value.version,
                app_version: value.app_version,
                last_block_height: value.last_block_height.try_into()?,
                last_block_app_hash: value.last_block_app_hash.try_into()?,
            })
        }
    }
}
