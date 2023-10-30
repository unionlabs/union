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

//! Crate-local prelude (for alloc-dependent features like `Vec`)

// TODO: switch to alloc::prelude
#[cfg(all(feature = "alloc", not(feature = "std")))]
pub use alloc::vec::Vec;

#[cfg(feature = "std")]
pub use std::vec::Vec;
