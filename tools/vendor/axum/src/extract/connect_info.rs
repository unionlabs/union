//! Extractor for getting connection information from a client.
//!
//! See [`Router::into_make_service_with_connect_info`] for more details.
//!
//! [`Router::into_make_service_with_connect_info`]: crate::routing::Router::into_make_service_with_connect_info

use super::{Extension, FromRequest, RequestParts};
use crate::middleware::AddExtension;
use async_trait::async_trait;
use hyper::server::conn::AddrStream;
use std::{
    convert::Infallible,
    fmt,
    future::ready,
    marker::PhantomData,
    net::SocketAddr,
    task::{Context, Poll},
};
use tower_layer::Layer;
use tower_service::Service;

/// A [`MakeService`] created from a router.
///
/// See [`Router::into_make_service_with_connect_info`] for more details.
///
/// [`MakeService`]: tower::make::MakeService
/// [`Router::into_make_service_with_connect_info`]: crate::routing::Router::into_make_service_with_connect_info
pub struct IntoMakeServiceWithConnectInfo<S, C> {
    svc: S,
    _connect_info: PhantomData<fn() -> C>,
}

#[test]
fn traits() {
    use crate::test_helpers::*;
    assert_send::<IntoMakeServiceWithConnectInfo<(), NotSendSync>>();
}

impl<S, C> IntoMakeServiceWithConnectInfo<S, C> {
    pub(crate) fn new(svc: S) -> Self {
        Self {
            svc,
            _connect_info: PhantomData,
        }
    }
}

impl<S, C> fmt::Debug for IntoMakeServiceWithConnectInfo<S, C>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntoMakeServiceWithConnectInfo")
            .field("svc", &self.svc)
            .finish()
    }
}

impl<S, C> Clone for IntoMakeServiceWithConnectInfo<S, C>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            svc: self.svc.clone(),
            _connect_info: PhantomData,
        }
    }
}

/// Trait that connected IO resources implement and use to produce information
/// about the connection.
///
/// The goal for this trait is to allow users to implement custom IO types that
/// can still provide the same connection metadata.
///
/// See [`Router::into_make_service_with_connect_info`] for more details.
///
/// [`Router::into_make_service_with_connect_info`]: crate::routing::Router::into_make_service_with_connect_info
pub trait Connected<T>: Clone + Send + Sync + 'static {
    /// Create type holding information about the connection.
    fn connect_info(target: T) -> Self;
}

impl Connected<&AddrStream> for SocketAddr {
    fn connect_info(target: &AddrStream) -> Self {
        target.remote_addr()
    }
}

impl<S, C, T> Service<T> for IntoMakeServiceWithConnectInfo<S, C>
where
    S: Clone,
    C: Connected<T>,
{
    type Response = AddExtension<S, ConnectInfo<C>>;
    type Error = Infallible;
    type Future = ResponseFuture<S, C>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, target: T) -> Self::Future {
        let connect_info = ConnectInfo(C::connect_info(target));
        let svc = Extension(connect_info).layer(self.svc.clone());
        ResponseFuture::new(ready(Ok(svc)))
    }
}

opaque_future! {
    /// Response future for [`IntoMakeServiceWithConnectInfo`].
    pub type ResponseFuture<S, C> =
        std::future::Ready<Result<AddExtension<S, ConnectInfo<C>>, Infallible>>;
}

/// Extractor for getting connection information produced by a [`Connected`].
///
/// Note this extractor requires you to use
/// [`Router::into_make_service_with_connect_info`] to run your app
/// otherwise it will fail at runtime.
///
/// See [`Router::into_make_service_with_connect_info`] for more details.
///
/// [`Router::into_make_service_with_connect_info`]: crate::routing::Router::into_make_service_with_connect_info
#[derive(Clone, Copy, Debug)]
pub struct ConnectInfo<T>(pub T);

#[async_trait]
impl<B, T> FromRequest<B> for ConnectInfo<T>
where
    B: Send,
    T: Clone + Send + Sync + 'static,
{
    type Rejection = <Extension<Self> as FromRequest<B>>::Rejection;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(connect_info) = Extension::<Self>::from_request(req).await?;
        Ok(connect_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{routing::get, Router, Server};
    use std::net::{SocketAddr, TcpListener};

    #[tokio::test]
    async fn socket_addr() {
        async fn handler(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> String {
            format!("{}", addr)
        }

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let (tx, rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let app = Router::new().route("/", get(handler));
            let server = Server::from_tcp(listener)
                .unwrap()
                .serve(app.into_make_service_with_connect_info::<SocketAddr>());
            tx.send(()).unwrap();
            server.await.expect("server error");
        });
        rx.await.unwrap();

        let client = reqwest::Client::new();

        let res = client.get(format!("http://{}", addr)).send().await.unwrap();
        let body = res.text().await.unwrap();
        assert!(body.starts_with("127.0.0.1:"));
    }

    #[tokio::test]
    async fn custom() {
        #[derive(Clone, Debug)]
        struct MyConnectInfo {
            value: &'static str,
        }

        impl Connected<&AddrStream> for MyConnectInfo {
            fn connect_info(_target: &AddrStream) -> Self {
                Self {
                    value: "it worked!",
                }
            }
        }

        async fn handler(ConnectInfo(addr): ConnectInfo<MyConnectInfo>) -> &'static str {
            addr.value
        }

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let (tx, rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let app = Router::new().route("/", get(handler));
            let server = Server::from_tcp(listener)
                .unwrap()
                .serve(app.into_make_service_with_connect_info::<MyConnectInfo>());
            tx.send(()).unwrap();
            server.await.expect("server error");
        });
        rx.await.unwrap();

        let client = reqwest::Client::new();

        let res = client.get(format!("http://{}", addr)).send().await.unwrap();
        let body = res.text().await.unwrap();
        assert_eq!(body, "it worked!");
    }
}
