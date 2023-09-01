use std::ops::Range;

use tendermint::genesis::Genesis;
use tendermint_rpc::{error::ErrorDetail, response_error::Code, Client, Error, HttpClient};
use tokio::time::{sleep, Duration};
use tracing::{debug, info};
use url::Url;

use crate::{
    hasura::*,
    tm::insert_block::{EventsArrRelInsertInput, EventsInsertInput},
};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub url: Url,
    pub chain_id: Option<String>,
    pub range: Option<Range<u32>>,
}

impl Config {
    pub async fn index<D: Datastore>(&self, db: D) -> Result<(), color_eyre::eyre::Report> {
        let client = HttpClient::new(self.url.as_str()).unwrap();
        let range = self.range.as_ref().unwrap_or(&(0..u32::MAX));

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

        let (mut height, chain_db_id) = get_current_data(&db, chain_id).await?;
        height += 1;
        loop {
            if !range.contains(&height) {
                return Ok(());
            }

            info!("indexing block {}", &height);
            // if we're caught up indexing to the latest height, this will error. In that case,
            // we retry until we obtain the next header.
            debug!("fetching block header for height: {}", &height);
            let header = match client.block(height).await {
                Err(err) => {
                    if is_height_exceeded_error(&err) {
                        debug!("caught up indexing, sleeping for 1 second: {:?}", err);
                        sleep(Duration::from_millis(1000)).await;
                        continue;
                    } else {
                        return Err(err.into());
                    }
                }
                Ok(val) => val.block.header,
            };
            debug!("fetching block results for height: {}", &height);
            let block = client.block_results(height).await?;
            let events = {
                let mut events = vec![];
                let mut i = 0;

                if let Some(begin_block_events) = block.begin_block_events {
                    for result in begin_block_events {
                        let event = EventsInsertInput {
                            data: Some(serde_json::to_value(&result).unwrap()),
                            index: Some(i),
                            block: None,
                            block_id: None,
                            id: None,
                        };
                        events.push(event);
                        i += 1;
                    }
                }

                if let Some(txs_results) = block.txs_results {
                    for result in txs_results {
                        let event = EventsInsertInput {
                            data: Some(serde_json::to_value(&result).unwrap()),
                            index: Some(i),
                            block: None,
                            block_id: None,
                            id: None,
                        };
                        events.push(event);
                        i += 1;
                    }
                }

                if let Some(end_block_events) = block.end_block_events {
                    for result in end_block_events {
                        let event = EventsInsertInput {
                            data: Some(serde_json::to_value(&result).unwrap()),
                            index: Some(i),
                            block: None,
                            block_id: None,
                            id: None,
                        };
                        events.push(event);
                        i += 1;
                    }
                }

                info!("found {} events for block {}", &i, &height);
                events
            };

            debug!("storing events for block {}", &height);
            let v = insert_block::Variables {
                chain_id: chain_db_id,
                hash: header.hash().to_string(),
                height: block.height.into(),
                events: Some(EventsArrRelInsertInput {
                    data: events,
                    on_conflict: None,
                }),
                finalized: true,
            };

            db.do_post::<InsertBlock>(v).await?;
            height += 1;
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
            && message.contains("must be less than or equal to");
    }
    false
}

/// Obtains the current height and chain_db_id for the chain_id. If the chain_id is not stored yet, an entry is created.
async fn get_current_data<D: Datastore>(
    db: &D,
    chain_id: String,
) -> Result<(u32, i64), color_eyre::eyre::Report> {
    // We query for the last indexed block to not waste resources re-indexing.
    debug!("fetching latest stored block");
    let latest_stored = db
        .do_post::<GetLatestBlock>(get_latest_block::Variables {
            chain_id: chain_id.clone(),
        })
        .await?;

    let data = latest_stored
        .data
        .expect("db should be prepared for indexing");

    let height: u32 = if data.blocks.is_empty() {
        0
    } else {
        TryInto::<u32>::try_into(data.blocks[0].height).unwrap()
    };
    debug!("latest stored block height is: {}", &height);

    let chain_db_id = if let Some(chains) = data.chains.get(0) {
        chains.id
    } else {
        let created = db
            .do_post::<InsertChain>(insert_chain::Variables { chain_id })
            .await?;

        created.data.unwrap().insert_chains_one.unwrap().id
    };

    Ok((height, chain_db_id))
}
