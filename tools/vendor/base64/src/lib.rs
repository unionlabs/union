//! # Getting started
//!
//! 1. Perhaps one of the preconfigured engines in [engine::general_purpose] will suit, e.g.
//! [engine::general_purpose::STANDARD_NO_PAD].
//!     - These are re-exported in [prelude] with a `BASE64_` prefix for those who prefer to
//!       `use base64::prelude::*` or equivalent, e.g. [prelude::BASE64_STANDARD_NO_PAD]
//! 1. If not, choose which alphabet you want. Most usage will want [alphabet::STANDARD] or [alphabet::URL_SAFE].
//! 1. Choose which [Engine] implementation you want. For the moment there is only one: [engine::GeneralPurpose].
//! 1. Configure the engine appropriately using the engine's `Config` type.
//!     - This is where you'll select whether to add padding (when encoding) or expect it (when
//!     decoding). If given the choice, prefer no padding.
//! 1. Build the engine using the selected alphabet and config.
//!
//! For more detail, see below.
//!
//! ## Alphabets
//!
//! An [alphabet::Alphabet] defines what ASCII symbols are used to encode to or decode from.
//!
//! Constants in [alphabet] like [alphabet::STANDARD] or [alphabet::URL_SAFE] provide commonly used
//! alphabets, but you can also build your own custom [alphabet::Alphabet] if needed.
//!
//! ## Engines
//!
//! Once you have an `Alphabet`, you can pick which `Engine` you want. A few parts of the public
//! API provide a default, but otherwise the user must provide an `Engine` to use.
//!
//! See [Engine] for more.
//!
//! ## Config
//!
//! In addition to an `Alphabet`, constructing an `Engine` also requires an [engine::Config]. Each
//! `Engine` has a corresponding `Config` implementation since different `Engine`s may offer different
//! levels of configurability.
//!
//! # Encoding
//!
//! Several different encoding methods on [Engine] are available to you depending on your desire for
//! convenience vs performance.
//!
//! | Method                   | Output                       | Allocates                      |
//! | ------------------------ | ---------------------------- | ------------------------------ |
//! | [Engine::encode]         | Returns a new `String`       | Always                         |
//! | [Engine::encode_string]  | Appends to provided `String` | Only if `String` needs to grow |
//! | [Engine::encode_slice]   | Writes to provided `&[u8]`   | Never - fastest                |
//!
//! All of the encoding methods will pad as per the engine's config.
//!
//! # Decoding
//!
//! Just as for encoding, there are different decoding methods available.
//!
//! | Method                   | Output                        | Allocates                      |
//! | ------------------------ | ----------------------------- | ------------------------------ |
//! | [Engine::decode]         | Returns a new `Vec<u8>`       | Always                         |
//! | [Engine::decode_vec]     | Appends to provided `Vec<u8>` | Only if `Vec` needs to grow    |
//! | [Engine::decode_slice]   | Writes to provided `&[u8]`    | Never - fastest                |
//!
//! Unlike encoding, where all possible input is valid, decoding can fail (see [DecodeError]).
//!
//! Input can be invalid because it has invalid characters or invalid padding. The nature of how
//! padding is checked depends on the engine's config.
//! Whitespace in the input is invalid, just like any other non-base64 byte.
//!
//! # `Read` and `Write`
//!
//! To decode a [std::io::Read] of b64 bytes, wrap a reader (file, network socket, etc) with
//! [read::DecoderReader].
//!
//! To write raw bytes and have them b64 encoded on the fly, wrap a [std::io::Write] with
//! [write::EncoderWriter].
//!
//! There is some performance overhead (15% or so) because of the necessary buffer shuffling --
//! still fast enough that almost nobody cares. Also, these implementations do not heap allocate.
//!
//! # `Display`
//!
//! See [display] for how to transparently base64 data via a `Display` implementation.
//!
//! # Examples
//!
//! ## Using predefined engines
//!
//! ```
//! use base64::{Engine as _, engine::general_purpose};
//!
//! let orig = b"data";
//! let encoded: String = general_purpose::STANDARD_NO_PAD.encode(orig);
//! assert_eq!("ZGF0YQ", encoded);
//! assert_eq!(orig.as_slice(), &general_purpose::STANDARD_NO_PAD.decode(encoded).unwrap());
//!
//! // or, URL-safe
//! let encoded_url = general_purpose::URL_SAFE_NO_PAD.encode(orig);
//! ```
//!
//! ## Custom alphabet, config, and engine
//!
//! ```
//! use base64::{engine, alphabet, Engine as _};
//!
//! // bizarro-world base64: +/ as the first symbols instead of the last
//! let alphabet =
//!     alphabet::Alphabet::new("+/ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789")
//!     .unwrap();
//!
//! // a very weird config that encodes with padding but requires no padding when decoding...?
//! let crazy_config = engine::GeneralPurposeConfig::new()
//!     .with_decode_allow_trailing_bits(true)
//!     .with_encode_padding(true)
//!     .with_decode_padding_mode(engine::DecodePaddingMode::RequireNone);
//!
//! let crazy_engine = engine::GeneralPurpose::new(&alphabet, crazy_config);
//!
//! let encoded = crazy_engine.encode(b"abc 123");
//!
//! ```
//!
//! # Panics
//!
//! If length calculations result in overflowing `usize`, a panic will result.

#![cfg_attr(feature = "cargo-clippy", allow(clippy::cast_lossless))]
#![deny(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_results,
    variant_size_differences,
    warnings
)]
#![forbid(unsafe_code)]
// Allow globally until https://github.com/rust-lang/rust-clippy/issues/8768 is resolved.
// The desired state is to allow it only for the rstest_reuse import.
#![allow(clippy::single_component_path_imports)]
#![cfg_attr(not(any(feature = "std", test)), no_std)]

#[cfg(all(feature = "alloc", not(any(feature = "std", test))))]
extern crate alloc;
#[cfg(any(feature = "std", test))]
extern crate std as alloc;

// has to be included at top level because of the way rstest_reuse defines its macros
#[cfg(test)]
use rstest_reuse;

mod chunked_encoder;
pub mod display;
#[cfg(any(feature = "std", test))]
pub mod read;
#[cfg(any(feature = "std", test))]
pub mod write;

pub mod engine;
pub use engine::Engine;

pub mod alphabet;

mod encode;
#[allow(deprecated)]
#[cfg(any(feature = "alloc", feature = "std", test))]
pub use crate::encode::{encode, encode_engine, encode_engine_string};
#[allow(deprecated)]
pub use crate::encode::{encode_engine_slice, encoded_len, EncodeSliceError};

mod decode;
#[allow(deprecated)]
#[cfg(any(feature = "alloc", feature = "std", test))]
pub use crate::decode::{decode, decode_engine, decode_engine_vec};
#[allow(deprecated)]
pub use crate::decode::{decode_engine_slice, decoded_len_estimate, DecodeError, DecodeSliceError};

pub mod prelude;

#[cfg(test)]
mod tests;

const PAD_BYTE: u8 = b'=';
