use chain_utils::GetChain;
use futures::Future;
use macros::apply;
use queue_msg::{queue_msg, HandleEffect, Op, QueueError, QueueMessage};
use tracing::instrument;
use unionlabs::{
    ibc::core::{
        channel::{
            msg_acknowledgement::MsgAcknowledgement, msg_channel_open_ack::MsgChannelOpenAck,
            msg_channel_open_confirm::MsgChannelOpenConfirm,
            msg_channel_open_init::MsgChannelOpenInit, msg_channel_open_try::MsgChannelOpenTry,
            msg_recv_packet::MsgRecvPacket, msg_timeout::MsgTimeout,
        },
        client::{msg_create_client::MsgCreateClient, msg_update_client::MsgUpdateClient},
        connection::{
            msg_connection_open_ack::MsgConnectionOpenAck,
            msg_connection_open_confirm::MsgConnectionOpenConfirm,
            msg_connection_open_init::MsgConnectionOpenInit,
            msg_connection_open_try::MsgConnectionOpenTry,
        },
    },
    id::ConnectionId,
    traits::{ClientIdOf, ClientStateOf, ConsensusStateOf, HeaderOf, HeightOf},
    MaybeRecoverableError,
};

use crate::{any_enum, any_lc, AnyLightClientIdentified, ChainExt, DoMsg, RelayMessage};

#[apply(any_enum)]
#[any = AnyEffect]
pub enum Effect<Hc: ChainExt, Tr: ChainExt> {
    ConnectionOpenInit(MsgConnectionOpenInitData<Hc, Tr>),
    ConnectionOpenTry(MsgConnectionOpenTryData<Hc, Tr>),
    ConnectionOpenAck(MsgConnectionOpenAckData<Hc, Tr>),
    ConnectionOpenConfirm(MsgConnectionOpenConfirmData<Hc, Tr>),

    ChannelOpenInit(MsgChannelOpenInitData<Hc, Tr>),
    ChannelOpenTry(MsgChannelOpenTryData<Hc, Tr>),
    ChannelOpenAck(MsgChannelOpenAckData<Hc, Tr>),
    ChannelOpenConfirm(MsgChannelOpenConfirmData<Hc, Tr>),

    RecvPacket(MsgRecvPacketData<Hc, Tr>),
    AckPacket(MsgAckPacketData<Hc, Tr>),
    TimeoutPacket(MsgTimeoutData<Hc, Tr>),

    CreateClient(MsgCreateClientData<Hc, Tr>),
    UpdateClient(MsgUpdateClientData<Hc, Tr>),

    Batch(BatchMsg<Hc, Tr>),
}

impl HandleEffect<RelayMessage> for AnyLightClientIdentified<AnyEffect> {
    #[instrument(skip_all, fields(chain_id = %self.chain_id()))]
    #[allow(clippy::manual_async_fn)]
    fn handle(
        self,
        store: &<RelayMessage as QueueMessage>::Store,
    ) -> impl Future<Output = Result<Op<RelayMessage>, QueueError>> + Send {
        async move {
            let msg = self;

            any_lc! {
                |msg| {
                    store
                        .with_chain(&msg.chain_id, move |c| async move { msg.t.handle(&c).await })
                        .map_err(|e| QueueError::Fatal(Box::new(e)))?
                        .await
                        .map_err(|e: <Hc as ChainExt>::MsgError| {
                            if e.is_recoverable() {
                                QueueError::Retry(Box::new(e))
                            } else {
                                QueueError::Fatal(Box::new(e))
                            }
                        })
                }
            }
        }
    }
}

impl<Hc, Tr> Effect<Hc, Tr>
where
    Hc: ChainExt + DoMsg<Hc, Tr>,
    Tr: ChainExt,
{
    pub fn handle(
        self,
        c: &Hc,
    ) -> impl Future<Output = Result<Op<RelayMessage>, Hc::MsgError>> + Send + '_ {
        <Hc as DoMsg<Hc, Tr>>::msg(c, self)
    }
}

#[queue_msg]
pub struct MsgConnectionOpenInitData<Hc: ChainExt, Tr: ChainExt>(
    pub MsgConnectionOpenInit<ClientIdOf<Hc>, ClientIdOf<Tr>>,
);

#[queue_msg]
pub struct MsgConnectionOpenTryData<Hc: ChainExt, Tr: ChainExt>(
    pub  MsgConnectionOpenTry<
        Tr::StoredClientState<Hc>,
        ClientIdOf<Hc>,
        ClientIdOf<Tr>,
        ConnectionId,
        Tr::Height,
        Hc::Height,
        Tr::StateProof,
        Tr::StateProof,
        Tr::StateProof,
    >,
);

#[queue_msg]
pub struct MsgConnectionOpenAckData<Hc: ChainExt, Tr: ChainExt>(
    pub  MsgConnectionOpenAck<
        Tr::StoredClientState<Hc>,
        Tr::StateProof,
        Tr::StateProof,
        Tr::StateProof,
    >,
);

#[queue_msg]
pub struct MsgConnectionOpenConfirmData<#[cover] Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgConnectionOpenConfirm<HeightOf<Tr>, Tr::StateProof>,
}

#[queue_msg]
pub struct MsgChannelOpenInitData<#[cover] Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub msg: MsgChannelOpenInit,
}

#[queue_msg]
pub struct MsgChannelOpenTryData<#[cover] Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgChannelOpenTry<Tr::StateProof>,
}

#[queue_msg]
pub struct MsgChannelOpenAckData<#[cover] Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgChannelOpenAck<Tr::StateProof, Tr::Height>,
}

#[queue_msg]
pub struct MsgChannelOpenConfirmData<#[cover] Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgChannelOpenConfirm<Tr::StateProof>,
}

#[queue_msg]
pub struct MsgRecvPacketData<#[cover] Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgRecvPacket<Tr::StateProof, Tr::Height>,
}

#[queue_msg]
pub struct MsgAckPacketData<#[cover] Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgAcknowledgement<Tr::StateProof, Tr::Height>,
}

#[queue_msg]
pub struct MsgTimeoutData<#[cover] Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgTimeout<Tr::StateProof, Tr::Height>,
}

#[queue_msg]
pub struct MsgCreateClientData<Hc: ChainExt, Tr: ChainExt> {
    pub config: Hc::Config,
    pub msg: MsgCreateClient<ClientStateOf<Tr>, ConsensusStateOf<Tr>>,
}

#[queue_msg]
pub struct MsgUpdateClientData<Hc: ChainExt, Tr: ChainExt>(
    pub MsgUpdateClient<ClientIdOf<Hc>, HeaderOf<Tr>>,
);

#[queue_msg]
#[debug(bound())] // break cyclic debug bounds
pub struct BatchMsg<Hc: ChainExt, Tr: ChainExt>(pub Vec<Effect<Hc, Tr>>);
