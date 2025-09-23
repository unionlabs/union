use std::{cell::RefCell, future::Future, sync::Arc, time::Duration};

use alloy_sol_types::SolValue as _;
use bip32::secp256k1::ecdsa::SigningKey;
use cometbft_rpc::types::code::Code;
use cosmos_client::wallet::{LocalSigner, WalletT as _};
use cosmwasm_std::Addr;
use hex_literal::hex;
use protos::cosmos::base::v1beta1::Coin as ProtoCoin;
use rand::RngCore as _;
use rand_chacha::{
    rand_core::{block::BlockRng, SeedableRng},
    ChaChaCore,
};
use serde::Deserialize;
use tokio::sync::{Mutex, OnceCell};
use tracing::info;
use tracing_subscriber::FmtSubscriber;
use ucs03_zkgm::com::{
    Instruction, SolverMetadata, TokenOrderV2, INSTR_VERSION_2, OP_TOKEN_ORDER, TAG_ACK_SUCCESS,
    TOKEN_ORDER_KIND_SOLVE,
};
use union_test::{
    cosmos::{self},
    cosmos_helpers::{
        calculate_cosmos_contract_address, SALT_ESCROW_VAULT, SALT_EU, SALT_LST_HUB, SALT_ZKGM,
    },
    evm, TestContext,
};
use unionlabs::{
    encoding::{EncodeAs, Json},
    primitives::{H160, H256},
};
use voyager_sdk::{anyhow, serde_json};

pub mod bond;

pub static CTX: OnceCell<(Mutex<Queue>, Arc<LstContext>)> = OnceCell::const_new();

pub static CHANNELS_OPENED: OnceCell<()> = OnceCell::const_new();

pub const ETH_ADDRESS_U: H160 = H160::new(hex!("0c8C6f58156D10d18193A8fFdD853e1b9F8D8836"));
pub const ETH_ADDRESS_ZKGM: H160 = H160::new(hex!("05FD55C1AbE31D3ED09A76216cA8F0372f4B2eC5"));

pub struct UnionAddressBook {
    pub zkgm: Addr,
    pub lst_hub: Addr,
    pub eu: Addr,
    pub escrow_vault: Addr,
}

pub struct LstContext {
    pub union_address: UnionAddressBook,
    pub staker: LocalSigner,
    pub ctx: TestContext<cosmos::Module, evm::Module>,
}

#[derive(Deserialize)]
pub struct Config {
    evm: evm::Config,
    union: cosmos::Config,
    needed_channel_count: u32,
    voyager_config_file_path: String,
    union_deployer_addr: String,
}

pub struct Queue {
    tests: Vec<String>,
}

async fn run_test_in_queue<'a, Fut: Future<Output = ()>, F: Fn(Arc<LstContext>) -> Fut>(
    key: &str,
    test_fn: F,
) {
    let ctx = CTX
        .get_or_init(|| async {
            let subscriber = FmtSubscriber::builder()
                .with_max_level(tracing::Level::INFO)
                .finish();
            tracing::subscriber::set_global_default(subscriber)
                .expect("setting default subscriber failed");
            let cfg: Config = serde_json::from_str(include_str!("./config.json")).unwrap();

            let src = cosmos::Module::new(cfg.union).await.unwrap();
            let dst = evm::Module::new(cfg.evm).await.unwrap();

            let ctx = TestContext::new(
                src,
                dst,
                cfg.needed_channel_count as usize,
                &cfg.voyager_config_file_path,
            )
            .await
            .unwrap();

            let mut rng =
                BlockRng::new(ChaChaCore::from_rng(rand_chacha::rand_core::OsRng).unwrap());

            let staker = LocalSigner::new(SigningKey::random(&mut rng).to_bytes().into(), "union");

            let k = ctx.src.keyring.with(async |k| k).await.unwrap();

            if ctx
                .src
                .native_balance(Addr::unchecked(staker.address().to_string()), "muno")
                .await
                .unwrap()
                < 90_000_000
            {
                let outcome = ctx
                    .src
                    .send_transaction(
                        protos::cosmos::bank::v1beta1::MsgSend {
                            from_address: k.address().to_string(),
                            to_address: staker.address().to_string(),
                            amount: vec![ProtoCoin {
                                denom: "muno".to_string(),
                                amount: 100_000_000.to_string(),
                            }],
                        },
                        k,
                    )
                    .await
                    .unwrap()
                    .unwrap();

                assert_eq!(outcome.tx_result.code, Code::Ok);
            }

            (
                Mutex::new(Queue {
                    tests: {
                        let mut t = vec!["bond".into()];
                        t.reverse();
                        t
                    },
                }),
                Arc::new(LstContext {
                    staker,
                    union_address: UnionAddressBook {
                        zkgm: calculate_cosmos_contract_address(
                            &cfg.union_deployer_addr,
                            SALT_ZKGM,
                        )
                        .unwrap(),
                        lst_hub: calculate_cosmos_contract_address(
                            &cfg.union_deployer_addr,
                            SALT_LST_HUB,
                        )
                        .unwrap(),
                        eu: calculate_cosmos_contract_address(&cfg.union_deployer_addr, SALT_EU)
                            .unwrap(),
                        escrow_vault: calculate_cosmos_contract_address(
                            &cfg.union_deployer_addr,
                            SALT_ESCROW_VAULT,
                        )
                        .unwrap(),
                    },
                    ctx,
                }),
            )
        })
        .await;

    loop {
        {
            let mut lock = ctx.0.lock().await;
            if lock.tests.last().unwrap() == key {
                test_fn(ctx.1.clone()).await;
                let _ = lock.tests.pop();
                return;
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
// async fn init_ctx<'a>(key: &str) -> Arc<LstContext> {
//     let ctx = CTX
//         .get_or_init(|| async {
//             let subscriber = FmtSubscriber::builder()
//                 .with_max_level(tracing::Level::INFO)
//                 .finish();
//             tracing::subscriber::set_global_default(subscriber)
//                 .expect("setting default subscriber failed");
//             let cfg: Config = serde_json::from_str(include_str!("./config.json")).unwrap();

//             let src = cosmos::Module::new(cfg.union).await.unwrap();
//             let dst = evm::Module::new(cfg.evm).await.unwrap();

//             let ctx = TestContext::new(
//                 src,
//                 dst,
//                 cfg.needed_channel_count as usize,
//                 &cfg.voyager_config_file_path,
//             )
//             .await
//             .unwrap();

//             let mut rng =
//                 BlockRng::new(ChaChaCore::from_rng(rand_chacha::rand_core::OsRng).unwrap());

//             (
//                 Arc::new(Mutex::new(Queue {
//                     tests: vec!["first".into(), "second".into(), "third".into()],
//                 })),
//                 Arc::new(LstContext {
//                     staker: LocalSigner::new(
//                         SigningKey::random(&mut rng).to_bytes().into(),
//                         "union",
//                     ),
//                     union_address: UnionAddressBook {
//                         zkgm: calculate_cosmos_contract_address(
//                             &cfg.union_deployer_addr,
//                             SALT_ZKGM,
//                         )
//                         .unwrap(),
//                         lst_hub: calculate_cosmos_contract_address(
//                             &cfg.union_deployer_addr,
//                             SALT_LST_HUB,
//                         )
//                         .unwrap(),
//                         eu: calculate_cosmos_contract_address(&cfg.union_deployer_addr, SALT_EU)
//                             .unwrap(),
//                         escrow_vault: calculate_cosmos_contract_address(
//                             &cfg.union_deployer_addr,
//                             SALT_ESCROW_VAULT,
//                         )
//                         .unwrap(),
//                     },
//                     ctx,
//                 }),
//             )
//         })
//         .await;

//     loop {
//         {
//             if ctx.0.lock().await.tests[0] == key {
//                 return ctx.1.clone();
//             }
//         }

//         tokio::time::sleep(Duration::from_secs(1)).await
//     }
// }

// pub async fn ensure_channels_opened(channel_count: usize) {
//     CHANNELS_OPENED
//         .get_or_init(|| async move {
//             let t = init_ctx().await;

//             let (src_client, dst_client) = t
//                 .ctx
//                 .create_clients(
//                     Duration::from_secs(60),
//                     "ibc-cosmwasm",
//                     "trusted/evm/mpt",
//                     "ibc-solidity",
//                     "cometbls",
//                 )
//                 .await
//                 .unwrap();

//             assert!(src_client.client_id > 0);
//             assert!(dst_client.client_id > 0);

//             let conn = t
//                 .ctx
//                 .open_connection::<cosmos::Module, evm::Module>(
//                     &t.ctx.src,
//                     src_client.client_id,
//                     &t.ctx.dst,
//                     dst_client.client_id,
//                     Duration::from_secs(180),
//                 )
//                 .await
//                 .unwrap();
//             assert!(conn.connection_id > 0);
//             assert!(conn.counterparty_connection_id > 0);

//             let current_available_count = t.ctx.get_available_channel_count().await;

//             let opened = t
//                 .ctx
//                 .open_channels(
//                     true,
//                     t.union_address.zkgm.as_bytes().into(),
//                     ETH_ADDRESS_ZKGM.into_bytes(),
//                     conn.counterparty_connection_id,
//                     "ucs03-zkgm-0".into(),
//                     channel_count,
//                     Duration::from_secs(360 * channel_count as u64),
//                 )
//                 .await
//                 .unwrap();
//             assert_eq!(opened, channel_count);

//             let available_count_after_open = t.ctx.get_available_channel_count().await;
//             assert_eq!(
//                 current_available_count + channel_count,
//                 available_count_after_open
//             );
//             let pair = t.ctx.get_channel().await.expect("channel available");
//             let available_count_after_get = t.ctx.get_available_channel_count().await;
//             assert_eq!(available_count_after_open - 1, available_count_after_get);
//             t.ctx.release_channel(pair).await;
//             let available_count_after_release = t.ctx.get_available_channel_count().await;
//             assert_eq!(available_count_after_open, available_count_after_release);
//         })
//         .await;
// }

pub async fn eth_set_fungible_counterparty(
    module: &evm::Module,
    channel_id: u32,
    base_token: &[u8],
    beneficiary: &[u8],
) -> anyhow::Result<()> {
    info!("registering fungible counterparty");

    let (_, priviledged_account) = module.get_provider_privileged().await;
    module
        .u_register_fungible_counterpart(
            ETH_ADDRESS_U,
            priviledged_account.clone(),
            alloy::primitives::U256::ZERO,
            channel_id,
            base_token.to_vec().into(),
            evm::u::U::FungibleCounterparty {
                beneficiary: beneficiary.to_vec().into(),
            },
        )
        .await
        .unwrap();

    Ok(())
}

pub async fn eth_fund_u(
    t: &LstContext,
    src_channel_id: u32,
    receiver: H160,
    min_amount: u64,
    amount: u64,
) -> anyhow::Result<()> {
    let (_, evm_provider) = t.ctx.dst.get_provider().await;
    let (cosmos_address, cosmos_provider) = t.ctx.src.get_signer().await;

    let u_balance = t
        .ctx
        .dst
        .zkgmerc20_balance_of(ETH_ADDRESS_U, receiver, evm_provider.clone())
        .await
        .unwrap();

    if min_amount > amount {
        return Err(anyhow::anyhow!("you seriously wanna fund your contract less than min amount, when min amount is greater?"));
    }

    if u_balance > min_amount.into() {
        Ok(())
    } else {
        info!("the {receiver} is running low on U (u_balance), funding it with {amount}");

        let metadata = SolverMetadata {
            solverAddress: ETH_ADDRESS_U.into_bytes().into(),
            metadata: Default::default(),
        }
        .abi_encode_params();

        let amount = Into::<unionlabs::primitives::U256>::into(amount).into();

        let instruction_cosmos = Instruction {
            version: INSTR_VERSION_2,
            opcode: OP_TOKEN_ORDER,
            operand: TokenOrderV2 {
                sender: cosmos_address.to_string().into_bytes().into(),
                receiver: receiver.into_bytes().into(),
                base_token: "muno".as_bytes().into(),
                base_amount: amount,
                kind: TOKEN_ORDER_KIND_SOLVE,
                metadata: metadata.into(),
                quote_token: ETH_ADDRESS_U.into_bytes().into(),
                quote_amount: amount,
            }
            .abi_encode_params()
            .into(),
        };

        let mut salt_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut salt_bytes);

        let (_, ack) = t
            .ctx
            .send_and_recv_and_ack_with_retry::<cosmos::Module, evm::Module>(
                &t.ctx.src,
                t.union_address.zkgm.clone(),
                (
                    ucs03_zkgm::msg::ExecuteMsg::Send {
                        channel_id: src_channel_id.try_into().unwrap(),
                        timeout_height: 0u64.into(),
                        timeout_timestamp: voyager_sdk::primitives::Timestamp::from_secs(
                            u32::MAX.into(),
                        ),
                        salt: salt_bytes.into(),
                        instruction: instruction_cosmos.abi_encode_params().into(),
                    }
                    .encode_as::<Json>(),
                    vec![ProtoCoin {
                        denom: "muno".into(),
                        amount: amount.to_string(),
                    }],
                ),
                &t.ctx.dst,
                3,
                Duration::from_secs(20),
                Duration::from_secs(720),
                cosmos_provider,
            )
            .await?;

        // make sure the transfer is successful
        assert_eq!(ack.tag, TAG_ACK_SUCCESS);

        Ok(())
    }
}

pub mod evm_helper {
    use alloy::primitives::Address;

    use super::*;

    pub fn make_token_order_v2(
        escrow_vault_address: &Addr,
        sender: &Address,
        receiver: &Addr,
        amount: alloy::primitives::U256,
    ) -> Instruction {
        Instruction {
            version: INSTR_VERSION_2,
            opcode: OP_TOKEN_ORDER,
            operand: TokenOrderV2 {
                sender: sender.to_vec().into(),
                receiver: receiver.as_bytes().to_vec().into(),
                base_token: ETH_ADDRESS_U.into_bytes().into(),
                base_amount: amount,
                quote_token: b"muno".into(),
                quote_amount: amount,
                kind: TOKEN_ORDER_KIND_SOLVE,
                metadata: SolverMetadata {
                    solverAddress: escrow_vault_address.as_bytes().to_vec().into(),
                    metadata: Default::default(),
                }
                .abi_encode_params()
                .into(),
            }
            .abi_encode_params()
            .into(),
        }
    }
}
