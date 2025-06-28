use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{packet_send_event::PacketSendEvent, supported::SupportedBlockEvent},
    tendermint::{event_decoder::EventDecoder, fetcher_client::TmFetcherClient},
};

impl TmFetcherClient {
    pub fn to_packet_send(
        &self,
        log: &EventDecoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_packet_send - {log}");

        Ok(vec![SupportedBlockEvent::PacketSend {
            inner: PacketSendEvent {
                header: log.header()?,
                channel_id: log.event.channel_id()?,
                packet_hash: log.event.packet_hash()?,
                source_channel_id: log.event.source_channel_id()?,
                destination_channel_id: log.event.destination_channel_id()?,
                timeout_height: log.event.timeout_height()?,
                timeout_timestamp: log.event.timeout_timestamp()?,
                data: log.event.data()?,
            },
        }])
    }
}
