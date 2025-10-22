use std::{collections::BTreeMap, num::NonZeroU32};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Addr, Binary, Coin, DecCoin, Decimal256, DelegatorReward, Deps, DepsMut, DistributionMsg, Env,
    Event, Int256, MessageInfo, Response, StakingMsg, StdError, Uint128, Uint256, to_json_binary,
    wasm_execute,
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _: Env,
    _: MessageInfo,
    msg: cw_account::msg::InitMsg,
) -> Result<Response, ContractError> {
    frissitheto::init_state_version(&mut deps, const { NonZeroU32::new(1).unwrap() })?;

    Ok(cw_account::init(deps, msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
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

            set_validators(deps, &env, validators)
        }
        ExecuteMsg::CwAccount(execute_msg) => {
            Ok(cw_account::execute(deps, env, info, execute_msg)?)
        }
        ExecuteMsg::Staker(StakerExecuteMsg::Stake {}) => {
            ensure_lst_hub(deps.as_ref(), &info)?;

            stake(deps.as_ref(), &env, &info)
        }
        ExecuteMsg::Staker(StakerExecuteMsg::Rebase {}) => {
            ensure_lst_hub(deps.as_ref(), &info)?;

            rebase(deps.as_ref(), &env)
        }
    }
}

fn set_validators(
    deps: DepsMut,
    env: &Env,
    validators: BTreeMap<Addr, Uint128>,
) -> Result<Response, ContractError> {
    let validators = validators
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect();

    let is_first = deps.storage.maybe_read_item::<Validators>()?.is_none();

    deps.storage.write_item::<Validators>(&validators);

    let total_shares = validators.values().try_fold(0_u128, |total, shares| {
        total
            .checked_add(*shares)
            .ok_or(ContractError::TooManyShares)
    })?;

    let mut response = Response::new()
        .add_event(
            Event::new("set_validators").add_attribute("total_shares", total_shares.to_string()),
        )
        .add_events(validators.iter().map(|(validator, shares)| {
            Event::new("validator_configured")
                .add_attribute("address", validator)
                .add_attribute("shares", shares.to_string())
                .add_attribute(
                    "weight",
                    Decimal256::from_ratio(*shares, total_shares).to_string(),
                )
        }));

    // if this isn't the first time setting the validators, rebase and rebalance the currently
    // staked amounts
    if !is_first {
        let rebase_response = rebase(deps.as_ref(), env)?;

        let native_token_denom = query_native_token_denom(deps.as_ref())?;

        let current_delegations = deps
            .querier
            .query_all_delegations(env.contract.address.clone())?
            .into_iter()
            .filter(|s| s.amount.denom == native_token_denom)
            .map(|s| (s.validator, s.amount.amount.u128()))
            .collect();

        let redelegation_msgs =
            redisribute_delegations(&native_token_denom, current_delegations, validators.clone())?;

        response = response
            .add_events(rebase_response.events)
            .add_submessages(rebase_response.messages)
            .add_messages(redelegation_msgs);
    }

    Ok(response)
}

fn stake(deps: Deps, env: &Env, info: &MessageInfo) -> Result<Response, ContractError> {
    let native_token_denom = query_native_token_denom(deps)?;

    let amount_to_stake = must_pay(info, &native_token_denom)?;

    let validators = deps.storage.read_item::<Validators>()?;

    let total_shares = validators.values().fold(0_u128, |total, shares| {
        total
            .checked_add(*shares)
            .expect("should never fail, checked before written; qed;")
    });

    let (native_token_denom, total_pending_rewards, withdraw_msgs) =
        withdraw_all_rewards(deps, env)?;

    let total_amount_to_stake = amount_to_stake + total_pending_rewards;

    Ok(Response::new()
        .add_messages(withdraw_msgs)
        .add_messages(
            validators
                .into_iter()
                .map(|(validator, shares)| StakingMsg::Delegate {
                    validator,
                    amount: Coin::new(
                        calculate_validator_delegation(total_amount_to_stake, shares, total_shares),
                        &native_token_denom,
                    ),
                }),
        )
        .add_event(
            Event::new("stake")
                .add_attribute("total", amount_to_stake)
                .add_attribute("pending_rewards", total_pending_rewards),
        ))
}

fn rebase(deps: Deps, env: &Env) -> Result<Response, ContractError> {
    let (native_token_denom, total_pending_rewards, submsgs) = withdraw_all_rewards(deps, env)?;

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

fn query_native_token_denom(deps: Deps) -> Result<String, ContractError> {
    Ok(deps
        .querier
        .query_wasm_smart::<ConfigResponse>(
            deps.storage.read_item::<LstHub>()?,
            &lst::msg::QueryMsg::Config {},
        )?
        .native_token_denom)
}

/// Given current delegations, redistribute the amounts to the new delegation set via MsgRedelegate.
///
/// Adapted from [Escher-finance/Escher-LST].
///
/// [Escher-finance/Escher-LST]: https://github.com/Escher-finance/Escher-LST/blob/506343c7f88879af203858265477fdbf49111aa9/cosmwasm/babylon-lst/contracts/liquidstaking/babylon/src/utils/delegation.rs#L279
fn redisribute_delegations(
    native_token_denom: &str,
    current_delegations: BTreeMap<String, u128>,
    new_validator_shares: BTreeMap<String, u128>,
) -> Result<Vec<StakingMsg>, ContractError> {
    let total_delegations = current_delegations
        .values()
        .try_fold(0_u128, |total, shares| {
            total
                .checked_add(*shares)
                .ok_or(ContractError::TooManyShares)
        })?;

    let total_shares = new_validator_shares
        .values()
        .try_fold(0_u128, |total, shares| {
            total
                .checked_add(*shares)
                .ok_or(ContractError::TooManyShares)
        })?;

    let mut diff = new_validator_shares
        .iter()
        .map(|(validator, shares)| {
            (
                validator,
                // all new delegations start off in a deficit
                -Int256::from(calculate_validator_delegation(
                    total_delegations,
                    *shares,
                    total_shares,
                )),
            )
        })
        .collect::<BTreeMap<_, _>>();

    // account for shares in the new set that already exist in the current set
    //
    // for example, if a validator has a delegation of 60 with it's current shares, and a delegation
    // of 100 with it's new shares, then it's in a deficit of 40. however, if a validator has more
    // delegation with it's current shares than with it's new shares, then the opposite is true;
    // with a current delegation of 100 and a new delegation of 60, then it's in a surplus of 40.
    for (validator, shares) in &current_delegations {
        *diff.entry(validator).or_default() += Int256::from(*shares);
    }

    let (mut surplus, mut deficient) = diff
        .into_iter()
        .partition::<BTreeMap<_, _>, _>(|(_, delegation)| delegation > &Int256::zero());

    let mut msgs = vec![];

    'outer: for (surplus_validator, surplus_amount) in &mut surplus {
        while *surplus_amount > Int256::zero() {
            let Some(mut deficient_validator) = deficient.first_entry() else {
                // no more deficient vals to redelegate to
                break 'outer;
            };

            let redelegate_amount = deficient_validator.get().abs().min(*surplus_amount);

            // SAFETY: redelegate_amount is min(surplus, abs(deficient))
            *deficient_validator.get_mut() += redelegate_amount;
            *surplus_amount -= redelegate_amount;

            msgs.push(StakingMsg::Redelegate {
                src_validator: (*surplus_validator).clone(),
                dst_validator: (**deficient_validator.key()).clone(),
                amount: Coin {
                    denom: native_token_denom.to_owned(),
                    amount: redelegate_amount
                        .try_into()
                        .map_err(|_| ContractError::TooManyDelegations)?,
                },
            });

            if deficient_validator.get().is_zero() {
                deficient_validator.remove();
            }
        }
    }

    Ok(msgs)
}

fn calculate_validator_delegation(
    total_amount_to_stake: impl Into<u128>,
    shares: impl Into<u128>,
    total_shares: impl Into<u128>,
) -> u128 {
    Uint128::new(Into::<u128>::into(total_amount_to_stake))
        .multiply_ratio(shares, total_shares)
        .u128()
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
    // if there are rewards of 1.5 and 0.5, the total will be 2.0 (even after flooring it), but the
    // *actual* claimable amount is floor(1.5) + floor(0.5), which is 1
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Validators {} => Ok(to_json_binary(&deps.storage.read_item::<Validators>()?)?),
        QueryMsg::CwAccount(msg) => Ok(cw_account::query(deps, env, msg)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
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

    #[error("validator delegation is more than {} (u128::MAX)", u128::MAX)]
    TooManyDelegations,

    #[error("sender {sender} is not the lst hub")]
    OnlyLstHub { sender: Addr },
}
