use std::{fs::read_to_string, sync::Arc};

use chain_utils::{cosmos_sdk::CosmosSdkChainExt, union::Union};
use clap::Parser;
use cli::Evm;
use contracts::{
    erc20,
    ucs01_relay::{LocalToken, UCS01Relay},
};
use ethers::{
    middleware::SignerMiddleware,
    signers::Signer,
    types::{Address, U256},
};
use unionlabs::ethereum::config::{ChainSpec, Mainnet, Minimal};

use crate::cli::{AppArgs, Config};

mod cli;

#[tokio::main]
async fn main() {
    let args = AppArgs::parse();
    let config = read_to_string(&args.config_file_path).unwrap();
    let config = serde_json::from_str::<Config>(&config).unwrap();

    match args.command {
        cli::Command::Tx(tx) => match tx {
            cli::TxCmd::Evm(evm_tx) => {
                match evm_tx {
                    cli::EvmTx::Transfer {
                        relay_address,
                        port_id,
                        channel_id,
                        receiver,
                        amount,
                        denom,
                    } => match config.evm {
                        cli::EvmChainConfig::Mainnet(config) => {
                            handle_transfer::<Mainnet>(
                                Evm::new(config).await.unwrap(),
                                relay_address.into(),
                                port_id,
                                channel_id,
                                receiver,
                                amount,
                                denom,
                            )
                            .await
                        }
                        cli::EvmChainConfig::Minimal(config) => {
                            handle_transfer::<Minimal>(
                                Evm::new(config).await.unwrap(),
                                relay_address.into(),
                                port_id,
                                channel_id,
                                receiver,
                                amount,
                                denom,
                            )
                            .await
                        }
                    },
                };
            }
        },
        cli::Command::Query(query) => match query {
            cli::QueryCmd::Evm(evm_query) => match evm_query {
                cli::EvmQuery::Ucs01Balance {
                    contract_address,
                    denom,
                    address,
                    channel_id,
                    port_id,
                } => match config.evm {
                    cli::EvmChainConfig::Mainnet(config) => {
                        handle_ucs_balance::<Mainnet>(
                            Evm::new(config).await.unwrap(),
                            contract_address.into(),
                            denom,
                            address.into(),
                            channel_id,
                            port_id,
                        )
                        .await
                    }
                    cli::EvmChainConfig::Minimal(config) => {
                        handle_ucs_balance::<Minimal>(
                            Evm::new(config).await.unwrap(),
                            contract_address.into(),
                            denom,
                            address.into(),
                            channel_id,
                            port_id,
                        )
                        .await
                    }
                },
                cli::EvmQuery::Erc20Balance {
                    contract_address,
                    address,
                } => match config.evm {
                    cli::EvmChainConfig::Mainnet(config) => {
                        handle_erc_balance::<Mainnet>(
                            Evm::new(config).await.unwrap(),
                            contract_address.into(),
                            address.into(),
                        )
                        .await
                    }
                    cli::EvmChainConfig::Minimal(config) => {
                        handle_erc_balance::<Minimal>(
                            Evm::new(config).await.unwrap(),
                            contract_address.into(),
                            address.into(),
                        )
                        .await
                    }
                },
            },
            cli::QueryCmd::Union(union_query) => match union_query {
                cli::UnionQuery::AccountInfo { address } => {
                    let info = Union::new(chain_utils::union::Config {
                        signers: config.union.signers,
                        fee_denom: config.union.fee_denom,
                        ws_url: config.union.ws_url,
                        prover_endpoint: config.union.prover_endpoint,
                        grpc_url: config.union.grpc_url,
                    })
                    .await
                    .unwrap()
                    .account_info(&address)
                    .await;
                    println!("{info:#?}");
                }
            },
        },
    }
}

async fn handle_ucs_balance<C: ChainSpec>(
    evm: Evm<C>,
    contract_address: Address,
    denom: String,
    address: Address,
    channel_id: String,
    port_id: String,
) {
    let signer_middleware = Arc::new(SignerMiddleware::new(
        evm.provider.clone(),
        evm.wallet.clone(),
    ));
    let relay = UCS01Relay::new(contract_address, signer_middleware.clone());

    let denom = relay
        .get_denom_address(port_id, channel_id, denom)
        .await
        .unwrap();
    println!("Corresponding ERC20 address: {}", denom);

    let erc_contract = erc20::ERC20::new(denom, signer_middleware.clone());

    let balance = erc_contract.balance_of(address).await.unwrap();
    println!("Balance is: {}", balance);
}

async fn handle_erc_balance<C: ChainSpec>(
    evm: Evm<C>,
    contract_address: Address,
    address: Address,
) {
    let signer_middleware = Arc::new(SignerMiddleware::new(
        evm.provider.clone(),
        evm.wallet.clone(),
    ));
    let erc_contract = erc20::ERC20::new(contract_address, signer_middleware);

    let balance = erc_contract.balance_of(address).await.unwrap();
    println!("Balance is: {}", balance);
}

async fn handle_transfer<C: ChainSpec>(
    evm: Evm<C>,
    relay_address: Address,
    port_id: String,
    channel_id: String,
    receiver: String,
    amount: u64,
    denom: String,
) {
    let signer_middleware = Arc::new(SignerMiddleware::new(
        evm.provider.clone(),
        evm.wallet.clone(),
    ));
    let relay = UCS01Relay::new(relay_address, signer_middleware.clone());

    let denom = relay
        .get_denom_address(port_id.clone(), channel_id.clone(), denom)
        .await
        .unwrap();
    println!("Address is: {}", denom);

    let erc_contract = erc20::ERC20::new(denom, signer_middleware.clone());

    let balance = erc_contract.balance_of(evm.wallet.address()).await.unwrap();
    println!("Balance is: {}", balance);

    erc_contract
        .approve(relay_address, U256::max_value() / 2)
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    let tx_rcp = relay
        .send(
            port_id,
            channel_id,
            hex::decode(receiver).unwrap().into(),
            [LocalToken {
                denom,
                amount: amount.into(),
            }]
            .into(),
            u64::MAX,
            u64::MAX,
        )
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    dbg!(tx_rcp);
}
