use std::{str::FromStr as _, sync::Arc, time::Duration};

use alloy_sol_types::SolValue as _;
use cosmwasm_std::Addr;
use hex_literal::hex;
use protos::cosmos::base::v1beta1::Coin as ProtoCoin;
use rand::RngCore as _;
use serde::Deserialize;
use tokio::sync::OnceCell;
use ucs03_zkgm::com::{
    Instruction, SolverMetadata, TokenOrderV2, INSTR_VERSION_2, OP_TOKEN_ORDER,
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
    primitives::{Bech32, H160},
};
use voyager_sdk::{anyhow, serde_json};

pub mod bond;

pub static CTX: OnceCell<Arc<LstContext>> = OnceCell::const_new();
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
    union_address: UnionAddressBook,
    ctx: TestContext<cosmos::Module, evm::Module>,
}

#[derive(Deserialize)]
pub struct Config {
    evm: evm::Config,
    union: cosmos::Config,
    needed_channel_count: u32,
    voyager_config_file_path: String,
    union_deployer_addr: String,
}

async fn init_ctx<'a>() -> Arc<LstContext> {
    CTX.get_or_init(|| async {
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

        Arc::new(LstContext {
            union_address: UnionAddressBook {
                zkgm: calculate_cosmos_contract_address(&cfg.union_deployer_addr, SALT_ZKGM)
                    .unwrap(),
                lst_hub: calculate_cosmos_contract_address(&cfg.union_deployer_addr, SALT_LST_HUB)
                    .unwrap(),
                eu: calculate_cosmos_contract_address(&cfg.union_deployer_addr, SALT_EU).unwrap(),
                escrow_vault: calculate_cosmos_contract_address(
                    &cfg.union_deployer_addr,
                    SALT_ESCROW_VAULT,
                )
                .unwrap(),
            },
            ctx,
        })
    })
    .await
    .clone()
}

pub async fn ensure_channels_opened(channel_count: usize) {
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
                    ETH_ADDRESS_ZKGM.into_bytes(),
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

pub async fn eth_set_fungible_counterparty(
    module: &evm::Module,
    channel_id: u32,
    base_token: &[u8],
    beneficiary: &[u8],
) -> anyhow::Result<()> {
    tracing::info!("registering fungible counterparty");

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
    sender: String,
    receiver: H160,
    min_amount: u64,
    amount: u64,
) -> anyhow::Result<()> {
    let (_, evm_provider) = t.ctx.dst.get_provider().await;
    let (_, cosmos_provider) = t.ctx.src.get_signer().await;

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
        let metadata = SolverMetadata {
            solverAddress: ETH_ADDRESS_U.into_bytes().into(),
            metadata: Default::default(),
        }
        .abi_encode_params();

        let instruction_cosmos = Instruction {
            version: INSTR_VERSION_2,
            opcode: OP_TOKEN_ORDER,
            operand: TokenOrderV2 {
                sender: sender.into_bytes().into(),
                receiver: receiver.into_bytes().into(),
                base_token: "muno".as_bytes().into(),
                base_amount: "100000".parse().unwrap(),
                kind: TOKEN_ORDER_KIND_SOLVE,
                metadata: metadata.into(),
                quote_token: ETH_ADDRESS_U.into_bytes().into(),
                quote_amount: Into::<unionlabs::primitives::U256>::into(amount).into(),
            }
            .abi_encode_params()
            .into(),
        };

        let mut salt_bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut salt_bytes);

        let _ = t
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

        Ok(())
    }
}
