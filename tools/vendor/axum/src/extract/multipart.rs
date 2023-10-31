//! Extractor that parses `multipart/form-data` requests commonly used with file uploads.
//!
//! See [`Multipart`] for more details.

use super::{rejection::*, BodyStream, FromRequest, RequestParts};
use crate::body::{Bytes, HttpBody};
use crate::BoxError;
use async_trait::async_trait;
use futures_util::stream::Stream;
use http::header::{HeaderMap, CONTENT_TYPE};
use std::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
};

/// Extractor that parses `multipart/form-data` requests (commonly used with file uploads).
///
/// # Example
///
/// ```rust,no_run
/// use axum::{
///     extract::Multipart,
///     routing::post,
///     Router,
/// };
/// use futures::stream::StreamExt;
///
/// async fn upload(mut multipart: Multipart) {
///     while let Some(mut field) = multipart.next_field().await.unwrap() {
///         let name = field.name().unwrap().to_string();
///         let data = field.bytes().await.unwrap();
///
///         println!("Length of `{}` is {} bytes", name, data.len());
///     }
/// }
///
/// let app = Router::new().route("/upload", post(upload));
/// # async {
/// # axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
/// # };
/// ```
///
/// For security reasons it's recommended to combine this with
/// [`ContentLengthLimit`](super::ContentLengthLimit) to limit the size of the request payload.
#[cfg_attr(docsrs, doc(cfg(feature = "multipart")))]
#[derive(Debug)]
pub struct Multipart {
    inner: multer::Multipart<'static>,
}

#[async_trait]
impl<B> FromRequest<B> for Multipart
where
    B: HttpBody<Data = Bytes> + Send + 'static,
    B::Error: Into<BoxError>,
{
    type Rejection = MultipartRejection;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let stream = BodyStream::from_request(req).await?;
        let headers = req.headers();
        let boundary = parse_boundary(headers).ok_or(InvalidBoundary)?;
        let multipart = multer::Multipart::new(stream, boundary);
        Ok(Self { inner: multipart })
    }
}

impl Multipart {
    /// Yields the next [`Field`] if available.
    pub async fn next_field(&mut self) -> Result<Option<Field<'_>>, MultipartError> {
        let field = self
            .inner
            .next_field()
            .await
            .map_err(MultipartError::from_multer)?;

        if let Some(field) = field {
            Ok(Some(Field {
                inner: field,
                _multipart: self,
            }))
        } else {
            Ok(None)
        }
    }
}

/// A single field in a multipart stream.
#[derive(Debug)]
pub struct Field<'a> {
    inner: multer::Field<'static>,
    // multer requires there to only be one live `multer::Field` at any point. This enforces that
    // statically, which multer does not do, it returns an error instead.
    _multipart: &'a mut Multipart,
}

impl<'a> Stream for Field<'a> {
    type Item = Result<Bytes, MultipartError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner)
            .poll_next(cx)
            .map_err(MultipartError::from_multer)
    }
}

impl<'a> Field<'a> {
    /// The field name found in the
    /// [`Content-Disposition`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Disposition)
    /// header.
    pub fn name(&self) -> Option<&str> {
        self.inner.name()
    }

    /// The file name found in the
    /// [`Content-Disposition`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Disposition)
    /// header.
    pub fn file_name(&self) -> Option<&str> {
        self.inner.file_name()
    }

    /// Get the [content type](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Type) of the field.
    pub fn content_type(&self) -> Option<&str> {
        self.inner.content_type().map(|m| m.as_ref())
    }

    /// Get a map of headers as [`HeaderMap`].
    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    /// Get the full data of the field as [`Bytes`].
    pub async fn bytes(self) -> Result<Bytes, MultipartError> {
        self.inner
            .bytes()
            .await
            .map_err(MultipartError::from_multer)
    }

    /// Get the full field data as text.
    pub async fn text(self) -> Result<String, MultipartError> {
        self.inner.text().await.map_err(MultipartError::from_multer)
    }

    /// Stream a chunk of the field data.
    ///
    /// When the field data has been exhausted, this will return [`None`].
    ///
    /// Note this does the same thing as `Field`'s [`Stream`] implementation.
    ///
    /// # Example
    ///
    /// ```
    /// use axum::{
    ///    extract::Multipart,
    ///    routing::post,
    ///    response::IntoResponse,
    ///    http::StatusCode,
    ///    Router,
    /// };
    ///
    /// async fn upload(mut multipart: Multipart) -> Result<(), (StatusCode, String)> {
    ///     while let Some(mut field) = multipart
    ///         .next_field()
    ///         .await
    ///         .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?
    ///     {
    ///         while let Some(chunk) = field
    ///             .chunk()
    ///             .await
    ///             .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?
    ///         {
    ///             println!("received {} bytes", chunk.len());
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    ///
    /// let app = Router::new().route("/upload", post(upload));
    /// # let _: Router<axum::body::Body> = app;
    /// ```
    pub async fn chunk(&mut self) -> Result<Option<Bytes>, MultipartError> {
        self.inner
            .chunk()
            .await
            .map_err(MultipartError::from_multer)
    }
}

/// Errors associated with parsing `multipart/form-data` requests.
#[derive(Debug)]
pub struct MultipartError {
    source: multer::Error,
}

impl MultipartError {
    fn from_multer(multer: multer::Error) -> Self {
        Self { source: multer }
    }
}

impl fmt::Display for MultipartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error parsing `multipart/form-data` request")
    }
}

impl std::error::Error for MultipartError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

fn parse_boundary(headers: &HeaderMap) -> Option<String> {
    let content_type = headers.get(CONTENT_TYPE)?.to_str().ok()?;
    multer::parse_boundary(content_type).ok()
}

composite_rejection! {
    /// Rejection used for [`Multipart`].
    ///
    /// Contains one variant for each way the [`Multipart`] extractor can fail.
    pub enum MultipartRejection {
        BodyAlreadyExtracted,
        InvalidBoundary,
    }
}

define_rejection! {
    #[status = BAD_REQUEST]
    #[body = "Invalid `boundary` for `multipart/form-data` request"]
    /// Rejection type used if the `boundary` in a `multipart/form-data` is
    /// missing or invalid.
    pub struct InvalidBoundary;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{response::IntoResponse, routing::post, test_helpers::*, Router};

    #[tokio::test]
    async fn content_type_with_encoding() {
        const BYTES: &[u8] = "<!doctype html><title>🦀</title>".as_bytes();
        const FILE_NAME: &str = "index.html";
        const CONTENT_TYPE: &str = "text/html; charset=utf-8";

        async fn handle(mut multipart: Multipart) -> impl IntoResponse {
            let field = multipart.next_field().await.unwrap().unwrap();

            assert_eq!(field.file_name().unwrap(), FILE_NAME);
            assert_eq!(field.content_type().unwrap(), CONTENT_TYPE);
            assert_eq!(field.bytes().await.unwrap(), BYTES);

            assert!(multipart.next_field().await.unwrap().is_none());
        }

        let app = Router::new().route("/", post(handle));

        let client = TestClient::new(app);

        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(BYTES)
                .file_name(FILE_NAME)
                .mime_str(CONTENT_TYPE)
                .unwrap(),
        );

        client.post("/").multipart(form).send().await;
    }
}
