use std::collections::HashSet;

use axum::async_trait;
use color_eyre::eyre::{eyre, Report};
use cometbft_rpc::{
    rpc_types::{BlockMeta, BlockResponse, CommitResponse},
    types::types::{block_id::BlockId, header::Header},
};
use futures::Stream;
use itertools::Itertools;
use serde_json::Value;
use sqlx::Postgres;
use time::OffsetDateTime;
use tracing::{debug, trace};

use crate::indexer::{
    api::{
        BlockHandle, BlockRange, BlockReference, BlockReferenceProvider, FetchMode, IndexerError,
    },
    tendermint::{
        fetcher_client::TmFetcherClient,
        postgres::{
            active_contracts, delete_tm_block_transactions_events, insert_batch_blocks,
            insert_batch_events, insert_batch_transactions, PgBlock, PgEvent, PgTransaction,
        },
        provider::RpcProviderId,
    },
};

#[derive(Clone)]
pub struct BlockHeader {
    pub block_id: BlockId,
    pub header: Header,
}

impl From<BlockResponse> for BlockHeader {
    fn from(block_response: BlockResponse) -> Self {
        BlockHeader {
            block_id: block_response.block_id,
            header: block_response.block.header,
        }
    }
}

impl From<BlockMeta> for BlockHeader {
    fn from(block_meta: BlockMeta) -> Self {
        BlockHeader {
            block_id: block_meta.block_id,
            header: block_meta.header,
        }
    }
}

impl From<CommitResponse> for BlockHeader {
    fn from(commit_response: CommitResponse) -> Self {
        BlockHeader {
            block_id: commit_response.signed_header.commit.block_id,
            header: commit_response.signed_header.header,
        }
    }
}

impl BlockReferenceProvider for BlockHeader {
    fn block_reference(&self) -> Result<BlockReference, Report> {
        Ok(BlockReference {
            height: self.header.height.inner().try_into().unwrap(),
            hash: self
                .block_id
                .hash
                .ok_or(IndexerError::ProviderError(eyre!("expected hash")))?
                .to_string(),
            timestamp: OffsetDateTime::from_unix_timestamp_nanos(
                self.header.time.as_unix_nanos().into(),
            )
            .map_err(|err| IndexerError::ProviderError(err.into()))?,
        })
    }
}

impl BlockReferenceProvider for BlockMeta {
    fn block_reference(&self) -> Result<BlockReference, Report> {
        Ok(BlockReference {
            height: self.header.height.inner().try_into().unwrap(),
            hash: self
                .block_id
                .hash
                .ok_or(IndexerError::ProviderError(eyre!("expected hash")))?
                .to_string(),
            timestamp: OffsetDateTime::from_unix_timestamp_nanos(
                self.header.time.as_unix_nanos().into(),
            )
            .map_err(|err| IndexerError::ProviderError(err.into()))?,
        })
    }
}

#[derive(Clone)]
pub enum BlockDetails {
    Lazy(Box<BlockHeader>),
    Eager(PgBlock, Vec<PgTransaction>, Vec<PgEvent>),
}

#[derive(Clone)]
pub struct TmBlockHandle {
    pub internal_chain_id: i32,
    pub reference: BlockReference,
    pub details: BlockDetails,
    pub tm_client: TmFetcherClient,
    pub provider_id: RpcProviderId,
}

impl TmBlockHandle {
    async fn get_block_insert(
        &self,
    ) -> Result<(PgBlock, Vec<PgTransaction>, Vec<PgEvent>), Report> {
        Ok(match self.details.clone() {
            BlockDetails::Eager(block, transactions, events) => (block, transactions, events),
            BlockDetails::Lazy(block_header) => {
                self.tm_client
                    .fetch_details(&block_header, self.provider_id)
                    .await?
            }
        })
    }
}

// checking if the _contract_address of the event exists in active_contracts.
// evaluating a json like below. we only include if:
// - event type starts with 'wasm-'
// - event attribute with key '_contract_address' exist and it's value exists in provided active contracts
//
// {
//     "type": "wasm-packet_send",
//     "attributes": [
//       {
//         "key": "_contract_address",
//         "index": true,
//         "value": "union17e93ukhcyesrvu72cgfvamdhyracghrx4f7ww89rqjg944ntdegscxepme"
//       },
//       ...
//
//     ]
// }
// or
// [
//    list of events as above (we ignore the arrays, because they are not related to a transaction)
// ]
//
fn should_include_event(
    reference: &BlockReference,
    event_data: &Value,
    active_contracts: &HashSet<String>,
) -> bool {
    // the event property that contains the wasm contract address that emitted the event
    const TYPE: &str = "type";

    // we only consider wasm events (that start with wasm-)
    const TYPE_WASM_PREFIX: &str = "wasm-";
    const TYPE_WASM_EVENT: &str = "wasm";

    // property that holds the event attributes
    const ATTRIBUTES: &str = "attributes";

    // property that holds the attribute key
    const ATTRIBUTE_KEY: &str = "key";

    // property that holds the attribute value
    const ATTRIBUTE_VALUE: &str = "value";

    // attribute we're looking for
    const ATTRIBUTE_KEY_FOR_CONTRACT_ADDRESS: &str = "_contract_address";

    let Value::Object(data) = event_data else {
        trace!(
            "{reference}: event is not of type object (probably a block event) => do not include"
        );
        return false;
    };

    // 1. fetch the event type.
    let Some(Value::String(event_type)) = data.get(TYPE) else {
        trace!(
            "{reference}: unexpected: event has no type with String value: {:?} => do not include",
            data.get(TYPE)
        );
        return false;
    };

    // 2. starts with 'wasm-'
    if !event_type.starts_with(TYPE_WASM_PREFIX) && event_type != TYPE_WASM_EVENT {
        trace!("{reference}: not a wasm event type: {event_type} => do not include");
        return false;
    }

    // 3. fetch the attributes
    let Some(Value::Array(event_attributes)) = data.get(ATTRIBUTES) else {
        trace!(
            "{reference}: unexpected: event has no attributes with Array value: {:?} => do not include",
            data.get(ATTRIBUTES)
        );
        return false;
    };

    // 4. evaluate the attributes
    for event_attribute in event_attributes {
        let Value::Object(event_attribute) = event_attribute else {
            trace!("{reference}: unexpected: event has attribute that is not an Object: {event_attribute:?}");
            continue;
        };

        // 5. fetch the attribute with key
        let Some(Value::String(key)) = event_attribute.get(ATTRIBUTE_KEY) else {
            trace!(
                "{reference}: unexpected: event attribute no 'key' of type String: {:?}",
                event_attribute.get(ATTRIBUTE_KEY)
            );
            continue;
        };

        // 6. find '_contract_address' attribute
        if ATTRIBUTE_KEY_FOR_CONTRACT_ADDRESS != key {
            // not a contract address attribute => check next attribute
            continue;
        }

        // 7. fetch the attribute value
        let Some(Value::String(event_contract_address)) = event_attribute.get(ATTRIBUTE_VALUE)
        else {
            trace!(
                "{reference}: unexpected: event attribute no 'value' of type String: {:?}",
                event_attribute.get(ATTRIBUTE_VALUE)
            );
            continue;
        };

        // 8. check if active
        let active = active_contracts.contains(event_contract_address);
        trace!("{reference}: found event contract address: {event_contract_address} => include: {active}");

        // found the contract address: include if active
        return active;
    }

    trace!(
        "{reference}: unexpected: there is no contract address in a wasm event => do not include"
    );

    false
}

#[async_trait]
impl BlockHandle for TmBlockHandle {
    fn reference(&self) -> BlockReference {
        self.reference.clone()
    }

    fn fetch_range(
        &self,
        block_range: BlockRange,
        fetch_mode: FetchMode,
    ) -> Result<impl Stream<Item = Result<Self, IndexerError>> + Send, IndexerError> {
        debug!("{}: fetching", block_range);

        self.tm_client
            .fetch_range_with_provider(block_range, fetch_mode, Some(self.provider_id))
    }

    async fn insert(&self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        let reference = self.reference();
        debug!("{}: inserting", reference);

        let (block, transactions, events) = self.get_block_insert().await?;

        let active_contracts = active_contracts(tx, self.internal_chain_id, block.height).await?;
        trace!("{reference}: active contracts: {}", active_contracts.len());

        let filtered_events = events
            .into_iter()
            .filter(|event| should_include_event(&self.reference, &event.data, &active_contracts))
            .collect_vec();

        let transaction_hashes_of_filtered_events = filtered_events
            .iter()
            .filter_map(|event| event.transaction_hash.clone())
            .collect::<HashSet<String>>();

        let filtered_transactions = transactions
            .into_iter()
            .filter(|transaction| transaction_hashes_of_filtered_events.contains(&transaction.hash))
            .collect_vec();

        if !&filtered_events.is_empty() {
            trace!(
                "{}: insert (transactions: {}, events:{})",
                reference,
                filtered_transactions.len(),
                filtered_events.len(),
            );

            insert_batch_blocks(tx, vec![block]).await?;
            insert_batch_transactions(tx, filtered_transactions).await?;
            insert_batch_events(tx, filtered_events).await?;
        } else {
            trace!("{}: ignore (no events for registered contracts)", reference);
        }

        debug!("{}: done", reference);
        Ok(())
    }

    async fn update(&self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        let reference = self.reference();
        debug!("{}: updating", reference);

        delete_tm_block_transactions_events(tx, self.tm_client.chain_id.db, self.reference.height)
            .await?;
        self.insert(tx).await?;

        debug!("{}: done", reference);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use serde_json::{json, Value};
    use time::OffsetDateTime;

    use crate::indexer::{api::BlockReference, tendermint::block_handle::should_include_event};

    #[tokio::test]
    async fn true_when_contract_address_is_active() {
        let should_include = get_should_include_event_result(
            &json!(
                {
                    "type": "wasm-packet_send",
                    "attributes": [
                    {
                        "key": "_contract_address",
                        "index": true,
                        "value": "union17e93ukhcyesrvu72cgfvamdhyracghrx4f7ww89rqjg944ntdegscxepme"
                    }
                    ]
                }
            ),
            &HashSet::from([
                "union17e93ukhcyesrvu72cgfvamdhyracghrx4f7ww89rqjg944ntdegscxepme".to_string(),
                "some-other-contract".to_string(),
            ]),
        );

        assert!(should_include);
    }

    #[tokio::test]
    async fn true_when_contract_address_is_active_in_a_wasm_event() {
        let should_include = get_should_include_event_result(
            &json!(
                {
                    "type": "wasm-packet_send",
                    "attributes": [
                    {
                        "key": "_contract_address",
                        "index": true,
                        "value": "union17e93ukhcyesrvu72cgfvamdhyracghrx4f7ww89rqjg944ntdegscxepme"
                    }
                    ]
                }
            ),
            &HashSet::from([
                "union17e93ukhcyesrvu72cgfvamdhyracghrx4f7ww89rqjg944ntdegscxepme".to_string(),
                "some-other-contract".to_string(),
            ]),
        );

        assert!(should_include);
    }

    #[tokio::test]
    async fn false_when_contract_address_is_not_active() {
        let should_include = get_should_include_event_result(
            &json!(
                {
                    "type": "wasm-packet_send",
                    "attributes": [
                    {
                        "key": "_contract_address",
                        "index": true,
                        "value": "union17e93ukhcyesrvu72cgfvamdhyracghrx4f7ww89rqjg944ntdegscxepme"
                    }
                    ]
                }
            ),
            &HashSet::from([
                "one-contract".to_string(),
                "some-other-contract".to_string(),
            ]),
        );

        assert!(!should_include);
    }

    #[tokio::test]
    async fn false_when_there_is_no_contract_address_attribute() {
        let should_include = get_should_include_event_result(
            &json!(
                {
                    "type": "wasm-packet_send",
                    "attributes": [
                    {
                        "key": "not-a-contract-address",
                        "index": true,
                        "value": "union17e93ukhcyesrvu72cgfvamdhyracghrx4f7ww89rqjg944ntdegscxepme"
                    }
                    ]
                }
            ),
            &HashSet::from([
                "union17e93ukhcyesrvu72cgfvamdhyracghrx4f7ww89rqjg944ntdegscxepme".to_string(),
                "some-other-contract".to_string(),
            ]),
        );

        assert!(!should_include);
    }

    #[tokio::test]
    async fn false_when_type_is_not_wasm() {
        let should_include = get_should_include_event_result(
            &json!(
                {
                    "type": "wasmXpacket_send", // wasm events start with `wasm-`.
                    "attributes": [
                    {
                        "key": "_contract_address",
                        "index": true,
                        "value": "union17e93ukhcyesrvu72cgfvamdhyracghrx4f7ww89rqjg944ntdegscxepme"
                    }
                    ]
                }
            ),
            &HashSet::from([
                "union17e93ukhcyesrvu72cgfvamdhyracghrx4f7ww89rqjg944ntdegscxepme".to_string(),
                "some-other-contract".to_string(),
            ]),
        );

        assert!(!should_include);
    }

    #[tokio::test]
    async fn false_when_data_is_array() {
        let should_include = get_should_include_event_result(
            &json!(
                [
                    {
                        "type": "wasm-packet_send",
                        "attributes": [
                        {
                            "key": "_contract_address",
                            "index": true,
                            "value": "union17e93ukhcyesrvu72cgfvamdhyracghrx4f7ww89rqjg944ntdegscxepme"
                        }
                        ]
                    }
                ]
            ),
            &HashSet::from([
                "union17e93ukhcyesrvu72cgfvamdhyracghrx4f7ww89rqjg944ntdegscxepme".to_string(),
                "some-other-contract".to_string(),
            ]),
        );

        assert!(!should_include);
    }

    #[tokio::test]
    async fn true_when_first_contract_address_is_active_others_are_ignored() {
        let should_include = get_should_include_event_result(
            &json!(
                {
                    "type": "wasm-packet_send",
                    "attributes": [
                    {
                        "key": "_contract_address",
                        "index": true,
                        "value": "union17e93ukhcyesrvu72cgfvamdhyracghrx4f7ww89rqjg944ntdegscxepme"
                    },
                    {
                        "key": "_contract_address",
                        "index": true,
                        "value": "ignored_contract_address"
                    }
                    ]
                }
            ),
            &HashSet::from([
                "union17e93ukhcyesrvu72cgfvamdhyracghrx4f7ww89rqjg944ntdegscxepme".to_string(),
                "some-other-contract".to_string(),
            ]),
        );

        assert!(should_include);
    }

    #[tokio::test]
    async fn true_when_first_contract_address_is_active_others_are_ignored_in_a_wasm_event() {
        let should_include = get_should_include_event_result(
            &json!(
                {
                    "type": "wasm",
                    "attributes": [
                    {
                        "key": "_contract_address",
                        "index": true,
                        "value": "union17e93ukhcyesrvu72cgfvamdhyracghrx4f7ww89rqjg944ntdegscxepme"
                    },
                    {
                        "key": "_contract_address",
                        "index": true,
                        "value": "ignored_contract_address"
                    }
                    ]
                }
            ),
            &HashSet::from([
                "union17e93ukhcyesrvu72cgfvamdhyracghrx4f7ww89rqjg944ntdegscxepme".to_string(),
                "some-other-contract".to_string(),
            ]),
        );

        assert!(should_include);
    }

    fn get_should_include_event_result(data: &Value, contracts: &HashSet<String>) -> bool {
        should_include_event(
            &BlockReference::new(0, "hash".to_string(), OffsetDateTime::now_utc()),
            data,
            contracts,
        )
    }
}
