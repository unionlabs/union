use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, mapping::decoder::Decoder},
    event::{create_wrapped_token::CreateWrappedTokenEvent, supported::SupportedBlockEvent},
};

impl EthFetcherClient {
    pub fn to_create_wrapped_token(
        &self,
        decoder: &Decoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_create_wrapped_token - {decoder}");

        Ok(vec![SupportedBlockEvent::CreateWrappedToken {
            inner: CreateWrappedTokenEvent {
                header: decoder.header()?,
                path: decoder.event.path()?,
                channel_id: decoder.event.channel_id()?,
                base_token: decoder.event.base_token()?,
                quote_token: decoder.event.quote_token()?,
                metadata: decoder.event.metadata()?,
                kind: decoder.event.create_wrapped_token_kind()?,
            },
        }])
    }
}
