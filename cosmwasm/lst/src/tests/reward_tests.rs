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
    wasm_execute, Addr, BankMsg, Coin, CosmosMsg, Event,
};
use depolama::StorageExt;

use super::test_helper::UNION1;
use crate::{
    contract::execute,
    error::ContractError,
    msg::{ExecuteMsg, StakerExecuteMsg},
    state::AccountingStateStore,
    tests::test_helper::{mock_init_msg, setup, FEE_RECIPIENT, NATIVE_TOKEN, STAKER_ADDRESS},
    types::AccountingState,
};

#[test]
fn receive_rewards_works() {
    let mut deps = setup();

    let state = deps
        .storage
        .update_item::<AccountingStateStore, ContractError, _>(|s| {
            s.total_bonded_native_tokens = 1_100;
            s.total_issued_lst = 1_000;
            Ok(s.clone())
        })
        .unwrap();

    let reward_amount = 100u128;
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(
            &Addr::unchecked(UNION1),
            &[Coin {
                denom: NATIVE_TOKEN.into(),
                amount: reward_amount.into(),
            }],
        ),
        ExecuteMsg::ReceiveRewards {},
    )
    .unwrap();

    // fee will be 10 because our protocol fee config is 10%
    let fee = 10u128;

    // amount - fee must be sent back to the staker to be restaked
    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Wasm(
            wasm_execute(
                STAKER_ADDRESS.to_owned(),
                &StakerExecuteMsg::Stake {},
                coins(90, NATIVE_TOKEN)
            )
            .unwrap()
        ),
    );

    // fee must be sent to the recipient
    assert_eq!(
        res.messages[1].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: mock_init_msg().protocol_fee_config.fee_recipient,
            amount: vec![Coin {
                denom: NATIVE_TOKEN.into(),
                amount: fee.into()
            }]
        })
    );

    // the state must be updated correctly
    let new_state = deps.storage.read_item::<AccountingStateStore>().unwrap();
    assert_eq!(
        new_state.total_bonded_native_tokens,
        state.total_bonded_native_tokens + (reward_amount - fee)
    );
    assert_eq!(new_state.total_reward_amount, reward_amount);

    // the event must be emitted correctly
    assert_eq!(
        res.events[0],
        Event::new("receive_rewards")
            .add_attribute("amount", reward_amount.to_string())
            .add_attribute(
                "amount_after_protocol_fee",
                (reward_amount - fee).to_string(),
            )
            .add_attribute("protocol_fee", fee.to_string()),
    );
}

#[test]
fn receive_rewards_and_send_fees_to_fee_recipient() {
    let mut deps = setup();
    let env = mock_env();

    deps.storage
        .update_item::<AccountingStateStore, ContractError, _>(|state| {
            state.total_issued_lst = 100_000;
            state.total_bonded_native_tokens = 100_000;
            state.total_reward_amount = 0;
            Ok(())
        })
        .unwrap();

    let res = execute(
        deps.as_mut(),
        env.clone(),
        message_info(&Addr::unchecked(UNION1), &coins(100, NATIVE_TOKEN)),
        ExecuteMsg::ReceiveRewards {},
    )
    .unwrap();

    assert_eq!(res.messages.len(), 2);

    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Wasm(
            wasm_execute(
                STAKER_ADDRESS.to_owned(),
                &StakerExecuteMsg::Stake {},
                coins(90, NATIVE_TOKEN)
            )
            .unwrap()
        ),
    );

    assert_eq!(
        res.messages[1].msg,
        CosmosMsg::from(BankMsg::Send {
            to_address: FEE_RECIPIENT.to_owned(),
            amount: coins(10, NATIVE_TOKEN)
        })
    );

    assert_eq!(
        deps.storage.read_item::<AccountingStateStore>().unwrap(),
        AccountingState {
            total_bonded_native_tokens: 100_090,
            total_issued_lst: 100_000,
            total_reward_amount: 100
        }
    );
}

#[test]
fn receive_rewards_with_zero_fees_fails() {
    let mut deps = setup();
    let env = mock_env();

    deps.storage
        .update_item::<AccountingStateStore, ContractError, _>(|state| {
            state.total_issued_lst = 100_000;
            state.total_bonded_native_tokens = 100_000;
            state.total_reward_amount = 0;
            Ok(())
        })
        .unwrap();

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&Addr::unchecked(UNION1), &coins(3, NATIVE_TOKEN)),
            ExecuteMsg::ReceiveRewards {}
        )
        .unwrap_err(),
        ContractError::ComputedFeesAreZero {
            received_rewards: 3
        }
    );
}
