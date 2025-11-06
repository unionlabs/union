use cosmwasm_std::{DepsMut, Env, Response};
use frissitheto::UpgradeMsg;
use ibc_union_light_client::{
    IbcClientError, default_migrate, default_query, default_reply, msg::InitMsg, noop_migration,
};
use serde::{Deserialize, Serialize};

use crate::client::ArbitrumLightClient;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct MigrateMsg {}

default_query!(ArbitrumLightClient);
default_migrate!(ArbitrumLightClient; MigrateMsg; noop_migration);
default_reply!();
