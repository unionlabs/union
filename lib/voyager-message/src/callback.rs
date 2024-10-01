use std::collections::VecDeque;

use enumorph::Enumorph;
use frunk::hlist_pat;
use futures::{stream, StreamExt, TryFutureExt, TryStreamExt};
use macros::{apply, model};
use unionlabs::{
    ibc::core::client::msg_update_client::MsgUpdateClient, id::ClientId, traits::Member,
};
use voyager_core::ClientInfo;
use voyager_vm::{aggregation::HListTryFromIterator, HandleCallback, Op, QueueError};

use crate::{
    core::ChainId,
    data::{Data, OrderedHeaders, OrderedMsgUpdateClients},
    error_object_to_queue_error, json_rpc_error_to_queue_error,
    module::{ClientModuleClient, PluginClient},
    top_level_identifiable_enum, Context, PluginMessage, VoyagerMessage,
};

#[apply(top_level_identifiable_enum)]
#[model]
#[derive(Enumorph)]
pub enum Callback<Cb = serde_json::Value> {
    AggregateMsgUpdateClientsFromOrderedHeaders(AggregateMsgUpdateClientsFromOrderedHeaders),

    Plugin(PluginMessage<Cb>),
}

impl<D: Member, C: Member, Cb: Member> HandleCallback<VoyagerMessage<D, C, Cb>> for Callback<Cb> {
    // #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn handle(
        self,
        ctx: &Context,
        data: VecDeque<Data<D>>,
    ) -> Result<Op<VoyagerMessage<D, C, Cb>>, QueueError> {
        match self {
            Callback::AggregateMsgUpdateClientsFromOrderedHeaders(
                AggregateMsgUpdateClientsFromOrderedHeaders {
                    chain_id,
                    counterparty_client_id,
                },
            ) => {
                let Ok(hlist_pat![OrderedHeaders { headers }]) =
                    HListTryFromIterator::try_from_iter(data)
                else {
                    panic!("bad data")
                };

                let ClientInfo {
                    client_type,
                    ibc_interface,
                    ..
                } = ctx
                    .rpc_server
                    .client_info(&chain_id, counterparty_client_id.clone())
                    .await
                    .map_err(error_object_to_queue_error)?;

                let client_module = ctx
                    .rpc_server
                    .modules()
                    .map_err(error_object_to_queue_error)?
                    .client_module(&client_type, &ibc_interface)?;

                Ok(voyager_vm::data(OrderedMsgUpdateClients {
                    // REVIEW: Use FuturesOrdered here?
                    updates: stream::iter(headers.into_iter())
                        .then(|(meta, header)| {
                            client_module
                                .encode_header(header)
                                .map_ok(|encoded_header| {
                                    (
                                        meta,
                                        MsgUpdateClient {
                                            client_id: counterparty_client_id.clone(),
                                            client_message: encoded_header.0,
                                        },
                                    )
                                })
                                .map_err(json_rpc_error_to_queue_error)
                        })
                        .try_collect::<Vec<_>>()
                        .await?,
                }))
            }
            Callback::Plugin(PluginMessage { plugin, message }) => Ok(ctx
                .plugin(&plugin)?
                .callback(message, data)
                .await
                .map_err(json_rpc_error_to_queue_error)?),
        }
    }
}

/// Required data: [`OrderedHeaders`]
#[model]
pub struct AggregateMsgUpdateClientsFromOrderedHeaders {
    pub chain_id: ChainId<'static>,
    pub counterparty_client_id: ClientId,
}
