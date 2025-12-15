use cainome_cairo_serde::{ByteArray, ContractAddress, NonZero};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CairoIbcEvent {
    RegisterClient {
        /// `#[key]`
        client_type: ByteArray,
        client_address: ContractAddress,
    },
    CreateClient {
        /// `#[key]`
        client_type: ByteArray,
        /// `#[key]`
        client_id: NonZero<u32>,
        /// `#[key]`
        counterparty_chain_id: ByteArray,
    },
    UpdateClient {
        /// `#[key]`
        client_id: NonZero<u32>,
        /// `#[key]`
        height: u64,
    },
    ConnectionOpenInit {
        /// `#[key]`
        connection_id: NonZero<u32>,
        /// `#[key]`
        client_id: NonZero<u32>,
        counterparty_client_id: NonZero<u32>,
    },
    ConnectionOpenTry {
        /// `#[key]`
        connection_id: NonZero<u32>,
        /// `#[key]`
        client_id: NonZero<u32>,
        counterparty_client_id: NonZero<u32>,
        counterparty_connection_id: NonZero<u32>,
    },
    ConnectionOpenAck {
        /// `#[key]`
        connection_id: NonZero<u32>,
        /// `#[key]`
        client_id: NonZero<u32>,
        counterparty_client_id: NonZero<u32>,
        counterparty_connection_id: NonZero<u32>,
    },
    ConnectionOpenConfirm {
        /// `#[key]`
        connection_id: NonZero<u32>,
        /// `#[key]`
        client_id: NonZero<u32>,
        counterparty_client_id: NonZero<u32>,
        counterparty_connection_id: NonZero<u32>,
    },
    ChannelOpenInit {
        /// `#[key]`
        port_id: ContractAddress,
        /// `#[key]`
        channel_id: NonZero<u32>,
        counterparty_port_id: ByteArray,
        connection_id: NonZero<u32>,
        /// `#[key]`
        version: ByteArray,
    },
    ChannelOpenTry {
        /// `#[key]`
        port_id: ContractAddress,
        /// `#[key]`
        channel_id: NonZero<u32>,
        counterparty_port_id: ByteArray,
        counterparty_channel_id: NonZero<u32>,
        connection_id: NonZero<u32>,
        /// `#[key]`
        counterparty_version: ByteArray,
    },
    ChannelOpenAck {
        /// `#[key]`
        port_id: ContractAddress,
        /// `#[key]`
        channel_id: NonZero<u32>,
        counterparty_port_id: ByteArray,
        counterparty_channel_id: NonZero<u32>,
        connection_id: NonZero<u32>,
    },
    ChannelOpenConfirm {
        /// `#[key]`
        port_id: ContractAddress,
        /// `#[key]`
        channel_id: NonZero<u32>,
        counterparty_port_id: ByteArray,
        counterparty_channel_id: NonZero<u32>,
        connection_id: NonZero<u32>,
    },
    ChannelCloseInit {
        /// `#[key]`
        port_id: ContractAddress,
        /// `#[key]`
        channel_id: NonZero<u32>,
        counterparty_port_id: ByteArray,
        counterparty_channel_id: NonZero<u32>,
    },
    ChannelCloseConfirm {
        /// `#[key]`
        port_id: ContractAddress,
        /// `#[key]`
        channel_id: NonZero<u32>,
        counterparty_port_id: ByteArray,
        counterparty_channel_id: NonZero<u32>,
    },
}
