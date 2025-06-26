use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, log_decoder::LogDecoder},
    event::{supported::SupportedBlockEvent, update_client_event::UpdateClientEvent},
};

impl EthFetcherClient {
    pub fn to_update_client(
        &self,
        log: &LogDecoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_update_client - {log}");

        Ok(vec![SupportedBlockEvent::UpdateClient {
            inner: UpdateClientEvent {
                header: log.header()?,
                client_id: log.event.client_id()?,
                counterparty_height: log.event.counterparty_height()?,
            },
        }])
    }
}
