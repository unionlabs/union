// tests/e2e.rs

use std::{num::NonZero, str::FromStr, sync::Arc, time::Duration};

use alloy::{
    hex::decode as hex_decode,
    sol_types::{SolCall, SolValue},
};
use concurrent_keyring::{KeyringConfig, KeyringConfigEntry};
use cosmos::{FeemarketConfig, GasFillerConfig};
use cw20::Cw20ExecuteMsg;
use hex_literal::hex;
use ibc_union_spec::ChannelId;
use protos::cosmos::base::v1beta1::Coin;
use rand::RngCore;
use serial_test::serial;
use tokio::sync::OnceCell;
use ucs03_zkgm::{
    self,
    com::{
        FungibleAssetOrder, FungibleAssetOrderV2, Instruction, Stake,
        FUNGIBLE_ASSET_METADATA_TYPE_PREIMAGE, INSTR_VERSION_0, INSTR_VERSION_1, INSTR_VERSION_2,
        OP_FUNGIBLE_ASSET_ORDER, OP_STAKE,
    },
};
use union_test::{
    cosmos::{self},
    evm::{
        self,
        zkgm::{Instruction as InstructionEvm, UCS03Zkgm},
        zkgmerc20::ZkgmERC20,
    },
    TestContext,
};
use unionlabs::{
    bech32::Bech32,
    encoding::{Encode, Json},
    ethereum::keccak256,
    primitives::{FixedBytes, H160, U256},
};
use voyager_sdk::primitives::ChainId;

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
            ws_url: "ws://localhost:8546".into(),
            keyring: KeyringConfig {
                name: "alice".into(),
                keys: vec![KeyringConfigEntry::Raw {
                    name: "alice".into(),
                    key: hex!("4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77")
                        .to_vec(),
                }],
            },
            max_gas_price: None,
            fixed_gas_price: Some(20_000_000_000u64.into()),
            gas_multiplier: 2.0,
        };
        let src = cosmos::Module::new(cosmos_cfg).await.unwrap();
        let dst = evm::Module::new(evm_cfg).await.unwrap();
        let needed_channel_count = 2; // TODO: Hardcoded now, it will be specified from config later.
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
                    Duration::from_secs(360 * channel_count as u64),
                )
                .await
                .unwrap();
            assert_eq!(opened, channel_count);

            let available_count_after_open = ctx.get_available_channel_count().await;
            assert_eq!(
                current_available_count + channel_count,
                available_count_after_open
            );
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

    let quote_token_addr = ctx
        .predict_wrapped_token::<evm::Module>(
            &ctx.dst,
            eth_zkgm_contract.into(),
            ChannelId::new(NonZero::new(dst_chain_id).unwrap()),
            "muno".into(),
        )
        .await
        .unwrap();

    let mut salt_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut salt_bytes);
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

    rand::rng().fill_bytes(&mut salt_bytes);

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

    let call = ucs03_zkgm
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

    let quote_token_addr = ctx
        .predict_wrapped_token::<cosmos::Module>(
            &ctx.src,
            union_zkgm_contract.into(),
            ChannelId::new(NonZero::new(src_chain_id).unwrap()),
            deployed_erc20.as_ref().to_vec(),
        )
        .await
        .unwrap();

    let quote_token_bytes = hex_decode(quote_token_addr.trim_start_matches("0x"))
        .expect("invalid quote‐token address hex");

    println!("Quote token address: {:?}", quote_token_addr);
    println!("deployed_erc20 address: {:?}", deployed_erc20);
    let mut salt_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut salt_bytes);

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

    let call = ucs03_zkgm
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
        Bech32::from_str(std::str::from_utf8(&quote_token_bytes).unwrap()).unwrap();

    let approve_recv_packet_data = ctx
        .src
        .send_transaction(approve_contract, (approve_msg_bin, vec![]).into())
        .await;

    assert!(
        approve_recv_packet_data.is_some(),
        "Failed to send approve transaction: {:?}",
        approve_recv_packet_data
    );

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

async fn test_stake_from_evm_to_union() {
    let ctx = init_ctx().await;
    ensure_channels_opened(ctx.channel_count).await;

    let available_channel = ctx.get_available_channel_count().await;
    assert_eq!(available_channel > 0, true);

    let pair = ctx.get_channel().await.expect("channel available");

    let eth_zkgm_contract = hex!("05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5");

    let img_metadata = ucs03_zkgm::com::FungibleAssetMetadata {
        implementation: hex!("999709eB04e8A30C7aceD9fd920f7e04EE6B97bA")
            .to_vec()
            .into(),
        initializer: ZkgmERC20::initializeCall {
            _authority: hex!("6C1D11bE06908656D16EBFf5667F1C45372B7c89").into(),
            _minter: eth_zkgm_contract.into(),
            _name: "muno".into(),
            _symbol: "muno".into(),
            _decimals: 6u8.into(),
        }
        .abi_encode()
        .into(),
    }
    .abi_encode_params();

    let img = keccak256(&img_metadata);

    let governance_token = ctx
        .dst
        .setup_governance_token(eth_zkgm_contract.into(), pair.dest, img)
        .await;

    assert!(
        governance_token.is_ok(),
        "Failed to setup governance token: {:?}",
        governance_token.err()
    );
    // let governance_token = governance_token.unwrap();
    // println!("✅ governance_token.unwrappedToken registered at: {:?}", governance_token.unwrappedToken);

    let mut salt_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut salt_bytes);

    let quote_token_addr = ctx
        .predict_wrapped_token_from_metadata_image_v2::<evm::Module>(
            &ctx.dst,
            eth_zkgm_contract.into(),
            ChannelId::new(NonZero::new(pair.dest).unwrap()),
            "muno".into(),
            img.into(),
        )
        .await
        .unwrap();

    println!("✅ Quote token address: {:?}", quote_token_addr);

    let instruction_cosmos = Instruction {
        version: INSTR_VERSION_2,
        opcode: OP_FUNGIBLE_ASSET_ORDER,
        operand: FungibleAssetOrderV2 {
            sender: "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"
                .as_bytes()
                .into(),
            receiver: hex!("Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD")
                .to_vec()
                .into(),
            base_token: "muno".as_bytes().into(),
            base_amount: "10".parse().unwrap(),
            metadata_type: FUNGIBLE_ASSET_METADATA_TYPE_PREIMAGE,
            metadata: img_metadata.into(),
            quote_token: quote_token_addr.as_ref().to_vec().into(),
            quote_amount: "10".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    let cw_msg = ucs03_zkgm::msg::ExecuteMsg::Send {
        channel_id: pair.src.try_into().unwrap(),
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

    let recv_packet_data = ctx
        .send_and_recv_with_retry::<cosmos::Module, evm::Module>(
            &ctx.src,
            Bech32::from_str("union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c")
                .unwrap(),
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

    println!("Received packet data: {:?}", recv_packet_data);
    println!("Calling approve on quote token: {:?}", quote_token_addr);

    let approve_tx_hash = ctx
        .dst
        .zkgmerc20_approve(
            quote_token_addr.into(),
            eth_zkgm_contract.into(),
            U256::from(100000000000u64),
        )
        .await;

    assert!(
        approve_tx_hash.is_ok(),
        "Failed to send approve transaction: {:?}",
        approve_tx_hash.err()
    );

    let given_validator = "unionvaloper1qp4uzhet2sd9mrs46kemse5dt9ncz4k3xuz7ej";
    let mut buf: [u8; 32] = [0u8; 32];
    rand::rng().fill_bytes(&mut buf);

    let random_token_id = U256::from_be_bytes(buf).into();
    println!("✅ random_token_id: {:?}", random_token_id);
    let instruction_from_evm_to_union = InstructionEvm {
        version: INSTR_VERSION_0,
        opcode: OP_STAKE,
        operand: Stake {
            token_id: random_token_id,
            governance_token: b"muno".into(),
            governance_metadata_image: img.into(),
            sender: hex!("Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD")
                .to_vec()
                .into(),
            beneficiary: hex!("Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD")
                .to_vec()
                .into(),
            validator: given_validator.as_bytes().into(),
            amount: "1".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    let evm_provider = ctx.dst.get_provider().await;

    let ucs03_zkgm = UCS03Zkgm::new(eth_zkgm_contract.into(), evm_provider);

    rand::rng().fill_bytes(&mut salt_bytes);
    let call = ucs03_zkgm
        .send(
            pair.dest.try_into().unwrap(),
            0u64.into(),
            4294967295000000000u64.into(),
            salt_bytes.into(),
            instruction_from_evm_to_union.clone(),
        )
        .clear_decoder();
    // let call = call.with_cloned_provider();
    let recv_packet_data = ctx
        .send_and_recv_stake::<evm::Module, cosmos::Module>(
            &ctx.dst,
            eth_zkgm_contract.into(),
            call,
            &ctx.src,
            Duration::from_secs(360),
            given_validator.to_string(),
        )
        .await;

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet for stake request: {:?}",
        recv_packet_data.err()
    );
    println!("Received packet data: {:?}", recv_packet_data);

    // Check random_token_id is ours or not now

    let snake_nft = ctx
        .dst
        .predict_stake_manager_address(eth_zkgm_contract.into())
        .await;
    assert!(snake_nft.is_ok(), "Failed to predict stake manager address");
    let snake_nft = snake_nft.unwrap();

    println!(
        "✅ Stake manager address: {:?}, random_token_id: {:?}",
        snake_nft, random_token_id
    );

    // let is_ours = ctx.dst.nft_owner_of(
    //     snake_nft,
    //     hex!("Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD")
    //             .to_vec()
    //             .into(),
    //             random_token_id.into()
    //         ).await;
    // assert!(is_ours.is_ok() && is_ours.unwrap(),
    //     "Failed to check NFT ownership after stake request: {:?}", is_ours.err());
}

#[tokio::test]
#[serial]
async fn from_evm_to_union0() {
    self::test_send_packet_from_evm_to_union_and_send_back_unwrap().await;
}

#[tokio::test]
#[serial]
async fn from_union_to_evm0() {
    self::test_send_packet_from_union_to_evm_and_send_back_unwrap().await;
}

#[tokio::test]
#[serial]
async fn from_evm_to_union_stake() {
    self::test_stake_from_evm_to_union().await;
}
