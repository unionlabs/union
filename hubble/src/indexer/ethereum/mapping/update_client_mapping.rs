use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, log_decoder::LogDecoder},
    event::{update_client_event::UpdateClientEvent, supported::SupportedBlockEvent},
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
                client_id: log.client_id()?,
                counterparty_height: log.counterparty_height()?,
            },
        }])
    }
}
