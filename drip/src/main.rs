use std::{ffi::OsString, fmt, fs::read_to_string, sync::Arc, time::Duration};

use async_graphql::{http::GraphiQLSource, *};
use async_graphql_axum::GraphQL;
use async_sqlite::{rusqlite::params, JournalMode, Pool, PoolBuilder};
use axum::{response::IntoResponse, routing::get, Router};
use chrono::{NaiveDateTime, Utc};
use clap::Parser;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Parse command-line arguments
    let args = AppArgs::parse();
    let config = load_config(&args.config_file_path).expect("Failed to load config");

    setup_logging(&config.log_format);

    let pool = setup_database().await.expect("Failed to initialize database");

    let schema = build_schema(pool.clone(), &config);

    let worker_handle = tokio::spawn(start_worker(pool.clone(), config.clone()));

    // Create Axum router
    let router = Router::new().route("/", get(graphiql).post_service(GraphQL::new(schema)));

    // Graceful shutdown signal
    tokio::select! {
        _ = axum::serve(TcpListener::bind("0.0.0.0:8000").await.unwrap(), router) => {},
        _ = signal::ctrl_c() => {
            info!("Shutdown signal received. Stopping...");
        }
    }

    worker_handle.abort();
    info!("Server stopped");
}

fn load_config(path: &OsString) -> Result<Config, String> {
    let content = read_to_string(path).map_err(|e| format!("Failed to read config file: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Invalid config format: {}", e))
}

fn setup_logging(log_format: &LogFormat) {
    match log_format {
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
}

async fn setup_database() -> Result<Pool, String> {
    let pool = PoolBuilder::new()
        .path("db.sqlite3")
        .journal_mode(JournalMode::Wal)
        .open()
        .await
        .map_err(|e| format!("Failed to open database: {}", e))?;

    pool.conn(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS requests (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                address TEXT NOT NULL,
                time TEXT,
                tx_hash TEXT
            )",
            (),
        )?;
        Ok(())
    })
    .await
    .map_err(|e| format!("Database setup error: {}", e))?;

    Ok(pool)
}

fn build_schema(pool: Pool, config: &Config) -> Schema<Query, Mutation, EmptySubscription> {
    Schema::build(
        Query,
        Mutation {
            ratelimit_seconds: config.ratelimit_seconds,
        },
        EmptySubscription,
    )
    .data(pool)
    .data(MaxRequestPolls(config.max_request_polls))
    .data(config.clone())
    .finish()
}

async fn start_worker(pool: Pool, config: Config) {
    info!("Worker started");
    loop {
        let requests = fetch_pending_requests(&pool, config.batch_size).await;
        if requests.is_empty() {
            debug!("No pending requests. Sleeping...");
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }

        // Process requests...
        debug!(requests = ?requests, "Processing batch");

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn fetch_pending_requests(pool: &Pool, batch_size: usize) -> Vec<(i64, String)> {
    pool.conn(move |conn| {
        let mut stmt = conn.prepare(
            "SELECT id, address FROM requests WHERE tx_hash IS NULL LIMIT ?1",
        )?;
        let rows = stmt.query_map([batch_size as i64], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

        rows.collect::<Result<Vec<_>, _>>()
    })
    .await
    .unwrap_or_default()
}

async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/").finish())
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub grpc_url: String,
    pub ws_url: String,
    pub max_request_polls: u32,
    pub ratelimit_seconds: u32,
    pub log_format: LogFormat,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LogFormat {
    Text,
    Json,
}

pub struct Query;
pub struct Mutation {
    pub ratelimit_seconds: u32,
}

#[Object]
impl Query {
    async fn dummy(&self) -> String {
        "Dummy Query".to_string()
    }
}
