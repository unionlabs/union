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

use crate::{
    BinaryReader, MemoryType, Result, SectionIteratorLimited, SectionReader,
    SectionWithLimitedItems,
};
use std::ops::Range;

/// A reader for the memory section of a WebAssembly module.
#[derive(Clone)]
pub struct MemorySectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> MemorySectionReader<'a> {
    /// Constructs a new `MemorySectionReader` for the given data and offset.
    pub fn new(data: &'a [u8], offset: usize) -> Result<MemorySectionReader<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(MemorySectionReader { reader, count })
    }

    /// Gets the original position of the section reader.
    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    /// Gets the count of items in the section.
    pub fn get_count(&self) -> u32 {
        self.count
    }

    /// Reads content of the memory section.
    ///
    /// # Examples
    /// ```
    /// use wasmparser::MemorySectionReader;
    /// # let data: &[u8] = &[0x01, 0x00, 0x02];
    /// let mut memory_reader = MemorySectionReader::new(data, 0).unwrap();
    /// for _ in 0..memory_reader.get_count() {
    ///     let memory = memory_reader.read().expect("memory");
    ///     println!("Memory: {:?}", memory);
    /// }
    /// ```
    pub fn read(&mut self) -> Result<MemoryType> {
        self.reader.read_memory_type()
    }
}

impl<'a> SectionReader for MemorySectionReader<'a> {
    type Item = MemoryType;
    fn read(&mut self) -> Result<Self::Item> {
        MemorySectionReader::read(self)
    }
    fn eof(&self) -> bool {
        self.reader.eof()
    }
    fn original_position(&self) -> usize {
        MemorySectionReader::original_position(self)
    }
    fn range(&self) -> Range<usize> {
        self.reader.range()
    }
}

impl<'a> SectionWithLimitedItems for MemorySectionReader<'a> {
    fn get_count(&self) -> u32 {
        MemorySectionReader::get_count(self)
    }
}

impl<'a> IntoIterator for MemorySectionReader<'a> {
    type Item = Result<MemoryType>;
    type IntoIter = SectionIteratorLimited<MemorySectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
