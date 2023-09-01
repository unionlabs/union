use reqwest::Client as Reqwest;
use tendermint::genesis::Genesis;
use tendermint_rpc::{Error, error::ErrorDetail, Client, HttpClient};
use tokio::time::{sleep, Duration};
use tracing::{debug, info};
use url::Url;
use tendermint_rpc::response_error::Code;
use crate::{
    hasura::*,
    tm::insert_block::{EventsArrRelInsertInput, EventsInsertInput},
};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub url: Url,
    pub chain_id: Option<String>,
}



impl Config {
    pub async fn index(
        &self,
        hasura_url: &Url,
        secret: &str,
    ) -> Result<(), color_eyre::eyre::Report> {
        let client = HttpClient::new(self.url.as_str()).unwrap();
        let db = Reqwest::new();
        let url = hasura_url.to_string();

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

        // We query for the last indexed block to not waste resources re-indexing.
        debug!("fetching latest stored block");
        let latest_stored =
            do_post::<GetLatestBlock>(secret, &url, &db, get_latest_block::Variables { chain_id })
                .await?;

        let data = latest_stored
            .data
            .expect("db should be prepared for indexing");

        let mut height: u32 = if data.blocks.is_empty() {
            0
        } else {
            TryInto::<u32>::try_into(data.blocks[0].height).unwrap()
        };
        debug!("latest stored block height is: {}", &height);

        let chain_db_id = data.chains[0].id;

        loop {
            height += 1;

            info!("indexing block {}", &height);
            // if we're caught up indexing to the latest height, this will error. In that case,
            // we retry until we obtain the next header.
            debug!("fetching block header for height: {}", &height);
            let header = match client.block(height).await {
                Err(err) => {
                    if is_height_exceeded_error(&err) {
                        debug!("caught up indexing, sleeping for 1 second");
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

            do_post::<InsertBlock>(secret, &url, &db, v).await?;
        }
    }
}

/// The RPC will return an internal error on queries for blocks exceeding the current height.
/// `is_height_exceeded_error` unwrangles the error and checks for this case. 
pub fn is_height_exceeded_error(err: &Error) -> bool {
    let detail = err.detail();
    if let ErrorDetail::Response(err) = detail {
        let inner = &err.source;
        let code = inner.code();
        let message = inner.data().unwrap_or_default();
        return matches!(code, Code::InternalError) && message.contains("must be less than or equal to")
    }
    false
}