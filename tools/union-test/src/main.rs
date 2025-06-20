use std::{str::FromStr, time::Duration};

use concurrent_keyring::{KeyringConfig, KeyringConfigEntry};
use cosmos::{FeemarketConfig, GasFillerConfig};
use hex_literal::hex;
use ibc_union_msg::msg::{ExecuteMsg, MsgCreateClient};
use unionlabs::{
    bech32::Bech32,
    encoding::{Bincode, EncodeAs, EthAbi},
};
use voyager_sdk::{anyhow, primitives::ChainId};

pub mod cosmos;
pub mod evm;
pub mod voyager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let union_config = cosmos::Config {
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

    let union = cosmos::Module::new(union_config).await?;

    let evm_config = evm::Config {
        chain_id: ChainId::new("32382"),
        ibc_handler_address: hex!("ed2af2aD7FE0D92011b26A2e5D1B4dC7D12A47C5").into(),
        multicall_address: hex!("84c4c2ee43ccfd523af9f78740256e0f60d38068").into(),
        rpc_url: "http://localhost:8545".into(),
        keyring: KeyringConfig {
            name: "alice".into(),
            keys: vec![KeyringConfigEntry::Raw {
                name: "alice".into(),
                key: hex!("4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77")
                    .to_vec(),
            }],
        },
        max_gas_price: None,
        fixed_gas_price: None,
        gas_multiplier: 2.0,
    };

    let eth = evm::Module::new(evm_config).await?;

    voyager::init_fetch(union.chain_id.clone())?;
    voyager::init_fetch(eth.chain_id.clone())?;

    voyager::create_client(
        union.chain_id.clone(),
        eth.chain_id.clone(),
        "ibc-cosmwasm".into(),
        "trusted/evm/mpt".into(),
    )?;

    voyager::create_client(
        eth.chain_id.clone(),
        union.chain_id.clone(),
        "ibc-solidity".into(),
        "cometbls".into(),
    )?;

    std::thread::sleep(Duration::from_secs(5));

    voyager::connection_open(union.chain_id.clone(), 1, 1)?;

    let confirm = eth.wait_for_connection_open_confirm().await?;

    let connection_id = confirm.connection_id;
    let counterparty_connection_id = confirm.counterparty_connection_id;

    println!("connections bro: {connection_id} {counterparty_connection_id}");

    /*

    1. configure and setup the union module
    2. configure and setup the evm module
    3. send create-client to voyager using cli and wait for the client creation event (on union)
    4. send create-client to voyager using cli and wait for the client creation event (on eth)
    5. send conn handshake to voyager and wait for the confirm event
    6. send chan handshake to voyager and wait for the confirm event

    7. run the actual test

    */

    Ok(())
}
