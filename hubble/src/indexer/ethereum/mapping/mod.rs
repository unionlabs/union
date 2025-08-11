use std::collections::HashMap;

use alloy::{network::AnyRpcBlock, rpc::types::Log};
use itertools::Itertools;
use tracing::{trace, warn};

use crate::indexer::{
    api::IndexerError,
    ethereum::{
        abi::{Abi, AbiRegistration},
        fetcher_client::EthFetcherClient,
        mapping::decoder::Decoder,
    },
    event::supported::SupportedBlockEvent,
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
mod create_wrapped_token;
mod decoder;
pub(crate) mod legacy;
mod packet_ack_mapping;
mod packet_recv_mapping;
mod packet_send_mapping;
mod packet_timeout_mapping;
mod token_bucket_update_mapping;
mod update_client_mapping;
mod write_ack_mapping;

impl EthFetcherClient {
    pub fn transform_logs_to_ucs_events(
        &self,
        abi_registration: &AbiRegistration,
        block: &AnyRpcBlock,
        logs: &[Log],
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        // group events by transaction
        let events_by_transaction = {
            let mut map: HashMap<_, Vec<Log>> = HashMap::with_capacity(logs.len());
            for log in logs {
                if log.removed {
                    continue;
                }

                map.entry(log.transaction_index.unwrap())
                    .and_modify(|logs| logs.push(log.clone()))
                    .or_insert(vec![log.clone()]);
            }
            map
        };

        Ok(events_by_transaction
            .into_iter()
            .sorted_by_key(|(transaction_index, _)| *transaction_index)
            .map(|(_, logs)| {
                logs.iter()
                    .sorted_by_key(|e| e.log_index)
                    .enumerate()
                    .map(|(transaction_log_index, log)| {
                        self.transform_log_to_ucs_events(
                            abi_registration,
                            block,
                            transaction_log_index,
                            log,
                        )
                    })
                    .collect::<Result<Vec<Vec<SupportedBlockEvent>>, IndexerError>>()
            })
            .collect::<Result<Vec<Vec<Vec<SupportedBlockEvent>>>, IndexerError>>()?
            .into_iter()
            .flatten()
            .flatten()
            .collect())
    }

    fn transform_log_to_ucs_events(
        &self,
        abi_registration: &AbiRegistration,
        block: &AnyRpcBlock,
        transaction_log_index: usize,
        log: &Log,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        let Some(abi) = abi_registration.get_abi_for_address(&log.address()) else {
            return Err(IndexerError::AbiNoAbiForAddress(log.address()));
        };

        let mut events = self.to_ucs_events(abi, block, transaction_log_index, log)?;

        events.push(self.to_decoded_log(abi, block, transaction_log_index, log)?);

        Ok(events)
    }

    fn to_ucs_events(
        &self,
        abi: &Abi,
        block: &AnyRpcBlock,
        transaction_log_index: usize,
        log: &Log,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        let event = abi.parse(log)?;

        let log_decoder = Decoder {
            event: &event,
            chain_id: self.chain_id,
            block,
            log,
            transaction_log_index,
        };

        trace!("to_ucs_events - {log_decoder}");

        Ok(match event.name.as_str() {
            "ChannelOpenInit" => self.to_channel_open_init(&log_decoder)?,
            "ChannelOpenTry" => self.to_channel_open_try(&log_decoder)?,
            "ChannelOpenAck" => self.to_channel_open_ack(&log_decoder)?,
            "ChannelOpenConfirm" => self.to_channel_open_confirm(&log_decoder)?,
            "ConnectionOpenInit" => self.to_connection_open_init(&log_decoder)?,
            "ConnectionOpenTry" => self.to_connection_open_try(&log_decoder)?,
            "ConnectionOpenAck" => self.to_connection_open_ack(&log_decoder)?,
            "ConnectionOpenConfirm" => self.to_connection_open_confirm(&log_decoder)?,
            "CreateClient" => self.to_create_client(&log_decoder)?,
            "CreateLensClient" => self.to_create_lens_client(&log_decoder)?,
            "UpdateClient" => self.to_update_client(&log_decoder)?,
            "PacketSend" => self.to_packet_send(&log_decoder)?,
            "PacketRecv" => self.to_packet_recv(&log_decoder)?,
            "WriteAck" => self.to_write_ack(&log_decoder)?,
            "PacketAck" => self.to_packet_ack(&log_decoder)?,
            "PacketTimeout" => self.to_packet_timeout(&log_decoder)?,
            "TokenBucketUpdate" => self.to_token_bucket_update(&log_decoder)?,
            "CreateWrappedToken" => self.to_create_wrapped_token(&log_decoder)?,
            name => {
                warn!("unsupported event: {name} ({log:?})");
                vec![]
            }
        })
    }
}
