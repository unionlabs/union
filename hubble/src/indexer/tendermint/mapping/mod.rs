use cometbft_rpc::{rpc_types::TxResponse, types::abci::event::Event};
use tracing::{trace, warn};

use crate::indexer::{
    api::{BlockReference, IndexerError},
    event::supported::SupportedBlockEvent,
    tendermint::{
        block_handle::{ActiveContracts, BlockHeader},
        fetcher_client::TmFetcherClient,
        mapping::decoder::Decoder,
    },
};

mod channel_open_ack_mapping;
mod channel_open_confirm_mapping;
mod channel_open_init_mapping;
mod channel_open_try_mapping;
mod connection_open_ack_mapping;
mod connection_open_confirm_mapping;
mod connection_open_init_mapping;
mod connection_open_try_mapping;
mod create_client_mapping;
mod create_lens_client_mapping;
mod decoder;
pub(crate) mod legacy;
mod packet_ack_mapping;
mod packet_recv_mapping;
mod packet_send_mapping;
mod packet_timeout_mapping;
mod token_bucket_update_mapping;
mod update_client_mapping;
mod wallet_mutation_entry_mapping;
mod write_ack_mapping;

impl TmFetcherClient {
    pub fn transform_to_ucs_events(
        &self,
        block_reference: &BlockReference,
        active_contracts: &ActiveContracts,
        block_header: &BlockHeader,
        transactions: &[TxResponse],
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        let mut event_index_in_block: usize = 0;

        transactions
            .iter()
            .filter(|tx| tx.tx_result.code.is_ok())
            .map(|transaction| {
                let result = self.transform_transaction_to_ucs_events(
                    block_reference,
                    active_contracts,
                    block_header,
                    transaction,
                    event_index_in_block,
                );

                event_index_in_block += transaction.tx_result.events.len();

                result
            })
            .collect::<Result<Vec<_>, _>>() // Result<Vec<Vec<SupportedBlockEvent>>, IndexerError>
            .map(|vecs| vecs.into_iter().flatten().collect())
    }

    fn transform_transaction_to_ucs_events(
        &self,
        block_reference: &BlockReference,
        active_contracts: &ActiveContracts,
        block_header: &BlockHeader,
        transaction: &TxResponse,
        event_index_of_first_event_in_transaction: usize,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        transaction
            .tx_result
            .events
            .iter()
            .enumerate()
            .map(|(event_index_in_transaction, event)| {
                self.transform_event_to_ucs_events(
                    block_reference,
                    active_contracts,
                    block_header,
                    transaction,
                    event,
                    event_index_of_first_event_in_transaction + event_index_in_transaction,
                )
            })
            .collect::<Result<Vec<_>, _>>() // Result<Vec<Vec<SupportedBlockEvent>>, IndexerError>
            .map(|vecs| vecs.into_iter().flatten().collect())
    }

    fn transform_event_to_ucs_events(
        &self,
        block_reference: &BlockReference,
        active_contracts: &ActiveContracts,
        block_header: &BlockHeader,
        transaction: &TxResponse,
        event: &Event,
        event_index: usize,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        let Some(wasm_contract_address) = wasm_contract_address(block_reference, event) else {
            trace!(
                "{block_reference}, {}-{} has no wasm contract address",
                transaction.hash,
                event.ty
            );
            return Ok(vec![]);
        };

        let Some(flows) = active_contracts.flows(&wasm_contract_address) else {
            trace!("{block_reference}, {}-{} has wasm contract address {wasm_contract_address} is not registered", transaction.hash, event.ty);
            return Ok(vec![]);
        };

        let event = &event.into();

        let event_decoder = Decoder {
            chain_id: self.chain_id,
            block_header,
            transaction,
            event,
            event_index,
        };

        flows.iter().map(|flow|match flow.as_str() {
            "ibc" => self.transform_ibc_event_to_ucs_events(&event_decoder),
            "cw20" => self.transform_cw20_event_to_ucs_events(&event_decoder),
            unsupported => {
                warn!("ignoring unsupported flow {unsupported} flow for contract {wasm_contract_address}");
                Ok(vec![])
            }
        })
            .collect::<Result<Vec<_>, _>>() // Result<Vec<Vec<SupportedBlockEvent>>, IndexerError>
            .map(|vecs| vecs.into_iter().flatten().collect())
    }

    fn transform_ibc_event_to_ucs_events(
        &self,
        event_decoder: &Decoder<'_>,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_ibc_event - {event_decoder}");

        Ok(match event_decoder.event.name.as_str() {
            "wasm-channel_open_init" => self.to_channel_open_init(event_decoder)?,
            "wasm-channel_open_try" => self.to_channel_open_try(event_decoder)?,
            "wasm-channel_open_ack" => self.to_channel_open_ack(event_decoder)?,
            "wasm-channel_open_confirm" => self.to_channel_open_confirm(event_decoder)?,
            "wasm-connection_open_init" => self.to_connection_open_init(event_decoder)?,
            "wasm-connection_open_try" => self.to_connection_open_try(event_decoder)?,
            "wasm-connection_open_ack" => self.to_connection_open_ack(event_decoder)?,
            "wasm-connection_open_confirm" => self.to_connection_open_confirm(event_decoder)?,
            "wasm-create_client" => self.to_create_client(event_decoder)?,
            "wasm-create_lens_client" => self.to_create_lens_client(event_decoder)?,
            "wasm-update_client" => self.to_update_client(event_decoder)?,
            "wasm-packet_send" => self.to_packet_send(event_decoder)?,
            "wasm-packet_recv" => self.to_packet_recv(event_decoder)?,
            "wasm-write_ack" => self.to_write_ack(event_decoder)?,
            "wasm-packet_ack" => self.to_packet_ack(event_decoder)?,
            "wasm-packet_timeout" => self.to_packet_timeout(event_decoder)?,
            "wasm-token_bucket_update" => self.to_token_bucket_update(event_decoder)?,
            name => {
                warn!("unsupported ibc event: {name} ({event_decoder})");
                vec![]
            }
        })
    }

    fn transform_cw20_event_to_ucs_events(
        &self,
        event_decoder: &Decoder<'_>,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_cw20_event - {event_decoder}");

        #[allow(clippy::match_single_binding)] // remove after adding support
        Ok(match event_decoder.event.name.as_str() {
            "wasm" => self.to_wallet_mutation_entry(event_decoder)?,
            name => {
                warn!("unsupported ibc event: {name} ({event_decoder})");
                vec![]
            }
        })
    }
}

// extracting the _contract_address of the event (if it exists in active_contracts).
// evaluating a json like below. we only include if:
// - event type starts with 'wasm-'
// - event attribute with key '_contract_address' exist and it's value exists in provided active contracts
fn wasm_contract_address(reference: &BlockReference, event: &Event) -> Option<String> {
    // we only consider wasm events (that start with wasm-)
    const TYPE_WASM_PREFIX: &str = "wasm-";
    const TYPE_WASM_EVENT: &str = "wasm";

    // attribute we're looking for
    const ATTRIBUTE_KEY_FOR_CONTRACT_ADDRESS: &str = "_contract_address";

    let event_type = &event.ty;

    // starts with 'wasm-'
    if !event_type.starts_with(TYPE_WASM_PREFIX) && event_type != TYPE_WASM_EVENT {
        trace!("{reference}: not a wasm event type: {event_type} => do not include");
        return None;
    }

    // first contract address value
    let result: Option<String> = event.attributes.iter().find_map(|event| {
        match ATTRIBUTE_KEY_FOR_CONTRACT_ADDRESS == event.key {
            true => Some(event.value.clone()),
            false => None,
        }
    });

    if result.is_none() {
        trace!(
            "{reference}: unexpected: there is no contract address in a wasm event => do not include"
        );
    }

    result
}
