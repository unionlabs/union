use std::{fs::read_to_string, sync::Arc};

use clap::Parser;
use cli::Evm;
use contracts::{
    devnet_ownable_ibc_handler, erc20,
    shared_types::{
        IbcCoreChannelV1ChannelData, IbcCoreChannelV1CounterpartyData,
        IbcCoreCommitmentV1MerklePrefixData, IbcCoreConnectionV1ConnectionEndData,
        IbcCoreConnectionV1CounterpartyData, IbcCoreConnectionV1VersionData,
    },
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
                    cli::EvmTx::InitialChannel {
                        module_address,
                        channel_id,
                        port_id,
                        counterparty_port_id,
                        client_id,
                        connection_id,
                        counterparty_client_id,
                        counterparty_connection_id,
                        counterparty_channel_id,
                        version,
                    } => match config.evm {
                        cli::EvmChainConfig::Minimal(config) => {
                            setup_initial_channel(
                                Evm::new(config).await.unwrap(),
                                module_address.into(),
                                client_id,
                                connection_id,
                                channel_id,
                                port_id,
                                counterparty_client_id,
                                counterparty_connection_id,
                                counterparty_channel_id,
                                counterparty_port_id,
                                version,
                            )
                            .await
                        }
                        _ => unimplemented!(),
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
                } => match config.evm {
                    cli::EvmChainConfig::Mainnet(config) => {
                        handle_ucs_balance::<Mainnet>(
                            Evm::new(config).await.unwrap(),
                            contract_address.into(),
                            denom,
                            address.into(),
                        )
                        .await
                    }
                    cli::EvmChainConfig::Minimal(config) => {
                        handle_ucs_balance::<Minimal>(
                            Evm::new(config).await.unwrap(),
                            contract_address.into(),
                            denom,
                            address.into(),
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
        },
    }
}

// TODO(aeryz): Move these arguments into `channel` struct, and get `channel` and `counterparty_channel` as params
#[allow(clippy::too_many_arguments)]
async fn setup_initial_channel(
    evm: Evm<Minimal>,
    module_address: Address,
    client_id: String,
    connection_id: String,
    channel_id: String,
    port_id: String,
    counterparty_client_id: String,
    counterparty_connection_id: String,
    counterparty_channel_id: String,
    counterparty_port_id: String,
    version: String,
) {
    let signer_middleware = Arc::new(SignerMiddleware::new(
        evm.provider.clone(),
        evm.wallet.clone(),
    ));

    let ibc_handler = devnet_ownable_ibc_handler::DevnetOwnableIBCHandler::new(
        evm.ibc_handler_address,
        signer_middleware,
    );

    ibc_handler
        .setup_initial_channel(
            connection_id,
            IbcCoreConnectionV1ConnectionEndData {
                client_id,
                versions: vec![IbcCoreConnectionV1VersionData {
                    identifier: "1".into(),
                    features: vec!["ORDER_ORDERED".into(), "ORDER_UNORDERED".into()],
                }],
                state: 3,
                counterparty: IbcCoreConnectionV1CounterpartyData {
                    client_id: counterparty_client_id,
                    connection_id: counterparty_connection_id.clone(),
                    prefix: IbcCoreCommitmentV1MerklePrefixData {
                        key_prefix: b"ibc".to_vec().into(),
                    },
                },
                delay_period: 6,
            },
            port_id,
            channel_id.clone(),
            IbcCoreChannelV1ChannelData {
                state: 3,
                ordering: 1,
                counterparty: IbcCoreChannelV1CounterpartyData {
                    port_id: counterparty_port_id,
                    channel_id: counterparty_channel_id,
                },
                connection_hops: vec![counterparty_connection_id],
                version,
            },
            module_address,
        )
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();
}

async fn handle_ucs_balance<C: ChainSpec>(
    evm: Evm<C>,
    contract_address: Address,
    denom: String,
    address: Address,
) {
    let signer_middleware = Arc::new(SignerMiddleware::new(
        evm.provider.clone(),
        evm.wallet.clone(),
    ));
    let relay = UCS01Relay::new(contract_address, signer_middleware.clone());

    let denom = relay.denom_to_address(denom).await.unwrap();
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

    let denom = relay.denom_to_address(denom).await.unwrap();
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
