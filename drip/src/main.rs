use std::{
    collections::HashMap, ffi::OsString, fmt, fs::read_to_string, rc::Rc, sync::Arc, time::Duration,
};

use async_graphql::{http::GraphiQLSource, *};
use async_graphql_axum::GraphQL;
use async_sqlite::{
    rusqlite::{self, params, OptionalExtension},
    JournalMode, Pool, PoolBuilder,
};
use axum::{
    response::{self, IntoResponse},
    routing::get,
    Router,
};
use chrono::{NaiveDateTime, Utc};
use clap::Parser;
use cosmos_client::{
    gas::{FeemarketGasFiller, GasFillerT, StaticGasFiller},
    rpc::{Rpc, RpcT},
    wallet::{LocalSigner, WalletT},
    TxClient,
};
use prost::{Message, Name};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tracing::{debug, error, info, instrument, warn, Instrument};
use tracing_subscriber::EnvFilter;
use unionlabs::{
    primitives::{encoding::HexUnprefixed, H256},
    ErrorReporter,
};

mod turnstile;

const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

//
// 1. Unified gas filler configuration types
//
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GasFillerMode {
    Static,
    Feemarket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasFillerConfig {
    pub mode: GasFillerMode,
    pub max_gas: u64,
    #[serde(default)]
    pub min_gas: u64,
    pub gas_multiplier: f64,
    // Optional for static mode
    #[serde(default)]
    pub gas_price: Option<f64>,
    #[serde(default)]
    pub gas_denom: Option<String>,

    // Optional for feemarket mode
    #[serde(default)]
    pub fee_denom: Option<String>,
}

//
// 2. Gas filler abstraction over static and fee–market fillers
//
// We do not derive Clone here since FeemarketGasFiller is not Clone.
#[derive(Debug)]
pub enum AnyGasFiller {
    Static(StaticGasFiller),
    Feemarket(FeemarketGasFiller),
}

impl GasFillerT for AnyGasFiller {
    async fn max_gas(&self) -> u64 {
        match self {
            Self::Static(f) => f.max_gas().await,
            Self::Feemarket(f) => f.max_gas().await,
        }
    }

    async fn mk_fee(&self, gas: u64) -> unionlabs::cosmos::tx::fee::Fee {
        match self {
            Self::Static(f) => f.mk_fee(gas).await,
            Self::Feemarket(f) => f.mk_fee(gas).await,
        }
    }
}

impl AnyGasFiller {
    pub async fn from_config(config: GasFillerConfig, rpc_url: String) -> Result<Self> {
        match config.mode {
            GasFillerMode::Static => {
                let gas_price = config
                    .gas_price
                    .ok_or_else(|| format!("missing gas_price for static gas config"))?;
                let gas_denom = config
                    .gas_denom
                    .clone()
                    .ok_or_else(|| format!("missing gas_denom for static gas config"))?;
                Ok(AnyGasFiller::Static(StaticGasFiller {
                    gas_price,
                    gas_denom,
                    gas_multiplier: config.gas_multiplier,
                    max_gas: config.max_gas,
                    min_gas: config.min_gas,
                }))
            }
            GasFillerMode::Feemarket => {
                let filler = FeemarketGasFiller::new(
                    rpc_url,
                    config.max_gas,
                    Some(config.gas_multiplier),
                    config.fee_denom,
                )
                .await?;
                Ok(AnyGasFiller::Feemarket(filler))
            }
        }
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = AppArgs::parse();
    let batch_size = args.batch_size;
    let max_paginated_responses = args.max_paginated_responses;

    let config = serde_json::from_str::<Config>(
        &read_to_string(&args.config_file_path).expect("can't read config file"),
    )
    .expect("invalid config file");

    match config.log_format {
        LogFormat::Text => {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .init();
        }
        LogFormat::Json => {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .json()
                .init();
        }
    }

    let secret = config.secret.clone().map(CaptchaSecret);

    info!("creating database");
    let pool = PoolBuilder::new()
        .path("db.sqlite3")
        .journal_mode(JournalMode::Wal)
        .open()
        .await
        .expect("opening db");

    pool.conn(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS requests (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                chain_id TEXT NOT NULL,
                denom TEXT NOT NULL,
                address TEXT NOT NULL,
                time TEXT,
                tx_hash TEXT
            )",
            (), // empty list of parameters.
        )?;
        Ok(())
    })
    .await
    .unwrap();

    let schema = Schema::build(
        Query,
        Mutation {
            ratelimit_seconds: config.ratelimit_seconds,
            chains: config.clone().chains,
        },
        EmptySubscription,
    )
    .data(pool.clone())
    .data(MaxRequestPolls(config.max_request_polls))
    .data(config.bypass_secret.clone().map(CaptchaBypassSecret))
    .data(MaxPaginatedResponses(max_paginated_responses))
    .data(secret)
    .finish();

    let config = config.clone();

    for chain in config.chains.clone() {
        let pool = pool.clone();
        tokio::spawn(
            poll_loop(pool, chain, batch_size), // .instrument(chain_polling_span.clone()),
        );
    }

    let router = Router::new().route("/", get(graphiql).post_service(GraphQL::new(schema)));

    info!("starting server");
    axum::serve(TcpListener::bind("0.0.0.0:8000").await.unwrap(), router)
        .await
        .unwrap();
}

#[instrument(skip_all, fields(chain_id = %chain.id))]
async fn poll_loop(pool: Pool, chain: Chain, batch_size: usize) {
    info!("spawning worker for chain");

    loop {
        let pool = pool.clone();

        info!("spawning worker thread");
        let chain = chain.clone();
        let handle = tokio::spawn(
            async move {
                // recreate each time so that if this task panics, the keyring gets rebuilt
                // make sure to panic *here* so that the tokio task will catch the panic!
                info!("creating chain client");
                let chain_client = ChainClient::new(&chain).await;
                info!("entering polling loop");
                loop {
                    let chain = chain.clone();
                    let requests: Vec<SendRequest> = pool
                        .conn(move |conn| {
                            let mut stmt = conn
                                .prepare_cached(
                                    "SELECT id, denom, address FROM requests 
                                     WHERE tx_hash IS NULL AND chain_id IS ?1 LIMIT ?2",
                                )
                                .expect("SQL statement is valid");

                            let mut rows = stmt
                                .query((&chain.id, batch_size as i64))
                                .expect("can't query rows");

                            let mut requests = vec![];

                            while let Some(row) = rows.next().expect("could not read row") {
                                let id: i64 = row.get(0).expect("could not read id");
                                let denom: String = row.get(1).expect("could not read denom");
                                let receiver: String = row.get(2).expect("could not read address");

                                let Some(coin) =
                                    chain.coins.iter().find(|coin| coin.denom == denom)
                                else {
                                    error!(
                            %denom,
                            "dropping request for unknown denom");
                                    break;
                                };

                                requests.push(SendRequest {
                                    id,
                                    receiver,
                                    denom,
                                    amount: coin.amount,
                                });
                            }

                            Ok(requests)
                        })
                        .await
                        .expect("pool error");

                    if requests.is_empty() {
                        debug!("no requests in queue");
                        tokio::time::sleep(Duration::from_millis(1000)).await;
                        continue;
                    }
                    let mut i = 0;

                    // try sending batch 5 times
                    let result = loop {
                        let send_res = chain_client.send(&requests).await;

                        match send_res {
                            Err(err) => {
                                if i >= 5 {
                                    break format!("ERROR: {}", ErrorReporter(&*err));
                                }
                                warn!(
                                    err = %ErrorReporter(&*err),
                                    attempt = i,
                                    "unable to submit transaction"
                                );
                                i += 1;
                            }
                            // this will be displayed to users, print the hash in the same way that cosmos sdk does
                            Ok(tx_hash) => {
                                break tx_hash
                                    .into_encoding::<HexUnprefixed>()
                                    .to_string()
                                    .to_uppercase()
                            }
                        };
                    };

                    pool.conn(move |conn| {
                        debug!("loading vtab array module required for `IN (1,42,76,...)`");
                        rusqlite::vtab::array::load_module(conn)
                            .expect("error loading vtab array module");

                        let mut stmt = conn
                            .prepare_cached(
                                "UPDATE requests SET tx_hash = ?1 WHERE id IN rarray(?2)",
                            )
                            .expect("???");

                        // https://docs.rs/rusqlite/latest/rusqlite/vtab/array/index.html
                        let rows_modified = stmt
                            .execute((
                                &result,
                                Rc::new(
                                    requests
                                        .iter()
                                        .map(|req| req.id)
                                        .map(rusqlite::types::Value::from)
                                        .collect::<Vec<rusqlite::types::Value>>(),
                                ),
                            ))
                            .expect("can't query rows");

                        info!(rows_modified, "updated requests");

                        Ok(())
                    })
                    .await
                    .expect("pool error");
                }
            }
            .in_current_span(),
        )
        .await;

        match handle {
            Ok(()) => {}
            Err(err) => {
                error!(err = %ErrorReporter(err), "handler panicked");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
pub struct AppArgs {
    #[arg(long, short = 'c')]
    pub config_file_path: OsString,

    #[arg(long, short = 'b', default_value_t = 6000)]
    pub batch_size: usize,

    #[arg(long, short = 'm', default_value_t = 50)]
    pub max_paginated_responses: i32,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogFormat {
    #[default]
    Text,
    Json,
}

impl fmt::Display for LogFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogFormat::Text => f.write_str("text"),
            LogFormat::Json => f.write_str("json"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub chains: Vec<Chain>,
    #[serde(default)]
    pub log_format: LogFormat,
    #[serde(default)]
    pub secret: Option<String>,
    #[serde(default)]
    pub bypass_secret: Option<String>,
    pub max_request_polls: u32,
    #[serde(default)]
    pub ratelimit_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chain {
    pub id: String,
    pub bech32_prefix: String,
    pub rpc_url: String,
    pub gas_config: GasFillerConfig,
    pub signer: H256,
    pub coins: Vec<Coin>,
    pub memo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coin {
    pub denom: String,
    pub amount: u64,
}

pub struct MaxRequestPolls(pub u32);
// pub struct Bech32Prefix(pub String);
pub struct CaptchaBypassSecret(pub String);

#[derive(Clone)]
struct ChainClient {
    pub chain: Chain,
    pub cosmos_ctx: Arc<TxClient<LocalSigner, Rpc, AnyGasFiller>>,
}

impl ChainClient {
    #[instrument(skip_all, fields(chain_id = %chain.id))]
    pub async fn new(chain: &Chain) -> Self {
        let rpc = Rpc::new(chain.rpc_url.clone()).await.unwrap();

        let bech32_prefix = rpc
            .client()
            .grpc_abci_query::<_, protos::cosmos::auth::v1beta1::Bech32PrefixResponse>(
                "/cosmos.auth.v1beta1.Query/Bech32Prefix",
                &protos::cosmos::auth::v1beta1::Bech32PrefixRequest {},
                None,
                false,
            )
            .await
            .unwrap()
            .into_result()
            .unwrap()
            .unwrap()
            .bech32_prefix;

        let gas_filler = AnyGasFiller::from_config(chain.gas_config.clone(), chain.rpc_url.clone())
            .await
            .expect("failed to build gas filler");

        let ctx = TxClient::new(
            LocalSigner::new(chain.signer, bech32_prefix),
            rpc,
            gas_filler,
        );

        let chain_id = ctx.rpc().chain_id();

        // Check if we are connected to a chain with the correct chain_id
        assert_eq!(
            chain_id, chain.id,
            "rpc_url {} is not for chain {}",
            chain.rpc_url, chain.id
        );

        Self {
            chain: chain.clone(),
            cosmos_ctx: Arc::new(ctx),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SendRequest {
    pub id: i64,
    pub receiver: String,
    pub denom: String,
    pub amount: u64,
}

struct AggregatedSendRequest {
    denom: String,
    total_amount: u64,
}

trait SendRequestAggregator {
    fn aggregate_by_denom(&self) -> Vec<AggregatedSendRequest>;
}

impl SendRequestAggregator for Vec<SendRequest> {
    fn aggregate_by_denom(&self) -> Vec<AggregatedSendRequest> {
        let mut denom_map: HashMap<String, (u64, Vec<(String, u64)>)> = HashMap::new();

        // Iterate over the requests and populate the hashmap
        for req in self {
            let entry = denom_map
                .entry(req.denom.clone())
                .or_insert((0, Vec::new()));
            entry.0 += req.amount; // Update the total amount for this denom
            entry.1.push((req.receiver.clone(), req.amount)); // Add (receiver, amount) to the list
        }

        denom_map
            .into_iter()
            .map(|(denom, (total_amount, _))| AggregatedSendRequest {
                denom,
                total_amount,
            })
            .collect()
    }
}

impl ChainClient {
    /// `MultiSend` to the specified addresses. Will return `None` if there are no signers available.
    #[instrument(
        skip_all,
        fields(
            chain_id = %self.chain.id,
            requests.len = %requests.len()
        )
    )]
    async fn send(&self, requests: &Vec<SendRequest>) -> anyhow::Result<H256> {
        let agg_reqs = requests.aggregate_by_denom();

        let msg = protos::cosmos::bank::v1beta1::MsgMultiSend {
            // this is required to be one element
            inputs: vec![protos::cosmos::bank::v1beta1::Input {
                address: self.cosmos_ctx.wallet().address().to_string(),
                coins: agg_reqs
                    .iter()
                    .map(|agg_req| protos::cosmos::base::v1beta1::Coin {
                        denom: agg_req.denom.to_string(),
                        amount: agg_req.total_amount.to_string(),
                    })
                    .collect(),
            }],
            outputs: requests
                .iter()
                .map(|req| protos::cosmos::bank::v1beta1::Output {
                    address: req.receiver.clone(),
                    coins: vec![protos::cosmos::base::v1beta1::Coin {
                        denom: req.denom.clone(),
                        amount: req.amount.to_string(),
                    }],
                })
                .collect(),
        };

        let msg = protos::google::protobuf::Any {
            type_url: protos::cosmos::bank::v1beta1::MsgMultiSend::type_url(),
            value: msg.encode_to_vec().into(),
        };

        let (tx_hash, res) = self
            .cosmos_ctx
            .broadcast_tx_commit([msg], self.chain.memo.clone(), true)
            .await?;

        info!(
            ?requests,
            %tx_hash,
            gas_used = %res.tx_result.gas_used,
            "submitted multisend"
        );

        Ok(tx_hash)
    }
}

struct Mutation {
    ratelimit_seconds: u32,
    chains: Vec<Chain>,
}

#[derive(Debug)]
pub struct CaptchaSecret(pub String);

#[Object]
impl Mutation {
    #[instrument(skip_all, fields(%chain_id, %address, %denom))]
    async fn send<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        captcha_token: String,
        chain_id: String,
        address: String,
        denom: String,
    ) -> Result<String> {
        let secret = ctx.data::<Option<CaptchaSecret>>().unwrap();
        let bypass_secret = ctx.data::<Option<CaptchaBypassSecret>>().unwrap();
        let max_request_polls = ctx.data::<MaxRequestPolls>().unwrap();

        // Get chain config
        let Some(chain) = self.chains.iter().find(|c| c.id == chain_id) else {
            return Err(format!("invalid chain_id {chain_id}").into());
        };

        // Ensure denom exists for chain
        if !chain.coins.iter().any(|coin| coin.denom == denom) {
            return Err(format!("invalid denom {denom}").into());
        };

        let allow_bypass = bypass_secret
            .as_ref()
            .is_some_and(|CaptchaBypassSecret(secret)| secret == &captcha_token);

        if let Some(secret) = secret {
            if !allow_bypass {
                let response = turnstile::verify(&captcha_token, &secret.0)
                    .await
                    .map_err(|err| format!("failed to verify turnstile: {:?}", err))?;

                if !response.success {
                    return Err(format!("failed turnstile request: {:?}", response).into());
                }
            }
        }

        match subtle_encoding::bech32::Bech32::lower_case().decode(&address) {
            Ok((hrp, _bz)) => {
                if hrp != chain.bech32_prefix {
                    return Err(format!(
                        "incorrect bech32 prefix, expected `{}` but found `{hrp}`",
                        chain.bech32_prefix
                    )
                    .into());
                }
            }
            Err(err) => return Err(err.into()),
        };

        let db = ctx.data::<Pool>().unwrap();

        let last_request_ts: Option<String> = db
            .conn({
                let chain_id = chain_id.clone();
                let denom = denom.clone();
                let address = address.clone();

                move |conn| {
                    let mut statement = conn.prepare_cached(
                        "SELECT time FROM requests WHERE chain_id = ? AND denom = ? AND address = ? ORDER BY id DESC LIMIT 1",
                    )?;
                    let time = statement
                        .query_row([chain_id, denom, address], |row| row.get(0))
                        .optional()?;
                    Ok(time)
                }
            })
            .await?;

        match last_request_ts {
            Some(ts) => {
                let ts = NaiveDateTime::parse_from_str(&ts, DATETIME_FORMAT)
                    .expect("invalid datetime present in database");

                let now = Utc::now().naive_utc();

                let delta = now - ts;
                if delta.num_seconds() < 0 {
                    error!(%now, %ts, %delta, "timestamp in the future?");
                }
                if delta.num_seconds() < self.ratelimit_seconds.into() {
                    info!(
                        %chain_id,
                        %denom,
                        %address,
                        %delta,
                        ratelimit_seconds = %self.ratelimit_seconds,
                        "ratelimited"
                    );

                    return Ok("ERROR: ratelimited".to_string());
                }
            }
            None => {
                info!(%address, "new user");
            }
        }

        let id: i64 = db
            .conn(move |conn| {
                let mut statement = conn.prepare_cached(
                    "INSERT INTO requests (chain_id, denom, address, time) VALUES (?, ?, ?, datetime('now')) RETURNING id",
                )?;
                let id = statement.query_row([&chain_id, &denom, &address], |row| row.get(0))?;
                Ok(id)
            })
            .await?;
        let mut counter = 0;
        let tx_hash = loop {
            let tx_hash: Option<String> = db
                .conn(move |conn| {
                    conn.query_row(
                        "SELECT tx_hash FROM requests WHERE id=? ORDER BY time DESC LIMIT 1",
                        [&id],
                        |row| row.get(0),
                    )
                })
                .await?;

            if let Some(tx_hash) = tx_hash {
                break tx_hash;
            } else {
                if counter > max_request_polls.0 {
                    break "ERROR".to_string();
                }
                counter += 1;
                debug!(counter, "no response yet, trying again");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        };

        Ok(tx_hash)
    }
}

#[derive(SimpleObject)]
struct Request {
    id: i64,
    address: String,
    time: String,
    tx_hash: Option<String>,
}

struct MaxPaginatedResponses(i32);
struct Query;

#[Object]
impl Query {
    async fn handled_transfers<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        limit: Option<i32>,
        offset_time: Option<String>,
    ) -> FieldResult<Vec<Request>> {
        let db = ctx.data::<Pool>().unwrap();
        let max_paginated_responses = ctx.data::<MaxPaginatedResponses>().unwrap().0;
        let limit = limit.unwrap_or(10).min(max_paginated_responses);
        let offset_time = offset_time
            .unwrap_or_else(|| Utc::now().naive_utc().format(DATETIME_FORMAT).to_string());
        let requests: Vec<Request> = db
            .conn(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, address, time, tx_hash 
                    FROM requests 
                    WHERE tx_hash IS NOT NULL 
                    AND tx_hash NOT LIKE 'ERROR%' 
                    AND time < ?1
                    ORDER BY time DESC
                    LIMIT ?2",
                )?;
                let rows = stmt.query_map(params![offset_time, limit], |row| {
                    Ok(Request {
                        id: row.get(0)?,
                        address: row.get(1)?,
                        time: row.get(2)?,
                        tx_hash: row.get(3)?,
                    })
                })?;
                let requests: Result<Vec<_>, _> = rows.collect();
                requests
            })
            .await
            .map_err(|e| e.to_string())?;

        Ok(requests)
    }

    async fn transfers_for_address<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        address: String,
        limit: Option<i32>,
        offset_time: Option<String>,
    ) -> FieldResult<Vec<Request>> {
        let db = ctx.data::<Pool>().unwrap();
        let max_paginated_responses = ctx.data::<MaxPaginatedResponses>().unwrap().0;
        let limit = limit.unwrap_or(10).min(max_paginated_responses);
        let offset_time = offset_time
            .unwrap_or_else(|| Utc::now().naive_utc().format(DATETIME_FORMAT).to_string());
        let requests: Vec<Request> = db
            .conn(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, address, time, tx_hash 
                     FROM requests 
                     WHERE address = ?1 
                     AND time < ?2
                     ORDER BY time DESC
                     LIMIT ?3",
                )?;
                let rows = stmt.query_map(params![address, offset_time, limit], |row| {
                    Ok(Request {
                        id: row.get(0)?,
                        address: row.get(1)?,
                        time: row.get(2)?,
                        tx_hash: row.get(3)?,
                    })
                })?;
                let requests: Result<Vec<_>, _> = rows.collect();
                requests
            })
            .await
            .map_err(|e| e.to_string())?;

        Ok(requests)
    }

    async fn unhandled_transfers<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        limit: Option<i32>,
        offset_time: Option<String>,
    ) -> FieldResult<Vec<Request>> {
        let db = ctx.data::<Pool>().unwrap();
        let max_paginated_responses = ctx.data::<MaxPaginatedResponses>().unwrap().0;
        let limit = limit.unwrap_or(10).min(max_paginated_responses);
        let offset_time = offset_time
            .unwrap_or_else(|| Utc::now().naive_utc().format(DATETIME_FORMAT).to_string());
        let requests: Vec<Request> = db
            .conn(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, address, time, tx_hash 
                    FROM requests 
                    WHERE tx_hash IS NULL 
                    AND time < ?1
                    ORDER BY time DESC
                    LIMIT ?2",
                )?;
                let rows = stmt.query_map(params![offset_time, limit], |row| {
                    Ok(Request {
                        id: row.get(0)?,
                        address: row.get(1)?,
                        time: row.get(2)?,
                        tx_hash: row.get(3)?,
                    })
                })?;
                let requests: Result<Vec<_>, _> = rows.collect();
                requests
            })
            .await
            .map_err(|e| e.to_string())?;

        Ok(requests)
    }
}

async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/").finish())
}
