/*
 * Copyright 2018 Google Inc. All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! # FlatBuffers
//!
//! A library for memory-efficient serialization of data.
//!
//! This crate provides runtime support for the FlatBuffers format in the Rust programming language.
//! To use this crate, first generate code with the `flatc` compiler, as described here: <https://google.github.io/flatbuffers/>
//! Then, include that code into your project.
//! Finally, add this crate to your `Cargo.toml`.
//!
//! At this time, Rust support is experimental, and APIs may change between minor versions.
//!
//! At this time, to generate Rust code, you will need the latest `master` version of `flatc`, available from here: <https://github.com/google/flatbuffers>
//! (On OSX, you can install FlatBuffers from `HEAD` with the Homebrew package manager.)

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(all(nightly, not(feature = "std")), feature(error_in_core))]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod array;
mod builder;
mod endian_scalar;
mod follow;
mod get_root;
mod primitives;
mod push;
mod table;
mod vector;
mod verifier;
mod vtable;
mod vtable_writer;

pub use crate::array::{array_init, emplace_scalar_array, Array};
pub use crate::builder::{Allocator, DefaultAllocator, FlatBufferBuilder};
pub use crate::endian_scalar::{emplace_scalar, read_scalar, read_scalar_at, EndianScalar};
pub use crate::follow::{Follow, FollowStart};
pub use crate::primitives::*;
pub use crate::push::Push;
pub use crate::table::{buffer_has_identifier, Table};
pub use crate::vector::{follow_cast_ref, Vector, VectorIter};
pub use crate::verifier::{
    ErrorTraceDetail, InvalidFlatbuffer, SimpleToVerifyInSlice, Verifiable, Verifier,
    VerifierOptions,
};
pub use crate::vtable::field_index_to_field_offset;
pub use bitflags;
pub use get_root::*;

// TODO(rw): Split fill ops in builder into fill_small, fill_big like in C++.
