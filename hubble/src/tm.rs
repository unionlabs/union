use color_eyre::eyre::{bail, Report};
use futures::future::join_all;
use hubble::hasura::*;
use tendermint::{block::Height, genesis::Genesis, Time};
use tendermint_rpc::{
    endpoint::block_results::Response as BlockResponse,
    error::ErrorDetail,
    query::{Condition, Query},
    response_error::Code,
    Client, Error, HttpClient, Order,
};
use tokio::time::{sleep, Duration};
use tracing::{debug, info};
use url::Url;

use crate::{metrics, tm::insert_blocks_many::V0EventsInsertInput};

pub const STAGE_BEGIN_BLOCK: i32 = 1;
pub const STAGE_END_BLOCK: i32 = 2;
pub const STAGE_FINALIZE_BLOCK: i32 = 3;
pub const STAGE_VALIDATOR_UPDATES: i32 = 4;
pub const STAGE_CONSENSUS_PARAM_UPDATES: i32 = 5;
pub const STAGE_TX: i32 = 6;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub url: Url,
    pub chain_id: Option<String>,
}

impl Config {
    /// The batch size for the fast sync protocol. This corresponds to the maximum number of headers returned over a node's RPC.
    pub const BATCH_SIZE: u32 = 20;

    pub async fn index<D: Datastore>(&self, db: D) -> Result<(), Report> {
        let client = HttpClient::new(self.url.as_str()).unwrap();

        // If there is no chain_id override, we query it from the node. This
        // is the expected default.
        let chain_id = match &self.chain_id {
            Some(chain_id) => chain_id.to_owned(),
            None => {
                info!("fetching chain-id from node");
                let genesis: Genesis<serde_json::Value> = client.genesis().await?;
                let chain_id = genesis.chain_id.to_string();
                info!("chain-id is {}", &chain_id);
                chain_id
            }
        };

        let (height, chain_db_id) = get_current_data(&db, &chain_id).await?;
        let mut height: Height = (height + 1).into();

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
                height = batch_sync(
                    &client,
                    &db,
                    &chain_id,
                    chain_db_id,
                    Self::BATCH_SIZE,
                    height,
                )
                .await?;
            }
        }

        info!("continuing regular sync protocol");

        let mut retry_count = 0;
        loop {
            debug!("starting regular sync protocol");
            // Regular sync protocol. This fetches blocks one-by-one.
            retry_count += 1;
            match sync_next(&client, &db, &chain_id, chain_db_id, height).await? {
                Some(h) => {
                    height = h;
                    retry_count = 0
                }
                None => {
                    if retry_count > 30 {
                        bail!("node has stopped providing new blocks")
                    }
                    retry_count += 1;
                    debug!("caught up indexing, sleeping for 1 second");
                    sleep(Duration::from_millis(1000)).await;
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

/// Obtains the current height and chain_db_id for the chain_id. If the chain_id is not stored yet, an entry is created.
async fn get_current_data<D: Datastore>(
    db: &D,
    chain_id: &str,
) -> Result<(u32, i64), color_eyre::eyre::Report> {
    // We query for the last indexed block to not waste resources re-indexing.
    debug!("fetching latest stored block");
    let latest_stored = db
        .do_post::<GetLatestBlock>(get_latest_block::Variables {
            chain_id: chain_id.to_string(),
        })
        .await?;

    let data = latest_stored
        .data
        .expect("db should be prepared for indexing");

    let height: u32 = if data.v0_blocks.is_empty() {
        0
    } else {
        data.v0_blocks[0]
            .height
            .try_into()
            .expect("invalid bigint stored in DB")
    };
    debug!("latest stored block height is: {}", &height);

    let chain_db_id = if let Some(chains) = data.v0_chains.first() {
        chains.id
    } else {
        let created = db
            .do_post::<InsertChain>(insert_chain::Variables {
                chain_id: chain_id.to_string(),
            })
            .await?;
        created.data.unwrap().insert_v0_chains_one.unwrap().id
    };

    Ok((height, chain_db_id))
}

/// Queries the node and current indexed height and determines if fast sync should be applied.
///
/// # Returns
/// The block up to which to fast sync.
///
/// # Errors
/// On IO errors when communicating with the datastore or the node.
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
async fn batch_sync<D: Datastore>(
    client: &HttpClient,
    db: &D,
    chain_id: &str,
    chain_db_id: i64,
    batch_size: u32,
    from: Height,
) -> Result<Height, Report> {
    let min = from.value() as u32;
    let max = min + batch_size - 1_u32;
    debug!("fetching batch of headers from {} to {}", min, max);

    let headers = client.blockchain(min, max).await?;

    let objects: Vec<_> = join_all(headers.block_metas.iter().rev().map(|header| async {
        debug!("fetching block results for height {}", header.header.height);
        let block = client.block_results(header.header.height).await?;
        let len = block
            .txs_results
            .as_ref()
            .map(|tx_results| tx_results.len());

        let txs = fetch_transactions_for_block(client, header.header.height, len).await?;
        let events: Vec<_> = block.events(&header.header.time).collect();
        // debug!(
        //     "found {} events for block {}",
        //     events.len() + txs.iter().len(),
        //     header.header.height
        // );
        Ok(insert_blocks_many::V0BlocksInsertInput {
            chain_id: Some(chain_db_id),
            chain: None,
            events: Some(insert_blocks_many::V0EventsArrRelInsertInput {
                data: events,
                on_conflict: None,
            }),
            hash: Some(header.header.hash().to_string()),
            height: Some(header.header.height.value().into()),
            id: None,
            created_at: None,
            updated_at: None,
            is_finalized: Some(true),
            data: Some(serde_json::to_value(header.clone())?),
            time: Some(header.header.time.to_rfc3339()),
            transactions: Some(transactions_into_many_blocks_input(
                txs,
                header.header.time.to_rfc3339(),
            )),
        })
    }))
    .await
    .into_iter()
    .collect::<Result<Vec<_>, Report>>()?;

    objects.iter().for_each(|block| {
        let num_events = block
            .events
            .as_ref()
            .map(|input| input.data.len())
            .unwrap_or_default();
        metrics::EVENT_COLLECTOR
            .with_label_values(&[chain_id, block.hash.as_ref().unwrap()])
            .inc_by(num_events as u64);
    });
    metrics::BLOCK_COLLECTOR
        .with_label_values(&[chain_id])
        .inc_by(objects.len() as u64);
    let variables = insert_blocks_many::Variables { objects };
    debug!("inserting batch of blocks");
    db.do_post::<InsertBlocksMany>(variables).await?;
    metrics::POST_COLLECTOR.with_label_values(&[chain_id]).inc();
    Ok((from.value() as u32 + headers.block_metas.len() as u32).into())
}

async fn sync_next<D: Datastore>(
    client: &HttpClient,
    db: &D,
    chain_id: &str,
    chain_db_id: i64,
    height: Height,
) -> Result<Option<Height>, Report> {
    info!("indexing block {}", &height);
    // if we're caught up indexing to the latest height, this will error. In that case,
    // we retry until we obtain the next header.
    debug!("fetching block header for height: {}", &height);
    let header = match client.block(height).await {
        Err(err) => {
            if is_height_exceeded_error(&err) {
                return Ok(None);
            } else {
                return Err(err.into());
            }
        }
        Ok(val) => val.block.header,
    };
    debug!("fetching block results for height: {}", &height);
    let block = match client.block_results(height).await {
        Err(err) => {
            if is_height_exceeded_error(&err) {
                return Ok(None);
            } else {
                return Err(err.into());
            }
        }
        Ok(block) => block,
    };

    let height = block.height;
    let len = block.txs_results.as_ref().map(|e| e.len());
    let events: Vec<_> = block.events(&header.time).collect();
    let txs = fetch_transactions_for_block(client, height, len).await?;
    info!("found {} events for block {}", &events.len(), &height);

    debug!("storing events for block {}", &height);

    metrics::EVENT_COLLECTOR
        .with_label_values(&[chain_id, &header.hash().to_string()])
        .inc_by(events.len() as u64);

    metrics::BLOCK_COLLECTOR
        .with_label_values(&[chain_id])
        .inc();
    let v = insert_blocks_many::Variables {
        objects: vec![insert_blocks_many::V0BlocksInsertInput {
            chain: None,
            chain_id: Some(chain_db_id),
            created_at: None,
            events: Some(insert_blocks_many::V0EventsArrRelInsertInput {
                data: events,
                on_conflict: None,
            }),
            hash: Some(header.hash().to_string()),
            data: Some(serde_json::to_value(header.clone())?),
            height: Some(header.height.value().into()),
            id: None,
            is_finalized: Some(true),
            updated_at: None,
            time: Some(header.time.to_rfc3339()),
            transactions: Some(transactions_into_many_blocks_input(
                txs,
                header.time.to_rfc3339(),
            )),
        }],
    };

    db.do_post::<InsertBlocksMany>(v).await?;
    metrics::POST_COLLECTOR.with_label_values(&[chain_id]).inc();
    Ok(Some(height.increment()))
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

fn transactions_into_many_blocks_input(
    txs: Vec<tendermint_rpc::endpoint::tx::Response>,
    time: String,
) -> crate::tm::insert_blocks_many::V0TransactionsArrRelInsertInput {
    let mut index = 0;
    crate::tm::insert_blocks_many::V0TransactionsArrRelInsertInput {
        data: txs
            .into_iter()
            .map(
                |tx| hubble::hasura::insert_blocks_many::V0TransactionsInsertInput {
                    block: None,
                    block_id: None,
                    created_at: None,
                    updated_at: None,
                    data: Some(serde_json::to_value(&tx).unwrap()),
                    hash: Some(tx.hash.to_string()),
                    id: None,
                    index: Some(tx.index.into()),
                    events: Some(insert_blocks_many::V0EventsArrRelInsertInput {
                        data: tx
                            .tx_result
                            .events
                            .into_iter()
                            .map(|r| {
                                let input = insert_blocks_many::V0EventsInsertInput {
                                    block: None,
                                    block_id: None,
                                    created_at: None,
                                    data: Some(serde_json::to_value(r).unwrap()),
                                    index: Some(index),
                                    time: Some(time.clone()),
                                    updated_at: None,
                                    transaction: None,
                                    transaction_id: None,
                                    stage: Some(STAGE_TX),
                                };
                                index += 1;
                                input
                            })
                            .collect(),
                        on_conflict: None,
                    }),
                },
            )
            .collect(),
        on_conflict: None,
    }
}

pub trait BlockExt {
    /// Returns the non-tx related events from a block formatted for insertion.
    fn events(self, timestamp: &Time) -> impl Iterator<Item = V0EventsInsertInput> + '_;
}

impl BlockExt for BlockResponse {
    fn events(self, timestamp: &Time) -> impl Iterator<Item = V0EventsInsertInput> + '_ {
        let begin_block_events = self
            .begin_block_events
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(i, e)| V0EventsInsertInput {
                block: None,
                block_id: None,
                created_at: None,
                updated_at: None,
                index: Some(i as i64),
                data: Some(serde_json::to_value(e).unwrap()),
                time: Some(timestamp.to_rfc3339()),
                stage: Some(STAGE_BEGIN_BLOCK),
                transaction_id: None,
                transaction: None,
            });
        let end_block_events =
            self.end_block_events
                .into_iter()
                .enumerate()
                .map(|(i, e)| V0EventsInsertInput {
                    block: None,
                    block_id: None,
                    created_at: None,
                    updated_at: None,
                    index: Some(i as i64),
                    data: Some(serde_json::to_value(e).unwrap()),
                    time: Some(timestamp.to_rfc3339()),
                    stage: Some(STAGE_END_BLOCK),
                    transaction_id: None,
                    transaction: None,
                });
        let finalize_block_events =
            self.finalize_block_events
                .into_iter()
                .enumerate()
                .map(|(i, e)| V0EventsInsertInput {
                    block: None,
                    block_id: None,
                    created_at: None,
                    updated_at: None,
                    index: Some(i as i64),
                    data: Some(serde_json::to_value(e).unwrap()),
                    time: Some(timestamp.to_rfc3339()),
                    stage: Some(STAGE_FINALIZE_BLOCK),
                    transaction_id: None,
                    transaction: None,
                });
        let validator_updates = self
            .validator_updates
            .into_iter()
            .enumerate()
            .map(|(i, e)| V0EventsInsertInput {
                block: None,
                block_id: None,
                created_at: None,
                updated_at: None,
                index: Some(i as i64),
                data: Some(serde_json::to_value(WithType::validator_update(e)).unwrap()),
                time: Some(timestamp.to_rfc3339()),
                stage: Some(STAGE_VALIDATOR_UPDATES),
                transaction_id: None,
                transaction: None,
            });
        let consensus_param_updates =
            self.consensus_param_updates
                .into_iter()
                .enumerate()
                .map(|(i, e)| V0EventsInsertInput {
                    block: None,
                    block_id: None,
                    created_at: None,
                    updated_at: None,
                    index: Some(i as i64),
                    data: Some(serde_json::to_value(WithType::consensus_param_update(e)).unwrap()),
                    time: Some(timestamp.to_rfc3339()),
                    stage: Some(STAGE_CONSENSUS_PARAM_UPDATES),
                    transaction_id: None,
                    transaction: None,
                });

        begin_block_events
            .chain(end_block_events)
            .chain(finalize_block_events)
            .chain(validator_updates)
            .chain(consensus_param_updates)
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
