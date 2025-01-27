use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, CosmosMsg, Uint128, Uint256};
use ibc_union_spec::types::Packet;
use unionlabs::primitives::{Bytes, H256};

use crate::state::Config;

#[cw_serde]
pub enum TokenMinterInitMsg {
    Cw20 { cw20_code_id: u64 },
    Native,
}

#[cw_serde]
pub struct InitMsg {
    pub config: Config,
    pub minter_init_msg: TokenMinterInitMsg,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Transfer {
        channel_id: u32,
        receiver: Bytes,
        base_token: String,
        base_amount: Uint128,
        quote_token: Bytes,
        quote_amount: Uint256,
        timeout_height: u64,
        timeout_timestamp: u64,
        salt: H256,
    },
    BatchExecute {
        msgs: Vec<CosmosMsg>,
    },
    ExecutePacket {
        packet: Packet,
        relayer: Addr,
        relayer_msg: Bytes,
    },
    IbcUnionMsg(ibc_union_msg::module::IbcUnionMsg),
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EurekaMsg {
    OnZkgm {
        channel_id: u32,
        sender: Bytes,
        message: Bytes,
    },
}

#[cw_serde]
pub struct MigrateMsg {}
