use std::{
    future::Future,
    sync::{Arc, Mutex},
    time::Duration,
};

use alloy::{network::AnyNetwork, providers::DynProvider};
use alloy_sol_types::SolValue as _;
use cosmwasm_std::Addr;
use hex_literal::hex;
use protos::cosmos::base::v1beta1::Coin as ProtoCoin;
use rand::RngCore as _;
use serde::Deserialize;
use tokio::sync::OnceCell;
use tracing::info;
use tracing_subscriber::FmtSubscriber;
use ucs03_zkgm::com::{
    INSTR_VERSION_2, Instruction, OP_TOKEN_ORDER, SolverMetadata, TAG_ACK_SUCCESS,
    TOKEN_ORDER_KIND_SOLVE, TokenOrderV2,
};
use union_test::{
    TestContext,
    cosmos::{self},
    cosmos_helpers::{
        SALT_ESCROW_VAULT, SALT_EU, SALT_LST_HUB, SALT_LST_STAKER, SALT_ZKGM,
        calculate_cosmos_contract_address,
    },
    evm,
};
use unionlabs::{
    encoding::{EncodeAs, Json},
    primitives::H160,
};
use voyager_sdk::{anyhow, serde_json};

pub static CTX: OnceCell<(Mutex<Queue>, Arc<LstContext>)> = OnceCell::const_new();

pub const ETH_ADDRESS_U: H160 = H160::new(hex!("0c8C6f58156D10d18193A8fFdD853e1b9F8D8836"));
pub const ETH_ADDRESS_ZKGM: H160 = H160::new(hex!("05FD55C1AbE31D3ED09A76216cA8F0372f4B2eC5"));

#[allow(unused)]
pub struct UnionAddressBook {
    pub zkgm: Addr,
    pub lst_hub: Addr,
    pub eu: Addr,
    pub escrow_vault: Addr,
    pub lst_staker: Addr,
}

pub struct LstContext {
    pub union_address: UnionAddressBook,
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

#[derive(Clone)]
pub struct SharedData {
    pub stakers: Vec<(alloy::primitives::Address, DynProvider<AnyNetwork>)>,
}

pub struct Queue {
    tests: Vec<String>,
    shared_data: SharedData,
}

pub async fn run_test_in_queue<
    Fut: Future<Output = SharedData>,
    F: Fn(Arc<LstContext>, SharedData) -> Fut,
>(
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
            let cfg: Config = serde_json::from_str(include_str!("../config.json")).unwrap();

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

            (
                Mutex::new(Queue {
                    tests: {
                        let mut t = vec![
                            "bond".into(),
                            "redelegation".into(),
                            "redelegation_too_soon".into(),
                        ];
                        // "unbond".into(), "withdraw".into()];
                        t.reverse();
                        t
                    },
                    shared_data: SharedData { stakers: vec![] },
                }),
                Arc::new(LstContext {
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
                        lst_staker: calculate_cosmos_contract_address(
                            &cfg.union_deployer_addr,
                            SALT_LST_STAKER,
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
            if ctx.0.lock().unwrap().tests.last().unwrap() == key {
                let shared_data = ctx.0.lock().unwrap().shared_data.clone();
                let shared_data = test_fn(ctx.1.clone(), shared_data).await;
                let mut lock = ctx.0.lock().unwrap();
                lock.shared_data = shared_data;
                let _ = lock.tests.pop();
                return;
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

pub async fn eth_set_fungible_counterparty(
    module: &evm::Module,
    channel_id: u32,
    base_token: &[u8],
    beneficiary: &[u8],
) -> anyhow::Result<()> {
    info!("registering fungible counterparty");

    let (_, privileged_account) = module.get_provider_privileged().await;
    module
        .u_register_fungible_counterpart(
            ETH_ADDRESS_U,
            privileged_account.clone(),
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
        return Err(anyhow::anyhow!(
            "you seriously wanna fund your contract less than min amount, when min amount is greater?"
        ));
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
                base_token: "au".as_bytes().into(),
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
                        denom: "au".into(),
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
                quote_token: b"au".into(),
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
