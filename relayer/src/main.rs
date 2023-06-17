// *almost* stable, more than safe enough to use imo https://github.com/rust-lang/rfcs/pull/3425
#![feature(return_position_impl_trait_in_trait)]
#![allow(clippy::manual_async_fn)]

use std::{collections::HashMap, str::FromStr, time::Duration};

use bip32::{DerivationPath, Language, XPrv};
use clap::{Args, Parser, Subcommand};
use contracts::{
    glue::UnionIbcLightclientsCometblsV1ClientStateData,
    ibc_handler::{
        self, IBCHandler, IBCHandlerEvents, IbcCoreChannelV1ChannelData,
        IbcCoreChannelV1CounterpartyData, IbcCoreChannelV1PacketData,
    },
    shared_types::IbcCoreClientV1HeightData,
};
use ethers::{
    abi::AbiDecode,
    prelude::decode_logs,
    providers::Middleware,
    types::{Address, H256},
};
use futures::StreamExt;
use prost::Message;
use protos::{
    cosmos::{
        self,
        auth::v1beta1::{BaseAccount, QueryAccountRequest},
        base::v1beta1::Coin,
    },
    google::protobuf::Any,
    ibc::{
        applications::transfer::v1 as transfer_v1,
        core::{channel::v1 as channel_v1, client::v1 as client_v1},
    },
};
use tendermint_rpc::{
    event::EventData, query::EventType, SubscriptionClient, WebSocketClient, WebSocketClientUrl,
};

use crate::chain::{
    cosmos::Ethereum,
    evm::Cometbls,
    msgs::{
        self,
        channel::{MsgChannelOpenAck, MsgChannelOpenConfirm, MsgChannelOpenTry},
        connection::{
            MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit,
            MsgConnectionOpenTry,
        },
        MerklePrefix,
    },
    ClientState, Connect, LightClient,
};
use crate::{
    chain::msgs::channel::{self, Channel, MsgChannelOpenInit},
    cosmos_to_eth::PORT_ID,
    eth_to_cosmos::{broadcast_tx_commit, signer_from_pk},
};

pub mod chain;

#[allow(
    dead_code,
    unused_variables,
    unreachable_code,
    clippy::diverging_sub_expression,
    clippy::let_underscore_future
)]
mod cosmos_to_eth;
#[allow(
    dead_code,
    unused_variables,
    unreachable_code,
    clippy::diverging_sub_expression,
    clippy::let_underscore_future
)]
mod eth_to_cosmos;

const ETH_BEACON_RPC_API: &str = "http://localhost:9596";

const ETH_RPC_API: &str = "http://localhost:8545";

const CHANNEL_VERSION: &str = "ics20-1";

#[derive(Debug, Parser)]
pub struct AppArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    OpenConnection(OpenConnectionArgs),
    OpenChannel(OpenChannelArgs),
}

#[derive(Debug, Parser)]
pub struct OpenConnectionArgs {
    #[command(flatten)]
    args: ClientArgs,
}

#[derive(Debug, Parser)]
pub struct OpenChannelArgs {
    #[command(flatten)]
    args: ClientArgs,

    /// format is client_id/connection_id
    #[arg(long)]
    cometbls: ConnectionEndInfo,
    /// format is client_id/connection_id
    #[arg(long)]
    ethereum: ConnectionEndInfo,
}

#[derive(Debug, Clone)]
pub struct ConnectionEndInfo {
    client_id: String,
    connection_id: String,
}

impl FromStr for ConnectionEndInfo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split('/');

        let client_id = split.next().ok_or("client id missing".to_string())?;
        let connection_id = split.next().ok_or("connection id missing".to_string())?;

        let extra = split.collect::<Vec<_>>().join("");

        if extra.is_empty() {
            Ok(Self {
                client_id: client_id.to_string(),
                connection_id: connection_id.to_string(),
            })
        } else {
            Err(format!("erroneous extra data: {extra}"))
        }
    }
}

#[derive(Debug, Parser)]
pub struct ClientArgs {
    #[command(flatten)]
    cometbls: CometblsClientArgs,
    #[command(flatten)]
    ethereum: EthereumClientArgs,
}

#[derive(Debug, Args)]
pub struct CometblsClientArgs {
    /// OwnableIBCHandler => address
    #[arg(long)]
    pub ibc_handler_address: Address,
    /// CometblsClient => address
    #[arg(long)]
    pub cometbls_client_address: Address,
    /// ICS20TransferBank => address
    #[arg(long)]
    pub ics20_module_address: Address,
}

#[derive(Debug, Args)]
pub struct EthereumClientArgs {
    #[arg(long = "code-id")]
    pub wasm_code_id: H256,
}

// #[derive(Debug, Subcommand)]
// pub enum CreateClientArgs {
//     Cometbls { ibc_handler_address: Address },
//     Ethereum { wasm_code_id: H256 },
// }

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Registry::default()
    //     .with(
    //             .with_filter(tracing_subscriber::filter::EnvFilter::from_default_env()),
    //     )
    //     .init();

    let args = AppArgs::parse();

    // dbg!(get_wallet());

    // panic!();

    // cosmos::get_wasm_code().await

    // let mut sequence = 0;

    // eth_to_cosmos::create_wasm_client(sequence).await;

    // sequence += 1;

    // // dbg!(cosmos::query_for_wasm_light_client().await);

    // eth_to_cosmos::update_wasm_client(sequence).await;

    // cosmos_to_eth::update_contract().await;

    // let sequence = account_info_of_signer(&get_wallet()).await.sequence;

    // let mut sequence = 3;

    // let ibc_handler = create_ibc_handler_client(&args).await;

    // let bind_rcp: TransactionReceipt = ibc_handler
    //     .bind_port(PORT_ID.into(), args.ics20_module_address)
    //     .send()
    //     .await
    //     .unwrap()
    //     .await
    //     .unwrap()
    //     .unwrap();

    // let connection_id = handshake(ibc_handler.clone(), &args).await;

    // "connection-0".to_string()
    // channel_handshake(ibc_handler, connection_id).await;

    // relay_packets(ibc_handler).await;

    do_main(args).await
}

async fn do_main(args: AppArgs) {
    // println!(
    //     "{}",
    //     wasm::v1::query_client::QueryClient::connect("tcp://0.0.0.0:9090")
    //         .await
    //         .unwrap()
    //         .code_ids(QueryCodeIdsRequest { pagination: None })
    //         .await
    //         .unwrap()
    //         .into_inner()
    //         .code_ids
    //         .first()
    //         .unwrap()
    // );

    match args.command {
        Command::OpenConnection(OpenConnectionArgs { args }) => {
            let cometbls = Cometbls::new(
                args.cometbls.cometbls_client_address,
                args.cometbls.ibc_handler_address,
                args.cometbls.ics20_module_address,
                args.ethereum.wasm_code_id,
            )
            .await;

            let ethereum = Ethereum::new(get_wallet(), args.ethereum.wasm_code_id).await;

            connection_handshake(cometbls, ethereum).await;
        }
        Command::OpenChannel(OpenChannelArgs {
            args,
            cometbls,
            ethereum,
        }) => {
            let cometbls_lc = Cometbls::new(
                args.cometbls.cometbls_client_address,
                args.cometbls.ibc_handler_address,
                args.cometbls.ics20_module_address,
                args.ethereum.wasm_code_id,
            )
            .await;

            let ethereum_lc = Ethereum::new(get_wallet(), args.ethereum.wasm_code_id).await;

            channel_handshake(cometbls_lc, ethereum_lc, cometbls, ethereum).await;
        }
    }

    // panic!();
}

async fn account_info_of_signer(signer: &XPrv) -> BaseAccount {
    let account = cosmos::auth::v1beta1::query_client::QueryClient::connect("http://0.0.0.0:9090")
        .await
        .unwrap()
        .account(QueryAccountRequest {
            address: signer_from_pk(&signer.public_key().public_key().to_bytes().to_vec()),
        })
        .await
        .unwrap()
        .into_inner()
        .account
        .unwrap();

    assert!(account.type_url == "/cosmos.auth.v1beta1.BaseAccount");

    BaseAccount::decode(&*account.value).unwrap()
}

// const API_URL: &str = "http://127.0.0.1:27444";

// fn default_merkle_prefix() -> MerklePrefix {
//     MerklePrefix {
//         key_prefix: b"ibc".to_vec(),
//     }
// }

fn get_wallet() -> XPrv {
    const MNEMONIC: &str = "wine parrot nominee girl exchange element pudding grow area twenty next junior come render shadow evidence sentence start rough debate feed all limb real";
    // const DERIVATION_PATH: &str = "m/44'/1337'/0'/0/0";
    const DERIVATION_PATH: &str = "m/44'/118'/0'/0/0";
    const PASSWORD: &str = "";

    let mnemonic = bip32::Mnemonic::new(MNEMONIC, Language::English);

    let derivation_path = DerivationPath::from_str(DERIVATION_PATH).unwrap();

    let alice = XPrv::derive_from_path(
        mnemonic.unwrap().to_seed(PASSWORD).as_bytes(),
        &derivation_path,
    )
    .unwrap();

    alice
}

async fn connection_handshake<Chain1, Chain2>(cometbls: Chain1, ethereum: Chain2)
where
    Chain1: LightClient + Connect<Chain2>,
    Chain2: LightClient + Connect<Chain1>,
    <Chain1 as LightClient>::ClientState: std::fmt::Debug,
    <Chain2 as LightClient>::ClientState: std::fmt::Debug,
{
    let cometbls_id = cometbls.chain_id().await;
    let ethereum_id = ethereum.chain_id().await;

    tracing::info!(cometbls_id, ethereum_id);

    let (cometbls_client_id, ethereum_latest_height) = {
        let latest_height = ethereum.query_latest_height().await;

        tracing::trace!("generating client state...");
        let client_state = ethereum
            .generate_counterparty_client_state(latest_height)
            .await;
        tracing::trace!("generating consensus state...");
        let consensus_state = ethereum
            .generate_counterparty_consensus_state(latest_height)
            .await;

        let client_id = cometbls.create_client(client_state, consensus_state).await;

        tracing::info!(chain_id = cometbls_id, client_id);

        (client_id, latest_height)
    };

    let (ethereum_client_id, cometbls_latest_height) = {
        let latest_height = cometbls.query_latest_height().await;

        tracing::trace!("generating client state...");
        let client_state = cometbls
            .generate_counterparty_client_state(latest_height)
            .await;
        tracing::trace!("generating consensus state...");
        let consensus_state = cometbls
            .generate_counterparty_consensus_state(latest_height)
            .await;

        let client_id = ethereum.create_client(client_state, consensus_state).await;

        tracing::info!(chain_id = ethereum_id, client_id);

        (client_id, latest_height)
    };

    tracing::info!(?cometbls_latest_height);
    tracing::info!(?ethereum_latest_height);

    let cometbls_connection_id = cometbls
        .connection_open_init(MsgConnectionOpenInit {
            client_id: cometbls_client_id.clone(),
            counterparty: msgs::connection::Counterparty {
                client_id: ethereum_client_id.clone(),
                // TODO(benluelo): Create a new struct with this field omitted as it's unused for open init
                connection_id: "".to_string(),
                prefix: MerklePrefix {
                    key_prefix: b"ibc".to_vec(),
                },
            },
            version: msgs::connection::Version {
                identifier: "1".into(),
                features: [channel::Order::Unordered, channel::Order::Ordered]
                    .into_iter()
                    .collect(),
            },
            delay_period: 6,
        })
        .await;

    let cometbls_update_from = cometbls_latest_height;
    let cometbls_update_to = cometbls.query_latest_height().await;

    let cometbls_latest_height = cometbls
        .update_counterparty_client(
            &ethereum,
            ethereum_client_id.clone(),
            cometbls_update_from,
            cometbls_update_to,
        )
        .await;

    tracing::info!(
        chain_id = cometbls_id,
        connection_id = cometbls_connection_id
    );

    // generate state proofs

    let cometbls_client_state_proof = cometbls
        .client_state_proof(cometbls_client_id.clone(), cometbls_latest_height)
        .await;
    let cometbls_consensus_state_proof = cometbls
        .consensus_state_proof(
            cometbls_client_id.clone(),
            ethereum_latest_height,
            cometbls_latest_height,
        )
        .await;
    let cometbls_connection_state_proof = cometbls
        .connection_state_proof(cometbls_connection_id.clone(), cometbls_latest_height)
        .await;

    let ethereum_connection_id = ethereum
        .connection_open_try(MsgConnectionOpenTry {
            client_id: ethereum_client_id.clone(),
            counterparty: msgs::connection::Counterparty {
                client_id: cometbls_client_id.clone(),
                connection_id: cometbls_connection_id.clone(),
                prefix: MerklePrefix {
                    key_prefix: b"ibc".to_vec(),
                },
            },
            delay_period: 6,
            client_state: cometbls_client_state_proof.state,
            counterparty_versions: cometbls_connection_state_proof.state.versions,
            proof_height: cometbls_consensus_state_proof.proof_height,
            proof_init: cometbls_connection_state_proof.proof,
            proof_client: cometbls_client_state_proof.proof,
            proof_consensus: cometbls_consensus_state_proof.proof,
            consensus_height: ethereum_latest_height,
        })
        .await;

    let ethereum_update_from = ethereum_latest_height;
    let ethereum_update_to = ethereum.query_latest_height().await;

    let ethereum_latest_height = ethereum
        .update_counterparty_client(
            &cometbls,
            cometbls_client_id.clone(),
            ethereum_update_from,
            ethereum_update_to,
        )
        .await;

    let ethereum_connection_state_proof = ethereum
        .connection_state_proof(ethereum_connection_id.clone(), ethereum_latest_height)
        .await;
    let ethereum_client_state_proof = ethereum
        .client_state_proof(ethereum_client_id.clone(), ethereum_latest_height)
        .await;
    let ethereum_consensus_state_proof = ethereum
        .consensus_state_proof(
            ethereum_client_id.clone(),
            cometbls_latest_height,
            ethereum_latest_height,
        )
        .await;

    cometbls
        .connection_open_ack(MsgConnectionOpenAck {
            connection_id: cometbls_connection_id.clone(),
            counterparty_connection_id: ethereum_connection_id.clone(),
            version: msgs::connection::Version {
                identifier: "1".into(),
                features: [channel::Order::Unordered, channel::Order::Ordered]
                    .into_iter()
                    .collect(),
            },
            client_state: ethereum_client_state_proof.state,
            proof_height: ethereum_connection_state_proof.proof_height,
            proof_try: ethereum_connection_state_proof.proof,
            proof_client: ethereum_client_state_proof.proof,
            proof_consensus: ethereum_consensus_state_proof.proof,
            consensus_height: ethereum_consensus_state_proof.proof_height,
        })
        .await;

    let cometbls_update_from = cometbls_latest_height;
    let cometbls_update_to = cometbls.query_latest_height().await;

    let cometbls_latest_height = cometbls
        .update_counterparty_client(
            &ethereum,
            ethereum_client_id.clone(),
            cometbls_update_from,
            cometbls_update_to,
        )
        .await;

    let cometbls_connection_state_proof = cometbls
        .connection_state_proof(cometbls_connection_id.clone(), cometbls_latest_height)
        .await;

    ethereum
        .connection_open_confirm(MsgConnectionOpenConfirm {
            connection_id: ethereum_connection_id.clone(),
            proof_ack: cometbls_connection_state_proof.proof,
            proof_height: cometbls_connection_state_proof.proof_height,
        })
        .await;

    tracing::info!(
        cometbls_connection_id,
        cometbls_client_id,
        ethereum_connection_id,
        ethereum_client_id,
        "connection opened"
    );
}

async fn channel_handshake<Chain1, Chain2>(
    cometbls: Chain1,
    ethereum: Chain2,
    cometbls_connection_info: ConnectionEndInfo,
    ethereum_connection_info: ConnectionEndInfo,
) where
    Chain1: LightClient + Connect<Chain2>,
    Chain2: LightClient + Connect<Chain1>,
    <Chain1 as LightClient>::ClientState: std::fmt::Debug + ClientState,
    <Chain2 as LightClient>::ClientState: std::fmt::Debug + ClientState,
{
    let cometbls_id = cometbls.chain_id().await;
    let ethereum_id = ethereum.chain_id().await;

    tracing::info!(cometbls_id, ethereum_id);

    const TRANSFER_PORT_ID: &str = "transfer";

    let cometbls_channel_id = cometbls
        .channel_open_init(MsgChannelOpenInit {
            port_id: TRANSFER_PORT_ID.to_string(),
            channel: Channel {
                state: channel::State::Init,
                ordering: channel::Order::Unordered,
                counterparty: channel::Counterparty {
                    port_id: TRANSFER_PORT_ID.to_string(),
                    channel_id: "".to_string(),
                },
                connection_hops: vec![cometbls_connection_info.connection_id.clone()],
                version: CHANNEL_VERSION.to_string(),
            },
        })
        .await;

    let ethereum_latest_trusted_height = ethereum
        .query_client_state(ethereum_connection_info.client_id.clone())
        .await
        .height();

    let cometbls_latest_height = cometbls.query_latest_height().await;

    let cometbls_latest_height = cometbls
        .update_counterparty_client(
            &ethereum,
            ethereum_connection_info.client_id.clone(),
            ethereum_latest_trusted_height,
            cometbls_latest_height,
        )
        .await;

    let proof = cometbls
        .channel_state_proof(
            cometbls_channel_id.clone(),
            TRANSFER_PORT_ID.to_string(),
            cometbls_latest_height,
        )
        .await;

    let ethereum_channel_id = ethereum
        .channel_open_try(MsgChannelOpenTry {
            port_id: TRANSFER_PORT_ID.to_string(),
            channel: Channel {
                state: channel::State::Tryopen,
                ordering: channel::Order::Unordered,
                counterparty: channel::Counterparty {
                    port_id: TRANSFER_PORT_ID.to_string(),
                    channel_id: cometbls_channel_id.clone(),
                },
                connection_hops: vec![cometbls_connection_info.connection_id.clone()],
                version: CHANNEL_VERSION.to_string(),
            },
            counterparty_version: CHANNEL_VERSION.to_string(),
            proof_init: proof.proof,
            proof_height: proof.proof_height,
        })
        .await;

    let cometbls_latest_trusted_height = cometbls
        .query_client_state(cometbls_connection_info.client_id.clone())
        .await
        .height();

    let ethereum_latest_height = ethereum.query_latest_height().await;

    let ethereum_latest_height = ethereum
        .update_counterparty_client(
            &cometbls,
            cometbls_connection_info.client_id.clone(),
            cometbls_latest_trusted_height,
            ethereum_latest_height,
        )
        .await;

    let proof = ethereum
        .channel_state_proof(
            ethereum_channel_id.clone(),
            TRANSFER_PORT_ID.to_string(),
            ethereum_latest_height,
        )
        .await;

    cometbls.channel_open_ack(MsgChannelOpenAck {
        port_id: TRANSFER_PORT_ID.to_string(),
        channel_id: cometbls_channel_id.clone(),
        counterparty_channel_id: ethereum_channel_id.clone(),
        counterparty_version: CHANNEL_VERSION.to_string(),
        proof_try: proof.proof,
        proof_height: proof.proof_height,
    });

    let ethereum_latest_trusted_height = ethereum
        .query_client_state(ethereum_connection_info.client_id.clone())
        .await
        .height();

    let cometbls_latest_height = cometbls.query_latest_height().await;

    let cometbls_latest_height = cometbls
        .update_counterparty_client(
            &ethereum,
            ethereum_connection_info.client_id.clone(),
            ethereum_latest_trusted_height,
            cometbls_latest_height,
        )
        .await;

    let proof = cometbls
        .channel_state_proof(
            cometbls_channel_id.clone(),
            TRANSFER_PORT_ID.to_string(),
            cometbls_latest_height,
        )
        .await;

    ethereum
        .channel_open_confirm(MsgChannelOpenConfirm {
            port_id: TRANSFER_PORT_ID.to_string(),
            channel_id: ethereum_channel_id.clone(),
            proof_ack: proof.proof,
            proof_height: proof.proof_height,
        })
        .await;

    tracing::info!(
        cometbls_connection_info.connection_id,
        cometbls_connection_info.client_id,
        cometbls_channel_id,
        ethereum_connection_info.connection_id,
        ethereum_connection_info.client_id,
        ethereum_channel_id,
        "channel opened"
    );
}

// #[allow(
//     dead_code,
//     unused_variables,
//     unreachable_code,
//     clippy::diverging_sub_expression,
//     clippy::let_underscore_future
// )]
// async fn handshake<M>(ibc_handler: IBCHandler<M>, args: &Args) -> String
// where
//     M: Middleware + 'static,
// {
//     const COMETBLS_CLIENT_ID: &str = "cometbls-0";

//     let (tm_client, tm_driver) = WebSocketClient::builder(
//         WebSocketClientUrl::from_str("ws://0.0.0.0:26657/websocket").unwrap(),
//     )
//     .compat_mode(tendermint_rpc::client::CompatMode::V0_37)
//     .build()
//     .await
//     .unwrap();

//     // let (rx, tx) = tendermint_rpc::client::sync::unbounded();

//     let _ = tokio::spawn(async move { tm_driver.run().await });

//     let mut staking_client =
//         staking::v1beta1::query_client::QueryClient::connect("http://0.0.0.0:9090")
//             .await
//             .unwrap();

//     let staking_params = staking_client
//         .params(staking::v1beta1::QueryParamsRequest {})
//         .await
//         .unwrap()
//         .into_inner()
//         .params
//         .unwrap();

//     let commit: commit::Response = tm_client.latest_commit().await.unwrap();

//     ibc_handler
//         .register_client(COMETBLS_CLIENT_TYPE.into(), args.cometbls_client_address)
//         .send()
//         .await
//         .unwrap()
//         .await
//         .unwrap();

//     println!("Creating client...");

//     let eth_client_id = create_client(&ibc_handler, &commit, &staking_params).await;

//     let create_wasm_client_response = create_wasm_client().await;

//     dbg!(create_wasm_client_response);

//     let alice = get_wallet();
//     let alice_pk = alice.public_key().public_key().to_bytes().to_vec();

//     let msg = protos::google::protobuf::Any {
//         type_url: "/ibc.core.connection.v1.MsgConnectionOpenInit".into(),
//         value: connection_v1::MsgConnectionOpenInit {
//             client_id: WASM_CLIENT_ID.to_string(),
//             counterparty: Some(connection_v1::Counterparty {
//                 client_id: eth_client_id.clone(),
//                 connection_id: "".to_string(),
//                 prefix: Some(default_merkle_prefix().into()),
//             }),
//             version: Some(todo!()),
//             delay_period: 0,
//             signer: signer_from_pk(&alice_pk),
//         }
//         .encode_to_vec(),
//     };

//     let response = broadcast_tx_commit([msg].to_vec()).await;

//     dbg!(&response);

//     let connection_id = response
//         .deliver_tx
//         .events
//         .into_iter()
//         .find(|event| event.kind == "connection_open_init")
//         .unwrap()
//         .attributes
//         .into_iter()
//         .find(|attr| attr.key == "connection_id")
//         .unwrap()
//         .value;

//     let mut connection_query_client =
//         connection_v1::query_client::QueryClient::connect("http://0.0.0.0:9090")
//             .await
//             .unwrap();

//     let connection_proof = connection_query_client
//         .connection(connection_v1::QueryConnectionRequest {
//             connection_id: connection_id.clone(),
//         })
//         .await
//         .unwrap()
//         .into_inner();

//     let mut client_query_client =
//         client_v1::query_client::QueryClient::connect("http://0.0.0.0:9090")
//             .await
//             .unwrap();

//     let client_state_proof = client_query_client
//         .client_state(client_v1::QueryClientStateRequest {
//             client_id: WASM_CLIENT_ID.to_string(),
//         })
//         .await
//         .unwrap()
//         .into_inner();

//     let consensus_state_proof = client_query_client
//         .consensus_state(client_v1::QueryConsensusStateRequest {
//             client_id: WASM_CLIENT_ID.to_string(),
//             revision_number: connection_proof
//                 .proof_height
//                 .clone()
//                 .unwrap()
//                 .revision_number,
//             revision_height: 0,
//             latest_height: true,
//         })
//         .await
//         .unwrap()
//         .into_inner();

//     dbg!(std::time::SystemTime::now());

//     let try_response = ibc_handler
//         .connection_open_try(ibc_handler::MsgConnectionOpenTry {
//             counterparty: IbcCoreConnectionV1CounterpartyData {
//                 client_id: WASM_CLIENT_ID.to_string(),
//                 connection_id: connection_id.clone(),
//                 prefix: IbcCoreCommitmentV1MerklePrefixData {
//                     key_prefix: default_merkle_prefix().key_prefix.into(),
//                 },
//             },
//             delay_period: 0,
//             client_id: COMETBLS_CLIENT_ID.to_string(),
//             // for membership verification, however it's stored in the store
//             // i.e. ibc/clientStates/whatever
//             // TYPE: proto(wasm<eth::v1::clientstate>)
//             // WasmEth::ClientState (proto encoded)
//             client_state_bytes: Default::default(),
//             counterparty_versions: [IbcCoreConnectionV1VersionData {
//                 // identifier: default_connection_version().identifier,
//                 // features: default_connection_version().features,
//                 identifier: todo!(),
//                 features: todo!(),
//             }]
//             .to_vec(),
//             proof_init: connection_proof.proof.into(),
//             proof_client: client_state_proof.proof.into(),
//             proof_consensus: consensus_state_proof.proof.into(),
//             proof_height: IbcCoreClientV1HeightData {
//                 revision_number: connection_proof
//                     .proof_height
//                     .clone()
//                     .unwrap()
//                     .revision_number,
//                 revision_height: connection_proof.proof_height.unwrap().revision_height,
//             },
//             consensus_height: IbcCoreClientV1HeightData {
//                 revision_number: consensus_state_proof
//                     .proof_height
//                     .clone()
//                     .unwrap()
//                     .revision_number,
//                 revision_height: consensus_state_proof
//                     .proof_height
//                     .clone()
//                     .unwrap()
//                     .revision_height,
//             },
//         })
//         .send()
//         .await
//         .unwrap()
//         .await
//         .unwrap()
//         .unwrap();

//     dbg!(std::time::SystemTime::now());

//     dbg!(try_response);

//     let (cometbls_client_state_bytes, is_found) = ibc_handler
//         .get_client_state(COMETBLS_CLIENT_ID.to_string())
//         .await
//         .unwrap();

//     assert!(is_found);

//     let cometbls_client_state: UnionIbcLightclientsCometblsV1ClientStateData =
//         AbiDecode::decode(cometbls_client_state_bytes).unwrap();

//     dbg!(&cometbls_client_state);

//     let wasm_client_state =
//         wasm_v1::ClientState::decode(&*client_state_proof.client_state.unwrap().value).unwrap();

//     dbg!(&wasm_client_state);

//     #[allow(deprecated)]
//     let msg = protos::google::protobuf::Any {
//         type_url: "/ibc.core.connection.v1.MsgConnectionOpenAck".into(),
//         value: connection_v1::MsgConnectionOpenAck {
//             connection_id: connection_id.clone(),
//             counterparty_connection_id: connection_id.clone(),
//             version: Some(todo!()),
//             // version: Some(default_connection_version()),
//             client_state: Some(protos::google::protobuf::Any {
//                 type_url: "/ibc.lightclients.wasm.v1.ClientState".to_string(),
//                 value: wasm_v1::ClientState {
//                     data: protos::google::protobuf::Any {
//                         type_url: "/ibc.lightclients.tendermint.v1.ClientState".to_string(),
//                         value: tendermint_v1::ClientState {
//                             chain_id: CHAIN_ID.to_string(),
//                             trust_level: Some(tendermint_v1::Fraction {
//                                 // numerator: cometbls_client_state.trust_level.numerator,
//                                 // denominator: cometbls_client_state.trust_level.denominator,
//                                 numerator: 1,
//                                 denominator: 3,
//                             }),
//                             trusting_period: Some(protobuf::Duration {
//                                 // seconds: cometbls_client_state.trusting_period.seconds,
//                                 // nanos: cometbls_client_state.trusting_period.nanos,
//                                 seconds: 1814400,
//                                 nanos: 0,
//                             }),
//                             unbonding_period: Some(protobuf::Duration {
//                                 // seconds: cometbls_client_state.unbonding_period.seconds,
//                                 // nanos: cometbls_client_state.unbonding_period.nanos,
//                                 seconds: 1814400,
//                                 nanos: 0,
//                             }),
//                             max_clock_drift: Some(protobuf::Duration {
//                                 // seconds: cometbls_client_state.max_clock_drift.seconds,
//                                 // nanos: cometbls_client_state.max_clock_drift.nanos,
//                                 seconds: 40,
//                                 nanos: 0,
//                             }),
//                             frozen_height: Some(client_v1::Height {
//                                 revision_number: cometbls_client_state
//                                     .frozen_height
//                                     .revision_number,
//                                 revision_height: cometbls_client_state
//                                     .frozen_height
//                                     .revision_height,
//                             }),
//                             latest_height: Some(client_v1::Height {
//                                 // revision_number: cometbls_client_state
//                                 //     .latest_height
//                                 //     .revision_number,
//                                 revision_number: 1,
//                                 revision_height: todo!(),
//                             }),
//                             proof_specs: [
//                                 ics23_v1::ProofSpec {
//                                     leaf_spec: Some(ics23_v1::LeafOp {
//                                         hash: ics23_v1::HashOp::Sha256 as _,
//                                         prehash_key: ics23_v1::HashOp::NoHash as _,
//                                         prehash_value: ics23_v1::HashOp::Sha256 as _,
//                                         length: ics23_v1::LengthOp::VarProto as _,
//                                         prefix: [0].to_vec(),
//                                     }),
//                                     inner_spec: Some(ics23_v1::InnerSpec {
//                                         child_order: vec![0, 1],
//                                         child_size: 33,
//                                         min_prefix_length: 4,
//                                         max_prefix_length: 12,
//                                         empty_child: vec![],
//                                         hash: ics23_v1::HashOp::Sha256 as _,
//                                     }),
//                                     max_depth: 0,
//                                     min_depth: 0,
//                                 },
//                                 ics23_v1::ProofSpec {
//                                     leaf_spec: Some(ics23_v1::LeafOp {
//                                         hash: ics23_v1::HashOp::Sha256 as _,
//                                         prehash_key: ics23_v1::HashOp::NoHash as _,
//                                         prehash_value: ics23_v1::HashOp::Sha256 as _,
//                                         length: ics23_v1::LengthOp::VarProto as _,
//                                         prefix: [0].to_vec(),
//                                     }),
//                                     inner_spec: Some(ics23_v1::InnerSpec {
//                                         child_order: vec![0, 1],
//                                         child_size: 32,
//                                         min_prefix_length: 1,
//                                         max_prefix_length: 1,
//                                         empty_child: vec![],
//                                         hash: ics23_v1::HashOp::Sha256 as _,
//                                     }),
//                                     max_depth: 0,
//                                     min_depth: 0,
//                                 },
//                             ]
//                             .to_vec(),
//                             upgrade_path: ["upgrade".to_string(), "upgradedIBCState".to_string()]
//                                 .to_vec(),
//                             // TODO: figure out where to get these values from
//                             allow_update_after_expiry: true,
//                             allow_update_after_misbehaviour: true,
//                         }
//                         .encode_to_vec(),
//                     }
//                     .encode_to_vec(),
//                     code_id: wasm_client_state.code_id,
//                     latest_height: Some(client_v1::Height {
//                         revision_number: 1,
//                         revision_height: wasm_client_state
//                             .latest_height
//                             .clone()
//                             .unwrap()
//                             .revision_height,
//                     }),
//                 }
//                 .encode_to_vec(),
//             }),
//             proof_height: wasm_client_state.latest_height.clone(),
//             proof_try: vec![1, 2, 3],
//             proof_client: vec![1, 2, 3],
//             proof_consensus: vec![1, 2, 3],
//             consensus_height: consensus_state_proof.proof_height.clone(),
//             signer: signer_from_pk(&alice_pk),
//             host_consensus_state_proof: vec![],
//         }
//         .encode_to_vec(),
//     };

//     let ack_response = broadcast_tx_commit([msg].to_vec()).await;

//     dbg!(ack_response);

//     let connection_proof = connection_query_client
//         .connection(connection_v1::QueryConnectionRequest {
//             connection_id: connection_id.clone(),
//         })
//         .await
//         .unwrap()
//         .into_inner();

//     dbg!(&connection_proof);

//     ibc_handler
//         .connection_open_confirm(ibc_handler::MsgConnectionOpenConfirm {
//             connection_id: connection_id.clone(),
//             proof_ack: connection_proof.proof.into(),
//             proof_height: IbcCoreClientV1HeightData {
//                 revision_number: connection_proof
//                     .proof_height
//                     .clone()
//                     .unwrap()
//                     .revision_number,
//                 revision_height: connection_proof.proof_height.unwrap().revision_height,
//             },
//         })
//         .send()
//         .await
//         .unwrap()
//         .await
//         .unwrap()
//         .unwrap();

//     connection_id
// }

#[allow(
    dead_code,
    unused_variables,
    unreachable_code,
    clippy::diverging_sub_expression,
    clippy::let_underscore_future
)]
async fn channel_handshake_old<M>(ibc_handler: IBCHandler<M>, connection_id: String)
where
    M: Middleware + 'static,
{
    const CHANNEL_VERSION: &str = "ics20-1";
    const COMETBLS_CLIENT_ID: &str = "cometbls-0";

    // let wasm_client_update = client_v1::MsgUpdateClient {
    //     client_id: WASM_CLIENT_ID.to_string(),
    //     client_message: todo!(),
    //     signer: todo!(),
    // };

    // let a_end = connection_v1::ChannelEnd {
    //     client_id: CLIENT_A_ID.to_string(),
    //     versions: vec![default_connection_version()],
    //     state: connection_v1::State::Init.into(),
    //     counterparty: Some(connection_v1::Counterparty {
    //         client_id: CLIENT_B_ID.to_string(),
    //         connection_id: "connection-1".to_string(),
    //         prefix: Some(default_merkle_prefix()),
    //     }),
    //     delay_period: 0,
    // };

    let (_tm_client, tm_driver) = WebSocketClient::builder(
        WebSocketClientUrl::from_str("ws://0.0.0.0:26657/websocket").unwrap(),
    )
    .compat_mode(tendermint_rpc::client::CompatMode::V0_37)
    .build()
    .await
    .unwrap();

    // let (rx, tx) = tendermint_rpc::client::sync::unbounded();

    let _ = tokio::spawn(async move { tm_driver.run().await });

    let alice = get_wallet();
    let alice_pk = alice.public_key().public_key().to_bytes().to_vec();

    let msg = protos::google::protobuf::Any {
        type_url: "/ibc.core.channel.v1.MsgChannelOpenInit".into(),
        value: channel_v1::MsgChannelOpenInit {
            signer: signer_from_pk(&alice_pk),
            port_id: PORT_ID.to_string(),
            channel: Some(channel_v1::Channel {
                state: channel_v1::State::Init as i32,
                ordering: channel_v1::Order::Unordered as i32,
                counterparty: Some(channel_v1::Counterparty {
                    port_id: PORT_ID.to_string(),
                    channel_id: "".to_string(),
                }),
                connection_hops: vec![connection_id.clone()],
                version: CHANNEL_VERSION.to_string(),
            }),
        }
        .encode_to_vec(),
    };

    let response = broadcast_tx_commit([msg].to_vec()).await;

    dbg!(&response);

    let cosmos_channel_id = response
        .deliver_tx
        .events
        .into_iter()
        .find(|event| event.kind == "channel_open_init")
        .unwrap()
        .attributes
        .into_iter()
        .find(|attr| attr.key == "channel_id")
        .unwrap()
        .value;

    let mut channel_query_client =
        channel_v1::query_client::QueryClient::connect("http://0.0.0.0:9090")
            .await
            .unwrap();

    let channel_proof = channel_query_client
        .channel(channel_v1::QueryChannelRequest {
            port_id: PORT_ID.to_string(),
            channel_id: cosmos_channel_id.clone(),
        })
        .await
        .unwrap()
        .into_inner();

    let channel_open_try_receipt = ibc_handler
        .channel_open_try(ibc_handler::MsgChannelOpenTry {
            proof_init: channel_proof.proof.clone().into(),
            proof_height: IbcCoreClientV1HeightData {
                revision_number: channel_proof.proof_height.clone().unwrap().revision_number,
                revision_height: channel_proof.proof_height.clone().unwrap().revision_height,
            },
            port_id: PORT_ID.to_string(),
            channel: IbcCoreChannelV1ChannelData {
                state: channel_v1::State::Tryopen as u8,
                ordering: channel_v1::Order::Ordered as u8,
                counterparty: IbcCoreChannelV1CounterpartyData {
                    port_id: PORT_ID.to_string(),
                    channel_id: cosmos_channel_id.clone(),
                },
                connection_hops: vec![connection_id],
                version: CHANNEL_VERSION.to_string(),
            },
            counterparty_version: CHANNEL_VERSION.to_string(),
        })
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    let eth_channel_id = decode_logs::<IBCHandlerEvents>(
        channel_open_try_receipt
            .logs
            .into_iter()
            .map(|l| l.into())
            .collect::<Vec<_>>()
            .as_ref(),
    )
    .unwrap()
    .into_iter()
    .find_map(|l| match l {
        IBCHandlerEvents::GeneratedChannelIdentifierFilter(channel_id) => Some(channel_id.0),
        _ => None,
    })
    .unwrap();

    dbg!(&eth_channel_id);

    let (cometbls_client_state_bytes, is_found) = ibc_handler
        .get_client_state(COMETBLS_CLIENT_ID.to_string())
        .await
        .unwrap();

    assert!(is_found);

    let cometbls_client_state: UnionIbcLightclientsCometblsV1ClientStateData =
        AbiDecode::decode(cometbls_client_state_bytes).unwrap();

    dbg!(&cometbls_client_state);

    let mut client_query_client =
        client_v1::query_client::QueryClient::connect("http://0.0.0.0:9090")
            .await
            .unwrap();

    let consensus_state_proof = client_query_client
        .consensus_state(client_v1::QueryConsensusStateRequest {
            client_id: "".to_string(),
            revision_number: channel_proof.proof_height.clone().unwrap().revision_number,
            revision_height: 0,
            latest_height: true,
        })
        .await
        .unwrap()
        .into_inner();

    dbg!(&consensus_state_proof);

    let height = client_query_client
        .consensus_state_heights(client_v1::QueryConsensusStateHeightsRequest {
            client_id: "".to_string(),
            pagination: None,
        })
        .await
        .unwrap()
        .into_inner()
        .consensus_state_heights
        .into_iter()
        .max()
        .unwrap();

    let msg = protos::google::protobuf::Any {
        type_url: "/ibc.core.channel.v1.MsgChannelOpenAck".into(),
        value: channel_v1::MsgChannelOpenAck {
            proof_height: Some(height),
            proof_try: vec![1, 2, 3],
            signer: signer_from_pk(&alice_pk),
            port_id: PORT_ID.to_string(),
            channel_id: cosmos_channel_id.clone(),
            counterparty_channel_id: eth_channel_id.clone(),
            counterparty_version: CHANNEL_VERSION.to_string(),
        }
        .encode_to_vec(),
    };

    let ack_response = broadcast_tx_commit([msg].to_vec()).await;

    dbg!(ack_response);

    let channel_proof = channel_query_client
        .channel(channel_v1::QueryChannelRequest {
            port_id: PORT_ID.to_string(),
            channel_id: cosmos_channel_id.clone(),
        })
        .await
        .unwrap()
        .into_inner();

    dbg!(&channel_proof);

    ibc_handler
        .channel_open_confirm(ibc_handler::MsgChannelOpenConfirm {
            port_id: PORT_ID.to_string(),
            channel_id: eth_channel_id.clone(),
            proof_ack: channel_proof.proof.into(),
            proof_height: IbcCoreClientV1HeightData {
                revision_number: channel_proof.proof_height.clone().unwrap().revision_number,
                revision_height: channel_proof.proof_height.unwrap().revision_height,
            },
        })
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    println!("successfully opened channel");
}

#[allow(
    dead_code,
    unused_variables,
    unreachable_code,
    clippy::diverging_sub_expression,
    clippy::let_underscore_future
)]
async fn relay_packets(ibc_handler: IBCHandler<impl Middleware + 'static>) {
    let listen_handle = tokio::spawn(async move {
        loop {
            let (client, driver) =
                WebSocketClient::builder("ws://127.0.0.1:26657/websocket".parse().unwrap())
                    .compat_mode(tendermint_rpc::client::CompatMode::V0_37)
                    .build()
                    .await
                    .unwrap();

            let driver_handle = tokio::spawn(async move { driver.run().await });

            // Subscription functionality
            let mut subs = client.subscribe(EventType::Tx.into()).await.unwrap();

            while let Some(res) = subs.next().await {
                let ev = res.unwrap();

                // ibc_transfer { sender, reciever, amount, denom, memo? }

                println!("Got event: {:#?}", ev.events);

                match ev.data {
                    EventData::NewBlock {
                        block: _,
                        result_begin_block: _,
                        result_end_block: _,
                    } => {
                        // dbg!(result_begin_block, result_end_block);

                        // client.block(block.unwrap().header.height).await.unwrap();
                    }
                    EventData::Tx { tx_result } => {
                        let send_packet_event = tx_result
                            .result
                            .events
                            .into_iter()
                            .find_map(|e| {
                                (e.kind == "send_packet").then(|| {
                                    e.attributes
                                        .into_iter()
                                        .map(|attr| (attr.key, attr.value))
                                        .collect::<HashMap<_, _>>()
                                })
                            })
                            .unwrap();

                        let sequence = send_packet_event["packet_sequence"].parse().unwrap();

                        let packet_commitment =
                            channel_v1::query_client::QueryClient::connect("http://0.0.0.0:9090")
                                .await
                                .unwrap()
                                .packet_commitment(channel_v1::QueryPacketCommitmentRequest {
                                    port_id: PORT_ID.to_string(),
                                    channel_id: "channel-0".to_string(),
                                    sequence,
                                })
                                .await
                                .unwrap()
                                .into_inner();

                        let rcp = ibc_handler
                            .recv_packet(ibc_handler::MsgPacketRecv {
                                packet: IbcCoreChannelV1PacketData {
                                    sequence,
                                    source_port: send_packet_event["packet_src_port"].clone(),
                                    source_channel: send_packet_event["packet_src_channel"].clone(),
                                    destination_port: send_packet_event["packet_dst_port"].clone(),
                                    destination_channel: send_packet_event["packet_dst_channel"]
                                        .clone(),
                                    data: send_packet_event["packet_data"]
                                        .clone()
                                        .into_bytes()
                                        .into(),
                                    timeout_height: {
                                        let (revision, height) = send_packet_event
                                            ["packet_timeout_height"]
                                            .split_once('-')
                                            .unwrap();

                                        IbcCoreClientV1HeightData {
                                            revision_number: revision.parse().unwrap(),
                                            revision_height: height.parse().unwrap(),
                                        }
                                    },
                                    timeout_timestamp: send_packet_event
                                        ["packet_timeout_timestamp"]
                                        .parse()
                                        .unwrap(),
                                },
                                proof: packet_commitment.proof.into(),
                                proof_height: IbcCoreClientV1HeightData {
                                    revision_number: packet_commitment
                                        .proof_height
                                        .as_ref()
                                        .unwrap()
                                        .revision_number,
                                    revision_height: packet_commitment
                                        .proof_height
                                        .unwrap()
                                        .revision_height,
                                },
                            })
                            .send()
                            .await
                            .unwrap()
                            .await
                            .unwrap()
                            .unwrap();

                        dbg!(rcp);
                    }
                    EventData::GenericJsonEvent(_) => todo!(),
                };
            }

            println!("events finished");

            // Signal to the driver to terminate.
            client.close().unwrap();

            // Await the driver's termination to ensure proper connection closure.
            let _ = driver_handle.await.unwrap();
        }
    });

    let send_handle = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(20)).await;

        let msg = transfer_v1::MsgTransfer {
            source_port: PORT_ID.to_string(),
            source_channel: "channel-0".to_string(),
            token: Some(Coin {
                denom: "stake".to_string(),
                amount: "1".to_string(),
            }),
            sender: signer_from_pk(&get_wallet().public_key().public_key().to_bytes().to_vec()),
            receiver: "union1nrv37pqfcqul73v7d2e8y0jhjyeuhg57m3eqdt".to_string(),
            timeout_height: Some(client_v1::Height {
                revision_number: 1,
                revision_height: 12_345_678_765,
            }),
            timeout_timestamp: Default::default(),
            memo: Default::default(),
        };

        broadcast_tx_commit(
            [Any {
                type_url: "/ibc.applications.transfer.v1.MsgTransfer".to_string(),
                value: msg.encode_to_vec(),
            }]
            .to_vec(),
        )
        .await;
    });

    let (listen, send) = tokio::join!(listen_handle, send_handle);

    listen.unwrap();
    send.unwrap();
}
