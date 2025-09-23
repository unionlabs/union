use std::time::Duration;

use alloy::{network::AnyNetwork, primitives::Address, providers::DynProvider};
use cosmwasm_std::{to_json_binary, Addr, Coin as CwCoin, CosmosMsg, Decimal, WasmMsg};
use lst::msg::{AccountingStateResponse, ExecuteMsg as LstExecuteMsg};
use rand::RngCore;
use tracing::info;
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

use crate::lst::*;

// static ERC20: OnceCell<H160> = OnceCell::const_new();

// u: union1pntx7gm7shsp6slef74ae7wvcc35t3wdmanh7wrg4xkq95m24qds5atmcp
// lst: union1fdg764zzxwvwyqkx3fuj0236l9ddh5xmutgvj2mv9cduffy82z9sp62ygc

fn make_zkgm_bond_payload_via_call(
    lst_hub: &str,
    mint_to: &str,
    min_mint_amount: u128,
    funds_denom: &str,
    funds_amount: u128,
) -> Vec<u8> {
    let wasm_exec: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lst_hub.to_string(),
        msg: to_json_binary(&LstExecuteMsg::Bond {
            mint_to_address: Addr::unchecked(mint_to),
            min_mint_amount: min_mint_amount.into(),
        })
        .unwrap(),
        funds: vec![CwCoin {
            denom: funds_denom.to_string(),
            amount: funds_amount.into(),
        }],
    });

    voyager_sdk::serde_json::to_vec(&vec![wasm_exec]).expect("vec cosmosmsg json")
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

async fn get_accounting_state(t: &LstContext) -> anyhow::Result<AccountingStateResponse> {
    t.ctx
        .src
        .query_wasm_smart::<_, AccountingStateResponse>(
            t.union_address.lst_hub.clone(),
            lst::msg::QueryMsg::AccountingState {},
        )
        .await
}

#[tokio::test]
async fn test_bond_success() {
    run_test_in_queue("bond", async |t| {
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

        let mut prev_rate = Decimal::MAX;
        for i in 1..10 {
            tokio::time::sleep(Duration::from_secs(10)).await;

            let new_rate = get_accounting_state(&t).await.unwrap().purchase_rate;
            info!("checking the rate ({i}).. new_rate: {new_rate} prev_rate: {prev_rate}");

            assert!(new_rate < prev_rate);

            prev_rate = new_rate;
        }
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
