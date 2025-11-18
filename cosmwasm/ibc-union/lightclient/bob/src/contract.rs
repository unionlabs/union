use std::num::NonZero;

use cosmwasm_std::{DepsMut, Env, Response, StdError, wasm_execute};
use depolama::StorageExt;
use ibc_union_light_client::{
    IbcClientError, default_migrate, default_query, default_reply, read_client_state,
    read_consensus_state, spec::ClientId, state::IbcHost,
};
use ibc_union_msg::msg::MsgMigrateState;
use unionlabs::{
    encoding::{Bincode, EncodeAs, EthAbi},
    primitives::{H160, U256},
};

use crate::client::BobLightClient;

default_query!(BobLightClient);
default_migrate!(BobLightClient; MigrateMsg; do_migrate);
default_reply!();

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MigrateMsg {
    V1ToV2 {
        clients_to_migrate: Vec<ClientId>,
        dispute_game_factory_address: H160,
        dispute_game_factory_dispute_game_list_slot: U256,
        fault_dispute_game_code_root_claim_index: u32,
    },
}

fn do_migrate(
    deps: DepsMut,
    _env: Env,
    migrate_msg: MigrateMsg,
    current_version: NonZero<u32>,
) -> Result<(Response, Option<NonZero<u32>>), IbcClientError<BobLightClient>> {
    match (migrate_msg, current_version.get()) {
        (
            MigrateMsg::V1ToV2 {
                clients_to_migrate,
                dispute_game_factory_address,
                dispute_game_factory_dispute_game_list_slot,
                fault_dispute_game_code_root_claim_index,
            },
            current_version @ (1 | 2),
        ) => {
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
                        add_migrate_state_msg(ibc_union_msg::msg::ExecuteMsg::MigrateState(
                            MsgMigrateState {
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
                                consensus_state: consensus_state.encode_as::<EthAbi>().into(),
                                height: latest_height,
                            },
                        ))?;
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
            Ok((
                Response::default().add_messages(migrate_state_msgs),
                if current_version == 1 {
                    Some(<NonZero<u32>>::new(2).expect("impossible"))
                } else {
                    None
                },
            ))
        }
        (msg, version) => Err(StdError::generic_err(format!(
            "unknown migration, message={:?}, version={}",
            msg, version
        ))
        .into()),
    }
}
