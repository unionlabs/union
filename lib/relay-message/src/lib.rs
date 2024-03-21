#![feature(trait_alias)]
#![allow(clippy::type_complexity, async_fn_in_trait)]

use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
    future::Future,
    marker::PhantomData,
};

use chain_utils::{cosmos::Cosmos, ethereum::Ethereum, union::Union, wasm::Wasm, Chains};
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use queue_msg::{seq, QueueMsg, QueueMsgTypes, QueueMsgTypesTraits};
use serde::{Deserialize, Serialize};
use unionlabs::{
    ethereum::config::{Mainnet, Minimal},
    proof::{self},
    traits::{Chain, ChainIdOf, ClientIdOf, HeightOf},
    MaybeArbitrary, MaybeRecoverableError,
};

use crate::{
    aggregate::AnyAggregate,
    data::AnyData,
    event::AnyEvent,
    fetch::{AnyFetch, DoFetch, FetchUpdateHeaders},
    msg::{AnyMsg, Msg},
    wait::AnyWait,
};

pub mod use_aggregate;

pub mod aggregate;
pub mod data;
pub mod event;
pub mod fetch;
pub mod msg;
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
    ) -> impl Future<Output = QueueMsg<RelayerMsgTypes>> + '_
    where
        Self::Fetch<Tr>: DoFetch<Self, Tr>,
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

impl TryFrom<QueueMsg<RelayerMsgTypes>> for AnyLightClientIdentified<AnyData> {
    type Error = QueueMsg<RelayerMsgTypes>;

    fn try_from(value: QueueMsg<RelayerMsgTypes>) -> Result<Self, Self::Error> {
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
    // The 08-wasm client tracking the state of Ethereum<Mainnet>.
    #[display(fmt = "EthereumMainnetOnUnion({}, {})", "_0.chain_id", "_0.t")]
    EthereumMainnetOnUnion(
        Identified<Wasm<Union>, Ethereum<Mainnet>, InnerOf<T, Wasm<Union>, Ethereum<Mainnet>>>,
    ),
    // The solidity client on Ethereum<Mainnet> tracking the state of Wasm<Union>.
    #[display(fmt = "UnionOnEthereumMainnet({}, {})", "_0.chain_id", "_0.t")]
    UnionOnEthereumMainnet(
        Identified<Ethereum<Mainnet>, Wasm<Union>, InnerOf<T, Ethereum<Mainnet>, Wasm<Union>>>,
    ),

    // The 08-wasm client tracking the state of Ethereum<Minimal>.
    #[display(fmt = "EthereumMinimalOnUnion({}, {})", "_0.chain_id", "_0.t")]
    EthereumMinimalOnUnion(
        Identified<Wasm<Union>, Ethereum<Minimal>, InnerOf<T, Wasm<Union>, Ethereum<Minimal>>>,
    ),
    // The solidity client on Ethereum<Minimal> tracking the state of Wasm<Union>.
    #[display(fmt = "UnionOnEthereumMinimal({}, {})", "_0.chain_id", "_0.t")]
    UnionOnEthereumMinimal(
        Identified<Ethereum<Minimal>, Wasm<Union>, InnerOf<T, Ethereum<Minimal>, Wasm<Union>>>,
    ),

    #[display(fmt = "CosmosOnUnion({}, {})", "_0.chain_id", "_0.t")]
    CosmosOnUnion(Identified<Union, Wasm<Cosmos>, InnerOf<T, Union, Wasm<Cosmos>>>),
    #[display(fmt = "UnionOnCosmos({}, {})", "_0.chain_id", "_0.t")]
    UnionOnCosmos(Identified<Wasm<Cosmos>, Union, InnerOf<T, Wasm<Cosmos>, Union>>),
}

#[derive(Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), untagged, deny_unknown_fields)]
#[allow(clippy::large_enum_variant)]
enum AnyLightClientIdentifiedSerde<T: AnyLightClient> {
    EthereumMainnetOnUnion(
        Inner<
            Wasm<Union>,
            Ethereum<Mainnet>,
            Identified<Wasm<Union>, Ethereum<Mainnet>, InnerOf<T, Wasm<Union>, Ethereum<Mainnet>>>,
        >,
    ),
    UnionOnEthereumMainnet(
        Inner<
            Ethereum<Mainnet>,
            Wasm<Union>,
            Identified<Ethereum<Mainnet>, Wasm<Union>, InnerOf<T, Ethereum<Mainnet>, Wasm<Union>>>,
        >,
    ),

    EthereumMinimalOnUnion(
        Inner<
            Wasm<Union>,
            Ethereum<Minimal>,
            Identified<Wasm<Union>, Ethereum<Minimal>, InnerOf<T, Wasm<Union>, Ethereum<Minimal>>>,
        >,
    ),
    UnionOnEthereumMinimal(
        Inner<
            Ethereum<Minimal>,
            Wasm<Union>,
            Identified<Ethereum<Minimal>, Wasm<Union>, InnerOf<T, Ethereum<Minimal>, Wasm<Union>>>,
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

impl<Hc: Chain, Tr, Data: std::error::Error> std::error::Error for Identified<Hc, Tr, Data> {}

impl<Hc: Chain, Tr, Data: std::fmt::Display> std::fmt::Display for Identified<Hc, Tr, Data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{}: {}}}", self.chain_id, self.t)
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
    ) -> QueueMsg<RelayerMsgTypes>;
}

pub trait DoFetchState<Hc: ChainExt, Tr: ChainExt>: ChainExt {
    fn state(hc: &Hc, at: Hc::Height, path: PathOf<Hc, Tr>) -> QueueMsg<RelayerMsgTypes>;

    #[deprecated = "will be removed in favor of an aggregation with state"]
    fn query_client_state(
        hc: &Hc,
        client_id: Hc::ClientId,
        height: Hc::Height,
    ) -> impl Future<Output = Hc::StoredClientState<Tr>> + '_;
}

pub trait DoFetchProof<Hc: ChainExt, Tr: ChainExt>: ChainExt {
    fn proof(hc: &Hc, at: HeightOf<Hc>, path: PathOf<Hc, Tr>) -> QueueMsg<RelayerMsgTypes>;
}

pub trait DoFetchUpdateHeaders<Hc: ChainExt, Tr: ChainExt>: ChainExt {
    fn fetch_update_headers(
        hc: &Hc,
        update_info: FetchUpdateHeaders<Hc, Tr>,
    ) -> QueueMsg<RelayerMsgTypes>;
}

pub trait DoMsg<Hc: ChainExt, Tr: ChainExt>: ChainExt {
    fn msg(&self, msg: Msg<Hc, Tr>) -> impl Future<Output = Result<(), Self::MsgError>> + '_;
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

mod sanity_checks {
    use chain_utils::{cosmos::Cosmos, ethereum::Ethereum, union::Union, wasm::Wasm};
    use queue_msg::aggregation::UseAggregate;
    use static_assertions::assert_impl_all;
    use unionlabs::ethereum::config::Mainnet;

    use crate::{
        aggregate::AggregateConnectionOpenTry, chain_impls::union::UnionFetch, fetch::DoFetch,
        DoFetchState, RelayerMsgTypes,
    };

    assert_impl_all!(Wasm<Cosmos>: DoFetchState<Wasm<Cosmos>, Union>);

    assert_impl_all!(identified!(AggregateConnectionOpenTry<Wasm<Cosmos>, Union>): UseAggregate<RelayerMsgTypes>);

    assert_impl_all!(UnionFetch<Wasm<Union>, Ethereum<Mainnet>>: DoFetch<Wasm<Union>, Ethereum<Mainnet>>);
}
