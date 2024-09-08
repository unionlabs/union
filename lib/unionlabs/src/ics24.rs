use core::{fmt::Display, num::NonZeroU64, str::FromStr};

use macros::{ibc_path, model};
use serde::{Deserialize, Serialize};
use serde_utils::Hex;

use crate::{
    hash::H256,
    ibc::core::{
        channel::channel::Channel, client::height::Height,
        connection::connection_end::ConnectionEnd,
    },
    id::{ChannelId, ClientId, ConnectionId, PortId},
    traits::Member,
};

/// `IbcPath` represents the path to a light client's ibc storage. The values stored at each path
/// are strongly typed, i.e. `connections/{connection_id}` always stores a [`ConnectionEnd`].
pub trait IbcPath: Member + Display + TryFrom<Path, Error = Path> + Into<Path> {
    type Value: Member;
}

#[model]
#[derive(Hash, derive_more::Display, clap::Subcommand, enumorph::Enumorph)]
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
#[ibc_path("clients/{client_id}/clientState", Hex<Vec<u8>>)]
pub struct ClientStatePath {
    pub client_id: ClientId,
}

/// The raw consensus state bytes as encoded by the light client.
#[ibc_path("clients/{client_id}/consensusStates/{height}", Hex<Vec<u8>>)]
pub struct ClientConsensusStatePath {
    pub client_id: ClientId,
    pub height: Height,
}

// REVIEW: Make this an `Option`?
#[ibc_path("connections/{connection_id}", Option<ConnectionEnd>)]
pub struct ConnectionPath {
    pub connection_id: ConnectionId,
}

// REVIEW: Make this an `Option`?
#[ibc_path(
    "channelEnds/ports/{port_id}/channels/{channel_id}",
    Option<Channel>
)]
pub struct ChannelEndPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
}

#[ibc_path(
    "commitments/ports/{port_id}/channels/{channel_id}/sequences/{sequence}",
    Option<H256>
)]
pub struct CommitmentPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: NonZeroU64,
}

/// SHA-256 of the packet acknowledgement.
/// If the packet has not yet been acknowledged (either because the packet does not exist or the packet has not been acknowledged yet), then the acknowledgement commitment is unset.
#[ibc_path("acks/ports/{port_id}/channels/{channel_id}/sequences/{sequence}", Option<H256>)]
pub struct AcknowledgementPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: NonZeroU64,
}

/// This defaults to `false` for packets which have not yet been received.
#[ibc_path(
    "receipts/ports/{port_id}/channels/{channel_id}/sequences/{sequence}",
    bool
)]
pub struct ReceiptPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: NonZeroU64,
}

#[ibc_path("nextSequenceSend/ports/{port_id}/channels/{channel_id}", u64)]
pub struct NextSequenceSendPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
}

#[ibc_path("nextSequenceRecv/ports/{port_id}/channels/{channel_id}", u64)]
pub struct NextSequenceRecvPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
}

#[ibc_path("nextSequenceAck/ports/{port_id}/channels/{channel_id}", u64)]
pub struct NextSequenceAckPath {
    pub port_id: PortId,
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
    use crate::{ibc::core::client::height::Height, validated::ValidateT};

    #[test]
    fn parse_ibc_paths_from_str() {
        assert_eq!(
            "clients/08-wasm-0/clientState".parse::<Path>().unwrap(),
            Path::ClientState(ClientStatePath {
                client_id: "08-wasm-0".to_string().validate().unwrap()
            })
        );
        assert_eq!(
            "clients/08-wasm-0/consensusStates/0-1"
                .parse::<Path>()
                .unwrap(),
            Path::ClientConsensusState(ClientConsensusStatePath {
                client_id: "08-wasm-0".to_string().validate().unwrap(),
                height: Height {
                    revision_number: 0,
                    revision_height: 1
                }
            })
        );
        assert_eq!(
            "connections/connection-0".parse::<Path>().unwrap(),
            Path::Connection(ConnectionPath {
                connection_id: "connection-0".to_string().validate().unwrap()
            })
        );
        assert_eq!(
            "channelEnds/ports/port/channels/channel-0"
                .parse::<Path>()
                .unwrap(),
            Path::ChannelEnd(ChannelEndPath {
                port_id: "port".to_string().validate().unwrap(),
                channel_id: "channel-0".to_string().validate().unwrap()
            })
        );
        assert_eq!(
            "commitments/ports/port/channels/channel-0/sequences/1"
                .parse::<Path>()
                .unwrap(),
            Path::Commitment(CommitmentPath {
                port_id: "port".to_string().validate().unwrap(),
                channel_id: "channel-0".to_string().validate().unwrap(),
                sequence: 1.try_into().unwrap()
            })
        );
        assert_eq!(
            "acks/ports/port/channels/channel-0/sequences/1"
                .parse::<Path>()
                .unwrap(),
            Path::Acknowledgement(AcknowledgementPath {
                port_id: "port".to_string().validate().unwrap(),
                channel_id: "channel-0".to_string().validate().unwrap(),
                sequence: 1.try_into().unwrap()
            })
        );
    }
}
