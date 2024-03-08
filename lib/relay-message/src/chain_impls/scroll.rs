use chain_utils::{
    evm::{HasIbcHandler, IbcHandlerExt},
    scroll::Scroll,
};
use enumorph::Enumorph;
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use futures::FutureExt;
use queue_msg::{aggregate, data, fetch, QueueMsg};
use serde::{Deserialize, Serialize};
use unionlabs::{
    encoding::{Decode, Encode, EthAbi},
    ethereum::config::Mainnet,
    never::Never,
    proof::{ClientStatePath, Path},
    traits::{Chain, ClientStateOf, ConsensusStateOf, HeaderOf, HeightOf, IbcStateEncodingOf},
    IntoEthAbi,
};

use crate::{
    aggregate::{Aggregate, AnyAggregate},
    chain_impls::evm::{do_get_proof, do_msg, EvmConfig, FetchIbcState, GetProof, TxSubmitError},
    data::{AnyData, Data, IbcState, LightClientSpecificData},
    fetch::{AnyFetch, DoFetch, Fetch, FetchUpdateHeaders, LightClientSpecificFetch},
    id, identified,
    msg::Msg,
    AnyLightClientIdentified, ChainExt, DoFetchProof, DoFetchState, DoFetchUpdateHeaders, DoMsg,
    PathOf, RelayerMsgTypes,
};

impl ChainExt for Scroll {
    type Data<Tr: ChainExt> = Never;
    type Fetch<Tr: ChainExt> = ScrollFetchMsg<Tr>;
    type Aggregate<Tr: ChainExt> = Never;

    type MsgError = TxSubmitError;

    type Config = EvmConfig;
}

impl<Tr: ChainExt> DoMsg<Self, Tr> for Scroll
where
    ConsensusStateOf<Tr>: IntoEthAbi,
    ClientStateOf<Tr>: IntoEthAbi,
    HeaderOf<Tr>: IntoEthAbi,

    ClientStateOf<Scroll>: Encode<Tr::IbcStateEncoding>,
    Tr::StoredClientState<Scroll>: Encode<Tr::IbcStateEncoding>,
    Tr::StateProof: Encode<EthAbi>,
{
    async fn msg(&self, msg: Msg<Self, Tr>) -> Result<(), Self::MsgError> {
        do_msg(&self.ibc_handlers, msg).await
    }
}

impl<Tr: ChainExt> DoFetchProof<Self, Tr> for Scroll
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Scroll, Tr>)>,
{
    fn proof(c: &Self, at: HeightOf<Self>, path: PathOf<Scroll, Tr>) -> QueueMsg<RelayerMsgTypes> {
        fetch(id::<Self, Tr, _>(
            c.chain_id(),
            LightClientSpecificFetch::<Self, Tr>(ScrollFetchMsg::from(GetProof {
                path,
                height: at,
            })),
        ))
    }
}

impl<Tr: ChainExt> DoFetchState<Self, Tr> for Scroll
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Scroll, Tr>)>,
    Tr::SelfClientState: Decode<IbcStateEncodingOf<Scroll>>,

    Tr::SelfClientState: Encode<EthAbi>,
    Tr::SelfClientState: unionlabs::EthAbi,
    Tr::SelfClientState: TryFrom<<Tr::SelfClientState as unionlabs::EthAbi>::EthAbi>,
    <Tr::SelfClientState as unionlabs::EthAbi>::EthAbi: From<Tr::SelfClientState>,
{
    fn state(hc: &Self, at: HeightOf<Self>, path: PathOf<Scroll, Tr>) -> QueueMsg<RelayerMsgTypes> {
        fetch(id::<Self, Tr, _>(
            hc.chain_id(),
            LightClientSpecificFetch::<Self, Tr>(ScrollFetchMsg::from(FetchIbcState {
                path,
                height: at,
            })),
        ))
    }

    async fn query_client_state(
        hc: &Self,
        client_id: Self::ClientId,
        height: Self::Height,
    ) -> Tr::SelfClientState {
        hc.ibc_handler()
            .ibc_state_read::<_, Scroll, Tr>(
                hc.batch_index_of_beacon_height(height)
                    .then(|bi| hc.scroll_height_of_batch_index(bi))
                    .await,
                ClientStatePath { client_id },
            )
            .await
            .unwrap()
    }
}

impl<Tr: ChainExt> DoFetchUpdateHeaders<Self, Tr> for Scroll
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Scroll, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Scroll, Tr>)>,
{
    fn fetch_update_headers(
        c: &Self,
        update_info: FetchUpdateHeaders<Self, Tr>,
    ) -> QueueMsg<RelayerMsgTypes> {
        todo!()
    }
}

impl<Tr: ChainExt> DoFetch<Scroll> for ScrollFetchMsg<Tr>
where
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Scroll, Tr>)>,

    Tr::SelfClientState: Decode<IbcStateEncodingOf<Scroll>>,
    Tr::SelfConsensusState: Decode<IbcStateEncodingOf<Scroll>>,

    Tr::SelfClientState: unionlabs::EthAbi,
    <Tr::SelfClientState as unionlabs::EthAbi>::EthAbi: From<Tr::SelfClientState>,
{
    async fn do_fetch(scroll: &Scroll, msg: Self) -> QueueMsg<RelayerMsgTypes> {
        let msg = match msg {
            ScrollFetchMsg::FetchGetProof(GetProof { path, height }) => {
                let execution_height = scroll
                    .scroll_height_of_batch_index(scroll.batch_index_of_beacon_height(height).await)
                    .await;

                return do_get_proof(
                    path,
                    scroll.ibc_handler_address.clone(),
                    scroll.chain_id(),
                    &scroll.provider,
                    execution_height,
                    height,
                )
                .await;
            }
            ScrollFetchMsg::FetchIbcState(get_storage_at) => {
                return match get_storage_at.path {
                    Path::ClientStatePath(path) => data(id::<Scroll, Tr, _>(
                        scroll.chain_id,
                        IbcState {
                            state: scroll
                                .ibc_state_read::<Mainnet, _, Tr>(
                                    get_storage_at.height,
                                    path.clone(),
                                )
                                .await
                                .unwrap(),
                            height: get_storage_at.height,
                            path,
                        },
                    )),
                    Path::ClientConsensusStatePath(path) => data(id::<Scroll, Tr, _>(
                        scroll.chain_id,
                        IbcState {
                            state: scroll
                                .ibc_state_read::<Mainnet, _, Tr>(
                                    get_storage_at.height,
                                    path.clone(),
                                )
                                .await
                                .unwrap(),
                            height: get_storage_at.height,
                            path,
                        },
                    )),
                    Path::ConnectionPath(path) => data(id::<Scroll, Tr, _>(
                        scroll.chain_id,
                        IbcState {
                            state: scroll
                                .ibc_state_read::<Mainnet, _, Tr>(
                                    get_storage_at.height,
                                    path.clone(),
                                )
                                .await
                                .unwrap(),
                            height: get_storage_at.height,
                            path,
                        },
                    )),
                    Path::ChannelEndPath(path) => data(id::<Scroll, Tr, _>(
                        scroll.chain_id,
                        IbcState {
                            state: scroll
                                .ibc_state_read::<Mainnet, _, Tr>(
                                    get_storage_at.height,
                                    path.clone(),
                                )
                                .await
                                .unwrap(),
                            height: get_storage_at.height,
                            path,
                        },
                    )),
                    Path::CommitmentPath(path) => data(id::<Scroll, Tr, _>(
                        scroll.chain_id,
                        IbcState {
                            state: scroll
                                .ibc_state_read::<Mainnet, _, Tr>(
                                    get_storage_at.height,
                                    path.clone(),
                                )
                                .await
                                .unwrap(),
                            height: get_storage_at.height,
                            path,
                        },
                    )),
                    Path::AcknowledgementPath(path) => data(id::<Scroll, Tr, _>(
                        scroll.chain_id,
                        IbcState {
                            state: scroll
                                .ibc_state_read::<Mainnet, _, Tr>(
                                    get_storage_at.height,
                                    path.clone(),
                                )
                                .await
                                .unwrap(),
                            height: get_storage_at.height,
                            path,
                        },
                    )),
                };
            }
        };

        data(id::<Scroll, Tr, _>(
            scroll.chain_id,
            LightClientSpecificData(msg),
        ))
    }
}

#[derive(
    DebugNoBound,
    CloneNoBound,
    PartialEqNoBound,
    Serialize,
    Deserialize,
    derive_more::Display,
    Enumorph,
)]
#[serde(
    bound(serialize = "", deserialize = ""),
    tag = "@type",
    content = "@value",
    rename_all = "snake_case"
)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "")
)]
pub enum ScrollFetchMsg<Tr: ChainExt> {
    #[display(fmt = "GetProof::{}", "_0.path")]
    FetchGetProof(GetProof<Mainnet, Tr>),
    #[display(fmt = "IbcState::{}", "_0.path")]
    FetchIbcState(FetchIbcState<Mainnet, Tr>),
}
