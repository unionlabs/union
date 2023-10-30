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

// Conditionally include the notes if we're on nightly (so we can include docs at all).
#![cfg_attr(
    feature = "nightly",
    doc(include = "../../../docs/parallel-formulas.md")
)]

#[cfg(not(any(target_feature = "avx2", target_feature = "avx512ifma", rustdoc)))]
compile_error!("simd_backend selected without target_feature=+avx2 or +avx512ifma");

#[cfg(any(
    all(target_feature = "avx2", not(target_feature = "avx512ifma")),
    rustdoc
))]
#[doc(cfg(all(target_feature = "avx2", not(target_feature = "avx512ifma"))))]
pub mod avx2;
#[cfg(any(
    all(target_feature = "avx2", not(target_feature = "avx512ifma")),
    rustdoc
))]
pub(crate) use self::avx2::{
    constants::BASEPOINT_ODD_LOOKUP_TABLE, edwards::CachedPoint, edwards::ExtendedPoint,
};

#[cfg(any(target_feature = "avx512ifma", rustdoc))]
#[doc(cfg(target_feature = "avx512ifma"))]
pub mod ifma;
#[cfg(target_feature = "avx512ifma")]
pub(crate) use self::ifma::{
    constants::BASEPOINT_ODD_LOOKUP_TABLE, edwards::CachedPoint, edwards::ExtendedPoint,
};

pub mod scalar_mul;
