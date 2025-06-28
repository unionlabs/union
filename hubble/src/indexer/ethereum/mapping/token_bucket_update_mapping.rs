use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, log_decoder::LogDecoder},
    event::{supported::SupportedBlockEvent, token_bucket_update_event::TokenBucketUpdateEvent},
};

impl EthFetcherClient {
    pub fn to_token_bucket_update(
        &self,
        log: &LogDecoder,
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
