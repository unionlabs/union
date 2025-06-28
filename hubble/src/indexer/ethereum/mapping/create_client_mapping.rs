use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, mapping::decoder::Decoder},
    event::{create_client_event::CreateClientEvent, supported::SupportedBlockEvent},
};

impl EthFetcherClient {
    pub fn to_create_client(
        &self,
        decoder: &Decoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_create_client - {decoder}");

        Ok(vec![SupportedBlockEvent::CreateClient {
            inner: CreateClientEvent {
                header: decoder.header()?,
                client_id: decoder.event.client_id()?,
                client_type: decoder.event.client_type()?,
                counterparty_chain_id: decoder.event.counterparty_chain_id()?,
            },
        }])
    }
}
