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
    coins,
    testing::{message_info, mock_env},
    to_json_binary, Addr, BankMsg, CosmosMsg, Decimal, Event, StdError, Uint128, Uint256, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use depolama::StorageExt;
use ibc_union_spec::ChannelId;
use on_zkgm_call_proxy::OnProxyOnZkgmCall;
use ucs03_zkgm::msg::OnZkgm;

use crate::{
    contract::execute,
    error::ContractError,
    msg::{ExecuteMsg, RemoteExecuteMsg},
    query::query_state,
    state::AccountingStateStore,
    tests::test_helper::{
        mock_init_msg, setup, ETH_SENDER1, ETH_SENDER2, LIQUID_STAKE_TOKEN_ADDRESS, NATIVE_TOKEN,
        ON_ZKGM_CALL_PROXY_ADDRESS, RELAYER, UNION1, UNION2, UNION_STAKER, ZKGM_ADDRESS,
    },
    types::BatchId,
};

#[test]
fn bond_local_works() {
    let mut deps = setup();
    let info = message_info(
        &Addr::unchecked(ON_ZKGM_CALL_PROXY_ADDRESS),
        &coins(1000, NATIVE_TOKEN),
    );
    let mint_amount = 1000u128.into();
    let msg = ExecuteMsg::Bond {
        mint_to_address: Addr::unchecked(UNION2),
        min_mint_amount: mint_amount,
    };

    let mut prev_state = deps.storage.read_item::<AccountingStateStore>().unwrap();

    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

    // the native funds are sent to the staker
    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: UNION_STAKER.into(),
            amount: info.funds.clone()
        })
    );

    // 1000 LST token is minted to the `mint_to` address.
    // the `mint_amount` is 1000, since no rewards have been processed yet.
    assert_eq!(
        res.messages[1].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: LIQUID_STAKE_TOKEN_ADDRESS.into(),
            msg: to_json_binary(&Cw20ExecuteMsg::Mint {
                recipient: UNION2.into(),
                amount: mint_amount
            })
            .unwrap(),
            funds: vec![]
        })
    );

    // there should be no further messages
    assert_eq!(res.messages.len(), 2);

    // the event is correct
    assert_eq!(
        res.events[0],
        Event::new("bond")
            .add_attribute("mint_to_address", UNION2)
            .add_attribute("sender", ON_ZKGM_CALL_PROXY_ADDRESS)
            .add_attribute("in_amount", mint_amount.to_string())
            .add_attribute("mint_amount", mint_amount.to_string())
    );

    let state = deps.storage.read_item::<AccountingStateStore>().unwrap();

    // state is properly adjusted
    assert_eq!(state.total_bonded_native_tokens, 1000);
    assert_eq!(state.total_issued_lst, 1000);

    prev_state.total_bonded_native_tokens = 1000;
    prev_state.total_issued_lst = 1000;

    // there is no further state change
    assert_eq!(state, prev_state);

    // manually changing the rate instead of going through the `rewards` entrypoint
    deps.storage
        .upsert::<AccountingStateStore, StdError>(&(), |s| {
            let mut s = s.expect("exists");
            s.total_bonded_native_tokens += 100;
            Ok(s)
        })
        .unwrap();

    let res = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::Bond {
            mint_to_address: Addr::unchecked(UNION2),
            min_mint_amount: 700u128.into(),
        }
        .clone(),
    )
    .unwrap();

    // Since this is a local call, there will be no slippage and all payments will be done to
    // the sender. Hence, the calculated 909 is paid to the sender.
    assert_eq!(
        res.messages[1].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: LIQUID_STAKE_TOKEN_ADDRESS.into(),
            msg: to_json_binary(&Cw20ExecuteMsg::Mint {
                recipient: UNION2.into(),
                amount: 909u128.into()
            })
            .unwrap(),
            funds: vec![]
        })
    );

    // no further messages (no slippage payment)
    assert_eq!(res.messages.len(), 2);
}

#[test]
fn bond_with_slippage_updates_issued_supply_including_relayer_mint() {
    let mut deps = setup();

    let info = message_info(
        &Addr::unchecked(ON_ZKGM_CALL_PROXY_ADDRESS),
        &coins(1000, NATIVE_TOKEN), // bond_amount = 1000 U
    );
    // bonded = 1100, issued = 1000  => compute_mint_amount = floor(1000 * 1000 / 1100) = 909
    deps.storage
        .upsert::<AccountingStateStore, cosmwasm_std::StdError>(&(), |s| {
            let mut s = s.expect("exists");
            s.total_bonded_native_tokens = 1100;
            s.total_issued_lst = 1000;
            Ok(s)
        })
        .unwrap();

    // min_mint_amount is 700
    // - users mint: 700
    // - slippage: 909 - 700 = 209 (relayers)
    let min_mint_amount = 700u128.into();

    let msg = ExecuteMsg::OnProxyOnZkgmCall(OnProxyOnZkgmCall {
        on_zkgm_msg: OnZkgm {
            caller: Addr::unchecked(ZKGM_ADDRESS),
            path: Uint256::zero(),
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(2),
            sender: ETH_SENDER1.into(),
            message: Default::default(),
            relayer: Addr::unchecked(RELAYER),
            relayer_msg: Default::default(),
        },
        msg: to_json_binary(&ExecuteMsg::Bond {
            mint_to_address: Addr::unchecked(UNION2),
            min_mint_amount,
        })
        .unwrap(),
    });

    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    // funds go to the staker address
    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: UNION_STAKER.to_string(),
            amount: coins(1000, NATIVE_TOKEN)
        })
    );
    assert_eq!(
        res.messages[1].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: LIQUID_STAKE_TOKEN_ADDRESS.into(),
            msg: to_json_binary(&Cw20ExecuteMsg::Mint {
                recipient: UNION2.into(),
                amount: 700u128.into(),
            })
            .unwrap(),
            funds: vec![]
        })
    );
    assert_eq!(
        res.messages[2].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: LIQUID_STAKE_TOKEN_ADDRESS.into(),
            msg: to_json_binary(&Cw20ExecuteMsg::Mint {
                recipient: RELAYER.into(),
                amount: 209u128.into(),
            })
            .unwrap(),
            funds: vec![]
        })
    );

    let state = deps.storage.read_item::<AccountingStateStore>().unwrap();
    assert_eq!(state.total_bonded_native_tokens, 2100);
    assert_eq!(
        state.total_issued_lst, 1909,
        "total_issued_lst MUST include both user mint (700) and relayer slippage mint (209). \
        If this assertion fails, it means that in bond(), issued is only increased by 'mint_amount'."
    );
}

#[test]
fn bond_remote_works() {
    let mut deps = setup();
    let info = message_info(
        &Addr::unchecked(ON_ZKGM_CALL_PROXY_ADDRESS),
        &coins(1000, NATIVE_TOKEN),
    );
    let mint_amount = 1000u128.into();
    let mut prev_state = deps.storage.read_item::<AccountingStateStore>().unwrap();

    let res = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::OnProxyOnZkgmCall(OnProxyOnZkgmCall {
            on_zkgm_msg: OnZkgm {
                caller: Addr::unchecked(ZKGM_ADDRESS),
                path: Uint256::zero(),
                source_channel_id: ChannelId!(1),
                destination_channel_id: ChannelId!(1),
                sender: ETH_SENDER1.into(),
                message: Default::default(),
                relayer: Addr::unchecked(RELAYER),
                relayer_msg: Default::default(),
            },
            msg: to_json_binary(&ExecuteMsg::Bond {
                mint_to_address: Addr::unchecked(UNION2),
                min_mint_amount: mint_amount,
            })
            .unwrap(),
        }),
    )
    .unwrap();

    // the native funds are sent to the staker
    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: UNION_STAKER.into(),
            amount: info.funds.clone()
        })
    );

    // 1000 LST token is minted to the `mint_to` address.
    // the `mint_amount` is 1000, since no rewards have been processed yet.
    assert_eq!(
        res.messages[1].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: LIQUID_STAKE_TOKEN_ADDRESS.into(),
            msg: to_json_binary(&Cw20ExecuteMsg::Mint {
                recipient: UNION2.into(),
                amount: mint_amount
            })
            .unwrap(),
            funds: vec![]
        })
    );

    // there should be no further messages
    assert_eq!(res.messages.len(), 2);

    // the event is correct
    assert_eq!(
        res.events[0],
        Event::new("bond")
            .add_attribute("mint_to_address", UNION2)
            .add_attribute("sender", ON_ZKGM_CALL_PROXY_ADDRESS)
            .add_attribute("in_amount", mint_amount.to_string())
            .add_attribute("mint_amount", mint_amount.to_string())
    );

    let state = deps.storage.read_item::<AccountingStateStore>().unwrap();

    // state is properly adjusted
    assert_eq!(state.total_bonded_native_tokens, 1000);
    assert_eq!(state.total_issued_lst, 1000);

    prev_state.total_bonded_native_tokens = 1000;
    prev_state.total_issued_lst = 1000;

    // there is no further state change
    assert_eq!(state, prev_state);

    // manually changing the rate instead of going through the `rewards` entrypoint
    deps.storage
        .upsert::<AccountingStateStore, StdError>(&(), |s| {
            let mut s = s.expect("exists");
            s.total_bonded_native_tokens += 100;
            Ok(s)
        })
        .unwrap();

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::OnProxyOnZkgmCall(OnProxyOnZkgmCall {
                on_zkgm_msg: OnZkgm {
                    caller: Addr::unchecked(ZKGM_ADDRESS),
                    path: Uint256::zero(),
                    source_channel_id: ChannelId!(1),
                    destination_channel_id: ChannelId!(1),
                    sender: ETH_SENDER2.into(),
                    message: Default::default(),
                    relayer: Addr::unchecked(RELAYER),
                    relayer_msg: Default::default(),
                },
                msg: to_json_binary(&ExecuteMsg::Bond {
                    mint_to_address: Addr::unchecked(UNION2),
                    min_mint_amount: mint_amount,
                })
                .unwrap(),
            }),
        )
        .unwrap_err(),
        ContractError::SlippageNotMet {
            min_mint_amount: 1000,
            actual: 909,
        }
    );

    let res = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::OnProxyOnZkgmCall(OnProxyOnZkgmCall {
            on_zkgm_msg: OnZkgm {
                caller: Addr::unchecked(ZKGM_ADDRESS),
                path: Uint256::zero(),
                source_channel_id: ChannelId!(1),
                destination_channel_id: ChannelId!(1),
                sender: ETH_SENDER2.into(),
                message: Default::default(),
                relayer: Addr::unchecked(RELAYER),
                relayer_msg: Default::default(),
            },
            msg: to_json_binary(&RemoteExecuteMsg::Bond {
                mint_to_address: Addr::unchecked(UNION2),
                min_mint_amount: Uint128::new(500),
            })
            .unwrap(),
        }),
    )
    .unwrap();

    // since this is a remote call, the min slippage must be specified, and any slippage will be
    // paid to the relayer
    assert_eq!(
        res.messages[1].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: LIQUID_STAKE_TOKEN_ADDRESS.into(),
            msg: to_json_binary(&Cw20ExecuteMsg::Mint {
                recipient: UNION2.into(),
                amount: 500_u128.into()
            })
            .unwrap(),
            funds: vec![]
        })
    );

    assert_eq!(
        res.messages[2].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: LIQUID_STAKE_TOKEN_ADDRESS.into(),
            msg: to_json_binary(&Cw20ExecuteMsg::Mint {
                recipient: RELAYER.into(),
                amount: 409_u128.into()
            })
            .unwrap(),
            funds: vec![]
        })
    );

    // no further messages (no slippage payment)
    assert_eq!(res.messages.len(), 3);
}

#[test]
fn remote_execute_msg_remote_not_proxy_fails() {
    let mut deps = setup();
    let info = message_info(&Addr::unchecked(UNION1), &coins(1000, NATIVE_TOKEN));
    let mint_amount = 1000u128.into();

    let on_zkgm_msg = OnZkgm {
        caller: Addr::unchecked(ZKGM_ADDRESS),
        path: Uint256::zero(),
        source_channel_id: ChannelId!(1),
        destination_channel_id: ChannelId!(1),
        sender: ETH_SENDER1.into(),
        message: Default::default(),
        relayer: Addr::unchecked(RELAYER),
        relayer_msg: Default::default(),
    };

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::OnProxyOnZkgmCall(OnProxyOnZkgmCall {
                on_zkgm_msg: on_zkgm_msg.clone(),
                msg: to_json_binary(&RemoteExecuteMsg::Bond {
                    mint_to_address: Addr::unchecked(UNION2),
                    min_mint_amount: mint_amount,
                })
                .unwrap(),
            }),
        )
        .unwrap_err(),
        ContractError::Unauthorized {
            sender: Addr::unchecked(UNION1)
        }
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::OnProxyOnZkgmCall(OnProxyOnZkgmCall {
                on_zkgm_msg: on_zkgm_msg.clone(),
                msg: to_json_binary(&RemoteExecuteMsg::Unbond {
                    amount: mint_amount,
                })
                .unwrap(),
            }),
        )
        .unwrap_err(),
        ContractError::Unauthorized {
            sender: Addr::unchecked(UNION1)
        }
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::OnProxyOnZkgmCall(OnProxyOnZkgmCall {
                on_zkgm_msg: on_zkgm_msg.clone(),
                msg: to_json_binary(&RemoteExecuteMsg::Withdraw {
                    batch_id: BatchId::ONE,
                    withdraw_to_address: Addr::unchecked(UNION2),
                })
                .unwrap(),
            }),
        )
        .unwrap_err(),
        ContractError::Unauthorized {
            sender: Addr::unchecked(UNION1)
        }
    );
}

#[test]
fn bond_remote_with_slippage_works() {
    let mut deps = setup();
    let info = message_info(
        &Addr::unchecked(ON_ZKGM_CALL_PROXY_ADDRESS),
        &coins(1000, NATIVE_TOKEN),
    );
    let min_mint_amount = 700u128.into();
    let msg = ExecuteMsg::OnProxyOnZkgmCall(OnProxyOnZkgmCall {
        on_zkgm_msg: OnZkgm {
            caller: Addr::unchecked(UNION1),
            path: Uint256::zero(),
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(2),
            sender: ETH_SENDER1.to_vec().into(),
            // we are ignoring the message since it's only handled at the funded dispatch contract
            message: Default::default(),
            relayer: Addr::unchecked(UNION_STAKER),
            relayer_msg: Default::default(),
        },
        msg: to_json_binary(&ExecuteMsg::Bond {
            mint_to_address: Addr::unchecked(UNION2),
            min_mint_amount,
        })
        .unwrap(),
    });

    deps.storage
        .upsert::<AccountingStateStore, StdError>(&(), |s| {
            let mut s = s.expect("exists");
            s.total_bonded_native_tokens += 1100;
            s.total_issued_lst += 1000;
            Ok(s)
        })
        .unwrap();

    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

    // Since this is a local call, there will be no slippage and all payments will be done to
    // the sender. Hence, the calculated 909 is paid to the sender.
    assert_eq!(
        res.messages[1].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: LIQUID_STAKE_TOKEN_ADDRESS.into(),
            msg: to_json_binary(&Cw20ExecuteMsg::Mint {
                recipient: UNION2.into(),
                amount: min_mint_amount
            })
            .unwrap(),
            funds: vec![]
        })
    );

    // although the computed mint amount is 909, the `min_mint_amount` is paid to the user
    assert_eq!(
        res.messages[1].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: LIQUID_STAKE_TOKEN_ADDRESS.into(),
            msg: to_json_binary(&Cw20ExecuteMsg::Mint {
                recipient: UNION2.into(),
                amount: min_mint_amount
            })
            .unwrap(),
            funds: vec![]
        })
    );

    // and the rest 209 slippage is paid to the relayer
    assert_eq!(
        res.messages[2].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: LIQUID_STAKE_TOKEN_ADDRESS.into(),
            msg: to_json_binary(&Cw20ExecuteMsg::Mint {
                recipient: UNION_STAKER.into(),
                amount: 209u128.into()
            })
            .unwrap(),
            funds: vec![]
        })
    );

    // the event is correct
    assert_eq!(
        res.events[1],
        Event::new("bond_slippage_paid")
            .add_attribute("slippage", 209.to_string())
            .add_attribute("relayer", UNION_STAKER)
    );
}

#[test]
fn liquid_stake_less_than_minimum() {
    let mut deps = setup();
    let info = message_info(
        &Addr::unchecked(ON_ZKGM_CALL_PROXY_ADDRESS),
        &coins(
            mock_init_msg().minimum_liquid_stake_amount - 1,
            NATIVE_TOKEN,
        ),
    );
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Bond {
            mint_to_address: Addr::unchecked(UNION2),
            min_mint_amount: u128::MAX.into(),
        },
    )
    .unwrap_err();

    assert_eq!(
        err,
        ContractError::MinimumLiquidStakeAmount {
            minimum_stake_amount: mock_init_msg().minimum_liquid_stake_amount,
            sent_amount: mock_init_msg().minimum_liquid_stake_amount - 1
        }
    );
}

#[test]
fn slippage_not_met() {
    let mut deps = setup();
    let info = message_info(
        &Addr::unchecked(ON_ZKGM_CALL_PROXY_ADDRESS),
        &coins(1000, NATIVE_TOKEN),
    );

    // manually changing the rate instead of going through the `rewards` entrypoint
    deps.storage
        .upsert::<AccountingStateStore, StdError>(&(), |s| {
            let mut s = s.expect("exists");
            s.total_bonded_native_tokens += 10000;
            s.total_issued_lst += 1234;
            Ok(s)
        })
        .unwrap();

    assert_eq!(
        query_state(deps.as_ref()).unwrap().purchase_rate,
        "0.1234".parse::<Decimal>().unwrap()
    );

    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Bond {
            mint_to_address: Addr::unchecked(UNION2),
            min_mint_amount: 1000_u128.into(),
        },
    )
    .unwrap_err();

    assert_eq!(
        err,
        ContractError::SlippageNotMet {
            min_mint_amount: 1000,
            actual: 123
        }
    );
}
