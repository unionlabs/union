use std::{ffi::OsString, fmt, fs::read_to_string, time::Duration};

use async_graphql::{http::GraphiQLSource, *};
use async_graphql_axum::GraphQL;
use async_sqlite::{rusqlite::params, JournalMode, Pool, PoolBuilder};
use axum::{
    response::{self, IntoResponse},
    routing::get,
    Router,
};
use chain_utils::{
    cosmos::{self, Cosmos},
    cosmos_sdk::{BroadcastTxCommitError, CosmosSdkChainExt},
};
use clap::Parser;
use prost::{Message, Name};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tracing::{debug, info, warn};
use tracing_subscriber::EnvFilter;
use unionlabs::{hash::H256, ErrorReporter};

#[tokio::main]
async fn main() {
    let args = AppArgs::parse();

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

    let secret = config.secret.map(CaptchaSecret);

    let pool = PoolBuilder::new()
        .path("db.sqlite3")
        .journal_mode(JournalMode::Wal)
        .open()
        .await
        .expect("opening db");

    pool.conn(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS requests (
                id   INTEGER PRIMARY KEY AUTOINCREMENT,
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

    let faucet = FaucetClient {
        cosmos: Cosmos::new(config.chain)
            .await
            .expect("unable to create cosmos client"),
        faucet_denom: config.faucet_denom,
    };

    info!("spawning worker");
    tokio::spawn({
        let pool = pool.clone();
        async move {
            loop {
                let (ids, addresses) = pool
                    .conn(|conn| {
                        let mut stmt = conn
                            .prepare_cached(
                                "SELECT id, address FROM requests WHERE tx_hash IS NULL",
                            )
                            .expect("???");

                        let mut rows = stmt.query([]).expect("can't query rows");

                        let mut addresses = vec![];
                        let mut ids = vec![];
                        while let Some(row) = rows.next().expect("could not read row") {
                            let id: i64 = row.get(0).expect("could not read id");
                            let address: String = row.get(1).expect("could not read address");

                            addresses.push(address);
                            ids.push(id);
                        }

                        Ok((ids, addresses))
                    })
                    .await
                    .expect("pool error");

                if ids.is_empty() {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    continue;
                }

                let mut i = 0;
                let result = loop {
                    let send_res = faucet
                        .send(
                            addresses
                                .clone()
                                .into_iter()
                                .map(|a| (a, config.amount))
                                .collect(),
                        )
                        .await;

                    match send_res {
                        Err(err) => {
                            if i >= 5 {
                                break format!("ERROR: {}", ErrorReporter(err));
                            }
                            warn!(err = %ErrorReporter(err), attempt = i, "unable to submit transaction");
                            i += 1;
                        }
                        // this will be displayed to users, print the hash in the same way that cosmos sdk does
                        Ok(tx_hash) => break tx_hash.to_string_unprefixed().to_uppercase(),
                    };
                };

                pool.conn(move |conn| {
                    let mut stmt = conn
                        .prepare_cached(
                            "UPDATE requests SET tx_hash = ?1 WHERE id >= ?2 AND id <= ?3",
                        )
                        .expect("???");

                    let rows_modified = stmt
                        .execute(params![
                            &result,
                            &ids.first().unwrap(),
                            &ids.last().unwrap()
                        ])
                        .expect("can't query rows");

                    info!(rows_modified, "updated requests");

                    Ok(())
                })
                .await
                .expect("pool error");
            }
        }
    });

    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .data(pool)
        .data(MaxRequestPolls(config.max_request_polls))
        .data(secret)
        .finish();

    // Build the router
    let router = Router::new().route("/", get(graphiql).post_service(GraphQL::new(schema)));
    // .route("/graphql", get(graphql).post_service(GraphQL::new(schema)));

    info!("starting server");
    axum::serve(TcpListener::bind("0.0.0.0:8000").await.unwrap(), router)
        .await
        .unwrap();
}

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
pub struct AppArgs {
    #[arg(long, short = 'c')]
    pub config_file_path: OsString,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub chain: cosmos::Config,
    pub faucet_denom: String,
    #[serde(default)]
    pub log_format: LogFormat,
    #[serde(default)]
    pub secret: Option<String>,
    pub amount: u64,
    pub max_request_polls: u32,
}

pub struct MaxRequestPolls(pub u32);

struct FaucetClient {
    cosmos: Cosmos,
    faucet_denom: String,
}

impl FaucetClient {
    /// `MultiSend` to the specified addresses. Will return `None` if there are no signers available.
    async fn send(&self, to_send: Vec<(String, u64)>) -> Result<H256, BroadcastTxCommitError> {
        self.cosmos
            .keyring
            .with(|signer| async move {
                let msg = protos::cosmos::bank::v1beta1::MsgMultiSend {
                    // this is required to be one element
                    inputs: vec![protos::cosmos::bank::v1beta1::Input {
                        address: signer.to_string(),
                        coins: vec![protos::cosmos::base::v1beta1::Coin {
                            denom: self.faucet_denom.clone(),
                            amount: to_send
                                .iter()
                                .map(|(_, amount)| *amount)
                                .sum::<u64>()
                                .to_string(),
                        }],
                    }],
                    outputs: to_send
                        .into_iter()
                        .map(|(address, amount)| protos::cosmos::bank::v1beta1::Output {
                            address,
                            coins: vec![protos::cosmos::base::v1beta1::Coin {
                                denom: self.faucet_denom.clone(),
                                amount: amount.to_string(),
                            }],
                        })
                        .collect(),
                };

                let msg = protos::google::protobuf::Any {
                    type_url: protos::cosmos::bank::v1beta1::MsgMultiSend::type_url(),
                    value: msg.encode_to_vec().into(),
                };

                let (tx_hash, gas_used) = self.cosmos.broadcast_tx_commit(signer, [msg]).await?;

                info!(
                    %tx_hash,
                    %gas_used,
                    "submitted multisend"
                );

                Ok(tx_hash)
            })
            .await
            .expect("no signers available?")
    }
}

struct Mutation;

pub struct CaptchaSecret(pub String);

#[Object]
impl Mutation {
    async fn send<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        captcha_token: String,
        to_address: String,
    ) -> Result<String> {
        let secret = ctx.data::<Option<CaptchaSecret>>().unwrap();
        let max_request_polls = ctx.data::<MaxRequestPolls>().unwrap();

        if let Some(secret) = secret {
            recaptcha_verify::verify(&secret.0, &captcha_token, None)
                .await
                .map_err(|err| format!("failed to verify captcha: {:?}", err))?;
        }

        let db = ctx.data::<Pool>().unwrap();
        let id: i64 = db
            .conn(move |conn| {
                let mut stmt = conn.prepare_cached(
                    "INSERT INTO requests (address, time) VALUES (?, TIME()) RETURNING id",
                )?;
                let id = stmt.query_row([&to_address], |row| row.get(0))?;
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

struct Query;

#[Object]
impl Query {
    async fn howdy(&self) -> &'static str {
        "partner"
    }
}

async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/").finish())
}
