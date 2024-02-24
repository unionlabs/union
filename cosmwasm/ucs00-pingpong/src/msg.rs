use cosmwasm_schema::cw_serde;
use cosmwasm_std::{IbcMsg, IbcTimeout, IbcTimeoutBlock};
use ethabi::{ParamType, Token};

use crate::{state::Config, ContractError};

#[cw_serde]
pub struct UCS00PingPong {
    pub ping: bool,
    pub counterparty_timeout_revision_number: u64,
    pub counterparty_timeout_revision_height: u64,
}

impl UCS00PingPong {
    pub fn decode(bz: impl AsRef<[u8]>) -> Result<Self, ContractError> {
        let values = ethabi::decode(
            &[ParamType::Bool, ParamType::Int(64), ParamType::Int(64)],
            bz.as_ref(),
        )
        .map_err(|_| ContractError::EthAbiDecoding)?;
        match &values[..] {
            &[Token::Bool(ping), Token::Int(timeout_revision_number), Token::Int(timeout_revision_height)] => {
                Ok(UCS00PingPong {
                    ping,
                    counterparty_timeout_revision_number: timeout_revision_number.as_u64(),
                    counterparty_timeout_revision_height: timeout_revision_height.as_u64(),
                })
            }
            _ => Err(ContractError::EthAbiDecoding),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        ethabi::encode(&[
            Token::Bool(self.ping),
            Token::Int(self.counterparty_timeout_revision_number.into()),
            Token::Int(self.counterparty_timeout_revision_height.into()),
        ])
    }
}

impl UCS00PingPong {
    pub fn reverse(&self, config: &Config, current_block: u64, channel_id: String) -> IbcMsg {
        let counterparty_packet = UCS00PingPong {
            ping: !self.ping,
            counterparty_timeout_revision_number: config.revision_number,
            counterparty_timeout_revision_height: config.number_of_block_before_pong_timeout
                + current_block,
        };
        IbcMsg::SendPacket {
            channel_id,
            data: counterparty_packet.encode().into(),
            timeout: IbcTimeout::with_block(IbcTimeoutBlock {
                revision: self.counterparty_timeout_revision_number,
                height: self.counterparty_timeout_revision_height,
            }),
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
