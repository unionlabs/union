//! Utilities for writing middleware
//!
#![doc = include_str!("../docs/middleware.md")]

mod from_extractor;
mod from_fn;

pub use self::from_extractor::{from_extractor, FromExtractor, FromExtractorLayer};
pub use self::from_fn::{from_fn, FromFn, FromFnLayer, Next};
pub use crate::extension::AddExtension;

pub mod future {
    //! Future types.

    pub use super::from_extractor::ResponseFuture as FromExtractorResponseFuture;
    pub use super::from_fn::ResponseFuture as FromFnResponseFuture;
}
