#![doc = include_str!("../docs/error_handling.md")]

use crate::{
    body::{boxed, Bytes, HttpBody},
    extract::{FromRequest, RequestParts},
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    BoxError,
};
use std::{
    convert::Infallible,
    fmt,
    future::Future,
    marker::PhantomData,
    task::{Context, Poll},
};
use tower::ServiceExt;
use tower_layer::Layer;
use tower_service::Service;

/// [`Layer`] that applies [`HandleError`] which is a [`Service`] adapter
/// that handles errors by converting them into responses.
///
/// See [module docs](self) for more details on axum's error handling model.
pub struct HandleErrorLayer<F, T> {
    f: F,
    _extractor: PhantomData<fn() -> T>,
}

impl<F, T> HandleErrorLayer<F, T> {
    /// Create a new `HandleErrorLayer`.
    pub fn new(f: F) -> Self {
        Self {
            f,
            _extractor: PhantomData,
        }
    }
}

impl<F, T> Clone for HandleErrorLayer<F, T>
where
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            f: self.f.clone(),
            _extractor: PhantomData,
        }
    }
}

impl<F, E> fmt::Debug for HandleErrorLayer<F, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HandleErrorLayer")
            .field("f", &format_args!("{}", std::any::type_name::<F>()))
            .finish()
    }
}

impl<S, F, T> Layer<S> for HandleErrorLayer<F, T>
where
    F: Clone,
{
    type Service = HandleError<S, F, T>;

    fn layer(&self, inner: S) -> Self::Service {
        HandleError::new(inner, self.f.clone())
    }
}

/// A [`Service`] adapter that handles errors by converting them into responses.
///
/// See [module docs](self) for more details on axum's error handling model.
pub struct HandleError<S, F, T> {
    inner: S,
    f: F,
    _extractor: PhantomData<fn() -> T>,
}

impl<S, F, T> HandleError<S, F, T> {
    /// Create a new `HandleError`.
    pub fn new(inner: S, f: F) -> Self {
        Self {
            inner,
            f,
            _extractor: PhantomData,
        }
    }
}

impl<S, F, T> Clone for HandleError<S, F, T>
where
    S: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            f: self.f.clone(),
            _extractor: PhantomData,
        }
    }
}

impl<S, F, E> fmt::Debug for HandleError<S, F, E>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HandleError")
            .field("inner", &self.inner)
            .field("f", &format_args!("{}", std::any::type_name::<F>()))
            .finish()
    }
}

impl<S, F, ReqBody, ResBody, Fut, Res> Service<Request<ReqBody>> for HandleError<S, F, ()>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Error: Send,
    S::Future: Send,
    F: FnOnce(S::Error) -> Fut + Clone + Send + 'static,
    Fut: Future<Output = Res> + Send,
    Res: IntoResponse,
    ReqBody: Send + 'static,
    ResBody: HttpBody<Data = Bytes> + Send + 'static,
    ResBody::Error: Into<BoxError>,
{
    type Response = Response;
    type Error = Infallible;
    type Future = future::HandleErrorFuture;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let f = self.f.clone();

        let clone = self.inner.clone();
        let inner = std::mem::replace(&mut self.inner, clone);

        let future = Box::pin(async move {
            match inner.oneshot(req).await {
                Ok(res) => Ok(res.map(boxed)),
                Err(err) => Ok(f(err).await.into_response()),
            }
        });

        future::HandleErrorFuture { future }
    }
}

#[allow(unused_macros)]
macro_rules! impl_service {
    ( $($ty:ident),* $(,)? ) => {
        impl<S, F, ReqBody, ResBody, Res, Fut, $($ty,)*> Service<Request<ReqBody>>
            for HandleError<S, F, ($($ty,)*)>
        where
            S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
            S::Error: Send,
            S::Future: Send,
            F: FnOnce($($ty),*, S::Error) -> Fut + Clone + Send + 'static,
            Fut: Future<Output = Res> + Send,
            Res: IntoResponse,
            $( $ty: FromRequest<ReqBody> + Send,)*
            ReqBody: Send + 'static,
            ResBody: HttpBody<Data = Bytes> + Send + 'static,
            ResBody::Error: Into<BoxError>,
        {
            type Response = Response;
            type Error = Infallible;

            type Future = future::HandleErrorFuture;

            fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }

            #[allow(non_snake_case)]
            fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
                let f = self.f.clone();

                let clone = self.inner.clone();
                let inner = std::mem::replace(&mut self.inner, clone);

                let future = Box::pin(async move {
                    let mut req = RequestParts::new(req);

                    $(
                        let $ty = match $ty::from_request(&mut req).await {
                            Ok(value) => value,
                            Err(rejection) => return Ok(rejection.into_response().map(boxed)),
                        };
                    )*

                    let req = match req.try_into_request() {
                        Ok(req) => req,
                        Err(err) => {
                            return Ok((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response());
                        }
                    };

                    match inner.oneshot(req).await {
                        Ok(res) => Ok(res.map(boxed)),
                        Err(err) => Ok(f($($ty),*, err).await.into_response().map(boxed)),
                    }
                });

                future::HandleErrorFuture { future }
            }
        }
    }
}

all_the_tuples!(impl_service);

pub mod future {
    //! Future types.

    use crate::response::Response;
    use pin_project_lite::pin_project;
    use std::{
        convert::Infallible,
        future::Future,
        pin::Pin,
        task::{Context, Poll},
    };

    pin_project! {
        /// Response future for [`HandleError`].
        pub struct HandleErrorFuture {
            #[pin]
            pub(super) future: Pin<Box<dyn Future<Output = Result<Response, Infallible>>
                + Send
                + 'static
            >>,
        }
    }

    impl Future for HandleErrorFuture {
        type Output = Result<Response, Infallible>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            self.project().future.poll(cx)
        }
    }
}

#[test]
fn traits() {
    use crate::test_helpers::*;

    assert_send::<HandleError<(), (), NotSendSync>>();
    assert_sync::<HandleError<(), (), NotSendSync>>();
}
