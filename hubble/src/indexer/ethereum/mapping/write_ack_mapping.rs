use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, log_decoder::LogDecoder},
    event::{supported::SupportedBlockEvent, write_ack_event::WriteAckEvent},
};

impl EthFetcherClient {
    pub fn to_write_ack(&self, log: &LogDecoder) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_write_ack - {log}");

        Ok(vec![SupportedBlockEvent::WriteAck {
            inner: WriteAckEvent {
                header: log.header()?,
                channel_id: log.event.channel_id()?,
                packet_hash: log.event.packet_hash()?,
                acknowledgement: log.event.acknowledgement()?,
            },
        }])
    }
}
