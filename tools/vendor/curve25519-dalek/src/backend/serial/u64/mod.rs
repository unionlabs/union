// -*- mode: rust; -*-
//
// This file is part of curve25519-dalek.
// Copyright (c) 2016-2021 isis lovecruft
// Copyright (c) 2016-2018 Henry de Valence
// See LICENSE for licensing information.
//
// Authors:
// - isis agora lovecruft <isis@patternsinthevoid.net>
// - Henry de Valence <hdevalence@hdevalence.ca>

//! The `u64` backend uses `u64`s and a `(u64, u64) -> u128` multiplier.
//!
//! On x86_64, the idiom `(x as u128) * (y as u128)` lowers to `MUL`
//! instructions taking 64-bit inputs and producing 128-bit outputs.  On
//! other platforms, this implementation is not recommended.
//!
//! On Haswell and newer, the BMI2 extension provides `MULX`, and on
//! Broadwell and newer, the ADX extension provides `ADCX` and `ADOX`
//! (allowing the CPU to compute two carry chains in parallel).  These
//! will be used if available.

pub mod field;

pub mod scalar;

pub mod constants;
