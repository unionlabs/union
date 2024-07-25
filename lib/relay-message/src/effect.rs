use chain_utils::GetChain;
use futures::Future;
use macros::apply;
use queue_msg::{queue_msg, HandleEffect, Op, QueueError, QueueMessage};
use tracing::{error, info, instrument};
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
    traits::{
        ClientIdOf, ClientState, ClientStateOf, ConsensusState, ConsensusStateOf, Header, HeaderOf,
        HeightOf,
    },
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

pub fn log_msg<Hc: ChainExt, Tr: ChainExt>(effect: Effect<Hc, Tr>) {
    match effect.clone() {
        Effect::ConnectionOpenInit(MsgConnectionOpenInitData(data)) => {
            info!(
                client_id = %data.client_id,
                counterparty.client_id = %data.counterparty.client_id,
                counterparty.connection_id = %data.counterparty.connection_id,
                counterparty.prefix.key_prefix = %::serde_utils::to_hex(data.counterparty.prefix.key_prefix),
                version.identifier = %data.version.identifier,
                version.features = %data.version.features.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
                delay_period = %data.delay_period,
            )
        }
        Effect::ConnectionOpenTry(MsgConnectionOpenTryData(data)) => {
            info!(
                client_id = %data.client_id.to_string(),
                client_state.height = %data.client_state.height(),
                counterparty.client_id = %data.counterparty.client_id,
                counterparty.connection_id = %data.counterparty.connection_id,
                counterparty.prefix.key_prefix = %::serde_utils::to_hex(data.counterparty.prefix.key_prefix),
                delay_period = %data.delay_period,
                // counterparty_versions = %data
                //     .counterparty_versions
                //     .into_iter()
                //     .map(Into::into)
                //     .collect(),
                proof_height = %data.proof_height,
                consensus_height = %data.consensus_height,
            )
        }
        Effect::ConnectionOpenAck(MsgConnectionOpenAckData(data)) => {
            info!(
                client_state.height = %data.client_state.height(),
                proof_height = %data.proof_height,
                consensus_height = %data.consensus_height,
                connection_id = %data.connection_id,
                counterparty_connection_id = %data.counterparty_connection_id,
                version.identifier = %data.version.identifier,
                version.features = %data.version.features.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
            )
        }
        Effect::ConnectionOpenConfirm(MsgConnectionOpenConfirmData { msg, __marker }) => {
            info!(
                connection_id = %msg.connection_id,
                proof_height = %msg.proof_height,
            )
        }
        Effect::ChannelOpenInit(MsgChannelOpenInitData { msg, __marker }) => {
            info!(
                port_id = %msg.port_id,

                channel.state = %msg.channel.state,
                channel.ordering = %msg.channel.ordering,
                channel.counterparty.port_id = %msg.channel.counterparty.port_id,
                channel.counterparty.channel_id = %msg.channel.counterparty.channel_id,
                channel.connection_hops = %msg.channel.connection_hops.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
                channel.version = %msg.channel.version,
            )
        }
        Effect::ChannelOpenTry(MsgChannelOpenTryData { msg, __marker }) => {
            info!(
                port_id = %msg.port_id.to_string(),

                channel.state = %msg.channel.state,
                channel.ordering = %msg.channel.ordering,
                channel.counterparty.port_id = %msg.channel.counterparty.port_id,
                channel.counterparty.channel_id = %msg.channel.counterparty.channel_id,
                channel.connection_hops = %msg.channel.connection_hops.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
                channel.version = %msg.channel.version,

                counterparty_version = %msg.counterparty_version,
                proof_height = %msg.proof_height,
            )
        }
        Effect::ChannelOpenAck(MsgChannelOpenAckData { msg, __marker }) => {
            info!(
                port_id = %msg.port_id,
                channel_id = %msg.channel_id,
                counterparty_version = %msg.counterparty_version,
                counterparty_channel_id = %msg.counterparty_channel_id,
                proof_height = %msg.proof_height,
            )
        }
        Effect::ChannelOpenConfirm(MsgChannelOpenConfirmData { msg, __marker }) => {
            info!(
                port_id = %msg.port_id,
                channel_id = %msg.channel_id,
                proof_height = %msg.proof_height,
            )
        }
        Effect::RecvPacket(MsgRecvPacketData { msg, __marker }) => {
            info!(
                sequence = %msg.packet.sequence,
                source_port = %msg.packet.source_port,
                source_channel = %msg.packet.source_channel,
                destination_port = %msg.packet.destination_port,
                destination_channel = %msg.packet.destination_channel,
                data = %::serde_utils::to_hex(msg.packet.data),
                timeout_height = %msg.packet.timeout_height,
                timeout_timestamp = %msg.packet.timeout_timestamp,

                proof_height = %msg.proof_height,
            )
        }
        Effect::AckPacket(MsgAckPacketData { msg, __marker }) => {
            info!(
                sequence = %msg.packet.sequence,
                source_port = %msg.packet.source_port,
                source_channel = %msg.packet.source_channel,
                destination_port = %msg.packet.destination_port,
                destination_channel = %msg.packet.destination_channel,
                data = %::serde_utils::to_hex(msg.packet.data),
                timeout_height = %msg.packet.timeout_height,
                timeout_timestamp = %msg.packet.timeout_timestamp,

                data = %::serde_utils::to_hex(msg.acknowledgement),
                proof_height = %msg.proof_height,
            )
        }
        Effect::TimeoutPacket(MsgTimeoutData { msg, __marker }) => {
            info!(
                sequence = %msg.packet.sequence,
                source_port = %msg.packet.source_port,
                source_channel = %msg.packet.source_channel,
                destination_port = %msg.packet.destination_port,
                destination_channel = %msg.packet.destination_channel,
                data = %::serde_utils::to_hex(msg.packet.data),
                timeout_height = %msg.packet.timeout_height,
                timeout_timestamp = %msg.packet.timeout_timestamp,

                proof_height = %msg.proof_height,
                next_sequence_recv = %msg.next_sequence_recv.get(),
            )
        }
        Effect::CreateClient(MsgCreateClientData { msg, config }) => {
            info!(
                config_json = %::serde_json::to_string(&config).expect("serialization is infallible"),
                client_state.height = %msg.client_state.height(),
                client_state.chain_id = %msg.client_state.chain_id(),
                consensus_state.timestamp = %msg.consensus_state.timestamp(),
            )
        }
        Effect::UpdateClient(MsgUpdateClientData(msg)) => {
            info!(
                client_id = %msg.client_id.to_string(),
                header.trusted_height = %msg.client_message.trusted_height(),
            )
        }
        Effect::Batch(BatchMsg(_msgs)) => error!("attempted to log a batch tx???"),
    }
}
