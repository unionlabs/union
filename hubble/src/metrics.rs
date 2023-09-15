use lazy_static::lazy_static;
use prometheus::{IntCounterVec, Opts, Registry};
use reqwest::StatusCode;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    pub static ref EVENT_COLLECTOR: IntCounterVec =
        IntCounterVec::new(Opts::new("events", "Events"), &["chain_id", "block_hash"])
            .expect("register EVENT_COLLECTOR");
    pub static ref BLOCK_COLLECTOR: IntCounterVec =
        IntCounterVec::new(Opts::new("blocks", "Blocks"), &["chain_id"])
            .expect("register BLOCK_COLLECTOR");
    pub static ref POST_COLLECTOR: IntCounterVec =
        IntCounterVec::new(Opts::new("posts", "Posts to Hasura"), &["chain_id"])
            .expect("register POSTS");
}

pub fn register_custom_metrics() {
    REGISTRY
        .register(Box::new(EVENT_COLLECTOR.clone()))
        .expect("EVENT_COLLECTOR can be registered");
    REGISTRY
        .register(Box::new(BLOCK_COLLECTOR.clone()))
        .expect("BLOCK_COLLECTOR can be registered");
    REGISTRY
        .register(Box::new(POST_COLLECTOR.clone()))
        .expect("BLOCK_COLLECTOR can be registered");
}

#[axum::debug_handler]
pub async fn handler() -> Result<String, StatusCode> {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();

    let mut buffer = Vec::new();
    encoder
        .encode(&REGISTRY.gather(), &mut buffer)
        .map_err(|err| {
            tracing::error!("could not gather metrics: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut res = String::from_utf8(buffer.clone()).map_err(|err| {
        tracing::error!("could not gather metrics: {}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    buffer.clear();

    let mut buffer = Vec::new();
    encoder
        .encode(&prometheus::gather(), &mut buffer)
        .map_err(|err| {
            tracing::error!("could not gather metrics: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let res_custom = String::from_utf8(buffer.clone()).map_err(|err| {
        tracing::error!("could not gather metrics: {}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    buffer.clear();
    res.push_str(&res_custom);
    Ok(res)
}
