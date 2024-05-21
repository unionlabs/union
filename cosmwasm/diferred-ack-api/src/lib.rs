use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, CosmosMsg, CustomMsg, CustomQuery, Uint64};

#[derive(thiserror::Error, Clone, PartialEq, Eq, Debug)]
pub enum EncodingError {
    #[error("Unable to encode or decode the data")]
    InvalidEncoding,
}

/// Special messages to be supported by any chain that supports diferred_ack
#[cw_serde]
pub enum DiferredAckMsg {
    WriteDiferredAck {
        sender: String,
        diferred_packet_info: DiferredPacketInfo,
        ack: Acknowledgement,
    },
}

impl TryFrom<DiferredAckMsg> for Binary {
    type Error = EncodingError;

    fn try_from(value: DiferredAckMsg) -> Result<Self, Self::Error> {
        Ok(cosmwasm_std::to_json_vec(&value)
            .map_err(|_| EncodingError::InvalidEncoding)?
            .into())
    }
}

#[cw_serde]
pub struct DiferredPacketInfo {
    pub refund_channel_id: String,
    pub refund_port_id: String,
    pub packet_src_channel_id: String,
    pub packet_src_port_id: String,
    pub packet_timeout_timestamp: Uint64,
    pub packet_timeout_height: String,
    pub packet_data: Binary,
    pub sequence: Uint64,
}

/// Acknowledgement is the recommended acknowledgement format to be used by
/// app-specific protocols.
#[cw_serde]
pub struct Acknowledgement {
    /// response contains either a result or an error and must be non-empty
    pub response: Option<Response>,
}

/// response contains either a result or an error and must be non-empty
#[cw_serde]
pub enum Response {
    Result(Binary),
    Error(String),
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
