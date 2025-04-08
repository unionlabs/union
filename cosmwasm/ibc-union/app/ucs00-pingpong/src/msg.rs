use cosmwasm_schema::cw_serde;
use ethabi::{ParamType, Token};
use ibc_union_msg::msg::MsgSendPacket;
use ibc_union_spec::{ChannelId, Timestamp};

use crate::{state::Config, ContractError};

#[cw_serde]
pub struct UCS00PingPong {
    pub ping: bool,
    // /// in seconds
    // pub counterparty_timeout: u64,
}

impl UCS00PingPong {
    pub fn decode(bz: impl AsRef<[u8]>) -> Result<Self, ContractError> {
        let values = ethabi::decode(&[ParamType::Bool /*ParamType::Int(64)*/], bz.as_ref())
            .map_err(|_| ContractError::EthAbiDecoding)?;
        match &values[..] {
            &[Token::Bool(ping),/* Token::Int(timeout) */] => Ok(UCS00PingPong {
                ping,
                // counterparty_timeout: timeout.as_u64(),
            }),
            _ => Err(ContractError::EthAbiDecoding),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        ethabi::encode(&[
            Token::Bool(self.ping),
            // Token::Int(self.counterparty_timeout.into()),
        ])
    }
}

impl UCS00PingPong {
    pub fn reverse(
        &self,
        config: &Config,
        current_timestamp: Timestamp,
        source_channel: ChannelId,
    ) -> ibc_union_msg::msg::ExecuteMsg {
        let counterparty_packet = UCS00PingPong {
            ping: !self.ping,
            // counterparty_timeout: config.seconds_before_timeout * 1_000_000_000 + current_timestamp,
        };
        ibc_union_msg::msg::ExecuteMsg::PacketSend(MsgSendPacket {
            source_channel_id: source_channel,
            timeout_height: 0,
            timeout_timestamp: current_timestamp
                + Timestamp::from_secs(config.seconds_before_timeout),
            data: counterparty_packet.encode().into(),
        })
    }
}

#[cw_serde]
pub struct InitMsg {
    pub config: Config,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Initiate {
        channel_id: ChannelId,
        packet: UCS00PingPong,
    },
    IbcUnionMsg(ibc_union_msg::module::IbcUnionMsg),
}
