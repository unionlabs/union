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

//! Pluggable implementations for different architectures.
//!
//! The backend code is split into two parts: a serial backend,
//! and a vector backend.
//!
//! The [`serial`] backend contains 32- and 64-bit implementations of
//! field arithmetic and scalar arithmetic, as well as implementations
//! of point operations using the mixed-model strategy (passing
//! between different curve models depending on the operation).
//!
//! The [`vector`] backend contains implementations of vectorized
//! field arithmetic, used to implement point operations using a novel
//! implementation strategy derived from parallel formulas of Hisil,
//! Wong, Carter, and Dawson.
//!
//! Because the two strategies give rise to different curve models,
//! it's not possible to reuse exactly the same scalar multiplication
//! code (or to write it generically), so both serial and vector
//! backends contain matching implementations of scalar multiplication
//! algorithms.  These are intended to be selected by a `#[cfg]`-based
//! type alias.
//!
//! The [`vector`] backend is selected by the `simd_backend` cargo
//! feature; it uses the [`serial`] backend for non-vectorized operations.

#[cfg(not(any(
    feature = "u32_backend",
    feature = "u64_backend",
    feature = "fiat_u32_backend",
    feature = "fiat_u64_backend",
    feature = "simd_backend",
)))]
compile_error!(
    "no curve25519-dalek backend cargo feature enabled! \
     please enable one of: u32_backend, u64_backend, fiat_u32_backend, fiat_u64_backend, simd_backend"
);

pub mod serial;

#[cfg(any(
    all(
        feature = "simd_backend",
        any(target_feature = "avx2", target_feature = "avx512ifma")
    ),
    all(feature = "nightly", rustdoc)
))]
#[cfg_attr(
    feature = "nightly",
    doc(cfg(any(all(
        feature = "simd_backend",
        any(target_feature = "avx2", target_feature = "avx512ifma")
    ))))
)]
pub mod vector;
