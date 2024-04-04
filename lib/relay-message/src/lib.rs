#![feature(trait_alias)]
#![allow(clippy::type_complexity, async_fn_in_trait)]

use std::{collections::VecDeque, fmt::Debug, future::Future, marker::PhantomData};

use chain_utils::{
    cosmos::Cosmos,
    cosmos_sdk::{BroadcastTxCommitError, CosmosSdkChain, CosmosSdkChainExt},
    ethereum::Ethereum,
    scroll::Scroll,
    union::Union,
    wasm::Wasm,
    Chains,
};
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use queue_msg::{seq, QueueMsg, QueueMsgTypes, QueueMsgTypesTraits};
use serde::{Deserialize, Serialize};
use unionlabs::{
    encoding::{Encode, Proto},
    ethereum::config::{Mainnet, Minimal},
    google::protobuf::any::{mk_any, Any, IntoAny},
    hash::H256,
    ibc::{core::client::height::IsHeight, lightclients::wasm},
    never::Never,
    proof,
    traits::{
        Chain, ChainIdOf, ClientIdOf, ClientState, ClientStateOf, ConsensusStateOf, HeaderOf,
        HeightOf,
    },
    MaybeArbitrary, MaybeRecoverableError, TypeUrl,
};

use crate::{
    aggregate::AnyAggregate,
    data::{AnyData, Data},
    effect::{
        AnyEffect, Effect, MsgConnectionOpenAckData, MsgConnectionOpenInitData,
        MsgConnectionOpenTryData, MsgUpdateClientData,
    },
    event::AnyEvent,
    fetch::{AnyFetch, DoFetch, Fetch, FetchUpdateHeaders},
    wait::{AnyWait, Wait},
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
        + Clone
        + PartialEq
        + Serialize
        + for<'de> Deserialize<'de>
        + MaybeArbitrary;
}

pub type InnerOf<T, Hc, Tr> = <T as AnyLightClient>::Inner<Hc, Tr>;

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, enumorph::Enumorph,
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
    EthereumMainnetOnUnion(
        Identified<Wasm<Union>, Ethereum<Mainnet>, InnerOf<T, Wasm<Union>, Ethereum<Mainnet>>>,
    ),
    // The solidity client on Ethereum<Mainnet> tracking the state of Wasm<Union>.
    UnionOnEthereumMainnet(
        Identified<Ethereum<Mainnet>, Wasm<Union>, InnerOf<T, Ethereum<Mainnet>, Wasm<Union>>>,
    ),

    // The 08-wasm client tracking the state of Ethereum<Minimal>.
    EthereumMinimalOnUnion(
        Identified<Wasm<Union>, Ethereum<Minimal>, InnerOf<T, Wasm<Union>, Ethereum<Minimal>>>,
    ),
    // The solidity client on Ethereum<Minimal> tracking the state of Wasm<Union>.
    UnionOnEthereumMinimal(
        Identified<Ethereum<Minimal>, Wasm<Union>, InnerOf<T, Ethereum<Minimal>, Wasm<Union>>>,
    ),

    // The 08-wasm client tracking the state of Scroll.
    ScrollOnUnion(Identified<Wasm<Union>, Scroll, InnerOf<T, Wasm<Union>, Scroll>>),
    // The solidity client on Scroll tracking the state of Wasm<Union>.
    UnionOnScroll(Identified<Scroll, Wasm<Union>, InnerOf<T, Scroll, Wasm<Union>>>),

    CosmosOnUnion(Identified<Union, Wasm<Cosmos>, InnerOf<T, Union, Wasm<Cosmos>>>),
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

    ScrollOnUnion(
        Inner<
            Wasm<Union>,
            Scroll,
            Identified<Wasm<Union>, Scroll, InnerOf<T, Wasm<Union>, Scroll>>,
        >,
    ),
    UnionOnScroll(
        Inner<
            Scroll,
            Wasm<Union>,
            Identified<Scroll, Wasm<Union>, InnerOf<T, Scroll, Wasm<Union>>>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct WasmConfig {
    pub checksum: H256,
    // pub inner: T,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), transparent)]
pub struct WasmDataMsg<Hc: ChainExt, Tr: ChainExt>(pub Hc::Data<Tr>);

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), transparent)]
pub struct WasmFetchMsg<Hc: ChainExt, Tr: ChainExt>(pub Hc::Fetch<Tr>);

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), transparent)]
pub struct WasmAggregateMsg<Hc: ChainExt, Tr: ChainExt>(pub Hc::Aggregate<Tr>);

impl<Hc: CosmosSdkChain + ChainExt, Tr: ChainExt> DoAggregate for identified!(WasmAggregateMsg<Hc, Tr>)
where
    Identified<Hc, Tr, Hc::Aggregate<Tr>>: DoAggregate,
{
    fn do_aggregate(
        i: Self,
        v: VecDeque<AnyLightClientIdentified<AnyData>>,
    ) -> QueueMsg<RelayMessageTypes> {
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

    ConsensusStateOf<Tr>: Encode<Proto> + TypeUrl,
    ClientStateOf<Tr>: Encode<Proto> + TypeUrl,
    HeaderOf<Tr>: Encode<Proto> + TypeUrl,

    ConsensusStateOf<Hc>: Encode<Proto> + TypeUrl,

    ClientStateOf<Hc>: Encode<Proto> + TypeUrl,

    // TODO: Move this associated type to this trait
    Wasm<Hc>: ChainExt<
        SelfClientState = Hc::SelfClientState,
        SelfConsensusState = Hc::SelfConsensusState,
        MsgError = BroadcastTxCommitError,
        Config = WasmConfig,
    >,

    Tr::StoredClientState<Wasm<Hc>>: Encode<Proto> + IntoAny,
    Tr::StateProof: Encode<Proto>,
{
    async fn msg(&self, msg: Effect<Self, Tr>) -> Result<(), Self::MsgError> {
        self.0
            .signers()
            .with(|signer| async {
                let msg_any = match msg.clone() {
                    Effect::ConnectionOpenInit(MsgConnectionOpenInitData(data)) => {
                        mk_any(&protos::ibc::core::connection::v1::MsgConnectionOpenInit {
                            client_id: data.client_id.to_string(),
                            counterparty: Some(data.counterparty.into()),
                            version: Some(data.version.into()),
                            signer: signer.to_string(),
                            delay_period: data.delay_period,
                        })
                    }
                    Effect::ConnectionOpenTry(MsgConnectionOpenTryData(data)) =>
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
                    Effect::ConnectionOpenAck(MsgConnectionOpenAckData(data)) => {
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
                    Effect::ConnectionOpenConfirm(data) => mk_any(
                        &protos::ibc::core::connection::v1::MsgConnectionOpenConfirm {
                            connection_id: data.msg.connection_id.to_string(),
                            proof_ack: data.msg.proof_ack.encode(),
                            proof_height: Some(data.msg.proof_height.into_height().into()),
                            signer: signer.to_string(),
                        },
                    ),
                    Effect::ChannelOpenInit(data) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgChannelOpenInit {
                            port_id: data.msg.port_id.to_string(),
                            channel: Some(data.msg.channel.into()),
                            signer: signer.to_string(),
                        })
                    }
                    Effect::ChannelOpenTry(data) =>
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
                    Effect::ChannelOpenAck(data) => {
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
                    Effect::ChannelOpenConfirm(data) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgChannelOpenConfirm {
                            port_id: data.msg.port_id.to_string(),
                            channel_id: data.msg.channel_id.to_string(),
                            proof_height: Some(data.msg.proof_height.into_height().into()),
                            signer: signer.to_string(),
                            proof_ack: data.msg.proof_ack.encode(),
                        })
                    }
                    Effect::RecvPacket(data) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgRecvPacket {
                            packet: Some(data.msg.packet.into()),
                            proof_height: Some(data.msg.proof_height.into_height().into()),
                            signer: signer.to_string(),
                            proof_commitment: data.msg.proof_commitment.encode(),
                        })
                    }
                    Effect::AckPacket(data) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgAcknowledgement {
                            packet: Some(data.msg.packet.into()),
                            acknowledgement: data.msg.acknowledgement,
                            proof_acked: data.msg.proof_acked.encode(),
                            proof_height: Some(data.msg.proof_height.into_height().into()),
                            signer: signer.to_string(),
                        })
                    }
                    Effect::CreateClient(data) => {
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
                    Effect::UpdateClient(MsgUpdateClientData(data)) => {
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

                let tx_hash = self.0.broadcast_tx_commit(signer, [msg_any]).await?;

                tracing::info!("cosmos tx {:?} => {:?}", tx_hash, msg);

                Ok(())
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
    fn proof(
        hc: &Self,
        at: HeightOf<Self>,
        path: PathOf<Wasm<Hc>, Tr>,
    ) -> QueueMsg<RelayMessageTypes> {
        Hc::proof(hc, at, path)
    }
}

impl<Hc: ChainExt + CosmosSdkChain + DoFetchState<Wasm<Hc>, Tr>, Tr: ChainExt>
    DoFetchState<Self, Tr> for Wasm<Hc>
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Wasm<Hc>, Tr>)>,
    Wasm<Hc>: ChainExt,
{
    fn state(
        hc: &Self,
        at: HeightOf<Self>,
        path: PathOf<Wasm<Hc>, Tr>,
    ) -> QueueMsg<RelayMessageTypes> {
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
    fn fetch_update_headers(
        hc: &Self,
        update_info: FetchUpdateHeaders<Self, Tr>,
    ) -> QueueMsg<RelayMessageTypes> {
        Hc::fetch_update_headers(
            hc,
            FetchUpdateHeaders {
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
