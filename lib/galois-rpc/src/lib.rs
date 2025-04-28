use tonic::{
    client::Grpc,
    codec::ProstCodec,
    transport::{Channel, Endpoint},
    GrpcMethod, IntoRequest, Status,
};
use unionlabs::ErrorReporter;

use crate::{poll_request::PollRequest, poll_response::PollResponse};

pub mod canonical_vote;
pub mod poll_request;
pub mod poll_response;
pub mod prove_request;
pub mod prove_response;
pub mod validator_set_commit;
pub mod zero_knowledge_proof;

#[derive(Debug, Clone)]
pub struct Client<T> {
    inner: Grpc<T>,
}

impl Client<Channel> {
    pub async fn connect(dst: impl Into<String>) -> Result<Self, tonic::transport::Error> {
        Ok(Client::<Channel> {
            inner: Grpc::new(Endpoint::new(dst.into())?.connect().await?),
        })
    }

    pub async fn poll(&mut self, request: PollRequest) -> Result<PollResponse, Status> {
        self.inner
            .ready()
            .await
            .map_err(|e| Status::unknown(format!("Service was not ready: {}", ErrorReporter(e))))?;

        let mut req = protos::union::galois::api::v3::PollRequest::from(request).into_request();

        req.extensions_mut().insert(GrpcMethod::new(
            "union.galois.api.v3.UnionProverAPI",
            "Poll",
        ));

        let response = self
            .inner
            .unary::<_, protos::union::galois::api::v3::PollResponse, _>(
                req,
                "/union.galois.api.v3.UnionProverAPI/Poll".parse().unwrap(),
                ProstCodec::default(),
            )
            .await?;

        response
            .into_inner()
            .try_into()
            .map_err(|e| Status::unknown(ErrorReporter(e).to_string()))
    }
}
