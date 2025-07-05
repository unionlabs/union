use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, mapping::decoder::Decoder},
    event::{supported::SupportedBlockEvent, update_client_event::UpdateClientEvent},
};

impl EthFetcherClient {
    pub fn to_update_client(
        &self,
        decoder: &Decoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_update_client - {decoder}");

        Ok(vec![SupportedBlockEvent::UpdateClient {
            inner: UpdateClientEvent {
                header: decoder.header()?,
                client_id: decoder.event.client_id()?,
                counterparty_height: decoder.event.counterparty_height()?,
            },
        }])
    }
}
