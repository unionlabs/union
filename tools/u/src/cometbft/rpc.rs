use std::{future::Future, time::Duration};

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use cometbft_rpc::CkUpstreamLoggerLayer;
use http_body_util::{BodyExt, Full};
use jsonrpsee::{
    core::middleware::{Batch, Notification, RpcServiceT},
    http_client::{HttpClientBuilder, HttpRequest},
    types::Id,
    ws_client::RpcServiceBuilder,
};
use tower::Layer;
use tower_http::trace::TraceLayer;
use tracing::{info, instrument, Instrument, Span};
use unionlabs::{bounded::BoundedI64, primitives::Bytes};

use crate::print_json;

#[derive(Debug, Args)]
pub struct Cmd {
    #[arg(global = true, short = 'r', default_value = "http://localhost:26657")]
    pub rpc_url: String,
    #[command(subcommand)]
    pub method: Method,
}

#[derive(Debug, Subcommand)]
pub enum Method {
    /// /abci_info?
    AbciInfo,
    /// /abci_query?path=_&data=_&height=_&prove=_
    AbciQuery {
        path: String,
        data: Bytes,
        #[arg(long, short = 'H')]
        height: Option<BoundedI64<1>>,
        #[arg(long, short = 'p', default_value_t = false)]
        prove: bool,
    },
    /// /block?height=_
    Block {
        #[arg(long, short = 'H')]
        height: Option<BoundedI64<1>>,
    },
    /// /block_by_hash?hash=_
    BlockByHash,
    /// /block_results?height=_
    BlockResults,
    /// /block_search?query=_&page=_&per_page=_&order_by=_
    BlockSearch,
    /// /blockchain?minHeight=_&maxHeight=_
    Blockchain,
    /// /broadcast_evidence?evidence=_
    BroadcastEvidence,
    /// /broadcast_tx_async?tx=_
    BroadcastTxAsync,
    /// /broadcast_tx_commit?tx=_
    BroadcastTxCommit,
    /// /broadcast_tx_sync?tx=_
    BroadcastTxSync,
    /// /check_tx?tx=_
    CheckTx,
    /// /commit?height=_
    Commit,
    /// /consensus_params?height=_
    ConsensusParams,
    /// /consensus_state?
    ConsensusState,
    /// /dump_consensus_state?
    DumpConsensusState,
    /// /genesis?
    Genesis,
    /// /genesis_chunked?chunk=_
    GenesisChunked,
    /// /header?height=_
    Header,
    /// /header_by_hash?hash=_
    HeaderByHash,
    /// /health?
    Health,
    /// /net_info?
    NetInfo,
    /// /num_unconfirmed_txs?
    NumUnconfirmedTxs,
    /// /status?
    Status,
    /// /subscribe?query=_
    Subscribe,
    /// /tx?hash=_&prove=_
    Tx,
    /// /tx_search?query=_&prove=_&page=_&per_page=_&order_by=_
    TxSearch,
    /// /unconfirmed_txs?limit=_
    UnconfirmedTxs,
    /// /unsubscribe?query=_
    Unsubscribe,
    /// /unsubscribe_all?
    UnsubscribeAll,
    /// /validators?height=_&page=_&per_page=_
    Validators,
}

impl Cmd {
    #[instrument(skip_all, fields())]
    pub async fn run(self) -> Result<()> {
        let client = cometbft_rpc::Client::on_http(
            HttpClientBuilder::new()
                .set_rpc_middleware(RpcServiceBuilder::new().layer(IdLoggerLayer::new()))
                .set_http_middleware(
                    tower::ServiceBuilder::new().layer(
                        TraceLayer::new_for_http()
                            .make_span_with(|request: &HttpRequest| {
                                // #[derive(serde::Deserialize)]
                                // struct OnlyIdResponse<'a> {
                                //     #[serde(borrow)]
                                //     id: Id<'a>,
                                // }

                                // span.record(
                                //     "id",
                                //     serde_json::from_str::<OnlyIdResponse>(
                                //         request.body().into_data_stream(),
                                //     )
                                //     .unwrap()
                                //     .id
                                //     .to_string(),
                                // );

                                tracing::info_span!(
                                    "request_info",
                                    ck_upstream = tracing::field::Empty,
                                    id = tracing::field::Empty,
                                )
                            })
                            // .on_request(|request: &HttpRequest, span: &Span| {
                            //     // dbg!(request.extensions().len());
                            //     // request.body()
                            // })
                            .on_response(
                                |response: &http::response::Response<hyper::body::Incoming>,
                                 latency: Duration,
                                 span: &Span| {
                                    let ck_upstream =
                                        response.headers().get("ck-upstream").cloned();

                                    span.record(
                                        "ck_upstream",
                                        ck_upstream.as_ref().map_or("unknown", |ck| {
                                            ck.to_str().unwrap_or("invalid")
                                        }),
                                    );

                                    info!(?ck_upstream, "ck_upstream");
                                },
                            ),
                    ),
                )
                .build(self.rpc_url)?,
        );

        match self.method {
            Method::AbciInfo => print_json(&client.abci_info().await?),
            Method::AbciQuery {
                path,
                data,
                height,
                prove,
            } => print_json(&client.abci_query(path, data, height, prove).await?),
            Method::Block { height } => print_json(&client.block(height).await?),
            Method::Status => print_json(&client.status().await?),
            _ => bail!("not yet implemented"),
        }

        Ok(())
    }
}

/// RPC logger layer.
#[derive(Copy, Clone, Debug)]
pub struct IdLoggerLayer;

impl IdLoggerLayer {
    /// Create a new logging layer.
    pub fn new() -> Self {
        Self
    }
}

impl<S> tower::Layer<S> for IdLoggerLayer {
    type Service = IdLogger<S>;

    fn layer(&self, service: S) -> Self::Service {
        IdLogger { service }
    }
}

#[derive(Debug, Clone)]
pub struct IdLogger<S> {
    service: S,
}

impl<S> RpcServiceT for IdLogger<S>
where
    S: RpcServiceT + Send + Sync + Clone + 'static,
    // S::MethodResponse: ToJson,
    // S::BatchResponse: ToJson,
{
    type MethodResponse = S::MethodResponse;
    type NotificationResponse = S::NotificationResponse;
    type BatchResponse = S::BatchResponse;

    fn call<'a>(
        &self,
        request: jsonrpsee::types::Request<'a>,
    ) -> impl Future<Output = Self::MethodResponse> + Send + 'a {
        dbg!(tracing::Span::current()).record("id", request.id.to_string());
        // info!(id = %request.id, len = request.extensions.len(), "id");
        self.service.call(request)
        //             .instrument(
        // // name = "method_call_with_id", skip_all, fields(id = %request.id), level = "info"
        //         )
    }

    fn batch<'a>(&self, batch: Batch<'a>) -> impl Future<Output = Self::BatchResponse> + Send + 'a {
        self.service.batch(batch)
    }

    fn notification<'a>(
        &self,
        n: Notification<'a>,
    ) -> impl Future<Output = Self::NotificationResponse> + Send + 'a {
        self.service.notification(n)
    }
}
