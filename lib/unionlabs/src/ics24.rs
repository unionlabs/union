use core::num::NonZeroU64;

use enumorph::Enumorph;
use macros::model;
use serde_utils::Hex;

use crate::{
    hash::H256,
    ibc::core::{
        channel::channel::Channel, client::height::Height,
        connection::connection_end::ConnectionEnd,
    },
    id::{ChannelId, ClientId, ConnectionId, PortId, CHANNEL_ID_PREFIX, CONNECTION_ID_PREFIX},
};

/// `IbcPath` represents the path to a light client's ibc storage. The values stored at each path
/// are strongly typed, i.e. `connections/{connection_id}` always stores a [`ConnectionEnd`].
pub trait IbcPath: TryFrom<Path, Error = Path> + Into<Path> {
    type Value;
}

#[model]
#[derive(Hash, Enumorph)]
pub enum Path {
    ClientState(ClientStatePath),
    ClientConsensusState(ClientConsensusStatePath),
    Connection(ConnectionPath),
    ChannelEnd(ChannelEndPath),
    Commitment(CommitmentPath),
    Acknowledgement(AcknowledgementPath),
    Receipt(ReceiptPath),
    NextSequenceSend(NextSequenceSendPath),
    NextSequenceRecv(NextSequenceRecvPath),
    NextSequenceAck(NextSequenceAckPath),
    NextConnectionSequence(NextConnectionSequencePath),
    NextClientSequence(NextClientSequencePath),
}

/// The raw client state bytes as encoded by the light client.
#[model]
#[derive(Hash)]
pub struct ClientStatePath {
    pub client_id: ClientId,
}

impl ClientStatePath {
    #[must_use]
    pub fn ics24_commitment_path(&self, client_type: impl AsRef<str>) -> String {
        format!(
            "clients/{}-{}/clientState",
            client_type.as_ref(),
            self.client_id.id()
        )
    }
}

impl IbcPath for ClientStatePath {
    type Value = Hex<Vec<u8>>;
}

/// The raw consensus state bytes as encoded by the light client.
#[model]
#[derive(Hash)]
pub struct ClientConsensusStatePath {
    pub client_id: ClientId,
    pub height: Height,
}

impl ClientConsensusStatePath {
    #[must_use]
    pub fn ics24_commitment_path(&self, client_type: impl AsRef<str>) -> String {
        format!(
            "clients/{}-{}/consensusStates/{}",
            client_type.as_ref(),
            self.client_id.id(),
            self.height
        )
    }
}

impl IbcPath for ClientConsensusStatePath {
    type Value = Hex<Vec<u8>>;
}

#[model]
#[derive(Hash)]
pub struct ConnectionPath {
    pub connection_id: ConnectionId,
}

impl ConnectionPath {
    #[must_use]
    pub fn ics24_commitment_path(&self) -> String {
        format!(
            "connections/{CONNECTION_ID_PREFIX}-{}",
            self.connection_id.id(),
        )
    }
}

impl IbcPath for ConnectionPath {
    type Value = Option<ConnectionEnd>;
}

#[model]
#[derive(Hash)]
pub struct ChannelEndPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
}

impl ChannelEndPath {
    #[must_use]
    pub fn ics24_commitment_path(&self) -> String {
        format!(
            "channelEnds/ports/{}/channels/{CHANNEL_ID_PREFIX}-{}",
            self.port_id,
            self.channel_id.id(),
        )
    }
}

impl IbcPath for ChannelEndPath {
    type Value = Option<Channel>;
}

#[model]
#[derive(Hash)]
pub struct CommitmentPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: NonZeroU64,
}

impl CommitmentPath {
    #[must_use]
    pub fn ics24_commitment_path(&self) -> String {
        format!(
            "commitments/ports/{}/channels/{CHANNEL_ID_PREFIX}-{}/sequences/{}",
            self.port_id,
            self.channel_id.id(),
            self.sequence,
        )
    }
}

impl IbcPath for CommitmentPath {
    type Value = Option<H256>;
}

/// SHA-256 of the packet acknowledgement.
///
/// If the packet has not yet been acknowledged (either because the packet does not exist or the packet has not been acknowledged yet), then the acknowledgement commitment is unset.
#[model]
#[derive(Hash)]
pub struct AcknowledgementPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: NonZeroU64,
}

impl AcknowledgementPath {
    #[must_use]
    pub fn ics24_commitment_path(&self) -> String {
        format!(
            "acks/ports/{}/channels/{CHANNEL_ID_PREFIX}-{}/sequences/{}",
            self.port_id,
            self.channel_id.id(),
            self.sequence,
        )
    }
}

impl IbcPath for AcknowledgementPath {
    type Value = Option<H256>;
}

/// This defaults to `false` for packets which have not yet been received.
#[model]
#[derive(Hash)]
pub struct ReceiptPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: NonZeroU64,
}

impl ReceiptPath {
    #[must_use]
    pub fn ics24_commitment_path(&self) -> String {
        format!(
            "receipts/ports/{}/channels/{CHANNEL_ID_PREFIX}-{}/sequences/{}",
            self.port_id,
            self.channel_id.id(),
            self.sequence,
        )
    }
}

impl IbcPath for ReceiptPath {
    type Value = bool;
}

#[model]
#[derive(Hash)]
pub struct NextSequenceSendPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
}

impl NextSequenceSendPath {
    #[must_use]
    pub fn ics24_commitment_path(&self) -> String {
        format!(
            "nextSequenceSend/ports/{}/channels/{CHANNEL_ID_PREFIX}-{}",
            self.port_id,
            self.channel_id.id(),
        )
    }
}

impl IbcPath for NextSequenceSendPath {
    type Value = u64;
}

#[model]
#[derive(Hash)]
pub struct NextSequenceRecvPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
}

impl NextSequenceRecvPath {
    #[must_use]
    pub fn ics24_commitment_path(&self) -> String {
        format!(
            "nextSequenceRecv/ports/{}/channels/{CHANNEL_ID_PREFIX}-{}",
            self.port_id,
            self.channel_id.id(),
        )
    }
}

impl IbcPath for NextSequenceRecvPath {
    type Value = u64;
}

#[model]
#[derive(Hash)]
pub struct NextSequenceAckPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
}

impl NextSequenceAckPath {
    #[must_use]
    pub fn ics24_commitment_path(&self) -> String {
        format!(
            "nextSequenceAck/ports/{}/channels/{CHANNEL_ID_PREFIX}-{}",
            self.port_id,
            self.channel_id.id(),
        )
    }
}

impl IbcPath for NextSequenceAckPath {
    type Value = u64;
}

#[model]
#[derive(Hash)]
pub struct NextConnectionSequencePath {}

impl NextConnectionSequencePath {
    #[must_use]
    pub fn ics24_commitment_path(&self) -> String {
        "nextConnectionSequence".to_owned()
    }
}

impl IbcPath for NextConnectionSequencePath {
    type Value = u64;
}

#[model]
#[derive(Hash)]
pub struct NextClientSequencePath {}

impl NextClientSequencePath {
    #[must_use]
    pub fn ics24_commitment_path(&self) -> String {
        "nextClientSequence".to_owned()
    }
}

impl IbcPath for NextClientSequencePath {
    type Value = u64;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ibc::core::client::height::Height;

    #[test]
    fn parse_ibc_paths_from_str() {
        assert_eq!(
            ClientStatePath {
                client_id: ClientId::new(0)
            }
            .ics24_commitment_path("08-wasm"),
            "clients/08-wasm-0/clientState".to_string(),
        );
        assert_eq!(
            ClientConsensusStatePath {
                client_id: ClientId::new(0),
                height: Height::new(1)
            }
            .ics24_commitment_path("08-wasm"),
            "clients/08-wasm-0/consensusStates/0-1",
        );
        assert_eq!(
            ClientConsensusStatePath {
                client_id: ClientId::new(0),
                height: Height::new_with_revision(1, 1)
            }
            .ics24_commitment_path("08-wasm"),
            "clients/08-wasm-0/consensusStates/1-1",
        );
        assert_eq!(
            ConnectionPath {
                connection_id: ConnectionId::new(0)
            }
            .ics24_commitment_path(),
            "connections/connection-0",
        );
        assert_eq!(
            ChannelEndPath {
                port_id: "port".parse().unwrap(),
                channel_id: ChannelId::new(0),
            }
            .ics24_commitment_path(),
            "channelEnds/ports/port/channels/channel-0",
        );
        assert_eq!(
            CommitmentPath {
                port_id: "port".parse().unwrap(),
                channel_id: ChannelId::new(0),
                sequence: 1.try_into().unwrap()
            }
            .ics24_commitment_path(),
            "commitments/ports/port/channels/channel-0/sequences/1",
        );
        assert_eq!(
            AcknowledgementPath {
                port_id: "port".parse().unwrap(),
                channel_id: ChannelId::new(0),
                sequence: 1.try_into().unwrap()
            }
            .ics24_commitment_path(),
            "acks/ports/port/channels/channel-0/sequences/1",
        );
    }
}
