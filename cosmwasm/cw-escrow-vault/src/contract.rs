use std::slice;

use access_managed::{EnsureCanCallResult, handle_consume_scheduled_op_reply, state::Authority};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo, Reply, Response,
    StdError, StdResult, to_json_binary, wasm_execute,
};
use cw20::Cw20ExecuteMsg;
use depolama::StorageExt;
use frissitheto::{UpgradeError, UpgradeMsg};
use ibc_union_spec::path::commit_packets;
use serde::{Deserialize, Serialize};
use ucs03_solvable::{Solvable, SolverQuery};
use ucs03_zkgm::contract::{SOLVER_EVENT, SOLVER_EVENT_MARKET_MAKER_ATTR};
use unionlabs_primitives::{Bytes, encoding::HexPrefixed};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, FungibleLaneConfig, InstantiateMsg, QueryMsg, RestrictedExecuteMsg},
    state::{Admin, FungibleCounterparty, FungibleLane, IntentWhitelist, Zkgm},
};

/// Major state versions of this contract, used in the [`frissitheto`] migrations.
pub mod version {
    use std::num::NonZeroU32;

    /// Initial state of the contract. Access management is handled internally in this contract for specific endpoints.
    pub const INIT: NonZeroU32 = NonZeroU32::new(1).unwrap();

    /// Same as [`INIT`], except that access management is handled externally via [`access_managed`]. All storage in this contract relating to internally handled access management has been removed, and additional storages for [`access_managed`] have been added.
    ///
    /// This is the current latest state version of this contract.
    pub const MANAGED: NonZeroU32 = NonZeroU32::new(2).unwrap();

    /// The latest state version of this contract. Any new deployments will be init'd with this version and the corresponding state.
    pub const LATEST: NonZeroU32 = MANAGED;
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(_: DepsMut, _: Env, _: MessageInfo, _: ()) -> StdResult<Response> {
    panic!(
        "this contract cannot be instantiated directly, but must be migrated from an existing instantiated contract."
    );
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrateMsg {
    pub access_managed_init_msg: access_managed::InitMsg,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    _: Env,
    msg: UpgradeMsg<InstantiateMsg, MigrateMsg>,
) -> Result<Response, ContractError> {
    msg.run(
        deps,
        |mut deps, msg| {
            access_managed::init(deps.branch(), msg.access_managed_init_msg)?;

            deps.storage.write_item::<Zkgm>(&msg.zkgm);

            Ok((Response::new(), Some(version::LATEST)))
        },
        |mut deps, msg, version| match version {
            version::INIT => {
                access_managed::init(deps.branch(), msg.access_managed_init_msg)?;
                deps.storage.delete_item::<Admin>();
                Ok((Response::default(), Some(version::MANAGED)))
            }
            version::MANAGED => Ok((Response::default(), None)),
            _ => Err(UpgradeError::UnknownStateVersion(version).into()),
        },
    )
}

fn ensure_zkgm(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let admin = deps.storage.read_item::<Zkgm>()?;
    if info.sender != admin {
        return Err(ContractError::OnlyZkgm);
    }
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Solvable(Solvable::DoSolve {
            packet,
            order,
            path,
            caller: _,
            relayer,
            relayer_msg: _,
            intent,
        }) => {
            ensure_zkgm(deps.as_ref(), &info)?;

            if intent {
                let packet_hash = commit_packets(slice::from_ref(&packet));

                let whitelisted = deps
                    .storage
                    .read::<IntentWhitelist>(&packet_hash)
                    .unwrap_or(false);

                if !whitelisted {
                    return Err(ContractError::IntentMustBeWhitelisted);
                }

                deps.storage.delete::<IntentWhitelist>(&packet_hash);
            }

            let fungible_lane = deps
                .storage
                .maybe_read::<FungibleCounterparty>(&(
                    path,
                    packet.destination_channel_id,
                    order.base_token,
                ))?
                .ok_or_else(|| ContractError::LaneIsNotFungible {
                    channel_id: packet.destination_channel_id,
                })?;

            let quote_token = String::from_utf8(Vec::from(order.quote_token))
                .map_err(|_| ContractError::InvalidQuoteToken)?;

            if quote_token != fungible_lane.escrowed_denom {
                return Err(ContractError::InvalidFill {
                    quote_token,
                    escrowed_denom: fungible_lane.escrowed_denom,
                });
            }

            let mut messages = Vec::<CosmosMsg>::with_capacity(2);
            let mut push_transfer = |to, amount: u128| -> StdResult<()> {
                if amount != 0 {
                    if fungible_lane.is_cw20 {
                        messages.push(
                            wasm_execute(
                                fungible_lane.escrowed_denom.clone(),
                                &Cw20ExecuteMsg::Transfer {
                                    recipient: to,
                                    amount: amount.into(),
                                },
                                vec![],
                            )?
                            .into(),
                        );
                    } else {
                        messages.push(
                            BankMsg::Send {
                                to_address: to,
                                amount: vec![Coin::new(
                                    amount,
                                    fungible_lane.escrowed_denom.clone(),
                                )],
                            }
                            .into(),
                        );
                    }
                }
                Ok(())
            };

            let fee = order
                .base_amount
                .checked_sub(order.quote_amount)
                .ok_or_else(|| ContractError::BaseAmountMustCoverQuoteAmount)?;
            push_transfer(relayer.into(), fee.try_into().expect("impossible"))?;

            let receiver = deps
                .api
                .addr_validate(
                    str::from_utf8(order.receiver.as_ref())
                        .map_err(|_| ContractError::InvalidReceiver)?,
                )
                .map_err(|_| ContractError::InvalidReceiver)?;
            push_transfer(
                receiver.into(),
                order.quote_amount.try_into().expect("impossible"),
            )?;

            Ok(Response::new().add_messages(messages).add_event(
                Event::new(SOLVER_EVENT).add_attribute(
                    SOLVER_EVENT_MARKET_MAKER_ATTR,
                    Bytes::<HexPrefixed>::from(fungible_lane.counterparty_beneficiary.to_vec())
                        .to_string(),
                ),
            ))
        }
        ExecuteMsg::AccessManaged(msg) => {
            access_managed::execute(deps, env, info, msg).map_err(Into::into)
        }
        ExecuteMsg::Restricted(msg) => {
            let msg = match msg.ensure_can_call::<Authority>(deps.branch(), &env, &info)? {
                EnsureCanCallResult::Msg(msg) => msg,
                EnsureCanCallResult::Scheduled(sub_msgs) => {
                    return Ok(Response::new().add_submessages(sub_msgs));
                }
            };

            match msg {
                RestrictedExecuteMsg::WhitelistIntents { hashes_whitelist } => {
                    for (packet_hash, allowed) in hashes_whitelist {
                        deps.storage
                            .write::<IntentWhitelist>(&packet_hash, &allowed);
                    }

                    Ok(Response::new())
                }
                RestrictedExecuteMsg::SetFungibleCounterparty {
                    path,
                    channel_id,
                    base_token,
                    counterparty_beneficiary,
                    escrowed_denom,
                } => {
                    let is_cw20 = deps
                        .querier
                        .query_wasm_contract_info(escrowed_denom.clone())
                        .is_ok();

                    deps.storage.write::<FungibleCounterparty>(
                        &(path, channel_id, base_token),
                        &FungibleLane {
                            counterparty_beneficiary,
                            escrowed_denom,
                            is_cw20,
                        },
                    );

                    Ok(Response::new())
                }
                RestrictedExecuteMsg::Upgradable(msg) => {
                    upgradable::execute(deps, env, info, msg).map_err(Into::into)
                }
            }
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::GetFungibleCounterparty {
            path,
            channel_id,
            base_token,
        } => Ok(to_json_binary(
            &deps
                .storage
                .maybe_read::<FungibleCounterparty>(&(path, channel_id, base_token))?,
        )?),
        QueryMsg::GetAllFungibleCounterparties => Ok(to_json_binary(
            &deps
                .storage
                .iter::<FungibleCounterparty>(cosmwasm_std::Order::Ascending)
                .map(|res| {
                    res.map(
                        |(
                            (path, channel_id, base_token),
                            FungibleLane {
                                counterparty_beneficiary,
                                escrowed_denom,
                                is_cw20,
                            },
                        )| FungibleLaneConfig {
                            path,
                            channel_id,
                            base_token,
                            counterparty_beneficiary,
                            escrowed_denom,
                            is_cw20,
                        },
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        )?),
        QueryMsg::Solvable(SolverQuery::IsSolver) => Ok(to_json_binary(&())?),
        QueryMsg::Solvable(SolverQuery::AllowMarketMakers) => Ok(to_json_binary(&true)?),
        QueryMsg::AccessManaged(msg) => access_managed::query(deps, env, msg).map_err(Into::into),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, _: Env, reply: Reply) -> Result<Response, ContractError> {
    if let Some(reply) = handle_consume_scheduled_op_reply(deps, reply)? {
        Err(StdError::generic_err(format!("unknown reply: {reply:?}")).into())
    } else {
        Ok(Response::new())
    }
}

#[cfg(test)]
mod tests {
    use access_managed::Restricted;
    use access_manager_types::{CanCall, managed::error::AccessManagedError};
    use cosmwasm_std::{
        Addr, ContractInfoResponse, ContractResult, Empty, OwnedDeps, QuerierResult, QueryRequest,
        SystemResult, WasmMsg, WasmQuery,
        testing::{
            MOCK_CONTRACT_ADDR, MockApi, MockQuerier, MockStorage, message_info, mock_dependencies,
            mock_env,
        },
    };
    use ibc_union_spec::{ChannelId, Packet, Timestamp};

    use super::*;

    const ZKGM_ADDR: &str = "zkgm";
    const ADMIN_ADDR: &str = "admin";
    const CALLER_ADDR: &str = "caller";

    pub const DESTINATION_CHANNEL_ID: ChannelId = ChannelId!(2);

    fn init(deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>) {
        let zkgm = Addr::unchecked(ZKGM_ADDR);

        migrate(
            deps.as_mut(),
            mock_env(),
            UpgradeMsg::Init(InstantiateMsg {
                zkgm: zkgm.clone(),
                access_managed_init_msg: access_managed::InitMsg {
                    initial_authority: Addr::unchecked("manager"),
                },
            }),
        )
        .unwrap();
    }

    fn mock_packet() -> Packet {
        Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: DESTINATION_CHANNEL_ID,
            data: Default::default(),
            timeout_height: ibc_union_spec::MustBeZero,
            timeout_timestamp: Timestamp::from_secs(10),
        }
    }

    fn mock_solve(base_amount: u128, quote_amount: u128, intent: bool) -> Solvable {
        Solvable::DoSolve {
            packet: mock_packet(),
            order: ucs03_solvable::CwTokenOrderV2 {
                sender: Default::default(),
                receiver: MOCK_CONTRACT_ADDR.as_bytes().into(),
                base_token: b"base_token".into(),
                base_amount: base_amount.into(),
                quote_token: b"muno".into(),
                quote_amount: quote_amount.into(),
                kind: 1,
                metadata: Default::default(),
            },
            path: 0u64.into(),
            caller: Addr::unchecked(CALLER_ADDR),
            relayer: Addr::unchecked(ZKGM_ADDR),
            relayer_msg: Default::default(),
            intent,
        }
    }

    fn solve(
        deps: DepsMut,
        base_amount: u128,
        quote_amount: u128,
        intent: bool,
    ) -> Result<Response, ContractError> {
        execute(
            deps,
            mock_env(),
            message_info(&Addr::unchecked(ZKGM_ADDR), &[]),
            ExecuteMsg::Solvable(mock_solve(base_amount, quote_amount, intent)),
        )
    }

    #[test]
    fn solve_successful() {
        let mut deps = mock_dependencies();

        deps.querier.update_wasm(|q: &WasmQuery| -> QuerierResult {
            match q {
                WasmQuery::Smart { contract_addr, .. } if contract_addr == "manager" => {
                    SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&CanCall {
                            allowed: true,
                            delay: 0,
                        })
                        .unwrap(),
                    ))
                }
                // delegate the rest to the default handler
                _ => <MockQuerier<Empty>>::new(&[]).handle_query(&QueryRequest::Wasm(q.clone())),
            }
        });

        init(&mut deps);

        assert_eq!(
            Err(ContractError::LaneIsNotFungible {
                channel_id: DESTINATION_CHANNEL_ID
            }),
            solve(deps.as_mut(), 150, 150, false)
        );

        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(ADMIN_ADDR), &[]),
            ExecuteMsg::Restricted(Restricted::wrap(
                RestrictedExecuteMsg::SetFungibleCounterparty {
                    path: 0u64.into(),
                    channel_id: DESTINATION_CHANNEL_ID,
                    base_token: b"base_token".into(),
                    counterparty_beneficiary: (&[0; 32]).into(),
                    escrowed_denom: "muno".into(),
                },
            )),
        )
        .unwrap();

        let res = solve(deps.as_mut(), 150, 150, false).unwrap();

        assert_eq!(
            res.events,
            vec![Event::new(SOLVER_EVENT).add_attribute(
                SOLVER_EVENT_MARKET_MAKER_ATTR,
                Bytes::<HexPrefixed>::from(vec![0; 32]).to_string(),
            )]
        );

        assert_eq!(
            res.messages[0].msg,
            BankMsg::Send {
                to_address: MOCK_CONTRACT_ADDR.into(),
                amount: vec![Coin::new(150u128, "muno")],
            }
            .into()
        );

        let res = solve(deps.as_mut(), 0, 0, false).unwrap();

        assert!(res.messages.is_empty());
    }

    #[test]
    fn solve_successful_with_cw20_fungible_lane() {
        let mut deps = mock_dependencies();

        deps.querier.update_wasm(|q: &WasmQuery| -> QuerierResult {
            match q {
                WasmQuery::Smart { contract_addr, .. } if contract_addr == "manager" => {
                    SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&CanCall {
                            allowed: true,
                            delay: 0,
                        })
                        .unwrap(),
                    ))
                }
                WasmQuery::ContractInfo { contract_addr } if contract_addr == "muno" => {
                    SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&ContractInfoResponse::new(
                            0,
                            Addr::unchecked(""),
                            None,
                            false,
                            None,
                        ))
                        .unwrap(),
                    ))
                }
                // delegate the rest to the default handler
                _ => <MockQuerier<Empty>>::new(&[]).handle_query(&QueryRequest::Wasm(q.clone())),
            }
        });

        init(&mut deps);

        assert_eq!(
            Err(ContractError::LaneIsNotFungible {
                channel_id: DESTINATION_CHANNEL_ID
            }),
            solve(deps.as_mut(), 150, 150, false)
        );

        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(ADMIN_ADDR), &[]),
            ExecuteMsg::Restricted(Restricted::wrap(
                RestrictedExecuteMsg::SetFungibleCounterparty {
                    path: 0u64.into(),
                    channel_id: DESTINATION_CHANNEL_ID,
                    base_token: b"base_token".into(),
                    counterparty_beneficiary: (&[0; 32]).into(),
                    escrowed_denom: "muno".into(),
                },
            )),
        )
        .unwrap();

        let res = solve(deps.as_mut(), 150, 150, false).unwrap();

        assert_eq!(
            res.messages[0].msg,
            WasmMsg::Execute {
                contract_addr: "muno".into(),
                msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: MOCK_CONTRACT_ADDR.into(),
                    amount: 150u128.into()
                })
                .unwrap(),
                funds: vec![],
            }
            .into()
        );
    }

    #[test]
    fn solve_with_excess_fee() {
        let mut deps = mock_dependencies();

        deps.querier.update_wasm(|q: &WasmQuery| -> QuerierResult {
            match q {
                WasmQuery::Smart { contract_addr, .. } if contract_addr == "manager" => {
                    SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&CanCall {
                            allowed: true,
                            delay: 0,
                        })
                        .unwrap(),
                    ))
                }
                // delegate the rest to the default handler
                _ => <MockQuerier<Empty>>::new(&[]).handle_query(&QueryRequest::Wasm(q.clone())),
            }
        });

        init(&mut deps);

        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(ADMIN_ADDR), &[]),
            ExecuteMsg::Restricted(Restricted::wrap(
                RestrictedExecuteMsg::SetFungibleCounterparty {
                    path: 0u64.into(),
                    channel_id: DESTINATION_CHANNEL_ID,
                    base_token: b"base_token".into(),
                    counterparty_beneficiary: (&[0; 32]).into(),
                    escrowed_denom: "muno".into(),
                },
            )),
        )
        .unwrap();

        let res = solve(deps.as_mut(), 150, 100, false).unwrap();

        assert_eq!(
            res.messages[0].msg,
            BankMsg::Send {
                to_address: ZKGM_ADDR.into(),
                amount: vec![Coin::new(50u128, "muno")],
            }
            .into()
        );

        assert_eq!(
            res.messages[1].msg,
            BankMsg::Send {
                to_address: MOCK_CONTRACT_ADDR.into(),
                amount: vec![Coin::new(100u128, "muno")],
            }
            .into()
        );
    }

    #[test]
    fn solve_with_intent() {
        let mut deps = mock_dependencies();

        deps.querier.update_wasm(|q: &WasmQuery| -> QuerierResult {
            match q {
                WasmQuery::Smart { contract_addr, .. } if contract_addr == "manager" => {
                    SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&CanCall {
                            allowed: true,
                            delay: 0,
                        })
                        .unwrap(),
                    ))
                }
                // delegate the rest to the default handler
                _ => <MockQuerier<Empty>>::new(&[]).handle_query(&QueryRequest::Wasm(q.clone())),
            }
        });

        init(&mut deps);

        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(ADMIN_ADDR), &[]),
            ExecuteMsg::Restricted(Restricted::wrap(
                RestrictedExecuteMsg::SetFungibleCounterparty {
                    path: 0u64.into(),
                    channel_id: DESTINATION_CHANNEL_ID,
                    base_token: b"base_token".into(),
                    counterparty_beneficiary: (&[0; 32]).into(),
                    escrowed_denom: "muno".into(),
                },
            )),
        )
        .unwrap();

        let commitment = commit_packets(&[mock_packet()]);

        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(ADMIN_ADDR), &[]),
            ExecuteMsg::Restricted(Restricted::wrap(RestrictedExecuteMsg::WhitelistIntents {
                hashes_whitelist: vec![(commitment, true)],
            })),
        )
        .unwrap();

        solve(deps.as_mut(), 150, 150, true).unwrap();
    }

    #[test]
    fn solve_fails_when_intent_not_whitelisted() {
        let mut deps = mock_dependencies();

        deps.querier.update_wasm(|q: &WasmQuery| -> QuerierResult {
            match q {
                WasmQuery::Smart { contract_addr, .. } if contract_addr == "manager" => {
                    SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&CanCall {
                            allowed: true,
                            delay: 0,
                        })
                        .unwrap(),
                    ))
                }
                // delegate the rest to the default handler
                _ => <MockQuerier<Empty>>::new(&[]).handle_query(&QueryRequest::Wasm(q.clone())),
            }
        });

        init(&mut deps);

        assert_eq!(
            Err(ContractError::IntentMustBeWhitelisted),
            solve(deps.as_mut(), 150, 150, true)
        );
    }

    #[test]
    fn solve_fails_when_fungible_lane_not_set() {
        let mut deps = mock_dependencies();

        deps.querier.update_wasm(|q: &WasmQuery| -> QuerierResult {
            match q {
                WasmQuery::Smart { contract_addr, .. } if contract_addr == "manager" => {
                    SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&CanCall {
                            allowed: true,
                            delay: 0,
                        })
                        .unwrap(),
                    ))
                }
                // delegate the rest to the default handler
                _ => <MockQuerier<Empty>>::new(&[]).handle_query(&QueryRequest::Wasm(q.clone())),
            }
        });

        init(&mut deps);

        assert_eq!(
            Err(ContractError::LaneIsNotFungible {
                channel_id: DESTINATION_CHANNEL_ID
            }),
            solve(deps.as_mut(), 150, 150, false)
        );
    }

    #[test]
    fn solve_fails_when_quote_token_is_wrong() {
        let mut deps = mock_dependencies();

        deps.querier.update_wasm(|q: &WasmQuery| -> QuerierResult {
            match q {
                WasmQuery::Smart { contract_addr, .. } if contract_addr == "manager" => {
                    SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&CanCall {
                            allowed: true,
                            delay: 0,
                        })
                        .unwrap(),
                    ))
                }
                // delegate the rest to the default handler
                _ => <MockQuerier<Empty>>::new(&[]).handle_query(&QueryRequest::Wasm(q.clone())),
            }
        });

        init(&mut deps);

        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(ADMIN_ADDR), &[]),
            ExecuteMsg::Restricted(Restricted::wrap(
                RestrictedExecuteMsg::SetFungibleCounterparty {
                    path: 0u64.into(),
                    channel_id: DESTINATION_CHANNEL_ID,
                    base_token: b"base_token".into(),
                    counterparty_beneficiary: (&[0; 32]).into(),
                    escrowed_denom: "not_muno".into(),
                },
            )),
        )
        .unwrap();

        assert_eq!(
            Err(ContractError::InvalidFill {
                quote_token: "muno".into(),
                escrowed_denom: "not_muno".into()
            }),
            solve(deps.as_mut(), 150, 150, false)
        );
    }

    #[test]
    fn solve_fails_when_base_amount_doesnt_cover_quote_amount() {
        let mut deps = mock_dependencies();

        deps.querier.update_wasm(|q: &WasmQuery| -> QuerierResult {
            match q {
                WasmQuery::Smart { contract_addr, .. } if contract_addr == "manager" => {
                    SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&CanCall {
                            allowed: true,
                            delay: 0,
                        })
                        .unwrap(),
                    ))
                }
                // delegate the rest to the default handler
                _ => <MockQuerier<Empty>>::new(&[]).handle_query(&QueryRequest::Wasm(q.clone())),
            }
        });

        init(&mut deps);

        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(ADMIN_ADDR), &[]),
            ExecuteMsg::Restricted(Restricted::wrap(
                RestrictedExecuteMsg::SetFungibleCounterparty {
                    path: 0u64.into(),
                    channel_id: DESTINATION_CHANNEL_ID,
                    base_token: b"base_token".into(),
                    counterparty_beneficiary: (&[0; 32]).into(),
                    escrowed_denom: "muno".into(),
                },
            )),
        )
        .unwrap();

        assert_eq!(
            Err(ContractError::BaseAmountMustCoverQuoteAmount),
            solve(deps.as_mut(), 100, 150, false)
        );
    }

    #[test]
    fn solve_fails_when_caller_is_not_zkgm() {
        let mut deps = mock_dependencies();

        deps.querier.update_wasm(|q: &WasmQuery| -> QuerierResult {
            match q {
                WasmQuery::ContractInfo { .. } => SystemResult::Ok(ContractResult::Ok(
                    to_json_binary(&ContractInfoResponse::new(
                        0,
                        Addr::unchecked(""),
                        None,
                        false,
                        None,
                    ))
                    .unwrap(),
                )),
                WasmQuery::Smart { .. } => SystemResult::Ok(ContractResult::Ok(
                    to_json_binary(&CanCall {
                        allowed: true,
                        delay: 0,
                    })
                    .unwrap(),
                )),
                msg => todo!("{msg:?}"),
            }
        });

        init(&mut deps);

        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(ADMIN_ADDR), &[]),
            ExecuteMsg::Restricted(Restricted::wrap(
                RestrictedExecuteMsg::SetFungibleCounterparty {
                    path: 0u64.into(),
                    channel_id: DESTINATION_CHANNEL_ID,
                    base_token: b"base_token".into(),
                    counterparty_beneficiary: (&[0; 32]).into(),
                    escrowed_denom: "muno".into(),
                },
            )),
        )
        .unwrap();

        let solve_msg = mock_solve(150, 150, false);

        assert_eq!(
            Err(ContractError::OnlyZkgm),
            execute(
                deps.as_mut(),
                mock_env(),
                message_info(&Addr::unchecked(ADMIN_ADDR), &[]),
                ExecuteMsg::Solvable(solve_msg),
            )
        );
    }

    #[test]
    fn set_fungible_counterparty_fails_when_not_admin() {
        let mut deps = mock_dependencies();

        deps.querier.update_wasm(|q: &WasmQuery| -> QuerierResult {
            match q {
                WasmQuery::Smart { contract_addr, .. } if contract_addr == "manager" => {
                    SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&CanCall {
                            allowed: false,
                            delay: 0,
                        })
                        .unwrap(),
                    ))
                }
                // delegate the rest to the default handler
                _ => <MockQuerier<Empty>>::new(&[]).handle_query(&QueryRequest::Wasm(q.clone())),
            }
        });

        init(&mut deps);

        assert_eq!(
            Err(ContractError::AccessManaged(
                access_managed::error::ContractError::AccessManaged(
                    AccessManagedError::AccessManagedUnauthorized {
                        caller: Addr::unchecked("zkgm")
                    }
                )
            )),
            execute(
                deps.as_mut(),
                mock_env(),
                message_info(&Addr::unchecked(ZKGM_ADDR), &[]),
                ExecuteMsg::Restricted(Restricted::wrap(
                    RestrictedExecuteMsg::SetFungibleCounterparty {
                        path: 0u64.into(),
                        channel_id: DESTINATION_CHANNEL_ID,
                        base_token: b"base_token".into(),
                        counterparty_beneficiary: (&[0; 32]).into(),
                        escrowed_denom: "muno".into(),
                    }
                )),
            )
        );
    }

    #[test]
    fn whitelist_admin_fails_when_not_admin() {
        let mut deps = mock_dependencies();

        deps.querier.update_wasm(|q: &WasmQuery| -> QuerierResult {
            match q {
                WasmQuery::Smart { contract_addr, .. } if contract_addr == "manager" => {
                    SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&CanCall {
                            allowed: false,
                            delay: 0,
                        })
                        .unwrap(),
                    ))
                }
                // delegate the rest to the default handler
                _ => <MockQuerier<Empty>>::new(&[]).handle_query(&QueryRequest::Wasm(q.clone())),
            }
        });

        init(&mut deps);

        assert_eq!(
            Err(ContractError::AccessManaged(
                access_managed::error::ContractError::AccessManaged(
                    AccessManagedError::AccessManagedUnauthorized {
                        caller: Addr::unchecked("zkgm")
                    }
                )
            )),
            execute(
                deps.as_mut(),
                mock_env(),
                message_info(&Addr::unchecked(ZKGM_ADDR), &[]),
                ExecuteMsg::Restricted(Restricted::wrap(RestrictedExecuteMsg::WhitelistIntents {
                    hashes_whitelist: vec![(Default::default(), true)],
                })),
            )
        );
    }
}
