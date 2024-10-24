use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub p2p: u64,
    pub block: u64,
    pub app: u64,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::p2p::protocol_version::ProtocolVersion;

    impl From<protos::tendermint::p2p::ProtocolVersion> for ProtocolVersion {
        fn from(value: protos::tendermint::p2p::ProtocolVersion) -> Self {
            Self {
                p2p: value.p2p,
                block: value.block,
                app: value.app,
            }
        }
    }
}
