use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use chain_utils::{
    evm::{Evm, EvmInitError},
    union::{Union, UnionInitError},
    Chain,
};
use futures::Future;
use serde::{Deserialize, Serialize};
use unionlabs::{
    ethereum_consts_traits::{Mainnet, Minimal},
    ibc::core::client::height::{HeightFromStrError, IsHeight},
    id, traits,
};

use crate::{
    chain::proof::{IbcStateRead, IbcStateReadPaths, StateProof},
    config::{self, ChainConfig, EvmChainConfig},
    msg::{
        aggregate::LightClientSpecificAggregate,
        data::LightClientSpecificData,
        fetch::{FetchUpdateHeaders, LightClientSpecificFetch},
        msg::Msg,
        DoAggregate, RelayerMsg,
    },
    queue::Queue,
};

pub mod evm;
pub mod union;

pub mod proof;

pub enum AnyChain {
    Union(Union),
    EvmMainnet(Evm<Mainnet>),
    EvmMinimal(Evm<Minimal>),
}

#[derive(Debug, thiserror::Error)]
pub enum AnyChainTryFromConfigError {
    #[error("error initializing a union chain")]
    Union(#[from] UnionInitError),
    #[error("error initializing an ethereum chain")]
    Evm(#[from] EvmInitError),
}

impl AnyChain {
    pub async fn try_from_config<Q: Queue>(
        voyager_config: &config::VoyagerConfig<Q>,
        config: ChainConfig,
    ) -> Result<Self, AnyChainTryFromConfigError> {
        Ok(match config {
            ChainConfig::Evm(EvmChainConfig::Mainnet(evm)) => Self::EvmMainnet(
                Evm::<Mainnet>::new(chain_utils::evm::Config {
                    ibc_handler_address: evm.ibc_handler_address,
                    signers: evm.signers,
                    eth_rpc_api: evm.eth_rpc_api,
                    eth_beacon_rpc_api: evm.eth_beacon_rpc_api,
                    hasura_config: voyager_config.hasura.clone(),
                })
                .await?,
            ),
            ChainConfig::Evm(EvmChainConfig::Minimal(evm)) => Self::EvmMinimal(
                Evm::<Minimal>::new(chain_utils::evm::Config {
                    ibc_handler_address: evm.ibc_handler_address,
                    signers: evm.signers,
                    eth_rpc_api: evm.eth_rpc_api,
                    eth_beacon_rpc_api: evm.eth_beacon_rpc_api,
                    hasura_config: voyager_config.hasura.clone(),
                })
                .await?,
            ),
            ChainConfig::Union(union) => Self::Union(
                Union::new(chain_utils::union::Config {
                    signers: union.signers,
                    ws_url: union.ws_url,
                    prover_endpoint: union.prover_endpoint,
                    grpc_url: union.grpc_url,
                    fee_denom: union.fee_denom,
                })
                .await?,
            ),
        })
    }
}

/// The IBC interface on a [`Chain`] that knows how to connect to a counterparty.
pub trait LightClient: Send + Sync + Sized {
    /// The chain that this light client is on.
    type HostChain: Chain + IbcStateReadPaths<<Self::Counterparty as LightClient>::HostChain>;
    type Counterparty: LightClient<Counterparty = Self>;

    type ClientId: traits::Id
        + TryFrom<<Self::HostChain as Chain>::ClientId>
        + Into<<Self::HostChain as Chain>::ClientId>;
    type ClientType: id::IdType
        + TryFrom<<Self::HostChain as Chain>::ClientType>
        + Into<<Self::HostChain as Chain>::ClientType>;

    /// The config required to construct this light client.
    type Config: Debug + Clone + PartialEq + Serialize + for<'de> Deserialize<'de>;

    type Data: Debug
        + Clone
        + PartialEq
        + Serialize
        + for<'de> Deserialize<'de>
        + Into<LightClientSpecificData<Self>>;
    type Fetch: Debug
        + Clone
        + PartialEq
        + Serialize
        + for<'de> Deserialize<'de>
        + Into<LightClientSpecificFetch<Self>>;
    type Aggregate: Debug
        + Clone
        + PartialEq
        + Serialize
        + for<'de> Deserialize<'de>
        + Into<LightClientSpecificAggregate<Self>>
        + DoAggregate<Self>;

    fn msg(&self, msg: Msg<Self>) -> impl Future + '_;

    /// Get the underlying [`Self::HostChain`] that this client is on.
    fn chain(&self) -> &Self::HostChain;

    fn from_chain(chain: Self::HostChain) -> Self;

    // TODO: Use state_proof instead
    fn query_client_state(
        &self,
        // TODO: Make this Into<_>
        client_id: <Self::HostChain as Chain>::ClientId,
        height: HeightOf<Self::HostChain>,
    ) -> impl Future<Output = ClientStateOf<<Self::Counterparty as LightClient>::HostChain>> + '_;

    fn do_fetch(&self, msg: Self::Fetch) -> impl Future<Output = Vec<RelayerMsg>> + '_;

    // Should (eventually) resolve to UpdateClientData
    fn generate_counterparty_updates(
        &self,
        update_info: FetchUpdateHeaders<Self>,
    ) -> Vec<RelayerMsg>;
}

pub type ClientStateOf<C> = <C as Chain>::SelfClientState;
pub type ConsensusStateOf<C> = <C as Chain>::SelfConsensusState;
pub type HeaderOf<C> = <C as Chain>::Header;
pub type HeightOf<C> = <C as Chain>::Height;
pub type ChainOf<L> = <L as LightClient>::HostChain;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    try_from = "&str",
    into = "String",
    bound(serialize = "", deserialize = "")
)]
pub enum QueryHeight<H: IsHeight> {
    Latest,
    Specific(H),
}

impl<H: IsHeight> Display for QueryHeight<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryHeight::Latest => f.write_str("latest"),
            QueryHeight::Specific(height) => f.write_fmt(format_args!("{height}")),
        }
    }
}

impl<H: IsHeight> From<QueryHeight<H>> for String {
    fn from(val: QueryHeight<H>) -> Self {
        val.to_string()
    }
}

impl<H: IsHeight> FromStr for QueryHeight<H> {
    type Err = HeightFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "latest" => Ok(Self::Latest),
            _ => s.parse().map(Self::Specific),
        }
    }
}

impl<H: IsHeight> TryFrom<&'_ str> for QueryHeight<H> {
    type Error = HeightFromStrError;

    fn try_from(value: &'_ str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

macro_rules! try_from_relayer_msg {
    (
        #[
            $Lc:ident($(
                lc_msg(
                    msg = $LcMsg:ident($Specific:ident),
                    ty = $Msg:ident,
                    variants($( $Var:ident($Ty:ty), )+),
                ),
            )+)
        ]
    ) => {
        $(
            $(
                impl TryFrom<RelayerMsg> for Identified<$Lc, $Ty> {
                    type Error = RelayerMsg;
                    fn try_from(value: RelayerMsg) -> Result<Identified<$Lc, $Ty>, RelayerMsg> {
                        match value {
                            RelayerMsg::Lc(AnyLcMsg::$Lc(LcMsg::$LcMsg(Identified {
                                chain_id,
                                data:
                                    $LcMsg::LightClientSpecific($Specific($Msg::$Var(
                                        data,
                                    ))),
                            }))) => Ok(Identified { chain_id, data }),
                            _ => Err(value),
                        }
                    }
                }
            )+

            crate::chain::this_is_a_hack_look_away! {
                $Lc(
                    lc_msg(
                        msg = $LcMsg($Specific),
                        ty = $Msg,
                        variants($( $Var($Ty), )+),
                    ),
                )
            }

            impl From<<$Lc as LightClient>::$LcMsg> for $Specific<$Lc> {
                fn from(msg: <$Lc as LightClient>::$LcMsg) -> Self {
                    Self(msg)
                }
            }
        )+
    };
}

macro_rules! this_is_a_hack_look_away {
    (
            $Lc:ident(
                lc_msg(
                    msg = Data(LightClientSpecificData),
                    ty = $Msg:ident,
                    variants($( $Var:ident($Ty:ty), )+),
                ),
            )
    ) => {
        $(
            impl From<Identified<$Lc, $Ty>> for AggregateData {
                fn from(Identified { chain_id, data }: Identified<$Lc, $Ty>) -> AggregateData {
                    AggregateData::$Lc(Identified {
                        chain_id,
                        data: Data::LightClientSpecific(LightClientSpecificData($Msg::$Var(
                            data,
                        ))),
                    })
                }
            }
        )+
    };

    ($($_:tt)*) => {};
}

pub(crate) use this_is_a_hack_look_away;
pub(crate) use try_from_relayer_msg;
