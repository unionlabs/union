use core::num::NonZeroU64;

use enumorph::Enumorph;
use macros::model;
use sha2::Digest;
use sha3::Keccak256;

use crate::{
    hash::H256,
    ibc::core::client::height::Height,
    id::{ChannelId, ClientId, ConnectionId, PortId, CHANNEL_ID_PREFIX, CONNECTION_ID_PREFIX},
};

/// 0x0100000000000000000000000000000000000000000000000000000000000000
pub const COMMITMENT_MAGIC: H256 = {
    let mut bz = [0; 32];
    bz[0] = 1;
    H256::new(bz)
};
pub const COMMITMENT_NULL: H256 = H256::new([0; 32]);

const CLIENT_STATE: u8 = 0x00;
const CONSENSUS_STATE: u8 = 0x01;
const CONNECTIONS: u8 = 0x02;
const CHANNELS: u8 = 0x03;
const PACKETS: u8 = 0x04;
const PACKET_ACKS: u8 = 0x05;
const NEXT_SEQ_SEND: u8 = 0x06;
const NEXT_SEQ_RECV: u8 = 0x07;
const NEXT_SEQ_ACK: u8 = 0x08;

#[model]
#[derive(Hash, Enumorph)]
#[cfg_attr(feature = "valuable", derive(valuable::Valuable))]
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
#[cfg_attr(feature = "valuable", derive(valuable::Valuable))]
pub struct ClientStatePath {
    pub client_id: ClientId,
}

impl ClientStatePath {
    #[must_use]
    pub fn ics24_commitment_path(&self, prefix: impl AsRef<str>) -> String {
        format!(
            "clients/{}-{}/clientState",
            prefix.as_ref(),
            self.client_id.id()
        )
    }

    #[must_use]
    pub fn commitments_key(&self) -> H256 {
        Keccak256::new()
            .chain_update([CLIENT_STATE])
            .chain_update(self.client_id.id().to_be_bytes())
            .finalize()
            .into()
    }
}

/// The raw consensus state bytes as encoded by the light client.
#[model]
#[derive(Hash)]
#[cfg_attr(feature = "valuable", derive(valuable::Valuable))]
pub struct ClientConsensusStatePath {
    pub client_id: ClientId,
    pub height: Height,
}

impl ClientConsensusStatePath {
    #[must_use]
    pub fn ics24_commitment_path(&self, prefix: impl AsRef<str>) -> String {
        format!(
            "clients/{}-{}/consensusStates/{}",
            prefix.as_ref(),
            self.client_id.id(),
            self.height
        )
    }

    #[must_use]
    pub fn commitments_key(&self) -> H256 {
        Keccak256::new()
            .chain_update([CONSENSUS_STATE])
            .chain_update(self.client_id.id().to_be_bytes())
            .chain_update(self.height.height().to_be_bytes())
            .finalize()
            .into()
    }
}

#[model]
#[derive(Hash)]
#[cfg_attr(feature = "valuable", derive(valuable::Valuable))]
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

    #[must_use]
    pub fn commitments_key(&self) -> H256 {
        Keccak256::new()
            .chain_update([CONNECTIONS])
            .chain_update(self.connection_id.id().to_be_bytes())
            .finalize()
            .into()
    }
}

#[model]
#[derive(Hash)]
#[cfg_attr(feature = "valuable", derive(valuable::Valuable))]
pub struct ChannelEndPath {
    pub channel_id: ChannelId,
}

impl ChannelEndPath {
    #[must_use]
    pub fn ics24_commitment_path(&self, port_id: &PortId) -> String {
        format!(
            "channelEnds/ports/{}/channels/{CHANNEL_ID_PREFIX}-{}",
            port_id,
            self.channel_id.id(),
        )
    }

    #[must_use]
    pub fn commitments_key(&self) -> H256 {
        Keccak256::new()
            .chain_update([CHANNELS])
            .chain_update(self.channel_id.id().to_be_bytes())
            .finalize()
            .into()
    }
}

#[model]
#[derive(Hash)]
#[cfg_attr(feature = "valuable", derive(valuable::Valuable))]
pub struct CommitmentPath {
    pub channel_id: ChannelId,
    pub sequence: NonZeroU64,
}

impl CommitmentPath {
    #[must_use]
    pub fn ics24_commitment_path(&self, port_id: &PortId) -> String {
        format!(
            "commitments/ports/{}/channels/{CHANNEL_ID_PREFIX}-{}/sequences/{}",
            port_id,
            self.channel_id.id(),
            self.sequence,
        )
    }

    #[must_use]
    pub fn commitments_key(&self) -> H256 {
        Keccak256::new()
            .chain_update([PACKETS])
            .chain_update(self.channel_id.id().to_be_bytes())
            .chain_update(self.sequence.get().to_be_bytes())
            .finalize()
            .into()
    }
}

/// SHA-256 of the packet acknowledgement.
///
/// If the packet has not yet been acknowledged (either because the packet does not exist or the packet has not been acknowledged yet), then the acknowledgement commitment is unset.
#[model]
#[derive(Hash)]
#[cfg_attr(feature = "valuable", derive(valuable::Valuable))]
pub struct AcknowledgementPath {
    pub channel_id: ChannelId,
    pub sequence: NonZeroU64,
}

impl AcknowledgementPath {
    #[must_use]
    pub fn ics24_commitment_path(&self, port_id: &PortId) -> String {
        format!(
            "acks/ports/{}/channels/{CHANNEL_ID_PREFIX}-{}/sequences/{}",
            port_id,
            self.channel_id.id(),
            self.sequence,
        )
    }

    #[must_use]
    pub fn commitments_key(&self) -> H256 {
        Keccak256::new()
            .chain_update([PACKET_ACKS])
            .chain_update(self.channel_id.id().to_be_bytes())
            .chain_update(self.sequence.get().to_be_bytes())
            .finalize()
            .into()
    }
}

/// This defaults to `false` for packets which have not yet been received.
#[model]
#[derive(Hash)]
#[cfg_attr(feature = "valuable", derive(valuable::Valuable))]
pub struct ReceiptPath {
    pub channel_id: ChannelId,
    pub sequence: NonZeroU64,
}

impl ReceiptPath {
    #[must_use]
    pub fn ics24_commitment_path(&self, port_id: &PortId) -> String {
        format!(
            "receipts/ports/{}/channels/{CHANNEL_ID_PREFIX}-{}/sequences/{}",
            port_id,
            self.channel_id.id(),
            self.sequence,
        )
    }

    #[must_use]
    pub fn commitments_key(&self) -> H256 {
        Keccak256::new()
            .chain_update([PACKETS])
            .chain_update(self.channel_id.id().to_be_bytes())
            .chain_update(self.sequence.get().to_be_bytes())
            .finalize()
            .into()
    }
}

#[model]
#[derive(Hash)]
#[cfg_attr(feature = "valuable", derive(valuable::Valuable))]
pub struct NextSequenceSendPath {
    pub channel_id: ChannelId,
}

impl NextSequenceSendPath {
    #[must_use]
    pub fn ics24_commitment_path(&self, port_id: &PortId) -> String {
        format!(
            "nextSequenceSend/ports/{}/channels/{CHANNEL_ID_PREFIX}-{}",
            port_id,
            self.channel_id.id(),
        )
    }

    #[must_use]
    pub fn commitments_key(&self) -> H256 {
        Keccak256::new()
            .chain_update([NEXT_SEQ_SEND])
            .chain_update(self.channel_id.id().to_be_bytes())
            .finalize()
            .into()
    }
}

#[model]
#[derive(Hash)]
#[cfg_attr(feature = "valuable", derive(valuable::Valuable))]
pub struct NextSequenceRecvPath {
    pub channel_id: ChannelId,
}

impl NextSequenceRecvPath {
    #[must_use]
    pub fn ics24_commitment_path(&self, port_id: &PortId) -> String {
        format!(
            "nextSequenceRecv/ports/{}/channels/{CHANNEL_ID_PREFIX}-{}",
            port_id,
            self.channel_id.id(),
        )
    }

    #[must_use]
    pub fn commitments_key(&self) -> H256 {
        Keccak256::new()
            .chain_update([NEXT_SEQ_RECV])
            .chain_update(self.channel_id.id().to_be_bytes())
            .finalize()
            .into()
    }
}

#[model]
#[derive(Hash)]
#[cfg_attr(feature = "valuable", derive(valuable::Valuable))]
pub struct NextSequenceAckPath {
    pub channel_id: ChannelId,
}

impl NextSequenceAckPath {
    #[must_use]
    pub fn ics24_commitment_path(&self, port_id: &PortId) -> String {
        format!(
            "nextSequenceAck/ports/{}/channels/{CHANNEL_ID_PREFIX}-{}",
            port_id,
            self.channel_id.id(),
        )
    }

    #[must_use]
    pub fn commitments_key(&self) -> H256 {
        Keccak256::new()
            .chain_update([NEXT_SEQ_ACK])
            .chain_update(self.channel_id.id().to_be_bytes())
            .finalize()
            .into()
    }
}

#[model]
#[derive(Hash)]
#[cfg_attr(feature = "valuable", derive(valuable::Valuable))]
pub struct NextConnectionSequencePath {}

impl NextConnectionSequencePath {
    #[must_use]
    pub fn ics24_commitment_path(&self) -> String {
        "nextConnectionSequence".to_owned()
    }
}

#[model]
#[derive(Hash)]
#[cfg_attr(feature = "valuable", derive(valuable::Valuable))]
pub struct NextClientSequencePath {}

impl NextClientSequencePath {
    #[must_use]
    pub fn ics24_commitment_path(&self) -> String {
        "nextClientSequence".to_owned()
    }
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
                channel_id: ChannelId::new(0),
            }
            .ics24_commitment_path(&"port".parse().unwrap()),
            "channelEnds/ports/port/channels/channel-0",
        );
        assert_eq!(
            CommitmentPath {
                channel_id: ChannelId::new(0),
                sequence: 1.try_into().unwrap()
            }
            .ics24_commitment_path(&"port".parse().unwrap()),
            "commitments/ports/port/channels/channel-0/sequences/1",
        );
        assert_eq!(
            AcknowledgementPath {
                channel_id: ChannelId::new(0),
                sequence: 1.try_into().unwrap()
            }
            .ics24_commitment_path(&"port".parse().unwrap()),
            "acks/ports/port/channels/channel-0/sequences/1",
        );
    }
}
