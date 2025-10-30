use access_managed::Restricted;
use cosmwasm_std::Addr;
use ibc_union_spec::ChannelId;
use serde::{Deserialize, Serialize};
use ucs03_solvable::{Solvable, SolverQuery};
use unionlabs_primitives::{Bytes, H256, U256};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub zkgm: Addr,
    pub access_managed_init_msg: access_managed::InitMsg,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    #[serde(untagged)]
    Solvable(Solvable),
    #[serde(untagged)]
    AccessManaged(access_managed::ExecuteMsg),
    #[serde(untagged)]
    Restricted(Restricted<RestrictedExecuteMsg>),
}

/// Subset of [`ExecuteMsg`] for entrypoints that are access managed.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum RestrictedExecuteMsg {
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
    Upgradable(upgradable::msg::ExecuteMsg),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum QueryMsg {
    GetFungibleCounterparty {
        path: U256,
        channel_id: ChannelId,
        base_token: Bytes,
    },
    GetAllFungibleCounterparties,
    #[serde(untagged)]
    Solvable(SolverQuery),
    #[serde(untagged)]
    AccessManaged(access_managed::QueryMsg),
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
