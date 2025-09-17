use cosmwasm_schema::cw_serde;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, wasm_execute, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, Event,
    MessageInfo, Response, StdResult,
};
use cw20::Cw20ExecuteMsg;
use depolama::StorageExt;
use frissitheto::UpgradeMsg;
use ibc_union_spec::path::commit_packets;
use ucs03_solvable::Solvable;
use ucs03_zkgm::contract::{SOLVER_EVENT, SOLVER_EVENT_MARKET_MAKER_ATTR};
use unionlabs_primitives::{encoding::HexPrefixed, Bytes};

use crate::{
    error::Error,
    msg::{ExecuteMsg, FungibleLaneConfig, InstantiateMsg, QueryMsg},
    state::{Admin, FungibleCounterparty, FungibleLane, IntentWhitelist, Zkgm},
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(_: DepsMut, _: Env, _: MessageInfo, _: ()) -> StdResult<Response> {
    panic!("this contract cannot be instantiated directly, but must be migrated from an existing instantiated contract.");
}

#[cw_serde]
pub struct MigrateMsg {}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    _: Env,
    msg: UpgradeMsg<InstantiateMsg, MigrateMsg>,
) -> Result<Response, Error> {
    msg.run(
        deps,
        |deps, msg| {
            deps.storage.write_item::<Admin>(&msg.admin);
            deps.storage.write_item::<Zkgm>(&msg.zkgm);
            Ok((Response::new(), None))
        },
        |_, _, _| Ok((Response::default(), None)),
    )
}

fn ensure_zkgm(deps: Deps, info: &MessageInfo) -> Result<(), Error> {
    let admin = deps.storage.read_item::<Zkgm>()?;
    if info.sender != admin {
        return Err(Error::OnlyZkgm);
    }
    Ok(())
}

fn ensure_admin(deps: Deps, info: &MessageInfo) -> Result<(), Error> {
    let admin = deps.storage.read_item::<Admin>()?;
    if info.sender != admin {
        return Err(Error::OnlyAdmin);
    }
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, Error> {
    match msg {
        ExecuteMsg::WhitelistIntents { hashes_whitelist } => {
            ensure_admin(deps.as_ref(), &info)?;
            for (packet_hash, allowed) in hashes_whitelist {
                deps.storage
                    .write::<IntentWhitelist>(&packet_hash, &allowed);
            }
            Ok(Response::new())
        }
        ExecuteMsg::SetFungibleCounterparty {
            path,
            channel_id,
            base_token,
            counterparty_beneficiary,
            escrowed_denom,
        } => {
            ensure_admin(deps.as_ref(), &info)?;
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
                let whitelisted = deps
                    .storage
                    .read::<IntentWhitelist>(&commit_packets(&[packet.clone()]))
                    .unwrap_or(false);
                if !whitelisted {
                    return Err(Error::IntentMustBeWhitelisted);
                }
            }

            let fungible_lane = deps
                .storage
                .maybe_read::<FungibleCounterparty>(&(
                    path,
                    packet.destination_channel_id,
                    order.base_token,
                ))?
                .ok_or_else(|| Error::LaneIsNotFungible {
                    channel_id: packet.destination_channel_id,
                })?;

            let quote_token = String::from_utf8(Vec::from(order.quote_token))
                .map_err(|_| Error::InvalidQuoteToken)?;

            if quote_token != fungible_lane.escrowed_denom {
                return Err(Error::InvalidFill {
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
                .ok_or_else(|| Error::BaseAmountMustCoverQuoteAmount)?;
            push_transfer(relayer.into(), fee.try_into().expect("impossible"))?;

            let receiver = deps
                .api
                .addr_validate(
                    str::from_utf8(order.receiver.as_ref()).map_err(|_| Error::InvalidReceiver)?,
                )
                .map_err(|_| Error::InvalidReceiver)?;
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
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsSolver => to_json_binary(&()),
        QueryMsg::AllowMarketMakers => to_json_binary(&true),
        QueryMsg::GetFungibleCounterparty {
            path,
            channel_id,
            base_token,
        } => deps
            .storage
            .maybe_read::<FungibleCounterparty>(&(path, channel_id, base_token))
            .and_then(|data| to_json_binary(&data)),
        QueryMsg::GetAllFungibleCounterparties => deps
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
            .collect::<Result<Vec<_>, _>>()
            .and_then(|data| to_json_binary(&data)),
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        testing::{message_info, mock_dependencies, mock_env, MOCK_CONTRACT_ADDR},
        Addr,
    };
    use ibc_union_spec::{ChannelId, Packet, Timestamp};

    use super::*;

    const ZKGM_ADDR: &str = "zkgm";
    const ADMIN_ADDR: &str = "admin";
    const CALLER_ADDR: &str = "caller";

    pub const DESTINATION_CHANNEL_ID: ChannelId = ChannelId!(2);

    fn init(deps: DepsMut) {
        let zkgm = Addr::unchecked(ZKGM_ADDR);
        migrate(
            deps,
            mock_env(),
            UpgradeMsg::Init(InstantiateMsg {
                zkgm: zkgm.clone(),
                admin: Addr::unchecked(ADMIN_ADDR),
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

    fn solve(
        deps: DepsMut,
        base_amount: u128,
        quote_amount: u128,
        intent: bool,
    ) -> Result<Response, Error> {
        let solve_msg = Solvable::DoSolve {
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
        };

        execute(
            deps,
            mock_env(),
            message_info(&Addr::unchecked(ZKGM_ADDR), &[]),
            ExecuteMsg::Solvable(solve_msg.clone()),
        )
    }

    #[test]
    fn solve_successfull() {
        let mut deps = mock_dependencies();

        init(deps.as_mut());

        assert_eq!(
            Err(Error::LaneIsNotFungible {
                channel_id: DESTINATION_CHANNEL_ID
            }),
            solve(deps.as_mut(), 150, 150, false)
        );

        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(ADMIN_ADDR), &[]),
            ExecuteMsg::SetFungibleCounterparty {
                path: 0u64.into(),
                channel_id: DESTINATION_CHANNEL_ID,
                base_token: b"base_token".into(),
                counterparty_beneficiary: (&[0; 32]).into(),
                escrowed_denom: "muno".into(),
            },
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
    fn solve_with_excess_fee() {
        let mut deps = mock_dependencies();

        init(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(ADMIN_ADDR), &[]),
            ExecuteMsg::SetFungibleCounterparty {
                path: 0u64.into(),
                channel_id: DESTINATION_CHANNEL_ID,
                base_token: b"base_token".into(),
                counterparty_beneficiary: (&[0; 32]).into(),
                escrowed_denom: "muno".into(),
            },
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

        init(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(ADMIN_ADDR), &[]),
            ExecuteMsg::SetFungibleCounterparty {
                path: 0u64.into(),
                channel_id: DESTINATION_CHANNEL_ID,
                base_token: b"base_token".into(),
                counterparty_beneficiary: (&[0; 32]).into(),
                escrowed_denom: "muno".into(),
            },
        )
        .unwrap();

        let commitment = commit_packets(&[mock_packet()]);

        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(ADMIN_ADDR), &[]),
            ExecuteMsg::WhitelistIntents {
                hashes_whitelist: vec![(commitment, true)],
            },
        )
        .unwrap();

        solve(deps.as_mut(), 150, 150, true).unwrap();
    }

    #[test]
    fn solve_fails_when_intent_not_whitelisted() {
        let mut deps = mock_dependencies();

        init(deps.as_mut());

        assert_eq!(
            Err(Error::IntentMustBeWhitelisted),
            solve(deps.as_mut(), 150, 150, true)
        );
    }

    #[test]
    fn solve_fails_when_fungible_lane_not_set() {
        let mut deps = mock_dependencies();

        init(deps.as_mut());

        assert_eq!(
            Err(Error::LaneIsNotFungible {
                channel_id: DESTINATION_CHANNEL_ID
            }),
            solve(deps.as_mut(), 150, 150, false)
        );
    }

    #[test]
    fn solve_fails_when_quote_token_is_wrong() {
        let mut deps = mock_dependencies();

        init(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(ADMIN_ADDR), &[]),
            ExecuteMsg::SetFungibleCounterparty {
                path: 0u64.into(),
                channel_id: DESTINATION_CHANNEL_ID,
                base_token: b"base_token".into(),
                counterparty_beneficiary: (&[0; 32]).into(),
                escrowed_denom: "not_muno".into(),
            },
        )
        .unwrap();

        assert_eq!(
            Err(Error::InvalidFill {
                quote_token: "muno".into(),
                escrowed_denom: "not_muno".into()
            }),
            solve(deps.as_mut(), 150, 150, false)
        );
    }

    #[test]
    fn solve_fails_when_base_amount_doesnt_cover_quote_amount() {
        let mut deps = mock_dependencies();

        init(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(ADMIN_ADDR), &[]),
            ExecuteMsg::SetFungibleCounterparty {
                path: 0u64.into(),
                channel_id: DESTINATION_CHANNEL_ID,
                base_token: b"base_token".into(),
                counterparty_beneficiary: (&[0; 32]).into(),
                escrowed_denom: "muno".into(),
            },
        )
        .unwrap();

        assert_eq!(
            Err(Error::BaseAmountMustCoverQuoteAmount),
            solve(deps.as_mut(), 100, 150, false)
        );
    }
}
