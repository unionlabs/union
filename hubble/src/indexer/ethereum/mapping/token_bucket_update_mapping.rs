use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, mapping::decoder::Decoder},
    event::{supported::SupportedBlockEvent, token_bucket_update_event::TokenBucketUpdateEvent},
};

impl EthFetcherClient {
    pub fn to_token_bucket_update(
        &self,
        decoder: &Decoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_token_bucket_update - {decoder}");

        Ok(vec![SupportedBlockEvent::TokenBucketUpdate {
            inner: TokenBucketUpdateEvent {
                header: decoder.header()?,
                denom: decoder.event.denom()?,
                capacity: decoder.event.capacity()?,
                refill_rate: decoder.event.refill_rate()?,
            },
        }])
    }
}
