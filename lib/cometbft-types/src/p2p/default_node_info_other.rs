use core::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefaultNodeInfoOther {
    pub tx_index: TxIndex,
    pub rpc_address: String,
}

// https://github.com/cometbft/cometbft/blob/bfff14c8ab31d62e942525659e6e1dba5d49fbd8/node/node.go#L1090-L1093
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TxIndex {
    #[serde(rename = "on")]
    On,
    #[serde(rename = "off")]
    Off,
}

impl FromStr for TxIndex {
    type Err = InvalidTxIndex;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "on" => Ok(Self::On),
            "off" => Ok(Self::Off),
            _ => Err(InvalidTxIndex(s.to_owned())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("invalid tx_index value, expected either `on` or `off` but found `{0}`")]
pub struct InvalidTxIndex(String);

impl fmt::Display for TxIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxIndex::On => write!(f, "on"),
            TxIndex::Off => write!(f, "off"),
        }
    }
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::p2p::default_node_info_other::{DefaultNodeInfoOther, InvalidTxIndex};

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid tx_index")]
        TxIndex(#[from] InvalidTxIndex),
    }

    impl TryFrom<protos::tendermint::p2p::DefaultNodeInfoOther> for DefaultNodeInfoOther {
        type Error = Error;

        fn try_from(
            value: protos::tendermint::p2p::DefaultNodeInfoOther,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                tx_index: value.tx_index.parse()?,
                rpc_address: value.rpc_address,
            })
        }
    }
}
