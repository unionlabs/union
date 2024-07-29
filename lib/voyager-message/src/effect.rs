use enumorph::Enumorph;
use macros::apply;
use queue_msg::{queue_msg, HandleEffect, Op, QueueError, SubsetOf};
use tracing::info;
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
    traits::Member,
};

use crate::{
    json_rpc_error_to_queue_error, plugin::TransactionSubmissionModuleClient,
    top_level_identifiable_enum, ClientType, Context, VoyagerMessage,
};

#[apply(top_level_identifiable_enum)]
#[queue_msg]
#[derive(Enumorph, SubsetOf)]
pub enum Effect {
    Single(WithChainId<Msg>),
    Batch(WithChainId<Vec<Msg>>),
}

#[queue_msg]
pub struct WithChainId<T> {
    pub chain_id: String,
    pub message: T,
}

impl<D: Member, F: Member, A: Member> HandleEffect<VoyagerMessage<D, F, A>> for Effect {
    // #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    #[allow(clippy::manual_async_fn)]
    async fn handle(self, ctx: &Context) -> Result<Op<VoyagerMessage<D, F, A>>, QueueError> {
        match self {
            Effect::Single(WithChainId { chain_id, message }) => ctx
                .transaction_module::<D, F, A>(&chain_id)?
                .send_transaction(vec![message])
                .await
                .map_err(json_rpc_error_to_queue_error),
            Effect::Batch(WithChainId { chain_id, message }) => ctx
                .transaction_module::<D, F, A>(&chain_id)?
                .send_transaction(message)
                .await
                .map_err(json_rpc_error_to_queue_error),
        }
    }
}

#[queue_msg]
#[derive(Enumorph, SubsetOf)]
pub enum Msg {
    ConnectionOpenInit(MsgConnectionOpenInit),
    ConnectionOpenTry(MsgConnectionOpenTry),
    ConnectionOpenAck(MsgConnectionOpenAck),
    ConnectionOpenConfirm(MsgConnectionOpenConfirm),

    ChannelOpenInit(MsgChannelOpenInit),
    ChannelOpenTry(MsgChannelOpenTry),
    ChannelOpenAck(MsgChannelOpenAck),
    ChannelOpenConfirm(MsgChannelOpenConfirm),

    RecvPacket(MsgRecvPacket),
    AckPacket(MsgAcknowledgement),
    TimeoutPacket(MsgTimeout),

    CreateClient(MsgCreateClientData),
    UpdateClient(MsgUpdateClient),
}

#[queue_msg]
pub struct MsgCreateClientData {
    pub msg: MsgCreateClient,
    pub client_type: ClientType<'static>,
}

pub fn log_msg(chain_id: &str, effect: Msg) {
    match effect.clone() {
        Msg::ConnectionOpenInit(message) => {
            info!(
                %chain_id,
                %message.client_id,
                %message.counterparty.client_id,
                message.counterparty.connection_id = %message.counterparty.connection_id.as_deref().unwrap_or_default(),
                message.counterparty.prefix.key_prefix = %::serde_utils::to_hex(message.counterparty.prefix.key_prefix),
                %message.version.identifier,
                message.version.features = %message
                    .version
                    .features
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                %message.delay_period,
            )
        }
        Msg::ConnectionOpenTry(message) => {
            info!(
                %chain_id,
                %message.client_id,
                %message.counterparty.client_id,
                message.counterparty.connection_id = %message.counterparty.connection_id.as_deref().unwrap_or_default(),
                message.counterparty.prefix.key_prefix = %::serde_utils::to_hex(message.counterparty.prefix.key_prefix),
                %message.delay_period,
                // TODO: This needs `valuable`
                // message.counterparty_versions = %message
                //     .counterparty_versions
                //     .into_iter()
                //     .map(Into::into)
                //     .collect(),
                %message.proof_height,
                %message.consensus_height,
            )
        }
        Msg::ConnectionOpenAck(message) => {
            info!(
                %chain_id,
                // client_state.height = message.%data.message.client_state.height(),
                %message.proof_height,
                %message.consensus_height,
                %message.connection_id,
                %message.counterparty_connection_id,
                %message.version.identifier,
                message.version.features = %message
                    .version
                    .features
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            )
        }
        Msg::ConnectionOpenConfirm(message) => {
            info!(
                %chain_id,
                %message.connection_id,
                %message.proof_height,
            )
        }
        Msg::ChannelOpenInit(message) => {
            info!(
                %chain_id,
                %message.port_id,
                %message.channel.state,
                %message.channel.ordering,
                %message.channel.counterparty.port_id,
                %message.channel.counterparty.channel_id,
                message.channel.connection_hops = %message
                    .channel
                    .connection_hops
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                %message.channel.version,
            )
        }
        Msg::ChannelOpenTry(message) => {
            info!(
                %chain_id,

                %message.port_id,
                %message.channel.state,
                %message.channel.ordering,
                %message.channel.counterparty.port_id,
                %message.channel.counterparty.channel_id,
                message.channel.connection_hops = %message
                    .channel
                    .connection_hops
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                %message.channel.version,
                %message.counterparty_version,
                %message.proof_height,
            )
        }
        Msg::ChannelOpenAck(message) => {
            info!(
                %chain_id,
                %message.port_id,
                %message.channel_id,
                %message.counterparty_version,
                %message.counterparty_channel_id,
                %message.proof_height,
            )
        }
        Msg::ChannelOpenConfirm(message) => {
            info!(
                %chain_id,
                %message.port_id,
                %message.channel_id,
                %message.proof_height,
            )
        }
        Msg::RecvPacket(message) => {
            info!(
                %chain_id,
                %message.packet.sequence,
                %message.packet.source_port,
                %message.packet.source_channel,
                %message.packet.destination_port,
                %message.packet.destination_channel,
                message.data = %::serde_utils::to_hex(message.packet.data),
                %message.packet.timeout_height,
                %message.packet.timeout_timestamp,

                %message.proof_height,
            )
        }
        Msg::AckPacket(message) => {
            info!(
                %chain_id,
                %message.packet.sequence,
                %message.packet.source_port,
                %message.packet.source_channel,
                %message.packet.destination_port,
                %message.packet.destination_channel,
                message.data = %::serde_utils::to_hex(message.packet.data),
                %message.packet.timeout_height,
                %message.packet.timeout_timestamp,

                message.data = %::serde_utils::to_hex(message.acknowledgement),
                %message.proof_height,
            )
        }
        Msg::TimeoutPacket(message) => {
            info!(
                %chain_id,
                %message.packet.sequence,
                %message.packet.source_port,
                %message.packet.source_channel,
                %message.packet.destination_port,
                %message.packet.destination_channel,
                message.data = %::serde_utils::to_hex(message.packet.data),
                %message.packet.timeout_height,
                %message.packet.timeout_timestamp,

                %message.proof_height,
                %message.next_sequence_recv,
            )
        }
        Msg::CreateClient(message) => {
            info!(
                %chain_id,
                %message.client_type,
            )
        }
        Msg::UpdateClient(message) => {
            info!(
                %chain_id,
                %message.client_id,
            )
        }
    }
}
