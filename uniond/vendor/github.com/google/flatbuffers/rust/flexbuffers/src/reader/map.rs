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

use super::{deref_offset, unpack_type, Error, Reader, ReaderIterator, VectorReader};
use crate::BitWidth;
use crate::Buffer;
use std::cmp::Ordering;
use std::iter::{DoubleEndedIterator, ExactSizeIterator, FusedIterator, Iterator};

/// Allows indexing on a flexbuffer map.
///
/// MapReaders may be indexed with strings or usizes. `index` returns a result type,
/// which may indicate failure due to a missing key or bad data, `idx` returns an Null Reader in
/// cases of error.
pub struct MapReader<B> {
    pub(super) buffer: B,
    pub(super) values_address: usize,
    pub(super) keys_address: usize,
    pub(super) values_width: BitWidth,
    pub(super) keys_width: BitWidth,
    pub(super) length: usize,
}

impl<B: Buffer> Clone for MapReader<B> {
    fn clone(&self) -> Self {
        MapReader {
            buffer: self.buffer.shallow_copy(),
            ..*self
        }
    }
}

impl<B: Buffer> Default for MapReader<B> {
    fn default() -> Self {
        MapReader {
            buffer: B::empty(),
            values_address: usize::default(),
            keys_address: usize::default(),
            values_width: BitWidth::default(),
            keys_width: BitWidth::default(),
            length: usize::default(),
        }
    }
}

// manual implementation of Debug because buffer slice can't be automatically displayed
impl<B: Buffer> std::fmt::Debug for MapReader<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // skips buffer field
        f.debug_struct("MapReader")
            .field("values_address", &self.values_address)
            .field("keys_address", &self.keys_address)
            .field("values_width", &self.values_width)
            .field("keys_width", &self.keys_width)
            .field("length", &self.length)
            .finish()
    }
}

impl<B: Buffer> MapReader<B> {
    /// Returns the number of key/value pairs are in the map.
    pub fn len(&self) -> usize {
        self.length
    }

    /// Returns true if the map has zero key/value pairs.
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    // Using &CStr will eagerly compute the length of the key. &str needs length info AND utf8
    // validation. This version is faster than both.
    fn lazy_strcmp(&self, key_addr: usize, key: &str) -> Ordering {
        // TODO: Can we know this won't OOB and panic?
        let k = self.buffer[key_addr..].iter().take_while(|&&b| b != b'\0');
        k.cmp(key.as_bytes().iter())
    }

    /// Returns the index of a given key in the map.
    pub fn index_key(&self, key: &str) -> Option<usize> {
        let (mut low, mut high) = (0, self.length);
        while low < high {
            let i = (low + high) / 2;
            let key_offset_address = self.keys_address + i * self.keys_width.n_bytes();
            let key_address =
                deref_offset(&self.buffer, key_offset_address, self.keys_width).ok()?;
            match self.lazy_strcmp(key_address, key) {
                Ordering::Equal => return Some(i),
                Ordering::Less => low = if i == low { i + 1 } else { i },
                Ordering::Greater => high = i,
            }
        }
        None
    }

    /// Index into a map with a key or usize.
    pub fn index<I: MapReaderIndexer>(&self, i: I) -> Result<Reader<B>, Error> {
        i.index_map_reader(self)
    }

    /// Index into a map with a key or usize. If any errors occur a Null reader is returned.
    pub fn idx<I: MapReaderIndexer>(&self, i: I) -> Reader<B> {
        i.index_map_reader(self).unwrap_or_default()
    }

    fn usize_index(&self, i: usize) -> Result<Reader<B>, Error> {
        if i >= self.length {
            return Err(Error::IndexOutOfBounds);
        }
        let data_address = self.values_address + self.values_width.n_bytes() * i;
        let type_address = self.values_address + self.values_width.n_bytes() * self.length + i;
        let (fxb_type, width) = self
            .buffer
            .get(type_address)
            .ok_or(Error::FlexbufferOutOfBounds)
            .and_then(|&b| unpack_type(b))?;
        Reader::new(
            self.buffer.shallow_copy(),
            data_address,
            fxb_type,
            width,
            self.values_width,
        )
    }

    fn key_index(&self, k: &str) -> Result<Reader<B>, Error> {
        let i = self.index_key(k).ok_or(Error::KeyNotFound)?;
        self.usize_index(i)
    }

    /// Iterate over the values of the map.
    pub fn iter_values(&self) -> ReaderIterator<B> {
        ReaderIterator::new(VectorReader {
            reader: Reader {
                buffer: self.buffer.shallow_copy(),
                fxb_type: crate::FlexBufferType::Map,
                width: self.values_width,
                address: self.values_address,
            },
            length: self.length,
        })
    }

    /// Iterate over the keys of the map.
    pub fn iter_keys(
        &self,
    ) -> impl Iterator<Item = B::BufferString> + DoubleEndedIterator + ExactSizeIterator + FusedIterator
    {
        self.keys_vector().iter().map(|k| k.as_str())
    }

    pub fn keys_vector(&self) -> VectorReader<B> {
        VectorReader {
            reader: Reader {
                buffer: self.buffer.shallow_copy(),
                fxb_type: crate::FlexBufferType::VectorKey,
                width: self.keys_width,
                address: self.keys_address,
            },
            length: self.length,
        }
    }
}

pub trait MapReaderIndexer {
    fn index_map_reader<B: Buffer>(self, r: &MapReader<B>) -> Result<Reader<B>, Error>;
}

impl MapReaderIndexer for usize {
    #[inline]
    fn index_map_reader<B: Buffer>(self, r: &MapReader<B>) -> Result<Reader<B>, Error> {
        r.usize_index(self)
    }
}

impl MapReaderIndexer for &str {
    #[inline]
    fn index_map_reader<B: Buffer>(self, r: &MapReader<B>) -> Result<Reader<B>, Error> {
        r.key_index(self)
    }
}
