use super::{default_body_limit::DefaultBodyLimitKind, rejection::*, FromRequest, RequestParts};
use crate::BoxError;
use async_trait::async_trait;
use bytes::Bytes;
use http::{Extensions, HeaderMap, Method, Request, Uri, Version};
use std::convert::Infallible;

#[async_trait]
impl<B> FromRequest<B> for Request<B>
where
    B: Send,
{
    type Rejection = BodyAlreadyExtracted;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let req = std::mem::replace(
            req,
            RequestParts {
                method: req.method.clone(),
                version: req.version,
                uri: req.uri.clone(),
                headers: HeaderMap::new(),
                extensions: Extensions::default(),
                body: None,
            },
        );

        req.try_into_request()
    }
}

#[async_trait]
impl<B> FromRequest<B> for Method
where
    B: Send,
{
    type Rejection = Infallible;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        Ok(req.method().clone())
    }
}

#[async_trait]
impl<B> FromRequest<B> for Uri
where
    B: Send,
{
    type Rejection = Infallible;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        Ok(req.uri().clone())
    }
}

#[async_trait]
impl<B> FromRequest<B> for Version
where
    B: Send,
{
    type Rejection = Infallible;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        Ok(req.version())
    }
}

/// Clone the headers from the request.
///
/// Prefer using [`TypedHeader`] to extract only the headers you need.
///
/// [`TypedHeader`]: https://docs.rs/axum/latest/axum/extract/struct.TypedHeader.html
#[async_trait]
impl<B> FromRequest<B> for HeaderMap
where
    B: Send,
{
    type Rejection = Infallible;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        Ok(req.headers().clone())
    }
}

#[async_trait]
impl<B> FromRequest<B> for Bytes
where
    B: http_body::Body + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = BytesRejection;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        // update docs in `axum-core/src/extract/default_body_limit.rs` and
        // `axum/src/docs/extract.md` if this changes
        const DEFAULT_LIMIT: usize = 2_097_152; // 2 mb

        let body = take_body(req)?;

        let limit_kind = req.extensions().get::<DefaultBodyLimitKind>().copied();
        let bytes = match limit_kind {
            Some(DefaultBodyLimitKind::Disable) => crate::body::to_bytes(body)
                .await
                .map_err(FailedToBufferBody::from_err)?,
            Some(DefaultBodyLimitKind::Limit(limit)) => {
                let body = http_body::Limited::new(body, limit);
                crate::body::to_bytes(body)
                    .await
                    .map_err(FailedToBufferBody::from_err)?
            }
            None => {
                let body = http_body::Limited::new(body, DEFAULT_LIMIT);
                crate::body::to_bytes(body)
                    .await
                    .map_err(FailedToBufferBody::from_err)?
            }
        };

        Ok(bytes)
    }
}

#[async_trait]
impl<B> FromRequest<B> for String
where
    B: http_body::Body + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = StringRejection;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req).await.map_err(|err| match err {
            BytesRejection::FailedToBufferBody(inner) => StringRejection::FailedToBufferBody(inner),
            BytesRejection::BodyAlreadyExtracted(inner) => {
                StringRejection::BodyAlreadyExtracted(inner)
            }
        })?;

        let string = std::str::from_utf8(&bytes)
            .map_err(InvalidUtf8::from_err)?
            .to_owned();

        Ok(string)
    }
}

#[async_trait]
impl<B> FromRequest<B> for http::request::Parts
where
    B: Send,
{
    type Rejection = Infallible;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let method = unwrap_infallible(Method::from_request(req).await);
        let uri = unwrap_infallible(Uri::from_request(req).await);
        let version = unwrap_infallible(Version::from_request(req).await);
        let headers = unwrap_infallible(HeaderMap::from_request(req).await);
        let extensions = std::mem::take(req.extensions_mut());

        let mut temp_request = Request::new(());
        *temp_request.method_mut() = method;
        *temp_request.uri_mut() = uri;
        *temp_request.version_mut() = version;
        *temp_request.headers_mut() = headers;
        *temp_request.extensions_mut() = extensions;

        let (parts, _) = temp_request.into_parts();

        Ok(parts)
    }
}

fn unwrap_infallible<T>(result: Result<T, Infallible>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => match err {},
    }
}

pub(crate) fn take_body<B>(req: &mut RequestParts<B>) -> Result<B, BodyAlreadyExtracted> {
    req.take_body().ok_or(BodyAlreadyExtracted)
}
