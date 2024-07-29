use enumorph::Enumorph;
use macros::apply;
use queue_msg::{
    data, defer_absolute, fetch, now, queue_msg, seq, wait, HandleWait, Op, QueueError, SubsetOf,
};
use tracing::debug;
use unionlabs::{
    ibc::core::client::height::Height, ics24::ClientStatePath, id::ClientId, traits::Member,
    QueryHeight,
};

use crate::{
    data::LatestHeight,
    fetch::FetchState,
    json_rpc_error_to_queue_error,
    plugin::{ChainModuleClient, ClientModuleClient},
    top_level_identifiable_enum, Context, VoyagerMessage,
};

#[apply(top_level_identifiable_enum)]
#[queue_msg]
#[derive(Enumorph, SubsetOf)]
pub enum Wait {
    Height(WaitForHeight),
    HeightRelative(WaitForHeightRelative),
    Timestamp(WaitForTimestamp),
    TrustedHeight(WaitForTrustedHeight),
}

#[queue_msg]
pub struct WaitForHeight {
    pub chain_id: String,
    pub height: Height,
}

#[queue_msg]
pub struct WaitForHeightRelative {
    pub chain_id: String,
    pub height: u64,
}

#[queue_msg]
pub struct WaitForTimestamp {
    pub chain_id: String,
    pub timestamp: i64,
}

/// Wait for the client `.client_id` on `Hc` to trust a height >= `.height`,
/// returning the counterparty's client state at that height when it's reached.
#[queue_msg]
pub struct WaitForTrustedHeight {
    pub chain_id: String,
    /// The id of the client on `Hc` who's [`ClientState::height()`] we're
    /// waiting to be >= `.height`.
    pub client_id: ClientId,
    /// The id of the counterparty client on `Tr`, who's state will be fetched
    /// at [`ClientState::height()`] when `.client_id` on `Hc` trusts a height
    /// >= `.height`.
    pub counterparty_client_id: ClientId,
    pub counterparty_chain_id: String,
    pub height: Height,
}

impl<D: Member, F: Member, A: Member> HandleWait<VoyagerMessage<D, F, A>> for Wait {
    // #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn handle(self, ctx: &Context) -> Result<Op<VoyagerMessage<D, F, A>>, QueueError> {
        match self {
            // TODO: Replace this with an aggregation
            Wait::Height(WaitForHeight { chain_id, height }) => {
                let chain_height = ctx
                    .chain_module::<D, F, A>(&chain_id)?
                    .query_latest_height()
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                assert_eq!(
                    chain_height.revision_number, height.revision_number,
                    "chain_height: {chain_height}, height: {height}",
                );

                debug!("latest height is {chain_height}, waiting for {height}");

                if chain_height.revision_height >= height.revision_height {
                    Ok(data(LatestHeight {
                        chain_id,
                        height: chain_height,
                    }))
                } else {
                    Ok(seq([
                        defer_absolute(now() + 1),
                        wait(WaitForHeight { chain_id, height }),
                    ]))
                }
            }
            // REVIEW: Perhaps remove, unused
            Wait::HeightRelative(WaitForHeightRelative { chain_id, height }) => {
                let chain_height = ctx
                    .chain_module::<D, F, A>(&chain_id)?
                    .query_latest_height()
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                Ok(wait(WaitForHeight {
                    chain_id,
                    height: Height {
                        revision_number: chain_height.revision_number,
                        revision_height: chain_height.revision_height + height,
                    },
                }))
            }
            Wait::Timestamp(WaitForTimestamp {
                chain_id,
                timestamp,
            }) => {
                let chain_ts = ctx
                    .chain_module::<D, F, A>(&chain_id)?
                    .query_latest_timestamp()
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                if chain_ts >= timestamp {
                    // TODO: Figure out a way to fetch a height at a specific timestamp
                    Ok(data(LatestHeight {
                        height: ctx
                            .chain_module::<D, F, A>(&chain_id)?
                            .query_latest_height()
                            .await
                            .map_err(json_rpc_error_to_queue_error)?,
                        chain_id,
                    }))
                } else {
                    Ok(seq([
                        // REVIEW: Defer until `now + chain.block_time()`? Would require a new
                        // method on chain
                        defer_absolute(now() + 1),
                        wait(WaitForTimestamp {
                            chain_id,
                            timestamp,
                        }),
                    ]))
                }
            }
            Wait::TrustedHeight(WaitForTrustedHeight {
                chain_id,
                client_id,
                counterparty_client_id,
                counterparty_chain_id,
                height,
            }) => {
                let client_state = ctx
                    .chain_module::<D, F, A>(&chain_id)?
                    .query_raw_unfinalized_trusted_client_state(client_id.clone())
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                let trusted_client_state_meta = ctx
                    .client_module::<D, F, A>(
                        &client_state.client_type,
                        &client_state.ibc_interface,
                    )?
                    .decode_client_state_meta(client_state.bytes)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                if trusted_client_state_meta.height.revision_height >= height.revision_height {
                    debug!(
                        "client height reached ({} >= {})",
                        trusted_client_state_meta.height, height
                    );

                    // the height has been reached, fetch the counterparty client state on `Tr` at
                    // the trusted height
                    Ok(fetch(FetchState {
                        chain_id: counterparty_chain_id,
                        at: QueryHeight::Specific(trusted_client_state_meta.height),
                        path: ClientStatePath {
                            client_id: counterparty_client_id.clone(),
                        }
                        .into(),
                    }))
                } else {
                    Ok(seq([
                        // REVIEW: Defer until `now + counterparty_chain.block_time()`? Would
                        // require a new method on chain
                        defer_absolute(now() + 1),
                        wait(WaitForTrustedHeight {
                            chain_id,
                            client_id,
                            height,
                            counterparty_client_id,
                            counterparty_chain_id,
                        }),
                    ]))
                }
            }
        }
    }
}
