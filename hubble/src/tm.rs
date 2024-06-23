use backon::Retryable;
use color_eyre::eyre::{bail, eyre, Report};
use futures::{stream, stream::TryStreamExt, TryFutureExt};
use sqlx::{Acquire, Postgres};
use tendermint::block::Height;
use tendermint_rpc::{
    endpoint::block_results::Response as BlockResponse,
    error::ErrorDetail,
    query::{Condition, Query},
    response_error::Code,
    Client, Error, HttpClient, Order,
};
use time::OffsetDateTime;
use tokio::time::{sleep, Duration};
use tracing::{debug, info, info_span, warn, Instrument};
use url::Url;

use crate::postgres::{self, ChainId};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub label: String,
    pub url: Url,
    /// The GRPC endpoint of this chain. required for `--fetch-client-chain-ids`.
    pub grpc_url: Option<String>,

    /// The height from which we start indexing
    pub start_height: Option<u32>,

    #[allow(dead_code)]
    pub until: Option<u64>,

    /// Attempt to retry and fix bad states. This makes the process less responsive, as any call may take longer
    /// since retries are happening. Best for systemd services and long-running jobs.
    pub harden: bool,
}

/// Unit struct describing parametrization of associated types for CosmosSDK based chains.
pub struct CosmosSDK;

impl postgres::ChainType for CosmosSDK {
    type BlockHash = String;
    type BlockHeight = i32;
    type TransactionHash = String;
}

pub type PgBlock = postgres::Block<CosmosSDK>;
pub type PgTransaction = postgres::Transaction<CosmosSDK>;
pub type PgEvent = postgres::Event<CosmosSDK>;

impl Config {
    /// The batch size for the fast sync protocol. This corresponds to the maximum number of headers returned over a node's RPC.
    pub const BATCH_SIZE: u32 = 20;

    pub async fn index<DB>(self, pool: DB) -> Result<(), Report>
    where
        for<'a> &'a DB:
            sqlx::Acquire<'a, Database = Postgres> + sqlx::Executor<'a, Database = Postgres>,
    {
        let client = HttpClient::new(self.url.as_str()).unwrap();

        let (chain_id, height) = if self.harden {
            (|| fetch_meta(&client, &pool).inspect_err(|e| debug!(?e, "error fetching meta")))
                .retry(&crate::expo_backoff())
                .await?
        } else {
            fetch_meta(&client, &pool).await?
        };
        let indexing_span = info_span!("indexer", chain_id = chain_id.canonical);
        async move {
            // Determine from which height we should start indexing if we haven't
            // indexed any blocks yet. If start_height > current_height, we jump to the new start height
            let mut height: Height = (height.unwrap_or_default().value() as u32)
                .max(self.start_height.unwrap_or_default())
                .into();

            // Fast sync protocol. We sync up to latest.height - batch-size + 1
            if let Some(up_to) = should_fast_sync_up_to(&client, Self::BATCH_SIZE, height).await? {
                debug!(
                    "syncing with batch size {} up to height {}",
                    Self::BATCH_SIZE,
                    up_to
                );
                loop {
                    let batch_end =
                        std::cmp::min(up_to.value(), height.value() + Self::BATCH_SIZE as u64);
                    if batch_end - height.value() != 20 {
                        break; // go back to the should_fast_sync_up_to. If this returns None, we continue to slow sync.
                    }

                    debug!(
                        "fast syncing for batch: {}..{}", height, batch_end
                    );
                    let mut tx = pool.begin().await?;
                    let next_height = fetch_and_insert_blocks(&client, &mut tx, chain_id, Self::BATCH_SIZE, height).await?.expect("batch sync with batch > 1 should error or succeed, but never reach head of chain");
                    tx.commit().await?;
                    info!(
                        "indexed blocks {}..{}",
                        height.value(),
                        next_height.value()
                    );
                    height = next_height
                }
            }

            info!(chain_id.canonical, "syncing block by block");
            let mut retry_count = 0;
            loop {
                debug!("starting regular sync protocol");
                // Regular sync protocol. This fetches blocks one-by-one.
                retry_count += 1;
                let mut tx = pool.begin().await?;
                match fetch_and_insert_blocks(&client, &mut tx, chain_id, 1, height).await? {
                    Some(h) => {
                        info!("indexed block {}", &height);
                        height = h;
                        retry_count = 0;
                        tx.commit().await?;
                    }
                    None => {
                        if retry_count > 30 {
                            warn!("node {chain_id} has stopped providing new blocks");
                            bail!("node {chain_id} has stopped providing new blocks");
                        }
                        retry_count += 1;
                        tx.rollback().await?;
                        debug!("caught up indexing, sleeping for 1 second");
                        sleep(Duration::from_millis(1000)).await;
                        continue;
                    }
                }
            }
        }.instrument(indexing_span).await
    }
}

/// fetches the ChainId for a given `HttpClient`, among with the `Height` up until we have indexed that chain in the DB.
/// If we have not yet indexed any block for that chain, then Height = None.
async fn fetch_meta<DB>(client: &HttpClient, pool: &DB) -> Result<(ChainId, Option<Height>), Report>
where
    for<'a> &'a DB:
        sqlx::Acquire<'a, Database = Postgres> + sqlx::Executor<'a, Database = Postgres>,
{
    info!(?client, "fetching chain-id from node");
    let chain_id = (|| {
        client
            .status()
            .inspect_err(|e| debug!(?e, "error fetching client-id"))
    })
    .retry(&crate::expo_backoff())
    .await?
    .node_info
    .network
    .as_str()
    .to_owned();
    info!(?client, "chain-id is {}", &chain_id);

    let chain_id = postgres::fetch_or_insert_chain_id(pool, chain_id)
        .await?
        .get_inner_logged();
    let height = sqlx::query!(
        r#"SELECT MAX(height) height FROM "v0".blocks WHERE chain_id = $1"#,
        chain_id.db
    )
    .fetch_optional(pool)
    .await?
    .map(|block| block.height.unwrap_or(0) + 1)
    .map(|h| Height::from(h as u32));
    Ok((chain_id, height))
}

/// The RPC will return an internal error on queries for blocks exceeding the current height.
/// `is_height_exceeded_error` untangles the error and checks for this case.
pub fn is_height_exceeded_error(err: &Error) -> bool {
    let detail = err.detail();
    if let ErrorDetail::Response(err) = detail {
        let inner = &err.source;
        let code = inner.code();
        let message = inner.data().unwrap_or_default();
        return matches!(code, Code::InternalError)
            && (message.contains("must be less than or equal to")
                | message.contains("could not find results for height"));
    }
    false
}

/// Queries the node and current indexed height and determines if fast sync should be applied.
///
/// # Returns
/// The block up to which to fast sync.
///
/// # Errors
/// On IO errors when communicating with the node.
async fn should_fast_sync_up_to(
    client: &HttpClient,
    batch_size: u32,
    current: Height,
) -> Result<Option<Height>, Report> {
    debug!(?client, "getting latest block to fast sync up to");
    let latest = (|| {
        client
            .latest_block()
            .inspect_err(|e| debug!(?e, "error fetching latest block"))
    })
    .retry(&crate::expo_backoff())
    .await?
    .block
    .header
    .height;
    if latest.value() - current.value() >= batch_size.into() {
        Ok(Some(latest))
    } else {
        Ok(None)
    }
}

/// Fetches and inserts blocks into the database. Will attempt to use more optimal RPC calls (such as /blockchain)
/// if the batch size > 1.
///
/// Will return None if the node had no new blocks and no inserts were made, otherwise it will returns the last height inserted.
async fn fetch_and_insert_blocks(
    client: &HttpClient,
    tx: &mut sqlx::Transaction<'_, Postgres>,
    chain_id: ChainId,
    batch_size: u32,
    from: Height,
) -> Result<Option<Height>, Report> {
    use itertools::Either;

    let min = from.value() as u32;
    let max = min + batch_size - 1_u32;
    debug!("fetching batch of headers from {} to {}", min, max);

    let headers = if batch_size > 1 {
        Either::Left(
            (|| {
                client
                    .blockchain(min, max)
                    .inspect_err(|e| debug!(?e, min, max, "error fetching blocks"))
            })
            .retry(&crate::expo_backoff())
            .await?
            .block_metas
            .into_iter()
            .map(|meta| meta.header),
        )
    } else {
        // We do need this arm, because client.blockchain will error if max > latest block (instead of just returning min..latest).
        match client.commit(min).await {
            Err(err) => {
                if is_height_exceeded_error(&err) {
                    return Ok(None);
                } else {
                    return Err(err.into());
                }
            }
            Ok(val) => Either::Right(std::iter::once(val.signed_header.header)),
        }
    };

    let submit_blocks = postgres::insert_batch_blocks(
        tx,
        stream::iter(headers.clone().into_iter().map(|header| {
            PgBlock {
                chain_id,
                hash: header.hash().to_string(),
                height: header.height.value() as i32,
                time: header.time.into(),
                data: serde_json::to_value(&header)
                    .unwrap()
                    .replace_escape_chars(),
            }
        })),
    );

    let block_results = stream::iter(headers.clone().into_iter().rev().map(Ok::<_, Report>))
        .and_then(|header| async {
            debug!("fetching block results for height {}", header.height);
            let block = (|| {
                client
                    .block_results(header.height)
                    .inspect_err(|e| debug!(?e, ?header.height, "error fetching block results"))
            })
            .retry(&crate::new_block_backoff())
            .await?;
            let txs = (|| {
                debug!(
                    "retrying fetching transactions for block for height {}",
                    header.height
                );
                fetch_transactions_for_block(client, header.height, None).inspect_err(
                    |e| debug!(?e, ?client, ?header.height, "error fetching transactions for block"),
                )
            })
            .retry(&crate::expo_backoff())
            .await?;
            Ok((header, block, txs))
        })
        .try_collect();

    // let (submit_blocks, block_results) = join!(submit_blocks, block_results);
    submit_blocks.await?;
    let block_results: Vec<_> = block_results.await?;

    // Initial capacity is a bit of an estimate, but shouldn't need to resize too often.
    let mut events = Vec::with_capacity(block_results.len() * 4 * 10);

    let transactions =
        block_results.into_iter().flat_map(|(header, block, txs)| {
            let block_height: i32 = block.height.value().try_into().unwrap();
            let block_hash = header.hash().to_string();
            let time: OffsetDateTime = header.time.into();
            let mut block_index = 0;
            let finalize_block_events = block.events(chain_id, block_hash.clone(), time);

            let txs =
                txs.into_iter()
                    .map(|tx| {
                        let transaction_hash = tx.hash.to_string();
                        let data = serde_json::to_value(&tx).unwrap().replace_escape_chars();
                        events.extend(tx.tx_result.events.into_iter().enumerate().map(
                            |(i, event)| {
                                let event = PgEvent {
                                    chain_id,
                                    block_hash: block_hash.clone(),
                                    block_height,
                                    time,
                                    data: serde_json::to_value(event)
                                        .unwrap()
                                        .replace_escape_chars(),
                                    transaction_hash: Some(transaction_hash.clone()),
                                    transaction_index: Some(i.try_into().unwrap()),
                                    block_index,
                                };
                                block_index += 1;
                                event
                            },
                        ));
                        PgTransaction {
                            chain_id,
                            block_hash: block_hash.clone(),
                            block_height,
                            time,
                            data,
                            hash: transaction_hash,
                            index: tx.index.try_into().unwrap(),
                        }
                    })
                    .collect::<Vec<_>>();
            events.extend(
                finalize_block_events
                    .into_iter()
                    .enumerate()
                    .map(|(i, e)| PgEvent {
                        block_index: i as i32 + block_index,
                        ..e
                    }),
            );
            txs
        });
    postgres::insert_batch_transactions(tx, stream::iter(transactions)).await?;
    postgres::insert_batch_events(tx, stream::iter(events)).await?;
    Ok(Some((from.value() as u32 + headers.len() as u32).into()))
}

async fn fetch_transactions_for_block(
    client: &HttpClient,
    height: Height,
    expected: impl Into<Option<usize>>,
) -> Result<Vec<tendermint_rpc::endpoint::tx::Response>, Report> {
    let expected = expected.into();

    debug!(
        ?client,
        ?height,
        ?expected,
        "fetching transactions for block"
    );
    let query = Query {
        event_type: None,
        conditions: vec![Condition {
            key: "tx.height".to_string(),
            operation: tendermint_rpc::query::Operation::Eq(
                tendermint_rpc::query::Operand::Unsigned(height.into()),
            ),
        }],
    };

    let mut txs = if let Some(expected) = expected {
        Vec::with_capacity(expected)
    } else {
        vec![]
    };

    for page in 1..u32::MAX {
        debug!("fetching page {page} for block {height}");
        let response = client
            .tx_search(query.clone(), false, page, 100, Order::Ascending)
            .await?;
        let len = response.txs.len();
        txs.extend(response.txs);

        // We always query for the maximum page size. If we get less items, we know pagination is done
        let current_count = (page - 1) * 100 + len as u32;
        let total_count = response.total_count;

        debug!(
            "fetched page {page} for block {height} (transaction {current_count} of {total_count})"
        );

        match current_count.cmp(&response.total_count) {
            std::cmp::Ordering::Less => debug!("fetching next page"),
            std::cmp::Ordering::Equal => {
                debug!("fetched all transactions");
                break;
            }
            std::cmp::Ordering::Greater => {
                debug!("fetched more transactions than expected");
                return Err(eyre!(
                    "fetched more transactions ({current_count}) than expected ({total_count})"
                ));
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

pub trait BlockExt {
    /// Returns the non-tx related events from a block formatted for insertion.
    fn events(self, chain_id: ChainId, block_hash: String, time: OffsetDateTime) -> Vec<PgEvent>;
}

impl BlockExt for BlockResponse {
    fn events(self, chain_id: ChainId, block_hash: String, time: OffsetDateTime) -> Vec<PgEvent> {
        let block_height: i32 = self.height.value().try_into().unwrap();
        let begin_block_events = self
            .begin_block_events
            .unwrap_or_default()
            .into_iter()
            .map(|e| PgEvent {
                chain_id,
                block_hash: block_hash.clone(),
                block_height,
                time,
                data: serde_json::to_value(e).unwrap().replace_escape_chars(),
                transaction_hash: None,
                transaction_index: None,
                block_index: 0,
            });
        let end_block_events = self.end_block_events.into_iter().map(|e| PgEvent {
            chain_id,
            block_hash: block_hash.clone(),
            block_height,
            time,
            data: serde_json::to_value(e).unwrap().replace_escape_chars(),
            transaction_hash: None,
            transaction_index: None,
            block_index: 0,
        });
        let finalize_block_events = self.finalize_block_events.into_iter().map(|e| PgEvent {
            chain_id,
            block_hash: block_hash.clone(),
            block_height,
            time,
            data: serde_json::to_value(e).unwrap().replace_escape_chars(),
            transaction_hash: None,
            transaction_index: None,
            block_index: 0,
        });
        let validator_updates = self.validator_updates.into_iter().map(|e| PgEvent {
            chain_id,
            block_hash: block_hash.clone(),
            block_height,
            time,
            data: serde_json::to_value(WithType::validator_update(e))
                .unwrap()
                .replace_escape_chars(),
            transaction_hash: None,
            transaction_index: None,
            block_index: 0,
        });
        let consensus_param_updates = self.consensus_param_updates.into_iter().map(|e| PgEvent {
            chain_id,
            block_hash: block_hash.clone(),
            block_height,
            time,
            data: serde_json::to_value(WithType::consensus_param_update(e))
                .unwrap()
                .replace_escape_chars(),
            transaction_hash: None,
            transaction_index: None,
            block_index: 0,
        });

        begin_block_events
            .chain(end_block_events)
            .chain(finalize_block_events)
            .chain(validator_updates)
            .chain(consensus_param_updates)
            .enumerate()
            .map(|(i, mut event)| {
                event.block_index = i as i32;
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

impl<I> WithType<I> {
    fn validator_update(inner: I) -> Self {
        WithType {
            kind: "validator_update",
            inner,
        }
    }

    fn consensus_param_update(inner: I) -> Self {
        WithType {
            kind: "consensus_param_update",
            inner,
        }
    }
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
