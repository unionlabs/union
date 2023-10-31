// -*- mode: rust; -*-
//
// This file is part of curve25519-dalek.
// Copyright (c) 2016-2021 isis lovecruft
// Copyright (c) 2016-2019 Henry de Valence
// See LICENSE for licensing information.
//
// Authors:
// - isis agora lovecruft <isis@patternsinthevoid.net>
// - Henry de Valence <hdevalence@hdevalence.ca>

//! Serial implementations of field, scalar, point arithmetic.
//!
//! When the vector backend is disabled, the crate uses the
//! mixed-model strategy for implementing point operations and scalar
//! multiplication; see the [`curve_models`](self::curve_models) and
//! [`scalar_mul`](self::scalar_mul) documentation for more
//! information.
//!
//! When the vector backend is enabled, the field and scalar
//! implementations are still used for non-vectorized operations.
//!
//! Note: at this time the `u32` and `u64` backends cannot be built
//! together.

#[cfg(not(any(
    feature = "u32_backend",
    feature = "u64_backend",
    feature = "fiat_u32_backend",
    feature = "fiat_u64_backend"
)))]
compile_error!(
    "no curve25519-dalek backend cargo feature enabled! \
     please enable one of: u32_backend, u64_backend, fiat_u32_backend, fiat_u64_backend"
);

#[cfg(feature = "u32_backend")]
pub mod u32;

#[cfg(feature = "u64_backend")]
pub mod u64;

#[cfg(feature = "fiat_u32_backend")]
pub mod fiat_u32;

#[cfg(feature = "fiat_u64_backend")]
pub mod fiat_u64;

pub mod curve_models;

#[cfg(not(all(
    feature = "simd_backend",
    any(target_feature = "avx2", target_feature = "avx512ifma")
)))]
pub mod scalar_mul;
