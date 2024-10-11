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

use crate::{Buffer, Reader, VectorReader};
use std::iter::{DoubleEndedIterator, ExactSizeIterator, FusedIterator, Iterator};

/// Iterates over a flexbuffer vector, typed vector, or map. Yields [Readers](struct.Reader.html).
///
/// If any error occurs, the Reader is defaulted to a Null flexbuffer Reader.
pub struct ReaderIterator<B> {
    pub(super) reader: VectorReader<B>,
    pub(super) front: usize,
    end: usize,
}

impl<B: Buffer> ReaderIterator<B> {
    pub(super) fn new(reader: VectorReader<B>) -> Self {
        let end = reader.len();
        ReaderIterator {
            reader,
            front: 0,
            end,
        }
    }
}

impl<B: Buffer> Iterator for ReaderIterator<B> {
    type Item = Reader<B>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.front < self.end {
            let r = self.reader.idx(self.front);
            self.front += 1;
            Some(r)
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.end - self.front;
        (remaining, Some(remaining))
    }
}

impl<B: Buffer> DoubleEndedIterator for ReaderIterator<B> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front < self.end {
            self.end -= 1;
            Some(self.reader.idx(self.end))
        } else {
            None
        }
    }
}

impl<B: Buffer> ExactSizeIterator for ReaderIterator<B> {}
impl<B: Buffer> FusedIterator for ReaderIterator<B> {}
