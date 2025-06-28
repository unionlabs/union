use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{channel_open_confirm_event::ChannelOpenConfirmEvent, supported::SupportedBlockEvent},
    tendermint::{event_decoder::EventDecoder, fetcher_client::TmFetcherClient},
};

impl TmFetcherClient {
    pub fn to_channel_open_confirm(
        &self,
        log: &EventDecoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_channel_open_confirm - {log}");

        Ok(vec![SupportedBlockEvent::ChannelOpenConfirm {
            inner: ChannelOpenConfirmEvent {
                header: log.header()?,
                connection_id: log.event.connection_id()?,
                channel_id: log.event.channel_id()?,
                port_id: log.event.port_id()?,
                counterparty_port_id: log.event.counterparty_port_id()?,
                counterparty_channel_id: log.event.counterparty_channel_id()?,
            },
        }])
    }
}
