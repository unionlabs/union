use core::{fmt, fmt::Display, num::NonZeroU64, str::FromStr};

use macros::{ibc_path, model};
use serde::{Deserialize, Serialize};

use crate::{
    bytes::Bytes,
    hash::H256,
    ibc::core::{
        channel::channel::Channel, client::height::Height,
        connection::connection_end::ConnectionEnd,
    },
    id::{ChannelId, ClientId, ConnectionId, PortId},
    traits::Member,
    ErrorReporter,
};

pub mod ethabi {
    use enumorph::Enumorph;
    use serde::{Deserialize, Serialize};
    use sha2::Digest;
    use sha3::Keccak256;

    use crate::{hash::H256, uint::U256};

    /// 0x0100000000000000000000000000000000000000000000000000000000000000
    pub const COMMITMENT_MAGIC: H256 = {
        let mut bz = [0; 32];
        bz[0] = 1;
        H256::new(bz)
    };
    pub const COMMITMENT_NULL: H256 = H256::new([0; 32]);

    const CLIENT_STATE: U256 = U256::from_limbs([0, 0, 0, 0]);
    const CONSENSUS_STATE: U256 = U256::from_limbs([0, 0, 0, 1]);
    const CONNECTIONS: U256 = U256::from_limbs([0, 0, 0, 2]);
    const CHANNELS: U256 = U256::from_limbs([0, 0, 0, 3]);
    const PACKETS: U256 = U256::from_limbs([0, 0, 0, 4]);
    const PACKET_ACKS: U256 = U256::from_limbs([0, 0, 0, 5]);

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Enumorph)]
    pub enum Path {
        ClientState(ClientStatePath),
        ConsensusState(ConsensusStatePath),
        Connection(ConnectionPath),
        Channel(ChannelPath),
        BatchReceipts(BatchReceiptsPath),
        BatchPackets(BatchPacketsPath),
    }

    impl Path {
        #[must_use]
        pub fn key(&self) -> H256 {
            match self {
                Path::ClientState(path) => path.key(),
                Path::ConsensusState(path) => path.key(),
                Path::Connection(path) => path.key(),
                Path::Channel(path) => path.key(),
                Path::BatchReceipts(path) => path.key(),
                Path::BatchPackets(path) => path.key(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    pub struct ClientStatePath {
        pub client_id: u32,
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

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    pub struct ConsensusStatePath {
        pub client_id: u32,
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

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    pub struct ConnectionPath {
        pub connection_id: u32,
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

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    pub struct ChannelPath {
        pub channel_id: u32,
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

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    pub struct BatchReceiptsPath {
        pub channel_id: u32,
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

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    pub struct BatchPacketsPath {
        pub channel_id: u32,
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

    #[must_use]
    pub fn client_state_key(client_id: u32) -> H256 {
        ClientStatePath { client_id }.key()
    }

    #[must_use]
    pub fn consensus_state_key(client_id: u32, height: u64) -> H256 {
        ConsensusStatePath { client_id, height }.key()
    }

    #[must_use]
    pub fn connection_key(connection_id: u32) -> H256 {
        ConnectionPath { connection_id }.key()
    }

    #[must_use]
    pub fn channel_key(channel_id: u32) -> H256 {
        ChannelPath { channel_id }.key()
    }

    #[must_use]
    pub fn batch_receipts_key(channel_id: u32, batch_hash: H256) -> H256 {
        BatchReceiptsPath {
            channel_id,
            batch_hash,
        }
        .key()
    }

    #[must_use]
    pub fn batch_packets_key(channel_id: u32, batch_hash: H256) -> H256 {
        BatchPacketsPath {
            channel_id,
            batch_hash,
        }
        .key()
    }
}

/// `IbcPath` represents the path to a light client's ibc storage. The values stored at each path
/// are strongly typed, i.e. `connections/{connection_id}` always stores a [`ConnectionEnd`].
#[allow(private_bounds)]
pub trait IbcPath: Sealed + Member + Display + TryFrom<Path, Error = Path> + Into<Path> {
    type Value: Member;
}

trait Sealed {}

#[model]
#[derive(Hash, derive_more::Display, enumorph::Enumorph)]
pub enum Path {
    #[display(fmt = "{_0}")]
    ClientState(ClientStatePath),
    #[display(fmt = "{_0}")]
    ClientConsensusState(ClientConsensusStatePath),
    #[display(fmt = "{_0}")]
    Connection(ConnectionPath),
    #[display(fmt = "{_0}")]
    ChannelEnd(ChannelEndPath),
    #[display(fmt = "{_0}")]
    Commitment(CommitmentPath),
    #[display(fmt = "{_0}")]
    Acknowledgement(AcknowledgementPath),
    #[display(fmt = "{_0}")]
    Receipt(ReceiptPath),
    #[display(fmt = "{_0}")]
    NextSequenceSend(NextSequenceSendPath),
    #[display(fmt = "{_0}")]
    NextSequenceRecv(NextSequenceRecvPath),
    #[display(fmt = "{_0}")]
    NextSequenceAck(NextSequenceAckPath),
    #[display(fmt = "{_0}")]
    NextConnectionSequence(NextConnectionSequencePath),
    #[display(fmt = "{_0}")]
    NextClientSequence(NextClientSequencePath),
}

impl Path {
    #[cfg(feature = "ethabi")]
    #[must_use]
    pub fn into_eth_commitment(self) -> H256 {
        match self {
            Path::ClientState(path) => ethabi::ClientStatePath {
                client_id: path.client_id.id(),
            }
            .key(),
            Path::ClientConsensusState(path) => ethabi::ConsensusStatePath {
                client_id: path.client_id.id(),
                height: path.height.height(),
            }
            .key(),
            Path::Connection(path) => ethabi::ConnectionPath {
                connection_id: path.connection_id.id(),
            }
            .key(),
            Path::ChannelEnd(path) => ethabi::ChannelPath {
                channel_id: path.channel_id.id(),
            }
            .key(),
            Path::Commitment(_path) => {
                // ethabi::commitments(path.channel_id.id(), path.sequence.into())
                todo!()
            }
            Path::Acknowledgement(_path) => {
                // ethabi::acknowledgements(path.channel_id.id(), path.sequence.into())
                todo!()
            }
            Path::Receipt(_path) => {
                todo!()
            }
            Path::NextSequenceSend(_path) => todo!(),
            Path::NextSequenceRecv(_path) => todo!(),
            Path::NextSequenceAck(_path) => todo!(),
            // TODO(aeryz): check these out
            Path::NextConnectionSequence(_path) => H256::default(),
            Path::NextClientSequence(_path) => H256::default(),
        }
    }
}

impl FromStr for Path {
    type Err = PathParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse()
            .map(Self::ClientState)
            .or_else(|_| s.parse().map(Self::ClientConsensusState))
            .or_else(|_| s.parse().map(Self::Connection))
            .or_else(|_| s.parse().map(Self::ChannelEnd))
            .or_else(|_| s.parse().map(Self::Commitment))
            .or_else(|_| s.parse().map(Self::Acknowledgement))
            .or_else(|_| s.parse().map(Self::Receipt))
            .or_else(|_| s.parse().map(Self::NextSequenceSend))
            .or_else(|_| s.parse().map(Self::NextSequenceRecv))
            .or_else(|_| s.parse().map(Self::NextSequenceAck))
            .or_else(|_| s.parse().map(Self::NextConnectionSequence))
    }
}

/// The raw client state bytes as encoded by the light client.
#[ibc_path("clients/{client_id}/clientState", Bytes)]
pub struct ClientStatePath {
    pub client_id: ClientId,
}

/// The raw consensus state bytes as encoded by the light client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, ::clap::Args)]
#[serde(deny_unknown_fields)]
pub struct ClientConsensusStatePath {
    pub client_id: ClientId,
    pub height: Height,
}

impl fmt::Display for ClientConsensusStatePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "clients/{}/consensusStates/{}-{}",
            self.client_id,
            self.height.revision(),
            self.height.height()
        )
    }
}

impl FromStr for ClientConsensusStatePath {
    type Err = PathParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        dbg!(s);

        let mut it = s.split('/');

        match it.next() {
            Some(s) => {
                if s != "clients" {
                    return Err(PathParseError::InvalidStaticSegment {
                        expected: "clients",
                        found: s.to_owned(),
                    });
                }
            }
            None => return Err(PathParseError::MissingStaticSegment("clients")),
        }

        let client_id = it
            .next()
            .ok_or(PathParseError::MissingSegment)?
            .parse()
            .map_err(|e| PathParseError::Parse(ErrorReporter(e).to_string()))?;

        match it.next() {
            Some(s) => {
                if s != "consensusStates" {
                    return Err(PathParseError::InvalidStaticSegment {
                        expected: "consensusStates",
                        found: s.to_owned(),
                    });
                }
            }
            None => return Err(PathParseError::MissingStaticSegment("consensusStates")),
        }

        let height =
            Height::from_str_allow_zero_revision(it.next().ok_or(PathParseError::MissingSegment)?)
                .map_err(|e| PathParseError::Parse(ErrorReporter(e).to_string()))?;

        if it.next().is_some() {
            return Err(PathParseError::TooManySegments);
        }

        Ok(Self { client_id, height })
    }
}

impl IbcPath for ClientConsensusStatePath {
    type Value = Bytes;
}

impl Sealed for ClientConsensusStatePath {}

// REVIEW: Make this an `Option`?
#[ibc_path("connections/{connection_id:#}", Option<ConnectionEnd>)]
pub struct ConnectionPath {
    #[ibc_path(ConnectionId::from_str_prefixed)]
    pub connection_id: ConnectionId,
}

// REVIEW: Make this an `Option`?
#[ibc_path(
    "channelEnds/ports/{port_id}/channels/{channel_id:#}",
    Option<Channel>
)]
pub struct ChannelEndPath {
    pub port_id: PortId,
    #[ibc_path(ChannelId::from_str_prefixed)]
    pub channel_id: ChannelId,
}

#[ibc_path(
    "commitments/ports/{port_id}/channels/{channel_id:#}/sequences/{sequence}",
    Option<H256>
)]
pub struct CommitmentPath {
    pub port_id: PortId,
    #[ibc_path(ChannelId::from_str_prefixed)]
    pub channel_id: ChannelId,
    pub sequence: NonZeroU64,
}

/// SHA-256 of the packet acknowledgement.
///
/// If the packet has not yet been acknowledged (either because the packet does not exist or the packet has not been acknowledged yet), then the acknowledgement commitment is unset.
#[ibc_path("acks/ports/{port_id}/channels/{channel_id:#}/sequences/{sequence}", Option<H256>)]
pub struct AcknowledgementPath {
    pub port_id: PortId,
    #[ibc_path(ChannelId::from_str_prefixed)]
    pub channel_id: ChannelId,
    pub sequence: NonZeroU64,
}

/// This defaults to `false` for packets which have not yet been received.
#[ibc_path(
    "receipts/ports/{port_id}/channels/{channel_id:#}/sequences/{sequence}",
    bool
)]
pub struct ReceiptPath {
    pub port_id: PortId,
    #[ibc_path(ChannelId::from_str_prefixed)]
    pub channel_id: ChannelId,
    pub sequence: NonZeroU64,
}

#[ibc_path("nextSequenceSend/ports/{port_id}/channels/{channel_id:#}", u64)]
pub struct NextSequenceSendPath {
    pub port_id: PortId,
    #[ibc_path(ChannelId::from_str_prefixed)]
    pub channel_id: ChannelId,
}

#[ibc_path("nextSequenceRecv/ports/{port_id}/channels/{channel_id:#}", u64)]
pub struct NextSequenceRecvPath {
    pub port_id: PortId,
    #[ibc_path(ChannelId::from_str_prefixed)]
    pub channel_id: ChannelId,
}

#[ibc_path("nextSequenceAck/ports/{port_id}/channels/{channel_id:#}", u64)]
pub struct NextSequenceAckPath {
    pub port_id: PortId,
    #[ibc_path(ChannelId::from_str_prefixed)]
    pub channel_id: ChannelId,
}

#[ibc_path("nextConnectionSequence", u64)]
pub struct NextConnectionSequencePath {}

#[ibc_path("nextClientSequence", u64)]
pub struct NextClientSequencePath {}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PathParseError {
    #[error("invalid static segment, expected `{expected}` but found `{found}`")]
    InvalidStaticSegment {
        expected: &'static str,
        found: String,
    },
    #[error("missing static segment `{0}`")]
    MissingStaticSegment(&'static str),
    // TODO: Figure out a way to provide more context here?
    #[error("missing segment")]
    MissingSegment,
    #[error("too many segments")]
    TooManySegments,
    // contains the stringified parse error
    #[error("error parsing segment: {0}")]
    Parse(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ibc::core::client::height::Height;

    #[test]
    fn parse_ibc_paths_from_str() {
        assert_eq!(
            "clients/08-wasm-0/clientState".parse::<Path>().unwrap(),
            Path::ClientState(ClientStatePath {
                client_id: ClientId::new("08-wasm", 0)
            })
        );
        assert_eq!(
            "clients/08-wasm-0/consensusStates/0-1"
                .parse::<Path>()
                .unwrap(),
            Path::ClientConsensusState(ClientConsensusStatePath {
                client_id: ClientId::new("08-wasm", 0),
                height: Height::new(1)
            })
        );
        assert_eq!(
            "connections/connection-0".parse::<Path>().unwrap(),
            Path::Connection(ConnectionPath {
                connection_id: ConnectionId::new(0)
            })
        );
        assert_eq!(
            "channelEnds/ports/port/channels/channel-0"
                .parse::<Path>()
                .unwrap(),
            Path::ChannelEnd(ChannelEndPath {
                port_id: PortId::new("port").unwrap(),
                channel_id: ChannelId::new(0)
            })
        );
        assert_eq!(
            "commitments/ports/port/channels/channel-0/sequences/1"
                .parse::<Path>()
                .unwrap(),
            Path::Commitment(CommitmentPath {
                port_id: PortId::new("port").unwrap(),
                channel_id: ChannelId::new(0),
                sequence: 1.try_into().unwrap()
            })
        );
        assert_eq!(
            "acks/ports/port/channels/channel-0/sequences/1"
                .parse::<Path>()
                .unwrap(),
            Path::Acknowledgement(AcknowledgementPath {
                port_id: PortId::new("port").unwrap(),
                channel_id: ChannelId::new(0),
                sequence: 1.try_into().unwrap()
            })
        );
    }
}
