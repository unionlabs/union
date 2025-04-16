use std::collections::VecDeque;

use enumorph::Enumorph;
use futures::{stream, StreamExt, TryFutureExt, TryStreamExt};
use itertools::Itertools;
use macros::model;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tracing::instrument;
use unionlabs::traits::Member;
use voyager_primitives::{ClientInfo, IbcSpecId};
use voyager_vm::{BoxDynError, CallbackT, Op, QueueError};

use crate::{
    call::SubmitTx,
    context::WithId,
    data::{Data, IbcDatagram, OrderedHeaders},
    error_object_to_queue_error, json_rpc_error_to_queue_error,
    module::{ClientModuleClient, PluginClient},
    primitives::ChainId,
    rpc::missing_state,
    Context, PluginMessage, RawClientId, VoyagerMessage,
};

#[model]
#[derive(Enumorph)]
pub enum Callback {
    AggregateSubmitTxFromOrderedHeaders(AggregateSubmitTxFromOrderedHeaders),

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
    #[instrument(skip_all, fields(id = ctx.id().raw()))]
    async fn process(
        self,
        ctx: voyager_vm::Context<&Context>,
        data: VecDeque<Data>,
    ) -> Result<Op<VoyagerMessage>, QueueError> {
        match self {
            Callback::AggregateSubmitTxFromOrderedHeaders(
                AggregateSubmitTxFromOrderedHeaders {
                    ibc_spec_id,
                    chain_id,
                    client_id,
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
                                "OrderedHeaders not present in data queue \
                                for AggregateSubmitTxFromOrderedHeaders, \
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
                    .with_id(Some(ctx.id()))
                    .client_info(&chain_id, &ibc_spec_id, client_id.clone())
                    .await
                    .map_err(error_object_to_queue_error)?
                    .ok_or_else(missing_state("client not found", None))
                    .map_err(error_object_to_queue_error)?;

                let client_module = ctx
                    .rpc_server
                    .modules()
                    .map_err(error_object_to_queue_error)?
                    .client_module(&client_type, &ibc_interface, &ibc_spec_id)?
                    .with_id(Some(ctx.id()));

                let ibc_spec_handler = ctx
                    .rpc_server
                    .modules()
                    .map_err(error_object_to_queue_error)?
                    .ibc_spec_handlers
                    .get(&ibc_spec_id)
                    .map_err(error_object_to_queue_error)?;

                // OrderedClientUpdates

                Ok(voyager_vm::call(SubmitTx {
                    chain_id,
                    // REVIEW: Use FuturesOrdered here?
                    datagrams: stream::iter(headers.into_iter())
                        .then(|(_, header)| {
                            client_module
                                .encode_header(header)
                                .map_err(json_rpc_error_to_queue_error)
                                .and_then(|encoded_header| {
                                    futures::future::ready(
                                        (ibc_spec_handler.msg_update_client)(
                                            client_id.clone(),
                                            encoded_header,
                                        )
                                        .map_err(|e| {
                                            QueueError::Fatal(<BoxDynError>::from(format!("{e:#}")))
                                        })
                                        .map(|datagram| {
                                            IbcDatagram {
                                                ibc_spec_id: ibc_spec_id.clone(),
                                                datagram,
                                            }
                                        }),
                                    )
                                })
                        })
                        .try_collect::<Vec<_>>()
                        .await?,
                }))
            }
            Callback::Plugin(PluginMessage { plugin, message }) => {
                Ok(PluginClient::<Value, Value>::callback(
                    &ctx.plugin(plugin)
                        .map_err(error_object_to_queue_error)?
                        .with_id(Some(ctx.id())),
                    message,
                    data,
                )
                .await
                .map_err(json_rpc_error_to_queue_error)?)
            }
        }
    }
}

/// Required data: [`OrderedHeaders`]
///
/// Returns: [`SubmitTx`]
#[model]
pub struct AggregateSubmitTxFromOrderedHeaders {
    pub ibc_spec_id: IbcSpecId,
    pub chain_id: ChainId,
    pub client_id: RawClientId,
}
