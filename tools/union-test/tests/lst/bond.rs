use std::time::Duration;

use alloy::{network::AnyNetwork, primitives::Address, providers::DynProvider};
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Coin as CwCoin, CosmosMsg, Decimal, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use lst::{
    msg::{AccountingStateResponse, Batch, BatchesResponse, ExecuteMsg as LstExecuteMsg},
    types::{BatchId, PendingBatch},
};
use rand::RngCore;
use serde::{de::DeserializeOwned, Serialize};
use tracing::{info, warn};
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
use voyager_sdk::serde_json;

use crate::lst::*;

// static ERC20: OnceCell<H160> = OnceCell::const_new();

// u: union1pntx7gm7shsp6slef74ae7wvcc35t3wdmanh7wrg4xkq95m24qds5atmcp
// lst: union1fdg764zzxwvwyqkx3fuj0236l9ddh5xmutgvj2mv9cduffy82z9sp62ygc

fn make_proxy_call(funded_msgs: &[(&str, Binary, Vec<CwCoin>)]) -> Vec<u8> {
    let wasm_msgs: Vec<CosmosMsg> = funded_msgs
        .into_iter()
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

    // funding the eth address that we execute bond with, with muno
    eth_fund_u(&t, src_channel_id, sender_on_evm.into(), 100_000, 500_000)
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
                        &proxy_address_on_union.to_string(),
                        min_mint_amount,
                        "muno",
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
            ETH_ADDRESS_ZKGM.into(),
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
            ETH_ADDRESS_ZKGM.into(),
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
            ETH_ADDRESS_ZKGM.into(),
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
async fn test_bond_success() {
    run_test_in_queue("bond", async |t, mut shared_data| {
        // ensure_channels_opened(ctx.channel_count).await;
        // let available_channel = ctx.get_available_channel_count().await;
        // assert!(available_channel > 0);
        // let pair = ctx.get_channel().await.expect("channel available");

        let dst_channel_id = 1;
        let src_channel_id = 1;

        // setting "muno" as the fungible counterparty
        eth_set_fungible_counterparty(
            &t.ctx.dst,
            src_channel_id,
            b"muno",
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

            let k = t
                .ctx
                .src
                .privileged_acc_keyring
                .with(async |k| k)
                .await
                .unwrap();

            let outcome = t
                .ctx
                .src
                .stake(
                    "unionvaloper1qp4uzhet2sd9mrs46kemse5dt9ncz4k3xuz7ej".to_string(),
                    bond_amount,
                    // &t.staker,
                    k,
                )
                .await
                .unwrap()
                .unwrap();

            assert_eq!(outcome.tx_result.code, Code::Ok);
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

        let k = t
            .ctx
            .src
            .privileged_acc_keyring
            .with(async |k| k)
            .await
            .unwrap();

        let outcome = t
            .ctx
            .src
            .unstake(
                "unionvaloper1qp4uzhet2sd9mrs46kemse5dt9ncz4k3xuz7ej".to_string(),
                total_unbond_amount,
                k,
            )
            .await
            .unwrap()
            .unwrap();

        assert_eq!(outcome.tx_result.code, Code::Ok);

        let (_, signer) = t.ctx.src.get_signer().await;

        let state = get_accounting_state(&t).await.unwrap();
        let outcome = t
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
                &signer,
            )
            .await
            .unwrap()
            .unwrap();

        assert_eq!(outcome.tx_result.code, Code::Ok);

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
                        denom: "muno".to_string(),
                        amount: fund_amount.to_string(),
                    }],
                ),
                &signer,
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
                .native_balance(proxy_address.clone(), "muno")
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

            let u_balance_after = t
                .ctx
                .src
                .native_balance(proxy_address, "muno")
                .await
                .unwrap();

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

// #[tokio::test]
// async fn test_escher_lst_unhappy_less_money_than_required() {
//     let t = init_ctx().await;

//     let (evm_address, evm_provider) = t.ctx.dst.get_provider().await;
//     let (cosmos_address, _) = t.ctx.src.get_signer().await;

//     // ensure_channels_opened(t.ctx.channel_count).await;
//     // let available_channel = t.ctx.get_available_channel_count().await;
//     // assert!(available_channel > 0);
//     // let pair = t.ctx.get_channel().await.expect("channel available");

//     let dst_channel_id = 1;
//     let src_channel_id = 1;

//     let vault_on_union = "union1skg5244hpkad603zz77kdekzw6ffgpfrde3ldk8rpdz06n62k4hqct0w4j";

//     let u_on_eth = hex_literal::hex!("0c8C6f58156D10d18193A8fFdD853e1b9F8D8836");

//     eth_set_fungible_counterparty(
//         &t.ctx.dst,
//         dst_channel_id,
//         b"muno",
//         t.union_address.escrow_vault.as_bytes(),
//     )
//     .await
//     .unwrap();

//     eth_fund_u(
//         &t,
//         src_channel_id,
//         cosmos_address.to_string(),
//         evm_address.into(),
//         5_000,
//         10_000,
//     )
//     .await
//     .unwrap();

//     let mut salt_bytes = [0u8; 32];
//     rand::rng().fill_bytes(&mut salt_bytes);

//     let new_u_balance = t
//         .ctx
//         .dst
//         .zkgmerc20_balance_of(
//             H160::from(u_on_eth),
//             evm_address.into(),
//             evm_provider.clone(),
//         )
//         .await
//         .unwrap();

//     let new_vault_balance = t
//         .ctx
//         .src
//         .native_balance(Bech32::from_str(vault_on_union).unwrap(), "muno")
//         .await
//         .unwrap();

//     // both balances are updated
//     assert!(new_u_balance > U256::ZERO);
//     assert!(new_vault_balance > 0);

//     println!("new_u_balance: {}", new_u_balance);

//     let lst_hub = "union1fdg764zzxwvwyqkx3fuj0236l9ddh5xmutgvj2mv9cduffy82z9sp62ygc";
//     // let lst = "union1jansh23v7teaznyljq6ss4vx6eym8yrz0dsjchap4u7j3etx94vqhmcwn5";

//     let addr: Bech32<FixedBytes<32>> = Bech32::from_str(UNION_ZKGM_ADDRESS).unwrap();
//     let canon = cosmwasm_std::CanonicalAddr::from(addr.data().as_ref());

//     let zkgm_proxy_canon = instantiate2_address(
//         // Checksum of the base contract
//         &hex_literal::hex!("ec827349ed4c1fec5a9c3462ff7c979d4c40e7aa43b16ed34469d04ff835f2a1"),
//         &canon,
//         proxy_account_salt_for_tests(
//             alloy::primitives::U256::ZERO,
//             dst_channel_id,
//             evm_address.as_slice(),
//         )
//         .get()
//         .as_slice(),
//     )
//     .unwrap();

//     let hrp = Bech32::<FixedBytes<32>>::from_str(UNION_ZKGM_ADDRESS)
//         .unwrap()
//         .hrp()
//         .to_string();

//     // 3. Build a Bech32 address with same HRP
//     let zkgm_proxy_calculated = Bech32::<FixedBytes<32>>::new(
//         hrp,
//         FixedBytes::<32>::try_from(zkgm_proxy_canon.as_slice()).unwrap(),
//     );
//     println!("ZKGM Proxy: {}", zkgm_proxy_calculated);

//     // let zkgm_proxy = zkgm_proxy_calculated.to_string().as_str();

//     // let bond_message: Bytes<Base64> = json!({
//     //     "bond": {
//     //         "mint_to_address": "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2",
//     //         "min_mint_amount": "3"
//     //     }
//     // })
//     // .to_string()
//     // .as_bytes()
//     // .into();

//     // let zkgm_message = json!({
//     //     "contract": lst_hub,
//     //     "msg": bond_message.to_string(),
//     //     "funds": [{ "denom": "muno", "amount": "3" }],
//     //     "call_action": "call_proxy"
//     // })
//     // .to_string();

//     let zkgm_msg_json = make_zkgm_call_payload(
//         lst_hub,
//         "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2",
//         1000000u128.into(),
//         "muno",
//         1000000,
//     );

//     let proxy_balance = t
//         .ctx
//         .src
//         .native_balance(zkgm_proxy_calculated.clone(), "muno")
//         .await
//         .unwrap();

//     println!("Proxy balance before: {}", proxy_balance);

//     let addr_str = zkgm_proxy_calculated.to_string();
//     let receiver = addr_str.into_bytes().into();

//     let instruction_from_evm_to_union = InstructionEvm {
//         version: INSTR_VERSION_0,
//         opcode: OP_BATCH,
//         operand: Batch {
//             instructions: vec![
//                 Instruction {
//                     version: INSTR_VERSION_2,
//                     opcode: OP_TOKEN_ORDER,
//                     operand: TokenOrderV2 {
//                         sender: evm_address.to_vec().into(),
//                         receiver,
//                         base_token: u_on_eth.to_vec().into(),
//                         // giving 150 but expecting 1000000 so it will fail.
//                         base_amount: "150".parse().unwrap(),
//                         kind: TOKEN_ORDER_KIND_SOLVE,
//                         metadata: SolverMetadata {
//                             solverAddress: vault_on_union.as_bytes().into(),
//                             metadata: Default::default(),
//                         }
//                         .abi_encode_params()
//                         .into(),
//                         quote_token: "muno".as_bytes().into(),
//                         // giving 150 but expecting 1000000 so it will fail.
//                         quote_amount: "150".parse().unwrap(),
//                     }
//                     .abi_encode_params()
//                     .into(),
//                 },
//                 Instruction {
//                     version: INSTR_VERSION_0,
//                     opcode: OP_CALL,
//                     operand: Call {
//                         sender: evm_address.to_vec().into(),
//                         eureka: false,
//                         contract_address: zkgm_proxy_calculated
//                             .to_string()
//                             .as_bytes()
//                             .to_vec()
//                             .into(),
//                         contract_calldata: zkgm_msg_json.as_bytes().to_vec().into(),
//                     }
//                     .abi_encode_params()
//                     .into(),
//                 },
//             ],
//         }
//         .abi_encode_params()
//         .into(),
//     };

//     let approve_tx_hash = t
//         .ctx
//         .dst
//         .zkgmerc20_approve(
//             u_on_eth.into(),
//             ETH_ADDRESS_ZKGM.into(),
//             U256::from(100000000000u64),
//             evm_provider.clone(),
//         )
//         .await;

//     assert!(
//         approve_tx_hash.is_ok(),
//         "Failed to send approve transaction: {:?}, from_account: {:?}",
//         approve_tx_hash.err(),
//         evm_address
//     );

//     let ucs03_zkgm = UCS03Zkgm::new(ETH_ADDRESS_ZKGM.into(), evm_provider.clone());

//     let call = ucs03_zkgm
//         .send(
//             dst_channel_id,
//             0u64,
//             4294967295000000000u64,
//             salt_bytes.into(),
//             instruction_from_evm_to_union.clone(),
//         )
//         .clear_decoder()
//         .with_cloned_provider();

//     let acked_packet = t
//         .ctx
//         .send_and_recv_and_ack_with_retry::<evm::Module, cosmos::Module>(
//             &t.ctx.dst,
//             ETH_ADDRESS_ZKGM.into(),
//             call,
//             &t.ctx.src,
//             3,
//             Duration::from_secs(20),
//             Duration::from_secs(720),
//             &evm_provider,
//         )
//         .await;

//     let acked_packet = acked_packet.unwrap();
//     assert!(
//         acked_packet.tag == 0,
//         "Packet is acked successfully, but it should not be. Tag: {:?}",
//         acked_packet.tag
//     );

//     let proxy_balance = t
//         .ctx
//         .src
//         .native_balance(zkgm_proxy_calculated, "muno")
//         .await
//         .unwrap();

//     println!("Proxy balance before: {}", proxy_balance);
// }

// #[tokio::test]
// async fn test_escher_lst_unhappy_wrong_denom() {
//     let t = init_ctx().await;

//     let (evm_address, evm_provider) = t.ctx.dst.get_provider().await;
//     let (cosmos_address, cosmos_provider) = t.ctx.src.get_signer().await;
//     let cosmos_address_bytes = cosmos_address.to_string().into_bytes();

//     // ensure_channels_opened(t.ctx.channel_count).await;
//     // let available_channel = t.ctx.get_available_channel_count().await;
//     // assert!(available_channel > 0);
//     // let pair = t.ctx.get_channel().await.expect("channel available");

//     let dst_channel_id = 1;
//     let src_channel_id = 1;

//     let vault_on_union = "union1skg5244hpkad603zz77kdekzw6ffgpfrde3ldk8rpdz06n62k4hqct0w4j";

//     let u_on_eth = hex_literal::hex!("0c8C6f58156D10d18193A8fFdD853e1b9F8D8836");

//     let metadata = SolverMetadata {
//         solverAddress: u_on_eth.to_vec().into(),
//         metadata: Default::default(),
//     }
//     .abi_encode_params();

//     let instruction_cosmos = Instruction {
//         version: INSTR_VERSION_2,
//         opcode: OP_TOKEN_ORDER,
//         operand: TokenOrderV2 {
//             sender: cosmos_address_bytes.clone().into(),
//             receiver: evm_address.to_vec().into(),
//             base_token: "muno".as_bytes().into(),
//             base_amount: "100000".parse().unwrap(),
//             kind: TOKEN_ORDER_KIND_SOLVE,
//             metadata: metadata.into(),
//             quote_token: u_on_eth.to_vec().into(),
//             quote_amount: "100000".parse().unwrap(),
//         }
//         .abi_encode_params()
//         .into(),
//     };

//     let (_, zkgm_deployer_provider) = t.ctx.dst.get_provider_privileged().await;
//     println!("registering u counterpart");
//     t.ctx
//         .dst
//         .u_register_fungible_counterpart(
//             H160::from(u_on_eth),
//             zkgm_deployer_provider.clone(),
//             alloy::primitives::U256::ZERO,
//             dst_channel_id,
//             b"muno".to_vec().into(),
//             evm::u::U::FungibleCounterparty {
//                 beneficiary: vault_on_union.as_bytes().to_vec().into(),
//             },
//         )
//         .await
//         .unwrap();
//     println!("u counterpart is registered");

//     let mut salt_bytes = [0u8; 32];
//     rand::rng().fill_bytes(&mut salt_bytes);

//     let cw_msg = ucs03_zkgm::msg::ExecuteMsg::Send {
//         channel_id: src_channel_id.try_into().unwrap(),
//         timeout_height: 0u64.into(),
//         timeout_timestamp: voyager_sdk::primitives::Timestamp::from_secs(u32::MAX.into()),
//         salt: salt_bytes.into(),
//         instruction: instruction_cosmos.abi_encode_params().into(),
//     };
//     let bin_msg: Vec<u8> = Encode::<Json>::encode(&cw_msg);

//     let funds = vec![Coin {
//         denom: "muno".into(),
//         amount: "100000".into(),
//     }];

//     let contract: Bech32<FixedBytes<32>> = Bech32::from_str(UNION_ZKGM_ADDRESS).unwrap();

//     let ack_packet_data = t
//         .ctx
//         .send_and_recv_and_ack_with_retry::<cosmos::Module, evm::Module>(
//             &t.ctx.src,
//             contract,
//             (bin_msg, funds),
//             &t.ctx.dst,
//             3,
//             Duration::from_secs(20),
//             Duration::from_secs(720),
//             cosmos_provider,
//         )
//         .await;

//     assert!(
//         ack_packet_data.is_ok(),
//         "Failed to send and ack packet: {:?}",
//         ack_packet_data.err()
//     );

//     let new_u_balance = t
//         .ctx
//         .dst
//         .zkgmerc20_balance_of(
//             H160::from(u_on_eth),
//             evm_address.into(),
//             evm_provider.clone(),
//         )
//         .await
//         .unwrap();

//     let new_vault_balance = t
//         .ctx
//         .src
//         .native_balance(Bech32::from_str(vault_on_union).unwrap(), "muno")
//         .await
//         .unwrap();

//     // both balances are updated
//     assert!(new_u_balance > U256::ZERO);
//     assert!(new_vault_balance > 0);

//     println!("new_u_balance: {}", new_u_balance);

//     let lst_hub = "union1fdg764zzxwvwyqkx3fuj0236l9ddh5xmutgvj2mv9cduffy82z9sp62ygc";
//     // let lst = "union1jansh23v7teaznyljq6ss4vx6eym8yrz0dsjchap4u7j3etx94vqhmcwn5";

//     let addr: Bech32<FixedBytes<32>> = Bech32::from_str(UNION_ZKGM_ADDRESS).unwrap();
//     let canon = cosmwasm_std::CanonicalAddr::from(addr.data().as_ref());

//     let zkgm_proxy_canon = instantiate2_address(
//         // Checksum of the base contract
//         &hex_literal::hex!("ec827349ed4c1fec5a9c3462ff7c979d4c40e7aa43b16ed34469d04ff835f2a1"),
//         &canon,
//         proxy_account_salt_for_tests(
//             alloy::primitives::U256::ZERO,
//             dst_channel_id,
//             evm_address.as_slice(),
//         )
//         .get()
//         .as_slice(),
//     )
//     .unwrap();

//     let hrp = Bech32::<FixedBytes<32>>::from_str(UNION_ZKGM_ADDRESS)
//         .unwrap()
//         .hrp()
//         .to_string();

//     // 3. Build a Bech32 address with same HRP
//     let zkgm_proxy_calculated = Bech32::<FixedBytes<32>>::new(
//         hrp,
//         FixedBytes::<32>::try_from(zkgm_proxy_canon.as_slice()).unwrap(),
//     );
//     println!("ZKGM Proxy: {}", zkgm_proxy_calculated);

//     // let zkgm_proxy = zkgm_proxy_calculated.to_string().as_str();

//     // let bond_message: Bytes<Base64> = json!({
//     //     "bond": {
//     //         "mint_to_address": "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2",
//     //         "min_mint_amount": "3"
//     //     }
//     // })
//     // .to_string()
//     // .as_bytes()
//     // .into();

//     // let zkgm_message = json!({
//     //     "contract": lst_hub,
//     //     "msg": bond_message.to_string(),
//     //     "funds": [{ "denom": "muno", "amount": "3" }],
//     //     "call_action": "call_proxy"
//     // })
//     // .to_string();

//     let zkgm_msg_json = make_zkgm_call_payload(
//         lst_hub,
//         "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2",
//         10u128.into(),
//         "muan", // wrong denom to make it fail
//         10,
//     );

//     let proxy_balance = t
//         .ctx
//         .src
//         .native_balance(zkgm_proxy_calculated.clone(), "muno")
//         .await
//         .unwrap();

//     println!("Proxy balance before: {}", proxy_balance);

//     let addr_str = zkgm_proxy_calculated.to_string();
//     let receiver = addr_str.into_bytes().into();

//     let instruction_from_evm_to_union = InstructionEvm {
//         version: INSTR_VERSION_0,
//         opcode: OP_BATCH,
//         operand: Batch {
//             instructions: vec![
//                 Instruction {
//                     version: INSTR_VERSION_2,
//                     opcode: OP_TOKEN_ORDER,
//                     operand: TokenOrderV2 {
//                         sender: evm_address.to_vec().into(),
//                         receiver,
//                         base_token: u_on_eth.to_vec().into(),
//                         base_amount: "150".parse().unwrap(),
//                         kind: TOKEN_ORDER_KIND_SOLVE,
//                         metadata: SolverMetadata {
//                             solverAddress: vault_on_union.as_bytes().into(),
//                             metadata: Default::default(),
//                         }
//                         .abi_encode_params()
//                         .into(),
//                         quote_token: "muno".as_bytes().into(),
//                         quote_amount: "150".parse().unwrap(),
//                     }
//                     .abi_encode_params()
//                     .into(),
//                 },
//                 Instruction {
//                     version: INSTR_VERSION_0,
//                     opcode: OP_CALL,
//                     operand: Call {
//                         sender: evm_address.to_vec().into(),
//                         eureka: false,
//                         contract_address: zkgm_proxy_calculated
//                             .to_string()
//                             .as_bytes()
//                             .to_vec()
//                             .into(),
//                         contract_calldata: zkgm_msg_json.as_bytes().to_vec().into(),
//                     }
//                     .abi_encode_params()
//                     .into(),
//                 },
//             ],
//         }
//         .abi_encode_params()
//         .into(),
//     };

//     let approve_tx_hash = t
//         .ctx
//         .dst
//         .zkgmerc20_approve(
//             u_on_eth.into(),
//             ETH_ADDRESS_ZKGM.into(),
//             U256::from(100000000000u64),
//             evm_provider.clone(),
//         )
//         .await;

//     assert!(
//         approve_tx_hash.is_ok(),
//         "Failed to send approve transaction: {:?}, from_account: {:?}",
//         approve_tx_hash.err(),
//         evm_address
//     );

//     let ucs03_zkgm = UCS03Zkgm::new(ETH_ADDRESS_ZKGM.into(), evm_provider.clone());

//     let call = ucs03_zkgm
//         .send(
//             dst_channel_id,
//             0u64,
//             4294967295000000000u64,
//             salt_bytes.into(),
//             instruction_from_evm_to_union.clone(),
//         )
//         .clear_decoder()
//         .with_cloned_provider();

//     let acked_packet = t
//         .ctx
//         .send_and_recv_and_ack_with_retry::<evm::Module, cosmos::Module>(
//             &t.ctx.dst,
//             ETH_ADDRESS_ZKGM.into(),
//             call,
//             &t.ctx.src,
//             3,
//             Duration::from_secs(20),
//             Duration::from_secs(720),
//             &evm_provider,
//         )
//         .await;

//     let acked_packet = acked_packet.unwrap();
//     assert!(
//         acked_packet.tag == 0,
//         "Packet is acked successfully, but it should not be. Tag: {:?}",
//         acked_packet.tag
//     );

//     let proxy_balance = t
//         .ctx
//         .src
//         .native_balance(zkgm_proxy_calculated, "muno")
//         .await
//         .unwrap();

//     println!("Proxy balance before: {}", proxy_balance);
// }

// #[tokio::test]
// async fn test_escher_lst_unhappy_under_minimum_ls_amount() {
//     let t = init_ctx().await;

//     let (evm_address, evm_provider) = t.ctx.dst.get_provider().await;
//     let (cosmos_address, cosmos_provider) = t.ctx.src.get_signer().await;
//     let cosmos_address_bytes = cosmos_address.to_string().into_bytes();

//     // ensure_channels_opened(t.ctx.channel_count).await;
//     // let available_channel = t.ctx.get_available_channel_count().await;
//     // assert!(available_channel > 0);
//     // let pair = ctx.get_channel().await.expect("channel available");

//     let dst_channel_id = 1;
//     let src_channel_id = 1;

//     let vault_on_union = "union1skg5244hpkad603zz77kdekzw6ffgpfrde3ldk8rpdz06n62k4hqct0w4j";

//     let u_on_eth = hex_literal::hex!("0c8C6f58156D10d18193A8fFdD853e1b9F8D8836");

//     let metadata = SolverMetadata {
//         solverAddress: u_on_eth.to_vec().into(),
//         metadata: Default::default(),
//     }
//     .abi_encode_params();

//     let instruction_cosmos = Instruction {
//         version: INSTR_VERSION_2,
//         opcode: OP_TOKEN_ORDER,
//         operand: TokenOrderV2 {
//             sender: cosmos_address_bytes.clone().into(),
//             receiver: evm_address.to_vec().into(),
//             base_token: "muno".as_bytes().into(),
//             base_amount: "100000".parse().unwrap(),
//             kind: TOKEN_ORDER_KIND_SOLVE,
//             metadata: metadata.into(),
//             quote_token: u_on_eth.to_vec().into(),
//             quote_amount: "100000".parse().unwrap(),
//         }
//         .abi_encode_params()
//         .into(),
//     };

//     let (_, zkgm_deployer_provider) = t.ctx.dst.get_provider_privileged().await;
//     println!("registering u counterpart");
//     t.ctx
//         .dst
//         .u_register_fungible_counterpart(
//             H160::from(u_on_eth),
//             zkgm_deployer_provider.clone(),
//             alloy::primitives::U256::ZERO,
//             dst_channel_id,
//             b"muno".to_vec().into(),
//             evm::u::U::FungibleCounterparty {
//                 beneficiary: vault_on_union.as_bytes().to_vec().into(),
//             },
//         )
//         .await
//         .unwrap();
//     println!("u counterpart is registered");

//     let mut salt_bytes = [0u8; 32];
//     rand::rng().fill_bytes(&mut salt_bytes);

//     let cw_msg = ucs03_zkgm::msg::ExecuteMsg::Send {
//         channel_id: src_channel_id.try_into().unwrap(),
//         timeout_height: 0u64.into(),
//         timeout_timestamp: voyager_sdk::primitives::Timestamp::from_secs(u32::MAX.into()),
//         salt: salt_bytes.into(),
//         instruction: instruction_cosmos.abi_encode_params().into(),
//     };
//     let bin_msg: Vec<u8> = Encode::<Json>::encode(&cw_msg);

//     let funds = vec![Coin {
//         denom: "muno".into(),
//         amount: "100000".into(),
//     }];

//     let contract: Bech32<FixedBytes<32>> = Bech32::from_str(UNION_ZKGM_ADDRESS).unwrap();

//     let ack_packet_data = t
//         .ctx
//         .send_and_recv_and_ack_with_retry::<cosmos::Module, evm::Module>(
//             &t.ctx.src,
//             contract,
//             (bin_msg, funds),
//             &t.ctx.dst,
//             3,
//             Duration::from_secs(20),
//             Duration::from_secs(720),
//             cosmos_provider,
//         )
//         .await;

//     assert!(
//         ack_packet_data.is_ok(),
//         "Failed to send and ack packet: {:?}",
//         ack_packet_data.err()
//     );

//     let new_u_balance = t
//         .ctx
//         .dst
//         .zkgmerc20_balance_of(
//             H160::from(u_on_eth),
//             evm_address.into(),
//             evm_provider.clone(),
//         )
//         .await
//         .unwrap();

//     let new_vault_balance = t
//         .ctx
//         .src
//         .native_balance(Bech32::from_str(vault_on_union).unwrap(), "muno")
//         .await
//         .unwrap();

//     // both balances are updated
//     assert!(new_u_balance > U256::ZERO);
//     assert!(new_vault_balance > 0);

//     println!("new_u_balance: {}", new_u_balance);

//     let lst_hub = "union1fdg764zzxwvwyqkx3fuj0236l9ddh5xmutgvj2mv9cduffy82z9sp62ygc";
//     // let lst = "union1jansh23v7teaznyljq6ss4vx6eym8yrz0dsjchap4u7j3etx94vqhmcwn5";

//     let addr: Bech32<FixedBytes<32>> = Bech32::from_str(UNION_ZKGM_ADDRESS).unwrap();
//     let canon = cosmwasm_std::CanonicalAddr::from(addr.data().as_ref());

//     let zkgm_proxy_canon = instantiate2_address(
//         // Checksum of the base contract
//         &hex_literal::hex!("ec827349ed4c1fec5a9c3462ff7c979d4c40e7aa43b16ed34469d04ff835f2a1"),
//         &canon,
//         proxy_account_salt_for_tests(
//             alloy::primitives::U256::ZERO,
//             dst_channel_id,
//             evm_address.as_slice(),
//         )
//         .get()
//         .as_slice(),
//     )
//     .unwrap();

//     let hrp = Bech32::<FixedBytes<32>>::from_str(UNION_ZKGM_ADDRESS)
//         .unwrap()
//         .hrp()
//         .to_string();

//     // 3. Build a Bech32 address with same HRP
//     let zkgm_proxy_calculated = Bech32::<FixedBytes<32>>::new(
//         hrp,
//         FixedBytes::<32>::try_from(zkgm_proxy_canon.as_slice()).unwrap(),
//     );
//     println!("ZKGM Proxy: {}", zkgm_proxy_calculated);

//     // let zkgm_proxy = zkgm_proxy_calculated.to_string().as_str();

//     // let bond_message: Bytes<Base64> = json!({
//     //     "bond": {
//     //         "mint_to_address": "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2",
//     //         "min_mint_amount": "3"
//     //     }
//     // })
//     // .to_string()
//     // .as_bytes()
//     // .into();

//     // let zkgm_message = json!({
//     //     "contract": lst_hub,
//     //     "msg": bond_message.to_string(),
//     //     "funds": [{ "denom": "muno", "amount": "3" }],
//     //     "call_action": "call_proxy"
//     // })
//     // .to_string();

//     let zkgm_msg_json = make_zkgm_call_payload(
//         lst_hub,
//         "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2",
//         0u128.into(),
//         "muno",
//         0, // under minimum amount to make it fail
//     );

//     let proxy_balance = t
//         .ctx
//         .src
//         .native_balance(zkgm_proxy_calculated.clone(), "muno")
//         .await
//         .unwrap();

//     println!("Proxy balance before: {}", proxy_balance);

//     let addr_str = zkgm_proxy_calculated.to_string();
//     let receiver = addr_str.into_bytes().into();

//     let instruction_from_evm_to_union = InstructionEvm {
//         version: INSTR_VERSION_0,
//         opcode: OP_BATCH,
//         operand: Batch {
//             instructions: vec![
//                 Instruction {
//                     version: INSTR_VERSION_2,
//                     opcode: OP_TOKEN_ORDER,
//                     operand: TokenOrderV2 {
//                         sender: evm_address.to_vec().into(),
//                         receiver,
//                         base_token: u_on_eth.to_vec().into(),
//                         base_amount: "150".parse().unwrap(),
//                         kind: TOKEN_ORDER_KIND_SOLVE,
//                         metadata: SolverMetadata {
//                             solverAddress: vault_on_union.as_bytes().into(),
//                             metadata: Default::default(),
//                         }
//                         .abi_encode_params()
//                         .into(),
//                         quote_token: "muno".as_bytes().into(),
//                         quote_amount: "150".parse().unwrap(),
//                     }
//                     .abi_encode_params()
//                     .into(),
//                 },
//                 Instruction {
//                     version: INSTR_VERSION_0,
//                     opcode: OP_CALL,
//                     operand: Call {
//                         sender: evm_address.to_vec().into(),
//                         eureka: false,
//                         contract_address: zkgm_proxy_calculated
//                             .to_string()
//                             .as_bytes()
//                             .to_vec()
//                             .into(),
//                         contract_calldata: zkgm_msg_json.as_bytes().to_vec().into(),
//                     }
//                     .abi_encode_params()
//                     .into(),
//                 },
//             ],
//         }
//         .abi_encode_params()
//         .into(),
//     };

//     let approve_tx_hash = t
//         .ctx
//         .dst
//         .zkgmerc20_approve(
//             u_on_eth.into(),
//             ETH_ADDRESS_ZKGM.into(),
//             U256::from(100000000000u64),
//             evm_provider.clone(),
//         )
//         .await;

//     assert!(
//         approve_tx_hash.is_ok(),
//         "Failed to send approve transaction: {:?}, from_account: {:?}",
//         approve_tx_hash.err(),
//         evm_address
//     );

//     let ucs03_zkgm = UCS03Zkgm::new(ETH_ADDRESS_ZKGM.into(), evm_provider.clone());

//     let call = ucs03_zkgm
//         .send(
//             dst_channel_id,
//             0u64,
//             4294967295000000000u64,
//             salt_bytes.into(),
//             instruction_from_evm_to_union.clone(),
//         )
//         .clear_decoder()
//         .with_cloned_provider();

//     let acked_packet = t
//         .ctx
//         .send_and_recv_and_ack_with_retry::<evm::Module, cosmos::Module>(
//             &t.ctx.dst,
//             ETH_ADDRESS_ZKGM.into(),
//             call,
//             &t.ctx.src,
//             3,
//             Duration::from_secs(20),
//             Duration::from_secs(720),
//             &evm_provider,
//         )
//         .await;

//     let acked_packet = acked_packet.unwrap();
//     assert!(
//         acked_packet.tag == 0,
//         "Packet is acked successfully, but it should not be. Tag: {:?}",
//         acked_packet.tag
//     );

//     let proxy_balance = t
//         .ctx
//         .src
//         .native_balance(zkgm_proxy_calculated, "muno")
//         .await
//         .unwrap();

//     println!("Proxy balance before: {}", proxy_balance);
// }

// #[tokio::test]
// async fn test_escher_lst_unhappy_no_funds() {
//     let t = init_ctx().await;

//     let (evm_address, evm_provider) = t.ctx.dst.get_provider().await;
//     let (cosmos_address, cosmos_provider) = t.ctx.src.get_signer().await;
//     let cosmos_address_bytes = cosmos_address.to_string().into_bytes();

//     // ensure_channels_opened(t.ctx.channel_count).await;
//     // let available_channel = t.ctx.get_available_channel_count().await;
//     // assert!(available_channel > 0);
//     // let pair = t.ctx.get_channel().await.expect("channel available");

//     let dst_channel_id = 1;
//     let src_channel_id = 1;

//     let vault_on_union = "union1skg5244hpkad603zz77kdekzw6ffgpfrde3ldk8rpdz06n62k4hqct0w4j";

//     let u_on_eth = hex_literal::hex!("0c8C6f58156D10d18193A8fFdD853e1b9F8D8836");

//     let metadata = SolverMetadata {
//         solverAddress: u_on_eth.to_vec().into(),
//         metadata: Default::default(),
//     }
//     .abi_encode_params();

//     let instruction_cosmos = Instruction {
//         version: INSTR_VERSION_2,
//         opcode: OP_TOKEN_ORDER,
//         operand: TokenOrderV2 {
//             sender: cosmos_address_bytes.clone().into(),
//             receiver: evm_address.to_vec().into(),
//             base_token: "muno".as_bytes().into(),
//             base_amount: "100000".parse().unwrap(),
//             kind: TOKEN_ORDER_KIND_SOLVE,
//             metadata: metadata.into(),
//             quote_token: u_on_eth.to_vec().into(),
//             quote_amount: "100000".parse().unwrap(),
//         }
//         .abi_encode_params()
//         .into(),
//     };

//     let (_, zkgm_deployer_provider) = t.ctx.dst.get_provider_privileged().await;
//     println!("registering u counterpart");
//     t.ctx
//         .dst
//         .u_register_fungible_counterpart(
//             H160::from(u_on_eth),
//             zkgm_deployer_provider.clone(),
//             alloy::primitives::U256::ZERO,
//             dst_channel_id,
//             b"muno".to_vec().into(),
//             evm::u::U::FungibleCounterparty {
//                 beneficiary: vault_on_union.as_bytes().to_vec().into(),
//             },
//         )
//         .await
//         .unwrap();
//     println!("u counterpart is registered");

//     let mut salt_bytes = [0u8; 32];
//     rand::rng().fill_bytes(&mut salt_bytes);

//     let cw_msg = ucs03_zkgm::msg::ExecuteMsg::Send {
//         channel_id: src_channel_id.try_into().unwrap(),
//         timeout_height: 0u64.into(),
//         timeout_timestamp: voyager_sdk::primitives::Timestamp::from_secs(u32::MAX.into()),
//         salt: salt_bytes.into(),
//         instruction: instruction_cosmos.abi_encode_params().into(),
//     };
//     let bin_msg: Vec<u8> = Encode::<Json>::encode(&cw_msg);

//     let funds = vec![Coin {
//         denom: "muno".into(),
//         amount: "100000".into(),
//     }];

//     let contract: Bech32<FixedBytes<32>> = Bech32::from_str(UNION_ZKGM_ADDRESS).unwrap();

//     let ack_packet_data = t
//         .ctx
//         .send_and_recv_and_ack_with_retry::<cosmos::Module, evm::Module>(
//             &t.ctx.src,
//             contract,
//             (bin_msg, funds),
//             &t.ctx.dst,
//             3,
//             Duration::from_secs(20),
//             Duration::from_secs(720),
//             cosmos_provider,
//         )
//         .await;

//     assert!(
//         ack_packet_data.is_ok(),
//         "Failed to send and ack packet: {:?}",
//         ack_packet_data.err()
//     );

//     let new_u_balance = t
//         .ctx
//         .dst
//         .zkgmerc20_balance_of(
//             H160::from(u_on_eth),
//             evm_address.into(),
//             evm_provider.clone(),
//         )
//         .await
//         .unwrap();

//     let new_vault_balance = t
//         .ctx
//         .src
//         .native_balance(Bech32::from_str(vault_on_union).unwrap(), "muno")
//         .await
//         .unwrap();

//     // both balances are updated
//     assert!(new_u_balance > U256::ZERO);
//     assert!(new_vault_balance > 0);

//     println!("new_u_balance: {}", new_u_balance);

//     let lst_hub = "union1fdg764zzxwvwyqkx3fuj0236l9ddh5xmutgvj2mv9cduffy82z9sp62ygc";
//     // let lst = "union1jansh23v7teaznyljq6ss4vx6eym8yrz0dsjchap4u7j3etx94vqhmcwn5";

//     let addr: Bech32<FixedBytes<32>> = Bech32::from_str(UNION_ZKGM_ADDRESS).unwrap();
//     let canon = cosmwasm_std::CanonicalAddr::from(addr.data().as_ref());

//     let zkgm_proxy_canon = instantiate2_address(
//         // Checksum of the base contract
//         &hex_literal::hex!("ec827349ed4c1fec5a9c3462ff7c979d4c40e7aa43b16ed34469d04ff835f2a1"),
//         &canon,
//         proxy_account_salt_for_tests(
//             alloy::primitives::U256::ZERO,
//             dst_channel_id,
//             evm_address.as_slice(),
//         )
//         .get()
//         .as_slice(),
//     )
//     .unwrap();

//     let hrp = Bech32::<FixedBytes<32>>::from_str(UNION_ZKGM_ADDRESS)
//         .unwrap()
//         .hrp()
//         .to_string();

//     // 3. Build a Bech32 address with same HRP
//     let zkgm_proxy_calculated = Bech32::<FixedBytes<32>>::new(
//         hrp,
//         FixedBytes::<32>::try_from(zkgm_proxy_canon.as_slice()).unwrap(),
//     );
//     println!("ZKGM Proxy: {}", zkgm_proxy_calculated);

//     // let zkgm_proxy = zkgm_proxy_calculated.to_string().as_str();

//     // let bond_message: Bytes<Base64> = json!({
//     //     "bond": {
//     //         "mint_to_address": "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2",
//     //         "min_mint_amount": "3"
//     //     }
//     // })
//     // .to_string()
//     // .as_bytes()
//     // .into();

//     // let zkgm_message = json!({
//     //     "contract": lst_hub,
//     //     "msg": bond_message.to_string(),
//     //     "funds": [{ "denom": "muno", "amount": "3" }],
//     //     "call_action": "call_proxy"
//     // })
//     // .to_string();

//     let bond = LstExecuteMsg::Bond {
//         mint_to_address: Addr::unchecked("union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"),
//         min_mint_amount: 0u128.into(),
//     };

//     let wasm_exec: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
//         contract_addr: lst_hub.to_string(),
//         msg: to_json_binary(&bond).unwrap(),
//         funds: vec![], // no funds array so it will fail.
//     });

//     let zkgm_msg_json =
//         voyager_sdk::serde_json::to_string(&vec![wasm_exec]).expect("vec cosmosmsg json");

//     let proxy_balance = t
//         .ctx
//         .src
//         .native_balance(zkgm_proxy_calculated.clone(), "muno")
//         .await
//         .unwrap();

//     println!("Proxy balance before: {}", proxy_balance);

//     let addr_str = zkgm_proxy_calculated.to_string();
//     let receiver = addr_str.into_bytes().into();

//     let instruction_from_evm_to_union = InstructionEvm {
//         version: INSTR_VERSION_0,
//         opcode: OP_BATCH,
//         operand: Batch {
//             instructions: vec![
//                 Instruction {
//                     version: INSTR_VERSION_2,
//                     opcode: OP_TOKEN_ORDER,
//                     operand: TokenOrderV2 {
//                         sender: evm_address.to_vec().into(),
//                         receiver,
//                         base_token: u_on_eth.to_vec().into(),
//                         base_amount: "150".parse().unwrap(), //giving 150 but expecting 1000000 so it will fail.
//                         kind: TOKEN_ORDER_KIND_SOLVE,
//                         metadata: SolverMetadata {
//                             solverAddress: vault_on_union.as_bytes().into(),
//                             metadata: Default::default(),
//                         }
//                         .abi_encode_params()
//                         .into(),
//                         quote_token: "muno".as_bytes().into(),
//                         quote_amount: "150".parse().unwrap(), //giving 150 but expecting 1000000 so it will fail.
//                     }
//                     .abi_encode_params()
//                     .into(),
//                 },
//                 Instruction {
//                     version: INSTR_VERSION_0,
//                     opcode: OP_CALL,
//                     operand: Call {
//                         sender: evm_address.to_vec().into(),
//                         eureka: false,
//                         contract_address: zkgm_proxy_calculated
//                             .to_string()
//                             .as_bytes()
//                             .to_vec()
//                             .into(),
//                         contract_calldata: zkgm_msg_json.as_bytes().to_vec().into(),
//                     }
//                     .abi_encode_params()
//                     .into(),
//                 },
//             ],
//         }
//         .abi_encode_params()
//         .into(),
//     };

//     let approve_tx_hash = t
//         .ctx
//         .dst
//         .zkgmerc20_approve(
//             u_on_eth.into(),
//             ETH_ADDRESS_ZKGM.into(),
//             U256::from(100000000000u64),
//             evm_provider.clone(),
//         )
//         .await;

//     assert!(
//         approve_tx_hash.is_ok(),
//         "Failed to send approve transaction: {:?}, from_account: {:?}",
//         approve_tx_hash.err(),
//         evm_address
//     );

//     let ucs03_zkgm = UCS03Zkgm::new(ETH_ADDRESS_ZKGM.into(), evm_provider.clone());

//     let call = ucs03_zkgm
//         .send(
//             dst_channel_id,
//             0u64,
//             4294967295000000000u64,
//             salt_bytes.into(),
//             instruction_from_evm_to_union.clone(),
//         )
//         .clear_decoder()
//         .with_cloned_provider();

//     let acked_packet = t
//         .ctx
//         .send_and_recv_and_ack_with_retry::<evm::Module, cosmos::Module>(
//             &t.ctx.dst,
//             ETH_ADDRESS_ZKGM.into(),
//             call,
//             &t.ctx.src,
//             3,
//             Duration::from_secs(20),
//             Duration::from_secs(720),
//             &evm_provider,
//         )
//         .await;

//     let acked_packet = acked_packet.unwrap();
//     assert!(
//         acked_packet.tag == 0,
//         "Packet is acked successfully, but it should not be. Tag: {:?}",
//         acked_packet.tag
//     );

//     let proxy_balance = t
//         .ctx
//         .src
//         .native_balance(zkgm_proxy_calculated, "muno")
//         .await
//         .unwrap();

//     println!("Proxy balance before: {}", proxy_balance);
// }
