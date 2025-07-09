use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{supported::SupportedBlockEvent, update_client_event::UpdateClientEvent},
    tendermint::{fetcher_client::TmFetcherClient, mapping::decoder::Decoder},
};

impl TmFetcherClient {
    pub fn to_update_client(
        &self,
        log: &Decoder,
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
