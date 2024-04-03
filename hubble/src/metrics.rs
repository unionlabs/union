use lazy_static::lazy_static;
use prometheus::{IntCounterVec, Opts, Registry};
use reqwest::StatusCode;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    pub static ref EVENT_COLLECTOR: IntCounterVec = IntCounterVec::new(
        Opts::new("events", "Events")
            .namespace("hubble")
            .subsystem("index"),
        &["chain_id"]
    )
    .expect("register EVENT_COLLECTOR");
    pub static ref BLOCK_COLLECTOR: IntCounterVec = IntCounterVec::new(
        Opts::new("blocks", "Blocks")
            .namespace("hubble")
            .subsystem("index"),
        &["chain_id"]
    )
    .expect("register BLOCK_COLLECTOR");
    pub static ref TRANSACTION_COLLECTOR: IntCounterVec = IntCounterVec::new(
        Opts::new("requests", "Transactions")
            .namespace("hubble")
            .subsystem("index"),
        &["chain_id"]
    )
    .expect("register TRANSACTION_COLLECTOR");
}

pub fn register_custom_metrics() {
    REGISTRY
        .register(Box::new(EVENT_COLLECTOR.clone()))
        .expect("EVENT_COLLECTOR can be registered");
    REGISTRY
        .register(Box::new(BLOCK_COLLECTOR.clone()))
        .expect("BLOCK_COLLECTOR can be registered");
    REGISTRY
        .register(Box::new(TRANSACTION_COLLECTOR.clone()))
        .expect("TRANSACTION_COLLECTOR can be registered");
}

#[axum::debug_handler]
pub async fn handler() -> Result<String, StatusCode> {
    let encoder = prometheus::TextEncoder::new();
    let mut response = encoder
        .encode_to_string(&REGISTRY.gather())
        .map_err(|err| {
            tracing::error!("could not gather metrics: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    encoder
        .encode_utf8(&prometheus::gather(), &mut response)
        .map_err(|err| {
            tracing::error!("could not gather metrics: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(response)
}
