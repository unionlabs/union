use std::{str::FromStr, time::Duration};

use alloy::sol_types::SolValue;
use cosmwasm_std::{
    instantiate2_address, to_json_binary, Addr, Coin as CwCoin, CosmosMsg, Uint128, WasmMsg,
};
use lst::msg::ExecuteMsg as LstExecuteMsg;
use protos::cosmos::base::v1beta1::Coin;
use rand::RngCore;
use ucs03_zkgm::{
    self,
    com::{
        Batch, Call, Instruction, SolverMetadata, TokenOrderV2, INSTR_VERSION_0, INSTR_VERSION_2,
        OP_BATCH, OP_CALL, OP_TOKEN_ORDER, TOKEN_ORDER_KIND_SOLVE,
    },
};
use union_test::{
    cosmos::{self},
    evm::{
        self,
        zkgm::{Instruction as InstructionEvm, UCS03Zkgm},
    },
};
use unionlabs::{
    encoding::{Encode, Json},
    primitives::{Bech32, FixedBytes, H160, U256},
};

use crate::lst::*;

// static ERC20: OnceCell<H160> = OnceCell::const_new();

// u: union1pntx7gm7shsp6slef74ae7wvcc35t3wdmanh7wrg4xkq95m24qds5atmcp
// lst: union1fdg764zzxwvwyqkx3fuj0236l9ddh5xmutgvj2mv9cduffy82z9sp62ygc

fn make_zkgm_call_payload(
    lst_hub: &str,
    mint_to: &str,
    min_mint_amount: Uint128,
    funds_denom: &str,
    funds_amount: u32,
) -> String {
    let wasm_exec: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lst_hub.to_string(),
        msg: to_json_binary(&LstExecuteMsg::Bond {
            mint_to_address: Addr::unchecked(mint_to),
            min_mint_amount,
        })
        .unwrap(),
        funds: vec![CwCoin {
            denom: funds_denom.to_string(),
            amount: funds_amount.into(),
        }],
    });

    voyager_sdk::serde_json::to_string(&vec![wasm_exec]).expect("vec cosmosmsg json")
}

#[tokio::test]
async fn test_escher_lst_success() {
    let t = init_ctx().await;

    let (evm_address, evm_provider) = t.ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = t.ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();

    // ensure_channels_opened(ctx.channel_count).await;
    // let available_channel = ctx.get_available_channel_count().await;
    // assert!(available_channel > 0);
    // let pair = ctx.get_channel().await.expect("channel available");

    let dst_channel_id = 1;
    let src_channel_id = 1;

    let vault_on_union = "union1skg5244hpkad603zz77kdekzw6ffgpfrde3ldk8rpdz06n62k4hqct0w4j";

    let u_on_eth = hex_literal::hex!("0c8C6f58156D10d18193A8fFdD853e1b9F8D8836");

    let metadata = SolverMetadata {
        solverAddress: u_on_eth.to_vec().into(),
        metadata: Default::default(),
    }
    .abi_encode_params();

    let instruction_cosmos = Instruction {
        version: INSTR_VERSION_2,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV2 {
            sender: cosmos_address_bytes.clone().into(),
            receiver: evm_address.to_vec().into(),
            base_token: "muno".as_bytes().into(),
            base_amount: "100000".parse().unwrap(),
            kind: TOKEN_ORDER_KIND_SOLVE,
            metadata: metadata.into(),
            quote_token: u_on_eth.to_vec().into(),
            quote_amount: "100000".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    let (_, zkgm_deployer_provider) = t.ctx.dst.get_provider_privileged().await;
    println!("registering u counterpart");
    t.ctx
        .dst
        .u_register_fungible_counterpart(
            H160::from(u_on_eth),
            zkgm_deployer_provider.clone(),
            alloy::primitives::U256::ZERO,
            dst_channel_id,
            b"muno".to_vec().into(),
            evm::u::U::FungibleCounterparty {
                beneficiary: vault_on_union.as_bytes().to_vec().into(),
            },
        )
        .await
        .unwrap();
    println!("u counterpart is registered");

    let mut salt_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut salt_bytes);

    let cw_msg = ucs03_zkgm::msg::ExecuteMsg::Send {
        channel_id: src_channel_id.try_into().unwrap(),
        timeout_height: 0u64.into(),
        timeout_timestamp: voyager_sdk::primitives::Timestamp::from_secs(u32::MAX.into()),
        salt: salt_bytes.into(),
        instruction: instruction_cosmos.abi_encode_params().into(),
    };
    let bin_msg: Vec<u8> = Encode::<Json>::encode(&cw_msg);

    let funds = vec![Coin {
        denom: "muno".into(),
        amount: "100000".into(),
    }];

    let ack_packet_data = t
        .ctx
        .send_and_recv_and_ack_with_retry::<cosmos::Module, evm::Module>(
            &t.ctx.src,
            contract,
            (bin_msg, funds),
            &t.ctx.dst,
            3,
            Duration::from_secs(20),
            Duration::from_secs(720),
            cosmos_provider,
        )
        .await;

    assert!(
        ack_packet_data.is_ok(),
        "Failed to send and ack packet: {:?}",
        ack_packet_data.err()
    );

    let new_u_balance = t
        .ctx
        .dst
        .zkgmerc20_balance_of(
            H160::from(u_on_eth),
            evm_address.into(),
            evm_provider.clone(),
        )
        .await
        .unwrap();

    let new_vault_balance = t
        .ctx
        .src
        .native_balance(Bech32::from_str(vault_on_union).unwrap(), "muno")
        .await
        .unwrap();

    // both balances are updated
    assert!(new_u_balance > U256::ZERO);
    assert!(new_vault_balance > 0);

    println!("new_u_balance: {}", new_u_balance);

    let lst_hub = "union1fdg764zzxwvwyqkx3fuj0236l9ddh5xmutgvj2mv9cduffy82z9sp62ygc";
    // let lst = "union1jansh23v7teaznyljq6ss4vx6eym8yrz0dsjchap4u7j3etx94vqhmcwn5";

    let addr: Bech32<FixedBytes<32>> = Bech32::from_str(UNION_ZKGM_ADDRESS).unwrap();
    let canon = cosmwasm_std::CanonicalAddr::from(addr.data().as_ref());

    let zkgm_proxy_canon = instantiate2_address(
        // Checksum of the base contract
        &hex_literal::hex!("ec827349ed4c1fec5a9c3462ff7c979d4c40e7aa43b16ed34469d04ff835f2a1"),
        &canon,
        proxy_account_salt_for_tests(
            alloy::primitives::U256::ZERO,
            dst_channel_id,
            evm_address.as_slice(),
        )
        .get()
        .as_slice(),
    )
    .unwrap();

    let hrp = Bech32::<FixedBytes<32>>::from_str(UNION_ZKGM_ADDRESS)
        .unwrap()
        .hrp()
        .to_string();

    // 3. Build a Bech32 address with same HRP
    let zkgm_proxy_calculated = Bech32::<FixedBytes<32>>::new(
        hrp,
        FixedBytes::<32>::try_from(zkgm_proxy_canon.as_slice()).unwrap(),
    );
    println!("ZKGM Proxy: {}", zkgm_proxy_calculated);

    // let zkgm_proxy = zkgm_proxy_calculated.to_string().as_str();

    // let bond_message: Bytes<Base64> = json!({
    //     "bond": {
    //         "mint_to_address": "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2",
    //         "min_mint_amount": "3"
    //     }
    // })
    // .to_string()
    // .as_bytes()
    // .into();

    // let zkgm_message = json!({
    //     "contract": lst_hub,
    //     "msg": bond_message.to_string(),
    //     "funds": [{ "denom": "muno", "amount": "3" }],
    //     "call_action": "call_proxy"
    // })
    // .to_string();

    let zkgm_msg_json = make_zkgm_call_payload(
        lst_hub,
        "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2",
        150u128.into(),
        "muno",
        150,
    );

    let proxy_balance = t
        .ctx
        .src
        .native_balance(zkgm_proxy_calculated.clone(), "muno")
        .await
        .unwrap();

    println!("Proxy balance before: {}", proxy_balance);

    let addr_str = zkgm_proxy_calculated.to_string();
    let receiver = addr_str.into_bytes().into();

    let instruction_from_evm_to_union = InstructionEvm {
        version: INSTR_VERSION_0,
        opcode: OP_BATCH,
        operand: Batch {
            instructions: vec![
                Instruction {
                    version: INSTR_VERSION_2,
                    opcode: OP_TOKEN_ORDER,
                    operand: TokenOrderV2 {
                        sender: evm_address.to_vec().into(),
                        receiver,
                        base_token: u_on_eth.to_vec().into(),
                        base_amount: "150".parse().unwrap(),
                        kind: TOKEN_ORDER_KIND_SOLVE,
                        metadata: SolverMetadata {
                            solverAddress: vault_on_union.as_bytes().into(),
                            metadata: Default::default(),
                        }
                        .abi_encode_params()
                        .into(),
                        quote_token: "muno".as_bytes().into(),
                        quote_amount: "150".parse().unwrap(),
                    }
                    .abi_encode_params()
                    .into(),
                },
                Instruction {
                    version: INSTR_VERSION_0,
                    opcode: OP_CALL,
                    operand: Call {
                        sender: evm_address.to_vec().into(),
                        eureka: false,
                        contract_address: zkgm_proxy_calculated
                            .to_string()
                            .as_bytes()
                            .to_vec()
                            .into(),
                        contract_calldata: zkgm_msg_json.as_bytes().to_vec().into(),
                    }
                    .abi_encode_params()
                    .into(),
                },
            ],
        }
        .abi_encode_params()
        .into(),
    };

    let approve_tx_hash = t
        .ctx
        .dst
        .zkgmerc20_approve(
            u_on_eth.into(),
            ETH_ADDRESS_ZKGM.into(),
            U256::from(100000000000u64),
            evm_provider.clone(),
        )
        .await;

    assert!(
        approve_tx_hash.is_ok(),
        "Failed to send approve transaction: {:?}, from_account: {:?}",
        approve_tx_hash.err(),
        evm_address
    );

    let ucs03_zkgm = UCS03Zkgm::new(ETH_ADDRESS_ZKGM.into(), evm_provider.clone());

    let call = ucs03_zkgm
        .send(
            dst_channel_id,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            instruction_from_evm_to_union.clone(),
        )
        .clear_decoder()
        .with_cloned_provider();

    let acked_packet = t
        .ctx
        .send_and_recv_and_ack_with_retry::<evm::Module, cosmos::Module>(
            &t.ctx.dst,
            ETH_ADDRESS_ZKGM.into(),
            call,
            &t.ctx.src,
            3,
            Duration::from_secs(20),
            Duration::from_secs(720),
            &evm_provider,
        )
        .await;

    let acked_packet = acked_packet.unwrap();
    assert!(
        acked_packet.tag == 1,
        "Packet is not acked successfully, but it should be. Tag: {:?}",
        acked_packet.tag
    );

    let proxy_balance = t
        .ctx
        .src
        .native_balance(zkgm_proxy_calculated, "muno")
        .await
        .unwrap();

    println!("Proxy balance before: {}", proxy_balance);
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

fn proxy_account_salt_for_tests(
    path: alloy::primitives::U256,
    channel_id: u32,
    sender: &[u8],
) -> unionlabs::primitives::H256 {
    use alloy_sol_types::SolValue;
    use unionlabs::ethereum::keccak256;

    let encoded = (path, channel_id, sender.to_vec()).abi_encode_params();
    keccak256(encoded)
}
