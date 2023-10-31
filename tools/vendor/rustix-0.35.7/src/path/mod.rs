//! Filesystem path operations.

mod arg;
#[cfg(feature = "itoa")]
mod dec_int;

pub use arg::Arg;
#[cfg(feature = "itoa")]
pub use dec_int::DecInt;

pub(crate) const SMALL_PATH_BUFFER_SIZE: usize = 256;
