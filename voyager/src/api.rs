use std::{net::SocketAddr, time::Duration};

use axum::{
    extract::State,
    routing::{get, post},
    Json,
};
use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    SinkExt,
};
use opentelemetry_otlp::WithExportConfig;
use prometheus::TextEncoder;
use reqwest::StatusCode;
use tracing::error;
use voyager_message::VoyagerMessage;
use voyager_vm::Op;

pub fn run(laddr: &SocketAddr) -> UnboundedReceiver<Op<VoyagerMessage>> {
    let (queue_tx, queue_rx) = unbounded::<Op<VoyagerMessage>>();

    let app = axum::Router::new()
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

    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_http()
        .with_endpoint("http://localhost:4318")
        .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
        .with_timeout(Duration::from_secs(3))
        .build()
        .expect("unable to build metrics exporter");

    let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder_empty()
                // .with_attributes([KeyValue::new("voyager.name", "example")])
                .build(),
        )
        .build();

    opentelemetry::global::set_meter_provider(provider);

    tokio::spawn(axum::Server::bind(laddr).serve(app.into_make_service()));

    queue_rx
}

// #[axum::debug_handler]
async fn enqueue(
    State(mut sender): State<UnboundedSender<Op<VoyagerMessage>>>,
    Json(op): Json<Op<VoyagerMessage>>,
) -> StatusCode {
    sender.send(op).await.expect("receiver should not close");

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
