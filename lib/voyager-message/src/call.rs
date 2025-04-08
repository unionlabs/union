use enumorph::Enumorph;
use macros::model;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tracing::{debug, info, instrument, trace};
use unionlabs::{ibc::core::client::height::Height, traits::Member};
use voyager_primitives::{ClientType, IbcSpecId, QueryHeight, Timestamp};
use voyager_vm::{call, defer, noop, now, seq, CallT, Op, QueueError};

use crate::{
    context::WithId, data::IbcDatagram, error_object_to_queue_error, json_rpc_error_to_queue_error,
    module::PluginClient, primitives::ChainId, Context, PluginMessage, RawClientId, VoyagerMessage,
};

#[model]
#[derive(Enumorph)]
pub enum Call {
    // hooks
    FetchBlocks(FetchBlocks),
    FetchUpdateHeaders(FetchUpdateHeaders),
    SubmitTx(SubmitTx),

    // generic waiting logic
    WaitForHeight(WaitForHeight),
    WaitForTimestamp(WaitForTimestamp),

    WaitForTrustedHeight(WaitForTrustedHeight),
    WaitForTrustedTimestamp(WaitForTrustedTimestamp),

    WaitForClientUpdate(WaitForClientUpdate),

    Plugin(PluginMessage),
}

impl Call {
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

#[model]
pub struct FetchBlockRange {
    pub chain_id: ChainId,
    pub from_height: Height,
    pub to_height: Height,
}

/// Fetch blocks on a chain, starting at height `start_height`.
///
/// This represents a request for IBC events on a chain and must be
/// picked up by a plugin. If it is not handled by a plugin, this will
/// return with a fatal error.
///
/// # Implementor's Note
///
/// This message is intended to act as a "seed" to an infinite stream of
/// unfolding messages. For example, if this is queued with height 10,
/// the plugin message this is replaced with should fetch all events in
/// block 10 and then wait for block 11 (which would then wait for block
/// 12, etc). Due to differing behaviours between chains, this may not
/// be the exact implementation, but the semantics of the unfold should
/// still hold.
#[model]
pub struct FetchBlocks {
    pub chain_id: ChainId,
    pub start_height: Height,
}

/// Generate a client update for a chain, tracked by a client type.
///
/// This represents a request for a client update and must be picked up
/// by a plugin. If it is not handled by a plugin, this will return with
/// a fatal error.
///
/// # Implementor's Note
///
/// The returned [`Op`] ***MUST*** resolve to an [`OrderedHeaders`] data.
/// This is the entrypoint called when a client update is requested, and
/// is intended to be called in the queue of an
/// [`AggregateSubmitTxFromOrderedHeaders`] message, which will
/// be used to build the actual [`MsgUpdateClient`]s.
#[model]
pub struct FetchUpdateHeaders {
    /// The type of client that is tracking the consensus on `self.chain_id`.
    pub client_type: ClientType,
    /// The ID of the chain that is being tracked by the `self.client_id` client on
    /// `self.counterparty_chain_id`.
    pub chain_id: ChainId,
    /// The chain that the light client tracking `self.chain_id` is on.
    pub counterparty_chain_id: ChainId,
    /// The ID of the client that is being updated.
    pub client_id: RawClientId,
    /// The currently trusted height of the client on `self.chain_id`.
    pub update_from: Height,
    /// The *minimum* height to update the client to. This is assumed to be finalized. Note that
    /// the generated update may not be to this exact height, but it *must* be >= it.
    pub update_to: Height,
}

/// Submit a batch of transactions on the specified chain.
///
/// This represents a request for transaction submission and must be picked up by a plugin. If it is
/// not handled by a plugin, this will return with a fatal error.
///
/// # Implementor's Note
///
/// The returned [`Op`] ***MUST*** resolve to a [`Op::Noop`].
#[model]
pub struct SubmitTx {
    /// The chain to submit the messages on.
    pub chain_id: ChainId,
    // TODO: Ensure this is non-empty
    pub datagrams: Vec<IbcDatagram>,
}

#[model]
pub struct WaitForHeight {
    pub chain_id: ChainId,
    pub height: Height,
    pub finalized: bool,
}

#[model]
pub struct WaitForTimestamp {
    pub chain_id: ChainId,
    pub timestamp: Timestamp,
    pub finalized: bool,
}

/// Wait for the client `.client_id` on `.chain_id` to trust a height >=
/// `.height`.
#[model]
pub struct WaitForTrustedHeight {
    pub chain_id: ChainId,
    pub ibc_spec_id: IbcSpecId,
    pub client_id: RawClientId,
    pub height: Height,
    pub finalized: bool,
}

/// Wait for the client `.client_id` on `.chain_id` to trust a timestamp >=
/// `.timestamp`.
#[model]
pub struct WaitForTrustedTimestamp {
    pub chain_id: ChainId,
    pub ibc_spec_id: IbcSpecId,
    pub client_id: RawClientId,
    pub timestamp: Timestamp,
    pub finalized: bool,
}

/// Wait for the client `.client_id` on `.chain_id` to trust a height >=
/// `.height`.
#[model]
pub struct WaitForClientUpdate {
    pub chain_id: ChainId,
    pub ibc_spec_id: IbcSpecId,
    pub client_id: RawClientId,
    pub height: Height,
    // pub finalized: bool,
}

impl CallT<VoyagerMessage> for Call {
    #[instrument(skip_all, fields(id = ctx.id().raw()))]
    async fn process(
        self,
        ctx: voyager_vm::Context<&Context>,
    ) -> Result<Op<VoyagerMessage>, QueueError> {
        match self {
            Call::FetchBlocks(FetchBlocks {
                start_height,
                chain_id,
            }) => {
                let message = format!(
                    "fetch blocks request received for chain `{chain_id}` at height \
                    {start_height} but it was not picked up by a plugin"
                );

                Err(QueueError::Unprocessable(message.into()))
            }

            Call::FetchUpdateHeaders(FetchUpdateHeaders {
                client_type,
                chain_id,
                counterparty_chain_id,
                client_id,
                update_from,
                update_to,
            }) => {
                let message = format!(
                    "client update request received for a {client_type} client \
                    (id {client_id}) on {counterparty_chain_id} tracking {chain_id} from \
                    height {update_from} to {update_to} but it was not picked up by a plugin"
                );

                Err(QueueError::Unprocessable(message.into()))
            }

            Call::SubmitTx(SubmitTx { chain_id, .. }) => {
                let message = format!(
                    "transaction submission request received for chain {chain_id} but \
                    it was not picked up by a plugin"
                );

                Err(QueueError::Unprocessable(message.into()))
            }

            // TODO: Replace this with an aggregation
            Call::WaitForHeight(WaitForHeight {
                chain_id,
                height,
                finalized,
            }) => {
                let chain_height = ctx
                    .rpc_server
                    .with_id(Some(ctx.id()))
                    .query_latest_height(&chain_id, finalized)
                    .await
                    .map_err(error_object_to_queue_error)?;

                if !chain_height.revision_matches(&height) {
                    return Err(QueueError::Fatal(
                        format!(
                            "revision number mismatch, \
                            chain_height: {chain_height}, height: {height}"
                        )
                        .into(),
                    ));
                }

                trace!("latest height is {chain_height}, waiting for {height}");

                if chain_height.height() >= height.height() {
                    Ok(noop())
                } else {
                    Ok(seq([
                        defer(now() + 1),
                        call(WaitForHeight {
                            chain_id,
                            height,
                            finalized,
                        }),
                    ]))
                }
            }

            Call::WaitForTimestamp(WaitForTimestamp {
                chain_id,
                timestamp,
                finalized,
            }) => {
                let chain_timestamp = ctx
                    .rpc_server
                    .with_id(Some(ctx.id()))
                    .query_latest_timestamp(&chain_id, finalized)
                    .await
                    .map_err(error_object_to_queue_error)?;

                if chain_timestamp >= timestamp {
                    info!(%chain_id, %timestamp, %chain_timestamp, "timestamp reached");
                    Ok(noop())
                } else {
                    debug!(%chain_id, %timestamp, %chain_timestamp, "timestamp not yet reached");
                    Ok(seq([
                        // REVIEW: Defer until `now + chain.block_time()`? Would require a new
                        // method on chain
                        defer(now() + 1),
                        call(WaitForTimestamp {
                            chain_id,
                            timestamp,
                            finalized,
                        }),
                    ]))
                }
            }

            Call::WaitForTrustedHeight(WaitForTrustedHeight {
                chain_id,
                ibc_spec_id,
                client_id,
                height,
                finalized,
            }) => {
                let trusted_client_state_meta = ctx
                    .rpc_server
                    .with_id(Some(ctx.id()))
                    .client_state_meta(
                        &chain_id,
                        &ibc_spec_id,
                        if finalized {
                            QueryHeight::Finalized
                        } else {
                            QueryHeight::Latest
                        },
                        client_id.clone(),
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

                let continuation = seq([
                    // REVIEW: Defer until `now + counterparty_chain.block_time()`? Would
                    // require a new method on chain
                    defer(now() + 1),
                    call(WaitForTrustedHeight {
                        chain_id: chain_id.clone(),
                        ibc_spec_id,
                        client_id: client_id.clone(),
                        height,
                        finalized,
                    }),
                ]);

                match trusted_client_state_meta {
                    Some(trusted_client_state_meta) => {
                        if trusted_client_state_meta.counterparty_height.height() >= height.height()
                        {
                            debug!(
                                "client height reached ({} >= {})",
                                trusted_client_state_meta.counterparty_height, height
                            );

                            Ok(noop())
                        } else {
                            Ok(continuation)
                        }
                    }
                    None => {
                        debug!("client {client_id} not found on chain {chain_id}");
                        Ok(continuation)
                    }
                }
            }

            Call::WaitForTrustedTimestamp(WaitForTrustedTimestamp {
                chain_id,
                ibc_spec_id,
                client_id,
                timestamp,
                finalized,
            }) => {
                let trusted_client_state_meta = ctx
                    .rpc_server
                    .with_id(Some(ctx.id()))
                    .client_state_meta(
                        &chain_id,
                        &ibc_spec_id,
                        if finalized {
                            QueryHeight::Finalized
                        } else {
                            QueryHeight::Latest
                        },
                        client_id.clone(),
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

                let continuation = seq([
                    // REVIEW: Defer until `now + counterparty_chain.block_time()`? Would
                    // require a new method on chain
                    defer(now() + 1),
                    call(WaitForTrustedTimestamp {
                        chain_id: chain_id.clone(),
                        ibc_spec_id: ibc_spec_id.clone(),
                        client_id: client_id.clone(),
                        timestamp,
                        finalized,
                    }),
                ]);

                match trusted_client_state_meta {
                    Some(trusted_client_state_meta) => {
                        let trusted_consensus_state_meta = ctx
                            .rpc_server
                            .with_id(Some(ctx.id()))
                            .consensus_state_meta(
                                &chain_id,
                                &ibc_spec_id,
                                if finalized {
                                    QueryHeight::Finalized
                                } else {
                                    QueryHeight::Latest
                                },
                                client_id.clone(),
                                trusted_client_state_meta.counterparty_height,
                            )
                            .await
                            .map_err(error_object_to_queue_error)?;

                        match trusted_consensus_state_meta {
                            Some(trusted_consensus_state_meta)
                                if trusted_consensus_state_meta.timestamp >= timestamp =>
                            {
                                debug!(
                                    "client timestamp reached ({} >= {})",
                                    trusted_client_state_meta.counterparty_height, timestamp
                                );

                                Ok(noop())
                            }
                            _ => Ok(continuation),
                        }
                    }
                    None => {
                        debug!("client {client_id} not found on chain {chain_id}");
                        Ok(continuation)
                    }
                }
            }

            Call::WaitForClientUpdate(WaitForClientUpdate {
                chain_id,
                ibc_spec_id,
                client_id,
                height,
                // finalized,
            }) => {
                let consensus_state_meta = ctx
                    .rpc_server
                    .with_id(Some(ctx.id()))
                    .consensus_state_meta(
                        &chain_id,
                        &ibc_spec_id,
                        QueryHeight::Latest,
                        client_id.clone(),
                        height,
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

                match consensus_state_meta {
                    Some(consensus_state_meta) => {
                        debug!(
                            consensus_state_meta.timestamp = %consensus_state_meta.timestamp,
                            "consensus state exists"
                        );
                        Ok(noop())
                    }
                    None => {
                        debug!("consensus state for client {client_id} not found at height {height} on chain {chain_id}");
                        Ok(seq([
                            defer(now() + 1),
                            call(WaitForClientUpdate {
                                chain_id: chain_id.clone(),
                                ibc_spec_id,
                                client_id: client_id.clone(),
                                height,
                                // finalized,
                            }),
                        ]))
                    }
                }
            }

            Call::Plugin(PluginMessage { plugin, message }) => {
                Ok(PluginClient::<Value, Value>::call(
                    &ctx.plugin(plugin)?.with_id(Some(ctx.id())),
                    message,
                )
                .await
                .map_err(json_rpc_error_to_queue_error)?)
            }
        }
    }
}
