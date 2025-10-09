use std::num::NonZeroU32;

use cosmwasm_std::{
    Addr, Binary, Coin, DecCoin, Decimal256, DelegatorReward, Deps, DepsMut, DistributionMsg, Env,
    Event, MessageInfo, Response, StakingMsg, StdError, Uint128, Uint256, entry_point,
    to_json_binary, wasm_execute,
};
use cw_account::ensure_local_admin_or_self;
use cw_utils::{PaymentError, must_pay};
use depolama::StorageExt;
use frissitheto::{InitStateVersionError, UpgradeError, UpgradeMsg};
use lst::msg::{ConfigResponse, StakerExecuteMsg};

use crate::{
    msg::{ExecuteMsg, MigrateMsg, QueryMsg},
    state::{LstHub, Validators},
};

pub mod msg;
pub mod state;

#[cfg(test)]
mod tests;

pub fn ensure_lst_hub(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    // allow reentrant calls into this contract
    if info.sender != deps.storage.read_item::<LstHub>()? {
        Err(ContractError::OnlyLstHub {
            sender: info.sender.clone(),
        })
    } else {
        Ok(())
    }
}

#[entry_point]
pub fn instantiate(
    mut deps: DepsMut,
    _: Env,
    _: MessageInfo,
    msg: cw_account::msg::InitMsg,
) -> Result<Response, ContractError> {
    frissitheto::init_state_version(&mut deps, const { NonZeroU32::new(1).unwrap() })?;

    Ok(cw_account::init(deps, msg))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetLstHubAddress(address) => {
            ensure_local_admin_or_self(deps.as_ref(), &env, &info)?;

            deps.storage.write_item::<LstHub>(&address);

            Ok(Response::new()
                .add_event(Event::new("set_lst_hub_address").add_attribute("address", address)))
        }
        ExecuteMsg::SetValidators(validators) => {
            ensure_local_admin_or_self(deps.as_ref(), &env, &info)?;

            let validators = validators
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect();

            deps.storage.write_item::<Validators>(&validators);

            let total_shares = validators.values().try_fold(0_u128, |total, shares| {
                total
                    .checked_add(*shares)
                    .ok_or(ContractError::TooManyShares)
            })?;

            Ok(Response::new()
                .add_event(
                    Event::new("set_validators")
                        .add_attribute("total_shares", total_shares.to_string()),
                )
                .add_events(validators.into_iter().map(|(validator, shares)| {
                    Event::new("validator_configured")
                        .add_attribute("address", validator)
                        .add_attribute("shares", shares.to_string())
                        .add_attribute(
                            "weight",
                            Decimal256::from_ratio(shares, total_shares).to_string(),
                        )
                })))
        }
        ExecuteMsg::CwAccount(execute_msg) => {
            Ok(cw_account::execute(deps, env, info, execute_msg)?)
        }
        ExecuteMsg::Staker(StakerExecuteMsg::Stake {}) => {
            ensure_lst_hub(deps.as_ref(), &info)?;

            let native_token_denom = deps
                .querier
                .query_wasm_smart::<ConfigResponse>(
                    deps.storage.read_item::<LstHub>()?,
                    &lst::msg::QueryMsg::Config {},
                )?
                .native_token_denom;

            let amount_to_stake = must_pay(&info, &native_token_denom)?;

            let validators = deps.storage.read_item::<Validators>()?;

            let total_shares = validators.values().fold(0_u128, |total, shares| {
                total
                    .checked_add(*shares)
                    .expect("should never fail, checked before written; qed;")
            });

            let (native_token_denom, total_pending_rewards, withdraw_msgs) =
                withdraw_all_rewards(deps.as_ref(), &env)?;

            let total_amount_to_stake = amount_to_stake + total_pending_rewards;

            Ok(Response::new()
                .add_messages(withdraw_msgs)
                .add_messages(validators.into_iter().map(|(validator, shares)| {
                    StakingMsg::Delegate {
                        validator,
                        amount: Coin::new(
                            total_amount_to_stake.multiply_ratio(shares, total_shares),
                            &native_token_denom,
                        ),
                    }
                }))
                .add_event(
                    Event::new("stake")
                        .add_attribute("total", amount_to_stake)
                        .add_attribute("pending_rewards", total_pending_rewards),
                ))
        }
        ExecuteMsg::Staker(StakerExecuteMsg::Rebase {}) => {
            ensure_lst_hub(deps.as_ref(), &info)?;

            let (native_token_denom, total_pending_rewards, submsgs) =
                withdraw_all_rewards(deps.as_ref(), &env)?;

            // claim all pending rewards and then send them back to the lst hub
            Ok(Response::new()
                .add_messages(submsgs)
                .add_message(wasm_execute(
                    deps.storage.read_item::<LstHub>()?,
                    &lst::msg::ExecuteMsg::ReceiveRewards {},
                    vec![Coin::new(total_pending_rewards, native_token_denom)],
                )?)
                .add_event(
                    Event::new("rebase")
                        .add_attribute("restaked_rewards", total_pending_rewards.to_string()),
                ))
        }
    }
}

fn withdraw_all_rewards(
    deps: Deps,
    env: &Env,
) -> Result<(String, Uint128, impl Iterator<Item = DistributionMsg>), ContractError> {
    let native_token_denom = deps
        .querier
        .query_wasm_smart::<ConfigResponse>(
            deps.storage.read_item::<LstHub>()?,
            &lst::msg::QueryMsg::Config {},
        )?
        .native_token_denom;

    let delegation_total_rewards = deps
        .querier
        .query_delegation_total_rewards(env.contract.address.clone())?;

    // NOTE: Sum the individual rewards, *NOT* the total rewards
    // if there are rewards of 1.5 and 0.5, the total will be 2.0 (even after flooring it), but the *actual* claimable amount is floor(1.5) + floor(0.5), which is 1
    let total_pending_rewards = delegation_total_rewards
        .rewards
        .iter()
        .flat_map(|d| &d.reward)
        .filter_map(|DecCoin { denom, amount }| (denom == &native_token_denom).then_some(amount))
        // TODO: Add a test to ensure there are no rounding errors here
        // for example if we add 0.5 and 0.5, it should be 0, not 1
        .try_fold(Uint256::zero(), |a, b| a.checked_add(b.to_uint_floor()))
        .map(Uint128::try_from)
        .expect("too many tokens")
        .expect("too many tokens");

    Ok((
        native_token_denom.clone(),
        total_pending_rewards,
        delegation_total_rewards.rewards.into_iter().filter_map(
            move |DelegatorReward {
                      validator_address,
                      reward,
                      ..
                  }| {
                (reward
                    .iter()
                    .any(|DecCoin { denom, amount: _ }| denom == &native_token_denom.clone()))
                .then_some(DistributionMsg::WithdrawDelegatorReward {
                    validator: validator_address,
                })
            },
        ),
    ))
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Validators {} => Ok(to_json_binary(&deps.storage.read_item::<Validators>()?)?),
        QueryMsg::CwAccount(msg) => Ok(cw_account::query(deps, env, msg)?),
    }
}

#[entry_point]
pub fn migrate(
    deps: DepsMut,
    _: Env,
    msg: UpgradeMsg<cw_account::msg::InitMsg, MigrateMsg>,
) -> Result<Response, ContractError> {
    msg.run(
        deps,
        |deps, init_msg| {
            let res = cw_account::init(deps, init_msg);
            Ok((res, None))
        },
        |_, _, _| Ok((Response::default(), None)),
    )
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("migration error: {0}")]
    Migrate(#[from] UpgradeError),

    #[error("init state version error: {0}")]
    InitStateVersion(#[from] InitStateVersionError),

    #[error(transparent)]
    CwAccount(#[from] cw_account::ContractError),

    #[error(transparent)]
    Payment(#[from] PaymentError),

    #[error(
        "validator shares can not total to more than {} (u128::MAX)",
        u128::MAX
    )]
    TooManyShares,

    #[error("sender {sender} is not the lst hub")]
    OnlyLstHub { sender: Addr },
}
