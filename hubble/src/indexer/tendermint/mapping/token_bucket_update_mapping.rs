use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{supported::SupportedBlockEvent, token_bucket_update_event::TokenBucketUpdateEvent},
    tendermint::{fetcher_client::TmFetcherClient, mapping::decoder::Decoder},
};

impl TmFetcherClient {
    pub fn to_token_bucket_update(
        &self,
        log: &Decoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_token_bucket_update - {log}");

        Ok(vec![SupportedBlockEvent::TokenBucketUpdate {
            inner: TokenBucketUpdateEvent {
                header: log.header()?,
                denom: log.event.denom()?,
                capacity: log.event.capacity()?,
                refill_rate: log.event.refill_rate()?,
            },
        }])
    }
}
