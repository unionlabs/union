use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, mapping::decoder::Decoder},
    event::{packet_send_event::PacketSendEvent, supported::SupportedBlockEvent},
};

impl EthFetcherClient {
    pub fn to_packet_send(
        &self,
        decoder: &Decoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_packet_send - {decoder}");

        let packet = decoder.event.packet()?;

        Ok(vec![SupportedBlockEvent::PacketSend {
            inner: PacketSendEvent {
                header: decoder.header()?,
                channel_id: decoder.event.channel_id()?,
                packet_hash: decoder.event.packet_hash()?,
                source_channel_id: packet.source_channel_id()?,
                destination_channel_id: packet.destination_channel_id()?,
                timeout_height: packet.timeout_height()?,
                timeout_timestamp: packet.timeout_timestamp()?,
                data: packet.data()?,
            },
        }])
    }
}
