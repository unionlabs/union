// tests/e2e.rs

use std::{
    num::NonZero,
    str::FromStr,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

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
use tokio::sync::OnceCell;
use ucs03_zkgm::{
    self,
    com::{
        Instruction, SolverMetadata, Stake, TokenOrderV1, TokenOrderV2, Unstake, WithdrawStake,
        INSTR_VERSION_0, INSTR_VERSION_1, INSTR_VERSION_2, OP_STAKE, OP_TOKEN_ORDER, OP_UNSTAKE,
        OP_WITHDRAW_STAKE, TOKEN_ORDER_KIND_INITIALIZE, TOKEN_ORDER_KIND_SOLVE,
    },
};
use union_test::{
    channel_provider::ChannelPair,
    cosmos::{self},
    evm::{
        self,
        zkgm::{Instruction as InstructionEvm, UCS03Zkgm},
        zkgmerc20::ZkgmERC20,
    },
    TestContext,
};
use unionlabs::{
    encoding::{Encode, Json},
    ethereum::keccak256,
    primitives::{Bech32, FixedBytes, H160, U256},
};
use voyager_sdk::primitives::{ChainId, Timestamp};

static CTX: OnceCell<Arc<TestContext<cosmos::Module, evm::Module>>> = OnceCell::const_new();
static CHANNELS_OPENED: OnceCell<()> = OnceCell::const_new();
// static ERC20: OnceCell<H160> = OnceCell::const_new();

static UNION_ZKGM_ADDRESS: &str =
    "union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c";
static UNION_MINTER_ADDRESS: &str =
    "union1lnagprksnq6md62p4exafvck5mrj8uhg5m67xqmuklfl5mfw8lnsr2q550";
static EVM_ZKGM_BYTES: [u8; 20] = hex!("05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5");

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
                keys: vec![
                    // KeyringConfigEntry::Raw {
                    //     name: "alice".into(),
                    //     key: hex_literal::hex!(
                    //         "aa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f"
                    //     )
                    //     .to_vec(),
                    // },
                    KeyringConfigEntry::Raw {
                        name: "bob".into(),
                        key: hex_literal::hex!(
                            "f562d20f0a4ffd8814d262f7023f33971cbcd14a96d60027585777f174b9cdeb"
                        )
                        .to_vec(),
                    },
                    KeyringConfigEntry::Raw {
                        name: "dave".into(),
                        key: hex_literal::hex!(
                            "edc165ff1ebc27044ddc284c9cf5da656dcbff324f6ecbb9d3203cf5f4738d6d"
                        )
                        .to_vec(),
                    },
                    KeyringConfigEntry::Raw {
                        name: "charlie".into(),
                        key: hex_literal::hex!(
                            "a1f713e0f36404586085a599a45ca8233e23709e23cd54bc8d5452ef8f7bc1e6"
                        )
                        .to_vec(),
                    },
                ],
            },
            rpc_url: "http://0.0.0.0:26657".into(),
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
            rpc_url: "http://0.0.0.0:8545".into(),
            ws_url: "ws://0.0.0.0:8546".into(),
            keyring: KeyringConfig {
                name: "evm-keyring".into(),
                keys: vec![
                    KeyringConfigEntry::Raw {
                        name: "dev-key0.prv".into(),
                        key: hex!(
                            "4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77"
                        )
                        .to_vec(),
                    },
                    // KeyringConfigEntry::Raw {
                    //     name: "dev-key1.prv".into(),
                    //     key: hex!(
                    //         "d9c5dc47ed678fc3e63249953866d79e5cf48418e79d8eec1a985be7393ef3b9"
                    //     )
                    //     .to_vec(),
                    // },
                    // KeyringConfigEntry::Raw {
                    //     name: "dev-key2.prv".into(),
                    //     key: hex!(
                    //         "eadf66c84a1c2768a14e883512724d6023a54d500bf91d910a7dace376a97d6b"
                    //     )
                    //     .to_vec(),
                    // },
                    // KeyringConfigEntry::Raw {
                    //     name: "dev-key3.prv".into(),
                    //     key: hex!(
                    //         "d56f932b298ba86341037f3871141a707330316f6f9493641a2cd59ba4a53710"
                    //     )
                    //     .to_vec(),
                    // },
                    // KeyringConfigEntry::Raw {
                    //     name: "dev-key4.prv".into(),
                    //     key: hex!(
                    //         "084494a1ff88a1319e493d32aa6e127ab0eaaaf74b8714edfd670a9ddc4a060d"
                    //     )
                    //     .to_vec(),
                    // },
                    // KeyringConfigEntry::Raw {
                    //     name: "dev-key5.prv".into(),
                    //     key: hex!(
                    //         "f977996449841b13ce9bbb99873006e04590ddbe28d9cd449dd33505851e74ba"
                    //     )
                    //     .to_vec(),
                    // },
                    // KeyringConfigEntry::Raw {
                    //     name: "dev-key6.prv".into(),
                    //     key: hex!(
                    //         "523776c0e15a5826c85f08e0dd20d70190b0001e87f6ff9f25854d10f24db63c"
                    //     )
                    //     .to_vec(),
                    // },
                    // KeyringConfigEntry::Raw {
                    //     name: "dev-key7.prv".into(),
                    //     key: hex!(
                    //         "b7d500ecae3d26deaa9547557822c95208163e230cc04345bd223da99f5bd058"
                    //     )
                    //     .to_vec(),
                    // },
                ],
            },
            max_gas_price: None,
            fixed_gas_price: None,
            gas_multiplier: 2.0,
        };
        let src = cosmos::Module::new(cosmos_cfg).await.unwrap();
        let dst = evm::Module::new(evm_cfg).await.unwrap();
        let needed_channel_count = 8; // TODO: Hardcoded now, it will be specified from config later.

        // TODO(aeryz): move config file into the testing framework's own config file
        let ctx = TestContext::new(
            src,
            dst,
            needed_channel_count,
            "/home/kaancaglan/dev/union/voyager/config.jsonc",
        )
        .await
        .unwrap_or_else(|e| panic!("failed to build TestContext: {:#?}", e));

        Arc::new(ctx)
    })
    .await
    .clone()
}

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
                    UNION_ZKGM_ADDRESS.as_bytes().into(),
                    EVM_ZKGM_BYTES.to_vec().into(),
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

async fn test_send_vault_success() {
    let ctx = init_ctx().await;

    let (evm_address, evm_provider) = ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();

    ensure_channels_opened(ctx.channel_count).await;
    let available_channel = ctx.get_available_channel_count().await;
    assert!(available_channel > 0);
    let pair = ctx.get_channel().await.expect("channel available");
    let dst_channel_id = pair.dest;
    let src_channel_id = pair.src;

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
            base_amount: "10".parse().unwrap(),
            kind: TOKEN_ORDER_KIND_SOLVE,
            metadata: metadata.into(),
            quote_token: u_on_eth.to_vec().into(),
            quote_amount: "10".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    ctx.dst
        .u_register_fungible_counterpart(
            H160::from(u_on_eth),
            evm_provider.clone(),
            alloy::primitives::U256::ZERO,
            dst_channel_id,
            b"muno".to_vec().into(),
            evm::u::U::FungibleCounterparty {
                beneficiary: vault_on_union.as_bytes().to_vec().into(),
            },
        )
        .await
        .unwrap();

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
        amount: "10".into(),
    }];

    let contract: Bech32<FixedBytes<32>> = Bech32::from_str(UNION_ZKGM_ADDRESS).unwrap();

    let initial_u_balance = ctx
        .dst
        .zkgmerc20_balance_of(
            H160::from(u_on_eth),
            evm_address.into(),
            evm_provider.clone(),
        )
        .await
        .unwrap();

    let initial_vault_balance = ctx
        .src
        .native_balance(Bech32::from_str(vault_on_union).unwrap(), "muno")
        .await
        .unwrap();

    println!("initial U balance on eth: {initial_u_balance}");
    println!("initial U balance on union vault: {initial_vault_balance}");

    let ack_packet_data = ctx
        .send_and_recv_and_ack_with_retry::<cosmos::Module, evm::Module>(
            &ctx.src,
            contract,
            (bin_msg, funds),
            &ctx.dst,
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

    let new_u_balance = ctx
        .dst
        .zkgmerc20_balance_of(
            H160::from(u_on_eth),
            evm_address.into(),
            evm_provider.clone(),
        )
        .await
        .unwrap();

    let new_vault_balance = ctx
        .src
        .native_balance(Bech32::from_str(vault_on_union).unwrap(), "muno")
        .await
        .unwrap();

    println!("new U balance on eth: {new_u_balance}");
    println!("new U balance on union vault: {new_vault_balance}");

    assert_eq!(new_u_balance - initial_u_balance, 10u64.into());
    assert_eq!(new_vault_balance - initial_vault_balance, 10);
}

async fn test_send_packet_from_union_to_evm_and_send_back_unwrap() {
    let ctx = init_ctx().await;
    ensure_channels_opened(ctx.channel_count).await;

    let (evm_address, evm_provider) = ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();

    let available_channel = ctx.get_available_channel_count().await;
    assert!(available_channel > 0);

    let pair = ctx.get_channel().await.expect("channel available");
    let dst_chain_id = pair.dest;
    let src_chain_id = pair.src;

    let quote_token_addr = ctx
        .predict_wrapped_token::<evm::Module>(
            &ctx.dst,
            EVM_ZKGM_BYTES.into(),
            ChannelId::new(NonZero::new(dst_chain_id).unwrap()),
            "muno".into(),
            &evm_provider,
        )
        .await
        .unwrap();

    let mut salt_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut salt_bytes);
    let contract: Bech32<FixedBytes<32>> = Bech32::from_str(UNION_ZKGM_ADDRESS).unwrap();

    let instruction_cosmos = Instruction {
        version: INSTR_VERSION_1,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV1 {
            sender: cosmos_address_bytes.clone().into(),
            receiver: evm_address.to_vec().into(),
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

    let recv_packet_data = ctx
        .send_and_recv_with_retry::<cosmos::Module, evm::Module>(
            &ctx.src,
            contract,
            (bin_msg, funds),
            &ctx.dst,
            3,
            Duration::from_secs(20),
            Duration::from_secs(720),
            cosmos_provider,
        )
        .await;
    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );

    rand::rng().fill_bytes(&mut salt_bytes);

    let instruction_from_evm_to_union = InstructionEvm {
        version: INSTR_VERSION_1,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV1 {
            sender: evm_address.to_vec().into(),
            receiver: cosmos_address_bytes.clone().into(),
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

    let ucs03_zkgm = UCS03Zkgm::new(EVM_ZKGM_BYTES.into(), evm_provider.clone());

    let call = ucs03_zkgm
        .send(
            dst_chain_id,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            instruction_from_evm_to_union.clone(),
        )
        .clear_decoder();

    let recv_packet_data = ctx
        .send_and_recv_with_retry::<evm::Module, cosmos::Module>(
            &ctx.dst,
            EVM_ZKGM_BYTES.into(),
            call,
            &ctx.src,
            3,
            Duration::from_secs(20),
            Duration::from_secs(720),
            &evm_provider,
        )
        .await;

    println!("Received packet data: {:?}", recv_packet_data);

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );
}

async fn test_send_packet_from_evm_to_union_and_send_back_unwrap() {
    let ctx = init_ctx().await;
    let (evm_address, evm_provider) = ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_signer) = ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();

    println!("EVM Address: {:?}", evm_address);
    println!("Cosmos Address: {:?}", cosmos_address);

    ensure_channels_opened(ctx.channel_count).await;

    let available_channel = ctx.get_available_channel_count().await;
    assert!(available_channel > 0);

    let pair = ctx.get_channel().await.expect("channel available");
    let dst_chain_id = pair.dest;
    let src_chain_id = pair.src;

    // let deployed_erc20 = ensure_erc20(EVM_ZKGM_BYTES.into()).await;

    let deployed_erc20 = ctx
        .dst
        .deploy_basic_erc20(EVM_ZKGM_BYTES.into(), evm_provider.clone())
        .await
        .expect("failed to deploy ERC20");

    let union_zkgm_contract: Bech32<FixedBytes<32>> = Bech32::from_str(UNION_ZKGM_ADDRESS).unwrap();

    let quote_token_addr = ctx
        .predict_wrapped_token::<cosmos::Module>(
            &ctx.src,
            union_zkgm_contract,
            ChannelId::new(NonZero::new(src_chain_id).unwrap()),
            deployed_erc20.as_ref().to_vec(),
            cosmos_signer,
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
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV1 {
            sender: evm_address.to_vec().into(),
            receiver: cosmos_address_bytes.clone().into(),
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

    let ucs03_zkgm = UCS03Zkgm::new(EVM_ZKGM_BYTES.into(), evm_provider.clone());

    let call = ucs03_zkgm
        .send(
            dst_chain_id,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            instruction_from_evm_to_union.clone(),
        )
        .clear_decoder();

    let recv_packet_data: Result<union_test::helpers::PacketRecv, voyager_sdk::anyhow::Error> = ctx
        .send_and_recv_with_retry::<evm::Module, cosmos::Module>(
            &ctx.dst,
            EVM_ZKGM_BYTES.into(),
            call,
            &ctx.src,
            3,
            Duration::from_secs(20),
            Duration::from_secs(720),
            &evm_provider,
        )
        .await;

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );
    println!(
        "Received packet data from evm->cosmos GOLD token: {:?}",
        recv_packet_data
    );

    let approve_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: UNION_MINTER_ADDRESS.into(),
        amount: "100".parse().unwrap(),
        expires: None,
    };

    let approve_msg_bin: Vec<u8> = Encode::<Json>::encode(&approve_msg);
    let quote_token_bytes = hex_decode(quote_token_addr.trim_start_matches("0x"))
        .expect("invalid quote‐token address hex");

    let approve_contract: Bech32<FixedBytes<32>> =
        Bech32::from_str(std::str::from_utf8(&quote_token_bytes).unwrap()).unwrap();

    println!("Calling approve on quote tokenbytes: {:?}, quote_token:{:?} -> from account: {:?}. Approve contract: {:?}",  quote_token_addr, quote_token_bytes, cosmos_address, approve_contract);

    let approve_recv_packet_data = ctx
        .src
        .send_transaction_with_retry(approve_contract, (approve_msg_bin, vec![]), cosmos_signer)
        .await;

    println!("Approve transaction data: {:?}", approve_recv_packet_data);
    assert!(
        approve_recv_packet_data.is_some(),
        "Failed to send approve transaction: {:?}",
        approve_recv_packet_data
    );

    let instruction_cosmos = Instruction {
        version: INSTR_VERSION_1,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV1 {
            sender: cosmos_address_bytes.clone().into(),
            receiver: evm_address.to_vec().into(),
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
    let union_zkgm_contract: Bech32<FixedBytes<32>> = Bech32::from_str(UNION_ZKGM_ADDRESS).unwrap();

    let recv_packet_data = ctx
        .send_and_recv_with_retry::<cosmos::Module, evm::Module>(
            &ctx.src,
            union_zkgm_contract,
            (bin_msg, funds),
            &ctx.dst,
            3,
            Duration::from_secs(20),
            Duration::from_secs(720),
            cosmos_signer,
        )
        .await;

    println!("Received packet data: {:?}", recv_packet_data);

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );
}

async fn test_send_packet_from_union_to_evm_get_refund() {
    let ctx = init_ctx().await;

    let (evm_address, evm_provider) = ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();

    ensure_channels_opened(ctx.channel_count).await;
    let available_channel = ctx.get_available_channel_count().await;
    assert!(available_channel > 0);
    let pair = ctx.get_channel().await.expect("channel available");

    // let pair = ChannelPair {
    //     src: 1.try_into().unwrap(),
    //     dest: 1.try_into().unwrap(),
    // };

    let dst_chain_id = pair.dest;
    let src_chain_id = pair.src;

    let quote_token_addr = ctx
        .predict_wrapped_token::<evm::Module>(
            &ctx.dst,
            EVM_ZKGM_BYTES.into(),
            ChannelId::new(NonZero::new(dst_chain_id).unwrap()),
            "muno".into(),
            &evm_provider,
        )
        .await
        .unwrap();

    println!("Quote token address: {:?}", quote_token_addr);

    let mut salt_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut salt_bytes);
    let contract: Bech32<FixedBytes<32>> = Bech32::from_str(UNION_ZKGM_ADDRESS).unwrap();
    let sending_amount = "9999999999999999999999";
    let instruction_cosmos = Instruction {
        version: INSTR_VERSION_1,
        opcode: OP_FUNGIBLE_ASSET_ORDER,
        operand: FungibleAssetOrder {
            sender: cosmos_address_bytes.clone().into(),
            receiver: evm_address.to_vec().into(),
            base_token: "muno".as_bytes().into(),
            base_amount: sending_amount.parse().unwrap(),
            base_token_symbol: "muno".into(),
            base_token_name: "muno".into(),
            base_token_decimals: 6,
            base_token_path: "0".parse().unwrap(),
            quote_token: cosmos_address_bytes.clone().into(), // it will revert
            quote_amount: sending_amount.parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let now_secs = now_secs / 1000000000;
    let timeout_timestamp = Timestamp::from_secs(now_secs + 60);

    println!(
        "Timeout now:{}, timeout_timestamp: {:?}",
        now_secs, timeout_timestamp
    );
    let cw_msg = ucs03_zkgm::msg::ExecuteMsg::Send {
        channel_id: src_chain_id.try_into().unwrap(),
        timeout_height: 0u64.into(),
        timeout_timestamp,
        salt: salt_bytes.into(),
        instruction: instruction_cosmos.abi_encode_params().into(),
    };
    let bin_msg: Vec<u8> = Encode::<Json>::encode(&cw_msg);

    let funds = vec![Coin {
        denom: "muno".into(),
        amount: sending_amount.into(),
    }];

    let muno_balance_before_send = ctx
        .src
        .get_balance(&cosmos_address.clone().to_string(), "muno".into())
        .await;
    assert!(
        muno_balance_before_send.is_ok(),
        "Failed to get muno balance before sending: {:?}",
        muno_balance_before_send.err()
    );
    let old_balance = muno_balance_before_send.unwrap().amount;
    println!(
        "Muno balance of {}: before sending: {:?}",
        cosmos_address.clone(),
        old_balance
    );

    let recv_packet_data = ctx
        .send_and_recv_refund_with_timeout::<cosmos::Module, evm::Module>(
            &ctx.src,
            contract,
            (bin_msg, funds),
            &ctx.dst,
            Duration::from_secs(1720),
            cosmos_provider,
        )
        .await;
    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );

    println!(
        "Received packet data from cosmos->evm: {:?}",
        recv_packet_data
    );

    let muno_balance_after_send = ctx
        .src
        .get_balance(&cosmos_address.clone().to_string(), "muno".into())
        .await;
    assert!(
        muno_balance_after_send.is_ok(),
        "Failed to get muno balance after sending: {:?}",
        muno_balance_after_send.err()
    );
    let new_balance = muno_balance_after_send.unwrap().amount;
    println!(
        "Muno balance of {}: after sending: {:?}",
        cosmos_address.clone(),
        new_balance
    );
    // This math is random, but we expect the balance to be greater than or equal to old balance - sending amount
    // because we are sending the packet to the union chain and it will fail on the destination
    // chain, so we should get a refund. But there is also a gas fee. Since sending_amount/2 will be higher than gas fee, it should work.
    // assert!(new_balance + sending_amount.into()/2 > old_balance - sending_amount.into(),
    //     "Muno balance should be greater than or equal to old balance - sending amount, but got: {} instead of: {}",
    //     new_balance, old_balance - sending_amount.into());
}

async fn test_send_packet_from_evm_to_union_get_refund() {
    let ctx = init_ctx().await;
    let (evm_address, evm_provider) = ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_signer) = ctx.src.get_signer().await;

    println!("EVM Address: {:?}", evm_address);
    println!("Cosmos Address: {:?}", cosmos_address);

    ensure_channels_opened(ctx.channel_count).await;

    let available_channel = ctx.get_available_channel_count().await;
    assert!(available_channel > 0);

    let pair = ctx.get_channel().await.expect("channel available");
    let dst_chain_id = pair.dest;
    let src_chain_id = pair.src;

    // let deployed_erc20 = ensure_erc20(EVM_ZKGM_BYTES.into()).await;

    let deployed_erc20 = ctx
        .dst
        .deploy_basic_erc20(EVM_ZKGM_BYTES.into(), evm_provider.clone())
        .await
        .expect("failed to deploy ERC20");

    let union_zkgm_contract: Bech32<FixedBytes<32>> = Bech32::from_str(UNION_ZKGM_ADDRESS).unwrap();

    let quote_token_addr = ctx
        .predict_wrapped_token::<cosmos::Module>(
            &ctx.src,
            union_zkgm_contract,
            ChannelId::new(NonZero::new(src_chain_id).unwrap()),
            deployed_erc20.as_ref().to_vec(),
            cosmos_signer,
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
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV1 {
            sender: evm_address.to_vec().into(),
            receiver: evm_address.to_vec().into(), // Here providing evm_address as receiver on purpose to
            // make recv_packet fail on the destination chain and get refund.
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

    let erc20_balance_before_send = ctx
        .dst
        .zkgmerc20_balance_of(deployed_erc20, evm_address.into(), evm_provider.clone())
        .await;
    assert!(
        erc20_balance_before_send.is_ok(),
        "Failed to get ERC20 balance: {:?}",
        erc20_balance_before_send.err()
    );
    let erc20_balance_before_send = erc20_balance_before_send.unwrap();
    println!(
        "ERC20 balance of {}: {}",
        evm_address, erc20_balance_before_send
    );

    let ucs03_zkgm = UCS03Zkgm::new(EVM_ZKGM_BYTES.into(), evm_provider.clone());

    let call = ucs03_zkgm
        .send(
            dst_chain_id,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            instruction_from_evm_to_union.clone(),
        )
        .clear_decoder();

    let recv_packet_data = ctx
        .send_and_recv_refund::<evm::Module, cosmos::Module>(
            &ctx.dst,
            EVM_ZKGM_BYTES.into(),
            call,
            &ctx.src,
            Duration::from_secs(720),
            &evm_provider,
        )
        .await;

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );
    println!(
        "Received packet data from evm->cosmos GOLD token: {:?}",
        recv_packet_data
    );

    let erc20_balance_after_send = ctx
        .dst
        .zkgmerc20_balance_of(deployed_erc20, evm_address.into(), evm_provider.clone())
        .await;
    assert!(
        erc20_balance_after_send.is_ok(),
        "Failed to get ERC20 balance after send: {:?}",
        erc20_balance_after_send.err()
    );
    let erc20_balance_after_send = erc20_balance_after_send.unwrap();

    println!(
        "ERC20 balance of {} after send: {}",
        evm_address, erc20_balance_after_send
    );

    assert_eq!(
        erc20_balance_before_send, erc20_balance_after_send,
        "ERC20 balance should remain the same after refund"
    );
}

async fn test_stake_from_evm_to_union() {
    let ctx = init_ctx().await;

    let (evm_address, evm_provider) = ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();
    println!("EVM Address: {:?}", evm_address);

    ensure_channels_opened(ctx.channel_count).await;

    let available_channel = ctx.get_available_channel_count().await;
    assert!(available_channel > 0);

    let pair = ctx.get_channel().await.expect("channel available");

    let img_metadata = ucs03_zkgm::com::TokenMetadata {
        implementation: hex!("999709eB04e8A30C7aceD9fd920f7e04EE6B97bA")
            .to_vec()
            .into(),
        initializer: ZkgmERC20::initializeCall {
            _authority: hex!("6C1D11bE06908656D16EBFf5667F1C45372B7c89").into(),
            _minter: EVM_ZKGM_BYTES.into(),
            _name: "muno".into(),
            _symbol: "muno".into(),
            _decimals: 6u8,
        }
        .abi_encode()
        .into(),
    }
    .abi_encode_params();

    let img = keccak256(&img_metadata);

    let governance_token = ctx
        .dst
        .setup_governance_token(EVM_ZKGM_BYTES.into(), pair.dest, img, evm_provider.clone())
        .await;

    assert!(
        governance_token.is_ok(),
        "Failed to setup governance token: {:?}",
        governance_token.err()
    );

    let mut salt_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut salt_bytes);

    let quote_token_addr = ctx
        .predict_wrapped_token_from_metadata_image_v2::<evm::Module>(
            &ctx.dst,
            EVM_ZKGM_BYTES.into(),
            ChannelId::new(NonZero::new(pair.dest).unwrap()),
            "muno".into(),
            img,
            &evm_provider,
        )
        .await
        .unwrap();

    println!("✅ Quote token address: {:?}", quote_token_addr);

    let instruction_cosmos = Instruction {
        version: INSTR_VERSION_2,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV2 {
            sender: cosmos_address_bytes.clone().into(),
            receiver: evm_address.to_vec().into(),
            base_token: "muno".as_bytes().into(),
            base_amount: "10".parse().unwrap(),
            kind: TOKEN_ORDER_KIND_INITIALIZE,
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
            Bech32::from_str(UNION_ZKGM_ADDRESS).unwrap(),
            (bin_msg, funds),
            &ctx.dst,
            3,
            Duration::from_secs(20),
            Duration::from_secs(720),
            cosmos_provider,
        )
        .await;

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );

    println!("Received packet data: {:?}", recv_packet_data);
    println!(
        "Calling approve on quote token: {:?} -> from account: {:?}",
        quote_token_addr, evm_address
    );

    let approve_tx_hash = ctx
        .dst
        .zkgmerc20_approve(
            quote_token_addr,
            EVM_ZKGM_BYTES.into(),
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
            governance_token_wrapped: quote_token_addr.into_bytes().into_vec().into(),
            sender: evm_address.to_vec().into(),
            beneficiary: evm_address.to_vec().into(),
            validator: given_validator.as_bytes().into(),
            amount: "1".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    let ucs03_zkgm = UCS03Zkgm::new(EVM_ZKGM_BYTES.into(), evm_provider.clone());

    rand::rng().fill_bytes(&mut salt_bytes);
    let call = ucs03_zkgm
        .send(
            pair.dest,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            instruction_from_evm_to_union.clone(),
        )
        .clear_decoder();

    let recv_packet_data = ctx
        .send_and_recv_stake::<evm::Module, cosmos::Module>(
            &ctx.dst,
            EVM_ZKGM_BYTES.into(),
            call,
            &ctx.src,
            Duration::from_secs(360),
            given_validator.to_string(),
            evm_provider.clone(),
        )
        .await;

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet for stake request: {:?}",
        recv_packet_data.err()
    );
    println!("Received packet data: {:?}", recv_packet_data);

    let snake_nft = ctx
        .dst
        .predict_stake_manager_address(EVM_ZKGM_BYTES.into(), evm_provider.clone())
        .await;
    assert!(snake_nft.is_ok(), "Failed to predict stake manager address");
    let snake_nft = snake_nft.unwrap();

    println!(
        "✅ Stake manager address: {:?}, random_token_id: {:?}, evm_address: {:?}",
        snake_nft, random_token_id, evm_address
    );

    // Check random_token_id is ours or not now
    let res = ctx
        .dst
        .nft_owner_of(
            snake_nft,
            evm_address.into(),
            random_token_id.into(),
            evm_provider.clone(),
        )
        .await;
    let is_ours: bool = res
        .unwrap_or_else(|e| panic!("Failed to check NFT ownership after stake request: {:?}", e));

    assert!(is_ours, "NFT ownership check returned false");
}

async fn test_stake_from_evm_to_union_and_refund() {
    let ctx = init_ctx().await;

    let (evm_address, evm_provider) = ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();
    println!("EVM Address: {:?}", evm_address);

    ensure_channels_opened(ctx.channel_count).await;

    let available_channel = ctx.get_available_channel_count().await;
    assert!(available_channel > 0);

    let pair = ctx.get_channel().await.expect("channel available");

    let img_metadata = ucs03_zkgm::com::TokenMetadata {
        implementation: hex!("999709eB04e8A30C7aceD9fd920f7e04EE6B97bA")
            .to_vec()
            .into(),
        initializer: ZkgmERC20::initializeCall {
            _authority: hex!("6C1D11bE06908656D16EBFf5667F1C45372B7c89").into(),
            _minter: EVM_ZKGM_BYTES.into(),
            _name: "muno".into(),
            _symbol: "muno".into(),
            _decimals: 6u8,
        }
        .abi_encode()
        .into(),
    }
    .abi_encode_params();

    let img = keccak256(&img_metadata);

    let governance_token = ctx
        .dst
        .setup_governance_token(EVM_ZKGM_BYTES.into(), pair.dest, img, evm_provider.clone())
        .await;

    assert!(
        governance_token.is_ok(),
        "Failed to setup governance token: {:?}",
        governance_token.err()
    );

    let mut salt_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut salt_bytes);

    let quote_token_addr = ctx
        .predict_wrapped_token_from_metadata_image_v2::<evm::Module>(
            &ctx.dst,
            EVM_ZKGM_BYTES.into(),
            ChannelId::new(NonZero::new(pair.dest).unwrap()),
            "muno".into(),
            img,
            &evm_provider,
        )
        .await
        .unwrap();

    println!("✅ Quote token address: {:?}", quote_token_addr);

    let instruction_cosmos = Instruction {
        version: INSTR_VERSION_2,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV2 {
            sender: cosmos_address_bytes.clone().into(),
            receiver: evm_address.to_vec().into(),
            base_token: "muno".as_bytes().into(),
            base_amount: "10".parse().unwrap(),
            kind: TOKEN_ORDER_KIND_INITIALIZE,
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
            Bech32::from_str(UNION_ZKGM_ADDRESS).unwrap(),
            (bin_msg, funds),
            &ctx.dst,
            3,
            Duration::from_secs(20),
            Duration::from_secs(720),
            cosmos_provider,
        )
        .await;

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );

    println!("Received packet data: {:?}", recv_packet_data);
    println!(
        "Calling approve on quote token: {:?} -> from account: {:?}",
        quote_token_addr, evm_address
    );

    let approve_tx_hash = ctx
        .dst
        .zkgmerc20_approve(
            quote_token_addr,
            EVM_ZKGM_BYTES.into(),
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
            governance_token_wrapped: quote_token_addr.into_bytes().into_vec().into(),
            sender: evm_address.to_vec().into(),
            beneficiary: evm_address.to_vec().into(),
            validator: evm_address.to_vec().into(), // Here providing validator address wrong on purpose to
            // make recv_packet fail on the destination chain and get refund.
            amount: "1".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    let ucs03_zkgm = UCS03Zkgm::new(EVM_ZKGM_BYTES.into(), evm_provider.clone());

    let erc20_balance_before_send = ctx
        .dst
        .zkgmerc20_balance_of(quote_token_addr, evm_address.into(), evm_provider.clone())
        .await;

    assert!(
        erc20_balance_before_send.is_ok(),
        "Failed to get ERC20 balance: {:?}",
        erc20_balance_before_send.err()
    );
    let erc20_balance_before_send = erc20_balance_before_send.unwrap();
    println!(
        "ERC20 balance of {}: {}",
        evm_address, erc20_balance_before_send
    );

    rand::rng().fill_bytes(&mut salt_bytes);
    let call = ucs03_zkgm
        .send(
            pair.dest,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            instruction_from_evm_to_union.clone(),
        )
        .clear_decoder();

    let recv_packet_data = ctx
        .send_and_recv_refund::<evm::Module, cosmos::Module>(
            &ctx.dst,
            EVM_ZKGM_BYTES.into(),
            call,
            &ctx.src,
            Duration::from_secs(360),
            &evm_provider,
        )
        .await;

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet for stake request: {:?}",
        recv_packet_data.err()
    );
    println!("Received packet data: {:?}", recv_packet_data);

    let erc20_balance_after_send = ctx
        .dst
        .zkgmerc20_balance_of(quote_token_addr, evm_address.into(), evm_provider.clone())
        .await;

    assert!(
        erc20_balance_after_send.is_ok(),
        "Failed to get ERC20 balance: {:?}",
        erc20_balance_after_send.err()
    );
    let erc20_balance_after_send = erc20_balance_after_send.unwrap();
    println!(
        "ERC20 balance of {} after send: {}",
        evm_address, erc20_balance_after_send
    );

    assert_eq!(
        erc20_balance_before_send, erc20_balance_after_send,
        "ERC20 balance should remain the same after refund"
    );
}

async fn test_stake_and_unstake_from_evm_to_union() {
    let ctx = init_ctx().await;

    let (evm_address, evm_provider) = ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();
    println!("EVM Address: {:?}", evm_address);

    ensure_channels_opened(ctx.channel_count).await;

    let available_channel = ctx.get_available_channel_count().await;
    assert!(available_channel > 0);

    let pair = ctx.get_channel().await.expect("channel available");

    let img_metadata = ucs03_zkgm::com::TokenMetadata {
        implementation: hex!("999709eB04e8A30C7aceD9fd920f7e04EE6B97bA")
            .to_vec()
            .into(),
        initializer: ZkgmERC20::initializeCall {
            _authority: hex!("6C1D11bE06908656D16EBFf5667F1C45372B7c89").into(),
            _minter: EVM_ZKGM_BYTES.into(),
            _name: "muno".into(),
            _symbol: "muno".into(),
            _decimals: 6u8,
        }
        .abi_encode()
        .into(),
    }
    .abi_encode_params();

    let img = keccak256(&img_metadata);

    let governance_token = ctx
        .dst
        .setup_governance_token(EVM_ZKGM_BYTES.into(), pair.dest, img, evm_provider.clone())
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
            EVM_ZKGM_BYTES.into(),
            ChannelId::new(NonZero::new(pair.dest).unwrap()),
            "muno".into(),
            img,
            &evm_provider,
        )
        .await
        .unwrap();

    println!("✅ Quote token address: {:?}", quote_token_addr);

    let instruction_cosmos = Instruction {
        version: INSTR_VERSION_2,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV2 {
            sender: cosmos_address_bytes.clone().into(),
            receiver: evm_address.to_vec().into(),
            base_token: "muno".as_bytes().into(),
            base_amount: "10".parse().unwrap(),
            kind: TOKEN_ORDER_KIND_INITIALIZE,
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
            Bech32::from_str(UNION_ZKGM_ADDRESS).unwrap(),
            (bin_msg, funds),
            &ctx.dst,
            3,
            Duration::from_secs(20),
            Duration::from_secs(720),
            cosmos_provider,
        )
        .await;

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );

    println!("Received packet data: {:?}", recv_packet_data);
    println!(
        "Calling approve on quote token: {:?} -> from account: {:?}",
        quote_token_addr, evm_address
    );

    let approve_tx_hash = ctx
        .dst
        .zkgmerc20_approve(
            quote_token_addr,
            EVM_ZKGM_BYTES.into(),
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
            governance_token_wrapped: quote_token_addr.into_bytes().into_vec().into(),
            sender: evm_address.to_vec().into(),
            beneficiary: evm_address.to_vec().into(),
            validator: given_validator.as_bytes().into(),
            amount: "1".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    let ucs03_zkgm = UCS03Zkgm::new(EVM_ZKGM_BYTES.into(), evm_provider.clone());

    rand::rng().fill_bytes(&mut salt_bytes);
    let call = ucs03_zkgm
        .send(
            pair.dest,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            instruction_from_evm_to_union.clone(),
        )
        .clear_decoder();
    // let call = call.with_cloned_provider();
    let recv_packet_data = ctx
        .send_and_recv_stake::<evm::Module, cosmos::Module>(
            &ctx.dst,
            EVM_ZKGM_BYTES.into(),
            call,
            &ctx.src,
            Duration::from_secs(360),
            given_validator.to_string(),
            evm_provider.clone(),
        )
        .await;

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet for stake request: {:?}",
        recv_packet_data.err()
    );
    println!("Received packet data: {:?}", recv_packet_data);

    let snake_nft = ctx
        .dst
        .predict_stake_manager_address(EVM_ZKGM_BYTES.into(), evm_provider.clone())
        .await;
    assert!(snake_nft.is_ok(), "Failed to predict stake manager address");
    let snake_nft = snake_nft.unwrap();

    println!(
        "✅ Stake manager address: {:?}, random_token_id: {:?}, evm_address: {:?}",
        snake_nft, random_token_id, evm_address
    );

    // Check random_token_id is ours or not now
    let res = ctx
        .dst
        .nft_owner_of(
            snake_nft,
            evm_address.into(),
            random_token_id.into(),
            evm_provider.clone(),
        )
        .await;
    let is_ours: bool = res
        .unwrap_or_else(|e| panic!("Failed to check NFT ownership after stake request: {:?}", e));

    assert!(is_ours, "NFT ownership check returned false");

    let approve_tx_hash = ctx
        .dst
        .zkgmerc721_approve(
            snake_nft,
            EVM_ZKGM_BYTES.into(),
            random_token_id.into(),
            evm_provider.clone(),
        )
        .await;

    assert!(
        approve_tx_hash.is_ok(),
        "Failed to send approve transaction for NFT: {:?}, from_account: {:?}",
        approve_tx_hash.err(),
        evm_address
    );
    println!("✅ Approve tx hash: {:?}", approve_tx_hash);

    let instruction_unstake = InstructionEvm {
        version: INSTR_VERSION_0,
        opcode: OP_UNSTAKE,
        operand: Unstake {
            token_id: random_token_id,
            governance_token: b"muno".into(),
            governance_token_wrapped: quote_token_addr.into_bytes().into_vec().into(),
            sender: evm_address.to_vec().into(),
            validator: given_validator.as_bytes().into(),
        }
        .abi_encode_params()
        .into(),
    };

    rand::rng().fill_bytes(&mut salt_bytes);
    let call_unstake = ucs03_zkgm
        .send(
            pair.dest,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            instruction_unstake.clone(),
        )
        .clear_decoder();

    let recv_unstake = ctx
        .send_and_recv_unstake::<evm::Module, cosmos::Module>(
            &ctx.dst,
            EVM_ZKGM_BYTES.into(),
            call_unstake,
            &ctx.src,
            Duration::from_secs(360),
            given_validator.to_string(),
            evm_provider,
        )
        .await;

    println!("Received packet data: {:?}", recv_unstake);
}

async fn test_stake_unstake_and_withdraw_from_evm_to_union() {
    let ctx = init_ctx().await;

    let (evm_address, evm_provider) = ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();
    println!("EVM Address: {:?}", evm_address);

    ensure_channels_opened(ctx.channel_count).await;

    let available_channel = ctx.get_available_channel_count().await;
    assert!(available_channel > 0);

    let pair = ctx.get_channel().await.expect("channel available");

    let img_metadata = ucs03_zkgm::com::TokenMetadata {
        implementation: hex!("999709eB04e8A30C7aceD9fd920f7e04EE6B97bA")
            .to_vec()
            .into(),
        initializer: ZkgmERC20::initializeCall {
            _authority: hex!("6C1D11bE06908656D16EBFf5667F1C45372B7c89").into(),
            _minter: EVM_ZKGM_BYTES.into(),
            _name: "muno".into(),
            _symbol: "muno".into(),
            _decimals: 6u8,
        }
        .abi_encode()
        .into(),
    }
    .abi_encode_params();

    let img = keccak256(&img_metadata);
    let governance_token = ctx
        .dst
        .setup_governance_token(EVM_ZKGM_BYTES.into(), pair.dest, img, evm_provider.clone())
        .await;

    assert!(
        governance_token.is_ok(),
        "Failed to setup governance token: {:?}",
        governance_token.err()
    );

    let mut salt_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut salt_bytes);

    let quote_token_addr = ctx
        .predict_wrapped_token_from_metadata_image_v2::<evm::Module>(
            &ctx.dst,
            EVM_ZKGM_BYTES.into(),
            ChannelId::new(NonZero::new(pair.dest).unwrap()),
            "muno".into(),
            img,
            &evm_provider,
        )
        .await
        .unwrap();

    println!("✅ Quote token address: {:?}", quote_token_addr);

    let instruction_cosmos = Instruction {
        version: INSTR_VERSION_2,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV2 {
            sender: cosmos_address_bytes.clone().into(),
            receiver: evm_address.to_vec().into(),
            base_token: "muno".as_bytes().into(),
            base_amount: "10".parse().unwrap(),
            kind: TOKEN_ORDER_KIND_INITIALIZE,
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
            Bech32::from_str(UNION_ZKGM_ADDRESS).unwrap(),
            (bin_msg, funds),
            &ctx.dst,
            3,
            Duration::from_secs(20),
            Duration::from_secs(720),
            cosmos_provider,
        )
        .await;

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );

    println!("Received packet data: {:?}", recv_packet_data);
    println!(
        "Calling approve on quote token: {:?} -> from account: {:?}",
        quote_token_addr, evm_address
    );

    let approve_tx_hash = ctx
        .dst
        .zkgmerc20_approve(
            quote_token_addr,
            EVM_ZKGM_BYTES.into(),
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
            governance_token_wrapped: quote_token_addr.into_bytes().into_vec().into(),
            sender: evm_address.to_vec().into(),
            beneficiary: evm_address.to_vec().into(),
            validator: given_validator.as_bytes().into(),
            amount: "1".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    let ucs03_zkgm = UCS03Zkgm::new(EVM_ZKGM_BYTES.into(), evm_provider.clone());

    rand::rng().fill_bytes(&mut salt_bytes);
    let call = ucs03_zkgm
        .send(
            pair.dest,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            instruction_from_evm_to_union.clone(),
        )
        .clear_decoder();

    let recv_packet_data = ctx
        .send_and_recv_stake::<evm::Module, cosmos::Module>(
            &ctx.dst,
            EVM_ZKGM_BYTES.into(),
            call,
            &ctx.src,
            Duration::from_secs(360),
            given_validator.to_string(),
            evm_provider.clone(),
        )
        .await;

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet for stake request: {:?}",
        recv_packet_data.err()
    );
    println!("Received packet data: {:?}", recv_packet_data);

    let snake_nft = ctx
        .dst
        .predict_stake_manager_address(EVM_ZKGM_BYTES.into(), evm_provider.clone())
        .await;

    assert!(snake_nft.is_ok(), "Failed to predict stake manager address");
    let snake_nft = snake_nft.unwrap();

    println!(
        "✅ Stake manager address: {:?}, random_token_id: {:?}, evm_address: {:?}",
        snake_nft, random_token_id, evm_address
    );

    // Check random_token_id is ours or not now
    let res = ctx
        .dst
        .nft_owner_of(
            snake_nft,
            evm_address.into(),
            random_token_id.into(),
            evm_provider.clone(),
        )
        .await;
    let is_ours: bool = res
        .unwrap_or_else(|e| panic!("Failed to check NFT ownership after stake request: {:?}", e));

    assert!(is_ours, "NFT ownership check returned false");

    println!(
        "Calling approve on NFT: {:?} -> from account: {:?}",
        snake_nft, evm_address
    );

    let approve_tx_hash = ctx
        .dst
        .zkgmerc721_approve(
            snake_nft,
            EVM_ZKGM_BYTES.into(),
            random_token_id.into(),
            evm_provider.clone(),
        )
        .await;

    assert!(
        approve_tx_hash.is_ok(),
        "Failed to send approve transaction for NFT: {:?}, from_account: {:?}",
        approve_tx_hash.err(),
        evm_address
    );
    println!("✅ Approve tx hash: {:?}", approve_tx_hash);

    tokio::time::sleep(Duration::from_secs(60)).await; // Wait for 1 minute before unstake

    let instruction_unstake = InstructionEvm {
        version: INSTR_VERSION_0,
        opcode: OP_UNSTAKE,
        operand: Unstake {
            token_id: random_token_id,
            governance_token: b"muno".into(),
            governance_token_wrapped: quote_token_addr.into_bytes().into_vec().into(),
            sender: evm_address.to_vec().into(),
            validator: given_validator.as_bytes().into(),
        }
        .abi_encode_params()
        .into(),
    };

    rand::rng().fill_bytes(&mut salt_bytes);
    let call_unstake = ucs03_zkgm
        .send(
            pair.dest,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            instruction_unstake.clone(),
        )
        .clear_decoder();

    let recv_unstake = ctx
        .send_and_recv_unstake::<evm::Module, cosmos::Module>(
            &ctx.dst,
            EVM_ZKGM_BYTES.into(),
            call_unstake,
            &ctx.src,
            Duration::from_secs(360),
            given_validator.to_string(),
            evm_provider.clone(),
        )
        .await;

    println!("Received packet data: {:?}", recv_unstake);

    tokio::time::sleep(Duration::from_secs(180)).await; // 2min is the withdraw min time, waiting 3m just to be sure

    let approve_tx_hash = ctx
        .dst
        .zkgmerc721_approve(
            snake_nft,
            EVM_ZKGM_BYTES.into(),
            random_token_id.into(),
            evm_provider.clone(),
        )
        .await;

    assert!(
        approve_tx_hash.is_ok(),
        "Failed to send approve transaction for NFT: {:?}, from_account: {:?}",
        approve_tx_hash.err(),
        evm_address
    );

    let instruction_withdraw = InstructionEvm {
        version: INSTR_VERSION_0,
        opcode: OP_WITHDRAW_STAKE,
        operand: WithdrawStake {
            token_id: random_token_id,
            governance_token: b"muno".into(),
            governance_token_wrapped: quote_token_addr.into_bytes().into_vec().into(),
            sender: evm_address.to_vec().into(),
            beneficiary: evm_address.to_vec().into(),
        }
        .abi_encode_params()
        .into(),
    };

    rand::rng().fill_bytes(&mut salt_bytes);
    let call_unstake = ucs03_zkgm
        .send(
            pair.dest,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            instruction_withdraw.clone(),
        )
        .clear_decoder();

    let recv_withdraw = ctx
        .send_and_recv_withdraw::<evm::Module, cosmos::Module>(
            &ctx.dst,
            EVM_ZKGM_BYTES.into(),
            call_unstake,
            &ctx.src,
            Duration::from_secs(360),
            evm_provider,
        )
        .await;

    assert!(
        recv_withdraw.is_ok(),
        "Failed to send and receive packet for withdraw request: {:?}",
        recv_withdraw.err()
    );
    println!("Received packet data for withdraw: {:?}", recv_withdraw);
}

#[tokio::test]
async fn send_stake_and_unstake_from_evm_to_union0() {
    self::test_stake_and_unstake_from_evm_to_union().await;
}

#[tokio::test]
async fn send_stake_unstake_and_withdraw_from_evm_to_union0() {
    self::test_stake_unstake_and_withdraw_from_evm_to_union().await;
}

#[tokio::test]
async fn from_evm_to_union0() {
    self::test_send_packet_from_evm_to_union_and_send_back_unwrap().await;
}

#[tokio::test]
async fn from_evm_to_union_refund() {
    self::test_send_packet_from_evm_to_union_get_refund().await;
}

#[tokio::test] // Note: For this one to work; timeout plugin should be enabled on voyager.
async fn from_union_to_evm_refund() {
    // TODO: Fix it later. Refund is not happening correctly.
    self::test_send_packet_from_union_to_evm_get_refund().await;
}

#[tokio::test]
async fn from_union_to_evm0() {
    self::test_send_packet_from_union_to_evm_and_send_back_unwrap().await;
}

#[tokio::test]
async fn from_evm_to_union_stake0() {
    self::test_stake_from_evm_to_union().await;
}

#[tokio::test]
async fn from_evm_to_union_stake_and_refund() {
    self::test_stake_from_evm_to_union_and_refund().await;
}

#[tokio::test]
async fn test_vault_works() {
    self::test_send_vault_success().await;
}
