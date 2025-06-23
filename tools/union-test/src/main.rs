use std::{str::FromStr, time::Duration};

use alloy::sol_types::SolValue;
use concurrent_keyring::{KeyringConfig, KeyringConfigEntry};
use cosmos::{FeemarketConfig, GasFillerConfig};
use hex_literal::hex;
use ibc_union_msg::msg::{ExecuteMsg, MsgCreateClient};
use ibc_union_spec::{ChannelId, Timestamp};
use protos::cosmos::base::v1beta1::Coin;
use ucs03_zkgm::com::{
    FungibleAssetOrder, FungibleAssetOrderV2, Instruction, INSTR_VERSION_1, OP_FUNGIBLE_ASSET_ORDER,
};
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

    let res = eth.wait_for_create_client(Duration::from_secs(15)).await;

    let counterparty_client_id = match res {
        Ok(confirm) => {
            println!(
                "✅ got create client result. client_id: {}",
                confirm.client_id,
            );
            confirm.client_id
        }
        Err(err) => {
            eprintln!("⚠️  error waiting for create-client-confirm: {}", err);
            return Ok(());
        }
    };

    std::thread::sleep(Duration::from_secs(5));

    voyager::connection_open(union.chain_id.clone(), 1, counterparty_client_id)?;

    let res = eth.wait_for_connection_open_confirm(Duration::from_secs(180)).await;

    let connection_id = match res {
        Ok(confirm) => {
            println!(
                "✅ got connection confirm: {} ↔ {}",
                confirm.connection_id,
                confirm.counterparty_connection_id,
            );
            confirm.connection_id
        }
        Err (err) => {
            println!("Error occured when waiting for connection open confirm. Err: {}", err);
            return Ok(());
        }
    };


    voyager::channel_open(
        union.chain_id.clone(),
        "union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c"
            .as_bytes()
            .into(),
        hex!("05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5")
            .to_vec()
            .into(),
        connection_id,
        "ucs03-zkgm-0".into(),
    )?;

    let channel_id = match 
        eth.wait_for_channel_open_confirm(Duration::from_secs(240)).await
    {
        Ok(confirm) => {
            println!(
                "✅ got channel confirm: {} ↔ {}",
                confirm.channel_id,
                confirm.counterparty_channel_id,
            );
            
            confirm.channel_id.try_into().unwrap()
            
        }
        Err(err) => {
            eprintln!("⚠️  error waiting for channel-open-confirm: {}", err);
            return Ok(());
        }
    };

    let cosmos::IbcEvent::WasmPacketSend { packet_hash, .. } = union
        .send_ibc_transaction(
            Bech32::from_str("union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c")
                .unwrap(),
            vec![(
                Box::new(ucs03_zkgm::msg::ExecuteMsg::Send {
                    channel_id,
                    timeout_height: 0u64.into(),
                    timeout_timestamp: Timestamp::from_secs(u32::MAX.into()),
                    salt: Default::default(),
                    instruction: Instruction {
                        version: INSTR_VERSION_1,
                        opcode: OP_FUNGIBLE_ASSET_ORDER,
                        operand: FungibleAssetOrder {
                            sender: "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"
                                .as_bytes()
                                .into(),
                            receiver: hex!("Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD")
                                .to_vec()
                                .into(),
                            base_token: "muno".as_bytes().into(),
                            base_amount: "10".parse().unwrap(),
                            base_token_symbol: "muno".into(),
                            base_token_name: "muno".into(),
                            base_token_decimals: 6,
                            base_token_path: "0".parse().unwrap(),
                            quote_token: hex!("16628cB81ffDA9B8470e16299eFa5F76bF45A579")
                                .to_vec()
                                .into(),
                            quote_amount: "10".parse().unwrap(),
                        }
                        .abi_encode_params()
                        .into(),
                    }
                    .abi_encode_params()
                    .into(),
                }),
                vec![Coin {
                    denom: "muno".into(),
                    amount: "10".into(),
                }],
            )],
        )
        .await
        .unwrap()
        .unwrap();

    let recv = match eth
        .wait_for_packet_recv(packet_hash, Duration::from_secs(240))
        .await
    {
        Ok(ev) => {
            println!("✅ packet received: {:?}", ev);
            ev
        }
        Err(err) => {
            eprintln!("⚠️  error waiting for PacketRecv: {}", err);
            return Ok(());
        }
    };


    Ok(())
}
