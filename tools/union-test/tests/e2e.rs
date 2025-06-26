// tests/e2e.rs

use std::sync::Arc;
use tokio::sync::OnceCell;
use serial_test::serial;
use union_test::{   // adjust to your crate name
    TestContext,
    evm::{self, Config as EvmConfig},
    cosmos::{self, Config as CosmosConfig},
    helpers::{CreateClientConfirm, ConnectionConfirm, ChannelOpenConfirm},
};
use std::{str::FromStr, time::Duration};

use cosmos::{FeemarketConfig, GasFillerConfig};
use concurrent_keyring::{KeyringConfig, KeyringConfigEntry};
use voyager_sdk::{anyhow, primitives::ChainId};
use hex_literal::hex;
use unionlabs::{
    bech32::Bech32,
    encoding::{Bincode, EncodeAs, EthAbi},
};
use tokio::sync::Mutex;

static CTX: OnceCell<Arc<TestContext<cosmos::Module, evm::Module>>> = OnceCell::const_new();
static CLC: OnceCell<CreateClientConfirm>     = OnceCell::const_new();
static DLC: OnceCell<CreateClientConfirm>     = OnceCell::const_new();
static CONN_FROM_UNION_TO_EVM: OnceCell<ConnectionConfirm>      = OnceCell::const_new();
static CHAN_FROM_UNION_TO_EVM: OnceCell<ChannelOpenConfirm>     = OnceCell::const_new();
static CONN_FROM_EVM_TO_UNION: OnceCell<ConnectionConfirm>      = OnceCell::const_new();
static CHAN_FROM_EVM_TO_UNION: OnceCell<ChannelOpenConfirm>     = OnceCell::const_new();


async fn init_ctx() -> Arc<TestContext<cosmos::Module, evm::Module>> {
    CTX.get_or_init(|| async {
            let cosmos_cfg = cosmos::Config {
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
            let evm_cfg = evm::Config {
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
            let src = cosmos::Module::new(cosmos_cfg).await.unwrap();
            let dst = evm::Module::new(evm_cfg).await.unwrap();
            let ctx = TestContext::new(src, dst).await
                .unwrap_or_else(|e| panic!("failed to build TestContext: {:#?}", e));
            Arc::new(ctx)
    })
    .await
    .clone()
}

async fn open_connection_from_union_to_evm() {
    let ctx = init_ctx().await;
    let (src_client, dst_client) = ctx
        .create_clients(
            Duration::from_secs(45),
            "ibc-cosmwasm", "trusted/evm/mpt",
            "ibc-solidity", "cometbls",
        )
        .await
        .unwrap();
    
    assert!(src_client.client_id > 0);
    assert!(dst_client.client_id > 0);
    CLC.set(src_client.clone()).unwrap();
    DLC.set(dst_client.clone()).unwrap();

    let conn = ctx
        .open_connection(true, src_client.client_id, dst_client.client_id, Duration::from_secs(180))
        .await
        .unwrap();
    assert!(conn.connection_id > 0);
    assert!(conn.counterparty_connection_id > 0);
    CONN_FROM_UNION_TO_EVM.set(conn).unwrap();

}


async fn open_connection_from_evm_to_union() {
    let ctx = init_ctx().await;
    let (src_client, dst_client) = ctx
        .create_clients(
            Duration::from_secs(45),
            "ibc-cosmwasm", "trusted/evm/mpt",
            "ibc-solidity", "cometbls",
        )
        .await
        .unwrap();
    
    assert!(src_client.client_id > 0);
    assert!(dst_client.client_id > 0);
    // CLC.set(src_client.clone()).unwrap();
    // DLC.set(dst_client.clone()).unwrap();

    let conn = ctx
        .open_connection(false, src_client.client_id, dst_client.client_id, Duration::from_secs(180))
        .await
        .unwrap();
    assert!(conn.connection_id > 0);
    assert!(conn.counterparty_connection_id > 0);
    CONN_FROM_EVM_TO_UNION.set(conn).unwrap();

}


#[tokio::test]
#[serial]
async fn test_open_channel_from_union_to_evm() {
    let ctx = init_ctx().await;
    open_connection_from_union_to_evm().await;
    let conn = CONN_FROM_UNION_TO_EVM.get().expect("connection available");
    
    let current_available_count = ctx.get_available_channel_count().await;

    let opened = ctx
        .open_channels(
            true,
           "union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c".as_bytes().into(),
            hex!("05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5").to_vec().into(),
            conn.connection_id,
            "ucs03-zkgm-0".into(),
            1,
            Duration::from_secs(240),
        )
        .await
        .unwrap();
    assert_eq!(opened, 1);

    let available_count_after_open = ctx.get_available_channel_count().await;
    assert_eq!(current_available_count + 1, available_count_after_open);
    let pair = ctx.get_channel().await.expect("channel available");
    let available_count_after_get = ctx.get_available_channel_count().await;
    assert_eq!(available_count_after_open - 1, available_count_after_get);
    ctx.release_channel(pair).await;
    let available_count_after_release = ctx.get_available_channel_count().await;
    assert_eq!(available_count_after_open, available_count_after_release);
}


#[tokio::test]
#[serial]
async fn test_open_channel_from_evm_to_union() {
    let ctx = init_ctx().await;
    open_connection_from_evm_to_union().await;
    let conn = CONN_FROM_EVM_TO_UNION.get().expect("connection available");

    let current_available_count = ctx.get_available_channel_count().await;

    let opened = ctx
        .open_channels(
            false,
           "union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c".as_bytes().into(),
            hex!("05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5").to_vec().into(),
            conn.connection_id,
            "ucs03-zkgm-0".into(),
            1,
            Duration::from_secs(240),
        )
        .await
        .unwrap();
    assert_eq!(opened, 1);

    let available_count_after_open = ctx.get_available_channel_count().await;
    assert_eq!(current_available_count + 1, available_count_after_open);
    let pair = ctx.get_channel().await.expect("channel available");
    let available_count_after_get = ctx.get_available_channel_count().await;
    assert_eq!(available_count_after_open - 1, available_count_after_get);
    ctx.release_channel(pair).await;
    let available_count_after_release = ctx.get_available_channel_count().await;
    assert_eq!(available_count_after_open, available_count_after_release);
}


