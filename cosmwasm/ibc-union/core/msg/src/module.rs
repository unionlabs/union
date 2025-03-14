use ibc_union_spec::{ChannelId, ConnectionId, Packet};
use unionlabs_primitives::Bytes;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum IbcUnionMsg {
    OnChannelOpenInit {
        connection_id: ConnectionId,
        channel_id: ChannelId,
        version: String,
        relayer: String,
    },
    OnChannelOpenTry {
        connection_id: ConnectionId,
        channel_id: ChannelId,
        version: String,
        counterparty_version: String,
        relayer: String,
    },
    OnChannelOpenAck {
        channel_id: ChannelId,
        counterparty_channel_id: ChannelId,
        counterparty_version: String,
        relayer: String,
    },
    OnChannelOpenConfirm {
        channel_id: ChannelId,
        relayer: String,
    },
    OnChannelCloseInit {
        channel_id: ChannelId,
        relayer: String,
    },
    OnChannelCloseConfirm {
        channel_id: ChannelId,
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
