//! This module implements deferred display helpers.
//!
//! These are particularly useful in logging contexts, where the maximum logging level filter might
//! be enabled, but we don't want the arguments to be evaluated early:
//!
//! ```
//! log::set_max_level(log::LevelFilter::max());
//! fn expensive_calculation() -> String {
//!   "a string that is very slow to generate".into()
//! }
//! log::debug!("{}", expensive_calculation());
//! ```
//!
//! If the associated log implementation filters out log debug entries, the expensive calculation
//! would have been spurious. In this case, we can wrap the expensive computation within an
//! `DeferredDisplay`, so that the computation only happens when the actual `fmt` function is
//! called.

use core::fmt;

pub(crate) struct DeferredDisplay<F>(F);

impl<F: Fn() -> T, T: fmt::Display> DeferredDisplay<F> {
    pub(crate) fn new(f: F) -> Self {
        Self(f)
    }
}

impl<F: Fn() -> T, T: fmt::Display> fmt::Display for DeferredDisplay<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0().fmt(f)
    }
}

impl<F: Fn() -> T, T: fmt::Debug> fmt::Debug for DeferredDisplay<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0().fmt(f)
    }
}
