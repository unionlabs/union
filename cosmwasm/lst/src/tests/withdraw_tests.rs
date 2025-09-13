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
    testing::{message_info, mock_env},
    Addr, Coin, CosmosMsg, StdError,
};
use depolama::StorageExt;

use crate::{
    contract::execute,
    error::ContractError,
    msg::ExecuteMsg,
    state::{AccountingStateStore, ReceivedBatches, UnstakeRequests},
    tests::test_helper::{setup, NATIVE_TOKEN, UNION1, UNION2, UNION3},
    types::{staker_hash, BatchId, ReceivedBatch, UnstakeRequest, UnstakeRequestKey},
};

#[test]
fn withdraw() {
    let mut deps = setup();

    deps.storage
        .upsert_item::<AccountingStateStore, StdError>(|s| {
            let mut s = s.unwrap();
            s.total_bonded_native_tokens = 300_000;
            s.total_issued_lst = 130_000;
            Ok(s)
        })
        .unwrap();

    let batch_id = BatchId::ONE;

    let staker_1 = Addr::unchecked(UNION1);

    deps.storage.write::<UnstakeRequests>(
        &UnstakeRequestKey {
            batch_id,
            staker_hash: staker_hash(&staker_1),
        },
        &UnstakeRequest {
            batch_id,
            staker: staker_1.to_string(),
            amount: 40_000,
        },
    );

    let staker_2 = Addr::unchecked(UNION2);

    deps.storage.write::<UnstakeRequests>(
        &UnstakeRequestKey {
            batch_id,
            staker_hash: staker_hash(&staker_2),
        },
        &UnstakeRequest {
            batch_id,
            staker: staker_2.to_string(),
            amount: 90_000,
        },
    );

    deps.storage.write::<ReceivedBatches>(
        &batch_id,
        &ReceivedBatch {
            total_lst_to_burn: 130_000,
            unstake_requests_count: 2,
            received_native_unstaked: 130_000,
        },
    );

    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked(UNION1), &[]),
        ExecuteMsg::Withdraw {
            // withdraw to a different address
            withdraw_to_address: Addr::unchecked(UNION3),
            batch_id,
        },
    )
    .unwrap();

    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
            to_address: UNION3.to_string(),
            amount: vec![Coin {
                denom: NATIVE_TOKEN.to_string(),
                amount: 40_000u128.into()
            }]
        })
    );

    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked(UNION2), &[]),
        ExecuteMsg::Withdraw {
            withdraw_to_address: Addr::unchecked(UNION2),
            batch_id,
        },
    )
    .unwrap();

    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
            to_address: UNION2.to_string(),
            amount: vec![Coin {
                denom: NATIVE_TOKEN.to_string(),
                amount: 90_000u128.into()
            }]
        })
    );
}

#[test]
fn non_existent_batch() {
    let mut deps = setup();

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(UNION1), &[]),
            ExecuteMsg::Withdraw {
                withdraw_to_address: Addr::unchecked(UNION1),
                batch_id: BatchId::TWO
            }
        )
        .unwrap_err(),
        ContractError::BatchNotYetReceived {
            batch_id: BatchId::TWO
        }
    );
}

#[test]
fn batch_not_yet_received() {
    let mut deps = setup();

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(UNION1), &[]),
            ExecuteMsg::Withdraw {
                withdraw_to_address: Addr::unchecked(UNION1),
                batch_id: BatchId::ONE
            }
        )
        .unwrap_err(),
        ContractError::BatchNotYetReceived {
            batch_id: BatchId::ONE
        }
    );
}

#[test]
fn no_request_in_batch() {
    let mut deps = setup();

    deps.storage.write::<ReceivedBatches>(
        &BatchId::ONE,
        &ReceivedBatch {
            total_lst_to_burn: 100,
            unstake_requests_count: 0,
            received_native_unstaked: 100,
        },
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(UNION1), &[]),
            ExecuteMsg::Withdraw {
                withdraw_to_address: Addr::unchecked(UNION1),
                batch_id: BatchId::ONE,
            }
        )
        .unwrap_err(),
        ContractError::NoRequestInBatch {
            batch_id: BatchId::ONE,
            staker: Addr::unchecked(UNION1),
        }
    );
}

// #[test]
// fn withdraw_slashing() {
//     let mut deps = init();
//     let env = mock_env();
//     let mut state = STATE.load(&deps.storage).unwrap();

//     state.total_liquid_stake_token = Uint128::from(130_000u128);
//     state.total_bonded_native_tokens = Uint128::from(300_000u128);
//     STATE.save(&mut deps.storage, &state).unwrap();

//     let mut pending_batch: Batch =
//         PendingBatch::new(1, Uint128::new(130_000), env.block.time.seconds() + 10_000);
//     new_unstake_request(
//         &mut deps.as_mut(),
//         "bob".to_string(),
//         1,
//         Uint128::from(40_000u128),
//     )
//     .unwrap();
//     new_unstake_request(
//         &mut deps.as_mut(),
//         "tom".to_string(),
//         1,
//         Uint128::from(90_000u128),
//     )
//     .unwrap();
//     let res = BATCHES.save(&mut deps.storage, 1, &pending_batch);
//     assert!(res.is_ok());

//     // batch ready
//     pending_batch.received_native_unstaked = Some(Uint128::new(990_000)); // slashing happened
//     pending_batch.state = crate::types::BatchState::Received;
//     let res = BATCHES.save(&mut deps.storage, 1, &pending_batch);
//     assert!(res.is_ok());

//     // success
//     let msg = ExecuteMsg::Withdraw {
//         batch_id: pending_batch.id,
//     };
//     let info = mock_info("bob", &[]);
//     let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
//     assert!(res.is_ok());
//     let messages = res.unwrap().messages;
//     assert_eq!(messages.len(), 2); // withdraw and redemption rate update

//     let msg = QueryMsg::UnstakeRequests {
//         user: Addr::unchecked("bob"),
//     };
//     let res = query(deps.as_ref(), env.clone(), msg);
//     assert!(res.is_ok());
//     let resp: Vec<UnstakeRequest> = from_json(res.unwrap()).unwrap();

//     assert!(resp.is_empty());

//     let config = CONFIG.load(&deps.storage).unwrap();
//     let coin = Coin {
//         denom: config.protocol_chain_config.native_token_denom.clone(),
//         amount: "304615".to_string(), //304615.384... = 304615
//     };

//     // check the MsgSend
//     let coins = vec![coin];
//     assert_eq!(
//         messages[0],
//         SubMsg {
//             id: 0,
//             msg: <MsgSend as Into<CosmosMsg>>::into(MsgSend {
//                 from_address: Addr::unchecked(MOCK_CONTRACT_ADDR).to_string(),
//                 to_address: "bob".to_string(),
//                 amount: coins,
//             }),
//             gas_limit: None,
//             reply_on: ReplyOn::Never,
//         }
//     );

//     // Tom withdraw
//     let msg = ExecuteMsg::Withdraw {
//         batch_id: pending_batch.id,
//     };
//     let info = mock_info("tom", &[]);
//     let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
//     assert!(res.is_ok());
//     let messages = res.unwrap().messages;
//     assert_eq!(messages.len(), 2); // withdraw and redemption/purchase rate update

//     let msg = QueryMsg::UnstakeRequests {
//         user: Addr::unchecked("tom"),
//     };
//     let res = query(deps.as_ref(), env.clone(), msg);
//     assert!(res.is_ok());
//     let resp: Vec<UnstakeRequest> = from_json(res.unwrap()).unwrap();

//     assert!(resp.is_empty());

//     let config = CONFIG.load(&deps.storage).unwrap();
//     let coin = Coin {
//         denom: config.protocol_chain_config.native_token_denom.clone(),
//         amount: "685384".to_string(), //685,384.615... = 685384
//     };

//     // check the MsgSend
//     let coins = vec![coin];
//     assert_eq!(
//         messages[0],
//         SubMsg {
//             id: 0,
//             msg: <MsgSend as Into<CosmosMsg>>::into(MsgSend {
//                 from_address: Addr::unchecked(MOCK_CONTRACT_ADDR).to_string(),
//                 to_address: "tom".to_string(),
//                 amount: coins,
//             }),
//             gas_limit: None,
//             reply_on: ReplyOn::Never,
//         }
//     );
// }

// #[test]
// fn fee_withdraw() {
//     let mut deps = init();
//     let env = mock_env();
//     let mut state = STATE.load(&deps.storage).unwrap();
//     state.total_fees = Uint128::from(1000u128);
//     STATE.save(&mut deps.storage, &state).unwrap();

//     let msg = ExecuteMsg::FeeWithdraw {
//         amount: Uint128::from(2000u128),
//     };
//     let info = mock_info("bob", &[]);
//     let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
//     assert!(res.is_err()); // because not admin

//     let info = mock_info(ADMIN, &[]);
//     let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
//     assert!(res.is_err()); // because too high amount

//     let msg = ExecuteMsg::FeeWithdraw {
//         amount: Uint128::from(1000u128),
//     };
//     let info = mock_info(ADMIN, &[]);
//     let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
//     assert!(res.is_ok());
//     assert_eq!(
//         res.unwrap().messages[0],
//         SubMsg {
//             id: 0,
//             msg: <MsgSend as Into<CosmosMsg>>::into(MsgSend {
//                 from_address: Addr::unchecked(MOCK_CONTRACT_ADDR).to_string(),
//                 to_address: OSMO1.to_string(),
//                 amount: vec![Coin {
//                     denom: NATIVE_TOKEN.to_string(),
//                     amount: "1000".to_string()
//                 }],
//             }),
//             gas_limit: None,
//             reply_on: ReplyOn::Never,
//         }
//     );
// }
