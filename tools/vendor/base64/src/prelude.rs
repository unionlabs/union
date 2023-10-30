//! Preconfigured engines for common use cases.
//!
//! These are re-exports of `const` engines in [crate::engine::general_purpose], renamed with a `BASE64_`
//! prefix for those who prefer to `use` the entire path to a name.
//!
//! # Examples
//!
//! ```
//! use base64::prelude::{Engine as _, BASE64_STANDARD_NO_PAD};
//!
//! assert_eq!("c29tZSBieXRlcw", &BASE64_STANDARD_NO_PAD.encode(b"some bytes"));
//! ```

pub use crate::engine::Engine;

pub use crate::engine::general_purpose::STANDARD as BASE64_STANDARD;
pub use crate::engine::general_purpose::STANDARD_NO_PAD as BASE64_STANDARD_NO_PAD;
pub use crate::engine::general_purpose::URL_SAFE as BASE64_URL_SAFE;
pub use crate::engine::general_purpose::URL_SAFE_NO_PAD as BASE64_URL_SAFE_NO_PAD;
