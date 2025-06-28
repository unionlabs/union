use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{supported::SupportedBlockEvent, write_ack_event::WriteAckEvent},
    tendermint::{event_decoder::EventDecoder, fetcher_client::TmFetcherClient},
};

impl TmFetcherClient {
    pub fn to_write_ack(
        &self,
        log: &EventDecoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
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
