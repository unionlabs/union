/* Copyright 2018 Mozilla Foundation
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

use crate::{BinaryReaderError, Result};
use std::ops::Range;

mod component;
mod core;

pub use self::component::*;
pub use self::core::*;

/// A trait implemented by section readers.
pub trait SectionReader {
    /// The item returned by the reader.
    type Item;

    /// Reads an item from the section.
    fn read(&mut self) -> Result<Self::Item>;

    /// Determines if the reader is at end-of-section.
    fn eof(&self) -> bool;

    /// Gets the original position of the reader.
    fn original_position(&self) -> usize;

    /// Gets the range of the reader.
    fn range(&self) -> Range<usize>;

    /// Ensures the reader is at the end of the section.
    ///
    /// This methods returns an error if there is more data in the section
    /// than what is described by the section's header.
    fn ensure_end(&self) -> Result<()> {
        if self.eof() {
            return Ok(());
        }
        Err(BinaryReaderError::new(
            "section size mismatch: unexpected data at the end of the section",
            self.original_position(),
        ))
    }
}

/// Implemented by sections with a limited number of items.
pub trait SectionWithLimitedItems: SectionReader {
    /// Gets the count of the items in the section.
    fn get_count(&self) -> u32;

    /// Returns an iterator over the items within this section where the offset
    /// in the original section is provided with the item.
    fn into_iter_with_offsets(self) -> IntoIterWithOffsets<Self>
    where
        Self: Sized,
    {
        IntoIterWithOffsets {
            iter: SectionIteratorLimited::new(self),
        }
    }
}

/// An iterator over items in a section.
pub struct SectionIterator<R>
where
    R: SectionReader,
{
    reader: R,
    err: bool,
}

impl<R> SectionIterator<R>
where
    R: SectionReader,
{
    /// Constructs a new `SectionIterator` for the given section reader.
    pub fn new(reader: R) -> SectionIterator<R> {
        SectionIterator { reader, err: false }
    }
}

impl<R> Iterator for SectionIterator<R>
where
    R: SectionReader,
{
    type Item = Result<R::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.err || self.reader.eof() {
            return None;
        }
        let result = self.reader.read();
        self.err = result.is_err();
        Some(result)
    }
}

/// An iterator over a limited section iterator.
pub struct SectionIteratorLimited<R> {
    reader: R,
    left: u32,
    end: bool,
}

impl<R> SectionIteratorLimited<R>
where
    R: SectionWithLimitedItems,
{
    /// Constructs a new `SectionIteratorLimited` for the given limited section reader.
    pub fn new(reader: R) -> SectionIteratorLimited<R> {
        let left = reader.get_count();
        SectionIteratorLimited {
            reader,
            left,
            end: false,
        }
    }
}

impl<R> Iterator for SectionIteratorLimited<R>
where
    R: SectionWithLimitedItems,
{
    type Item = Result<R::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }
        if self.left == 0 {
            return match self.reader.ensure_end() {
                Ok(()) => None,
                Err(err) => {
                    self.end = true;
                    Some(Err(err))
                }
            };
        }
        let result = self.reader.read();
        self.end = result.is_err();
        self.left -= 1;
        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.reader.get_count() as usize;
        (count, Some(count))
    }
}

/// An iterator over a limited section iterator.
pub struct IntoIterWithOffsets<R> {
    iter: SectionIteratorLimited<R>,
}

impl<R> Iterator for IntoIterWithOffsets<R>
where
    R: SectionWithLimitedItems,
{
    type Item = Result<(usize, R::Item)>;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.iter.reader.original_position();
        Some(self.iter.next()?.map(|item| (pos, item)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
