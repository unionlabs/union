use std::collections::BTreeMap;

use cosmwasm_std::{Addr, Uint128};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Set the LST hub address.
    SetLstHubAddress(Addr),
    /// Set the validators to stake against. This completely overwrites the existing configuration.
    SetValidators(BTreeMap<Addr, Uint128>),
    #[serde(untagged)]
    CwAccount(cw_account::msg::ExecuteMsg),
    #[serde(untagged)]
    Staker(lst::msg::StakerExecuteMsg),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum QueryMsg {
    Validators {},
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct MigrateMsg {}
