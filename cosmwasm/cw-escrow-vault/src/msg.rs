use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint256};
use depolama::Bytes;
use ibc_union_spec::{ChannelId, Packet};
use ucs03_zkgm::com::CwTokenOrderV2;
use unionlabs::primitives::H256;

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
        path: Uint256,
        channel_id: ChannelId,
        base_token: Bytes,
        counterparty_beneficiary: Bytes,
        escrowed_denom: String,
    },
    DoSolve {
        packet: Packet,
        order: Box<CwTokenOrderV2>,
        path: Uint256,
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
}
