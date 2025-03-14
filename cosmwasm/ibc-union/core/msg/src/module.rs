use ibc_union_spec::{ChannelId, ConnectionId, Packet};
use unionlabs_primitives::Bytes;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum IbcUnionMsg {
    OnChannelOpenInit {
        caller: String,
        connection_id: ConnectionId,
        channel_id: ChannelId,
        version: String,
        relayer: String,
    },
    OnChannelOpenTry {
        caller: String,
        connection_id: ConnectionId,
        channel_id: ChannelId,
        version: String,
        counterparty_version: String,
        relayer: String,
    },
    OnChannelOpenAck {
        caller: String,
        channel_id: ChannelId,
        counterparty_channel_id: ChannelId,
        counterparty_version: String,
        relayer: String,
    },
    OnChannelOpenConfirm {
        caller: String,
        channel_id: ChannelId,
        relayer: String,
    },
    OnChannelCloseInit {
        caller: String,
        channel_id: ChannelId,
        relayer: String,
    },
    OnChannelCloseConfirm {
        caller: String,
        channel_id: ChannelId,
        relayer: String,
    },
    OnIntentRecvPacket {
        caller: String,
        packet: Packet,
        market_maker: String,
        market_maker_msg: Bytes,
    },
    OnRecvPacket {
        caller: String,
        packet: Packet,
        relayer: String,
        relayer_msg: Bytes,
    },
    OnAcknowledgementPacket {
        caller: String,
        packet: Packet,
        acknowledgement: Bytes,
        relayer: String,
    },
    OnTimeoutPacket {
        caller: String,
        packet: Packet,
        relayer: String,
    },
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ExecuteMsg {
    IbcUnionMsg(IbcUnionMsg),
}
