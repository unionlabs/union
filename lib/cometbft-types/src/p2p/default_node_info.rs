use serde::{Deserialize, Serialize};

use crate::p2p::{
    default_node_info_other::DefaultNodeInfoOther, protocol_version::ProtocolVersion,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefaultNodeInfo {
    pub protocol_version: ProtocolVersion,
    pub default_node_id: String,
    pub listen_addr: String,
    pub network: String,
    pub version: String,
    // REVIEW: Is this fixed size (10 bytes)?
    #[serde(with = "::serde_utils::hex_string")]
    pub channels: Vec<u8>,
    pub moniker: String,
    pub other: DefaultNodeInfoOther,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{errors::MissingField, required};

    use crate::p2p::default_node_info::DefaultNodeInfo;

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
    }

    impl TryFrom<protos::tendermint::p2p::DefaultNodeInfo> for DefaultNodeInfo {
        type Error = Error;

        fn try_from(value: protos::tendermint::p2p::DefaultNodeInfo) -> Result<Self, Self::Error> {
            Ok(Self {
                protocol_version: required!(value.protocol_version)?.into(),
                default_node_id: value.default_node_id,
                listen_addr: value.listen_addr,
                network: value.network,
                version: value.version,
                channels: value.channels,
                moniker: value.moniker,
                other: required!(value.other)?.into(),
            })
        }
    }
}
