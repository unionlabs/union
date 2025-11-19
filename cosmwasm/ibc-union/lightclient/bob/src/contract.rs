use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, wasm_execute};
use depolama::StorageExt;
use frissitheto::UpgradeMsg;
use ibc_union_light_client::{
    IbcClientError,
    access_managed::{EnsureCanCallResult, Restricted, state::Authority},
    default_query, default_reply,
    msg::{InitMsg, MigrateMsg},
    read_client_state, read_consensus_state,
    spec::ClientId,
    state::IbcHost,
    version,
};
use ibc_union_msg::msg::MsgMigrateState;
use serde::{Deserialize, Serialize};
use unionlabs::{
    encoding::{Bincode, EncodeAs, EthAbi},
    primitives::{H160, U256},
};

use crate::client::BobLightClient;

default_query!(BobLightClient);
default_reply!();

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ExecuteMsg {
    #[serde(untagged)]
    LightClient(ibc_union_light_client::msg::ExecuteMsg),
    #[serde(untagged)]
    Restricted(Restricted<RestrictedExecuteMsg>),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum RestrictedExecuteMsg {
    V1ToV2 {
        clients_to_migrate: Vec<ClientId>,
        dispute_game_factory_address: H160,
        dispute_game_factory_dispute_game_list_slot: U256,
        fault_dispute_game_code_root_claim_index: u32,
    },
}

#[cosmwasm_std::entry_point]
pub fn execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, IbcClientError<BobLightClient>> {
    match msg {
        ExecuteMsg::LightClient(msg) => ibc_union_light_client::execute(deps, env, info, msg),
        ExecuteMsg::Restricted(msg) => {
            let msg = match msg.ensure_can_call::<Authority>(deps.branch(), &env, &info)? {
                EnsureCanCallResult::Msg(msg) => msg,
                EnsureCanCallResult::Scheduled(sub_msgs) => {
                    return Ok(Response::new()
                        .add_submessages(sub_msgs)
                        .change_custom()
                        .unwrap());
                }
            };

            match msg {
                RestrictedExecuteMsg::V1ToV2 {
                    clients_to_migrate,
                    dispute_game_factory_address,
                    dispute_game_factory_dispute_game_list_slot,
                    fault_dispute_game_code_root_claim_index,
                } => {
                    let ibc_host = deps.storage.read_item::<IbcHost>()?;
                    let mut migrate_state_msgs = Vec::new();
                    let mut add_migrate_state_msg = |msg: ibc_union_msg::msg::ExecuteMsg| {
                        migrate_state_msgs.push(wasm_execute(&ibc_host, &msg, vec![])?);
                        Ok::<_, IbcClientError<_>>(())
                    };
                    for client_id in clients_to_migrate {
                        match read_client_state(&*deps.querier, &ibc_host, client_id)? {
                            bob_light_client_types::ClientState::V1(v1) => {
                                let latest_height = v1.latest_height;
                                let consensus_state = read_consensus_state::<BobLightClient>(
                                    &*deps.querier,
                                    &ibc_host,
                                    client_id,
                                    latest_height,
                                )?;
                                add_migrate_state_msg(
                                    ibc_union_msg::msg::ExecuteMsg::MigrateState(MsgMigrateState {
                                        client_id,
                                        client_state: bob_light_client_types::ClientState::V2(
                                            bob_light_client_types::ClientStateV2 {
                                                chain_id: v1.chain_id,
                                                latest_height: v1.latest_height,
                                                l1_client_id: v1.l1_client_id,
                                                dispute_game_factory_address,
                                                dispute_game_factory_dispute_game_list_slot,
                                                fault_dispute_game_code_root_claim_index,
                                                frozen_height: v1.frozen_height,
                                                ibc_contract_address: v1.ibc_contract_address,
                                            },
                                        )
                                        .encode_as::<Bincode>()
                                        .into(),
                                        consensus_state: consensus_state
                                            .encode_as::<EthAbi>()
                                            .into(),
                                        height: latest_height,
                                    }),
                                )?;
                            }
                            bob_light_client_types::ClientState::V2(_) => {
                                return Err(StdError::generic_err(format!(
                                    "unexpected client state v2 for v1 to v2 migration, client={}",
                                    client_id
                                ))
                                .into());
                            }
                        }
                    }

                    Ok(Response::default().add_messages(migrate_state_msgs))
                }
            }
        }
    }
}

#[cosmwasm_std::entry_point]
pub fn migrate(
    deps: DepsMut,
    _: Env,
    msg: UpgradeMsg<InitMsg, MigrateMsg>,
) -> Result<Response, IbcClientError<BobLightClient>> {
    msg.run(
        deps,
        |deps, init_msg| {
            let res = ibc_union_light_client::init(deps, init_msg)?;
            Ok((res, Some(version::LATEST)))
        },
        |mut deps, msg, version| match version {
            version::INIT => {
                ibc_union_light_client::access_managed::init(
                    deps.branch(),
                    msg.access_managed_init_msg,
                )?;
                Ok((Response::default(), Some(version::MANAGED)))
            }
            // NOTE: We messed with the state version previously, this is required to standardize it between all of the contracts
            version::MANAGED => {
                ibc_union_light_client::access_managed::init(
                    deps.branch(),
                    msg.access_managed_init_msg,
                )?;
                Ok((Response::default(), None))
            }
            _ => Err(::frissitheto::UpgradeError::UnknownStateVersion(version).into()),
        },
    )
}
