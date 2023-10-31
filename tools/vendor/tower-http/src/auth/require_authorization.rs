//! Authorize requests using the [`Authorization`] header.
//!
//! [`Authorization`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Authorization
//!
//! # Example
//!
//! ```
//! use tower_http::auth::RequireAuthorizationLayer;
//! use hyper::{Request, Response, Body, Error};
//! use http::{StatusCode, header::AUTHORIZATION};
//! use tower::{Service, ServiceExt, ServiceBuilder, service_fn};
//!
//! async fn handle(request: Request<Body>) -> Result<Response<Body>, Error> {
//!     Ok(Response::new(Body::empty()))
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut service = ServiceBuilder::new()
//!     // Require the `Authorization` header to be `Bearer passwordlol`
//!     .layer(RequireAuthorizationLayer::bearer("passwordlol"))
//!     .service_fn(handle);
//!
//! // Requests with the correct token are allowed through
//! let request = Request::builder()
//!     .header(AUTHORIZATION, "Bearer passwordlol")
//!     .body(Body::empty())
//!     .unwrap();
//!
//! let response = service
//!     .ready()
//!     .await?
//!     .call(request)
//!     .await?;
//!
//! assert_eq!(StatusCode::OK, response.status());
//!
//! // Requests with an invalid token get a `401 Unauthorized` response
//! let request = Request::builder()
//!     .body(Body::empty())
//!     .unwrap();
//!
//! let response = service
//!     .ready()
//!     .await?
//!     .call(request)
//!     .await?;
//!
//! assert_eq!(StatusCode::UNAUTHORIZED, response.status());
//! # Ok(())
//! # }
//! ```
//!
//! Custom authorization schemes can be made by implementing [`AuthorizeRequest`]:
//!
//! ```
//! use tower_http::auth::{RequireAuthorizationLayer, AuthorizeRequest};
//! use hyper::{Request, Response, Body, Error};
//! use http::{StatusCode, header::AUTHORIZATION};
//! use tower::{Service, ServiceExt, ServiceBuilder, service_fn};
//!
//! #[derive(Clone, Copy)]
//! struct MyAuth;
//!
//! impl<B> AuthorizeRequest<B> for MyAuth {
//!     type ResponseBody = Body;
//!
//!     fn authorize(
//!         &mut self,
//!         request: &mut Request<B>,
//!     ) -> Result<(), Response<Self::ResponseBody>> {
//!         if let Some(user_id) = check_auth(request) {
//!             // Set `user_id` as a request extension so it can be accessed by other
//!             // services down the stack.
//!             request.extensions_mut().insert(user_id);
//!
//!             Ok(())
//!         } else {
//!             let unauthorized_response = Response::builder()
//!                 .status(StatusCode::UNAUTHORIZED)
//!                 .body(Body::empty())
//!                 .unwrap();
//!
//!             Err(unauthorized_response)
//!         }
//!     }
//! }
//!
//! fn check_auth<B>(request: &Request<B>) -> Option<UserId> {
//!     // ...
//!     # unimplemented!()
//! }
//!
//! #[derive(Debug)]
//! struct UserId(String);
//!
//! async fn handle(request: Request<Body>) -> Result<Response<Body>, Error> {
//!     // Access the `UserId` that was set in `on_authorized`. If `handle` gets called the
//!     // request was authorized and `UserId` will be present.
//!     let user_id = request
//!         .extensions()
//!         .get::<UserId>()
//!         .expect("UserId will be there if request was authorized");
//!
//!     println!("request from {:?}", user_id);
//!
//!     Ok(Response::new(Body::empty()))
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let service = ServiceBuilder::new()
//!     // Authorize requests using `MyAuth`
//!     .layer(RequireAuthorizationLayer::custom(MyAuth))
//!     .service_fn(handle);
//! # Ok(())
//! # }
//! ```
//!
//! Or using a closure:
//!
//! ```
//! use tower_http::auth::{RequireAuthorizationLayer, AuthorizeRequest};
//! use hyper::{Request, Response, Body, Error};
//! use http::{StatusCode, header::AUTHORIZATION};
//! use tower::{Service, ServiceExt, ServiceBuilder, service_fn};
//!
//! async fn handle(request: Request<Body>) -> Result<Response<Body>, Error> {
//!     # todo!();
//!     // ...
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let service = ServiceBuilder::new()
//!     .layer(RequireAuthorizationLayer::custom(|request: &mut Request<Body>| {
//!         // authorize the request
//!         # Ok::<_, Response<Body>>(())
//!     }))
//!     .service_fn(handle);
//! # Ok(())
//! # }
//! ```

use http::{
    header::{self, HeaderValue},
    Request, Response, StatusCode,
};
use http_body::Body;
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

/// Layer that applies [`RequireAuthorization`] which authorizes all requests using the
/// [`Authorization`] header.
///
/// See the [module docs](crate::auth::require_authorization) for an example.
///
/// [`Authorization`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Authorization
#[derive(Debug, Clone)]
pub struct RequireAuthorizationLayer<T> {
    auth: T,
}

impl<ResBody> RequireAuthorizationLayer<Bearer<ResBody>> {
    /// Authorize requests using a "bearer token". Commonly used for OAuth 2.
    ///
    /// The `Authorization` header is required to be `Bearer {token}`.
    ///
    /// # Panics
    ///
    /// Panics if the token is not a valid [`HeaderValue`](http::header::HeaderValue).
    pub fn bearer(token: &str) -> Self
    where
        ResBody: Body + Default,
    {
        Self::custom(Bearer::new(token))
    }
}

impl<ResBody> RequireAuthorizationLayer<Basic<ResBody>> {
    /// Authorize requests using a username and password pair.
    ///
    /// The `Authorization` header is required to be `Basic {credentials}` where `credentials` is
    /// `base64_encode("{username}:{password}")`.
    ///
    /// Since the username and password is sent in clear text it is recommended to use HTTPS/TLS
    /// with this method. However use of HTTPS/TLS is not enforced by this middleware.
    pub fn basic(username: &str, password: &str) -> Self
    where
        ResBody: Body + Default,
    {
        Self::custom(Basic::new(username, password))
    }
}

impl<T> RequireAuthorizationLayer<T> {
    /// Authorize requests using a custom scheme.
    pub fn custom(auth: T) -> RequireAuthorizationLayer<T> {
        Self { auth }
    }
}

impl<S, T> Layer<S> for RequireAuthorizationLayer<T>
where
    T: Clone,
{
    type Service = RequireAuthorization<S, T>;

    fn layer(&self, inner: S) -> Self::Service {
        RequireAuthorization::new(inner, self.auth.clone())
    }
}

/// Middleware that authorizes all requests using the [`Authorization`] header.
///
/// See the [module docs](crate::auth::require_authorization) for an example.
///
/// [`Authorization`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Authorization
#[derive(Clone, Debug)]
pub struct RequireAuthorization<S, T> {
    inner: S,
    auth: T,
}

impl<S, T> RequireAuthorization<S, T> {
    fn new(inner: S, auth: T) -> Self {
        Self { inner, auth }
    }

    define_inner_service_accessors!();
}

impl<S, ResBody> RequireAuthorization<S, Bearer<ResBody>> {
    /// Authorize requests using a "bearer token". Commonly used for OAuth 2.
    ///
    /// The `Authorization` header is required to be `Bearer {token}`.
    ///
    /// # Panics
    ///
    /// Panics if the token is not a valid [`HeaderValue`](http::header::HeaderValue).
    pub fn bearer(inner: S, token: &str) -> Self
    where
        ResBody: Body + Default,
    {
        Self::custom(inner, Bearer::new(token))
    }
}

impl<S, ResBody> RequireAuthorization<S, Basic<ResBody>> {
    /// Authorize requests using a username and password pair.
    ///
    /// The `Authorization` header is required to be `Basic {credentials}` where `credentials` is
    /// `base64_encode("{username}:{password}")`.
    ///
    /// Since the username and password is sent in clear text it is recommended to use HTTPS/TLS
    /// with this method. However use of HTTPS/TLS is not enforced by this middleware.
    pub fn basic(inner: S, username: &str, password: &str) -> Self
    where
        ResBody: Body + Default,
    {
        Self::custom(inner, Basic::new(username, password))
    }
}

impl<S, T> RequireAuthorization<S, T> {
    /// Authorize requests using a custom scheme.
    ///
    /// The `Authorization` header is required to have the value provided.
    pub fn custom(inner: S, auth: T) -> RequireAuthorization<S, T> {
        Self { inner, auth }
    }
}

impl<ReqBody, ResBody, S, Auth> Service<Request<ReqBody>> for RequireAuthorization<S, Auth>
where
    Auth: AuthorizeRequest<ReqBody, ResponseBody = ResBody>,
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = Response<ResBody>;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future, ResBody>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        match self.auth.authorize(&mut req) {
            Ok(_) => ResponseFuture::future(self.inner.call(req)),
            Err(res) => ResponseFuture::invalid_auth(res),
        }
    }
}

pin_project! {
    /// Response future for [`RequireAuthorization`].
    pub struct ResponseFuture<F, B> {
        #[pin]
        kind: Kind<F, B>,
    }
}

impl<F, B> ResponseFuture<F, B> {
    fn future(future: F) -> Self {
        Self {
            kind: Kind::Future { future },
        }
    }

    fn invalid_auth(res: Response<B>) -> Self {
        Self {
            kind: Kind::Error {
                response: Some(res),
            },
        }
    }
}

pin_project! {
    #[project = KindProj]
    enum Kind<F, B> {
        Future {
            #[pin]
            future: F,
        },
        Error {
            response: Option<Response<B>>,
        },
    }
}

impl<F, B, E> Future for ResponseFuture<F, B>
where
    F: Future<Output = Result<Response<B>, E>>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project().kind.project() {
            KindProj::Future { future } => future.poll(cx),
            KindProj::Error { response } => {
                let response = response.take().unwrap();
                Poll::Ready(Ok(response))
            }
        }
    }
}

/// Trait for authorizing requests.
pub trait AuthorizeRequest<B> {
    /// The body type used for responses to unauthorized requests.
    type ResponseBody;

    /// Authorize the request.
    ///
    /// If `Ok(())` is returned then the request is allowed through, otherwise not.
    fn authorize(&mut self, request: &mut Request<B>) -> Result<(), Response<Self::ResponseBody>>;
}

impl<B, F, ResBody> AuthorizeRequest<B> for F
where
    F: FnMut(&mut Request<B>) -> Result<(), Response<ResBody>>,
{
    type ResponseBody = ResBody;

    fn authorize(&mut self, request: &mut Request<B>) -> Result<(), Response<Self::ResponseBody>> {
        self(request)
    }
}

/// Type that performs "bearer token" authorization.
///
/// See [`RequireAuthorization::bearer`] for more details.
pub struct Bearer<ResBody> {
    header_value: HeaderValue,
    _ty: PhantomData<fn() -> ResBody>,
}

impl<ResBody> Bearer<ResBody> {
    fn new(token: &str) -> Self
    where
        ResBody: Body + Default,
    {
        Self {
            header_value: format!("Bearer {}", token)
                .parse()
                .expect("token is not a valid header value"),
            _ty: PhantomData,
        }
    }
}

impl<ResBody> Clone for Bearer<ResBody> {
    fn clone(&self) -> Self {
        Self {
            header_value: self.header_value.clone(),
            _ty: PhantomData,
        }
    }
}

impl<ResBody> fmt::Debug for Bearer<ResBody> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bearer")
            .field("header_value", &self.header_value)
            .finish()
    }
}

impl<B, ResBody> AuthorizeRequest<B> for Bearer<ResBody>
where
    ResBody: Body + Default,
{
    type ResponseBody = ResBody;

    fn authorize(&mut self, request: &mut Request<B>) -> Result<(), Response<Self::ResponseBody>> {
        match request.headers().get(header::AUTHORIZATION) {
            Some(actual) if actual == self.header_value => Ok(()),
            _ => {
                let mut res = Response::new(ResBody::default());
                *res.status_mut() = StatusCode::UNAUTHORIZED;
                Err(res)
            }
        }
    }
}

/// Type that performs basic authorization.
///
/// See [`RequireAuthorization::basic`] for more details.
pub struct Basic<ResBody> {
    header_value: HeaderValue,
    _ty: PhantomData<fn() -> ResBody>,
}

impl<ResBody> Basic<ResBody> {
    fn new(username: &str, password: &str) -> Self
    where
        ResBody: Body + Default,
    {
        let encoded = base64::encode(format!("{}:{}", username, password));
        let header_value = format!("Basic {}", encoded).parse().unwrap();
        Self {
            header_value,
            _ty: PhantomData,
        }
    }
}

impl<ResBody> Clone for Basic<ResBody> {
    fn clone(&self) -> Self {
        Self {
            header_value: self.header_value.clone(),
            _ty: PhantomData,
        }
    }
}

impl<ResBody> fmt::Debug for Basic<ResBody> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Basic")
            .field("header_value", &self.header_value)
            .finish()
    }
}

impl<B, ResBody> AuthorizeRequest<B> for Basic<ResBody>
where
    ResBody: Body + Default,
{
    type ResponseBody = ResBody;

    fn authorize(&mut self, request: &mut Request<B>) -> Result<(), Response<Self::ResponseBody>> {
        match request.headers().get(header::AUTHORIZATION) {
            Some(actual) if actual == self.header_value => Ok(()),
            _ => {
                let mut res = Response::new(ResBody::default());
                *res.status_mut() = StatusCode::UNAUTHORIZED;
                res.headers_mut()
                    .insert(header::WWW_AUTHENTICATE, "Basic".parse().unwrap());
                Err(res)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use http::header;
    use hyper::Body;
    use tower::{BoxError, ServiceBuilder, ServiceExt};

    #[tokio::test]
    async fn valid_basic_token() {
        let mut service = ServiceBuilder::new()
            .layer(RequireAuthorizationLayer::basic("foo", "bar"))
            .service_fn(echo);

        let request = Request::get("/")
            .header(
                header::AUTHORIZATION,
                format!("Basic {}", base64::encode("foo:bar")),
            )
            .body(Body::empty())
            .unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn invalid_basic_token() {
        let mut service = ServiceBuilder::new()
            .layer(RequireAuthorizationLayer::basic("foo", "bar"))
            .service_fn(echo);

        let request = Request::get("/")
            .header(
                header::AUTHORIZATION,
                format!("Basic {}", base64::encode("wrong:credentials")),
            )
            .body(Body::empty())
            .unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        let www_authenticate = res.headers().get(header::WWW_AUTHENTICATE).unwrap();
        assert_eq!(www_authenticate, "Basic");
    }

    #[tokio::test]
    async fn valid_bearer_token() {
        let mut service = ServiceBuilder::new()
            .layer(RequireAuthorizationLayer::bearer("foobar"))
            .service_fn(echo);

        let request = Request::get("/")
            .header(header::AUTHORIZATION, "Bearer foobar")
            .body(Body::empty())
            .unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn basic_auth_is_case_sensitive_in_prefix() {
        let mut service = ServiceBuilder::new()
            .layer(RequireAuthorizationLayer::basic("foo", "bar"))
            .service_fn(echo);

        let request = Request::get("/")
            .header(
                header::AUTHORIZATION,
                format!("basic {}", base64::encode("foo:bar")),
            )
            .body(Body::empty())
            .unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn basic_auth_is_case_sensitive_in_value() {
        let mut service = ServiceBuilder::new()
            .layer(RequireAuthorizationLayer::basic("foo", "bar"))
            .service_fn(echo);

        let request = Request::get("/")
            .header(
                header::AUTHORIZATION,
                format!("Basic {}", base64::encode("Foo:bar")),
            )
            .body(Body::empty())
            .unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn invalid_bearer_token() {
        let mut service = ServiceBuilder::new()
            .layer(RequireAuthorizationLayer::bearer("foobar"))
            .service_fn(echo);

        let request = Request::get("/")
            .header(header::AUTHORIZATION, "Bearer wat")
            .body(Body::empty())
            .unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn bearer_token_is_case_sensitive_in_prefix() {
        let mut service = ServiceBuilder::new()
            .layer(RequireAuthorizationLayer::bearer("foobar"))
            .service_fn(echo);

        let request = Request::get("/")
            .header(header::AUTHORIZATION, "bearer foobar")
            .body(Body::empty())
            .unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn bearer_token_is_case_sensitive_in_token() {
        let mut service = ServiceBuilder::new()
            .layer(RequireAuthorizationLayer::bearer("foobar"))
            .service_fn(echo);

        let request = Request::get("/")
            .header(header::AUTHORIZATION, "Bearer Foobar")
            .body(Body::empty())
            .unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    async fn echo(req: Request<Body>) -> Result<Response<Body>, BoxError> {
        Ok(Response::new(req.into_body()))
    }
}
