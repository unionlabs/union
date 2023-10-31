//! Convert an extractor into a middleware.
//!
//! See [`extractor_middleware`] for more details.

use crate::middleware::from_extractor;

pub use crate::middleware::{
    future::FromExtractorResponseFuture as ResponseFuture, FromExtractor as ExtractorMiddleware,
    FromExtractorLayer as ExtractorMiddlewareLayer,
};

/// Convert an extractor into a middleware.
#[deprecated(note = "Please use `axum::middleware::from_extractor` instead")]
pub fn extractor_middleware<E>() -> ExtractorMiddlewareLayer<E> {
    from_extractor()
}
