use unionlabs::{ibc::core::client::height::Height, primitives::Bytes};
use voyager_primitives::{IbcSpec, IbcSpecId};

pub mod datagram;
pub mod event;
pub mod path;
pub mod query;

pub(crate) mod types;

pub use voyager_primitives::Timestamp;

pub use crate::types::{
    channel::{Channel, ChannelState},
    connection::{Connection, ConnectionState},
    packet::Packet,
    ChannelId, ClientId, ConnectionId,
};
use crate::{
    datagram::{Datagram, MsgUpdateClient},
    event::FullEvent,
    path::{ClientStatePath, ConsensusStatePath, StorePath},
    query::Query,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IbcUnion {}

impl IbcSpec for IbcUnion {
    const ID: IbcSpecId = IbcSpecId::new_static(IbcSpecId::UNION);

    type ClientId = ClientId;

    type StorePath = StorePath;

    type Query = Query;

    type Datagram = Datagram;

    type Event = FullEvent;

    fn update_client_datagram(client_id: Self::ClientId, client_message: Bytes) -> Self::Datagram {
        MsgUpdateClient {
            client_id,
            client_message,
        }
        .into()
    }

    fn client_state_path(client_id: Self::ClientId) -> Self::StorePath {
        ClientStatePath { client_id }.into()
    }

    fn consensus_state_path(client_id: Self::ClientId, height: Height) -> Self::StorePath {
        ConsensusStatePath {
            client_id,
            height: height.height(),
        }
        .into()
    }
}

#[cfg(feature = "tracing")]
pub fn log_event(e: &FullEvent, chain_id: &voyager_primitives::ChainId) {
    use tracing::info;

    let event = e.name();

    match e {
        FullEvent::CreateClient(e) => info!(
            event,
            %chain_id,
            data.client_id = %e.client_id,
            data.client_type = %e.client_type,
            "event"
        ),
        FullEvent::UpdateClient(e) => info!(
            event,
            %chain_id,
            data.client_id = %e.client_id,
            data.client_type = %e.client_type,
            data.height = e.height,
            "event"
        ),
        FullEvent::ConnectionOpenInit(e) => info!(
            event,
            %chain_id,
            data.connection_id = %e.connection_id,
            data.client_id = %e.client_id,
            data.counterparty_client_id = %e.counterparty_client_id,
            "event"
        ),
        FullEvent::ConnectionOpenTry(e) => info!(
            event,
            %chain_id,
            data.connection_id = %e.connection_id,
            data.client_id = %e.client_id,
            data.counterparty_client_id = %e.counterparty_client_id,
            data.counterparty_connection_id = %e.counterparty_connection_id,
            "event"
        ),
        FullEvent::ConnectionOpenAck(e) => info!(
            event,
            %chain_id,
            data.connection_id = %e.connection_id,
            data.client_id = %e.client_id,
            data.counterparty_client_id = %e.counterparty_client_id,
            data.counterparty_connection_id = %e.counterparty_connection_id,
            "event"
        ),
        FullEvent::ConnectionOpenConfirm(e) => info!(
            event,
            %chain_id,
            data.connection_id = %e.connection_id,
            data.client_id = %e.client_id,
            data.counterparty_client_id = %e.counterparty_client_id,
            data.counterparty_connection_id = %e.counterparty_connection_id,
            "event"
        ),
        FullEvent::ChannelOpenInit(e) => info!(
            event,
            %chain_id,
            data.port_id = %e.port_id,
            data.channel_id = %e.channel_id,
            data.counterparty_port_id = %e.counterparty_port_id,
            data.connection.state = ?e.connection.state,
            data.connection.client_id = %e.connection.client_id,
            data.connection.counterparty_client_id = %e.connection.counterparty_client_id,
            data.connection.counterparty_connection_id = e.connection.counterparty_connection_id.map(|id|id.get()),
            data.version = %e.version,
            "event"
        ),
        FullEvent::ChannelOpenTry(e) => info!(
            event,
            %chain_id,
            data.port_id = %e.port_id,
            data.channel_id = %e.channel_id,
            data.counterparty_port_id = %e.counterparty_port_id,
            data.counterparty_channel_id = %e.counterparty_channel_id,
            data.connection.state = ?e.connection.state,
            data.connection.client_id = %e.connection.client_id,
            data.connection.counterparty_client_id = %e.connection.counterparty_client_id,
            data.connection.counterparty_connection_id = e.connection.counterparty_connection_id.map(|id|id.get()),
            data.version = %e.version,
            "event"
        ),
        FullEvent::ChannelOpenAck(e) => info!(
            event,
            %chain_id,
            data.port_id = %e.port_id,
            data.channel_id = %e.channel_id,
            data.counterparty_port_id = %e.counterparty_port_id,
            data.counterparty_channel_id = %e.counterparty_channel_id,
            data.connection.state = ?e.connection.state,
            data.connection.client_id = %e.connection.client_id,
            data.connection.counterparty_client_id = %e.connection.counterparty_client_id,
            data.connection.counterparty_connection_id = e.connection.counterparty_connection_id.map(|id|id.get()),
            data.version = %e.version,
            "event"
        ),
        FullEvent::ChannelOpenConfirm(e) => info!(
            event,
            %chain_id,
            data.port_id = %e.port_id,
            data.channel_id = %e.channel_id,
            data.counterparty_port_id = %e.counterparty_port_id,
            data.counterparty_channel_id = %e.counterparty_channel_id,
            data.connection.state = ?e.connection.state,
            data.connection.client_id = %e.connection.client_id,
            data.connection.counterparty_client_id = %e.connection.counterparty_client_id,
            data.connection.counterparty_connection_id = e.connection.counterparty_connection_id.map(|id|id.get()),
            data.version = %e.version,
            "event"
        ),
        FullEvent::ChannelCloseInit(_e) => info!(event, "event"),
        FullEvent::ChannelCloseConfirm(_e) => info!(event, "event"),
        FullEvent::PacketSend(e) => info!(
            event,
            %chain_id,

            data.packet_hash = %e.packet().hash(),

            data.packet_data = %e.packet_data,

            data.packet.source_channel.channel_id = %e.packet.source_channel.channel_id,
            data.packet.source_channel.version = %e.packet.source_channel.version,
            data.packet.source_channel.connection.client_id = %e.packet.source_channel.connection.client_id,
            data.packet.source_channel.connection.connection_id = %e.packet.source_channel.connection.connection_id,

            data.packet.destination_channel.channel_id = %e.packet.destination_channel.channel_id,
            data.packet.destination_channel.version = %e.packet.destination_channel.version,
            data.packet.destination_channel.connection.client_id = %e.packet.destination_channel.connection.client_id,
            data.packet.destination_channel.connection.connection_id = %e.packet.destination_channel.connection.connection_id,

            data.packet.timeout_height = %e.packet.timeout_height,
            data.packet.timeout_timestamp = %e.packet.timeout_timestamp,
            "event"
        ),
        FullEvent::PacketRecv(e) => info!(
            event,
            %chain_id,

            data.packet_hash = %e.packet().hash(),

            data.packet_data = %e.packet_data,
            data.maker_msg = %e.maker_msg,

            data.packet.source_channel.channel_id = %e.packet.source_channel.channel_id,
            data.packet.source_channel.version = %e.packet.source_channel.version,
            data.packet.source_channel.connection.client_id = %e.packet.source_channel.connection.client_id,
            data.packet.source_channel.connection.connection_id = %e.packet.source_channel.connection.connection_id,

            data.packet.destination_channel.channel_id = %e.packet.destination_channel.channel_id,
            data.packet.destination_channel.version = %e.packet.destination_channel.version,
            data.packet.destination_channel.connection.client_id = %e.packet.destination_channel.connection.client_id,
            data.packet.destination_channel.connection.connection_id = %e.packet.destination_channel.connection.connection_id,

            data.packet.timeout_height = %e.packet.timeout_height,
            data.packet.timeout_timestamp = %e.packet.timeout_timestamp,
            "event"
        ),
        FullEvent::IntentPacketRecv(e) => info!(
            event,
            %chain_id,

            data.packet_hash = %e.packet().hash(),

            data.packet_data = %e.packet_data,
            data.market_maker_msg = %e.market_maker_msg,

            data.packet.source_channel.channel_id = %e.packet.source_channel.channel_id,
            data.packet.source_channel.version = %e.packet.source_channel.version,
            data.packet.source_channel.connection.client_id = %e.packet.source_channel.connection.client_id,
            data.packet.source_channel.connection.connection_id = %e.packet.source_channel.connection.connection_id,

            data.packet.destination_channel.channel_id = %e.packet.destination_channel.channel_id,
            data.packet.destination_channel.version = %e.packet.destination_channel.version,
            data.packet.destination_channel.connection.client_id = %e.packet.destination_channel.connection.client_id,
            data.packet.destination_channel.connection.connection_id = %e.packet.destination_channel.connection.connection_id,

            data.packet.timeout_height = %e.packet.timeout_height,
            data.packet.timeout_timestamp = %e.packet.timeout_timestamp,
            "event"
        ),
        FullEvent::WriteAck(e) => info!(
            event,
            %chain_id,

            data.packet_hash = %e.packet().hash(),

            data.packet_data = %e.packet_data,
            data.acknowledgement = %e.acknowledgement,

            data.packet.source_channel.channel_id = %e.packet.source_channel.channel_id,
            data.packet.source_channel.version = %e.packet.source_channel.version,
            data.packet.source_channel.connection.client_id = %e.packet.source_channel.connection.client_id,
            data.packet.source_channel.connection.connection_id = %e.packet.source_channel.connection.connection_id,

            data.packet.destination_channel.channel_id = %e.packet.destination_channel.channel_id,
            data.packet.destination_channel.version = %e.packet.destination_channel.version,
            data.packet.destination_channel.connection.client_id = %e.packet.destination_channel.connection.client_id,
            data.packet.destination_channel.connection.connection_id = %e.packet.destination_channel.connection.connection_id,

            data.packet.timeout_height = %e.packet.timeout_height,
            data.packet.timeout_timestamp = %e.packet.timeout_timestamp,
            "event"
        ),
        FullEvent::PacketAck(e) => info!(
            event,
            %chain_id,

            data.packet_hash = %e.packet().hash(),

            data.packet_data = %e.packet_data,
            data.acknowledgement = %e.acknowledgement,

            data.packet.source_channel.channel_id = %e.packet.source_channel.channel_id,
            data.packet.source_channel.version = %e.packet.source_channel.version,
            data.packet.source_channel.connection.client_id = %e.packet.source_channel.connection.client_id,
            data.packet.source_channel.connection.connection_id = %e.packet.source_channel.connection.connection_id,

            data.packet.destination_channel.channel_id = %e.packet.destination_channel.channel_id,
            data.packet.destination_channel.version = %e.packet.destination_channel.version,
            data.packet.destination_channel.connection.client_id = %e.packet.destination_channel.connection.client_id,
            data.packet.destination_channel.connection.connection_id = %e.packet.destination_channel.connection.connection_id,

            data.packet.timeout_height = %e.packet.timeout_height,
            data.packet.timeout_timestamp = %e.packet.timeout_timestamp,
            "event"
        ),
        FullEvent::PacketTimeout(e) => info!(
            event,
            %chain_id,

            data.packet_hash = %e.packet().hash(),

            data.packet_data = %e.packet_data,

            data.packet.source_channel.channel_id = %e.packet.source_channel.channel_id,
            data.packet.source_channel.version = %e.packet.source_channel.version,
            data.packet.source_channel.connection.client_id = %e.packet.source_channel.connection.client_id,
            data.packet.source_channel.connection.connection_id = %e.packet.source_channel.connection.connection_id,

            data.packet.destination_channel.channel_id = %e.packet.destination_channel.channel_id,
            data.packet.destination_channel.version = %e.packet.destination_channel.version,
            data.packet.destination_channel.connection.client_id = %e.packet.destination_channel.connection.client_id,
            data.packet.destination_channel.connection.connection_id = %e.packet.destination_channel.connection.connection_id,

            data.packet.timeout_height = %e.packet.timeout_height,
            data.packet.timeout_timestamp = %e.packet.timeout_timestamp,
            "event"
        ),
    }
}
