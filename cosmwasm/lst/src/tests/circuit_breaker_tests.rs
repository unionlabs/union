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
    Addr, Event, Uint128,
    testing::{message_info, mock_env},
};
use depolama::StorageExt;

use crate::{
    contract::execute,
    error::ContractError,
    msg::ExecuteMsg,
    state::Stopped,
    tests::test_helper::{ADMIN, UNION_MONITOR_1, UNION1, ensure_execute_error, setup},
    types::BatchId,
};

#[test]
fn stop_works_as_admin() {
    let mut deps = setup();

    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked(ADMIN), &[]),
        ExecuteMsg::CircuitBreaker {},
    )
    .unwrap();

    assert!(deps.storage.read_item::<Stopped>().unwrap());

    assert_eq!(
        res.events,
        &[Event::new("circuit_breaker").add_attribute("breaker", ADMIN)]
    );
    assert!(res.attributes.is_empty());
    assert!(res.messages.is_empty());
}

#[test]
fn stop_works_as_monitor() {
    let mut deps = setup();

    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked(UNION_MONITOR_1), &[]),
        ExecuteMsg::CircuitBreaker {},
    )
    .unwrap();

    assert!(deps.storage.read_item::<Stopped>().unwrap());

    assert_eq!(
        res.events,
        &[Event::new("circuit_breaker").add_attribute("breaker", UNION_MONITOR_1)]
    );
    assert!(res.attributes.is_empty());
    assert!(res.messages.is_empty());
}

#[test]
fn stop_when_already_stopped_fails() {
    let mut deps = setup();

    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked(UNION_MONITOR_1), &[]),
        ExecuteMsg::CircuitBreaker {},
    )
    .unwrap();

    assert!(deps.storage.read_item::<Stopped>().unwrap());

    assert_eq!(
        res.events,
        &[Event::new("circuit_breaker").add_attribute("breaker", UNION_MONITOR_1)]
    );
    assert!(res.attributes.is_empty());
    assert!(res.messages.is_empty());
}

#[test]
fn stop_non_admin_or_monitor_fails() {
    let deps = setup();

    ensure_execute_error(
        deps.as_ref(),
        &mock_env(),
        &message_info(&Addr::unchecked(UNION1), &[]),
        ExecuteMsg::CircuitBreaker {},
        ContractError::Unauthorized {
            sender: Addr::unchecked(UNION1),
        },
    );
}

#[test]
fn stop_prevents_execution() {
    let mut deps = setup();

    let env = mock_env();

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&Addr::unchecked(UNION_MONITOR_1), &[]),
        ExecuteMsg::CircuitBreaker {},
    )
    .unwrap();

    let info = message_info(&Addr::unchecked(UNION1), &[]);

    let stoppable_msgs = [
        ExecuteMsg::Bond {
            mint_to_address: Addr::unchecked(UNION1),
            min_mint_amount: Uint128::zero(),
        },
        ExecuteMsg::Unbond {
            amount: Uint128::zero(),
        },
        ExecuteMsg::SubmitBatch {},
        ExecuteMsg::ReceiveRewards {},
        ExecuteMsg::Rebase {},
        ExecuteMsg::ReceiveUnstakedTokens {
            batch_id: BatchId::ONE,
        },
        ExecuteMsg::Withdraw {
            withdraw_to_address: Addr::unchecked(UNION1),
            batch_id: BatchId::ONE,
        },
        // can't stop again if already stopped
        ExecuteMsg::CircuitBreaker {},
    ];

    for msg in stoppable_msgs {
        ensure_execute_error(
            deps.as_ref(),
            &env.clone(),
            &info,
            msg,
            ContractError::Stopped,
        );
    }
}

#[test]
fn resume_resumes() {
    let mut deps = setup();

    let msg = ExecuteMsg::ResumeContract {
        total_bonded_native_tokens: 1_u128.into(),
        total_issued_lst: 2_u128.into(),
        total_reward_amount: 3_u128.into(),
    };

    ensure_execute_error(
        deps.as_ref(),
        &mock_env(),
        &message_info(&Addr::unchecked(ADMIN), &[]),
        msg.clone(),
        ContractError::NotStopped,
    );

    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked(ADMIN), &[]),
        ExecuteMsg::CircuitBreaker {},
    )
    .unwrap();

    assert!(deps.storage.read_item::<Stopped>().unwrap());

    ensure_execute_error(
        deps.as_ref(),
        &mock_env(),
        &message_info(&Addr::unchecked(UNION1), &[]),
        msg.clone(),
        ContractError::Unauthorized {
            sender: Addr::unchecked(UNION1),
        },
    );

    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked(ADMIN), &[]),
        msg,
    )
    .unwrap();

    assert!(!deps.storage.read_item::<Stopped>().unwrap());

    assert_eq!(
        res.events,
        &[Event::new("resume_contract")
            .add_attribute("total_bonded_native_tokens", "1")
            .add_attribute("total_issued_lst", "2")
            .add_attribute("total_reward_amount", "3")]
    );
    assert!(res.attributes.is_empty());
    assert!(res.messages.is_empty());
}
