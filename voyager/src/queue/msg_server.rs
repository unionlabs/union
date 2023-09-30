use std::collections::VecDeque;

use axum::{
    self,
    extract::State,
    routing::{get, post},
    Json,
};
use chain_utils::EventSource;
use futures::StreamExt;
use reqwest::StatusCode;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio_stream::{wrappers::UnboundedReceiverStream, Stream};

use crate::msg::RelayerMsg;

#[derive(Debug, Clone)]
pub struct MsgServer;

impl EventSource for MsgServer {
    type Event = RelayerMsg;
    type Error = ();
    type Seed = ();

    fn events(&self, _: Self::Seed) -> impl Stream<Item = Result<Self::Event, Self::Error>> + '_ {
        let (tx, rx) = unbounded_channel();

        let app = axum::Router::new()
            .route("/msg", post(msg))
            .route("/msgs", post(msgs))
            .route("/health", get(|| async move { StatusCode::OK }))
            .with_state(tx);

        // #[axum::debug_handler]
        async fn msg(
            State(sender): State<UnboundedSender<RelayerMsg>>,
            Json(msg): Json<RelayerMsg>,
        ) -> StatusCode {
            tracing::info!(?msg, "received msg");
            sender.send(msg).unwrap();

            StatusCode::OK
        }

        // #[axum::debug_handler]
        async fn msgs(
            State(sender): State<UnboundedSender<RelayerMsg>>,
            Json(msgs): Json<Vec<RelayerMsg>>,
        ) -> StatusCode {
            tracing::info!(?msgs, "received msgs");
            for msg in msgs {
                sender.send(msg).unwrap();
            }

            StatusCode::OK
        }

        tokio::spawn(
            // TODO: Make this configurable
            axum::Server::bind(&"0.0.0.0:65534".parse().unwrap()).serve(app.into_make_service()),
        );

        UnboundedReceiverStream::new(rx).map(Ok)
    }
}
