#![no_std]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/8f1a9894/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/8f1a9894/logo.svg"
)]
#![forbid(unsafe_code)]
#![warn(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::checked_conversions,
    clippy::implicit_saturating_sub,
    clippy::integer_arithmetic,
    clippy::mod_module_files,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_used,
    missing_docs,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]

//! ## Usage
//!
//! This crate provides traits for describing elliptic curves, along with
//! types which are generic over elliptic curves which can be used as the
//! basis of curve-agnostic code.
//!
//! It's intended to be used with the following concrete elliptic curve
//! implementations from the [`RustCrypto/elliptic-curves`] project:
//!
//! - [`bp256`]: brainpoolP256r1 and brainpoolP256t1
//! - [`bp384`]: brainpoolP384r1 and brainpoolP384t1
//! - [`k256`]: secp256k1 a.k.a. K-256
//! - [`p224`]: NIST P-224 a.k.a. secp224r1
//! - [`p256`]: NIST P-256 a.k.a secp256r1, prime256v1
//! - [`p384`]: NIST P-384 a.k.a. secp384r1
//! - [`p521`]: NIST P-521 a.k.a. secp521r1
//!
//! The [`ecdsa`] crate provides a generic implementation of the
//! Elliptic Curve Digital Signature Algorithm which can be used with any of
//! the above crates, either via an external ECDSA implementation, or
//! using native curve arithmetic where applicable.
//!
//! ## Type conversions
//!
//! The following chart illustrates the various conversions possible between
//! the various types defined by this crate.
//!
//! ![Type Conversion Map](https://raw.githubusercontent.com/RustCrypto/media/master/img/elliptic-curve/type-transforms.svg)
//!
//! ## `serde` support
//!
//! When the `serde` feature of this crate is enabled, `Serialize` and
//! `Deserialize` impls are provided for the following types:
//!
//! - [`JwkEcKey`]
//! - [`PublicKey`]
//! - [`ScalarPrimitive`]
//!
//! Please see type-specific documentation for more information.
//!
//! [`RustCrypto/elliptic-curves`]: https://github.com/RustCrypto/elliptic-curves
//! [`bp256`]: https://github.com/RustCrypto/elliptic-curves/tree/master/bp256
//! [`bp384`]: https://github.com/RustCrypto/elliptic-curves/tree/master/bp384
//! [`k256`]: https://github.com/RustCrypto/elliptic-curves/tree/master/k256
//! [`p224`]: https://github.com/RustCrypto/elliptic-curves/tree/master/p224
//! [`p256`]: https://github.com/RustCrypto/elliptic-curves/tree/master/p256
//! [`p384`]: https://github.com/RustCrypto/elliptic-curves/tree/master/p384
//! [`p521`]: https://github.com/RustCrypto/elliptic-curves/tree/master/p521
//! [`ecdsa`]: https://github.com/RustCrypto/signatures/tree/master/ecdsa

#[cfg(feature = "alloc")]
#[allow(unused_imports)]
#[macro_use]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub mod point;
pub mod scalar;

#[cfg(feature = "dev")]
pub mod dev;
#[cfg(feature = "ecdh")]
pub mod ecdh;
#[cfg(feature = "hash2curve")]
pub mod hash2curve;
#[cfg(feature = "arithmetic")]
pub mod ops;
#[cfg(feature = "sec1")]
pub mod sec1;
#[cfg(feature = "arithmetic")]
pub mod weierstrass;

mod error;
mod field;
mod secret_key;

#[cfg(feature = "arithmetic")]
mod arithmetic;
#[cfg(feature = "arithmetic")]
mod public_key;

#[cfg(feature = "jwk")]
mod jwk;

#[cfg(feature = "voprf")]
mod voprf;

pub use crate::{
    error::{Error, Result},
    field::{FieldBytes, FieldBytesEncoding, FieldBytesSize},
    scalar::ScalarPrimitive,
    secret_key::SecretKey,
};
pub use crypto_bigint as bigint;
pub use generic_array::{self, typenum::consts};
pub use rand_core;
pub use subtle;
pub use zeroize;

#[cfg(feature = "arithmetic")]
pub use {
    crate::{
        arithmetic::{CurveArithmetic, PrimeCurveArithmetic},
        point::{AffinePoint, ProjectivePoint},
        public_key::PublicKey,
        scalar::{NonZeroScalar, Scalar},
    },
    ff::{self, Field, PrimeField},
    group::{self, Group},
};

#[cfg(feature = "jwk")]
pub use crate::jwk::{JwkEcKey, JwkParameters};

#[cfg(feature = "pkcs8")]
pub use pkcs8;

#[cfg(feature = "voprf")]
pub use crate::voprf::VoprfParameters;

use core::{
    fmt::Debug,
    ops::{Add, ShrAssign},
};
use generic_array::ArrayLength;

/// Algorithm [`ObjectIdentifier`][`pkcs8::ObjectIdentifier`] for elliptic
/// curve public key cryptography (`id-ecPublicKey`).
///
/// <http://oid-info.com/get/1.2.840.10045.2.1>
#[cfg(feature = "pkcs8")]
pub const ALGORITHM_OID: pkcs8::ObjectIdentifier =
    pkcs8::ObjectIdentifier::new_unwrap("1.2.840.10045.2.1");

/// Elliptic curve.
///
/// This trait is intended to be impl'd by a ZST which represents a concrete
/// elliptic curve.
///
/// Other traits in this crate which are bounded by [`Curve`] are intended to
/// be impl'd by these ZSTs, facilitating types which are generic over elliptic
/// curves (e.g. [`SecretKey`]).
pub trait Curve: 'static + Copy + Clone + Debug + Default + Eq + Ord + Send + Sync {
    /// Size of a serialized field element in bytes.
    ///
    /// This is typically the same as `Self::Uint::ByteSize` but for curves
    /// with an unusual field modulus (e.g. P-224, P-521) it may be different.
    type FieldBytesSize: ArrayLength<u8> + Add + Eq;

    /// Integer type used to represent field elements of this elliptic curve.
    type Uint: bigint::ArrayEncoding
        + bigint::AddMod<Output = Self::Uint>
        + bigint::Encoding
        + bigint::Integer
        + bigint::NegMod<Output = Self::Uint>
        + bigint::Random
        + bigint::RandomMod
        + bigint::SubMod<Output = Self::Uint>
        + zeroize::Zeroize
        + FieldBytesEncoding<Self>
        + ShrAssign<usize>;

    /// Order of this elliptic curve, i.e. number of elements in the scalar
    /// field.
    const ORDER: Self::Uint;
}

/// Marker trait for elliptic curves with prime order.
pub trait PrimeCurve: Curve {}
