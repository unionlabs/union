#![feature(trait_alias)]
#![allow(clippy::type_complexity)]

use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
    future::Future,
    marker::PhantomData,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chain_utils::{
    evm::Evm,
    union::{broadcast_tx_commit, BroadcastTxCommitError, CosmosSdkChain, Union},
};
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use futures::{future::BoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use unionlabs::{
    encoding::Proto,
    ethereum::config::{Mainnet, Minimal},
    google::protobuf::any::{mk_any, Any},
    hash::H256,
    ibc::{core::client::height::IsHeight, lightclients::wasm},
    proof::{self},
    traits::{
        Chain, ChainIdOf, ClientIdOf, ClientState, ClientStateOf, ConsensusStateOf, HeaderOf,
        HeightOf,
    },
    IntoProto, MaybeRecoverableError, TypeUrl,
};

use crate::{
    aggregate::{Aggregate, AnyAggregate},
    data::{AnyData, Data},
    event::{AnyEvent, Event},
    fetch::{AnyFetch, DoFetch, Fetch, FetchUpdateHeaders},
    msg::{AnyMsg, Msg},
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

pub trait RelayerMsgDatagram =
    Debug + Display + Clone + PartialEq + Serialize + for<'de> Deserialize<'de> + 'static;

pub trait ChainExt: Chain {
    type Data<Tr: ChainExt>: RelayerMsgDatagram;
    type Fetch<Tr: ChainExt>: RelayerMsgDatagram;
    type Aggregate<Tr: ChainExt>: RelayerMsgDatagram;

    /// Error type for [`Self::msg`].
    type MsgError: Debug + MaybeRecoverableError;

    /// The config required to construct this light client.
    type Config: Debug + Clone + PartialEq + Serialize + for<'de> Deserialize<'de>;

    // // i hate this
    // fn encode_client_state_for_counterparty<Tr: ChainExt>(cs: Tr::SelfClientState) -> Vec<u8>
    // where
    //     Tr::SelfClientState: Encode<Self::IbcStateEncoding>;
    // fn encode_consensus_state_for_counterparty<Tr: ChainExt>(cs: Tr::SelfConsensusState) -> Vec<u8>
    // where
    //     Tr::SelfConsensusState: Encode<Self::IbcStateEncoding>;

    fn do_fetch<Tr: ChainExt>(
        &self,
        msg: Self::Fetch<Tr>,
    ) -> impl Future<Output = Vec<RelayerMsg>> + '_
    where
        Self::Fetch<Tr>: DoFetch<Self>,
    {
        DoFetch::do_fetch(self, msg)
    }
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
    Event(AnyLightClientIdentified<AnyEvent>),
    // data that has been read
    Data(AnyLightClientIdentified<AnyData>),
    // read
    Fetch(AnyLightClientIdentified<AnyFetch>),
    // write
    Msg(AnyLightClientIdentified<AnyMsg>),
    Wait(AnyLightClientIdentified<AnyWait>),
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

pub trait GetChain<Hc: ChainExt> {
    fn get_chain(&self, chain_id: &ChainIdOf<Hc>) -> Hc;
}

impl RelayerMsg {
    // NOTE: Box is required bc recursion
    pub fn handle<G>(
        self,
        g: &G,
        depth: usize,
    ) -> BoxFuture<'_, Result<Vec<RelayerMsg>, Box<dyn std::error::Error>>>
    where
        G: Send + Sync + GetChain<Evm<Mainnet>> + GetChain<Evm<Minimal>> + GetChain<Wasm<Union>>,
    {
        tracing::info!(
            depth,
            %self,
            "handling message",
        );

        macro_rules! any_lc {
            (|$msg:ident| $expr:expr) => {
                match $msg {
                    AnyLightClientIdentified::EvmMainnetOnUnion($msg) => $expr,
                    AnyLightClientIdentified::UnionOnEvmMainnet($msg) => $expr,
                    AnyLightClientIdentified::EvmMinimalOnUnion($msg) => $expr,
                    AnyLightClientIdentified::UnionOnEvmMinimal($msg) => $expr,
                }
            };
        }

        async move {
            match self {
                RelayerMsg::Event(event) => any_lc! {
                    |event| Ok(event.data.handle(g.get_chain(&event.chain_id)))
                },
                RelayerMsg::Data(data) => {
                    tracing::error!(
                        data = %serde_json::to_string(&data).unwrap(),
                        "received data outside of an aggregation"
                    );

                    Ok([].into())
                }
                RelayerMsg::Fetch(fetch) => any_lc! {
                    |fetch| Ok(fetch.data.handle(g.get_chain(&fetch.chain_id)).await)
                },
                RelayerMsg::Msg(msg) => {
                    // NOTE: `Msg`s don't requeue any `RelayerMsg`s; they are side-effect only.
                    match msg {
  AnyLightClientIdentified::EvmMainnetOnUnion(msg) => DoMsg::msg(&GetChain::<Wasm<Union>>::get_chain(g, &msg.chain_id),msg.data).await?,
  AnyLightClientIdentified::EvmMinimalOnUnion(msg) => DoMsg::msg(&GetChain::<Wasm<Union>>::get_chain(g, &msg.chain_id),msg.data).await?,
  AnyLightClientIdentified::UnionOnEvmMainnet(msg) => DoMsg::msg(&GetChain::<Evm<Mainnet>>::get_chain(g, &msg.chain_id),msg.data).await?,
  AnyLightClientIdentified::UnionOnEvmMinimal(msg) => DoMsg::msg(&GetChain::<Evm<Minimal>>::get_chain(g, &msg.chain_id),msg.data).await?,

  };

                    Ok([].into())
                },
                RelayerMsg::Wait(wait) => any_lc! {
                    |wait| Ok(wait.data.handle(g.get_chain(&wait.chain_id)).await)
                },

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
                            AnyLightClientIdentified::EvmMainnetOnUnion(msg) => {
                                msg.handle(data)
                            }
                            AnyLightClientIdentified::EvmMinimalOnUnion(msg) => {
                                msg.handle(data)
                            }
                            AnyLightClientIdentified::UnionOnEvmMainnet(msg) => {
                                msg.handle(data)
                            }
                            AnyLightClientIdentified::UnionOnEvmMinimal(msg) => {
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

impl std::fmt::Display for RelayerMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelayerMsg::Event(event) => write!(f, "Event({event})"),
            RelayerMsg::Data(data) => write!(f, "Data({data})"),
            RelayerMsg::Fetch(fetch) => write!(f, "Fetch({fetch})"),
            RelayerMsg::Msg(msg) => write!(f, "Msg({msg})"),
            RelayerMsg::Wait(wait) => write!(f, "Wait({wait})"),
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
        pub enum $Enum<Hc: ChainExt, Tr: ChainExt> {
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
            type Inner<Hc: ChainExt, Tr: ChainExt> = $Enum<Hc, Tr>;
        }

        $(
            $(
                impl<Hc: ChainExt, Tr: ChainExt> TryFrom<$Enum<Hc, Tr>> for $VariantInner {
                    type Error = $Enum<Hc, Tr>;

                    fn try_from(value: $Enum<Hc, Tr>) -> Result<Self, Self::Error> {
                        match value {
                            $Enum::$Variant(t) => Ok(t),
                            _ => Err(value),
                        }
                    }
                }

                impl<Hc: ChainExt, Tr: ChainExt> From<$VariantInner> for $Enum<Hc, Tr> {
                    fn from(value: $VariantInner) -> Self {
                        Self::$Variant(value)
                    }
                }
            )?
        )+
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
        + for<'de> Deserialize<'de>;
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
#[serde(bound(serialize = "", deserialize = ""))]
#[allow(clippy::large_enum_variant)]
pub enum AnyLightClientIdentified<T: AnyLightClient> {
    // The 08-wasm client tracking the state of Evm<Mainnet>.
    #[display(fmt = "EvmMainnetOnUnion({}, {})", "_0.chain_id", "_0.data")]
    EvmMainnetOnUnion(Identified<Wasm<Union>, Evm<Mainnet>, InnerOf<T, Wasm<Union>, Evm<Mainnet>>>),
    // The solidity client on Evm<Mainnet> tracking the state of Wasm<Union>.
    #[display(fmt = "UnionOnEvmMainnet({}, {})", "_0.chain_id", "_0.data")]
    UnionOnEvmMainnet(Identified<Evm<Mainnet>, Wasm<Union>, InnerOf<T, Evm<Mainnet>, Wasm<Union>>>),

    // The 08-wasm client tracking the state of Evm<Minimal>.
    #[display(fmt = "EvmMinimalOnUnion({}, {})", "_0.chain_id", "_0.data")]
    EvmMinimalOnUnion(Identified<Wasm<Union>, Evm<Minimal>, InnerOf<T, Wasm<Union>, Evm<Minimal>>>),
    // The solidity client on Evm<Minimal> tracking the state of Wasm<Union>.
    #[display(fmt = "UnionOnEvmMinimal({}, {})", "_0.chain_id", "_0.data")]
    UnionOnEvmMinimal(Identified<Evm<Minimal>, Wasm<Union>, InnerOf<T, Evm<Minimal>, Wasm<Union>>>),
}

#[macro_export]
// TODO: Replace all uses of this with enumorph
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
#[serde(bound(
    serialize = "Data: ::serde::Serialize",
    deserialize = "Data: for<'d> Deserialize<'d>"
))]
// TODO: `Data: AnyLightClient`
// prerequisites: derive macro for AnyLightClient
pub struct Identified<Hc: Chain, Tr, Data> {
    pub chain_id: ChainIdOf<Hc>,
    pub data: Data,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> Tr>,
}

impl<Hc: Chain, Tr, Data: PartialEq> PartialEq for Identified<Hc, Tr, Data> {
    fn eq(&self, other: &Self) -> bool {
        self.chain_id == other.chain_id && self.data == other.data
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
        write!(f, "(chain id `{}`): {}", self.chain_id, self.data)
    }
}

impl<Hc: Chain, Tr, Data: Debug> Debug for Identified<Hc, Tr, Data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Identified")
            .field("chain_id", &self.chain_id)
            .field("data", &self.data)
            .finish()
    }
}

impl<Hc: Chain, Tr, Data: Clone> Clone for Identified<Hc, Tr, Data> {
    fn clone(&self) -> Self {
        Self {
            chain_id: self.chain_id.clone(),
            data: self.data.clone(),
            __marker: PhantomData,
        }
    }
}

impl<Hc: Chain, Tr, Data: Debug + Clone + PartialEq> Identified<Hc, Tr, Data> {
    pub fn new(chain_id: ChainIdOf<Hc>, data: Data) -> Self {
        Self {
            chain_id,
            data,
            __marker: PhantomData,
        }
    }
}

pub trait DoAggregate: Sized + Debug + Clone + PartialEq {
    fn do_aggregate(_: Self, _: VecDeque<AnyLightClientIdentified<AnyData>>) -> Vec<RelayerMsg>;
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

pub fn fetch<Hc: ChainExt, Tr: ChainExt>(
    chain_id: ChainIdOf<Hc>,
    t: impl Into<Fetch<Hc, Tr>>,
) -> RelayerMsg
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
{
    RelayerMsg::Fetch(AnyLightClientIdentified::from(Identified::new(
        chain_id,
        t.into(),
    )))
}

pub fn msg<Hc: ChainExt, Tr: ChainExt>(
    chain_id: ChainIdOf<Hc>,
    t: impl Into<Msg<Hc, Tr>>,
) -> RelayerMsg
where
    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<Hc, Tr>)>,
{
    RelayerMsg::Msg(AnyLightClientIdentified::from(Identified::new(
        chain_id,
        t.into(),
    )))
}

pub fn data<Hc: ChainExt, Tr: ChainExt>(
    chain_id: ChainIdOf<Hc>,
    t: impl Into<Data<Hc, Tr>>,
) -> RelayerMsg
where
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Hc, Tr>)>,
{
    RelayerMsg::Data(AnyLightClientIdentified::from(Identified::new(
        chain_id,
        t.into(),
    )))
}

pub fn wait<Hc: ChainExt, Tr: ChainExt>(
    chain_id: ChainIdOf<Hc>,
    t: impl Into<Wait<Hc, Tr>>,
) -> RelayerMsg
where
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Hc, Tr>)>,
{
    RelayerMsg::Wait(AnyLightClientIdentified::from(Identified::new(
        chain_id,
        t.into(),
    )))
}

pub fn event<Hc: ChainExt, Tr: ChainExt>(
    chain_id: ChainIdOf<Hc>,
    t: impl Into<Event<Hc, Tr>>,
) -> RelayerMsg
where
    AnyLightClientIdentified<AnyEvent>: From<identified!(Event<Hc, Tr>)>,
{
    RelayerMsg::Event(AnyLightClientIdentified::from(Identified::new(
        chain_id,
        t.into(),
    )))
}

pub fn aggregate<Hc: ChainExt, Tr: ChainExt>(
    chain_id: ChainIdOf<Hc>,
    t: impl Into<Aggregate<Hc, Tr>>,
) -> AnyLightClientIdentified<AnyAggregate>
where
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    AnyLightClientIdentified::from(Identified::new(chain_id, t.into()))
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

    use chain_utils::{evm::Evm, union::Union};
    use hex_literal::hex;
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
        proof::ConnectionPath,
        uint::U256,
        validated::ValidateT,
        EmptyString, QueryHeight, DELAY_PERIOD,
    };

    use crate::{
        aggregate,
        aggregate::{Aggregate, AggregateCreateClient, AnyAggregate},
        chain_impls::evm::EvmConfig,
        data::Data,
        defer_relative, event,
        event::{Event, IbcEvent},
        fetch,
        fetch::{AnyFetch, Fetch, FetchSelfClientState, FetchSelfConsensusState, FetchState},
        msg,
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

        print_json(msg::<Wasm<Union>, Evm<Minimal>>(
            union_chain_id.clone(),
            MsgConnectionOpenInitData {
                msg: MsgConnectionOpenInit {
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
                },
            },
        ));

        print_json(msg::<Wasm<Union>, Evm<Minimal>>(
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

        print_json(msg::<Evm<Minimal>, Wasm<Union>>(
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

        print_json(msg::<Evm<Minimal>, Wasm<Union>>(
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

        print_json(event::<Evm<Minimal>, Wasm<Union>>(
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
                event::<Evm<Minimal>, Wasm<Union>>(
                    eth_chain_id,
                    crate::event::Command::UpdateClient {
                        client_id: parse!("cometbls-0"),
                        counterparty_client_id: parse!("08-wasm-0"),
                    },
                ),
                defer_relative(10),
            ])),
        });

        print_json(RelayerMsg::Repeat {
            times: u64::MAX,
            msg: Box::new(seq([
                event::<Wasm<Union>, Evm<Minimal>>(
                    union_chain_id.clone(),
                    crate::event::Command::UpdateClient {
                        client_id: parse!("08-wasm-0"),
                        counterparty_client_id: parse!("cometbls-0"),
                    },
                ),
                defer_relative(10),
            ])),
        });

        println!("\ncreate client msgs\n");

        print_json(RelayerMsg::Sequence(
            [
                RelayerMsg::Aggregate {
                    queue: [
                        fetch::<Wasm<Union>, Evm<Minimal>>(
                            union_chain_id.clone(),
                            FetchSelfClientState {
                                at: QueryHeight::Latest,
                                __marker: PhantomData,
                            },
                        ),
                        fetch::<Wasm<Union>, Evm<Minimal>>(
                            union_chain_id.clone(),
                            FetchSelfConsensusState {
                                at: QueryHeight::Latest,
                                __marker: PhantomData,
                            },
                        ),
                    ]
                    .into(),
                    data: [].into_iter().collect(),
                    receiver: aggregate::<Evm<Minimal>, Wasm<Union>>(
                        eth_chain_id,
                        AggregateCreateClient {
                            config: EvmConfig {
                                client_type: "cometbls".to_string(),
                                client_address: H160(hex!(
                                    "83428c7db9815f482a39a1715684dcf755021997"
                                )),
                            },
                            __marker: PhantomData,
                        },
                    ),
                },
                RelayerMsg::Aggregate {
                    queue: [
                        fetch::<Evm<Minimal>, Wasm<Union>>(
                            eth_chain_id,
                            FetchSelfClientState {
                                at: QueryHeight::Latest,
                                __marker: PhantomData,
                            },
                        ),
                        fetch::<Evm<Minimal>, Wasm<Union>>(
                            eth_chain_id,
                            FetchSelfConsensusState {
                                at: QueryHeight::Latest,
                                __marker: PhantomData,
                            },
                        ),
                    ]
                    .into(),
                    data: [].into_iter().collect(),
                    receiver: aggregate::<Wasm<Union>, Evm<Minimal>>(
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
        print_json(fetch::<Wasm<Union>, Evm<Minimal>>(
            union_chain_id.clone(),
            FetchState {
                at: parse!("1-103"),
                path: ConnectionPath {
                    connection_id: parse!("connection-1"),
                }
                .into(),
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

    fn signers(&self) -> &chain_utils::Pool<unionlabs::CosmosAccountId> {
        self.0.signers()
    }

    // fn decode_client_state<Cs: Decode<Proto>>(bz: &[u8]) -> Cs {
    //     dbg!(serde_utils::to_hex(bz));
    //     <Any<wasm::client_state::ClientState<Cs>> as Decode<Proto>>::decode(bz)
    //         .unwrap()
    //         .0
    //         .data
    // }

    // fn decode_consensus_state<Cs: Decode<Proto>>(bz: &[u8]) -> Cs {
    //     <Any<wasm::consensus_state::ConsensusState<Cs>> as Decode<Proto>>::decode(bz)
    //         .unwrap()
    //         .0
    //         .data
    // }
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
pub struct WasmConfig {
    pub checksum: H256,
    // pub inner: T,
}

impl<Hc: CosmosSdkChain> Chain for Wasm<Hc> {
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
#[serde(bound(serialize = "", deserialize = ""))]
#[display(fmt = "{_0}")]
pub struct WasmDataMsg<Hc: ChainExt, Tr: ChainExt>(pub Hc::Data<Tr>);

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(bound(serialize = "", deserialize = ""))]
#[display(fmt = "{_0}")]
pub struct WasmFetchMsg<Hc: ChainExt, Tr: ChainExt>(pub Hc::Fetch<Tr>);

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(bound(serialize = "", deserialize = ""))]
#[display(fmt = "{_0}")]
pub struct WasmAggregateMsg<Hc: ChainExt, Tr: ChainExt>(pub Hc::Aggregate<Tr>);

impl<Hc: CosmosSdkChain + ChainExt, Tr: ChainExt> DoAggregate for identified!(WasmAggregateMsg<Hc, Tr>)
where
    Identified<Hc, Tr, Hc::Aggregate<Tr>>: DoAggregate,
{
    fn do_aggregate(i: Self, v: VecDeque<AnyLightClientIdentified<AnyData>>) -> Vec<RelayerMsg> {
        <Identified<_, _, Hc::Aggregate<Tr>>>::do_aggregate(
            Identified {
                chain_id: i.chain_id,
                data: i.data.0,
                __marker: PhantomData,
            },
            v,
        )
    }
}

// impl<Hc> ChainExt for Wasm<Hc>
// where
//     Self: Chain<ClientId = Hc::ClientId, ClientType = Hc::ClientType, Height = Hc::Height>,

//     Hc: ChainExt + CosmosSdkChain,
// {
//     type Data<Tr: ChainExt> = WasmDataMsg<Hc, Tr>;
//     type Fetch<Tr: ChainExt> = WasmFetchMsg<Hc, Tr>;
//     type Aggregate<Tr: ChainExt> = WasmAggregateMsg<Hc, Tr>;

//     type MsgError = Hc::MsgError;

//     type Config = WasmConfig;
// }

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

    Tr::StoredClientState<Wasm<Hc>>: IntoProto,
{
    async fn msg(&self, msg: Msg<Self, Tr>) -> Result<(), Self::MsgError> {
        self.0
            .signers()
            .with(|signer| async {
                let msg_any = match msg {
                    Msg::ConnectionOpenInit(data) => {
                        mk_any(&protos::ibc::core::connection::v1::MsgConnectionOpenInit {
                            client_id: data.msg.client_id.to_string(),
                            counterparty: Some(data.msg.counterparty.into()),
                            version: Some(data.msg.version.into()),
                            signer: signer.to_string(),
                            delay_period: data.msg.delay_period,
                        })
                    }
                    Msg::ConnectionOpenTry(data) =>
                    {
                        #[allow(deprecated)]
                        mk_any(&protos::ibc::core::connection::v1::MsgConnectionOpenTry {
                            client_id: data.msg.client_id.to_string(),
                            previous_connection_id: String::new(),
                            client_state: Some(dbg!(Any(wasm::client_state::ClientState {
                                latest_height: data.msg.client_state.height().into(),
                                data: data.msg.client_state,
                                checksum: H256::default(),
                            })
                            .into())),
                            counterparty: Some(data.msg.counterparty.into()),
                            delay_period: data.msg.delay_period,
                            counterparty_versions: data
                                .msg
                                .counterparty_versions
                                .into_iter()
                                .map(Into::into)
                                .collect(),
                            proof_height: Some(data.msg.proof_height.into_height().into()),
                            proof_init: data.msg.proof_init,
                            proof_client: data.msg.proof_client,
                            proof_consensus: data.msg.proof_consensus,
                            consensus_height: Some(data.msg.consensus_height.into_height().into()),
                            signer: signer.to_string(),
                            host_consensus_state_proof: vec![],
                        })
                    }
                    Msg::ConnectionOpenAck(data) => {
                        mk_any(&protos::ibc::core::connection::v1::MsgConnectionOpenAck {
                            client_state: Some(
                                Any(wasm::client_state::ClientState {
                                    latest_height: data.msg.client_state.height().into(),
                                    data: data.msg.client_state,
                                    checksum: H256::default(),
                                })
                                .into(),
                            ),
                            proof_height: Some(data.msg.proof_height.into_height().into()),
                            proof_client: data.msg.proof_client,
                            proof_consensus: data.msg.proof_consensus,
                            consensus_height: Some(data.msg.consensus_height.into_height().into()),
                            signer: signer.to_string(),
                            host_consensus_state_proof: vec![],
                            connection_id: data.msg.connection_id.to_string(),
                            counterparty_connection_id: data
                                .msg
                                .counterparty_connection_id
                                .to_string(),
                            version: Some(data.msg.version.into()),
                            proof_try: data.msg.proof_try,
                        })
                    }
                    Msg::ConnectionOpenConfirm(data) => mk_any(
                        &protos::ibc::core::connection::v1::MsgConnectionOpenConfirm {
                            connection_id: data.msg.connection_id.to_string(),
                            proof_ack: data.msg.proof_ack,
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
                            proof_init: data.msg.proof_init,
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
                            proof_try: data.msg.proof_try,
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
                            proof_ack: data.msg.proof_ack,
                        })
                    }
                    Msg::RecvPacket(data) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgRecvPacket {
                            packet: Some(data.msg.packet.into()),
                            proof_height: Some(data.msg.proof_height.into()),
                            signer: signer.to_string(),
                            proof_commitment: data.msg.proof_commitment,
                        })
                    }
                    Msg::AckPacket(data) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgAcknowledgement {
                            packet: Some(data.msg.packet.into()),
                            acknowledgement: data.msg.acknowledgement,
                            proof_acked: data.msg.proof_acked,
                            proof_height: Some(data.msg.proof_height.into()),
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
                    Msg::UpdateClient(data) => {
                        mk_any(&protos::ibc::core::client::v1::MsgUpdateClient {
                            signer: signer.to_string(),
                            client_id: data.msg.client_id.to_string(),
                            client_message: Some(
                                Any(wasm::client_message::ClientMessage {
                                    data: data.msg.client_message,
                                })
                                .into(),
                            ),
                        })
                    }
                };

                broadcast_tx_commit(&self.0, signer, [msg_any])
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
