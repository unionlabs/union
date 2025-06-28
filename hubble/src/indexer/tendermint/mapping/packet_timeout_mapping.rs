use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{packet_timeout_event::PacketTimeoutEvent, supported::SupportedBlockEvent},
    tendermint::{event_decoder::EventDecoder, fetcher_client::TmFetcherClient},
};

impl TmFetcherClient {
    pub fn to_packet_timeout(
        &self,
        log: &EventDecoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_packet_timeout - {log}");

        Ok(vec![SupportedBlockEvent::PacketTimeout {
            inner: PacketTimeoutEvent {
                header: log.header()?,
                channel_id: log.event.channel_id()?,
                packet_hash: log.event.packet_hash()?,
                maker: log.event.maker()?,
            },
        }])
    }
}
