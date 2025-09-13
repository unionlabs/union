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
    ensure, wasm_execute, Addr, BankMsg, Coin, Deps, DepsMut, Env, Event, MessageInfo, Response,
    Uint128,
};
use cw20::Cw20ExecuteMsg;
use cw_utils::must_pay;
use depolama::StorageExt;

use crate::{
    error::{ContractError, ContractResult},
    helpers::{compute_mint_amount, compute_unbond_amount},
    state::{
        AccountingStateStore, Admin, ConfigStore, CurrentPendingBatch, LstAddress, Monitors,
        PendingOwnerStore, ProtocolFeeConfigStore, ReceivedBatches, StakerAddress, Stopped,
        SubmittedBatches, UnstakeRequests, UnstakeRequestsByStakerHash,
    },
    types::{
        staker_hash, AccountingState, BatchExpectedAmount, BatchId, Config, PendingBatch,
        PendingOwner, ProtocolFeeConfig, ReceivedBatch, SubmittedBatch, UnstakeRequest,
        UnstakeRequestKey,
    },
};

pub const FEE_RATE_DENOMINATOR: u64 = 100_000;
/// 7 days
pub const OWNERSHIP_CLAIM_DELAY_PERIOD_SECONDS: u64 = 60 * 60 * 24 * 7;

pub fn ensure_stopped(deps: Deps) -> Result<(), ContractError> {
    if !deps.storage.read_item::<Stopped>()? {
        return Err(ContractError::NotStopped);
    }
    Ok(())
}

pub fn ensure_not_stopped(deps: Deps) -> Result<(), ContractError> {
    if deps.storage.read_item::<Stopped>()? {
        return Err(ContractError::Stopped);
    }
    Ok(())
}

pub fn ensure_admin(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    if deps.storage.read_item::<Admin>()? != info.sender {
        Err(ContractError::Unauthorized {
            sender: info.sender.clone(),
        })
    } else {
        Ok(())
    }
}

pub fn ensure_sender(info: &MessageInfo, staker: &Addr) -> Result<(), ContractError> {
    if info.sender != staker {
        Err(ContractError::Unauthorized {
            sender: info.sender.clone(),
        })
    } else {
        Ok(())
    }
}

/// 1. Ensure native tokens are provided.
/// 2. Ensure stake amount is >= minimum stake amount.
/// 3. Ensure minted LST amount is within slippage.
/// 4. Send funds to staker contract.
/// 5. Update state
pub fn bond(
    deps: DepsMut,
    info: MessageInfo,
    mint_to_address: Addr,
    min_mint_amount: u128,
) -> ContractResult<Response> {
    ensure_not_stopped(deps.as_ref())?;

    let config = deps.storage.read_item::<ConfigStore>()?;

    let bond_amount = must_pay(&info, &config.native_token_denom)?.u128();

    ensure!(
        bond_amount >= config.minimum_liquid_stake_amount,
        ContractError::MinimumLiquidStakeAmount {
            minimum_stake_amount: config.minimum_liquid_stake_amount,
            sent_amount: bond_amount,
        }
    );

    let mut accounting_state = deps.storage.read_item::<AccountingStateStore>()?;

    // Compute mint amount
    let mint_amount = compute_mint_amount(
        accounting_state.total_bonded_native_tokens,
        accounting_state.total_issued_lst,
        bond_amount,
    );

    // config.minimum_liquid_stake_amount *should* prevent this branch from being hit, however if
    // the ratio becomes very unbalanced then this may be hit
    ensure!(mint_amount > 0, ContractError::ComputedMintAmountIsZero);

    // update the accounting state first
    accounting_state.total_bonded_native_tokens = accounting_state
        .total_bonded_native_tokens
        .checked_add(bond_amount)
        .expect("overflow");
    accounting_state.total_issued_lst = accounting_state
        .total_issued_lst
        .checked_add(mint_amount)
        .expect("overflow");

    deps.storage
        .write_item::<AccountingStateStore>(&accounting_state);

    ensure!(
        mint_amount >= min_mint_amount,
        // slippage not met
        ContractError::SlippageNotMet {
            min_mint_amount,
            actual: mint_amount,
        }
    );

    let lst_address = deps.storage.read_item::<LstAddress>()?;

    // TODO: Emit staker hash
    let response = Response::new()
        // transfer native token to staker address
        .add_message(BankMsg::Send {
            to_address: deps.storage.read_item::<StakerAddress>()?.to_string(),
            amount: info.funds,
        })
        // send the minted lst tokens to recipient
        .add_message(wasm_execute(
            // eU address
            &lst_address,
            &Cw20ExecuteMsg::Mint {
                amount: mint_amount.into(),
                recipient: mint_to_address.to_string(),
            },
            vec![],
        )?)
        .add_event(
            Event::new("bond")
                .add_attribute("mint_to_address", mint_to_address.to_string())
                .add_attribute("sender", info.sender.to_string())
                .add_attribute("in_amount", bond_amount.to_string())
                .add_attribute("mint_amount", mint_amount.to_string()),
        );

    Ok(response)
}

/// Unbond the LST.
///
/// The LST is sent to this contract, and an unstaking request is added to the current batch. Once
/// the batch is submitted, [`withdraw`] can be called to withdraw the unstaked native token.
///
/// 1. Write the new unbond request to storage.
/// 2. Update the batch.
/// 3. Transfer the LST to this contract. Note that this requires an allowance to spend these tokens
///    on behalf of the staker.
///
/// # LST Balance Tracking
///
/// It should be noted that this contract does NOT track the balance of the LST. The LST contract
/// itself is expected to correctly track and maintain it's own balances. This prevents unbonding
/// more tokens than there are in total, since the TransferFrom call to the LST will fail.
pub fn unbond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    unbond_amount: u128,
) -> ContractResult<Response> {
    ensure_not_stopped(deps.as_ref())?;

    let staker_hash = staker_hash(&info.sender);

    let mut current_pending_batch = deps.storage.read_item::<CurrentPendingBatch>()?;

    // 1.
    let mut is_new_request = false;
    let unstake_request_key = UnstakeRequestKey {
        batch_id: current_pending_batch.batch_id,
        staker_hash,
    };
    let updated_unstake_request = deps.storage.upsert::<UnstakeRequests, ContractError>(
        &unstake_request_key,
        |maybe_unstake_request| {
            Ok(match maybe_unstake_request {
                Some(r) => {
                    assert_eq!(r.batch_id, current_pending_batch.batch_id);
                    assert_eq!(r.staker, info.sender.as_str());

                    UnstakeRequest {
                        batch_id: r.batch_id,
                        staker: r.staker,
                        amount: r.amount.checked_add(unbond_amount).expect("overflow"),
                    }
                }
                None => {
                    // this is a bit of a hack since .upsert() doesn't allow for returning anything
                    // else other than the T of the IndexMap. ideally it would return (R, T), write
                    // T to storage, and then return (R, T)
                    is_new_request = true;
                    UnstakeRequest {
                        batch_id: current_pending_batch.batch_id,
                        staker: info.sender.to_string(),
                        amount: unbond_amount,
                    }
                }
            })
        },
    )?;

    // duplicate this in storage for easy querying
    deps.storage
        .write::<UnstakeRequestsByStakerHash>(&unstake_request_key, &updated_unstake_request);

    current_pending_batch.total_lst_to_burn = current_pending_batch
        .total_lst_to_burn
        .checked_add(unbond_amount)
        .expect("overflow");

    if is_new_request {
        current_pending_batch.unstake_requests_count = current_pending_batch
            .unstake_requests_count
            .checked_add(1)
            .expect("overflow");
    }

    // 2.
    deps.storage
        .write_item::<CurrentPendingBatch>(&current_pending_batch);

    let response = Response::new()
        // 3.
        .add_message(wasm_execute(
            &deps.storage.read_item::<LstAddress>()?,
            &Cw20ExecuteMsg::TransferFrom {
                owner: info.sender.to_string(),
                recipient: env.contract.address.to_string(),
                amount: unbond_amount.into(),
            },
            vec![],
        )?)
        .add_event(
            Event::new("unbond")
                .add_attribute("staker", info.sender)
                .add_attribute("batch", current_pending_batch.batch_id.to_string())
                .add_attribute("amount", unbond_amount.to_string())
                .add_attribute("is_new_request", is_new_request.to_string()),
        );

    Ok(response)
}

/// Submit batch and transition pending batch to submitted.
///
/// TODO: Withdraw unstaked tokens in this function
/// TODO: Incentivize this call
pub fn submit_batch(deps: DepsMut, env: Env) -> ContractResult<Response> {
    ensure_not_stopped(deps.as_ref())?;

    let config = deps.storage.read_item::<ConfigStore>()?;

    let PendingBatch {
        batch_id,
        total_lst_to_burn,
        unstake_requests_count,
        submit_time,
    } = deps.storage.read_item::<CurrentPendingBatch>()?;

    ensure!(
        env.block.time.seconds() >= submit_time,
        ContractError::BatchNotReady {
            now: env.block.time.seconds(),
            ready_at: submit_time,
        }
    );

    ensure!(
        unstake_requests_count > 0,
        ContractError::NoUnstakeRequestsInCurrentBatch { batch_id }
    );

    let accounting_state = deps.storage.read_item::<AccountingStateStore>()?;

    // REVIEW: Is it possible to hit this case? It should not be possible to unbond more LST than
    // has been issued TODO: Make this an invariant error
    // reduce underlying LST balance by batch total
    // ensure we have more issued LST than LST we're trying to burn
    let new_total_issued_lst = match accounting_state
        .total_issued_lst
        .checked_sub(total_lst_to_burn)
    {
        Some(new_total_issued_lst) => new_total_issued_lst,
        None => {
            return Err(ContractError::InvalidUnstakeAmount {
                total_issued_lst: accounting_state.total_issued_lst,
                amount_to_unstake: total_lst_to_burn,
            });
        }
    };

    let unbond_amount = compute_unbond_amount(
        accounting_state.total_bonded_native_tokens,
        accounting_state.total_issued_lst,
        total_lst_to_burn,
    );

    deps.storage
        .write_item::<AccountingStateStore>(&AccountingState {
            // reduce underlying native token balance by unbonded amount
            total_bonded_native_tokens: accounting_state
                .total_bonded_native_tokens
                .checked_sub(unbond_amount)
                // REVIEW: Is it possible to hit this case?
                // TODO: Make this an invariant error
                .ok_or_else(|| ContractError::AttemptedToUnbondMoreThanBonded {
                    unbond_amount,
                    total_bonded_native_tokens: accounting_state.total_bonded_native_tokens,
                })?,
            total_issued_lst: new_total_issued_lst,
            ..accounting_state
        });

    // let unbonding_period =
    //     query_and_validate_unbonding_period(deps.as_ref(), config.batch_period_seconds)?;

    // save previously pending batch as submitted
    deps.storage.write::<SubmittedBatches>(
        &batch_id,
        &SubmittedBatch {
            total_lst_to_burn,
            unstake_requests_count,
            receive_time: env
                .block
                .time
                .seconds()
                .checked_add(config.unbonding_period_seconds)
                .expect("overflow"),
            expected_native_unstaked: unbond_amount,
        },
    );

    // finally, save new pending batch
    deps.storage
        .write_item::<CurrentPendingBatch>(&PendingBatch::new(
            batch_id.increment(),
            env.block
                .time
                .seconds()
                .checked_add(config.batch_period_seconds)
                .expect("overflow"),
        ));

    Ok(Response::new()
        // burn all unbonded LST tokens on batch submission
        .add_message(wasm_execute(
            deps.storage.read_item::<LstAddress>()?,
            &Cw20ExecuteMsg::Burn {
                amount: total_lst_to_burn.into(),
            },
            vec![],
        )?)
        .add_event(
            Event::new("submit_batch")
                .add_attribute("batch_id", batch_id.to_string())
                .add_attribute("batch_total", total_lst_to_burn.to_string())
                .add_attribute("expected_unstaked", unbond_amount.to_string())
                .add_attribute(
                    "current_unbonding_period",
                    config.unbonding_period_seconds.to_string(),
                ),
        ))
}

/// Receive rewards (denominated in the native token) to this contract.
///
/// The native token is the reward token for U<->eU. Anyone is able to call this entrypoint to
/// increase the backing of the LST (eU).
///
/// - Send native token to the contract
/// - Accrue (rewards, fees) based on the amount of rewards sent
// TODO: Incentivize this call
pub fn receive_rewards(deps: DepsMut, info: MessageInfo) -> ContractResult<Response> {
    ensure_not_stopped(deps.as_ref())?;

    let config = deps.storage.read_item::<ConfigStore>()?;

    let received_rewards = must_pay(&info, &config.native_token_denom)?.u128();

    let protocol_fee_config = deps.storage.read_item::<ProtocolFeeConfigStore>()?;

    let protocol_fee = Uint128::new(protocol_fee_config.fee_rate)
        .multiply_ratio(received_rewards, FEE_RATE_DENOMINATOR)
        .u128();

    if protocol_fee == 0 {
        return Err(ContractError::ComputedFeesAreZero { received_rewards });
    }

    // TODO: This branch is not possible, protocol_fee is < received_rewards
    let amount_after_protocol_fee =
        received_rewards.checked_sub(protocol_fee).ok_or_else(|| {
            ContractError::RewardsReceivedLessThanProtocolFee {
                received_rewards,
                protocol_fee,
            }
        })?;

    deps.storage
        .upsert_item::<AccountingStateStore, ContractError>(|accounting_state| {
            let mut accounting_state = accounting_state.expect("should exist");
            if accounting_state.total_issued_lst == 0 {
                return Err(ContractError::NoLiquidStake);
            }

            // update the accounting of tokens
            accounting_state.total_bonded_native_tokens = accounting_state
                .total_bonded_native_tokens
                .checked_add(amount_after_protocol_fee)
                .expect("overflow");
            accounting_state.total_reward_amount = accounting_state
                .total_reward_amount
                .checked_add(received_rewards)
                .expect("overflow");

            Ok(accounting_state)
        })?;

    Ok(Response::new()
        .add_event(
            Event::new("receive_rewards")
                .add_attribute("amount", received_rewards.to_string())
                .add_attribute(
                    "amount_after_protocol_fee",
                    amount_after_protocol_fee.to_string(),
                )
                .add_attribute("protocol_fee", protocol_fee.to_string()),
        )
        // send amount after fees to the staker
        .add_message(BankMsg::Send {
            to_address: deps.storage.read_item::<StakerAddress>()?.to_string(),
            amount: vec![Coin::new(
                amount_after_protocol_fee,
                &config.native_token_denom,
            )],
        })
        // send fees to the fee recipient
        .add_message(BankMsg::Send {
            to_address: protocol_fee_config.fee_recipient.to_string(),
            amount: vec![cosmwasm_std::Coin::new(
                protocol_fee,
                config.native_token_denom,
            )],
        }))
}

/// Marks a batch as received
/// Public function? Permissionless?
pub fn receive_unstaked_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    batch_id: BatchId,
) -> ContractResult<Response> {
    ensure_not_stopped(deps.as_ref())?;

    let config = deps.storage.read_item::<ConfigStore>()?;

    let received_native_amount = must_pay(&info, &config.native_token_denom)?.u128();

    let SubmittedBatch {
        total_lst_to_burn,
        unstake_requests_count,
        receive_time,
        expected_native_unstaked,
    } = deps
        .storage
        .maybe_read::<SubmittedBatches>(&batch_id)?
        .ok_or(ContractError::BatchNotFound { batch_id })?;
    deps.storage.delete::<SubmittedBatches>(&batch_id);

    ensure!(
        receive_time <= env.block.time.seconds(),
        ContractError::BatchNotReady {
            now: env.block.time.seconds(),
            ready_at: receive_time,
        }
    );

    ensure!(
        expected_native_unstaked == received_native_amount,
        ContractError::ReceivedWrongBatchAmount {
            batch_id,
            expected: expected_native_unstaked,
            received: received_native_amount,
        }
    );

    deps.storage.write::<ReceivedBatches>(
        &batch_id,
        &ReceivedBatch {
            total_lst_to_burn,
            unstake_requests_count,
            received_native_unstaked: received_native_amount,
        },
    );

    Ok(Response::new().add_event(
        Event::new("receive_unstaked_tokens")
            .add_attribute("batch", batch_id.to_string())
            .add_attribute("amount", received_native_amount.to_string()),
    ))
}

pub fn withdraw(
    deps: DepsMut,
    info: MessageInfo,
    batch_id: BatchId,
    withdraw_to_address: Addr,
) -> ContractResult<Response> {
    ensure_not_stopped(deps.as_ref())?;

    let config = deps.storage.read_item::<ConfigStore>()?;

    let ReceivedBatch {
        total_lst_to_burn,
        unstake_requests_count: _,
        received_native_unstaked,
    } = deps
        .storage
        .maybe_read::<ReceivedBatches>(&batch_id)?
        .ok_or(ContractError::BatchNotYetReceived { batch_id })?;

    let unstake_request_key = UnstakeRequestKey {
        batch_id,
        staker_hash: staker_hash(&info.sender),
    };
    let liquid_unstake_request = deps
        .storage
        .maybe_read::<UnstakeRequests>(&unstake_request_key)?
        .ok_or_else(|| ContractError::NoRequestInBatch {
            batch_id,
            staker: info.sender.clone(),
        })?;

    let amount = Uint128::new(received_native_unstaked)
        .multiply_ratio(liquid_unstake_request.amount, total_lst_to_burn)
        .u128();

    // delete unstake request from both maps
    deps.storage.delete::<UnstakeRequests>(&unstake_request_key);
    deps.storage
        .delete::<UnstakeRequestsByStakerHash>(&unstake_request_key);

    Ok(Response::new()
        .add_event(
            Event::new("withdraw")
                .add_attribute("staker", info.sender)
                .add_attribute("batch_id", batch_id.to_string())
                .add_attribute("withdraw_to_address", &withdraw_to_address)
                .add_attribute("amount", amount.to_string()),
        )
        // send the native token (U) back to the desired address
        .add_message(BankMsg::Send {
            to_address: withdraw_to_address.to_string(),
            amount: vec![Coin {
                denom: config.native_token_denom.clone(),
                amount: amount.into(),
            }],
        }))
}

// Transfer ownership to another account; callable by the owner
// This will require the new owner to accept to take effect.
// No need to handle case of overwriting the pending owner
// Ownership can only be claimed after 7 days to mitigate fat finger errors
pub fn transfer_ownership(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_owner: String,
) -> ContractResult<Response> {
    ensure_admin(deps.as_ref(), &info)?;

    deps.storage.write_item::<PendingOwnerStore>(&PendingOwner {
        address: new_owner.clone(),
        owner_transfer_min_time_seconds: env
            .block
            .time
            .seconds()
            .checked_add(OWNERSHIP_CLAIM_DELAY_PERIOD_SECONDS)
            .expect("overflow"),
    });

    Ok(Response::new().add_event(
        Event::new("transfer_ownership")
            .add_attribute("new_owner", new_owner)
            .add_attribute("previous_owner", info.sender),
    ))
}

// Revoke transfer ownership, callable by the owner
pub fn revoke_ownership_transfer(deps: DepsMut, info: MessageInfo) -> ContractResult<Response> {
    ensure_admin(deps.as_ref(), &info)?;

    deps.storage.delete_item::<PendingOwnerStore>();

    Ok(Response::new().add_event(Event::new("revoke_ownership_transfer")))
}

pub fn accept_ownership(deps: DepsMut, env: Env, info: MessageInfo) -> ContractResult<Response> {
    let PendingOwner {
        address: pending_owner,
        owner_transfer_min_time_seconds,
    } = deps
        .storage
        .maybe_read_item::<PendingOwnerStore>()?
        .ok_or(ContractError::NoPendingOwner)?;

    ensure!(
        owner_transfer_min_time_seconds <= env.block.time.seconds(),
        ContractError::OwnershipTransferNotReady {
            claimable_at_seconds: owner_transfer_min_time_seconds,
            now_seconds: env.block.time.seconds()
        }
    );

    if pending_owner == info.sender.as_str() {
        deps.storage.delete_item::<PendingOwnerStore>();
        deps.storage.write_item::<Admin>(&info.sender);
        Ok(Response::new()
            .add_event(Event::new("accept_ownership").add_attribute("new_owner", info.sender)))
    } else {
        Err(ContractError::CallerIsNotPendingOwner)
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    protocol_fee_config: Option<ProtocolFeeConfig>,
    monitors: Option<Vec<Addr>>,
    batch_period_seconds: Option<u64>,
    unbonding_period_seconds: Option<u64>,
) -> ContractResult<Response> {
    ensure_admin(deps.as_ref(), &info)?;

    let mut event = Event::new("update_config");

    if let Some(protocol_fee_config) = protocol_fee_config {
        deps.storage
            .write_item::<ProtocolFeeConfigStore>(&protocol_fee_config);

        event = event
            .add_attribute(
                "protocol_fee_rate",
                protocol_fee_config.fee_rate.to_string(),
            )
            .add_attribute(
                "protocol_fee_recpient",
                protocol_fee_config.fee_recipient.to_string(),
            );
    }

    if let Some(monitors) = monitors {
        // collect to Vec<String> as that's the type used in the store (and .join() can be called on
        // it as well to format it in the event)
        let monitors = monitors.into_iter().map(Into::into).collect();
        deps.storage.write_item::<Monitors>(&monitors);
        event = event.add_attribute("monitors", format!("[{}]", monitors.join(",")));
    }

    if let Some(batch_period_seconds) = batch_period_seconds {
        // let unbonding_period =
        //     query_and_validate_unbonding_period(deps.as_ref(), batch_period_seconds)?;

        deps.storage
            .upsert_item::<ConfigStore, ContractError>(|config| {
                Ok(Config {
                    batch_period_seconds,
                    ..config.expect("should exist")
                })
            })?;

        event = event.add_attribute("batch_period_seconds", batch_period_seconds.to_string());
        // .add_attribute("current_unbonding_period", unbonding_period.to_string());
    }

    if let Some(unbonding_period_seconds) = unbonding_period_seconds {
        deps.storage
            .upsert_item::<ConfigStore, ContractError>(|config| {
                Ok(Config {
                    unbonding_period_seconds,
                    ..config.expect("should exist")
                })
            })?;

        event = event.add_attribute(
            "unbonding_period_seconds",
            unbonding_period_seconds.to_string(),
        );
    }

    Ok(Response::new().add_event(event))
}

pub fn circuit_breaker(deps: DepsMut, info: MessageInfo) -> ContractResult<Response> {
    ensure_not_stopped(deps.as_ref())?;

    // must either be admin or a monitor to halt the contract
    if deps.storage.read_item::<Admin>()? == info.sender
        || deps
            .storage
            .read_item::<Monitors>()?
            .contains(&info.sender.to_string())
    {
        deps.storage.write_item::<Stopped>(&true);

        Ok(Response::new()
            .add_event(Event::new("circuit_breaker").add_attribute("breaker", info.sender)))
    } else {
        Err(ContractError::Unauthorized {
            sender: info.sender,
        })
    }
}

pub fn resume_contract(
    deps: DepsMut,
    info: MessageInfo,
    new_accounting_state: AccountingState,
) -> ContractResult<Response> {
    ensure_admin(deps.as_ref(), &info)?;
    ensure_stopped(deps.as_ref())?;

    deps.storage.write_item::<Stopped>(&false);

    deps.storage
        .write_item::<AccountingStateStore>(&new_accounting_state);

    Ok(Response::new().add_event(
        Event::new("resume_contract")
            .add_attribute(
                "total_bonded_native_tokens",
                new_accounting_state.total_bonded_native_tokens.to_string(),
            )
            .add_attribute(
                "total_issued_lst",
                new_accounting_state.total_issued_lst.to_string(),
            )
            .add_attribute(
                "total_reward_amount",
                new_accounting_state.total_reward_amount.to_string(),
            ),
    ))
}

pub fn slash_batches(
    deps: DepsMut,
    info: MessageInfo,
    expected_amounts: Vec<BatchExpectedAmount>,
) -> ContractResult<Response> {
    ensure_admin(deps.as_ref(), &info)?;

    // ensure the contract is stopped before slashing the batches
    ensure_stopped(deps.as_ref())?;

    for BatchExpectedAmount {
        batch_id,
        expected_native_amount,
    } in &expected_amounts
    {
        deps.storage
            .update::<SubmittedBatches, ContractError, _>(batch_id, |batch| {
                batch.expected_native_unstaked = *expected_native_amount;
                Ok(())
            })?;
    }

    Ok(Response::new().add_events(expected_amounts.into_iter().map(
        |BatchExpectedAmount {
             batch_id,
             expected_native_amount: amount,
         }| {
            Event::new("slash_batch")
                .add_attribute("batch_id", batch_id.to_string())
                .add_attribute("amount", amount.to_string())
        },
    )))
}
