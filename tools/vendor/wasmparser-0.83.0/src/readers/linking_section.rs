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

use super::{
    BinaryReader, LinkingType, Range, Result, SectionIteratorLimited, SectionReader,
    SectionWithLimitedItems,
};

pub struct LinkingSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> LinkingSectionReader<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Result<LinkingSectionReader<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(LinkingSectionReader { reader, count })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    pub fn read<'b>(&mut self) -> Result<LinkingType>
    where
        'a: 'b,
    {
        self.reader.read_linking_type()
    }
}

impl<'a> SectionReader for LinkingSectionReader<'a> {
    type Item = LinkingType;
    fn read(&mut self) -> Result<Self::Item> {
        LinkingSectionReader::read(self)
    }
    fn eof(&self) -> bool {
        self.reader.eof()
    }
    fn original_position(&self) -> usize {
        LinkingSectionReader::original_position(self)
    }
    fn range(&self) -> Range {
        self.reader.range()
    }
}

impl<'a> SectionWithLimitedItems for LinkingSectionReader<'a> {
    fn get_count(&self) -> u32 {
        LinkingSectionReader::get_count(self)
    }
}

impl<'a> IntoIterator for LinkingSectionReader<'a> {
    type Item = Result<LinkingType>;
    type IntoIter = SectionIteratorLimited<LinkingSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
