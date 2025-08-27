use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use ibc_union_spec::{ChannelId, Packet};
use ucs03_zkgm_api::CwTokenOrderV2;
use unionlabs::primitives::{Bytes, H256, U256};

#[cw_serde]
pub struct InstantiateMsg {
    pub zkgm: Addr,
    pub admin: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    WhitelistIntents {
        hashes_whitelist: Vec<(H256, bool)>,
    },
    SetFungibleCounterparty {
        path: U256,
        channel_id: ChannelId,
        base_token: Bytes,
        counterparty_beneficiary: Bytes,
        escrowed_denom: String,
    },
    DoSolve {
        packet: Packet,
        order: Box<CwTokenOrderV2>,
        path: U256,
        caller: Addr,
        relayer: Addr,
        relayer_msg: Bytes,
        intent: bool,
    },
}

#[cw_serde]
pub enum QueryMsg {
    IsSolver,
    AllowMarketMakers,
    GetFungibleCounterparty {
        path: U256,
        channel_id: ChannelId,
        base_token: Bytes,
    },
    GetAllFungibleCounterparties,
}

#[derive(serde::Serialize)]
pub struct FungibleLaneConfig {
    pub path: U256,
    pub channel_id: ChannelId,
    pub base_token: Bytes,
    pub counterparty_beneficiary: Bytes,
    pub escrowed_denom: String,
    pub is_cw20: bool,
}
