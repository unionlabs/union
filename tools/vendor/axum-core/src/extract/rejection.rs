//! Rejection response types.

use crate::{
    response::{IntoResponse, Response},
    BoxError,
};
use http::StatusCode;
use std::fmt;

/// Rejection type used if you try and extract the request body more than
/// once.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct BodyAlreadyExtracted;

impl BodyAlreadyExtracted {
    const BODY: &'static str = "Cannot have two request body extractors for a single handler";
}

impl IntoResponse for BodyAlreadyExtracted {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Self::BODY).into_response()
    }
}

impl fmt::Display for BodyAlreadyExtracted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::BODY)
    }
}

impl std::error::Error for BodyAlreadyExtracted {}

composite_rejection! {
    /// Rejection type for extractors that buffer the request body. Used if the
    /// request body cannot be buffered due to an error.
    pub enum FailedToBufferBody {
        LengthLimitError,
        UnknownBodyError,
    }
}

impl FailedToBufferBody {
    pub(crate) fn from_err<E>(err: E) -> Self
    where
        E: Into<BoxError>,
    {
        match err.into().downcast::<http_body::LengthLimitError>() {
            Ok(err) => Self::LengthLimitError(LengthLimitError::from_err(err)),
            Err(err) => Self::UnknownBodyError(UnknownBodyError::from_err(err)),
        }
    }
}

define_rejection! {
    #[status = PAYLOAD_TOO_LARGE]
    #[body = "Failed to buffer the request body"]
    /// Encountered some other error when buffering the body.
    ///
    /// This can  _only_ happen when you're using [`tower_http::limit::RequestBodyLimitLayer`] or
    /// otherwise wrapping request bodies in [`http_body::Limited`].
    ///
    /// [`tower_http::limit::RequestBodyLimitLayer`]: https://docs.rs/tower-http/0.3/tower_http/limit/struct.RequestBodyLimitLayer.html
    pub struct LengthLimitError(Error);
}

define_rejection! {
    #[status = BAD_REQUEST]
    #[body = "Failed to buffer the request body"]
    /// Encountered an unknown error when buffering the body.
    pub struct UnknownBodyError(Error);
}

define_rejection! {
    #[status = BAD_REQUEST]
    #[body = "Request body didn't contain valid UTF-8"]
    /// Rejection type used when buffering the request into a [`String`] if the
    /// body doesn't contain valid UTF-8.
    pub struct InvalidUtf8(Error);
}

composite_rejection! {
    /// Rejection used for [`Bytes`](bytes::Bytes).
    ///
    /// Contains one variant for each way the [`Bytes`](bytes::Bytes) extractor
    /// can fail.
    pub enum BytesRejection {
        BodyAlreadyExtracted,
        FailedToBufferBody,
    }
}

composite_rejection! {
    /// Rejection used for [`String`].
    ///
    /// Contains one variant for each way the [`String`] extractor can fail.
    pub enum StringRejection {
        BodyAlreadyExtracted,
        FailedToBufferBody,
        InvalidUtf8,
    }
}
