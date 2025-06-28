use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{
        supported::SupportedBlockEvent,
        types::{MutationDirection, WalletAddress},
        wallet_mutation_entry_event::WalletMutationEntryEvent,
    },
    tendermint::{event_decoder::EventDecoder, fetcher_client::TmFetcherClient},
};

impl TmFetcherClient {
    pub fn to_wallet_mutation_entry(
        &self,
        log: &EventDecoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_wallet_mutation_entry - {log}");

        Ok(match log.event.action()?.as_str() {
            "burn" | "burn_from" | "send" | "send_from" | "transfer" | "transfer_from" | "mint" => {
                let mut events = Vec::new();

                // Add 'from' mutation (outgoing)
                if let Some(from_addr) = log.event.from_opt()? {
                    trace!("to_wallet_mutation_entry - {log} => out ({from_addr:?})");
                    events.push(self.create_event(log, from_addr, MutationDirection::Out)?);
                }

                // Add 'to' mutation (incoming)
                if let Some(to_addr) = log.event.to_opt()? {
                    trace!("to_wallet_mutation_entry - {log} => in ({to_addr:?})");
                    events.push(self.create_event(log, to_addr, MutationDirection::In)?);
                }

                events
            }
            unsupported => {
                trace!("to_wallet_mutation_entry - {log} => ignore ({unsupported})");

                vec![]
            }
        })
    }

    fn create_event(
        &self,
        log: &EventDecoder,
        wallet_address_canonical: WalletAddress,
        direction: MutationDirection,
    ) -> Result<SupportedBlockEvent, IndexerError> {
        Ok(SupportedBlockEvent::WalletMutationEntry {
            inner: WalletMutationEntryEvent {
                header: log.header()?,
                contract_address_canonical: log.event.contract_address()?,
                wallet_address_canonical,
                amount: log.event.amount()?,
                direction,
            },
        })
    }
}
