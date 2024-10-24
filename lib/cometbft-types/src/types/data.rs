use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Data {
    #[serde(with = "::serde_utils::hex_string_list")]
    pub txs: Vec<Vec<u8>>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::types::data::Data;

    impl From<Data> for protos::tendermint::types::Data {
        fn from(value: Data) -> Self {
            Self { txs: value.txs }
        }
    }

    impl From<protos::tendermint::types::Data> for Data {
        fn from(value: protos::tendermint::types::Data) -> Self {
            Self { txs: value.txs }
        }
    }
}
