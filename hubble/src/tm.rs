use color_eyre::eyre::{bail, Report};
use futures::{
    future::{ready, TryFutureExt},
    stream,
    stream::TryStreamExt,
    try_join,
};
use sqlx::{Acquire, Postgres};
use tendermint::{block::Height, genesis::Genesis};
use tendermint_rpc::{
    endpoint::block_results::Response as BlockResponse,
    error::ErrorDetail,
    query::{Condition, Query},
    response_error::Code,
    Client, Error, HttpClient, Order,
};
use time::OffsetDateTime;
use tokio::time::{sleep, Duration};
use tracing::{debug, info};
use url::Url;

use crate::postgres::{self, ChainId};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub url: Url,
    pub start: Option<u64>,
    pub until: Option<u64>,
}

impl Config {
    /// The batch size for the fast sync protocol. This corresponds to the maximum number of headers returned over a node's RPC.
    pub const BATCH_SIZE: u32 = 20;

    pub async fn index<DB>(self, pool: DB) -> Result<(), Report>
    where
        for<'a> &'a DB:
            sqlx::Acquire<'a, Database = Postgres> + sqlx::Executor<'a, Database = Postgres>,
    {
        let client = HttpClient::new(self.url.as_str()).unwrap();

        // If there is no chain_id override, we query it from the node. This
        // is the expected default.
        info!("fetching chain-id from node");
        let genesis: Genesis<serde_json::Value> = client.genesis().await?;
        let chain_id = genesis.chain_id.to_string();
        info!("chain-id is {}", &chain_id);

        let chain_id = postgres::fetch_or_insert_chain_id(&pool, chain_id)
            .await?
            .get_inner_logged();
        let mut height = Height::from(sqlx::query!("SELECT height FROM \"v0\".blocks WHERE chain_id = $1 ORDER BY time DESC NULLS LAST LIMIT 1", chain_id.db).fetch_optional(&pool).await?.map(|block| block.height + 1).unwrap_or_default() as u32);
        // Fast sync protocol. We sync up to latest.height - batch-size + 1
        while let Some(up_to) = should_fast_sync_up_to(&client, Self::BATCH_SIZE, height).await? {
            info!("starting fast sync protocol up to: {}", up_to);
            loop {
                let batch_end =
                    std::cmp::min(up_to.value(), height.value() + Self::BATCH_SIZE as u64);
                if batch_end - height.value() != 20 {
                    info!("re-evaluating fast sync protocol");
                    break; // go back to the should_fast_sync_up_to. If this returns None, we continue to slow sync.
                }

                info!("fast syncing for batch: {}..{}", height, batch_end);
                let mut tx = pool.begin().await?;
                height = batch_sync(&client, &mut tx, chain_id, Self::BATCH_SIZE, height).await?;
                tx.commit().await?;
            }
        }

        info!("continuing regular sync protocol");

        let mut retry_count = 0;
        loop {
            debug!("starting regular sync protocol");
            // Regular sync protocol. This fetches blocks one-by-one.
            retry_count += 1;
            let mut tx = pool.begin().await?;
            match sync_next(&client, &mut tx, chain_id, height).await? {
                Some(h) => {
                    height = h;
                    retry_count = 0;
                    tx.commit().await?;
                }
                None => {
                    if retry_count > 30 {
                        bail!("node has stopped providing new blocks")
                    }
                    retry_count += 1;
                    debug!("caught up indexing, sleeping for 1 second");
                    sleep(Duration::from_millis(1000)).await;
                    tx.rollback().await?;
                    continue;
                }
            }
        }
    }
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
    let latest = client.latest_block().await?.block.header.height;
    if latest.value() - current.value() > batch_size.into() {
        Ok(Some(latest))
    } else {
        Ok(None)
    }
}

/// Uses batch processing to fast sync up to the provided height.
async fn batch_sync(
    client: &HttpClient,
    tx: &mut sqlx::Transaction<'_, Postgres>,
    chain_id: ChainId,
    batch_size: u32,
    from: Height,
) -> Result<Height, Report> {
    let min = from.value() as u32;
    let max = min + batch_size - 1_u32;
    debug!("fetching batch of headers from {} to {}", min, max);

    let headers = client.blockchain(min, max).await?;

    let submit_blocks = postgres::insert_batch_blocks(
        tx,
        stream::iter(headers.block_metas.clone().into_iter().map(|meta| {
            postgres::Block {
                chain_id,
                hash: meta.header.hash().to_string(),
                height: meta.header.height.value() as i32,
                time: meta.header.time.into(),
                data: serde_json::to_value(&meta.header)
                    .unwrap()
                    .replace_escape_chars(),
            }
        })),
    );

    let block_results = stream::iter(
        headers
            .block_metas
            .clone()
            .into_iter()
            .rev()
            .map(Ok::<_, Report>),
    )
    .and_then(|meta| async {
        debug!("fetching block results for height {}", meta.header.height);
        let block = client.block_results(meta.header.height).await?;
        let txs = fetch_transactions_for_block(client, meta.header.height, None).await?;
        Ok((meta, block, txs))
    })
    .try_collect();

    // let (submit_blocks, block_results) = join!(submit_blocks, block_results);
    submit_blocks.await?;
    let block_results: Vec<_> = block_results.await?;

    // Initial capacity is a bit of an estimate, but shouldn't need to resize too often.
    let mut events = Vec::with_capacity(block_results.len() * 4 * 10);

    let transactions =
        block_results.into_iter().flat_map(|(meta, block, txs)| {
            let block_height: i32 = block.height.value().try_into().unwrap();
            let block_hash = meta.header.hash().to_string();
            let time: OffsetDateTime = meta.header.time.into();
            let mut block_index = 0;
            let finalize_block_events = block.events(chain_id, block_hash.clone(), time);

            let txs =
                txs.into_iter()
                    .map(|tx| {
                        let transaction_hash = tx.hash.to_string();
                        let data = serde_json::to_value(&tx).unwrap().replace_escape_chars();
                        events.extend(tx.tx_result.events.into_iter().enumerate().map(
                            |(i, event)| {
                                let event = postgres::Event {
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
                        postgres::Transaction {
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
            events.extend(finalize_block_events.into_iter().enumerate().map(|(i, e)| {
                postgres::Event {
                    block_index: i as i32 + block_index,
                    ..e
                }
            }));
            txs
        });
    postgres::insert_batch_transactions(tx, stream::iter(transactions)).await?;
    postgres::insert_batch_events(tx, stream::iter(events)).await?;
    Ok((from.value() as u32 + headers.block_metas.len() as u32).into())
}

async fn sync_next(
    client: &HttpClient,
    tx: &mut sqlx::Transaction<'_, Postgres>,
    chain_id: ChainId,
    block_height: Height,
) -> Result<Option<Height>, Report> {
    info!("indexing block {}", &block_height);
    // If we're caught up indexing to the latest height, this will error. In that case,
    // we retry until we obtain the next header.
    debug!("fetching block header for height: {}", &block_height);
    let header = match client.block(block_height).await {
        Err(err) => {
            if is_height_exceeded_error(&err) {
                return Ok(None);
            } else {
                return Err(err.into());
            }
        }
        Ok(val) => val.block.header,
    };
    debug!("fetching block results for height: {}", &block_height);

    let (block, finalize_events) = match client.block_results(block_height).await {
        Err(err) => {
            if is_height_exceeded_error(&err) {
                return Ok(None);
            } else {
                return Err(err.into());
            }
        }
        Ok(block) => (
            postgres::Block {
                chain_id,
                hash: header.hash().to_string(),
                height: block_height.value().try_into().unwrap(),
                time: header.time.into(),
                data: serde_json::to_value(&header)
                    .unwrap()
                    .replace_escape_chars(),
            },
            block.events(chain_id, header.hash().to_string(), header.time.into()),
        ),
    };

    let txs = fetch_transactions_for_block(client, block_height, None);
    let (_, txs) = try_join!(
        postgres::insert_batch_blocks(tx, stream::once(ready(block))).map_err(Report::from),
        txs
    )?;

    let mut events = vec![];
    let mut block_index = 0;

    let txs: Vec<_> = txs
        .iter()
        .map(|tx| {
            tx.tx_result
                .events
                .iter()
                .enumerate()
                .for_each(|(i, event)| {
                    events.push(postgres::Event {
                        chain_id,
                        block_hash: header.hash().to_string(),
                        block_height: block_height.value().try_into().unwrap(),
                        time: header.time.into(),
                        data: serde_json::to_value(event).unwrap().replace_escape_chars(),
                        transaction_hash: Some(tx.hash.to_string()),
                        transaction_index: Some(i as i32),
                        block_index,
                    });
                    block_index += 1;
                });
            postgres::Transaction {
                chain_id,
                block_hash: header.hash().to_string(),
                block_height: block_height.value().try_into().unwrap(),
                time: header.time.into(),
                data: serde_json::to_value(tx).unwrap().replace_escape_chars(),
                hash: tx.hash.to_string(),
                index: tx.index as i32,
            }
        })
        .collect();

    let events_len = events.len();

    let events = events
        .into_iter()
        .chain(finalize_events)
        .enumerate()
        .map(|(i, e)| postgres::Event {
            block_index: i as i32,
            ..e
        });

    postgres::insert_batch_transactions(tx, stream::iter(txs)).await?;
    postgres::insert_batch_events(tx, stream::iter(events)).await?;

    info!("found {} events for block {}", events_len, &block_height);
    debug!("storing events for block {}", &block_height);
    Ok(Some(block_height.increment()))
}

async fn fetch_transactions_for_block(
    client: &HttpClient,
    height: Height,
    expected: impl Into<Option<usize>>,
) -> Result<Vec<tendermint_rpc::endpoint::tx::Response>, Report> {
    let query = Query {
        event_type: None,
        conditions: vec![Condition {
            key: "tx.height".to_string(),
            operation: tendermint_rpc::query::Operation::Eq(
                tendermint_rpc::query::Operand::Unsigned(height.into()),
            ),
        }],
    };
    let expected = expected.into();

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
        if len < 100 {
            break;
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
    fn events(
        self,
        chain_id: ChainId,
        block_hash: String,
        time: OffsetDateTime,
    ) -> Vec<postgres::Event>;
}

impl BlockExt for BlockResponse {
    fn events(
        self,
        chain_id: ChainId,
        block_hash: String,
        time: OffsetDateTime,
    ) -> Vec<postgres::Event> {
        let block_height: i32 = self.height.value().try_into().unwrap();
        let begin_block_events = self
            .begin_block_events
            .unwrap_or_default()
            .into_iter()
            .map(|e| postgres::Event {
                chain_id,
                block_hash: block_hash.clone(),
                block_height,
                time,
                data: serde_json::to_value(e).unwrap().replace_escape_chars(),
                transaction_hash: None,
                transaction_index: None,
                block_index: 0,
            });
        let end_block_events = self.end_block_events.into_iter().map(|e| postgres::Event {
            chain_id,
            block_hash: block_hash.clone(),
            block_height,
            time,
            data: serde_json::to_value(e).unwrap().replace_escape_chars(),
            transaction_hash: None,
            transaction_index: None,
            block_index: 0,
        });
        let finalize_block_events =
            self.finalize_block_events
                .into_iter()
                .map(|e| postgres::Event {
                    chain_id,
                    block_hash: block_hash.clone(),
                    block_height,
                    time,
                    data: serde_json::to_value(e).unwrap().replace_escape_chars(),
                    transaction_hash: None,
                    transaction_index: None,
                    block_index: 0,
                });
        let validator_updates = self.validator_updates.into_iter().map(|e| postgres::Event {
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
        let consensus_param_updates =
            self.consensus_param_updates
                .into_iter()
                .map(|e| postgres::Event {
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
