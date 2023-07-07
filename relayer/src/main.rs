// *almost* stable, more than safe enough to use imo https://github.com/rust-lang/rfcs/pull/3425
#![feature(return_position_impl_trait_in_trait)]
// #![warn(clippy::pedantic)]
#![allow(clippy::manual_async_fn)]

// nix run .# -- tx wasm instantiate 1 '{"default_timeout":10000,"gov_contract":"union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2","allowlist":[]}' --label blah --from alice --gas auto --keyring-backend test --gas-adjustment 1.3 --amount 100stake --no-admin --chain-id union-devnet-1

use bip32::{DerivationPath, Language, XPrv};
use chain::evm::CometblsConfig;
use clap::{Args, Parser, Subcommand};
use contracts::{ics20_bank::ics20_bank, ics20_transfer_bank::ics20_transfer_bank};
use ethers::{
    prelude::{EthAbiCodec, EthAbiType, SignerMiddleware},
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, H256},
};
use futures::StreamExt;
use ibc_types::{
    ethereum_consts_traits::{ChainSpec, Minimal},
    ibc::core::{
        channel::{
            self, channel::Channel, msg_channel_open_ack::MsgChannelOpenAck,
            msg_channel_open_confirm::MsgChannelOpenConfirm,
            msg_channel_open_init::MsgChannelOpenInit, msg_channel_open_try::MsgChannelOpenTry,
            msg_recv_packet::MsgRecvPacket, order::Order, packet::Packet,
        },
        client::height::Height,
        commitment::merkle_prefix::MerklePrefix,
        connection::{
            self, msg_channel_open_ack::MsgConnectionOpenAck,
            msg_channel_open_confirm::MsgConnectionOpenConfirm,
            msg_channel_open_init::MsgConnectionOpenInit,
            msg_channel_open_try::MsgConnectionOpenTry, version::Version,
        },
    },
    IntoProto,
};
use prost::Message;
use protos::ibc::core::channel::v1 as channel_v1;
use reqwest::Url;
use std::{collections::HashMap, str::FromStr, sync::Arc};
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
    Transfer(TransferArgs),
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
    cometbls_client_id: String,

    #[arg(long)]
    ethereum_client_id: String,

    #[arg(long)]
    cometbls_port_id: Option<String>,
    #[arg(long)]
    ethereum_port_id: Option<String>,
}

#[derive(Debug, Parser)]
pub struct TransferArgs {
    #[arg(long)]
    ics20_transfer_address: Address,
    #[arg(long)]
    ics20_bank_address: Address,
    #[arg(long)]
    denom: String,
    #[arg(long)]
    amount: u64,
    #[arg(long)]
    receiver: String,
    #[arg(long)]
    wallet: LocalWallet,
    #[arg(long)]
    source_port: String,
    #[arg(long)]
    source_channel: String,
    #[arg(long)]
    eth_rpc_api: Url,
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

    do_main::<Minimal>(args).await;
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

async fn do_main<C: ChainSpec>(args: AppArgs) {
    match args.command {
        Command::OpenConnection(OpenConnectionArgs { args }) => {
            let cometbls = Cometbls::<C>::new(CometblsConfig {
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
            let cometbls_lc = Cometbls::<C>::new(CometblsConfig {
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

            let ethereum_lc = Ethereum::<C>::new(get_wallet(), args.ethereum.wasm_code_id).await;

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
            cometbls_client_id,
            ethereum_client_id,
        }) => {
            let cometbls_lc = Cometbls::<C>::new(CometblsConfig {
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

            let ethereum_lc = Ethereum::<C>::new(get_wallet(), args.ethereum.wasm_code_id).await;

            // ICS20Transfer.sol -> onRecvPacket -> replace body with `return _newAcknowledgement(true);`

            // let balance = cometbls_lc
            //     .ics20_bank
            //     .balance_of(
            //         ethers::types::H160::from(b"aaaaa55555aaaaa44444"),
            //         format!("{}/channel-11/stake", cometbls_port_id.clone().unwrap()),
            //     )
            //     .await
            //     .unwrap();
            // dbg!(balance);
            // panic!();

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

            relay_packets(
                cometbls_lc,
                ethereum_lc,
                cometbls_client_id,
                ethereum_client_id,
            )
            .await;
        }
        Command::Transfer(TransferArgs {
            ics20_transfer_address,
            ics20_bank_address,
            denom,
            amount,
            receiver,
            wallet,
            source_port,
            source_channel,
            eth_rpc_api,
        }) => {
            let provider = Provider::new(Http::new(eth_rpc_api));

            let chain_id = provider.get_chainid().await.unwrap();

            let wallet = wallet.with_chain_id(chain_id.as_u64());

            let signer_middleware = Arc::new(SignerMiddleware::new(provider.clone(), wallet));

            let ics20_transfer_bank = ics20_transfer_bank::ICS20TransferBank::new(
                ics20_transfer_address,
                signer_middleware.clone(),
            );

            let ics20_bank =
                ics20_bank::ICS20Bank::new(ics20_bank_address, signer_middleware.clone());

            tracing::info!("Setting the account as operator");

            ics20_bank
                .set_operator(ics20_transfer_address)
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();

            // tracing::info!("Minting 1000000000 tokens");

            // ics20_bank
            //     .mint(
            //         signer_middleware.address(),
            //         denom.clone(),
            //         1000000000.into(),
            //     )
            //     .send()
            //     .await
            //     .unwrap()
            //     .await
            //     .unwrap()
            //     .unwrap();

            let balance = ics20_bank
                .balance_of(signer_middleware.address(), denom.clone())
                .await
                .unwrap();

            tracing::info!(balance = balance.as_u64());

            ics20_transfer_bank
                .send_transfer(
                    denom,
                    amount,
                    receiver,
                    source_port,
                    source_channel,
                    1,
                    1000000000,
                )
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();
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
    <Chain1 as LightClient>::ClientState: std::fmt::Debug + IntoProto,
    <Chain2 as LightClient>::ClientState: std::fmt::Debug + IntoProto,
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

    let delay_period = 6;

    let (cometbls_connection_id, _) = cometbls
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
                features: [Order::Ordered, Order::Unordered].into_iter().collect(),
            },
            delay_period,
        })
        .await;

    tracing::info!(
        cometbls_connection_id,
        ?cometbls_latest_height,
        ?ethereum_latest_height,
        cometbls_client_id,
        ethereum_client_id,
        "right after connection init"
    );

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
        latest_height = ?cometbls_latest_height,
        "right after updating cosmos"
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

    let (ethereum_connection_id, connection_try_height) = ethereum
        .connection_open_try(MsgConnectionOpenTry {
            client_id: ethereum_client_id.clone(),
            counterparty: connection::counterparty::Counterparty {
                client_id: cometbls_client_id.clone(),
                connection_id: cometbls_connection_id.clone(),
                prefix: MerklePrefix {
                    key_prefix: b"ibc".to_vec(),
                },
            },
            delay_period,
            client_state: cometbls_client_state_proof.state,
            counterparty_versions: cometbls_connection_state_proof.state.versions,
            proof_height: cometbls_consensus_state_proof.proof_height,
            proof_init: cometbls_connection_state_proof.proof,
            proof_client: cometbls_client_state_proof.proof,
            proof_consensus: cometbls_consensus_state_proof.proof,
            consensus_height: ethereum_latest_height,
        })
        .await;

    tracing::info!(
        "Connection open try executed at {:?}",
        connection_try_height
    );

    let ethereum_update_from = ethereum_latest_height;
    let ethereum_update_to = loop {
        let height = ethereum.query_latest_height().await;
        if height >= connection_try_height.increment() {
            break connection_try_height.increment();
        }
    };

    tracing::info!("Querying proof at {:?}", connection_try_height);

    let _ = ethereum
        .update_counterparty_client(
            cometbls,
            cometbls_client_id.clone(),
            ethereum_update_from,
            ethereum_update_to,
        )
        .await;

    let ethereum_connection_state_proof = ethereum
        .connection_state_proof(ethereum_connection_id.clone(), connection_try_height)
        .await;
    let ethereum_client_state_proof = ethereum
        .client_state_proof(ethereum_client_id.clone(), connection_try_height)
        .await;
    let ethereum_consensus_state_proof = ethereum
        .consensus_state_proof(
            ethereum_client_id.clone(),
            cometbls
                .process_height_for_counterparty(cometbls_latest_height)
                .await,
            connection_try_height,
        )
        .await;

    let cl = cometbls
        .query_client_state(cometbls_client_id.clone())
        .await;

    tracing::debug!(
        "Cometbls client state {:?}",
        ethers::utils::hex::encode(cl.into_proto().encode_to_vec())
    );

    let cl = ethereum
        .query_client_state(ethereum_client_id.clone())
        .await;

    tracing::debug!(
        "Evm client state {:?}",
        ethers::utils::hex::encode(cl.into_proto().encode_to_vec())
    );

    tracing::debug!(
        "Proof Connection {:?}",
        ethers::utils::hex::encode(&ethereum_connection_state_proof.proof)
    );
    tracing::debug!(
        "Proof Client {:?}",
        ethers::utils::hex::encode(&ethereum_client_state_proof.proof)
    );
    tracing::debug!(
        "Proof Consensus {:?}",
        ethers::utils::hex::encode(&ethereum_consensus_state_proof.proof)
    );

    cometbls
        .connection_open_ack(MsgConnectionOpenAck {
            connection_id: cometbls_connection_id.clone(),
            counterparty_connection_id: ethereum_connection_id.clone(),
            version: Version {
                identifier: "1".into(),
                features: [Order::Ordered, Order::Unordered].into_iter().collect(),
            },
            client_state: ethereum_client_state_proof.state,
            proof_height: ethereum_update_to,
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

    tracing::debug!("ChannelOpenInit");

    let (cometbls_channel_id, _) = cometbls
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

    tracing::debug!("ChannelOpenTry");

    let (ethereum_channel_id, channel_try_height) = ethereum
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
    let ethereum_update_to = loop {
        let height = ethereum.query_latest_height().await;
        if height >= channel_try_height.increment() {
            break channel_try_height.increment();
        }
    };

    tracing::info!("Querying proof at {:?}", channel_try_height);

    let _ = ethereum
        .update_counterparty_client(
            cometbls,
            cometbls_connection_info.client_id.clone(),
            cometbls_latest_trusted_height,
            ethereum_update_to,
        )
        .await;

    let proof = ethereum
        .channel_state_proof(
            ethereum_channel_id.clone(),
            ethereum_port_id.clone(),
            channel_try_height,
        )
        .await;

    tracing::debug!("ChannelOpenAck");

    cometbls
        .channel_open_ack(MsgChannelOpenAck {
            port_id: cometbls_port_id.clone(),
            channel_id: cometbls_channel_id.clone(),
            counterparty_channel_id: ethereum_channel_id.clone(),
            counterparty_version: CHANNEL_VERSION.to_string(),
            proof_try: proof.proof,
            proof_height: ethereum_update_to,
        })
        .await;

    let update_to = cometbls.query_latest_height().await;

    let cometbls_latest_height = cometbls
        .update_counterparty_client(
            ethereum,
            ethereum_connection_info.client_id.clone(),
            cometbls_latest_height,
            update_to,
        )
        .await;

    let proof = cometbls
        .channel_state_proof(
            cometbls_channel_id.clone(),
            cometbls_port_id.clone(),
            cometbls_latest_height,
        )
        .await;

    tracing::debug!("ChannelOpenConfirm");

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

async fn relay_packets(
    cometbls: Cometbls,
    ethereum: Ethereum,
    cometbls_client_id: String,
    ethereum_client_id: String,
) {
    let cometbls = Arc::new(cometbls);
    let ethereum = Arc::new(ethereum);

    let cosmos_to_eth_handle = tokio::spawn(relay_packets_from_cosmos_to_ethereum(
        cometbls.clone(),
        ethereum.clone(),
        cometbls_client_id,
    ));
    let eth_to_cosmos_handle = tokio::spawn(relay_packets_from_ethereum_to_cosmos(
        cometbls,
        ethereum,
        ethereum_client_id,
    ));

    let (h1, h2) = tokio::join!(cosmos_to_eth_handle, eth_to_cosmos_handle);

    h1.unwrap();
    h2.unwrap();
}

async fn relay_packets_from_cosmos_to_ethereum(
    cometbls: Arc<Cometbls>,
    ethereum: Arc<Ethereum>,
    cometbls_client_id: String,
) {
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

                // NOTE: `packet_data` is deprecated and invalid!!! this assertion will fail!
                // assert_eq!(
                //     send_packet_event["packet_data"].clone().into_bytes(),
                //     ethers::utils::hex::decode(&send_packet_event["packet_data_hex"])
                //         .unwrap()
                // );

                let latest_height = cometbls
                    .query_client_state(cometbls_client_id.clone())
                    .await
                    .height();

                let event_height = Height::new(1, tx_result.height.unsigned_abs());

                let ethereum_update_to = loop {
                    let height = ethereum.query_latest_height().await;
                    if height >= event_height.increment() {
                        break event_height.increment();
                    }
                };

                let _ = ethereum
                    .update_counterparty_client(
                        &cometbls,
                        cometbls_client_id.clone(),
                        latest_height,
                        ethereum_update_to,
                    )
                    .await;

                let commitment_proof = ethereum
                    .packet_commitment_proof(
                        send_packet_event["packet_src_port"].clone(),
                        send_packet_event["packet_src_channel"].clone(),
                        sequence,
                        event_height,
                    )
                    .await;

                let rcp = cometbls
                    .recv_packet(MsgRecvPacket {
                        packet: Packet {
                            sequence,
                            source_port: send_packet_event["packet_src_port"].clone(),
                            source_channel: send_packet_event["packet_src_channel"].clone(),
                            destination_port: send_packet_event["packet_dst_port"].clone(),
                            destination_channel: send_packet_event["packet_dst_channel"].clone(),
                            data: ethers::utils::hex::decode(&send_packet_event["packet_data_hex"])
                                .unwrap(),
                            timeout_height: {
                                let (revision, height) = send_packet_event["packet_timeout_height"]
                                    .split_once('-')
                                    .unwrap();

                                Height {
                                    revision_number: revision.parse().unwrap(),
                                    revision_height: height.parse().unwrap(),
                                }
                            },
                            timeout_timestamp: send_packet_event["packet_timeout_timestamp"]
                                .parse()
                                .unwrap(),
                        },
                        proof_height: ethereum_update_to,
                        proof_commitment: commitment_proof.proof,
                    })
                    .await;

                dbg!(rcp);
            }
            EventData::GenericJsonEvent(_) => todo!(),
        };
    }
}

async fn relay_packets_from_ethereum_to_cosmos(
    cometbls: Arc<Cometbls>,
    ethereum: Arc<Ethereum>,
    ethereum_client_id: String,
) {
    let event = cometbls.ibc_handler.send_packet_filter();
    let mut event_stream = event.stream_with_meta().await.unwrap();

    while let Some(Ok((event, meta))) = event_stream.next().await {
        let event: contracts::ibc_handler::SendPacketFilter = event;

        tracing::info!(event = ?event, "new event");
        println!("EVENT DATA: {:?}", event.data.to_vec());

        cometbls
            .wait_for_execution_block(meta.block_number.as_u64().into())
            .await;

        let latest_height = ethereum
            .query_client_state(ethereum_client_id.clone())
            .await
            .height();

        let updated_height = cometbls
            .update_counterparty_client(
                &ethereum,
                ethereum_client_id.clone(),
                latest_height,
                cometbls.query_latest_height().await,
            )
            .await;

        let commitment_proof = cometbls
            .packet_commitment_proof(
                event.source_port.clone(),
                event.source_channel.clone(),
                event.sequence,
                updated_height,
            )
            .await;

        let (channel_data, _): (contracts::ibc_handler::IbcCoreChannelV1ChannelData, bool) =
            cometbls
                .ibc_handler
                .get_channel(event.source_port.clone(), event.source_channel.clone())
                .block(
                    cometbls
                        .process_height_for_counterparty(updated_height)
                        .await
                        .revision_height,
                )
                .await
                .unwrap();

        let rcp = ethereum
            .recv_packet(MsgRecvPacket {
                packet: Packet {
                    sequence: event.sequence,
                    source_port: event.source_port,
                    source_channel: event.source_channel,
                    destination_port: channel_data.counterparty.port_id,
                    destination_channel: channel_data.counterparty.channel_id,
                    data: event.data.to_vec(),
                    timeout_height: Height::new(
                        event.timeout_height.revision_number,
                        event.timeout_height.revision_height,
                    ),
                    timeout_timestamp: event.timeout_timestamp,
                },
                proof_commitment: commitment_proof.proof,
                proof_height: commitment_proof.proof_height,
            })
            .await;

        tracing::info!(rcp = ?rcp, "received packet");
    }
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
