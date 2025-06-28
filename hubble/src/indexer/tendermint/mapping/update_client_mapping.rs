use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    tendermint::{fetcher_client::TmFetcherClient, event_decoder::EventDecoder},
    event::{supported::SupportedBlockEvent, update_client_event::UpdateClientEvent},
};

impl TmFetcherClient {
    pub fn to_update_client(
        &self,
        log: &EventDecoder,
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
