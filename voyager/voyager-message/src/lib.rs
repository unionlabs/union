#![feature(trait_alias)]
#![allow(clippy::type_complexity, async_fn_in_trait)]

use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    fmt::{Debug, Display},
    future::Future,
    marker::PhantomData,
};

use chain_utils::{
    cosmos::Cosmos,
    cosmos_sdk::{BroadcastTxCommitError, CosmosSdkChain, CosmosSdkChainExt},
    evm::Evm,
    union::Union,
};
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use queue_msg::{seq, QueueMsg, QueueMsgTypes};
use serde::{Deserialize, Serialize};
use unionlabs::{
    encoding::{Encode, Proto},
    ethereum::config::{Mainnet, Minimal},
    google::protobuf::any::{mk_any, Any, IntoAny},
    hash::H256,
    ibc::{core::client::height::IsHeight, lightclients::wasm},
    proof::{self},
    traits::{
        Chain, ChainIdOf, ClientIdOf, ClientState, ClientStateOf, ConsensusStateOf, HeaderOf,
        HeightOf,
    },
    IntoProto, MaybeArbitrary, MaybeRecoverableError, TypeUrl,
};

use crate::{
    aggregate::AnyAggregate,
    data::{AnyData, Data},
    event::AnyEvent,
    fetch::{AnyFetch, DoFetch, Fetch, FetchUpdateHeaders},
    msg::{
        AnyMsg, Msg, MsgConnectionOpenAckData, MsgConnectionOpenInitData, MsgConnectionOpenTryData,
        MsgUpdateClientData,
    },
    wait::{AnyWait, Wait},
};

pub mod use_aggregate;

pub mod aggregate;
pub mod data;
pub mod event;
pub mod fetch;
pub mod msg;
pub mod wait;

// TODO: Rename this module to something better, `lightclient` clashes with the workspace crate (could also rename the crate)
pub mod chain_impls;

pub trait RelayerMsgDatagram = Debug
    + Display
    + Clone
    + PartialEq
    + Serialize
    + for<'de> Deserialize<'de>
    + 'static
    + MaybeArbitrary;

pub trait ChainExt: Chain {
    type Data<Tr: ChainExt>: RelayerMsgDatagram;
    type Fetch<Tr: ChainExt>: RelayerMsgDatagram;
    type Aggregate<Tr: ChainExt>: RelayerMsgDatagram;

    /// Error type for [`Self::msg`].
    type MsgError: Debug + MaybeRecoverableError;

    /// The config required to construct this light client.
    type Config: Debug + Clone + PartialEq + Serialize + for<'de> Deserialize<'de> + MaybeArbitrary;

    fn do_fetch<Tr: ChainExt>(&self, msg: Self::Fetch<Tr>) -> impl Future<Output = RelayerMsg> + '_
    where
        Self::Fetch<Tr>: DoFetch<Self>,
    {
        DoFetch::do_fetch(self, msg)
    }
}

pub struct RelayerMsgTypes;

impl QueueMsgTypes for RelayerMsgTypes {
    type Event = AnyLightClientIdentified<AnyEvent>;
    type Data = AnyLightClientIdentified<AnyData>;
    type Fetch = AnyLightClientIdentified<AnyFetch>;
    type Msg = AnyLightClientIdentified<AnyMsg>;
    type Wait = AnyLightClientIdentified<AnyWait>;
    type Aggregate = AnyLightClientIdentified<AnyAggregate>;

    type Store = Chains;
}

pub type RelayerMsg = QueueMsg<RelayerMsgTypes>;

pub trait GetChain<Hc: ChainExt> {
    fn get_chain(&self, chain_id: &ChainIdOf<Hc>) -> Hc;
}

#[derive(Debug, Clone)]
pub struct Chains {
    // TODO: Use some sort of typemap here instead of individual fields
    pub evm_minimal: HashMap<ChainIdOf<Evm<Minimal>>, Evm<Minimal>>,
    pub evm_mainnet: HashMap<ChainIdOf<Evm<Mainnet>>, Evm<Mainnet>>,
    pub union: HashMap<ChainIdOf<Union>, Union>,
    pub cosmos: HashMap<ChainIdOf<Cosmos>, Cosmos>,
}

impl GetChain<Wasm<Union>> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Wasm<Union>>) -> Wasm<Union> {
        Wasm(self.union.get(chain_id).unwrap().clone())
    }
}

impl GetChain<Wasm<Cosmos>> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Wasm<Cosmos>>) -> Wasm<Cosmos> {
        Wasm(self.cosmos.get(chain_id).unwrap().clone())
    }
}

impl GetChain<Union> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Union>) -> Union {
        self.union.get(chain_id).unwrap().clone()
    }
}

impl GetChain<Evm<Minimal>> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Evm<Minimal>>) -> Evm<Minimal> {
        self.evm_minimal.get(chain_id).unwrap().clone()
    }
}

impl GetChain<Evm<Mainnet>> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Evm<Mainnet>>) -> Evm<Mainnet> {
        self.evm_mainnet.get(chain_id).unwrap().clone()
    }
}

// #[derive(Debug, thiserror::Error)]
// pub enum HandleMsgError {
//     #[error(transparent)]
//     Lc(#[from] AnyLightClientIdentified<AnyLcError>),
// }

// pub enum AnyLcError {}
// impl AnyLightClient for AnyLcError {
//     type Inner<Hc: ChainExt, Tr: ChainExt> = LcError<Hc, Tr>;
// }

// pub enum AnyLcError {
//     #[error(transparent)]
//     EthereumMainnet(identified!(LcError<Wasm<Union>, Evm<Mainnet>>)),
//     #[error(transparent)]
//     CometblsMainnet(identified!(LcError<Evm<Mainnet>, Wasm<Union>>)),
//     #[error(transparent)]
//     EthereumMinimal(identified!(LcError<Wasm<Union>, Evm<Minimal>>)),
//     #[error(transparent)]
//     CometblsMinimal(identified!(LcError<Evm<Minimal>, Wasm<Union>>)),
// }

impl TryFrom<RelayerMsg> for AnyLightClientIdentified<AnyData> {
    type Error = RelayerMsg;

    fn try_from(value: RelayerMsg) -> Result<Self, Self::Error> {
        match value {
            RelayerMsg::Data(data) => Ok(data),
            _ => Err(value),
        }
    }
}

macro_rules! any_enum {
    (
        $(#[doc = $outer_doc:literal])*
        #[any = $Any:ident]
        pub enum $Enum:ident<Hc: ChainExt, Tr: ChainExt> {
            $(
                $(#[doc = $doc:literal])*
                $(#[serde($untagged:ident)])*
                $Variant:ident(
                    $(#[$variant_inner_meta:meta])*
                    $VariantInner:ty
                ),
            )+
        }
    ) => {
        #[derive(
            ::frame_support_procedural::DebugNoBound,
            ::frame_support_procedural::CloneNoBound,
            ::frame_support_procedural::PartialEqNoBound,
            ::serde::Serialize,
            ::serde::Deserialize,
            ::enumorph::Enumorph,
        )]
        #[cfg_attr(
            feature = "arbitrary",
            derive(arbitrary::Arbitrary),
            arbitrary(bound = "Hc: ChainExt, Tr: ChainExt")
        )]
        #[serde(
            bound(serialize = "", deserialize = ""),
            tag = "@type",
            content = "@value",
            rename_all = "snake_case"
        )]
        $(#[doc = $outer_doc])*
        #[allow(clippy::large_enum_variant)]
        pub enum $Enum<Hc: ChainExt, Tr: ChainExt> {
            $(
                $(#[doc = $doc])*
                $(#[serde($untagged)])*
                $Variant(
                    $(#[$variant_inner_meta])*
                    $VariantInner
                ),
            )+
        }

        pub enum $Any {}
        impl crate::AnyLightClient for $Any {
            type Inner<Hc: ChainExt, Tr: ChainExt> = $Enum<Hc, Tr>;
        }

        const _: () = {
            use crate::{AnyLightClientIdentified, Identified};

            $(
                impl<Hc: ChainExt, Tr: ChainExt> From<Identified<Hc, Tr, $VariantInner>>
                    for AnyLightClientIdentified<$Any>
                where
                    $VariantInner: Into<$Enum<Hc, Tr>>,
                    AnyLightClientIdentified<$Any>: From<Identified<Hc, Tr, $Enum<Hc, Tr>>>,
                {
                    fn from(
                        Identified {
                            chain_id,
                            t,
                            __marker: _,
                        }: Identified<Hc, Tr, $VariantInner>,
                    ) -> Self {
                        Self::from(Identified::new(
                            chain_id,
                            <$Enum<Hc, Tr>>::from(t),
                        ))
                    }
                }

                impl<Hc: ChainExt, Tr: ChainExt>
                    TryFrom<AnyLightClientIdentified<$Any>> for Identified<Hc, Tr, $VariantInner>
                where
                    Identified<Hc, Tr, $Enum<Hc, Tr>>: TryFrom<AnyLightClientIdentified<$Any>, Error = AnyLightClientIdentified<$Any>>
                    + Into<AnyLightClientIdentified<$Any>>,
                {
                    type Error = AnyLightClientIdentified<$Any>;

                    fn try_from(value: AnyLightClientIdentified<$Any>) -> Result<Self, Self::Error> {
                        let Identified {
                            chain_id,
                            t,
                            __marker: _,
                        } = <Identified<Hc, Tr, $Enum<Hc, Tr>>>::try_from(value)?;

                        Ok(Identified::new(
                            chain_id.clone(),
                            <$VariantInner>::try_from(t).map_err(|x: $Enum<Hc, Tr>| {
                                Into::<AnyLightClientIdentified<_>>::into(Identified::new(chain_id, x))
                            })?,
                        ))
                    }
                }
            )+
        };
    };
}
pub(crate) use any_enum;

pub type PathOf<Hc, Tr> = proof::Path<ClientIdOf<Hc>, HeightOf<Tr>>;

pub trait AnyLightClient {
    type Inner<Hc: ChainExt, Tr: ChainExt>: Debug
        + Display
        + Clone
        + PartialEq
        + Serialize
        + for<'de> Deserialize<'de>
        + MaybeArbitrary;
}

pub type InnerOf<T, Hc, Tr> = <T as AnyLightClient>::Inner<Hc, Tr>;

#[derive(
    DebugNoBound,
    CloneNoBound,
    PartialEqNoBound,
    Serialize,
    Deserialize,
    derive_more::Display,
    enumorph::Enumorph,
)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "T: AnyLightClient")
)]
#[serde(
    from = "AnyLightClientIdentifiedSerde<T>",
    into = "AnyLightClientIdentifiedSerde<T>",
    bound(serialize = "", deserialize = "")
)]
#[allow(clippy::large_enum_variant)]
pub enum AnyLightClientIdentified<T: AnyLightClient> {
    // The 08-wasm client tracking the state of Evm<Mainnet>.
    #[display(fmt = "EvmMainnetOnUnion({}, {})", "_0.chain_id", "_0.t")]
    EvmMainnetOnUnion(Identified<Wasm<Union>, Evm<Mainnet>, InnerOf<T, Wasm<Union>, Evm<Mainnet>>>),
    // The solidity client on Evm<Mainnet> tracking the state of Wasm<Union>.
    #[display(fmt = "UnionOnEvmMainnet({}, {})", "_0.chain_id", "_0.t")]
    UnionOnEvmMainnet(Identified<Evm<Mainnet>, Wasm<Union>, InnerOf<T, Evm<Mainnet>, Wasm<Union>>>),

    // The 08-wasm client tracking the state of Evm<Minimal>.
    #[display(fmt = "EvmMinimalOnUnion({}, {})", "_0.chain_id", "_0.t")]
    EvmMinimalOnUnion(Identified<Wasm<Union>, Evm<Minimal>, InnerOf<T, Wasm<Union>, Evm<Minimal>>>),
    // The solidity client on Evm<Minimal> tracking the state of Wasm<Union>.
    #[display(fmt = "UnionOnEvmMinimal({}, {})", "_0.chain_id", "_0.t")]
    UnionOnEvmMinimal(Identified<Evm<Minimal>, Wasm<Union>, InnerOf<T, Evm<Minimal>, Wasm<Union>>>),

    #[display(fmt = "CosmosOnUnion({}, {})", "_0.chain_id", "_0.t")]
    CosmosOnUnion(Identified<Union, Wasm<Cosmos>, InnerOf<T, Union, Wasm<Cosmos>>>),
    #[display(fmt = "UnionOnCosmos({}, {})", "_0.chain_id", "_0.t")]
    UnionOnCosmos(Identified<Wasm<Cosmos>, Union, InnerOf<T, Wasm<Cosmos>, Union>>),
}

#[derive(Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), untagged, deny_unknown_fields)]
#[allow(clippy::large_enum_variant)]
enum AnyLightClientIdentifiedSerde<T: AnyLightClient> {
    EvmMainnetOnUnion(
        Inner<
            Wasm<Union>,
            Evm<Mainnet>,
            Identified<Wasm<Union>, Evm<Mainnet>, InnerOf<T, Wasm<Union>, Evm<Mainnet>>>,
        >,
    ),
    UnionOnEvmMainnet(
        Inner<
            Evm<Mainnet>,
            Wasm<Union>,
            Identified<Evm<Mainnet>, Wasm<Union>, InnerOf<T, Evm<Mainnet>, Wasm<Union>>>,
        >,
    ),

    EvmMinimalOnUnion(
        Inner<
            Wasm<Union>,
            Evm<Minimal>,
            Identified<Wasm<Union>, Evm<Minimal>, InnerOf<T, Wasm<Union>, Evm<Minimal>>>,
        >,
    ),
    UnionOnEvmMinimal(
        Inner<
            Evm<Minimal>,
            Wasm<Union>,
            Identified<Evm<Minimal>, Wasm<Union>, InnerOf<T, Evm<Minimal>, Wasm<Union>>>,
        >,
    ),

    CosmosOnUnion(
        Inner<
            Union,
            Wasm<Cosmos>,
            Identified<Union, Wasm<Cosmos>, InnerOf<T, Union, Wasm<Cosmos>>>,
        >,
    ),
    UnionOnCosmos(
        Inner<
            Wasm<Cosmos>,
            Union,
            Identified<Wasm<Cosmos>, Union, InnerOf<T, Wasm<Cosmos>, Union>>,
        >,
    ),
}

impl<T: AnyLightClient> From<AnyLightClientIdentified<T>> for AnyLightClientIdentifiedSerde<T> {
    fn from(value: AnyLightClientIdentified<T>) -> Self {
        match value {
            AnyLightClientIdentified::EvmMainnetOnUnion(t) => {
                Self::EvmMainnetOnUnion(Inner::new(t))
            }
            AnyLightClientIdentified::UnionOnEvmMainnet(t) => {
                Self::UnionOnEvmMainnet(Inner::new(t))
            }
            AnyLightClientIdentified::EvmMinimalOnUnion(t) => {
                Self::EvmMinimalOnUnion(Inner::new(t))
            }
            AnyLightClientIdentified::UnionOnEvmMinimal(t) => {
                Self::UnionOnEvmMinimal(Inner::new(t))
            }
            AnyLightClientIdentified::CosmosOnUnion(t) => Self::CosmosOnUnion(Inner::new(t)),
            AnyLightClientIdentified::UnionOnCosmos(t) => Self::UnionOnCosmos(Inner::new(t)),
        }
    }
}

impl<T: AnyLightClient> From<AnyLightClientIdentifiedSerde<T>> for AnyLightClientIdentified<T> {
    fn from(value: AnyLightClientIdentifiedSerde<T>) -> Self {
        match value {
            AnyLightClientIdentifiedSerde::EvmMainnetOnUnion(t) => Self::EvmMainnetOnUnion(t.inner),
            AnyLightClientIdentifiedSerde::UnionOnEvmMainnet(t) => Self::UnionOnEvmMainnet(t.inner),
            AnyLightClientIdentifiedSerde::EvmMinimalOnUnion(t) => Self::EvmMinimalOnUnion(t.inner),
            AnyLightClientIdentifiedSerde::UnionOnEvmMinimal(t) => Self::UnionOnEvmMinimal(t.inner),
            AnyLightClientIdentifiedSerde::CosmosOnUnion(t) => Self::CosmosOnUnion(t.inner),
            AnyLightClientIdentifiedSerde::UnionOnCosmos(t) => Self::UnionOnCosmos(t.inner),
        }
    }
}

#[macro_export]
macro_rules! identified {
    ($Ty:ident<$Hc:ty, $Tr:ty>) => {
        $crate::Identified<$Hc, $Tr, $Ty<$Hc, $Tr>>
    };
}

#[derive(DebugNoBound, thiserror::Error)]
pub enum LcError<Hc: ChainExt, Tr: ChainExt> {
    #[error(transparent)]
    Msg(Hc::MsgError),
    __Marker(PhantomData<fn() -> Tr>),
}

#[derive(Serialize, Deserialize)]
#[serde(
    bound(
        serialize = "T: ::serde::Serialize",
        deserialize = "T: for<'d> Deserialize<'d>"
    ),
    deny_unknown_fields
)]
// TODO: `T: AnyLightClient`
// prerequisites: derive macro for AnyLightClient
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Hc: Chain, T: arbitrary::Arbitrary<'arbitrary>")
)]
pub struct Identified<Hc: Chain, Tr, T> {
    // #[serde(rename = "@chain_id")]
    pub chain_id: ChainIdOf<Hc>,
    pub t: T,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> Tr>,
}

impl<Hc: Chain, Tr, Data: PartialEq> PartialEq for Identified<Hc, Tr, Data> {
    fn eq(&self, other: &Self) -> bool {
        self.chain_id == other.chain_id && self.t == other.t
    }
}

impl<Hc: Chain, Tr, Data: std::error::Error + Debug + Clone + PartialEq> std::error::Error
    for Identified<Hc, Tr, Data>
{
}

impl<Hc: Chain, Tr, Data: std::fmt::Display + Debug + Clone + PartialEq> std::fmt::Display
    for Identified<Hc, Tr, Data>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(chain id `{}`): {}", self.chain_id, self.t)
    }
}

impl<Hc: Chain, Tr, Data: Debug> Debug for Identified<Hc, Tr, Data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Identified")
            .field("chain_id", &self.chain_id)
            .field("t", &self.t)
            .finish()
    }
}

impl<Hc: Chain, Tr, Data: Clone> Clone for Identified<Hc, Tr, Data> {
    fn clone(&self) -> Self {
        Self {
            chain_id: self.chain_id.clone(),
            t: self.t.clone(),
            __marker: PhantomData,
        }
    }
}

impl<Hc: Chain, Tr, Data: Debug + Clone + PartialEq> Identified<Hc, Tr, Data> {
    pub fn new(chain_id: ChainIdOf<Hc>, data: Data) -> Self {
        Self {
            chain_id,
            t: data,
            __marker: PhantomData,
        }
    }
}

pub trait DoAggregate: Sized + Debug + Clone + PartialEq {
    fn do_aggregate(_: Self, _: VecDeque<AnyLightClientIdentified<AnyData>>) -> RelayerMsg;
}

pub trait DoFetchState<Hc: ChainExt, Tr: ChainExt>: ChainExt {
    fn state(hc: &Hc, at: Hc::Height, path: PathOf<Hc, Tr>) -> RelayerMsg;

    #[deprecated = "will be removed in favor of an aggregation with state"]
    fn query_client_state(
        hc: &Hc,
        client_id: Hc::ClientId,
        height: Hc::Height,
    ) -> impl Future<Output = Hc::StoredClientState<Tr>> + '_;
}

pub trait DoFetchProof<Hc: ChainExt, Tr: ChainExt>: ChainExt {
    fn proof(hc: &Hc, at: HeightOf<Hc>, path: PathOf<Hc, Tr>) -> RelayerMsg;
}

pub trait DoFetchUpdateHeaders<Hc: ChainExt, Tr: ChainExt>: ChainExt {
    fn fetch_update_headers(hc: &Hc, update_info: FetchUpdateHeaders<Hc, Tr>) -> RelayerMsg;
}

pub trait DoMsg<Hc: ChainExt, Tr: ChainExt>: ChainExt {
    fn msg(&self, msg: Msg<Hc, Tr>) -> impl Future<Output = Result<(), Self::MsgError>> + '_;
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    // use hex_literal::hex;

    // use super::*;
    // use crate::{chain::cosmos::EthereumConfig, msg::CreateClientData};

    use std::{collections::VecDeque, fmt::Debug, marker::PhantomData};

    use chain_utils::{cosmos::Cosmos, evm::Evm, union::Union};
    use hex_literal::hex;
    use queue_msg::{aggregate, defer_relative, event, fetch, msg, repeat};
    use serde::{de::DeserializeOwned, Serialize};
    use unionlabs::{
        ethereum::config::Minimal,
        events::{ConnectionOpenAck, ConnectionOpenTry},
        hash::{H160, H256},
        ibc::core::{
            channel::{
                self, channel::Channel, msg_channel_open_init::MsgChannelOpenInit, order::Order,
            },
            commitment::merkle_prefix::MerklePrefix,
            connection::{
                self, msg_connection_open_init::MsgConnectionOpenInit,
                msg_connection_open_try::MsgConnectionOpenTry, version::Version,
            },
        },
        proof::{self, ConnectionPath},
        uint::U256,
        validated::ValidateT,
        EmptyString, QueryHeight, DELAY_PERIOD,
    };

    use crate::{
        aggregate::{Aggregate, AggregateCreateClient, AnyAggregate},
        chain_impls::{
            cosmos_sdk::fetch::{AbciQueryType, FetchAbciQuery},
            evm::EvmConfig,
            union::UnionFetch,
        },
        data::Data,
        event::{Event, IbcEvent},
        fetch::{
            AnyFetch, Fetch, FetchSelfClientState, FetchSelfConsensusState, FetchState,
            LightClientSpecificFetch,
        },
        msg::{
            AnyMsg, Msg, MsgChannelOpenInitData, MsgConnectionOpenInitData,
            MsgConnectionOpenTryData,
        },
        seq, Identified, RelayerMsg, Wasm, WasmConfig,
    };

    macro_rules! parse {
        ($expr:expr) => {
            $expr.parse().unwrap()
        };
    }

    #[test]
    fn msg_serde() {
        let union_chain_id: String = parse!("union-devnet-1");
        let eth_chain_id: U256 = parse!("32382");
        let cosmos_chain_id: String = parse!("simd-devnet-1");

        println!("---------------------------------------");
        println!("Union - Eth (Sending to Union) Connection Open: ");
        println!("---------------------------------------");
        print_json(msg(Identified::<Wasm<Union>, Evm<Minimal>, _>::new(
            union_chain_id.clone(),
            MsgConnectionOpenInitData(MsgConnectionOpenInit {
                client_id: parse!("08-wasm-2"),
                counterparty: connection::counterparty::Counterparty {
                    client_id: parse!("cometbls-0"),
                    connection_id: parse!(""),
                    prefix: MerklePrefix {
                        key_prefix: b"ibc".to_vec(),
                    },
                },
                version: Version {
                    identifier: "1".into(),
                    features: [Order::Ordered, Order::Unordered].into_iter().collect(),
                },
                delay_period: DELAY_PERIOD,
            }),
        )));

        println!("---------------------------------------");
        println!("Fetch Client State: ");
        println!("---------------------------------------");
        print_json(fetch(Identified::<Wasm<Union>, Evm<Minimal>, _>::new(
            union_chain_id.clone(),
            LightClientSpecificFetch(UnionFetch::AbciQuery(FetchAbciQuery {
                path: proof::Path::ClientStatePath(proof::ClientStatePath {
                    client_id: parse!("client-id"),
                }),
                height: parse!("123-456"),
                ty: AbciQueryType::State,
            })),
        )));

        println!("---------------------------------------");
        println!("Eth - Union (Sending to Union) Channel Open: ");
        println!("---------------------------------------");
        print_json(msg(Identified::<Wasm<Union>, Evm<Minimal>, _>::new(
            union_chain_id.clone(),
            MsgChannelOpenInitData {
                msg: MsgChannelOpenInit {
                    port_id: parse!("WASM_PORT_ID"),
                    channel: Channel {
                        state: channel::state::State::Init,
                        ordering: channel::order::Order::Unordered,
                        counterparty: channel::counterparty::Counterparty {
                            port_id: parse!("ucs01-relay"),
                            channel_id: parse!(""),
                        },
                        connection_hops: vec![parse!("connection-8")],
                        version: "ucs01-0".to_string(),
                    },
                },
                __marker: PhantomData,
            },
        )));

        println!("---------------------------------------");
        println!("Eth - Union (Starting on Union) Channel Open: ");
        println!("---------------------------------------");
        print_json(msg(Identified::<Evm<Minimal>, Wasm<Union>, _>::new(
            eth_chain_id,
            MsgChannelOpenInitData {
                msg: MsgChannelOpenInit {
                    port_id: parse!("ucs01-relay"),
                    channel: Channel {
                        state: channel::state::State::Init,
                        ordering: channel::order::Order::Ordered,
                        counterparty: channel::counterparty::Counterparty {
                            port_id: parse!("ucs01-relay"),
                            channel_id: parse!(""),
                        },
                        connection_hops: vec![parse!("connection-8")],
                        version: "ucs001-pingpong".to_string(),
                    },
                },
                __marker: PhantomData,
            },
        )));

        println!("---------------------------------------");
        println!("Eth - Union (Sending to Eth) Connection Open: ");
        println!("---------------------------------------");
        print_json(msg(Identified::<Evm<Minimal>, Wasm<Union>, _>::new(
            eth_chain_id,
            MsgConnectionOpenInitData(MsgConnectionOpenInit {
                client_id: parse!("cometbls-0"),
                counterparty: connection::counterparty::Counterparty {
                    client_id: parse!("08-wasm-0"),
                    connection_id: parse!(""),
                    prefix: MerklePrefix {
                        key_prefix: b"ibc".to_vec(),
                    },
                },
                version: Version {
                    identifier: "1".into(),
                    features: [Order::Ordered, Order::Unordered].into_iter().collect(),
                },
                delay_period: DELAY_PERIOD,
            }),
        )));

        println!("---------------------------------------");
        println!("Eth - Union (Sending to Eth) Connection Try: ");
        println!("---------------------------------------");
        print_json(event(Identified::<Evm<Minimal>, Wasm<Union>, _>::new(
            eth_chain_id,
            IbcEvent {
                block_hash: H256([0; 32]),
                height: parse!("0-2941"),
                event: unionlabs::events::IbcEvent::ConnectionOpenTry(ConnectionOpenTry {
                    connection_id: parse!("connection-0"),
                    client_id: parse!("cometbls-0"),
                    counterparty_client_id: parse!("08-wasm-1"),
                    counterparty_connection_id: parse!("connection-14"),
                }),
            },
        )));

        println!("---------------------------------------");
        println!("Eth - Union (Sending to Eth) Update Client: ");
        println!("---------------------------------------");
        print_json(repeat(
            u64::MAX,
            seq([
                event(Identified::<Evm<Minimal>, Wasm<Union>, _>::new(
                    eth_chain_id,
                    crate::event::Command::UpdateClient {
                        client_id: parse!("cometbls-0"),
                        counterparty_client_id: parse!("08-wasm-0"),
                    },
                )),
                defer_relative(10),
            ]),
        ));

        println!("---------------------------------------");
        println!("Eth - Union (Sending to Union) Update Client: ");
        println!("---------------------------------------");
        print_json(repeat(
            u64::MAX,
            seq([
                event(Identified::<Wasm<Union>, Evm<Minimal>, _>::new(
                    union_chain_id.clone(),
                    crate::event::Command::UpdateClient {
                        client_id: parse!("08-wasm-0"),
                        counterparty_client_id: parse!("cometbls-0"),
                    },
                )),
                defer_relative(10),
            ]),
        ));

        println!("---------------------------------------");
        println!("Cosmos - Union (Sending to Cosmos) Update Client: ");
        println!("---------------------------------------");
        print_json(repeat(
            u64::MAX,
            seq([
                event(Identified::<Wasm<Cosmos>, Union, _>::new(
                    cosmos_chain_id.clone(),
                    crate::event::Command::UpdateClient {
                        client_id: parse!("08-wasm-0"),
                        counterparty_client_id: parse!("07-tendermint-0"),
                    },
                )),
                defer_relative(10),
            ]),
        ));

        println!("---------------------------------------");
        println!("Cosmos - Union (Sending to Union) Update Client: ");
        println!("---------------------------------------");
        print_json(repeat(
            u64::MAX,
            seq([
                event(Identified::<Union, Wasm<Cosmos>, _>::new(
                    union_chain_id.clone(),
                    crate::event::Command::UpdateClient {
                        client_id: parse!("07-tendermint-0"),
                        counterparty_client_id: parse!("08-wasm-0"),
                    },
                )),
                defer_relative(10),
            ]),
        ));

        println!("---------------------------------------");
        println!("Union - Eth Create Both Clients: ");
        println!("---------------------------------------");
        print_json(seq([
            aggregate(
                [
                    fetch(Identified::<Wasm<Union>, Evm<Minimal>, _>::new(
                        union_chain_id.clone(),
                        FetchSelfClientState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                    fetch(Identified::<Wasm<Union>, Evm<Minimal>, _>::new(
                        union_chain_id.clone(),
                        FetchSelfConsensusState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                ],
                [],
                Identified::<Evm<Minimal>, Wasm<Union>, _>::new(
                    eth_chain_id,
                    AggregateCreateClient {
                        config: EvmConfig {
                            client_type: "cometbls".to_string(),
                            client_address: H160(hex!("83428c7db9815f482a39a1715684dcf755021997")),
                        },
                        __marker: PhantomData,
                    },
                ),
            ),
            aggregate(
                [
                    fetch(Identified::<Evm<Minimal>, Wasm<Union>, _>::new(
                        eth_chain_id,
                        FetchSelfClientState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                    fetch(Identified::<Evm<Minimal>, Wasm<Union>, _>::new(
                        eth_chain_id,
                        FetchSelfConsensusState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                ],
                [],
                Identified::<Wasm<Union>, Evm<Minimal>, _>::new(
                    union_chain_id.clone(),
                    AggregateCreateClient {
                        config: WasmConfig {
                            checksum: H256(hex!(
                                "78266014ea77f3b785e45a33d1f8d3709444a076b3b38b2aeef265b39ad1e494"
                            )),
                        },
                        __marker: PhantomData,
                    },
                ),
            ),
        ]));

        println!("---------------------------------------");
        println!("Union - Cosmos Create Both Client: ");
        println!("---------------------------------------");
        print_json(seq([
            aggregate(
                [
                    fetch(Identified::<Wasm<Cosmos>, Union, _>::new(
                        cosmos_chain_id.clone(),
                        FetchSelfClientState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                    fetch(Identified::<Wasm<Cosmos>, Union, _>::new(
                        cosmos_chain_id.clone(),
                        FetchSelfConsensusState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                ],
                [],
                Identified::<Union, Wasm<Cosmos>, _>::new(
                    union_chain_id.clone(),
                    AggregateCreateClient {
                        config: (),
                        __marker: PhantomData,
                    },
                ),
            ),
            aggregate(
                [
                    fetch(Identified::<Union, Wasm<Cosmos>, _>::new(
                        union_chain_id.clone(),
                        FetchSelfClientState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                    fetch(Identified::<Union, Wasm<Cosmos>, _>::new(
                        union_chain_id.clone(),
                        FetchSelfConsensusState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                ],
                [],
                Identified::<Wasm<Cosmos>, Union, _>::new(
                    cosmos_chain_id,
                    AggregateCreateClient {
                        config: WasmConfig {
                            checksum: H256(hex!(
                                "78266014ea77f3b785e45a33d1f8d3709444a076b3b38b2aeef265b39ad1e494"
                            )),
                        },
                        __marker: PhantomData,
                    },
                ),
            ),
        ]));

        // print_json(RelayerMsg::Lc(AnyLcMsg::EthereumMinimal(LcMsg::Event(
        //     Identified {
        //         chain_id: union_chain_id.clone(),
        //         data: crate::event::Event {
        //             block_hash: H256([0; 32]),
        //             height: parse!("1-1433"),
        //             event: IbcEvent::ConnectionOpenAck(ConnectionOpenAck {
        //                 connection_id: parse!("connection-5"),
        //                 client_id: parse!("08-wasm-0"),
        //                 counterparty_client_id: parse!("cometbls-0"),
        //                 counterparty_connection_id: parse!("connection-4"),
        //             }),
        //         },
        //     },
        // ))));
        print_json(fetch(Identified::<Wasm<Union>, Evm<Minimal>, _>::new(
            union_chain_id.clone(),
            FetchState {
                at: parse!("1-103"),
                path: ConnectionPath {
                    connection_id: parse!("connection-1"),
                }
                .into(),
            },
        )))
    }

    fn print_json(msg: RelayerMsg) {
        let json = serde_json::to_string(&msg).unwrap();

        println!("{json}\n");

        let from_json = serde_json::from_str(&json).unwrap();

        assert_eq!(&msg, &from_json, "json roundtrip failed");
    }
}

#[derive(Debug, Clone)]
pub struct Wasm<C: Chain>(pub C);

pub trait Wraps<T: CosmosSdkChain + ChainExt>: CosmosSdkChain + ChainExt {
    fn inner(&self) -> &T;
}

impl<T: CosmosSdkChain> CosmosSdkChain for Wasm<T> {
    fn grpc_url(&self) -> String {
        self.0.grpc_url()
    }

    fn fee_denom(&self) -> String {
        self.0.fee_denom()
    }

    fn tm_client(&self) -> &tendermint_rpc::WebSocketClient {
        self.0.tm_client()
    }

    fn signers(&self) -> &chain_utils::Pool<unionlabs::signer::CosmosSigner> {
        self.0.signers()
    }

    fn checksum_cache(&self) -> &std::sync::Arc<dashmap::DashMap<H256, unionlabs::WasmClientType>> {
        self.0.checksum_cache()
    }
}

impl<T: CosmosSdkChain + ChainExt> Wraps<T> for T {
    fn inner(&self) -> &T {
        self
    }
}

impl<T: CosmosSdkChain + ChainExt> Wraps<T> for Wasm<T>
where
    Wasm<T>: ChainExt,
{
    fn inner(&self) -> &T {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct WasmConfig {
    pub checksum: H256,
    // pub inner: T,
}

impl<Hc: CosmosSdkChain> Chain for Wasm<Hc> {
    type ChainType = Hc::ChainType;

    type SelfClientState = Hc::SelfClientState;
    type SelfConsensusState = Hc::SelfConsensusState;
    type Header = Hc::Header;

    type StoredClientState<Tr: Chain> = Any<wasm::client_state::ClientState<Tr::SelfClientState>>;
    type StoredConsensusState<Tr: Chain> =
        Any<wasm::consensus_state::ConsensusState<Tr::SelfConsensusState>>;

    type Height = Hc::Height;

    type ClientId = Hc::ClientId;
    type ClientType = Hc::ClientType;

    type Error = Hc::Error;

    type IbcStateEncoding = Proto;

    type StateProof = Hc::StateProof;

    fn chain_id(&self) -> <Self::SelfClientState as unionlabs::traits::ClientState>::ChainId {
        self.0.chain_id()
    }

    fn query_latest_height(&self) -> impl Future<Output = Result<Self::Height, Self::Error>> + '_ {
        self.0.query_latest_height()
    }

    fn query_latest_height_as_destination(
        &self,
    ) -> impl Future<Output = Result<Self::Height, Self::Error>> + '_ {
        self.0.query_latest_height_as_destination()
    }

    fn query_latest_timestamp(&self) -> impl Future<Output = Result<i64, Self::Error>> + '_ {
        self.0.query_latest_timestamp()
    }

    fn self_client_state(
        &self,
        height: Self::Height,
    ) -> impl Future<Output = Self::SelfClientState> + '_ {
        self.0.self_client_state(height)
    }

    fn self_consensus_state(
        &self,
        height: Self::Height,
    ) -> impl Future<Output = Self::SelfConsensusState> + '_ {
        self.0.self_consensus_state(height)
    }

    fn read_ack(
        &self,
        block_hash: unionlabs::hash::H256,
        destination_channel_id: unionlabs::id::ChannelId,
        destination_port_id: unionlabs::id::PortId,
        sequence: u64,
    ) -> impl Future<Output = Vec<u8>> + '_ {
        self.0.read_ack(
            block_hash,
            destination_channel_id,
            destination_port_id,
            sequence,
        )
    }
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(bound(serialize = "", deserialize = ""), transparent)]
#[display(fmt = "{_0}")]
pub struct WasmDataMsg<Hc: ChainExt, Tr: ChainExt>(pub Hc::Data<Tr>);

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(bound(serialize = "", deserialize = ""), transparent)]
#[display(fmt = "{_0}")]
pub struct WasmFetchMsg<Hc: ChainExt, Tr: ChainExt>(pub Hc::Fetch<Tr>);

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(bound(serialize = "", deserialize = ""), transparent)]
#[display(fmt = "{_0}")]
pub struct WasmAggregateMsg<Hc: ChainExt, Tr: ChainExt>(pub Hc::Aggregate<Tr>);

impl<Hc: CosmosSdkChain + ChainExt, Tr: ChainExt> DoAggregate for identified!(WasmAggregateMsg<Hc, Tr>)
where
    Identified<Hc, Tr, Hc::Aggregate<Tr>>: DoAggregate,
{
    fn do_aggregate(i: Self, v: VecDeque<AnyLightClientIdentified<AnyData>>) -> RelayerMsg {
        <Identified<_, _, Hc::Aggregate<Tr>>>::do_aggregate(
            Identified {
                chain_id: i.chain_id,
                t: i.t.0,
                __marker: PhantomData,
            },
            v,
        )
    }
}

impl<Hc, Tr> DoMsg<Self, Tr> for Wasm<Hc>
where
    Hc: ChainExt<MsgError = BroadcastTxCommitError> + CosmosSdkChain,
    Tr: ChainExt,

    ConsensusStateOf<Tr>: IntoProto,
    <ConsensusStateOf<Tr> as unionlabs::Proto>::Proto: TypeUrl,

    ClientStateOf<Tr>: IntoProto,
    <ClientStateOf<Tr> as unionlabs::Proto>::Proto: TypeUrl,

    HeaderOf<Tr>: IntoProto,
    <HeaderOf<Tr> as unionlabs::Proto>::Proto: TypeUrl,

    ConsensusStateOf<Hc>: IntoProto,
    <ConsensusStateOf<Hc> as unionlabs::Proto>::Proto: TypeUrl,

    ClientStateOf<Hc>: IntoProto,
    <ClientStateOf<Hc> as unionlabs::Proto>::Proto: TypeUrl,

    HeaderOf<Hc>: IntoProto,
    <HeaderOf<Hc> as unionlabs::Proto>::Proto: TypeUrl,

    // TODO: Move this associated type to this trait
    Wasm<Hc>: ChainExt<
        SelfClientState = Hc::SelfClientState,
        SelfConsensusState = Hc::SelfConsensusState,
        MsgError = BroadcastTxCommitError,
        Config = WasmConfig,
    >,

    Tr::StoredClientState<Wasm<Hc>>: IntoProto + IntoAny,
    Tr::StateProof: Encode<Proto>,
{
    async fn msg(&self, msg: Msg<Self, Tr>) -> Result<(), Self::MsgError> {
        self.0
            .signers()
            .with(|signer| async {
                let msg_any = match msg {
                    Msg::ConnectionOpenInit(MsgConnectionOpenInitData(data)) => {
                        mk_any(&protos::ibc::core::connection::v1::MsgConnectionOpenInit {
                            client_id: data.client_id.to_string(),
                            counterparty: Some(data.counterparty.into()),
                            version: Some(data.version.into()),
                            signer: signer.to_string(),
                            delay_period: data.delay_period,
                        })
                    }
                    Msg::ConnectionOpenTry(MsgConnectionOpenTryData(data)) =>
                    {
                        #[allow(deprecated)]
                        mk_any(&protos::ibc::core::connection::v1::MsgConnectionOpenTry {
                            client_id: data.client_id.to_string(),
                            previous_connection_id: String::new(),
                            client_state: Some(data.client_state.into_any().into()),
                            counterparty: Some(data.counterparty.into()),
                            delay_period: data.delay_period,
                            counterparty_versions: data
                                .counterparty_versions
                                .into_iter()
                                .map(Into::into)
                                .collect(),
                            proof_height: Some(data.proof_height.into_height().into()),
                            proof_init: data.proof_init.encode(),
                            proof_client: data.proof_client.encode(),
                            proof_consensus: data.proof_consensus.encode(),
                            consensus_height: Some(data.consensus_height.into_height().into()),
                            signer: signer.to_string(),
                            host_consensus_state_proof: vec![],
                        })
                    }
                    Msg::ConnectionOpenAck(MsgConnectionOpenAckData(data)) => {
                        mk_any(&protos::ibc::core::connection::v1::MsgConnectionOpenAck {
                            client_state: Some(data.client_state.into_any().into()),
                            proof_height: Some(data.proof_height.into_height().into()),
                            proof_client: data.proof_client.encode(),
                            proof_consensus: data.proof_consensus.encode(),
                            consensus_height: Some(data.consensus_height.into_height().into()),
                            signer: signer.to_string(),
                            host_consensus_state_proof: vec![],
                            connection_id: data.connection_id.to_string(),
                            counterparty_connection_id: data.counterparty_connection_id.to_string(),
                            version: Some(data.version.into()),
                            proof_try: data.proof_try.encode(),
                        })
                    }
                    Msg::ConnectionOpenConfirm(data) => mk_any(
                        &protos::ibc::core::connection::v1::MsgConnectionOpenConfirm {
                            connection_id: data.msg.connection_id.to_string(),
                            proof_ack: data.msg.proof_ack.encode(),
                            proof_height: Some(data.msg.proof_height.into_height().into()),
                            signer: signer.to_string(),
                        },
                    ),
                    Msg::ChannelOpenInit(data) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgChannelOpenInit {
                            port_id: data.msg.port_id.to_string(),
                            channel: Some(data.msg.channel.into()),
                            signer: signer.to_string(),
                        })
                    }
                    Msg::ChannelOpenTry(data) =>
                    {
                        #[allow(deprecated)]
                        mk_any(&protos::ibc::core::channel::v1::MsgChannelOpenTry {
                            port_id: data.msg.port_id.to_string(),
                            channel: Some(data.msg.channel.into()),
                            counterparty_version: data.msg.counterparty_version,
                            proof_init: data.msg.proof_init.encode(),
                            proof_height: Some(data.msg.proof_height.into()),
                            previous_channel_id: String::new(),
                            signer: signer.to_string(),
                        })
                    }
                    Msg::ChannelOpenAck(data) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgChannelOpenAck {
                            port_id: data.msg.port_id.to_string(),
                            channel_id: data.msg.channel_id.to_string(),
                            counterparty_version: data.msg.counterparty_version,
                            counterparty_channel_id: data.msg.counterparty_channel_id.to_string(),
                            proof_try: data.msg.proof_try.encode(),
                            proof_height: Some(data.msg.proof_height.into_height().into()),
                            signer: signer.to_string(),
                        })
                    }
                    Msg::ChannelOpenConfirm(data) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgChannelOpenConfirm {
                            port_id: data.msg.port_id.to_string(),
                            channel_id: data.msg.channel_id.to_string(),
                            proof_height: Some(data.msg.proof_height.into_height().into()),
                            signer: signer.to_string(),
                            proof_ack: data.msg.proof_ack.encode(),
                        })
                    }
                    Msg::RecvPacket(data) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgRecvPacket {
                            packet: Some(data.msg.packet.into()),
                            proof_height: Some(data.msg.proof_height.into_height().into()),
                            signer: signer.to_string(),
                            proof_commitment: data.msg.proof_commitment.encode(),
                        })
                    }
                    Msg::AckPacket(data) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgAcknowledgement {
                            packet: Some(data.msg.packet.into()),
                            acknowledgement: data.msg.acknowledgement,
                            proof_acked: data.msg.proof_acked.encode(),
                            proof_height: Some(data.msg.proof_height.into_height().into()),
                            signer: signer.to_string(),
                        })
                    }
                    Msg::CreateClient(data) => {
                        mk_any(&protos::ibc::core::client::v1::MsgCreateClient {
                            client_state: Some(
                                Any(wasm::client_state::ClientState {
                                    latest_height: data.msg.client_state.height().into(),
                                    data: data.msg.client_state,
                                    checksum: data.config.checksum,
                                })
                                .into(),
                            ),
                            consensus_state: Some(
                                Any(wasm::consensus_state::ConsensusState {
                                    data: data.msg.consensus_state,
                                })
                                .into(),
                            ),
                            signer: signer.to_string(),
                        })
                    }
                    Msg::UpdateClient(MsgUpdateClientData(data)) => {
                        mk_any(&protos::ibc::core::client::v1::MsgUpdateClient {
                            signer: signer.to_string(),
                            client_id: data.client_id.to_string(),
                            client_message: Some(
                                Any(wasm::client_message::ClientMessage {
                                    data: data.client_message,
                                })
                                .into(),
                            ),
                        })
                    }
                };

                self.0
                    .broadcast_tx_commit(signer, [msg_any])
                    .await
                    .map(|_| ())
            })
            .await
    }
}

impl<Hc: ChainExt + CosmosSdkChain + DoFetchProof<Wasm<Hc>, Tr>, Tr: ChainExt>
    DoFetchProof<Self, Tr> for Wasm<Hc>
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Wasm<Hc>, Tr>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Wasm<Hc>, Tr>)>,
    Wasm<Hc>: ChainExt,
{
    fn proof(hc: &Self, at: HeightOf<Self>, path: PathOf<Wasm<Hc>, Tr>) -> RelayerMsg {
        Hc::proof(hc, at, path)
    }
}

impl<Hc: ChainExt + CosmosSdkChain + DoFetchState<Wasm<Hc>, Tr>, Tr: ChainExt>
    DoFetchState<Self, Tr> for Wasm<Hc>
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Wasm<Hc>, Tr>)>,
    Wasm<Hc>: ChainExt,
{
    fn state(hc: &Self, at: HeightOf<Self>, path: PathOf<Wasm<Hc>, Tr>) -> RelayerMsg {
        Hc::state(hc, at, path)
    }

    fn query_client_state(
        hc: &Self,
        client_id: Self::ClientId,
        height: Self::Height,
    ) -> impl Future<Output = Self::StoredClientState<Tr>> + '_ {
        Hc::query_client_state(hc, client_id, height)
    }
}

impl<Hc: ChainExt + CosmosSdkChain + DoFetchUpdateHeaders<Self, Tr>, Tr: ChainExt>
    DoFetchUpdateHeaders<Self, Tr> for Wasm<Hc>
where
    Wasm<Hc>: ChainExt,
{
    fn fetch_update_headers(hc: &Self, update_info: FetchUpdateHeaders<Self, Tr>) -> RelayerMsg {
        Hc::fetch_update_headers(
            hc,
            FetchUpdateHeaders {
                client_id: update_info.client_id,
                counterparty_chain_id: update_info.counterparty_chain_id,
                counterparty_client_id: update_info.counterparty_client_id,
                update_from: update_info.update_from,
                update_to: update_info.update_to,
            },
        )
    }
}

#[derive(Serialize, Deserialize)]
#[serde(
    bound(serialize = "S: Serialize", deserialize = "S: for<'d> Deserialize<'d>"),
    deny_unknown_fields
)]
struct Inner<Hc: Chain, Tr: Chain, S> {
    #[serde(rename = "@host_chain", with = "::unionlabs::traits::from_str_exact")]
    host_chain: Hc::ChainType,
    #[serde(rename = "@tracking", with = "::unionlabs::traits::from_str_exact")]
    tracking: Tr::ChainType,
    #[serde(rename = "@value")]
    inner: S,
}

impl<Hc: Chain, Tr: Chain, S> Inner<Hc, Tr, S> {
    fn new(s: S) -> Inner<Hc, Tr, S> {
        Self {
            host_chain: Hc::ChainType::default(),
            tracking: Tr::ChainType::default(),
            inner: s,
        }
    }
}

// #[test]
// fn test_tester() {
//     let json = serde_json::to_string_pretty(&Tester::AB(Struct { field: 1 })).unwrap();
//     println!("{json}");
// }

macro_rules! any_lc {
    (|$msg:ident| $expr:expr) => {
        match $msg {
            AnyLightClientIdentified::EvmMainnetOnUnion($msg) => $expr,
            AnyLightClientIdentified::UnionOnEvmMainnet($msg) => $expr,
            AnyLightClientIdentified::EvmMinimalOnUnion($msg) => $expr,
            AnyLightClientIdentified::UnionOnEvmMinimal($msg) => $expr,
            AnyLightClientIdentified::CosmosOnUnion($msg) => $expr,
            AnyLightClientIdentified::UnionOnCosmos($msg) => $expr,
        }
    };
}
pub(crate) use any_lc;
