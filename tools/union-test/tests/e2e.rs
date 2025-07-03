// tests/e2e.rs

use std::{str::FromStr, sync::Arc, time::Duration};

use alloy::sol_types::SolValue;
use concurrent_keyring::{KeyringConfig, KeyringConfigEntry};
use cosmos::{FeemarketConfig, GasFillerConfig};
use hex_literal::hex;
use protos::{cosmos::base::v1beta1::Coin, ibc::core::channel};
use rand::RngCore;
use serial_test::serial;
use tokio::sync::{Mutex, OnceCell};
use ucs03_zkgm::{
    self,
    com::{
        FungibleAssetOrder, FungibleAssetOrderV2, Instruction, INSTR_VERSION_1,
        OP_FUNGIBLE_ASSET_ORDER,
    },
};
use union_test::{
    cosmos::{self, Config as CosmosConfig},
    evm::{self, zkgm::Instruction as InstructionEvm, zkgm::UCS03Zkgm, Config as EvmConfig},
    helpers::{ChannelOpenConfirm, ConnectionConfirm, CreateClientConfirm},
    ContractAddr,
    // adjust to your crate name
    TestContext,
};
use unionlabs::{
    bech32::Bech32,
    encoding::{Bincode, Encode, EncodeAs, EthAbi, Json},
    primitives::FixedBytes,
};
use voyager_sdk::{anyhow, primitives::ChainId};

static CTX: OnceCell<Arc<TestContext<cosmos::Module, evm::Module>>> = OnceCell::const_new();
static CLC: OnceCell<CreateClientConfirm> = OnceCell::const_new();
static DLC: OnceCell<CreateClientConfirm> = OnceCell::const_new();
static CONN_FROM_UNION_TO_EVM: OnceCell<ConnectionConfirm> = OnceCell::const_new();
static CHAN_FROM_UNION_TO_EVM: OnceCell<ChannelOpenConfirm> = OnceCell::const_new();
static CONN_FROM_EVM_TO_UNION: OnceCell<ConnectionConfirm> = OnceCell::const_new();
static CHAN_FROM_EVM_TO_UNION: OnceCell<ChannelOpenConfirm> = OnceCell::const_new();

async fn init_ctx<'a>() -> Arc<TestContext<cosmos::Module, evm::Module<'a>>> {
    CTX.get_or_init(|| async {
        let cosmos_cfg = cosmos::Config {
            chain_id: ChainId::new("union-devnet-1"),
            ibc_host_contract_address: Bech32::from_str(
                "union1nk3nes4ef6vcjan5tz6stf9g8p08q2kgqysx6q5exxh89zakp0msq5z79t",
            )
            .unwrap(),
            keyring: KeyringConfig {
                name: "alice".into(),
                keys: vec![KeyringConfigEntry::Raw {
                    name: "alice".into(),
                    key: hex_literal::hex!(
                        "aa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f"
                    )
                    .to_vec(),
                }],
            },
            rpc_url: "http://localhost:26657".into(),
            gas_config: GasFillerConfig::Feemarket(FeemarketConfig {
                max_gas: 10000000,
                gas_multiplier: Some(1.4),
                denom: None,
            }),
            fee_recipient: None,
        };
        let evm_cfg = evm::Config {
            chain_id: ChainId::new("32382"),
            ibc_handler_address: hex!("ed2af2aD7FE0D92011b26A2e5D1B4dC7D12A47C5").into(),
            multicall_address: hex!("84c4c2ee43ccfd523af9f78740256e0f60d38068").into(),
            rpc_url: "http://localhost:8545".into(),
            keyring: KeyringConfig {
                name: "alice".into(),
                keys: vec![KeyringConfigEntry::Raw {
                    name: "alice".into(),
                    key: hex!("4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77")
                        .to_vec(),
                }],
            },
            max_gas_price: None,
            fixed_gas_price: None,
            gas_multiplier: 2.0,
        };
        let src = cosmos::Module::new(cosmos_cfg).await.unwrap();
        let dst = evm::Module::new(evm_cfg).await.unwrap();
        let ctx = TestContext::new(src, dst)
            .await
            .unwrap_or_else(|e| panic!("failed to build TestContext: {:#?}", e));
        Arc::new(ctx)
    })
    .await
    .clone()
}

async fn open_channel_from_union_to_evm() {
    let ctx = init_ctx().await;
    let (src_client, dst_client) = ctx
        .create_clients(
            Duration::from_secs(60),
            "ibc-cosmwasm",
            "trusted/evm/mpt",
            "ibc-solidity",
            "cometbls",
        )
        .await
        .unwrap();

    assert!(src_client.client_id > 0);
    assert!(dst_client.client_id > 0);
    CLC.set(src_client.clone()).unwrap();
    DLC.set(dst_client.clone()).unwrap();

    // let conn = ctx
    //     .open_connection(true, src_client.client_id, dst_client.client_id, Duration::from_secs(180))
    //     .await
    //     .unwrap();
    let conn = ctx
        .open_connection::<cosmos::Module, evm::Module>(
            &ctx.src,
            src_client.client_id,
            &ctx.dst,
            dst_client.client_id,
            Duration::from_secs(180),
        )
        .await
        .unwrap();
    assert!(conn.connection_id > 0);
    assert!(conn.counterparty_connection_id > 0);

    let current_available_count = ctx.get_available_channel_count().await;

    let opened = ctx
        .open_channels(
            true,
            "union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c"
                .as_bytes()
                .into(),
            hex!("05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5")
                .to_vec()
                .into(),
            conn.counterparty_connection_id,
            "ucs03-zkgm-0".into(),
            1,
            Duration::from_secs(240),
        )
        .await
        .unwrap();
    assert_eq!(opened, 1);

    let available_count_after_open = ctx.get_available_channel_count().await;
    assert_eq!(current_available_count + 1, available_count_after_open);
    let pair = ctx.get_channel().await.expect("channel available");
    let available_count_after_get = ctx.get_available_channel_count().await;
    assert_eq!(available_count_after_open - 1, available_count_after_get);
    ctx.release_channel(pair).await;
    let available_count_after_release = ctx.get_available_channel_count().await;
    assert_eq!(available_count_after_open, available_count_after_release);
}

async fn _open_connection_from_evm_to_union() {
    let ctx = init_ctx().await;
    let (src_client, dst_client) = ctx
        .create_clients(
            Duration::from_secs(45),
            "ibc-cosmwasm",
            "trusted/evm/mpt",
            "ibc-solidity",
            "cometbls",
        )
        .await
        .unwrap();

    assert!(src_client.client_id > 0);
    assert!(dst_client.client_id > 0);
    // CLC.set(src_client.clone()).unwrap();
    // DLC.set(dst_client.clone()).unwrap();

    let conn = ctx
        .open_connection::<evm::Module, cosmos::Module>(
            &ctx.dst,
            dst_client.client_id,
            &ctx.src,
            src_client.client_id,
            Duration::from_secs(180),
        )
        .await
        .unwrap();
    assert!(conn.connection_id > 0);
    assert!(conn.counterparty_connection_id > 0);
    CONN_FROM_EVM_TO_UNION.set(conn).unwrap();
}

// #[tokio::test]
// #[serial]
// async fn test_open_channel_from_union_to_evm() {
//     let ctx = init_ctx().await;
//     open_connection_from_union_to_evm().await;
//     let conn = CONN_FROM_UNION_TO_EVM.get().expect("connection available");

//     let current_available_count = ctx.get_available_channel_count().await;

//     let opened = ctx
//         .open_channels(
//             true,
//            "union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c".as_bytes().into(),
//             hex!("05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5").to_vec().into(),
//             conn.connection_id,
//             "ucs03-zkgm-0".into(),
//             1,
//             Duration::from_secs(240),
//         )
//         .await
//         .unwrap();
//     assert_eq!(opened, 1);

//     let available_count_after_open = ctx.get_available_channel_count().await;
//     assert_eq!(current_available_count + 1, available_count_after_open);
//     let pair = ctx.get_channel().await.expect("channel available");
//     let available_count_after_get = ctx.get_available_channel_count().await;
//     assert_eq!(available_count_after_open - 1, available_count_after_get);
//     ctx.release_channel(pair).await;
//     let available_count_after_release = ctx.get_available_channel_count().await;
//     assert_eq!(available_count_after_open, available_count_after_release);
// }

// #[tokio::test]
// #[serial]
// async fn test_send_packet_from_evm_to_union() {
//     let ctx = init_ctx().await;
//     // open_channel_from_union_to_evm().await;

//     let available_channel =1;// ctx.get_available_channel_count().await;
//     assert_eq!(available_channel > 0, true);

//     // let pair = ctx.get_channel().await.expect("channel available");
//     let dst_chain_id = 1;//pair.dest;
//     let mut salt_bytes = [0u8; 32];
//     rand::thread_rng().fill_bytes(&mut salt_bytes);

//     let contract = hex!("05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5");

//     let instruction_from_evm_to_union = InstructionEvm {
//         version: INSTR_VERSION_1,
//         opcode: OP_FUNGIBLE_ASSET_ORDER,
//         operand: FungibleAssetOrder {
//             sender: hex!("Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD").to_vec().into(),
//             receiver: "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2".as_bytes().into(),
//             base_token: hex!("16628cB81ffDA9B8470e16299eFa5F76bF45A579").to_vec().into(),
//             base_amount: "1".parse().unwrap(),
//             base_token_symbol: "muno".into(),
//             base_token_name: "muno".into(),
//             base_token_decimals: 6,
//             base_token_path: "1".parse().unwrap(),
//             quote_token:  "muno".into(),
//             quote_amount: "1".parse().unwrap(),
//         }
//         .abi_encode_params()
//         .into(),
//     };

//     let send_call_struct = UCS03Zkgm::sendCall {
//         channelId: 1.try_into().unwrap(),
//         timeoutTimestamp: 4294967295000000000u64.into(),
//         timeoutHeight: 0u64.into(),
//         salt: salt_bytes.into(),
//         instruction: instruction_from_evm_to_union.clone(),
//     };

//     let recv_packet_data = ctx.send_and_recv_eth(
//         false,
//         contract.into(),
//         send_call_struct,
//         Duration::from_secs(360),
//     ).await;
//     assert!(recv_packet_data.is_ok(), "Failed to send and receive packet: {:?}", recv_packet_data.err());

// }

#[tokio::test]
#[serial]
async fn test_send_packet_from_union_to_evm_and_send_back_unwrap() {
    let ctx = init_ctx().await;
    open_channel_from_union_to_evm().await;

    let available_channel = ctx.get_available_channel_count().await;
    assert_eq!(available_channel > 0, true);

    let pair = ctx.get_channel().await.expect("channel available");
    let dst_chain_id = pair.dest;
    let src_chain_id = pair.src;

    let mut salt_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt_bytes);
    let contract: Bech32<FixedBytes<32>> =
        Bech32::from_str("union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c")
            .unwrap();

    let instruction_cosmos = Instruction {
        version: INSTR_VERSION_1,
        opcode: OP_FUNGIBLE_ASSET_ORDER,
        operand: FungibleAssetOrder {
            sender: "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"
                .as_bytes()
                .into(),
            receiver: hex!("Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD")
                .to_vec()
                .into(),
            base_token: "muno".as_bytes().into(),
            base_amount: "10".parse().unwrap(),
            base_token_symbol: "muno".into(),
            base_token_name: "muno".into(),
            base_token_decimals: 6,
            base_token_path: "0".parse().unwrap(),
            quote_token: hex!("16628cB81ffDA9B8470e16299eFa5F76bF45A579")
                .to_vec()
                .into(),
            quote_amount: "10".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    let cw_msg = ucs03_zkgm::msg::ExecuteMsg::Send {
        channel_id: src_chain_id.try_into().unwrap(),
        timeout_height: 0u64.into(),
        timeout_timestamp: voyager_sdk::primitives::Timestamp::from_secs(u32::MAX.into()),
        salt: salt_bytes.into(),
        instruction: instruction_cosmos.abi_encode_params().into(),
    };
    let bin_msg: Vec<u8> = Encode::<Json>::encode(&cw_msg);

    let funds = vec![Coin {
        denom: "muno".into(),
        amount: "10".into(),
    }];

    // TODO: Here we should check the muno balance of sender account
    // Also token balanceOf the receiver account
    let recv_packet_data = ctx
        .send_and_recv::<cosmos::Module, evm::Module>(
            &ctx.src,
            contract,
            (bin_msg, funds).into(),
            &ctx.dst,
            Duration::from_secs(360),
        )
        .await;
    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );
    // TODO: Here we should check the muno balance of sender account
    // Also token balanceOf the receiver account
    // And see if muno is decreased by 10 and the receiver's muno is increased by 10

    rand::thread_rng().fill_bytes(&mut salt_bytes);

    let contract = hex!("05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5");

    let instruction_from_evm_to_union = InstructionEvm {
        version: INSTR_VERSION_1,
        opcode: OP_FUNGIBLE_ASSET_ORDER,
        operand: FungibleAssetOrder {
            sender: hex!("Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD")
                .to_vec()
                .into(),
            receiver: "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"
                .as_bytes()
                .into(),
            base_token: hex!("16628cB81ffDA9B8470e16299eFa5F76bF45A579")
                .to_vec()
                .into(),
            base_amount: "1".parse().unwrap(),
            base_token_symbol: "muno".into(),
            base_token_name: "muno".into(),
            base_token_decimals: 6,
            base_token_path: "1".parse().unwrap(),
            quote_token: "muno".into(),
            quote_amount: "1".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    // TODO: Here we should check the muno balance of sender account
    // Also token balanceOf the receiver account

    let evm_provider = ctx.dst.get_provider().await;

    let ucs03_zkgm = UCS03Zkgm::new(contract.into(), evm_provider);

    let mut call = ucs03_zkgm
        .send(
            dst_chain_id.try_into().unwrap(),
            0u64.into(),
            4294967295000000000u64.into(),
            salt_bytes.into(),
            instruction_from_evm_to_union.clone(),
        )
        .clear_decoder();
    // let call = call.with_cloned_provider();
    let recv_packet_data = ctx
        .send_and_recv::<evm::Module, cosmos::Module>(
            &ctx.dst,
            contract.into(),
            call,
            &ctx.src,
            Duration::from_secs(360),
        )
        .await;

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );
    // TODO: Here we should check the muno balance of sender account
    // Also token balanceOf the receiver account
    // And see if muno is decreased by 10 and the receiver's muno is increased by 10
}

// #[tokio::test]
// #[serial]
// async fn test_open_channel_from_evm_to_union() {
//     let ctx = init_ctx().await;
//     open_connection_from_evm_to_union().await;
//     let conn = CONN_FROM_EVM_TO_UNION.get().expect("connection available");

//     let current_available_count = ctx.get_available_channel_count().await;

//     let opened = ctx
//         .open_channels(
//             false,
//            "union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c".as_bytes().into(),
//             hex!("05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5").to_vec().into(),
//             conn.connection_id,
//             "ucs03-zkgm-0".into(),
//             1,
//             Duration::from_secs(240),
//         )
//         .await
//         .unwrap();
//     assert_eq!(opened, 1);

//     let available_count_after_open = ctx.get_available_channel_count().await;
//     assert_eq!(current_available_count + 1, available_count_after_open);
//     let pair = ctx.get_channel().await.expect("channel available");
//     let available_count_after_get = ctx.get_available_channel_count().await;
//     assert_eq!(available_count_after_open - 1, available_count_after_get);
//     ctx.release_channel(pair).await;
//     let available_count_after_release = ctx.get_available_channel_count().await;
//     assert_eq!(available_count_after_open, available_count_after_release);
// }
