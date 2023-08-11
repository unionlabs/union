use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use futures::{Future, Stream};
use serde::Serialize;
use unionlabs::{
    ethereum_consts_traits::{Mainnet, Minimal},
    ibc::{
        core::{
            channel::{
                msg_channel_open_ack::MsgChannelOpenAck,
                msg_channel_open_confirm::MsgChannelOpenConfirm,
                msg_channel_open_init::MsgChannelOpenInit, msg_channel_open_try::MsgChannelOpenTry,
                msg_recv_packet::MsgRecvPacket, packet::Packet,
            },
            client::height::{Height, HeightFromStrError},
            connection::{
                msg_channel_open_ack::MsgConnectionOpenAck,
                msg_channel_open_confirm::MsgConnectionOpenConfirm,
                msg_channel_open_init::MsgConnectionOpenInit,
                msg_channel_open_try::MsgConnectionOpenTry,
            },
        },
        google::protobuf::any::Any,
        lightclients::{cometbls, ethereum, wasm},
    },
};

use crate::{
    chain::{
        cosmos::{Ethereum, Union},
        evm::{Cometbls, Evm},
        proof::{IbcStateRead, IbcStateReadPaths, StateProof},
    },
    config::{ChainConfig, EvmChainConfig},
};

pub mod cosmos;
pub mod evm;

pub mod dumper;
pub mod events;
pub mod proof;

pub enum AnyChain {
    Union(Union),
    EvmMainnet(Evm<Mainnet>),
    EvmMinimal(Evm<Minimal>),
}

impl AnyChain {
    pub async fn try_from_config(config: ChainConfig) -> Self {
        match config {
            ChainConfig::Evm(EvmChainConfig::Mainnet(evm)) => {
                Self::EvmMainnet(Evm::<Mainnet>::new(evm).await)
            }
            ChainConfig::Evm(EvmChainConfig::Minimal(evm)) => {
                Self::EvmMinimal(Evm::<Minimal>::new(evm).await)
            }
            ChainConfig::Union(union) => Self::Union(Union::new(union).await),
        }
    }
}

pub enum AnyLightClient {
    UnionEthereumMainnet(Ethereum<Mainnet>),
    UnionEthereumMinimal(Ethereum<Minimal>),
    EvmCometblsMainnet(Cometbls<Mainnet>),
    EvmCometblsMinimal(Cometbls<Minimal>),
}

/// The IBC interface on a [`Chain`] that knows how to connect to a counterparty.
pub trait LightClient: Send + Sync + Sized {
    // /// The client state type that this light client stores about the counterparty.
    // type CounterpartyClientState: ClientState;

    // /// The consensus state type that this light client stores about the counterparty.
    // type CounterpartyConsensusState;

    type UpdateClientMessage;

    type IbcStateRead: IbcStateReadPaths<Self>;

    /// The chain that this light client is on.
    type HostChain: Chain;

    /// The chain that this light client is tracking.
    type CounterpartyChain: Chain;

    /// The config required to construct this light client.
    type Config;

    /// Get the underlying [`Self::HostChain`] that this client is on.
    fn chain(&self) -> &Self::HostChain;

    fn update_client(
        &self,
        client_id: String,
        _: Self::UpdateClientMessage,
    ) -> impl Future<Output = ()> + '_;

    fn state_proof<P: proof::IbcPath + 'static>(
        &self,
        path: P,
        self_height: Height,
    ) -> impl Future<Output = StateProof<P::Output<Self>>> + '_
    where
        Self::IbcStateRead: IbcStateRead<Self, P>,
    {
        async move {
            tracing::info!(%path, %self_height, chain_id = %self.chain().chain_id().await);

            let state_proof =
                <Self::IbcStateRead as IbcStateRead<Self, P>>::state_proof(self, path, self_height)
                    .await;

            tracing::info!(
                state = ?state_proof.state,
                proof = %serde_utils::to_hex(&state_proof.proof),
                proof_height = %state_proof.proof_height
            );

            state_proof
        }
    }

    // TODO: Use state_proof instead
    fn query_client_state(
        &self,
        client_id: String,
    ) -> impl Future<Output = ClientStateOf<Self::CounterpartyChain>> + '_;

    fn process_height_for_counterparty(&self, height: Height) -> impl Future<Output = Height> + '_;
}

pub type ClientStateOf<C> = <C as Chain>::SelfClientState;
pub type ConsensusStateOf<C> = <C as Chain>::SelfConsensusState;

/// Represents a block chain. One [`Chain`] may have many related [`LightClient`]s for connecting to
/// various other [`Chain`]s, all sharing a common config.
pub trait Chain {
    type SelfClientState: ClientState + Debug + Serialize;
    type SelfConsensusState: Debug + Serialize;

    fn chain_id(&self) -> impl Future<Output = String> + '_;

    fn query_latest_height(&self) -> impl Future<Output = Height> + '_;

    /// The client state on this chain at the current height.
    fn self_client_state(&self, height: Height)
        -> impl Future<Output = Self::SelfClientState> + '_;

    /// The latest consensus state for the this chain.
    fn self_consensus_state(
        &self,
        height: Height,
    ) -> impl Future<Output = Self::SelfConsensusState> + '_;

    fn packet_stream(&self)
        -> impl Future<Output = impl Stream<Item = (Height, Packet)> + '_> + '_;
}

pub trait CreateClient<L: LightClient>: Chain {
    fn create_client(
        &self,
        config: L::Config,
        counterparty_chain: L::CounterpartyChain,
    ) -> impl Future<Output = (String, L)> + '_;
}

#[derive(Debug, Clone)]
pub enum QueryHeight {
    Latest,
    Specific(Height),
}

impl Display for QueryHeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryHeight::Latest => f.write_str("latest"),
            QueryHeight::Specific(height) => f.write_fmt(format_args!("{height}")),
        }
    }
}

impl FromStr for QueryHeight {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "latest" => Ok(Self::Latest),
            _ => s
                .parse()
                .map_err(|x: HeightFromStrError| x.to_string())
                .map(Self::Specific),
        }
    }
}

pub trait Connect<L>: LightClient
where
    L: LightClient,
{
    // CONNECTION HANDSHAKE

    fn connection_open_init(
        &self,
        _: MsgConnectionOpenInit,
    ) -> impl Future<Output = (String, Height)> + '_;

    fn connection_open_try(
        &self,
        _: MsgConnectionOpenTry<ClientStateOf<L::CounterpartyChain>>,
    ) -> impl Future<Output = (String, Height)> + '_;

    fn connection_open_ack(
        &self,
        _: MsgConnectionOpenAck<ClientStateOf<L::CounterpartyChain>>,
    ) -> impl Future<Output = Height> + '_;

    fn connection_open_confirm(
        &self,
        _: MsgConnectionOpenConfirm,
    ) -> impl Future<Output = Height> + '_;

    // CHANNEL HANDSHAKE

    fn channel_open_init(
        &self,
        _: MsgChannelOpenInit,
    ) -> impl Future<Output = (String, Height)> + '_;

    fn channel_open_try(&self, _: MsgChannelOpenTry)
        -> impl Future<Output = (String, Height)> + '_;

    fn channel_open_ack(&self, _: MsgChannelOpenAck) -> impl Future<Output = Height> + '_;

    fn channel_open_confirm(&self, _: MsgChannelOpenConfirm) -> impl Future<Output = Height> + '_;

    // PACKETS

    fn recv_packet(&self, _: MsgRecvPacket) -> impl Future<Output = ()> + Send + '_;

    // OTHER STUFF

    fn update_counterparty_client<'a>(
        &'a self,
        counterparty: &'a L,
        counterparty_client_id: String,
        update_from: Height,
        update_to: Height,
    ) -> impl Future<Output = Height> + 'a;
}

pub trait ChainConnection<To: ChainConnection<Self>>:
    Chain + CreateClient<Self::LightClient> + Sized
{
    type LightClient: LightClient<HostChain = Self>
        + Connect<To::LightClient, CounterpartyChain = To>;

    fn light_client(&self) -> Self::LightClient;
}

// some hackery to work around wrapping in wasm::ClientState
//
// avert your eyes

pub trait InnerClientState {
    fn height(&self) -> Option<Height>;
}

pub trait ClientState {
    fn height(&self) -> Height;
}

impl InnerClientState for ethereum::client_state::ClientState {
    fn height(&self) -> Option<Height> {
        Some(Height {
            revision_number: 0,
            revision_height: self.latest_slot,
        })
    }
}

impl InnerClientState for cometbls::client_state::ClientState {
    fn height(&self) -> Option<Height> {
        None
    }
}

impl<Data: InnerClientState> ClientState for wasm::client_state::ClientState<Data> {
    fn height(&self) -> Height {
        self.data.height().unwrap_or(self.latest_height)
    }
}

impl<T> ClientState for Any<T>
where
    T: ClientState,
{
    fn height(&self) -> Height {
        self.0.height()
    }
}
