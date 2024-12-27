use enumorph::Enumorph;
use macros::model;
use serde::de::DeserializeOwned;
use tracing::{debug, error, info, instrument};
use unionlabs::{ibc::core::client::height::Height, traits::Member};
use voyager_core::{ClientType, IbcSpecId, QueryHeight};
use voyager_vm::{call, defer, noop, now, seq, CallT, Op, QueueError};

use crate::{
    core::ChainId, error_object_to_queue_error, json_rpc_error_to_queue_error,
    module::PluginClient, Context, PluginMessage, RawClientId, VoyagerMessage,
};

#[model]
#[derive(Enumorph)]
pub enum Call {
    FetchBlocks(FetchBlocks),

    FetchUpdateHeaders(FetchUpdateHeaders),

    // MakeMsgCreateClient(MakeMsgCreateClient),
    WaitForHeight(WaitForHeight),
    WaitForTimestamp(WaitForTimestamp),
    WaitForTrustedHeight(WaitForTrustedHeight),

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
/// [`AggregateMsgUpdateClientsFromOrderedHeaders`] message, which will
/// be used to build the actual [`MsgUpdateClient`]s.
#[model]
pub struct FetchUpdateHeaders {
    /// The type of client that is tracking the consensus on `self.chain_id`.
    pub client_type: ClientType,
    /// The ID of the chain that is being tracked by a client on `self.counterparty_chain_id`.
    pub chain_id: ChainId,
    /// The chain that the light client tracking `self.chain_id` is on.
    pub counterparty_chain_id: ChainId,
    /// The currently trusted height of the client on `self.chain_id`.
    pub update_from: Height,
    /// The *minimum* height to update the client to. This is assumed to be finalized. Note that the generated update may not be to this exact height, but it *must* be >= it.
    pub update_to: Height,
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
    /// THIS IS NANOSECONDS
    pub timestamp: i64,
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

                error!(%message);

                Err(QueueError::Fatal(message.into()))
            }

            Call::FetchUpdateHeaders(FetchUpdateHeaders {
                client_type,
                chain_id,
                counterparty_chain_id,
                update_from,
                update_to,
            }) => {
                let message = format!(
                    "client update request received for a {client_type} client on \
                    {counterparty_chain_id} tracking {chain_id} from height {update_from}
                    to {update_to} but it was not picked up by a plugin"
                );

                error!(%message);

                Err(QueueError::Fatal(message.into()))
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

                debug!("latest height is {chain_height}, waiting for {height}");

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
            }) => {
                let trusted_client_state_meta = ctx
                    .rpc_server
                    .with_id(Some(ctx.id()))
                    .client_meta(
                        &chain_id,
                        &ibc_spec_id,
                        QueryHeight::Latest,
                        client_id.clone(),
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

                if trusted_client_state_meta.height.height() >= height.height() {
                    debug!(
                        "client height reached ({} >= {})",
                        trusted_client_state_meta.height, height
                    );

                    Ok(noop())
                } else {
                    Ok(seq([
                        // REVIEW: Defer until `now + counterparty_chain.block_time()`? Would
                        // require a new method on chain
                        defer(now() + 1),
                        call(WaitForTrustedHeight {
                            chain_id,
                            ibc_spec_id,
                            client_id,
                            height,
                        }),
                    ]))
                }
            }
            Call::Plugin(PluginMessage { plugin, message }) => Ok(ctx
                .plugin(plugin)?
                .call(message)
                .await
                .map_err(json_rpc_error_to_queue_error)?),
        }
    }
}
