/* Copyright 2020 Mozilla Foundation
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
    BinaryReader, Result, SectionIteratorLimited, SectionReader, SectionWithLimitedItems, TagType,
};
use std::ops::Range;

/// A reader for the tags section of a WebAssembly module.
#[derive(Clone)]
pub struct TagSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> TagSectionReader<'a> {
    /// Constructs a new `TagSectionReader` for the given data and offset.
    pub fn new(data: &'a [u8], offset: usize) -> Result<TagSectionReader<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(TagSectionReader { reader, count })
    }

    /// Gets the original position of the section reader.
    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    /// Gets the count of items in the section.
    pub fn get_count(&self) -> u32 {
        self.count
    }

    /// Reads content of the tag section.
    ///
    /// # Examples
    /// ```
    /// use wasmparser::TagSectionReader;
    /// # let data: &[u8] = &[0x01, 0x00, 0x01];
    /// let mut reader = TagSectionReader::new(data, 0).unwrap();
    /// for _ in 0..reader.get_count() {
    ///     let ty = reader.read().expect("tag type");
    ///     println!("Tag type: {:?}", ty);
    /// }
    /// ```
    pub fn read(&mut self) -> Result<TagType> {
        self.reader.read_tag_type()
    }
}

impl<'a> SectionReader for TagSectionReader<'a> {
    type Item = TagType;
    fn read(&mut self) -> Result<Self::Item> {
        TagSectionReader::read(self)
    }
    fn eof(&self) -> bool {
        self.reader.eof()
    }
    fn original_position(&self) -> usize {
        TagSectionReader::original_position(self)
    }
    fn range(&self) -> Range<usize> {
        self.reader.range()
    }
}

impl<'a> SectionWithLimitedItems for TagSectionReader<'a> {
    fn get_count(&self) -> u32 {
        TagSectionReader::get_count(self)
    }
}

impl<'a> IntoIterator for TagSectionReader<'a> {
    type Item = Result<TagType>;
    type IntoIter = SectionIteratorLimited<TagSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
