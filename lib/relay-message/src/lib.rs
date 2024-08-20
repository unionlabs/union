#![feature(trait_alias)]
#![allow(clippy::type_complexity, async_fn_in_trait)]

use std::{collections::VecDeque, fmt::Debug, future::Future, marker::PhantomData};

use chain_utils::{
    arbitrum::Arbitrum, berachain::Berachain, cosmos::Cosmos, ethereum::Ethereum, scroll::Scroll,
    union::Union, wasm::Wasm, Chains,
};
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use queue_msg::{seq, Op, OpT, QueueMessage};
use serde::{Deserialize, Serialize};
use unionlabs::{
    ethereum::config::{Mainnet, Minimal},
    ics24,
    never::Never,
    traits::{Chain, ChainIdOf, ClientIdOf, HeightOf},
    MaybeArbitrary, MaybeRecoverableError,
};

use crate::{
    aggregate::AnyAggregate,
    data::AnyData,
    effect::{AnyEffect, Effect},
    event::AnyEvent,
    fetch::{AnyFetch, FetchUpdateHeaders},
    wait::AnyWait,
};

pub mod use_aggregate;

pub mod aggregate;
pub mod data;
pub mod effect;
pub mod event;
pub mod fetch;
pub mod wait;

pub mod chain;

pub trait ChainExt: Chain {
    type Data<Tr: ChainExt>: OpT;
    type Fetch<Tr: ChainExt>: OpT;
    type Aggregate<Tr: ChainExt>: OpT;

    /// Error type for [`Self::msg`].
    type MsgError: Debug + MaybeRecoverableError;

    /// The config required to create a light client on this chain.
    type Config: Debug
        + Clone
        + PartialEq
        + Serialize
        + for<'de> Deserialize<'de>
        + MaybeArbitrary
        + Send
        + Sync;
}

pub enum RelayMessage {}

impl QueueMessage for RelayMessage {
    type Event = AnyLightClientIdentified<AnyEvent>;
    type Data = AnyLightClientIdentified<AnyData>;
    type Fetch = AnyLightClientIdentified<AnyFetch>;
    type Effect = AnyLightClientIdentified<AnyEffect>;
    type Wait = AnyLightClientIdentified<AnyWait>;
    type Aggregate = AnyLightClientIdentified<AnyAggregate>;

    type Store = Chains;
}

impl TryFrom<Op<RelayMessage>> for AnyLightClientIdentified<AnyData> {
    type Error = Op<RelayMessage>;

    fn try_from(value: Op<RelayMessage>) -> Result<Self, Self::Error> {
        match value {
            Op::Data(data) => Ok(data),
            _ => Err(value),
        }
    }
}

macro_rules! any_enum {
    (
        $(#[doc = $outer_doc:literal])*
        #[any = $Any:ident]
        $(#[specific = $Specific:ident])?
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
        #[::queue_msg::queue_msg]
        #[derive(::enumorph::Enumorph)]
        $(#[doc = $outer_doc])*
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
                        Self::from(crate::id(
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

                        Ok(crate::id(
                            chain_id.clone(),
                            <$VariantInner>::try_from(t).map_err(|x: $Enum<Hc, Tr>| {
                                Into::<AnyLightClientIdentified<_>>::into(crate::id(chain_id, x))
                            })?,
                        ))
                    }
                }
            )+

            $(
                impl<Hc: ChainExt, Tr: ChainExt> $Enum<Hc, Tr> {
                    pub fn specific(t: impl Into<Hc::$Enum<Tr>>) -> $Enum<Hc, Tr> {
                        $Specific(t.into()).into()
                    }
                }
            )?
        };
    };
}
pub(crate) use any_enum;

pub type PathOf<Hc, Tr> = ics24::Path<ClientIdOf<Hc>, HeightOf<Tr>>;

pub trait AnyLightClient {
    type Inner<Hc: ChainExt, Tr: ChainExt>: Debug + Clone + PartialEq;
    // + Serialize
    // + for<'de> Deserialize<'de>
    // + MaybeArbitrary;
}

pub type InnerOf<T, Hc, Tr> = <T as AnyLightClient>::Inner<Hc, Tr>;

macro_rules! lc {
    // A on B
    ($Tr:ty => $Hc:ty) => {
        Identified<$Hc, $Tr, InnerOf<T, $Hc, $Tr>>
    };
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, enumorph::Enumorph,
)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "")
)]
#[serde(
    from = "AnyLightClientIdentifiedSerde<T>",
    into = "AnyLightClientIdentifiedSerde<T>",
    bound(
        serialize = "AnyLightClientIdentifiedSerde<T>: Serialize",
        deserialize = "AnyLightClientIdentifiedSerde<T>: Deserialize<'de>"
    )
)]
#[allow(clippy::large_enum_variant)]
pub enum AnyLightClientIdentified<T: AnyLightClient> {
    /// The 08-wasm client tracking the state of Ethereum<Mainnet>.
    EthereumMainnetOnUnion(lc!(Ethereum<Mainnet> => Wasm<Union>)),
    /// The solidity client on Ethereum<Mainnet> tracking the state of Wasm<Union>.
    UnionOnEthereumMainnet(lc!(Wasm<Union> => Ethereum<Mainnet>)),

    /// The 08-wasm client tracking the state of Ethereum<Minimal>.
    EthereumMinimalOnUnion(lc!(Ethereum<Minimal> => Wasm<Union>)),
    /// The solidity client on Ethereum<Minimal> tracking the state of Wasm<Union>.
    UnionOnEthereumMinimal(lc!(Wasm<Union> => Ethereum<Minimal>)),

    /// The 08-wasm client tracking the state of Scroll.
    ScrollOnUnion(lc!(Scroll => Wasm<Union>)),
    /// The solidity client on Scroll tracking the state of Wasm<Union>.
    UnionOnScroll(lc!(Wasm<Union> => Scroll)),

    /// The 08-wasm client tracking the state of Arbitrum.
    ArbitrumOnUnion(lc!(Arbitrum => Wasm<Union>)),
    /// The solidity client on Arbitrum tracking the state of Wasm<Union>.
    UnionOnArbitrum(lc!(Wasm<Union> => Arbitrum)),

    /// The 08-wasm client tracking the state of Berachain.
    BerachainOnUnion(lc!(Berachain => Wasm<Union>)),
    /// The solidity client on Berachain tracking the state of Wasm<Union>.
    UnionOnBerachain(lc!(Wasm<Union> => Berachain)),

    /// The native tendermint client on Union tracking the state of Wasm<Cosmos>.
    WasmCosmosOnUnion(lc!(Wasm<Cosmos> => Union)),
    /// The 08-wasm client on Cosmos tracking the state of Union.
    UnionOnWasmCosmos(lc!(Union => Wasm<Cosmos>)),

    /// The native tendermint client on Union tracking the state of Cosmos.
    CosmosOnUnion(lc!(Cosmos => Union)),
    /// The native cometbls client on Cosmos tracking the state of Union.
    UnionOnCosmos(lc!(Union => Cosmos)),

    /// The 08-wasm client tracking the state of Cosmos.
    CosmosOnCosmos(lc!(Cosmos => Cosmos)),
}

impl<T: AnyLightClient> AnyLightClientIdentified<T> {
    fn chain_id(&self) -> String {
        let i = self;

        any_lc! {
            |i| i.chain_id.to_string()
        }
    }
}

#[derive(Serialize, Deserialize, enumorph::Enumorph)]
#[serde(
    bound(
        serialize = "
            Inner<Wasm<Union>, Ethereum<Mainnet>, lc!(Ethereum<Mainnet> => Wasm<Union>)>: Serialize,
            Inner<Ethereum<Mainnet>, Wasm<Union>, lc!(Wasm<Union> => Ethereum<Mainnet>)>: Serialize,
            Inner<Wasm<Union>, Ethereum<Minimal>, lc!(Ethereum<Minimal> => Wasm<Union>)>: Serialize,

            Inner<Ethereum<Minimal>, Wasm<Union>, lc!(Wasm<Union> => Ethereum<Minimal>)>: Serialize,

            Inner<Wasm<Union>, Scroll, lc!(Scroll => Wasm<Union>)>: Serialize,
            Inner<Scroll, Wasm<Union>, lc!(Wasm<Union> => Scroll)>: Serialize,

            Inner<Wasm<Union>, Arbitrum, lc!(Arbitrum => Wasm<Union>)>: Serialize,
            Inner<Arbitrum, Wasm<Union>, lc!(Wasm<Union> => Arbitrum)>: Serialize,

            Inner<Wasm<Union>, Berachain, lc!(Berachain => Wasm<Union>)>: Serialize,
            Inner<Berachain, Wasm<Union>, lc!(Wasm<Union> => Berachain)>: Serialize,
            Inner<Union, Wasm<Cosmos>, lc!(Wasm<Cosmos> => Union)>: Serialize,
            Inner<Wasm<Cosmos>, Union, lc!(Union => Wasm<Cosmos>)>: Serialize,

            Inner<Union, Cosmos, lc!(Cosmos => Union)>: Serialize,
            Inner<Cosmos, Union, lc!(Union => Cosmos)>: Serialize,

            Inner<Cosmos, Cosmos, lc!(Cosmos => Cosmos)>: Serialize,
        ",
        deserialize = "
            Inner<Wasm<Union>, Ethereum<Mainnet>, lc!(Ethereum<Mainnet> => Wasm<Union>)>: Deserialize<'de>,
            Inner<Ethereum<Mainnet>, Wasm<Union>, lc!(Wasm<Union> => Ethereum<Mainnet>)>: Deserialize<'de>,
            Inner<Wasm<Union>, Ethereum<Minimal>, lc!(Ethereum<Minimal> => Wasm<Union>)>: Deserialize<'de>,

            Inner<Ethereum<Minimal>, Wasm<Union>, lc!(Wasm<Union> => Ethereum<Minimal>)>: Deserialize<'de>,

            Inner<Wasm<Union>, Scroll, lc!(Scroll => Wasm<Union>)>: Deserialize<'de>,
            Inner<Scroll, Wasm<Union>, lc!(Wasm<Union> => Scroll)>: Deserialize<'de>,

            Inner<Wasm<Union>, Arbitrum, lc!(Arbitrum => Wasm<Union>)>: Deserialize<'de>,
            Inner<Arbitrum, Wasm<Union>, lc!(Wasm<Union> => Arbitrum)>: Deserialize<'de>,

            Inner<Wasm<Union>, Berachain, lc!(Berachain => Wasm<Union>)>: Deserialize<'de>,
            Inner<Berachain, Wasm<Union>, lc!(Wasm<Union> => Berachain)>: Deserialize<'de>,
            Inner<Union, Wasm<Cosmos>, lc!(Wasm<Cosmos> => Union)>: Deserialize<'de>,
            Inner<Wasm<Cosmos>, Union, lc!(Union => Wasm<Cosmos>)>: Deserialize<'de>,

            Inner<Union, Cosmos, lc!(Cosmos => Union)>: Deserialize<'de>,
            Inner<Cosmos, Union, lc!(Union => Cosmos)>: Deserialize<'de>,

            Inner<Cosmos, Cosmos, lc!(Cosmos => Cosmos)>: Deserialize<'de>,
        "
    ),
    untagged,
    deny_unknown_fields
)]
#[allow(clippy::large_enum_variant)]
enum AnyLightClientIdentifiedSerde<T: AnyLightClient> {
    EthereumMainnetOnUnion(
        Inner<Wasm<Union>, Ethereum<Mainnet>, lc!(Ethereum<Mainnet> => Wasm<Union>)>,
    ),
    UnionOnEthereumMainnet(
        Inner<Ethereum<Mainnet>, Wasm<Union>, lc!(Wasm<Union> => Ethereum<Mainnet>)>,
    ),

    EthereumMinimalOnUnion(
        Inner<Wasm<Union>, Ethereum<Minimal>, lc!(Ethereum<Minimal> => Wasm<Union>)>,
    ),
    UnionOnEthereumMinimal(
        Inner<Ethereum<Minimal>, Wasm<Union>, lc!(Wasm<Union> => Ethereum<Minimal>)>,
    ),

    ScrollOnUnion(Inner<Wasm<Union>, Scroll, lc!(Scroll => Wasm<Union>)>),
    UnionOnScroll(Inner<Scroll, Wasm<Union>, lc!(Wasm<Union> => Scroll)>),

    ArbitrumOnUnion(Inner<Wasm<Union>, Arbitrum, lc!(Arbitrum => Wasm<Union>)>),
    UnionOnArbitrum(Inner<Arbitrum, Wasm<Union>, lc!(Wasm<Union> => Arbitrum)>),

    BerachainOnUnion(Inner<Wasm<Union>, Berachain, lc!(Berachain => Wasm<Union>)>),
    UnionOnBerachain(Inner<Berachain, Wasm<Union>, lc!(Wasm<Union> => Berachain)>),
    WasmCosmosOnUnion(Inner<Union, Wasm<Cosmos>, lc!(Wasm<Cosmos> => Union)>),
    UnionOnWasmCosmos(Inner<Wasm<Cosmos>, Union, lc!(Union => Wasm<Cosmos>)>),

    CosmosOnUnion(Inner<Union, Cosmos, lc!(Cosmos => Union)>),
    UnionOnCosmos(Inner<Cosmos, Union, lc!(Union => Cosmos)>),

    CosmosOnCosmos(Inner<Cosmos, Cosmos, lc!(Cosmos => Cosmos)>),
}

impl<T: AnyLightClient> From<AnyLightClientIdentified<T>> for AnyLightClientIdentifiedSerde<T> {
    fn from(value: AnyLightClientIdentified<T>) -> Self {
        any_lc! {
            |value| Inner::new(value).into()
        }
    }
}

impl<T: AnyLightClient> From<AnyLightClientIdentifiedSerde<T>> for AnyLightClientIdentified<T> {
    fn from(value: AnyLightClientIdentifiedSerde<T>) -> Self {
        match value {
            AnyLightClientIdentifiedSerde::EthereumMainnetOnUnion(t) => {
                Self::EthereumMainnetOnUnion(t.inner)
            }
            AnyLightClientIdentifiedSerde::UnionOnEthereumMainnet(t) => {
                Self::UnionOnEthereumMainnet(t.inner)
            }

            AnyLightClientIdentifiedSerde::EthereumMinimalOnUnion(t) => {
                Self::EthereumMinimalOnUnion(t.inner)
            }
            AnyLightClientIdentifiedSerde::UnionOnEthereumMinimal(t) => {
                Self::UnionOnEthereumMinimal(t.inner)
            }

            AnyLightClientIdentifiedSerde::ScrollOnUnion(t) => Self::ScrollOnUnion(t.inner),
            AnyLightClientIdentifiedSerde::UnionOnScroll(t) => Self::UnionOnScroll(t.inner),

            AnyLightClientIdentifiedSerde::ArbitrumOnUnion(t) => Self::ArbitrumOnUnion(t.inner),
            AnyLightClientIdentifiedSerde::UnionOnArbitrum(t) => Self::UnionOnArbitrum(t.inner),

            AnyLightClientIdentifiedSerde::BerachainOnUnion(t) => Self::BerachainOnUnion(t.inner),
            AnyLightClientIdentifiedSerde::UnionOnBerachain(t) => Self::UnionOnBerachain(t.inner),
            AnyLightClientIdentifiedSerde::WasmCosmosOnUnion(t) => Self::WasmCosmosOnUnion(t.inner),
            AnyLightClientIdentifiedSerde::UnionOnWasmCosmos(t) => Self::UnionOnWasmCosmos(t.inner),

            AnyLightClientIdentifiedSerde::CosmosOnCosmos(t) => Self::CosmosOnCosmos(t.inner),
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

#[derive(macros::Debug, Serialize, Deserialize)]
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
    arbitrary(bound = "T: arbitrary::Arbitrary<'arbitrary>")
)]
pub struct Identified<Hc: Chain, Tr, T> {
    pub chain_id: ChainIdOf<Hc>,
    pub t: T,
    #[serde(skip)]
    #[debug(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> Tr>,
}

impl<Hc: Chain, Tr, Data: PartialEq> PartialEq for Identified<Hc, Tr, Data> {
    fn eq(&self, other: &Self) -> bool {
        self.chain_id == other.chain_id && self.t == other.t
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

pub fn id<Hc: Chain, Tr, T>(chain_id: ChainIdOf<Hc>, t: T) -> Identified<Hc, Tr, T> {
    Identified {
        chain_id,
        t,
        __marker: PhantomData,
    }
}

pub trait DoAggregate: Sized + Debug + Clone + PartialEq {
    fn do_aggregate(_: Self, _: VecDeque<AnyLightClientIdentified<AnyData>>) -> Op<RelayMessage>;
}

impl<Hc: Chain, Tr> DoAggregate for Identified<Hc, Tr, Never> {
    fn do_aggregate(s: Self, _: VecDeque<AnyLightClientIdentified<AnyData>>) -> Op<RelayMessage> {
        match s.t {}
    }
}

pub trait DoFetchState<Hc: ChainExt, Tr: ChainExt>: ChainExt {
    type QueryUnfinalizedTrustedClientStateError: Debug
        + Clone
        + PartialEq
        + std::error::Error
        + Send
        + Sync;

    fn state(hc: &Hc, at: Hc::Height, path: PathOf<Hc, Tr>) -> Op<RelayMessage>;

    // SEE: <https://github.com/unionlabs/union/issues/1813>
    fn query_unfinalized_trusted_client_state(
        hc: &Hc,
        client_id: Hc::ClientId,
    ) -> impl Future<
        Output = Result<Hc::StoredClientState<Tr>, Self::QueryUnfinalizedTrustedClientStateError>,
    > + '_;
}

pub trait DoFetchProof<Hc: ChainExt, Tr: ChainExt>: ChainExt {
    fn proof(hc: &Hc, at: HeightOf<Hc>, path: PathOf<Hc, Tr>) -> Op<RelayMessage>;
}

pub trait DoFetchUpdateHeaders<Hc: ChainExt, Tr: ChainExt>: ChainExt {
    fn fetch_update_headers(hc: &Hc, update_info: FetchUpdateHeaders<Hc, Tr>) -> Op<RelayMessage>;
}

pub trait DoMsg<Hc: ChainExt, Tr: ChainExt>: ChainExt {
    fn msg(
        &self,
        msg: Effect<Hc, Tr>,
    ) -> impl Future<Output = Result<Op<RelayMessage>, Self::MsgError>> + Send + '_;
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(
    bound(serialize = "T: Serialize", deserialize = "T: for<'d> Deserialize<'d>"),
    deny_unknown_fields
)]
struct Inner<Hc: Chain, Tr: Chain, T> {
    #[serde(rename = "@host_chain", with = "::unionlabs::traits::from_str_exact")]
    host_chain: Hc::ChainType,
    #[serde(rename = "@tracking", with = "::unionlabs::traits::from_str_exact")]
    tracking: Tr::ChainType,
    #[serde(rename = "@value")]
    inner: T,
}

impl<Hc: Chain, Tr: Chain, T> Inner<Hc, Tr, T> {
    fn new(s: T) -> Inner<Hc, Tr, T> {
        Self {
            host_chain: Hc::ChainType::default(),
            tracking: Tr::ChainType::default(),
            inner: s,
        }
    }
}

macro_rules! any_lc {
    (|$msg:ident| $expr:expr) => {
        match $msg {
            AnyLightClientIdentified::EthereumMainnetOnUnion($msg) => {
                #[allow(dead_code)]
                type Hc = chain_utils::wasm::Wasm<chain_utils::union::Union>;
                #[allow(dead_code)]
                type Tr = chain_utils::ethereum::Ethereum<unionlabs::ethereum::config::Mainnet>;

                $expr
            }
            AnyLightClientIdentified::UnionOnEthereumMainnet($msg) => {
                #[allow(dead_code)]
                type Hc = chain_utils::ethereum::Ethereum<unionlabs::ethereum::config::Mainnet>;
                #[allow(dead_code)]
                type Tr = chain_utils::wasm::Wasm<chain_utils::union::Union>;

                $expr
            }

            AnyLightClientIdentified::EthereumMinimalOnUnion($msg) => {
                #[allow(dead_code)]
                type Hc = chain_utils::wasm::Wasm<chain_utils::union::Union>;
                #[allow(dead_code)]
                type Tr = chain_utils::ethereum::Ethereum<unionlabs::ethereum::config::Minimal>;

                $expr
            }
            AnyLightClientIdentified::UnionOnEthereumMinimal($msg) => {
                #[allow(dead_code)]
                type Hc = chain_utils::ethereum::Ethereum<unionlabs::ethereum::config::Minimal>;
                #[allow(dead_code)]
                type Tr = chain_utils::wasm::Wasm<chain_utils::union::Union>;

                $expr
            }

            AnyLightClientIdentified::ScrollOnUnion($msg) => {
                #[allow(dead_code)]
                type Hc = chain_utils::wasm::Wasm<chain_utils::union::Union>;
                #[allow(dead_code)]
                type Tr = chain_utils::scroll::Scroll;

                $expr
            }
            AnyLightClientIdentified::UnionOnScroll($msg) => {
                #[allow(dead_code)]
                type Hc = chain_utils::scroll::Scroll;
                #[allow(dead_code)]
                type Tr = chain_utils::wasm::Wasm<chain_utils::union::Union>;

                $expr
            }

            AnyLightClientIdentified::ArbitrumOnUnion($msg) => {
                #[allow(dead_code)]
                type Hc = chain_utils::wasm::Wasm<chain_utils::union::Union>;
                #[allow(dead_code)]
                type Tr = chain_utils::arbitrum::Arbitrum;

                $expr
            }
            AnyLightClientIdentified::UnionOnArbitrum($msg) => {
                #[allow(dead_code)]
                type Hc = chain_utils::arbitrum::Arbitrum;
                #[allow(dead_code)]
                type Tr = chain_utils::wasm::Wasm<chain_utils::union::Union>;

                $expr
            }

            AnyLightClientIdentified::BerachainOnUnion($msg) => {
                #[allow(dead_code)]
                type Hc = chain_utils::wasm::Wasm<chain_utils::union::Union>;
                #[allow(dead_code)]
                type Tr = chain_utils::berachain::Berachain;

                $expr
            }
            AnyLightClientIdentified::UnionOnBerachain($msg) => {
                #[allow(dead_code)]
                type Hc = chain_utils::berachain::Berachain;
                #[allow(dead_code)]
                type Tr = chain_utils::wasm::Wasm<chain_utils::union::Union>;

                $expr
            }
            AnyLightClientIdentified::WasmCosmosOnUnion($msg) => {
                #[allow(dead_code)]
                type Hc = chain_utils::union::Union;
                #[allow(dead_code)]
                type Tr = chain_utils::wasm::Wasm<chain_utils::cosmos::Cosmos>;

                $expr
            }
            AnyLightClientIdentified::UnionOnWasmCosmos($msg) => {
                #[allow(dead_code)]
                type Hc = chain_utils::wasm::Wasm<chain_utils::cosmos::Cosmos>;
                #[allow(dead_code)]
                type Tr = chain_utils::union::Union;

                $expr
            }

            AnyLightClientIdentified::CosmosOnUnion($msg) => {
                #[allow(dead_code)]
                type Hc = chain_utils::union::Union;
                #[allow(dead_code)]
                type Tr = chain_utils::cosmos::Cosmos;

                $expr
            }
            AnyLightClientIdentified::UnionOnCosmos($msg) => {
                #[allow(dead_code)]
                type Hc = chain_utils::cosmos::Cosmos;
                #[allow(dead_code)]
                type Tr = chain_utils::union::Union;

                $expr
            }

            AnyLightClientIdentified::CosmosOnCosmos($msg) => {
                #[allow(dead_code)]
                type Hc = chain_utils::cosmos::Cosmos;
                #[allow(dead_code)]
                type Tr = chain_utils::cosmos::Cosmos;

                $expr
            }
        }
    };
}
pub(crate) use any_lc;

mod sanity_checks {
    use chain_utils::{cosmos::Cosmos, ethereum::Ethereum, union::Union, wasm::Wasm};
    use queue_msg::aggregation::UseAggregate;
    use static_assertions::assert_impl_all;
    use unionlabs::ethereum::config::Mainnet;

    use crate::{
        aggregate::AggregateMsgConnectionOpenTry, chain::union::UnionFetch, fetch::DoFetch,
        DoFetchState, RelayMessage,
    };

    assert_impl_all!(Wasm<Cosmos>: DoFetchState<Wasm<Cosmos>, Union>);

    assert_impl_all!(identified!(AggregateMsgConnectionOpenTry<Wasm<Cosmos>, Union>): UseAggregate<RelayMessage>);

    assert_impl_all!(UnionFetch<Wasm<Union>, Ethereum<Mainnet>>: DoFetch<Wasm<Union>>);
}
