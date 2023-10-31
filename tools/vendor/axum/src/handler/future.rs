//! Handler future types.

use crate::response::Response;
use futures_util::future::Map;
use http::Request;
use pin_project_lite::pin_project;
use std::{convert::Infallible, future::Future, pin::Pin, task::Context};
use tower::util::Oneshot;
use tower_service::Service;

opaque_future! {
    /// The response future for [`IntoService`](super::IntoService).
    pub type IntoServiceFuture<F> =
        Map<
            F,
            fn(Response) -> Result<Response, Infallible>,
        >;
}

pin_project! {
    /// The response future for [`Layered`](super::Layered).
    pub struct LayeredFuture<S, ReqBody>
    where
        S: Service<Request<ReqBody>>,
    {
        #[pin]
        inner: Map<Oneshot<S, Request<ReqBody>>, fn(Result<S::Response, S::Error>) -> Response>,
    }
}

impl<S, ReqBody> LayeredFuture<S, ReqBody>
where
    S: Service<Request<ReqBody>>,
{
    pub(super) fn new(
        inner: Map<Oneshot<S, Request<ReqBody>>, fn(Result<S::Response, S::Error>) -> Response>,
    ) -> Self {
        Self { inner }
    }
}

impl<S, ReqBody> Future for LayeredFuture<S, ReqBody>
where
    S: Service<Request<ReqBody>>,
{
    type Output = Response;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> std::task::Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}
