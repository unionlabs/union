#![feature(trait_alias)]
#![allow(clippy::type_complexity, async_fn_in_trait)]

use std::{collections::VecDeque, fmt::Debug, future::Future, marker::PhantomData};

use chain_utils::{
    cosmos::Cosmos, ethereum::Ethereum, scroll::Scroll, union::Union, wasm::Wasm, Chains,
};
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use queue_msg::{seq, QueueMsg, QueueMsgTypes, QueueMsgTypesTraits};
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
    fetch::{AnyFetch, DoFetch, FetchUpdateHeaders},
    wait::AnyWait,
};

pub mod use_aggregate;

pub mod aggregate;
pub mod data;
pub mod effect;
pub mod event;
pub mod fetch;
pub mod wait;

pub mod chain_impls;

pub trait ChainExt: Chain {
    type Data<Tr: ChainExt>: QueueMsgTypesTraits;
    type Fetch<Tr: ChainExt>: QueueMsgTypesTraits;
    type Aggregate<Tr: ChainExt>: QueueMsgTypesTraits;

    /// Error type for [`Self::msg`].
    type MsgError: Debug + MaybeRecoverableError;

    /// The config required to construct this light client.
    type Config: Debug + Clone + PartialEq + Serialize + for<'de> Deserialize<'de> + MaybeArbitrary;

    fn do_fetch<Tr: ChainExt>(
        &self,
        msg: Self::Fetch<Tr>,
    ) -> impl Future<Output = QueueMsg<RelayMessageTypes>> + '_
    where
        Self::Fetch<Tr>: DoFetch<Self>,
    {
        DoFetch::do_fetch(self, msg)
    }
}

pub struct RelayMessageTypes;

impl QueueMsgTypes for RelayMessageTypes {
    type Event = AnyLightClientIdentified<AnyEvent>;
    type Data = AnyLightClientIdentified<AnyData>;
    type Fetch = AnyLightClientIdentified<AnyFetch>;
    type Effect = AnyLightClientIdentified<AnyEffect>;
    type Wait = AnyLightClientIdentified<AnyWait>;
    type Aggregate = AnyLightClientIdentified<AnyAggregate>;

    type Store = Chains;
}

impl TryFrom<QueueMsg<RelayMessageTypes>> for AnyLightClientIdentified<AnyData> {
    type Error = QueueMsg<RelayMessageTypes>;

    fn try_from(value: QueueMsg<RelayMessageTypes>) -> Result<Self, Self::Error> {
        match value {
            QueueMsg::Data(data) => Ok(data),
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
        };

        $(
            impl<Hc: ChainExt, Tr: ChainExt> $Enum<Hc, Tr> {
                pub fn specific(t: impl Into<Hc::$Enum<Tr>>) -> $Enum<Hc, Tr> {
                    $Specific(t.into()).into()
                }
            }
        )?
    };
}
pub(crate) use any_enum;

pub type PathOf<Hc, Tr> = ics24::Path<ClientIdOf<Hc>, HeightOf<Tr>>;

pub trait AnyLightClient {
    type Inner<Hc: ChainExt, Tr: ChainExt>: Debug
        + Clone
        + PartialEq
        + Serialize
        + for<'de> Deserialize<'de>
        + MaybeArbitrary;
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
    bound(serialize = "", deserialize = "")
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

    /// The 08-wasm client tracking the state of Cosmos.
    CosmosOnUnion(lc!(Wasm<Cosmos> => Union)),
    /// The solidity client on Cosmos tracking the state of Wasm<Union>.
    UnionOnCosmos(lc!(Union => Wasm<Cosmos>)),
}

#[derive(Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), untagged, deny_unknown_fields)]
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

    CosmosOnUnion(Inner<Union, Wasm<Cosmos>, lc!(Wasm<Cosmos> => Union)>),
    UnionOnCosmos(Inner<Wasm<Cosmos>, Union, lc!(Union => Wasm<Cosmos>)>),
}

impl<T: AnyLightClient> From<AnyLightClientIdentified<T>> for AnyLightClientIdentifiedSerde<T> {
    fn from(value: AnyLightClientIdentified<T>) -> Self {
        match value {
            AnyLightClientIdentified::EthereumMainnetOnUnion(t) => {
                Self::EthereumMainnetOnUnion(Inner::new(t))
            }
            AnyLightClientIdentified::UnionOnEthereumMainnet(t) => {
                Self::UnionOnEthereumMainnet(Inner::new(t))
            }
            AnyLightClientIdentified::EthereumMinimalOnUnion(t) => {
                Self::EthereumMinimalOnUnion(Inner::new(t))
            }
            AnyLightClientIdentified::UnionOnEthereumMinimal(t) => {
                Self::UnionOnEthereumMinimal(Inner::new(t))
            }
            AnyLightClientIdentified::ScrollOnUnion(t) => Self::ScrollOnUnion(Inner::new(t)),
            AnyLightClientIdentified::UnionOnScroll(t) => Self::UnionOnScroll(Inner::new(t)),
            AnyLightClientIdentified::CosmosOnUnion(t) => Self::CosmosOnUnion(Inner::new(t)),
            AnyLightClientIdentified::UnionOnCosmos(t) => Self::UnionOnCosmos(Inner::new(t)),
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
    fn do_aggregate(
        _: Self,
        _: VecDeque<AnyLightClientIdentified<AnyData>>,
    ) -> QueueMsg<RelayMessageTypes>;
}

impl<Hc: Chain, Tr> DoAggregate for Identified<Hc, Tr, Never> {
    fn do_aggregate(
        s: Self,
        _: VecDeque<AnyLightClientIdentified<AnyData>>,
    ) -> QueueMsg<RelayMessageTypes> {
        match s.t {}
    }
}

pub trait DoFetchState<Hc: ChainExt, Tr: ChainExt>: ChainExt {
    fn state(hc: &Hc, at: Hc::Height, path: PathOf<Hc, Tr>) -> QueueMsg<RelayMessageTypes>;

    #[deprecated = "will be removed in favor of an aggregation with state"]
    fn query_client_state(
        hc: &Hc,
        client_id: Hc::ClientId,
        height: Hc::Height,
    ) -> impl Future<Output = Hc::StoredClientState<Tr>> + '_;
}

pub trait DoFetchProof<Hc: ChainExt, Tr: ChainExt>: ChainExt {
    fn proof(hc: &Hc, at: HeightOf<Hc>, path: PathOf<Hc, Tr>) -> QueueMsg<RelayMessageTypes>;
}

pub trait DoFetchUpdateHeaders<Hc: ChainExt, Tr: ChainExt>: ChainExt {
    fn fetch_update_headers(
        hc: &Hc,
        update_info: FetchUpdateHeaders<Hc, Tr>,
    ) -> QueueMsg<RelayMessageTypes>;
}

pub trait DoMsg<Hc: ChainExt, Tr: ChainExt>: ChainExt {
    fn msg(&self, msg: Effect<Hc, Tr>) -> impl Future<Output = Result<(), Self::MsgError>> + '_;
}

#[derive(Serialize, Deserialize)]
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

            AnyLightClientIdentified::CosmosOnUnion($msg) => {
                #[allow(dead_code)]
                type Hc = chain_utils::union::Union;
                #[allow(dead_code)]
                type Tr = chain_utils::wasm::Wasm<chain_utils::cosmos::Cosmos>;

                $expr
            }
            AnyLightClientIdentified::UnionOnCosmos($msg) => {
                #[allow(dead_code)]
                type Hc = chain_utils::wasm::Wasm<chain_utils::cosmos::Cosmos>;
                #[allow(dead_code)]
                type Tr = chain_utils::union::Union;

                $expr
            }
        }
    };
}
pub(crate) use any_lc;
