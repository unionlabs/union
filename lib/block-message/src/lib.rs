#![feature(trait_alias)]

use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
};

use chain_utils::{cosmos::Cosmos, ethereum::Ethereum, scroll::Scroll, union::Union, Chains};
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use queue_msg::{QueueMsg, QueueMsgTypes, QueueMsgTypesTraits};
use serde::{Deserialize, Serialize};
use unionlabs::{
    ethereum::config::{Mainnet, Minimal},
    never::Never,
    traits::{Chain, ChainIdOf},
    MaybeArbitrary,
};

use crate::{aggregate::AnyAggregate, data::AnyData, fetch::AnyFetch, wait::AnyWait};

pub mod aggregate;
pub mod data;
pub mod fetch;
pub mod wait;

pub mod chain_impls;

pub trait ChainExt: Chain {
    type Data: QueueMsgTypesTraits;
    type Fetch: QueueMsgTypesTraits;
    type Aggregate: QueueMsgTypesTraits;
}

pub struct BlockPollingTypes;

impl QueueMsgTypes for BlockPollingTypes {
    type Event = Never;
    type Data = AnyChainIdentified<AnyData>;
    type Fetch = AnyChainIdentified<AnyFetch>;
    type Effect = Never;
    type Wait = AnyChainIdentified<AnyWait>;
    type Aggregate = AnyChainIdentified<AnyAggregate>;

    type Store = Chains;
}

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
    arbitrary(bound = "T: AnyChain")
)]
#[serde(
    tag = "@type",
    content = "@value",
    rename_all = "kebab-case",
    bound(serialize = "", deserialize = "")
)]
pub enum AnyChainIdentified<T: AnyChain> {
    Union(Identified<Union, InnerOf<T, Union>>),
    Cosmos(Identified<Cosmos, InnerOf<T, Cosmos>>),
    EthMainnet(Identified<Ethereum<Mainnet>, InnerOf<T, Ethereum<Mainnet>>>),
    EthMinimal(Identified<Ethereum<Minimal>, InnerOf<T, Ethereum<Minimal>>>),
    Scroll(Identified<Scroll, InnerOf<T, Scroll>>),
}

pub trait AnyChain {
    type Inner<C: ChainExt>: Debug
        + Display
        + Clone
        + PartialEq
        + Serialize
        + for<'de> Deserialize<'de>
        + MaybeArbitrary;
}

pub type InnerOf<T, C> = <T as AnyChain>::Inner<C>;

#[derive(DebugNoBound, PartialEqNoBound, CloneNoBound, Serialize, Deserialize)]
#[serde(
    bound(
        serialize = "T: ::serde::Serialize",
        deserialize = "T: for<'d> Deserialize<'d>"
    ),
    deny_unknown_fields
)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "")
)]
pub struct Identified<C: Chain, T: Debug + Clone + PartialEq> {
    pub chain_id: ChainIdOf<C>,
    pub t: T,
}

impl<C: Chain, T: Debug + Clone + PartialEq> Identified<C, T> {
    pub fn new(chain_id: ChainIdOf<C>, t: T) -> Self {
        Self { chain_id, t }
    }
}

pub fn id<C: Chain, T: Debug + Clone + PartialEq>(
    chain_id: ChainIdOf<C>,
    t: T,
) -> Identified<C, T> {
    Identified::new(chain_id, t)
}

impl<C: Chain, Data: std::fmt::Display + Debug + Clone + PartialEq> std::fmt::Display
    for Identified<C, Data>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{}: {}}}", self.chain_id, self.t)
    }
}

macro_rules! any_enum {
    (
        $(#[doc = $outer_doc:literal])*
        #[any = $Any:ident]
        $(#[specific = $Specific:ident])?
        pub enum $Enum:ident<C: ChainExt> {
            $(
                $(#[doc = $doc:literal])*
                $(#[serde($untagged:ident)])*
                $Variant:ident(
                    $(#[$variant_inner_meta:meta])*
                    $VariantInner:ty
                ),
            )*
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
            arbitrary(bound = "C: ChainExt")
        )]
        #[serde(
            bound(serialize = "", deserialize = ""),
            tag = "@type",
            content = "@value",
            rename_all = "snake_case"
        )]
        $(#[doc = $outer_doc])*
        #[allow(clippy::large_enum_variant)]
        pub enum $Enum<C: ChainExt> {
            $(
                $(#[doc = $doc])*
                $(#[serde($untagged)])*
                $Variant(
                    $(#[$variant_inner_meta])*
                    $VariantInner
                ),
            )*
        }

        pub enum $Any {}
        impl crate::AnyChain for $Any {
            type Inner<C: ChainExt> = $Enum<C>;
        }

        const _: () = {
            use crate::{AnyChainIdentified, Identified};

            $(
                impl<C: ChainExt> From<Identified<C, $VariantInner>>
                    for AnyChainIdentified<$Any>
                where
                    $VariantInner: Into<$Enum<C>>,
                    AnyChainIdentified<$Any>: From<Identified<C, $Enum<C>>>,
                {
                    fn from(
                        Identified {
                            chain_id,
                            t,
                        }: Identified<C, $VariantInner>,
                    ) -> Self {
                        Self::from(Identified::new(
                            chain_id,
                            <$Enum<C>>::from(t),
                        ))
                    }
                }

                impl<C: ChainExt> TryFrom<AnyChainIdentified<$Any>> for Identified<C, $VariantInner>
                where
                    Identified<C, $Enum<C>>: TryFrom<AnyChainIdentified<$Any>, Error = AnyChainIdentified<$Any>>
                    + Into<AnyChainIdentified<$Any>>,
                {
                    type Error = AnyChainIdentified<$Any>;

                    fn try_from(value: AnyChainIdentified<$Any>) -> Result<Self, Self::Error> {
                        let Identified {
                            chain_id,
                            t,
                        } = <Identified<C, $Enum<C>>>::try_from(value)?;

                        Ok(Identified::new(
                            chain_id.clone(),
                            <$VariantInner>::try_from(t).map_err(|x: $Enum<C>| {
                                Into::<AnyChainIdentified<_>>::into(Identified::new(chain_id, x))
                            })?,
                        ))
                    }
                }
            )*
        };

        $(
            impl<C: ChainExt> $Enum<C> {
                pub fn specific(t: impl Into<C::$Enum>) -> $Enum<C> {
                    $Specific(t.into()).into()
                }
            }
        )?
    };
}
pub(crate) use any_enum;

pub trait IsAggregateData = TryFrom<AnyChainIdentified<AnyData>, Error = AnyChainIdentified<AnyData>>
    + Into<AnyChainIdentified<AnyData>>;

pub trait DoAggregate: Sized + Debug + Clone + PartialEq {
    fn do_aggregate(
        _: Self,
        _: VecDeque<AnyChainIdentified<AnyData>>,
    ) -> QueueMsg<BlockPollingTypes>;
}

macro_rules! any_chain {
    (|$msg:ident| $expr:expr) => {
        match $msg {
            AnyChainIdentified::EthMainnet($msg) => $expr,
            AnyChainIdentified::EthMinimal($msg) => $expr,
            AnyChainIdentified::Union($msg) => $expr,
            AnyChainIdentified::Cosmos($msg) => $expr,
            AnyChainIdentified::Scroll($msg) => $expr,
        }
    };
}
pub(crate) use any_chain;
