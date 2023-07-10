// *almost* stable, more than safe enough to use imo https://github.com/rust-lang/rfcs/pull/3425
#![feature(return_position_impl_trait_in_trait)]
// #![warn(clippy::pedantic)]
#![allow(clippy::manual_async_fn)]

// nix run .# -- tx wasm instantiate 1 '{"default_timeout":10000,"gov_contract":"union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2","allowlist":[]}' --label blah --from alice --gas auto --keyring-backend test --gas-adjustment 1.3 --amount 100stake --no-admin --chain-id union-devnet-1

use bip32::{DerivationPath, Language, XPrv};
use clap::{Args, Parser, Subcommand};
use contracts::{ics20_bank::ics20_bank, ics20_transfer_bank::ics20_transfer_bank};
use ethers::{
    prelude::{EthAbiCodec, EthAbiType, SignerMiddleware},
    providers::{Http, Middleware, Provider, Ws},
    signers::{LocalWallet, Signer},
    types::{Address, H256, U256},
};
use futures::{future::join, Stream, StreamExt};
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
use reqwest::Url;
use std::{str::FromStr, sync::Arc};

use crate::chain::{
    cosmos::Ethereum,
    evm::{Cometbls, CometblsConfig},
    ClientState, Connect, LightClient,
};

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
    QueryBalances {
        #[arg(long)]
        wallet: LocalWallet,
        #[arg(long)]
        denom: String,
        #[arg(long)]
        eth_rpc_api: Url,
        #[arg(long)]
        ics20_bank_address: Address,
    },
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

            let (cometbls_client_id, ethereum_client_id) = if open_channel {
                tracing::info!("opening_channel");

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
                        client_id: cometbls_client_id.clone(),
                        connection_id: cometbls_connection_id,
                    },
                    ConnectionEndInfo {
                        client_id: ethereum_client_id.clone(),
                        connection_id: ethereum_connection_id,
                    },
                    cometbls_port_id.unwrap(),
                    ethereum_port_id.unwrap(),
                )
                .await;

                (cometbls_client_id, ethereum_client_id)
            } else {
                (cometbls_client_id, ethereum_client_id)
            };

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
            let provider = Provider::new(Ws::connect(eth_rpc_api).await.unwrap());

            let chain_id = provider.get_chainid().await.unwrap();

            tracing::info!(chain_id = chain_id.to_string());

            let wallet = wallet.with_chain_id(chain_id.as_u64());

            let signer_middleware = Arc::new(SignerMiddleware::new(provider.clone(), wallet));

            let ics20_transfer_bank = ics20_transfer_bank::ICS20TransferBank::new(
                ics20_transfer_address,
                signer_middleware.clone(),
            );

            let ics20_bank =
                ics20_bank::ICS20Bank::new(ics20_bank_address, signer_middleware.clone());

            // ics20_bank
            //     .grant_role([0; 32], signer_middleware.address())
            //     .send()
            //     .await
            //     .unwrap()
            //     .await
            //     .unwrap()
            //     .unwrap();

            // let has_role = ics20_bank
            //     .has_role(
            //         ics20_bank.admin_role().await.unwrap(),
            //         signer_middleware.address(),
            //     )
            //     .await
            //     .unwrap();

            // tracing::info!(has_role);

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

            tracing::info!(balance_before = %balance, %denom);

            ics20_transfer_bank
                .send_transfer(
                    denom.clone(),
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

            let balance: U256 = ics20_bank
                .balance_of(signer_middleware.address(), denom.clone())
                .await
                .unwrap();

            tracing::info!(balance_after = %balance, %denom);
        }
        Command::QueryBalances {
            wallet,
            denom,
            eth_rpc_api,
            ics20_bank_address,
        } => {
            let provider = Provider::new(Http::new(eth_rpc_api));
            let wallet = wallet.with_chain_id(provider.get_chainid().await.unwrap().as_u64());
            let signer_middleware = Arc::new(SignerMiddleware::new(provider, wallet.clone()));

            let ics20_bank =
                ics20_bank::ICS20Bank::new(ics20_bank_address, signer_middleware.clone());

            let stripped_denom = denom.split('/').last().unwrap().to_string();

            let balance = ics20_bank
                .balance_of(wallet.address(), denom.clone())
                .await
                .unwrap();

            println!("0x{:x}: {balance}{stripped_denom}", wallet.address());
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
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
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
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
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

async fn relay_packets<C: ChainSpec>(
    cometbls: Cometbls<C>,
    ethereum: Ethereum<C>,
    cometbls_client_id: String,
    ethereum_client_id: String,
) {
    let cometbls_packet_stream = cometbls.packet_stream().await;
    let ethereum_packet_stream = ethereum.packet_stream().await;

    join(
        relay_packets_inner(
            &ethereum,
            ethereum_packet_stream,
            &cometbls,
            cometbls_client_id,
        ),
        relay_packets_inner(
            &cometbls,
            cometbls_packet_stream,
            &ethereum,
            ethereum_client_id,
        ),
    )
    .await;
}

async fn relay_packets_inner<L1, L2>(
    lc1: &L2,
    lc1_event_stream: impl Stream<Item = (Height, Packet)>,
    lc2: &L1,
    lc2_event_stream: String,
) where
    L1: LightClient + Connect<L2>,
    L2: LightClient + Connect<L1>,
{
    lc1_event_stream
        .for_each(move |(event_height, packet)| {
            let lc2_event_stream = lc2_event_stream.clone();
            async move {
                let sequence = packet.sequence;

                let latest_height = lc2
                    .query_client_state(lc2_event_stream.clone())
                    .await
                    .height();

                dbg!(&latest_height);

                let lc1_update_to = loop {
                    let height = lc1.query_latest_height().await;
                    if height >= event_height.increment() {
                        break event_height.increment();
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                };

                // REVIEW: Should we use the returned height?
                // let ethereum_update_to = lc1
                let lc1_update_to = lc1
                    .update_counterparty_client(
                        lc2,
                        lc2_event_stream.clone(),
                        latest_height,
                        lc1_update_to,
                    )
                    .await;

                dbg!(&lc1_update_to);

                let commitment_proof = lc1
                    .packet_commitment_proof(
                        packet.source_port.clone(),
                        packet.source_channel.clone(),
                        sequence,
                        event_height,
                    )
                    .await;

                let rcp = lc2
                    .recv_packet(MsgRecvPacket {
                        packet,
                        proof_height: lc1_update_to,
                        proof_commitment: commitment_proof.proof,
                    })
                    .await;

                dbg!(rcp);
            }
        })
        .await
}
