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

#![cfg_attr(
    feature = "nightly",
    doc(include = "../../../../docs/avx2-notes.md")
)]

pub(crate) mod field;

pub(crate) mod edwards;

pub(crate) mod constants;
