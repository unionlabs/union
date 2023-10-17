use std::{fs::read_to_string, sync::Arc};

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
use unionlabs::ethereum_consts_traits::{Mainnet, Minimal};

use crate::cli::{AppArgs, Config};

mod cli;

#[cfg(not(feature = "eth-minimal"))]
pub type EvmConfig = Minimal;
#[cfg(feature = "eth-minimal")]
pub type EvmConfig = Mainnet;

#[tokio::main]
async fn main() {
    let args = AppArgs::parse();
    let config = read_to_string(&args.config_file_path).unwrap();
    let config = serde_json::from_str::<Config>(&config).unwrap();

    match args.command {
        cli::Command::Tx(tx) => match tx {
            cli::TxCmd::Evm(evm_tx) => {
                let evm: Evm<EvmConfig> = Evm::new(config.evm).await.unwrap();
                match evm_tx {
                    cli::EvmTx::Transfer {
                        relay_address,
                        port_id,
                        channel_id,
                        receiver,
                        amount,
                        denom,
                    } => {
                        handle_transfer(
                            evm,
                            relay_address.into(),
                            port_id,
                            channel_id,
                            receiver,
                            amount,
                            denom,
                        )
                        .await
                    }
                };
            }
        },
    }
}

async fn handle_transfer(
    evm: Evm<EvmConfig>,
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
    println!("{:?}", evm.wallet.address());

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
