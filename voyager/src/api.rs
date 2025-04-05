use std::net::SocketAddr;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use prometheus::TextEncoder;
use reqwest::StatusCode;
use tracing::error;
use voyager_message::VoyagerMessage;
use voyager_vm::Op;

use tokio::sync::mpsc::{unbounded_channel, UnboundedSender, UnboundedReceiver};

pub fn run(laddr: &SocketAddr) -> UnboundedReceiver<Op<VoyagerMessage>> {
    let (queue_tx, queue_rx) = unbounded_channel::<Op<VoyagerMessage>>();

    let app = Router::new()
        .route("/enqueue", post(enqueue))
        .route("/health", get(|| async move { StatusCode::OK }))
        .route("/metrics", get(metrics))
        // .route(
        //     "/signer/balances",
        //     get({
        //         let chains = self.chains.clone();
        //         || async move { Json(signer_balances(&chains).await) }
        //     }),
        // )
        .with_state(queue_tx.clone());

    tokio::spawn(axum::Server::bind(laddr).serve(app.into_make_service()));

    queue_rx
}

// #[axum::debug_handler]
async fn enqueue(
    State(sender): State<UnboundedSender<Op<VoyagerMessage>>>,
    Json(op): Json<Op<VoyagerMessage>>,
) -> StatusCode {
    if sender.send(op).is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}

async fn metrics() -> Result<String, StatusCode> {
    TextEncoder::new()
        .encode_to_string(&prometheus::gather())
        .map_err(|err| {
            error!(?err, "could not gather metrics");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
