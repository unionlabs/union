use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, log_decoder::LogDecoder},
    event::{packet_send_event::PacketSendEvent, supported::SupportedBlockEvent},
};

impl EthFetcherClient {
    pub fn to_packet_send(
        &self,
        log: &LogDecoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_packet_send - {log}");

        let packet = log.event.packet()?;

        Ok(vec![SupportedBlockEvent::PacketSend {
            inner: PacketSendEvent {
                header: log.header()?,
                channel_id: log.event.channel_id()?,
                packet_hash: log.event.packet_hash()?,
                source_channel_id: packet.source_channel_id()?,
                destination_channel_id: packet.destination_channel_id()?,
                timeout_height: packet.timeout_height()?,
                timeout_timestamp: packet.timeout_timestamp()?,
                data: packet.data()?,
            },
        }])
    }
}
