use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefaultNodeInfoOther {
    // REVIEW: Is this more strongly typed? So far all i've seen is "on" and "off"
    pub tx_index: String,
    pub rpc_address: String,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::p2p::default_node_info_other::DefaultNodeInfoOther;

    impl From<protos::tendermint::p2p::DefaultNodeInfoOther> for DefaultNodeInfoOther {
        fn from(value: protos::tendermint::p2p::DefaultNodeInfoOther) -> Self {
            Self {
                tx_index: value.tx_index,
                rpc_address: value.rpc_address,
            }
        }
    }
}
