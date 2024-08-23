// use enumorph::Enumorph;
// use macros::apply;
// use queue_msg::{queue_msg, Op, QueueError, SubsetOf};
// use tracing::info;
// use unionlabs::{
//     ibc::core::{
//         channel::{
//             msg_acknowledgement::MsgAcknowledgement, msg_channel_open_ack::MsgChannelOpenAck,
//             msg_channel_open_confirm::MsgChannelOpenConfirm,
//             msg_channel_open_init::MsgChannelOpenInit, msg_channel_open_try::MsgChannelOpenTry,
//             msg_recv_packet::MsgRecvPacket, msg_timeout::MsgTimeout,
//         },
//         client::{msg_create_client::MsgCreateClient, msg_update_client::MsgUpdateClient},
//         connection::{
//             msg_connection_open_ack::MsgConnectionOpenAck,
//             msg_connection_open_confirm::MsgConnectionOpenConfirm,
//             msg_connection_open_init::MsgConnectionOpenInit,
//             msg_connection_open_try::MsgConnectionOpenTry,
//         },
//     },
//     traits::Member,
// };

// use crate::{
//     json_rpc_error_to_queue_error, plugin::TransactionSubmissionModuleClient,
//     top_level_identifiable_enum, ClientType, Context, VoyagerMessage,
// };

// #[apply(top_level_identifiable_enum)]
// #[queue_msg]
// #[derive(Enumorph, SubsetOf)]
// pub enum Effect {}

// impl<D: Member, F: Member, A: Member> HandleEffect<VoyagerMessage<D, F, A>> for Effect {
//     // #[instrument(skip_all, fields(chain_id = %self.chain_id))]
//     #[allow(clippy::manual_async_fn)]
//     async fn handle(self, ctx: &Context) -> Result<Op<VoyagerMessage<D, F, A>>, QueueError> {
//         match self {
//             Effect::Single(WithChainId { chain_id, message }) => ctx
//                 .transaction_module::<D, F, A>(&chain_id)?
//                 .send_transaction(vec![message])
//                 .await
//                 .map_err(json_rpc_error_to_queue_error),
//             Effect::Batch(WithChainId { chain_id, message }) => ctx
//                 .transaction_module::<D, F, A>(&chain_id)?
//                 .send_transaction(message)
//                 .await
//                 .map_err(json_rpc_error_to_queue_error),
//         }
//     }
// }
