use ibc_solidity::Packet;
use unionlabs::bytes::Bytes;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum IbcUnionMsg {
    OnChannelOpenInit {
        connection_id: u32,
        channel_id: u32,
        version: String,
        relayer: String,
    },
    OnChannelOpenTry {
        connection_id: u32,
        channel_id: u32,
        version: String,
        counterparty_version: String,
        relayer: String,
    },
    OnChannelOpenAck {
        channel_id: u32,
        counterparty_channel_id: u32,
        counterparty_version: String,
        relayer: String,
    },
    OnChannelOpenConfirm {
        channel_id: u32,
        relayer: String,
    },
    OnChannelCloseInit {
        channel_id: u32,
        relayer: String,
    },
    OnChannelCloseConfirm {
        channel_id: u32,
        relayer: String,
    },
    OnIntentRecvPacket {
        packet: Packet,
        market_maker: String,
        market_maker_msg: Bytes,
    },
    OnRecvPacket {
        packet: Packet,
        relayer: String,
        relayer_msg: Bytes,
    },
    OnAcknowledgementPacket {
        packet: Packet,
        acknowledgement: Bytes,
        relayer: String,
    },
    OnTimeoutPacket {
        packet: Packet,
        relayer: String,
    },
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ExecuteMsg {
    IbcUnionMsg(IbcUnionMsg),
}
