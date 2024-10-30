use serde::{Deserialize, Serialize};
use unionlabs::{bytes::Bytes, hash::hash_v2::Base64};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Data {
    pub txs: Vec<Bytes<Base64>>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::types::data::Data;

    impl From<Data> for protos::cometbft::types::v1::Data {
        fn from(value: Data) -> Self {
            Self {
                txs: value.txs.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl From<protos::cometbft::types::v1::Data> for Data {
        fn from(value: protos::cometbft::types::v1::Data) -> Self {
            Self {
                txs: value.txs.into_iter().map(Into::into).collect(),
            }
        }
    }
}
