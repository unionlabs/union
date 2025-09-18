// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.
//
// Parameters
//
// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's cosmwasm/lst subdirectory
//                       The Licensed Work is (c) 2025 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
//
//
// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.
//
// Notice
//
// Business Source License 1.1
//
// Terms
//
// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.
//
// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.
//
// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.
//
// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.
//
// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.
//
// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.
//
// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).
//
// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

use cosmwasm_std::{
    ensure, entry_point, to_json_binary, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response,
    StdResult,
};
use depolama::StorageExt;
use frissitheto::UpgradeMsg;
use serde::{Deserialize, Serialize};

use crate::{
    error::ContractError,
    execute::{
        accept_ownership, bond, circuit_breaker, receive_rewards, receive_unstaked_tokens,
        resume_contract, revoke_ownership_transfer, slash_batches, submit_batch,
        transfer_ownership, unbond, update_config, withdraw, FEE_RATE_DENOMINATOR,
    },
    msg::{ExecuteMsg, InitMsg, QueryMsg},
    query::{
        query_all_unstake_requests, query_batch, query_batches, query_batches_by_ids, query_config,
        query_pending_batch, query_state, query_unstake_requests,
    },
    state::{
        AccountingStateStore, Admin, ConfigStore, CurrentPendingBatch, LstAddress, Monitors,
        ProtocolFeeConfigStore, ReceivedBatches, StakerAddress, Stopped, SubmittedBatches,
    },
    types::{AccountingState, BatchId, Config, PendingBatch},
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(_: DepsMut, _: Env, _: MessageInfo, _: ()) -> StdResult<Response> {
    panic!("this contract cannot be instantiated directly, but must be migrated from an existing instantiated contract.");
}

pub fn init(deps: DepsMut, env: Env, msg: InitMsg) -> Result<Response, ContractError> {
    let InitMsg {
        native_token_denom,
        minimum_liquid_stake_amount,
        staker_address,
        protocol_fee_config,
        lst_address,
        batch_period_seconds,
        unbonding_period_seconds,
        monitors,
        admin,
    } = msg;

    // TODO: Use this once unbonding period is queryable from the chain
    // let unbonding_period =
    //     query_and_validate_unbonding_period(deps.as_ref(), batch_period_seconds)?;

    ensure!(
        batch_period_seconds <= unbonding_period_seconds,
        ContractError::BatchPeriodLargerThanUnbondingPeriod {
            batch_period_seconds,
            unbonding_period_seconds,
        }
    );

    ensure!(
        protocol_fee_config.fee_rate <= FEE_RATE_DENOMINATOR as u128,
        ContractError::InvalidProtocolFeeRate
    );

    // save various addresses
    deps.storage.write_item::<Admin>(&admin);
    deps.storage.write_item::<StakerAddress>(&staker_address);
    deps.storage.write_item::<LstAddress>(&lst_address);
    deps.storage
        .write_item::<Monitors>(&monitors.into_iter().map(Into::into).collect());

    // save configs and state
    deps.storage
        .write_item::<ProtocolFeeConfigStore>(&protocol_fee_config);
    deps.storage.write_item::<ConfigStore>(&Config {
        native_token_denom: native_token_denom.clone(),
        minimum_liquid_stake_amount,
        batch_period_seconds,
        unbonding_period_seconds,
    });
    deps.storage
        .write_item::<AccountingStateStore>(&AccountingState {
            total_bonded_native_tokens: 0,
            total_issued_lst: 0,
            total_reward_amount: 0,
        });

    // init first batch
    deps.storage
        .write_item::<CurrentPendingBatch>(&PendingBatch::new(
            BatchId::ONE,
            env.block
                .time
                .seconds()
                .checked_add(batch_period_seconds)
                .expect("overflow"),
        ));

    deps.storage.write_item::<Stopped>(&false);

    Ok(Response::new().add_event(
        Event::new("init")
            .add_attribute("admin", admin)
            .add_attribute("native_token_denom", native_token_denom)
            .add_attribute(
                "minimum_liquid_stake_amount",
                minimum_liquid_stake_amount.to_string(),
            )
            .add_attribute(
                "protocol_fee_rate",
                protocol_fee_config.fee_rate.to_string(),
            )
            .add_attribute("protocol_fee_recipient", protocol_fee_config.fee_recipient)
            .add_attribute(
                "current_unbonding_period",
                unbonding_period_seconds.to_string(),
            )
            .add_attribute("staker_address", staker_address)
            .add_attribute("lst_address", lst_address),
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Bond {
            mint_to_address,
            min_mint_amount,
        } => bond(deps, info, mint_to_address, min_mint_amount.u128()),
        ExecuteMsg::Unbond { amount } => unbond(deps, env, info, amount.u128()),
        ExecuteMsg::SubmitBatch {} => submit_batch(deps, env),
        ExecuteMsg::Withdraw {
            batch_id,
            withdraw_to_address,
        } => withdraw(deps, info, batch_id, withdraw_to_address),
        ExecuteMsg::TransferOwnership { new_owner } => {
            transfer_ownership(deps, env, info, new_owner)
        }
        ExecuteMsg::AcceptOwnership {} => accept_ownership(deps, env, info),
        ExecuteMsg::RevokeOwnershipTransfer {} => revoke_ownership_transfer(deps, info),
        ExecuteMsg::UpdateConfig {
            protocol_fee_config,
            monitors,
            batch_period_seconds,
            unbonding_period_seconds,
        } => update_config(
            deps,
            info,
            protocol_fee_config,
            monitors,
            batch_period_seconds,
            unbonding_period_seconds,
        ),
        ExecuteMsg::ReceiveRewards {} => receive_rewards(deps, info),
        ExecuteMsg::ReceiveUnstakedTokens { batch_id } => {
            receive_unstaked_tokens(deps, env, info, batch_id)
        }
        ExecuteMsg::CircuitBreaker {} => circuit_breaker(deps, info),
        ExecuteMsg::ResumeContract {
            total_bonded_native_tokens,
            total_issued_lst,
            total_reward_amount,
        } => resume_contract(
            deps,
            info,
            AccountingState {
                total_bonded_native_tokens: total_bonded_native_tokens.u128(),
                total_issued_lst: total_issued_lst.u128(),
                total_reward_amount: total_reward_amount.u128(),
            },
        ),
        ExecuteMsg::SlashBatches { new_amounts } => slash_batches(deps, info, new_amounts),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    Ok(match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?)?,
        QueryMsg::AccountingState {} => to_json_binary(&query_state(deps)?)?,
        QueryMsg::Batch { batch_id } => to_json_binary(&query_batch(deps, batch_id)?)?,
        QueryMsg::SubmittedBatches { start_after, limit } => {
            to_json_binary(&query_batches::<SubmittedBatches>(
                deps,
                start_after,
                limit,
            )?)?
        }
        QueryMsg::ReceivedBatches { start_after, limit } => {
            to_json_binary(&query_batches::<ReceivedBatches>(deps, start_after, limit)?)?
        }
        QueryMsg::BatchesByIds { batch_ids } => {
            to_json_binary(&query_batches_by_ids(deps, &batch_ids)?)?
        }
        QueryMsg::PendingBatch {} => to_json_binary(&query_pending_batch(deps)?)?,
        QueryMsg::UnstakeRequestsByStaker { staker } => {
            to_json_binary(&query_unstake_requests(deps, &staker)?)?
        }
        QueryMsg::AllUnstakeRequests { start_after, limit } => {
            to_json_binary(&query_all_unstake_requests(deps, start_after, limit)?)?
        }
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MigrateMsg {}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    env: Env,
    msg: UpgradeMsg<InitMsg, MigrateMsg>,
) -> Result<Response, ContractError> {
    msg.run(
        deps,
        |deps, msg| {
            let res = init(deps, env, msg)?;
            Ok((res, None))
        },
        |_, _, _| Ok((Response::default(), None)),
    )
}
