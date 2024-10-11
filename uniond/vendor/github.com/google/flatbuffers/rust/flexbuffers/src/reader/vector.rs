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

use super::{unpack_type, Error, Reader, ReaderIterator};
use crate::{BitWidth, Buffer, FlexBufferType};

/// Allows indexing on any flexbuffer vector type, (heterogenous vector, typed vector, or fixed
/// length typed vector).
///
/// VectorReaders may be indexed with usize, `index` returns a result type
/// which may indicate failure due to indexing out of bounds or bad data. `idx` returns a
/// Null Reader in the event of any failure.
pub struct VectorReader<B> {
    pub(super) reader: Reader<B>,
    // Cache the length because read_usize can be slow.
    pub(super) length: usize,
}

impl<B: Buffer> Clone for VectorReader<B> {
    fn clone(&self) -> Self {
        VectorReader {
            reader: self.reader.clone(),
            ..*self
        }
    }
}

impl<B: Buffer> Default for VectorReader<B> {
    fn default() -> Self {
        VectorReader {
            reader: Reader::default(),
            length: usize::default(),
        }
    }
}

impl<B: Buffer> VectorReader<B> {
    /// Returns the number of elements in the vector.
    pub fn len(&self) -> usize {
        self.length
    }
    /// Returns true if there are 0 elements in the vector.
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
    fn get_elem_type(&self, i: usize) -> Result<(FlexBufferType, BitWidth), Error> {
        if let Some(ty) = self.reader.fxb_type.typed_vector_type() {
            Ok((ty, self.reader.width))
        } else {
            let types_addr = self.reader.address + self.length * self.reader.width.n_bytes();
            self.reader
                .buffer
                .get(types_addr + i)
                .ok_or(Error::FlexbufferOutOfBounds)
                .and_then(|&t| unpack_type(t))
        }
    }
    /// Index into a flexbuffer vector. Any errors are defaulted to Null Readers.
    pub fn idx(&self, i: usize) -> Reader<B> {
        self.index(i).unwrap_or_default()
    }
    /// Index into a flexbuffer.
    pub fn index(&self, i: usize) -> Result<Reader<B>, Error> {
        if i >= self.length {
            return Err(Error::IndexOutOfBounds);
        }
        let (fxb_type, bw) = self.get_elem_type(i)?;
        let data_address = self.reader.address + self.reader.width.n_bytes() * i;
        Reader::new(
            self.reader.buffer.shallow_copy(),
            data_address,
            fxb_type,
            bw,
            self.reader.width,
        )
    }

    pub fn iter(&self) -> ReaderIterator<B> {
        ReaderIterator::new(self.clone())
    }
}
