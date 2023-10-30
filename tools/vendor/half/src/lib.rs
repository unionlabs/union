//! A crate that provides support for half-precision 16-bit floating point types.
//!
//! This crate provides the [`f16`] type, which is an implementation of the IEEE 754-2008 standard
//! [`binary16`] a.k.a `half` floating point type. This 16-bit floating point type is intended for
//! efficient storage where the full range and precision of a larger floating point value is not
//! required. This is especially useful for image storage formats.
//!
//! This crate also provides a [`bf16`] type, an alternative 16-bit floating point format. The
//! [`bfloat16`] format is a truncated IEEE 754 standard `binary32` float that preserves the
//! exponent to allow the same range as [`f32`] but with only 8 bits of precision (instead of 11
//! bits for [`f16`]). See the [`bf16`] type for details.
//!
//! Because [`f16`] and [`bf16`] are primarily for efficient storage, floating point operations such
//! as addition, multiplication, etc. are not implemented by hardware. While this crate does provide
//! the appropriate trait implementations for basic operations, they each convert the value to
//! [`f32`] before performing the operation and then back afterward. When performing complex
//! arithmetic, manually convert to and from [`f32`] before and after to reduce repeated conversions
//! for each operation.
//!
//! This crate also provides a [`slice`][mod@slice] module for zero-copy in-place conversions of
//! [`u16`] slices to both [`f16`] and [`bf16`], as well as efficient vectorized conversions of
//! larger buffers of floating point values to and from these half formats.
//!
//! The crate uses `#[no_std]` by default, so can be used in embedded environments without using the
//! Rust [`std`] library. A `std` feature to enable support for the standard library is available,
//! see the [Cargo Features](#cargo-features) section below.
//!
//! A [`prelude`] module is provided for easy importing of available utility traits.
//!
//! # Cargo Features
//!
//! This crate supports a number of optional cargo features. None of these features are enabled by
//! default, even `std`.
//!
//! - **`use-intrinsics`** -- Use [`core::arch`] hardware intrinsics for `f16` and `bf16` conversions
//!   if available on the compiler target. This feature currently only works on nightly Rust
//!   until the corresponding intrinsics are stabilized.
//!
//!   When this feature is enabled and the hardware supports it, the functions and traits in the
//!   [`slice`][mod@slice] module will use vectorized SIMD intructions for increased efficiency.
//!
//!   By default, without this feature, conversions are done only in software, which will also be
//!   the fallback if the target does not have hardware support. Note that without the `std`
//!   feature enabled, no runtime CPU feature detection is used, so the hardware support is only
//!   compiled if the compiler target supports the CPU feature.
//!
//! - **`alloc`** -- Enable use of the [`alloc`] crate when not using the `std` library.
//!
//!   Among other functions, this enables the [`vec`] module, which contains zero-copy
//!   conversions for the [`Vec`] type. This allows fast conversion between raw `Vec<u16>` bits and
//!   `Vec<f16>` or `Vec<bf16>` arrays, and vice versa.
//!
//! - **`std`** -- Enable features that depend on the Rust [`std`] library. This also enables the
//!   `alloc` feature automatically.
//!
//!   Enabling the `std` feature also enables runtime CPU feature detection when the
//!   `use-intrsincis` feature is also enabled. Without this feature detection, intrinsics are only
//!   used when compiler target supports the target feature.
//!
//! - **`serde`** -- Adds support for the [`serde`] crate by implementing [`Serialize`] and
//!   [`Deserialize`] traits for both [`f16`] and [`bf16`].
//!
//! - **`num-traits`** -- Adds support for the [`num-traits`] crate by implementing [`ToPrimitive`],
//!   [`FromPrimitive`], [`AsPrimitive`], [`Num`], [`Float`], [`FloatCore`], and [`Bounded`] traits
//!   for both [`f16`] and [`bf16`].
//!
//! - **`bytemuck`** -- Adds support for the [`bytemuck`] crate by implementing [`Zeroable`] and
//!   [`Pod`] traits for both [`f16`] and [`bf16`].
//!
//! - **`zerocopy`** -- Adds support for the [`zerocopy`] crate by implementing [`AsBytes`] and
//!   [`FromBytes`] traits for both [`f16`] and [`bf16`].
//!
//! [`alloc`]: https://doc.rust-lang.org/alloc/
//! [`std`]: https://doc.rust-lang.org/std/
//! [`binary16`]: https://en.wikipedia.org/wiki/Half-precision_floating-point_format
//! [`bfloat16`]: https://en.wikipedia.org/wiki/Bfloat16_floating-point_format
//! [`serde`]: https://crates.io/crates/serde
//! [`bytemuck`]: https://crates.io/crates/bytemuck
//! [`num-traits`]: https://crates.io/crates/num-traits
//! [`zerocopy`]: https://crates.io/crates/zerocopy
#![cfg_attr(
    feature = "alloc",
    doc = "
[`vec`]: mod@vec"
)]
#![cfg_attr(
    not(feature = "alloc"),
    doc = "
[`vec`]: #
[`Vec`]: https://docs.rust-lang.org/stable/alloc/vec/struct.Vec.html"
)]
#![cfg_attr(
    feature = "serde",
    doc = "
[`Serialize`]: serde::Serialize
[`Deserialize`]: serde::Deserialize"
)]
#![cfg_attr(
    not(feature = "serde"),
    doc = "
[`Serialize`]: https://docs.rs/serde/*/serde/trait.Serialize.html
[`Deserialize`]: https://docs.rs/serde/*/serde/trait.Deserialize.html"
)]
#![cfg_attr(
    feature = "num-traits",
    doc = "
[`ToPrimitive`]: ::num_traits::ToPrimitive
[`FromPrimitive`]: ::num_traits::FromPrimitive
[`AsPrimitive`]: ::num_traits::AsPrimitive
[`Num`]: ::num_traits::Num
[`Float`]: ::num_traits::Float
[`FloatCore`]: ::num_traits::float::FloatCore
[`Bounded`]: ::num_traits::Bounded"
)]
#![cfg_attr(
    not(feature = "num-traits"),
    doc = "
[`ToPrimitive`]: https://docs.rs/num-traits/*/num_traits/cast/trait.ToPrimitive.html
[`FromPrimitive`]: https://docs.rs/num-traits/*/num_traits/cast/trait.FromPrimitive.html
[`AsPrimitive`]: https://docs.rs/num-traits/*/num_traits/cast/trait.AsPrimitive.html
[`Num`]: https://docs.rs/num-traits/*/num_traits/trait.Num.html
[`Float`]: https://docs.rs/num-traits/*/num_traits/float/trait.Float.html
[`FloatCore`]: https://docs.rs/num-traits/*/num_traits/float/trait.FloatCore.html
[`Bounded`]: https://docs.rs/num-traits/*/num_traits/bounds/trait.Bounded.html"
)]
#![cfg_attr(
    feature = "bytemuck",
    doc = "
[`Zeroable`]: bytemuck::Zeroable
[`Pod`]: bytemuck::Pod"
)]
#![cfg_attr(
    not(feature = "bytemuck"),
    doc = "
[`Zeroable`]: https://docs.rs/bytemuck/*/bytemuck/trait.Zeroable.html
[`Pod`]: https://docs.rs/bytemuck/*bytemuck/trait.Pod.html"
)]
#![cfg_attr(
    feature = "zerocopy",
    doc = "
[`AsBytes`]: zerocopy::AsBytes
[`FromBytes`]: zerocopy::FromBytes"
)]
#![cfg_attr(
    not(feature = "zerocopy"),
    doc = "
[`AsBytes`]: https://docs.rs/zerocopy/*/zerocopy/trait.AsBytes.html
[`FromBytes`]: https://docs.rs/zerocopy/*/zerocopy/trait.FromBytes.html"
)]
#![warn(
    missing_docs,
    missing_copy_implementations,
    missing_debug_implementations,
    trivial_numeric_casts,
    future_incompatible
)]
#![allow(clippy::verbose_bit_mask, clippy::cast_lossless)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(
    all(
        feature = "use-intrinsics",
        any(target_arch = "x86", target_arch = "x86_64")
    ),
    feature(stdsimd, f16c_target_feature)
)]
#![doc(html_root_url = "https://docs.rs/half/1.8.2")]
#![doc(test(attr(deny(warnings), allow(unused))))]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

mod bfloat;
mod binary16;
#[cfg(feature = "num-traits")]
mod num_traits;

pub mod slice;
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub mod vec;

pub use bfloat::bf16;
#[doc(hidden)]
#[allow(deprecated)]
pub use binary16::consts;
pub use binary16::f16;

/// A collection of the most used items and traits in this crate for easy importing.
///
/// # Examples
///
/// ```rust
/// use half::prelude::*;
/// ```
pub mod prelude {
    #[doc(no_inline)]
    pub use crate::{
        bf16, f16,
        slice::{HalfBitsSliceExt, HalfFloatSliceExt},
    };

    #[cfg(feature = "alloc")]
    #[doc(no_inline)]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub use crate::vec::{HalfBitsVecExt, HalfFloatVecExt};
}

// Keep this module private to crate
mod private {
    use crate::{bf16, f16};

    pub trait SealedHalf {}

    impl SealedHalf for f16 {}
    impl SealedHalf for bf16 {}
}
