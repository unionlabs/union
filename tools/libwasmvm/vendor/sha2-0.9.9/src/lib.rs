//! An implementation of the [SHA-2][1] cryptographic hash algorithms.
//!
//! There are 6 standard algorithms specified in the SHA-2 standard:
//!
//! * `Sha224`, which is the 8 x 32-bit `Sha256` algorithm with the result truncated
//! to 224 bits.
//! * `Sha256`, which is the 8 x 32-bit `Sha256` algorithm.
//! * `Sha384`, which is the 8 x 64-bit `Sha512` algorithm with the result truncated
//! to 384 bits.
//! * `Sha512`, which is the 8 x 64-bit `Sha512` algorithm.
//! * `Sha512Trunc224`, which is the 8 x 64-bit `Sha512` algorithm with the result
//! truncated to 224 bits.
//! * `Sha512Trunc256`, which is the 8 x 64-bit `Sha512` algorithm with the result
//! truncated to 256 bits.
//!
//! Algorithmically, there are only 2 core algorithms: `Sha256` and `Sha512`.
//! All other algorithms are just applications of these with different initial
//! hash values, and truncated to different digest bit lengths.
//!
//! # Usage
//!
//! ```rust
//! use hex_literal::hex;
//! use sha2::{Sha256, Sha512, Digest};
//!
//! // create a Sha256 object
//! let mut hasher = Sha256::new();
//!
//! // write input message
//! hasher.update(b"hello world");
//!
//! // read hash digest and consume hasher
//! let result = hasher.finalize();
//!
//! assert_eq!(result[..], hex!("
//!     b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9
//! ")[..]);
//!
//! // same for Sha512
//! let mut hasher = Sha512::new();
//! hasher.update(b"hello world");
//! let result = hasher.finalize();
//!
//! assert_eq!(result[..], hex!("
//!     309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f
//!     989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f
//! ")[..]);
//! ```
//!
//! Also see [RustCrypto/hashes][2] readme.
//!
//! [1]: https://en.wikipedia.org/wiki/SHA-2
//! [2]: https://github.com/RustCrypto/hashes

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg"
)]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(feature = "std")]
extern crate std;

mod consts;
mod sha256;
mod sha512;

pub use digest::{self, Digest};
#[cfg(feature = "compress")]
pub use sha256::compress256;
pub use sha256::{Sha224, Sha256};
#[cfg(feature = "compress")]
pub use sha512::compress512;
pub use sha512::{Sha384, Sha512, Sha512Trunc224, Sha512Trunc256};
