// tests/e2e_lst.rs

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
use cosmwasm_std::{instantiate2_address, to_json_binary, Coin as CwCoin, CosmosMsg, WasmMsg};
use cw20::Cw20ExecuteMsg;
use hex_literal::hex;
use ibc_union_spec::ChannelId;
use protos::cosmos::base::v1beta1::Coin;
use rand::RngCore;
use serde::Serialize;
use tokio::sync::OnceCell;
use ucs03_zkgm::{
    self,
    com::{
        Batch, Call, Instruction, SolverMetadata, TokenOrderV1, TokenOrderV2, INSTR_VERSION_0,
        INSTR_VERSION_1, INSTR_VERSION_2, OP_BATCH, OP_CALL, OP_FORWARD, OP_TOKEN_ORDER,
        TOKEN_ORDER_KIND_ESCROW, TOKEN_ORDER_KIND_INITIALIZE, TOKEN_ORDER_KIND_SOLVE,
        TOKEN_ORDER_KIND_UNESCROW,
    },
};
use union_test::{
    cosmos::{self},
    evm::{
        self,
        zkgm::{
            IBCPacket, Instruction as InstructionEvm, MsgPacketRecv, UCS03Zkgm, ZkgmPacket, IBC,
        },
        zkgmerc20::ZkgmERC20,
    },
    TestContext,
};
use unionlabs::{
    encoding::{Encode, Json},
    ethereum::keccak256,
    primitives::{encoding::Base64, Bech32, Bytes, FixedBytes, H160, U256},
};
use voyager_sdk::{
    primitives::{ChainId, Timestamp},
    serde_json::json,
};
static CTX: OnceCell<Arc<TestContext<cosmos::Module, evm::Module>>> = OnceCell::const_new();
static CHANNELS_OPENED: OnceCell<()> = OnceCell::const_new();
// static ERC20: OnceCell<H160> = OnceCell::const_new();

static UNION_ZKGM_ADDRESS: &str =
    "union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c";
// static UNION_MINTER_ADDRESS: &str =
//     "union1tt6nn3qv0q0z4gq4s2h65a2acv3lcwxjwf8ey3jgnwmtqkfnyq9q4q5y8x";
static EVM_ZKGM_BYTES: [u8; 20] = hex!("05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5");
static EVM_IBC_BYTES: [u8; 20] = hex!("ed2af2aD7FE0D92011b26A2e5D1B4dC7D12A47C5");

// u: union1pntx7gm7shsp6slef74ae7wvcc35t3wdmanh7wrg4xkq95m24qds5atmcp
// lst: union1fdg764zzxwvwyqkx3fuj0236l9ddh5xmutgvj2mv9cduffy82z9sp62ygc

#[derive(Serialize)]
struct BondInner<'a> {
    mint_to_address: &'a str,
    min_mint_amount: &'a str,
}
#[derive(Serialize)]
struct BondMsg<'a> {
    bond: BondInner<'a>,
}

fn make_zkgm_call_payload(
    lst_hub: &str,
    mint_to: &str,
    min_amount: &str,
    funds_denom: &str,
    funds_amount: u32,
) -> String {
    let bond = BondMsg {
        bond: BondInner {
            mint_to_address: mint_to,
            min_mint_amount: min_amount,
        },
    };

    let wasm_exec: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lst_hub.to_string(),
        msg: to_json_binary(&bond).unwrap(),
        funds: vec![CwCoin {
            denom: funds_denom.to_string(),
            amount: funds_amount.into(),
        }],
    });

    voyager_sdk::serde_json::to_string(&vec![wasm_exec]).expect("vec cosmosmsg json")
}

async fn init_ctx<'a>() -> Arc<TestContext<cosmos::Module, evm::Module<'a>>> {
    CTX.get_or_init(|| async {
        let cosmos_cfg = cosmos::Config {
            chain_id: ChainId::new("union-devnet-1"),
            ibc_host_contract_address: Bech32::from_str(
                "union1nk3nes4ef6vcjan5tz6stf9g8p08q2kgqysx6q5exxh89zakp0msq5z79t",
            )
            .unwrap(),
            privileged_acc_keyring: KeyringConfig {
                name: "privileged_acc".into(),
                keys: vec![KeyringConfigEntry::Raw {
                    name: "privileged_acc".into(),
                    key: hex!("aa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f")
                        .to_vec(),
                }],
            },
            keyring: KeyringConfig {
                name: "alice".into(),
                keys: vec![
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
            privileged_acc_keyring: KeyringConfig {
                name: "zkgm-deployer".into(),
                keys: vec![KeyringConfigEntry::Raw {
                    name: "zkgm-deployer-key".into(),
                    key: hex!("4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77")
                        .to_vec(),
                }],
            },
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
        let needed_channel_count = 1; // TODO: Hardcoded now, it will be specified from config later.

        // TODO(aeryz): move config file into the testing framework's own config file
        let ctx = TestContext::new(
            src,
            dst,
            needed_channel_count,
            "/home/kaancaglan/dev/union/union/voyager/config.jsonc",
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

#[tokio::test]
async fn test_escher_lst() {
    let ctx = init_ctx().await;

    let (evm_address, evm_provider) = ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = ctx.src.get_signer().await;
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

    let (_, zkgm_deployer_provider) = ctx.dst.get_provider_privileged().await;
    println!("registering u counterpart");
    ctx.dst
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

    let contract: Bech32<FixedBytes<32>> = Bech32::from_str(UNION_ZKGM_ADDRESS).unwrap();

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
        "1000000",
        "muno",
        1000000,
    );

    let proxy_balance = ctx
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
                        base_amount: "150".parse().unwrap(), //giving 150 but expecting 1000000 so it will fail.
                        kind: TOKEN_ORDER_KIND_SOLVE,
                        metadata: SolverMetadata {
                            solverAddress: vault_on_union.as_bytes().into(),
                            metadata: Default::default(),
                        }
                        .abi_encode_params()
                        .into(),
                        quote_token: "muno".as_bytes().into(),
                        quote_amount: "150".parse().unwrap(), //giving 150 but expecting 1000000 so it will fail.
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

    let approve_tx_hash = ctx
        .dst
        .zkgmerc20_approve(
            u_on_eth.into(),
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

    let ucs03_zkgm = UCS03Zkgm::new(EVM_ZKGM_BYTES.into(), evm_provider.clone());

    let call = ucs03_zkgm
        .send(
            dst_channel_id,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            instruction_from_evm_to_union.clone(),
        )
        .clear_decoder();

    let acked_packet = ctx
        .send_and_recv_and_ack_with_retry::<evm::Module, cosmos::Module>(
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

    let acked_packet = acked_packet.unwrap();
    assert!(
        acked_packet.tag == 0,
        "Packet is acked successfully, but it should not be. Tag: {:?}",
        acked_packet.tag
    );

    let proxy_balance = ctx
        .src
        .native_balance(zkgm_proxy_calculated, "muno")
        .await
        .unwrap();

    println!("Proxy balance before: {}", proxy_balance);
}

#[tokio::test]
async fn test_escher_lst_unhappy_less_money_than_required() {
    let ctx = init_ctx().await;

    let (evm_address, evm_provider) = ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = ctx.src.get_signer().await;
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

    let (_, zkgm_deployer_provider) = ctx.dst.get_provider_privileged().await;
    println!("registering u counterpart");
    ctx.dst
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

    let contract: Bech32<FixedBytes<32>> = Bech32::from_str(UNION_ZKGM_ADDRESS).unwrap();

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
        "10",
        "muno",
        10,
    );

    let proxy_balance = ctx
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

    let approve_tx_hash = ctx
        .dst
        .zkgmerc20_approve(
            u_on_eth.into(),
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

    let ucs03_zkgm = UCS03Zkgm::new(EVM_ZKGM_BYTES.into(), evm_provider.clone());

    let call = ucs03_zkgm
        .send(
            dst_channel_id,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            instruction_from_evm_to_union.clone(),
        )
        .clear_decoder();

    let acked_packet = ctx
        .send_and_recv_and_ack_with_retry::<evm::Module, cosmos::Module>(
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

    let acked_packet = acked_packet.unwrap();
    assert!(
        acked_packet.tag == 1,
        "Packet is acked not successfully, but it should be. Tag: {:?}",
        acked_packet.tag
    );

    let proxy_balance = ctx
        .src
        .native_balance(zkgm_proxy_calculated, "muno")
        .await
        .unwrap();

    println!("Proxy balance before: {}", proxy_balance);
}


#[tokio::test]
async fn test_escher_lst_unhappy_wrong_denom() {
    let ctx = init_ctx().await;

    let (evm_address, evm_provider) = ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = ctx.src.get_signer().await;
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

    let (_, zkgm_deployer_provider) = ctx.dst.get_provider_privileged().await;
    println!("registering u counterpart");
    ctx.dst
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

    let contract: Bech32<FixedBytes<32>> = Bech32::from_str(UNION_ZKGM_ADDRESS).unwrap();

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
        "10",
        "muan", // wrong denom to make it fail
        10,
    );

    let proxy_balance = ctx
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

    let approve_tx_hash = ctx
        .dst
        .zkgmerc20_approve(
            u_on_eth.into(),
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

    let ucs03_zkgm = UCS03Zkgm::new(EVM_ZKGM_BYTES.into(), evm_provider.clone());

    let call = ucs03_zkgm
        .send(
            dst_channel_id,
            0u64,
            4294967295000000000u64,
            salt_bytes.into(),
            instruction_from_evm_to_union.clone(),
        )
        .clear_decoder();

    let acked_packet = ctx
        .send_and_recv_and_ack_with_retry::<evm::Module, cosmos::Module>(
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

    let acked_packet = acked_packet.unwrap();
    assert!(
        acked_packet.tag == 1,
        "Packet is acked not successfully, but it should be. Tag: {:?}",
        acked_packet.tag
    );

    let proxy_balance = ctx
        .src
        .native_balance(zkgm_proxy_calculated, "muno")
        .await
        .unwrap();

    println!("Proxy balance before: {}", proxy_balance);
}

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
