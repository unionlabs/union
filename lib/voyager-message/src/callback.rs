use std::collections::VecDeque;

use enumorph::Enumorph;
use futures::{stream, StreamExt, TryFutureExt, TryStreamExt};
use itertools::Itertools;
use macros::model;
use serde::de::DeserializeOwned;
use unionlabs::{
    ibc::core::client::msg_update_client::MsgUpdateClient, id::ClientId, traits::Member,
};
use voyager_core::ClientInfo;
use voyager_vm::{CallbackT, Op, QueueError};

use crate::{
    core::ChainId,
    data::{Data, OrderedHeaders, OrderedMsgUpdateClients},
    error_object_to_queue_error, json_rpc_error_to_queue_error,
    module::{ClientModuleClient, PluginClient},
    Context, PluginMessage, VoyagerMessage,
};

#[model]
#[derive(Enumorph)]
pub enum Callback {
    AggregateMsgUpdateClientsFromOrderedHeaders(AggregateMsgUpdateClientsFromOrderedHeaders),

    Plugin(PluginMessage),
}

impl Callback {
    #[allow(clippy::result_large_err)]
    pub fn as_plugin<T: DeserializeOwned>(self, plugin_name: impl AsRef<str>) -> Result<T, Self> {
        match self {
            Self::Plugin(plugin_message) => {
                plugin_message.downcast(plugin_name).map_err(Self::Plugin)
            }
            this => Err(this),
        }
    }
}

impl CallbackT<VoyagerMessage> for Callback {
    async fn process(
        self,
        ctx: &Context,
        data: VecDeque<Data>,
    ) -> Result<Op<VoyagerMessage>, QueueError> {
        match self {
            Callback::AggregateMsgUpdateClientsFromOrderedHeaders(
                AggregateMsgUpdateClientsFromOrderedHeaders {
                    chain_id,
                    counterparty_client_id,
                },
            ) => {
                let OrderedHeaders { headers } = data
                    .into_iter()
                    .exactly_one()
                    .map_err(|found| serde_json::to_string(&found.collect::<Vec<_>>()).unwrap())
                    .and_then(|d| {
                        d.try_into()
                            .map_err(|found| serde_json::to_string(&found).unwrap())
                    })
                    .map_err(|found| {
                        QueueError::Fatal(
                            format!(
                                "OrderedHeaders not present in data queue for \
                                AggregateMsgUpdateClientsFromOrderedHeaders, \
                                found {found}",
                            )
                            .into(),
                        )
                    })?;

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
                                            client_message: encoded_header,
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
