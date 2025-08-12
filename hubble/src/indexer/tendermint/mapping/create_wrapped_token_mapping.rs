use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{create_wrapped_token::CreateWrappedTokenEvent, supported::SupportedBlockEvent},
    tendermint::{fetcher_client::TmFetcherClient, mapping::decoder::Decoder},
};

impl TmFetcherClient {
    pub fn to_create_wrapped_token(
        &self,
        log: &Decoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_create_wrapped_token - {log}");

        Ok(vec![SupportedBlockEvent::CreateWrappedToken {
            inner: CreateWrappedTokenEvent {
                header: log.header()?,
                channel_id: log.event.channel_id()?,
                path: log.event.path()?,
                base_token: log.event.base_token()?,
                quote_token: log.event.quote_token()?,
                metadata: log.event.metadata()?,
                kind: log.event.create_wrapped_token_kind()?,
            },
        }])
    }
}
