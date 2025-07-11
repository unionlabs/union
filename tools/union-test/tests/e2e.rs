// tests/e2e.rs

use std::{str::FromStr, sync::Arc, time::Duration};

use alloy::sol_types::SolValue;
use concurrent_keyring::{KeyringConfig, KeyringConfigEntry};
use cosmos::{FeemarketConfig, GasFillerConfig};
use hex_literal::hex;
use alloy::hex::decode as hex_decode;

use protos::{cosmos::base::v1beta1::Coin, ibc::core::channel};
use rand::RngCore;
use serial_test::serial;
use tokio::sync::{Mutex, OnceCell};
use std::num::NonZero;
use ibc_union_spec::{ChannelId};
use cw20::{Cw20ExecuteMsg};
use ucs03_zkgm::{
    self,
    com::{
        FungibleAssetOrder, FungibleAssetOrderV2, Instruction, INSTR_VERSION_1,
        OP_FUNGIBLE_ASSET_ORDER, Stake, OP_STAKE, INSTR_VERSION_0
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
    primitives::{FixedBytes, H160},
};
use voyager_sdk::{anyhow, primitives::ChainId};

static CTX: OnceCell<Arc<TestContext<cosmos::Module, evm::Module>>> = OnceCell::const_new();
static CHANNELS_OPENED: OnceCell<()> = OnceCell::const_new();
static ERC20: OnceCell<H160> = OnceCell::const_new();


/// Returns the one–and–only deployed ERC20 address.
/// Deploys it the first time it’s called, then just returns the stored value.
async fn ensure_erc20(spender: H160) -> H160 {
    ERC20
        .get_or_init(|| async move {
            let ctx = init_ctx().await;
            ctx.dst
               .deploy_basic_erc20(spender)
               .await
               .expect("failed to deploy ERC20")
        })
        .await
        .clone()
}

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
        let needed_channel_count = 24; // TODO: Hardcoded now, it will be specified from config later.
        let ctx = TestContext::new(src, dst, needed_channel_count)
            .await
            .unwrap_or_else(|e| panic!("failed to build TestContext: {:#?}", e));
        
        Arc::new(ctx)
    })
    .await
    .clone()
}
/// do the “open clients → open connection → open channel” dance exactly once
async fn ensure_channels_opened(channel_count: usize) {
    CHANNELS_OPENED
        .get_or_init(|| async move {
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
                    channel_count,
                    Duration::from_secs(360*channel_count as u64),
                )
                .await
                .unwrap();
            assert_eq!(opened, channel_count);

            let available_count_after_open = ctx.get_available_channel_count().await;
            assert_eq!(current_available_count + channel_count, available_count_after_open);
            let pair = ctx.get_channel().await.expect("channel available");
            let available_count_after_get = ctx.get_available_channel_count().await;
            assert_eq!(available_count_after_open - 1, available_count_after_get);
            ctx.release_channel(pair).await;
            let available_count_after_release = ctx.get_available_channel_count().await;
            assert_eq!(available_count_after_open, available_count_after_release);

            // 4) done
        })
        .await;
}

async fn open_channel_from_union_to_evm(channel_count: usize) {
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
            channel_count,
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
}

async fn test_send_packet_from_union_to_evm_and_send_back_unwrap() {
    let ctx = init_ctx().await;
    ensure_channels_opened(ctx.channel_count).await;
    
    let available_channel = ctx.get_available_channel_count().await;
    assert_eq!(available_channel > 0, true);

    let pair = ctx.get_channel().await.expect("channel available");
    let dst_chain_id = pair.dest;
    let src_chain_id = pair.src;

    let eth_zkgm_contract = hex!("05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5");

    let quote_token_addr  = ctx.predict_wrapped_token::<evm::Module>(
        &ctx.dst,
        eth_zkgm_contract.into(),
        ChannelId::new(NonZero::new(dst_chain_id).unwrap()),
        "muno".into(),
    ).await.unwrap();


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
            quote_token: quote_token_addr.as_ref().to_vec().into(),
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
        .send_and_recv_with_retry::<cosmos::Module, evm::Module>(
            &ctx.src,
            contract,
            (bin_msg, funds).into(),
            &ctx.dst,
            3,
            Duration::from_secs(20),
            Duration::from_secs(720),
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
            base_token: quote_token_addr.as_ref().to_vec().into(),
            base_amount: "1".parse().unwrap(),
            base_token_symbol: "muno".into(),
            base_token_name: "muno".into(),
            base_token_decimals: 6,
            base_token_path: dst_chain_id.try_into().unwrap(),
            quote_token: "muno".into(),
            quote_amount: "1".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    println!("quote_token:  {:?}", quote_token_addr);

    // TODO: Here we should check the muno balance of sender account
    // Also token balanceOf the receiver account

    let evm_provider = ctx.dst.get_provider().await;

    let ucs03_zkgm = UCS03Zkgm::new(eth_zkgm_contract.into(), evm_provider);

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
        .send_and_recv_with_retry::<evm::Module, cosmos::Module>(
            &ctx.dst,
            eth_zkgm_contract.into(),
            call,
            &ctx.src,
            3,
            Duration::from_secs(20),
            Duration::from_secs(720),
        )
        .await;

    println!("Received packet data: {:?}", recv_packet_data);

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );
    // TODO: Here we should check the muno balance of sender account
    // Also token balanceOf the receiver account
    // And see if muno is decreased by 10 and the receiver's muno is increased by 10
}

async fn test_send_packet_from_evm_to_union_and_send_back_unwrap() {
    let ctx = init_ctx().await;
    ensure_channels_opened(ctx.channel_count).await;

    let available_channel = ctx.get_available_channel_count().await;
    assert_eq!(available_channel > 0, true);

    let pair = ctx.get_channel().await.expect("channel available");
    let dst_chain_id = pair.dest;
    let src_chain_id = pair.src;

    let eth_zkgm_contract = hex!("05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5");


    let deployed_erc20 = ensure_erc20(eth_zkgm_contract.into()).await;

    let union_zkgm_contract: Bech32<FixedBytes<32>> =
        Bech32::from_str("union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c")
            .unwrap();


    let quote_token_addr  = ctx.predict_wrapped_token::<cosmos::Module>(
        &ctx.src,
        union_zkgm_contract.into(),
        ChannelId::new(NonZero::new(src_chain_id).unwrap()),
        deployed_erc20.as_ref().to_vec(),
    ).await.unwrap();

    let quote_token_bytes = hex_decode(quote_token_addr.trim_start_matches("0x"))
        .expect("invalid quote‐token address hex");


    println!("Quote token address: {:?}", quote_token_addr);
    println!("deployed_erc20 address: {:?}", deployed_erc20);
    let mut salt_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt_bytes);

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
            base_token: deployed_erc20.as_ref().to_vec().into(),
            base_amount: "10".parse().unwrap(),
            base_token_symbol: "GLD".into(),
            base_token_name: "Gold".into(),
            base_token_decimals: 18,
            base_token_path: "0".parse().unwrap(),
            quote_token: quote_token_bytes.into(),
            quote_amount: "10".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    let evm_provider = ctx.dst.get_provider().await;

    let ucs03_zkgm = UCS03Zkgm::new(eth_zkgm_contract.into(), evm_provider);


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
        .send_and_recv_with_retry::<evm::Module, cosmos::Module>(
            &ctx.dst,
            eth_zkgm_contract.into(),
            call,
            &ctx.src,
            3,
            Duration::from_secs(20),
            Duration::from_secs(720),
        )
        .await;

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );
    println!("Received packet data: {:?}", recv_packet_data);



    let approve_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: "union1lnagprksnq6md62p4exafvck5mrj8uhg5m67xqmuklfl5mfw8lnsr2q550".into(),
        amount: "10".parse().unwrap(),
        expires: None,
    };

    let approve_msg_bin: Vec<u8> = Encode::<Json>::encode(&approve_msg);
    let quote_token_bytes = hex_decode(quote_token_addr.trim_start_matches("0x"))
        .expect("invalid quote‐token address hex");

    let approve_contract: Bech32<FixedBytes<32>> =
        Bech32::from_str(std::str::from_utf8(&quote_token_bytes).unwrap())
            .unwrap();

    let approve_recv_packet_data = ctx.src.send_transaction(
        approve_contract,
        (approve_msg_bin, vec![]).into()
    ).await;
    
    println!("Approve transaction result: {:?}", approve_recv_packet_data);

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
            base_token: quote_token_bytes.into(),
            base_amount: "1".parse().unwrap(),
            base_token_symbol: "GLD".into(),
            base_token_name: "Gold".into(),
            base_token_decimals: 18,
            base_token_path: src_chain_id.try_into().unwrap(),
            quote_token: deployed_erc20.as_ref().to_vec().into(),
            quote_amount: "1".parse().unwrap(),
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

    let funds = vec![];

    // TODO: Here we should check the muno balance of sender account
    // Also token balanceOf the receiver account

    let union_zkgm_contract: Bech32<FixedBytes<32>> =
        Bech32::from_str("union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c")
            .unwrap();

    let recv_packet_data = ctx
        .send_and_recv_with_retry::<cosmos::Module, evm::Module>(
            &ctx.src,
            union_zkgm_contract,
            (bin_msg, funds).into(),
            &ctx.dst,
            3,
            Duration::from_secs(20),
            Duration::from_secs(720),
        )
        .await;

    println!("Received packet data: {:?}", recv_packet_data);

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );


}




    // #[tokio::test]
    // #[serial]
    // async fn from_evm_to_union0() {
    //     self::test_send_packet_from_evm_to_union_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_evm_to_union1() {
    //     self::test_send_packet_from_evm_to_union_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_evm_to_union2() {
    //     self::test_send_packet_from_evm_to_union_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_evm_to_union3() {
    //     self::test_send_packet_from_evm_to_union_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_evm_to_union4() {
    //     self::test_send_packet_from_evm_to_union_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_evm_to_union5() {
    //     self::test_send_packet_from_evm_to_union_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_evm_to_union6() {
    //     self::test_send_packet_from_evm_to_union_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_evm_to_union7() {
    //     self::test_send_packet_from_evm_to_union_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_evm_to_union8() {
    //     self::test_send_packet_from_evm_to_union_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_evm_to_union9() {
    //     self::test_send_packet_from_evm_to_union_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_evm_to_union10() {
    //     self::test_send_packet_from_evm_to_union_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_evm_to_union11() {
    //     self::test_send_packet_from_evm_to_union_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_union_to_evm0() {
    //     self::test_send_packet_from_union_to_evm_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_union_to_evm1() {
    //     self::test_send_packet_from_union_to_evm_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_union_to_evm2() {
    //     self::test_send_packet_from_union_to_evm_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_union_to_evm3() {
    //     self::test_send_packet_from_union_to_evm_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_union_to_evm4() {
    //     self::test_send_packet_from_union_to_evm_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_union_to_evm5() {
    //     self::test_send_packet_from_union_to_evm_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_union_to_evm6() {
    //     self::test_send_packet_from_union_to_evm_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_union_to_evm7() {
    //     self::test_send_packet_from_union_to_evm_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_union_to_evm8() {
    //     self::test_send_packet_from_union_to_evm_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_union_to_evm9() {
    //     self::test_send_packet_from_union_to_evm_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_union_to_evm10() {
    //     self::test_send_packet_from_union_to_evm_and_send_back_unwrap().await;
    // }
    


    // #[tokio::test]
    // #[serial]
    // async fn from_union_to_evm11() {
    //     self::test_send_packet_from_union_to_evm_and_send_back_unwrap().await;
    // }
    

async fn e2e_stake_flow() {
    // 1) Initialize context & open IBC channels once for all tests
    let ctx = init_ctx().await;
    ensure_channels_opened(ctx.channel_count).await;

    // 2) Pick a free channel pair
    let pair = ctx.get_channel().await.expect("no channel available");
    let cosmos_chan_id = pair.src;
    let evm_chan_id    = pair.dest;

    // 3) Deploy a fresh ERC-20 on the EVM side and register it
    let zkgm_evm_addr = hex!("05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5").into();
    let erc20 = ensure_erc20(zkgm_evm_addr).await;

    // 4) Build a random salt + a single STAKE instruction
    let mut salt = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt);

    let mut img = [0u8; 32];
    img[30..].copy_from_slice(&hex!("1234"));     

    let stake = Stake {
        token_id:  "1".parse().unwrap(),
        governance_token: hex!("BABE")
                .to_vec().into(),
        governance_metadata_image: img.into(),
        sender:   hex!("Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD")
                .to_vec()
                .into(),
        beneficiary:  hex!("Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD")
                .to_vec()
                .into(),
        validator:    hex!("Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD")
                .to_vec()
                .into(),
        amount:   "1000".parse().unwrap(),
    };

    let instr = Instruction {
        version: INSTR_VERSION_0,
        opcode:  OP_STAKE,
        operand: Stake::abi_encode(&stake).into(),
    };

    // // 5) Send the IBC packet from Cosmos → EVM
    // let cw_msg = ucs03_zkgm::msg::ExecuteMsg::Send {
    //     channel_id:      cosmos_chan_id.try_into().unwrap(),
    //     timeout_height:  0u64.into(),
    //     timeout_timestamp: voyager_sdk::primitives::Timestamp::from_secs(u64::MAX),
    //     salt:            salt.into(),
    //     instruction:     instr.abi_encode().into(),
    // };
    // let bin = unionlabs::encoding::Json::encode(&cw_msg);
    // let funds = vec![protos::cosmos::base::v1beta1::Coin {
    //     denom:   "muno".into(),
    //     amount:  "1000".into(),
    // }];

    // // this will retry the send & wait for `wasm-packet_recv` on the EVM side
    // let ack = ctx
    //     .send_and_recv_with_retry(
    //         &ctx.src,
    //         Bech32::from_str("union1…yourCosmosZkgmAddr…").unwrap(),
    //         (bin, funds).into(),
    //         &ctx.dst,
    //         3,                       // retries
    //         Duration::from_secs(5),  // between retries
    //         Duration::from_secs(60), // timeout per attempt
    //     )
    //     .await?;

    // // 6) Now you can verify on EVM:
    // //    * NFT ownerOf(tokenId) == your Cosmos address
    // //    * `stakes[tokenId].state == STAKED`
    // //
    // // For example (pseudo):
    // //
    // // let provider = ctx.dst.get_provider().await;
    // // let zkgm = UCS03Zkgm::new(zkgm_evm_addr, provider);
    // // let owner = zkgm.owner_of(token_id).call().await?;
    // // assert_eq!(owner, your_cosmos_addr_as_h160);

    // // don’t forget to return the channel when you’re done
    // ctx.release_channel(pair).await;
    // Ok(())
}
