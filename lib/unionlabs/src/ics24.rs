use core::{fmt::Display, num::NonZeroU64, str::FromStr};

use macros::ibc_path;
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

#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    derive_more::Display,
    clap::Subcommand,
    enumorph::Enumorph,
)]
#[serde(
    bound(
        serialize = "ClientId: Serialize",
        deserialize = "ClientId: for<'d> Deserialize<'d>",
    ),
    tag = "@type",
    content = "@value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
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

#[ibc_path("clients/{client_id}/clientState")]
pub struct ClientStatePath {
    pub client_id: ClientId,
}

impl IbcPath for ClientStatePath {
    type Value = Hex<Vec<u8>>;
}

#[ibc_path("clients/{client_id}/consensusStates/{height}")]
pub struct ClientConsensusStatePath {
    pub client_id: ClientId,
    pub height: Height,
}

impl IbcPath for ClientConsensusStatePath {
    type Value = Hex<Vec<u8>>;
}

#[ibc_path("connections/{connection_id}")]
pub struct ConnectionPath {
    pub connection_id: ConnectionId,
}

impl IbcPath for ConnectionPath {
    // TODO: This is really `Either<ClientId, EmptyString>`
    type Value = ConnectionEnd;
}

#[ibc_path("channelEnds/ports/{port_id}/channels/{channel_id}")]
pub struct ChannelEndPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
}

impl IbcPath for ChannelEndPath {
    type Value = Channel;
}

#[ibc_path("commitments/ports/{port_id}/channels/{channel_id}/sequences/{sequence}")]
pub struct CommitmentPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: NonZeroU64,
}

impl IbcPath for CommitmentPath {
    type Value = H256;
}

#[ibc_path("acks/ports/{port_id}/channels/{channel_id}/sequences/{sequence}")]
pub struct AcknowledgementPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: NonZeroU64,
}

impl IbcPath for AcknowledgementPath {
    type Value = H256;
}

#[ibc_path("receipts/ports/{port_id}/channels/{channel_id}/sequences/{sequence}")]
pub struct ReceiptPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: NonZeroU64,
}

impl IbcPath for ReceiptPath {
    type Value = bool;
}

#[ibc_path("nextSequenceSend/ports/{port_id}/channels/{channel_id}")]
pub struct NextSequenceSendPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
}

impl IbcPath for NextSequenceSendPath {
    type Value = u64;
}

#[ibc_path("nextSequenceRecv/ports/{port_id}/channels/{channel_id}")]
pub struct NextSequenceRecvPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
}

impl IbcPath for NextSequenceRecvPath {
    type Value = u64;
}

#[ibc_path("nextSequenceAck/ports/{port_id}/channels/{channel_id}")]
pub struct NextSequenceAckPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
}

impl IbcPath for NextSequenceAckPath {
    type Value = u64;
}

#[ibc_path("nextConnectionSequence")]
pub struct NextConnectionSequencePath {}

impl IbcPath for NextConnectionSequencePath {
    type Value = u64;
}

#[ibc_path("nextClientSequence")]
pub struct NextClientSequencePath {}

impl IbcPath for NextClientSequencePath {
    type Value = u64;
}

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
        type PathT = Path;
        assert_eq!(
            "clients/08-wasm-0/clientState".parse::<PathT>().unwrap(),
            Path::ClientState(ClientStatePath {
                client_id: "08-wasm-0".to_string().validate().unwrap()
            })
        );
        assert_eq!(
            "clients/08-wasm-0/consensusStates/0-1"
                .parse::<PathT>()
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
            "connections/connection-0".parse::<PathT>().unwrap(),
            Path::Connection(ConnectionPath {
                connection_id: "connection-0".to_string().validate().unwrap()
            })
        );
        assert_eq!(
            "channelEnds/ports/port/channels/channel-0"
                .parse::<PathT>()
                .unwrap(),
            Path::ChannelEnd(ChannelEndPath {
                port_id: "port".to_string().validate().unwrap(),
                channel_id: "channel-0".to_string().validate().unwrap()
            })
        );
        assert_eq!(
            "commitments/ports/port/channels/channel-0/sequences/1"
                .parse::<PathT>()
                .unwrap(),
            Path::Commitment(CommitmentPath {
                port_id: "port".to_string().validate().unwrap(),
                channel_id: "channel-0".to_string().validate().unwrap(),
                sequence: 1.try_into().unwrap()
            })
        );
        assert_eq!(
            "acks/ports/port/channels/channel-0/sequences/1"
                .parse::<PathT>()
                .unwrap(),
            Path::Acknowledgement(AcknowledgementPath {
                port_id: "port".to_string().validate().unwrap(),
                channel_id: "channel-0".to_string().validate().unwrap(),
                sequence: 1.try_into().unwrap()
            })
        );
    }
}
