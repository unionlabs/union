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
    BinaryReader, Range, Result, SectionIteratorLimited, SectionReader, SectionWithLimitedItems,
};

#[derive(Clone)]
pub struct FunctionSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> FunctionSectionReader<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Result<FunctionSectionReader<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(FunctionSectionReader { reader, count })
    }

    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    /// Reads function type index from the function section.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasmparser::FunctionSectionReader;
    /// # let data: &[u8] = &[0x01, 0x00];
    /// let mut function_reader = FunctionSectionReader::new(data, 0).unwrap();
    /// for _ in 0..function_reader.get_count() {
    ///     let ty = function_reader.read().expect("function type index");
    ///     println!("Function type index: {}", ty);
    /// }
    /// ```
    pub fn read(&mut self) -> Result<u32> {
        self.reader.read_var_u32()
    }
}

impl<'a> SectionReader for FunctionSectionReader<'a> {
    type Item = u32;
    fn read(&mut self) -> Result<Self::Item> {
        FunctionSectionReader::read(self)
    }
    fn eof(&self) -> bool {
        self.reader.eof()
    }
    fn original_position(&self) -> usize {
        FunctionSectionReader::original_position(self)
    }
    fn range(&self) -> Range {
        self.reader.range()
    }
}

impl<'a> SectionWithLimitedItems for FunctionSectionReader<'a> {
    fn get_count(&self) -> u32 {
        FunctionSectionReader::get_count(self)
    }
}

impl<'a> IntoIterator for FunctionSectionReader<'a> {
    type Item = Result<u32>;
    type IntoIter = SectionIteratorLimited<FunctionSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
