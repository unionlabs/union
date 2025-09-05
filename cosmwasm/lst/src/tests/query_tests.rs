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
    from_json,
    testing::{mock_dependencies, mock_env},
    Decimal, Deps, Storage, Uint128,
};
use depolama::StorageExt;
use hex_literal::hex;
use ibc_union_spec::ChannelId;
use serde::de::DeserializeOwned;
use unionlabs_primitives::{encoding::Base64, Bytes, U256};

use crate::{
    contract::{init, query},
    error::ContractError,
    msg::{AccountingStateResponse, Batch, BatchesResponse, IdentifiedBatch, QueryMsg},
    query::query_batches_by_ids,
    state::{AccountingStateStore, ReceivedBatches, SubmittedBatches, UnstakeRequestsByStakerHash},
    tests::test_helper::{mock_init_msg, setup, UNION1},
    types::{
        AccountingState, BatchId, PendingBatch, ReceivedBatch, Staker, SubmittedBatch,
        UnstakeRequest, UnstakeRequestKey,
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
            total_bonded_native_tokens: Uint128::zero(),
            total_issued_lst: Uint128::zero(),
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
            total_bonded_native_tokens: 400_000_u128.into(),
            total_issued_lst: 100_000_u128.into(),
            total_reward_amount: 100_u128.into(),
            redemption_rate: Decimal::from_ratio(4_u128, 1_u128),
            purchase_rate: Decimal::percent(25),
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
            batch_id: BatchId::ONE.increment(),
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
fn batches_by_ids() {
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
        query_batches_by_ids(deps.as_ref(), &[BatchId::ONE, BatchId::ONE.increment()]).unwrap_err(),
        ContractError::BatchNotFound {
            batch_id: BatchId::ONE.increment()
        }
    );

    deps.storage.write::<SubmittedBatches>(
        &BatchId::ONE.increment(),
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
            batch_id: BatchId::ONE.increment(),
        },
        &Some(Batch::Submitted(SubmittedBatch {
            total_lst_to_burn: 0,
            unstake_requests_count: 0,
            receive_time: 0,
            expected_native_unstaked: 0,
        })),
    );

    deps.storage.write::<ReceivedBatches>(
        &BatchId::ONE.increment().increment(),
        &ReceivedBatch {
            total_lst_to_burn: 0,
            unstake_requests_count: 0,
            received_native_unstaked: 0,
        },
    );

    assert_query_result(
        deps.as_ref(),
        QueryMsg::Batch {
            batch_id: BatchId::ONE.increment().increment(),
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
            batch_ids: vec![
                BatchId::ONE,
                BatchId::ONE.increment(),
                BatchId::ONE.increment().increment(),
            ],
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
                    batch_id: BatchId::ONE.increment(),
                    batch: Batch::Submitted(SubmittedBatch {
                        total_lst_to_burn: 0,
                        unstake_requests_count: 0,
                        receive_time: 0,
                        expected_native_unstaked: 0,
                    }),
                },
                IdentifiedBatch {
                    batch_id: BatchId::ONE.increment().increment(),
                    batch: Batch::Received(ReceivedBatch {
                        total_lst_to_burn: 0,
                        unstake_requests_count: 0,
                        received_native_unstaked: 0,
                    }),
                },
            ],
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

    let staker = Staker::Local {
        address: UNION1.to_owned(),
    };

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
            staker_hash: staker.hash(),
        },
        &UnstakeRequest {
            batch_id: BatchId::ONE,
            staker: staker.clone(),
            amount: 1,
        },
    );

    deps.storage.write::<UnstakeRequestsByStakerHash>(
        &UnstakeRequestKey {
            batch_id: BatchId::ONE.increment(),
            staker_hash: staker.hash(),
        },
        &UnstakeRequest {
            batch_id: BatchId::ONE.increment(),
            staker: staker.clone(),
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
                staker: staker.clone(),
                amount: 1,
            },
            UnstakeRequest {
                batch_id: BatchId::ONE.increment(),
                staker: staker.clone(),
                amount: 2,
            },
        ],
    );
}

#[test]
fn all_unstake_requests() {
    let mut deps = mock_dependencies();

    deps.storage.set(
        &hex!("756E7374616B655F72657175657374730000000000000000018CEC68F17FE9B07AA92127C903A61C2FFB47FE91A66CA52A1624CEC2334674E9"),
        &"AQAAAAAAAAABAAAAFAAAAAAAAAAMbx0tRx2V8jmtcKDgl6Zc5f4AEhQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB4AAAAAAAAAAAAAAAAAAAA=".parse::<Bytes<Base64>>().unwrap()
    );

    deps.storage.set(
        &hex!("756E7374616B655F7265717565737473000000000000000001C193453EB2D0F78A20ACFB33EE486A3D90FCAF389048A9E2C93CA7E7110C533D"),
        &"AQAAAAAAAAABAAAAFAAAAAAAAAAGYncU8/F6cB9wdKEsAoR6XSykhxQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAGYAAAAAAAAAAAAAAAAAAAA=".parse::<Bytes<Base64>>().unwrap()
    );

    deps.storage.set(
        &hex!("756E7374616B655F72657175657374735F62795F7374616B65725F68617368008CEC68F17FE9B07AA92127C903A61C2FFB47FE91A66CA52A1624CEC2334674E90000000000000001"),
        &"AQAAAAAAAAABAAAAFAAAAAAAAAAMbx0tRx2V8jmtcKDgl6Zc5f4AEhQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB4AAAAAAAAAAAAAAAAAAAA=".parse::<Bytes<Base64>>().unwrap()
    );

    deps.storage.set(
        &hex!("756E7374616B655F72657175657374735F62795F7374616B65725F6861736800C193453EB2D0F78A20ACFB33EE486A3D90FCAF389048A9E2C93CA7E7110C533D0000000000000001"),
        &"AQAAAAAAAAABAAAAFAAAAAAAAAAGYncU8/F6cB9wdKEsAoR6XSykhxQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAGYAAAAAAAAAAAAAAAAAAAA=".parse::<Bytes<Base64>>().unwrap()
    );

    assert_query_result::<Vec<UnstakeRequest>>(
        deps.as_ref(),
        QueryMsg::AllUnstakeRequests {
            start_after: None,
            limit: None,
        },
        &vec![
            UnstakeRequest {
                batch_id: BatchId::ONE,
                staker: Staker::Remote {
                    address: hex!("0c6f1d2d471d95f239ad70a0e097a65ce5fe0012").into(),
                    channel_id: ChannelId!(20),
                    path: U256::ZERO,
                },
                amount: 30,
            },
            UnstakeRequest {
                batch_id: BatchId::ONE,
                staker: Staker::Remote {
                    address: hex!("06627714f3f17a701f7074a12c02847a5d2ca487").into(),
                    channel_id: ChannelId!(20),
                    path: U256::ZERO,
                },
                amount: 102,
            },
        ],
    );
}
