use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;
use ethabi::{ParamType, Token};

use crate::ContractError;

#[cw_serde]
pub struct UCS00PingPong {
    pub ping: bool,
}

impl UCS00PingPong {
    pub fn reverse(self) -> Self {
        UCS00PingPong { ping: !self.ping }
    }
}

impl From<UCS00PingPong> for Binary {
    fn from(value: UCS00PingPong) -> Self {
        ethabi::encode(&[Token::Bool(value.ping)]).into()
    }
}

impl TryFrom<Binary> for UCS00PingPong {
    type Error = ContractError;
    fn try_from(value: Binary) -> Result<Self, Self::Error> {
        let values = ethabi::decode(&[ParamType::Bool], &value.0)
            .map_err(|_| ContractError::EthAbiDecoding)?;
        match &values[..] {
            &[Token::Bool(ping)] => Ok(UCS00PingPong { ping }),
            _ => Err(ContractError::EthAbiDecoding),
        }
    }
}

#[cw_serde]
pub struct InitMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Initiate {
        channel_id: String,
        packet: UCS00PingPong,
    },
}
