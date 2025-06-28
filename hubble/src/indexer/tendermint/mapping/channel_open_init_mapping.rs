use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{channel_open_init_event::ChannelOpenInitEvent, supported::SupportedBlockEvent},
    tendermint::{fetcher_client::TmFetcherClient, mapping::decoder::Decoder},
};

impl TmFetcherClient {
    pub fn to_channel_open_init(
        &self,
        log: &Decoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_channel_open_init - {log}");

        Ok(vec![SupportedBlockEvent::ChannelOpenInit {
            inner: ChannelOpenInitEvent {
                header: log.header()?,
                connection_id: log.event.connection_id()?,
                channel_id: log.event.channel_id()?,
                port_id: log.event.port_id()?,
                counterparty_port_id: log.event.counterparty_port_id()?,
                version: log.event.version()?,
            },
        }])
    }
}
