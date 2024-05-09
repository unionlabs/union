use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    Binary, CosmosMsg, CustomMsg, CustomQuery, IbcAcknowledgement, IbcTimeoutBlock, Uint64,
};

/// Special messages to be supported by any chain that supports diferred_ack
#[cw_serde]
pub enum DiferredAckMsg {
    WriteDiferredAck {
        packet: Packet,
        data: FungibleTokenPacketData,
        diferred_packet_info: DiferredPacketInfo,
        ack: IbcAcknowledgement,
    },
}

#[cw_serde]
pub struct Packet {
    pub sequence: Uint64,
    pub source_port: String,
    pub source_channel: String,
    pub destination_port: String,
    pub destination_channel: String,
    pub timeout_height: IbcTimeoutBlock,
    pub timeout_timestamp: Uint64,
}

#[cw_serde]
pub struct FungibleTokenPacketData {
    pub denom: String,
    pub amount: String,
    pub sender: String,
    pub receiver: String,
    pub memo: String,
}

#[cw_serde]
pub struct DiferredPacketInfo {
    pub original_sender_address: String,
    pub refund_channel_id: String,
    pub refund_port_id: String,
    pub packet_src_channel_id: String,
    pub packet_src_port_id: String,
    pub packet_timeout_timestamp: Uint64,
    pub packet_timeout_height: String,
    pub packet_data: Binary,
    pub refund_sequence: Uint64,
    pub timeout: Uint64,
    pub nonrefundable: bool,
}

/// This maps to diferredack.v1beta1.Params protobuf struct
#[cw_serde]
pub struct Params {}

impl From<DiferredAckMsg> for CosmosMsg<DiferredAckMsg> {
    fn from(msg: DiferredAckMsg) -> CosmosMsg<DiferredAckMsg> {
        CosmosMsg::Custom(msg)
    }
}

impl CustomMsg for DiferredAckMsg {}

/// This is in the data field in the reply from a DiferredAck::WriteDiferredAck SubMsg
pub struct WriteDiferredAckResponse {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum DiferredAckQuery {
    /// Returns configuration params for Diferred Ack modules
    #[returns(ParamsResponse)]
    Params {},
}

impl CustomQuery for DiferredAckQuery {}

#[cw_serde]
pub struct ParamsResponse {
    pub params: Params,
}
