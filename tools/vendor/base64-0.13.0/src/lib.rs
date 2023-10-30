//! # Configs
//!
//! There isn't just one type of Base64; that would be too simple. You need to choose a character
//! set (standard, URL-safe, etc) and padding suffix (yes/no).
//! The `Config` struct encapsulates this info. There are some common configs included: `STANDARD`,
//! `URL_SAFE`, etc. You can also make your own `Config` if needed.
//!
//! The functions that don't have `config` in the name (e.g. `encode()` and `decode()`) use the
//! `STANDARD` config .
//!
//! The functions that write to a slice (the ones that end in `_slice`) are generally the fastest
//! because they don't need to resize anything. If it fits in your workflow and you care about
//! performance, keep using the same buffer (growing as need be) and use the `_slice` methods for
//! the best performance.
//!
//! # Encoding
//!
//! Several different encoding functions are available to you depending on your desire for
//! convenience vs performance.
//!
//! | Function                | Output                       | Allocates                      |
//! | ----------------------- | ---------------------------- | ------------------------------ |
//! | `encode`                | Returns a new `String`       | Always                         |
//! | `encode_config`         | Returns a new `String`       | Always                         |
//! | `encode_config_buf`     | Appends to provided `String` | Only if `String` needs to grow |
//! | `encode_config_slice`   | Writes to provided `&[u8]`   | Never                          |
//!
//! All of the encoding functions that take a `Config` will pad as per the config.
//!
//! # Decoding
//!
//! Just as for encoding, there are different decoding functions available.
//!
//! | Function                | Output                        | Allocates                      |
//! | ----------------------- | ----------------------------- | ------------------------------ |
//! | `decode`                | Returns a new `Vec<u8>`       | Always                         |
//! | `decode_config`         | Returns a new `Vec<u8>`       | Always                         |
//! | `decode_config_buf`     | Appends to provided `Vec<u8>` | Only if `Vec` needs to grow    |
//! | `decode_config_slice`   | Writes to provided `&[u8]`    | Never                          |
//!
//! Unlike encoding, where all possible input is valid, decoding can fail (see `DecodeError`).
//!
//! Input can be invalid because it has invalid characters or invalid padding. (No padding at all is
//! valid, but excess padding is not.) Whitespace in the input is invalid.
//!
//! # `Read` and `Write`
//!
//! To map a `Read` of b64 bytes to the decoded bytes, wrap a reader (file, network socket, etc)
//! with `base64::read::DecoderReader`. To write raw bytes and have them b64 encoded on the fly,
//! wrap a writer with `base64::write::EncoderWriter`. There is some performance overhead (15% or
//! so) because of the necessary buffer shuffling -- still fast enough that almost nobody cares.
//! Also, these implementations do not heap allocate.
//!
//! # Panics
//!
//! If length calculations result in overflowing `usize`, a panic will result.
//!
//! The `_slice` flavors of encode or decode will panic if the provided output slice is too small,

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
#![cfg_attr(not(any(feature = "std", test)), no_std)]

#[cfg(all(feature = "alloc", not(any(feature = "std", test))))]
extern crate alloc;
#[cfg(any(feature = "std", test))]
extern crate std as alloc;

mod chunked_encoder;
pub mod display;
#[cfg(any(feature = "std", test))]
pub mod read;
mod tables;
#[cfg(any(feature = "std", test))]
pub mod write;

mod encode;
pub use crate::encode::encode_config_slice;
#[cfg(any(feature = "alloc", feature = "std", test))]
pub use crate::encode::{encode, encode_config, encode_config_buf};

mod decode;
#[cfg(any(feature = "alloc", feature = "std", test))]
pub use crate::decode::{decode, decode_config, decode_config_buf};
pub use crate::decode::{decode_config_slice, DecodeError};

#[cfg(test)]
mod tests;

/// Available encoding character sets
#[derive(Clone, Copy, Debug)]
pub enum CharacterSet {
    /// The standard character set (uses `+` and `/`).
    ///
    /// See [RFC 3548](https://tools.ietf.org/html/rfc3548#section-3).
    Standard,
    /// The URL safe character set (uses `-` and `_`).
    ///
    /// See [RFC 3548](https://tools.ietf.org/html/rfc3548#section-4).
    UrlSafe,
    /// The `crypt(3)` character set (uses `./0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz`).
    ///
    /// Not standardized, but folk wisdom on the net asserts that this alphabet is what crypt uses.
    Crypt,
    /// The bcrypt character set (uses `./ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789`).
    Bcrypt,
    /// The character set used in IMAP-modified UTF-7 (uses `+` and `,`).
    ///
    /// See [RFC 3501](https://tools.ietf.org/html/rfc3501#section-5.1.3)
    ImapMutf7,
    /// The character set used in BinHex 4.0 files.
    ///
    /// See [BinHex 4.0 Definition](http://files.stairways.com/other/binhex-40-specs-info.txt)
    BinHex,
}

impl CharacterSet {
    fn encode_table(self) -> &'static [u8; 64] {
        match self {
            CharacterSet::Standard => tables::STANDARD_ENCODE,
            CharacterSet::UrlSafe => tables::URL_SAFE_ENCODE,
            CharacterSet::Crypt => tables::CRYPT_ENCODE,
            CharacterSet::Bcrypt => tables::BCRYPT_ENCODE,
            CharacterSet::ImapMutf7 => tables::IMAP_MUTF7_ENCODE,
            CharacterSet::BinHex => tables::BINHEX_ENCODE,
        }
    }

    fn decode_table(self) -> &'static [u8; 256] {
        match self {
            CharacterSet::Standard => tables::STANDARD_DECODE,
            CharacterSet::UrlSafe => tables::URL_SAFE_DECODE,
            CharacterSet::Crypt => tables::CRYPT_DECODE,
            CharacterSet::Bcrypt => tables::BCRYPT_DECODE,
            CharacterSet::ImapMutf7 => tables::IMAP_MUTF7_DECODE,
            CharacterSet::BinHex => tables::BINHEX_DECODE,
        }
    }
}

/// Contains configuration parameters for base64 encoding
#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// Character set to use
    char_set: CharacterSet,
    /// True to pad output with `=` characters
    pad: bool,
    /// True to ignore excess nonzero bits in the last few symbols, otherwise an error is returned.
    decode_allow_trailing_bits: bool,
}

impl Config {
    /// Create a new `Config`.
    pub const fn new(char_set: CharacterSet, pad: bool) -> Config {
        Config {
            char_set,
            pad,
            decode_allow_trailing_bits: false,
        }
    }

    /// Sets whether to pad output with `=` characters.
    pub const fn pad(self, pad: bool) -> Config {
        Config { pad, ..self }
    }

    /// Sets whether to emit errors for nonzero trailing bits.
    ///
    /// This is useful when implementing
    /// [forgiving-base64 decode](https://infra.spec.whatwg.org/#forgiving-base64-decode).
    pub const fn decode_allow_trailing_bits(self, allow: bool) -> Config {
        Config {
            decode_allow_trailing_bits: allow,
            ..self
        }
    }
}

/// Standard character set with padding.
pub const STANDARD: Config = Config {
    char_set: CharacterSet::Standard,
    pad: true,
    decode_allow_trailing_bits: false,
};

/// Standard character set without padding.
pub const STANDARD_NO_PAD: Config = Config {
    char_set: CharacterSet::Standard,
    pad: false,
    decode_allow_trailing_bits: false,
};

/// URL-safe character set with padding
pub const URL_SAFE: Config = Config {
    char_set: CharacterSet::UrlSafe,
    pad: true,
    decode_allow_trailing_bits: false,
};

/// URL-safe character set without padding
pub const URL_SAFE_NO_PAD: Config = Config {
    char_set: CharacterSet::UrlSafe,
    pad: false,
    decode_allow_trailing_bits: false,
};

/// As per `crypt(3)` requirements
pub const CRYPT: Config = Config {
    char_set: CharacterSet::Crypt,
    pad: false,
    decode_allow_trailing_bits: false,
};

/// Bcrypt character set
pub const BCRYPT: Config = Config {
    char_set: CharacterSet::Bcrypt,
    pad: false,
    decode_allow_trailing_bits: false,
};

/// IMAP modified UTF-7 requirements
pub const IMAP_MUTF7: Config = Config {
    char_set: CharacterSet::ImapMutf7,
    pad: false,
    decode_allow_trailing_bits: false,
};

/// BinHex character set
pub const BINHEX: Config = Config {
    char_set: CharacterSet::BinHex,
    pad: false,
    decode_allow_trailing_bits: false,
};

const PAD_BYTE: u8 = b'=';
