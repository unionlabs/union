use std::{
    num::NonZero,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use alloy::{
    hex::decode as hex_decode,
    sol_types::{SolCall, SolValue},
};
use cosmwasm_std::Addr;
use cw20::Cw20ExecuteMsg;
use hex_literal::hex;
use ibc_union_spec::ChannelId;
use protos::cosmos::base::v1beta1::Coin;
use rand::RngCore;
use serde::Deserialize;
use tokio::sync::OnceCell;
use ucs03_zkgm::{
    self,
    com::{
        INSTR_VERSION_0, INSTR_VERSION_1, INSTR_VERSION_2, Instruction, OP_BATCH, OP_FORWARD,
        OP_TOKEN_ORDER, SolverMetadata, TOKEN_ORDER_KIND_ESCROW, TOKEN_ORDER_KIND_INITIALIZE,
        TOKEN_ORDER_KIND_SOLVE, TOKEN_ORDER_KIND_UNESCROW, TokenOrderV1, TokenOrderV2,
    },
};
use union_test::{
    TestContext,
    cosmos::{self},
    cosmos_helpers::{SALT_ESCROW_VAULT, SALT_ZKGM, calculate_cosmos_contract_address},
    evm::{
        self,
        zkgm::{
            IBC, IBCPacket, Instruction as InstructionEvm, MsgPacketRecv, UCS03Zkgm, ZkgmPacket,
        },
        zkgmerc20::ZkgmERC20,
    },
};
use unionlabs::{
    encoding::{Encode, Json},
    ethereum::keccak256,
    primitives::{H160, U256},
};
use voyager_sdk::{primitives::Timestamp, serde_json};

static CTX: OnceCell<Arc<ZkgmCtx>> = OnceCell::const_new();
static CHANNELS_OPENED: OnceCell<()> = OnceCell::const_new();

pub const ETH_ADDRESS_U: H160 = H160::new(hex!("0c8C6f58156D10d18193A8fFdD853e1b9F8D8836"));
pub const ETH_ADDRESS_ZKGM: H160 = H160::new(hex!("05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5"));
pub const ETH_ADDRESS_IBC: H160 = H160::new(hex!("ed2af2aD7FE0D92011b26A2e5D1B4dC7D12A47C5"));

#[derive(Deserialize)]
pub struct Config {
    evm: evm::Config,
    union: cosmos::Config,
    needed_channel_count: u32,
    voyager_config_file_path: String,
    union_deployer_addr: String,
}

pub struct UnionAddressBook {
    pub zkgm: Addr,
    pub escrow_vault: Addr,
}

pub struct ZkgmCtx {
    pub union_address: UnionAddressBook,
    pub ctx: TestContext<cosmos::Module, evm::Module>,
}

async fn init_ctx() -> Arc<ZkgmCtx> {
    CTX.get_or_init(|| async {
        let cfg: Config = serde_json::from_str(include_str!("./config.json")).unwrap();

        let src = cosmos::Module::new(cfg.union).await.unwrap();
        let dst = evm::Module::new(cfg.evm).await.unwrap();

        let ctx = ZkgmCtx {
            ctx: TestContext::new(
                src,
                dst,
                cfg.needed_channel_count as usize,
                &cfg.voyager_config_file_path,
            )
            .await
            .unwrap(),
            union_address: UnionAddressBook {
                zkgm: calculate_cosmos_contract_address(&cfg.union_deployer_addr, SALT_ZKGM)
                    .unwrap(),
                escrow_vault: calculate_cosmos_contract_address(
                    &cfg.union_deployer_addr,
                    SALT_ESCROW_VAULT,
                )
                .unwrap(),
            },
        };

        Arc::new(ctx)
    })
    .await
    .clone()
}

async fn ensure_channels_opened(channel_count: usize) {
    CHANNELS_OPENED
        .get_or_init(|| async move {
            let t = init_ctx().await;

            let (src_client, dst_client) = t
                .ctx
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

            let conn = t
                .ctx
                .open_connection::<cosmos::Module, evm::Module>(
                    &t.ctx.src,
                    src_client.client_id,
                    &t.ctx.dst,
                    dst_client.client_id,
                    Duration::from_secs(180),
                )
                .await
                .unwrap();
            assert!(conn.connection_id > 0);
            assert!(conn.counterparty_connection_id > 0);

            let current_available_count = t.ctx.get_available_channel_count().await;

            let opened = t
                .ctx
                .open_channels(
                    true,
                    t.union_address.zkgm.as_bytes().into(),
                    ETH_ADDRESS_ZKGM.into(),
                    conn.counterparty_connection_id,
                    "ucs03-zkgm-0".into(),
                    channel_count,
                    Duration::from_secs(360 * channel_count as u64),
                )
                .await
                .unwrap();
            assert_eq!(opened, channel_count);

            let available_count_after_open = t.ctx.get_available_channel_count().await;
            assert_eq!(
                current_available_count + channel_count,
                available_count_after_open
            );
            let pair = t.ctx.get_channel().await.expect("channel available");
            let available_count_after_get = t.ctx.get_available_channel_count().await;
            assert_eq!(available_count_after_open - 1, available_count_after_get);
            t.ctx.release_channel(pair).await;
            let available_count_after_release = t.ctx.get_available_channel_count().await;
            assert_eq!(available_count_after_open, available_count_after_release);
        })
        .await;
}

#[tokio::test]
async fn test_send_vault_success() {
    let t = init_ctx().await;

    let (evm_address, evm_provider) = t.ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = t.ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();

    ensure_channels_opened(t.ctx.channel_count).await;
    let available_channel = t.ctx.get_available_channel_count().await;
    assert!(available_channel > 0);
    let pair = t.ctx.get_channel().await.expect("channel available");

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
            base_token: "au".as_bytes().into(),
            base_amount: "10".parse().unwrap(),
            kind: TOKEN_ORDER_KIND_SOLVE,
            metadata: metadata.into(),
            quote_token: u_on_eth.to_vec().into(),
            quote_amount: "10".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    let (_, zkgm_deployer_provider) = t.ctx.dst.get_provider_privileged().await;
    t.ctx
        .dst
        .u_register_fungible_counterpart(
            H160::from(u_on_eth),
            zkgm_deployer_provider.clone(),
            alloy::primitives::U256::ZERO,
            dst_channel_id,
            b"au".to_vec().into(),
            evm::u::U::FungibleCounterparty {
                beneficiary: vault_on_union.as_bytes().to_vec().into(),
            },
        )
        .await
        .unwrap();

    let mut salt_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt_bytes);

    let cw_msg = ucs03_zkgm::msg::ExecuteMsg::Send {
        channel_id: src_channel_id.try_into().unwrap(),
        timeout_height: 0u64.into(),
        timeout_timestamp: voyager_sdk::primitives::Timestamp::from_secs(u32::MAX.into()),
        salt: salt_bytes.into(),
        instruction: instruction_cosmos.abi_encode_params().into(),
    };
    let bin_msg: Vec<u8> = Encode::<Json>::encode(&cw_msg);

    let funds = vec![Coin {
        denom: "au".into(),
        amount: "10".into(),
    }];

    let initial_u_balance = t
        .ctx
        .dst
        .zkgmerc20_balance_of(
            H160::from(u_on_eth),
            evm_address.into(),
            evm_provider.clone(),
        )
        .await
        .unwrap();

    let initial_vault_balance = t
        .ctx
        .src
        .native_balance(t.union_address.escrow_vault.clone(), "au")
        .await
        .unwrap();

    println!("initial U balance on eth: {initial_u_balance}");
    println!("initial U balance on union vault: {initial_vault_balance}");

    let ack_packet_data = t
        .ctx
        .send_and_recv_and_ack_with_retry::<cosmos::Module, evm::Module>(
            &t.ctx.src,
            t.union_address.zkgm.clone(),
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
        .zkgmerc20_balance_of(ETH_ADDRESS_U, evm_address.into(), evm_provider.clone())
        .await
        .unwrap();

    let new_vault_balance = t
        .ctx
        .src
        .native_balance(t.union_address.escrow_vault.clone(), "au")
        .await
        .unwrap();

    println!("new U balance on eth: {new_u_balance}");
    println!("new U balance on union vault: {new_vault_balance}");

    assert_eq!(new_u_balance - initial_u_balance, 10u64.into());
    assert_eq!(new_vault_balance - initial_vault_balance, 10);
}

#[tokio::test]
async fn test_send_vault_success_with_fee() {
    let t = init_ctx().await;

    let (evm_address, evm_provider) = t.ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = t.ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();

    ensure_channels_opened(t.ctx.channel_count).await;
    let available_channel = t.ctx.get_available_channel_count().await;
    assert!(available_channel > 0);
    let pair = t.ctx.get_channel().await.expect("channel available");

    let dst_channel_id = pair.dest;
    let src_channel_id = pair.src;

    let vault_on_union = "union1skg5244hpkad603zz77kdekzw6ffgpfrde3ldk8rpdz06n62k4hqct0w4j";

    let u_on_eth = hex_literal::hex!("0c8C6f58156D10d18193A8fFdD853e1b9F8D8836");

    let metadata = SolverMetadata {
        solverAddress: u_on_eth.to_vec().into(),
        metadata: Default::default(),
    }
    .abi_encode_params();

    let recv_addr = hex_literal::hex!("Dab9b2F47515d2e76DC5F96a5901Ba1a0a018975");
    let instruction_cosmos = Instruction {
        version: INSTR_VERSION_2,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV2 {
            sender: cosmos_address_bytes.clone().into(),
            receiver: recv_addr.to_vec().into(),
            base_token: "au".as_bytes().into(),
            base_amount: "15".parse().unwrap(), // So fee will be 5 and will be minted to relayer
            kind: TOKEN_ORDER_KIND_SOLVE,
            metadata: metadata.into(),
            quote_token: u_on_eth.to_vec().into(),
            quote_amount: "10".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    let (_, zkgm_deployer_provider) = t.ctx.dst.get_provider_privileged().await;
    t.ctx
        .dst
        .u_register_fungible_counterpart(
            H160::from(u_on_eth),
            zkgm_deployer_provider.clone(),
            alloy::primitives::U256::ZERO,
            dst_channel_id,
            b"au".to_vec().into(),
            evm::u::U::FungibleCounterparty {
                beneficiary: vault_on_union.as_bytes().to_vec().into(),
            },
        )
        .await
        .unwrap();

    let mut salt_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt_bytes);

    let cw_msg = ucs03_zkgm::msg::ExecuteMsg::Send {
        channel_id: src_channel_id.try_into().unwrap(),
        timeout_height: 0u64.into(),
        timeout_timestamp: voyager_sdk::primitives::Timestamp::from_secs(u32::MAX.into()),
        salt: salt_bytes.into(),
        instruction: instruction_cosmos.abi_encode_params().into(),
    };
    let bin_msg: Vec<u8> = Encode::<Json>::encode(&cw_msg);

    let funds = vec![Coin {
        denom: "au".into(),
        amount: "15".into(), // So fee will be 5 and will be minted to relayer
    }];

    let initial_u_balance = t
        .ctx
        .dst
        .zkgmerc20_balance_of(ETH_ADDRESS_U, recv_addr.into(), evm_provider.clone())
        .await
        .unwrap();

    let initial_vault_balance = t
        .ctx
        .src
        .native_balance(t.union_address.escrow_vault.clone(), "au")
        .await
        .unwrap();

    let initial_balance_of_relayer = t
        .ctx
        .dst
        .zkgmerc20_balance_of(ETH_ADDRESS_U, evm_address.into(), evm_provider.clone())
        .await
        .unwrap();

    println!("initial U balance on eth: {initial_u_balance}");
    println!("initial U balance on union vault: {initial_vault_balance}");
    println!("initial balance of relayer: {initial_balance_of_relayer}");

    let ack_packet_data = t
        .ctx
        .send_and_recv_and_ack_with_retry::<cosmos::Module, evm::Module>(
            &t.ctx.src,
            t.union_address.zkgm.clone(),
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
        .zkgmerc20_balance_of(ETH_ADDRESS_U, recv_addr.into(), evm_provider.clone())
        .await
        .unwrap();

    let new_vault_balance = t
        .ctx
        .src
        .native_balance(t.union_address.escrow_vault.clone(), "au")
        .await
        .unwrap();

    let new_balance_of_relayer = t
        .ctx
        .dst
        .zkgmerc20_balance_of(
            H160::from(u_on_eth),
            evm_address.into(),
            evm_provider.clone(),
        )
        .await
        .unwrap();

    println!("new U balance on eth: {new_u_balance}");
    println!("new U balance on union vault: {new_vault_balance}");
    println!("new balance of relayer: {new_balance_of_relayer}");

    assert_eq!(new_u_balance - initial_u_balance, 10u64.into());
    assert_eq!(new_vault_balance - initial_vault_balance, 15u64.into());
    assert_eq!(
        new_balance_of_relayer - initial_balance_of_relayer,
        5u64.into()
    );
}

#[tokio::test]
async fn test_send_packet_from_union_to_evm_and_send_back_unwrap() {
    let t = init_ctx().await;
    ensure_channels_opened(t.ctx.channel_count).await;

    let (evm_address, evm_provider) = t.ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = t.ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();

    let available_channel = t.ctx.get_available_channel_count().await;
    assert!(available_channel > 0);

    let pair = t.ctx.get_channel().await.expect("channel available");
    let dst_chain_id = pair.dest;
    let src_chain_id = pair.src;

    let quote_token_addr = t
        .ctx
        .predict_wrapped_token::<evm::Module>(
            &t.ctx.dst,
            ETH_ADDRESS_ZKGM,
            ChannelId::new(NonZero::new(dst_chain_id).unwrap()),
            "au".into(),
            &evm_provider,
        )
        .await
        .unwrap();

    let mut salt_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt_bytes);

    let instruction_cosmos = Instruction {
        version: INSTR_VERSION_1,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV1 {
            sender: cosmos_address_bytes.clone().into(),
            receiver: evm_address.to_vec().into(),
            base_token: "au".as_bytes().into(),
            base_amount: "10".parse().unwrap(),
            base_token_symbol: "au".into(),
            base_token_name: "au".into(),
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
        denom: "au".into(),
        amount: "10".into(),
    }];

    let recv_packet_data = t
        .ctx
        .send_and_recv_with_retry::<cosmos::Module, evm::Module>(
            &t.ctx.src,
            t.union_address.zkgm.clone(),
            (bin_msg, funds),
            &t.ctx.dst,
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

    rand::thread_rng().fill_bytes(&mut salt_bytes);

    let instruction_from_evm_to_union = InstructionEvm {
        version: INSTR_VERSION_1,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV1 {
            sender: evm_address.to_vec().into(),
            receiver: cosmos_address_bytes.clone().into(),
            base_token: quote_token_addr.as_ref().to_vec().into(),
            base_amount: "1".parse().unwrap(),
            base_token_symbol: "au".into(),
            base_token_name: "au".into(),
            base_token_decimals: 6,
            base_token_path: dst_chain_id.try_into().unwrap(),
            quote_token: "au".into(),
            quote_amount: "1".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    println!("quote_token:  {:?}", quote_token_addr);

    let ucs03_zkgm = UCS03Zkgm::new(ETH_ADDRESS_ZKGM.into(), evm_provider.clone());

    let call = ucs03_zkgm
        .send(
            dst_chain_id,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            instruction_from_evm_to_union.clone(),
        )
        .clear_decoder()
        .with_cloned_provider();

    let recv_packet_data = t
        .ctx
        .send_and_recv_with_retry::<evm::Module, cosmos::Module>(
            &t.ctx.dst,
            ETH_ADDRESS_ZKGM,
            call,
            &t.ctx.src,
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

#[tokio::test]
async fn test_send_packet_from_evm_to_union_and_send_back_unwrap() {
    let t = init_ctx().await;
    let (evm_address, evm_provider) = t.ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_signer) = t.ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();

    println!("EVM Address: {:?}", evm_address);
    println!("Cosmos Address: {:?}", cosmos_address);

    ensure_channels_opened(t.ctx.channel_count).await;
    let available_channel = t.ctx.get_available_channel_count().await;
    assert!(available_channel > 0);
    let pair = t.ctx.get_channel().await.expect("channel available");

    let dst_chain_id = pair.dest;
    let src_chain_id = pair.src;

    // let deployed_erc20 = ensure_erc20(EVM_ZKGM_BYTES.into()).await;

    let deployed_erc20 = t
        .ctx
        .dst
        .deploy_basic_erc20(ETH_ADDRESS_ZKGM, evm_provider.clone())
        .await
        .expect("failed to deploy ERC20");

    let quote_token_addr = t
        .ctx
        .predict_wrapped_token::<cosmos::Module>(
            &t.ctx.src,
            t.union_address.zkgm.clone(),
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
    rand::thread_rng().fill_bytes(&mut salt_bytes);

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

    let ucs03_zkgm = UCS03Zkgm::new(ETH_ADDRESS_ZKGM.into(), evm_provider.clone());

    let call = ucs03_zkgm
        .send(
            dst_chain_id,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            instruction_from_evm_to_union.clone(),
        )
        .clear_decoder()
        .with_cloned_provider();

    let recv_packet_data: Result<union_test::helpers::PacketRecv, voyager_sdk::anyhow::Error> = t
        .ctx
        .send_and_recv_with_retry::<evm::Module, cosmos::Module>(
            &t.ctx.dst,
            ETH_ADDRESS_ZKGM,
            call,
            &t.ctx.src,
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

    let get_minter_result = t
        .ctx
        .src
        .get_minter(t.union_address.zkgm.clone())
        .await
        .expect("failed to get minter address");

    let approve_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: get_minter_result,
        amount: "100".parse().unwrap(),
        expires: None,
    };

    let approve_msg_bin: Vec<u8> = Encode::<Json>::encode(&approve_msg);
    let quote_token_bytes = hex_decode(quote_token_addr.trim_start_matches("0x"))
        .expect("invalid quote‐token address hex");

    let approve_contract = Addr::unchecked(str::from_utf8(&quote_token_bytes).unwrap());

    println!(
        "Calling approve on quote tokenbytes: {:?}, quote_token:{:?} -> from account: {:?}. Approve contract: {:?}",
        quote_token_addr, quote_token_bytes, cosmos_address, approve_contract
    );

    let approve_recv_packet_data = t
        .ctx
        .src
        .send_cosmwasm_transaction_with_retry(
            approve_contract,
            (approve_msg_bin, vec![]),
            cosmos_signer,
        )
        .await;

    // println!("Approve transaction data: {:?}", approve_recv_packet_data);
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

    let recv_packet_data = t
        .ctx
        .send_and_recv_with_retry::<cosmos::Module, evm::Module>(
            &t.ctx.src,
            t.union_address.zkgm.clone(),
            (bin_msg, funds),
            &t.ctx.dst,
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

#[tokio::test]
async fn test_send_packet_from_union_to_evm_get_refund() {
    let t = init_ctx().await;

    let (evm_address, evm_provider) = t.ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = t.ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();

    ensure_channels_opened(t.ctx.channel_count).await;
    let available_channel = t.ctx.get_available_channel_count().await;
    assert!(available_channel > 0);
    let pair = t.ctx.get_channel().await.expect("channel available");

    let dst_chain_id = pair.dest;
    let src_chain_id = pair.src;

    let quote_token_addr = t
        .ctx
        .predict_wrapped_token::<evm::Module>(
            &t.ctx.dst,
            ETH_ADDRESS_ZKGM,
            ChannelId::new(NonZero::new(dst_chain_id).unwrap()),
            "au".into(),
            &evm_provider,
        )
        .await
        .unwrap();

    println!("Quote token address: {:?}", quote_token_addr);

    let mut salt_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt_bytes);
    let sending_amount = "9999999999999999999999";
    let instruction_cosmos = Instruction {
        version: INSTR_VERSION_1,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV1 {
            sender: cosmos_address_bytes.clone().into(),
            receiver: evm_address.to_vec().into(),
            base_token: "au".as_bytes().into(),
            base_amount: sending_amount.parse().unwrap(),
            base_token_symbol: "au".into(),
            base_token_name: "au".into(),
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
        denom: "au".into(),
        amount: sending_amount.into(),
    }];

    let muno_balance_before_send = t
        .ctx
        .src
        .get_balance(&cosmos_address.clone().to_string(), "au")
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

    let recv_packet_data = t
        .ctx
        .send_and_recv_refund_with_timeout::<cosmos::Module, evm::Module>(
            &t.ctx.src,
            t.union_address.zkgm.clone(),
            (bin_msg, funds),
            &t.ctx.dst,
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

    let muno_balance_after_send = t
        .ctx
        .src
        .get_balance(&cosmos_address.clone().to_string(), "au")
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

#[tokio::test]
async fn test_send_packet_from_evm_to_union_get_refund() {
    let t = init_ctx().await;
    let (evm_address, evm_provider) = t.ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_signer) = t.ctx.src.get_signer().await;

    println!("EVM Address: {:?}", evm_address);
    println!("Cosmos Address: {:?}", cosmos_address);

    ensure_channels_opened(t.ctx.channel_count).await;

    let available_channel = t.ctx.get_available_channel_count().await;
    assert!(available_channel > 0);

    let pair = t.ctx.get_channel().await.expect("channel available");
    let dst_chain_id = pair.dest;
    let src_chain_id = pair.src;

    // let deployed_erc20 = ensure_erc20(EVM_ZKGM_BYTES.into()).await;

    let deployed_erc20 = t
        .ctx
        .dst
        .deploy_basic_erc20(ETH_ADDRESS_ZKGM, evm_provider.clone())
        .await
        .expect("failed to deploy ERC20");

    let quote_token_addr = t
        .ctx
        .predict_wrapped_token::<cosmos::Module>(
            &t.ctx.src,
            t.union_address.zkgm.clone(),
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
    rand::thread_rng().fill_bytes(&mut salt_bytes);

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

    let erc20_balance_before_send = t
        .ctx
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

    let ucs03_zkgm = UCS03Zkgm::new(ETH_ADDRESS_ZKGM.into(), evm_provider.clone());

    let call = ucs03_zkgm
        .send(
            dst_chain_id,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            instruction_from_evm_to_union.clone(),
        )
        .clear_decoder()
        .with_cloned_provider();

    let recv_packet_data = t
        .ctx
        .send_and_recv_refund::<evm::Module, cosmos::Module>(
            &t.ctx.dst,
            ETH_ADDRESS_ZKGM,
            call,
            &t.ctx.src,
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

    let erc20_balance_after_send = t
        .ctx
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

#[tokio::test]
async fn test_from_evm_to_union_tokenv2_unhappy_only_maker_err() {
    let t = init_ctx().await;

    let (evm_address, _evm_provider) = t.ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = t.ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();
    println!("EVM Address: {:?}", evm_address);

    ensure_channels_opened(t.ctx.channel_count).await;
    let available_channel = t.ctx.get_available_channel_count().await;
    assert!(available_channel > 0);
    let pair = t.ctx.get_channel().await.expect("channel available");

    let img_metadata = ucs03_zkgm::com::TokenMetadata {
        implementation: hex!("999709eB04e8A30C7aceD9fd920f7e04EE6B97bA")
            .to_vec()
            .into(),
        initializer: ZkgmERC20::initializeCall {
            _authority: hex!("6C1D11bE06908656D16EBFf5667F1C45372B7c89").into(),
            _minter: ETH_ADDRESS_ZKGM.into(),
            _name: "au".into(),
            _symbol: "au".into(),
            _decimals: 6u8,
        }
        .abi_encode()
        .into(),
    }
    .abi_encode_params();

    let (zkgm_deployer_address, zkgm_deployer_provider) = t.ctx.dst.get_provider_privileged().await;

    let mut salt_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt_bytes);

    let instruction_cosmos = Instruction {
        version: INSTR_VERSION_2,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV2 {
            sender: cosmos_address_bytes.clone().into(),
            receiver: evm_address.to_vec().into(),
            base_token: "au".as_bytes().into(),
            base_amount: "10".parse().unwrap(),
            kind: TOKEN_ORDER_KIND_INITIALIZE,
            metadata: img_metadata.clone().into(),
            quote_token: evm_address.to_vec().into(), // Wrong quote token, so it will revert ONLY_MAKER
            quote_amount: "10".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    let instruction_evm = InstructionEvm {
        version: INSTR_VERSION_2,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV2 {
            sender: cosmos_address_bytes.clone().into(),
            receiver: evm_address.to_vec().into(),
            base_token: "au".as_bytes().into(),
            base_amount: "10".parse().unwrap(),
            kind: TOKEN_ORDER_KIND_INITIALIZE,
            metadata: img_metadata.into(),
            quote_token: evm_address.to_vec().into(), //quote_token_addr.as_ref().to_vec().into(),
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
        denom: "au".into(),
        amount: "10".into(),
    }];

    let height = t
        .ctx
        .send_and_get_height::<cosmos::Module, evm::Module>(
            &t.ctx.src,
            t.union_address.zkgm.clone(),
            (bin_msg, funds),
            &t.ctx.dst,
            Duration::from_secs(720),
            cosmos_provider,
        )
        .await;

    assert!(
        height.is_ok(),
        "Failed to send and receive packet: {:?}",
        height.err()
    );
    let height = height.unwrap();

    let hashed_salt = keccak256((cosmos_address_bytes.clone(), salt_bytes).abi_encode());

    let zkgm_packet = ZkgmPacket {
        salt: hashed_salt.into(),
        path: U256::from(0u32).into(),
        instruction: instruction_evm.clone(),
    };

    let encoded_packet: Vec<u8> = zkgm_packet.abi_encode_params();

    let packets = vec![IBCPacket {
        sourceChannelId: pair.src,
        destinationChannelId: pair.dest,
        data: encoded_packet.clone().into(),
        timeoutHeight: 0u64,
        timeoutTimestamp: 4294967295000000000,
    }];

    let proof = t
        .ctx
        .calculate_proof::<evm::Module>(
            &t.ctx.dst,
            pair.src,
            pair.dest,
            encoded_packet,
            height,
            "union-devnet-1",
        )
        .await;

    assert!(
        proof.is_ok(),
        "Failed to calculate proof: {:?}",
        proof.err()
    );
    let proof = proof.unwrap();

    let recv_packet_msg = MsgPacketRecv {
        packets,
        relayerMsgs: vec![vec![].into()],
        relayer: zkgm_deployer_address,

        proof: proof.into(),
        proofHeight: height,
    };

    let ibc = IBC::new(ETH_ADDRESS_IBC.into(), zkgm_deployer_provider.clone());
    let call = ibc
        .recvPacket(recv_packet_msg)
        .clear_decoder()
        .with_cloned_provider();

    let expected_revert_code = 0x3717ba2c; // Only maker
    let recv_packet_data = t
        .ctx
        .send_and_expect_revert::<evm::Module, cosmos::Module>(
            &t.ctx.dst,
            ETH_ADDRESS_IBC,
            call,
            expected_revert_code,
            &zkgm_deployer_provider,
        )
        .await;

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );
}

#[tokio::test]
async fn test_from_evm_to_union_tokenv2_unhappy_err_invalid_unescrow() {
    let t = init_ctx().await;

    let (evm_address, evm_provider) = t.ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = t.ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();
    println!("EVM Address: {:?}", evm_address);

    ensure_channels_opened(t.ctx.channel_count).await;
    let available_channel = t.ctx.get_available_channel_count().await;
    assert!(available_channel > 0);
    let pair = t.ctx.get_channel().await.expect("channel available");

    let img_metadata = ucs03_zkgm::com::TokenMetadata {
        implementation: hex!("999709eB04e8A30C7aceD9fd920f7e04EE6B97bA")
            .to_vec()
            .into(),
        initializer: ZkgmERC20::initializeCall {
            _authority: hex!("6C1D11bE06908656D16EBFf5667F1C45372B7c89").into(),
            _minter: ETH_ADDRESS_ZKGM.into(),
            _name: "au".into(),
            _symbol: "au".into(),
            _decimals: 6u8,
        }
        .abi_encode()
        .into(),
    }
    .abi_encode_params();

    let (_zkgm_deployer_address, _zkgm_deployer_provider) =
        t.ctx.dst.get_provider_privileged().await;

    let img = keccak256(&img_metadata);
    let (_, zkgm_deployer_provider) = t.ctx.dst.get_provider_privileged().await;
    // let governance_token = ctx
    //     .dst
    //     .setup_governance_token(
    //         EVM_ZKGM_BYTES.into(),
    //         pair.dest,
    //         img,
    //         zkgm_deployer_provider.clone(),
    //     )
    //     .await;

    // assert!(
    //     governance_token.is_ok(),
    //     "Failed to setup governance token: {:?}",
    //     governance_token.err()
    // );

    let mut salt_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt_bytes);
    let quote_token_addr = t
        .ctx
        .predict_wrapped_token_from_metadata_image_v2::<evm::Module>(
            &t.ctx.dst,
            ETH_ADDRESS_ZKGM,
            ChannelId::new(NonZero::new(pair.dest).unwrap()),
            "au".into(),
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
            base_token: "au".as_bytes().into(),
            base_amount: "10".parse().unwrap(),
            kind: TOKEN_ORDER_KIND_INITIALIZE,
            metadata: img_metadata.clone().into(),
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
        denom: "au".into(),
        amount: "10".into(),
    }];

    let recv_packet_data = t
        .ctx
        .send_and_recv_with_retry::<cosmos::Module, evm::Module>(
            &t.ctx.src,
            t.union_address.zkgm.clone(),
            (bin_msg, funds),
            &t.ctx.dst,
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

    let approve_tx_hash = t
        .ctx
        .dst
        .zkgmerc20_approve(
            quote_token_addr,
            ETH_ADDRESS_ZKGM,
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
    rand::thread_rng().fill_bytes(&mut buf);

    let random_token_id: U256 = U256::from_be_bytes(buf);
    println!("✅ random_token_id: {:?}", random_token_id);

    let instruction_from_evm_to_union = InstructionEvm {
        version: INSTR_VERSION_2,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV2 {
            sender: cosmos_address_bytes.clone().into(),
            receiver: evm_address.to_vec().into(),
            base_token: "au".as_bytes().into(), // Which is wrong, so it will revert ErrInvalidUnescrow
            base_amount: "10".parse().unwrap(),
            kind: TOKEN_ORDER_KIND_UNESCROW,
            metadata: img_metadata.into(),
            quote_token: quote_token_addr.as_ref().to_vec().into(),
            quote_amount: "10".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    let ucs03_zkgm = UCS03Zkgm::new(ETH_ADDRESS_ZKGM.into(), evm_provider.clone());

    rand::thread_rng().fill_bytes(&mut salt_bytes);
    let call = ucs03_zkgm
        .send(
            pair.dest,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            instruction_from_evm_to_union.clone(),
        )
        .clear_decoder()
        .with_cloned_provider();

    let expected_revert_code = 0x96c99a39; // ErrChannelGovernanceTokenNotSet
    let recv_packet_data = t
        .ctx
        .send_and_expect_revert::<evm::Module, cosmos::Module>(
            &t.ctx.dst,
            ETH_ADDRESS_ZKGM,
            call,
            expected_revert_code,
            &zkgm_deployer_provider,
        )
        .await;

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );
}

#[tokio::test]
async fn test_from_evm_to_union_tokenv2_unhappy_err_cannot_deploy() {
    let t = init_ctx().await;

    let (evm_address, evm_provider) = t.ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = t.ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();
    println!("EVM Address: {:?}", evm_address);

    ensure_channels_opened(t.ctx.channel_count).await;
    let available_channel = t.ctx.get_available_channel_count().await;
    assert!(available_channel > 0);
    let pair = t.ctx.get_channel().await.expect("channel available");

    let img_metadata = ucs03_zkgm::com::TokenMetadata {
        implementation: hex!("999709eB04e8A30C7aceD9fd920f7e04EE6B97bA")
            .to_vec()
            .into(),
        initializer: ZkgmERC20::initializeCall {
            _authority: hex!("6C1D11bE06908656D16EBFf5667F1C45372B7c89").into(),
            _minter: ETH_ADDRESS_ZKGM.into(),
            _name: "au".into(),
            _symbol: "au".into(),
            _decimals: 6u8,
        }
        .abi_encode()
        .into(),
    }
    .abi_encode_params();

    let mut salt_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt_bytes);

    let quote_token_addr = t
        .ctx
        .predict_wrapped_token::<evm::Module>(
            &t.ctx.dst,
            ETH_ADDRESS_ZKGM,
            ChannelId::new(NonZero::new(pair.dest).unwrap()),
            "au".into(),
            &evm_provider,
        )
        .await
        .unwrap();

    let instruction_cosmos = Instruction {
        version: INSTR_VERSION_2,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV2 {
            sender: cosmos_address_bytes.clone().into(),
            receiver: evm_address.to_vec().into(),
            base_token: "au".as_bytes().into(),
            base_amount: "10".parse().unwrap(),
            kind: TOKEN_ORDER_KIND_ESCROW, // Which is wrong, so it will revert CANNOT_DEPLOY
            metadata: img_metadata.clone().into(),
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
        denom: "au".into(),
        amount: "10".into(),
    }];

    let acked_packet = t
        .ctx
        .send_and_recv_ack::<cosmos::Module, evm::Module>(
            &t.ctx.src,
            t.union_address.zkgm.clone(),
            (bin_msg, funds),
            &t.ctx.dst,
            Duration::from_secs(720),
            cosmos_provider,
        )
        .await;

    assert!(
        acked_packet.is_ok(),
        "Failed to send and receive packet: {:?}",
        acked_packet.err()
    );
    let acked_packet = acked_packet.unwrap();
    assert!(
        acked_packet.tag == alloy::primitives::U256::ZERO,
        "Packet is acked successfully, but it should not be. Tag: {:?}",
        acked_packet.tag
    );
}

#[tokio::test]
async fn test_from_evm_to_union_batch_err_invalid_batch_instruction() {
    let t = init_ctx().await;
    let (evm_address, evm_provider) = t.ctx.dst.get_provider().await;
    let (cosmos_address, _cosmos_signer) = t.ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();

    println!("EVM Address: {:?}", evm_address);
    println!("Cosmos Address: {:?}", cosmos_address);

    ensure_channels_opened(t.ctx.channel_count).await;
    let available_channel = t.ctx.get_available_channel_count().await;
    assert!(available_channel > 0);
    let pair = t.ctx.get_channel().await.expect("channel available");

    let dst_chain_id = pair.dest;
    let _src_chain_id = pair.src;

    // let deployed_erc20 = ensure_erc20(EVM_ZKGM_BYTES.into()).await;

    let deployed_erc20 = t
        .ctx
        .dst
        .deploy_basic_erc20(ETH_ADDRESS_ZKGM, evm_provider.clone())
        .await
        .expect("failed to deploy ERC20");

    let mut salt_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt_bytes);

    let inner_token_order_inst = ucs03_zkgm::com::Instruction {
        version: INSTR_VERSION_1,
        opcode: OP_BATCH, // Using OP_BATCH to make this test fail with  ErrInvalidBatchInstruction
        operand: TokenOrderV1 {
            sender: evm_address.to_vec().into(),
            receiver: cosmos_address_bytes.clone().into(),
            base_token: deployed_erc20.as_ref().to_vec().into(),
            base_amount: "10".parse().unwrap(),
            base_token_symbol: "GLD".into(),
            base_token_name: "Gold".into(),
            base_token_decimals: 18,
            base_token_path: "0".parse().unwrap(),
            quote_token: evm_address.to_vec().into(), //anything is ok, it wont be used
            quote_amount: "10".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    let batch_operand = ucs03_zkgm::com::Batch {
        instructions: vec![inner_token_order_inst],
    }
    .abi_encode_params();

    let instruction_from_evm_to_union = InstructionEvm {
        version: INSTR_VERSION_0,
        opcode: OP_BATCH,
        operand: batch_operand.into(),
    };

    let ucs03_zkgm = UCS03Zkgm::new(ETH_ADDRESS_ZKGM.into(), evm_provider.clone());

    let call = ucs03_zkgm
        .send(
            dst_chain_id,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            instruction_from_evm_to_union.clone(),
        )
        .clear_decoder()
        .with_cloned_provider();

    let (_, zkgm_deployer_provider) = t.ctx.dst.get_provider_privileged().await;
    let expected_revert_code = 0x746a20f8; // ErrInvalidBatchInstruction
    let recv_packet_data = t
        .ctx
        .send_and_expect_revert::<evm::Module, cosmos::Module>(
            &t.ctx.dst,
            ETH_ADDRESS_ZKGM,
            call,
            expected_revert_code,
            &zkgm_deployer_provider,
        )
        .await;

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );
}

#[tokio::test]
async fn test_from_evm_to_union_batch_err_invalid_forward_instruction() {
    let t = init_ctx().await;
    let (evm_address, evm_provider) = t.ctx.dst.get_provider().await;
    let (cosmos_address, _cosmos_signer) = t.ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();

    println!("EVM Address: {:?}", evm_address);
    println!("Cosmos Address: {:?}", cosmos_address);

    ensure_channels_opened(t.ctx.channel_count).await;
    let available_channel = t.ctx.get_available_channel_count().await;
    assert!(available_channel > 0);
    let pair = t.ctx.get_channel().await.expect("channel available");

    let dst_chain_id = pair.dest;
    let _src_chain_id: u32 = pair.src;

    let deployed_erc20 = t
        .ctx
        .dst
        .deploy_basic_erc20(ETH_ADDRESS_ZKGM, evm_provider.clone())
        .await
        .expect("failed to deploy ERC20");

    let mut salt_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt_bytes);

    let inner_token_order_inst = ucs03_zkgm::com::Instruction {
        version: INSTR_VERSION_1,
        opcode: OP_FORWARD, // Using OP_FORWARD to make this test fail with  ErrInvalidForwardInstruction
        operand: TokenOrderV1 {
            sender: evm_address.to_vec().into(),
            receiver: cosmos_address_bytes.clone().into(),
            base_token: deployed_erc20.as_ref().to_vec().into(),
            base_amount: "10".parse().unwrap(),
            base_token_symbol: "GLD".into(),
            base_token_name: "Gold".into(),
            base_token_decimals: 18,
            base_token_path: "0".parse().unwrap(),
            quote_token: evm_address.to_vec().into(), //anything is ok, it wont be used
            quote_amount: "10".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };
    let forward_operand = ucs03_zkgm::com::Forward {
        instruction: inner_token_order_inst,
        timeout_height: 0u64,
        timeout_timestamp: u32::MAX.into(),
        path: U256::from(0u64).into(),
    }
    .abi_encode_params();

    let instruction_from_evm_to_union = InstructionEvm {
        version: INSTR_VERSION_0,
        opcode: OP_FORWARD,
        operand: forward_operand.into(),
    };

    let ucs03_zkgm = UCS03Zkgm::new(ETH_ADDRESS_ZKGM.into(), evm_provider.clone());

    let call = ucs03_zkgm
        .send(
            dst_chain_id,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            instruction_from_evm_to_union.clone(),
        )
        .clear_decoder()
        .with_cloned_provider();

    let (_, zkgm_deployer_provider) = t.ctx.dst.get_provider_privileged().await;
    let expected_revert_code = 0x1dbb3218; // ErrInvalidForwardInstruction
    let recv_packet_data = t
        .ctx
        .send_and_expect_revert::<evm::Module, cosmos::Module>(
            &t.ctx.dst,
            ETH_ADDRESS_ZKGM,
            call,
            expected_revert_code,
            &zkgm_deployer_provider,
        )
        .await;

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );
}

#[tokio::test]
async fn test_send_vault_unhappy_u_counterparty_is_not_fungible() {
    let t = init_ctx().await;

    let (evm_address, _evm_provider) = t.ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = t.ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();

    ensure_channels_opened(t.ctx.channel_count).await;
    let available_channel = t.ctx.get_available_channel_count().await;
    assert!(available_channel > 0);
    let pair = t.ctx.get_channel().await.expect("channel available");

    let dst_channel_id = pair.dest;
    let src_channel_id = pair.src;

    let _vault_on_union = "union1skg5244hpkad603zz77kdekzw6ffgpfrde3ldk8rpdz06n62k4hqct0w4j";

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
            base_token: "au".as_bytes().into(),
            base_amount: "10".parse().unwrap(),
            kind: TOKEN_ORDER_KIND_SOLVE,
            metadata: metadata.clone().into(),
            quote_token: u_on_eth.to_vec().into(),
            quote_amount: "10".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    let instruction_evm = InstructionEvm {
        version: INSTR_VERSION_2,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV2 {
            sender: cosmos_address_bytes.clone().into(),
            receiver: evm_address.to_vec().into(),
            base_token: "au".as_bytes().into(),
            base_amount: "10".parse().unwrap(),
            kind: TOKEN_ORDER_KIND_SOLVE,
            metadata: metadata.into(),
            quote_token: u_on_eth.to_vec().into(),
            quote_amount: "10".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };
    let (zkgm_deployer_address, zkgm_deployer_provider) = t.ctx.dst.get_provider_privileged().await;
    let empty_beneficiary = "".as_bytes().to_vec().into();
    t.ctx
        .dst
        .u_register_fungible_counterpart(
            H160::from(u_on_eth),
            zkgm_deployer_provider.clone(),
            alloy::primitives::U256::ZERO,
            dst_channel_id,
            b"au".to_vec().into(),
            evm::u::U::FungibleCounterparty {
                beneficiary: empty_beneficiary, // Sending it empty to make this
                                                // test revert due to U_CounterpartyIsNotFungible and get ErrOnlyMaker
            },
        )
        .await
        .unwrap();

    let mut salt_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt_bytes);

    let cw_msg = ucs03_zkgm::msg::ExecuteMsg::Send {
        channel_id: src_channel_id.try_into().unwrap(),
        timeout_height: 0u64.into(),
        timeout_timestamp: voyager_sdk::primitives::Timestamp::from_secs(u32::MAX.into()),
        salt: salt_bytes.into(),
        instruction: instruction_cosmos.abi_encode_params().into(),
    };
    let bin_msg: Vec<u8> = Encode::<Json>::encode(&cw_msg);

    let funds = vec![Coin {
        denom: "au".into(),
        amount: "10".into(),
    }];

    let height = t
        .ctx
        .send_and_get_height::<cosmos::Module, evm::Module>(
            &t.ctx.src,
            t.union_address.zkgm.clone(),
            (bin_msg, funds),
            &t.ctx.dst,
            Duration::from_secs(720),
            cosmos_provider,
        )
        .await;
    assert!(
        height.is_ok(),
        "Failed to send and receive packet: {:?}",
        height.err()
    );
    let height = height.unwrap();

    let hashed_salt = keccak256((cosmos_address_bytes.clone(), salt_bytes).abi_encode());

    let zkgm_packet = ZkgmPacket {
        salt: hashed_salt.into(),
        path: U256::from(0u32).into(),
        instruction: instruction_evm.clone(),
    };

    let encoded_packet: Vec<u8> = zkgm_packet.abi_encode_params();

    let packets = vec![IBCPacket {
        sourceChannelId: pair.src,
        destinationChannelId: pair.dest,
        data: encoded_packet.clone().into(),
        timeoutHeight: 0u64,
        timeoutTimestamp: 4294967295000000000,
    }];

    let proof = t
        .ctx
        .calculate_proof::<evm::Module>(
            &t.ctx.dst,
            pair.src,
            pair.dest,
            encoded_packet,
            height,
            "union-devnet-1",
        )
        .await;

    assert!(
        proof.is_ok(),
        "Failed to calculate proof: {:?}",
        proof.err()
    );
    let proof = proof.unwrap();

    let recv_packet_msg = MsgPacketRecv {
        packets,
        relayerMsgs: vec![vec![].into()],
        relayer: zkgm_deployer_address,

        proof: proof.into(),
        proofHeight: height,
    };

    let ibc = IBC::new(ETH_ADDRESS_IBC.into(), zkgm_deployer_provider.clone());
    let call = ibc
        .recvPacket(recv_packet_msg)
        .clear_decoder()
        .with_cloned_provider();

    let expected_revert_code = 0x3717ba2c; // Only maker
    let recv_packet_data = t
        .ctx
        .send_and_expect_revert::<evm::Module, cosmos::Module>(
            &t.ctx.dst,
            ETH_ADDRESS_IBC,
            call,
            expected_revert_code,
            &zkgm_deployer_provider,
        )
        .await;

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );
}

#[tokio::test]
async fn test_send_vault_unhappy_u_base_amount_must_cover_quote_amount() {
    let t = init_ctx().await;

    let (evm_address, _evm_provider) = t.ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = t.ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();

    ensure_channels_opened(t.ctx.channel_count).await;
    let available_channel = t.ctx.get_available_channel_count().await;
    assert!(available_channel > 0);
    let pair = t.ctx.get_channel().await.expect("channel available");

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
            base_token: "au".as_bytes().into(),
            base_amount: "10".parse().unwrap(),
            kind: TOKEN_ORDER_KIND_SOLVE,
            metadata: metadata.clone().into(),
            quote_token: u_on_eth.to_vec().into(),
            quote_amount: "11".parse().unwrap(), // Sending it bigger than base_amount to make this
                                                 // test revert due to U_BaseAmountMustCoverQuoteAmount and get ErrOnlyMaker
        }
        .abi_encode_params()
        .into(),
    };

    let instruction_evm = InstructionEvm {
        version: INSTR_VERSION_2,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV2 {
            sender: cosmos_address_bytes.clone().into(),
            receiver: evm_address.to_vec().into(),
            base_token: "au".as_bytes().into(),
            base_amount: "10".parse().unwrap(),
            kind: TOKEN_ORDER_KIND_SOLVE,
            metadata: metadata.into(),
            quote_token: u_on_eth.to_vec().into(),
            quote_amount: "11".parse().unwrap(), // Sending it bigger than base_amount to make this
                                                 // test revert due to U_BaseAmountMustCoverQuoteAmount and get ErrOnlyMaker
        }
        .abi_encode_params()
        .into(),
    };
    let (zkgm_deployer_address, zkgm_deployer_provider) = t.ctx.dst.get_provider_privileged().await;
    t.ctx
        .dst
        .u_register_fungible_counterpart(
            H160::from(u_on_eth),
            zkgm_deployer_provider.clone(),
            alloy::primitives::U256::ZERO,
            dst_channel_id,
            b"au".to_vec().into(),
            evm::u::U::FungibleCounterparty {
                beneficiary: vault_on_union.as_bytes().to_vec().into(),
            },
        )
        .await
        .unwrap();

    let mut salt_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt_bytes);

    let cw_msg = ucs03_zkgm::msg::ExecuteMsg::Send {
        channel_id: src_channel_id.try_into().unwrap(),
        timeout_height: 0u64.into(),
        timeout_timestamp: voyager_sdk::primitives::Timestamp::from_secs(u32::MAX.into()),
        salt: salt_bytes.into(),
        instruction: instruction_cosmos.abi_encode_params().into(),
    };
    let bin_msg: Vec<u8> = Encode::<Json>::encode(&cw_msg);

    let funds = vec![Coin {
        denom: "au".into(),
        amount: "10".into(),
    }];

    let height = t
        .ctx
        .send_and_get_height::<cosmos::Module, evm::Module>(
            &t.ctx.src,
            t.union_address.zkgm.clone(),
            (bin_msg, funds),
            &t.ctx.dst,
            Duration::from_secs(720),
            cosmos_provider,
        )
        .await;
    assert!(
        height.is_ok(),
        "Failed to send and receive packet: {:?}",
        height.err()
    );
    let height = height.unwrap();

    let hashed_salt = keccak256((cosmos_address_bytes.clone(), salt_bytes).abi_encode());

    let zkgm_packet = ZkgmPacket {
        salt: hashed_salt.into(),
        path: U256::from(0u32).into(),
        instruction: instruction_evm.clone(),
    };

    let encoded_packet: Vec<u8> = zkgm_packet.abi_encode_params();

    let packets = vec![IBCPacket {
        sourceChannelId: pair.src,
        destinationChannelId: pair.dest,
        data: encoded_packet.clone().into(),
        timeoutHeight: 0u64,
        timeoutTimestamp: 4294967295000000000,
    }];

    let proof = t
        .ctx
        .calculate_proof::<evm::Module>(
            &t.ctx.dst,
            pair.src,
            pair.dest,
            encoded_packet,
            height,
            "union-devnet-1",
        )
        .await;

    assert!(
        proof.is_ok(),
        "Failed to calculate proof: {:?}",
        proof.err()
    );
    let proof = proof.unwrap();

    let recv_packet_msg = MsgPacketRecv {
        packets,
        relayerMsgs: vec![vec![].into()],
        relayer: zkgm_deployer_address,

        proof: proof.into(),
        proofHeight: height,
    };

    let ibc = IBC::new(ETH_ADDRESS_IBC.into(), zkgm_deployer_provider.clone());

    let call = ibc
        .recvPacket(recv_packet_msg)
        .clear_decoder()
        .with_cloned_provider();

    let expected_revert_code = 0x3717ba2c; // Only maker
    let recv_packet_data = t
        .ctx
        .send_and_expect_revert::<evm::Module, cosmos::Module>(
            &t.ctx.dst,
            ETH_ADDRESS_IBC,
            call,
            expected_revert_code,
            &zkgm_deployer_provider,
        )
        .await;

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );
}

#[tokio::test]
async fn test_send_vault_unhappy_u_fool() {
    let t = init_ctx().await;

    let (evm_address, _evm_provider) = t.ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = t.ctx.src.get_signer().await;
    let cosmos_address_bytes = cosmos_address.to_string().into_bytes();

    ensure_channels_opened(t.ctx.channel_count).await;
    let available_channel = t.ctx.get_available_channel_count().await;
    assert!(available_channel > 0);
    let pair = t.ctx.get_channel().await.expect("channel available");

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
            base_token: "au".as_bytes().into(),
            base_amount: "10".parse().unwrap(),
            kind: TOKEN_ORDER_KIND_SOLVE,
            metadata: metadata.clone().into(),
            quote_token: evm_address.to_vec().into(), // Sending it wrong to make this
            // test revert due to U_Fool and get ErrOnlyMaker
            quote_amount: "10".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    let instruction_evm = InstructionEvm {
        version: INSTR_VERSION_2,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV2 {
            sender: cosmos_address_bytes.clone().into(),
            receiver: evm_address.to_vec().into(),
            base_token: "au".as_bytes().into(),
            base_amount: "10".parse().unwrap(),
            kind: TOKEN_ORDER_KIND_SOLVE,
            metadata: metadata.into(),
            quote_token: evm_address.to_vec().into(), // Sending it wrong to make this
            // test revert due to U_Fool and get ErrOnlyMaker
            quote_amount: "10".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };
    let (zkgm_deployer_address, zkgm_deployer_provider) = t.ctx.dst.get_provider_privileged().await;
    t.ctx
        .dst
        .u_register_fungible_counterpart(
            H160::from(u_on_eth),
            zkgm_deployer_provider.clone(),
            alloy::primitives::U256::ZERO,
            dst_channel_id,
            b"au".to_vec().into(),
            evm::u::U::FungibleCounterparty {
                beneficiary: vault_on_union.as_bytes().to_vec().into(),
            },
        )
        .await
        .unwrap();

    let mut salt_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt_bytes);

    let cw_msg = ucs03_zkgm::msg::ExecuteMsg::Send {
        channel_id: src_channel_id.try_into().unwrap(),
        timeout_height: 0u64.into(),
        timeout_timestamp: voyager_sdk::primitives::Timestamp::from_secs(u32::MAX.into()),
        salt: salt_bytes.into(),
        instruction: instruction_cosmos.abi_encode_params().into(),
    };
    let bin_msg: Vec<u8> = Encode::<Json>::encode(&cw_msg);

    let funds = vec![Coin {
        denom: "au".into(),
        amount: "10".into(),
    }];

    let height = t
        .ctx
        .send_and_get_height::<cosmos::Module, evm::Module>(
            &t.ctx.src,
            t.union_address.zkgm.clone(),
            (bin_msg, funds),
            &t.ctx.dst,
            Duration::from_secs(720),
            cosmos_provider,
        )
        .await;
    assert!(
        height.is_ok(),
        "Failed to send and receive packet: {:?}",
        height.err()
    );
    let height = height.unwrap();

    let hashed_salt = keccak256((cosmos_address_bytes.clone(), salt_bytes).abi_encode());

    let zkgm_packet = ZkgmPacket {
        salt: hashed_salt.into(),
        path: U256::from(0u32).into(),
        instruction: instruction_evm.clone(),
    };

    let encoded_packet: Vec<u8> = zkgm_packet.abi_encode_params();

    let packets = vec![IBCPacket {
        sourceChannelId: pair.src,
        destinationChannelId: pair.dest,
        data: encoded_packet.clone().into(),
        timeoutHeight: 0u64,
        timeoutTimestamp: 4294967295000000000,
    }];

    let proof = t
        .ctx
        .calculate_proof::<evm::Module>(
            &t.ctx.dst,
            pair.src,
            pair.dest,
            encoded_packet,
            height,
            "union-devnet-1",
        )
        .await;

    assert!(
        proof.is_ok(),
        "Failed to calculate proof: {:?}",
        proof.err()
    );
    let proof = proof.unwrap();

    let recv_packet_msg = MsgPacketRecv {
        packets,
        relayerMsgs: vec![vec![].into()],
        relayer: zkgm_deployer_address,

        proof: proof.into(),
        proofHeight: height,
    };

    let ibc = IBC::new(ETH_ADDRESS_IBC.into(), zkgm_deployer_provider.clone());

    let call = ibc
        .recvPacket(recv_packet_msg)
        .clear_decoder()
        .with_cloned_provider();

    let expected_revert_code = 0x3717ba2c; // Only maker
    let recv_packet_data = t
        .ctx
        .send_and_expect_revert::<evm::Module, cosmos::Module>(
            &t.ctx.dst,
            ETH_ADDRESS_IBC,
            call,
            expected_revert_code,
            &zkgm_deployer_provider,
        )
        .await;

    assert!(
        recv_packet_data.is_ok(),
        "Failed to send and receive packet: {:?}",
        recv_packet_data.err()
    );
}
