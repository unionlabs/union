use cosmwasm_std::{Addr, CosmosMsg};
use depolama::Bytes;
use ibc_union_spec::ChannelId;
use serde::{Deserialize, Serialize};
use ucs03_zkgmable::Zkgmable;
use unionlabs_primitives::U256;

use crate::types::Admin;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum InitMsg {
    Zkgm {
        zkgm: Addr,
        path: U256,
        channel_id: ChannelId,
        sender: Bytes,
    },
    Local {
        admin: Addr,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ExecuteMsg {
    SetZkgm(Addr),
    AddAdmin(Admin),
    RemoveAdmin(Admin),
    Dispatch(Vec<CosmosMsg>),
    #[serde(untagged)]
    Zkgmable(Zkgmable),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum QueryMsg {
    Admins {},
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct MigrateMsg {}
