use std::{collections::BTreeMap, str::FromStr as _, time::Duration};

use alloy::{network::AnyNetwork, primitives::Address, providers::DynProvider};
use cometbft_rpc::{rpc_types::TxResponse, types::code::Code};
use cosmwasm_std::{
    Addr, Binary, Coin as CwCoin, CosmosMsg, Decimal, Uint128, WasmMsg, to_json_binary,
};
use cw20::Cw20ExecuteMsg;
use lst::{
    msg::{AccountingStateResponse, Batch, ExecuteMsg as LstExecuteMsg},
    types::{BatchId, PendingBatch},
};
use protos::cosmos::{base::v1beta1::Coin as ProtoCoin, staking::v1beta1 as staking_proto};
use rand::RngCore;
use serde::{Serialize, de::DeserializeOwned};
use tracing::{info, warn};
use ucs03_zkgm::com::TAG_ACK_SUCCESS;
use union_test::{
    cosmos::{self},
    cosmos_helpers::calculate_proxy_address,
    evm::{
        self,
        zkgm::{Instruction as InstructionEvm, UCS03Zkgm},
    },
    zkgm_helper,
};
use unionlabs::primitives::U256;
use voyager_sdk::{anyhow, serde_json};

mod lst_common;

use lst_common::*;

const VALIDATORS: &[&str] = &[
    "unionvaloper1qp4uzhet2sd9mrs46kemse5dt9ncz4k3xuz7ej",
    "unionvaloper1fktal7292h36h7glff5edq59vpdfn750yrwmej",
    "unionvaloper1d348dktd9nz0y6afzh3az5j39qahc93c2qyclq",
    "unionvaloper1asxs295fuy7jph8p8eqtc2r8zxggdc20y7vf70",
];

const INITIAL_VALIDATOR_SHARES: &[(&str, Uint128)] = &[
    (VALIDATORS[0], Uint128::new(40)),
    (VALIDATORS[1], Uint128::new(30)),
    (VALIDATORS[2], Uint128::new(20)),
    (VALIDATORS[3], Uint128::new(10)),
];

fn make_proxy_call(funded_msgs: &[(&str, Binary, Vec<CwCoin>)]) -> Vec<u8> {
    let wasm_msgs: Vec<CosmosMsg> = funded_msgs
        .iter()
        .map(|(contract, msg, funds)| {
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract.to_string(),
                msg: msg.clone(),
                funds: funds.clone(),
            })
        })
        .collect();

    serde_json::to_vec(&wasm_msgs).expect("vec cosmosmsg json")
}

fn make_zkgm_bond_payload_via_call(
    lst_hub: &str,
    mint_to: &str,
    min_mint_amount: u128,
    funds_denom: &str,
    funds_amount: u128,
) -> Vec<u8> {
    make_proxy_call(&[(
        lst_hub,
        to_json_binary(&LstExecuteMsg::Bond {
            mint_to_address: Addr::unchecked(mint_to),
            min_mint_amount: min_mint_amount.into(),
        })
        .unwrap(),
        vec![CwCoin {
            denom: funds_denom.to_string(),
            amount: funds_amount.into(),
        }],
    )])
}

fn make_zkgm_unbond_payload_via_call(cw20_token: &str, lst_hub: &str, amount: u128) -> Vec<u8> {
    make_proxy_call(&[
        (
            cw20_token,
            to_json_binary(&Cw20ExecuteMsg::IncreaseAllowance {
                spender: lst_hub.to_string(),
                amount: amount.into(),
                expires: None,
            })
            .unwrap(),
            Vec::new(),
        ),
        (
            lst_hub,
            to_json_binary(&LstExecuteMsg::Unbond {
                amount: amount.into(),
            })
            .unwrap(),
            Vec::new(),
        ),
    ])
}

fn make_zkgm_withdraw_payload_via_call(
    lst_hub: &str,
    withdraw_to_address: Addr,
    batch_id: BatchId,
) -> Vec<u8> {
    make_proxy_call(&[(
        lst_hub,
        to_json_binary(&LstExecuteMsg::Withdraw {
            withdraw_to_address,
            batch_id,
        })
        .unwrap(),
        Vec::new(),
    )])
}

async fn bond(
    t: &LstContext,
    src_channel_id: u32,
    dst_channel_id: u32,
    sender_on_evm: Address,
    sender_evm_provider: DynProvider<AnyNetwork>,
    min_mint_amount: u128,
    bond_amount: u128,
) {
    let proxy_address_on_union = calculate_proxy_address(
        &t.union_address.zkgm,
        alloy::primitives::U256::ZERO,
        dst_channel_id,
        sender_on_evm.as_ref(),
    );

    // funding the eth address that we execute bond with, with au
    eth_fund_u(t, src_channel_id, sender_on_evm.into(), 100_000, 500_000)
        .await
        .unwrap();

    let mut salt_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt_bytes);

    let _ = t
        .ctx
        .dst
        .zkgmerc20_approve(
            ETH_ADDRESS_U,
            ETH_ADDRESS_ZKGM,
            U256::from(100000000000u64),
            sender_evm_provider.clone(),
        )
        .await
        .unwrap();

    let ucs03_zkgm = UCS03Zkgm::new(ETH_ADDRESS_ZKGM.into(), sender_evm_provider.clone());

    let call = ucs03_zkgm
        .send(
            dst_channel_id,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            InstructionEvm::from(zkgm_helper::make_batch(vec![
                evm_helper::make_token_order_v2(
                    &t.union_address.escrow_vault,
                    &sender_on_evm,
                    &proxy_address_on_union,
                    alloy::primitives::U256::from(bond_amount),
                ),
                zkgm_helper::make_call(
                    sender_on_evm.to_vec().into(),
                    proxy_address_on_union.as_bytes().to_vec().into(),
                    make_zkgm_bond_payload_via_call(
                        t.union_address.lst_hub.as_str(),
                        proxy_address_on_union.as_ref(),
                        min_mint_amount,
                        "au",
                        bond_amount,
                    )
                    .into(),
                ),
            ])),
        )
        .clear_decoder()
        .with_cloned_provider();

    let (_, ack) = t
        .ctx
        .send_and_recv_and_ack_with_retry::<evm::Module, cosmos::Module>(
            &t.ctx.dst,
            ETH_ADDRESS_ZKGM,
            call,
            &t.ctx.src,
            3,
            Duration::from_secs(20),
            Duration::from_secs(720),
            &sender_evm_provider,
        )
        .await
        .unwrap();

    assert_eq!(ack.tag, TAG_ACK_SUCCESS);
}

async fn unbond(
    t: &LstContext,
    channel_id_on_eth: u32,
    channel_id_on_union: u32,
    sender_on_evm: Address,
    sender_evm_provider: DynProvider<AnyNetwork>,
    unbond_amount: u128,
) {
    let proxy_address_on_union = calculate_proxy_address(
        &t.union_address.zkgm,
        alloy::primitives::U256::ZERO,
        channel_id_on_union,
        sender_on_evm.as_ref(),
    );

    let mut salt_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt_bytes);

    let ucs03_zkgm = UCS03Zkgm::new(ETH_ADDRESS_ZKGM.into(), sender_evm_provider.clone());

    let call = ucs03_zkgm
        .send(
            channel_id_on_eth,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            InstructionEvm::from(zkgm_helper::make_call(
                sender_on_evm.to_vec().into(),
                proxy_address_on_union.as_bytes().to_vec().into(),
                make_zkgm_unbond_payload_via_call(
                    t.union_address.eu.as_str(),
                    t.union_address.lst_hub.as_str(),
                    unbond_amount,
                )
                .into(),
            )),
        )
        .clear_decoder()
        .with_cloned_provider();

    let (_, ack) = t
        .ctx
        .send_and_recv_and_ack_with_retry::<evm::Module, cosmos::Module>(
            &t.ctx.dst,
            ETH_ADDRESS_ZKGM,
            call,
            &t.ctx.src,
            6,
            Duration::from_secs(10),
            Duration::from_secs(720),
            &sender_evm_provider,
        )
        .await
        .unwrap();

    assert_eq!(ack.tag, TAG_ACK_SUCCESS);
}

async fn withdraw(
    t: &LstContext,
    channel_id_on_eth: u32,
    channel_id_on_union: u32,
    sender_on_evm: Address,
    sender_evm_provider: DynProvider<AnyNetwork>,
    batch_id: BatchId,
) {
    let proxy_address_on_union = calculate_proxy_address(
        &t.union_address.zkgm,
        alloy::primitives::U256::ZERO,
        channel_id_on_union,
        sender_on_evm.as_ref(),
    );

    let mut salt_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt_bytes);

    let ucs03_zkgm = UCS03Zkgm::new(ETH_ADDRESS_ZKGM.into(), sender_evm_provider.clone());

    let call = ucs03_zkgm
        .send(
            channel_id_on_eth,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            InstructionEvm::from(zkgm_helper::make_call(
                sender_on_evm.to_vec().into(),
                proxy_address_on_union.as_bytes().to_vec().into(),
                make_zkgm_withdraw_payload_via_call(
                    t.union_address.lst_hub.as_str(),
                    proxy_address_on_union.clone(),
                    batch_id,
                )
                .into(),
            )),
        )
        .clear_decoder()
        .with_cloned_provider();

    let (_, ack) = t
        .ctx
        .send_and_recv_and_ack_with_retry::<evm::Module, cosmos::Module>(
            &t.ctx.dst,
            ETH_ADDRESS_ZKGM,
            call,
            &t.ctx.src,
            6,
            Duration::from_secs(10),
            Duration::from_secs(720),
            &sender_evm_provider,
        )
        .await
        .unwrap();

    assert_eq!(ack.tag, TAG_ACK_SUCCESS);
}

async fn set_validators(t: &LstContext, validators: &[(&str, Uint128)]) -> TxResponse {
    let acc = t
        .ctx
        .src
        .privileged_acc_keyring
        .with(|k| async move { k })
        .await
        .unwrap();

    t.ctx
        .src
        .send_cosmwasm_transaction_with_retry(
            t.union_address.lst_staker.clone(),
            (
                to_json_binary(&lst_staker::msg::ExecuteMsg::SetValidators(
                    BTreeMap::from_iter(validators.iter().map(|(k, v)| (Addr::unchecked(*k), *v))),
                ))
                .unwrap()
                .into(),
                vec![],
            ),
            acc,
        )
        .await
        .unwrap()
        .unwrap()
}

async fn query_delegation(t: &LstContext, validator: &str) -> anyhow::Result<Uint128> {
    let balance = &t
        .ctx
        .src
        .query::<_, staking_proto::QueryDelegationResponse>(
            "/cosmos.staking.v1beta1.Query/Delegation",
            staking_proto::QueryDelegationRequest {
                delegator_addr: t.union_address.lst_staker.to_string(),
                validator_addr: validator.to_string(),
            },
        )
        .await
        .unwrap()
        .delegation_response
        .unwrap()
        .balance
        .unwrap();

    Ok(balance.amount.parse().unwrap())
}

async fn query_lst_hub<Q: Clone + Serialize, Res: DeserializeOwned>(
    t: &LstContext,
    query: Q,
) -> anyhow::Result<Res> {
    loop {
        match t
            .ctx
            .src
            .query_wasm_smart::<_, Res>(t.union_address.lst_hub.clone(), query.clone())
            .await
        {
            Ok(state) => break Ok(state),
            Err(_) => {
                warn!("the query didn't work for some reason, will retry in a sec.");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn get_batch_by_id(t: &LstContext, id: BatchId) -> anyhow::Result<Batch> {
    Ok(
        query_lst_hub::<_, Option<Batch>>(t, lst::msg::QueryMsg::Batch { batch_id: id })
            .await?
            .unwrap(),
    )
}

async fn get_pending_batch(t: &LstContext) -> anyhow::Result<PendingBatch> {
    query_lst_hub(t, lst::msg::QueryMsg::PendingBatch {}).await
}

async fn get_accounting_state(t: &LstContext) -> anyhow::Result<AccountingStateResponse> {
    query_lst_hub(t, lst::msg::QueryMsg::AccountingState {}).await
}

#[tokio::test]
async fn test_redelegation() {
    run_test_in_queue("redelegation", async |t, shared_data| {
        let delegation_1 = query_delegation(&t, VALIDATORS[0]).await.unwrap();
        let delegation_2 = query_delegation(&t, VALIDATORS[1]).await.unwrap();
        let delegation_3 = query_delegation(&t, VALIDATORS[2]).await.unwrap();
        let delegation_4 = query_delegation(&t, VALIDATORS[3]).await.unwrap();

        println!("[LST_OUTPUT] delegation 1 before: {delegation_1}");
        println!("[LST_OUTPUT] delegation 2 before: {delegation_2}");
        println!("[LST_OUTPUT] delegation 3 before: {delegation_3}");
        println!("[LST_OUTPUT] delegation 4 before: {delegation_4}");

        let _ = set_validators(
            &t,
            &[
                (VALIDATORS[0], 100u128.into()),
                (VALIDATORS[1], 50u128.into()),
            ],
        )
        .await;

        let delegation_1 = query_delegation(&t, VALIDATORS[0]).await.unwrap();
        let delegation_2 = query_delegation(&t, VALIDATORS[1]).await.unwrap();

        println!("[LST_OUTPUT] delegation 1 after: {delegation_1}");
        println!("[LST_OUTPUT] delegation 2 after: {delegation_2}");

        assert_eq!(delegation_1.u128(), 600);

        assert_eq!(delegation_2.u128(), 300);

        shared_data
    })
    .await;
}

#[tokio::test]
async fn test_redelegation_failure() {
    run_test_in_queue("redelegation_too_soon", async |t, shared_data| {
        let tx_resp = set_validators(
            &t,
            &[
                (VALIDATORS[0], 100u128.into()),
                (VALIDATORS[1], 50u128.into()),
            ],
        )
        .await;

        assert!(matches!(tx_resp.tx_result.code, Code::Err(_)));

        shared_data
    })
    .await;
}

#[tokio::test]
async fn test_bond_success() {
    run_test_in_queue("bond", async |t, mut shared_data| {
        let dst_channel_id = 1;
        let src_channel_id = 1;

        set_validators(&t, INITIAL_VALIDATOR_SHARES).await;

        // setting "au" as the fungible counterparty
        eth_set_fungible_counterparty(
            &t.ctx.dst,
            src_channel_id,
            b"au",
            t.union_address.escrow_vault.as_bytes(),
        )
        .await
        .unwrap();

        let bonds = [(120, 200), (400, 700)];

        let evm_signers = [
            t.ctx.dst.get_provider().await,
            t.ctx.dst.get_provider().await,
        ];

        shared_data.stakers = evm_signers.to_vec();

        for (bond_amount, evm_signer) in bonds.into_iter().zip(evm_signers) {
            let (evm_address, evm_provider): (Address, DynProvider<AnyNetwork>) = evm_signer;
            let (min_mint_amount, bond_amount) = bond_amount;

            let proxy_address = calculate_proxy_address(
                &t.union_address.zkgm,
                alloy::primitives::U256::ZERO,
                dst_channel_id,
                evm_address.as_ref(),
            );

            let eu_balance = || async {
                t.ctx
                    .src
                    .get_cw20_balance(&proxy_address, &t.union_address.eu)
                    .await
                    .unwrap()
            };

            let eu_balance_before = eu_balance().await;

            let state = get_accounting_state(&t).await.unwrap();

            bond(
                &t,
                src_channel_id,
                dst_channel_id,
                evm_address,
                evm_provider,
                min_mint_amount,
                bond_amount,
            )
            .await;

            let new_state = get_accounting_state(&t).await.unwrap();

            // We expect to get the same amount of `total_asset` change. We don't have to check the
            // actual value since it's the unit test's job.
            let eu_balance_after_stake = eu_balance().await;
            assert_eq!(
                eu_balance_after_stake - eu_balance_before,
                new_state.total_shares.u128() - state.total_shares.u128()
            );
        }

        // we are making sure that the 2 mins ubonding time passes, while making sure that the
        // rate continues to go down since more and more rewards will be earned with new blocks.
        let mut prev_rate = Decimal::MAX;
        for i in 1..8 {
            let new_rate = get_accounting_state(&t).await.unwrap().purchase_rate;

            tokio::time::sleep(Duration::from_secs(15)).await;

            info!("checking the rate ({i}).. new_rate: {new_rate} prev_rate: {prev_rate}");

            assert!(new_rate < prev_rate);

            prev_rate = new_rate;
        }

        shared_data
    })
    .await;
}

#[tokio::test]
async fn test_unbond_success() {
    run_test_in_queue("unbond", async |t, shared_data| {
        let dst_channel_id = 1;
        let src_channel_id = 1;

        let unbond_amounts = [30, 40];

        let mut total_unbond_amount = 0;

        for (amount, staker) in unbond_amounts.into_iter().zip(&shared_data.stakers) {
            unbond(
                &t,
                dst_channel_id,
                src_channel_id,
                staker.0,
                staker.1.clone(),
                amount,
            )
            .await;

            // we are unbonding twice so that the `is_new_request` can be triggered
            // and we test that case as well
            unbond(
                &t,
                dst_channel_id,
                src_channel_id,
                staker.0,
                staker.1.clone(),
                amount,
            )
            .await;

            total_unbond_amount += amount * 2;
        }

        let (_, signer) = t.ctx.src.get_signer().await;

        let state = get_accounting_state(&t).await.unwrap();
        let mut worked = false;
        for _ in 1..6 {
            if let Ok(outcome) = t
                .ctx
                .src
                .send_cosmwasm_transaction(
                    t.union_address.lst_hub.clone(),
                    (
                        to_json_binary(&LstExecuteMsg::SubmitBatch {})
                            .unwrap()
                            .into(),
                        vec![],
                    ),
                    signer,
                )
                .await
                .unwrap()
            {
                assert_eq!(outcome.tx_result.code, Code::Ok);
                worked = true;
                break;
            } else {
                info!("waiting 20 seconds before submitting batch");
                tokio::time::sleep(Duration::from_secs(20)).await;
            }
        }
        assert!(worked);
        let new_state = get_accounting_state(&t).await.unwrap();

        // We burned exactly the `total_unbond_amount` of eU's
        assert_eq!(
            total_unbond_amount,
            state.total_shares.u128() - new_state.total_shares.u128()
        );

        shared_data
    })
    .await;
}

#[tokio::test]
async fn test_withdraw_success() {
    run_test_in_queue("withdraw", async |t, shared_data| {
        // Waiting for at least the unbond amount before receiving the tokens
        tokio::time::sleep(Duration::from_secs(120)).await;

        let dst_channel_id = 1;
        let src_channel_id = 1;

        let unbond_amounts = [60u128, 80];

        let (_, signer) = t.ctx.src.get_signer().await;

        let pending_batch = get_pending_batch(&t).await.unwrap();
        let submitted_batch_id = BatchId::from_raw(pending_batch.batch_id.get().get() - 1).unwrap();

        let Batch::Submitted(submitted_batch) =
            get_batch_by_id(&t, submitted_batch_id).await.unwrap()
        else {
            panic!("expected submitted batch");
        };

        let fund_amount = submitted_batch.expected_native_unstaked;

        let outcome = t
            .ctx
            .src
            .send_cosmwasm_transaction(
                t.union_address.lst_hub.clone(),
                (
                    to_json_binary(&LstExecuteMsg::ReceiveUnstakedTokens {
                        batch_id: submitted_batch_id,
                    })
                    .unwrap()
                    .into(),
                    vec![ProtoCoin {
                        denom: "au".to_string(),
                        amount: fund_amount.to_string(),
                    }],
                ),
                signer,
            )
            .await
            .unwrap()
            .unwrap();

        assert_eq!(outcome.tx_result.code, Code::Ok);

        // Now the batch turns into `Received` since we actually received it
        let Batch::Received(received_batch) =
            get_batch_by_id(&t, submitted_batch_id).await.unwrap()
        else {
            panic!("expected received batch");
        };

        for (amount, staker) in unbond_amounts.into_iter().zip(&shared_data.stakers) {
            let proxy_address = calculate_proxy_address(
                &t.union_address.zkgm,
                alloy::primitives::U256::ZERO,
                src_channel_id,
                staker.0.as_ref(),
            );
            let u_balance_before = t
                .ctx
                .src
                .native_balance(proxy_address.clone(), "au")
                .await
                .unwrap();

            withdraw(
                &t,
                dst_channel_id,
                src_channel_id,
                staker.0,
                staker.1.clone(),
                submitted_batch_id,
            )
            .await;

            let u_balance_after = t.ctx.src.native_balance(proxy_address, "au").await.unwrap();

            assert_eq!(
                u_balance_after - u_balance_before,
                Uint128::new(received_batch.received_native_unstaked)
                    .multiply_ratio(amount, 140u128)
                    .u128()
            );
        }

        shared_data
    })
    .await;
}
