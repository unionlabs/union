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

use super::{BinaryReaderError, Range, Result};

pub trait SectionReader {
    type Item;
    fn read(&mut self) -> Result<Self::Item>;
    fn eof(&self) -> bool;
    fn original_position(&self) -> usize;
    fn range(&self) -> Range;
    fn ensure_end(&self) -> Result<()> {
        if self.eof() {
            return Ok(());
        }
        Err(BinaryReaderError::new(
            "Unexpected data at the end of the section",
            self.original_position(),
        ))
    }
}

pub trait SectionWithLimitedItems {
    fn get_count(&self) -> u32;
}

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

pub struct SectionIteratorLimited<R>
where
    R: SectionReader + SectionWithLimitedItems,
{
    reader: R,
    left: u32,
    end: bool,
}

impl<R> SectionIteratorLimited<R>
where
    R: SectionReader + SectionWithLimitedItems,
{
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
    R: SectionReader + SectionWithLimitedItems,
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
