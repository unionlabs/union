use crate::{
    body::{Bytes, HttpBody},
    extract::{FromRequest, RequestParts},
    response::{IntoResponse, Response},
    BoxError,
};
use futures_util::{future::BoxFuture, ready};
use http::Request;
use pin_project_lite::pin_project;
use std::{
    fmt,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use tower_layer::Layer;
use tower_service::Service;

/// Create a middleware from an extractor.
///
/// If the extractor succeeds the value will be discarded and the inner service
/// will be called. If the extractor fails the rejection will be returned and
/// the inner service will _not_ be called.
///
/// This can be used to perform validation of requests if the validation doesn't
/// produce any useful output, and run the extractor for several handlers
/// without repeating it in the function signature.
///
/// Note that if the extractor consumes the request body, as `String` or
/// [`Bytes`] does, an empty body will be left in its place. Thus wont be
/// accessible to subsequent extractors or handlers.
///
/// # Example
///
/// ```rust
/// use axum::{
///     extract::{FromRequest, RequestParts},
///     middleware::from_extractor,
///     routing::{get, post},
///     Router,
/// };
/// use http::{header, StatusCode};
/// use async_trait::async_trait;
///
/// // An extractor that performs authorization.
/// struct RequireAuth;
///
/// #[async_trait]
/// impl<B> FromRequest<B> for RequireAuth
/// where
///     B: Send,
/// {
///     type Rejection = StatusCode;
///
///     async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
///         let auth_header = req
///             .headers()
///             .get(header::AUTHORIZATION)
///             .and_then(|value| value.to_str().ok());
///
///         match auth_header {
///             Some(auth_header) if token_is_valid(auth_header) => {
///                 Ok(Self)
///             }
///             _ => Err(StatusCode::UNAUTHORIZED),
///         }
///     }
/// }
///
/// fn token_is_valid(token: &str) -> bool {
///     // ...
///     # false
/// }
///
/// async fn handler() {
///     // If we get here the request has been authorized
/// }
///
/// async fn other_handler() {
///     // If we get here the request has been authorized
/// }
///
/// let app = Router::new()
///     .route("/", get(handler))
///     .route("/foo", post(other_handler))
///     // The extractor will run before all routes
///     .route_layer(from_extractor::<RequireAuth>());
/// # async {
/// # axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
/// # };
/// ```
pub fn from_extractor<E>() -> FromExtractorLayer<E> {
    FromExtractorLayer(PhantomData)
}

/// [`Layer`] that applies [`FromExtractor`] that runs an extractor and
/// discards the value.
///
/// See [`from_extractor`] for more details.
///
/// [`Layer`]: tower::Layer
pub struct FromExtractorLayer<E>(PhantomData<fn() -> E>);

impl<E> Clone for FromExtractorLayer<E> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

impl<E> fmt::Debug for FromExtractorLayer<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FromExtractorLayer")
            .field("extractor", &format_args!("{}", std::any::type_name::<E>()))
            .finish()
    }
}

impl<E, S> Layer<S> for FromExtractorLayer<E> {
    type Service = FromExtractor<S, E>;

    fn layer(&self, inner: S) -> Self::Service {
        FromExtractor {
            inner,
            _extractor: PhantomData,
        }
    }
}

/// Middleware that runs an extractor and discards the value.
///
/// See [`from_extractor`] for more details.
pub struct FromExtractor<S, E> {
    inner: S,
    _extractor: PhantomData<fn() -> E>,
}

#[test]
fn traits() {
    use crate::test_helpers::*;
    assert_send::<FromExtractor<(), NotSendSync>>();
    assert_sync::<FromExtractor<(), NotSendSync>>();
}

impl<S, E> Clone for FromExtractor<S, E>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _extractor: PhantomData,
        }
    }
}

impl<S, E> fmt::Debug for FromExtractor<S, E>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FromExtractor")
            .field("inner", &self.inner)
            .field("extractor", &format_args!("{}", std::any::type_name::<E>()))
            .finish()
    }
}

impl<S, E, ReqBody, ResBody> Service<Request<ReqBody>> for FromExtractor<S, E>
where
    E: FromRequest<ReqBody> + 'static,
    ReqBody: Default + Send + 'static,
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone,
    ResBody: HttpBody<Data = Bytes> + Send + 'static,
    ResBody::Error: Into<BoxError>,
{
    type Response = Response;
    type Error = S::Error;
    type Future = ResponseFuture<ReqBody, S, E>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let extract_future = Box::pin(async move {
            let mut req = RequestParts::new(req);
            let extracted = E::from_request(&mut req).await;
            (req, extracted)
        });

        ResponseFuture {
            state: State::Extracting {
                future: extract_future,
            },
            svc: Some(self.inner.clone()),
        }
    }
}

pin_project! {
    /// Response future for [`FromExtractor`].
    #[allow(missing_debug_implementations)]
    pub struct ResponseFuture<ReqBody, S, E>
    where
        E: FromRequest<ReqBody>,
        S: Service<Request<ReqBody>>,
    {
        #[pin]
        state: State<ReqBody, S, E>,
        svc: Option<S>,
    }
}

pin_project! {
    #[project = StateProj]
    enum State<ReqBody, S, E>
    where
        E: FromRequest<ReqBody>,
        S: Service<Request<ReqBody>>,
    {
        Extracting { future: BoxFuture<'static, (RequestParts<ReqBody>, Result<E, E::Rejection>)> },
        Call { #[pin] future: S::Future },
    }
}

impl<ReqBody, S, E, ResBody> Future for ResponseFuture<ReqBody, S, E>
where
    E: FromRequest<ReqBody>,
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ReqBody: Default,
    ResBody: HttpBody<Data = Bytes> + Send + 'static,
    ResBody::Error: Into<BoxError>,
{
    type Output = Result<Response, S::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            let new_state = match this.state.as_mut().project() {
                StateProj::Extracting { future } => {
                    let (req, extracted) = ready!(future.as_mut().poll(cx));

                    match extracted {
                        Ok(_) => {
                            let mut svc = this.svc.take().expect("future polled after completion");
                            let req = req.try_into_request().unwrap_or_default();
                            let future = svc.call(req);
                            State::Call { future }
                        }
                        Err(err) => {
                            let res = err.into_response();
                            return Poll::Ready(Ok(res));
                        }
                    }
                }
                StateProj::Call { future } => {
                    return future
                        .poll(cx)
                        .map(|result| result.map(|response| response.map(crate::body::boxed)));
                }
            };

            this.state.set(new_state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{handler::Handler, routing::get, test_helpers::*, Router};
    use http::{header, StatusCode};

    #[tokio::test]
    async fn test_from_extractor() {
        struct RequireAuth;

        #[async_trait::async_trait]
        impl<B> FromRequest<B> for RequireAuth
        where
            B: Send,
        {
            type Rejection = StatusCode;

            async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
                if let Some(auth) = req
                    .headers()
                    .get(header::AUTHORIZATION)
                    .and_then(|v| v.to_str().ok())
                {
                    if auth == "secret" {
                        return Ok(Self);
                    }
                }

                Err(StatusCode::UNAUTHORIZED)
            }
        }

        async fn handler() {}

        let app = Router::new().route("/", get(handler.layer(from_extractor::<RequireAuth>())));

        let client = TestClient::new(app);

        let res = client.get("/").send().await;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        let res = client
            .get("/")
            .header(http::header::AUTHORIZATION, "secret")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);
    }
}
