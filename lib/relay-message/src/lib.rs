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
    type Event = Identified<Event>;
    type Data = Identified<Data>;
    type Fetch = Identified<Fetch>;
    type Effect = Identified<Effect>;
    type Wait = Identified<Wait>;
    type Aggregate = Identified<Aggregate>;

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
                impl<Hc: ChainExt, Tr: ChainExt> From<Identified<$VariantInner>>
                    for AnyLightClientIdentified<$Any>
                where
                    $VariantInner: Into<$Enum<Hc, Tr>>,
                    AnyLightClientIdentified<$Any>: From<Identified<$Enum<Hc, Tr>>>,
                {
                    fn from(
                        Identified {
                            chain_id,
                            t,
                            __marker: _,
                        }: Identified<$VariantInner>,
                    ) -> Self {
                        Self::from(crate::id(
                            chain_id,
                            <$Enum<Hc, Tr>>::from(t),
                        ))
                    }
                }

                impl<Hc: ChainExt, Tr: ChainExt>
                    TryFrom<AnyLightClientIdentified<$Any>> for Identified<$VariantInner>
                where
                    Identified<$Enum<Hc, Tr>>: TryFrom<AnyLightClientIdentified<$Any>, Error = AnyLightClientIdentified<$Any>>
                    + Into<AnyLightClientIdentified<$Any>>,
                {
                    type Error = AnyLightClientIdentified<$Any>;

                    fn try_from(value: AnyLightClientIdentified<$Any>) -> Result<Self, Self::Error> {
                        let Identified {
                            chain_id,
                            t,
                            __marker: _,
                        } = <Identified<$Enum<Hc, Tr>>>::try_from(value)?;

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
    // arbitrary(bound = "T: arbitrary::Arbitrary<'arbitrary>")
)]
pub struct Identified<T> {
    pub chain_id: ChainIdOf<Hc>,
    pub t: T,
}

impl<Hc: Chain, Tr, Data: PartialEq> PartialEq for Identified<Data> {
    fn eq(&self, other: &Self) -> bool {
        self.chain_id == other.chain_id && self.t == other.t
    }
}

impl<Hc: Chain, Tr, Data: Clone> Clone for Identified<Data> {
    fn clone(&self) -> Self {
        Self {
            chain_id: self.chain_id.clone(),
            t: self.t.clone(),
        }
    }
}

pub fn id<Hc: Chain, Tr, T>(chain_id: ChainIdOf<Hc>, t: T) -> Identified<T> {
    Identified { chain_id, t }
}

pub trait DoAggregate: Sized + Debug + Clone + PartialEq {
    fn do_aggregate(_: Self, _: VecDeque<AnyLightClientIdentified<AnyData>>) -> Op<RelayMessage>;
}

impl DoAggregate for Identified<Never> {
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
