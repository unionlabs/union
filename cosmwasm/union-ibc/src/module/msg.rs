use cosmwasm_std::{Addr, Binary};
use ibc_solidity::ibc::{ChannelOrder, Packet};

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ExecuteMsg {
    OnChannelOpenInit {
        order: ChannelOrder,
        connection_id: u32,
        channel_id: u32,
        version: String,
        relayer: Addr,
    },
    OnChannelOpenTry {
        order: ChannelOrder,
        connection_id: u32,
        channel_id: u32,
        version: String,
        counterparty_version: String,
        relayer: Addr,
    },
    OnChannelOpenAck {
        channel_id: u32,
        counterparty_channel_id: u32,
        counterparty_version: String,
        relayer: Addr,
    },
    OnChannelOpenConfirm {
        channel_id: u32,
        relayer: Addr,
    },
    OnChannelCloseInit {
        channel_id: u32,
        relayer: Addr,
    },
    OnChannelCloseConfirm {
        channel_id: u32,
        relayer: Addr,
    },
    OnIntentRecvPacket {
        packet: Packet,
        market_maker: Addr,
        market_maker_msg: Binary,
    },
    OnRecvPacket {
        packet: Packet,
        relayer: Addr,
        relayer_msg: Binary,
    },
    OnAcknowledgementPacket {
        packet: Packet,
        acknowledgement: Binary,
        relayer: Addr,
    },
    OnTimeoutPacket {
        packet: Packet,
        relayer: Addr,
    },
}
