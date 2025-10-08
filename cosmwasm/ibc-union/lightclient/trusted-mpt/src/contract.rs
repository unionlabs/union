use cosmwasm_std::{Binary, Deps, DepsMut, Env, Response, StdResult, entry_point};
use frissitheto::UpgradeMsg;
use ibc_union_light_client::{
    IbcClientError,
    msg::{InitMsg, QueryMsg},
};
use serde::{Deserialize, Serialize};

use crate::client::MptTrustedLightClient;

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    ibc_union_light_client::query::<MptTrustedLightClient>(deps, env, msg).map_err(Into::into)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrateMsg {}

#[entry_point]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    msg: UpgradeMsg<InitMsg, MigrateMsg>,
) -> Result<Response, IbcClientError<MptTrustedLightClient>> {
    msg.run(
        deps,
        |deps, init_msg| {
            let res = ibc_union_light_client::init(deps, init_msg)?;

            Ok((res, None))
        },
        |_deps, _migrate_msg, _current_version| Ok((Response::default(), None)),
    )
}
