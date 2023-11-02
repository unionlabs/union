#![feature(return_position_impl_trait_in_trait, trait_alias)]
#![allow(clippy::type_complexity)]

use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
    future::Future,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use ::lightclient::{
    cometbls::{CometblsMainnet, CometblsMinimal},
    ethereum::{EthereumMainnet, EthereumMinimal},
};
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use futures::{future::BoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use unionlabs::{
    proof::{
        AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, IbcPath,
    },
    traits::{Chain, ChainIdOf, ChainOf, LightClientBase},
    MaybeRecoverableError,
};

use crate::{
    aggregate::{Aggregate, AnyAggregate, LightClientSpecificAggregate},
    data::{AnyData, Data, LightClientSpecificData},
    event::{AnyEvent, Event},
    fetch::{AnyFetch, Fetch, FetchStateProof, FetchUpdateHeaders, LightClientSpecificFetch},
    msg::{AnyMsg, Msg},
    wait::{AnyWait, Wait},
};

pub mod use_aggregate;

pub mod aggregate;
pub mod data;
pub mod event;
pub mod fetch;
#[allow(clippy::module_inception)] // fight me clippy
pub mod msg;
pub mod wait;

// TODO: Rename this module to something better, `lightclient` clashes with the workspace crate (could also rename the crate)
pub mod lightclient_impls;

pub trait LightClient: LightClientBase<Counterparty = Self::BaseCounterparty> {
    // https://github.com/rust-lang/rust/issues/20671
    type BaseCounterparty: LightClient<BaseCounterparty = Self, Counterparty = Self>;

    type Data: Debug
        + Display
        + Clone
        + PartialEq
        + Serialize
        + for<'de> Deserialize<'de>
        + Into<LightClientSpecificData<Self>>;
    type Fetch: Debug
        + Display
        + Clone
        + PartialEq
        + Serialize
        + for<'de> Deserialize<'de>
        + Into<LightClientSpecificFetch<Self>>;
    type Aggregate: Debug
        + Display
        + Clone
        + PartialEq
        + Serialize
        + for<'de> Deserialize<'de>
        + Into<LightClientSpecificAggregate<Self>>
        + DoAggregate<Self>;

    /// Error type for [`Self::msg`].
    type MsgError: MaybeRecoverableError;

    fn proof(&self, msg: FetchStateProof<Self>) -> RelayerMsg;

    fn msg(&self, msg: Msg<Self>) -> impl Future<Output = Result<(), Self::MsgError>> + '_;

    fn do_fetch(&self, msg: Self::Fetch) -> impl Future<Output = Vec<RelayerMsg>> + '_;

    // Should (eventually) resolve to UpdateClientData
    fn generate_counterparty_updates(
        &self,
        update_info: FetchUpdateHeaders<Self>,
    ) -> Vec<RelayerMsg>;
}

pub trait IntoRelayerMsg {
    fn into_relayer_msg(self) -> RelayerMsg;
}

pub trait TryFromRelayerMsg: Sized {
    fn try_from_relayer_msg(msg: RelayerMsg) -> Result<Self, RelayerMsg>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeferPoint {
    Absolute,
    Relative,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum RelayerMsg {
    Lc(AnyLightClientIdentified<AnyLcMsg>),
    DeferUntil {
        point: DeferPoint,
        seconds: u64,
    },
    Repeat {
        times: u64,
        msg: Box<RelayerMsg>,
    },
    Timeout {
        timeout_timestamp: u64,
        msg: Box<RelayerMsg>,
    },
    Sequence(VecDeque<RelayerMsg>),
    Retry(u8, Box<RelayerMsg>),
    Aggregate {
        /// Messages that are expected to resolve to [`Data`].
        queue: VecDeque<RelayerMsg>,
        /// The resolved data messages.
        data: VecDeque<AnyLightClientIdentified<AnyData>>,
        /// The message that will utilize the aggregated data.
        receiver: AnyLightClientIdentified<AnyAggregate>,
    },
}

pub trait GetLc<L: LightClient> {
    fn get_lc(&self, chain_id: &ChainIdOf<L>) -> L;
}

impl RelayerMsg {
    // NOTE: Box is required bc recursion
    pub fn handle<G>(
        self,
        g: &G,
        depth: usize,
    ) -> BoxFuture<'_, Result<Vec<RelayerMsg>, HandleMsgError>>
    where
        G: Send
            + Sync
            + GetLc<EthereumMinimal>
            + GetLc<EthereumMainnet>
            + GetLc<CometblsMinimal>
            + GetLc<CometblsMainnet>,
    {
        tracing::info!(
            depth,
            %self,
            "handling message",
        );

        async move {
            match self {
                RelayerMsg::Lc(any_lc_msg) => {
                    let res = match any_lc_msg {
                        AnyLightClientIdentified::EthereumMainnet(msg) => {
                            msg.data.handle(g.get_lc(&msg.chain_id)).await.map_err(AnyLcError::EthereumMainnet)?
                        }
                        AnyLightClientIdentified::EthereumMinimal(msg) => {
                            msg.data.handle(g.get_lc(&msg.chain_id)).await.map_err(AnyLcError::EthereumMinimal)?
                        }
                        AnyLightClientIdentified::CometblsMainnet(msg) => {
                            msg.data.handle(g.get_lc(&msg.chain_id)).await.map_err(AnyLcError::CometblsMainnet)?
                        }
                        AnyLightClientIdentified::CometblsMinimal(msg) => {
                            msg.data.handle(g.get_lc(&msg.chain_id)).await.map_err(AnyLcError::CometblsMinimal)?
                        }                    };

                    Ok(res)
                }

                RelayerMsg::DeferUntil { point: DeferPoint::Relative, seconds } =>
                    Ok([RelayerMsg::DeferUntil { point: DeferPoint::Absolute, seconds: now() + seconds }].into()),

                RelayerMsg::DeferUntil { seconds, .. } => {
                    // if we haven't hit the time yet, requeue the defer msg
                    if now() < seconds {
                        // TODO: Make the time configurable?
                        tokio::time::sleep(Duration::from_secs(1)).await;

                        Ok([defer(seconds)].into())
                    } else {
                        Ok(vec![])
                    }
                }

                RelayerMsg::Timeout {
                    timeout_timestamp,
                    msg,
                } => {
                    // if we haven't hit the timeout yet, handle the msg
                    if now() > timeout_timestamp {
                        tracing::warn!(json = %serde_json::to_string(&msg).unwrap(), "message expired");

                        Ok([].into())
                    } else {
                        msg.handle(g, depth + 1).await
                    }
                }
                RelayerMsg::Sequence(mut s) => {
                    let msgs = match s.pop_front() {
                        Some(msg) => msg.handle(g, depth + 1).await?,
                        None => return Ok(vec![]),
                    };

                    for msg in msgs.into_iter().rev() {
                        s.push_front(msg);
                    }

                    Ok([flatten_seq(seq(s))].into())
                }

                RelayerMsg::Retry(count, msg) =>  {
                    const RETRY_DELAY_SECONDS: u64 = 3;

                    match msg.clone().handle(g, depth + 1).await {
                        Ok(ok) => Ok(ok),
                        Err(err) => if count > 0 {
                            let retries_left = count - 1;
                            tracing::warn!(
                                %msg,
                                retries_left,
                                ?err,
                                "msg failed, retrying in {RETRY_DELAY_SECONDS} seconds"
                            );
                            Ok([seq([defer(now() + RETRY_DELAY_SECONDS), retry(retries_left, *msg)])].into())
                        } else {
                            tracing::error!(%msg, "msg failed after all retries");
                            Err(err)
                        },
                    }
                },

                RelayerMsg::Aggregate {
                    mut queue,
                    mut data,
                    receiver,
                } => {
                    if let Some(msg) = queue.pop_front() {
                        let msgs = msg.handle(g, depth + 1).await?;

                        for m in msgs {
                            match <AnyLightClientIdentified<AnyData>>::try_from(m) {
                                Ok(d) => {
                                    data.push_back(d);
                                }
                                Err(m) => {
                                    queue.push_back(m);
                                }
                            }
                        }

                        let res = [RelayerMsg::Aggregate {
                            queue,
                            data,
                            receiver,
                        }]
                        .into();

                        Ok(res)
                    } else {
                        // queue is empty, handle msg

                        let res = match receiver {
                            AggregateReceiver::EthereumMainnet(msg) => {
                                msg.handle(data)
                            }
                            AggregateReceiver::EthereumMinimal(msg) => {
                                msg.handle(data)
                            }
                            AggregateReceiver::CometblsMainnet(msg) => {
                                msg.handle(data)
                            }
                            AggregateReceiver::CometblsMinimal(msg) => {
                                msg.handle(data)
                            }
                        };

                        Ok(res)
                    }
                }
                RelayerMsg::Repeat { times: 0, .. } => Ok([].into()),
                RelayerMsg::Repeat { times, msg } => {
                    Ok([flatten_seq(seq([*msg.clone(), RelayerMsg::Repeat { times: times - 1, msg}]))].into())
                },
            }
        }
        .boxed()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HandleMsgError {
    #[error(transparent)]
    Lc(#[from] AnyLcError),
}

enum_variants_conversions! {
    #[derive(Debug, thiserror::Error)]
    pub enum AnyLcError {
        // The 08-wasm client tracking the state of Evm<Mainnet>.
        #[error(transparent)]
        EthereumMainnet(LcError<EthereumMainnet>),
        // The 08-wasm client tracking the state of Evm<Minimal>.
        #[error(transparent)]
        EthereumMinimal(LcError<EthereumMinimal>),
        // The solidity client on Evm<Mainnet> tracking the state of Union.
        #[error(transparent)]
        CometblsMainnet(LcError<CometblsMainnet>),
        // The solidity client on Evm<Minimal> tracking the state of Union.
        #[error(transparent)]
        CometblsMinimal(LcError<CometblsMinimal>),
    }
}

impl TryFrom<AnyLightClientIdentified<AnyLcMsg>> for AnyLightClientIdentified<AnyData> {
    type Error = AnyLightClientIdentified<AnyLcMsg>;

    fn try_from(value: AnyLightClientIdentified<AnyLcMsg>) -> Result<Self, Self::Error> {
        match value {
            AnyLightClientIdentified::EthereumMainnet(i) => <Data<_>>::try_from(i.data)
                .map(|d| Identified::new(i.chain_id.clone(), d))
                .map(AnyLightClientIdentified::EthereumMainnet)
                .map_err(|l| Identified::new(i.chain_id, l))
                .map_err(AnyLightClientIdentified::EthereumMainnet),
            AnyLightClientIdentified::EthereumMinimal(i) => <Data<_>>::try_from(i.data)
                .map(|d| Identified::new(i.chain_id.clone(), d))
                .map(AnyLightClientIdentified::EthereumMinimal)
                .map_err(|l| Identified::new(i.chain_id, l))
                .map_err(AnyLightClientIdentified::EthereumMinimal),
            AnyLightClientIdentified::CometblsMainnet(i) => <Data<_>>::try_from(i.data)
                .map(|d| Identified::new(i.chain_id, d))
                .map(AnyLightClientIdentified::CometblsMainnet)
                .map_err(|l| Identified::new(i.chain_id, l))
                .map_err(AnyLightClientIdentified::CometblsMainnet),
            AnyLightClientIdentified::CometblsMinimal(i) => <Data<_>>::try_from(i.data)
                .map(|d| Identified::new(i.chain_id, d))
                .map(AnyLightClientIdentified::CometblsMinimal)
                .map_err(|l| Identified::new(i.chain_id, l))
                .map_err(AnyLightClientIdentified::CometblsMinimal),
        }
    }
}

pub type AggregateReceiver = AnyLightClientIdentified<AnyAggregate>;
pub type AggregateData = AnyLightClientIdentified<AnyData>;

impl std::fmt::Display for RelayerMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelayerMsg::Lc(lc) => write!(f, "Lc({lc})"),
            RelayerMsg::DeferUntil { point, seconds } => {
                write!(f, "DeferUntil({:?}, {seconds})", point)
            }
            RelayerMsg::Repeat { times, msg } => write!(f, "Repeat({times}, {msg})"),
            RelayerMsg::Timeout {
                timeout_timestamp,
                msg,
            } => write!(f, "Timeout({timeout_timestamp}, {msg})"),
            RelayerMsg::Sequence(seq) => {
                write!(f, "Sequence [")?;
                let len = seq.len();
                for (idx, msg) in seq.iter().enumerate() {
                    write!(f, "{msg}")?;
                    if idx != len - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            RelayerMsg::Retry(remaining, msg) => write!(f, "Retry({remaining}, {msg})"),
            RelayerMsg::Aggregate {
                queue,
                data,
                receiver,
            } => {
                let data = data
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                let queue = queue
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(f, "Aggregate([{queue}] -> [{data}] -> {receiver})")
            }
        }
    }
}

impl TryFrom<RelayerMsg> for AnyLightClientIdentified<AnyLcMsg> {
    type Error = RelayerMsg;

    fn try_from(value: RelayerMsg) -> Result<Self, Self::Error> {
        match value {
            RelayerMsg::Lc(ok) => Ok(ok),
            _ => Err(value),
        }
    }
}

impl TryFrom<RelayerMsg> for AnyLightClientIdentified<AnyData> {
    type Error = RelayerMsg;

    fn try_from(value: RelayerMsg) -> Result<Self, Self::Error> {
        match value {
            RelayerMsg::Lc(any_lc_msg) => {
                AnyLightClientIdentified::<AnyData>::try_from(any_lc_msg).map_err(RelayerMsg::Lc)
            }
            _ => Err(value),
        }
    }
}

impl From<AnyLightClientIdentified<AnyLcMsg>> for RelayerMsg {
    fn from(value: AnyLightClientIdentified<AnyLcMsg>) -> Self {
        Self::Lc(value)
    }
}

impl<L: LightClient> TryFrom<RelayerMsg> for LcMsg<L>
where
    LcMsg<L>: TryFrom<AnyLightClientIdentified<AnyLcMsg>, Error = AnyLightClientIdentified<AnyLcMsg>>
        + Into<AnyLightClientIdentified<AnyLcMsg>>,
{
    type Error = RelayerMsg;

    fn try_from(value: RelayerMsg) -> Result<Self, Self::Error> {
        LcMsg::<L>::try_from(<AnyLightClientIdentified<AnyLcMsg>>::try_from(value)?)
            .map_err(Into::into)
    }
}

impl<L: LightClient> From<LcMsg<L>> for RelayerMsg
where
    AnyLightClientIdentified<AnyLcMsg>: From<LcMsg<L>>,
{
    fn from(value: LcMsg<L>) -> Self {
        RelayerMsg::Lc(<AnyLightClientIdentified<AnyLcMsg>>::from(value))
    }
}

macro_rules! any_enum {
    (
        $(#[doc = $outer_doc:literal])*
        #[any = $Any:ident]
        pub enum $Enum:ident<L: LightClient> {
            $(
                $(#[doc = $doc:literal])*
                $Variant:ident$((
                    $(#[$variant_inner_meta:meta])*
                    $VariantInner:ty
                ))?,
            )+
        }
    ) => {
        #[derive(frame_support_procedural::DebugNoBound, frame_support_procedural::CloneNoBound, frame_support_procedural::PartialEqNoBound, serde::Serialize, serde::Deserialize)]
        #[serde(bound(serialize = "", deserialize = ""))]
        $(#[doc = $outer_doc])*
        #[allow(clippy::large_enum_variant)]
        pub enum $Enum<L: LightClient> {
            $(
                $(#[doc = $doc])*
                $Variant$((
                    $(#[$variant_inner_meta])*
                    $VariantInner
                ))?,
            )+
        }

        pub enum $Any {}
        impl crate::AnyLightClient for $Any {
            type Inner<L: LightClient> = $Enum<L>;
        }

        impl<L: LightClient> TryFrom<crate::LcMsg<L>> for $Enum<L> {
            type Error = crate::LcMsg<L>;

            fn try_from(value: crate::LcMsg<L>) -> Result<Self, Self::Error> {
                if let crate::LcMsg::$Enum(t) = value {
                    Ok(t)
                } else {
                    Err(value)
                }
            }
        }

        impl<L: LightClient> From<crate::Identified<L, crate::InnerOf<$Any, L>>> for crate::RelayerMsg
        where
            crate::LcMsg<L>: From<crate::InnerOf<$Any, L>>,
            crate::AnyLightClientIdentified<crate::AnyLcMsg>:
                From<crate::Identified<L, crate::InnerOf<crate::AnyLcMsg, L>>>
        {
            fn from(value: crate::Identified<L, crate::InnerOf<$Any, L>>) -> Self {
                Self::Lc(
                    <crate::AnyLightClientIdentified<crate::AnyLcMsg>>::from(
                        crate::Identified {
                            chain_id: value.chain_id, data: crate::LcMsg::from(value.data)
                        }
                    )
                )
            }
        }

        impl<L: LightClient> TryFrom<crate::RelayerMsg> for crate::Identified<L, crate::InnerOf<$Any, L>>
        where
            crate::AnyLightClientIdentified<crate::AnyLcMsg>: TryFrom<crate::RelayerMsg, Error = crate::RelayerMsg> + Into<crate::RelayerMsg>,
            crate::Identified<L, crate::LcMsg<L>>: TryFrom<crate::AnyLightClientIdentified<crate::AnyLcMsg>, Error = crate::AnyLightClientIdentified<crate::AnyLcMsg>>
                + Into<crate::AnyLightClientIdentified<crate::AnyLcMsg>>,
            crate::InnerOf<$Any, L>: TryFrom<crate::LcMsg<L>, Error = crate::LcMsg<L>> + Into<crate::LcMsg<L>>,
        {
            type Error = crate::RelayerMsg;
            fn try_from(value: crate::RelayerMsg) -> Result<Self, crate::RelayerMsg> {
                let any_lc_msg = <crate::AnyLightClientIdentified<crate::AnyLcMsg>>::try_from(value)?;
                let identified_lc_msg = <crate::Identified<L, crate::LcMsg<L>>>::try_from(any_lc_msg)
                    .map_err(<crate::AnyLightClientIdentified<crate::AnyLcMsg>>::from)?;
                let data =
                    <crate::InnerOf<$Any, L>>::try_from(identified_lc_msg.data).map_err(|x: crate::LcMsg<L>| {
                        Into::<crate::AnyLightClientIdentified<crate::AnyLcMsg>>::into(crate::Identified::<L, crate::LcMsg<L>>::new(
                            identified_lc_msg.chain_id.clone(),
                            x,
                        ))
                    })?;

                Ok(crate::Identified::new(identified_lc_msg.chain_id, data))
            }
        }

        $(
            $(
                impl<L: LightClient> TryFrom<$Enum<L>> for $VariantInner {
                    type Error = $Enum<L>;

                    fn try_from(value: $Enum<L>) -> Result<Self, Self::Error> {
                        match value {
                            $Enum::$Variant(t) => Ok(t),
                            _ => Err(value),
                        }
                    }
                }

                impl<L: LightClient> From<$VariantInner> for $Enum<L> {
                    fn from(value: $VariantInner) -> Self {
                        Self::$Variant(value)
                    }
                }

                impl<L: LightClient> TryInto<crate::Identified<L, $VariantInner>> for crate::RelayerMsg
                where
                    crate::AnyLightClientIdentified<crate::AnyLcMsg>: TryFrom<crate::RelayerMsg, Error = crate::RelayerMsg> + Into<crate::RelayerMsg>,
                    crate::LcMsg<L>: TryFrom<crate::AnyLightClientIdentified<crate::AnyLcMsg>, Error = crate::AnyLightClientIdentified<crate::AnyLcMsg>> + Into<crate::AnyLightClientIdentified<crate::AnyLcMsg>>,
                    crate::Identified<L, $VariantInner>: TryFrom<crate::LcMsg<L>, Error = crate::LcMsg<L>> + Into<crate::LcMsg<L>>,
                {
                    type Error = crate::RelayerMsg;

                    fn try_into(self) -> Result<crate::Identified<L, $VariantInner>, crate::RelayerMsg> {
                        <crate::AnyLightClientIdentified<crate::AnyLcMsg>>::try_from(self)
                            .and_then(|x| <crate::LcMsg<L>>::try_from(x).map_err(Into::into))
                            .and_then(|x| {
                                <crate::Identified<L, $VariantInner>>::try_from(x)
                                    .map_err(Into::<crate::LcMsg<L>>::into)
                                    .map_err(Into::<crate::AnyLightClientIdentified<crate::AnyLcMsg>>::into)
                                    .map_err(Into::<crate::RelayerMsg>::into)
                            })
                    }
                }

                impl<L: LightClient> TryFrom<crate::LcMsg<L>> for $VariantInner {
                    type Error = crate::LcMsg<L>;

                    fn try_from(value: crate::LcMsg<L>) -> Result<Self, crate::LcMsg<L>> {
                        match value {
                            crate::LcMsg::$Enum($Enum::$Variant(data)) => Ok(data),
                            _ => Err(value),
                        }
                    }
                }

                impl<L: LightClient> TryFrom<crate::AnyLightClientIdentified<crate::AnyLcMsg>> for crate::Identified<L, $VariantInner>
                where
                    crate::LcMsg<L>: TryFrom<crate::AnyLightClientIdentified<crate::AnyLcMsg>, Error = crate::AnyLightClientIdentified<crate::AnyLcMsg>> + Into<crate::AnyLightClientIdentified<crate::AnyLcMsg>>,
                    Self: TryFrom<crate::LcMsg<L>, Error = crate::LcMsg<L>> + Into<crate::LcMsg<L>>,
                {
                    type Error = crate::AnyLightClientIdentified<crate::AnyLcMsg>;

                    fn try_from(value: crate::AnyLightClientIdentified<crate::AnyLcMsg>) -> Result<Self, crate::AnyLightClientIdentified<crate::AnyLcMsg>> {
                        crate::LcMsg::<L>::try_from(value).and_then(|x| Self::try_from(x).map_err(Into::into))
                    }
                }
            )?
        )+
    };
}

pub(crate) use any_enum;

pub trait IbcPathExt<L: LightClient>: IbcPath<ChainOf<L>, ChainOf<L::Counterparty>> {
    type Data: TryFrom<AggregateData, Error = AggregateData> + Into<AggregateData>;
    type Proof: TryFrom<AggregateData, Error = AggregateData> + Into<AggregateData>;
}

pub trait AnyPath<L: LightClient> {
    type Inner<P: IbcPath<L::HostChain, ChainOf<L::Counterparty>>>;
}

pub enum Path2<L: LightClient, P: AnyPath<L>> {
    ClientStatePath(P::Inner<ClientStatePath<<L::HostChain as Chain>::ClientId>>),
    ClientConsensusStatePath(
        P::Inner<
            ClientConsensusStatePath<
                <L::HostChain as Chain>::ClientId,
                <ChainOf<L::Counterparty> as Chain>::Height,
            >,
        >,
    ),
    ConnectionPath(P::Inner<ConnectionPath>),
    ChannelEndPath(P::Inner<ChannelEndPath>),
    CommitmentPath(P::Inner<CommitmentPath>),
    AcknowledgementPath(P::Inner<AcknowledgementPath>),
}

pub type PathOf<L> = unionlabs::proof::Path<
    <<L as LightClientBase>::HostChain as Chain>::ClientId,
    <ChainOf<<L as LightClientBase>::Counterparty> as Chain>::Height,
>;

pub trait AnyLightClient {
    type Inner<L: LightClient>: Debug
        + Display
        + Clone
        + PartialEq
        + Serialize
        + for<'de> Deserialize<'de>;
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(bound(serialize = "", deserialize = ""))]
#[allow(clippy::large_enum_variant)]
pub enum AnyLightClientIdentified<T: AnyLightClient> {
    // The 08-wasm client tracking the state of Evm<Mainnet>.
    #[display(fmt = "EthereumMainnet({}, {})", "_0.chain_id", "_0.data")]
    EthereumMainnet(Identified<EthereumMainnet, InnerOf<T, EthereumMainnet>>),
    // The 08-wasm client tracking the state of Evm<Minimal>.
    #[display(fmt = "EthereumMinimal({}, {})", "_0.chain_id", "_0.data")]
    EthereumMinimal(Identified<EthereumMinimal, InnerOf<T, EthereumMinimal>>),
    // The solidity client on Evm<Mainnet> tracking the state of Union.
    #[display(fmt = "CometblsMainnet({}, {})", "_0.chain_id", "_0.data")]
    CometblsMainnet(Identified<CometblsMainnet, InnerOf<T, CometblsMainnet>>),
    // The solidity client on Evm<Minimal> tracking the state of Union.
    #[display(fmt = "CometblsMinimal({}, {})", "_0.chain_id", "_0.data")]
    CometblsMinimal(Identified<CometblsMinimal, InnerOf<T, CometblsMinimal>>),
}

impl<T: AnyLightClient> From<Identified<EthereumMainnet, InnerOf<T, EthereumMainnet>>>
    for AnyLightClientIdentified<T>
{
    fn from(v: Identified<EthereumMainnet, InnerOf<T, EthereumMainnet>>) -> Self {
        Self::EthereumMainnet(v)
    }
}

impl<T: AnyLightClient> TryFrom<AnyLightClientIdentified<T>>
    for Identified<EthereumMainnet, InnerOf<T, EthereumMainnet>>
{
    type Error = AnyLightClientIdentified<T>;
    fn try_from(v: AnyLightClientIdentified<T>) -> Result<Self, Self::Error> {
        if let AnyLightClientIdentified::EthereumMainnet(v) = v {
            Ok(v)
        } else {
            Err(v)
        }
    }
}

impl<T: AnyLightClient> From<Identified<EthereumMinimal, InnerOf<T, EthereumMinimal>>>
    for AnyLightClientIdentified<T>
{
    fn from(v: Identified<EthereumMinimal, InnerOf<T, EthereumMinimal>>) -> Self {
        Self::EthereumMinimal(v)
    }
}

impl<T: AnyLightClient> TryFrom<AnyLightClientIdentified<T>>
    for Identified<EthereumMinimal, InnerOf<T, EthereumMinimal>>
{
    type Error = AnyLightClientIdentified<T>;
    fn try_from(v: AnyLightClientIdentified<T>) -> Result<Self, Self::Error> {
        if let AnyLightClientIdentified::EthereumMinimal(v) = v {
            Ok(v)
        } else {
            Err(v)
        }
    }
}

impl<T: AnyLightClient> From<Identified<CometblsMainnet, InnerOf<T, CometblsMainnet>>>
    for AnyLightClientIdentified<T>
{
    fn from(v: Identified<CometblsMainnet, InnerOf<T, CometblsMainnet>>) -> Self {
        Self::CometblsMainnet(v)
    }
}

impl<T: AnyLightClient> TryFrom<AnyLightClientIdentified<T>>
    for Identified<CometblsMainnet, InnerOf<T, CometblsMainnet>>
{
    type Error = AnyLightClientIdentified<T>;
    fn try_from(v: AnyLightClientIdentified<T>) -> Result<Self, Self::Error> {
        if let AnyLightClientIdentified::CometblsMainnet(v) = v {
            Ok(v)
        } else {
            Err(v)
        }
    }
}

impl<T: AnyLightClient> From<Identified<CometblsMinimal, InnerOf<T, CometblsMinimal>>>
    for AnyLightClientIdentified<T>
{
    fn from(v: Identified<CometblsMinimal, InnerOf<T, CometblsMinimal>>) -> Self {
        Self::CometblsMinimal(v)
    }
}

impl<T: AnyLightClient> TryFrom<AnyLightClientIdentified<T>>
    for Identified<CometblsMinimal, InnerOf<T, CometblsMinimal>>
{
    type Error = AnyLightClientIdentified<T>;
    fn try_from(v: AnyLightClientIdentified<T>) -> Result<Self, Self::Error> {
        if let AnyLightClientIdentified::CometblsMinimal(v) = v {
            Ok(v)
        } else {
            Err(v)
        }
    }
}

#[macro_export]
macro_rules! enum_variants_conversions {
    (
        $(#[$meta:meta])*
        pub enum $Enum:ident {
            $(
                $(#[$inner_meta:meta])*
                $Variant:ident($Inner:ty),
            )+
        }
    ) => {
        $(#[$meta])*
        pub enum $Enum {
            $(
                $(#[$inner_meta])*
                $Variant($Inner),
            )+
        }

        $(
            impl From<$Inner> for $Enum {
                fn from(inner: $Inner) -> Self {
                    Self::$Variant(inner)
                }
            }

            impl TryFrom<$Enum> for $Inner {
                type Error = $Enum;

                fn try_from(value: $Enum) -> Result<Self, Self::Error> {
                    match value {
                        $Enum::$Variant(inner) => Ok(inner),
                        _ => Err(value),
                    }
                }
            }
        )+
    };
}

#[macro_export]
macro_rules! identified {
    ($Ty:ident<$L:ty>) => {
        $crate::Identified<$L, $Ty<$L>>
    };
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(bound(serialize = "", deserialize = ""))]
#[allow(clippy::large_enum_variant)]
pub enum LcMsg<L: LightClient> {
    #[display(fmt = "Event({})", "_0")]
    Event(InnerOf<AnyEvent, L>),
    // data that has been read
    #[display(fmt = "Data({})", "_0")]
    Data(InnerOf<AnyData, L>),
    // read
    #[display(fmt = "Fetch({})", "_0")]
    Fetch(InnerOf<AnyFetch, L>),
    // write
    #[display(fmt = "Msg({})", "_0")]
    Msg(InnerOf<AnyMsg, L>),
    #[display(fmt = "Wait({})", "_0")]
    Wait(InnerOf<AnyWait, L>),
    // REVIEW: Does this make sense as a top-level message?
    #[display(fmt = "Aggregate({})", "_0")]
    Aggregate(InnerOf<AnyAggregate, L>),
}

impl<L: LightClient> LcMsg<L> {
    pub async fn handle(self, l: L) -> Result<Vec<RelayerMsg>, LcError<L>>
    where
        AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L>)>,
        AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L::Counterparty>)>,
        AggregateReceiver: From<identified!(Aggregate<L>)>,
        AnyLightClientIdentified<AnyData>: From<identified!(Data<L>)>,
        // TODO: Remove once we no longer unwrap in fetch.handle()
        <<L as LightClientBase>::ClientId as TryFrom<
            <<L as LightClientBase>::HostChain as Chain>::ClientId,
        >>::Error: Debug,
        <<L::Counterparty as LightClientBase>::ClientId as TryFrom<
            <<L::Counterparty as LightClientBase>::HostChain as Chain>::ClientId,
        >>::Error: Debug,
    {
        match self {
            LcMsg::Event(event) => Ok(event.handle(l)),
            LcMsg::Data(data) => {
                // TODO: Figure out a way to bubble it up to the top level

                let data = AnyLightClientIdentified::<AnyData>::from(Identified::new(
                    l.chain().chain_id(),
                    data,
                ));

                tracing::error!(
                    data = %serde_json::to_string(&data).unwrap(),
                    "received data outside of an aggregation"
                );

                Ok([].into())
            }
            LcMsg::Fetch(fetch) => Ok(fetch.handle(l).await),
            LcMsg::Msg(m) => {
                // NOTE: `Msg`s don't requeue any `RelayerMsg`s; they are side-effect only.
                l.msg(m).await.map_err(LcError::Msg)?;

                Ok([].into())
            }
            LcMsg::Wait(wait) => Ok(wait.handle(l).await),
            LcMsg::Aggregate(_) => {
                todo!()
            }
        }
    }
}

#[derive(DebugNoBound, thiserror::Error)]
pub enum LcError<L: LightClient> {
    #[error(transparent)]
    Msg(L::MsgError),
}

pub type InnerOf<T, L> = <T as AnyLightClient>::Inner<L>;

pub enum AnyLcMsg {}

impl AnyLightClient for AnyLcMsg {
    type Inner<L: LightClient> = LcMsg<L>;
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(
    serialize = "Data: ::serde::Serialize",
    deserialize = "Data: for<'d> Deserialize<'d>"
))]
// TODO: `Data: AnyLightClient`
// prerequisites: derive macro for AnyLightClient
pub struct Identified<L: LightClient, Data: Debug + Clone + PartialEq> {
    pub chain_id: ChainIdOf<L>,
    pub data: Data,
}

impl<L: LightClient, Data: Debug + Clone + PartialEq> Identified<L, Data> {
    pub fn new(chain_id: ChainIdOf<L>, data: Data) -> Self {
        Self { chain_id, data }
    }
}

pub trait DoAggregate<L>: Sized + Debug + Clone + PartialEq
where
    L: LightClient,
{
    fn do_aggregate(
        _: Identified<L, Self>,
        _: VecDeque<AnyLightClientIdentified<AnyData>>,
    ) -> Vec<RelayerMsg>;
}

// helper fns

pub fn retry(count: u8, t: impl Into<RelayerMsg>) -> RelayerMsg {
    RelayerMsg::Retry(count, Box::new(t.into()))
}

pub fn seq(ts: impl IntoIterator<Item = RelayerMsg>) -> RelayerMsg {
    RelayerMsg::Sequence(ts.into_iter().collect())
}

pub fn defer(timestamp: u64) -> RelayerMsg {
    RelayerMsg::DeferUntil {
        point: DeferPoint::Absolute,
        seconds: timestamp,
    }
}

pub fn defer_relative(seconds: u64) -> RelayerMsg {
    RelayerMsg::DeferUntil {
        point: DeferPoint::Relative,
        seconds,
    }
}

pub fn fetch<L: LightClient>(chain_id: ChainIdOf<L>, t: impl Into<Fetch<L>>) -> RelayerMsg
where
    AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L>)>,
{
    RelayerMsg::Lc(AnyLightClientIdentified::from(Identified::new(
        chain_id,
        LcMsg::Fetch(t.into()),
    )))
}

pub fn msg<L: LightClient>(chain_id: ChainIdOf<L>, t: impl Into<Msg<L>>) -> RelayerMsg
where
    AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L>)>,
{
    RelayerMsg::Lc(AnyLightClientIdentified::from(Identified::new(
        chain_id,
        LcMsg::Msg(t.into()),
    )))
}

pub fn data<L: LightClient>(chain_id: ChainIdOf<L>, t: impl Into<Data<L>>) -> RelayerMsg
where
    AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L>)>,
{
    RelayerMsg::Lc(AnyLightClientIdentified::from(Identified::new(
        chain_id,
        LcMsg::Data(t.into()),
    )))
}

pub fn wait<L: LightClient>(chain_id: ChainIdOf<L>, t: impl Into<Wait<L>>) -> RelayerMsg
where
    AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L>)>,
{
    RelayerMsg::Lc(AnyLightClientIdentified::from(Identified::new(
        chain_id,
        LcMsg::Wait(t.into()),
    )))
}

pub fn event<L: LightClient>(chain_id: ChainIdOf<L>, t: impl Into<Event<L>>) -> RelayerMsg
where
    AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L>)>,
{
    RelayerMsg::Lc(AnyLightClientIdentified::from(Identified::new(
        chain_id,
        LcMsg::Event(t.into()),
    )))
}

/// Returns the current unix timestamp in seconds.
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn flatten_seq(msg: RelayerMsg) -> RelayerMsg {
    fn flatten(msg: RelayerMsg) -> VecDeque<RelayerMsg> {
        if let RelayerMsg::Sequence(new_seq) = msg {
            new_seq.into_iter().flat_map(flatten).collect()
        } else {
            [msg].into()
        }
    }

    let mut msgs = flatten(msg);

    if msgs.len() == 1 {
        msgs.pop_front().unwrap()
    } else {
        seq(msgs)
    }
}

#[test]
fn flatten() {
    use crate::{defer, seq};

    let msg = seq([
        defer(1),
        seq([defer(2), defer(3)]),
        seq([defer(4)]),
        defer(5),
    ]);

    let msg = flatten_seq(msg);

    dbg!(msg);
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    // use hex_literal::hex;

    // use super::*;
    // use crate::{chain::cosmos::EthereumConfig, msg::CreateClientData};

    use std::{collections::VecDeque, fmt::Debug, marker::PhantomData};

    use hex_literal::hex;
    use lightclient::{
        cometbls::{CometblsConfig, CometblsMinimal},
        ethereum::{EthereumConfig, EthereumMinimal},
    };
    use serde::{de::DeserializeOwned, Serialize};
    use unionlabs::{
        ethereum::{Address, H256, U256},
        events::{ConnectionOpenAck, ConnectionOpenTry},
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
        validated::ValidateT,
        EmptyString, QueryHeight, DELAY_PERIOD,
    };

    use super::LcMsg;
    use crate::{
        aggregate::{Aggregate, AggregateCreateClient, AnyAggregate},
        data::Data,
        defer_relative, event,
        event::{Event, IbcEvent},
        fetch,
        fetch::{
            AnyFetch, Fetch, FetchConnectionEnd, FetchSelfClientState, FetchSelfConsensusState,
            FetchTrustedClientState,
        },
        msg,
        msg::{Msg, MsgChannelOpenInitData, MsgConnectionOpenInitData, MsgConnectionOpenTryData},
        seq, AggregateReceiver, AnyLcMsg, AnyMsg, Identified, RelayerMsg,
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

        print_json(msg::<EthereumMinimal>(
            union_chain_id.clone(),
            MsgConnectionOpenInitData {
                msg: MsgConnectionOpenInit {
                    client_id: parse!("08-wasm-2"),
                    counterparty: connection::counterparty::Counterparty {
                        client_id: parse!("cometbls-0"),
                        connection_id: ""
                            .to_string()
                            .validate()
                            .expect("empty string is a valid empty string; qed;"),
                        prefix: MerklePrefix {
                            key_prefix: b"ibc".to_vec(),
                        },
                    },
                    version: Version {
                        identifier: "1".into(),
                        features: [Order::Ordered, Order::Unordered].into_iter().collect(),
                    },
                    delay_period: DELAY_PERIOD,
                },
            },
        ));

        print_json(msg::<EthereumMinimal>(
            union_chain_id.clone(),
            MsgChannelOpenInitData {
                msg: MsgChannelOpenInit {
                    port_id: parse!("ucs01-relay"),
                    channel: Channel {
                        state: channel::state::State::Init,
                        ordering: channel::order::Order::Unordered,
                        counterparty: channel::counterparty::Counterparty {
                            port_id: parse!("WASM_PORT_ID"),
                            channel_id: parse!(""),
                        },
                        connection_hops: vec![parse!("connection-8")],
                        version: "ucs00-pingpong-1".to_string(),
                    },
                },
                __marker: PhantomData,
            },
        ));

        print_json(msg::<CometblsMinimal>(
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
        ));

        print_json(msg::<CometblsMinimal>(
            eth_chain_id,
            MsgConnectionOpenInitData {
                msg: MsgConnectionOpenInit {
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
                },
            },
        ));

        print_json(event::<CometblsMinimal>(
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
        ));

        print_json(RelayerMsg::Repeat {
            times: u64::MAX,
            msg: Box::new(seq([
                defer_relative(30),
                event::<CometblsMinimal>(
                    eth_chain_id,
                    crate::event::Command::UpdateClient {
                        client_id: parse!("cometbls-0"),
                        counterparty_client_id: parse!("08-wasm-0"),
                    },
                ),
            ])),
        });

        print_json(RelayerMsg::Repeat {
            times: u64::MAX,
            msg: Box::new(seq([
                defer_relative(30),
                event::<EthereumMinimal>(
                    union_chain_id.clone(),
                    crate::event::Command::UpdateClient {
                        client_id: parse!("08-wasm-0"),
                        counterparty_client_id: parse!("cometbls-0"),
                    },
                ),
            ])),
        });

        println!("\ncreate client msgs\n");

        print_json(RelayerMsg::Sequence(
            [
                RelayerMsg::Aggregate {
                    queue: [
                        fetch::<EthereumMinimal>(
                            union_chain_id.clone(),
                            FetchSelfClientState {
                                at: QueryHeight::Latest,
                            },
                        ),
                        fetch::<EthereumMinimal>(
                            union_chain_id.clone(),
                            FetchSelfConsensusState {
                                at: QueryHeight::Latest,
                            },
                        )
                    ]
                    .into(),
                    data: [].into_iter().collect(),
                    receiver: AggregateReceiver::CometblsMinimal(Identified {
                        chain_id: eth_chain_id,
                        data: Aggregate::CreateClient(AggregateCreateClient {
                            config: CometblsConfig {
                                client_type: "cometbls".to_string(),
                                cometbls_client_address: Address(hex!(
                                    "83428c7db9815f482a39a1715684dcf755021997"
                                )),
                            },
                        }),
                    }),
                },
                RelayerMsg::Aggregate {
                    queue: [
                        fetch::<CometblsMinimal>(
                            eth_chain_id,
                            FetchSelfClientState {
                                at: QueryHeight::Latest,
                            },
                        ),
                        fetch::<CometblsMinimal>(
                            eth_chain_id,
                            FetchSelfConsensusState {
                                at: QueryHeight::Latest,
                            },
                        )
                    ]
                    .into(),
                    data: [].into_iter().collect(),
                    receiver: AggregateReceiver::EthereumMinimal(Identified {
                        chain_id: union_chain_id.clone(),
                        data: Aggregate::CreateClient(AggregateCreateClient {
                            config: EthereumConfig {
                                code_id: H256(hex!(
                                    "78266014ea77f3b785e45a33d1f8d3709444a076b3b38b2aeef265b39ad1e494"
                                )),
                            },
                        }),
                    }),
                },
            ]
            .into(),
        ));

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
        print_json(fetch::<EthereumMinimal>(
            union_chain_id.clone(),
            FetchConnectionEnd {
                at: parse!("1-103"),
                connection_id: parse!("connection-1"),
            },
        ))
    }

    fn print_json<T: Serialize + DeserializeOwned + PartialEq + Debug>(msg: T) {
        let json = serde_json::to_string(&msg).unwrap();

        println!("{json}\n");

        let from_json = serde_json::from_str(&json).unwrap();

        assert_eq!(&msg, &from_json, "json roundtrip failed");
    }
}
