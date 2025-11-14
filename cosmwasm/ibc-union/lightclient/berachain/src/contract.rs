use ibc_union_light_client::{default_migrate, default_query, default_reply, noop_migration};
use serde::{Deserialize, Serialize};

use crate::client::BerachainLightClient;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct MigrateMsg {}

default_query!(BerachainLightClient);
default_migrate!(BerachainLightClient; MigrateMsg; noop_migration);
default_reply!();
