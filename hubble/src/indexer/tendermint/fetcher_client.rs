use std::{error::Error, fmt::Display};

use axum::async_trait;
use color_eyre::{
    eyre::{eyre, Report},
    Result,
};
use cometbft_rpc::{
    rpc_types::{BlockMeta, BlockResultsResponse, Order, TxResponse},
    JsonRpcError,
};
use futures::{
    join,
    stream::{BoxStream, FuturesOrdered},
    FutureExt, Stream, StreamExt, TryFutureExt,
};
use itertools::Itertools;
use jsonrpsee::types::{error::INTERNAL_ERROR_CODE, ErrorObject};
use time::OffsetDateTime;
use tokio::task::JoinSet;
use tracing::{debug, info, info_span, trace, Instrument};

use crate::{
    indexer::{
        api::{
            BlockHeight, BlockRange, BlockReferenceProvider, BlockSelection, FetchMode,
            FetcherClient, IndexerError,
        },
        tendermint::{
            block_handle::{BlockDetails, BlockHeader, TmBlockHandle},
            context::TmContext,
            postgres::{PgBlock, PgEvent, PgTransaction},
            provider::{Provider, RpcProviderId},
        },
    },
    postgres::{fetch_or_insert_chain_id_tx, ChainId},
};

#[derive(Clone)]
pub struct TmFetcherClient {
    pub chain_id: ChainId,
    pub provider: Provider,
    pub tx_search_max_page_size: u8,
}

impl Display for TmFetcherClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "chain_id: {}", self.chain_id)
    }
}

impl TmFetcherClient {
    pub fn fetch_range_with_provider(
        &self,
        block_range: BlockRange,
        fetch_mode: FetchMode,
        provider_id: Option<RpcProviderId>,
    ) -> Result<impl Stream<Item = Result<TmBlockHandle, IndexerError>> + use<'_>, IndexerError>
    {
        debug!("{}: fetching", block_range);

        let block_range_clone = block_range.clone();
        let futures = async move {
            let block_range_clone = block_range_clone.clone();
            let metas_response = self
                .provider
                .blockchain(
                    block_range_clone.start_inclusive,
                    block_range_clone.end_exclusive - 1,
                    provider_id,
                )
                .inspect_err(|e| debug!(?e, "{}: error fetching blocks", block_range_clone))
                .await;

            match metas_response {
                Ok(result) => self.handle_ok_fetching_metas(
                    result.response.block_metas,
                    fetch_mode,
                    result.provider_id,
                ),
                Err(error) => Self::handle_err_fetching_metas(error),
            }
        };

        Ok(futures.flatten_stream())
    }

    pub fn handle_ok_fetching_metas(
        &self,
        block_metas: Vec<BlockMeta>,
        fetch_mode: FetchMode,
        provider_id: RpcProviderId,
    ) -> BoxStream<Result<TmBlockHandle, IndexerError>> {
        FuturesOrdered::from_iter(
            block_metas
                .into_iter()
                .sorted_by_key(|meta| meta.header.height)
                .map(|meta| async move {
                    Ok(TmBlockHandle {
                        internal_chain_id: self.chain_id.db,
                        reference: meta.block_reference()?,
                        details: match fetch_mode {
                            FetchMode::Lazy => BlockDetails::Lazy(Box::new(meta.into())),
                            FetchMode::Eager => {
                                let (block, transactions, events) =
                                    self.fetch_details(&meta.into(), provider_id).await?;
                                BlockDetails::Eager(block, transactions, events)
                            }
                        },
                        tm_client: self.clone(),
                        provider_id,
                    })
                }),
        )
        .boxed()
    }

    pub async fn fetch_details(
        &self,
        block_header: &BlockHeader,
        provider_id: RpcProviderId,
    ) -> Result<(PgBlock, Vec<PgTransaction>, Vec<PgEvent>), IndexerError> {
        let block_reference = block_header.block_reference()?;

        info!("{}: fetch details", block_reference);

        let (block_results, transactions_response) = self
            .fetch_details_from_provider(block_reference.height, provider_id)
            .await?;

        self.check_consistency(provider_id, &block_results, &transactions_response)?;

        self.convert_to_pg_data(block_header, block_results, transactions_response)
    }

    pub async fn fetch_details_from_provider(
        &self,
        height: BlockHeight,
        provider_id: RpcProviderId,
    ) -> Result<(BlockResultsResponse, Vec<TxResponse>), IndexerError> {
        debug!("{}: fetching block results", height);
        let block_results = self
            .provider
            .block_results(height, Some(provider_id))
            .inspect_err(|e| debug!(?e, ?height, "error fetching block results"));

        let transactions_response = self
            .fetch_transactions_for_block(height, None, provider_id)
            .inspect_err(|e| debug!(?e, "error fetching transactions for block"));

        // wait for results
        let (block_results, transactions_response) = join!(block_results, transactions_response);

        // handle errors
        Ok((block_results?.response, transactions_response?))
    }

    pub fn check_consistency(
        &self,
        provider_id: RpcProviderId,
        block_results: &BlockResultsResponse,
        transactions_response: &[TxResponse],
    ) -> Result<(), IndexerError> {
        let txs_event_count: usize = transactions_response
            .iter()
            .map(|tx| tx.tx_result.events.len())
            .sum();

        let block_tx_event_count: usize = block_results
            .txs_results
            .iter()
            .map(|tx_results| {
                tx_results
                    .iter()
                    .map(|tx_result| tx_result.events.len())
                    .sum::<usize>()
            })
            .sum();

        match txs_event_count == block_tx_event_count {
            true => Ok(()),
            false => Err(IndexerError::ProviderError(eyre!("provider: {:?} at height {} block_results tx events: {} <> transactions events: {}",
                    provider_id,
                    block_results.height,
                    block_tx_event_count,
                    txs_event_count
                )
            )),
        }
    }

    pub fn convert_to_pg_data(
        &self,
        block_header: &BlockHeader,
        block_results: BlockResultsResponse,
        transactions_response: Vec<TxResponse>,
    ) -> Result<(PgBlock, Vec<PgTransaction>, Vec<PgEvent>), IndexerError> {
        let (block_id, header, block_reference) = (
            block_header.block_id.clone(),
            block_header.header.clone(),
            block_header.block_reference()?,
        );

        let pg_block = PgBlock {
            chain_id: self.chain_id,
            hash: block_id
                .hash
                .ok_or(IndexerError::ProviderError(eyre!("expected hash")))?
                .to_string(),
            height: header.height.inner().try_into().unwrap(),
            time: OffsetDateTime::from_unix_timestamp_nanos(header.time.as_unix_nanos().into())
                .map_err(|err| IndexerError::ProviderError(err.into()))?,
            data: serde_json::to_value(&header)
                .unwrap()
                .replace_escape_chars(),
        };

        // Initial capacity is a bit of an estimate, but shouldn't need to resize too often.
        let mut pg_events = Vec::with_capacity(4 * 10);

        let mut block_index = 0;

        let pg_transactions =
            transactions_response
                .into_iter()
                .filter(|tx| tx.tx_result.code.is_ok())
                .map(|tx| {
                    let transaction_hash = tx.hash.to_string();
                    let data = serde_json::to_value(&tx).unwrap().replace_escape_chars();
                    pg_events.extend(tx.tx_result.events.into_iter().enumerate().map(
                        |(i, event)| {
                            let event = PgEvent {
                                chain_id: self.chain_id,
                                block_hash: block_reference.hash.clone(),
                                block_height: block_reference.height,
                                time: block_reference.timestamp,
                                data: serde_json::to_value(event).unwrap().replace_escape_chars(),
                                transaction_hash: Some(transaction_hash.clone()),
                                transaction_index: Some(i.try_into().unwrap()),
                                block_index,
                            };

                            block_index += 1;
                            event
                        },
                    ));
                    PgTransaction {
                        chain_id: self.chain_id,
                        block_hash: block_reference.hash.clone(),
                        block_height: block_reference.height,
                        time: block_reference.timestamp,
                        data,
                        hash: transaction_hash,
                        index: tx.index.try_into().unwrap(),
                    }
                })
                .collect::<Vec<_>>();

        // add all block events
        pg_events.extend(
            block_results
                .events(
                    self.chain_id,
                    block_reference.hash,
                    block_reference.timestamp,
                )
                .into_iter()
                .enumerate()
                .map(|(i, e)| {
                    let index: i32 = i.try_into().unwrap();
                    PgEvent {
                        block_index: index + block_index,
                        ..e
                    }
                }),
        );

        Ok((pg_block, pg_transactions, pg_events))
    }

    pub fn handle_err_fetching_metas(
        error: JsonRpcError,
    ) -> BoxStream<'static, Result<TmBlockHandle, IndexerError>> {
        futures::stream::once(async move { Err(error.into()) }).boxed()
    }

    pub async fn fetch_single_with_provider(
        &self,
        selection: BlockSelection,
        mode: FetchMode,
        provider_id: Option<RpcProviderId>,
    ) -> Result<TmBlockHandle, IndexerError> {
        debug!("{}: fetching", selection);

        let block_header: Result<Option<(RpcProviderId, BlockHeader)>, JsonRpcError> =
            match selection {
                BlockSelection::LastFinalized => self
                    .provider
                    .latest_block(provider_id)
                    .inspect_err(|e| debug!(?e, "error fetching latest block"))
                    .await
                    .map(|response| Some((response.provider_id, response.response.into()))),
                BlockSelection::Height(height) => match self
                    .provider
                    .block(height, provider_id)
                    .inspect_err(|e| debug!(?e, "error fetching block at {}", height))
                    .await
                {
                    Ok(result) => Ok(Some((result.provider_id, result.response.into()))),
                    Err(err) => Self::detect_reading_beyond_tip(err, &selection),
                },
            };

        match block_header {
            Ok(Some((provider_id, header))) => {
                debug!(
                    "{}: fetched (provider id: {:?})",
                    selection,
                    provider_id.clone()
                );

                Ok(TmBlockHandle {
                    internal_chain_id: self.chain_id.db,
                    reference: header.block_reference()?,
                    details: match mode {
                        FetchMode::Lazy => BlockDetails::Lazy(Box::new(header)),
                        FetchMode::Eager => {
                            let (block, transactions, events) =
                                self.fetch_details(&header, provider_id).await?;
                            BlockDetails::Eager(block, transactions, events)
                        }
                    },
                    tm_client: self.clone(),
                    provider_id,
                })
            }
            Ok(None) => {
                info!("{}: does not exist", selection);

                Err(IndexerError::NoBlock(selection))
            }
            Err(report) => {
                info!("{}: error: {}", selection, report);

                Err(report.into())
            }
        }
    }

    fn detect_reading_beyond_tip(
        error: JsonRpcError,
        selection: &BlockSelection,
    ) -> Result<Option<(RpcProviderId, BlockHeader)>, JsonRpcError> {
        if let Some(source) = error.source() {
            if let Some(error_object) = source.downcast_ref::<ErrorObject>() {
                if let (INTERNAL_ERROR_CODE, Some(message)) = (
                    error_object.code(),
                    error_object.data().map(|data| data.to_string()),
                ) {
                    if message.contains("must be less than or equal to")
                        || message.contains("could not find results for height")
                    {
                        trace!("{}: no block: beyond tip error: {}", selection, message);

                        return Ok(None); // we're reading beyond the tip
                    };
                };
            };
        };

        Err(error)
    }

    async fn fetch_transactions_for_block(
        &self,
        height: BlockHeight,
        expected: impl Into<Option<usize>>,
        provider_id: RpcProviderId,
    ) -> Result<Vec<TxResponse>, Report> {
        let expected = expected.into();

        debug!("{}: fetching", height);
        let mut txs = if let Some(expected) = expected {
            Vec::with_capacity(expected)
        } else {
            vec![]
        };

        for page in 1..u32::MAX {
            debug!("{height}: fetching transactions page {page}");
            let response = self
                .provider
                .tx_search(
                    height,
                    false,
                    page,
                    self.tx_search_max_page_size,
                    Order::Asc,
                    Some(provider_id),
                )
                .await?
                .response;
            let len = response.txs.len();
            txs.extend(response.txs);

            // We always query for the maximum page size. If we get less items, we know pagination is done
            let tx_search_max_page_size: u32 = self.tx_search_max_page_size.into();
            let len: u32 = len.try_into().unwrap();
            let current_count = (page - 1) * tx_search_max_page_size + len;
            let total_count = response.total_count;

            debug!("{height}: fetched transactions page {page} ({current_count}/{total_count})");

            match current_count.cmp(&response.total_count) {
                std::cmp::Ordering::Less => debug!("{height}: fetching transaction => next page"),
                std::cmp::Ordering::Equal => {
                    debug!("{height}: fetched all transactions");
                    break;
                }
                std::cmp::Ordering::Greater => {
                    debug!("{height}: fetched more transactions ({current_count}) than expected ({total_count})");
                    let var_name = eyre!(
                        "fetched more transactions ({current_count}) than expected ({total_count})"
                    );
                    return Err(var_name);
                }
            }

            // If we deduce the number from expected, we end pagination once we reach expected.
            if txs.len() == expected.unwrap_or(usize::MAX) {
                break;
            }
        }

        if let Some(expected) = expected {
            assert_eq!(txs.len(), expected, "block {height}");
        }
        Ok(txs)
    }
}

pub trait BlockExt {
    /// Returns the non-tx related events from a block formatted for insertion.
    fn events(self, chain_id: ChainId, block_hash: String, time: OffsetDateTime) -> Vec<PgEvent>;
}

impl BlockExt for BlockResultsResponse {
    fn events(self, chain_id: ChainId, block_hash: String, time: OffsetDateTime) -> Vec<PgEvent> {
        let finalize_block_events = self.finalize_block_events.iter().map(|e| PgEvent {
            chain_id,
            block_hash: block_hash.clone(),
            block_height: self.height,
            time,
            data: serde_json::to_value(e).unwrap().replace_escape_chars(),
            transaction_hash: None,
            transaction_index: None,
            block_index: 0,
        });

        finalize_block_events
            .enumerate()
            .map(|(i, mut event)| {
                event.block_index = i.try_into().unwrap();
                event
            })
            .collect()
    }
}

#[derive(serde::Serialize)]
pub struct WithType<I> {
    #[serde(rename = "type")]
    kind: &'static str,
    #[serde(flatten)]
    inner: I,
}

trait SerdeValueExt {
    fn replace_escape_chars(self) -> Self;
}

impl SerdeValueExt for serde_json::Value {
    /// Replaces \u0000 from JSON objects and replaces it with \\u0000
    fn replace_escape_chars(mut self) -> Self {
        replace_escape_chars(&mut self);
        self
    }
}

fn replace_escape_chars(val: &mut serde_json::Value) {
    use base64::{engine::general_purpose, Engine as _};

    match val {
        serde_json::Value::Null => (),
        serde_json::Value::Bool(_) => (),
        serde_json::Value::Number(_) => (),
        serde_json::Value::String(ref mut data) => {
            if data.contains('\u{0000}') {
                // https://github.com/rust-lang/rust-clippy/issues/12856
                #[allow(clippy::needless_borrows_for_generic_args)]
                let encoded = general_purpose::STANDARD.encode(&data);
                *data = encoded;
            }
        }
        serde_json::Value::Array(ref mut arr) => {
            for item in arr.iter_mut() {
                replace_escape_chars(item)
            }
        }
        serde_json::Value::Object(ref mut obj) => {
            for item in obj.values_mut() {
                replace_escape_chars(item)
            }
        }
    }
}

#[async_trait]
impl FetcherClient for TmFetcherClient {
    type BlockHandle = TmBlockHandle;
    type Context = TmContext;

    async fn create(
        pg_pool: sqlx::PgPool,
        _join_set: &mut JoinSet<Result<(), IndexerError>>,
        context: TmContext,
    ) -> Result<Self, IndexerError> {
        let provider = Provider::new(context.rpc_urls).await?;

        info!("fetching chain-id from node");
        let chain_id = provider
            .status(None)
            .inspect_err(|e| debug!(?e, "error fetching chain-id: {}", e))
            .await?
            .response
            .node_info
            .network
            .to_string();

        info!("fetched chain-id from node: {}", chain_id);

        let indexing_span = info_span!("indexer", chain_id = chain_id).or_current();
        async move {
            let mut tx = pg_pool.begin().await?;

            let chain_id = fetch_or_insert_chain_id_tx(&mut tx, chain_id.to_string())
                .await?
                .get_inner_logged();

            tx.commit().await?;

            Ok(TmFetcherClient {
                chain_id,
                provider,
                tx_search_max_page_size: context.tx_search_max_page_size,
            })
        }
        .instrument(indexing_span)
        .await
    }

    async fn fetch_single(
        &self,
        selection: BlockSelection,
        mode: FetchMode,
    ) -> Result<Self::BlockHandle, IndexerError> {
        self.fetch_single_with_provider(selection, mode, None).await
    }
}
