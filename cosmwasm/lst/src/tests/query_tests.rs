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

use std::fmt::Debug;

use cosmwasm_std::{
    Addr, Decimal, Deps, Uint128, from_json,
    testing::{mock_dependencies, mock_env},
};
use depolama::StorageExt;
use serde::de::DeserializeOwned;

use crate::{
    contract::{init, query},
    error::ContractError,
    msg::{
        AccountingStateResponse, Batch, BatchesResponse, ConfigResponse, IdentifiedBatch, QueryMsg,
    },
    query::query_batches_by_ids,
    state::{
        AccountingStateStore, ReceivedBatches, SubmittedBatches, UnstakeRequests,
        UnstakeRequestsByStakerHash,
    },
    tests::test_helper::{
        FEE_RECIPIENT, LST_ADDRESS, NATIVE_TOKEN, STAKER_ADDRESS, UNION_MONITOR_1, UNION_MONITOR_2,
        UNION1, UNION2, mock_init_msg, set_rewards, setup,
    },
    types::{
        AccountingState, BatchId, PendingBatch, ProtocolFeeConfig, ReceivedBatch, SubmittedBatch,
        UnstakeRequest, UnstakeRequestKey, staker_hash,
    },
};

#[track_caller]
fn assert_query_result<T: Debug + PartialEq + DeserializeOwned>(
    deps: Deps,
    msg: QueryMsg,
    expected: &T,
) {
    let res = query(deps, mock_env(), msg).unwrap();
    assert_eq!(&from_json::<T>(res).unwrap(), expected);
}

#[test]
fn state() {
    let mut deps = setup();

    assert_query_result(
        deps.as_ref(),
        QueryMsg::AccountingState {},
        &AccountingStateResponse {
            total_assets: Uint128::zero(),
            total_shares: Uint128::zero(),
            total_reward_amount: Uint128::zero(),
            redemption_rate: Decimal::one(),
            purchase_rate: Decimal::one(),
        },
    );

    deps.storage
        .write_item::<AccountingStateStore>(&AccountingState {
            total_bonded_native_tokens: 400_000,
            total_issued_lst: 100_000,
            total_reward_amount: 100,
        });

    assert_query_result(
        deps.as_ref(),
        QueryMsg::AccountingState {},
        &AccountingStateResponse {
            total_assets: 400_000_u128.into(),
            total_shares: 100_000_u128.into(),
            total_reward_amount: 100_u128.into(),
            redemption_rate: Decimal::from_ratio(4_u128, 1_u128),
            purchase_rate: Decimal::percent(25),
        },
    );

    set_rewards(
        &mut deps.querier,
        [("validator1", 20_000), ("validator2", 80_000)],
    );

    assert_query_result(
        deps.as_ref(),
        QueryMsg::AccountingState {},
        &AccountingStateResponse {
            total_assets: 500_000_u128.into(),
            total_shares: 100_000_u128.into(),
            total_reward_amount: 100_u128.into(),
            redemption_rate: Decimal::from_ratio(5_u128, 1_u128),
            purchase_rate: Decimal::percent(20),
        },
    );
}

#[test]
fn batch() {
    let mut deps = mock_dependencies();
    let msg = mock_init_msg();

    let mut env = mock_env();
    env.block.time = Default::default();

    init(deps.as_mut(), env, msg.clone()).unwrap();

    assert_query_result(
        deps.as_ref(),
        QueryMsg::Batch {
            batch_id: BatchId::ONE,
        },
        &Some(Batch::Pending(PendingBatch {
            batch_id: BatchId::ONE,
            total_lst_to_burn: 0,
            unstake_requests_count: 0,
            submit_time: msg.batch_period_seconds,
        })),
    );

    assert_query_result(
        deps.as_ref(),
        QueryMsg::Batch {
            batch_id: BatchId::TWO,
        },
        &None::<Batch>,
    );
}

#[test]
fn pending_batch() {
    let mut deps = mock_dependencies();
    let msg = mock_init_msg();

    let mut env = mock_env();
    env.block.time = Default::default();

    init(deps.as_mut(), env, msg.clone()).unwrap();

    assert_query_result(
        deps.as_ref(),
        QueryMsg::PendingBatch {},
        &PendingBatch {
            batch_id: BatchId::ONE,
            total_lst_to_burn: 0,
            unstake_requests_count: 0,
            submit_time: msg.batch_period_seconds,
        },
    );
}

#[test]
fn batches() {
    let mut deps = mock_dependencies();
    let msg = mock_init_msg();

    let mut env = mock_env();
    env.block.time = Default::default();

    init(deps.as_mut(), env, msg.clone()).unwrap();

    assert_query_result(
        deps.as_ref(),
        QueryMsg::BatchesByIds {
            batch_ids: vec![BatchId::ONE],
        },
        &BatchesResponse {
            batches: vec![IdentifiedBatch {
                batch_id: BatchId::ONE,
                batch: Batch::Pending(PendingBatch {
                    batch_id: BatchId::ONE,
                    total_lst_to_burn: 0,
                    unstake_requests_count: 0,
                    submit_time: msg.batch_period_seconds,
                }),
            }],
        },
    );

    assert_eq!(
        query_batches_by_ids(deps.as_ref(), &[BatchId::ONE, BatchId::TWO]).unwrap_err(),
        ContractError::BatchNotFound {
            batch_id: BatchId::TWO
        }
    );

    deps.storage.write::<SubmittedBatches>(
        &BatchId::TWO,
        &SubmittedBatch {
            total_lst_to_burn: 0,
            unstake_requests_count: 0,
            receive_time: 0,
            expected_native_unstaked: 0,
        },
    );

    assert_query_result(
        deps.as_ref(),
        QueryMsg::Batch {
            batch_id: BatchId::TWO,
        },
        &Some(Batch::Submitted(SubmittedBatch {
            total_lst_to_burn: 0,
            unstake_requests_count: 0,
            receive_time: 0,
            expected_native_unstaked: 0,
        })),
    );

    deps.storage.write::<ReceivedBatches>(
        &BatchId::THREE,
        &ReceivedBatch {
            total_lst_to_burn: 0,
            unstake_requests_count: 0,
            received_native_unstaked: 0,
        },
    );

    assert_eq!(
        query_batches_by_ids(deps.as_ref(), &[BatchId::from_raw(4).unwrap()]).unwrap_err(),
        ContractError::BatchNotFound {
            batch_id: BatchId::from_raw(4).unwrap()
        }
    );

    assert_query_result(
        deps.as_ref(),
        QueryMsg::Batch {
            batch_id: BatchId::THREE,
        },
        &Some(Batch::Received(ReceivedBatch {
            total_lst_to_burn: 0,
            unstake_requests_count: 0,
            received_native_unstaked: 0,
        })),
    );

    assert_query_result(
        deps.as_ref(),
        QueryMsg::BatchesByIds {
            batch_ids: vec![BatchId::ONE, BatchId::TWO, BatchId::THREE],
        },
        &BatchesResponse {
            batches: vec![
                IdentifiedBatch {
                    batch_id: BatchId::ONE,
                    batch: Batch::Pending(PendingBatch {
                        batch_id: BatchId::ONE,
                        total_lst_to_burn: 0,
                        unstake_requests_count: 0,
                        submit_time: msg.batch_period_seconds,
                    }),
                },
                IdentifiedBatch {
                    batch_id: BatchId::TWO,
                    batch: Batch::Submitted(SubmittedBatch {
                        total_lst_to_burn: 0,
                        unstake_requests_count: 0,
                        receive_time: 0,
                        expected_native_unstaked: 0,
                    }),
                },
                IdentifiedBatch {
                    batch_id: BatchId::THREE,
                    batch: Batch::Received(ReceivedBatch {
                        total_lst_to_burn: 0,
                        unstake_requests_count: 0,
                        received_native_unstaked: 0,
                    }),
                },
            ],
        },
    );

    assert_query_result(
        deps.as_ref(),
        QueryMsg::SubmittedBatches {
            start_after: None,
            limit: None,
        },
        &BatchesResponse {
            batches: vec![IdentifiedBatch {
                batch_id: BatchId::TWO,
                batch: SubmittedBatch {
                    total_lst_to_burn: 0,
                    unstake_requests_count: 0,
                    receive_time: 0,
                    expected_native_unstaked: 0,
                },
            }],
        },
    );

    assert_query_result(
        deps.as_ref(),
        QueryMsg::ReceivedBatches {
            start_after: None,
            limit: None,
        },
        &BatchesResponse {
            batches: vec![IdentifiedBatch {
                batch_id: BatchId::THREE,
                batch: ReceivedBatch {
                    total_lst_to_burn: 0,
                    unstake_requests_count: 0,
                    received_native_unstaked: 0,
                },
            }],
        },
    );
}

#[test]
fn unstake_requests_by_user() {
    let mut deps = mock_dependencies();
    let msg = mock_init_msg();

    let mut env = mock_env();
    env.block.time = Default::default();

    init(deps.as_mut(), env, msg.clone()).unwrap();

    let staker = Addr::unchecked(UNION1);

    assert_query_result::<Vec<UnstakeRequest>>(
        deps.as_ref(),
        QueryMsg::UnstakeRequestsByStaker {
            staker: staker.clone(),
        },
        &vec![],
    );

    deps.storage.write::<UnstakeRequestsByStakerHash>(
        &UnstakeRequestKey {
            batch_id: BatchId::ONE,
            staker_hash: staker_hash(&staker),
        },
        &UnstakeRequest {
            batch_id: BatchId::ONE,
            staker: staker.to_string(),
            amount: 1,
        },
    );

    deps.storage.write::<UnstakeRequestsByStakerHash>(
        &UnstakeRequestKey {
            batch_id: BatchId::TWO,
            staker_hash: staker_hash(&staker),
        },
        &UnstakeRequest {
            batch_id: BatchId::TWO,
            staker: staker.to_string(),
            amount: 2,
        },
    );

    assert_query_result(
        deps.as_ref(),
        QueryMsg::UnstakeRequestsByStaker {
            staker: staker.clone(),
        },
        &vec![
            UnstakeRequest {
                batch_id: BatchId::ONE,
                staker: staker.to_string(),
                amount: 1,
            },
            UnstakeRequest {
                batch_id: BatchId::TWO,
                staker: staker.to_string(),
                amount: 2,
            },
        ],
    );
}

#[test]
fn all_unstake_requests() {
    let mut deps = setup();

    let staker_1 = Addr::unchecked(UNION1);
    let staker_2 = Addr::unchecked(UNION2);

    deps.storage.write::<UnstakeRequests>(
        &UnstakeRequestKey {
            batch_id: BatchId::ONE,
            staker_hash: staker_hash(&staker_1),
        },
        &UnstakeRequest {
            batch_id: BatchId::ONE,
            staker: staker_1.to_string(),
            amount: 1,
        },
    );

    deps.storage.write::<UnstakeRequests>(
        &UnstakeRequestKey {
            batch_id: BatchId::ONE,
            staker_hash: staker_hash(&staker_2),
        },
        &UnstakeRequest {
            batch_id: BatchId::ONE,
            staker: staker_2.to_string(),
            amount: 1,
        },
    );

    deps.storage.write::<UnstakeRequests>(
        &UnstakeRequestKey {
            batch_id: BatchId::TWO,
            staker_hash: staker_hash(&staker_2),
        },
        &UnstakeRequest {
            batch_id: BatchId::TWO,
            staker: staker_2.to_string(),
            amount: 2,
        },
    );

    // full iter
    assert_query_result(
        deps.as_ref(),
        QueryMsg::AllUnstakeRequests {
            limit: None,
            start_after: None,
        },
        &vec![
            UnstakeRequest {
                batch_id: BatchId::ONE,
                staker: staker_1.to_string(),
                amount: 1,
            },
            UnstakeRequest {
                batch_id: BatchId::ONE,
                staker: staker_2.to_string(),
                amount: 1,
            },
            UnstakeRequest {
                batch_id: BatchId::TWO,
                staker: staker_2.to_string(),
                amount: 2,
            },
        ],
    );

    // limit is 2, but there's only 1 unstake request after the specified key
    assert_query_result(
        deps.as_ref(),
        QueryMsg::AllUnstakeRequests {
            limit: Some(2),
            start_after: Some(UnstakeRequestKey {
                batch_id: BatchId::ONE,
                staker_hash: staker_hash(&staker_2),
            }),
        },
        &vec![UnstakeRequest {
            batch_id: BatchId::TWO,
            staker: staker_2.to_string(),
            amount: 2,
        }],
    );
}

#[test]
fn config() {
    let deps = setup();

    assert_query_result(
        deps.as_ref(),
        QueryMsg::Config {},
        &ConfigResponse {
            native_token_denom: NATIVE_TOKEN.to_owned(),
            minimum_liquid_stake_amount: 100_u128.into(),
            protocol_fee_config: ProtocolFeeConfig {
                fee_rate: 10000,
                fee_recipient: FEE_RECIPIENT.to_owned(),
            },
            monitors: vec![
                Addr::unchecked(UNION_MONITOR_1),
                Addr::unchecked(UNION_MONITOR_2),
            ],
            lst_address: Addr::unchecked(LST_ADDRESS),
            staker_address: Addr::unchecked(STAKER_ADDRESS),
            batch_period_seconds: 86400,
            unbonding_period_seconds: 1000000,
            stopped: false,
        },
    );
}
