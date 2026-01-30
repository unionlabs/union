use std::collections::BTreeMap;

use cosmwasm_std::{Addr, Uint128};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Set the LST hub address.
    SetLstHubAddress(Addr),
    /// Configure the delegation set. This completely overwrites the existing configuration.
    ///
    /// If this is not the first time configuring the delegation set, the LST will rebase and the
    /// current delegated amounts will be rebalanced amongst the new set with `MsgRedelegate`s. Due
    /// to the [redelegation queue], it is (in practice) only possible to re-configure the validator
    /// set once per unbonding period.
    ///
    /// [redelegation queue]: https://docs.cosmos.network/main/build/modules/staking#redelegation
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
    #[serde(untagged)]
    CwAccount(cw_account::msg::QueryMsg),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct MigrateMsg {}
