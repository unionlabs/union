use enumorph::Enumorph;
use sha3::{Digest, Keccak256};
use unionlabs::primitives::{Bytes, H256, U256};
use voyager_core::IbcStorePathKey;

use super::IbcUnion;
use crate::types::{Channel, ChannelId, ClientId, Connection, ConnectionId};

/// 0x0100000000000000000000000000000000000000000000000000000000000000
pub const COMMITMENT_MAGIC: H256 = {
    let mut bz = [0; 32];
    bz[0] = 1;
    H256::new(bz)
};

pub const COMMITMENT_NULL: H256 = H256::new([0; 32]);

pub const CLIENT_STATE: U256 = U256::from_limbs([0, 0, 0, 0]);
pub const CONSENSUS_STATE: U256 = U256::from_limbs([1, 0, 0, 0]);
pub const CONNECTIONS: U256 = U256::from_limbs([2, 0, 0, 0]);
pub const CHANNELS: U256 = U256::from_limbs([3, 0, 0, 0]);
pub const PACKETS: U256 = U256::from_limbs([4, 0, 0, 0]);
pub const PACKET_ACKS: U256 = U256::from_limbs([5, 0, 0, 0]);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Enumorph)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "@type", content = "@value", rename_all = "snake_case")
)]
pub enum StorePath {
    ClientState(ClientStatePath),
    ConsensusState(ConsensusStatePath),
    Connection(ConnectionPath),
    Channel(ChannelPath),
    BatchReceipts(BatchReceiptsPath),
    BatchPackets(BatchPacketsPath),
}

impl StorePath {
    #[must_use]
    pub fn key(&self) -> H256 {
        match self {
            StorePath::ClientState(path) => path.key(),
            StorePath::ConsensusState(path) => path.key(),
            StorePath::Connection(path) => path.key(),
            StorePath::Channel(path) => path.key(),
            StorePath::BatchReceipts(path) => path.key(),
            StorePath::BatchPackets(path) => path.key(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct ClientStatePath {
    pub client_id: ClientId,
}

impl ClientStatePath {
    #[must_use]
    pub fn key(&self) -> H256 {
        Keccak256::new()
            .chain_update(CLIENT_STATE.to_be_bytes())
            .chain_update(U256::from(self.client_id).to_be_bytes())
            .finalize()
            .into()
    }
}

impl IbcStorePathKey for ClientStatePath {
    type Spec = IbcUnion;

    type Value = Option<Bytes>;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct ConsensusStatePath {
    pub client_id: ClientId,
    pub height: u64,
}

impl ConsensusStatePath {
    #[must_use]
    pub fn key(&self) -> H256 {
        Keccak256::new()
            .chain_update(CONSENSUS_STATE.to_be_bytes())
            .chain_update(U256::from(self.client_id).to_be_bytes())
            .chain_update(U256::from(self.height).to_be_bytes())
            .finalize()
            .into()
    }
}

impl IbcStorePathKey for ConsensusStatePath {
    type Spec = IbcUnion;

    type Value = Option<Bytes>;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct ConnectionPath {
    pub connection_id: ConnectionId,
}

impl ConnectionPath {
    #[must_use]
    pub fn key(&self) -> H256 {
        Keccak256::new()
            .chain_update(CONNECTIONS.to_be_bytes())
            .chain_update(U256::from(self.connection_id).to_be_bytes())
            .finalize()
            .into()
    }
}

impl IbcStorePathKey for ConnectionPath {
    type Spec = IbcUnion;

    type Value = Option<Connection>;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct ChannelPath {
    pub channel_id: ChannelId,
}

impl ChannelPath {
    #[must_use]
    pub fn key(&self) -> H256 {
        Keccak256::new()
            .chain_update(CHANNELS.to_be_bytes())
            .chain_update(U256::from(self.channel_id).to_be_bytes())
            .finalize()
            .into()
    }
}

impl IbcStorePathKey for ChannelPath {
    type Spec = IbcUnion;

    type Value = Option<Channel>;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct BatchReceiptsPath {
    pub channel_id: ChannelId,
    pub batch_hash: H256,
}

impl BatchReceiptsPath {
    #[must_use]
    pub fn key(&self) -> H256 {
        Keccak256::new()
            .chain_update(PACKET_ACKS.to_be_bytes())
            .chain_update(U256::from(self.channel_id).to_be_bytes())
            .chain_update(self.batch_hash)
            .finalize()
            .into()
    }
}

impl IbcStorePathKey for BatchReceiptsPath {
    type Spec = IbcUnion;

    type Value = H256;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct BatchPacketsPath {
    pub channel_id: ChannelId,
    pub batch_hash: H256,
}

impl BatchPacketsPath {
    #[must_use]
    pub fn key(&self) -> H256 {
        Keccak256::new()
            .chain_update(PACKETS.to_be_bytes())
            .chain_update(U256::from(self.channel_id).to_be_bytes())
            .chain_update(self.batch_hash)
            .finalize()
            .into()
    }
}

impl IbcStorePathKey for BatchPacketsPath {
    type Spec = IbcUnion;

    type Value = H256;
}
