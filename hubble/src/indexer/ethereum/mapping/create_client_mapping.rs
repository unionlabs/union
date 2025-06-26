use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, log_decoder::LogDecoder},
    event::{create_client_event::CreateClientEvent, supported::SupportedBlockEvent},
};

impl EthFetcherClient {
    pub fn to_create_client(
        &self,
        log: &LogDecoder,
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
