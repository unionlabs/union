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
    Addr, Coin, CosmosMsg, Event, Timestamp, WasmMsg,
    testing::{message_info, mock_dependencies, mock_env},
    to_json_binary,
};
use cw20::Cw20ExecuteMsg;
use depolama::StorageExt;

use crate::{
    contract::{execute, init},
    error::ContractError,
    helpers::shares_to_assets,
    msg::ExecuteMsg,
    state::{AccountingStateStore, ConfigStore, CurrentPendingBatch, SubmittedBatches},
    tests::test_helper::{
        ADMIN, LST_ADDRESS, NATIVE_TOKEN, UNION1, UNION2, UNION3, mock_init_msg, set_rewards, setup,
    },
    types::{BatchId, PendingBatch, SubmittedBatch},
};

#[test]
fn submit_batch_works() {
    let mut deps = setup();

    // UNION1 bonds 1000 tokens
    let union1_bond_amount = 1000_u128;
    let union1_shares = 1000_u128;
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(
            &Addr::unchecked(UNION1),
            &[Coin {
                denom: NATIVE_TOKEN.into(),
                amount: union1_bond_amount.into(),
            }],
        ),
        ExecuteMsg::Bond {
            mint_to_address: Addr::unchecked(UNION1),
            min_mint_amount: union1_shares.into(),
        },
    )
    .unwrap();

    let pending_rewards = 500;
    set_rewards(&mut deps.querier, [("validator1", pending_rewards)]);

    // UNION2 bonds 2000 tokens
    let union2_bond_amount = 2000_u128;
    let union2_shares = 1333_u128;
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(
            &Addr::unchecked(UNION2),
            &[Coin {
                denom: NATIVE_TOKEN.into(),
                amount: union2_bond_amount.into(),
            }],
        ),
        ExecuteMsg::Bond {
            mint_to_address: Addr::unchecked(UNION2),
            min_mint_amount: union2_shares.into(),
        },
    )
    .unwrap();

    // UNION3 unbonds 500 tokens
    let union3_unbond_amount = 500_u128;
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked(UNION3), &[]),
        ExecuteMsg::Unbond {
            amount: union3_unbond_amount.into(),
        },
    )
    .unwrap();

    let state = deps.storage.read_item::<AccountingStateStore>().unwrap();

    // state before submitting the batch is as expected
    assert_eq!(state.total_bonded_native_tokens, 3000);
    assert_eq!(state.total_issued_lst, 2333);

    let batch = deps.storage.read_item::<CurrentPendingBatch>().unwrap();

    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(batch.submit_time + 1);

    let res = execute(
        deps.as_mut(),
        env.clone(),
        message_info(&Addr::unchecked(UNION1), &[]),
        ExecuteMsg::SubmitBatch {},
    )
    .unwrap();

    // the given unbond amount will be burnt
    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: LST_ADDRESS.into(),
            msg: to_json_binary(&Cw20ExecuteMsg::Burn {
                amount: union3_unbond_amount.into(),
            })
            .unwrap(),
            funds: vec![]
        })
    );

    // no further messages
    assert_eq!(res.messages.len(), 1);

    // event is emitted correctly
    assert_eq!(
        res.events,
        [Event::new("submit_batch")
            .add_attribute("batch_id", "1")
            .add_attribute("batch_total", "500")
            .add_attribute("expected_unstaked", "750")
            .add_attribute("current_unbonding_period", "1000000")],
    );

    // new pending batch is stored correctly
    assert_eq!(
        deps.storage.read_item::<CurrentPendingBatch>().unwrap(),
        PendingBatch::new(
            BatchId::TWO,
            env.block.time.seconds()
                + deps
                    .storage
                    .read_item::<ConfigStore>()
                    .unwrap()
                    .batch_period_seconds
        ),
    );

    let expected_native_unstaked = shares_to_assets(
        union1_bond_amount + union2_bond_amount + pending_rewards,
        union1_shares + union2_shares,
        union3_unbond_amount,
    );

    assert_eq!(expected_native_unstaked, 750);

    let state = deps.storage.read_item::<AccountingStateStore>().unwrap();

    // the rate is updated properly
    assert_eq!(state.total_bonded_native_tokens, 2250);
    assert_eq!(state.total_issued_lst, 1833);

    // batch is moved to the submitted batches storage
    let submitted_batch = deps
        .storage
        .read::<SubmittedBatches>(&BatchId::ONE)
        .unwrap();

    assert_eq!(
        submitted_batch,
        SubmittedBatch {
            total_lst_to_burn: batch.total_lst_to_burn,
            unstake_requests_count: batch.unstake_requests_count,
            receive_time: env.block.time.seconds() + 1_000_000,
            expected_native_unstaked: 750,
        }
    );
}

#[test]
fn empty_submit_batch() {
    let mut deps = setup();
    let mut env = mock_env();

    env.block.time = env.block.time.plus_seconds(
        deps.storage
            .read_item::<ConfigStore>()
            .unwrap()
            .batch_period_seconds,
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env,
            message_info(&Addr::unchecked(ADMIN), &[]),
            ExecuteMsg::SubmitBatch {},
        )
        .unwrap_err(),
        ContractError::NoUnstakeRequestsInCurrentBatch {
            batch_id: BatchId::ONE
        }
    );
}

#[test]
fn not_ready_submit_batch() {
    let mut deps = mock_dependencies();

    let mut env = mock_env();
    env.block.time = Default::default();

    init(deps.as_mut(), env.clone(), mock_init_msg()).unwrap();

    deps.storage
        .update_item::<AccountingStateStore, ContractError, _>(|s| {
            s.total_issued_lst = 100_000;
            s.total_bonded_native_tokens = 300_000;
            Ok(())
        })
        .unwrap();

    // not ready
    env.block.time = env.block.time.plus_seconds(
        deps.storage
            .read_item::<ConfigStore>()
            .unwrap()
            .batch_period_seconds
            - 1,
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env,
            message_info(&Addr::unchecked(ADMIN), &[]),
            ExecuteMsg::SubmitBatch {},
        )
        .unwrap_err(),
        ContractError::BatchNotReady {
            now: 86399,
            ready_at: 86400
        }
    );
}
