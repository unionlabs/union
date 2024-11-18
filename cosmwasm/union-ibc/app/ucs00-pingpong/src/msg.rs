use cosmwasm_schema::cw_serde;
use ethabi::{ParamType, Token};
use union_ibc_msg::msg::MsgSendPacket;

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
        current_timestamp: u64,
        source_channel: u32,
    ) -> union_ibc_msg::msg::ExecuteMsg {
        let counterparty_packet = UCS00PingPong {
            ping: !self.ping,
            // counterparty_timeout: config.seconds_before_timeout * 1_000_000_000 + current_timestamp,
        };
        union_ibc_msg::msg::ExecuteMsg::PacketSend(MsgSendPacket {
            source_channel,
            timeout_height: 0,
            timeout_timestamp: current_timestamp + config.seconds_before_timeout * 1_000_000_000,
            data: counterparty_packet.encode().into(),
        })
    }
}

#[cw_serde]
pub struct InitMsg {
    pub config: Config,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ExecuteMsg {
    Initiate {
        channel_id: u32,
        packet: UCS00PingPong,
    },
    UnionIbc(union_ibc_msg::module::UnionIbcMsg),
}
