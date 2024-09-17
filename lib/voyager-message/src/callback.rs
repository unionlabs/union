use std::collections::VecDeque;

use enumorph::Enumorph;
use frunk::{hlist_pat, HList};
use futures::{stream, StreamExt, TryFutureExt, TryStreamExt};
use macros::apply;
use queue_msg::{
    aggregation::{do_callback, DoCallback, HListTryFromIterator},
    call, queue_msg, HandleCallback, Op, QueueError,
};
use serde_json::Value;
use unionlabs::{
    ibc::core::client::{height::Height, msg_update_client::MsgUpdateClient},
    id::ClientId,
    traits::Member,
};

use crate::{
    call::FetchBlockRange,
    data::{ClientInfo, Data, LatestHeight, OrderedHeaders, OrderedMsgUpdateClients},
    error_object_to_queue_error, json_rpc_error_to_queue_error,
    module::{ClientModuleClient, QueueInteractionsClient},
    top_level_identifiable_enum, ChainId, Context, PluginMessage, VoyagerMessage,
};

#[apply(top_level_identifiable_enum)]
#[queue_msg]
#[derive(Enumorph)]
pub enum Callback<Cb = serde_json::Value> {
    // originally block
    FetchBlockRange(AggregateFetchBlockRange),

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
            Callback::FetchBlockRange(aggregate) => Ok(do_callback(aggregate, data)),

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
                    .client_module::<Value, Value, Value>(&client_type, &ibc_interface)
                    .map_err(error_object_to_queue_error)?;

                Ok(queue_msg::data(OrderedMsgUpdateClients {
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

#[queue_msg]
pub struct AggregateFetchBlockRange {
    pub from_height: Height,
}

impl<D: Member, C: Member, Cb: Member> DoCallback<VoyagerMessage<D, C, Cb>>
    for AggregateFetchBlockRange
{
    type Params = HList![LatestHeight];

    fn call(
        Self { from_height }: Self,
        hlist_pat![LatestHeight {
            chain_id,
            height: to_height
        }]: Self::Params,
    ) -> Op<VoyagerMessage<D, C, Cb>> {
        assert!(to_height.revision_height > from_height.revision_height);

        call(FetchBlockRange {
            chain_id,
            from_height,
            to_height,
        })
    }
}

/// Required data: [`OrderedHeaders`]
#[queue_msg]
pub struct AggregateMsgUpdateClientsFromOrderedHeaders {
    pub chain_id: ChainId<'static>,
    pub counterparty_client_id: ClientId,
}
