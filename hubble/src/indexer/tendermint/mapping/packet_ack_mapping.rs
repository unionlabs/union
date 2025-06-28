use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{packet_ack_event::PacketAckEvent, supported::SupportedBlockEvent},
    tendermint::{event_decoder::EventDecoder, fetcher_client::TmFetcherClient},
};

impl TmFetcherClient {
    pub fn to_packet_ack(
        &self,
        log: &EventDecoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_packet_ack - {log}");

        Ok(vec![SupportedBlockEvent::PacketAck {
            inner: PacketAckEvent {
                header: log.header()?,
                channel_id: log.event.channel_id()?,
                packet_hash: log.event.packet_hash()?,
                acknowledgement: log.event.acknowledgement()?,
                maker: log.event.maker()?,
            },
        }])
    }
}
