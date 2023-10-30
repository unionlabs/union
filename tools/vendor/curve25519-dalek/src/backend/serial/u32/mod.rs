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

//! The `u32` backend uses `u32`s and a `(u32, u32) -> u64` multiplier.
//!
//! This code is intended to be portable, but it requires that
//! multiplication of two \\(32\\)-bit values to a \\(64\\)-bit result
//! is constant-time on the target platform.

pub mod field;

pub mod scalar;

pub mod constants;
