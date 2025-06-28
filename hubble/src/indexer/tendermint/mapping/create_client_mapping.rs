use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{create_client_event::CreateClientEvent, supported::SupportedBlockEvent},
    tendermint::{event_decoder::EventDecoder, fetcher_client::TmFetcherClient},
};

impl TmFetcherClient {
    pub fn to_create_client(
        &self,
        log: &EventDecoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_create_client - {log}");

        Ok(vec![SupportedBlockEvent::CreateClient {
            inner: CreateClientEvent {
                header: log.header()?,
                client_id: log.event.client_id()?,
                client_type: log.event.client_type()?,
                counterparty_chain_id: log.event.counterparty_chain_id()?,
            },
        }])
    }
}
