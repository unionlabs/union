use crate::{
    body::{self, Bytes, HttpBody},
    response::{IntoResponse, Response},
    BoxError,
};
use http::Request;
use pin_project_lite::pin_project;
use std::{
    any::type_name,
    convert::Infallible,
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{util::BoxCloneService, ServiceBuilder};
use tower_http::ServiceBuilderExt;
use tower_layer::Layer;
use tower_service::Service;

/// Create a middleware from an async function.
///
/// `from_fn` requires the function given to
///
/// 1. Be an `async fn`.
/// 2. Take [`Request<B>`](http::Request) as the first argument.
/// 3. Take [`Next<B>`](Next) as the second argument.
/// 4. Return something that implements [`IntoResponse`].
///
/// # Example
///
/// ```rust
/// use axum::{
///     Router,
///     http::{Request, StatusCode},
///     routing::get,
///     response::{IntoResponse, Response},
///     middleware::{self, Next},
/// };
///
/// async fn auth<B>(req: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
///     let auth_header = req.headers()
///         .get(http::header::AUTHORIZATION)
///         .and_then(|header| header.to_str().ok());
///
///     match auth_header {
///         Some(auth_header) if token_is_valid(auth_header) => {
///             Ok(next.run(req).await)
///         }
///         _ => Err(StatusCode::UNAUTHORIZED),
///     }
/// }
///
/// fn token_is_valid(token: &str) -> bool {
///     // ...
///     # false
/// }
///
/// let app = Router::new()
///     .route("/", get(|| async { /* ... */ }))
///     .route_layer(middleware::from_fn(auth));
/// # let app: Router = app;
/// ```
///
/// # Passing state
///
/// State can be passed to the function like so:
///
/// ```rust
/// use axum::{
///     Router,
///     http::{Request, StatusCode},
///     routing::get,
///     response::{IntoResponse, Response},
///     middleware::{self, Next}
/// };
///
/// #[derive(Clone)]
/// struct State { /* ... */ }
///
/// async fn my_middleware<B>(
///     req: Request<B>,
///     next: Next<B>,
///     state: State,
/// ) -> Response {
///     // ...
///     # ().into_response()
/// }
///
/// let state = State { /* ... */ };
///
/// let app = Router::new()
///     .route("/", get(|| async { /* ... */ }))
///     .route_layer(middleware::from_fn(move |req, next| {
///         my_middleware(req, next, state.clone())
///     }));
/// # let app: Router = app;
/// ```
///
/// Or via extensions:
///
/// ```rust
/// use axum::{
///     Router,
///     extract::Extension,
///     http::{Request, StatusCode},
///     routing::get,
///     response::{IntoResponse, Response},
///     middleware::{self, Next},
/// };
/// use tower::ServiceBuilder;
///
/// #[derive(Clone)]
/// struct State { /* ... */ }
///
/// async fn my_middleware<B>(
///     req: Request<B>,
///     next: Next<B>,
/// ) -> Response {
///     let state: &State = req.extensions().get().unwrap();
///
///     // ...
///     # ().into_response()
/// }
///
/// let state = State { /* ... */ };
///
/// let app = Router::new()
///     .route("/", get(|| async { /* ... */ }))
///     .layer(
///         ServiceBuilder::new()
///             .layer(Extension(state))
///             .layer(middleware::from_fn(my_middleware)),
///     );
/// # let app: Router = app;
/// ```
pub fn from_fn<F>(f: F) -> FromFnLayer<F> {
    FromFnLayer { f }
}

/// A [`tower::Layer`] from an async function.
///
/// [`tower::Layer`] is used to apply middleware to [`Router`](crate::Router)'s.
///
/// Created with [`from_fn`]. See that function for more details.
#[derive(Clone, Copy)]
pub struct FromFnLayer<F> {
    f: F,
}

impl<S, F> Layer<S> for FromFnLayer<F>
where
    F: Clone,
{
    type Service = FromFn<F, S>;

    fn layer(&self, inner: S) -> Self::Service {
        FromFn {
            f: self.f.clone(),
            inner,
        }
    }
}

impl<F> fmt::Debug for FromFnLayer<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FromFnLayer")
            // Write out the type name, without quoting it as `&type_name::<F>()` would
            .field("f", &format_args!("{}", type_name::<F>()))
            .finish()
    }
}

/// A middleware created from an async function.
///
/// Created with [`from_fn`]. See that function for more details.
#[derive(Clone, Copy)]
pub struct FromFn<F, S> {
    f: F,
    inner: S,
}

impl<F, Fut, Out, S, ReqBody, ResBody> Service<Request<ReqBody>> for FromFn<F, S>
where
    F: FnMut(Request<ReqBody>, Next<ReqBody>) -> Fut,
    Fut: Future<Output = Out>,
    Out: IntoResponse,
    S: Service<Request<ReqBody>, Response = Response<ResBody>, Error = Infallible>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    ResBody: HttpBody<Data = Bytes> + Send + 'static,
    ResBody::Error: Into<BoxError>,
{
    type Response = Response;
    type Error = Infallible;
    type Future = ResponseFuture<Fut>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let not_ready_inner = self.inner.clone();
        let ready_inner = std::mem::replace(&mut self.inner, not_ready_inner);

        let inner = ServiceBuilder::new()
            .boxed_clone()
            .map_response_body(body::boxed)
            .service(ready_inner);
        let next = Next { inner };

        ResponseFuture {
            inner: (self.f)(req, next),
        }
    }
}

impl<F, S> fmt::Debug for FromFn<F, S>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FromFnLayer")
            .field("f", &format_args!("{}", type_name::<F>()))
            .field("inner", &self.inner)
            .finish()
    }
}

/// The remainder of a middleware stack, including the handler.
pub struct Next<ReqBody> {
    inner: BoxCloneService<Request<ReqBody>, Response, Infallible>,
}

impl<ReqBody> Next<ReqBody> {
    /// Execute the remaining middleware stack.
    pub async fn run(mut self, req: Request<ReqBody>) -> Response {
        match self.inner.call(req).await {
            Ok(res) => res,
            Err(err) => match err {},
        }
    }
}

impl<ReqBody> fmt::Debug for Next<ReqBody> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FromFnLayer")
            .field("inner", &self.inner)
            .finish()
    }
}

pin_project! {
    /// Response future for [`FromFn`].
    pub struct ResponseFuture<F> {
        #[pin]
        inner: F,
    }
}

impl<F, Out> Future for ResponseFuture<F>
where
    F: Future<Output = Out>,
    Out: IntoResponse,
{
    type Output = Result<Response, Infallible>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project()
            .inner
            .poll(cx)
            .map(IntoResponse::into_response)
            .map(Ok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{body::Empty, routing::get, Router};
    use http::{HeaderMap, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn basic() {
        async fn insert_header<B>(mut req: Request<B>, next: Next<B>) -> impl IntoResponse {
            req.headers_mut()
                .insert("x-axum-test", "ok".parse().unwrap());

            next.run(req).await
        }

        async fn handle(headers: HeaderMap) -> String {
            (&headers["x-axum-test"]).to_str().unwrap().to_owned()
        }

        let app = Router::new()
            .route("/", get(handle))
            .layer(from_fn(insert_header));

        let res = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .body(body::boxed(Empty::new()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = hyper::body::to_bytes(res).await.unwrap();
        assert_eq!(&body[..], b"ok");
    }
}
