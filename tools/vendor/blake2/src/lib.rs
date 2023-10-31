//! An implementation of the [BLAKE2][1] hash functions.
//!
//! # Usage
//!
//! [`Blake2b512`] and [`Blake2s256`] can be used in the following way:
//!
//! ```rust
//! use blake2::{Blake2b512, Blake2s256, Digest};
//! use hex_literal::hex;
//!
//! // create a Blake2b512 object
//! let mut hasher = Blake2b512::new();
//!
//! // write input message
//! hasher.update(b"hello world");
//!
//! // read hash digest and consume hasher
//! let res = hasher.finalize();
//! assert_eq!(res[..], hex!("
//!     021ced8799296ceca557832ab941a50b4a11f83478cf141f51f933f653ab9fbc
//!     c05a037cddbed06e309bf334942c4e58cdf1a46e237911ccd7fcf9787cbc7fd0
//! ")[..]);
//!
//! // same example for Blake2s256:
//! let mut hasher = Blake2s256::new();
//! hasher.update(b"hello world");
//! let res = hasher.finalize();
//! assert_eq!(res[..], hex!("
//!     9aec6806794561107e594b1f6a8a6b0c92a0cba9acf5e5e93cca06f781813b0b
//! ")[..]);
//! ```
//!
//! Also see [RustCrypto/hashes](https://github.com/RustCrypto/hashes) readme.
//!
//! ## Variable output size
//!
//! This implementation supports run and compile time variable sizes.
//!
//! Run time variable output example:
//! ```rust
//! use blake2::Blake2bVar;
//! use blake2::digest::{Update, VariableOutput};
//! use hex_literal::hex;
//!
//! let mut hasher = Blake2bVar::new(10).unwrap();
//! hasher.update(b"my_input");
//! let mut buf = [0u8; 10];
//! hasher.finalize_variable(&mut buf).unwrap();
//! assert_eq!(buf, hex!("2cc55c84e416924e6400"));
//! ```
//!
//! Compile time variable output example:
//! ```rust
//! use blake2::{Blake2b, Digest, digest::consts::U10};
//! use hex_literal::hex;
//!
//! type Blake2b80 = Blake2b<U10>;
//!
//! let mut hasher = Blake2b80::new();
//! hasher.update(b"my_input");
//! let res = hasher.finalize();
//! assert_eq!(res[..], hex!("2cc55c84e416924e6400")[..]);
//! ```
//!
//! # Acknowledgment
//! Based on the [blake2-rfc][2] crate.
//!
//! [1]: https://en.wikipedia.org/wiki/BLAKE_(hash_function)#BLAKE2
//! [2]: https://github.com/cesarb/blake2-rfc

#![no_std]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg"
)]
#![warn(missing_docs, rust_2018_idioms)]
#![cfg_attr(feature = "simd", feature(platform_intrinsics, repr_simd))]
#![cfg_attr(feature = "simd", allow(incomplete_features))]

#[cfg(feature = "std")]
extern crate std;

pub use digest::{self, Digest};

use core::{convert::TryInto, fmt, marker::PhantomData, ops::Div};
use digest::{
    block_buffer::{Lazy, LazyBuffer},
    consts::{U128, U32, U4, U64},
    core_api::{
        AlgorithmName, Block, BlockSizeUser, Buffer, BufferKindUser, CoreWrapper,
        CtVariableCoreWrapper, OutputSizeUser, RtVariableCoreWrapper, TruncSide, UpdateCore,
        VariableOutputCore,
    },
    crypto_common::{InvalidLength, Key, KeyInit, KeySizeUser},
    generic_array::{ArrayLength, GenericArray},
    typenum::{IsLessOrEqual, LeEq, NonZero, Unsigned},
    FixedOutput, HashMarker, InvalidOutputSize, MacMarker, Output, Update,
};
#[cfg(feature = "reset")]
use digest::{FixedOutputReset, Reset};

mod as_bytes;
mod consts;

mod simd;

#[macro_use]
mod macros;

use as_bytes::AsBytes;
use consts::{BLAKE2B_IV, BLAKE2S_IV};
use simd::{u32x4, u64x4, Vector4};

blake2_impl!(
    Blake2bVarCore,
    "Blake2b",
    u64,
    u64x4,
    U64,
    U128,
    32,
    24,
    16,
    63,
    BLAKE2B_IV,
    "Blake2b instance with a variable output.",
    "Blake2b instance with a fixed output.",
);

/// BLAKE2b which allows to choose output size at runtime.
pub type Blake2bVar = RtVariableCoreWrapper<Blake2bVarCore>;
/// Core hasher state of BLAKE2b generic over output size.
pub type Blake2bCore<OutSize> = CtVariableCoreWrapper<Blake2bVarCore, OutSize>;
/// BLAKE2b generic over output size.
pub type Blake2b<OutSize> = CoreWrapper<Blake2bCore<OutSize>>;
/// BLAKE2b-512 hasher state.
pub type Blake2b512 = Blake2b<U64>;

blake2_mac_impl!(Blake2bMac, Blake2bVarCore, U64, "Blake2b MAC function");

/// BLAKE2b-512 MAC state.
pub type Blake2bMac512 = Blake2bMac<U64>;

blake2_impl!(
    Blake2sVarCore,
    "Blake2s",
    u32,
    u32x4,
    U32,
    U64,
    16,
    12,
    8,
    7,
    BLAKE2S_IV,
    "Blake2s instance with a variable output.",
    "Blake2s instance with a fixed output.",
);

/// BLAKE2s which allows to choose output size at runtime.
pub type Blake2sVar = RtVariableCoreWrapper<Blake2sVarCore>;
/// Core hasher state of BLAKE2s generic over output size.
pub type Blake2sCore<OutSize> = CtVariableCoreWrapper<Blake2sVarCore, OutSize>;
/// BLAKE2s generic over output size.
pub type Blake2s<OutSize> = CoreWrapper<Blake2sCore<OutSize>>;
/// BLAKE2s-256 hasher state.
pub type Blake2s256 = Blake2s<U32>;

blake2_mac_impl!(Blake2sMac, Blake2sVarCore, U32, "Blake2s MAC function");

/// BLAKE2s-256 MAC state.
pub type Blake2sMac256 = Blake2sMac<U32>;
