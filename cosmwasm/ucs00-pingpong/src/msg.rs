use cosmwasm_schema::cw_serde;
use cosmwasm_std::{IbcMsg, IbcTimeout, Timestamp};
use ethabi::{ParamType, Token};

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
    pub fn reverse(&self, config: &Config, current_timestamp: u64, channel_id: String) -> IbcMsg {
        let counterparty_packet = UCS00PingPong {
            ping: !self.ping,
            // counterparty_timeout: config.seconds_before_timeout * 1_000_000_000 + current_timestamp,
        };
        IbcMsg::SendPacket {
            channel_id,
            data: counterparty_packet.encode().into(),
            timeout: IbcTimeout::with_timestamp(Timestamp::from_nanos(
                current_timestamp + config.seconds_before_timeout * 1_000_000_000,
            )),
        }
    }
}

#[cw_serde]
pub struct InitMsg {
    pub config: Config,
}

#[cw_serde]
pub enum ExecuteMsg {
    Initiate {
        channel_id: String,
        packet: UCS00PingPong,
    },
}
