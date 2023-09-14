use color_eyre::eyre::{bail, Report};
use futures::future::join_all;
use tendermint::{block::Height, consensus::Params, genesis::Genesis, validator::Update};
use tendermint_rpc::{
    dialect::v0_37::Event, endpoint::block_results::Response as BlockResponse, error::ErrorDetail,
    response_error::Code, Client, Error, HttpClient,
};
use tokio::time::{sleep, Duration};
use tracing::{debug, info};
use url::Url;

use crate::hasura::*;

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

        let (height, chain_db_id) = get_current_data(&db, chain_id).await?;
        let mut height: Height = (height + 1).into();

        // Fast sync protocol. We sync up to latest.height - batch-size + 1
        while let Some(up_to) = should_fast_sync_up_to(&client, Self::BATCH_SIZE, height).await? {
            debug!("starting fast sync protocol up to: {}", up_to);
            loop {
                height = batch_sync(&client, &db, chain_db_id, Self::BATCH_SIZE, height).await?;
                if height >= up_to {
                    debug!("re-evaluating fast sync protocol");
                    break; // go back to the should_fast_sync_up_to. If this returns None, we continue to slow sync.
                }
            }
        }

        let mut retry_count = 0;
        loop {
            debug!("starting regular sync protocol");
            // Regular sync protocol. This fetches blocks one-by-one.
            retry_count += 1;
            match sync_next(&client, &db, chain_db_id, height).await? {
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

    let data = dbg!(latest_stored)
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
    chain_db_id: i64,
    batch_size: u32,
    from: Height,
) -> Result<Height, Report> {
    if from.value() == 1 {
        sync_next(client, db, chain_db_id, from).await?;
    }

    let headers = client
        .blockchain(
            // Tendermint-rs is buggy, it
            from.value() as u32,
            (from.value() + batch_size as u64) as u32,
        )
        .await?;

    let objects: Result<Vec<_>, Report> =
        join_all(headers.block_metas.iter().rev().map(|header| async {
            let block = client.block_results(header.header.height).await?;
            let events: Vec<_> = block
                .events()
                .enumerate()
                .map(|event| event.into())
                .collect();
            Ok(insert_blocks_many::BlocksInsertInput {
                chain_id: Some(chain_db_id),
                chain: None,
                events: Some(insert_blocks_many::EventsArrRelInsertInput {
                    data: events,
                    on_conflict: None,
                }),
                hash: Some(header.header.hash().to_string()),
                height: Some(header.header.height.into()),
                id: None,
                created_at: None,
                updated_at: None,
                is_finalized: Some(true),
                extra_data: Some(serde_json::to_value(header.clone())?),
            })
        }))
        .await
        .into_iter()
        .collect();

    let variables = insert_blocks_many::Variables { objects: objects? };
    db.do_post::<InsertBlocksMany>(variables).await?;
    Ok((from.value() as u32 + headers.block_metas.len() as u32).into())
}

async fn sync_next<D: Datastore>(
    client: &HttpClient,
    db: &D,
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
    let block = client.block_results(height).await?;
    let height = block.height;
    let events: Vec<_> = block.events().enumerate().map(Into::into).collect();
    info!("found {} events for block {}", &events.len(), &height);

    debug!("storing events for block {}", &height);
    let v = insert_block::Variables {
        object: insert_block::BlocksInsertInput {
            chain: None,
            chain_id: Some(chain_db_id),
            created_at: None,
            events: Some(insert_block::EventsArrRelInsertInput {
                data: events,
                on_conflict: None,
            }),
            hash: Some(header.hash().to_string()),
            extra_data: Some(serde_json::to_value(header.clone())?),
            height: Some(header.height.into()),
            id: None,
            is_finalized: Some(true),
            updated_at: None,
        },
    };

    db.do_post::<InsertBlock>(v).await?;
    Ok(Some(height.increment()))
}

impl From<(usize, StateChange)> for insert_blocks_many::EventsInsertInput {
    fn from(value: (usize, StateChange)) -> Self {
        Self {
            id: None,
            index: Some(value.0 as i64),
            block: None,
            block_id: None,
            data: Some(serde_json::to_value(&value.1).unwrap()),
        }
    }
}

impl From<(usize, StateChange)> for insert_block::EventsInsertInput {
    fn from(value: (usize, StateChange)) -> Self {
        Self {
            id: None,
            index: Some(value.0 as i64),
            block: None,
            block_id: None,
            data: Some(serde_json::to_value(&value.1).unwrap()),
        }
    }
}

pub trait BlockExt {
    fn events(self) -> impl Iterator<Item = StateChange>;
}

impl BlockExt for BlockResponse {
    fn events(self) -> impl Iterator<Item = StateChange> {
        self.begin_block_events
            .unwrap_or_default()
            .into_iter()
            .chain(
                self.txs_results
                    .unwrap_or_default()
                    .into_iter()
                    .flat_map(|tx| tx.events),
            )
            .chain(self.end_block_events.unwrap_or_default())
            .chain(self.finalize_block_events)
            .map(StateChange::Event)
            .chain(
                self.validator_updates
                    .into_iter()
                    .map(StateChange::validator_update),
            )
            .chain(
                self.consensus_param_updates
                    .into_iter()
                    .map(StateChange::consensus_param_update),
            )
    }
}

#[derive(serde::Serialize)]
#[serde(untagged)]
pub enum StateChange {
    Event(Event),
    ValidatorUpdate(WithType<Update>),
    ConsensusUpdate(WithType<Params>),
}

impl StateChange {
    fn validator_update(inner: Update) -> Self {
        StateChange::ValidatorUpdate(WithType::validator_update(inner))
    }

    fn consensus_param_update(inner: Params) -> Self {
        StateChange::ConsensusUpdate(WithType::consensus_param_update(inner))
    }
}

#[derive(serde::Serialize)]
pub struct WithType<I> {
    r#type: &'static str,
    #[serde(flatten)]
    inner: I,
}

impl<I> WithType<I> {
    fn validator_update(inner: I) -> Self {
        WithType {
            r#type: "validator_update",
            inner,
        }
    }

    fn consensus_param_update(inner: I) -> Self {
        WithType {
            r#type: "consensus_param_update",
            inner,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;
    use tendermint::{abci::EventAttribute, vote::Power};

    use super::*;

    #[test]
    fn state_change_serializes_correctly() {
        use serde_json::{json, to_value};

        fn check<T: Serialize>(t: T, json: serde_json::Value) {
            assert_eq!(to_value(t).unwrap(), json)
        }

        check(
            StateChange::Event(Event {
                kind: "foo".to_string(),
                attributes: vec![EventAttribute {
                    index: false,
                    key: "bar".to_string(),
                    value: "bax".to_string(),
                }],
            }),
            json!({
                "type": "foo",
                "attributes": [
                    {
                        "key": "bar",
                        "index": false,
                        "value": "bax",
                    }
                ]
            }),
        );
        check(
            StateChange::validator_update(Update {
                pub_key: tendermint::PublicKey::Bn254(Default::default()),
                power: Power::from(1_u8),
            }),
            json!({
                "type": "validator_update",
                "power": "1",
                "pub_key": {
                    "type": "tendermint/PubKeyBn254",
                    "value": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
                }
            }),
        );

        check(
            StateChange::validator_update(Update {
                pub_key: tendermint::PublicKey::Bn254(Default::default()),
                power: Power::from(1_u8),
            }),
            json!({
                "type": "validator_update",
                "power": "1",
                "pub_key": {
                    "type": "tendermint/PubKeyBn254",
                    "value": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
                }
            }),
        );
    }
}
