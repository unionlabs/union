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

pub mod variable_base;

pub mod vartime_double_base;

#[cfg(feature = "alloc")]
pub mod straus;

#[cfg(feature = "alloc")]
pub mod precomputed_straus;

#[cfg(feature = "alloc")]
pub mod pippenger;
