use macros::model;

use crate::{
    cometbft::p2p::{
        default_node_info_other::DefaultNodeInfoOther, protocol_version::ProtocolVersion,
    },
    errors::{required, MissingField},
};

#[model(proto(raw(protos::cometbft::p2p::v1::DefaultNodeInfo)))]
pub struct DefaultNodeInfo {
    pub protocol_version: ProtocolVersion,
    pub default_node_id: String,
    pub listen_addr: String,
    pub network: String,
    pub version: String,
    // REVIEW: Is this fixed size (10 bytes)?
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub channels: Vec<u8>,
    pub moniker: String,
    pub other: DefaultNodeInfoOther,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromDefaultNodeInfoError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
}

impl TryFrom<protos::cometbft::p2p::v1::DefaultNodeInfo> for DefaultNodeInfo {
    type Error = TryFromDefaultNodeInfoError;

    fn try_from(value: protos::cometbft::p2p::v1::DefaultNodeInfo) -> Result<Self, Self::Error> {
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
