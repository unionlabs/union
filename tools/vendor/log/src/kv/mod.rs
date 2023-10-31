//! **UNSTABLE:** Structured key-value pairs.
//!
//! This module is unstable and breaking changes may be made
//! at any time. See [the tracking issue](https://github.com/rust-lang-nursery/log/issues/328)
//! for more details.
//!
//! Add the `kv_unstable` feature to your `Cargo.toml` to enable
//! this module:
//!
//! ```toml
//! [dependencies.log]
//! features = ["kv_unstable"]
//! ```

mod error;
mod key;
pub mod source;

pub mod value;

pub use self::error::Error;
pub use self::key::{Key, ToKey};
pub use self::source::{Source, Visitor};

#[doc(inline)]
pub use self::value::{ToValue, Value};
