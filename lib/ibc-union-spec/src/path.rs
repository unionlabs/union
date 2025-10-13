use enumorph::Enumorph;
use sha3::{Digest, Keccak256};
use unionlabs::primitives::{Bytes, H256, U256};
use voyager_primitives::IbcStorePathKey;

#[cfg(feature = "ethabi")]
use crate::Packet;
use crate::{
    Channel, Connection, IbcUnion,
    types::{ChannelId, ClientId, ConnectionId},
};

pub const IBC_UNION_COSMWASM_COMMITMENT_PREFIX: [u8; 1] = [0x00];

/// 0x0100000000000000000000000000000000000000000000000000000000000000
pub const COMMITMENT_MAGIC: H256 = {
    let mut bz = [0; 32];
    bz[0] = 1;
    H256::new(bz)
};

/// 0x0200000000000000000000000000000000000000000000000000000000000000
pub const COMMITMENT_MAGIC_ACK: H256 = {
    let mut bz = [0; 32];
    bz[0] = 2;
    H256::new(bz)
};

/// 0x0000000000000000000000000000000000000000000000000000000000000001
pub const NON_MEMBERSHIP_COMMITMENT_VALUE: H256 = {
    let mut bz = [0; 32];
    bz[31] = 1;
    H256::new(bz)
};

pub const CLIENT_STATE: U256 = U256::from_limbs([0, 0, 0, 0]);
pub const CONSENSUS_STATE: U256 = U256::from_limbs([1, 0, 0, 0]);
pub const CONNECTIONS: U256 = U256::from_limbs([2, 0, 0, 0]);
pub const CHANNELS: U256 = U256::from_limbs([3, 0, 0, 0]);
pub const PACKETS: U256 = U256::from_limbs([4, 0, 0, 0]);
pub const PACKET_ACKS: U256 = U256::from_limbs([5, 0, 0, 0]);
pub const MEMBERSHIP_PROOF: U256 = U256::from_limbs([6, 0, 0, 0]);
pub const NON_MEMBERSHIP_PROOF: U256 = U256::from_limbs([7, 0, 0, 0]);
pub const PACKET_TIMEOUTS: U256 = U256::from_limbs([8, 0, 0, 0]);

#[cfg(feature = "ethabi")]
#[must_use]
pub fn commit_packets(packets: &[Packet]) -> H256 {
    use alloy_sol_types::SolValue;

    Keccak256::new()
        .chain_update(packets.abi_encode())
        .finalize()
        .into()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Enumorph)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum StorePath {
    ClientState(ClientStatePath),
    ConsensusState(ConsensusStatePath),
    Connection(ConnectionPath),
    Channel(ChannelPath),
    BatchReceipts(BatchReceiptsPath),
    BatchPackets(BatchPacketsPath),
    MembershipProof(MembershipProofPath),
    NonMembershipProof(NonMembershipProofPath),
    BatchTimeouts(BatchTimeoutPath),
}

impl StorePath {
    #[must_use]
    pub fn key(&self) -> H256 {
        match self {
            Self::ClientState(path) => path.key(),
            Self::ConsensusState(path) => path.key(),
            Self::Connection(path) => path.key(),
            Self::Channel(path) => path.key(),
            Self::BatchReceipts(path) => path.key(),
            Self::BatchPackets(path) => path.key(),
            Self::MembershipProof(path) => path.key(),
            Self::NonMembershipProof(path) => path.key(),
            Self::BatchTimeouts(path) => path.key(),
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
            .chain_update(U256::from(self.client_id.get()).to_be_bytes())
            .finalize()
            .into()
    }
}

impl IbcStorePathKey for ClientStatePath {
    type Spec = IbcUnion;

    type Value = Bytes;
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
            .chain_update(U256::from(self.client_id.get()).to_be_bytes())
            .chain_update(U256::from(self.height).to_be_bytes())
            .finalize()
            .into()
    }
}

impl IbcStorePathKey for ConsensusStatePath {
    type Spec = IbcUnion;

    type Value = Bytes;
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
            .chain_update(U256::from(self.connection_id.get()).to_be_bytes())
            .finalize()
            .into()
    }
}

impl IbcStorePathKey for ConnectionPath {
    type Spec = IbcUnion;

    type Value = Connection;
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
            .chain_update(U256::from(self.channel_id.get()).to_be_bytes())
            .finalize()
            .into()
    }
}

impl IbcStorePathKey for ChannelPath {
    type Spec = IbcUnion;

    type Value = Channel;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct BatchReceiptsPath {
    pub batch_hash: H256,
}

impl BatchReceiptsPath {
    #[cfg(feature = "ethabi")]
    #[must_use]
    pub fn from_packets(packets: &[Packet]) -> Self {
        Self {
            batch_hash: commit_packets(packets),
        }
    }

    #[must_use]
    pub fn key(&self) -> H256 {
        Keccak256::new()
            .chain_update(PACKET_ACKS.to_be_bytes())
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
    pub batch_hash: H256,
}

impl BatchPacketsPath {
    #[cfg(feature = "ethabi")]
    #[must_use]
    pub fn from_packets(packets: &[Packet]) -> Self {
        use alloy_sol_types::SolValue;
        Self {
            batch_hash: Keccak256::new()
                .chain_update(packets.abi_encode())
                .finalize()
                .into(),
        }
    }

    #[must_use]
    pub fn key(&self) -> H256 {
        Keccak256::new()
            .chain_update(PACKETS.to_be_bytes())
            .chain_update(self.batch_hash)
            .finalize()
            .into()
    }
}

impl IbcStorePathKey for BatchPacketsPath {
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
pub struct MembershipProofPath {
    pub client_id: ClientId,
    pub proof_height: u64,
    pub path: Bytes,
}

impl MembershipProofPath {
    #[must_use]
    pub fn key(&self) -> H256 {
        Keccak256::new()
            .chain_update(MEMBERSHIP_PROOF.to_be_bytes())
            .chain_update(U256::from(self.client_id.get()).to_be_bytes())
            .chain_update(U256::from(self.proof_height).to_be_bytes())
            .chain_update(&self.path)
            .finalize()
            .into()
    }
}

impl IbcStorePathKey for MembershipProofPath {
    type Spec = IbcUnion;

    type Value = Bytes;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct NonMembershipProofPath {
    pub client_id: ClientId,
    pub proof_height: u64,
    pub path: Bytes,
}

impl NonMembershipProofPath {
    #[must_use]
    pub fn key(&self) -> H256 {
        Keccak256::new()
            .chain_update(NON_MEMBERSHIP_PROOF.to_be_bytes())
            .chain_update(U256::from(self.client_id.get()).to_be_bytes())
            .chain_update(U256::from(self.proof_height).to_be_bytes())
            .chain_update(&self.path)
            .finalize()
            .into()
    }
}

impl IbcStorePathKey for NonMembershipProofPath {
    type Spec = IbcUnion;

    type Value = ();
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct BatchTimeoutPath {
    pub batch_hash: H256,
}

impl BatchTimeoutPath {
    #[cfg(feature = "ethabi")]
    #[must_use]
    pub fn from_packet(packet: &Packet) -> Self {
        use alloy_sol_types::SolValue;

        Self {
            batch_hash: {
                Keccak256::new()
                    .chain_update(packet.abi_encode())
                    .finalize()
                    .into()
            },
        }
    }

    #[must_use]
    pub fn key(&self) -> H256 {
        Keccak256::new()
            .chain_update(PACKET_TIMEOUTS.to_be_bytes())
            .chain_update(&self.batch_hash)
            .finalize()
            .into()
    }
}

impl IbcStorePathKey for BatchTimeoutPath {
    type Spec = IbcUnion;

    type Value = H256;
}
