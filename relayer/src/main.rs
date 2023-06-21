// *almost* stable, more than safe enough to use imo https://github.com/rust-lang/rfcs/pull/3425
#![feature(return_position_impl_trait_in_trait)]
// #![warn(clippy::pedantic)]
#![allow(clippy::manual_async_fn)]

// nix run .# -- tx wasm instantiate 1 '{"default_timeout":10000,"gov_contract":"union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2","allowlist":[]}' --label blah --from alice --gas auto --keyring-backend test --gas-adjustment 1.3 --amount 100stake --no-admin --chain-id union-devnet-1

use std::{collections::HashMap, str::FromStr};

use bip32::{DerivationPath, Language, XPrv};
use chain::evm::CometblsConfig;
use clap::{Args, Parser, Subcommand};
use ethers::{
    prelude::{EthAbiCodec, EthAbiType},
    signers::LocalWallet,
    types::{Address, H256},
};
use futures::StreamExt;
use ibc_types::core::{
    channel::{
        self, channel::Channel, msg_channel_open_ack::MsgChannelOpenAck,
        msg_channel_open_confirm::MsgChannelOpenConfirm, msg_channel_open_init::MsgChannelOpenInit,
        msg_channel_open_try::MsgChannelOpenTry, msg_recv_packet::MsgRecvPacket, order::Order,
        packet::Packet,
    },
    client::height::Height,
    commitment::merkle_prefix::MerklePrefix,
    connection::{
        self, msg_channel_open_ack::MsgConnectionOpenAck,
        msg_channel_open_confirm::MsgConnectionOpenConfirm,
        msg_channel_open_init::MsgConnectionOpenInit, msg_channel_open_try::MsgConnectionOpenTry,
        version::Version,
    },
};
use protos::ibc::core::channel::v1 as channel_v1;
use reqwest::Url;
use tendermint_rpc::{event::EventData, query::EventType, SubscriptionClient};

use crate::chain::{cosmos::Ethereum, evm::Cometbls, ClientState, Connect, LightClient};

pub mod chain;

// const ETH_BEACON_RPC_API: &str = "http://localhost:9596";
// const ETH_RPC_API: &str = "http://localhost:8545";
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
    RelayPackets(RelayPacketsArgs),
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

    #[arg(long)]
    cometbls_port_id: String,
    #[arg(long)]
    ethereum_port_id: String,

    /// format is client_id/connection_id
    #[arg(long)]
    cometbls: ConnectionEndInfo,
    /// format is client_id/connection_id
    #[arg(long)]
    ethereum: ConnectionEndInfo,
}

#[derive(Debug, Parser)]
pub struct RelayPacketsArgs {
    #[command(flatten)]
    args: ClientArgs,

    #[arg(long)]
    open_channel: bool,

    #[arg(long)]
    cometbls_port_id: Option<String>,
    #[arg(long)]
    ethereum_port_id: Option<String>,
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
    pub ics20_transfer_address: Address,
    /// ICS20Bank => address
    #[arg(long)]
    pub ics20_bank_address: Address,

    #[arg(long)]
    pub wallet: LocalWallet,

    #[arg(long)]
    pub eth_rpc_api: Url,

    #[arg(long)]
    pub eth_beacon_rpc_api: String,
}

#[derive(Debug, Args)]
pub struct EthereumClientArgs {
    #[arg(long = "code-id")]
    pub wasm_code_id: H256,
}

#[derive(Debug, Subcommand)]
pub enum CreateClientArgs {
    Cometbls { ibc_handler_address: Address },
    Ethereum { wasm_code_id: H256 },
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = AppArgs::parse();

    do_main(args).await;
}

#[derive(Debug, EthAbiCodec, EthAbiType)]
pub struct Ics20Packet {
    /// amount of tokens to transfer is encoded as a string, but limited to u64 max
    pub amount: u64,
    /// the token denomination to be transferred
    pub denom: String,
    /// the recipient address on the destination chain
    pub receiver: String,
    /// the sender address
    pub sender: String,
}

async fn do_main(args: AppArgs) {
    // let packet_hex = hex!("0000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000057374616b6500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000146161616161353535353561616161613535353535000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a3479743267320000000000000000000000000000000000000000");

    // dbg!(Ics20Packet::decode(left).unwrap());
    // dbg!(Ics20Packet::decode(right).unwrap());

    // panic!();

    match args.command {
        Command::OpenConnection(OpenConnectionArgs { args }) => {
            let cometbls = Cometbls::new(CometblsConfig {
                cometbls_client_address: args.cometbls.cometbls_client_address,
                ibc_handler_address: args.cometbls.ibc_handler_address,
                ics20_transfer_address: args.cometbls.ics20_transfer_address,
                ics20_bank_address: args.cometbls.ics20_bank_address,
                wasm_code_id: args.ethereum.wasm_code_id,
                wallet: args.cometbls.wallet,
                eth_rpc_api: args.cometbls.eth_rpc_api,
                eth_beacon_rpc_api: args.cometbls.eth_beacon_rpc_api,
            })
            .await;

            let ethereum = Ethereum::new(get_wallet(), args.ethereum.wasm_code_id).await;

            connection_handshake(&cometbls, &ethereum).await;
        }
        Command::OpenChannel(OpenChannelArgs {
            args,
            cometbls,
            ethereum,
            cometbls_port_id,
            ethereum_port_id,
        }) => {
            let cometbls_lc = Cometbls::new(CometblsConfig {
                cometbls_client_address: args.cometbls.cometbls_client_address,
                ibc_handler_address: args.cometbls.ibc_handler_address,
                ics20_transfer_address: args.cometbls.ics20_transfer_address,
                ics20_bank_address: args.cometbls.ics20_bank_address,
                wasm_code_id: args.ethereum.wasm_code_id,
                wallet: args.cometbls.wallet,
                eth_rpc_api: args.cometbls.eth_rpc_api,
                eth_beacon_rpc_api: args.cometbls.eth_beacon_rpc_api,
            })
            .await;

            let ethereum_lc = Ethereum::new(get_wallet(), args.ethereum.wasm_code_id).await;

            channel_handshake(
                &cometbls_lc,
                &ethereum_lc,
                cometbls,
                ethereum,
                cometbls_port_id,
                ethereum_port_id,
            )
            .await;
        }
        Command::RelayPackets(RelayPacketsArgs {
            args,
            open_channel,
            cometbls_port_id,
            ethereum_port_id,
        }) => {
            let cometbls_lc = Cometbls::new(CometblsConfig {
                cometbls_client_address: args.cometbls.cometbls_client_address,
                ibc_handler_address: args.cometbls.ibc_handler_address,
                ics20_transfer_address: args.cometbls.ics20_transfer_address,
                ics20_bank_address: args.cometbls.ics20_bank_address,
                wasm_code_id: args.ethereum.wasm_code_id,
                wallet: args.cometbls.wallet,
                eth_rpc_api: args.cometbls.eth_rpc_api,
                eth_beacon_rpc_api: args.cometbls.eth_beacon_rpc_api,
            })
            .await;

            // let channel: Channel = cometbls_lc
            //     .ibc_handler
            //     .get_channel("transfer".to_string(), "channel-2".to_string())
            //     .await
            //     .unwrap()
            //     .0
            //     .try_into()
            //     .unwrap();

            // dbg!(channel);

            // panic!();

            let ethereum_lc = Ethereum::new(get_wallet(), args.ethereum.wasm_code_id).await;

            // ICS20Transfer.sol -> onRecvPacket -> replace body with `return _newAcknowledgement(true);`

            // let balance = cometbls_lc
            //     .ics20_bank
            //     .balance_of(
            //         H160::from(b"aaaaa55555aaaaa55555"),
            //         format!("{}/channel-0/stake", cometbls_port_id.clone().unwrap()),
            //     )
            //     .await
            //     .unwrap();
            // dbg!(balance);

            if open_channel {
                let (
                    cometbls_client_id,
                    ethereum_client_id,
                    cometbls_connection_id,
                    ethereum_connection_id,
                ) = connection_handshake(&cometbls_lc, &ethereum_lc).await;

                let (_cometbls_channel_id, _ethereum_channel_id) = channel_handshake(
                    &cometbls_lc,
                    &ethereum_lc,
                    ConnectionEndInfo {
                        client_id: cometbls_client_id,
                        connection_id: cometbls_connection_id,
                    },
                    ConnectionEndInfo {
                        client_id: ethereum_client_id,
                        connection_id: ethereum_connection_id,
                    },
                    cometbls_port_id.unwrap(),
                    ethereum_port_id.unwrap(),
                )
                .await;
            }

            relay_packets(cometbls_lc, ethereum_lc).await;
        }
    }
}

// TODO(benluelo): Pass this in as as command line argument
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

/// Returns (c1 client id, c2 client id, c1 conn id, c2 conn id)
async fn connection_handshake<Chain1, Chain2>(
    cometbls: &Chain1,
    ethereum: &Chain2,
) -> (String, String, String, String)
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
            counterparty: connection::counterparty::Counterparty {
                client_id: ethereum_client_id.clone(),
                // TODO(benluelo): Create a new struct with this field omitted as it's unused for open init
                connection_id: "".to_string(),
                prefix: MerklePrefix {
                    key_prefix: b"ibc".to_vec(),
                },
            },
            version: Version {
                identifier: "1".into(),
                features: [Order::Unordered, Order::Ordered].into_iter().collect(),
            },
            delay_period: 6,
        })
        .await;

    let cometbls_update_from = cometbls_latest_height;
    let cometbls_update_to = cometbls.query_latest_height().await;

    let cometbls_latest_height = cometbls
        .update_counterparty_client(
            ethereum,
            ethereum_client_id.clone(),
            cometbls_update_from,
            cometbls_update_to,
        )
        .await;

    tracing::info!(
        chain_id = cometbls_id,
        connection_id = cometbls_connection_id,
        latest_height = ?cometbls_latest_height
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
            counterparty: connection::counterparty::Counterparty {
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
            cometbls,
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
            version: Version {
                identifier: "1".into(),
                features: [Order::Unordered, Order::Ordered].into_iter().collect(),
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
            ethereum,
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

    (
        cometbls_client_id,
        ethereum_client_id,
        cometbls_connection_id,
        ethereum_connection_id,
    )
}

async fn channel_handshake<Chain1, Chain2>(
    cometbls: &Chain1,
    ethereum: &Chain2,
    cometbls_connection_info: ConnectionEndInfo,
    ethereum_connection_info: ConnectionEndInfo,
    cometbls_port_id: String,
    ethereum_port_id: String,
) -> (String, String)
where
    Chain1: LightClient + Connect<Chain2>,
    Chain2: LightClient + Connect<Chain1>,
    <Chain1 as LightClient>::ClientState: std::fmt::Debug + ClientState,
    <Chain2 as LightClient>::ClientState: std::fmt::Debug + ClientState,
{
    let cometbls_id = cometbls.chain_id().await;
    let ethereum_id = ethereum.chain_id().await;

    tracing::info!(cometbls_id, ethereum_id);

    let cometbls_channel_id = cometbls
        .channel_open_init(MsgChannelOpenInit {
            port_id: cometbls_port_id.to_string(),
            channel: Channel {
                state: channel::state::State::Init,
                ordering: Order::Unordered,
                counterparty: channel::counterparty::Counterparty {
                    port_id: ethereum_port_id.to_string(),
                    // TODO(benluelo): Make a struct without this field?
                    channel_id: String::new(),
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
            ethereum,
            ethereum_connection_info.client_id.clone(),
            ethereum_latest_trusted_height,
            cometbls_latest_height,
        )
        .await;

    let proof = cometbls
        .channel_state_proof(
            cometbls_channel_id.clone(),
            cometbls_port_id.to_string(),
            cometbls_latest_height,
        )
        .await;

    let ethereum_channel_id = ethereum
        .channel_open_try(MsgChannelOpenTry {
            port_id: ethereum_port_id.clone(),
            channel: Channel {
                state: channel::state::State::Tryopen,
                ordering: Order::Unordered,
                counterparty: channel::counterparty::Counterparty {
                    port_id: cometbls_port_id.clone(),
                    channel_id: cometbls_channel_id.clone(),
                },
                connection_hops: vec![ethereum_connection_info.connection_id.clone()],
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
            cometbls,
            cometbls_connection_info.client_id.clone(),
            cometbls_latest_trusted_height,
            ethereum_latest_height,
        )
        .await;

    let proof = ethereum
        .channel_state_proof(
            ethereum_channel_id.clone(),
            ethereum_port_id.clone(),
            ethereum_latest_height,
        )
        .await;

    cometbls
        .channel_open_ack(MsgChannelOpenAck {
            port_id: cometbls_port_id.clone(),
            channel_id: cometbls_channel_id.clone(),
            counterparty_channel_id: ethereum_channel_id.clone(),
            counterparty_version: CHANNEL_VERSION.to_string(),
            proof_try: proof.proof,
            proof_height: proof.proof_height,
        })
        .await;

    let ethereum_latest_trusted_height = ethereum
        .query_client_state(ethereum_connection_info.client_id.clone())
        .await
        .height();

    let cometbls_latest_height = cometbls.query_latest_height().await;

    let cometbls_latest_height = cometbls
        .update_counterparty_client(
            ethereum,
            ethereum_connection_info.client_id.clone(),
            ethereum_latest_trusted_height,
            cometbls_latest_height,
        )
        .await;

    let proof = cometbls
        .channel_state_proof(
            cometbls_channel_id.clone(),
            cometbls_port_id.clone(),
            cometbls_latest_height,
        )
        .await;

    ethereum
        .channel_open_confirm(MsgChannelOpenConfirm {
            port_id: ethereum_port_id.clone(),
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

    (cometbls_channel_id, ethereum_channel_id)
}

async fn relay_packets(cometbls: Cometbls, ethereum: Ethereum) {
    let listen_handle = tokio::spawn(async move {
        loop {
            let mut subs = ethereum
                .tm_client
                .subscribe(EventType::Tx.into())
                .await
                .unwrap();

            while let Some(res) = subs.next().await {
                let ev = res.unwrap();

                tracing::info!(event = ?ev.events, "new event");

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
                        let send_packet_event = tx_result.result.events.into_iter().find_map(|e| {
                            (e.kind == "send_packet").then(|| {
                                e.attributes
                                    .into_iter()
                                    .map(|attr| (attr.key, attr.value))
                                    .collect::<HashMap<_, _>>()
                            })
                        });

                        let Some(send_packet_event) = send_packet_event else {
                            continue;
                        };

                        tracing::info!(?send_packet_event);

                        let sequence = send_packet_event["packet_sequence"].parse().unwrap();

                        let packet_commitment =
                            channel_v1::query_client::QueryClient::connect("http://0.0.0.0:9090")
                                .await
                                .unwrap()
                                .packet_commitment(channel_v1::QueryPacketCommitmentRequest {
                                    port_id: send_packet_event["packet_src_port"].clone(),
                                    channel_id: send_packet_event["packet_src_channel"].clone(),
                                    sequence,
                                })
                                .await
                                .unwrap()
                                .into_inner();

                        // NOTE: `packet_data` is deprecated and invalid!!! this assertion will fail!
                        // assert_eq!(
                        //     send_packet_event["packet_data"].clone().into_bytes(),
                        //     ethers::utils::hex::decode(&send_packet_event["packet_data_hex"])
                        //         .unwrap()
                        // );

                        let rcp = cometbls
                            .recv_packet(MsgRecvPacket {
                                packet: Packet {
                                    sequence,
                                    source_port: send_packet_event["packet_src_port"].clone(),
                                    source_channel: send_packet_event["packet_src_channel"].clone(),
                                    destination_port: send_packet_event["packet_dst_port"].clone(),
                                    destination_channel: send_packet_event["packet_dst_channel"]
                                        .clone(),
                                    data: ethers::utils::hex::decode(
                                        &send_packet_event["packet_data_hex"],
                                    )
                                    .unwrap(),
                                    timeout_height: {
                                        let (revision, height) = send_packet_event
                                            ["packet_timeout_height"]
                                            .split_once('-')
                                            .unwrap();

                                        Height {
                                            revision_number: revision.parse().unwrap(),
                                            revision_height: height.parse().unwrap(),
                                        }
                                    },
                                    timeout_timestamp: send_packet_event
                                        ["packet_timeout_timestamp"]
                                        .parse()
                                        .unwrap(),
                                },
                                proof_height: packet_commitment.proof_height.unwrap().into(),
                                proof_commitment: packet_commitment.commitment,
                            })
                            .await;

                        dbg!(rcp);
                    }
                    EventData::GenericJsonEvent(_) => todo!(),
                };
            }
        }
    });

    // let send_handle = tokio::spawn(async move {
    //     tokio::time::sleep(Duration::from_secs(20)).await;

    //     let msg = transfer_v1::MsgTransfer {
    //         source_port: PORT_ID.to_string(),
    //         source_channel: "channel-0".to_string(),
    //         token: Some(Coin {
    //             denom: "stake".to_string(),
    //             amount: "1".to_string(),
    //         }),
    //         sender: signer_from_pk(&get_wallet().public_key().public_key().to_bytes().to_vec()),
    //         receiver: "union1nrv37pqfcqul73v7d2e8y0jhjyeuhg57m3eqdt".to_string(),
    //         timeout_height: Some(client_v1::Height {
    //             revision_number: 1,
    //             revision_height: 12_345_678_765,
    //         }),
    //         timeout_timestamp: Default::default(),
    //         memo: Default::default(),
    //     };

    //     broadcast_tx_commit(
    //         [Any {
    //             type_url: "/ibc.applications.transfer.v1.MsgTransfer".to_string(),
    //             value: msg.encode_to_vec(),
    //         }]
    //         .to_vec(),
    //     )
    //     .await;
    // });

    // let (listen, send) = tokio::join!(listen_handle, send_handle);

    listen_handle.await.unwrap();

    // listen.unwrap();
    // send.unwrap();
}

// trait Msg<Client, Counterparty>: Sized
// where
//     Client: LightClient + Connect<Counterparty>,
//     Counterparty: LightClient + Connect<Client>,
// {
//     type Response;

//     type Previous: Msg<Counterparty, Client>;

//     type RequiredProofs;

//     fn construct(
//         previous_response: <Self::Previous as Msg<Counterparty, Client>>::Response,
//         proofs: Self::RequiredProofs,
//     ) -> Self;
// }

// impl<Client, Counterparty> Msg<Client, Counterparty> for MsgConnectionOpenInit
// where
//     Client: LightClient + Connect<Counterparty>,
//     Counterparty: LightClient + Connect<Client>,
// {
//     type Response = String;

//     type Previous = ();

//     type RequiredProofs = ();

//     fn construct(
//         previous_response: <Self::Previous as Msg<Counterparty, Client>>::Response,
//         proofs: Self::RequiredProofs,
//     ) -> Self {
//         MsgConnectionOpenInit {
//             client_id: todo!(),
//             counterparty: todo!(),
//             version: todo!(),
//             delay_period: todo!(),
//         }
//     }
// }

// impl<Client, Counterparty> Msg<Client, Counterparty> for ()
// where
//     Client: LightClient + Connect<Counterparty>,
//     Counterparty: LightClient + Connect<Client>,
// {
//     type Response = ();
//     type Previous = ();
//     type RequiredProofs = ();

//     fn construct(
//         previous_response: <Self::Previous as Msg<Counterparty, Client>>::Response,
//         proofs: Self::RequiredProofs,
//     ) -> Self {
//     }
// }

// 2023-06-17T20:49:01.879223Z  INFO relayer: channel opened cometbls_connection_info.connection_id="connection-0" cometbls_connection_info.client_id="cometbls-0" cometbls_channel_id="channel-0" ethereum_connection_info.connection_id="connection-8" ethereum_connection_info.client_id="08-wasm-8" ethereum_channel_id="channel-0"

// {"transfer": {"channel": "channel-0", "remote_address": "union1nrv37pqfcqul73v7d2e8y0jhjyeuhg57m3eqdt"}}
// Deploying IBCClient...
// IBCClient => 0x3fD5289eD1dC27A857A4CdEdec9Bf2c96D6C1EB3
// Deploying IBCConnection...
// IBCConnection => 0xf9FE9712A91fb3da09852a544F5A344E4EF333Aa
// Deploying IBCChannelHandshake...
// IBCChannelHandshake => 0x57b85f23f022d88b61515bF91bFE5238fCedbBD6
// Deploying IBCPacket...
// IBCPacket => 0x6744135DAA742c32e3c15619173095fC7E51dda3
// Deploying OwnableIBCHandler...
// OwnableIBCHandler => 0x7e37dA319C3008374379c7755a6C9A7CE65e1517
// Deploying TestnetVerifier...
// TestnetVerifier => 0x2C045082c6cA9a17031DA0Ca28b9Ca2617a2338A
// Deploying CometblsClient...
// CometblsClient => 0x176298b5aabE45Efa36b04452d4BB5b7bB615f1C
// Deploying ICS20Bank...
// ICS20Bank => 0xa8Ab1e8afDa14A4d0538520057EA0f9515C7D610
// Deploying ICS20TransferBank...
// ICS20TransferBank => 0x7A1d42bAe222eF8E7ED935ac5Ee5Fc6d89fB63bC
