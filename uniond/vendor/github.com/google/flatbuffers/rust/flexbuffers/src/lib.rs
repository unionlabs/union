// Copyright 2019 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Flexbuffers is a high performance schemaless binary data format designed at Google.
//! It is complementary to the schema-ed format [Flatbuffers](http://docs.rs/flatbuffers/).
//! See [Flexbuffer Internals](https://google.github.io/flatbuffers/flatbuffers_internals.html)
//! for details on the binary format.
//!
//! See the examples for usage:
//! * [Example](https://github.com/google/flatbuffers/blob/master/samples/sample_flexbuffers.rs)
//! * [Serde Example](https://github.com/google/flatbuffers/blob/master/samples/sample_flexbuffers_serde.rs)
//!
//! This rust implementation is in progress and, until the 1.0 release, breaking API changes may
//! happen between minor versions.
// TODO(cneo): serde stuff are behind a default-on feature flag
//             Reader to Json is behind a default-off feature flag
//             Serializable structs are Pushable
//             Serde with maps - field names and type names.

// Until flat/flexbuffers is on Rust v1.42, we cannot use the previously unstable matches! macro.
#![allow(unknown_lints)]
#![allow(clippy::match_like_matches_macro)]

#[macro_use]
extern crate bitflags;
extern crate byteorder;
#[macro_use]
extern crate serde_derive;
extern crate num_enum;
extern crate serde;

mod bitwidth;
mod buffer;
mod builder;
mod flexbuffer_type;
mod reader;

pub use bitwidth::BitWidth;
pub use buffer::Buffer;
pub use builder::Error as SerializationError;
pub use builder::{
    singleton, Builder, BuilderOptions, FlexbufferSerializer, MapBuilder, Pushable, VectorBuilder,
};
pub use flexbuffer_type::FlexBufferType;
pub use reader::Error as ReaderError;
pub use reader::{DeserializationError, MapReader, Reader, ReaderIterator, VectorReader};
use serde::{Deserialize, Serialize};

mod private {
    pub trait Sealed {}
}

/// Serialize as a flexbuffer into a vector.
pub fn to_vec<T: Serialize>(x: T) -> Result<Vec<u8>, SerializationError> {
    let mut s = FlexbufferSerializer::new();
    x.serialize(&mut s)?;
    Ok(s.take_buffer())
}

/// Deserialize a type from a flexbuffer.
pub fn from_slice<'de, T: Deserialize<'de>>(buf: &'de [u8]) -> Result<T, DeserializationError> {
    let r = Reader::get_root(buf)?;
    T::deserialize(r)
}

/// Deserialize a type from a flexbuffer.
pub fn from_buffer<'de, T: Deserialize<'de>, B: Buffer>(
    buf: &'de B,
) -> Result<T, DeserializationError> {
    let r = Reader::get_root(buf as &'de [u8])?;
    T::deserialize(r)
}

/// This struct, when pushed will be serialized as a `FlexBufferType::Blob`.
///
/// A `Blob` is a variable width `length` followed by that many bytes of data.
#[derive(Debug, PartialEq, Eq)]
pub struct Blob<B>(pub B);

impl<B: Buffer> Clone for Blob<B> {
    fn clone(&self) -> Self {
        Blob(self.0.shallow_copy())
    }
}

/// This struct, when pushed, will be serialized as a `FlexBufferType::IndirectUInt`.
///
/// It is an unsigned integer stored by reference in the flexbuffer. This can reduce the
/// size of vectors and maps containing the `IndirectUInt`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct IndirectUInt(pub u64);

/// This struct, when pushed, will be serialized as a `FlexBufferType::IndirectInt`.
///
/// It is a signed integer stored by reference in the flexbuffer. This can reduce the
/// size of vectors and maps containing the `IndirectInt`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct IndirectInt(pub i64);

/// This struct, when pushed, will be serialized as a `FlexBufferType::IndirectFloat`.
///
/// It is a floating point stored by reference in the flexbuffer. This can reduce the
/// size of vectors and maps containing the `IndirectFloat`.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct IndirectFloat(pub f64);
