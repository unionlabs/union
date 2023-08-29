use axum::{self, extract::State, routing::post, Json};
use chain_utils::EventSource;
use futures::StreamExt;
use reqwest::StatusCode;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio_stream::{wrappers::UnboundedReceiverStream, Stream};

use crate::msg::RelayerMsg;

pub struct MsgServer;

impl EventSource for MsgServer {
    type Event = RelayerMsg;
    type Error = ();
    type Seed = ();

    fn events(&self, _: Self::Seed) -> impl Stream<Item = Result<Self::Event, Self::Error>> + '_ {
        let (tx, rx) = unbounded_channel();

        let app = axum::Router::new()
            .route("/msg", post(msg_listener))
            .with_state(tx);

        #[axum::debug_handler]
        async fn msg_listener(
            State(sender): State<UnboundedSender<RelayerMsg>>,
            Json(msg): Json<RelayerMsg>,
        ) -> StatusCode {
            tracing::info!(?msg, "received msg");
            sender.send(msg).unwrap();

            StatusCode::OK
        }

        tokio::spawn(
            // TODO: Make this configurable
            axum::Server::bind(&"0.0.0.0:65534".parse().unwrap()).serve(app.into_make_service()),
        );

        UnboundedReceiverStream::new(rx).map(Ok)
    }
}
