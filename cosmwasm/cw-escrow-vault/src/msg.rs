use cosmwasm_std::Addr;
use ibc_union_spec::ChannelId;
use serde::{Deserialize, Serialize};
use ucs03_solvable::Solvable;
use unionlabs_primitives::{Bytes, H256, U256};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub zkgm: Addr,
    pub admin: Addr,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
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
    #[serde(untagged)]
    Solvable(Solvable),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
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

#[derive(Serialize)]
pub struct FungibleLaneConfig {
    pub path: U256,
    pub channel_id: ChannelId,
    pub base_token: Bytes,
    pub counterparty_beneficiary: Bytes,
    pub escrowed_denom: String,
    pub is_cw20: bool,
}
