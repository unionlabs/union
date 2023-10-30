use crate::{
    body::{boxed, Body, Empty, HttpBody},
    response::Response,
};
use bytes::Bytes;
use http::{
    header::{self, CONTENT_LENGTH},
    HeaderMap, HeaderValue, Request,
};
use pin_project_lite::pin_project;
use std::{
    convert::Infallible,
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{
    util::{BoxCloneService, Oneshot},
    ServiceExt,
};
use tower_service::Service;

/// How routes are stored inside a [`Router`](super::Router).
///
/// You normally shouldn't need to care about this type. It's used in
/// [`Router::layer`](super::Router::layer).
pub struct Route<B = Body, E = Infallible>(BoxCloneService<Request<B>, Response, E>);

impl<B, E> Route<B, E> {
    pub(super) fn new<T>(svc: T) -> Self
    where
        T: Service<Request<B>, Response = Response, Error = E> + Clone + Send + 'static,
        T::Future: Send + 'static,
    {
        Self(BoxCloneService::new(svc))
    }

    pub(crate) fn oneshot_inner(
        &mut self,
        req: Request<B>,
    ) -> Oneshot<BoxCloneService<Request<B>, Response, E>, Request<B>> {
        self.0.clone().oneshot(req)
    }
}

impl<ReqBody, E> Clone for Route<ReqBody, E> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<ReqBody, E> fmt::Debug for Route<ReqBody, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Route").finish()
    }
}

impl<B, E> Service<Request<B>> for Route<B, E>
where
    B: HttpBody,
{
    type Response = Response;
    type Error = E;
    type Future = RouteFuture<B, E>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: Request<B>) -> Self::Future {
        RouteFuture::from_future(self.oneshot_inner(req))
    }
}

pin_project! {
    /// Response future for [`Route`].
    pub struct RouteFuture<B, E> {
        #[pin]
        kind: RouteFutureKind<B, E>,
        strip_body: bool,
        allow_header: Option<Bytes>,
    }
}

pin_project! {
    #[project = RouteFutureKindProj]
    enum RouteFutureKind<B, E> {
        Future {
            #[pin]
            future: Oneshot<
                BoxCloneService<Request<B>, Response, E>,
                Request<B>,
            >,
        },
        Response {
            response: Option<Response>,
        }
    }
}

impl<B, E> RouteFuture<B, E> {
    pub(crate) fn from_future(
        future: Oneshot<BoxCloneService<Request<B>, Response, E>, Request<B>>,
    ) -> Self {
        Self {
            kind: RouteFutureKind::Future { future },
            strip_body: false,
            allow_header: None,
        }
    }

    pub(super) fn from_response(response: Response) -> Self {
        Self {
            kind: RouteFutureKind::Response {
                response: Some(response),
            },
            strip_body: false,
            allow_header: None,
        }
    }

    pub(crate) fn strip_body(mut self, strip_body: bool) -> Self {
        self.strip_body = strip_body;
        self
    }

    pub(crate) fn allow_header(mut self, allow_header: Bytes) -> Self {
        self.allow_header = Some(allow_header);
        self
    }
}

impl<B, E> Future for RouteFuture<B, E>
where
    B: HttpBody,
{
    type Output = Result<Response, E>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        #[derive(Clone, Copy)]
        struct AlreadyPassedThroughRouteFuture;

        let this = self.project();

        let mut res = match this.kind.project() {
            RouteFutureKindProj::Future { future } => match future.poll(cx) {
                Poll::Ready(Ok(res)) => res,
                Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
                Poll::Pending => return Poll::Pending,
            },
            RouteFutureKindProj::Response { response } => {
                response.take().expect("future polled after completion")
            }
        };

        if res
            .extensions()
            .get::<AlreadyPassedThroughRouteFuture>()
            .is_some()
        {
            return Poll::Ready(Ok(res));
        } else {
            res.extensions_mut().insert(AlreadyPassedThroughRouteFuture);
        }

        set_allow_header(res.headers_mut(), this.allow_header);

        // make sure to set content-length before removing the body
        set_content_length(res.size_hint(), res.headers_mut());

        let res = if *this.strip_body {
            res.map(|_| boxed(Empty::new()))
        } else {
            res
        };

        Poll::Ready(Ok(res))
    }
}

fn set_allow_header(headers: &mut HeaderMap, allow_header: &mut Option<Bytes>) {
    match allow_header.take() {
        Some(allow_header) if !headers.contains_key(header::ALLOW) => {
            headers.insert(
                header::ALLOW,
                HeaderValue::from_maybe_shared(allow_header).expect("invalid `Allow` header"),
            );
        }
        _ => {}
    }
}

fn set_content_length(size_hint: http_body::SizeHint, headers: &mut HeaderMap) {
    if headers.contains_key(CONTENT_LENGTH) {
        return;
    }

    if let Some(size) = size_hint.exact() {
        let header_value = if size == 0 {
            #[allow(clippy::declare_interior_mutable_const)]
            const ZERO: HeaderValue = HeaderValue::from_static("0");

            ZERO
        } else {
            let mut buffer = itoa::Buffer::new();
            HeaderValue::from_str(buffer.format(size)).unwrap()
        };

        headers.insert(CONTENT_LENGTH, header_value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traits() {
        use crate::test_helpers::*;
        assert_send::<Route<()>>();
    }
}
